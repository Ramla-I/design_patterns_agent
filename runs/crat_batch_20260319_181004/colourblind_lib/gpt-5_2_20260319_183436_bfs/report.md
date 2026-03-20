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

### 1. Colour impairment selector validity (0/1/2 only)

**Location**: `/data/test_case/lib.rs:1-61`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The `Impairment: cb_impairment` argument is a `u32` but only the values 0, 1, and 2 are meaningful. Other values cause an early return (no output written). This is an implicit validity invariant on the input domain that is not enforced by the type system; callers can pass any `u32` and silently get a no-op.

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

pub type cb_impairment = u32;

#[inline]
fn protanopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = 0.170_556_99 * r + 0.829_443_04 * g + 2.911_88e-9 * b;
    rgb[1] = 0.170_556_99 * r + 0.829_443_04 * g - 5.986_79e-10 * b;
    rgb[2] = -0.004_517_144 * r + 0.004_517_144 * g + b;
}

#[inline]
fn deuteranopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = 0.330_660_07 * r + 0.669_339_95 * g + 3.559_314e-9 * b;
    rgb[1] = 0.330_660_07 * r + 0.669_339_95 * g - 1.758_327e-9 * b;
    rgb[2] = -0.027_855_383 * r + 0.027_855_383 * g + b;
}

#[inline]
fn tritanopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = r + 0.127_398_86 * g - 0.127_398_86 * b;
    rgb[1] = -4.486e-11 * r + 0.873_909_3 * g + 0.126_090_7 * b;
    rgb[2] = 3.111_3e-10 * r + 0.873_909_3 * g + 0.126_090_7 * b;
}

#[no_mangle]
pub extern "C" fn colourblind(
    Impairment: cb_impairment,
    mut R: Option<&mut f32>,
    mut G: Option<&mut f32>,
    mut B: Option<&mut f32>,
) {
    let (Some(r), Some(g), Some(b)) = (R.as_deref_mut(), G.as_deref_mut(), B.as_deref_mut())
    else {
        return;
    };

    let mut rgb = [*r, *g, *b];

    match Impairment {
        0 => protanopia(&mut rgb),
        1 => deuteranopia(&mut rgb),
        2 => tritanopia(&mut rgb),
        _ => return,
    }

    *r = rgb[0];
    *g = rgb[1];
    *b = rgb[2];
}
```

**Entity:** cb_impairment / colourblind() Impairment argument

**States:** Protanopia(0), Deuteranopia(1), Tritanopia(2), InvalidOther(u32)

**Transitions:**
- InvalidOther(u32) -> (no-op return) via match _ => return
- Protanopia/Deuteranopia/Tritanopia -> (transform applied) via match 0/1/2 arms

**Evidence:** pub type cb_impairment = u32; (selector is an unconstrained integer type); match Impairment { 0 => protanopia(...), 1 => deuteranopia(...), 2 => tritanopia(...), _ => return } (runtime check with silent invalid-case return)

**Implementation:** Introduce a Rust-only `enum Impairment { Protanopia, Deuteranopia, Tritanopia }` (or `struct Impairment(u8)` with `TryFrom<u32>` validation). Expose a separate `extern "C"` shim that maps `u32` to the enum (`Option/Result`) and calls a safe internal function `fn colourblind_safe(imp: Impairment, rgb: &mut [f32;3])`.

---

## Protocol Invariants

### 2. All-or-nothing RGB output protocol (must provide all channels)

**Location**: `/data/test_case/lib.rs:1-61`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The function only performs the conversion if *all three* channel pointers are present; otherwise it returns immediately without writing anything. This is an implicit calling protocol: providing only R or only RG is treated as an invalid call. The requirement is currently enforced by runtime pattern matching over `Option<&mut f32>` and is not representable to the C caller at compile time (and not encoded as a single composite parameter).

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

pub type cb_impairment = u32;

#[inline]
fn protanopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = 0.170_556_99 * r + 0.829_443_04 * g + 2.911_88e-9 * b;
    rgb[1] = 0.170_556_99 * r + 0.829_443_04 * g - 5.986_79e-10 * b;
    rgb[2] = -0.004_517_144 * r + 0.004_517_144 * g + b;
}

#[inline]
fn deuteranopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = 0.330_660_07 * r + 0.669_339_95 * g + 3.559_314e-9 * b;
    rgb[1] = 0.330_660_07 * r + 0.669_339_95 * g - 1.758_327e-9 * b;
    rgb[2] = -0.027_855_383 * r + 0.027_855_383 * g + b;
}

#[inline]
fn tritanopia(rgb: &mut [f32; 3]) {
    let [r, g, b] = *rgb;
    rgb[0] = r + 0.127_398_86 * g - 0.127_398_86 * b;
    rgb[1] = -4.486e-11 * r + 0.873_909_3 * g + 0.126_090_7 * b;
    rgb[2] = 3.111_3e-10 * r + 0.873_909_3 * g + 0.126_090_7 * b;
}

#[no_mangle]
pub extern "C" fn colourblind(
    Impairment: cb_impairment,
    mut R: Option<&mut f32>,
    mut G: Option<&mut f32>,
    mut B: Option<&mut f32>,
) {
    let (Some(r), Some(g), Some(b)) = (R.as_deref_mut(), G.as_deref_mut(), B.as_deref_mut())
    else {
        return;
    };

    let mut rgb = [*r, *g, *b];

    match Impairment {
        0 => protanopia(&mut rgb),
        1 => deuteranopia(&mut rgb),
        2 => tritanopia(&mut rgb),
        _ => return,
    }

    *r = rgb[0];
    *g = rgb[1];
    *b = rgb[2];
}
```

**Entity:** colourblind() R/G/B pointer trio

**States:** AllChannelsPresent, SomeMissingOrNull

**Transitions:**
- SomeMissingOrNull -> (no-op return) via `let (Some(r), Some(g), Some(b)) = ... else { return; }`
- AllChannelsPresent -> (read/transform/write) via building `rgb` and writing back to *r/*g/*b

**Evidence:** colourblind(..., mut R: Option<&mut f32>, mut G: Option<&mut f32>, mut B: Option<&mut f32>) (three independent optional pointers); let (Some(r), Some(g), Some(b)) = (R.as_deref_mut(), G.as_deref_mut(), B.as_deref_mut()) else { return; }; (enforces all-or-nothing at runtime); writes only occur after this check: `*r = rgb[0]; *g = rgb[1]; *b = rgb[2];`

**Implementation:** Move the safe core to `fn apply(imp: Impairment, rgb: &mut [f32;3])`. For FFI, accept a single pointer to a 3-float struct/array (e.g., `*mut [f32;3]` or `*mut Rgb { r: f32, g: f32, b: f32 }`) so the API requires the complete triple. If separate pointers are required for ABI reasons, wrap them in a validated Rust struct `struct RgbPtrs<'a> { r: &'a mut f32, g: &'a mut f32, b: &'a mut f32 }` constructed only when all are non-null.

---

