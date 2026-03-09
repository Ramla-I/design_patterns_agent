use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use serde::Deserialize;

use crate::llm::{LlmClient, LlmRequest};
use crate::navigation::AnalysisChunk;
use crate::report::{Confidence, Evidence, Invariant, InvariantType, Location};

use super::evidence::EvidenceExtractor;

/// Detector that analyzes module-level code chunks for latent invariants —
/// implicit protocols and ordering requirements NOT yet enforced by the type system.
pub struct InvariantInferenceDetector;

const SYSTEM_PROMPT: &str = r#"You are an expert Rust code analyst. Your task is to find **latent invariants** — implicit protocols, temporal ordering requirements, and state dependencies that exist in code but are NOT yet enforced by the type system.

You are NOT looking for patterns already well-implemented (e.g., existing typestate with PhantomData). You ARE looking for places where the developer relies on runtime checks, comments, naming conventions, or error handling to enforce rules that COULD be compile-time guarantees.

## Signal Categories (ranked by reliability)

1. **Runtime state checks**: `if !self.initialized`, `assert!(self.is_connected())`, `.is_null()` checks, `unwrap()` with panic messages describing preconditions

2. **Boolean/enum state fields**: Fields like `is_open: bool`, `state: AtomicU8`, `inner: Option<T>` that track state at runtime instead of at the type level

3. **Comment-based protocols**: `// must call X before Y`, `// SAFETY: assumes ...`, `// Invariant:`, `// Precondition:`

4. **Error messages revealing invariants**: `Err("not initialized")`, `panic!("connection closed")` — the error text IS the invariant

5. **Self-consuming methods**: `fn close(self)` or `fn close(mut self)` that destroy the current state — these are transitions that should produce a new type

6. **Method availability patterns**: Methods that check state before acting (e.g., `read()` checks `is_open`) — these should only be callable in the correct state

7. **Option<T>/UnsafeCell patterns**: `inner: UnsafeCell<Option<T>>` indicates uninitialized/initialized states; "written to at most once" invariants

8. **Atomic state machines**: `state: AtomicU8` with named constants like `INCOMPLETE=0, RUNNING=1, COMPLETE=2` — these are explicit state machines encoded as integers instead of types

## Worked Example

**Input code:**
```rust
pub struct FileHandle {
    fd: i32,
    is_open: bool,
}

impl FileHandle {
    pub fn open(path: &str) -> FileHandle {
        FileHandle { fd: 3, is_open: true }
    }
    pub fn read(&self) -> Result<Vec<u8>, &'static str> {
        if !self.is_open { return Err("closed") }
        Ok(vec![1, 2, 3])
    }
    pub fn close(mut self) -> Result<(), &'static str> {
        if !self.is_open { return Err("already closed") }
        self.is_open = false;
        Ok(())
    }
}
```

**Expected output:**
```json
[
  {
    "entity": "FileHandle",
    "name": "FileHandle::Open state",
    "state": "Open",
    "kind": "state_machine",
    "description": "FileHandle is in the Open state after open(); read() is valid, close() transitions to Closed",
    "invariants": [
      "is_open == true",
      "file descriptor is valid",
      "read() operations are valid"
    ],
    "transitions": ["Open -> Closed via close()"],
    "evidence": [
      "is_open: bool field tracks state at runtime",
      "if !self.is_open { return Err(\"closed\") } in read()",
      "close(mut self) consumes ownership"
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "struct FileHandle<S> { fd: i32, _state: PhantomData<S> } with Open/Closed zero-sized types; read() only on FileHandle<Open>; close(self) -> FileHandle<Closed>",
    "confidence": "high"
  },
  {
    "entity": "FileHandle",
    "name": "FileHandle::Closed state",
    "state": "Closed",
    "kind": "state_machine",
    "description": "FileHandle is in the Closed state after close(); read() is invalid, no further transitions",
    "invariants": [
      "is_open == false",
      "read() returns Err(\"closed\")",
      "no valid operations remain"
    ],
    "transitions": [],
    "evidence": [
      "is_open set to false in close()",
      "Err(\"already closed\") error message in close()",
      "read() checks is_open before proceeding"
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "FileHandle<Closed> has no read() method — compile error instead of runtime Err",
    "confidence": "high"
  }
]
```

