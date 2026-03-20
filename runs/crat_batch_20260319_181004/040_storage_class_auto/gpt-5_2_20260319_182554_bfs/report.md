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

### 1. Stdin input parsing precondition (Non-empty & valid i32)

**Location**: `/data/test_case/main.rs:1-25`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: main() implicitly assumes stdin contains at least one whitespace-separated token and that this token parses as an i32. These conditions are enforced only via runtime fallbacks/panics: missing token falls back to "0", while invalid integer format causes a panic via unwrap(). The type system does not distinguish between 'validated integer input' and 'unvalidated/raw string', so the parsing/validation protocol can be accidentally bypassed or partially applied.

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

pub(crate) fn driver(x: i32) {
    let mut y: i32 = 2 * x;
    y += 300;
    println!("{y}");
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap();
    driver(x);
}
```

**Entity:** main() input parsing pipeline (String -> token -> i32)

**States:** RawInput, HasFirstToken, ParsedI32

**Transitions:**
- RawInput -> HasFirstToken via split_whitespace().next().unwrap_or("0")
- HasFirstToken -> ParsedI32 via parse().unwrap()

**Evidence:** main(): io::stdin().read_to_string(&mut input).unwrap() (assumes stdin read succeeds); main(): input.split_whitespace().next().unwrap_or("0") (encodes 'missing token' state with a fallback string); main(): .parse().unwrap() (panics if token is not a valid i32)

**Implementation:** Introduce a validated newtype, e.g. `struct ParsedI32(i32); impl TryFrom<&str> for ParsedI32 { ... }`, and have parsing return `Result<ParsedI32, ParseError>` (or `Option<ParsedI32>` if defaulting is desired). Pass `ParsedI32` (or `i32` extracted from it) to `driver`, making the 'validated' state explicit at the type level and avoiding unwrap-based preconditions.

---

