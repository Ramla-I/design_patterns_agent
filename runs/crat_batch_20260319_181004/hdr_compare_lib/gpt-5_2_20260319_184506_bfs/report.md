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

### 1. Header slice validity/length precondition (>= HDR_LEN and MP3-like sync constraints)

**Location**: `/data/test_case/lib.rs:1-60`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Both hdr_valid() and hdr_compare_internal() assume the input slices represent a header with at least HDR_LEN bytes; additionally, hdr_compare_internal() only returns true when the second header (h2) satisfies hdr_valid() and when selected bitfields match between h1 and h2. These requirements are enforced via runtime length checks and bit-tests on raw &[u8], rather than a type representing a validated header of known length. The type system does not prevent calling compare/validity-sensitive logic with a too-short slice or with unvalidated bytes.

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

const HDR_LEN: usize = 3;

#[inline]
fn hdr_valid(h: &[u8]) -> i32 {
    if h.len() < HDR_LEN {
        return 0;
    }

    let b0 = h[0];
    let b1 = h[1];
    let b2 = h[2];

    let ok = b0 == 0xff
        && ((b1 & 0xf0) == 0xf0 || (b1 & 0xfe) == 0xe2)
        && ((b1 >> 1) & 3) != 0
        && (b2 >> 4) != 15
        && ((b2 >> 2) & 3) != 3;

    ok as i32
}

#[inline]
pub(crate) fn hdr_compare_internal(h1: &[u8], h2: &[u8]) -> i32 {
    if h1.len() < HDR_LEN || h2.len() < HDR_LEN {
        return 0;
    }

    let ok = hdr_valid(h2) != 0
        && ((h1[1] ^ h2[1]) & 0xfe) == 0
        && ((h1[2] ^ h2[2]) & 0x0c) == 0
        && (((h1[2] & 0xf0) == 0) as i32 ^ ((h2[2] & 0xf0) == 0) as i32) == 0;

    ok as i32
}

#[no_mangle]
pub unsafe extern "C" fn hdr_compare(h1: *const u8, h2: *const u8) -> i32 {
    hdr_compare_internal(
        if h1.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(h1, 1024)
        },
        if h2.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(h2, 1024)
        },
    )
}
```

**Entity:** hdr_valid / hdr_compare_internal header slices (&[u8])

**States:** TooShort(<HDR_LEN), LongEnoughButInvalid, ValidHeader

**Transitions:**
- TooShort(<HDR_LEN) -> LongEnoughButInvalid by providing a slice with len >= HDR_LEN but failing hdr_valid() bit constraints
- LongEnoughButInvalid -> ValidHeader by providing bytes that satisfy hdr_valid() constraints
- ValidHeader + ValidHeader -> (comparable) via hdr_compare_internal() bitmask equivalence checks

**Evidence:** const HDR_LEN: usize = 3; hdr_valid(h): `if h.len() < HDR_LEN { return 0; }` then indexes h[0], h[1], h[2]; hdr_compare_internal(h1,h2): `if h1.len() < HDR_LEN || h2.len() < HDR_LEN { return 0; }` then indexes h1[1],h2[1],h1[2],h2[2]; hdr_compare_internal: `let ok = hdr_valid(h2) != 0 && ...` (h2 must be validated before bit-compare)

**Implementation:** Introduce `struct Header([u8; HDR_LEN]);` plus `impl TryFrom<&[u8]> for Header` that checks length and (optionally) `hdr_valid` constraints. Then make `hdr_compare_internal` take `Header`/`ValidatedHeader` newtypes (e.g., `fn compare(h1: HeaderBits, h2: ValidHeaderBits) -> bool`) so indexing and validity are compile-time/constructor-enforced rather than ad hoc runtime checks.

---

## Protocol Invariants

### 2. FFI pointer/provenance protocol (nullable pointer => empty, non-null => readable 1024 bytes)

**Location**: `/data/test_case/lib.rs:1-60`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: hdr_compare() treats null pointers as 'no data' by converting them to an empty slice, but treats any non-null pointer as if it points to at least 1024 readable bytes, constructing `from_raw_parts(h, 1024)`. This is an implicit FFI contract: callers must pass either NULL or a pointer valid for reading 1024 bytes for the duration of the call. The Rust type system cannot enforce that contract for `*const u8`, so the function relies on an unsafe runtime convention; violating it is immediate UB.

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

const HDR_LEN: usize = 3;

#[inline]
fn hdr_valid(h: &[u8]) -> i32 {
    if h.len() < HDR_LEN {
        return 0;
    }

    let b0 = h[0];
    let b1 = h[1];
    let b2 = h[2];

    let ok = b0 == 0xff
        && ((b1 & 0xf0) == 0xf0 || (b1 & 0xfe) == 0xe2)
        && ((b1 >> 1) & 3) != 0
        && (b2 >> 4) != 15
        && ((b2 >> 2) & 3) != 3;

    ok as i32
}

#[inline]
pub(crate) fn hdr_compare_internal(h1: &[u8], h2: &[u8]) -> i32 {
    if h1.len() < HDR_LEN || h2.len() < HDR_LEN {
        return 0;
    }

    let ok = hdr_valid(h2) != 0
        && ((h1[1] ^ h2[1]) & 0xfe) == 0
        && ((h1[2] ^ h2[2]) & 0x0c) == 0
        && (((h1[2] & 0xf0) == 0) as i32 ^ ((h2[2] & 0xf0) == 0) as i32) == 0;

    ok as i32
}

#[no_mangle]
pub unsafe extern "C" fn hdr_compare(h1: *const u8, h2: *const u8) -> i32 {
    hdr_compare_internal(
        if h1.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(h1, 1024)
        },
        if h2.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(h2, 1024)
        },
    )
}
```

**Entity:** hdr_compare (FFI) raw pointers (*const u8)

**States:** NullPointer, NonNullButNotReadable(UB), NonNullAndReadable(>=1024 bytes)

**Transitions:**
- NullPointer -> (safe empty slice) via `if h.is_null() { &[] }`
- NonNullAndReadable(>=1024 bytes) -> (safe slice view) via `std::slice::from_raw_parts(h, 1024)`

**Evidence:** pub unsafe extern "C" fn hdr_compare(h1: *const u8, h2: *const u8) -> i32; hdr_compare: `if h1.is_null() { &[] } else { std::slice::from_raw_parts(h1, 1024) }` (same for h2); use of `from_raw_parts(..., 1024)` encodes the implicit 'must be readable for 1024 bytes' requirement

**Implementation:** For Rust callers, provide a safe wrapper taking `Option<&[u8; 1024]>` (or `&[u8]` plus explicit length) and keep the raw-pointer function as a thin unsafe shim. Alternatively, change the C ABI to `(*const u8, usize)` for each header so the slice length is provided by the caller and can be checked against `HDR_LEN` before indexing.

---

