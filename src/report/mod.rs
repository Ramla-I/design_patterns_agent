mod markdown;
mod json;

pub use markdown::generate_markdown;
pub use json::generate_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub summary: Summary,
    pub invariants: Vec<Invariant>,
    #[serde(default)]
    pub parse_failures: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_invariants: usize,
    pub temporal_ordering_count: usize,
    pub resource_lifecycle_count: usize,
    pub state_machine_count: usize,
    pub precondition_count: usize,
    pub protocol_count: usize,
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
    pub suggested_pattern: String,
    pub confidence: Confidence,
    #[serde(default)]
    pub entity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantType {
    TemporalOrdering,
    ResourceLifecycle,
    StateMachine,
    Precondition,
    Protocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
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

impl Invariant {
    pub fn confidence_label(&self) -> &'static str {
        match self.confidence {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
        }
    }
}

impl Report {
    pub fn new() -> Self {
        Self {
            summary: Summary {
                total_invariants: 0,
                temporal_ordering_count: 0,
                resource_lifecycle_count: 0,
                state_machine_count: 0,
                precondition_count: 0,
                protocol_count: 0,
                modules_analyzed: 0,
            },
            invariants: Vec::new(),
            parse_failures: Vec::new(),
        }
    }

    pub fn add_invariant(&mut self, invariant: Invariant) {
        match invariant.invariant_type {
            InvariantType::TemporalOrdering => self.summary.temporal_ordering_count += 1,
            InvariantType::ResourceLifecycle => self.summary.resource_lifecycle_count += 1,
            InvariantType::StateMachine => self.summary.state_machine_count += 1,
            InvariantType::Precondition => self.summary.precondition_count += 1,
            InvariantType::Protocol => self.summary.protocol_count += 1,
        }
        self.summary.total_invariants += 1;
        self.invariants.push(invariant);
    }
}

pub fn deduplicate(invariants: Vec<Invariant>) -> Vec<Invariant> {
    use std::collections::HashMap;

    // Phase 1: exact key dedup (entity + title + type)
    let mut groups: HashMap<String, Vec<Invariant>> = HashMap::new();
    for inv in invariants {
        let key = normalize_dedup_key(&inv);
        groups.entry(key).or_default().push(inv);
    }
    let mut deduped: Vec<Invariant> = groups.into_values().map(|mut group| {
        group.sort_by(|a, b| {
            confidence_rank(&b.confidence).cmp(&confidence_rank(&a.confidence))
                .then_with(|| b.evidence.code_snippet.len().cmp(&a.evidence.code_snippet.len()))
        });
        group.remove(0)
    }).collect();

    // Phase 2: fuzzy dedup — within same entity+type, merge if titles share >60% words
    fuzzy_dedup(&mut deduped);

    deduped
}

fn normalize_dedup_key(inv: &Invariant) -> String {
    let entity = inv.entity.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    let title = inv.title.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{}|{}|{:?}", entity, title, inv.invariant_type)
}

/// Fuzzy dedup: for invariants with the same entity and type, merge if their
/// titles have >60% word overlap (e.g., "Waker::Empty state" vs "Waker::Empty (idle) state").
/// Keeps the one with highest confidence / longest evidence.
fn fuzzy_dedup(invariants: &mut Vec<Invariant>) {
    use std::collections::HashSet;

    let mut to_remove: HashSet<usize> = HashSet::new();

    for i in 0..invariants.len() {
        if to_remove.contains(&i) { continue; }
        for j in (i + 1)..invariants.len() {
            if to_remove.contains(&j) { continue; }

            let a = &invariants[i];
            let b = &invariants[j];

            // Must share entity and type
            if a.entity.to_lowercase() != b.entity.to_lowercase() { continue; }
            if a.invariant_type != b.invariant_type { continue; }

            if title_word_overlap(&a.title, &b.title) > 0.6 {
                // Keep the better one (higher confidence, then longer evidence)
                let a_rank = confidence_rank(&a.confidence);
                let b_rank = confidence_rank(&b.confidence);
                if b_rank > a_rank || (b_rank == a_rank && b.evidence.code_snippet.len() > a.evidence.code_snippet.len()) {
                    to_remove.insert(i);
                    break; // i is removed, stop comparing it
                } else {
                    to_remove.insert(j);
                }
            }
        }
    }

    // Remove in reverse order to preserve indices
    let mut remove_sorted: Vec<usize> = to_remove.into_iter().collect();
    remove_sorted.sort_unstable_by(|a, b| b.cmp(a));
    for idx in remove_sorted {
        invariants.remove(idx);
    }
}

fn title_word_overlap(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let words_a: std::collections::HashSet<&str> = a_lower.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b_lower.split_whitespace().collect();
    let union = words_a.union(&words_b).count();
    if union == 0 { return 0.0; }
    let intersection = words_a.intersection(&words_b).count();
    intersection as f64 / union as f64
}

