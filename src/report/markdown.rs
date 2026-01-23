use anyhow::Result;
use super::{Report, InvariantType};

pub fn generate_markdown(report: &Report) -> Result<String> {
    let mut output = String::new();

    // Title
    output.push_str("# Invariant Analysis Report\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total invariants discovered**: {}\n", report.summary.total_invariants));
    output.push_str(&format!("- **State machine invariants**: {}\n", report.summary.state_machine_count));
    output.push_str(&format!("- **Linear type invariants**: {}\n", report.summary.linear_type_count));
    output.push_str(&format!("- **Ownership invariants**: {}\n", report.summary.ownership_count));
    output.push_str(&format!("- **Modules analyzed**: {}\n\n", report.summary.modules_analyzed));

    // Group invariants by type
    let state_machine_invs: Vec<_> = report.invariants.iter()
        .filter(|i| i.invariant_type == InvariantType::StateMachine)
        .collect();
    let linear_type_invs: Vec<_> = report.invariants.iter()
        .filter(|i| i.invariant_type == InvariantType::LinearType)
        .collect();
    let ownership_invs: Vec<_> = report.invariants.iter()
        .filter(|i| i.invariant_type == InvariantType::Ownership)
        .collect();

    // State Machine Invariants
    if !state_machine_invs.is_empty() {
        output.push_str("## State Machine Invariants\n\n");
        for inv in state_machine_invs {
            format_invariant(&mut output, inv)?;
        }
    }

    // Linear Type Invariants
    if !linear_type_invs.is_empty() {
        output.push_str("## Linear Type Invariants\n\n");
        for inv in linear_type_invs {
            format_invariant(&mut output, inv)?;
        }
    }

    // Ownership Invariants
    if !ownership_invs.is_empty() {
        output.push_str("## Ownership Invariants\n\n");
        for inv in ownership_invs {
            format_invariant(&mut output, inv)?;
        }
    }

    Ok(output)
}

fn format_invariant(output: &mut String, inv: &super::Invariant) -> Result<()> {
    output.push_str(&format!("### {}. {}\n\n", inv.id, inv.title));
    output.push_str(&format!("**Location**: `{}:{}-{}`\n\n",
        inv.location.file_path,
        inv.location.line_start,
        inv.location.line_end));
    output.push_str(&format!("**Description**: {}\n\n", inv.description));

    output.push_str("**Evidence**:\n\n");
    output.push_str("```rust\n");
    output.push_str(&inv.evidence.code_snippet);
    output.push_str("\n```\n\n");

    if !inv.evidence.explanation.is_empty() {
        output.push_str(&format!("**Explanation**: {}\n\n", inv.evidence.explanation));
    }

    output.push_str("---\n\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{Invariant, Location, Evidence, InvariantType};

    #[test]
    fn test_generate_empty_report() {
        let report = Report::new();
        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("# Invariant Analysis Report"));
        assert!(markdown.contains("Total invariants discovered**: 0"));
    }

    #[test]
    fn test_generate_report_with_invariant() {
        let mut report = Report::new();

        report.add_invariant(Invariant {
            id: 1,
            invariant_type: InvariantType::StateMachine,
            title: "FileHandle Typestate".to_string(),
            description: "FileHandle uses typestate pattern".to_string(),
            location: Location {
                file_path: "src/file.rs".to_string(),
                line_start: 45,
                line_end: 78,
            },
            evidence: Evidence {
                code_snippet: "pub struct FileHandle<S> { ... }".to_string(),
                explanation: "Files must be opened before reading".to_string(),
            },
        });

        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("## State Machine Invariants"));
        assert!(markdown.contains("### 1. FileHandle Typestate"));
        assert!(markdown.contains("`src/file.rs:45-78`"));
        assert!(markdown.contains("```rust"));
        assert!(markdown.contains("FileHandle uses typestate pattern"));
    }

    #[test]
    fn test_multiple_invariant_types() {
        let mut report = Report::new();

        report.add_invariant(Invariant {
            id: 1,
            invariant_type: InvariantType::StateMachine,
            title: "SM".to_string(),
            description: "State machine".to_string(),
            location: Location {
                file_path: "test.rs".to_string(),
                line_start: 1,
                line_end: 10,
            },
            evidence: Evidence {
                code_snippet: "code".to_string(),
                explanation: "".to_string(),
            },
        });

        report.add_invariant(Invariant {
            id: 2,
            invariant_type: InvariantType::LinearType,
            title: "LT".to_string(),
            description: "Linear type".to_string(),
            location: Location {
                file_path: "test.rs".to_string(),
                line_start: 20,
                line_end: 30,
            },
            evidence: Evidence {
                code_snippet: "code2".to_string(),
                explanation: "".to_string(),
            },
        });

        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("## State Machine Invariants"));
        assert!(markdown.contains("## Linear Type Invariants"));
        assert!(markdown.contains("### 1. SM"));
        assert!(markdown.contains("### 2. LT"));
    }
}
