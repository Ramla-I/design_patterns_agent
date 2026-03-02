use super::ProgramType;
use super::report::TestVectorResults;

/// Formats error feedback for LLM retry attempts
pub struct FeedbackFormatter;

impl FeedbackFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format a build error for feedback
    pub fn format_build_error(&self, error: &anyhow::Error) -> String {
        self.format_build_error_for(error, &ProgramType::Library)
    }

    /// Format a build error for feedback, with program-type-aware hints
    pub fn format_build_error_for(&self, error: &anyhow::Error, program_type: &ProgramType) -> String {
        let error_str = error.to_string();

        let hints = match program_type {
            ProgramType::Library => r#"## Common Issues to Check:
1. Missing or incorrect imports
2. Type mismatches
3. Lifetime issues
4. Missing #![feature(...)] declarations
5. Incorrect FFI function signatures

Please fix these compilation errors while maintaining FFI compatibility."#,
            ProgramType::Executable => r#"## Common Issues to Check:
1. Missing or incorrect imports
2. Type mismatches
3. Lifetime issues
4. Do not use the `libc` crate — use std equivalents
5. Ensure fn main() exists

Please fix these compilation errors. This is an executable (not a library)."#,
        };

        format!(
            "## Build Error\n\nThe code failed to compile with the following errors:\n\n```\n{}\n```\n\n{}",
            self.truncate_output(&error_str, 2000),
            hints,
        )
    }

    /// Format test failures for feedback
    pub fn format_test_failures(&self, results: &TestVectorResults) -> String {
        self.format_test_failures_for(results, &ProgramType::Library)
    }

    /// Format test failures for feedback, with program-type-aware hints
    pub fn format_test_failures_for(&self, results: &TestVectorResults, program_type: &ProgramType) -> String {
        let failures_text = if results.failures.is_empty() {
            "No detailed failure information available.".to_string()
        } else {
            results.failures.join("\n\n")
        };

        let hints = match program_type {
            ProgramType::Library => r#"## Common Causes:
1. Incorrect algorithm implementation
2. Off-by-one errors
3. Incorrect handling of edge cases
4. Memory layout differences
5. Integer overflow/underflow handling

Please fix the failing tests while keeping FFI signatures unchanged."#,
            ProgramType::Executable => r#"## Common Causes:
1. Incorrect algorithm implementation
2. Off-by-one errors in output
3. Wrong output format (extra/missing whitespace or newlines)
4. Incorrect stdin parsing
5. Wrong exit code

Please fix the failing tests. Match the exact stdout output format."#,
        };

        format!(
            "## Test Failures\n\nThe code compiled but {} of {} tests failed.\n\n### Failure Details:\n```\n{}\n```\n\n{}",
            results.failed,
            results.total,
            self.truncate_output(&failures_text, 2000),
            hints,
        )
    }

    /// Format a test execution error for feedback
    pub fn format_test_error(&self, error: &str) -> String {
        self.format_test_error_for(error, &ProgramType::Library)
    }

    /// Format a test execution error for feedback, with program-type-aware hints
    pub fn format_test_error_for(&self, error: &str, program_type: &ProgramType) -> String {
        let hints = match program_type {
            ProgramType::Library => r#"## Possible Causes:
1. Runtime panic
2. Segmentation fault
3. Stack overflow
4. Missing symbols (check #[no_mangle] exports)
5. Incorrect FFI calling convention

Please ensure all #[no_mangle] pub extern "C" functions are correctly exported."#,
            ProgramType::Executable => r#"## Possible Causes:
1. Runtime panic
2. Segmentation fault
3. Stack overflow
4. Infinite loop (program didn't exit)
5. Missing fn main()

Please ensure the program reads stdin, processes argv, and writes to stdout correctly."#,
        };

        format!(
            "## Test Execution Error\n\nThe tests could not be executed due to an error:\n\n```\n{}\n```\n\n{}",
            self.truncate_output(error, 2000),
            hints,
        )
    }

    /// Format clippy warnings as suggestions for improvement
    pub fn format_clippy_suggestions(&self, warnings: &[String]) -> String {
        if warnings.is_empty() {
            return "No clippy warnings.".to_string();
        }

        let warnings_text = warnings
            .iter()
            .take(10)
            .enumerate()
            .map(|(i, w)| format!("{}. {}", i + 1, w))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"## Clippy Warnings

The following warnings were detected:

{}

{}"#,
            warnings_text,
            if warnings.len() > 10 {
                format!("... and {} more warnings", warnings.len() - 10)
            } else {
                String::new()
            }
        )
    }

    /// Truncate output to prevent excessively long feedback
    fn truncate_output(&self, text: &str, max_chars: usize) -> String {
        if text.len() <= max_chars {
            text.to_string()
        } else {
            let truncated = &text[..max_chars];
            format!("{}...\n[Output truncated]", truncated)
        }
    }
}

impl Default for FeedbackFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_build_error() {
        let formatter = FeedbackFormatter::new();
        let error = anyhow::anyhow!("error[E0425]: cannot find value `x` in this scope");

        let feedback = formatter.format_build_error(&error);

        assert!(feedback.contains("Build Error"));
        assert!(feedback.contains("E0425"));
        assert!(feedback.contains("compilation errors"));
    }

    #[test]
    fn test_format_test_failures() {
        let formatter = FeedbackFormatter::new();
        let results = TestVectorResults {
            total: 10,
            passed: 7,
            failed: 3,
            failures: vec![
                "Test 2: expected [1,2,3], got [1,2,4]".to_string(),
                "Test 5: assertion failed".to_string(),
            ],
        };

        let feedback = formatter.format_test_failures(&results);

        assert!(feedback.contains("Test Failures"));
        assert!(feedback.contains("3 of 10 tests failed"));
        assert!(feedback.contains("Test 2:"));
    }

    #[test]
    fn test_format_test_error() {
        let formatter = FeedbackFormatter::new();
        let error = "thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'";

        let feedback = formatter.format_test_error(error);

        assert!(feedback.contains("Test Execution Error"));
        assert!(feedback.contains("panicked"));
    }

    #[test]
    fn test_truncate_output() {
        let formatter = FeedbackFormatter::new();

        let short = "Hello";
        assert_eq!(formatter.truncate_output(short, 100), short);

        let long = "a".repeat(200);
        let truncated = formatter.truncate_output(&long, 50);
        assert!(truncated.len() < 200);
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn test_format_clippy_suggestions() {
        let formatter = FeedbackFormatter::new();

        let empty = formatter.format_clippy_suggestions(&[]);
        assert!(empty.contains("No clippy warnings"));

        let warnings = vec![
            "warning: unused variable".to_string(),
            "warning: this function could be const".to_string(),
        ];
        let feedback = formatter.format_clippy_suggestions(&warnings);
        assert!(feedback.contains("Clippy Warnings"));
        assert!(feedback.contains("unused variable"));
    }
}