**Why this is detected:** The `is_open` boolean field is a runtime encoding of two distinct states. The `if !self.is_open` checks in `read()` and `close()` are runtime guards that would be unnecessary if the type system enforced which state the handle is in. The error messages `"closed"` and `"already closed"` name the invariants directly.

## Worked Example 2 — OnceCell / Option-based initialization

**Input code:**
```rust
pub struct Config {
    inner: Option<ConfigData>,
}

struct ConfigData {
    db_url: String,
    pool_size: usize,
}

impl Config {
    pub fn new() -> Self {
        Config { inner: None }
    }

    /// Must be called before any get() calls.
    pub fn initialize(&mut self, db_url: String, pool_size: usize) {
        if self.inner.is_some() {
            panic!("Config already initialized");
        }
        self.inner = Some(ConfigData { db_url, pool_size });
    }

    pub fn db_url(&self) -> &str {
        // Precondition: must call initialize() first
        self.inner.as_ref().expect("not initialized").db_url.as_str()
    }

    pub fn pool_size(&self) -> usize {
        self.inner.as_ref().expect("not initialized").pool_size
    }
}
```

**Expected output:**
```json
[
  {
    "entity": "Config",
    "name": "Config::Uninitialized state",
    "state": "Uninitialized",
    "kind": "temporal_ordering",
    "description": "Config is created empty via new(); no getters are valid until initialize() is called",
    "invariants": [
      "inner == None",
      "db_url() and pool_size() will panic",
      "initialize() is the only valid operation"
    ],
    "transitions": ["Uninitialized -> Initialized via initialize()"],
    "evidence": [
      "inner: Option<ConfigData> field — None encodes uninitialized",
      "expect(\"not initialized\") in db_url() and pool_size()",
      "comment: \"Must be called before any get() calls.\""
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "Split into Config<Uninit> and Config<Init>; new() returns Config<Uninit>; initialize(self) -> Config<Init>; db_url()/pool_size() only on Config<Init>",
    "confidence": "high"
  },
  {
    "entity": "Config",
    "name": "Config::Initialized state",
    "state": "Initialized",
    "kind": "temporal_ordering",
    "description": "Config holds valid ConfigData after initialize(); getters are safe, re-initialization panics",
    "invariants": [
      "inner == Some(ConfigData { .. })",
      "db_url() and pool_size() return valid values",
      "initialize() panics with \"Config already initialized\""
    ],
    "transitions": [],
    "evidence": [
      "inner set to Some(..) in initialize()",
      "panic!(\"Config already initialized\") guards against double-init",
      "expect(\"not initialized\") would succeed"
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "Config<Init> has no initialize() method — double-init is a compile error instead of a runtime panic",
    "confidence": "high"
  }
]
```

**Why this is detected:** The `Option<ConfigData>` field is a runtime encoding of Uninitialized/Initialized states. The `expect("not initialized")` calls in getters and the `panic!("Config already initialized")` guard in `initialize()` are runtime invariant checks. The comment "Must be called before any get() calls" explicitly names the temporal ordering requirement.

## What to Report

For each entity with implicit states, identify:
1. **Each distinct state** the entity can be in (e.g., Open, Closed, Uninitialized, Initialized)
2. **Invariants per state** — what must be true in that state
3. **Valid transitions** — how to move between states (which method, consuming self or not)
4. **Evidence** — specific fields, checks, error messages, or comments that reveal the state

## Classification

Classify each invariant as:
- `temporal_ordering`: "must call X before Y"
- `resource_lifecycle`: "must acquire then release"
- `state_machine`: "valid state transitions between distinct states"
- `precondition`: "must satisfy X before calling Y"
- `protocol`: "multi-step interaction pattern"

## Suggested Patterns

- `typestate`: PhantomData<State> to track state at type level — best for entities with distinct states and transitions
- `builder`: Builder pattern for complex initialization sequences
- `raii`: RAII/Drop for cleanup guarantees
- `newtype`: Newtype wrapper for validity invariants (e.g., NonEmptyVec, ValidatedEmail)
- `session_type`: Session types for protocol enforcement
- `capability`: Capability/token passing for authorization

## Rules

