# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 2
- **Modules analyzed**: 2

## Precondition Invariants

### 1. FFI slice/length contract for vector ops (ValidPtr+Len, NonZeroMagnitude)

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: These functions rely on an implicit contract that the provided pointer/slices are valid for `length` elements and that normalization is only performed when the vector magnitude is non-zero. `normalize` converts a raw pointer + length into a mutable slice and divides by the computed magnitude; if `length` exceeds the allocation, this is UB, and if the magnitude is 0.0 the division produces NaNs/Infs. `spectral_contrast` additionally assumes its `&mut [f32]` inputs are at least `length` long because it forwards `length` to `normalize`/`dot_product` without checking.

**Evidence**:

```rust
#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(builtin_syntax)]
#![feature(core_intrinsics)]
#![feature(derive_clone_copy)]
#![feature(hint_must_use)]
#![feature(panic_internals)]
#![feature(formatting_options)]
#![feature(coverage_attribute)]

pub mod src {
    pub mod spectral_contrast {
        pub type float_t = f32;

        unsafe extern "C" fn dot_product(a: &[f32], b: &[f32], length: i32) -> f64 {
            let mut sum = 0.0f64;
            let len = length as usize;
            for i in 0..len {
                sum += (a[i] * b[i]) as f64;
            }
            sum
        }

        unsafe extern "C" fn normalize(v: *mut f32, length: i32) {
            // Match original C2Rust behavior: it created slices of length 100000 when non-null,
            // then used `length` for the loop bounds. We can safely use `length` for the slice.
            if v.is_null() {
                return;
            }
            let len = length as usize;
            let slice = std::slice::from_raw_parts_mut(v, len);
            let magnitude = dot_product(slice, slice, length).sqrt();
            for x in slice.iter_mut() {
                *x = (*x as f64 / magnitude) as float_t;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn spectral_contrast(a: &mut [f32], b: &mut [f32], length: i32) -> f64 {
            normalize(a.as_mut_ptr(), length);
            normalize(b.as_mut_ptr(), length);
            dot_product(a, b, length)
        }
    }

    pub mod lib {
        use crate::src::spectral_contrast::spectral_contrast;

        extern "C" {
            fn memcpy(
                __dest: *mut core::ffi::c_void,
                __src: *const core::ffi::c_void,
                __n: usize,
            ) -> *mut core::ffi::c_void;
        }

        pub type size_t = usize;
        pub type float_t = f64;

        pub const N_SMOOTH: i32 = 16;

        unsafe extern "C" fn total(v: &[f64], length: i32) -> f64 {
            let mut sum = 0.0f64;
            for &x in v.iter().take(length as usize) {
                sum += x;
            }
            sum
        }

        unsafe extern "C" fn smoothen(v: &mut [f64], length: i32) {
            // Preserve original in-place forward smoothing and fixed divisor N_SMOOTH.
            let len = length as usize;
            for i in 0..len {
                let mut sum = 0.0f64;
                let mut j = 0i32;
                while j < N_SMOOTH && (i as i32 + j) < length {
                    sum += v[i + j as usize];
                    j += 1;
                }
                v[i] = sum / (N_SMOOTH as f64);
            }
        }

        unsafe extern "C" fn differentiate(v: &mut [f64], length: i32) {
            let len = length as usize;
            for i in 0..(len - 1) {
                v[i] = v[i + 1] - v[i];
            }
            v[len - 1] = 0.0;
        }

        unsafe extern "C" fn preprocess(v: &mut [f64], source: &[f64], length: i32) {
            let len = length as usize;
            v[..len].copy_from_slice(&source[..len]);
            smoothen(v, length);
            differentiate(v, length);
            smoothen(v, length);
        }

        #[export_name = "match"]
        pub unsafe extern "C" fn match_0(test: &[f64], reference: &[f64], bins: i32, threshold: f64) -> i32 {
            let bins_usize = bins as usize;

            let mut t: Vec<float_t> = vec![0.0; bins_usize];
            let mut r: Vec<float_t> = vec![0.0; bins_usize];

            if total(test, bins) < threshold * total(reference, bins) {
                return 0;
            }

            preprocess(&mut t, test, bins);
            preprocess(&mut r, reference, bins);

            // IMPORTANT: The original C2Rust used `bytemuck::cast_slice_mut(&mut t)` where
            // `t: Vec<f64>` and the callee expects `&mut [f32]`. This is a byte reinterpretation:
            // the f64 buffer is viewed as 2*bins f32 values, and only the first `bins` are used.
            // Reproduce that exactly (endianness-dependent, as in the original).
            let t32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut t);
            let r32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut r);

            (spectral_contrast(t32, r32, bins) >= threshold) as i32
        }
    }
}

mod c_lib {
    pub static mut STDOUT_ERROR: i32 = 0;
    pub static mut STDERR_ERROR: i32 = 0;
    unsafe extern "C" {
        #[link_name = "stdout"]
        pub static mut STDOUT: *mut std::ffi::c_void;
        #[link_name = "stderr"]
        pub static mut STDERR: *mut std::ffi::c_void;
    }
}
```

