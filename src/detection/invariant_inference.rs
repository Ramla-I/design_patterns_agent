use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::CodeContext;
use crate::report::{Evidence, Invariant, InvariantType, Location};

use super::evidence::EvidenceExtractor;

/// Detector that infers invariants from non-idiomatic code and suggests design patterns
pub struct InvariantInferenceDetector;

impl InvariantInferenceDetector {
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

        let system_prompt = r#"You are an expert in Rust programming, design patterns, and code analysis. Your task is to analyze Rust code (which may be non-idiomatic, such as C2Rust transpiled code) and:

1. INFER INVARIANTS: Identify implicit invariants that the code assumes but doesn't enforce at compile time. Look for:
   - Ordering requirements (e.g., "init() must be called before use()")
   - State dependencies (e.g., "file must be open before read")
   - Resource lifecycle patterns (e.g., "allocated memory must be freed")
   - Null/validity checks that suggest preconditions
   - Comments or function names that hint at required sequences
   - Parameter patterns that suggest state requirements
   - Error handling that reveals assumptions

2. IDENTIFY EVIDENCE: Point to specific code elements that reveal the invariant:
   - Function names suggesting order (init, open, close, start, end)
   - Comments describing requirements
   - Conditional checks (if ptr.is_null(), if !initialized)
   - Assertion patterns
   - Parameter types and names

3. SUGGEST DESIGN PATTERNS: Recommend Rust design patterns to enforce the invariant at compile time:
   - Typestate pattern (using PhantomData<State> to track state at type level)
   - Builder pattern (for complex initialization sequences)
   - RAII/Drop (for cleanup guarantees)
   - Newtype wrappers (for validity invariants)
   - Session types (for protocol enforcement)
   - Linear types (for must-use semantics)

Focus on finding real, meaningful invariants - not trivial observations. The goal is to help convert unsafe runtime assumptions into compile-time guarantees."#;

        let user_prompt = format!(
            r#"Analyze the following Rust code and infer any implicit invariants that could be enforced with design patterns.

Module: `{}`

```rust
{}
```

Context: {}

For each invariant you find, respond in this EXACT format:

INVARIANT
Name: <short descriptive name>
Description: <what must be true, e.g., "File handle must be opened before any read/write operations">
Evidence: <specific code elements that reveal this - function names, comments, checks, line references>
Pattern: <recommended Rust design pattern to enforce this>
Implementation: <brief sketch of how to apply the pattern>

If you find multiple invariants, repeat the INVARIANT block for each.

If no meaningful invariants are found, respond with:
NO INVARIANTS FOUND
Reason: <why no invariants were detected>"#,
            context.module_path,
            code_snippet,
            context.item.reason()
        );

        let request = LlmRequest::new(system_prompt, user_prompt).with_temperature(0.4);
        let response = llm_client.complete(request).await?;

        let invariants = self.parse_response(&response.content, context, &code_snippet, next_id)?;