- **Only report invariants backed by concrete evidence in the code.** Do NOT invent methods, fields, or behaviors not present in the snippet.
- **You MAY infer** states from: field types (Option<T>, enums, booleans, atomics), method signatures (self vs &self vs &mut self), error messages, comments, and API structure.
- **You MAY NOT invent**: methods not shown in the snippet, fields not present, speculative behaviors without code support.
- **Quality over quantity** — one well-evidenced invariant is better than five speculative ones."#;

impl InvariantInferenceDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn detect(
        &self,
        chunk: &AnalysisChunk,
        llm_client: &dyn LlmClient,
        next_id: &AtomicUsize,
    ) -> Result<Vec<Invariant>> {
        let code_content = EvidenceExtractor::format_chunk(chunk);

        let user_prompt = format!(
            r#"Analyze the following Rust module for latent invariants — implicit states, ordering requirements, and protocols that could be enforced at compile time.

Module: `{module_path}`
File: `{file_path}`

```rust
{code}
```

For each entity (struct, enum, or function group) with implicit states, produce one entry **per state**. Respond with a JSON array:

```json
[
  {{
    "entity": "the struct/type/module this invariant applies to",
    "name": "short descriptive name (e.g., 'FileHandle::Open state')",
    "state": "the specific state (e.g., 'Open', 'Uninitialized', 'Connected')",
    "kind": "temporal_ordering | resource_lifecycle | state_machine | precondition | protocol",
    "description": "what must be true in this state",
    "invariants": ["condition 1 that holds in this state", "condition 2"],
    "transitions": ["Open -> Closed via close()", "or empty if terminal state"],
    "evidence": ["is_open boolean field", "if !self.is_open check on line N", "Err(\"closed\") error message"],
    "suggested_pattern": "typestate | builder | raii | newtype | session_type | capability",
    "implementation_sketch": "brief description of how to apply the pattern",
    "confidence": "high | medium | low"
  }}
]
```

If no meaningful invariants are found, respond with an empty array: `[]`

Focus on invariants that are **implicit** — enforced by runtime checks, comments, boolean flags, Option<T> fields, or conventions rather than the type system. Ground every claim in specific code elements."#,
            module_path = chunk.module_path,
            file_path = chunk.file_path.display(),
            code = code_content,
        );

        let request = LlmRequest::new(SYSTEM_PROMPT, user_prompt).with_temperature(0.3);
        let response = llm_client.complete(request).await?;

        let invariants = self.parse_response(&response.content, chunk, &code_content, next_id)?;
        Ok(invariants)
    }

    fn parse_response(
        &self,
        response: &str,
        chunk: &AnalysisChunk,
        code_snippet: &str,
        next_id: &AtomicUsize,
    ) -> Result<Vec<Invariant>> {
        // Try JSON parsing first
        if let Some(invariants) = self.try_parse_json(response, chunk, code_snippet, next_id) {
            return Ok(invariants);
        }

        // Fallback: text-based parsing for robustness
        self.parse_text_fallback(response, chunk, code_snippet, next_id)
    }

    fn try_parse_json(
        &self,
        response: &str,
        chunk: &AnalysisChunk,
        code_snippet: &str,
        next_id: &AtomicUsize,
    ) -> Option<Vec<Invariant>> {
        // Find JSON array in response (may be wrapped in markdown code fences)
        let json_str = extract_json_array(response)?;

        let parsed: Vec<LlmInvariant> = serde_json::from_str(&json_str).ok()?;

        let invariants = parsed
            .into_iter()
            .map(|inv| {
                // Build rich explanation with state info
                let mut explanation = String::new();

                if !inv.entity.is_empty() {
                    explanation.push_str(&format!("**Entity:** {}\n\n", inv.entity));
                }
                if !inv.state.is_empty() {
                    explanation.push_str(&format!("**State:** {}\n\n", inv.state));
                }
                if !inv.invariants.is_empty() {
                    explanation.push_str("**State invariants:**\n");
                    for i in &inv.invariants {
                        explanation.push_str(&format!("- {}\n", i));
                    }
                    explanation.push('\n');
                }
                if !inv.transitions.is_empty() {
                    explanation.push_str("**Transitions:**\n");
                    for t in &inv.transitions {
                        explanation.push_str(&format!("- {}\n", t));
                    }
                    explanation.push('\n');
                }
                explanation.push_str(&format!("**Evidence:** {}\n\n", inv.evidence.join("; ")));
                explanation.push_str(&format!("**Implementation:** {}", inv.implementation_sketch));

                let id = next_id.fetch_add(1, Ordering::Relaxed);
                Invariant {
                    id,
                    invariant_type: parse_invariant_kind(&inv.kind),
                    title: inv.name,
                    description: inv.description,
                    location: Location {
                        file_path: chunk.file_path.to_string_lossy().to_string(),
                        line_start: 1,
                        line_end: code_snippet.lines().count(),
                    },
                    evidence: Evidence {
                        code_snippet: code_snippet.to_string(),
                        explanation,
                    },
                    suggested_pattern: inv.suggested_pattern,
                    confidence: parse_confidence(&inv.confidence),
                }
            })
            .collect();

        Some(invariants)
    }

    fn parse_text_fallback(
        &self,
        response: &str,
        chunk: &AnalysisChunk,
        code_snippet: &str,
        next_id: &AtomicUsize,
    ) -> Result<Vec<Invariant>> {
        if response.contains("[]") || response.contains("NO INVARIANTS") || response.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut invariants = Vec::new();

        for section in response.split("INVARIANT").skip(1) {
            if section.trim().starts_with("S FOUND") {
                continue;
            }

            if let Some(invariant) = self.parse_text_section(section, chunk, code_snippet, next_id) {
                invariants.push(invariant);
            }
        }

        Ok(invariants)
    }

    fn parse_text_section(
        &self,
        section: &str,
        chunk: &AnalysisChunk,
        code_snippet: &str,
        next_id: &AtomicUsize,
    ) -> Option<Invariant> {
        let name = extract_field(section, "Name:")?;
        let description = extract_field(section, "Description:")?;
        let evidence = extract_field(section, "Evidence:").unwrap_or_default();
        let pattern = extract_field(section, "Pattern:").unwrap_or_default();
        let implementation = extract_field(section, "Implementation:").unwrap_or_default();

        let explanation = format!(
            "**Evidence:** {}\n\n**Implementation:** {}",
            evidence, implementation
        );

        let id = next_id.fetch_add(1, Ordering::Relaxed);
        let invariant = Invariant {
            id,
            invariant_type: classify_from_text(&pattern, &description),
            title: name,
            description,
            location: Location {
                file_path: chunk.file_path.to_string_lossy().to_string(),
                line_start: 1,
                line_end: code_snippet.lines().count(),
            },
            evidence: Evidence {
                code_snippet: code_snippet.to_string(),
                explanation,
            },
            suggested_pattern: pattern,
            confidence: Confidence::Medium,
        };

        Some(invariant)
    }
}