**Entity:** src::spectral_contrast::{normalize, dot_product, spectral_contrast}

**States:** Valid (non-null, length matches allocation, magnitude>0), Invalid (null/short buffer or magnitude==0)

**Transitions:**
- Invalid -> Valid only by satisfying caller-side allocation/length and non-zero magnitude preconditions (not represented in types)

**Evidence:** fn normalize(v: *mut f32, length: i32): `if v.is_null() { return; }` then `from_raw_parts_mut(v, len)` (raw ptr + len validity is assumed); fn dot_product(a: &[f32], b: &[f32], length: i32): loops `for i in 0..len { sum += (a[i] * b[i]) ... }` (assumes a.len() and b.len() >= length); fn normalize: `let magnitude = dot_product(slice, slice, length).sqrt();` then `*x = (*x as f64 / magnitude) as float_t;` (assumes magnitude != 0.0); pub fn spectral_contrast(a: &mut [f32], b: &mut [f32], length: i32): passes `a.as_mut_ptr()`/`b.as_mut_ptr()` + `length` (no bounds check against slice lengths)

**Implementation:** Introduce a validated wrapper like `struct Len(usize); impl TryFrom<i32> for Len` ensuring non-negative, and accept `a: &mut [f32]` without separate `length` (use `a.len()`), or accept `struct SliceLen<'a, T>(&'a mut [T]);` with constructors that validate `len >= required`. For normalization, use `NonZeroF64`-like guard by returning `Result<NormalizedSlice<'a>, ZeroMagnitude>` from a safe normalization API, keeping the current `extern "C"` as a thin unsafe adapter.

---

## Protocol Invariants

### 3. Global I/O handle validity + error-flag protocol (must be initialized and used consistently)

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The module exposes mutable global state: raw pointers to C stdio objects and separate mutable error flags. Correct use implicitly requires that `STDOUT`/`STDERR` are valid pointers (not null/dangling) and that `STDOUT_ERROR`/`STDERR_ERROR` are updated in a disciplined way relative to I/O operations. This coupling and the initialization/validity of these globals are not represented in types; any use must be `unsafe` and relies on external initialization and ordering.

**Evidence**:

