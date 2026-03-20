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

### 1. bin2hex_internal buffer/length precondition protocol (sufficient capacity + consistent lengths)

**Location**: `/data/test_case/lib.rs:1-58`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: bin2hex_internal relies on a set of implicit preconditions tying together (hex, hex_maxlen, bin, bin_len): (1) bin_len must not overflow when doubled, (2) hex_maxlen must be strictly greater than 2*bin_len (needs room for NUL), (3) the provided slices must be at least bin_len and (2*bin_len+1) respectively. These are enforced by runtime aborts, not by the type system. When satisfied, the function writes exactly 2*bin_len bytes of hex plus a NUL terminator and returns hex.as_mut_ptr().

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

pub(crate) fn bin2hex_internal(
    hex: &mut [i8],
    hex_maxlen: usize,
    bin: &[u8],
    bin_len: usize,
) -> *mut i8 {
    // Need space for 2*bin_len hex chars + NUL terminator.
    if bin_len > (usize::MAX / 2) || hex_maxlen <= bin_len * 2 {
        std::process::abort();
    }
    if hex.len() < bin_len * 2 + 1 || bin.len() < bin_len {
        std::process::abort();
    }

    const HEX: &[u8; 16] = b"0123456789abcdef";

    for (i, &byte) in bin[..bin_len].iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0x0f) as usize;
        hex[i * 2] = HEX[hi] as i8;
        hex[i * 2 + 1] = HEX[lo] as i8;
    }

    hex[bin_len * 2] = 0;
    hex.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn bin2hex(
    hex: *mut i8,
    hex_maxlen: usize,
    bin: *const u8,
    bin_len: usize,
) -> *mut i8 {
    let hex_slice = if hex.is_null() {
        &mut []
    } else {
        // Preserve original behavior/assumptions about available memory.
        std::slice::from_raw_parts_mut(hex, 1024)
    };
    let bin_slice = if bin.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(bin, 1024)
    };

    bin2hex_internal(hex_slice, hex_maxlen, bin_slice, bin_len)
}
```

**Entity:** bin2hex_internal(hex: &mut [i8], hex_maxlen: usize, bin: &[u8], bin_len: usize)

**States:** PreconditionsSatisfied, PreconditionsViolated

**Transitions:**
- PreconditionsSatisfied -> PreconditionsViolated via invalid (bin_len, hex_maxlen, slice lengths) leading to std::process::abort()

**Evidence:** bin2hex_internal: comment 'Need space for 2*bin_len hex chars + NUL terminator.'; bin2hex_internal: `if bin_len > (usize::MAX / 2) || hex_maxlen <= bin_len * 2 { std::process::abort(); }`; bin2hex_internal: `if hex.len() < bin_len * 2 + 1 || bin.len() < bin_len { std::process::abort(); }`; bin2hex_internal: writes `hex[i * 2]`, `hex[i * 2 + 1]`, and `hex[bin_len * 2] = 0` (requires capacity invariant)

**Implementation:** Introduce validated newtypes that encode the coupled invariants, e.g. `struct BinLen(usize); impl TryFrom<usize> for BinLen { ... ensure <= usize::MAX/2 ... }` and `struct HexOut<'a> { buf: &'a mut [i8] }` constructed via a `try_new(buf, bin_len)` that guarantees `buf.len() >= 2*bin_len+1`. Alternatively accept `hex: &mut [i8; 2*BIN_LEN+1]` with const generics when BIN_LEN is known at compile time, eliminating runtime abort paths.

---

## Protocol Invariants

### 2. FFI pointer validity + capacity protocol (non-null implies at least 1024 accessible bytes and compatibility with bin_len/hex_maxlen)

**Location**: `/data/test_case/lib.rs:1-58`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The FFI wrapper implicitly assumes that if `hex`/`bin` are non-null, they point to at least 1024 bytes of valid memory (readable for `bin`, writable for `hex`). It then passes fixed-size slices (len=1024) to bin2hex_internal, which further requires that those slices be large enough for the requested `bin_len` and output size. These requirements are not represented in the type system; they are partially guarded by aborts in bin2hex_internal, but pointer validity/alignment/lifetime are entirely unchecked and can cause UB before any abort if the pointers are invalid.

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

pub(crate) fn bin2hex_internal(
    hex: &mut [i8],
    hex_maxlen: usize,
    bin: &[u8],
    bin_len: usize,
) -> *mut i8 {
    // Need space for 2*bin_len hex chars + NUL terminator.
    if bin_len > (usize::MAX / 2) || hex_maxlen <= bin_len * 2 {
        std::process::abort();
    }
    if hex.len() < bin_len * 2 + 1 || bin.len() < bin_len {
        std::process::abort();
    }

    const HEX: &[u8; 16] = b"0123456789abcdef";

    for (i, &byte) in bin[..bin_len].iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0x0f) as usize;
        hex[i * 2] = HEX[hi] as i8;
        hex[i * 2 + 1] = HEX[lo] as i8;
    }

    hex[bin_len * 2] = 0;
    hex.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn bin2hex(
    hex: *mut i8,
    hex_maxlen: usize,
    bin: *const u8,
    bin_len: usize,
) -> *mut i8 {
    let hex_slice = if hex.is_null() {
        &mut []
    } else {
        // Preserve original behavior/assumptions about available memory.
        std::slice::from_raw_parts_mut(hex, 1024)
    };
    let bin_slice = if bin.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(bin, 1024)
    };

    bin2hex_internal(hex_slice, hex_maxlen, bin_slice, bin_len)
}
```

**Entity:** unsafe extern "C" fn bin2hex(hex: *mut i8, hex_maxlen: usize, bin: *const u8, bin_len: usize)

**States:** NullPointers, NonNullPointersWithValidMemory, NonNullPointersWithInvalidMemory

**Transitions:**
- NullPointers -> (abort or OK) depending on bin_len/hex_maxlen checks in bin2hex_internal (since slices are empty)
- NonNullPointersWithValidMemory -> (abort or OK) depending on bin_len/hex_maxlen and 1024-based slice length constraints
- NonNullPointersWithInvalidMemory -> UB when calling `from_raw_parts(_mut)` or when reading/writing through the slices

**Evidence:** bin2hex: `pub unsafe extern "C" fn bin2hex(hex: *mut i8, ... bin: *const u8, ...)` indicates raw-pointer protocol is externalized; bin2hex: `if hex.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(hex, 1024) }` hard-codes required accessible length for non-null pointers; bin2hex: `if bin.is_null() { &[] } else { std::slice::from_raw_parts(bin, 1024) }` same for input; bin2hex: comment 'Preserve original behavior/assumptions about available memory.' documents the implicit contract

**Implementation:** Split the safe core from the FFI boundary more explicitly: keep `bin2hex_internal` as the safe API, and create an FFI-only wrapper that first converts raw pointers into a validated capability type like `struct FfiBuf<T> { ptr: NonNull<T>, len: usize }` created from (ptr,len) provided by the caller (or require caller to pass explicit buffer lengths instead of the hard-coded 1024). Then call a safe function `fn bin2hex_safe(hex: &mut [i8], bin: &[u8], bin_len: usize) -> Result<NonNull<i8>, Error>` where construction of slices/capabilities performs all checks up front.

---