impl Default for InvariantInferenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

// --- JSON response structures ---

#[derive(Deserialize)]
struct LlmInvariant {
    #[serde(default)]
    entity: String,
    name: String,
    #[serde(default)]
    state: String,
    kind: String,
    description: String,
    #[serde(default)]
    invariants: Vec<String>,
    #[serde(default)]
    transitions: Vec<String>,
    evidence: Vec<String>,
    suggested_pattern: String,
    implementation_sketch: String,
    confidence: String,
}

// --- Parsing helpers ---

fn extract_json_array(text: &str) -> Option<String> {
    // Try to find JSON array, possibly inside markdown code fences
    let text = text.trim();

    // Remove markdown fences if present
    let text = if text.contains("```json") {
        text.split("```json").nth(1)?.split("```").next()?
    } else if text.contains("```") {
        text.split("```").nth(1)?.split("```").next().unwrap_or(text)
    } else {
        text
    };

    // Find the array bounds
    let start = text.find('[')?;
    let end = text.rfind(']')? + 1;
    if start >= end {
        return None;
    }

    Some(text[start..end].to_string())
}

fn parse_invariant_kind(kind: &str) -> InvariantType {
    match kind.to_lowercase().as_str() {
        "temporal_ordering" => InvariantType::TemporalOrdering,
        "resource_lifecycle" => InvariantType::ResourceLifecycle,
        "state_machine" => InvariantType::StateMachine,
        "precondition" => InvariantType::Precondition,
        "protocol" => InvariantType::Protocol,
        _ => {
            // Fuzzy match
            let k = kind.to_lowercase();
            if k.contains("temporal") || k.contains("order") || k.contains("sequence") {
                InvariantType::TemporalOrdering
            } else if k.contains("resource") || k.contains("lifecycle") || k.contains("cleanup") {
                InvariantType::ResourceLifecycle
            } else if k.contains("state") || k.contains("machine") {
                InvariantType::StateMachine
            } else if k.contains("precondition") || k.contains("require") {
                InvariantType::Precondition
            } else {
                InvariantType::Protocol
            }
        }
    }
}

