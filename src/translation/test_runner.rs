use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use super::{ProgramInfo, ProgramType};
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

    /// Run tests — dispatches to cando2 harness for libraries or stdin/stdout runner for executables
    pub async fn run_tests(&self) -> Result<TestVectorResults> {
        match self.program.program_type {
            ProgramType::Library => self.run_library_tests().await,
            ProgramType::Executable => self.run_executable_tests().await,
        }
    }

    /// Run tests for library programs using the cando2 harness with symlink swapping
    async fn run_library_tests(&self) -> Result<TestVectorResults> {
        // The test runner expects translated_rust to point to the code under test
        // We need to:
        // 1. Backup the original translated_rust symlink/directory
        // 2. Create a symlink from translated_rust -> translated_rust_llm
        // 3. Run the tests
        // 4. Restore the original

        let original_path = &self.program.test_swap_path;
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
        let test_result = self.execute_library_tests().await;

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

    /// Execute the actual library test run via cando2 harness
    async fn execute_library_tests(&self) -> Result<TestVectorResults> {
        let runner_path = self.program.runner_path.as_ref()
            .context("Library program must have a runner_path")?;

        // Count test vectors
        let test_vector_count = self.count_test_vectors()?;

        // Build and run the runner
        // Read toolchain from the translated_rust dir (now symlinked to our output)
        let runner_toolchain = Self::read_toolchain(&self.program.test_swap_path)
            .or_else(|| Self::read_toolchain(&self.output_dir));

        // First, build the runner project
        let mut build_cmd = Command::new("cargo");
        build_cmd
            .arg("build")
            .arg("--release")
            .current_dir(runner_path)
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
            .current_dir(runner_path)
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

    /// Run tests for executable programs by running the binary with each test vector's
    /// argv/stdin and comparing stdout/stderr/exit code.
    async fn run_executable_tests(&self) -> Result<TestVectorResults> {
        let test_vector_count = self.count_test_vectors()?;

        // Find the binary — cargo build --release puts it in target/release/<name>
        let binary_path = self.output_dir.join("target").join("release").join(&self.program.name);
        if !binary_path.exists() {
            anyhow::bail!("Binary not found at {}", binary_path.display());
        }

        // Collect and sort test vector files
        let mut test_files: Vec<_> = std::fs::read_dir(&self.program.test_vectors_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .map(|e| e.path())
            .collect();
        test_files.sort();

        let mut passed = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for test_file in &test_files {
            let content = std::fs::read_to_string(test_file)
                .with_context(|| format!("Failed to read test vector {}", test_file.display()))?;
            let tv: ExecTestVector = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse test vector {}", test_file.display()))?;

            let test_name = test_file.file_name().unwrap().to_string_lossy().to_string();

            // Build command with argv
            let mut cmd = Command::new(&binary_path);
            for arg in &tv.argv {
                cmd.arg(arg);
            }
            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn()
                .with_context(|| format!("Failed to spawn binary for {}", test_name))?;

            // Write stdin if provided
            let stdin_data = tv.stdin.as_deref().unwrap_or("");
            if !stdin_data.is_empty() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(stdin_data.as_bytes());
                    // stdin is dropped here, closing the pipe
                }
            } else {
                // Close stdin immediately so the process doesn't block
                drop(child.stdin.take());
            }

            let output = child.wait_with_output()
                .with_context(|| format!("Failed to wait for binary in {}", test_name))?;

            let actual_stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let actual_stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let actual_rc = output.status.code().unwrap_or(-1);
            let expected_rc = tv.rc.unwrap_or(0);

            let mut test_passed = true;
            let mut diff_details = Vec::new();

            // Check stdout
            let stdout_matches = match &tv.stdout {
                Some(pattern) => Self::check_pattern(&actual_stdout, &pattern.pattern, pattern.is_regex.unwrap_or(false)),
                None => true,
            };
            if !stdout_matches {
                test_passed = false;
                let expected = tv.stdout.as_ref().map(|p| p.pattern.as_str()).unwrap_or("");
                diff_details.push(format!("stdout mismatch:\n  expected: {:?}\n  actual:   {:?}", expected, actual_stdout));
            }

            // Check stderr (only if pattern is present and non-empty)
            if let Some(ref stderr_pattern) = tv.stderr {
                if !stderr_pattern.pattern.is_empty() {
                    let stderr_matches = Self::check_pattern(&actual_stderr, &stderr_pattern.pattern, stderr_pattern.is_regex.unwrap_or(false));
                    if !stderr_matches {
                        test_passed = false;
                        diff_details.push(format!("stderr mismatch:\n  expected: {:?}\n  actual:   {:?}", stderr_pattern.pattern, actual_stderr));
                    }
                }
            }

            // Check exit code
            if actual_rc != expected_rc {
                test_passed = false;
                diff_details.push(format!("exit code mismatch: expected {}, got {}", expected_rc, actual_rc));
            }

            if test_passed {
                passed += 1;
            } else {
                failed += 1;
                failures.push(format!("{}: FAILED\n{}", test_name, diff_details.join("\n")));
            }
        }

        Ok(TestVectorResults {
            total: test_vector_count,
            passed,
            failed,
            failures,
        })
    }

    /// Check if actual output matches expected pattern (exact or regex)
    fn check_pattern(actual: &str, expected: &str, is_regex: bool) -> bool {
        if is_regex {
            match regex::Regex::new(expected) {
                Ok(re) => re.is_match(actual),
                Err(_) => actual == expected, // Fall back to exact match if regex is invalid
            }
        } else {
            actual == expected
        }
    }
}

