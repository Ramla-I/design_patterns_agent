# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Precondition Invariants

### 1. Divisor validity for division (NonZero / PossiblyZero)

**Location**: `/data/test_case/main.rs:1-80`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The `data` value is used as a divisor in `100.0 / data`. This requires the implicit precondition that `data != 0.0` (or at least |data| > epsilon) to avoid divide-by-zero (or extreme results). In `bad()`, the divisor is unchecked and may be 0.0 because `read_float_from_stdin_or_report()` maps both I/O failure and parse failure to 0.0. In `goodB2G()`, a runtime check enforces the precondition before performing the division. The type system does not distinguish between an arbitrary `f32` and a validated non-zero divisor, so callers can accidentally divide by an invalid value.

**Evidence**:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::io::{self, Read};

pub const CHAR_ARRAY_SIZE: i32 = 20;

pub(crate) fn printLine(line: &str) {
    println!("{line}");
}

pub(crate) fn printIntLine(intNumber: i32) {
    println!("{intNumber}");
}

fn read_float_from_stdin_or_report() -> f32 {
    // Mimic fgets into a fixed-size buffer (up to CHAR_ARRAY_SIZE-1 chars + NUL in C).
    // Here we read a line and truncate to the same max character count.
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        printLine("fgets() failed.");
        return 0.0;
    }

    // Truncate to at most CHAR_ARRAY_SIZE-1 characters (leaving room for '\0' in C).
    let max_len = (CHAR_ARRAY_SIZE as usize).saturating_sub(1);
    if input.len() > max_len {
        input.truncate(max_len);
    }

    // C's atof parses leading whitespace, optional sign, number; returns 0.0 on failure.
    input.trim().parse::<f32>().unwrap_or(0.0)
}

pub(crate) fn bad() {
    let data: f32 = read_float_from_stdin_or_report();
    let result: i32 = (100.0f64 / data as f64) as i32;
    printIntLine(result);
}

fn goodG2B() {
    let data: f32 = 2.0f32;
    let result: i32 = (100.0f64 / data as f64) as i32;
    printIntLine(result);
}

fn goodB2G() {
    let data: f32 = read_float_from_stdin_or_report();

    if (data as f64).abs() > 0.000001f64 {
        let result: i32 = (100.0f64 / data as f64) as i32;
        printIntLine(result);
    } else {
        printLine("This would result in a divide by zero");
    }
}

pub(crate) fn good() {
    goodG2B();
    goodB2G();
}

