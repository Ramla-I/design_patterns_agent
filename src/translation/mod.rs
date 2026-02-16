mod translator;
mod test_runner;
mod clippy;
mod feedback;
mod report;

pub use translator::Translator;
pub use test_runner::TestRunner;
pub use clippy::ClippyAnalyzer;
pub use feedback::FeedbackFormatter;
pub use report::{TranslationReport, TranslationResult, ProgramStatus, TestVectorResults, IdiomaticityMetrics, DesignPatternMetrics};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli::Config;
use crate::llm;

/// Configuration for the translation pipeline
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    /// Maximum number of retry attempts for failed translations
    pub max_retries: usize,
    /// Maximum lines in a source file before skipping
    pub max_lines: usize,
    /// Whether to run design patterns analysis on successful translations
    pub analyze_patterns: bool,
    /// Path to the cando2 tool (relative to runner directory)
    pub cando2_path: String,
    /// Skip running tests (only verify build succeeds)
    pub skip_tests: bool,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            max_lines: 1000,
            analyze_patterns: false,
            cando2_path: "../../../../tools/cando2".to_string(),
            skip_tests: false,
        }
    }
}

/// A discovered program ready for translation
#[derive(Debug, Clone)]
pub struct ProgramInfo {
    /// Name of the program (directory name)
    pub name: String,
    /// Collection name (parent directory name, e.g., "B02_organic")
    pub collection: String,
    /// Path to the program directory
    pub path: PathBuf,
    /// Path to the translated_rust directory
    pub translated_rust_path: PathBuf,
    /// Path to the runner directory
    pub runner_path: PathBuf,
    /// Path to test vectors directory
    pub test_vectors_path: PathBuf,
}

/// Main orchestrator for the translation pipeline
pub struct TranslationAgent {
    config: TranslationConfig,
    llm_config: Config,
}

impl TranslationAgent {
    pub fn new(config: TranslationConfig, llm_config: Config) -> Self {
        Self { config, llm_config }
    }

