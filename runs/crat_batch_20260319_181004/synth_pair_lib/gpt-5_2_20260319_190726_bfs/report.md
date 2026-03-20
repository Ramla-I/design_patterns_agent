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

### 1. PCM/Z buffer size & layout preconditions (expects 1024 samples and valid channel stride)

**Location**: `/data/test_case/lib.rs:1-79`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: synth_pair_internal implicitly assumes fixed-size working buffers (1024 elements) and a channel/layout contract for the output index (computed from nch). These requirements are only enforced by runtime length checks and a guarded write; the type system does not express that pcm and z must be length-1024, nor that nch must correspond to a valid output position for the second sample write.

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

#[inline]
fn mp3d_scale_pcm(sample: f32) -> i16 {
    // Clamp to i16 range with the same thresholds as the original.
    if sample as f64 >= 32766.5f64 {
        return i16::MAX;
    }
    if sample as f64 <= -32767.5f64 {
        return i16::MIN;
    }

    // Round-to-nearest with the original negative adjustment.
    let s = (sample + 0.5f32) as i16;
    let si = s as i32;
    (si - (si < 0) as i32) as i16
}

pub(crate) fn synth_pair_internal(pcm: &mut [i16], nch: i32, z: &[f32]) {
    // The original code assumes enough space; keep behavior but avoid panics on null inputs.
    if pcm.len() < 1024 || z.len() < 1024 {
        return;
    }

    const STRIDE: usize = 64;

    let mut a: f32;

    a = (z[14 * STRIDE] - z[0]) * 29.0;
    a += (z[1 * STRIDE] + z[13 * STRIDE]) * 213.0;
    a += (z[12 * STRIDE] - z[2 * STRIDE]) * 459.0;
    a += (z[3 * STRIDE] + z[11 * STRIDE]) * 2037.0;
    a += (z[10 * STRIDE] - z[4 * STRIDE]) * 5153.0;
    a += (z[5 * STRIDE] + z[9 * STRIDE]) * 6574.0;
    a += (z[8 * STRIDE] - z[6 * STRIDE]) * 37489.0;
    a += z[7 * STRIDE] * 75038.0;
    pcm[0] = mp3d_scale_pcm(a);

    let z = &z[2..];

    a = z[14 * STRIDE] * 104.0;
    a += z[12 * STRIDE] * 1567.0;
    a += z[10 * STRIDE] * 9727.0;
    a += z[8 * STRIDE] * 64019.0;
    a += z[6 * STRIDE] * -9975.0;
    a += z[4 * STRIDE] * -45.0;
    a += z[2 * STRIDE] * 146.0;
    a += z[0] * -5.0;

    let out_idx = (16i32 * nch) as isize;
    if out_idx >= 0 && (out_idx as usize) < pcm.len() {
        pcm[out_idx as usize] = mp3d_scale_pcm(a);
    }
}

