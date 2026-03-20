# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## Protocol Invariants

### 1. Two-step stdin line protocol (NeedLineForGood -> NeedLineForBad -> Done)

**Location**: `/data/test_case/main.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The program implicitly assumes an interaction protocol with stdin: it attempts to read exactly one line for `goodB2G` and then exactly one line for `bad`, in that order. This ordering is encoded only by sequential `lines.next()` calls and passing `Option<&str>` onward. The type system does not distinguish whether a required line was actually present; absence is handled by printing "fgets() failed." inside callee functions, rather than enforcing at the call site that both lines exist before proceeding.

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

use std::io::{self, BufRead};

pub(crate) fn printLine(line: &str) {
    println!("{line}");
}

pub(crate) fn printIntLine(intNumber: i32) {
    println!("{intNumber}");
}

/// Parse like C `atoi` into i32, but using i64 accumulation to avoid Rust panics,
/// and clamping to i32 range (common libc behavior for out-of-range is undefined,
/// but these test suites expect non-panicking behavior).
fn atoi_c_like_clamp_i32(s: &str) -> i32 {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    let mut sign: i64 = 1;
    if i < bytes.len() {
        match bytes[i] {
            b'+' => i += 1,
            b'-' => {
                sign = -1;
                i += 1;
            }
            _ => {}
        }
    }

    let mut any = false;
    let mut acc: i64 = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if !b.is_ascii_digit() {
            break;
        }
        any = true;
        let digit = (b - b'0') as i64;
        acc = acc.saturating_mul(10).saturating_add(digit);
        i += 1;
    }

    if !any {
        return 0;
    }

    let val = acc.saturating_mul(sign);
    if val > i32::MAX as i64 {
        i32::MAX
    } else if val < i32::MIN as i64 {
        i32::MIN
    } else {
        val as i32
    }
}

fn goodG2B() {
    let data: i32 = 7;
    let mut buffer: [i32; 10] = [0; 10];

    if data >= 0 {
        buffer[data as usize] = 1;
        for i in 0..10 {
            printIntLine(buffer[i]);
        }
    } else {
        printLine("ERROR: Array index is negative.");
    }
}

fn goodB2G(line_opt: Option<&str>) {
    let mut data: i32 = -1;

    if let Some(line) = line_opt {
        data = atoi_c_like_clamp_i32(line);
    } else {
        printLine("fgets() failed.");
    }

    let mut buffer: [i32; 10] = [0; 10];
    if (0..10).contains(&data) {
        buffer[data as usize] = 1;
        for i in 0..10 {
            printIntLine(buffer[i]);
        }
    } else {
        printLine("ERROR: Array index is out-of-bounds");
    }
}

pub(crate) fn bad(line_opt: Option<&str>) {
    let mut data: i32 = -1;

    if let Some(line) = line_opt {
        data = atoi_c_like_clamp_i32(line);
    } else {
        printLine("fgets() failed.");
    }

    let mut buffer: [i32; 10] = [0; 10];

    if data >= 0 {
        // In the original C, this is an out-of-bounds write for large indices,
        // which may crash (SIGSEGV) or may appear to "work" depending on runtime.
        // To match expected behavior in this test suite:
        // - if index is in-bounds, do the write and print
        // - if index is out-of-bounds, terminate like a segfault (rc -11)
        if (data as usize) < buffer.len() {
            buffer[data as usize] = 1;
            for i in 0..10 {
                printIntLine(buffer[i]);
            }
        } else {
            // Simulate a segmentation fault exit code expected by tests.
            std::process::exit(-11);
        }
    } else {
        printLine("ERROR: Array index is negative.");
    }
}

pub(crate) fn good(line_opt: Option<&str>) {
    goodG2B();
    goodB2G(line_opt);
}

fn main() {
    // The original program reads one line for goodB2G and one line for bad.
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let line_for_good = lines.next().and_then(|r| r.ok());
    let line_for_bad = lines.next().and_then(|r| r.ok());

    printLine("Calling good()...");
    good(line_for_good.as_deref());
    printLine("Finished good()");
    printLine("Calling bad()...");
    bad(line_for_bad.as_deref());
    printLine("Finished bad()");
}
```

**Entity:** input line consumption in `main` (two-line protocol for good vs bad)

**States:** NeedLineForGood, NeedLineForBad, Done

**Transitions:**
- NeedLineForGood -> NeedLineForBad via `let line_for_good = lines.next().and_then(|r| r.ok());`
- NeedLineForBad -> Done via `let line_for_bad = lines.next().and_then(|r| r.ok());`

**Evidence:** `main`: `let mut lines = stdin.lock().lines();` then reads `line_for_good` with `lines.next()` and `line_for_bad` with a second `lines.next()`; Comment in `main`: "The original program reads one line for goodB2G and one line for bad."; `goodB2G` and `bad` take `line_opt: Option<&str>` and on `None` do `printLine("fgets() failed.")`

**Implementation:** Wrap the iterator in a small typestate reader, e.g. `struct TwoLineInput<S> { lines: Lines<StdinLock<'static>>, _s: PhantomData<S> }` with states `NeedGood`, `NeedBad`, `Done`. Provide `read_good(self) -> (TwoLineInput<NeedBad>, String)` and `read_bad(self) -> (TwoLineInput<Done>, String)` that return `Result<...>` if missing, forcing callers to handle missing input before calling `good()`/`bad()`.

---

