# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 2

## Protocol Invariants

### 1. FFI pointer validity/size protocol (null or at least 1024 f32 accessible)

**Location**: `/data/test_case/lib.rs:1-62`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The FFI wrapper interprets raw pointers as slices of length 1024. If a pointer is null, it is treated as an empty slice, causing the internal function to early-return and do nothing. If non-null, the code assumes the pointer is valid for 1024 `f32` elements (properly aligned, readable for src, writable for dest). These are crucial safety and size preconditions that are not enforced by the type system and are only implicitly relied on by `from_raw_parts(_mut)`.

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

pub(crate) fn hsv_to_rgb_internal(dest: &mut [f32], src: &[f32]) {
    if dest.len() < 3 || src.len() < 3 {
        return;
    }

    let mut h = src[0];
    let s = src[1];
    let v = src[2];

    if s == 0.0 {
        dest[..3].fill(v);
        return;
    }

    h /= 60.0;
    let i = h.floor() as i32;
    let f = h - i as f32;

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    dest[0] = r;
    dest[1] = g;
    dest[2] = b;
}

#[no_mangle]
pub unsafe extern "C" fn hsv_to_rgb(dest: *mut f32, src: *const f32) {
    let dest_slice = if dest.is_null() {
        &mut []
    } else {
        std::slice::from_raw_parts_mut(dest, 1024)
    };

    let src_slice = if src.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(src, 1024)
    };

    hsv_to_rgb_internal(dest_slice, src_slice)
}
```

**Entity:** unsafe extern "C" fn hsv_to_rgb(dest: *mut f32, src: *const f32)

**States:** NullPointer (treated as empty slice), NonNullButInvalid (dangling/too short/unaligned), NonNullValid (points to >=1024 f32)

**Transitions:**
- NullPointer -> NonNullValid via caller passing a valid allocated buffer
- NonNullValid -> NullPointer via caller passing null (no-op behavior)

**Evidence:** hsv_to_rgb: `if dest.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dest, 1024) }` encodes null vs non-null state and assumes length 1024 when non-null; hsv_to_rgb: `if src.is_null() { &[] } else { std::slice::from_raw_parts(src, 1024) }` same for src; hsv_to_rgb signature: `pub unsafe extern "C" fn ...` indicates required external safety preconditions; hsv_to_rgb: fixed constant `1024` passed to `from_raw_parts(_mut)` implies a hidden 'buffer must be >= 1024' requirement

**Implementation:** Provide a safe Rust API that requires proven-valid buffers, e.g. `pub fn hsv_to_rgb_safe(dest: &mut [f32; 3], src: &[f32; 3])` (or `[f32; 1024]` if that size is truly required), and keep the `extern "C"` wrapper minimal: convert raw pointers to `Option<NonNull<f32>>` and/or require explicit lengths from C (`dest_len`, `src_len`) to validate before creating slices. A wrapper type like `struct FfiBuf1024(NonNull<f32>);` constructed via `unsafe fn new(ptr: *mut f32) -> Option<Self>` can centralize and document the invariant.

---

