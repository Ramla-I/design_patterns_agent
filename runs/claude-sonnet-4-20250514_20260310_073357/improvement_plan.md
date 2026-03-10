# Improvement Plan: design_patterns_agent

Based on the first stdlib run review. 7 improvements in 3 phases.

## Dependency Graph

```
Phase 1 (independent, high-impact):
  5. Filter compile-time constructs  [small]
  1. Fix nightly parsing              [medium]
  7. Coverage rebalancing             [small]

Phase 2 (builds on Phase 1):
  3. Narrow chunk scope               [large]
  2. Post-emission dedup              [medium, benefits from #3]

Phase 3 (polish):
  6. Confidence calibration           [medium]
  4. Validation pass                  [large, benefits from #2 and #5 reducing volume]
```

---

## Phase 1: Quick Wins + Unblock Coverage

### Improvement 5: Filter Compile-time Constructs

**Complexity:** Small
**Files:** `src/detection/invariant_inference.rs`

The problem: invariants about `cfg_select`, enum variant correctness, and documentation policies are noise.

**Changes:**

1. Add to `SYSTEM_PROMPT`:
   ```
   Do NOT report invariants about: conditional compilation (cfg/cfg_if/cfg_select),
   enum variant exhaustiveness, documentation policies, or feature gates.
   These are compile-time constructs, not latent runtime invariants.
   ```

2. Add a post-parse filter function:
   ```rust
   fn is_compile_time_noise(inv: &LlmInvariant) -> bool {
       let text = format!("{} {} {}", inv.entity, inv.name, inv.description).to_lowercase();
       let noise_patterns = [
           "cfg_select", "cfg_if", "cfg_attr", "feature gate",
           "conditional compilation", "variant correctness",
           "enum exhaustiveness", "documentation policy",
           "doc comment", "api stability",
       ];
       noise_patterns.iter().any(|p| text.contains(p))
   }
   ```

3. Apply in `try_parse_json` after deserialization, before converting to `Invariant`.

---

### Improvement 1: Fix Parsing of core/alloc/std

**Complexity:** Medium
**Files:** `src/parser/mod.rs`

The current `parse_file_tolerant` strips `#![feature(...)]` and `#![cfg_attr(...)]` but 28 files still fail. Nightly stdlib uses constructs `syn` (stable) can't handle.

**Changes to `parse_file_tolerant`:**

1. **Multi-pass stripping** (extend existing retry logic):
   - Pass 1 (existing): strip `#![feature(...)]` and `#![cfg_attr(...)]` single lines
   - Pass 2: strip `cfg_select! { ... }` blocks using brace-depth counting
   - Pass 3: replace `unsafe extern "C" {` → `extern "C" {` (edition 2024 syntax)
   - Pass 4: strip remaining `#![...]` inner attributes (except `#[doc]`, `#[allow]`)

2. **Item-level fallback** (new):
   If all passes fail, split by top-level braces, parse each item individually with `syn::parse_str::<syn::Item>()`, collect whatever succeeds. Partial results beat total failure.

3. **Better error logging**: capture `syn` error span position and report which specific line/construct caused the failure.

**Tests:** Unit tests with `cfg_select!`, `unsafe extern`, multi-line feature gates.

---

### Improvement 7: Coverage Rebalancing

**Complexity:** Small
**Files:** `src/navigation/priority.rs`, `src/agent/mod.rs`

Priority modules get sorted first but may still be skipped if the budget runs out on other high-scoring chunks.

**Changes:**

1. In `priority.rs` `score_chunk`: boost priority module match from +100 → +1000.

2. In `agent/mod.rs`: partition chunks into `priority_chunks` and `remaining_chunks`. Process all priority chunks first (budget-exempt), then remaining with budget checking.

3. Add coverage stats to report footer:
   ```
   Priority modules analyzed: 8/8
   Other modules analyzed: 120/341
   Token budget used: 850K/1M
   ```

---

## Phase 2: Quality Improvements

### Improvement 3: Narrow Chunk Scope to Single Entity

**Complexity:** Large
**Files:** `src/navigation/context.rs`, `src/parser/ast.rs`, `src/detection/invariant_inference.rs`, `Cargo.toml`

The core change: every chunk sent to the LLM should focus on one type + its impls, not a full module.

**Changes:**

1. **Always cluster by type affinity** in `build_chunks` (context.rs). Remove the "fits in one chunk" fast-path that bypasses clustering. Every module goes through `cluster_by_type_affinity`.

