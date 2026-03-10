# Run Review: claude-sonnet-4-20250514 — 2026-03-10

## Run Metadata

| Field | Value |
|-------|-------|
| Model | claude-sonnet-4-20250514 |
| Date | 2026-03-10 07:33–12:39 UTC |
| Modules analyzed | 1,349 |
| Invariants emitted | 69 |
| Unique invariants (est.) | ~30–35 |
| Skipped files | 28 (parse failures) |
| Crates reached | proc_macro, panic_abort, panic_unwind, test |
| Crates NOT reached | core, alloc, std |

## Executive Summary

The first stdlib run produced 69 invariants, but signal-to-noise is low. Roughly half are duplicates, 4 contain factual errors or hallucinations, and 4 more describe compile-time properties misclassified as runtime invariants. The highest-value findings are in the panic runtime (single-use payload cleanup, abort terminal state) and proc_macro input validation (Punct/Ident/Literal preconditions). The run's biggest limitation is coverage: parse failures blocked all of `core`, `alloc`, and `std` proper, so none of the safety-critical types (Mutex, File, channels, Vec, etc.) were analyzed.

## Coverage Gap

28 files failed `syn` parsing — all in `core` and `alloc`. These are the most valuable modules:

- `core::marker` (PhantomData, Send, Sync)
- `core::pin`, `core::cell`, `core::sync::atomic`
- `alloc::vec`, `alloc::collections`
- `core::iter`, `core::slice`, `core::ops`

The feature-gate stripping retry exists but doesn't handle all nightly syntax constructs. As a result, **78% of invariants come from a single file** (`proc_macro/src/lib.rs`), which is a relatively niche crate.

## Category Breakdown

| Category | Emitted | Unique (est.) | Notes |
|----------|---------|---------------|-------|
| Precondition | 32 | ~15 | Dominated by proc_macro input validation |
| State Machine | 10 | ~3 | Mostly TokenStream Some/None (trivial) |
| Resource Lifecycle | 10 | ~5 | Panic payload cleanup is the standout |
| Temporal Ordering | 9 | ~3 | ConcatHelper builder pattern, heavily duplicated |
| Protocol | 8 | ~5 | Mixed quality; includes the best and worst findings |
| **Total** | **69** | **~30–35** | |

Confidence distribution: 24 high, 32 medium, 13 low.

## Top Invariants (by importance)

### Tier 1 — Safety-Critical (panic runtime)

#### PanicPayload::take_box() single-use (IDs #19, #20) — HIGH
`take_box()` extracts the panic payload as a raw pointer for `Box::from_raw()`. It can only be called once; a second call would produce a dangling pointer. Currently enforced by convention (`&mut dyn PanicPayload`), not by the type system. A typestate split (`PanicPayload<Active>` → `PanicPayload<Consumed>` via `take_box(self)`) would make double-take a compile error.

**Verdict**: Real invariant. High value. Typestate suggestion is appropriate.

#### Cleanup payload must-clean-once (IDs #21, #22) — HIGH
The raw `*mut u8` from platform-specific panic implementations must be cleaned via `imp::cleanup()` exactly once. Double-free is unsound; leak loses resources. RAII wrapper (`CleanupGuard`) with `Drop` is the right suggestion.

**Verdict**: Real invariant. High value. RAII suggestion is appropriate.

#### panic_abort terminal state (IDs #11, #12) — HIGH (with caveat)
Correctly identifies that `__rust_start_panic` calls `abort()` → no recovery. `__rust_panic_cleanup` contains `unreachable!()`. However, this is a crate-level property, not a per-object state machine — see bugs section.

**Verdict**: Real observation, wrong categorization.

### Tier 2 — API Safety (proc_macro)

#### Punct::new() panics on invalid char (IDs #25, #29, #58) — HIGH
`Punct::new(ch)` panics if `ch` is not in `LEGAL_CHARS`. A `ValidPunctChar` newtype would move validation to construction time, eliminating runtime panics for downstream proc macro authors.

**Verdict**: Real invariant. Practical value. Newtype suggestion appropriate. Emitted 3 times (dedup needed).

#### Literal kind-specific extraction (IDs #38–42) — HIGH
Methods like `str_value()`, `character_value()`, `byte_str_value()` return `Err(InvalidLiteralKind)` on kind mismatch. Typestate `Literal<Str>`, `Literal<Char>` etc. would make this compile-time — though it would be a large API change for proc_macro.

**Verdict**: Real invariant. Note: #38 has a factual error (see bugs section).

#### Ident validation (ID #34) — HIGH
`Ident::new()` panics on invalid identifier strings. `Ident::new_raw()` additionally rejects `_` and certain keywords. Newtype `ValidIdentStr` appropriate.

**Verdict**: Real invariant. Practical value.

#### Float literal finiteness (ID #43) — HIGH
Float literal constructors reject NaN/infinity at runtime. Newtype `FiniteFloat` appropriate.

**Verdict**: Real invariant.

### Tier 3 — Test Framework

#### RunningTest join_handle lifecycle (IDs #65, #66) — MEDIUM
Thread join handle must be consumed exactly once. Classic linear type / typestate candidate.

