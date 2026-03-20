# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 1

## Precondition Invariants

### 1. Signed-byte arithmetic safety precondition (No-overflow doubling)

**Location**: `/data/test_case/main.rs:1-70`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on a value-range precondition before performing `data * 2` and casting back to `i8`. In `bad()` and `goodG2B()` the multiplication is done unconditionally (modulo the `data > 0` check), which can overflow the `i8` domain after the cast. In `goodB2G()` the code enforces at runtime that `data < CHAR_MAX/2` before doubling. This safety requirement ("only call/double when value is in range") is not encoded in the type system; `data: i8` can represent both safe and unsafe-to-double values.

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

pub const __SCHAR_MAX__: i32 = 127;
pub const CHAR_MAX: i32 = __SCHAR_MAX__;

fn printLine(line: &str) {
    println!("{line}");
}

fn printHexCharLine(charHex: i8) {
    // Match original behavior: treat as unsigned byte when formatting.
    let v = (charHex as u8) as u32;
    println!("{:02x}", v);
}

fn bad() {
    let data: i8 = CHAR_MAX as i8;
    if data > 0 {
        let result: i8 = (data as i32 * 2) as i8;
        printHexCharLine(result);
    }
}

fn goodG2B() {
    let data: i8 = 2;
    if data > 0 {
        let result: i8 = (data as i32 * 2) as i8;
        printHexCharLine(result);
    }
}

fn goodB2G() {
    let data: i8 = CHAR_MAX as i8;
    if data > 0 {
        if (data as i32) < CHAR_MAX / 2 {
            let result: i8 = (data as i32 * 2) as i8;
            printHexCharLine(result);
        } else {
            printLine("data value is too large to perform arithmetic safely.");
        }
    }
}

fn good() {
    goodG2B();
    goodB2G();
}

fn main() {
    // C version reads an int via scanf-like behavior; emulate by reading stdin and parsing first token.
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).ok();
    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);

    if x != 0 {
        good();
    } else {
        bad();
    }
}
```

**Entity:** i8 (used as `data` in bad/goodG2B/goodB2G)

**States:** SafeToDouble, TooLargeToDouble

**Transitions:**
- TooLargeToDouble -> SafeToDouble via runtime check `(data as i32) < CHAR_MAX / 2` in goodB2G()
- SafeToDouble -> (produce result) via `let result: i8 = (data as i32 * 2) as i8`

**Evidence:** bad(): `let data: i8 = CHAR_MAX as i8;` followed by `let result: i8 = (data as i32 * 2) as i8;` without a range check; goodG2B(): `let data: i8 = 2;` then `let result: i8 = (data as i32 * 2) as i8;` (implicitly assumes '2' is safe); goodB2G(): runtime guard `if (data as i32) < CHAR_MAX / 2 { ... } else { printLine("data value is too large to perform arithmetic safely."); }`

**Implementation:** Introduce a newtype representing a value proven safe to double, e.g. `struct SafeToDoubleI8(i8); impl TryFrom<i8> for SafeToDoubleI8 { /* check v>0 && v<=i8::MAX/2 */ }` and provide `fn double(self) -> i8` (or `i16`) on the newtype. Then `bad()`/`goodB2G()` must construct `SafeToDoubleI8` before doubling, pushing the check to construction time and preventing unchecked doubling sites.

---

