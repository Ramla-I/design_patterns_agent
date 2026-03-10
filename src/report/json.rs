use anyhow::Result;
use super::Report;

pub fn generate_json(report: &Report) -> Result<String> {
    let json = serde_json::to_string_pretty(report)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{Invariant, Location, Evidence, InvariantType, Confidence};

    fn make_invariant(id: usize, inv_type: InvariantType) -> Invariant {
        Invariant {
            id,
            invariant_type: inv_type,
            title: format!("Test {}", id),
            description: "Description".to_string(),
            location: Location {
                file_path: "test.rs".to_string(),
                line_start: 1,
                line_end: 10,
            },
            evidence: Evidence {
                code_snippet: "code".to_string(),
                explanation: "explanation".to_string(),
            },
            suggested_pattern: "typestate".to_string(),
            confidence: Confidence::Medium,
            entity: String::new(),
        }
    }

    #[test]
    fn test_generate_json_empty() {
        let report = Report::new();
        let json = generate_json(&report).unwrap();

        assert!(json.contains("\"total_invariants\": 0"));
        assert!(json.contains("\"invariants\": []"));
    }

    #[test]
    fn test_generate_json_with_invariant() {
        let mut report = Report::new();
        report.add_invariant(make_invariant(1, InvariantType::TemporalOrdering));

        let json = generate_json(&report).unwrap();

        assert!(json.contains("\"total_invariants\": 1"));
        assert!(json.contains("\"temporal_ordering\""));
        assert!(json.contains("\"Test 1\""));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut report = Report::new();
        report.add_invariant(make_invariant(1, InvariantType::ResourceLifecycle));

        let json = generate_json(&report).unwrap();
        let deserialized: Report = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.summary.total_invariants, 1);
        assert_eq!(deserialized.invariants.len(), 1);
        assert_eq!(deserialized.invariants[0].id, 1);
    }
}
