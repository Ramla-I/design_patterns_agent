# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 3. btac1c index/predictor record validity (raw-bytes -> valid state)

**Location**: `/data/test_case/lib.rs:1-15`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: btac1c_idxstate_s is a plain #[repr(C)] Copy struct of numeric fields that likely encode a coherent predictor/index state. As written, any bit-pattern can be constructed (or transmuted/FFI-filled) and then used as if it were meaningful, but the type system does not enforce basic validity constraints (e.g., idx range, tag/fcn fields within expected domains, firfx initialized to sensible values, etc.). The implicit invariant is that instances must be validated/initialized according to the codec/predictor rules before being used, but this is not represented in types.

**Evidence**:

```rust
// Note: Other parts of this module contain: 16 free function(s)

pub type btac1c_idxstate = btac1c_idxstate_s;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct btac1c_idxstate_s {
    pub idx: btac1c_u16,
    pub lpred: btac1c_s16,
    pub rpred: btac1c_s16,
    pub tag: btac1c_byte,
    pub bcfcn: btac1c_byte,
    pub bsfcn: btac1c_byte,
    pub usefx: btac1c_byte,
    pub firfx: [[btac1c_s16; 8]; 4],
}

```

**Entity:** btac1c_idxstate_s (aka btac1c_idxstate)

**States:** Unvalidated/Raw, Validated

**Transitions:**
- Unvalidated/Raw -> Validated via (not shown) initialization/validation routine

**Evidence:** pub type btac1c_idxstate = btac1c_idxstate_s; (type alias exposes raw representation everywhere); #[repr(C)] on btac1c_idxstate_s (implies FFI/byte-level interchange where invalid bit patterns are possible); #[derive(Copy, Clone)] on btac1c_idxstate_s (permits implicit copying of potentially-unvalidated state); fields are all primitive numeric types with no domain restrictions: idx, lpred, rpred, tag, bcfcn, bsfcn, usefx, firfx

**Implementation:** Introduce validated wrappers for domain-limited fields (e.g., struct Tag(u8); struct Sfc(u8); struct Idx(u16);) with TryFrom primitives enforcing ranges, and wrap the whole record as either btac1c_idxstate_raw (repr(C)) plus btac1c_idxstate_valid (non-repr(C)) produced only by a fallible validate(raw) -> Result<Valid,_>. Use the Valid type in APIs that assume invariants.

---

### 2. Ring-buffer pointer/length precondition (8-sample window)

**Location**: `/data/test_case/lib.rs:1-306`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: samp() implicitly assumes `psamp` points to a contiguous ring buffer of at least 8 i32 samples and is non-null/aligned. Indexing uses `*psamp.add(((idx - back) & 7) as usize)`, so the valid address range is always within `[psamp, psamp+7]`. This safety contract is not expressed in the type system because `psamp` is a raw pointer and the expected length (8) is only implicit in the masking operation.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct btac1c_idxstate_s

#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type btac1c_idxstate = btac1c_idxstate_s;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct btac1c_idxstate_s {
    pub idx: btac1c_u16,
    pub lpred: btac1c_s16,
    pub rpred: btac1c_s16,
    pub tag: btac1c_byte,
    pub bcfcn: btac1c_byte,
    pub bsfcn: btac1c_byte,
    pub usefx: btac1c_byte,
    pub firfx: [[btac1c_s16; 8]; 4],
}
pub type btac1c_s16 = i16;
pub type btac1c_byte = u8;
pub type btac1c_u16 = u16;

type PredictFn = unsafe extern "C" fn(*mut i32, i32, i32, *mut btac1c_idxstate) -> i32;

#[inline(always)]
unsafe fn samp(psamp: *mut i32, idx: i32, back: i32) -> i32 {
    // Ring buffer of 8 samples, index masked like original C.
    *psamp.add(((idx - back) & 7) as usize)
}

