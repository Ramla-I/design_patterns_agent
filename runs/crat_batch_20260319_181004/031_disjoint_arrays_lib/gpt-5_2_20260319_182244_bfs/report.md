# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 1. fma_array slice-length precondition (len and all inputs must be >= len)

**Location**: `/data/test_case/lib.rs:1-72`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: fma_array uses `len` (after clamping to >=0) as the loop upper bound and then indexes `out[i]`, `mul1[i]`, `mul2[i]`, and `add[i]`. This requires, at runtime, that all four slices have length at least `len`. The code does not check these relationships; violating them will panic due to out-of-bounds indexing. This is an implicit validity state of the argument bundle that could be enforced by a safer API shape.

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

// === driver.rs ===
extern "C" {
    fn sscanf(__s: *const i8, __format: *const i8, ...) -> i32;
}

pub(crate) fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32) {
    let len = len.max(0) as usize;
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

pub(crate) fn call_fma(data: &[i32], len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let len_usize = len as usize;

    let mut out = vec![0i32; len_usize];
    let ones = vec![1i32; len_usize];
    let zeros = vec![0i32; len_usize];

    fma_array(&mut out, &ones, data, &zeros, len);
    out[len_usize - 1]
}

pub(crate) unsafe fn driver_internal(mut in_0: &[i8]) {
    let mut data: [i32; 100] = [0; 100];

    let mut i: usize = 0;
    while i < data.len() {
        let mut nb: usize = 0;
        let parsed = sscanf(
            in_0.as_ptr(),
            b"%d%zn\0" as *const u8 as *const i8,
            data[i..].as_mut_ptr(),
            &raw mut nb,
        );
        if parsed != 1 {
            break;
        }

        // Avoid panicking on malformed `nb` coming from C.
        if nb > in_0.len() {
            break;
        }
        in_0 = &in_0[nb..];
        i += 1;
    }

    let result: i32 = call_fma(&data, i as i32);
    println!("{result}");
}

#[no_mangle]
pub unsafe extern "C" fn driver(in_0: *const i8) {
    driver_internal(if in_0.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(in_0, 1024)
    })
}
```

**Entity:** fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32)

**States:** InvalidInputs (some slice shorter than len), ValidInputs (all slices length >= len)

**Transitions:**
- InvalidInputs -> ValidInputs by constructing/validating an argument bundle where all slices share a common length

**Evidence:** in fma_array: `let len = len.max(0) as usize; for i in 0..len { out[i] = mul1[i] * mul2[i] + add[i]; }` (indexes all slices by i without bounds checks against each slice); in call_fma: allocates `out`, `ones`, `zeros` sized to `len_usize` before calling `fma_array(&mut out, &ones, data, &zeros, len)`—showing the intended invariant is 'all slices are len_usize long'

**Implementation:** Replace `len: i32` with `len: usize` and/or take a single `&mut [i32]` plus a single `&[(i32,i32,i32)]`-like zipped input, or introduce a validated wrapper `struct FmaArgs<'a> { out: &'a mut [i32], mul1: &'a [i32], mul2: &'a [i32], add: &'a [i32] }` with `TryFrom` that checks all lengths match and exposes `len()`; then iterate over `zip`/`iter_mut` to avoid manual indexing.

---

### 3. FFI pointer+length validity precondition (null-terminated / readable buffer of fixed size)

**Location**: `/data/test_case/lib.rs:1-72`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: driver converts a raw C pointer into a Rust slice using `from_raw_parts(in_0, 1024)` when the pointer is non-null. This assumes the pointer is valid to read 1024 bytes (and properly aligned for i8) for the duration of the call. That requirement is an implicit precondition of the API; it is not captured by the type system (it accepts any `*const i8`) and violations are immediate UB. The null case is handled by mapping to an empty slice, creating two implicit input states.

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

// === driver.rs ===
extern "C" {
    fn sscanf(__s: *const i8, __format: *const i8, ...) -> i32;
}

