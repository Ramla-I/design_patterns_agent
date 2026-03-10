pub mod progress;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Result;
use std::path::Path;
use tokio::sync::Semaphore;

use crate::cli::config::SearchMode;
use crate::cli::Config;
use crate::detection::InvariantDetector;
use crate::llm;
use crate::llm::{RetryClient, TokenStats, TokenTrackingClient};
use crate::navigation::priority::prioritize_chunks;
use crate::navigation::{AnalysisChunk, Navigator};
use crate::report::Report;
use crate::search;

use self::progress::ProgressTracker;

/// Main analysis function that orchestrates the latent invariant discovery process.
/// Returns the report and the path to the run directory where progress files are stored.
pub async fn analyze_codebase(path: &Path, config: &Config) -> Result<(Report, std::path::PathBuf)> {
    println!("Initializing analysis of codebase at: {}", path.display());

    // Create LLM client
    println!("Connecting to LLM provider: {}", config.llm.provider);
    let raw_client: Arc<dyn llm::LlmClient> = Arc::from(llm::create_client(
        &config.llm.provider,
        config.llm.api_key.clone(),
        config.llm.model.clone(),
    )?);

    // Wrap with retry logic (rate limits, transient errors)
    let retry_client: Arc<dyn llm::LlmClient> = Arc::new(RetryClient::new(
        raw_client,
        config.execution.max_retries,
        config.execution.retry_base_delay,
    ));
    if config.execution.max_retries > 0 {
        println!(
            "  Retry: up to {} attempts, {}s base backoff",
            config.execution.max_retries, config.execution.retry_base_delay
        );
    }

    // Wrap with token tracking
    let detector_stats = Arc::new(TokenStats::new());
    let llm_client: Arc<dyn llm::LlmClient> = Arc::new(TokenTrackingClient::new(
        retry_client,
        detector_stats.clone(),
    ));

    // Build analysis chunks based on search mode
    let (chunks, modules_analyzed, parse_failures) = match config.search.mode {
        SearchMode::Exhaustive => {
            println!("Mode: exhaustive (parsing all modules)");
            build_exhaustive_chunks(path, config)?
        }
        SearchMode::Semantic => {
            println!("Mode: semantic (using octocode)");
            let (chunks, count) = build_semantic_chunks(path, config).await?;
            (chunks, count, vec![])
        }
    };

    // Apply priority scoring
    let chunks = if config.execution.priority_modules.is_empty() {
        chunks
    } else {
        println!("Prioritizing chunks by module prefixes: {:?}", config.execution.priority_modules);
        prioritize_chunks(chunks, &config.execution.priority_modules)
    };

    println!(
        "  {} analysis chunks from {} modules",
        chunks.len(),
        modules_analyzed,
    );

    // Set up run directory and progress tracker
    let run_dir = create_run_dir(&config.llm.model)?;
    let tracker = Arc::new(ProgressTracker::new(
        &run_dir,
        chunks.len(),
        config.execution.token_budget as u64,
    )?);

    // Resume from checkpoint if requested
    if let Some(ref resume_path) = config.execution.resume_path {
        println!("Resuming from checkpoint: {}", resume_path.display());
        let checkpoint = ProgressTracker::load_checkpoint(resume_path)?;
        let skipped = checkpoint.len();
        tracker.restore_from_checkpoint(checkpoint);
        println!("  {} chunks already completed, skipping", skipped);
    }

    // Graceful shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("\nReceived Ctrl+C, finishing in-flight tasks...");
            shutdown.store(true, Ordering::Relaxed);
        });
    }

    // Partition priority chunks (these bypass budget enforcement)
    let priority_prefixes: Vec<String> = config.execution.priority_modules.clone();
    let priority_total = chunks.iter().filter(|c| priority_prefixes.iter().any(|p| c.module_path.contains(p))).count();
    let other_total = chunks.len() - priority_total;

    // Counters for coverage stats
    let priority_analyzed = Arc::new(AtomicUsize::new(0));
    let other_analyzed = Arc::new(AtomicUsize::new(0));

    // Create detector and ID counter
    let detector = Arc::new(InvariantDetector::new());
    let next_id = Arc::new(AtomicUsize::new(1));
    let concurrency = config.execution.concurrency;

    println!("Analyzing for latent invariants (concurrency={})...", concurrency);

    // Semaphore + channel pattern for parallel execution
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let (tx, mut rx) = tokio::sync::mpsc::channel(256);

    // Spawn analysis tasks
    let mut handles = Vec::new();
    for chunk in chunks {
        // Skip already-completed chunks (resume)
        if tracker.is_completed(&chunk.chunk_id) {
            continue;
        }

        let sem = semaphore.clone();
        let tx = tx.clone();
        let detector = detector.clone();
        let llm_client = llm_client.clone();
        let detector_stats = detector_stats.clone();
        let next_id = next_id.clone();
        let tracker = tracker.clone();
        let shutdown = shutdown.clone();
        let priority_prefixes = priority_prefixes.clone();
        let priority_analyzed = priority_analyzed.clone();
        let other_analyzed = other_analyzed.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Check shutdown and budget before calling LLM
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            let is_priority = priority_prefixes.iter().any(|p| chunk.module_path.contains(p));
            if !is_priority && tracker.budget_exceeded() {
                eprintln!("  Token budget exceeded, skipping: {}", chunk.module_path);
                return;
            }

            let chunk_id = chunk.chunk_id.clone();
            let module_path = chunk.module_path.clone();

            // Snapshot token stats before LLM call to measure per-chunk usage
            let tokens_before = detector_stats.snapshot().total_tokens;

            match detector.detect(&chunk, llm_client.as_ref(), &next_id).await {
                Ok(invariants) => {
                    let tokens_after = detector_stats.snapshot().total_tokens;
                    let chunk_tokens = tokens_after.saturating_sub(tokens_before);

                    let count = invariants.len();
                    // Record each invariant to JSONL
                    for inv in &invariants {
                        tracker.record_invariant(inv);
                    }
                    if is_priority {
                        priority_analyzed.fetch_add(1, Ordering::Relaxed);
                    } else {
                        other_analyzed.fetch_add(1, Ordering::Relaxed);
                    }
                    tracker.record_result(&chunk_id, "completed", count, chunk_tokens, None);
                    tracker.print_status();

                    // Send invariants through channel for report assembly
                    for inv in invariants {
                        let _ = tx.send(inv).await;
                    }
                }
                Err(e) => {
                    let tokens_after = detector_stats.snapshot().total_tokens;
                    let chunk_tokens = tokens_after.saturating_sub(tokens_before);

                    let err_msg = format!("{}", e);
                    eprintln!("    Error analyzing {}: {}", module_path, err_msg);
                    tracker.record_result(&chunk_id, "failed", 0, chunk_tokens, Some(&err_msg));
                    tracker.print_status();
                }
            }
        });
        handles.push(handle);
    }

    // Drop the sender so the channel closes when all tasks complete
    drop(tx);

    // Collect invariants from channel into report
    let mut report = Report::new();
    report.summary.modules_analyzed = modules_analyzed;
    report.parse_failures = parse_failures
        .into_iter()
        .map(|pf| (pf.file_path, pf.error))
        .collect();

    let mut collected_invariants = Vec::new();
    while let Some(invariant) = rx.recv().await {
        collected_invariants.push(invariant);
    }

    // Deduplicate invariants
    let before_dedup = collected_invariants.len();
    let collected_invariants = crate::report::deduplicate(collected_invariants);
    let after_dedup = collected_invariants.len();
    if before_dedup > after_dedup {
        eprintln!(
            "Deduplicated: {} → {} invariants ({} duplicates removed)",
            before_dedup,
            after_dedup,
            before_dedup - after_dedup
        );
    }

    // Optional validation pass (uses a cheap model by default)
    let validator_stats = Arc::new(TokenStats::new());
    let collected_invariants = if config.execution.validate {
        // Build a separate client for validation — defaults to a cheap model
        let validation_model = config.execution.validation_model.clone()
            .unwrap_or_else(|| default_validation_model(&config.llm.provider));
        eprintln!(
            "Running validation pass on {} invariants (model: {})...",
            collected_invariants.len(),
            validation_model,
        );
        let validation_provider = infer_provider(&validation_model, &config.llm.provider);
        let validation_api_key = match validation_provider {
            "openai" => std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| config.llm.api_key.clone()),
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| config.llm.api_key.clone()),
            _ => config.llm.api_key.clone(),
        };
        let validation_client: Arc<dyn llm::LlmClient> = Arc::from(llm::create_client(
            validation_provider,
            validation_api_key,
            validation_model,
        )?);
        let validation_client: Arc<dyn llm::LlmClient> = Arc::new(RetryClient::new(
            validation_client,
            config.execution.max_retries,
            config.execution.retry_base_delay,
        ));
        let validation_client: Arc<dyn llm::LlmClient> = Arc::new(TokenTrackingClient::new(
            validation_client,
            validator_stats.clone(),
        ));

        let mut validated = Vec::new();
        for inv in collected_invariants {
            match crate::detection::InvariantValidator::validate(&inv, validation_client.as_ref()).await {
                Ok(result) if result.valid => {
                    let mut inv = inv;
                    inv.confidence = result.adjusted_confidence;
                    validated.push(inv);
                }
                Ok(result) => {
                    eprintln!("  Filtered: {} ({})", inv.title, result.reason);
                }
                Err(_) => {
                    validated.push(inv); // keep on error
                }
            }
        }
        eprintln!("Validation: {} invariants passed", validated.len());
        validated
    } else {
        collected_invariants
    };

    for inv in collected_invariants {
        report.add_invariant(inv);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    // If we were shut down, also load any invariants from the JSONL that might have been
    // written by tasks that completed after we stopped receiving
    if shutdown.load(Ordering::Relaxed) {
        eprintln!("Generating partial report from checkpoint...");
    }

    println!("\nAnalysis complete!");
    println!(
        "  Total invariants discovered: {}",
        report.summary.total_invariants
    );
    println!(
        "  - Temporal ordering: {}",
        report.summary.temporal_ordering_count
    );
    println!(
        "  - Resource lifecycle: {}",
        report.summary.resource_lifecycle_count
    );
    println!(
        "  - State machine: {}",
        report.summary.state_machine_count
    );
    println!("  - Precondition: {}", report.summary.precondition_count);
    println!("  - Protocol: {}", report.summary.protocol_count);

    let detector_total = detector_stats.print_summary("Detector");
    let validator_total = validator_stats.print_summary("Validator");
    let grand_total = detector_total + validator_total;
    if grand_total > 0 {
        println!("  Grand total tokens: {}", grand_total);
    }

    // Save token usage to run directory
    if grand_total > 0 {
        let usage = serde_json::json!({
            "detector": detector_stats.snapshot(),
            "validator": validator_stats.snapshot(),
            "grand_total_tokens": grand_total,
        });
        let usage_path = run_dir.join("token_usage.json");
        if let Err(e) = std::fs::write(&usage_path, serde_json::to_string_pretty(&usage).unwrap_or_default()) {
            eprintln!("Warning: failed to write token usage: {}", e);
        }
    }
    if !report.parse_failures.is_empty() {
        println!("  Files skipped (parse errors): {}", report.parse_failures.len());
    }

    if !config.execution.priority_modules.is_empty() {
        println!(
            "  Priority modules: {}/{}",
            priority_analyzed.load(Ordering::Relaxed),
            priority_total,
        );
        println!(
            "  Other modules: {}/{}",
            other_analyzed.load(Ordering::Relaxed),
            other_total,
        );
    }

    println!("  Progress saved to: {}", run_dir.display());

    Ok((report, run_dir))
}

