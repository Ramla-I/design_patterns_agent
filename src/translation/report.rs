use serde::{Deserialize, Serialize};

/// Overall translation report for all programs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationReport {
    pub total_programs: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    /// Path to the run directory where outputs are stored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_dir: Option<String>,
    pub results: Vec<TranslationResult>,
}

/// Result for a single program translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub program: String,
    pub collection: String,
    pub status: ProgramStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    pub attempts: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_vectors: Option<TestVectorResults>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idiomaticity: Option<IdiomaticityMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_patterns: Option<DesignPatternMetrics>,
    /// Last build error message (if build failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_build_error: Option<String>,
    /// Total LLM tokens used across all attempts
    pub total_tokens: usize,
    /// Total LLM time in seconds across all attempts
    pub total_llm_secs: f64,
}

/// Status of a program translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgramStatus {
    Success,
    Failed,
    Skipped,
}

/// Test vector execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVectorResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
}

/// Idiomaticity metrics for translated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdiomaticityMetrics {
    /// Score from 0-100, higher is more idiomatic
    pub score: f64,
    /// Number of unsafe blocks in the code
    pub unsafe_blocks: usize,
    /// Number of raw pointer usages
    pub raw_pointers: usize,
    /// Number of clippy warnings
    pub clippy_warnings: usize,
}

/// Design pattern analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPatternMetrics {
    pub invariants_found: usize,
    pub state_machine: usize,
    pub linear_type: usize,
    pub ownership: usize,
}

impl TranslationReport {
    pub fn new() -> Self {
        Self {
            total_programs: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            run_dir: None,
            results: Vec::new(),
        }
    }

    /// Generate a markdown summary of the report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Translation Report\n\n");

        md.push_str("## Summary\n\n");
        md.push_str(&format!("| Metric | Count |\n"));
        md.push_str(&format!("|--------|-------|\n"));
        md.push_str(&format!("| Total Programs | {} |\n", self.total_programs));
        md.push_str(&format!("| ✅ Success | {} |\n", self.success_count));
        md.push_str(&format!("| ❌ Failed | {} |\n", self.failed_count));
        md.push_str(&format!("| ⏭️ Skipped | {} |\n", self.skipped_count));

        let success_rate = if self.total_programs > 0 {
            (self.success_count as f64 / self.total_programs as f64) * 100.0
        } else {
            0.0
        };
        md.push_str(&format!("| Success Rate | {:.1}% |\n", success_rate));

        let grand_total_tokens: usize = self.results.iter().map(|r| r.total_tokens).sum();
        let grand_total_secs: f64 = self.results.iter().map(|r| r.total_llm_secs).sum();
        md.push_str(&format!("| Total LLM Tokens | {} |\n", grand_total_tokens));
        md.push_str(&format!("| Total LLM Time | {:.1}s |\n\n", grand_total_secs));

        // Group results by collection
        let mut by_collection: std::collections::HashMap<&str, Vec<&TranslationResult>> =
            std::collections::HashMap::new();
        for result in &self.results {
            by_collection
                .entry(&result.collection)
                .or_default()
                .push(result);
        }

        for (collection, results) in by_collection {
            md.push_str(&format!("## {}\n\n", collection));
            md.push_str("| Program | Status | Attempts | Tests | Idiomaticity | Tokens | LLM Time |\n");
            md.push_str("|---------|--------|----------|-------|-------------|--------|----------|\n");

            for result in results {
                let status = match result.status {
                    ProgramStatus::Success => "✅",
                    ProgramStatus::Failed => "❌",
                    ProgramStatus::Skipped => "⏭️",
                };

                let tests = result
                    .test_vectors
                    .as_ref()
                    .map(|t| format!("{}/{}", t.passed, t.total))
                    .unwrap_or_else(|| "-".to_string());

                let idiom = result
                    .idiomaticity
                    .as_ref()
                    .map(|i| format!("{:.1}", i.score))
                    .unwrap_or_else(|| "-".to_string());

                let tokens = if result.total_tokens > 0 {
                    format!("{}", result.total_tokens)
                } else {
                    "-".to_string()
                };
                let llm_time = if result.total_llm_secs > 0.0 {
                    format!("{:.1}s", result.total_llm_secs)
                } else {
                    "-".to_string()
                };

                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} |\n",
                    result.program, status, result.attempts, tests, idiom, tokens, llm_time
                ));
            }
            md.push('\n');
        }

        // Failed programs details
        let failed: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.status == ProgramStatus::Failed)
            .collect();

        if !failed.is_empty() {
            md.push_str("## Failed Programs Details\n\n");
            for result in failed {
                md.push_str(&format!("### {}/{}\n", result.collection, result.program));
                if let Some(reason) = &result.skip_reason {
                    md.push_str(&format!("**Reason:** {}\n\n", reason));
                }
                if let Some(build_err) = &result.last_build_error {
                    md.push_str("**Last Build Error:**\n```\n");
                    // Truncate long errors to keep report readable
                    let truncated: String = build_err.lines().take(30).collect::<Vec<_>>().join("\n");
                    md.push_str(&truncated);
                    md.push_str("\n```\n\n");
                }
                if let Some(tests) = &result.test_vectors {
                    if !tests.failures.is_empty() {
                        md.push_str("**Test Failures:**\n```\n");
                        for failure in tests.failures.iter().take(5) {
                            md.push_str(failure);
                            md.push('\n');
                        }
                        md.push_str("```\n\n");
                    }
                }
            }
        }

        // Skipped programs
        let skipped: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.status == ProgramStatus::Skipped)
            .collect();

        if !skipped.is_empty() {
            md.push_str("## Skipped Programs\n\n");
            md.push_str("| Program | Collection | Reason |\n");
            md.push_str("|---------|------------|--------|\n");
            for result in skipped {
                let reason = result
                    .skip_reason
                    .as_deref()
                    .unwrap_or("Unknown");
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    result.program, result.collection, reason
                ));
            }
            md.push('\n');
        }

        md
    }
}

