use anyhow::Result;
use super::Report;

pub fn generate_json(report: &Report) -> Result<String> {
    let json = serde_json::to_string_pretty(report)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{Invariant, Location, Evidence, InvariantType};

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

        report.add_invariant(Invariant {
            id: 1,
            invariant_type: InvariantType::StateMachine,
            title: "Test".to_string(),
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
        });

        let json = generate_json(&report).unwrap();

        assert!(json.contains("\"total_invariants\": 1"));
        assert!(json.contains("\"state_machine\""));
        assert!(json.contains("\"Test\""));
        assert!(json.contains("\"test.rs\""));
    }

    #[test]
    fn test_json_deserialize() {
        let mut report = Report::new();

        report.add_invariant(Invariant {
            id: 1,
            invariant_type: InvariantType::LinearType,
            title: "Linear".to_string(),
            description: "Desc".to_string(),
            location: Location {
                file_path: "file.rs".to_string(),
                line_start: 5,
                line_end: 15,
            },
            evidence: Evidence {
                code_snippet: "snippet".to_string(),
                explanation: "exp".to_string(),
            },
        });

        let json = generate_json(&report).unwrap();
        let deserialized: Report = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.summary.total_invariants, 1);
        assert_eq!(deserialized.invariants.len(), 1);
        assert_eq!(deserialized.invariants[0].id, 1);
    }
}
