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

### 1. Vector sizing / initialization precondition for indexed writes/reads

**Location**: `/data/test_case/main.rs:1-52`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on `data` having enough elements before using indexing (`data[i]`, `data[0]`). In `good()`, `data` is created with length 10 so `data[i] = ...` and `data[0]` are valid. In `bad()`, `data` is intentionally empty to avoid UB, and the code avoids indexing entirely; the original intent (from comments) was an out-of-bounds access when treating undersized allocation as `i32[10]`. This is a latent invariant: certain operations are only valid when the vector has been sized/initialized appropriately, but the type system does not distinguish "sized for 10" vs "not sized".

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

pub(crate) fn printIntLine(intNumber: i32) {
    println!("{intNumber}");
}

pub(crate) fn bad() {
    // C2Rust version allocated 10 bytes then reinterpreted as i32 slice (too small).
    // Preserve intent (demonstrate incorrect sizing) without UB by allocating 0 i32s.
    let data: Vec<i32> = Vec::new();
    let source: [i32; 10] = [0; 10];

    // Copy would be impossible due to insufficient capacity; skip to avoid panic/UB.
    let _ = source;

    // In the original, reading data[0] is out-of-bounds; avoid UB and print 0.
    printIntLine(0);
}

pub(crate) fn good() {
    let mut data: Vec<i32> = vec![0; 10];
    let source: [i32; 10] = [0; 10];

    for i in 0..10 {
        data[i] = source[i];
    }

    printIntLine(data[0]);
}

fn main() {
    // Read an integer from stdin (similar to scanf("%d", &x)).
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

**Entity:** Vec<i32> ("data")

**States:** Insufficient length (empty or too short), Sufficient length (len >= 10)

**Transitions:**
- Insufficient length -> Sufficient length via vec![0; 10] (in good())

**Evidence:** bad(): `let data: Vec<i32> = Vec::new();` creates a 0-length vector; bad() comment: "allocated 10 bytes then reinterpreted as i32 slice (too small)" and "reading data[0] is out-of-bounds"; good(): `let mut data: Vec<i32> = vec![0; 10];` establishes required length; good(): `for i in 0..10 { data[i] = source[i]; }` relies on len >= 10; good(): `printIntLine(data[0]);` relies on len >= 1

**Implementation:** Use a fixed-size type to encode the length requirement, e.g. replace `Vec<i32>` with `[i32; 10]` where appropriate, or introduce a `struct TenI32([i32; 10]);` / `struct I32Buf10(Vec<i32>);` that can only be constructed after validating `len == 10`, and expose safe indexed access without runtime bounds risk.

---

