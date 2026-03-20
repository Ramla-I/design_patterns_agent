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

### 1. Input parsing/validation protocol (RawInput -> ParsedInt -> BranchDecision)

**Location**: `/data/test_case/main.rs:1-40`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: main() relies on a multi-step runtime protocol: read stdin into a String, select the first whitespace-delimited token (or a default), parse it as i32 (or default to 0), then branch on x != 0 to decide whether to call good() or bad(). The code encodes 'invalid/missing input' as the same value as legitimate input '0' via unwrap_or("0") and unwrap_or(0), collapsing distinct states (missing token, parse failure, actual 0) into one runtime value. This protocol is not enforced by the type system: callers cannot distinguish whether x==0 came from valid input, absence, or parse failure.

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

fn printLine(line: &str) {
    if !line.is_empty() {
        println!("{line}");
    }
}

fn bad() {
    let data = "";
    printLine(data);
}

fn good() {
    let data = "string";
    printLine(data);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);

    if x != 0 {
        good();
    } else {
        bad();
    }
}
```

**Entity:** User input parsing in main() (String -> i32 decision)

**States:** RawInput, TokenSelected, ParsedInt, DefaultedToZero, BranchDecision

**Transitions:**
- RawInput -> TokenSelected via input.split_whitespace().next()
- TokenSelected -> ParsedInt via .parse()
- TokenSelected -> DefaultedToZero via .unwrap_or("0") when no token
- ParsedInt -> DefaultedToZero via .unwrap_or(0) when parse fails
- ParsedInt/DefaultedToZero -> BranchDecision via if x != 0 { good() } else { bad() }

**Evidence:** fn main: io::stdin().read_to_string(&mut input).unwrap() establishes a 'have raw input' step; fn main: input.split_whitespace().next().unwrap_or("0") defaults missing token to "0"; fn main: ...parse().unwrap_or(0) defaults parse failure to 0; fn main: if x != 0 { good(); } else { bad(); } treats x==0 as the 'bad' branch

**Implementation:** Introduce a validated input type, e.g. enum ParsedX { Present(i32), Missing, Invalid } or newtype NonZeroI32 (std::num::NonZeroI32) if the intent is 'non-zero means good'. Parse into Result/Option (or a small enum) first; only allow calling good() when you have NonZeroI32, making the 'bad() due to missing/invalid' explicit rather than conflated with numeric zero.

---

