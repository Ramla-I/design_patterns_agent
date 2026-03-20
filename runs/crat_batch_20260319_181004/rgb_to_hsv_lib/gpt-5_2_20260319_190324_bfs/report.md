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

### 1. FFI pointer/length protocol (null-or-valid pointer to >=1024 f32, non-aliasing)

**Location**: `/data/test_case/lib.rs:1-67`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The extern "C" entrypoint treats `dest`/`src` as either null (interpreted as an empty slice) or as pointers to at least 1024 `f32` elements, and then forwards them to the internal function. This protocol (pointer validity, minimum allocation size, alignment, and that `dest` is writable and does not violate aliasing) is not enforced by the type system; it is relied on by the unsafe `from_raw_parts(_mut)` calls. Passing a non-null but invalid/short pointer is UB. Also, allowing null pointers causes a silent no-op path instead of an explicit error.

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

pub(crate) fn rgb_to_hsv_internal(dest: &mut [f32], src: &[f32]) {
    // Expect at least 3 floats in each slice (r,g,b) and (h,s,v).
    if dest.len() < 3 || src.len() < 3 {
        return;
    }

    let [r, g, b] = [src[0], src[1], src[2]];

    let min = r.min(g).min(b);
    let max = r.max(g).max(b);

    let delta = max - min;
    let v = max;

    // Match original behavior: if delta == 0 or max == 0, h and s are 0.
    if delta == 0.0 || max == 0.0 {
        dest[0] = 0.0;
        dest[1] = 0.0;
        dest[2] = v;
        return;
    }

    let s = delta / max;

    let mut h = if r == max {
        (g - b) / delta
    } else if g == max {
        2.0 + (b - r) / delta
    } else {
        4.0 + (r - g) / delta
    };

    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }

    dest[0] = h;
    dest[1] = s;
    dest[2] = v;
}

#[no_mangle]
pub unsafe extern "C" fn rgb_to_hsv(dest: *mut f32, src: *const f32) {
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

    rgb_to_hsv_internal(dest_slice, src_slice)
}
```

**Entity:** rgb_to_hsv (FFI pointer inputs)

**States:** NullPointer, NonNullValid(>=1024), NonNullInvalid(<1024 or dangling)

**Transitions:**
- NullPointer -> (empty slice, no-op) via `if dest.is_null()` / `if src.is_null()` branches
- NonNullValid(>=1024) -> (slice view) via `std::slice::from_raw_parts_mut(dest, 1024)` / `from_raw_parts(src, 1024)`
- NonNullInvalid -> (UB) via `from_raw_parts(_mut)` on invalid memory

**Evidence:** rgb_to_hsv: `pub unsafe extern "C" fn rgb_to_hsv(dest: *mut f32, src: *const f32)` accepts raw pointers with no length; rgb_to_hsv: `if dest.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dest, 1024) }` hard-codes length 1024; rgb_to_hsv: `if src.is_null() { &[] } else { std::slice::from_raw_parts(src, 1024) }` hard-codes length 1024; rgb_to_hsv: calls `rgb_to_hsv_internal(dest_slice, src_slice)` which expects at least 3 elements but relies on slice construction for safety

**Implementation:** Expose a safe Rust wrapper that requires a validated buffer capability and (optionally) a length: e.g., `struct F32Buf1024<'a>(&'a mut [f32; 1024]);` and `struct F32Buf1024RO<'a>(&'a [f32; 1024]);`. Provide `unsafe fn from_raw(ptr: *mut f32) -> Option<F32Buf1024>` that returns `None` on null, and make `rgb_to_hsv` call that. This centralizes the unsafe contract and makes the required size explicit in types for Rust callers; for C callers, document the contract and/or take an explicit `len` parameter and validate `len >= 3`/`>=1024` before forming slices.

---