fn confidence_rank(c: &Confidence) -> u8 {
    match c {
        Confidence::High => 3,
        Confidence::Medium => 2,
        Confidence::Low => 1,
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

    fn make_test_invariant(id: usize, inv_type: InvariantType) -> Invariant {
        Invariant {
            id,
            invariant_type: inv_type,
            title: format!("Test {}", id),
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
            suggested_pattern: "typestate".to_string(),
            confidence: Confidence::Medium,
            entity: String::new(),
        }
    }

    #[test]
    fn test_report_creation() {
        let report = Report::new();
        assert_eq!(report.summary.total_invariants, 0);
        assert_eq!(report.invariants.len(), 0);
    }

    #[test]
    fn test_add_invariant() {
        let mut report = Report::new();
        report.add_invariant(make_test_invariant(1, InvariantType::StateMachine));

        assert_eq!(report.summary.total_invariants, 1);
        assert_eq!(report.summary.state_machine_count, 1);
        assert_eq!(report.invariants.len(), 1);
    }

    #[test]
    fn test_multiple_invariant_types() {
        let mut report = Report::new();
        report.add_invariant(make_test_invariant(1, InvariantType::TemporalOrdering));
        report.add_invariant(make_test_invariant(2, InvariantType::ResourceLifecycle));

        assert_eq!(report.summary.total_invariants, 2);
        assert_eq!(report.summary.temporal_ordering_count, 1);
        assert_eq!(report.summary.resource_lifecycle_count, 1);
    }

    #[test]
    fn test_dedup_same_entity_title() {
        let mut inv1 = make_test_invariant(1, InvariantType::StateMachine);
        inv1.entity = "Foo".to_string();
        inv1.title = "Foo::Open state".to_string();
        inv1.confidence = Confidence::Medium;

        let mut inv2 = make_test_invariant(2, InvariantType::StateMachine);
        inv2.entity = "Foo".to_string();
        inv2.title = "Foo::Open state".to_string();
        inv2.confidence = Confidence::High;

        let result = deduplicate(vec![inv1, inv2]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence, Confidence::High);
    }

    #[test]
    fn test_dedup_different_entities() {
        let mut inv1 = make_test_invariant(1, InvariantType::StateMachine);
        inv1.entity = "Foo".to_string();
        inv1.title = "Foo::Open state".to_string();

        let mut inv2 = make_test_invariant(2, InvariantType::StateMachine);
        inv2.entity = "Bar".to_string();
        inv2.title = "Bar::Open state".to_string();

        let result = deduplicate(vec![inv1, inv2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_dedup_confidence_tiebreak() {
        let mut inv1 = make_test_invariant(1, InvariantType::StateMachine);
        inv1.entity = "Foo".to_string();
        inv1.title = "Foo state".to_string();
        inv1.confidence = Confidence::High;
        inv1.evidence = Evidence {
            code_snippet: "short".to_string(),
            explanation: "exp".to_string(),
        };

        let mut inv2 = make_test_invariant(2, InvariantType::StateMachine);
        inv2.entity = "Foo".to_string();
        inv2.title = "Foo state".to_string();
        inv2.confidence = Confidence::High;
        inv2.evidence = Evidence {
            code_snippet: "a much longer code snippet that has more evidence".to_string(),
            explanation: "exp".to_string(),
        };

        let result = deduplicate(vec![inv1, inv2]);
        assert_eq!(result.len(), 1);
        assert!(result[0].evidence.code_snippet.contains("much longer"));
    }

    #[test]
    fn test_fuzzy_dedup_similar_titles() {
        let mut inv1 = make_test_invariant(1, InvariantType::StateMachine);
        inv1.entity = "Waker".to_string();
        inv1.title = "Waker::Empty state".to_string();
        inv1.confidence = Confidence::Low;

        let mut inv2 = make_test_invariant(2, InvariantType::StateMachine);
        inv2.entity = "Waker".to_string();
        inv2.title = "Waker::Empty (idle) state".to_string();
        inv2.confidence = Confidence::Medium;

        let result = deduplicate(vec![inv1, inv2]);
        assert_eq!(result.len(), 1);
        // Should keep the higher-confidence one
        assert_eq!(result[0].confidence, Confidence::Medium);
    }

    #[test]
    fn test_fuzzy_dedup_different_titles() {
        let mut inv1 = make_test_invariant(1, InvariantType::StateMachine);
        inv1.entity = "Channel".to_string();
        inv1.title = "Channel::Connected state".to_string();

        let mut inv2 = make_test_invariant(2, InvariantType::Protocol);
        inv2.entity = "Channel".to_string();
        inv2.title = "Channel send reservation protocol".to_string();

        // Different types → should NOT be merged
        let result = deduplicate(vec![inv1, inv2]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_title_word_overlap() {
        assert!(title_word_overlap("Waker::Empty state", "Waker::Empty (idle) state") > 0.6);
        assert!(title_word_overlap("Channel connected", "Receiver disconnected") < 0.5);
        assert!((title_word_overlap("exact match", "exact match") - 1.0).abs() < 0.01);
        assert!(title_word_overlap("", "") == 0.0);
    }
}