impl Default for TranslationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_result_serialization() {
        let result = TranslationResult {
            program: "test_lib".to_string(),
            collection: "B02_organic".to_string(),
            status: ProgramStatus::Success,
            skip_reason: None,
            attempts: 2,
            test_vectors: Some(TestVectorResults {
                total: 10,
                passed: 10,
                failed: 0,
                failures: vec![],
            }),
            idiomaticity: Some(IdiomaticityMetrics {
                score: 75.0,
                unsafe_blocks: 3,
                raw_pointers: 5,
                clippy_warnings: 2,
            }),
            design_patterns: None,
            last_build_error: None,
            total_tokens: 0,
            total_llm_secs: 0.0,
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        assert!(json.contains("\"status\": \"success\""));
        assert!(json.contains("\"score\": 75.0"));

        // Ensure skip_reason is not included when None
        assert!(!json.contains("skip_reason"));
    }

    #[test]
    fn test_skipped_result_serialization() {
        let result = TranslationResult {
            program: "big_lib".to_string(),
            collection: "B02_organic".to_string(),
            status: ProgramStatus::Skipped,
            skip_reason: Some("File exceeds 1000 lines (1523)".to_string()),
            attempts: 0,
            test_vectors: None,
            idiomaticity: None,
            design_patterns: None,
            last_build_error: None,
            total_tokens: 0,
            total_llm_secs: 0.0,
        };

        let json = serde_json::to_string_pretty(&result).unwrap();
        assert!(json.contains("\"status\": \"skipped\""));
        assert!(json.contains("1523"));
    }

    #[test]
    fn test_report_to_markdown() {
        let report = TranslationReport {
            total_programs: 3,
            success_count: 2,
            failed_count: 1,
            skipped_count: 0,
            run_dir: None,
            results: vec![
                TranslationResult {
                    program: "lib_a".to_string(),
                    collection: "test".to_string(),
                    status: ProgramStatus::Success,
                    skip_reason: None,
                    attempts: 1,
                    test_vectors: Some(TestVectorResults {
                        total: 5,
                        passed: 5,
                        failed: 0,
                        failures: vec![],
                    }),
                    idiomaticity: Some(IdiomaticityMetrics {
                        score: 80.0,
                        unsafe_blocks: 2,
                        raw_pointers: 3,
                        clippy_warnings: 1,
                    }),
                    design_patterns: None,
                    last_build_error: None,
                    total_tokens: 1500,
                    total_llm_secs: 3.2,
                },
                TranslationResult {
                    program: "lib_b".to_string(),
                    collection: "test".to_string(),
                    status: ProgramStatus::Failed,
                    skip_reason: Some("Tests failed after 5 attempts".to_string()),
                    attempts: 5,
                    test_vectors: None,
                    idiomaticity: None,
                    design_patterns: None,
                    last_build_error: None,
                    total_tokens: 800,
                    total_llm_secs: 2.1,
                },
            ],
        };

        let md = report.to_markdown();
        assert!(md.contains("# Translation Report"));
        assert!(md.contains("Success Rate"));
        assert!(md.contains("lib_a"));
        assert!(md.contains("lib_b"));
    }
}