unsafe extern "C" fn BTAC1C2_PredictSample(
    psamp: *mut i32,
    idx: i32,
    pfcn: i32,
    ridx: *mut btac1c_idxstate,
) -> i32 {
    match pfcn {
        0 => samp(psamp, idx, 1),
        1 => 2 * samp(psamp, idx, 1) - samp(psamp, idx, 2),
        2 => (3 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 1,
        3 => (5 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 2,
        4 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            p0 - (p1 >> 1)
        }
        5 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            (3 * p0 - p1) >> 2
        }
        6 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            (5 * p0 - p1) >> 3
        }
        7 => {
            (18 * samp(psamp, idx, 1)
                - 4 * samp(psamp, idx, 2)
                + 3 * samp(psamp, idx, 3)
                - 2 * samp(psamp, idx, 4)
                + samp(psamp, idx, 5))
                / 16
        }
        8 => {
            (72 * samp(psamp, idx, 1)
                - 16 * samp(psamp, idx, 2)
                + 12 * samp(psamp, idx, 3)
                - 8 * samp(psamp, idx, 4)
                + 5 * samp(psamp, idx, 5)
                - 3 * samp(psamp, idx, 6)
                + 3 * samp(psamp, idx, 7)
                - samp(psamp, idx, 8))
                / 64
        }
        9 => {
            (76 * samp(psamp, idx, 1)
                - 17 * samp(psamp, idx, 2)
                + 10 * samp(psamp, idx, 3)
                - 7 * samp(psamp, idx, 4)
                + 5 * samp(psamp, idx, 5)
                - 4 * samp(psamp, idx, 6)
                + 4 * samp(psamp, idx, 7)
                - 3 * samp(psamp, idx, 8))
                / 64
        }
        10 => {
            let p0 = samp(psamp, idx, 1)
                + samp(psamp, idx, 2)
                + samp(psamp, idx, 3)
                + samp(psamp, idx, 4);
            let p1 = samp(psamp, idx, 5)
                + samp(psamp, idx, 6)
                + samp(psamp, idx, 7)
                + samp(psamp, idx, 8);
            (5 * p0 - p1) >> 4
        }
        11 => {
            let p0 = samp(psamp, idx, 1)
                + samp(psamp, idx, 2)
                + samp(psamp, idx, 3)
                + samp(psamp, idx, 4);
            let p1 = samp(psamp, idx, 5)
                + samp(psamp, idx, 6)
                + samp(psamp, idx, 7)
                + samp(psamp, idx, 8);
            (p0 + p1) >> 3
        }
        12..=15 => {
            let st = &*ridx.cast::<btac1c_idxstate_s>();
            let fx = &st.firfx[(pfcn - 12) as usize];
            let acc = (fx[0] as i32) * samp(psamp, idx, 1)
                + (fx[1] as i32) * samp(psamp, idx, 2)
                + (fx[2] as i32) * samp(psamp, idx, 3)
                + (fx[3] as i32) * samp(psamp, idx, 4)
                + (fx[4] as i32) * samp(psamp, idx, 5)
                + (fx[5] as i32) * samp(psamp, idx, 6)
                + (fx[6] as i32) * samp(psamp, idx, 7)
                + (fx[7] as i32) * samp(psamp, idx, 8);
            acc / 256
        }
        _ => 0,
    }
}