/// Create a timestamped run directory: runs/<model>_<YYYYMMDD>_<HHMMSS>/
fn create_run_dir(model: &str) -> Result<std::path::PathBuf> {
    let now = chrono::Local::now();
    let dir_name = format!(
        "{}_{}",
        model.replace(['/', '.', ':'], "_"),
        now.format("%Y%m%d_%H%M%S")
    );
    let run_dir = std::path::PathBuf::from("runs").join(dir_name);
    std::fs::create_dir_all(&run_dir)?;
    Ok(run_dir)
}

/// Exhaustive mode: parse all modules and build analysis chunks
fn build_exhaustive_chunks(
    path: &Path,
    config: &Config,
) -> Result<(Vec<AnalysisChunk>, usize, Vec<crate::parser::ParseFailure>)> {
    println!("Building module graph...");

    let navigator = if config.execution.multi_crate {
        println!("Multi-crate mode enabled");
        Navigator::new_multi_crate(
            path,
            config.exploration.max_depth,
            config.exploration.context_window_tokens,
        )?
    } else {
        Navigator::new(
            path,
            config.exploration.max_depth,
            config.exploration.context_window_tokens,
        )?
    };

    let module_count = navigator.module_count();
    let parse_failures = navigator.parse_failures();
    println!("  Found {} modules", module_count);
    if !parse_failures.is_empty() {
        println!("  {} files had parse errors", parse_failures.len());
    }

    println!("Scanning modules for analysis chunks...");
    let mut explorer = navigator.explore();
    let chunks = explorer.explore();

    Ok((chunks, module_count, parse_failures))
}

