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

### 1. Input parsing preconditions (non-empty token, valid i32)

**Location**: `/data/test_case/main.rs:1-31`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The program relies on a multi-step runtime protocol to obtain an i32: (1) read stdin into a String, (2) select the first whitespace-delimited token or default to "0", (3) parse that token as i32. Correctness depends on the token being a valid i32 representation; otherwise the program panics via unwrap(). These preconditions and transitions are not represented in the type system; intermediate states are plain String/&str, and failures are handled by panics rather than typed states/results.

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

fn print_hex(bytes: &[u8]) {
    for &b in bytes {
        print!("{:02x}", b);
    }
    println!();
}

fn driver(x: i32) {
    let bytes = x.to_ne_bytes();
    print_hex(&bytes);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap();
    driver(x);
}
```

**Entity:** main() input parsing pipeline (String -> i32)

**States:** RawInput, TokenSelected, ParsedI32

**Transitions:**
- RawInput -> TokenSelected via input.split_whitespace().next().unwrap_or("0")
- TokenSelected -> ParsedI32 via .parse().unwrap()
- ParsedI32 -> (used) via driver(x)

**Evidence:** line: io::stdin().read_to_string(&mut input).unwrap() — assumes stdin read succeeds; panics otherwise; line: input.split_whitespace().next().unwrap_or("0") — implicit state: token may be missing, defaulting to "0"; line: ...parse().unwrap() — precondition: selected token must be a valid i32, otherwise panic

**Implementation:** Introduce a newtype like `struct ParsedI32(i32);` with `impl TryFrom<&str> for ParsedI32` returning `Result<ParsedI32, ParseIntError>`. Make `driver` take `ParsedI32` (or return `Result` from main) so the parse-success state is explicit and panics are avoided.

---

