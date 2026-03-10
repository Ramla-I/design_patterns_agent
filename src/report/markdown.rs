use anyhow::Result;
use super::{Report, InvariantType};

pub fn generate_markdown(report: &Report) -> Result<String> {
    let mut output = String::new();

    // Title
    output.push_str("# Latent Invariant Analysis Report\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total invariants discovered**: {}\n", report.summary.total_invariants));
    output.push_str(&format!("- **Temporal ordering**: {}\n", report.summary.temporal_ordering_count));
    output.push_str(&format!("- **Resource lifecycle**: {}\n", report.summary.resource_lifecycle_count));
    output.push_str(&format!("- **State machine**: {}\n", report.summary.state_machine_count));
    output.push_str(&format!("- **Precondition**: {}\n", report.summary.precondition_count));
    output.push_str(&format!("- **Protocol**: {}\n", report.summary.protocol_count));
    output.push_str(&format!("- **Modules analyzed**: {}\n\n", report.summary.modules_analyzed));

    // Group invariants by type
    let sections: &[(InvariantType, &str)] = &[
        (InvariantType::TemporalOrdering, "Temporal Ordering Invariants"),
        (InvariantType::ResourceLifecycle, "Resource Lifecycle Invariants"),
        (InvariantType::StateMachine, "State Machine Invariants"),
        (InvariantType::Precondition, "Precondition Invariants"),
        (InvariantType::Protocol, "Protocol Invariants"),
    ];

    for (inv_type, title) in sections {
        let invs: Vec<_> = report.invariants.iter()
            .filter(|i| i.invariant_type == *inv_type)
            .collect();

        if !invs.is_empty() {
            output.push_str(&format!("## {}\n\n", title));
            for inv in invs {
                format_invariant(&mut output, inv)?;
            }
        }
    }

    // Skipped files section
    if !report.parse_failures.is_empty() {
        output.push_str("## Skipped Files\n\n");
        output.push_str(&format!("{} file(s) could not be parsed:\n\n", report.parse_failures.len()));
        for (file_path, error) in &report.parse_failures {
            output.push_str(&format!("- `{}`: {}\n", file_path, error));
        }
        output.push('\n');
    }

    Ok(output)
}

fn format_invariant(output: &mut String, inv: &super::Invariant) -> Result<()> {
    output.push_str(&format!("### {}. {}\n\n", inv.id, inv.title));
    output.push_str(&format!("**Location**: `{}:{}-{}`\n\n",
        inv.location.file_path,
        inv.location.line_start,
        inv.location.line_end));
    output.push_str(&format!("**Confidence**: {}\n\n", inv.confidence_label()));
    output.push_str(&format!("**Suggested Pattern**: {}\n\n", inv.suggested_pattern));
    output.push_str(&format!("**Description**: {}\n\n", inv.description));

    output.push_str("**Evidence**:\n\n");
    output.push_str("```rust\n");
    output.push_str(&inv.evidence.code_snippet);
    output.push_str("\n```\n\n");

    if !inv.evidence.explanation.is_empty() {
        output.push_str(&format!("{}\n\n", inv.evidence.explanation));
    }

    output.push_str("---\n\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{Invariant, Location, Evidence, InvariantType, Confidence};

    fn make_invariant(id: usize, inv_type: InvariantType) -> Invariant {
        Invariant {
            id,
            invariant_type: inv_type,
            title: format!("Test Invariant {}", id),
            description: "Must call init before use".to_string(),
            location: Location {
                file_path: "src/conn.rs".to_string(),
                line_start: 10,
                line_end: 50,
            },
            evidence: Evidence {
                code_snippet: "fn init() {}".to_string(),
                explanation: "Runtime check reveals ordering requirement".to_string(),
            },
            suggested_pattern: "typestate".to_string(),
            confidence: Confidence::High,
            entity: String::new(),
        }
    }

    #[test]
    fn test_generate_empty_report() {
        let report = Report::new();
        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("# Latent Invariant Analysis Report"));
        assert!(markdown.contains("Total invariants discovered**: 0"));
    }

    #[test]
    fn test_generate_report_with_invariant() {
        let mut report = Report::new();
        report.add_invariant(make_invariant(1, InvariantType::TemporalOrdering));

        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("## Temporal Ordering Invariants"));
        assert!(markdown.contains("### 1. Test Invariant 1"));
        assert!(markdown.contains("**Confidence**: high"));
        assert!(markdown.contains("**Suggested Pattern**: typestate"));
    }

    #[test]
    fn test_multiple_invariant_types() {
        let mut report = Report::new();
        report.add_invariant(make_invariant(1, InvariantType::TemporalOrdering));
        report.add_invariant(make_invariant(2, InvariantType::ResourceLifecycle));

        let markdown = generate_markdown(&report).unwrap();

        assert!(markdown.contains("## Temporal Ordering Invariants"));
        assert!(markdown.contains("## Resource Lifecycle Invariants"));
    }
}