```rust
#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(builtin_syntax)]
#![feature(core_intrinsics)]
#![feature(derive_clone_copy)]
#![feature(hint_must_use)]
#![feature(panic_internals)]
#![feature(formatting_options)]
#![feature(coverage_attribute)]

pub mod src {
    pub mod spectral_contrast {
        pub type float_t = f32;

        unsafe extern "C" fn dot_product(a: &[f32], b: &[f32], length: i32) -> f64 {
            let mut sum = 0.0f64;
            let len = length as usize;
            for i in 0..len {
                sum += (a[i] * b[i]) as f64;
            }
            sum
        }

        unsafe extern "C" fn normalize(v: *mut f32, length: i32) {
            // Match original C2Rust behavior: it created slices of length 100000 when non-null,
            // then used `length` for the loop bounds. We can safely use `length` for the slice.
            if v.is_null() {
                return;
            }
            let len = length as usize;
            let slice = std::slice::from_raw_parts_mut(v, len);
            let magnitude = dot_product(slice, slice, length).sqrt();
            for x in slice.iter_mut() {
                *x = (*x as f64 / magnitude) as float_t;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn spectral_contrast(a: &mut [f32], b: &mut [f32], length: i32) -> f64 {
            normalize(a.as_mut_ptr(), length);
            normalize(b.as_mut_ptr(), length);
            dot_product(a, b, length)
        }
    }

    pub mod lib {
        use crate::src::spectral_contrast::spectral_contrast;

        extern "C" {
            fn memcpy(
                __dest: *mut core::ffi::c_void,
                __src: *const core::ffi::c_void,
                __n: usize,
            ) -> *mut core::ffi::c_void;
        }

        pub type size_t = usize;
        pub type float_t = f64;

        pub const N_SMOOTH: i32 = 16;

        unsafe extern "C" fn total(v: &[f64], length: i32) -> f64 {
            let mut sum = 0.0f64;
            for &x in v.iter().take(length as usize) {
                sum += x;
            }
            sum
        }

        unsafe extern "C" fn smoothen(v: &mut [f64], length: i32) {
            // Preserve original in-place forward smoothing and fixed divisor N_SMOOTH.
            let len = length as usize;
            for i in 0..len {
                let mut sum = 0.0f64;
                let mut j = 0i32;
                while j < N_SMOOTH && (i as i32 + j) < length {
                    sum += v[i + j as usize];
                    j += 1;
                }
                v[i] = sum / (N_SMOOTH as f64);
            }
        }

        unsafe extern "C" fn differentiate(v: &mut [f64], length: i32) {
            let len = length as usize;
            for i in 0..(len - 1) {
                v[i] = v[i + 1] - v[i];
            }
            v[len - 1] = 0.0;
        }

        unsafe extern "C" fn preprocess(v: &mut [f64], source: &[f64], length: i32) {
            let len = length as usize;
            v[..len].copy_from_slice(&source[..len]);
            smoothen(v, length);
            differentiate(v, length);
            smoothen(v, length);
        }

        #[export_name = "match"]
        pub unsafe extern "C" fn match_0(test: &[f64], reference: &[f64], bins: i32, threshold: f64) -> i32 {
            let bins_usize = bins as usize;

            let mut t: Vec<float_t> = vec![0.0; bins_usize];
            let mut r: Vec<float_t> = vec![0.0; bins_usize];

            if total(test, bins) < threshold * total(reference, bins) {
                return 0;
            }

            preprocess(&mut t, test, bins);
            preprocess(&mut r, reference, bins);

            // IMPORTANT: The original C2Rust used `bytemuck::cast_slice_mut(&mut t)` where
            // `t: Vec<f64>` and the callee expects `&mut [f32]`. This is a byte reinterpretation:
            // the f64 buffer is viewed as 2*bins f32 values, and only the first `bins` are used.
            // Reproduce that exactly (endianness-dependent, as in the original).
            let t32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut t);
            let r32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut r);

            (spectral_contrast(t32, r32, bins) >= threshold) as i32
        }
    }
}

mod c_lib {
    pub static mut STDOUT_ERROR: i32 = 0;
    pub static mut STDERR_ERROR: i32 = 0;
    unsafe extern "C" {
        #[link_name = "stdout"]
        pub static mut STDOUT: *mut std::ffi::c_void;
        #[link_name = "stderr"]
        pub static mut STDERR: *mut std::ffi::c_void;
    }
}
```

**Entity:** c_lib::{STDOUT_ERROR, STDERR_ERROR, STDOUT, STDERR}

**States:** Handles valid/initialized (STDOUT/STDERR non-null, error flags meaningful), Handles invalid/uninitialized (null/dangling pointers, error flags stale)