fn parse_confidence(confidence: &str) -> Confidence {
    match confidence.to_lowercase().as_str() {
        "high" => Confidence::High,
        "low" => Confidence::Low,
        _ => Confidence::Medium,
    }
}

fn classify_from_text(pattern: &str, description: &str) -> InvariantType {
    let p = pattern.to_lowercase();
    let d = description.to_lowercase();

    if p.contains("typestate") || p.contains("session") || d.contains("order") || d.contains("sequence") || d.contains("before") {
        InvariantType::TemporalOrdering
    } else if p.contains("raii") || p.contains("drop") || d.contains("cleanup") || d.contains("free") || d.contains("release") || d.contains("close") {
        InvariantType::ResourceLifecycle
    } else if d.contains("state") || d.contains("transition") {
        InvariantType::StateMachine
    } else if d.contains("must") || d.contains("precondition") || d.contains("require") {
        InvariantType::Precondition
    } else {
        InvariantType::Protocol
    }
}

fn extract_field(text: &str, field_name: &str) -> Option<String> {
    let start = text.find(field_name)? + field_name.len();
    let remaining = &text[start..];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invariant_kind() {
        assert_eq!(parse_invariant_kind("temporal_ordering"), InvariantType::TemporalOrdering);
        assert_eq!(parse_invariant_kind("resource_lifecycle"), InvariantType::ResourceLifecycle);
        assert_eq!(parse_invariant_kind("state_machine"), InvariantType::StateMachine);
        assert_eq!(parse_invariant_kind("precondition"), InvariantType::Precondition);
        assert_eq!(parse_invariant_kind("protocol"), InvariantType::Protocol);
        // Fuzzy
        assert_eq!(parse_invariant_kind("ordering requirement"), InvariantType::TemporalOrdering);
    }

    #[test]
    fn test_parse_confidence() {
        assert_eq!(parse_confidence("high"), Confidence::High);
        assert_eq!(parse_confidence("medium"), Confidence::Medium);
        assert_eq!(parse_confidence("low"), Confidence::Low);
        assert_eq!(parse_confidence("unknown"), Confidence::Medium);
    }

    #[test]
    fn test_extract_json_array() {
        let response = r#"Here are the invariants:
```json
[{"name": "test", "kind": "temporal_ordering"}]
```"#;
        let json = extract_json_array(response).unwrap();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }

    #[test]
    fn test_extract_json_array_no_fences() {
        let response = r#"[{"name": "test"}]"#;
        let json = extract_json_array(response).unwrap();
        assert_eq!(json, r#"[{"name": "test"}]"#);
    }

    #[test]
    fn test_extract_json_array_empty() {
        let json = extract_json_array("[]").unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_classify_from_text() {
        assert_eq!(
            classify_from_text("typestate", "must call init before use"),
            InvariantType::TemporalOrdering
        );
        assert_eq!(
            classify_from_text("RAII", "cleanup resources on drop"),
            InvariantType::ResourceLifecycle
        );
        assert_eq!(
            classify_from_text("newtype", "state transitions"),
            InvariantType::StateMachine
        );
    }

    #[test]
    fn test_extract_field() {
        let text = "Name: File Handle State\nDescription: File must be opened first\nPattern: Typestate";
        assert_eq!(extract_field(text, "Name:"), Some("File Handle State".to_string()));
        assert_eq!(extract_field(text, "Description:"), Some("File must be opened first".to_string()));
        assert_eq!(extract_field(text, "Pattern:"), Some("Typestate".to_string()));
    }
}
