use anyhow::Result;
use std::time::Instant;

use crate::llm::{LlmClient, LlmRequest};
use crate::translation::{ProgramType, SourceType};

/// Result of a single LLM translation call
pub struct TranslationOutput {
    pub code: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub duration_secs: f64,
}

/// LLM-based translator for converting C2Rust or C code to idiomatic Rust
pub struct Translator;

impl Translator {
    pub fn new() -> Self {
        Self
    }

    /// Translate source code to idiomatic Rust.
    /// When retrying, `previous_code` contains the last translation attempt and
    /// `feedback` describes why it failed — so the LLM can make targeted fixes.
    pub async fn translate(
        &self,
        source_code: &str,
        feedback: Option<&str>,
        previous_code: Option<&str>,
        llm_client: &dyn LlmClient,
        source_type: &SourceType,
        program_type: &ProgramType,
    ) -> Result<TranslationOutput> {
        let system_prompt = match (source_type, program_type) {
            (SourceType::C, ProgramType::Executable) => self.build_executable_c_system_prompt(),
            (SourceType::C, ProgramType::Library) => self.build_c_system_prompt(),
            (SourceType::Rust, ProgramType::Executable) => self.build_executable_system_prompt(),
            (SourceType::Rust, ProgramType::Library) => self.build_system_prompt(),
        };
        let user_prompt = match (source_type, program_type) {
            (SourceType::C, ProgramType::Executable) => self.build_executable_c_user_prompt(source_code, feedback, previous_code),
            (SourceType::C, ProgramType::Library) => self.build_c_user_prompt(source_code, feedback, previous_code),
            (SourceType::Rust, ProgramType::Executable) => self.build_executable_user_prompt(source_code, feedback, previous_code),
            (SourceType::Rust, ProgramType::Library) => self.build_user_prompt(source_code, feedback, previous_code),
        };

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

    fn build_user_prompt(&self, source_code: &str, feedback: Option<&str>, previous_code: Option<&str>) -> String {
        let mut prompt = format!("Transform this C2Rust code to idiomatic Rust:\n\n```rust\n{}\n```", source_code);

        if let Some(prev) = previous_code {
            if let Some(feedback) = feedback {
                prompt.push_str(&format!(
                    "\n\nYour previous translation:\n```rust\n{}\n```\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix the specific issues in your previous translation while keeping FFI compatibility.",
                    prev, feedback
                ));
            }
        } else if let Some(feedback) = feedback {
            prompt.push_str(&format!("\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix these issues while keeping FFI compatibility.", feedback));
        }

        prompt
    }

    fn build_c_system_prompt(&self) -> String {
        r#"You translate C code to idiomatic Rust while preserving FFI compatibility.

RULES (violating any = broken build):
- Every public C function must become `#[no_mangle] pub unsafe extern "C" fn` with the EXACT same name, parameter types (using C-compatible types: c_int, c_uint, c_char, etc.), and return type
- Use `use std::os::raw::*;` or `use std::ffi::*;` for C type aliases
- Keep ALL global/static state as `#[no_mangle] pub static` or `static mut` where needed
- Keep FFI-boundary types C-compatible (repr(C) for structs, primitives, raw pointers)
- Preserve the algorithm/logic exactly; only change implementation style
- The output must compile as a single `lib.rs` file

TRANSFORMS (internal code only):
- malloc/calloc/free → Vec, Box, or Rust allocation
- Raw pointer arithmetic → slice indexing where safe
- C-style loops → iterators or for-range
- C string handling → Rust string/CStr where possible internally
- C macros → Rust functions, constants, or inline code
- C structs → Rust structs with #[repr(C)] when exposed at FFI boundary
- Use const, meaningful names, Option/Result internally

Be conservative. When in doubt, keep the unsafe implementation close to the C original.
Produce a SINGLE lib.rs file with all code.

Output ONLY the code in a ```rust block, no explanations."#.to_string()
    }

    fn build_c_user_prompt(&self, source_code: &str, feedback: Option<&str>, previous_code: Option<&str>) -> String {
        let mut prompt = format!("Translate this C code to idiomatic Rust:\n\n```c\n{}\n```", source_code);

        if let Some(prev) = previous_code {
            if let Some(feedback) = feedback {
                prompt.push_str(&format!(
                    "\n\nYour previous translation:\n```rust\n{}\n```\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix the specific issues in your previous translation while keeping FFI compatibility.",
                    prev, feedback
                ));
            }
        } else if let Some(feedback) = feedback {
            prompt.push_str(&format!("\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix these issues while keeping FFI compatibility.", feedback));
        }

        prompt
    }

    fn build_executable_system_prompt(&self) -> String {
        r#"You convert C2Rust executable code to idiomatic Rust.

RULES (violating any = broken program):
- The output must be a single `main.rs` file with `fn main()`
- Convert the C2Rust `main_0`/`main` pattern to a clean `fn main()`
- Convert `argc`/`argv` patterns to `std::env::args()`
- Convert `printf`/`fprintf` to `print!()`/`eprint!()`/`write!()` — match the EXACT output format
- Convert `scanf`/`fgets`/`getchar` to `std::io::stdin().read_line()` or similar
- Convert `atoi`/`atof`/`strtol` to Rust's `.parse()` methods
- Convert `exit(n)` to `std::process::exit(n)`
- Keep ALL `#![feature(...)]` and `#![allow(...)]` declarations
- Preserve the algorithm/logic; only change implementation style
- Do NOT use the `libc` crate — use `std` equivalents for all I/O and string operations
- Do NOT use `#[no_mangle]`, `extern "C"`, or any FFI exports

TRANSFORMS (internal code only):
- malloc/calloc/free → Vec, Box, or Rust allocation
- Raw pointer arithmetic → slice indexing where safe
- C-style loops → iterators or for-range
- Redundant casts → direct literals
- C string handling → Rust String/&str
- Use const, meaningful names, Option/Result

Be conservative. Preserve exact output formatting (whitespace, newlines).

Output ONLY the code in a ```rust block, no explanations."#.to_string()
    }

    fn build_executable_user_prompt(&self, source_code: &str, feedback: Option<&str>, previous_code: Option<&str>) -> String {
        let mut prompt = format!("Transform this C2Rust executable code to idiomatic Rust:\n\n```rust\n{}\n```", source_code);

        if let Some(prev) = previous_code {
            if let Some(feedback) = feedback {
                prompt.push_str(&format!(
                    "\n\nYour previous translation:\n```rust\n{}\n```\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix the specific issues in your previous translation. This is an executable program (not a library), so use fn main() and std I/O.",
                    prev, feedback
                ));
            }
        } else if let Some(feedback) = feedback {
            prompt.push_str(&format!("\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix these issues. This is an executable program (not a library), so use fn main() and std I/O.", feedback));
        }

        prompt
    }

    fn build_executable_c_system_prompt(&self) -> String {
        r#"You translate C executable code to idiomatic Rust.

RULES (violating any = broken program):
- The output must be a single `main.rs` file with `fn main()`
- `argc`/`argv` → `std::env::args()`
- `printf`/`fprintf(stdout, ...)` → `print!()`/`write!()` — match EXACT output format (spacing, newlines)
- `fprintf(stderr, ...)` → `eprint!()`/`write!(std::io::stderr(), ...)`
- `scanf`/`fgets`/`getchar`/`getc(stdin)` → `std::io::stdin().read_line()` or `.bytes()`
- `atoi`/`atof`/`strtol` → `.parse::<i32>()` etc.
- `exit(n)` → `std::process::exit(n)`
- `malloc`/`calloc`/`free` → Vec, Box, or Rust allocation
- Do NOT use the `libc` crate — use only `std` library
- Do NOT use `#[no_mangle]`, `extern "C"`, or any FFI exports
- Preserve the algorithm/logic exactly; only change implementation style
- C structs → Rust structs (no need for #[repr(C)] since there's no FFI boundary)
- C macros → Rust functions, constants, or inline code

IMPORTANT: The program is tested by comparing stdout output character-by-character.
Match the exact output format of the original C program.

Output ONLY the code in a ```rust block, no explanations."#.to_string()
    }

    fn build_executable_c_user_prompt(&self, source_code: &str, feedback: Option<&str>, previous_code: Option<&str>) -> String {
        let mut prompt = format!("Translate this C executable code to idiomatic Rust:\n\n```c\n{}\n```", source_code);

        if let Some(prev) = previous_code {
            if let Some(feedback) = feedback {
                prompt.push_str(&format!(
                    "\n\nYour previous translation:\n```rust\n{}\n```\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix the specific issues in your previous translation. This is an executable program (not a library), so use fn main() and std I/O. Do not use the libc crate.",
                    prev, feedback
                ));
            }
        } else if let Some(feedback) = feedback {
            prompt.push_str(&format!("\n\nPREVIOUS ATTEMPT FAILED with error:\n{}\nFix these issues. This is an executable program (not a library), so use fn main() and std I/O. Do not use the libc crate.", feedback));
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
    fn test_c_system_prompt_contains_key_requirements() {
        let translator = Translator::new();
        let prompt = translator.build_c_system_prompt();

        assert!(prompt.contains("#[no_mangle]"));
        assert!(prompt.contains("extern \"C\""));
        assert!(prompt.contains("FFI"));
        assert!(prompt.contains("C-compatible"));
    }

    #[test]
    fn test_user_prompt_with_feedback() {
        let translator = Translator::new();
        let code = "fn test() {}";
        let feedback = "Build error: missing semicolon";
        let prev = "fn broken() {}";

        let prompt = translator.build_user_prompt(code, Some(feedback), Some(prev));

        assert!(prompt.contains(code));
        assert!(prompt.contains(feedback));
        assert!(prompt.contains(prev));
        assert!(prompt.contains("previous translation"));
    }

    #[test]
    fn test_user_prompt_no_feedback() {
        let translator = Translator::new();
        let code = "fn test() {}";

        let prompt = translator.build_user_prompt(code, None, None);

        assert!(prompt.contains(code));
        assert!(!prompt.contains("PREVIOUS"));
    }

    #[test]
    fn test_c_user_prompt_with_feedback() {
        let translator = Translator::new();
        let code = "void test() {}";
        let feedback = "Build error: missing semicolon";
        let prev = "fn broken() {}";

        let prompt = translator.build_c_user_prompt(code, Some(feedback), Some(prev));

        assert!(prompt.contains(code));
        assert!(prompt.contains(feedback));
        assert!(prompt.contains(prev));
        assert!(prompt.contains("```c"));
    }

    #[test]
    fn test_executable_system_prompt_no_ffi() {
        let translator = Translator::new();
        let prompt = translator.build_executable_system_prompt();

        assert!(prompt.contains("fn main()"));
        assert!(prompt.contains("main.rs"));
        // Prompt should tell the LLM NOT to use FFI
        assert!(prompt.contains("Do NOT use"));
        assert!(prompt.contains("libc"));
        assert!(prompt.contains("std::env::args"));
    }

    #[test]
    fn test_executable_c_system_prompt_no_ffi() {
        let translator = Translator::new();
        let prompt = translator.build_executable_c_system_prompt();

        assert!(prompt.contains("fn main()"));
        assert!(prompt.contains("main.rs"));
        // Prompt should tell the LLM NOT to use FFI
        assert!(prompt.contains("Do NOT use"));
        assert!(prompt.contains("stdout"));
        assert!(prompt.contains("libc"));
    }

    #[test]
    fn test_executable_user_prompt_with_feedback() {
        let translator = Translator::new();
        let code = "fn main_0() {}";
        let feedback = "stdout mismatch";
        let prev = "fn main() { println!(\"wrong\"); }";

        let prompt = translator.build_executable_user_prompt(code, Some(feedback), Some(prev));

        assert!(prompt.contains(code));
        assert!(prompt.contains(prev));
        assert!(prompt.contains("previous translation"));
        assert!(prompt.contains("executable"));
    }
}
