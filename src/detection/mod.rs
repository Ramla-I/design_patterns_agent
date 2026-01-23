mod state_machine;
mod linear_types;
mod ownership;
mod evidence;

pub use state_machine::StateMachineDetector;
pub use linear_types::LinearTypeDetector;
pub use ownership::OwnershipDetector;
pub use evidence::EvidenceExtractor;

use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::{CodeContext, InterestingItem};
use crate::report::{Evidence, Invariant, InvariantType, Location};

/// Coordinator for all invariant detection strategies
pub struct InvariantDetector {
    state_machine: StateMachineDetector,
    linear_type: LinearTypeDetector,
    ownership: OwnershipDetector,
}

impl InvariantDetector {
    pub fn new() -> Self {
        Self {
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
                // For generic items, try all detectors
                let mut results = Vec::new();
                results.extend(self.state_machine.detect(context, llm_client, next_id).await?);
                results.extend(self.linear_type.detect(context, llm_client, next_id).await?);
                results.extend(self.ownership.detect(context, llm_client, next_id).await?);
                Ok(results)
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
