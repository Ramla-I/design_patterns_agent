# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 1. Wide-string slice preconditions (NUL-terminated src; dst contains NUL within cap; dst capacity sufficient)

**Location**: `/data/test_case/lib.rs:1-65`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: wcscat_internal implicitly treats `dst` and `src` as C wide-strings: `dst` must contain a terminating 0 within `cap = min(numElem, dst.len())` so the scan for the end of string is bounded; `src` must be NUL-terminated because the append loop reads `src` with `get_unchecked(j)` until it finds a 0. Additionally, `dst` must have enough capacity to fit `src` including the terminating 0; otherwise the function reports an error and clears `dst[0]`. These are protocol/precondition requirements encoded via runtime checks and unsafe unchecked indexing rather than enforced by types.

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

extern crate libc;

pub type wchar_t = libc::wchar_t;

pub(crate) unsafe fn wcscat_internal(dst: &mut [i32], numElem: usize, src: &[i32]) -> i32 {
    if dst.is_empty() || numElem == 0 {
        return 22;
    }
    if src.is_empty() {
        dst[0] = 0;
        return 22;
    }

    let cap = numElem.min(dst.len());

    // Find end of current dst string within bounds.
    let mut i = 0usize;
    while i < cap && dst[i] != 0 {
        i += 1;
    }

    // Append src (including its terminating 0) while space remains.
    let mut j = 0usize;
    while i < cap {
        let ch = *src.get_unchecked(j);
        dst[i] = ch;
        i += 1;
        j += 1;

        if ch == 0 {
            return 0;
        }
    }

    // Not enough space: clear dst and report error.
    dst[0] = 0;
    34
}

#[no_mangle]
pub unsafe extern "C" fn wcscat(dst: *mut i32, numElem: usize, src: *const i32) -> i32 {
    wcscat_internal(
        if dst.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dst, 1024)
        },
        numElem,
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** wcscat_internal(dst: &mut [i32], numElem: usize, src: &[i32])

**States:** ValidInputs, EINVAL (invalid/empty inputs), ERANGE (insufficient capacity)

**Transitions:**
- ValidInputs -> ValidInputs (success) when appended NUL encountered (returns 0)
- ValidInputs -> ERANGE when `cap` reached before copying NUL (sets dst[0]=0, returns 34)
- Any -> EINVAL when `dst.is_empty() || numElem == 0` (returns 22)
- Any -> EINVAL when `src.is_empty()` (writes dst[0]=0, returns 22)

**Evidence:** wcscat_internal: `if dst.is_empty() || numElem == 0 { return 22; }` runtime input validity gate; wcscat_internal: `if src.is_empty() { dst[0] = 0; return 22; }` treats empty src as invalid and mutates dst; wcscat_internal: `let cap = numElem.min(dst.len());` establishes a logical bounds contract separate from slice length; wcscat_internal: scan loop `while i < cap && dst[i] != 0` assumes dst is NUL-terminated within cap for C-string semantics; wcscat_internal: append loop uses `let ch = *src.get_unchecked(j);` and stops only when `ch == 0` — requires src be NUL-terminated to avoid OOB UB; wcscat_internal: on overflow `dst[0] = 0; 34` encodes an error-state transition and side-effect protocol

**Implementation:** Introduce validated wrapper types like `struct WideCStr<'a>(&'a [i32]);` (guaranteed NUL-terminated) and `struct WideCStrMut<'a>{ buf: &'a mut [i32], cap: usize }` (guaranteed non-empty, cap<=len, and contains a NUL within cap). Provide `TryFrom` constructors that scan/validate once; then `wcscat_internal(dst: WideCStrMut, src: WideCStr) -> Result<(), WcscatError>` can remove `get_unchecked` and runtime state checks from the core logic.

---

## Protocol Invariants

### 2. FFI pointer/length protocol (non-null, valid for `numElem`, properly terminated strings)

**Location**: `/data/test_case/lib.rs:1-65`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The extern "C" wrapper encodes an implicit FFI contract: if pointers are non-null, they must point to valid memory for at least the number of elements the function will touch, and the sequences must represent NUL-terminated wide strings. However the wrapper constructs slices of fixed length 1024 regardless of `numElem`, and uses empty slices to represent null pointers, relying on wcscat_internal's runtime checks to turn these into error codes. The required pointer validity/size/termination properties are not enforced by the type system and are partially violated/obscured by the fixed-size slice construction.

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

extern crate libc;

pub type wchar_t = libc::wchar_t;

pub(crate) unsafe fn wcscat_internal(dst: &mut [i32], numElem: usize, src: &[i32]) -> i32 {
    if dst.is_empty() || numElem == 0 {
        return 22;
    }
    if src.is_empty() {
        dst[0] = 0;
        return 22;
    }

    let cap = numElem.min(dst.len());

    // Find end of current dst string within bounds.
    let mut i = 0usize;
    while i < cap && dst[i] != 0 {
        i += 1;
    }

    // Append src (including its terminating 0) while space remains.
    let mut j = 0usize;
    while i < cap {
        let ch = *src.get_unchecked(j);
        dst[i] = ch;
        i += 1;
        j += 1;

        if ch == 0 {
            return 0;
        }
    }

    // Not enough space: clear dst and report error.
    dst[0] = 0;
    34
}

#[no_mangle]
pub unsafe extern "C" fn wcscat(dst: *mut i32, numElem: usize, src: *const i32) -> i32 {
    wcscat_internal(
        if dst.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dst, 1024)
        },
        numElem,
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** wcscat(dst: *mut i32, numElem: usize, src: *const i32)

**States:** NullPointerInputs, NonNullButInsufficientlyBackedMemory, ValidFFIInputs

**Transitions:**
- NullPointerInputs -> EINVAL behavior by mapping null to empty slices (delegated to wcscat_internal returning 22)
- ValidFFIInputs -> delegated success/ERANGE outcomes from wcscat_internal

**Evidence:** wcscat: `if dst.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dst, 1024) }` maps null to empty and otherwise assumes at least 1024 writable elements; wcscat: `if src.is_null() { &[] } else { std::slice::from_raw_parts(src, 1024) }` likewise assumes at least 1024 readable elements; wcscat: passes `numElem` separately from the created slice length, indicating an unstated invariant that the pointed-to memory is valid for `min(numElem, 1024)` elements; wcscat_internal ultimately does `src.get_unchecked(j)` until NUL, so the FFI-level contract must include NUL termination within the readable region

**Implementation:** Expose a safe Rust API that requires capabilities/wrappers proving pointer validity and bounds, e.g. `fn wcscat_safe(dst: WideCStrMut, src: WideCStr) -> Result<(), WcscatError>`. Keep the `extern "C"` function as a thin adapter that validates pointers/lengths (e.g., using `numElem` to form slices, not a hardcoded 1024) before constructing the capability types; the unchecked/unsafe remains only in the adapter.

---

