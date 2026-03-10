mod evidence;
mod invariant_inference;
pub mod validation;

pub use invariant_inference::InvariantInferenceDetector;
pub use validation::InvariantValidator;

use std::sync::atomic::AtomicUsize;

use anyhow::Result;

use crate::llm::LlmClient;
use crate::navigation::AnalysisChunk;
use crate::report::Invariant;

/// Coordinator for invariant detection — routes all analysis through
/// the latent invariant inference detector.
pub struct InvariantDetector {
    inference: InvariantInferenceDetector,
}

impl InvariantDetector {
    pub fn new() -> Self {
        Self {
            inference: InvariantInferenceDetector::new(),
        }
    }

    /// Detect latent invariants in a module analysis chunk
    pub async fn detect(
        &self,
        chunk: &AnalysisChunk,
        llm_client: &dyn LlmClient,
        next_id: &AtomicUsize,
    ) -> Result<Vec<Invariant>> {
        self.inference.detect(chunk, llm_client, next_id).await
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
    }
}