        Ok(invariants)
    }

    fn parse_response(
        &self,
        response: &str,
        context: &CodeContext,
        code_snippet: &str,
        next_id: &mut usize,
    ) -> Result<Vec<Invariant>> {
        if response.contains("NO INVARIANTS FOUND") {
            return Ok(vec![]);
        }

        let mut invariants = Vec::new();

        // Parse each INVARIANT block
        for section in response.split("INVARIANT").skip(1) {
            // Skip if this is just the "NO INVARIANTS" message
            if section.trim().starts_with("S FOUND") {
                continue;
            }

            if let Some(invariant) = self.parse_invariant_section(section, context, code_snippet, next_id) {
                invariants.push(invariant);
            }
        }

        Ok(invariants)
    }

    fn parse_invariant_section(
        &self,
        section: &str,
        context: &CodeContext,
        code_snippet: &str,
        next_id: &mut usize,
    ) -> Option<Invariant> {
        let name = self.extract_field(section, "Name:")?;
        let description = self.extract_field(section, "Description:")?;
        let evidence = self.extract_field(section, "Evidence:").unwrap_or_default();
        let pattern = self.extract_field(section, "Pattern:").unwrap_or_default();
        let implementation = self.extract_field(section, "Implementation:").unwrap_or_default();

        // Combine pattern suggestion into the explanation
        let explanation = if !pattern.is_empty() {
            format!(
                "**Evidence:** {}\n\n**Suggested Pattern:** {}\n\n**Implementation:** {}",
                evidence, pattern, implementation
            )
        } else {
            evidence.clone()
        };

        // Determine invariant type based on the suggested pattern
        let invariant_type = self.classify_invariant_type(&pattern, &description);

        let invariant = Invariant {
            id: *next_id,
            invariant_type,
            title: name,
            description,
            location: Location {
                file_path: self.get_file_path(context),
                line_start: 1,
                line_end: code_snippet.lines().count(),
            },
            evidence: Evidence {
                code_snippet: code_snippet.to_string(),
                explanation,
            },
        };

        *next_id += 1;

        Some(invariant)
    }

    fn classify_invariant_type(&self, pattern: &str, description: &str) -> InvariantType {
        let pattern_lower = pattern.to_lowercase();
        let desc_lower = description.to_lowercase();

        if pattern_lower.contains("typestate") ||
           pattern_lower.contains("state machine") ||
           pattern_lower.contains("builder") ||
           desc_lower.contains("state") ||
           desc_lower.contains("sequence") ||
           desc_lower.contains("order") {
            InvariantType::StateMachine
        } else if pattern_lower.contains("linear") ||
                  pattern_lower.contains("must_use") ||
                  pattern_lower.contains("raii") ||
                  pattern_lower.contains("drop") ||
                  desc_lower.contains("cleanup") ||
                  desc_lower.contains("free") ||
                  desc_lower.contains("release") {
            InvariantType::LinearType
        } else {
            InvariantType::Ownership
        }
    }

    fn extract_field(&self, text: &str, field_name: &str) -> Option<String> {
        let start = text.find(field_name)? + field_name.len();
        let remaining = &text[start..];

        // Find the end - either the next field or end of section
        let next_fields = ["Name:", "Description:", "Evidence:", "Pattern:", "Implementation:", "INVARIANT"];
        let mut end = remaining.len();

        for field in &next_fields {
            if let Some(pos) = remaining.find(field) {
                if pos > 0 && pos < end {
                    end = pos;
                }
            }
        }

        let value = remaining[..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    fn get_file_path(&self, context: &CodeContext) -> String {
        match &context.item {
            crate::navigation::InterestingItem::StructWithImpls { struct_def, .. } => {
                struct_def.source_location.file_path.clone()
            }
            crate::navigation::InterestingItem::StandaloneImpl { impl_block } => {
                impl_block.source_location.file_path.clone()
            }
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

impl Default for InvariantInferenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = InvariantInferenceDetector::new();
        assert!(std::mem::size_of_val(&detector) == 0); // Zero-sized type
    }

    #[test]
    fn test_classify_invariant_type() {
        let detector = InvariantInferenceDetector::new();

        assert!(matches!(
            detector.classify_invariant_type("Typestate pattern", "state transition"),
            InvariantType::StateMachine
        ));

        assert!(matches!(
            detector.classify_invariant_type("RAII with Drop", "cleanup resources"),
            InvariantType::LinearType
        ));

        assert!(matches!(
            detector.classify_invariant_type("Newtype wrapper", "validity check"),
            InvariantType::Ownership
        ));
    }

    #[test]
    fn test_extract_field() {
        let detector = InvariantInferenceDetector::new();
        let text = "Name: File Handle State\nDescription: File must be opened first\nPattern: Typestate";

        assert_eq!(
            detector.extract_field(text, "Name:"),
            Some("File Handle State".to_string())
        );
        assert_eq!(
            detector.extract_field(text, "Description:"),
            Some("File must be opened first".to_string())
        );
        assert_eq!(
            detector.extract_field(text, "Pattern:"),
            Some("Typestate".to_string())
        );
    }
}
