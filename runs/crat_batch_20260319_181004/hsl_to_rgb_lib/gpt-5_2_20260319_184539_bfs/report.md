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

### 1. HSL/RGB slice length precondition (needs >=3 elements)

**Location**: `/data/test_case/lib.rs:1-75`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: hsl_to_rgb_internal implicitly requires that both src and dest have at least 3 elements. It indexes src[0..2] and writes dest[0..2] unconditionally (except for early return after writing), so shorter slices will panic at runtime. The type system currently accepts any slice length, so callers must uphold the length protocol manually.

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

pub(crate) fn hsl_to_rgb_internal(dest: &mut [f32], src: &[f32]) {
    // Preserve original behavior: assumes at least 3 elements are available.
    let h: f32 = src[0];
    let s: f32 = src[1];
    let l: f32 = src[2];

    if s == 0.0f32 {
        dest[0] = l;
        dest[1] = l;
        dest[2] = l;
        return;
    }

    let c: f32 = (1.0f32 - (2.0f32 * l - 1.0f32).abs()) * s;
    let m: f32 = 1.0f32 * (l - 0.5f32 * c);
    let x: f32 = c * (1.0f32 - (((h / 60.0f32) % 2.0f32) - 1.0f32).abs());

    if (0.0f32..60.0f32).contains(&h) {
        dest[0] = c + m;
        dest[1] = x + m;
        dest[2] = m;
    } else if (60.0f32..120.0f32).contains(&h) {
        dest[0] = x + m;
        dest[1] = c + m;
        dest[2] = m;
    } else if h < 120.0f32 && h < 180.0f32 {
        // NOTE: This intentionally preserves the original (buggy) condition from the C2Rust code.
        // It makes the 120..180 sector unreachable, matching the expected test behavior.
        dest[0] = m;
        dest[1] = c + m;
        dest[2] = x + m;
    } else if (180.0f32..240.0f32).contains(&h) {
        dest[0] = m;
        dest[1] = x + m;
        dest[2] = c + m;
    } else if (240.0f32..300.0f32).contains(&h) {
        dest[0] = x + m;
        dest[1] = m;
        dest[2] = c + m;
    } else if (300.0f32..360.0f32).contains(&h) {
        dest[0] = c + m;
        dest[1] = m;
        dest[2] = x + m;
    } else {
        dest[0] = m;
        dest[1] = m;
        dest[2] = m;
    };
}

#[no_mangle]
pub unsafe extern "C" fn hsl_to_rgb(dest: *mut f32, src: *const f32) {
    hsl_to_rgb_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** hsl_to_rgb_internal(dest: &mut [f32], src: &[f32])

**States:** ValidSlices(len>=3), InvalidSlices(len<3)

**Transitions:**
- InvalidSlices -> panic via src[0]/dest[0] indexing
- ValidSlices -> ValidSlices (computes and writes RGB)

**Evidence:** comment: "assumes at least 3 elements are available"; code: `let h: f32 = src[0]; let s: f32 = src[1]; let l: f32 = src[2];`; code: writes `dest[0]`, `dest[1]`, `dest[2]` in all branches

**Implementation:** Change the internal API to take fixed-size arrays or a validated wrapper: `fn hsl_to_rgb_internal(dest: &mut [f32; 3], src: &[f32; 3])` (or `struct Hsl([f32;3]); struct Rgb([f32;3]);`). If larger buffers are required, take `&mut [f32]` but also accept explicit offsets/lengths or provide a `try_from_slice` newtype that checks `len>=3` once.

---

## Protocol Invariants

### 2. FFI pointer validity/size protocol (non-null implies readable/writable for 1024 f32)

**Location**: `/data/test_case/lib.rs:1-75`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: hsl_to_rgb encodes an implicit FFI protocol: a null `dest`/`src` pointer is treated as a sentinel and converted to an empty slice, but any non-null pointer is assumed to point to at least 1024 `f32` elements (readable for src, writable for dest). This is not enforced by the type system; violating it yields UB via `from_raw_parts(_mut)` and/or panics later when indexing. Additionally, passing null effectively guarantees a panic because the internal function requires len>=3.

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

pub(crate) fn hsl_to_rgb_internal(dest: &mut [f32], src: &[f32]) {
    // Preserve original behavior: assumes at least 3 elements are available.
    let h: f32 = src[0];
    let s: f32 = src[1];
    let l: f32 = src[2];

    if s == 0.0f32 {
        dest[0] = l;
        dest[1] = l;
        dest[2] = l;
        return;
    }

    let c: f32 = (1.0f32 - (2.0f32 * l - 1.0f32).abs()) * s;
    let m: f32 = 1.0f32 * (l - 0.5f32 * c);
    let x: f32 = c * (1.0f32 - (((h / 60.0f32) % 2.0f32) - 1.0f32).abs());

    if (0.0f32..60.0f32).contains(&h) {
        dest[0] = c + m;
        dest[1] = x + m;
        dest[2] = m;
    } else if (60.0f32..120.0f32).contains(&h) {
        dest[0] = x + m;
        dest[1] = c + m;
        dest[2] = m;
    } else if h < 120.0f32 && h < 180.0f32 {
        // NOTE: This intentionally preserves the original (buggy) condition from the C2Rust code.
        // It makes the 120..180 sector unreachable, matching the expected test behavior.
        dest[0] = m;
        dest[1] = c + m;
        dest[2] = x + m;
    } else if (180.0f32..240.0f32).contains(&h) {
        dest[0] = m;
        dest[1] = x + m;
        dest[2] = c + m;
    } else if (240.0f32..300.0f32).contains(&h) {
        dest[0] = x + m;
        dest[1] = m;
        dest[2] = c + m;
    } else if (300.0f32..360.0f32).contains(&h) {
        dest[0] = c + m;
        dest[1] = m;
        dest[2] = x + m;
    } else {
        dest[0] = m;
        dest[1] = m;
        dest[2] = m;
    };
}

#[no_mangle]
pub unsafe extern "C" fn hsl_to_rgb(dest: *mut f32, src: *const f32) {
    hsl_to_rgb_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** unsafe extern "C" fn hsl_to_rgb(dest: *mut f32, src: *const f32)

**States:** NullPointerSentinel, ValidPointerTo1024Floats

**Transitions:**
- NullPointerSentinel -> InvalidSlices(len=0) via `&mut []` / `&[]`
- ValidPointerTo1024Floats -> ValidSlices(len=1024) via `from_raw_parts(_mut)(..., 1024)`

**Evidence:** code: `if dest.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dest, 1024) }`; code: `if src.is_null() { &[] } else { std::slice::from_raw_parts(src, 1024) }`; call: `hsl_to_rgb_internal(...)` which indexes `[0]..[2]` regardless of slice origin

**Implementation:** For Rust callers, expose a safe wrapper that requires the right sizes: `pub fn hsl_to_rgb_safe(dest: &mut [f32;3], src: &[f32;3])`. Keep the extern C function but implement it in terms of a checked conversion: return early on null, or use `NonNull<f32>` and an explicit `len` parameter from C (preferred), or at least document/encode the contract with `struct PtrLen<T> { ptr: NonNull<T>, len: usize }` on the Rust side for internal use.

---

