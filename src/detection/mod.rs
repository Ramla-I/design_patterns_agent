mod state_machine;
mod linear_types;
mod ownership;
mod evidence;
mod invariant_inference;

pub use state_machine::StateMachineDetector;
pub use linear_types::LinearTypeDetector;
pub use ownership::OwnershipDetector;
pub use evidence::EvidenceExtractor;
pub use invariant_inference::InvariantInferenceDetector;

use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::{CodeContext, InterestingItem};
use crate::report::{Evidence, Invariant, InvariantType, Location};

/// Coordinator for all invariant detection strategies
pub struct InvariantDetector {
    invariant_inference: InvariantInferenceDetector,
    state_machine: StateMachineDetector,
    linear_type: LinearTypeDetector,
    ownership: OwnershipDetector,
}

impl InvariantDetector {
    pub fn new() -> Self {
        Self {
            invariant_inference: InvariantInferenceDetector::new(),
            state_machine: StateMachineDetector::new(),
            linear_type: LinearTypeDetector::new(),
            ownership: OwnershipDetector::new(),
        }
    }

    /// Detect invariants in a code context using the appropriate detector
    pub async fn detect(
        &self,
        context: &CodeContext,
        llm_client: &dyn LlmClient,
        next_id: &mut usize,
    ) -> Result<Vec<Invariant>> {
        match &context.item {
            // New: Use invariant inference for structs and impl blocks
            InterestingItem::StructWithImpls { .. } |
            InterestingItem::StandaloneImpl { .. } => {
                self.invariant_inference.detect(context, llm_client, next_id).await
            }
            // Legacy detectors for backwards compatibility
            InterestingItem::TypeStateCandidate { .. } => {
                self.state_machine.detect(context, llm_client, next_id).await
            }
            InterestingItem::LinearTypeCandidate { .. } => {
                self.linear_type.detect(context, llm_client, next_id).await
            }
            InterestingItem::StateTransition { .. } => {
                self.state_machine.detect(context, llm_client, next_id).await
            }
            InterestingItem::Generic { .. } => {
                // For generic items, use invariant inference
                self.invariant_inference.detect(context, llm_client, next_id).await
            }
        }
    }
}

impl Default for InvariantDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let _detector = InvariantDetector::new();
        // Just test that it can be created without panicking
    }
}