**Verdict**: Real invariant. Moderate value.

#### Primary/Secondary process modes (IDs #68, #69) — MEDIUM
Test runner distinguishes coordinator vs spawned child processes via env vars (`__RUST_TEST_INVOKE`). Type-level distinction would prevent mode confusion.

**Verdict**: Real invariant. Moderate value.

#### Delimiter::None precedence (IDs #55, #56) — HIGH confidence, but problematic
Documents real rustc bug #67062, but the claimed two-state machine is fabricated from a Display impl. See bugs section.

**Verdict**: Real issue exists, but the invariant description is wrong.

## Bugs and Mismatches

### Factual Errors

#### BUG 1: #38 confuses LitKind::Byte with LitKind::Char
- **Claim**: "Literal contains a byte character (`bridge::LitKind::Char`) and `byte_character_value()` will succeed"
- **Reality**: Byte characters (`b'x'`) use `LitKind::Byte`, not `LitKind::Char`. The LLM confused the two kinds. `LitKind::Char` is for regular char literals (`'x'`).
- **Impact**: A developer trusting this invariant would check the wrong kind.

#### BUG 2: #55/#56 fabricate a precedence state machine
- **Claim**: Two-state machine — Group with Delimiter::None either "preserves" or "loses" operator precedence.
- **Reality**: The code evidence is a `fmt::Display` impl. The precedence distinction is a compile-time rustc behavior, not a runtime state transition on Group objects. The LLM hallucinated the state machine from a doc comment about bug #67062.
- **Impact**: Misleading. Suggests runtime state exists where there is none.

#### BUG 3: #48 invents a semantic constraint
- **Claim**: "Group delimiter and stream are semantically consistent — content matches what the delimiter promises"
- **Reality**: No such constraint exists. A `Group` with `Delimiter::Parenthesis` can contain arbitrary tokens. There is no validation.
- **Impact**: False invariant.

#### BUG 4: #57 contradicts its own evidence
- **Claim**: "TokenStream/TokenTree can be converted to string and back without semantic loss"
- **Evidence**: Doc comment literally says "except for possibly `TokenTree::Group`s with `Delimiter::None` delimiters"
- **Impact**: Claim is directly contradicted by the cited evidence.

### Category Errors (not runtime invariants)

| ID | Issue |
|----|-------|
| #11, #12 | `panic_abort` is a crate, not a stateful object. "Initialized" means "the binary was linked with this crate" — a compile-time decision, not a runtime state. |
| #23, #24 | `cfg_select!` is compile-time conditional compilation. At runtime exactly one platform implementation exists. There is no "unselected" state. |

### Trivially Obvious / Non-Invariants

| ID | Issue |
|----|-------|
| #49 | "TokenTree variant matches its contained data" — this is how Rust enums work by definition. |
| #18 | "Error message content is unstable" — a documentation/versioning policy, not a code invariant. |
| #61, #62 | proc_macro only works inside proc macros — universally known, not a discovery. |

### Duplication

| Entity | Duplicate IDs | Should be |
|--------|--------------|-----------|
| TokenStream Empty/NonEmpty | 6, 7, 30, 31, 45, 47, 52, 53, 59, 60 | 2 invariants |
| ConcatTreesHelper builder | 1, 3, 9, 50, 63 | 1 invariant |
| ConcatStreamsHelper builder | 2, 4, 10, 64 | 1 invariant |
| Punct valid char | 25, 29, 58 | 1 invariant |
| Ident valid identifier | 26, 34 | 1 invariant |

**~30–35 of 69 invariants are duplicates.** The tool analyzes overlapping code windows and the LLM re-discovers the same patterns.

### Evidence Mismatch

Many invariants share identical code snippets despite claiming different things:
- IDs 6–8 share float-panic evidence but claim to be about TokenStream states
- IDs 25–28 share `Ident::new_raw` evidence but claim to be about Punct, Ident, Span, and TokenStream respectively
- IDs 65–69 share the test runner loop but claim five different invariants

The tool sends a module-level window and the LLM generates multiple invariants from the whole window, assigning each the same evidence block.

## Signal-to-Noise Summary

| Category | Count |
|----------|-------|
| Unique, correct, valuable invariants | ~20–25 |
| Duplicates | ~30–35 |
| Factual errors / hallucinations | 4 |
| Category errors (compile-time, not runtime) | 4 |
| Trivially obvious / non-invariants | 4–5 |
| Evidence contradicts claim | 1 |
| **Effective signal rate** | **~30–36%** |

## Recommendations

See companion implementation plan for details. In priority order:

1. **Fix parsing** of core/alloc/std — this is the #1 blocker to useful results
2. **Deduplicate** invariants post-emission by (entity, normalized state)
3. **Narrow chunk scope** to one type + its impls per chunk (prevents multi-entity evidence confusion)
4. **Add a validation pass** — second LLM call to verify evidence supports claim
5. **Filter compile-time constructs** — reject cfg/cfg_if/cfg_select, trivial enum correctness
6. **Calibrate confidence** — require specific line citations; downgrade if LLM can't point to concrete code
7. **Rebalance coverage** — prioritize core/std safety-critical modules over proc_macro
