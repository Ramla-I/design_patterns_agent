# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## State Machine Invariants

### 2. Scanf-style cursor protocol (cursor position advances only on successful conversions; record is all-or-nothing)

**Location**: `/data/test_case/main.rs:1-241`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The parsing logic encodes a multi-step protocol over a shared mutable cursor `i`: parse timestamp, then luggage_id, flight_id, departure, arrival, comments. A directive is appended only if all steps succeed in order; otherwise the entire input loop terminates (not just the current record). Correctness depends on the implicit state machine of the cursor and on the all-or-nothing nature of record parsing, but this is not represented in types—it's managed by `Option` returns and `break` control flow. Additionally, individual conversion functions have special cursor semantics (e.g., `read_i32_decimal` rewinds `i` on failure) that callers must understand.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RoutingDirective, 2 free function(s)

#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(builtin_syntax)]
#![feature(core_intrinsics)]
#![feature(derive_clone_copy)]
#![feature(hint_must_use)]
#![feature(panic_internals)]
#![feature(formatting_options)]
#![feature(coverage_attribute)]

use std::io::{self, BufRead, Write};

#[derive(Clone)]
struct RoutingDirective {
    time_stamp: u32,
    luggage_id: String,
    flight_id: String,
    departure: String,
    arrival: String,
    comments: String, // up to 80 chars, may be empty
}

// expected matches actual if expected begins with '-' OR strings equal
fn matches(expected: &str, actual: &str) -> bool {
    expected.as_bytes().first() == Some(&b'-') || expected == actual
}

// A directive is superseded if there exists a later directive with same luggage_id and departure.
fn superseded(directives: &[RoutingDirective], idx: usize) -> bool {
    let lid = &directives[idx].luggage_id;
    let dep = &directives[idx].departure;
    directives
        .iter()
        .skip(idx + 1)
        .any(|d| &d.luggage_id == lid && &d.departure == dep)
}

fn print_matching_directives(
    directives: &[RoutingDirective],
    expected_luggage_id: &str,
    expected_flight_id: &str,
    expected_departure: &str,
    expected_arrival: &str,
) {
    let mut out = io::stdout().lock();
    for i in 0..directives.len() {
        let d = &directives[i];
        if !superseded(directives, i)
            && matches(expected_luggage_id, &d.luggage_id)
            && matches(expected_flight_id, &d.flight_id)
            && matches(expected_departure, &d.departure)
            && matches(expected_arrival, &d.arrival)
        {
            let _ = writeln!(
                out,
                "{:>010} {} {} {} {} {}",
                d.time_stamp, d.luggage_id, d.flight_id, d.departure, d.arrival, d.comments
            );
        }
    }
}

