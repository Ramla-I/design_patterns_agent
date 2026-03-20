# Top 10 Invariants for Typestate / Linear Type Enforcement

Results from analyzing 189 C-to-Rust translated test cases (`crat_batch_20260319_181004`).
These invariants represent temporal ordering and state machine protocols currently enforced
only at runtime, where compile-time enforcement via typestate or linear types would
eliminate entire classes of bugs.

---

## 1. Analyzer Init-Before-Use Protocol

| | |
|---|---|
| **Case** | `static-vars-fpts` |
| **Type** | temporal_ordering |
| **Confidence** | high |
| **Entity** | `analyzer` (thread-local globals: `ANALYZER_OPS` / `INITIALIZED` / `TOKEN_TYPE_COUNTS`) |
| **Suggested pattern** | typestate |

`analyzer_init(ops)` must be called before `analyze_text_internal()`. Currently enforced by
a runtime `INITIALIZED` flag and panicking `expect()` on null function pointers.

**Typestate encoding:**
```
Analyzer<Uninitialized> → Analyzer<Ready>
```
Makes it impossible to call analysis functions without initialization.

---

## 2. file_queue Lifecycle (Name → Opened → Reading)

| | |
|---|---|
| **Case** | `file_queue_lib` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | `crate::src::driver::file_queue` |
| **Suggested pattern** | typestate |

Three distinct phases: `GetFile_Queue()` sets the filename, `Handle_Queue()` opens the file
handle, then seek/read operations require an open handle. Currently `fp` is `Option<File>`
unwrapped at runtime.

**Typestate encoding:**
```
FileQueue<Named> → FileQueue<Opened> → FileQueue<Reading>
```
Classic open-before-read — impossible to read before opening.

---

## 3. Authentication Session Protocol

| | |
|---|---|
| **Case** | `strcmp` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | `State` |
| **Suggested pattern** | typestate |

`State` uses `current_user: Option<usize>` plus `User.logged_in: bool`. Commands require a
logged-in user via runtime `require_logged_in()` checks.

**Typestate encoding:**
```
Session<Anonymous> → Session<Authenticated<UserId>>
```
Auth-required commands only callable on authenticated sessions.

---

## 4. Hex Decoder State Machine

| | |
|---|---|
| **Case** | `hex2bin_lib` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | `hex2bin_internal` |
| **Suggested pattern** | typestate |

Two-state automaton: at a nibble boundary it reads the high nibble; in the accumulated
state it reads the low nibble and emits a byte. Uses an integer `state` flag.

**Typestate encoding:**
```
Decoder<HighNibble> ↔ Decoder<LowNibble>
```
The alternation between high-nibble and low-nibble processing becomes a compile-time guarantee.

---

## 5. Scanf-Style Cursor Protocol

| | |
|---|---|
| **Case** | `underhanded-c-luggage` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | main parsing loop (input cursor) |
| **Suggested pattern** | session_type |

Record parsing must proceed in fixed order: timestamp → luggage_id → flight_id →
departure → arrival → comments. A record is appended only if ALL steps succeed.

**Session type encoding:**
```
Parser<NeedTimestamp> → Parser<NeedLuggageId> → Parser<NeedFlightId>
    → Parser<NeedDeparture> → Parser<NeedArrival> → Parser<NeedComments>
    → Parser<Complete>
```
Impossible to skip fields or reorder the parse sequence.

---

## 6. regex_t Compile/Free Lifecycle

| | |
|---|---|
| **Case** | `parse_uname_lib` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | `w_regexec` (`regex_t` / `re_pattern_buffer`) |
| **Suggested pattern** | RAII + typestate |

The regex must be zeroed, then `regcomp`'d, then `regexec`'d, then `regfree`'d exactly
once. Currently managed via manual runtime checks and early returns.

**Typestate encoding:**
```
Regex<Uncompiled> → Regex<Compiled>   (Drop calls regfree)
```
Prevents use-after-free, double-free, and use-before-compile.

---

## 7. ConfigFlags Init-Before-Use

| | |
|---|---|
| **Case** | `envy_lib` |
| **Type** | temporal_ordering |
| **Confidence** | high |
| **Entity** | `ConfigFlags` (via `Option<&mut ConfigFlags>`) |
| **Suggested pattern** | typestate |

Functions accept `Option<&mut ConfigFlags>` but immediately `unwrap()`, meaning `Some(...)`
is mandatory. `init_config_from_env()` must run before flags are read.

**Typestate encoding:**
```
ConfigFlags<Unset> → ConfigFlags<Initialized>
```
Eliminates the Option wrapper and the runtime initialization check.

---

## 8. Inflate/Decode Buffer Protocol

| | |
|---|---|
| **Case** | `pinflate_lib` |
| **Type** | protocol |
| **Confidence** | high |
| **Entity** | `cp_state_t` |
| **Suggested pattern** | typestate |

C-FFI inflate/decode context with raw pointers and cursors that must be initialized before
decoding, remain valid during decoding, and cleaned up after.

**Typestate encoding:**
```
DecoderCtx<Setup> → DecoderCtx<Decoding> → DecoderCtx<Done>
```
Prevents calling decode functions before buffer setup.

---

## 9. Program Instruction Stream

| | |
|---|---|
| **Case** | `tu_linkage` |
| **Type** | state_machine |
| **Confidence** | high |
| **Entity** | `Program<'a>` |
| **Suggested pattern** | typestate |

`Program` has an instruction pointer `ip` that must stay within `[0, n]` where
`n <= code.len()`. `fetch()` returns `None` when exhausted. No compile-time protection
against calling fetch after exhaustion.

**Typestate encoding:**
```
Program<Ready> → Program<Exhausted>
```
Prevents fetching from a consumed instruction stream.

---

## 10. Scanner Consumption Protocol

| | |
|---|---|
| **Case** | `memcpy-fun-buffers` |
| **Type** | protocol |
| **Confidence** | medium |
| **Entity** | `Scanner` |
| **Suggested pattern** | session_type |

Stateful token stream where `next_i32()` advances a cursor. Many call sites treat missing
tokens as fatal. The **only session_type candidate** across all 189 cases.

**Session type encoding:**
```
Scanner<UnreadInput> → Scanner<PartiallyConsumed> → Scanner<Exhausted>
```
Encodes remaining token count at the type level, making exhaustion a compile-time error.

---

## Summary

| # | Case | Pattern | Type | Confidence |
|---|------|---------|------|------------|
| 1 | `static-vars-fpts` | typestate | temporal_ordering | high |
| 2 | `file_queue_lib` | typestate | state_machine | high |
| 3 | `strcmp` | typestate | state_machine | high |
| 4 | `hex2bin_lib` | typestate | state_machine | high |
| 5 | `underhanded-c-luggage` | session_type | state_machine | high |
| 6 | `parse_uname_lib` | RAII + typestate | state_machine | high |
| 7 | `envy_lib` | typestate | temporal_ordering | high |
| 8 | `pinflate_lib` | typestate | protocol | high |
| 9 | `tu_linkage` | typestate | state_machine | high |
| 10 | `memcpy-fun-buffers` | session_type | protocol | medium |

**Pattern distribution:** typestate (7), session_type (2), RAII + typestate (1)

**Bug classes eliminated by these patterns:**
- Null dereference / use-before-init
- Use-after-free / double-free
- Protocol violations (skipped phases, reordered calls)
- State machine bugs (invalid transitions)
- Cursor exhaustion errors
