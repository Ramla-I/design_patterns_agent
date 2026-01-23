mod markdown;
mod json;

pub use markdown::generate_markdown;
pub use json::generate_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub summary: Summary,
    pub invariants: Vec<Invariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_invariants: usize,
    pub state_machine_count: usize,
    pub linear_type_count: usize,
    pub ownership_count: usize,
    pub modules_analyzed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub id: usize,
    pub invariant_type: InvariantType,
    pub title: String,
    pub description: String,
    pub location: Location,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantType {
    StateMachine,
    LinearType,
    Ownership,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub code_snippet: String,
    pub explanation: String,
}

impl Report {
    pub fn new() -> Self {
        Self {
            summary: Summary {
                total_invariants: 0,
                state_machine_count: 0,
                linear_type_count: 0,
                ownership_count: 0,
                modules_analyzed: 0,
            },
            invariants: Vec::new(),
        }
    }

    pub fn add_invariant(&mut self, invariant: Invariant) {
        match invariant.invariant_type {
            InvariantType::StateMachine => self.summary.state_machine_count += 1,
            InvariantType::LinearType => self.summary.linear_type_count += 1,
            InvariantType::Ownership => self.summary.ownership_count += 1,
        }
        self.summary.total_invariants += 1;
        self.invariants.push(invariant);
    }
}

impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let report = Report::new();
        assert_eq!(report.summary.total_invariants, 0);
        assert_eq!(report.invariants.len(), 0);
    }

    #[test]
    fn test_add_invariant() {
        let mut report = Report::new();

        let invariant = Invariant {
            id: 1,
            invariant_type: InvariantType::StateMachine,
            title: "Test Invariant".to_string(),
            description: "A test".to_string(),
            location: Location {
                file_path: "test.rs".to_string(),
                line_start: 1,
                line_end: 10,
            },
            evidence: Evidence {
                code_snippet: "code".to_string(),
                explanation: "explanation".to_string(),
            },
        };

        report.add_invariant(invariant);

        assert_eq!(report.summary.total_invariants, 1);
        assert_eq!(report.summary.state_machine_count, 1);
        assert_eq!(report.invariants.len(), 1);
    }

    #[test]
    fn test_multiple_invariant_types() {
        let mut report = Report::new();

        report.add_invariant(Invariant {
            id: 1,
            invariant_type: InvariantType::StateMachine,
            title: "SM".to_string(),
            description: "".to_string(),
            location: Location {
                file_path: "".to_string(),
                line_start: 1,
                line_end: 1,
            },
            evidence: Evidence {
                code_snippet: "".to_string(),
                explanation: "".to_string(),
            },
        });

        report.add_invariant(Invariant {
            id: 2,
            invariant_type: InvariantType::LinearType,
            title: "LT".to_string(),
            description: "".to_string(),
            location: Location {
                file_path: "".to_string(),
                line_start: 1,
                line_end: 1,
            },
            evidence: Evidence {
                code_snippet: "".to_string(),
                explanation: "".to_string(),
            },
        });

        assert_eq!(report.summary.total_invariants, 2);
        assert_eq!(report.summary.state_machine_count, 1);
        assert_eq!(report.summary.linear_type_count, 1);
    }
}