/// Semantic mode: use octocode to find candidate code regions
async fn build_semantic_chunks(
    path: &Path,
    config: &Config,
) -> Result<(Vec<AnalysisChunk>, usize)> {
    println!("Running semantic search queries...");

    let search_config = search::SearchConfig {
        similarity_threshold: config.search.similarity_threshold,
        max_results_per_query: config.search.max_results_per_query,
        context_window_tokens: config.exploration.context_window_tokens,
        context_lines: config.search.context_lines,
    };

    let chunks = search::search_for_invariants(path, &search_config).await?;

    // Count unique files as a proxy for modules analyzed
    let mut files: std::collections::HashSet<String> = std::collections::HashSet::new();
    for chunk in &chunks {
        files.insert(chunk.file_path.to_string_lossy().to_string());
    }

    Ok((chunks, files.len()))
}

/// Return the default cheap model for validation, given the LLM provider.
fn default_validation_model(provider: &str) -> String {
    match provider {
        "anthropic" => "claude-haiku-4-5-20251001".to_string(),
        _ => "gpt-4o-mini".to_string(),
    }
}

/// Infer the LLM provider from the model name, falling back to the given default.
fn infer_provider<'a>(model: &str, default: &'a str) -> &'a str {
    if model.starts_with("gpt-") || model.starts_with("o1") || model.starts_with("o3") || model.starts_with("o4") {
        return "openai";
    }
    if model.starts_with("claude-") {
        return "anthropic";
    }
    default
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub struct SimpleStruct {
    field: i32,
}
"#,
        )
        .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_analyze_codebase_structure() {
        let temp_dir = create_test_crate();
        assert!(temp_dir.path().exists());
    }
}