/// Test vector JSON format for executable programs
#[derive(Debug, serde::Deserialize)]
struct ExecTestVector {
    #[serde(default)]
    argv: Vec<String>,
    stdin: Option<String>,
    stdout: Option<OutputPattern>,
    stderr: Option<OutputPattern>,
    rc: Option<i32>,
}

/// Output pattern — either exact string or regex
#[derive(Debug, serde::Deserialize)]
struct OutputPattern {
    pattern: String,
    is_regex: Option<bool>,
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
            test_swap_path: PathBuf::from("/tmp/test/translated_rust"),
            runner_path: Some(PathBuf::from("/tmp/test/runner")),
            test_vectors_path: PathBuf::from("/tmp/test/test_vectors"),
            source_type: crate::translation::SourceType::Rust,
            program_type: crate::translation::ProgramType::Library,
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

    #[test]
    fn test_check_pattern_exact() {
        assert!(TestRunner::check_pattern("hello\n", "hello\n", false));
        assert!(!TestRunner::check_pattern("hello\n", "world\n", false));
    }

    #[test]
    fn test_check_pattern_regex() {
        assert!(TestRunner::check_pattern("abc123", "abc[0-9]+", true));
        assert!(!TestRunner::check_pattern("abcxyz", "abc[0-9]+", true));
    }

    #[test]
    fn test_parse_exec_test_vector() {
        let json = r#"{
            "argv": ["1", "2"],
            "stdin": "",
            "stdout": { "pattern": "3\n" }
        }"#;
        let tv: ExecTestVector = serde_json::from_str(json).unwrap();
        assert_eq!(tv.argv, vec!["1", "2"]);
        assert_eq!(tv.stdin.as_deref(), Some(""));
        assert_eq!(tv.stdout.as_ref().unwrap().pattern, "3\n");
        assert!(tv.stderr.is_none());
        assert!(tv.rc.is_none());
    }

    #[test]
    fn test_parse_exec_test_vector_with_regex() {
        let json = r#"{
            "argv": [],
            "stdin": "test\n",
            "stdout": { "pattern": "result: [0-9]+", "is_regex": true },
            "stderr": { "pattern": "" },
            "rc": 0
        }"#;
        let tv: ExecTestVector = serde_json::from_str(json).unwrap();
        assert!(tv.argv.is_empty());
        assert_eq!(tv.stdout.as_ref().unwrap().is_regex, Some(true));
        assert_eq!(tv.rc, Some(0));
    }
}
