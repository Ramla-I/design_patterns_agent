# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 1

## Precondition Invariants

### 2. call_fma length protocol (len must be within data bounds and non-negative)

**Location**: `/data/test_case/main.rs:1-55`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: call_fma uses `len` both to size temporary vectors and to decide which element to return. It special-cases `len == 0`, otherwise it casts `len` to usize and indexes `out[len_usize - 1]`. This assumes `len > 0` implies `len` is non-negative and that `data` has at least `len` elements (because it passes `data` into `fma_array` with the same `len`). If `len` is negative, the cast yields a huge usize and will attempt massive allocation and/or panic; if `len > data.len()`, `fma_array` may panic on indexing. None of these constraints are captured by the type system.

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

pub(crate) fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32) {
    let len = len as usize;
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

pub(crate) fn call_fma(data: &[i32], len: i32) -> i32 {
    if len == 0 {
        return 0;
    }
    let len_usize = len as usize;

    let mut out = vec![0i32; len_usize];
    let ones = vec![1i32; len_usize];
    let zeros = vec![0i32; len_usize];

    fma_array(&mut out, &ones, data, &zeros, len);
    out[len_usize - 1]
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut data = [0i32; 100];
    let mut count = 0usize;

    for token in input.split_whitespace() {
        if count >= 100 {
            break;
        }
        if let Ok(v) = token.parse::<i32>() {
            data[count] = v;
            count += 1;
        } else {
            break;
        }
    }

    let result = call_fma(&data, count as i32);
    println!("{result}");
}
```

**Entity:** call_fma(data: &[i32], len: i32) -> i32

**States:** LenZero, LenPositiveValid, LenInvalid

**Transitions:**
- LenZero -> returns 0 via `if len == 0 { return 0; }`
- LenPositiveValid -> computes and returns last element via `out[len_usize - 1]`
- LenInvalid -> panic/OOM via `let len_usize = len as usize;`, `vec![..; len_usize]`, or downstream `fma_array` indexing

**Evidence:** call_fma: `if len == 0 { return 0; }` (distinguishes len==0 from other cases but not negative); call_fma: `let len_usize = len as usize;` (casts possibly-negative i32); call_fma: `let mut out = vec![0i32; len_usize];` (allocation size depends on unchecked len); call_fma: `fma_array(&mut out, &ones, data, &zeros, len);` (assumes data.len() >= len); call_fma: `out[len_usize - 1]` (requires len_usize > 0)

**Implementation:** Change signature to `fn call_fma(data: &[i32]) -> Option<i32>` (derive length from `data.len()` and return `None` on empty), or accept `NonZeroUsize`/`usize` instead of `i32` plus a checked constructor that ensures `len <= data.len()` and `len > 0` when indexing the last element.

---

### 1. Slice-length / bounds precondition for fma_array (len must fit all slices)

**Location**: `/data/test_case/main.rs:1-55`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: fma_array assumes that `len` is non-negative and that `out`, `mul1`, `mul2`, and `add` each have length at least `len`. This is enforced only implicitly: `len` is cast to usize and then used for indexing in a `for i in 0..len` loop. If `len` is negative it becomes a huge usize; if any slice is shorter than `len`, indexing will panic. The type system does not express the relationship between `len` and the slice lengths.

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

pub(crate) fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32) {
    let len = len as usize;
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

pub(crate) fn call_fma(data: &[i32], len: i32) -> i32 {
    if len == 0 {
        return 0;
    }
    let len_usize = len as usize;

    let mut out = vec![0i32; len_usize];
    let ones = vec![1i32; len_usize];
    let zeros = vec![0i32; len_usize];

    fma_array(&mut out, &ones, data, &zeros, len);
    out[len_usize - 1]
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut data = [0i32; 100];
    let mut count = 0usize;

    for token in input.split_whitespace() {
        if count >= 100 {
            break;
        }
        if let Ok(v) = token.parse::<i32>() {
            data[count] = v;
            count += 1;
        } else {
            break;
        }
    }

    let result = call_fma(&data, count as i32);
    println!("{result}");
}
```

**Entity:** fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32)

**States:** ValidLenAndSlices, InvalidLenOrSlices

**Transitions:**
- InvalidLenOrSlices -> panic via out[i]/mul1[i]/mul2[i]/add[i] indexing

**Evidence:** fma_array: `let len = len as usize;` (casts possibly-negative i32 to usize); fma_array: `for i in 0..len { out[i] = mul1[i] * mul2[i] + add[i]; }` (unchecked indexing requires all slices >= len)

**Implementation:** Remove the separate `len` parameter and use `for (((o, a), b), c) in out.iter_mut().zip(mul1).zip(mul2).zip(add)` (or zip chains) so the loop length is derived from slices. Alternatively introduce a `struct Len(usize)` created via a checked constructor that validates `len <= out.len()` etc., and accept `Len` instead of `i32`.

---

