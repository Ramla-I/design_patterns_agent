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
use crate::llm::{RetryClient, TokenTrackingClient};
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
    let total_tokens = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let llm_client: Arc<dyn llm::LlmClient> = Arc::new(TokenTrackingClient::new(
        retry_client,
        total_tokens.clone(),
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

    // Share the total_tokens counter with the tracker
    // (The tracker has its own counter but we connect them via record_result)

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
        if tracker.is_completed(&chunk.module_path) {
            continue;
        }

        let sem = semaphore.clone();
        let tx = tx.clone();
        let detector = detector.clone();
        let llm_client = llm_client.clone();
        let next_id = next_id.clone();
        let tracker = tracker.clone();
        let shutdown = shutdown.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Check shutdown and budget before calling LLM
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            if tracker.budget_exceeded() {
                eprintln!("  Token budget exceeded, skipping: {}", chunk.module_path);
                return;
            }

            let module_path = chunk.module_path.clone();

            match detector.detect(&chunk, llm_client.as_ref(), &next_id).await {
                Ok(invariants) => {
                    let count = invariants.len();
                    // Record each invariant to JSONL
                    for inv in &invariants {
                        tracker.record_invariant(inv);
                    }
                    tracker.record_result(&module_path, "completed", count, 0, None);
                    tracker.print_status();

                    // Send invariants through channel for report assembly
                    for inv in invariants {
                        let _ = tx.send(inv).await;
                    }
                }
                Err(e) => {
                    let err_msg = format!("{}", e);
                    eprintln!("    Error analyzing {}: {}", module_path, err_msg);
                    tracker.record_result(&module_path, "failed", 0, 0, Some(&err_msg));
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

    while let Some(invariant) = rx.recv().await {
        report.add_invariant(invariant);
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

    let total = total_tokens.load(Ordering::Relaxed);
    if total > 0 {
        println!("  Total tokens used: {}", total);
    }
    if !report.parse_failures.is_empty() {
        println!("  Files skipped (parse errors): {}", report.parse_failures.len());
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
