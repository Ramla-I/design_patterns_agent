# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 2. Buffer length + non-aliasing/overlap preconditions for raw-pointer arithmetic

**Location**: `/data/test_case/lib.rs:1-57`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `fma_array` and `inner` operate on raw pointers with `offset` in a loop up to `len`, implicitly requiring that all involved pointers (`out`, `mul1`, `mul2`, `add`) are valid for reads/writes of `len` elements and properly aligned. In `inner`, `fma_array(out, out, out, out, len)` relies on a specific aliasing/overlap behavior: reading and writing the same buffer in-place. This is an implicit protocol that is not expressed in the type system; incorrect `len` or invalid pointers can cause UB, and the intended in-place semantics are not documented/enforced by the signature.

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
pub(crate) unsafe fn fma_array(
    out: *mut i32,
    mul1: *const i32,
    mul2: *const i32,
    add: *const i32,
    len: i32,
) {
    // Match original behavior: loop condition `i < len` naturally does nothing for len <= 0.
    let mut i: i32 = 0;
    while i < len {
        // Preserve original Rust semantics (panic on overflow in debug, wrap in release).
        // Do NOT force wrapping_* here; tests expect the original behavior.
        *out.offset(i as isize) =
            *mul1.offset(i as isize) * *mul2.offset(i as isize) + *add.offset(i as isize);
        i += 1;
    }
}

unsafe fn inner(out: *mut i32, len: i32) {
    fma_array(out, out, out, out, len);

    let mut i: i32 = 0;
    while i < len {
        println!("{0}", *out.offset(i as isize));
        i += 1;
    }
}

pub(crate) unsafe fn driver_internal(data: &[i32], len: i32) {
    let vla = len as usize;
    let mut out: Vec<i32> = ::std::vec::from_elem(0, vla);
    out[..len as usize].copy_from_slice(&data[..len as usize]);
    inner(out.as_mut_ptr(), len);
}

#[no_mangle]
pub unsafe extern "C" fn driver(data: *const i32, len: i32) {
    driver_internal(
        if data.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(data, 1024)
        },
        len,
    )
}
```

**Entity:** fma_array / inner (raw-pointer buffer processing)

**States:** ValidBuffers, InvalidBuffers

**Transitions:**
- ValidBuffers -> in-place FMA transform via `fma_array(out, out, out, out, len)` in `inner`
- ValidBuffers -> read-only printing via `println!("{0}", *out.offset(i as isize))` in `inner`
- InvalidBuffers -> UB via raw pointer deref/offset when pointers are null/unaligned/out-of-bounds

**Evidence:** `fma_array` signature: `out: *mut i32, mul1: *const i32, mul2: *const i32, add: *const i32, len: i32` (raw pointers + runtime length); `fma_array`: `*out.offset(i as isize) = *mul1.offset(i as isize) * *mul2.offset(i as isize) + *add.offset(i as isize);` requires all offsets to be in-bounds and aligned; `inner`: `fma_array(out, out, out, out, len);` demonstrates intended aliasing/in-place update; `inner`: `println!("{0}", *out.offset(i as isize));` additional raw deref requiring same validity

**Implementation:** Prefer slice-based APIs to encode bounds: `fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32])` and require equal lengths at compile-time via generic consts or at construction time via a wrapper. For the in-place case, provide an explicit `fn fma_in_place(buf: &mut [i32])` to make the aliasing requirement explicit and avoid passing four pointers that are expected to be the same.

---

### 1. FFI pointer+len validity protocol (Null vs NonNull, length bounds, and aliasing expectations)

**Location**: `/data/test_case/lib.rs:1-57`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The public `extern "C" fn driver(data: *const i32, len: i32)` implicitly requires that when `data` is non-null, it points to at least 1024 contiguous `i32`s (because `from_raw_parts(data, 1024)` is unconditionally used). Additionally, `len` is expected to be within bounds for the backing storage used later (`0 <= len <= 1024`), otherwise `driver_internal` will panic (slice indexing) or misbehave. The null case is treated as an empty slice, but negative `len` will still be cast to `usize` and can lead to huge allocations and/or panics. None of these requirements are represented in the types; they are implicit in raw-pointer handling and indexing.

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
pub(crate) unsafe fn fma_array(
    out: *mut i32,
    mul1: *const i32,
    mul2: *const i32,
    add: *const i32,
    len: i32,
) {
    // Match original behavior: loop condition `i < len` naturally does nothing for len <= 0.
    let mut i: i32 = 0;
    while i < len {
        // Preserve original Rust semantics (panic on overflow in debug, wrap in release).
        // Do NOT force wrapping_* here; tests expect the original behavior.
        *out.offset(i as isize) =
            *mul1.offset(i as isize) * *mul2.offset(i as isize) + *add.offset(i as isize);
        i += 1;
    }
}

unsafe fn inner(out: *mut i32, len: i32) {
    fma_array(out, out, out, out, len);

    let mut i: i32 = 0;
    while i < len {
        println!("{0}", *out.offset(i as isize));
        i += 1;
    }
}

pub(crate) unsafe fn driver_internal(data: &[i32], len: i32) {
    let vla = len as usize;
    let mut out: Vec<i32> = ::std::vec::from_elem(0, vla);
    out[..len as usize].copy_from_slice(&data[..len as usize]);
    inner(out.as_mut_ptr(), len);
}

#[no_mangle]
pub unsafe extern "C" fn driver(data: *const i32, len: i32) {
    driver_internal(
        if data.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(data, 1024)
        },
        len,
    )
}
```

**Entity:** driver (FFI entrypoint) / driver_internal (slice-based API)

**States:** NullInput, NonNullInputValid, NonNullInputInvalid

**Transitions:**
- NullInput -> (treated as empty slice) via `if data.is_null() { &[] }` in `driver`
- NonNullInputValid -> slice-based processing via `std::slice::from_raw_parts(data, 1024)`
- NonNullInputInvalid -> UB/panic paths if pointer is not valid for 1024 elements or if `len` is out of range

**Evidence:** `driver`: `if data.is_null() { &[] } else { std::slice::from_raw_parts(data, 1024) }` fixes required allocation size to 1024 when non-null; `driver_internal`: `out[..len as usize].copy_from_slice(&data[..len as usize]);` requires `len as usize` in-bounds for both `out` and `data`; `driver_internal`: `let vla = len as usize; let mut out: Vec<i32> = from_elem(0, vla);` shows negative `len` becomes a huge `usize` allocation request

**Implementation:** Introduce validated wrappers: `struct NonNegativeLen(usize)` with `TryFrom<i32>` rejecting negatives; and a `struct InputBlock<'a>(&'a [i32; 1024])` (or `&'a [i32]` plus a checked length) constructed only after validating the raw pointer and the intended `len`. Change `driver_internal` to accept `(InputBlock, NonNegativeLen)` so bounds/negativity are checked once at the boundary and enforced thereafter.

---

