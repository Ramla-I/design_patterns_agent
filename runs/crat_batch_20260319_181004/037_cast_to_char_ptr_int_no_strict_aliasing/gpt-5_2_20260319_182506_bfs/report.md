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

### 1. Input parse preconditions (Non-empty, valid i32)

**Location**: `/data/test_case/main.rs:1-31`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The program relies on a multi-step runtime protocol: read all of stdin into a String, extract the first whitespace-separated token, and parse it as i32. Failure at any step currently panics via unwrap(), and the type system does not encode the requirement that the token exists and is a valid i32. The code also implicitly treats 'missing token' as the special value "0" via unwrap_or("0"), which is a semantic default not reflected in types.

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
    for b in bytes {
        print!("{:02x}", b);
    }
    println!();
}

fn driver(x: i32) {
    let bytes = x.to_ne_bytes(); // matches memcpy of i32 into raw bytes (native endianness)
    print_hex(&bytes);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap();
    driver(x);
}
```

**Entity:** stdin input parsing (String -> i32)

**States:** RawInput(String), Tokenized(&str), Parsed(i32)

**Transitions:**
- RawInput(String) -> Tokenized(&str) via split_whitespace().next().unwrap_or("0")
- Tokenized(&str) -> Parsed(i32) via parse().unwrap()

**Evidence:** main(): io::stdin().read_to_string(&mut input).unwrap() (panic if read fails); main(): input.split_whitespace().next().unwrap_or("0") (implicit 'must have token' protocol with default); main(): .parse().unwrap() (panic if token is not a valid i32)

**Implementation:** Introduce a validated type like `struct ParsedI32(i32);` with `TryFrom<&str>` (or `FromStr`) returning `Result<ParsedI32, ParseIntError>`, and plumb `Result` through main/driver instead of `unwrap()`. Optionally introduce `NonEmptyToken<'a>(&'a str)` newtype from `split_whitespace().next()` to encode presence before parsing.

---