**Transitions:**
- Handles invalid/uninitialized -> Handles valid/initialized via external C runtime initialization (implicit)
- Valid/initialized -> error flags updated via writes to STDOUT_ERROR/STDERR_ERROR (implicit)

**Evidence:** module c_lib: `pub static mut STDOUT_ERROR: i32` and `pub static mut STDERR_ERROR: i32` (mutable global error state); extern statics: `pub static mut STDOUT: *mut std::ffi::c_void;` and `pub static mut STDERR: *mut std::ffi::c_void;` (raw pointers with implicit validity requirements)

**Implementation:** Hide the `static mut` behind a safe API that hands out a `StdIo<'a>` capability token created once (e.g., `fn stdio() -> Option<StdIo>` that checks non-null). Methods on `StdIo` can encapsulate updating error flags and prevent arbitrary mutation. If mutation must remain, wrap pointers in `NonNull<c_void>` inside the capability to encode non-null at the type level.

---

### 2. Preprocess + byte-reinterpretation protocol (Vec<f64> treated as &[f32] with size/endianness constraints)

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: match_0 relies on a multi-step protocol: allocate `Vec<f64>` of length `bins`, run `preprocess` (which expects buffers to have at least `bins`), then reinterpret the underlying bytes as `&mut [f32]` via `bytemuck::cast_slice_mut::<f64,f32>` and call `spectral_contrast` using the original `bins` as the f32-length. This implicitly assumes (1) `bins` is non-negative and fits in usize, (2) the `Vec<f64>` has enough bytes for at least `bins` f32 values (true only if `bins <= 2*bins` i.e. relies on the f64->f32 widening trick), and (3) the byte-level reinterpretation is intended/acceptable and is endianness-dependent as documented. None of these constraints are expressed in the type system; a negative `bins` or misuse of the cast would be catastrophic or produce meaningless results.

**Evidence**:

```rust
#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(builtin_syntax)]
#![feature(core_intrinsics)]
#![feature(derive_clone_copy)]
#![feature(hint_must_use)]
#![feature(panic_internals)]
#![feature(formatting_options)]
#![feature(coverage_attribute)]

pub mod src {
    pub mod spectral_contrast {
        pub type float_t = f32;

        unsafe extern "C" fn dot_product(a: &[f32], b: &[f32], length: i32) -> f64 {
            let mut sum = 0.0f64;
            let len = length as usize;
            for i in 0..len {
                sum += (a[i] * b[i]) as f64;
            }
            sum
        }

        unsafe extern "C" fn normalize(v: *mut f32, length: i32) {
            // Match original C2Rust behavior: it created slices of length 100000 when non-null,
            // then used `length` for the loop bounds. We can safely use `length` for the slice.
            if v.is_null() {
                return;
            }
            let len = length as usize;
            let slice = std::slice::from_raw_parts_mut(v, len);
            let magnitude = dot_product(slice, slice, length).sqrt();
            for x in slice.iter_mut() {
                *x = (*x as f64 / magnitude) as float_t;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn spectral_contrast(a: &mut [f32], b: &mut [f32], length: i32) -> f64 {
            normalize(a.as_mut_ptr(), length);
            normalize(b.as_mut_ptr(), length);
            dot_product(a, b, length)
        }
    }

    pub mod lib {
        use crate::src::spectral_contrast::spectral_contrast;

        extern "C" {
            fn memcpy(
                __dest: *mut core::ffi::c_void,
                __src: *const core::ffi::c_void,
                __n: usize,
            ) -> *mut core::ffi::c_void;
        }

        pub type size_t = usize;
        pub type float_t = f64;

        pub const N_SMOOTH: i32 = 16;

        unsafe extern "C" fn total(v: &[f64], length: i32) -> f64 {
            let mut sum = 0.0f64;
            for &x in v.iter().take(length as usize) {
                sum += x;
            }
            sum
        }

        unsafe extern "C" fn smoothen(v: &mut [f64], length: i32) {
            // Preserve original in-place forward smoothing and fixed divisor N_SMOOTH.
            let len = length as usize;
            for i in 0..len {
                let mut sum = 0.0f64;
                let mut j = 0i32;
                while j < N_SMOOTH && (i as i32 + j) < length {
                    sum += v[i + j as usize];
                    j += 1;
                }
                v[i] = sum / (N_SMOOTH as f64);
            }
        }

        unsafe extern "C" fn differentiate(v: &mut [f64], length: i32) {
            let len = length as usize;
            for i in 0..(len - 1) {
                v[i] = v[i + 1] - v[i];
            }
            v[len - 1] = 0.0;
        }

        unsafe extern "C" fn preprocess(v: &mut [f64], source: &[f64], length: i32) {
            let len = length as usize;
            v[..len].copy_from_slice(&source[..len]);
            smoothen(v, length);
            differentiate(v, length);
            smoothen(v, length);
        }

        #[export_name = "match"]
        pub unsafe extern "C" fn match_0(test: &[f64], reference: &[f64], bins: i32, threshold: f64) -> i32 {
            let bins_usize = bins as usize;

            let mut t: Vec<float_t> = vec![0.0; bins_usize];
            let mut r: Vec<float_t> = vec![0.0; bins_usize];

            if total(test, bins) < threshold * total(reference, bins) {
                return 0;
            }

            preprocess(&mut t, test, bins);
            preprocess(&mut r, reference, bins);

            // IMPORTANT: The original C2Rust used `bytemuck::cast_slice_mut(&mut t)` where
            // `t: Vec<f64>` and the callee expects `&mut [f32]`. This is a byte reinterpretation:
            // the f64 buffer is viewed as 2*bins f32 values, and only the first `bins` are used.
            // Reproduce that exactly (endianness-dependent, as in the original).
            let t32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut t);
            let r32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut r);

            (spectral_contrast(t32, r32, bins) >= threshold) as i32
        }
    }
}

mod c_lib {
    pub static mut STDOUT_ERROR: i32 = 0;
    pub static mut STDERR_ERROR: i32 = 0;
    unsafe extern "C" {
        #[link_name = "stdout"]
        pub static mut STDOUT: *mut std::ffi::c_void;
        #[link_name = "stderr"]
        pub static mut STDERR: *mut std::ffi::c_void;
    }
}
```

