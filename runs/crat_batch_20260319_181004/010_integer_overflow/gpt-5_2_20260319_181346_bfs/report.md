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

### 1. Input byte validity protocol (ASCII/byte domain -> signed char semantics)

**Location**: `/data/test_case/main.rs:1-29`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The program implements an implicit protocol for deriving a single character-like value from stdin: read all input, select the first non-whitespace byte if any, otherwise default to space. It then treats that byte as an `i8` (signed) and performs wrapping arithmetic. This relies on runtime/defaulting behavior and unchecked casts: converting a `u8` byte to `i8` can produce negative values for bytes >= 128, which changes the meaning of the subsequent `wrapping_add(1)` and the later hex printing (via `as u8`). The type system does not express whether the chosen byte is guaranteed ASCII (0..=127) vs arbitrary (0..=255), nor does it encode whether the value came from real input vs the fallback.

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

pub(crate) fn printHexCharLine(charHex: i8) {
    println!("{:>02x}", charHex as u8);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let data: i8 = input
        .bytes()
        .find(|&b| !b.is_ascii_whitespace())
        .map(|b| b as i8)
        .unwrap_or(b' ' as i8);

    let result: i8 = data.wrapping_add(1);
    printHexCharLine(result);
}
```

**Entity:** main input parsing (String -> first non-whitespace byte -> i8)

**States:** NoInputOrAllWhitespace, HasFirstNonWhitespaceByte

**Transitions:**
- NoInputOrAllWhitespace -> HasFirstNonWhitespaceByte via bytes().find(|b| !b.is_ascii_whitespace())
- NoInputOrAllWhitespace -> HasFirstNonWhitespaceByte via unwrap_or(b' ')
- HasFirstNonWhitespaceByte -> (incremented) via wrapping_add(1)

**Evidence:** main: io::stdin().read_to_string(&mut input).unwrap() (all input is read into a String); main: input.bytes().find(|&b| !b.is_ascii_whitespace()) (selects first non-whitespace byte); main: .map(|b| b as i8) (unchecked cast u8 -> i8; bytes >= 128 become negative); main: .unwrap_or(b' ' as i8) (fallback state: no input/all whitespace becomes space); main: let result: i8 = data.wrapping_add(1) (wraparound arithmetic depends on signedness)

**Implementation:** Introduce a newtype representing the intended domain, e.g. `struct AsciiNonWhitespace(u8);` with `TryFrom<u8>` checking `b.is_ascii() && !b.is_ascii_whitespace()`. Parse into `Option<AsciiNonWhitespace>` (or an enum distinguishing FallbackSpace vs FromInput) and only then perform increment/printing on `u8` (or on the newtype) to make the domain and fallback explicit at compile time.

---