    /// Discover all programs in the given path that can be translated.
    /// Prefers `translated_rust/` (crat output), falls back to `dst/<name>/` (raw c2rust).
    pub fn discover_programs(&self, path: &Path) -> Result<Vec<ProgramInfo>> {
        let mut programs = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Walk the directory looking for program directories.
        // A program directory contains runner/ + test_vectors/ + (translated_rust/ or dst/).
        // min_depth=0 so we also check if `path` itself is a program directory.
        for entry in WalkDir::new(path)
            .min_depth(0)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir() {
                continue;
            }

            let program_dir = entry.path();
            let runner_path = program_dir.join("runner");
            let test_vectors_path = program_dir.join("test_vectors");

            // Must have runner and test_vectors
            if !runner_path.exists() || !test_vectors_path.exists() {
                continue;
            }

            // Deduplicate — don't re-discover nested directories
            if !seen.insert(program_dir.to_path_buf()) {
                continue;
            }

            let program_name = program_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let collection_name = program_dir
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Resolve source directory: prefer translated_rust/, fall back to dst/<name>/
            let translated_rust = program_dir.join("translated_rust");
            let dst_nested = program_dir.join("dst").join(&program_name);

            let source_path = if translated_rust.join("src").join("lib.rs").exists()
                || translated_rust.join("lib.rs").exists()
            {
                translated_rust
            } else if dst_nested.join("src").join("lib.rs").exists()
                || dst_nested.join("lib.rs").exists()
            {
                dst_nested
            } else {
                // Try first subdirectory of dst/
                let dst_dir = program_dir.join("dst");
                if dst_dir.exists() {
                    let mut found = None;
                    if let Ok(entries) = std::fs::read_dir(&dst_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            if entry.path().is_dir() && entry.path().join("lib.rs").exists() {
                                found = Some(entry.path());
                                break;
                            }
                        }
                    }
                    match found {
                        Some(p) => p,
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            programs.push(ProgramInfo {
                name: program_name,
                collection: collection_name,
                path: program_dir.to_path_buf(),
                translated_rust_path: source_path,
                runner_path,
                test_vectors_path,
            });
        }

        // Sort by collection then name for consistent ordering
        programs.sort_by(|a, b| {
            (&a.collection, &a.name).cmp(&(&b.collection, &b.name))
        });

        Ok(programs)
    }

    /// Collect all Rust source code from a program directory into a single string.
    /// Reads lib.rs and src/lib.rs (skipping mod.rs and build.rs), combining them
    /// so the LLM sees the full picture.
    fn collect_source_code(&self, source_dir: &Path) -> Result<String> {
        let mut parts = Vec::new();

        // Read the top-level lib.rs (crate attributes and module declarations)
        let top_lib = source_dir.join("lib.rs");
        if top_lib.exists() {
            let content = std::fs::read_to_string(&top_lib)
                .with_context(|| format!("Failed to read {}", top_lib.display()))?;
            // Only include if it has more than just module declarations
            let has_real_code = content.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("pub mod ")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("#![")
                    && !trimmed.starts_with("} //")
            });
            // Always include #![feature(...)] and #![allow(...)] attributes
            let attrs: String = content
                .lines()
                .filter(|l| l.trim().starts_with("#!["))
                .collect::<Vec<_>>()
                .join("\n");
            if !attrs.is_empty() {
                parts.push(attrs);
            }
            if has_real_code {
                // Include non-attribute, non-module-decl lines
                let code: String = content
                    .lines()
                    .filter(|l| {
                        let t = l.trim();
                        !t.starts_with("#![")
                            && !t.starts_with("pub mod ")
                            && !t.starts_with("} //")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let code = code.trim();
                if !code.is_empty() {
                    parts.push(code.to_string());
                }
            }
        }

        // Read src/lib.rs (the actual function implementations)
        let src_lib = source_dir.join("src").join("lib.rs");
        if src_lib.exists() {
            let content = std::fs::read_to_string(&src_lib)
                .with_context(|| format!("Failed to read {}", src_lib.display()))?;
            parts.push(content);
        }

        if parts.is_empty() {
            anyhow::bail!("No Rust source files found in {}", source_dir.display());
        }

        Ok(parts.join("\n\n"))
    }

    /// Translate a single program
    pub async fn translate_program(&self, program: &ProgramInfo, run_dir: &Path) -> Result<TranslationResult> {
        let source_label = if program.translated_rust_path.to_string_lossy().contains("/dst/") {
            "dst (c2rust)"
        } else {
            "translated_rust (crat)"
        };
        println!("📝 Processing: {}/{} [source: {}]", program.collection, program.name, source_label);

        // Collect all source code from the program
        let source_code = self.collect_source_code(&program.translated_rust_path)?;

        let line_count = source_code.lines().count();
        if line_count > self.config.max_lines {
            println!("   ⏭️  Skipping: File exceeds {} lines ({} lines)", self.config.max_lines, line_count);
            return Ok(TranslationResult {
                program: program.name.clone(),
                collection: program.collection.clone(),
                status: ProgramStatus::Skipped,
                skip_reason: Some(format!("File exceeds {} lines ({} lines)", self.config.max_lines, line_count)),
                attempts: 0,
                test_vectors: None,
                idiomaticity: None,
                design_patterns: None,
                last_build_error: None,
                total_tokens: 0,
                total_llm_secs: 0.0,
            });
        }

        // Create LLM client
        let llm_client = llm::create_client(
            self.llm_config.llm.api_key.clone(),
            self.llm_config.llm.model.clone(),
        )?;

        // Create output directory inside the run directory
        let output_dir = run_dir.join(&program.name).join("translated_rust_llm");
        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir)?;
        }
        std::fs::create_dir_all(&output_dir)?;

        // Copy supporting files
        self.copy_supporting_files(&program.translated_rust_path, &output_dir)?;

        // Initialize components
        let translator = Translator::new();
        let test_runner = TestRunner::new(program.clone(), output_dir.clone());
        let clippy_analyzer = ClippyAnalyzer::new();
        let feedback_formatter = FeedbackFormatter::new();