**Entity:** src::lib::match_0

**States:** Prepared buffers (preprocess done, cast length sufficient, bins valid), Unprepared/invalid (bins too small/negative, cast not semantically valid)

**Transitions:**
- Unprepared/invalid -> Prepared buffers via: allocate t/r with bins, pass bins through preprocess, then reinterpret via cast_slice_mut, then call spectral_contrast

**Evidence:** pub fn match_0(test: &[f64], reference: &[f64], bins: i32, threshold: f64): `let bins_usize = bins as usize;` (negative bins becomes huge usize); allocations depend on bins: `let mut t: Vec<float_t> = vec![0.0; bins_usize];` and same for r; preprocess assumes `v[..len].copy_from_slice(&source[..len]);` where `len = length as usize` (requires test/reference and v to be at least bins); comment: "IMPORTANT ... byte reinterpretation ... f64 buffer is viewed as 2*bins f32 values ... endianness-dependent"; `let t32: &mut [f32] = bytemuck::cast_slice_mut::<f64, f32>(&mut t);` and call `spectral_contrast(t32, r32, bins)` (passes bins as f32 length without tying it to t32.len())

**Implementation:** Model the pipeline with types: `struct RawBins(i32)` -> `struct Bins(usize)` via `TryFrom` (reject negative/too large). Use `struct PreprocessedF64(Vec<f64>);` returned from `preprocess_owned(test: &[f64], bins: Bins) -> PreprocessedF64`. Then provide an explicit, named conversion `fn as_f32_words_mut(&mut self) -> F32Words<'_>` that exposes `&mut [f32]` along with a statically-related length (e.g., `struct F32Words<'a> { words: &'a mut [f32], bins: Bins }`) so `spectral_contrast` can take `F32Words` instead of `(slice, i32)` and cannot be called before preprocessing/casting.

---