#[no_mangle]
pub unsafe extern "C" fn synth_pair(pcm: *mut i16, nch: i32, z: *const f32) {
    synth_pair_internal(
        if pcm.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(pcm, 1024)
        },
        nch,
        if z.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(z, 1024)
        },
    )
}
```

**Entity:** synth_pair_internal(pcm: &mut [i16], nch: i32, z: &[f32])

**States:** InvalidInputs (too short buffers / bad nch), ValidInputs (buffers sized for 1024 and nch matches layout)

**Transitions:**
- InvalidInputs -> ValidInputs via caller providing correctly sized buffers and a valid nch

**Evidence:** in synth_pair_internal: `if pcm.len() < 1024 || z.len() < 1024 { return; }` length-gates the algorithm; in synth_pair_internal: `pcm[0] = ...` relies on the above check to ensure index 0 is valid for required output; in synth_pair_internal: `let out_idx = (16i32 * nch) as isize;` encodes an output-layout rule based on nch; in synth_pair_internal: `if out_idx >= 0 && (out_idx as usize) < pcm.len() { pcm[out_idx as usize] = ... }` is a runtime guard compensating for the lack of a type-level nch/layout guarantee; in synth_pair_internal: `let z = &z[2..];` further relies on the initial `z.len() >= 1024` precondition to keep subsequent fixed indexing like `z[14 * STRIDE]` safe

**Implementation:** Introduce fixed-size buffer wrappers so the function signature encodes the 1024-element requirement, e.g. `struct Pcm1024([i16; 1024]); struct Z1024([f32; 1024]);` (or `&mut [i16; 1024]` / `&[f32; 1024]`). Additionally wrap/validate `nch` into a `ChannelCount`/`Nch` newtype that ensures `(16*nch)` is within the intended output layout (e.g., only 0 or 1 for mono/stereo), eliminating the need for the runtime bounds check on `out_idx`.

---

## Protocol Invariants

### 2. FFI pointer validity/length protocol (null-or-1024-elements)

**Location**: `/data/test_case/lib.rs:1-79`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The extern "C" entry point implements an implicit protocol: callers may pass null pointers (handled by substituting empty slices, leading to early-return in the internal function), or they may pass non-null pointers that are assumed to reference at least 1024 contiguous elements. This is not expressed in the type system; it is encoded via null checks and `from_raw_parts[_mut](..., 1024)`, which becomes UB if the caller violates the 1024-element requirement.

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

#[inline]
fn mp3d_scale_pcm(sample: f32) -> i16 {
    // Clamp to i16 range with the same thresholds as the original.
    if sample as f64 >= 32766.5f64 {
        return i16::MAX;
    }
    if sample as f64 <= -32767.5f64 {
        return i16::MIN;
    }

    // Round-to-nearest with the original negative adjustment.
    let s = (sample + 0.5f32) as i16;
    let si = s as i32;
    (si - (si < 0) as i32) as i16
}

pub(crate) fn synth_pair_internal(pcm: &mut [i16], nch: i32, z: &[f32]) {
    // The original code assumes enough space; keep behavior but avoid panics on null inputs.
    if pcm.len() < 1024 || z.len() < 1024 {
        return;
    }

    const STRIDE: usize = 64;

    let mut a: f32;

    a = (z[14 * STRIDE] - z[0]) * 29.0;
    a += (z[1 * STRIDE] + z[13 * STRIDE]) * 213.0;
    a += (z[12 * STRIDE] - z[2 * STRIDE]) * 459.0;
    a += (z[3 * STRIDE] + z[11 * STRIDE]) * 2037.0;
    a += (z[10 * STRIDE] - z[4 * STRIDE]) * 5153.0;
    a += (z[5 * STRIDE] + z[9 * STRIDE]) * 6574.0;
    a += (z[8 * STRIDE] - z[6 * STRIDE]) * 37489.0;
    a += z[7 * STRIDE] * 75038.0;
    pcm[0] = mp3d_scale_pcm(a);

    let z = &z[2..];

    a = z[14 * STRIDE] * 104.0;
    a += z[12 * STRIDE] * 1567.0;
    a += z[10 * STRIDE] * 9727.0;
    a += z[8 * STRIDE] * 64019.0;
    a += z[6 * STRIDE] * -9975.0;
    a += z[4 * STRIDE] * -45.0;
    a += z[2 * STRIDE] * 146.0;
    a += z[0] * -5.0;

    let out_idx = (16i32 * nch) as isize;
    if out_idx >= 0 && (out_idx as usize) < pcm.len() {
        pcm[out_idx as usize] = mp3d_scale_pcm(a);
    }
}

#[no_mangle]
pub unsafe extern "C" fn synth_pair(pcm: *mut i16, nch: i32, z: *const f32) {
    synth_pair_internal(
        if pcm.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(pcm, 1024)
        },
        nch,
        if z.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(z, 1024)
        },
    )
}
```

**Entity:** synth_pair(pcm: *mut i16, nch: i32, z: *const f32)

**States:** NullPointerInputs (treated as empty slices), NonNullValidPointers (must point to 1024 elements), NonNullInvalidPointers (UB risk)

**Transitions:**
- NullPointerInputs -> (early return behavior) via synth_pair_internal length check
- NonNullValidPointers -> (safe processing) via from_raw_parts/from_raw_parts_mut(…, 1024)
- NonNullInvalidPointers -> (UB) via from_raw_parts/from_raw_parts_mut(…, 1024) with insufficient allocation

**Evidence:** in synth_pair: `if pcm.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(pcm, 1024) }` encodes 'null or pointer to 1024 i16s'; in synth_pair: `if z.is_null() { &[] } else { std::slice::from_raw_parts(z, 1024) }` encodes 'null or pointer to 1024 f32s'; in synth_pair_internal: `if pcm.len() < 1024 || z.len() < 1024 { return; }` makes null map to a no-op/early-return semantic

**Implementation:** Keep the raw-pointer `extern "C"` shim but immediately convert into validated wrappers: e.g. `struct Ptr1024<T>(*mut T);` constructed only after checking non-null and (if available) an explicit length argument from C. Alternatively, change the C API to `synth_pair(pcm: *mut i16, pcm_len: usize, z: *const f32, z_len: usize, nch: i32)` and create safe Rust `NonNull<[T]>`-like newtypes after validating lengths; then call a safe `fn synth_pair_internal(pcm: &mut [i16;1024], z: &[f32;1024], nch: Nch)`.

---

