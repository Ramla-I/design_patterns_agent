use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use super::report::IdiomaticityMetrics;

/// Analyzes code using Clippy and calculates idiomaticity metrics
pub struct ClippyAnalyzer;

#[derive(Debug, Deserialize)]
struct ClippyMessage {
    reason: String,
    #[serde(default)]
    message: Option<ClippyDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct ClippyDiagnostic {
    level: String,
    #[serde(default)]
    code: Option<ClippyCode>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ClippyCode {
    code: String,
}

impl ClippyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Run clippy and analyze the code for idiomaticity
    pub async fn analyze(&self, path: &Path) -> Result<IdiomaticityMetrics> {
        // Run clippy with JSON output
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--message-format=json")
            .arg("--")
            .arg("-W")
            .arg("clippy::all")
            .current_dir(path)
            .env("CARGO_TERM_COLOR", "never")
            .output()
            .with_context(|| "Failed to execute cargo clippy")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse clippy output
        let mut warnings = Vec::new();
        for line in stdout.lines() {
            if let Ok(msg) = serde_json::from_str::<ClippyMessage>(line) {
                if msg.reason == "compiler-message" {
                    if let Some(diagnostic) = msg.message {
                        if diagnostic.level == "warning" {
                            warnings.push(diagnostic.message);
                        }
                    }
                }
            }
        }

        // Read source code for additional analysis
        let lib_rs = path.join("lib.rs");
        let source = if lib_rs.exists() {
            std::fs::read_to_string(&lib_rs).unwrap_or_default()
        } else {
            String::new()
        };

        // Count unsafe blocks
        let unsafe_blocks = self.count_unsafe_blocks(&source);

        // Count raw pointer usage
        let raw_pointers = self.count_raw_pointers(&source);

        // Calculate idiomaticity score
        let score = self.calculate_score(unsafe_blocks, raw_pointers, warnings.len());

        Ok(IdiomaticityMetrics {
            score,
            unsafe_blocks,
            raw_pointers,
            clippy_warnings: warnings.len(),
        })
    }

    /// Count the number of unsafe blocks in the source
    fn count_unsafe_blocks(&self, source: &str) -> usize {
        // Count occurrences of "unsafe {" and "unsafe fn"
        // This is a simple heuristic, not a full parser
        let mut count = 0;

        // Count standalone unsafe blocks
        let unsafe_block_pattern = regex::Regex::new(r"unsafe\s*\{").unwrap();
        count += unsafe_block_pattern.find_iter(source).count();

        count
    }

    /// Count raw pointer declarations and usage
    fn count_raw_pointers(&self, source: &str) -> usize {
        let mut count = 0;

        // Count *mut and *const type declarations
        let raw_ptr_pattern = regex::Regex::new(r"\*(?:mut|const)\s+\w+").unwrap();
        count += raw_ptr_pattern.find_iter(source).count();

        // Count pointer casts
        let cast_pattern = regex::Regex::new(r"as\s+\*(?:mut|const)").unwrap();
        count += cast_pattern.find_iter(source).count();

        count
    }

    /// Calculate an idiomaticity score (0-100)
    /// Higher score = more idiomatic
    fn calculate_score(&self, unsafe_blocks: usize, raw_pointers: usize, warnings: usize) -> f64 {
        // Start with 100 and deduct points
        let mut score = 100.0;

        // Deduct for unsafe blocks (but some are necessary for FFI)
        // Penalize less for first few (expected for FFI), more for excessive
        if unsafe_blocks > 5 {
            score -= (unsafe_blocks - 5) as f64 * 2.0;
        }

        // Deduct for raw pointers (again, some expected for FFI)
        if raw_pointers > 10 {
            score -= (raw_pointers - 10) as f64 * 1.0;
        }

        // Deduct for clippy warnings
        score -= warnings as f64 * 3.0;

        // Ensure score is in valid range
        score.max(0.0).min(100.0)
    }
}

impl Default for ClippyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_unsafe_blocks() {
        let analyzer = ClippyAnalyzer::new();

        let source = r#"
fn main() {
    unsafe {
        // something
    }

    let x = unsafe { ptr.read() };
}
"#;
        assert_eq!(analyzer.count_unsafe_blocks(source), 2);
    }

    #[test]
    fn test_count_raw_pointers() {
        let analyzer = ClippyAnalyzer::new();

        let source = r#"
let ptr: *mut u8 = std::ptr::null_mut();
let cptr: *const i32 = data as *const i32;
"#;
        // Counts: *mut u8, *const i32, as *const i32, and potentially others
        // The exact count depends on regex matching, just verify it's > 0
        let count = analyzer.count_raw_pointers(source);
        assert!(count >= 3, "Expected at least 3 raw pointer usages, got {}", count);
    }

    #[test]
    fn test_calculate_score() {
        let analyzer = ClippyAnalyzer::new();

        // Minimal issues = high score
        let score = analyzer.calculate_score(2, 5, 0);
        assert!(score > 90.0);

        // Many issues = lower score
        let score = analyzer.calculate_score(20, 50, 10);
        assert!(score < 50.0);
    }

    #[test]
    fn test_score_bounds() {
        let analyzer = ClippyAnalyzer::new();

        // Score should never exceed 100
        let score = analyzer.calculate_score(0, 0, 0);
        assert_eq!(score, 100.0);

        // Score should never go below 0
        let score = analyzer.calculate_score(100, 200, 100);
        assert_eq!(score, 0.0);
    }
}