2. **Fix line number tracking** (ast.rs). Currently all `SourceLocation`s are hardcoded to `line: 1`. Use `syn::spanned::Spanned` to get real line numbers:
   ```rust
   SourceLocation { line: item.ident.span().start().line }
   ```
   Add `proc-macro2` with feature `span-locations` to `Cargo.toml`.

3. **Improve source extraction per cluster.** Use real line numbers from step 2 to extract exact spans from raw source, instead of the current name-mention heuristic with +/-5 line context.

4. **Update LLM prompt** in `invariant_inference.rs`: change "analyze the following Rust module" → "Focus on the primary entity in this chunk." Sibling summaries still provide module context.

---

### Improvement 2: Post-emission Deduplication

**Complexity:** Medium
**Files:** `src/report/mod.rs`, `src/detection/invariant_inference.rs`, `src/agent/mod.rs`

**Changes:**

1. Add `entity: String` field to `Invariant` struct in `report/mod.rs`:
   ```rust
   pub struct Invariant {
       pub entity: String,  // new
       // ... existing fields
   }
   ```
   Use `#[serde(default)]` for backward compat with existing JSONL.

2. In `invariant_inference.rs` `try_parse_json`: propagate `entity` from `LlmInvariant` to `Invariant`.

3. New function in `report/mod.rs`:
   ```rust
   pub fn deduplicate(mut invariants: Vec<Invariant>) -> Vec<Invariant> {
       // Key: (entity_lowercase, state_normalized, invariant_type)
       // Keep highest confidence; if tied, longest evidence
       // Merge discarded evidence into kept invariant's explanation
   }
   ```
   Normalization: lowercase, collapse whitespace, strip punctuation.

4. Call `deduplicate()` in `agent/mod.rs` after all invariants are collected, before report generation.

---

## Phase 3: Advanced Quality

### Improvement 6: Confidence Calibration via Citation Verification

**Complexity:** Medium
**Files:** `src/detection/invariant_inference.rs`

**Changes:**

1. **Modify LLM prompt** to require line citations:
   ```
   Each evidence item MUST start with 'line N:' citing the specific line number.
   Example: "line 15: if !self.is_open { return Err(...) }"
   ```

2. **Add programmatic verification** after parsing each `LlmInvariant`:
   ```rust
   fn verify_citations(inv: &LlmInvariant, snippet: &str) -> f64 {
       // For each evidence item, extract "line N:" prefix
       // Check if cited text appears on/near that line in snippet
       // Return fraction of verified citations
   }
   ```

3. **Downgrade confidence** based on verification rate:
   - < 50% verified → force Low
   - < 80% verified → downgrade one level (High→Medium, Medium→Low)
   - ≥ 80% verified → keep as-is

---

### Improvement 4: Validation Pass (Second LLM Call)

**Complexity:** Large
**Files:** new `src/detection/validation.rs`, `src/detection/mod.rs`, `src/agent/mod.rs`, `src/cli/config.rs`

Gate behind `--validate` flag (off by default, doubles LLM cost).

**Changes:**

1. New `validation.rs` with system prompt:
   ```
   You are a critical reviewer. Given a claimed invariant and its evidence code:
   (a) Does the evidence actually contain the cited behavior?
   (b) Is this a runtime invariant or a compile-time/type-system guarantee?
   (c) Is the claim specific, non-trivial, and non-obvious?
   Respond: {"valid": bool, "reason": "...", "adjusted_confidence": "high|medium|low"}
   ```

2. Temperature 0.1 (yes/no judgment, not creative).

3. Each validation call is small (~200–500 tokens). For 69 invariants → ~15–35K tokens total.

4. Filter out `valid == false`. Adjust confidence for the rest.

5. Add `--validate` flag to CLI config. Default false.

---

## Summary

| # | Improvement | Phase | Complexity | Expected Impact |
|---|------------|-------|-----------|----------------|
| 5 | Filter compile-time | 1 | Small | -5–10% noise |
| 1 | Fix nightly parsing | 1 | Medium | +28 files → 2–3x more coverage |
| 7 | Coverage rebalancing | 1 | Small | Guarantees priority modules analyzed |
| 3 | Narrow chunk scope | 2 | Large | -50% duplication, better evidence |
| 2 | Post-emission dedup | 2 | Medium | Merges remaining duplicates |
| 6 | Confidence calibration | 3 | Medium | Downgrades hallucinations |
| 4 | Validation pass | 3 | Large | Catches remaining false positives |

**Expected outcome after all phases:** Signal rate improves from ~30% to ~70–80%. Coverage expands from 4 crates to full stdlib. Duplicate count drops from ~50% to <5%.
