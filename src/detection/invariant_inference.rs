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
    "name": "FileHandle state machine (Open / Closed)",
    "kind": "state_machine",
    "states": ["Open", "Closed"],
    "description": "FileHandle has two runtime states tracked by is_open: bool. In Open state read() is valid; close() transitions to Closed. In Closed state all operations return errors. The type system does not distinguish these states.",
    "transitions": ["Open -> Closed via close()"],
    "evidence": [
      "line 3: is_open: bool field tracks state at runtime",
      "line 12: if !self.is_open { return Err(\"closed\") } guards read()",
      "line 15: close(mut self) consumes and transitions to Closed",
      "line 16: Err(\"already closed\") names the invalid-state invariant"
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "struct FileHandle<S> { fd: i32, _state: PhantomData<S> } with Open/Closed zero-sized types; read() only on FileHandle<Open>; close(self) -> FileHandle<Closed>",
    "confidence": "high"
  }
]
```

**Why this is good:** ONE entry per entity capturing the COMPLETE state machine — all states, all transitions, all evidence. The `is_open` boolean is a runtime encoding of two distinct states, and the guards/error messages are the evidence.

## Worked Example 2 — OnceCell / Option-based initialization

**Input code:**
```rust
pub struct Config {
    inner: Option<ConfigData>,
}

impl Config {
    pub fn new() -> Self { Config { inner: None } }

    /// Must be called before any get() calls.
    pub fn initialize(&mut self, db_url: String, pool_size: usize) {
        if self.inner.is_some() { panic!("Config already initialized"); }
        self.inner = Some(ConfigData { db_url, pool_size });
    }