        let mut last_feedback: Option<String> = None;
        let mut last_build_error_msg: Option<String> = None;
        let mut attempts = 0;
        let mut cumulative_tokens: usize = 0;
        let mut cumulative_llm_secs: f64 = 0.0;

        // Translation retry loop
        loop {
            attempts += 1;
            println!("   🔄 Attempt {}/{}", attempts, self.config.max_retries);

            // Translate the code
            let translated_code = match translator
                .translate(&source_code, last_feedback.as_deref(), llm_client.as_ref())
                .await
            {
                Ok(output) => {
                    println!("   📊 LLM: {:.1}s | {} prompt + {} completion = {} tokens",
                        output.duration_secs, output.prompt_tokens, output.completion_tokens, output.total_tokens);
                    cumulative_tokens += output.total_tokens;
                    cumulative_llm_secs += output.duration_secs;
                    output.code
                }
                Err(e) => {
                    let err_str = format!("{:#}", e);
                    if err_str.contains("context_length_exceeded") || err_str.contains("maximum context length") {
                        println!("   ⏭️  Skipping: LLM context length exceeded");
                        return Ok(TranslationResult {
                            program: program.name.clone(),
                            collection: program.collection.clone(),
                            status: ProgramStatus::Skipped,
                            skip_reason: Some("LLM context length exceeded".to_string()),
                            attempts,
                            test_vectors: None,
                            idiomaticity: None,
                            design_patterns: None,
                            last_build_error: None,
                            total_tokens: cumulative_tokens,
                            total_llm_secs: cumulative_llm_secs,
                        });
                    }
                    // Transient LLM errors (deserialization, network, rate limit) — retry
                    println!("   ⚠️  LLM error: {}", err_str.lines().next().unwrap_or(&err_str));
                    if attempts >= self.config.max_retries {
                        return Ok(TranslationResult {
                            program: program.name.clone(),
                            collection: program.collection.clone(),
                            status: ProgramStatus::Failed,
                            skip_reason: Some(format!("LLM error after {} attempts: {}", attempts, err_str.lines().next().unwrap_or(&err_str))),
                            attempts,
                            test_vectors: None,
                            idiomaticity: None,
                            design_patterns: None,
                            last_build_error: None,
                            total_tokens: cumulative_tokens,
                            total_llm_secs: cumulative_llm_secs,
                        });
                    }
                    continue;
                }
            };

            // Write translated code
            let output_lib_rs = output_dir.join("lib.rs");
            std::fs::write(&output_lib_rs, &translated_code)?;

            // Build the translation
            println!("   🔨 Building...");
            match test_runner.build().await {
                Ok(_) => {
                    println!("   ✅ Build succeeded");
                }
                Err(build_error) => {
                    let err_msg = format!("{:#}", build_error);
                    println!("   ❌ Build failed");
                    last_build_error_msg = Some(err_msg);
                    if attempts >= self.config.max_retries {
                        return Ok(TranslationResult {
                            program: program.name.clone(),
                            collection: program.collection.clone(),
                            status: ProgramStatus::Failed,
                            skip_reason: Some(format!("Build failed after {} attempts", attempts)),
                            attempts,
                            test_vectors: None,
                            idiomaticity: None,
                            design_patterns: None,
                            last_build_error: last_build_error_msg,
                        total_tokens: cumulative_tokens,
                        total_llm_secs: cumulative_llm_secs,
                        });
                    }
                    last_feedback = Some(feedback_formatter.format_build_error(&build_error));
                    continue;
                }
            }

            // Skip tests if configured
            if self.config.skip_tests {
                println!("   ⏭️  Skipping tests (--skip-tests)");

                // Run clippy analysis
                println!("   📋 Running clippy...");
                let idiomaticity = clippy_analyzer.analyze(&output_dir).await?;

                // Run design patterns analysis if enabled
                let design_patterns = if self.config.analyze_patterns {
                    println!("   🔍 Running design patterns analysis...");
                    Some(self.run_design_patterns_analysis(&output_dir).await?)
                } else {
                    None
                };

                return Ok(TranslationResult {
                    program: program.name.clone(),
                    collection: program.collection.clone(),
                    status: ProgramStatus::Success,
                    skip_reason: None,
                    attempts,
                    test_vectors: None,
                    idiomaticity: Some(idiomaticity),
                    design_patterns,
                    last_build_error: None,
                    total_tokens: cumulative_tokens,
                    total_llm_secs: cumulative_llm_secs,
                });
            }

            // Run tests
            println!("   🧪 Running tests...");
            match test_runner.run_tests().await {
                Ok(test_results) => {
                    if test_results.failed == 0 {
                        println!("   ✅ All {} tests passed", test_results.total);

                        // Run clippy analysis
                        println!("   📋 Running clippy...");
                        let idiomaticity = clippy_analyzer.analyze(&output_dir).await?;

                        // Run design patterns analysis if enabled
                        let design_patterns = if self.config.analyze_patterns {
                            println!("   🔍 Running design patterns analysis...");
                            Some(self.run_design_patterns_analysis(&output_dir).await?)
                        } else {
                            None
                        };

                        return Ok(TranslationResult {
                            program: program.name.clone(),
                            collection: program.collection.clone(),
                            status: ProgramStatus::Success,
                            skip_reason: None,
                            attempts,
                            test_vectors: Some(test_results),
                            idiomaticity: Some(idiomaticity),
                            design_patterns,
                            last_build_error: None,
                            total_tokens: cumulative_tokens,
                            total_llm_secs: cumulative_llm_secs,
                        });
                    } else {
                        println!("   ❌ {} of {} tests failed", test_results.failed, test_results.total);
                        if attempts >= self.config.max_retries {
                            return Ok(TranslationResult {
                                program: program.name.clone(),
                                collection: program.collection.clone(),
                                status: ProgramStatus::Failed,
                                skip_reason: Some(format!("Tests failed after {} attempts", attempts)),
                                attempts,
                                test_vectors: Some(test_results.clone()),
                                idiomaticity: None,
                                design_patterns: None,
                                last_build_error: None,
                                total_tokens: cumulative_tokens,
                                total_llm_secs: cumulative_llm_secs,
                            });
                        }
                        last_feedback = Some(feedback_formatter.format_test_failures(&test_results));
                    }
                }
                Err(test_error) => {
                    println!("   ❌ Test execution failed: {}", test_error);
                    if attempts >= self.config.max_retries {
                        return Ok(TranslationResult {
                            program: program.name.clone(),
                            collection: program.collection.clone(),
                            status: ProgramStatus::Failed,
                            skip_reason: Some(format!("Test execution failed: {}", test_error)),
                            attempts,
                            test_vectors: None,
                            idiomaticity: None,
                            total_tokens: cumulative_tokens,
                            total_llm_secs: cumulative_llm_secs,
                            design_patterns: None,
                            last_build_error: None,
                        });
                    }
                    last_feedback = Some(feedback_formatter.format_test_error(&test_error.to_string()));
                }
            }
        }
    }

    /// Copy supporting files (Cargo.toml, .cargo/config.toml, rust-toolchain, build.rs)
    /// and patch Cargo.toml so lib.path = "lib.rs" (flat layout, no src/ module tree).
    fn copy_supporting_files(&self, src: &Path, dst: &Path) -> Result<()> {
        // Copy and patch Cargo.toml — ensure lib path is "lib.rs" and crate-type includes cdylib
        let cargo_toml = src.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            let dst_toml = dst.join("Cargo.toml");
            // Parse, patch lib.path, and rewrite
            if let Ok(mut doc) = content.parse::<toml_edit::DocumentMut>() {
                if let Some(lib) = doc.get_mut("lib").and_then(|v| v.as_table_mut()) {
                    lib.insert("path", toml_edit::value("lib.rs"));
                }
                std::fs::write(&dst_toml, doc.to_string())?;
            } else {
                // Fallback: copy as-is
                std::fs::copy(&cargo_toml, &dst_toml)?;
            }
        }

        // Copy .cargo directory
        let cargo_dir = src.join(".cargo");
        if cargo_dir.exists() {
            let dst_cargo_dir = dst.join(".cargo");
            std::fs::create_dir_all(&dst_cargo_dir)?;
            for entry in std::fs::read_dir(&cargo_dir)? {
                let entry = entry?;
                std::fs::copy(entry.path(), dst_cargo_dir.join(entry.file_name()))?;
            }
        }

        // Copy rust-toolchain
        let rust_toolchain = src.join("rust-toolchain");
        if rust_toolchain.exists() {
            std::fs::copy(&rust_toolchain, dst.join("rust-toolchain"))?;
        }

        // Copy build.rs if it exists
        let build_rs = src.join("build.rs");
        if build_rs.exists() {
            std::fs::copy(&build_rs, dst.join("build.rs"))?;
        }

        Ok(())
    }

    /// Run design patterns analysis on translated code
    async fn run_design_patterns_analysis(&self, path: &Path) -> Result<DesignPatternMetrics> {
        // Use the existing agent to analyze
        let report = crate::agent::analyze_codebase(path, &self.llm_config).await?;

        Ok(DesignPatternMetrics {
            invariants_found: report.summary.total_invariants,
            state_machine: report.summary.state_machine_count,
            linear_type: report.summary.linear_type_count,
            ownership: report.summary.ownership_count,
        })
    }

    /// Translate all discovered programs.
    /// Outputs are placed in `runs/<model>_<YYYYMMDD>_<HHMMSS>/`.
    pub async fn translate_all(&self, path: &Path) -> Result<TranslationReport> {
        let programs = self.discover_programs(path)?;
        println!("🔍 Discovered {} programs to translate\n", programs.len());

        // Build run directory: runs/<model>_<YYYYMMDD>_<HHMMSS>
        let now = chrono::Local::now();
        let model_slug = self.llm_config.llm.model
            .replace('/', "_")
            .replace(' ', "_");
        let run_name = format!("{}_{}", model_slug, now.format("%Y%m%d_%H%M%S"));
        let run_dir = PathBuf::from("runs").join(&run_name);
        std::fs::create_dir_all(&run_dir)
            .with_context(|| format!("Failed to create run directory: {}", run_dir.display()))?;

        println!("📂 Run directory: {}\n", run_dir.display());

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;

        for (idx, program) in programs.iter().enumerate() {
            println!("\n[{}/{}] {}/{}", idx + 1, programs.len(), program.collection, program.name);

            let result = self.translate_program(program, &run_dir).await?;

            match result.status {
                ProgramStatus::Success => success_count += 1,
                ProgramStatus::Failed => failed_count += 1,
                ProgramStatus::Skipped => skipped_count += 1,
            }

            // Write individual result file inside the run directory
            let result_path = run_dir.join(&program.name).join("translated_rust_llm").join("results.json");
            if let Ok(json) = serde_json::to_string_pretty(&result) {
                let _ = std::fs::write(&result_path, json);
            }

            results.push(result);
        }

        let grand_total_tokens: usize = results.iter().map(|r| r.total_tokens).sum();
        let grand_total_secs: f64 = results.iter().map(|r| r.total_llm_secs).sum();

        println!("\n📊 Translation Summary:");
        println!("   ✅ Success: {}", success_count);
        println!("   ❌ Failed: {}", failed_count);
        println!("   ⏭️  Skipped: {}", skipped_count);
        println!("   📁 Total: {}", programs.len());
        println!("   🔢 Total tokens: {}", grand_total_tokens);
        println!("   ⏱️  Total LLM time: {:.1}s", grand_total_secs);

        let run_dir_str = run_dir.to_string_lossy().to_string();

        Ok(TranslationReport {
            total_programs: programs.len(),
            success_count,
            failed_count,
            skipped_count,
            run_dir: Some(run_dir_str),
            results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_config_default() {
        let config = TranslationConfig::default();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.max_lines, 1000);
        assert!(!config.analyze_patterns);
    }
}
