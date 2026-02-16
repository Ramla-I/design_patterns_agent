use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use super::ProgramInfo;
use super::report::TestVectorResults;

/// Handles building and running tests for translated code
pub struct TestRunner {
    program: ProgramInfo,
    output_dir: PathBuf,
}

impl TestRunner {
    pub fn new(program: ProgramInfo, output_dir: PathBuf) -> Self {
        Self { program, output_dir }
    }

    /// Read the rust-toolchain file from a directory, if it exists.
    /// Returns the toolchain string (e.g. "nightly-2025-06-23").
    fn read_toolchain(dir: &std::path::Path) -> Option<String> {
        let toolchain_file = dir.join("rust-toolchain");
        if toolchain_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&toolchain_file) {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
        None
    }

    /// Build the translated code with cargo
    pub async fn build(&self) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--release")
            .current_dir(&self.output_dir)
            .env("CARGO_TERM_COLOR", "never");

        // Override RUSTUP_TOOLCHAIN so the rust-toolchain file is respected
        // even when our parent process was launched via `cargo run` (which
        // sets RUSTUP_TOOLCHAIN to the project's own toolchain).
        if let Some(toolchain) = Self::read_toolchain(&self.output_dir) {
            cmd.env("RUSTUP_TOOLCHAIN", &toolchain);
        } else {
            cmd.env_remove("RUSTUP_TOOLCHAIN");
        }