fn main() {
    // Consume args to match the original pattern (though unused).
    let _args: Vec<String> = std::env::args().collect();

    printLine("Calling good()...");
    good();
    printLine("Finished good()");
    printLine("Calling bad()...");
    bad();
    printLine("Finished bad()");
}
```

**Entity:** f32 value returned by read_float_from_stdin_or_report (used as `data` in bad()/goodB2G())

**States:** PossiblyZeroOrInvalid, NonZeroValidated

**Transitions:**
- PossiblyZeroOrInvalid -> NonZeroValidated via `if (data as f64).abs() > 0.000001f64` in goodB2G()

**Evidence:** fn read_float_from_stdin_or_report(): on stdin read error prints "fgets() failed." and `return 0.0;`; fn read_float_from_stdin_or_report(): `parse::<f32>().unwrap_or(0.0)` maps parse failure to 0.0; pub(crate) fn bad(): `let result: i32 = (100.0f64 / data as f64) as i32;` with no zero/epsilon check; fn goodB2G(): `if (data as f64).abs() > 0.000001f64 { ... 100.0f64 / data as f64 ... } else { printLine("This would result in a divide by zero"); }`

**Implementation:** Introduce `struct NonZeroF32(f32);` with `TryFrom<f32>` (or `fn new(x: f32) -> Option<Self>`) enforcing `abs(x) > EPS`. Provide `impl NonZeroF32 { fn as_f64(self) -> f64 { self.0 as f64 } }`. Change division sites to accept `NonZeroF32` (e.g., `fn compute(nonzero: NonZeroF32) -> i32`). Have `read_float_from_stdin_or_report` return `Result<NonZeroF32, InputError>` (or return `Result<f32, _>` and validate separately) so `bad()` cannot compile without handling the invalid/zero case.

---

## Protocol Invariants

### 2. Input acquisition/parsing protocol (ReadOk+Parsed / FailedMappedTo0)

**Location**: `/data/test_case/main.rs:1-80`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: The function encodes multiple outcomes (stdin read failure, parse failure, successful parse) into a single `f32` by collapsing failures into the sentinel value `0.0`. This creates an implicit protocol requirement for callers: they must treat `0.0` as potentially meaning "error" rather than a legitimate numeric input, and (if used as a divisor) must validate it before use. The type system does not force callers to acknowledge or handle the failure states because the function does not return a `Result`/`Option` representing those states.

**Evidence**:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::io::{self, Read};

pub const CHAR_ARRAY_SIZE: i32 = 20;

pub(crate) fn printLine(line: &str) {
    println!("{line}");
}

pub(crate) fn printIntLine(intNumber: i32) {
    println!("{intNumber}");
}

fn read_float_from_stdin_or_report() -> f32 {
    // Mimic fgets into a fixed-size buffer (up to CHAR_ARRAY_SIZE-1 chars + NUL in C).
    // Here we read a line and truncate to the same max character count.
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        printLine("fgets() failed.");
        return 0.0;
    }

    // Truncate to at most CHAR_ARRAY_SIZE-1 characters (leaving room for '\0' in C).
    let max_len = (CHAR_ARRAY_SIZE as usize).saturating_sub(1);
    if input.len() > max_len {
        input.truncate(max_len);
    }

    // C's atof parses leading whitespace, optional sign, number; returns 0.0 on failure.
    input.trim().parse::<f32>().unwrap_or(0.0)
}

pub(crate) fn bad() {
    let data: f32 = read_float_from_stdin_or_report();
    let result: i32 = (100.0f64 / data as f64) as i32;
    printIntLine(result);
}

fn goodG2B() {
    let data: f32 = 2.0f32;
    let result: i32 = (100.0f64 / data as f64) as i32;
    printIntLine(result);
}

fn goodB2G() {
    let data: f32 = read_float_from_stdin_or_report();

    if (data as f64).abs() > 0.000001f64 {
        let result: i32 = (100.0f64 / data as f64) as i32;
        printIntLine(result);
    } else {
        printLine("This would result in a divide by zero");
    }
}

pub(crate) fn good() {
    goodG2B();
    goodB2G();
}

fn main() {
    // Consume args to match the original pattern (though unused).
    let _args: Vec<String> = std::env::args().collect();

    printLine("Calling good()...");
    good();
    printLine("Finished good()");
    printLine("Calling bad()...");
    bad();
    printLine("Finished bad()");
}
```

**Entity:** read_float_from_stdin_or_report() input parsing protocol

**States:** ReadOrParseFailedMappedToZero, ReadOkParsedToNumber

**Transitions:**
- ReadOrParseFailedMappedToZero -> (caller must branch) via checking the returned value (e.g., epsilon check in goodB2G())

**Evidence:** fn read_float_from_stdin_or_report(): `if io::stdin().read_line(&mut input).is_err() { printLine("fgets() failed."); return 0.0; }`; fn read_float_from_stdin_or_report(): comment `// C's atof ... returns 0.0 on failure.` and implementation `parse::<f32>().unwrap_or(0.0)`; fn goodB2G(): treats near-zero as invalid before division, effectively compensating for the sentinel 0.0 encoding

**Implementation:** Change `read_float_from_stdin_or_report() -> f32` to `read_float_from_stdin_or_report() -> Result<f32, InputError>` where `InputError` distinguishes I/O failure vs parse failure. If the intent is specifically 'validated divisor', return `Result<NonZeroF32, InputError>` (newtype + fallible construction). This forces `bad()`-style code to handle the error/invalid states before it can obtain a usable value.

---