fn is_allowed_lid_fid(b: u8) -> bool {
    b.is_ascii_uppercase() || b.is_ascii_digit()
}
fn is_allowed_airport(b: u8) -> bool {
    b.is_ascii_uppercase()
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

// Emulate scanf("%d"): skip leading whitespace, parse optional sign + digits.
// Returns None if no conversion (including EOF before any non-ws).
fn read_i32_decimal(bytes: &[u8], i: &mut usize) -> Option<i32> {
    skip_ws(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    let start = *i;
    if bytes[*i] == b'+' || bytes[*i] == b'-' {
        *i += 1;
    }
    let digits_start = *i;
    while *i < bytes.len() && bytes[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i == digits_start {
        *i = start;
        return None;
    }
    let s = String::from_utf8_lossy(&bytes[start..*i]);
    s.parse::<i32>().ok()
}

// Emulate scanf("%8[A-Z0-9]") / "%6[A-Z0-9]" / "%3[A-Z]":
// - skips leading whitespace
// - reads up to max_len chars from allowed set
// - fails if first char not allowed (returns None)
fn read_token_limited<F: Fn(u8) -> bool>(
    bytes: &[u8],
    i: &mut usize,
    max_len: usize,
    allowed: F,
) -> Option<String> {
    skip_ws(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    let start = *i;
    let mut len = 0usize;
    while *i < bytes.len() && len < max_len && allowed(bytes[*i]) {
        *i += 1;
        len += 1;
    }
    if len == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes[start..start + len]).into_owned())
}

// Emulate scanf("%80[^\n]"):
// - does NOT skip whitespace
// - reads up to 80 chars that are not '\n'
// - fails if next char is '\n' or EOF (conversion fails -> 0 items)
// - does NOT consume '\n'
fn read_comments_field(bytes: &[u8], i: &mut usize) -> Option<String> {
    if *i >= bytes.len() {
        return None;
    }
    if bytes[*i] == b'\n' {
        return None;
    }
    let start = *i;
    let mut len = 0usize;
    while *i < bytes.len() && len < 80 && bytes[*i] != b'\n' {
        *i += 1;
        len += 1;
    }
    if len == 0 {
        None
    } else {
        Some(String::from_utf8_lossy(&bytes[start..start + len]).into_owned())
    }
}

fn read_all<R: BufRead>(r: &mut R, out: &mut Vec<u8>) -> io::Result<()> {
    loop {
        let buf = r.fill_buf()?;
        if buf.is_empty() {
            break;
        }
        out.extend_from_slice(buf);
        let n = buf.len();
        r.consume(n);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        let _ = write!(
            io::stderr(),
            "Command line error: 4 arguments expected\n"
        );
        std::process::exit(1);
    }

    let expected_luggage_id = &args[1];
    let expected_flight_id = &args[2];
    let expected_departure = &args[3];
    let expected_arrival = &args[4];

    // Read entire stdin as bytes to emulate scanf-style parsing.
    let mut input = Vec::<u8>::new();
    let mut stdin = io::stdin().lock();
    let _ = read_all(&mut stdin, &mut input);

    let mut directives: Vec<RoutingDirective> = Vec::new();
    let mut i = 0usize;

    loop {
        let Some(ts_i32) = read_i32_decimal(&input, &mut i) else { break };
        let time_stamp = ts_i32 as u32;

        let Some(luggage_id) = read_token_limited(&input, &mut i, 8, is_allowed_lid_fid) else {
            break;
        };
        let Some(flight_id) = read_token_limited(&input, &mut i, 6, is_allowed_lid_fid) else {
            break;
        };
        let Some(departure) = read_token_limited(&input, &mut i, 3, is_allowed_airport) else {
            break;
        };
        let Some(arrival) = read_token_limited(&input, &mut i, 3, is_allowed_airport) else {
            break;
        };

        // IMPORTANT: In the original C2Rust, if scanning comments fails (e.g., immediate '\n'),
        // it BREAKS the input loop (even though comments[0]=0 was set).
        let Some(comments) = read_comments_field(&input, &mut i) else { break };

        // Do not consume the newline; next %d will skip whitespace anyway.

        directives.push(RoutingDirective {
            time_stamp,
            luggage_id,
            flight_id,
            departure,
            arrival,
            comments,
        });
    }

    // Linked-list insertion sorts by time_stamp ascending, inserting before first greater element.
    // That is stable for equal timestamps.
    directives.sort_by_key(|d| d.time_stamp);

    print_matching_directives(
        &directives,
        expected_luggage_id,
        expected_flight_id,
        expected_departure,
        expected_arrival,
    );

    std::process::exit(0);
}
```

**Entity:** main parsing loop (input cursor i over input: Vec<u8>)

**States:** At record boundary (ready to parse next directive), Mid-record (partially parsed directive), Terminated (parsing stopped)

**Transitions:**
- At record boundary -> Mid-record via successful read_i32_decimal()
- Mid-record -> Mid-record via successive successful read_token_limited()/read_comments_field() calls in fixed order
- Mid-record -> At record boundary via directives.push(...) after all fields parsed
- At record boundary or Mid-record -> Terminated via `else { break }` on any failed conversion

**Evidence:** main: `let mut i = 0usize; loop { ... }` establishes a shared mutable cursor across conversions; read_i32_decimal(bytes, &mut i) returns Option and on digit-missing failure does `*i = start; return None;` (rewind semantics that are a protocol requirement for callers); main repeatedly uses `let Some(x) = read_...( &mut i) else { break };` for each field, so any failure terminates parsing entirely; comment in main: "IMPORTANT: ... if scanning comments fails ... it BREAKS the input loop" documents the protocol/termination behavior; read_comments_field(): "does NOT skip whitespace" and "does NOT consume '\n'"—call ordering relative to whitespace/newline handling is part of the protocol, enforced only by comments and caller behavior; skip_ws() is called inside read_i32_decimal/read_token_limited but explicitly not in read_comments_field, requiring the exact field order used in main

**Implementation:** Wrap `(bytes, i)` in a `Parser<State>` type where `State` encodes the next expected field (e.g., `ExpectTs`, `ExpectLid`, `ExpectFid`, `ExpectDep`, `ExpectArr`, `ExpectComments`, `ExpectNewlineOrWs`). Each method consumes `self` and returns `Parser<NextState>` plus the parsed value, and only `Parser<ExpectComments>` exposes `read_comments_field`. Provide a `parse_directive(self) -> (self, Option<RoutingDirective>)` that guarantees all-or-nothing without relying on ad-hoc `break` logic.

---

## Precondition Invariants

### 3. RoutingDirective field validity invariants (bounded comments, structured IDs/locations)

**Location**: `/data/test_case/main.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: RoutingDirective appears to rely on implicit validity rules for its fields (e.g., comments length limit, and likely well-formed identifiers/timestamps). These constraints are not enforced by the type system: all fields are plain `String`/`u32`, so any string (including empty/oversized/ill-formed) can be constructed and propagated as a RoutingDirective. The comment indicates a concrete bound (comments up to 80 chars, may be empty), which today is only documentary and would require runtime checking elsewhere.

**Evidence**:

```rust
// Note: Other parts of this module contain: 9 free function(s)

use std::io::{self, BufRead, Write};

#[derive(Clone)]
struct RoutingDirective {
    time_stamp: u32,
    luggage_id: String,
    flight_id: String,
    departure: String,
    arrival: String,
    comments: String, // up to 80 chars, may be empty
}

```

**Entity:** RoutingDirective

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via validation/parsing (not shown in this snippet)

**Evidence:** struct RoutingDirective { ... comments: String, // up to 80 chars, may be empty } — comment documents an invariant not enforced by the type system; fields `luggage_id: String`, `flight_id: String`, `departure: String`, `arrival: String` are unrefined Strings, allowing invalid/empty/ill-formed values at compile time

**Implementation:** Introduce refined newtypes with constructors that enforce invariants: e.g., `struct Comments(SmallString<[u8; 80]>)` or `struct Comments(String)` with `TryFrom<String>` ensuring `len() <= 80`; similarly `LuggageId`, `FlightId`, `AirportCode` (for departure/arrival) as `NonEmptyString`/regex-validated newtypes. Then make `RoutingDirective` fields use these newtypes so invalid directives cannot be constructed without an explicit fallible conversion.

---

## Protocol Invariants

### 1. RoutingDirective validated-fields protocol (Parsed/Validated -> Usable for matching/printing)

**Location**: `/data/test_case/main.rs:1-241`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program assumes RoutingDirective fields obey specific format/length constraints established only by the parsing functions (scanf-emulation). After construction, other logic (matching, sorting, superseded) treats the strings as well-formed IDs/codes and the timestamp as a meaningful non-negative time. None of these constraints are encoded in the type system: RoutingDirective is constructible with arbitrary Strings and a u32 timestamp (including ones produced by lossy casts from negative i32). This is a latent protocol: directives must be created via the parser (or otherwise validated) before being stored/sorted/printed and before semantic comparisons (e.g., superseded by luggage_id+departure) are meaningful.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RoutingDirective, 2 free function(s)

#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(builtin_syntax)]
#![feature(core_intrinsics)]
#![feature(derive_clone_copy)]
#![feature(hint_must_use)]
#![feature(panic_internals)]
#![feature(formatting_options)]
#![feature(coverage_attribute)]

use std::io::{self, BufRead, Write};

#[derive(Clone)]
struct RoutingDirective {
    time_stamp: u32,
    luggage_id: String,
    flight_id: String,
    departure: String,
    arrival: String,
    comments: String, // up to 80 chars, may be empty
}

// expected matches actual if expected begins with '-' OR strings equal
fn matches(expected: &str, actual: &str) -> bool {
    expected.as_bytes().first() == Some(&b'-') || expected == actual
}

// A directive is superseded if there exists a later directive with same luggage_id and departure.
fn superseded(directives: &[RoutingDirective], idx: usize) -> bool {
    let lid = &directives[idx].luggage_id;
    let dep = &directives[idx].departure;
    directives
        .iter()
        .skip(idx + 1)
        .any(|d| &d.luggage_id == lid && &d.departure == dep)
}

fn print_matching_directives(
    directives: &[RoutingDirective],
    expected_luggage_id: &str,
    expected_flight_id: &str,
    expected_departure: &str,
    expected_arrival: &str,
) {
    let mut out = io::stdout().lock();
    for i in 0..directives.len() {
        let d = &directives[i];
        if !superseded(directives, i)
            && matches(expected_luggage_id, &d.luggage_id)
            && matches(expected_flight_id, &d.flight_id)
            && matches(expected_departure, &d.departure)
            && matches(expected_arrival, &d.arrival)
        {
            let _ = writeln!(
                out,
                "{:>010} {} {} {} {} {}",
                d.time_stamp, d.luggage_id, d.flight_id, d.departure, d.arrival, d.comments
            );
        }
    }
}

fn is_allowed_lid_fid(b: u8) -> bool {
    b.is_ascii_uppercase() || b.is_ascii_digit()
}
fn is_allowed_airport(b: u8) -> bool {
    b.is_ascii_uppercase()
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

// Emulate scanf("%d"): skip leading whitespace, parse optional sign + digits.
// Returns None if no conversion (including EOF before any non-ws).
fn read_i32_decimal(bytes: &[u8], i: &mut usize) -> Option<i32> {
    skip_ws(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    let start = *i;
    if bytes[*i] == b'+' || bytes[*i] == b'-' {
        *i += 1;
    }
    let digits_start = *i;
    while *i < bytes.len() && bytes[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i == digits_start {
        *i = start;
        return None;
    }
    let s = String::from_utf8_lossy(&bytes[start..*i]);
    s.parse::<i32>().ok()
}

// Emulate scanf("%8[A-Z0-9]") / "%6[A-Z0-9]" / "%3[A-Z]":
// - skips leading whitespace
// - reads up to max_len chars from allowed set
// - fails if first char not allowed (returns None)
fn read_token_limited<F: Fn(u8) -> bool>(
    bytes: &[u8],
    i: &mut usize,
    max_len: usize,
    allowed: F,
) -> Option<String> {
    skip_ws(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    let start = *i;
    let mut len = 0usize;
    while *i < bytes.len() && len < max_len && allowed(bytes[*i]) {
        *i += 1;
        len += 1;
    }
    if len == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes[start..start + len]).into_owned())
}

// Emulate scanf("%80[^\n]"):
// - does NOT skip whitespace
// - reads up to 80 chars that are not '\n'
// - fails if next char is '\n' or EOF (conversion fails -> 0 items)
// - does NOT consume '\n'
fn read_comments_field(bytes: &[u8], i: &mut usize) -> Option<String> {
    if *i >= bytes.len() {
        return None;
    }
    if bytes[*i] == b'\n' {
        return None;
    }
    let start = *i;
    let mut len = 0usize;
    while *i < bytes.len() && len < 80 && bytes[*i] != b'\n' {
        *i += 1;
        len += 1;
    }
    if len == 0 {
        None
    } else {
        Some(String::from_utf8_lossy(&bytes[start..start + len]).into_owned())
    }
}

fn read_all<R: BufRead>(r: &mut R, out: &mut Vec<u8>) -> io::Result<()> {
    loop {
        let buf = r.fill_buf()?;
        if buf.is_empty() {
            break;
        }
        out.extend_from_slice(buf);
        let n = buf.len();
        r.consume(n);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        let _ = write!(
            io::stderr(),
            "Command line error: 4 arguments expected\n"
        );
        std::process::exit(1);
    }

    let expected_luggage_id = &args[1];
    let expected_flight_id = &args[2];
    let expected_departure = &args[3];
    let expected_arrival = &args[4];

    // Read entire stdin as bytes to emulate scanf-style parsing.
    let mut input = Vec::<u8>::new();
    let mut stdin = io::stdin().lock();
    let _ = read_all(&mut stdin, &mut input);

    let mut directives: Vec<RoutingDirective> = Vec::new();
    let mut i = 0usize;

    loop {
        let Some(ts_i32) = read_i32_decimal(&input, &mut i) else { break };
        let time_stamp = ts_i32 as u32;

        let Some(luggage_id) = read_token_limited(&input, &mut i, 8, is_allowed_lid_fid) else {
            break;
        };
        let Some(flight_id) = read_token_limited(&input, &mut i, 6, is_allowed_lid_fid) else {
            break;
        };
        let Some(departure) = read_token_limited(&input, &mut i, 3, is_allowed_airport) else {
            break;
        };
        let Some(arrival) = read_token_limited(&input, &mut i, 3, is_allowed_airport) else {
            break;
        };

        // IMPORTANT: In the original C2Rust, if scanning comments fails (e.g., immediate '\n'),
        // it BREAKS the input loop (even though comments[0]=0 was set).
        let Some(comments) = read_comments_field(&input, &mut i) else { break };

        // Do not consume the newline; next %d will skip whitespace anyway.

        directives.push(RoutingDirective {
            time_stamp,
            luggage_id,
            flight_id,
            departure,
            arrival,
            comments,
        });
    }

    // Linked-list insertion sorts by time_stamp ascending, inserting before first greater element.
    // That is stable for equal timestamps.
    directives.sort_by_key(|d| d.time_stamp);

    print_matching_directives(
        &directives,
        expected_luggage_id,
        expected_flight_id,
        expected_departure,
        expected_arrival,
    );

    std::process::exit(0);
}
```

**Entity:** RoutingDirective

**States:** Raw/Unvalidated, Validated (field constraints satisfied)

**Transitions:**
- Raw/Unvalidated -> Validated via read_i32_decimal/read_token_limited/read_comments_field + push into directives

**Evidence:** struct RoutingDirective fields are all unconstrained primitives: time_stamp: u32, luggage_id: String, flight_id: String, departure: String, arrival: String, comments: String; comment on RoutingDirective.comments: "up to 80 chars, may be empty" but actual parsing in read_comments_field fails if len==0 (returns None), so construction relies on parser behavior rather than types; read_token_limited(..., max_len=8, is_allowed_lid_fid) used for luggage_id; max_len=6 for flight_id; max_len=3 with is_allowed_airport for departure/arrival; is_allowed_lid_fid(): only ASCII uppercase or digits; is_allowed_airport(): only ASCII uppercase; in main: let time_stamp = ts_i32 as u32; (cast from i32 without checking non-negativity) establishes an implicit 'timestamp must be non-negative' precondition not enforced by types; print_matching_directives() and superseded() compare these Strings directly (e.g., "&d.luggage_id == lid" and matches(expected, actual)) assuming the ID semantics established by parsing

**Implementation:** Introduce validated newtypes like `struct LuggageId(String)`, `struct FlightId(String)`, `struct AirportCode([u8;3])` (or `String` with invariant), and `struct Comment(String)` with constructors that enforce max length/charset. Make `RoutingDirective` store these newtypes, and implement parsing to return a `RoutingDirective` only on successful validation (e.g., `TryFrom<&str>`/`FromStr`). Also replace `ts_i32 as u32` with a checked conversion to a `struct TimeStamp(u32)` that rejects negatives.

---

