use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::CodeContext;
use crate::report::{Evidence, Invariant, InvariantType, Location};

use super::evidence::EvidenceExtractor;

pub struct LinearTypeDetector;

impl LinearTypeDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn detect(
        &self,
        context: &CodeContext,
        llm_client: &dyn LlmClient,
        next_id: &mut usize,
    ) -> Result<Vec<Invariant>> {
        let code_snippet = EvidenceExtractor::extract_code_snippet(context);

        let system_prompt = r#"You are an expert in Rust programming and design patterns. Your task is to analyze Rust code and identify linear type invariants and capability patterns.

Linear types and capabilities ensure certain operations happen in a specific order, often using:
- Types that must be consumed in order (no cloning, must use)
- Drop guards that enforce cleanup
- Capability tokens passed between functions
- #[must_use] annotations

For each invariant you find, provide:
1. A concise title
2. A description of the invariant
3. An explanation of the ordering requirement"#;

        let user_prompt = format!(
            r#"Analyze the following Rust code from module `{}` and identify any linear type or ordering invariants:

```rust
{}
```

Reason about why this code is interesting: {}

If you find linear type invariants, respond in this format:
INVARIANT FOUND
Title: <title>
Description: <description>
Explanation: <explanation>

If no linear type invariants are found, respond with:
NO INVARIANTS FOUND"#,
            context.module_path,
            code_snippet,
            context.item.reason()
        );

        let request = LlmRequest::new(system_prompt, user_prompt).with_temperature(0.3);
        let response = llm_client.complete(request).await?;

        // Reuse the same parsing logic as state machine detector
        let invariants = self.parse_response(&response.content, context, next_id)?;

        Ok(invariants)
    }

    fn parse_response(
        &self,
        response: &str,
        context: &CodeContext,
        next_id: &mut usize,
    ) -> Result<Vec<Invariant>> {
        if response.contains("NO INVARIANTS FOUND") {
            return Ok(vec![]);
        }

        let mut invariants = Vec::new();

        for section in response.split("INVARIANT FOUND").skip(1) {
            if let Some(invariant) = self.parse_invariant_section(section, context, next_id) {
                invariants.push(invariant);
            }
        }

        Ok(invariants)
    }

    fn parse_invariant_section(
        &self,
        section: &str,
        context: &CodeContext,
        next_id: &mut usize,
    ) -> Option<Invariant> {
        let title = self.extract_field(section, "Title:")?;
        let description = self.extract_field(section, "Description:")?;
        let explanation = self.extract_field(section, "Explanation:").unwrap_or_default();

        let invariant = Invariant {
            id: *next_id,
            invariant_type: InvariantType::LinearType,
            title,
            description,
            location: Location {
                file_path: self.get_file_path(context),
                line_start: 1,
                line_end: 100,
            },
            evidence: Evidence {
                code_snippet: EvidenceExtractor::extract_code_snippet(context),
                explanation,
            },
        };

        *next_id += 1;

        Some(invariant)
    }

    fn extract_field(&self, text: &str, field_name: &str) -> Option<String> {
        let start = text.find(field_name)? + field_name.len();
        let remaining = &text[start..];
        let end = remaining
            .find('\n')
            .map(|i| {
                let next_fields = ["Title:", "Description:", "Explanation:", "INVARIANT"];
                let mut min_pos = i;
                for field in &next_fields {
                    if let Some(pos) = remaining.find(field) {
                        if pos > 0 && pos < min_pos {
                            min_pos = pos;
                        }
                    }
                }
                min_pos
            })
            .unwrap_or(remaining.len());

        Some(remaining[..end].trim().to_string())
    }

    fn get_file_path(&self, context: &CodeContext) -> String {
        match &context.item {
            crate::navigation::InterestingItem::TypeStateCandidate { struct_def, .. } => {
                struct_def.source_location.file_path.clone()
            }
            crate::navigation::InterestingItem::LinearTypeCandidate { struct_def, .. } => {
                struct_def.source_location.file_path.clone()
            }
            crate::navigation::InterestingItem::StateTransition { impl_block } => {
                impl_block.source_location.file_path.clone()
            }
            crate::navigation::InterestingItem::Generic { .. } => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = LinearTypeDetector::new();
        assert!(std::mem::size_of_val(&detector) == 0);
    }
}