    pub fn db_url(&self) -> &str {
        self.inner.as_ref().expect("not initialized").db_url.as_str()
    }
}
```

**Expected output:**
```json
[
  {
    "entity": "Config",
    "name": "Config initialization protocol (Uninitialized -> Initialized)",
    "kind": "temporal_ordering",
    "states": ["Uninitialized", "Initialized"],
    "description": "Config must have initialize() called before db_url()/pool_size(). Option<ConfigData> encodes this at runtime; getters panic if called before initialize(), and double-init panics.",
    "transitions": ["Uninitialized -> Initialized via initialize()"],
    "evidence": [
      "line 2: inner: Option<ConfigData> — None encodes uninitialized",
      "line 8: comment: Must be called before any get() calls",
      "line 10: panic!(\"Config already initialized\") guards double-init",
      "line 14: expect(\"not initialized\") in db_url() — panic on wrong state"
    ],
    "suggested_pattern": "typestate",
    "implementation_sketch": "Split into Config<Uninit> and Config<Init>; initialize(self) -> Config<Init>; getters only on Config<Init>",
    "confidence": "high"
  }
]
```

**Why this is good:** ONE entry for the whole protocol. The description covers both states and the transition. Evidence cites specific lines.

## What to Report

For each entity, produce **ONE invariant entry** that captures the complete picture:
1. **All states** the entity can be in
2. **All transitions** between those states
3. **Evidence** — specific fields, checks, error messages, or comments

**Produce ONE entry per entity, NOT one per state.** A FileHandle with Open/Closed states is ONE invariant, not two.

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
- **Quality over quantity** — one well-evidenced invariant is better than five speculative ones. Aim for 1-3 invariants per chunk, not 5-10.
- **Only report if actionable** — the invariant should suggest a concrete type-system improvement. "This enum has variants" is not actionable. "This boolean field gates method validity and could be a typestate" IS actionable.

EXCLUSIONS — do NOT report:
- Conditional compilation (cfg, cfg_if, cfg_select, cfg_attr, feature gates)
- Enum variant type safety (e.g. "TokenTree variant matches its data" — this is enforced by the type system)
- Error types that simply describe failure modes (e.g., TryRecvError::Empty, SendError::Disconnected) — these are already encoded as enum variants
- Guard/lock types whose state is already enforced by RAII (e.g., MutexGuard's lifetime)
- Documentation or API stability policies
- Properties that are already compile-time guarantees
- Types that merely wrap or re-export other types without adding implicit state"#;

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
            r#"Analyze the following Rust code from module `{module_path}`.
Find latent invariants — implicit states, ordering requirements, and protocols that could be enforced at compile time.
File: `{file_path}`

```rust
{code}
```

For each entity with implicit state, produce **ONE entry per entity** (not one per state). Respond with a JSON array:

```json
[
  {{
    "entity": "the struct/type this invariant applies to",
    "name": "short name describing the whole invariant (e.g., 'FileHandle state machine (Open / Closed)')",
    "states": ["State1", "State2"],
    "kind": "temporal_ordering | resource_lifecycle | state_machine | precondition | protocol",
    "description": "complete description covering all states, transitions, and what is NOT enforced by the type system",
    "transitions": ["State1 -> State2 via method()", "..."],
    "evidence": ["line 5: is_open boolean field", "line 12: if !self.is_open check guards read()"],
    "suggested_pattern": "typestate | builder | raii | newtype | session_type | capability",
    "implementation_sketch": "brief description of how to apply the pattern",
    "confidence": "high | medium | low"
  }}
]
```

Each `evidence` item MUST reference concrete code: a field name, method name, error message, or comment. Starting with `line N:` is preferred but not required.

If no meaningful invariants are found, respond with an empty array: `[]`

Guidelines:
- **ONE entry per entity**, covering ALL its states and transitions together.
- **1-3 invariants per chunk** is typical. Only report genuinely latent invariants backed by strong evidence.
- Skip error types, guard wrappers, and types whose state is already enforced by the type system.
- Focus on invariants that are **actionable** — where a typestate, newtype, RAII, or capability pattern would eliminate a class of runtime errors."#,
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
            .filter(|inv| !is_compile_time_noise(inv))
            .filter(|inv| !is_enum_variant_noise(inv))
            .map(|inv| {
                // Build rich explanation with state info
                let mut explanation = String::new();

                if !inv.entity.is_empty() {
                    explanation.push_str(&format!("**Entity:** {}\n\n", inv.entity));
                }
                // Use new `states` list, falling back to legacy `state` field
                let all_states: Vec<&str> = if !inv.states.is_empty() {
                    inv.states.iter().map(|s| s.as_str()).collect()
                } else if !inv.state.is_empty() {
                    vec![inv.state.as_str()]
                } else {
                    vec![]
                };
                if !all_states.is_empty() {
                    explanation.push_str(&format!("**States:** {}\n\n", all_states.join(", ")));
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

                // Citation verification: check if evidence references real code
                // Be lenient: evidence that names fields/methods/error messages without
                // line numbers is still valuable. Only penalize fabricated citations.
                let citation_rate = verify_citations(&inv.evidence, code_snippet);
                let adjusted_confidence = if citation_rate < 0.3 {
                    // Most cited lines are wrong — likely hallucinated
                    downgrade_confidence(&inv.confidence)
                } else {
                    inv.confidence.clone()
                };

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
                    confidence: parse_confidence(&adjusted_confidence),
                    entity: inv.entity.clone(),
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
            entity: String::new(),
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
    /// New format: list of states (e.g., ["Open", "Closed"])
    #[serde(default)]
    states: Vec<String>,
    /// Legacy format: single state string (accepted for backward compat)
    #[serde(default)]
    state: String,
    kind: String,
    description: String,
    #[serde(default)]
    transitions: Vec<String>,
    evidence: Vec<String>,
    suggested_pattern: String,
    implementation_sketch: String,
    confidence: String,
}

fn is_compile_time_noise(inv: &LlmInvariant) -> bool {
    let text = format!("{} {} {}", inv.entity, inv.name, inv.description).to_lowercase();
    let noise = ["cfg_select", "cfg_if", "cfg_attr", "feature gate",
                  "conditional compilation", "variant correctness",
                  "enum exhaustiveness", "documentation policy",
                  "doc comment", "api stability", "compile-time"];
    noise.iter().any(|p| text.contains(p))
}

/// Filter out invariants that just describe enum variants / error types
/// which are already enforced by the type system.
fn is_enum_variant_noise(inv: &LlmInvariant) -> bool {
    let entity = inv.entity.to_lowercase();
    let desc = inv.description.to_lowercase();
    let name = inv.name.to_lowercase();

    // Error/Result types whose variants ARE the enforcement
    let error_types = ["error", "result"];
    let is_error_type = error_types.iter().any(|e| entity.contains(e));

    // Description just explains what an enum variant means
    let variant_descriptions = [
        "represents the state where",
        "this variant",
        "enum variant",
        "represents a",
        "represents the",
        "simply wraps",
        "just describes",
    ];
    let is_variant_desc = variant_descriptions.iter().any(|p| desc.contains(p));

    // Entity::Variant naming pattern with no real transitions
    let is_single_variant_listing = name.contains("::") &&
        (name.ends_with(" state") || name.ends_with(" variant")) &&
        inv.transitions.is_empty() &&
        inv.states.len() <= 1 &&
        inv.state.is_empty();

    (is_error_type && is_variant_desc) || is_single_variant_listing
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

fn verify_citations(evidence: &[String], snippet: &str) -> f64 {
    let lines: Vec<&str> = snippet.lines().collect();
    let snippet_lower = snippet.to_lowercase();
    let mut verified = 0;
    let mut total = 0;
    for ev in evidence {
        if let Some(rest) = ev.strip_prefix("line ") {
            if let Some(colon_pos) = rest.find(':') {
                if let Ok(line_num) = rest[..colon_pos].trim().parse::<usize>() {
                    total += 1;
                    let cited_text = rest[colon_pos + 1..].trim();
                    if cited_text.is_empty() {
                        continue;
                    }
                    // Check line_num (1-indexed) and ±5 neighbors (generous window
                    // because chunks may have different line numbering than the LLM sees)
                    let idx = line_num.saturating_sub(1);
                    let start = idx.saturating_sub(5);
                    let end = (idx + 6).min(lines.len());
                    if (start..end).any(|i| lines[i].contains(cited_text)) {
                        verified += 1;
                    } else if snippet_lower.contains(&cited_text.to_lowercase()) {
                        // Content exists in snippet, just at a different line
                        verified += 1;
                    }
                }
            }
        }
    }
    if total == 0 { return 0.5; } // no line citations → neutral (don't penalize)
    verified as f64 / total as f64
}

fn downgrade_confidence(confidence: &str) -> String {
    match confidence.to_lowercase().as_str() {
        "high" => "medium".to_string(),
        "medium" => "low".to_string(),
        _ => "low".to_string(),
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
    fn test_compile_time_noise_filter() {
        let noisy = LlmInvariant {
            entity: "TokenTree".to_string(),
            name: "Conditional compilation guard".to_string(),
            states: vec![],
            state: String::new(),
            kind: "precondition".to_string(),
            description: "cfg_select macro ensures correct platform code".to_string(),

            transitions: vec![],
            evidence: vec![],
            suggested_pattern: String::new(),
            implementation_sketch: String::new(),
            confidence: "high".to_string(),
        };
        assert!(is_compile_time_noise(&noisy));

        let good = LlmInvariant {
            entity: "Connection".to_string(),
            name: "Connection state machine (Open / Closed)".to_string(),
            states: vec!["Open".to_string(), "Closed".to_string()],
            state: String::new(),
            kind: "state_machine".to_string(),
            description: "Connection must be opened before reading".to_string(),

            transitions: vec!["Open -> Closed via close()".to_string()],
            evidence: vec![],
            suggested_pattern: "typestate".to_string(),
            implementation_sketch: String::new(),
            confidence: "high".to_string(),
        };
        assert!(!is_compile_time_noise(&good));
    }

    #[test]
    fn test_enum_variant_noise_filter() {
        // Error type describing variants — should be filtered
        let error_noise = LlmInvariant {
            entity: "TryRecvError".to_string(),
            name: "TryRecvError::Empty state".to_string(),
            states: vec![],
            state: String::new(),
            kind: "state_machine".to_string(),
            description: "Represents the state where a recv could not obtain a message".to_string(),

            transitions: vec![],
            evidence: vec![],
            suggested_pattern: "newtype".to_string(),
            implementation_sketch: String::new(),
            confidence: "low".to_string(),
        };
        assert!(is_enum_variant_noise(&error_noise));

        // Real state machine — should NOT be filtered
        let real = LlmInvariant {
            entity: "Channel<T>".to_string(),
            name: "Channel state machine (Connected / Disconnected)".to_string(),
            states: vec!["Connected".to_string(), "Disconnected".to_string()],
            state: String::new(),
            kind: "state_machine".to_string(),
            description: "Channel transitions from Connected to Disconnected when all senders drop".to_string(),

            transitions: vec!["Connected -> Disconnected via disconnect()".to_string()],
            evidence: vec!["line 5: mark bit in tail".to_string()],
            suggested_pattern: "typestate".to_string(),
            implementation_sketch: String::new(),
            confidence: "medium".to_string(),
        };
        assert!(!is_enum_variant_noise(&real));
    }

    #[test]
    fn test_citation_verified() {
        let snippet = "fn foo() {\n    let x = 1;\n    if x > 0 {\n        bar();\n    }\n}";
        let evidence = vec![
            "line 2: let x = 1".to_string(),
            "line 3: if x > 0".to_string(),
        ];
        let rate = verify_citations(&evidence, snippet);
        assert!(rate > 0.9);
    }

    #[test]
    fn test_citation_wrong_line() {
        let snippet = "fn foo() {\n    let x = 1;\n}";
        let evidence = vec![
            "line 50: something that does not exist".to_string(),
        ];
        let rate = verify_citations(&evidence, snippet);
        assert!(rate < 0.1);
    }

    #[test]
    fn test_citation_no_prefix() {
        let snippet = "fn foo() {}";
        let evidence = vec![
            "the foo function does something".to_string(),
        ];
        let rate = verify_citations(&evidence, snippet);
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_downgrade_confidence() {
        assert_eq!(downgrade_confidence("high"), "medium");
        assert_eq!(downgrade_confidence("medium"), "low");
        assert_eq!(downgrade_confidence("low"), "low");
    }

    #[test]
    fn test_extract_field() {
        let text = "Name: File Handle State\nDescription: File must be opened first\nPattern: Typestate";
        assert_eq!(extract_field(text, "Name:"), Some("File Handle State".to_string()));
        assert_eq!(extract_field(text, "Description:"), Some("File must be opened first".to_string()));
        assert_eq!(extract_field(text, "Pattern:"), Some("Typestate".to_string()));
    }
}
