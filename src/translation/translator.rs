use anyhow::Result;
use std::time::Instant;

use crate::llm::{LlmClient, LlmRequest};

/// Result of a single LLM translation call
pub struct TranslationOutput {
    pub code: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub duration_secs: f64,
}

/// LLM-based translator for converting C2Rust code to idiomatic Rust
pub struct Translator;

impl Translator {
    pub fn new() -> Self {
        Self
    }

    /// Translate C2Rust code to idiomatic Rust
    pub async fn translate(
        &self,
        source_code: &str,
        feedback: Option<&str>,
        llm_client: &dyn LlmClient,
    ) -> Result<TranslationOutput> {
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(source_code, feedback);

        let request = LlmRequest::new(system_prompt, user_prompt)
            .with_temperature(0.2);

        let start = Instant::now();
        let response = llm_client.complete(request).await?;
        let duration_secs = start.elapsed().as_secs_f64();

        let code = self.extract_code(&response.content)?;

        Ok(TranslationOutput {
            code,
            prompt_tokens: response.prompt_tokens,
            completion_tokens: response.completion_tokens,
            total_tokens: response.tokens_used,
            duration_secs,
        })
    }

    fn build_system_prompt(&self) -> String {
        r#"You convert C2Rust code to idiomatic Rust while preserving FFI compatibility.

RULES (violating any = broken build):
- Keep ALL `#[no_mangle] pub unsafe extern "C"` function signatures EXACTLY: same name, params (including `mut`), types, return type
- Keep ALL `#[no_mangle] pub static` declarations unchanged
- Keep ALL `#![feature(...)]` declarations
- Keep FFI-boundary types C-compatible (repr(C), primitives, raw pointers)
- Keep module structure (`pub mod src { pub mod lib { ... } }`)
- Preserve the algorithm/logic; only change implementation style

TRANSFORMS (internal code only):
- malloc/calloc/free → Vec, Box, or Rust allocation
- Raw pointer arithmetic → slice indexing where safe
- C-style loops → iterators or for-range
- Redundant casts → direct literals (e.g. `8u8`)
- `0 as *const T` → `std::ptr::null()`/`null_mut()`
- Use const, meaningful names, Option/Result internally

Be conservative. When in doubt, preserve original code.

Output ONLY the code in a ```rust block, no explanations."#.to_string()
    }

    fn build_user_prompt(&self, source_code: &str, feedback: Option<&str>) -> String {
        let mut prompt = format!("Transform this C2Rust code to idiomatic Rust:\n\n```rust\n{}\n```", source_code);

        if let Some(feedback) = feedback {
            prompt.push_str(&format!("\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix these issues while keeping FFI compatibility.", feedback));
        }

        prompt
    }

    /// Extract Rust code from LLM response
    fn extract_code(&self, response: &str) -> Result<String> {
        // Look for code block
        if let Some(start) = response.find("```rust") {
            let code_start = start + 7; // Skip "```rust"
            if let Some(end) = response[code_start..].find("```") {
                let code = response[code_start..code_start + end].trim();
                return Ok(code.to_string());
            }
        }

        // Try without language specifier
        if let Some(start) = response.find("```") {
            let code_start = start + 3;
            // Skip to next line if there's a language identifier
            let code_start = if let Some(newline) = response[code_start..].find('\n') {
                code_start + newline + 1
            } else {
                code_start
            };
            if let Some(end) = response[code_start..].find("```") {
                let code = response[code_start..code_start + end].trim();
                return Ok(code.to_string());
            }
        }

        // If no code block found, assume the entire response is code
        // (some models might not wrap in code blocks)
        Ok(response.trim().to_string())
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_with_rust_block() {
        let translator = Translator::new();
        let response = r#"Here's the code:

```rust
fn main() {
    println!("Hello");
}
```

That's it!"#;

        let code = translator.extract_code(response).unwrap();
        assert!(code.contains("fn main()"));
        assert!(code.contains("println!"));
    }

    #[test]
    fn test_extract_code_without_block() {
        let translator = Translator::new();
        let response = r#"fn main() {
    println!("Hello");
}"#;

        let code = translator.extract_code(response).unwrap();
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn test_system_prompt_contains_key_requirements() {
        let translator = Translator::new();
        let prompt = translator.build_system_prompt();

        assert!(prompt.contains("#[no_mangle]"));
        assert!(prompt.contains("extern \"C\""));
        assert!(prompt.contains("FFI"));
        assert!(prompt.contains("Nightly") || prompt.contains("feature"));
    }

    #[test]
    fn test_user_prompt_with_feedback() {
        let translator = Translator::new();
        let code = "fn test() {}";
        let feedback = "Build error: missing semicolon";

        let prompt = translator.build_user_prompt(code, Some(feedback));

        assert!(prompt.contains(code));
        assert!(prompt.contains(feedback));
        assert!(prompt.contains("PREVIOUS ATTEMPT FAILED"));
    }
}
