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

### 1. Input parsing precondition (Non-empty input & valid i32 token)

**Location**: `/data/test_case/main.rs:1-25`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: main assumes stdin contains at least one whitespace-separated token and that the first token parses as an i32. These are enforced only by runtime panics via unwrap(), not by the type system. The implicit protocol is: read stdin -> extract first token -> parse i32 -> call driver(x). If stdin is empty or the token is not an i32, the program panics before reaching driver().

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

    let x: i32 = input.split_whitespace().next().unwrap().parse().unwrap();
    driver(x);
}
```

**Entity:** main (stdin parsing pipeline)

**States:** NoInputOrInvalid, ValidI32Parsed

**Transitions:**
- NoInputOrInvalid -> ValidI32Parsed via input.split_whitespace().next().unwrap().parse().unwrap()

**Evidence:** main(): io::stdin().read_to_string(&mut input).unwrap() assumes stdin read succeeds; main(): input.split_whitespace().next().unwrap() panics if there is no first token; main(): .parse().unwrap() panics if the first token is not a valid i32; main(): driver(x) is only reached after the unwrap-based checks implicitly succeed

**Implementation:** Introduce a validated newtype like `struct ParsedI32(i32);` with `impl TryFrom<&str> for ParsedI32` (or a `fn parse_first_i32(input: &str) -> Result<i32, Error>`), and make `main` handle the Result instead of unwrapping. This moves the 'must be present and parseable' invariant into an explicit type/Result boundary.

---