pub(crate) fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32) {
    let len = len.max(0) as usize;
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

pub(crate) fn call_fma(data: &[i32], len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let len_usize = len as usize;

    let mut out = vec![0i32; len_usize];
    let ones = vec![1i32; len_usize];
    let zeros = vec![0i32; len_usize];

    fma_array(&mut out, &ones, data, &zeros, len);
    out[len_usize - 1]
}

pub(crate) unsafe fn driver_internal(mut in_0: &[i8]) {
    let mut data: [i32; 100] = [0; 100];

    let mut i: usize = 0;
    while i < data.len() {
        let mut nb: usize = 0;
        let parsed = sscanf(
            in_0.as_ptr(),
            b"%d%zn\0" as *const u8 as *const i8,
            data[i..].as_mut_ptr(),
            &raw mut nb,
        );
        if parsed != 1 {
            break;
        }

        // Avoid panicking on malformed `nb` coming from C.
        if nb > in_0.len() {
            break;
        }
        in_0 = &in_0[nb..];
        i += 1;
    }

    let result: i32 = call_fma(&data, i as i32);
    println!("{result}");
}

#[no_mangle]
pub unsafe extern "C" fn driver(in_0: *const i8) {
    driver_internal(if in_0.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(in_0, 1024)
    })
}
```

**Entity:** driver(in_0: *const i8)

**States:** NullPtr (treated as empty), NonNullPtrReadable(>=1024 bytes)

**Transitions:**
- NullPtr -> (empty slice) via `if in_0.is_null()`
- NonNullPtrReadable -> (slice of len 1024) via `from_raw_parts`

**Evidence:** in driver: `if in_0.is_null() { &[] } else { std::slice::from_raw_parts(in_0, 1024) }` (encodes the two input states and the unchecked read-length assumption); signature: `pub unsafe extern "C" fn driver(in_0: *const i8)` (unsafe boundary indicates caller must uphold validity invariants)

**Implementation:** Expose a safe Rust entrypoint that takes `&[u8]`/`&CStr` (capability: a validated borrow proving readability), and keep the `extern "C"` function as a thin unsafe adapter that performs validation (e.g., require a length argument from C, or use `CStr::from_ptr` and then `to_bytes_with_nul()` to bound the read) before calling the safe function.

---

## Protocol Invariants

### 2. C sscanf parsing protocol (advance by nb, bounded within remaining buffer)

**Location**: `/data/test_case/lib.rs:1-72`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: driver_internal maintains an implicit parse cursor over `in_0` and repeatedly calls C `sscanf` with `%zn` to learn how many bytes were consumed (`nb`). Correctness/safety relies on the protocol: only advance the slice by exactly `nb` bytes, and only if `nb` is within the current remaining buffer. The code enforces this with a runtime check (`if nb > in_0.len() { break; }`) and a `parsed != 1` stop condition, but the type system cannot express 'this nb came from sscanf and has been validated against this buffer'. A safer design could encapsulate the cursor and the validated advance operation so callers cannot forget the bounds check or slice update rule.

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

// === driver.rs ===
extern "C" {
    fn sscanf(__s: *const i8, __format: *const i8, ...) -> i32;
}

pub(crate) fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32], len: i32) {
    let len = len.max(0) as usize;
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

pub(crate) fn call_fma(data: &[i32], len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let len_usize = len as usize;

    let mut out = vec![0i32; len_usize];
    let ones = vec![1i32; len_usize];
    let zeros = vec![0i32; len_usize];

    fma_array(&mut out, &ones, data, &zeros, len);
    out[len_usize - 1]
}

pub(crate) unsafe fn driver_internal(mut in_0: &[i8]) {
    let mut data: [i32; 100] = [0; 100];

    let mut i: usize = 0;
    while i < data.len() {
        let mut nb: usize = 0;
        let parsed = sscanf(
            in_0.as_ptr(),
            b"%d%zn\0" as *const u8 as *const i8,
            data[i..].as_mut_ptr(),
            &raw mut nb,
        );
        if parsed != 1 {
            break;
        }

        // Avoid panicking on malformed `nb` coming from C.
        if nb > in_0.len() {
            break;
        }
        in_0 = &in_0[nb..];
        i += 1;
    }

    let result: i32 = call_fma(&data, i as i32);
    println!("{result}");
}

#[no_mangle]
pub unsafe extern "C" fn driver(in_0: *const i8) {
    driver_internal(if in_0.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(in_0, 1024)
    })
}
```

**Entity:** driver_internal(mut in_0: &[i8])

**States:** Parsing (cursor within buffer), Stopped (parse failed or nb invalid)

**Transitions:**
- Parsing -> Parsing via successful sscanf parse and validated cursor advance (`in_0 = &in_0[nb..]`)
- Parsing -> Stopped via `parsed != 1`
- Parsing -> Stopped via `nb > in_0.len()`

**Evidence:** in driver_internal: `let parsed = sscanf(..., b"%d%zn\0"..., data[i..].as_mut_ptr(), &raw mut nb); if parsed != 1 { break; }` (parse-success gate); comment: `// Avoid panicking on malformed nb coming from C.` (explicitly documents the protocol hazard); in driver_internal: `if nb > in_0.len() { break; } in_0 = &in_0[nb..];` (runtime enforcement of the cursor-advance invariant)

**Implementation:** Introduce a small cursor type `struct InputCursor<'a> { buf: &'a [u8], pos: usize }` with a method `fn advance(&mut self, n: usize) -> Option<&'a [u8]>` that performs the bounds check internally and updates `pos`. Wrap `nb` in a validated `Consumed(usize)` newtype produced only by a safe wrapper around `sscanf` that returns `Option<(i32, Consumed)>`.

---

