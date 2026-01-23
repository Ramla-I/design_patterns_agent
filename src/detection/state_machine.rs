use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::CodeContext;
use crate::report::{Evidence, Invariant, InvariantType, Location};

use super::evidence::EvidenceExtractor;

pub struct StateMachineDetector;

impl StateMachineDetector {
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

        let system_prompt = r#"You are an expert in Rust programming and design patterns. Your task is to analyze Rust code and identify state machine invariants, particularly typestate patterns.

A typestate pattern uses the type system to enforce state transitions at compile time. Common patterns include:
- Structs with PhantomData<S> where S represents different states
- Methods that consume self and return a different type (state transition)
- Builder patterns with type-level state tracking

For each invariant you find, provide:
1. A concise title
2. A description of the invariant
3. An explanation of how the code enforces it"#;

        let user_prompt = format!(
            r#"Analyze the following Rust code from module `{}` and identify any state machine invariants:

```rust
{}
```

Reason about why this code is interesting: {}

If you find state machine invariants, respond in this format:
INVARIANT FOUND
Title: <title>
Description: <description>
Explanation: <explanation>

If no state machine invariants are found, respond with:
NO INVARIANTS FOUND"#,
            context.module_path,
            code_snippet,
            context.item.reason()
        );

        let request = LlmRequest::new(system_prompt, user_prompt).with_temperature(0.3);
        let response = llm_client.complete(request).await?;

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

        // Simple parsing - look for INVARIANT FOUND markers
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
            invariant_type: InvariantType::StateMachine,
            title,
            description,
            location: Location {
                file_path: self.get_file_path(context),
                line_start: 1,
                line_end: 100, // TODO: Get actual line numbers
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
                // Look for the next field or end of text
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
        let detector = StateMachineDetector::new();
        assert!(std::mem::size_of_val(&detector) == 0); // Zero-sized type
    }

    #[test]
    fn test_extract_field() {
        let detector = StateMachineDetector::new();
        let text = "Title: Typestate Pattern\nDescription: Uses PhantomData";

        let title = detector.extract_field(text, "Title:");
        assert_eq!(title, Some("Typestate Pattern".to_string()));

        let description = detector.extract_field(text, "Description:");
        assert_eq!(description, Some("Uses PhantomData".to_string()));
    }

    #[test]
    fn test_parse_response_no_invariants() {
        let detector = StateMachineDetector::new();
        let response = "NO INVARIANTS FOUND";

        let context = create_test_context();
        let mut next_id = 1;
        let invariants = detector.parse_response(response, &context, &mut next_id).unwrap();

        assert!(invariants.is_empty());
    }

    fn create_test_context() -> CodeContext {
        use crate::navigation::InterestingItem;
        use crate::parser::{SourceLocation, StructDef, Visibility};

        CodeContext {
            item: InterestingItem::TypeStateCandidate {
                struct_def: StructDef {
                    name: "Test".to_string(),
                    generics: "".to_string(),
                    fields: vec![],
                    visibility: Visibility::Public,
                    doc_comment: None,
                    has_phantom_data: true,
                    source_location: SourceLocation {
                        file_path: "test.rs".to_string(),
                        line: 1,
                    },
                },
                impl_blocks: vec![],
            },
            surrounding_code: String::new(),
            module_path: "test".to_string(),
        }
    }
}
