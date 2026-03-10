use anyhow::Result;

use crate::llm::{LlmClient, LlmRequest};
use crate::report::{Confidence, Invariant};

const VALIDATION_PROMPT: &str = r#"You are a critical reviewer of LLM-generated code analysis. Your job is to verify whether a claimed invariant is:
1. **Actually present** in the code (not hallucinated)
2. **Actually latent** (enforced at runtime, not already a compile-time guarantee)
3. **Correctly classified** (the kind, confidence, and description match what the code shows)

Be skeptical. Many LLM-generated invariants are:
- Duplicates of compile-time guarantees (enum exhaustiveness, type safety)
- Hallucinated methods or fields not present in the code
- Overly speculative with low evidence

Respond with JSON:
```json
{
  "valid": true/false,
  "reason": "brief explanation of why valid or invalid",
  "adjusted_confidence": "high" | "medium" | "low"
}
```"#;

pub struct InvariantValidator;

pub struct ValidationResult {
    pub valid: bool,
    pub reason: String,
    pub adjusted_confidence: Confidence,
}

impl InvariantValidator {
    pub async fn validate(
        invariant: &Invariant,
        llm_client: &dyn LlmClient,
    ) -> Result<ValidationResult> {
        let user_prompt = format!(
            r#"Review the following claimed invariant:

**Title:** {title}
**Entity:** {entity}
**Type:** {inv_type:?}
**Description:** {description}
**Confidence:** {confidence}

**Code snippet:**
```rust
{snippet}
```

**Evidence/Explanation:**
{explanation}

Is this invariant valid? Is it truly latent (runtime-enforced, not compile-time)? Is the confidence level appropriate?"#,
            title = invariant.title,
            entity = invariant.entity,
            inv_type = invariant.invariant_type,
            description = invariant.description,
            confidence = invariant.confidence_label(),
            snippet = truncate_snippet(&invariant.evidence.code_snippet, 2000),
            explanation = invariant.evidence.explanation,
        );

        let request = LlmRequest::new(VALIDATION_PROMPT, user_prompt).with_temperature(0.1);
        let response = llm_client.complete(request).await?;

        parse_validation_response(&response.content)
    }
}

fn truncate_snippet(snippet: &str, max_chars: usize) -> String {
    if snippet.len() <= max_chars {
        snippet.to_string()
    } else {
        let mut end = max_chars;
        while !snippet.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &snippet[..end])
    }
}

fn parse_validation_response(response: &str) -> Result<ValidationResult> {
    // Try to extract JSON from response
    let json_str = extract_json_object(response)
        .unwrap_or_else(|| response.to_string());

    #[derive(serde::Deserialize)]
    struct RawResult {
        #[serde(default)]
        valid: bool,
        #[serde(default)]
        reason: String,
        #[serde(default = "default_confidence")]
        adjusted_confidence: String,
    }

    fn default_confidence() -> String {
        "medium".to_string()
    }

    match serde_json::from_str::<RawResult>(&json_str) {
        Ok(raw) => Ok(ValidationResult {
            valid: raw.valid,
            reason: raw.reason,
            adjusted_confidence: parse_confidence(&raw.adjusted_confidence),
        }),
        Err(_) => {
            // If JSON parsing fails, assume valid with medium confidence
            Ok(ValidationResult {
                valid: true,
                reason: "Could not parse validation response".to_string(),
                adjusted_confidence: Confidence::Medium,
            })
        }
    }
}

fn extract_json_object(text: &str) -> Option<String> {
    let text = text.trim();

    // Remove markdown fences if present
    let text = if text.contains("```json") {
        text.split("```json").nth(1)?.split("```").next()?
    } else if text.contains("```") {
        text.split("```").nth(1)?.split("```").next().unwrap_or(text)
    } else {
        text
    };

    let start = text.find('{')?;
    let end = text.rfind('}')? + 1;
    if start >= end {
        return None;
    }

    Some(text[start..end].to_string())
}

fn parse_confidence(confidence: &str) -> Confidence {
    match confidence.to_lowercase().as_str() {
        "high" => Confidence::High,
        "low" => Confidence::Low,
        _ => Confidence::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_validation_response_valid() {
        let response = r#"```json
{
  "valid": true,
  "reason": "The invariant is well-supported by evidence",
  "adjusted_confidence": "high"
}
```"#;
        let result = parse_validation_response(response).unwrap();
        assert!(result.valid);
        assert_eq!(result.adjusted_confidence, Confidence::High);
    }

    #[test]
    fn test_parse_validation_response_invalid() {
        let response = r#"{"valid": false, "reason": "This is a compile-time guarantee", "adjusted_confidence": "low"}"#;
        let result = parse_validation_response(response).unwrap();
        assert!(!result.valid);
        assert!(result.reason.contains("compile-time"));
        assert_eq!(result.adjusted_confidence, Confidence::Low);
    }

    #[test]
    fn test_parse_validation_response_fallback() {
        let response = "This is not JSON at all";
        let result = parse_validation_response(response).unwrap();
        assert!(result.valid); // defaults to valid on parse failure
        assert_eq!(result.adjusted_confidence, Confidence::Medium);
    }

    #[test]
    fn test_truncate_snippet() {
        let short = "fn foo() {}";
        assert_eq!(truncate_snippet(short, 100), short);

        let long = "x".repeat(100);
        let truncated = truncate_snippet(&long, 50);
        assert!(truncated.len() <= 54); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }
}
