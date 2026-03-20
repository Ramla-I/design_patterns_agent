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

### 2. Contrast ratio numeric-domain preconditions (nonnegative luminance; avoid division by zero; standard uses (L1+0.05)/(L2+0.05))

**Location**: `/data/test_case/lib.rs:1-59`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: cbContrastRatio computes `high / low` after converting to luminance. This implicitly requires luminance values to be nonnegative and (for a finite result) `low != 0`. With sRGB inputs, black (all zeros) yields luminance 0, so the function can produce Inf (or NaN if both are zero). The type system does not encode any of these numeric-domain constraints (bounded channels, nonzero denominator), nor does the API signal that some inputs may produce non-finite outputs.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cb_rgb_255, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cb_rgb_255 {
    pub R: u8,
    pub G: u8,
    pub B: u8,
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if (c as f64) > 0.04045f64 {
        (((c as f64) + 0.055f64) / 1.055f64).powf(2.4f64) as f32
    } else {
        (c as f64 / 12.92f64) as f32
    }
}

#[inline]
fn cbLuminance(R: f32, G: f32, B: f32) -> f32 {
    let r = srgb_to_linear(R);
    let g = srgb_to_linear(G);
    let b = srgb_to_linear(B);
    0.2126f32 * r + 0.7152f32 * g + 0.0722f32 * b
}

#[inline]
fn cbContrastRatio(RA: f32, GA: f32, BA: f32, RB: f32, GB: f32, BB: f32) -> f32 {
    let lum_a = cbLuminance(RA, GA, BA);
    let lum_b = cbLuminance(RB, GB, BB);

    let (high, low) = if lum_a < lum_b { (lum_b, lum_a) } else { (lum_a, lum_b) };
    high / low
}

#[no_mangle]
pub extern "C" fn contrast_ratio(A: cb_rgb_255, B: cb_rgb_255) -> f32 {
    const INV_255: f32 = 1.0f32 / 255.0f32;

    cbContrastRatio(
        A.R as f32 * INV_255,
        A.G as f32 * INV_255,
        A.B as f32 * INV_255,
        B.R as f32 * INV_255,
        B.G as f32 * INV_255,
        B.B as f32 * INV_255,
    )
}
```

**Entity:** cbContrastRatio (and helpers cbLuminance/srgb_to_linear)

**States:** ValidInputs (channels in 0..=1, luminance > 0), InvalidInputs (out-of-range channels or luminance == 0 leading to Inf/NaN)

**Transitions:**
- ValidInputs -> InvalidInputs when low luminance becomes 0 (e.g., pure black) and `high / low` divides by zero

**Evidence:** cbContrastRatio(): `let (high, low) = ...; high / low` has no guard for `low == 0.0`; cbLuminance(): returns `0.2126*r + 0.7152*g + 0.0722*b`; with R=G=B=0, luminance is 0; srgb_to_linear(c: f32) accepts any f32; no clamping/validation that c is within 0..=1

**Implementation:** Use validated newtypes like `struct UnitF32(f32)` (ensuring 0.0..=1.0) and `struct PositiveF32(f32)` (ensuring > 0.0) / `NonZeroF32` (custom wrapper) for denominators. Have `cbLuminance` return a `NonNegativeF32`, and define `cbContrastRatio` to take `NonNegativeF32` and return `Option<f32>`/`NonZero`-guarded computation, or model the W3C-style ratio with an added epsilon/newtype that encodes the +0.05 offset if that’s the intended standard.

---

### 1. cb_rgb_255 color-space/range precondition (sRGB 0..255 -> normalized 0..1)

**Location**: `/data/test_case/lib.rs:1-59`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The contrast computation assumes inputs represent sRGB channel values and that they are normalized to 0.0..=1.0 before calling srgb_to_linear()/cbLuminance()/cbContrastRatio(). This is enforced only by convention: cb_rgb_255 stores u8 channels (implicitly 'raw 0..255 sRGB'), and contrast_ratio() performs the normalization internally using INV_255. The type system does not distinguish 'raw u8 sRGB' from other possible interpretations (e.g., already-normalized floats, linear RGB, HDR) and does not prevent accidentally bypassing normalization if cbContrastRatio/cbLuminance were reused elsewhere with unnormalized values.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cb_rgb_255, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cb_rgb_255 {
    pub R: u8,
    pub G: u8,
    pub B: u8,
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if (c as f64) > 0.04045f64 {
        (((c as f64) + 0.055f64) / 1.055f64).powf(2.4f64) as f32
    } else {
        (c as f64 / 12.92f64) as f32
    }
}

#[inline]
fn cbLuminance(R: f32, G: f32, B: f32) -> f32 {
    let r = srgb_to_linear(R);
    let g = srgb_to_linear(G);
    let b = srgb_to_linear(B);
    0.2126f32 * r + 0.7152f32 * g + 0.0722f32 * b
}

#[inline]
fn cbContrastRatio(RA: f32, GA: f32, BA: f32, RB: f32, GB: f32, BB: f32) -> f32 {
    let lum_a = cbLuminance(RA, GA, BA);
    let lum_b = cbLuminance(RB, GB, BB);

    let (high, low) = if lum_a < lum_b { (lum_b, lum_a) } else { (lum_a, lum_b) };
    high / low
}

#[no_mangle]
pub extern "C" fn contrast_ratio(A: cb_rgb_255, B: cb_rgb_255) -> f32 {
    const INV_255: f32 = 1.0f32 / 255.0f32;

    cbContrastRatio(
        A.R as f32 * INV_255,
        A.G as f32 * INV_255,
        A.B as f32 * INV_255,
        B.R as f32 * INV_255,
        B.G as f32 * INV_255,
        B.B as f32 * INV_255,
    )
}
```

**Entity:** cb_rgb_255

**States:** Raw8BitSrgb (0..=255 per channel), NormalizedSrgb (0.0..=1.0 per channel)

**Transitions:**
- Raw8BitSrgb -> NormalizedSrgb via contrast_ratio() scaling by INV_255 (A.R/G/B and B.R/G/B multiplied by 1/255)

**Evidence:** struct cb_rgb_255 { R: u8, G: u8, B: u8 } encodes 0..=255 channels but not the color space; contrast_ratio(): const INV_255: f32 = 1.0f32 / 255.0f32; then uses A.R as f32 * INV_255 etc.; srgb_to_linear(c: f32) uses the sRGB transfer function threshold 0.04045, implying c is in sRGB normalized domain

**Implementation:** Introduce distinct types for the domains, e.g. `struct Srgb8 { r: u8, g: u8, b: u8 }` and `struct Srgb01 { r: f32, g: f32, b: f32 }` (and optionally `struct LinearRgb { r: f32, g: f32, b: f32 }`). Provide `impl From<Srgb8> for Srgb01` to do the INV_255 scaling, and make `srgb_to_linear` take `Srgb01` (or channel newtype) so it cannot be called with unnormalized values.

---