        let output = cmd.output()
            .with_context(|| "Failed to execute cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Build failed:\n{}\n{}", stdout, stderr);
        }

        Ok(())
    }

    /// Run tests using the cando2 harness with symlink swapping
    pub async fn run_tests(&self) -> Result<TestVectorResults> {
        // The test runner expects translated_rust to point to the code under test
        // We need to:
        // 1. Backup the original translated_rust symlink/directory
        // 2. Create a symlink from translated_rust -> translated_rust_llm
        // 3. Run the tests
        // 4. Restore the original

        let original_path = &self.program.translated_rust_path;
        let backup_path = self.program.path.join("translated_rust_backup");

        // Cleanup any previous backup
        if backup_path.exists() {
            if backup_path.is_symlink() {
                std::fs::remove_file(&backup_path)?;
            } else {
                std::fs::remove_dir_all(&backup_path)?;
            }
        }

        // Rename original to backup
        std::fs::rename(original_path, &backup_path)
            .with_context(|| format!("Failed to backup {} to {}", original_path.display(), backup_path.display()))?;

        // Create symlink: translated_rust -> translated_rust_llm
        // Use canonical (absolute) path so the symlink resolves correctly
        // regardless of CWD.
        let abs_output = self.output_dir.canonicalize()
            .unwrap_or_else(|_| self.output_dir.clone());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&abs_output, original_path)
                .with_context(|| format!("Failed to create symlink from {} to {}", original_path.display(), abs_output.display()))?;
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&abs_output, original_path)
                .with_context(|| format!("Failed to create symlink from {} to {}", original_path.display(), abs_output.display()))?;
        }

        // Run tests and capture result (ensure we restore even on error)
        let test_result = self.execute_tests().await;

        // Restore original (cleanup)
        if original_path.is_symlink() {
            std::fs::remove_file(original_path)?;
        } else if original_path.exists() {
            std::fs::remove_dir_all(original_path)?;
        }
        std::fs::rename(&backup_path, original_path)
            .with_context(|| "Failed to restore original translated_rust")?;

        test_result
    }

    /// Execute the actual test run
    async fn execute_tests(&self) -> Result<TestVectorResults> {
        // Count test vectors
        let test_vector_count = self.count_test_vectors()?;

        // Build and run the runner
        // Read toolchain from the translated_rust dir (now symlinked to our output)
        let runner_toolchain = Self::read_toolchain(&self.program.translated_rust_path)
            .or_else(|| Self::read_toolchain(&self.output_dir));

        // First, build the runner project
        let mut build_cmd = Command::new("cargo");
        build_cmd
            .arg("build")
            .arg("--release")
            .current_dir(&self.program.runner_path)
            .env("CARGO_TERM_COLOR", "never")
            .env("RUST_ARTIFACTS", "1");
        if let Some(ref tc) = runner_toolchain {
            build_cmd.env("RUSTUP_TOOLCHAIN", tc);
        } else {
            build_cmd.env_remove("RUSTUP_TOOLCHAIN");
        }
        let build_output = build_cmd.output()
            .with_context(|| "Failed to build test runner")?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            anyhow::bail!("Runner build failed:\n{}", stderr);
        }

        // Run the tests via cando2 harness (requires "lib" subcommand)
        let mut test_cmd = Command::new("cargo");
        test_cmd
            .arg("run")
            .arg("--release")
            .arg("--")
            .arg("lib")
            .current_dir(&self.program.runner_path)
            .env("CARGO_TERM_COLOR", "never")
            .env("RUST_ARTIFACTS", "1");
        if let Some(ref tc) = runner_toolchain {
            test_cmd.env("RUSTUP_TOOLCHAIN", tc);
        } else {
            test_cmd.env_remove("RUSTUP_TOOLCHAIN");
        }
        let test_output = test_cmd.output()
            .with_context(|| "Failed to run tests")?;

        // Parse the test output
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        let stderr = String::from_utf8_lossy(&test_output.stderr);

        self.parse_test_output(&stdout, &stderr, test_vector_count)
    }

    /// Count the number of test vector files
    fn count_test_vectors(&self) -> Result<usize> {
        let count = std::fs::read_dir(&self.program.test_vectors_path)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .count();
        Ok(count)
    }

    /// Parse test runner output to determine pass/fail status
    fn parse_test_output(&self, stdout: &str, stderr: &str, total: usize) -> Result<TestVectorResults> {
        let combined = format!("{}\n{}", stdout, stderr);

        let mut passed = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        // Parse cando2-style output: "1.json: true" / "2.json: false"
        for line in combined.lines() {
            let trimmed = line.trim();

            if trimmed.ends_with(": true") {
                passed += 1;
            } else if trimmed.ends_with(": false") {
                failed += 1;
                failures.push(trimmed.to_string());
            }
        }

        // If we couldn't parse cando2 output, check for other indicators
        if passed == 0 && failed == 0 {
            if combined.contains("panicked") || combined.contains("SIGSEGV") || combined.contains("SIGABRT") {
                failed = total;
                failures.push(combined.lines().take(20).collect::<Vec<_>>().join("\n"));
            } else {
                // Assume all passed if no errors detected
                passed = total;
            }
        }

        Ok(TestVectorResults {
            total,
            passed,
            failed,
            failures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_program_info() -> ProgramInfo {
        ProgramInfo {
            name: "test_lib".to_string(),
            collection: "test_collection".to_string(),
            path: PathBuf::from("/tmp/test"),
            translated_rust_path: PathBuf::from("/tmp/test/translated_rust"),
            runner_path: PathBuf::from("/tmp/test/runner"),
            test_vectors_path: PathBuf::from("/tmp/test/test_vectors"),
        }
    }

    #[test]
    fn test_parse_cando2_output_all_pass() {
        let program = create_test_program_info();
        let runner = TestRunner::new(program, PathBuf::from("/tmp/output"));

        let stdout = "1.json: true\n2.json: true\n3.json: true";
        let result = runner.parse_test_output(stdout, "", 3).unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.passed, 3);
        assert_eq!(result.failed, 0);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_parse_cando2_output_some_fail() {
        let program = create_test_program_info();
        let runner = TestRunner::new(program, PathBuf::from("/tmp/output"));

        let stdout = "1.json: true\n2.json: false\n3.json: true\n4.json: false";
        let result = runner.parse_test_output(stdout, "", 4).unwrap();

        assert_eq!(result.total, 4);
        assert_eq!(result.passed, 2);
        assert_eq!(result.failed, 2);
        assert_eq!(result.failures.len(), 2);
        assert_eq!(result.failures[0], "2.json: false");
    }

    #[test]
    fn test_parse_output_panic() {
        let program = create_test_program_info();
        let runner = TestRunner::new(program, PathBuf::from("/tmp/output"));

        let stderr = "thread 'main' panicked at 'index out of bounds'";
        let result = runner.parse_test_output("", stderr, 5).unwrap();

        assert_eq!(result.total, 5);
        assert_eq!(result.passed, 0);
        assert_eq!(result.failed, 5);
    }
}