unsafe extern "C" fn BTAC1C2_PredictSample_Pfn0(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    samp(psamp, idx, 1)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn1(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    2 * samp(psamp, idx, 1) - samp(psamp, idx, 2)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn2(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (3 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 1
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn3(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (5 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 2
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn4(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    p0 - (p1 >> 1)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn5(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    (3 * p0 - p1) >> 2
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn6(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    (5 * p0 - p1) >> 3
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn7(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (18 * samp(psamp, idx, 1)
        - 4 * samp(psamp, idx, 2)
        + 3 * samp(psamp, idx, 3)
        - 2 * samp(psamp, idx, 4)
        + samp(psamp, idx, 5))
        / 16
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn8(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (72 * samp(psamp, idx, 1)
        - 16 * samp(psamp, idx, 2)
        + 12 * samp(psamp, idx, 3)
        - 8 * samp(psamp, idx, 4)
        + 5 * samp(psamp, idx, 5)
        - 3 * samp(psamp, idx, 6)
        + 3 * samp(psamp, idx, 7)
        - samp(psamp, idx, 8))
        / 64
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn9(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (76 * samp(psamp, idx, 1)
        - 17 * samp(psamp, idx, 2)
        + 10 * samp(psamp, idx, 3)
        - 7 * samp(psamp, idx, 4)
        + 5 * samp(psamp, idx, 5)
        - 4 * samp(psamp, idx, 6)
        + 4 * samp(psamp, idx, 7)
        - 3 * samp(psamp, idx, 8))
        / 64
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn10(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2) + samp(psamp, idx, 3) + samp(psamp, idx, 4);
    let p1 = samp(psamp, idx, 5) + samp(psamp, idx, 6) + samp(psamp, idx, 7) + samp(psamp, idx, 8);
    (5 * p0 - p1) >> 3
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn11(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2) + samp(psamp, idx, 3) + samp(psamp, idx, 4);
    let p1 = samp(psamp, idx, 5) + samp(psamp, idx, 6) + samp(psamp, idx, 7) + samp(psamp, idx, 8);
    (p0 + p1) >> 1
}

#[inline]
unsafe fn BTAC1C2_GetPredictFunc(pfcn: i32) -> *mut std::ffi::c_void {
    let f: PredictFn = match pfcn {
        0 => BTAC1C2_PredictSample_Pfn0,
        1 => BTAC1C2_PredictSample_Pfn1,
        2 => BTAC1C2_PredictSample_Pfn2,
        3 => BTAC1C2_PredictSample_Pfn3,
        4 => BTAC1C2_PredictSample_Pfn4,
        5 => BTAC1C2_PredictSample_Pfn5,
        6 => BTAC1C2_PredictSample_Pfn6,
        7 => BTAC1C2_PredictSample_Pfn7,
        8 => BTAC1C2_PredictSample_Pfn8,
        9 => BTAC1C2_PredictSample_Pfn9,
        10 => BTAC1C2_PredictSample_Pfn10,
        11 => BTAC1C2_PredictSample_Pfn11,
        _ => BTAC1C2_PredictSample,
    };
    f as *mut std::ffi::c_void
}

#[no_mangle]
pub unsafe extern "C" fn get_predict_func(pfcn: i32) -> i32 {
    let fcn = BTAC1C2_GetPredictFunc(pfcn);

    let expected: Option<PredictFn> = match pfcn {
        0 => Some(BTAC1C2_PredictSample_Pfn0),
        1 => Some(BTAC1C2_PredictSample_Pfn1),
        2 => Some(BTAC1C2_PredictSample_Pfn2),
        3 => Some(BTAC1C2_PredictSample_Pfn3),
        4 => Some(BTAC1C2_PredictSample_Pfn4),
        5 => Some(BTAC1C2_PredictSample_Pfn5),
        6 => Some(BTAC1C2_PredictSample_Pfn6),
        7 => Some(BTAC1C2_PredictSample_Pfn7),
        8 => Some(BTAC1C2_PredictSample_Pfn8),
        9 => Some(BTAC1C2_PredictSample_Pfn9),
        10 => Some(BTAC1C2_PredictSample_Pfn10),
        11 => Some(BTAC1C2_PredictSample_Pfn11),
        _ => None,
    };

    match expected {
        Some(f) => (fcn == (f as *mut std::ffi::c_void)) as i32,
        None => 0,
    }
}
```

**Entity:** samp(psamp, idx, back)

**States:** ValidRingPtr(len>=8), InvalidRingPtr(len<8 or null)

**Transitions:**
- InvalidRingPtr(len<8 or null) -> UB when samp() dereferences
- ValidRingPtr(len>=8) -> safe logical ring access for any idx/back

**Evidence:** samp(psamp: *mut i32, idx: i32, back: i32) -> i32 uses `*psamp.add(((idx - back) & 7) as usize)` (requires at least 8 elements); comment in samp(): "Ring buffer of 8 samples" (documents the latent length invariant); BTAC1C2_PredictSample and BTAC1C2_PredictSample_Pfn{0..11} call samp() with back values up to 8 (e.g., samp(psamp, idx, 8) in pfcn 8/9/10/11 cases), reinforcing the required window size

**Implementation:** Wrap the buffer as `struct Ring8(*mut [i32; 8]);` or, for non-FFI internal code, accept `&mut [i32; 8]` / `&[i32; 8]` and index safely. At the FFI boundary, convert `*mut i32` to `NonNull<i32>` and a `Ring8` after validating non-null (and, if possible, provenance/length based on how the pointer is obtained). This makes the 8-element requirement explicit and eliminates the latent UB precondition from downstream callers.

---

## Protocol Invariants

### 1. Predict-function selection protocol (pfcn-dispatch table consistency)

**Location**: `/data/test_case/lib.rs:1-306`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on an implicit dispatch-table protocol: for pfcn in 0..=11, BTAC1C2_GetPredictFunc(pfcn) must return exactly the corresponding specialized function pointer (BTAC1C2_PredictSample_Pfn{N}); otherwise selection falls back to BTAC1C2_PredictSample. This mapping is duplicated in multiple match statements and only checked at runtime in get_predict_func by comparing raw void pointers. The type system does not encode (a) the valid pfcn set, (b) that the mapping is total/consistent, or (c) that the returned pointer is a PredictFn rather than an untyped c_void pointer.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct btac1c_idxstate_s

#![warn(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type btac1c_idxstate = btac1c_idxstate_s;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct btac1c_idxstate_s {
    pub idx: btac1c_u16,
    pub lpred: btac1c_s16,
    pub rpred: btac1c_s16,
    pub tag: btac1c_byte,
    pub bcfcn: btac1c_byte,
    pub bsfcn: btac1c_byte,
    pub usefx: btac1c_byte,
    pub firfx: [[btac1c_s16; 8]; 4],
}
pub type btac1c_s16 = i16;
pub type btac1c_byte = u8;
pub type btac1c_u16 = u16;

type PredictFn = unsafe extern "C" fn(*mut i32, i32, i32, *mut btac1c_idxstate) -> i32;

#[inline(always)]
unsafe fn samp(psamp: *mut i32, idx: i32, back: i32) -> i32 {
    // Ring buffer of 8 samples, index masked like original C.
    *psamp.add(((idx - back) & 7) as usize)
}

unsafe extern "C" fn BTAC1C2_PredictSample(
    psamp: *mut i32,
    idx: i32,
    pfcn: i32,
    ridx: *mut btac1c_idxstate,
) -> i32 {
    match pfcn {
        0 => samp(psamp, idx, 1),
        1 => 2 * samp(psamp, idx, 1) - samp(psamp, idx, 2),
        2 => (3 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 1,
        3 => (5 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 2,
        4 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            p0 - (p1 >> 1)
        }
        5 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            (3 * p0 - p1) >> 2
        }
        6 => {
            let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
            let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
            (5 * p0 - p1) >> 3
        }
        7 => {
            (18 * samp(psamp, idx, 1)
                - 4 * samp(psamp, idx, 2)
                + 3 * samp(psamp, idx, 3)
                - 2 * samp(psamp, idx, 4)
                + samp(psamp, idx, 5))
                / 16
        }
        8 => {
            (72 * samp(psamp, idx, 1)
                - 16 * samp(psamp, idx, 2)
                + 12 * samp(psamp, idx, 3)
                - 8 * samp(psamp, idx, 4)
                + 5 * samp(psamp, idx, 5)
                - 3 * samp(psamp, idx, 6)
                + 3 * samp(psamp, idx, 7)
                - samp(psamp, idx, 8))
                / 64
        }
        9 => {
            (76 * samp(psamp, idx, 1)
                - 17 * samp(psamp, idx, 2)
                + 10 * samp(psamp, idx, 3)
                - 7 * samp(psamp, idx, 4)
                + 5 * samp(psamp, idx, 5)
                - 4 * samp(psamp, idx, 6)
                + 4 * samp(psamp, idx, 7)
                - 3 * samp(psamp, idx, 8))
                / 64
        }
        10 => {
            let p0 = samp(psamp, idx, 1)
                + samp(psamp, idx, 2)
                + samp(psamp, idx, 3)
                + samp(psamp, idx, 4);
            let p1 = samp(psamp, idx, 5)
                + samp(psamp, idx, 6)
                + samp(psamp, idx, 7)
                + samp(psamp, idx, 8);
            (5 * p0 - p1) >> 4
        }
        11 => {
            let p0 = samp(psamp, idx, 1)
                + samp(psamp, idx, 2)
                + samp(psamp, idx, 3)
                + samp(psamp, idx, 4);
            let p1 = samp(psamp, idx, 5)
                + samp(psamp, idx, 6)
                + samp(psamp, idx, 7)
                + samp(psamp, idx, 8);
            (p0 + p1) >> 3
        }
        12..=15 => {
            let st = &*ridx.cast::<btac1c_idxstate_s>();
            let fx = &st.firfx[(pfcn - 12) as usize];
            let acc = (fx[0] as i32) * samp(psamp, idx, 1)
                + (fx[1] as i32) * samp(psamp, idx, 2)
                + (fx[2] as i32) * samp(psamp, idx, 3)
                + (fx[3] as i32) * samp(psamp, idx, 4)
                + (fx[4] as i32) * samp(psamp, idx, 5)
                + (fx[5] as i32) * samp(psamp, idx, 6)
                + (fx[6] as i32) * samp(psamp, idx, 7)
                + (fx[7] as i32) * samp(psamp, idx, 8);
            acc / 256
        }
        _ => 0,
    }
}

unsafe extern "C" fn BTAC1C2_PredictSample_Pfn0(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    samp(psamp, idx, 1)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn1(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    2 * samp(psamp, idx, 1) - samp(psamp, idx, 2)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn2(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (3 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 1
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn3(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (5 * samp(psamp, idx, 1) - samp(psamp, idx, 2)) >> 2
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn4(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    p0 - (p1 >> 1)
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn5(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    (3 * p0 - p1) >> 2
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn6(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2);
    let p1 = samp(psamp, idx, 2) + samp(psamp, idx, 3);
    (5 * p0 - p1) >> 3
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn7(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (18 * samp(psamp, idx, 1)
        - 4 * samp(psamp, idx, 2)
        + 3 * samp(psamp, idx, 3)
        - 2 * samp(psamp, idx, 4)
        + samp(psamp, idx, 5))
        / 16
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn8(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (72 * samp(psamp, idx, 1)
        - 16 * samp(psamp, idx, 2)
        + 12 * samp(psamp, idx, 3)
        - 8 * samp(psamp, idx, 4)
        + 5 * samp(psamp, idx, 5)
        - 3 * samp(psamp, idx, 6)
        + 3 * samp(psamp, idx, 7)
        - samp(psamp, idx, 8))
        / 64
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn9(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    (76 * samp(psamp, idx, 1)
        - 17 * samp(psamp, idx, 2)
        + 10 * samp(psamp, idx, 3)
        - 7 * samp(psamp, idx, 4)
        + 5 * samp(psamp, idx, 5)
        - 4 * samp(psamp, idx, 6)
        + 4 * samp(psamp, idx, 7)
        - 3 * samp(psamp, idx, 8))
        / 64
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn10(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2) + samp(psamp, idx, 3) + samp(psamp, idx, 4);
    let p1 = samp(psamp, idx, 5) + samp(psamp, idx, 6) + samp(psamp, idx, 7) + samp(psamp, idx, 8);
    (5 * p0 - p1) >> 3
}
unsafe extern "C" fn BTAC1C2_PredictSample_Pfn11(
    psamp: *mut i32,
    idx: i32,
    _: i32,
    _: *mut btac1c_idxstate,
) -> i32 {
    let p0 = samp(psamp, idx, 1) + samp(psamp, idx, 2) + samp(psamp, idx, 3) + samp(psamp, idx, 4);
    let p1 = samp(psamp, idx, 5) + samp(psamp, idx, 6) + samp(psamp, idx, 7) + samp(psamp, idx, 8);
    (p0 + p1) >> 1
}

#[inline]
unsafe fn BTAC1C2_GetPredictFunc(pfcn: i32) -> *mut std::ffi::c_void {
    let f: PredictFn = match pfcn {
        0 => BTAC1C2_PredictSample_Pfn0,
        1 => BTAC1C2_PredictSample_Pfn1,
        2 => BTAC1C2_PredictSample_Pfn2,
        3 => BTAC1C2_PredictSample_Pfn3,
        4 => BTAC1C2_PredictSample_Pfn4,
        5 => BTAC1C2_PredictSample_Pfn5,
        6 => BTAC1C2_PredictSample_Pfn6,
        7 => BTAC1C2_PredictSample_Pfn7,
        8 => BTAC1C2_PredictSample_Pfn8,
        9 => BTAC1C2_PredictSample_Pfn9,
        10 => BTAC1C2_PredictSample_Pfn10,
        11 => BTAC1C2_PredictSample_Pfn11,
        _ => BTAC1C2_PredictSample,
    };
    f as *mut std::ffi::c_void
}

#[no_mangle]
pub unsafe extern "C" fn get_predict_func(pfcn: i32) -> i32 {
    let fcn = BTAC1C2_GetPredictFunc(pfcn);

    let expected: Option<PredictFn> = match pfcn {
        0 => Some(BTAC1C2_PredictSample_Pfn0),
        1 => Some(BTAC1C2_PredictSample_Pfn1),
        2 => Some(BTAC1C2_PredictSample_Pfn2),
        3 => Some(BTAC1C2_PredictSample_Pfn3),
        4 => Some(BTAC1C2_PredictSample_Pfn4),
        5 => Some(BTAC1C2_PredictSample_Pfn5),
        6 => Some(BTAC1C2_PredictSample_Pfn6),
        7 => Some(BTAC1C2_PredictSample_Pfn7),
        8 => Some(BTAC1C2_PredictSample_Pfn8),
        9 => Some(BTAC1C2_PredictSample_Pfn9),
        10 => Some(BTAC1C2_PredictSample_Pfn10),
        11 => Some(BTAC1C2_PredictSample_Pfn11),
        _ => None,
    };

    match expected {
        Some(f) => (fcn == (f as *mut std::ffi::c_void)) as i32,
        None => 0,
    }
}
```

**Entity:** PredictFn / get_predict_func / BTAC1C2_GetPredictFunc

**States:** KnownPfcn(0..=11), FallbackPfcn(other)

**Transitions:**
- KnownPfcn(0..=11) -> returns specialized PredictFn via BTAC1C2_GetPredictFunc()
- FallbackPfcn(other) -> returns generic BTAC1C2_PredictSample via BTAC1C2_GetPredictFunc()

**Evidence:** type PredictFn = unsafe extern "C" fn(*mut i32, i32, i32, *mut btac1c_idxstate) -> i32; (implicit required signature for all returned function pointers); BTAC1C2_GetPredictFunc(pfcn): match pfcn { 0 => BTAC1C2_PredictSample_Pfn0, ... 11 => BTAC1C2_PredictSample_Pfn11, _ => BTAC1C2_PredictSample } (hard-coded dispatch mapping); BTAC1C2_GetPredictFunc returns *mut std::ffi::c_void (erases the PredictFn type); get_predict_func(pfcn): duplicates the same match into `expected: Option<PredictFn>` and checks `fcn == (f as *mut c_void)` (runtime check that the mapping is consistent)

**Implementation:** Introduce a `#[repr(i32)] enum Pfcn { P0=0, ..., P11=11 }` plus `TryFrom<i32>` for validation. Make `BTAC1C2_GetPredictFunc(pfcn: Pfcn) -> PredictFn` return a typed function pointer (no c_void). If an FFI boundary must return an integer/pointer, provide a separate wrapper `fn as_void(f: PredictFn) -> *mut c_void` while keeping the internal mapping single-sourced (e.g., const array `[PredictFn; 12]`). This removes the need for runtime pointer-equality self-tests and prevents invalid pfcn values from reaching the 'KnownPfcn' path.

---

