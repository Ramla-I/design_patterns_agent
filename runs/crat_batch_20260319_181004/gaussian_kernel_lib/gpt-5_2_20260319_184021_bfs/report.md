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

### 1. Gaussian kernel input validity (NonZeroSize & NonZeroRadius) + buffer capacity coupling

**Location**: `/data/test_case/lib.rs:1-55`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The function has an implicit precondition that meaningful work only occurs when `size > 0` and `radius != 0.0`. Otherwise it returns early (NoOpInputs). Additionally, the intended amount of output is coupled to `size`, but correctness/intent depends on `dest` having at least `min(size, dest.len())` writable elements; this is enforced by runtime bounds logic (`break`/`min`) rather than by the type system. The type system does not distinguish the 'will compute and normalize' path from the 'no-op' path, nor does it encode that the destination buffer must be large enough for the requested kernel size.

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

pub(crate) fn gaussian_kernel_internal(dest: &mut [f32], size: i32, radius: f32) {
    if size <= 0 || radius == 0.0 {
        return;
    }

    let sigma: f32 = 1.6f32;
    let tetha: f32 = 2.25f32;
    let hsize: i32 = size / 2;

    let s2: f32 = 1.0f32 / (sigma * sigma * tetha).exp();
    let rs: f32 = sigma / radius;

    let mut sum: f32 = 0.0f32;

    for (i, r) in (-hsize..=hsize).enumerate() {
        if i >= dest.len() {
            break;
        }
        let x: f32 = r as f32 * rs;
        let v: f32 = (1.0f32 / (x * x).exp() - s2).max(0.0f32);
        dest[i] = v;
        sum += v;
    }

    if sum > 0.0f32 {
        let isum: f32 = 1.0f32 / sum;
        let n = (size as usize).min(dest.len());
        for v in &mut dest[..n] {
            *v *= isum;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn gaussian_kernel(dest: *mut f32, size: i32, radius: f32) {
    gaussian_kernel_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        size,
        radius,
    )
}
```

**Entity:** gaussian_kernel_internal(dest: &mut [f32], size: i32, radius: f32)

**States:** ValidInputs, NoOpInputs

**Transitions:**
- NoOpInputs -> ValidInputs by calling with size>0 and radius!=0.0

**Evidence:** gaussian_kernel_internal: `if size <= 0 || radius == 0.0 { return; }` encodes an input-state split; gaussian_kernel_internal loop: `if i >= dest.len() { break; }` shows runtime enforcement of dest capacity; normalization: `let n = (size as usize).min(dest.len()); for v in &mut dest[..n] { ... }` couples `size` to required writable output length

**Implementation:** Introduce validated parameter types like `NonZeroI32` (or `PositiveI32`) for `size` and a `NonZeroF32`-like wrapper for `radius` (constructed via `TryFrom<f32>` rejecting 0.0/NaN). Optionally take `dest: &mut [f32; N]` (const generic) or a `KernelDest<'a>` newtype that guarantees `dest.len() >= size as usize` at construction time, eliminating the internal `break/min` truncation semantics.

---

## Protocol Invariants

### 2. FFI pointer validity and fixed-capacity output protocol (null vs non-null, length=1024)

**Location**: `/data/test_case/lib.rs:1-55`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The exported FFI function encodes a protocol: if `dest` is null, the call is treated as a no-op sink (`&mut []`), otherwise `dest` must point to a writable buffer of at least 1024 `f32` elements for the duration of the call. This requirement is not represented in the type system (raw pointer + unsafe), and the function unconditionally creates a `&mut [f32]` of length 1024 from any non-null pointer, which is only sound if the caller upholds the implicit buffer-size and aliasing rules.

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

pub(crate) fn gaussian_kernel_internal(dest: &mut [f32], size: i32, radius: f32) {
    if size <= 0 || radius == 0.0 {
        return;
    }

    let sigma: f32 = 1.6f32;
    let tetha: f32 = 2.25f32;
    let hsize: i32 = size / 2;

    let s2: f32 = 1.0f32 / (sigma * sigma * tetha).exp();
    let rs: f32 = sigma / radius;

    let mut sum: f32 = 0.0f32;

    for (i, r) in (-hsize..=hsize).enumerate() {
        if i >= dest.len() {
            break;
        }
        let x: f32 = r as f32 * rs;
        let v: f32 = (1.0f32 / (x * x).exp() - s2).max(0.0f32);
        dest[i] = v;
        sum += v;
    }

    if sum > 0.0f32 {
        let isum: f32 = 1.0f32 / sum;
        let n = (size as usize).min(dest.len());
        for v in &mut dest[..n] {
            *v *= isum;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn gaussian_kernel(dest: *mut f32, size: i32, radius: f32) {
    gaussian_kernel_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        size,
        radius,
    )
}
```

**Entity:** gaussian_kernel(dest: *mut f32, size: i32, radius: f32) (extern "C")

**States:** NullDest, NonNullDestValidFor1024Writes

**Transitions:**
- NullDest -> NonNullDestValidFor1024Writes by passing a non-null pointer to >=1024 f32s

**Evidence:** gaussian_kernel: `if dest.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dest, 1024) }` encodes the null/non-null state split and the fixed length requirement; gaussian_kernel signature: `pub unsafe extern "C" fn gaussian_kernel(dest: *mut f32, ...)` indicates the protocol is enforced only by caller discipline (unsafe/raw pointer)

**Implementation:** Provide a safe Rust wrapper that requires `&mut [f32; 1024]` (or `&mut [f32]` checked to be len>=1024) and exposes a separate raw-FFI entrypoint. E.g., `pub fn gaussian_kernel_safe(dest: &mut [f32; 1024], size: PositiveI32, radius: NonZeroRadius)`. The raw `extern "C"` function becomes a thin adapter that validates (or documents) the capability requirements, while Rust callers use the capability-typed API.

---

