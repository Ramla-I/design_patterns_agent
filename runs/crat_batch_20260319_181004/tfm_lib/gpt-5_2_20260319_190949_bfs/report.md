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

### 1. Slice length/stride protocol for tfm_internal (src: 3*f32 per item, dest: 2*f32 per item)

**Location**: `/data/test_case/lib.rs:1-64`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: tfm_internal assumes a strided layout: each iteration reads exactly 3 f32s from src (src[0..3]) and writes exactly 2 f32s to dest (dest[0..2]), then advances the slices by those strides. This implicitly requires that src has at least 3*count elements and dest has at least 2*count elements (after clamping count to non-negative). The type system only knows these are slices; it does not encode the required relationship between slice lengths and count, so an invalid caller can trigger out-of-bounds indexing/slicing at runtime.

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

pub mod src {
    pub mod lib {
        #[inline]
        fn clamp_nonneg(x: f32) -> f32 {
            if x < 0.0 { 0.0 } else { x }
        }

        #[inline]
        fn compute(dx2: f32, dy2: f32, dxy: f32) -> (f32, f32) {
            let sqd = dy2 * dy2 - 2.0 * dx2 * dy2 + dx2 * dx2 + 4.0 * dxy * dxy;
            let lambda = 0.5 * (dy2 + dx2 + clamp_nonneg(sqd).sqrt());
            (dx2 - lambda, dxy)
        }

        pub(crate) fn tfm_internal(mut dest: &mut [f32], mut src: &[f32], count: i32) {
            let count = count.max(0) as usize;

            for _ in 0..count {
                let (a, b, c) = (src[0], src[1], src[2]);

                if a < b {
                    let (out0, out1) = compute(a, b, c);
                    dest[0] = out0;
                    dest[1] = out1;
                } else {
                    // Swap roles to match original branch behavior.
                    let (out0, out1) = compute(b, a, c);
                    dest[0] = c;
                    dest[1] = out0;
                }

                src = &src[3..];
                dest = &mut dest[2..];
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn tfm(dest: *mut f32, src: *const f32, count: i32) {
            tfm_internal(
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
                count,
            )
        }
    }
}
```

**Entity:** src::lib::tfm_internal

**States:** ValidSlices (len(src) >= 3*count && len(dest) >= 2*count), InvalidSlices (insufficient length for count)

**Transitions:**
- ValidSlices -> ValidSlices via loop body advancing src = &src[3..], dest = &mut dest[2..]

**Evidence:** tfm_internal: reads (a,b,c) = (src[0], src[1], src[2]) each iteration; tfm_internal: writes dest[0] and dest[1] each iteration; tfm_internal: advances src = &src[3..] and dest = &mut dest[2..] inside the loop; tfm_internal: count is converted to usize and used to control number of iterations: for _ in 0..count

**Implementation:** Introduce newtypes that carry validated chunking, e.g. fn tfm_internal(dest: Dest2Chunks<'_>, src: Src3Chunks<'_>) where Dest2Chunks and Src3Chunks are constructed only via checked constructors that ensure lengths are multiples/at least required. Alternatively accept iterators over fixed-size arrays: src: impl Iterator<Item=[f32;3]>, dest: impl Iterator<Item=&mut [f32;2]> (or arrays) to make the stride explicit.

---

### 2. FFI pointer validity/ownership precondition for tfm (nonnull => points to >=1024 f32)

**Location**: `/data/test_case/lib.rs:1-64`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The FFI entrypoint converts raw pointers to Rust slices of fixed length 1024 when pointers are non-null, otherwise it uses empty slices. This encodes an implicit contract: if dest/src are non-null, they must be valid for creating a 1024-element slice (dest: writable, src: readable) for the duration of the call, with proper alignment and no violating aliasing for the mutable dest slice. The type system cannot express these pointer validity requirements, and the function currently enforces only the null/non-null split at runtime.

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

pub mod src {
    pub mod lib {
        #[inline]
        fn clamp_nonneg(x: f32) -> f32 {
            if x < 0.0 { 0.0 } else { x }
        }

        #[inline]
        fn compute(dx2: f32, dy2: f32, dxy: f32) -> (f32, f32) {
            let sqd = dy2 * dy2 - 2.0 * dx2 * dy2 + dx2 * dx2 + 4.0 * dxy * dxy;
            let lambda = 0.5 * (dy2 + dx2 + clamp_nonneg(sqd).sqrt());
            (dx2 - lambda, dxy)
        }

        pub(crate) fn tfm_internal(mut dest: &mut [f32], mut src: &[f32], count: i32) {
            let count = count.max(0) as usize;

            for _ in 0..count {
                let (a, b, c) = (src[0], src[1], src[2]);

                if a < b {
                    let (out0, out1) = compute(a, b, c);
                    dest[0] = out0;
                    dest[1] = out1;
                } else {
                    // Swap roles to match original branch behavior.
                    let (out0, out1) = compute(b, a, c);
                    dest[0] = c;
                    dest[1] = out0;
                }

                src = &src[3..];
                dest = &mut dest[2..];
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn tfm(dest: *mut f32, src: *const f32, count: i32) {
            tfm_internal(
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
                count,
            )
        }
    }
}
```

**Entity:** src::lib::tfm (extern "C" API)

**States:** NullPointers (treated as empty slices), NonNullValidPointers (points to 1024 f32 readable/writable), NonNullInvalidPointers (dangling/too-short/misaligned/aliased)

**Transitions:**
- NullPointers -> (calls tfm_internal with empty slices)
- NonNullValidPointers -> (calls tfm_internal with from_raw_parts(_mut)(..., 1024))

**Evidence:** tfm: signature uses raw pointers: dest: *mut f32, src: *const f32; tfm: null handling via dest.is_null() ? &mut [] : from_raw_parts_mut(dest, 1024); tfm: null handling via src.is_null() ? &[] : from_raw_parts(src, 1024); tfm is declared unsafe extern "C", indicating the caller must uphold safety preconditions

**Implementation:** Expose a safe Rust wrapper that requires typed capabilities instead of raw pointers, e.g. pub fn tfm_safe(dest: &mut [f32;1024], src: &[f32;1024], count: usize) and keep the unsafe extern "C" shim minimal. This moves the key invariant (exact length/valid memory) into the type system for Rust callers, while still permitting FFI callers via the unsafe shim.

---

