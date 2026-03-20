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

### 2. Tritanopia parameter non-null/complete-channel precondition (all Some)

**Location**: `/data/test_case/lib.rs:1-114`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Tritanopia is written as if its three channel parameters are always present, but it accepts `Option<&mut f32>` and immediately unwraps all three. This creates an implicit precondition that callers must pass `Some` for Red/Green/Blue; otherwise the function will panic. The type system could enforce "all three channels are provided" by taking `&mut cb_rgb` (or a dedicated channel struct) instead of three `Option`s.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cb_rgb_255, 3 free function(s); struct cb_rgb, 2 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cb_rgb {
    pub R: f32,
    pub G: f32,
    pub B: f32,
}

#[inline]
fn cbRemoveGammaRGB(RGB: cb_rgb) -> cb_rgb {
    #[inline]
    fn remove_gamma(c: f32) -> f32 {
        let c64 = c as f64;
        if c64 > 0.04045f64 {
            (((c64 + 0.055f64) / 1.055f64).powf(2.4f64)) as f32
        } else {
            (c64 / 12.92f64) as f32
        }
    }

    cb_rgb {
        R: remove_gamma(RGB.R),
        G: remove_gamma(RGB.G),
        B: remove_gamma(RGB.B),
    }
}

#[inline]
fn cbNorm(RGB: cb_rgb_255) -> cb_rgb {
    cb_rgb {
        R: RGB.R as f32 / 255.0f32,
        G: RGB.G as f32 / 255.0f32,
        B: RGB.B as f32 / 255.0f32,
    }
}

#[inline]
fn cbDenorm(RGB: cb_rgb) -> cb_rgb_255 {
    // Match C-style float->u8 conversion behavior more closely:
    // - add 0.5 then truncate toward zero
    // - wrap on overflow (as C does for unsigned narrowing conversions)
    #[inline]
    fn to_u8_wrapping(x: f32) -> u8 {
        let t = (x * 255.0f32 + 0.5f32) as i32;
        (t as u32 as u8)
    }

    cb_rgb_255 {
        R: to_u8_wrapping(RGB.R),
        G: to_u8_wrapping(RGB.G),
        B: to_u8_wrapping(RGB.B),
    }
}

#[inline]
fn cbApplyGammaRGB(RGB: cb_rgb) -> cb_rgb {
    #[inline]
    fn apply_gamma(c: f32) -> f32 {
        let c64 = c as f64;
        if c64 > 0.003_130_804_953_560_371_3_f64 {
            (1.055f64 * c64.powf(0.4166666666f64) - 0.055f64) as f32
        } else {
            (c64 * 12.92f64) as f32
        }
    }

    cb_rgb {
        R: apply_gamma(RGB.R),
        G: apply_gamma(RGB.G),
        B: apply_gamma(RGB.B),
    }
}

#[inline]
fn Tritanopia(Red: Option<&mut f32>, Green: Option<&mut f32>, Blue: Option<&mut f32>) {
    let R: f32 = *Red.as_deref().unwrap();
    let G: f32 = *Green.as_deref().unwrap();
    let B: f32 = *Blue.as_deref().unwrap();
    *Red.unwrap() = R + 0.127_398_86_f32 * G - 0.127_398_86_f32 * B;
    *Green.unwrap() = -4.486E-11f32 * R + 0.873_909_3_f32 * G + 0.126_090_7_f32 * B;
    *Blue.unwrap() = 3.1113E-10f32 * R + 0.873_909_3_f32 * G + 0.126_090_7_f32 * B;
}

#[no_mangle]
pub extern "C" fn tritanopia(RGB: cb_rgb_255) -> cb_rgb_255 {
    let mut RGBNorm: cb_rgb = cbRemoveGammaRGB(cbNorm(RGB));
    Tritanopia(
        Some(&mut RGBNorm.R),
        Some(&mut RGBNorm.G),
        Some(&mut RGBNorm.B),
    );
    let Result: cb_rgb_255 = cbDenorm(cbApplyGammaRGB(RGBNorm));
    Result
}
```

**Entity:** Tritanopia(Red: Option<&mut f32>, Green: Option<&mut f32>, Blue: Option<&mut f32>)

**States:** AllChannelsPresent, SomeChannelMissing

**Transitions:**
- SomeChannelMissing -> panic via `as_deref().unwrap()` / `unwrap()`
- AllChannelsPresent -> AllChannelsPresent with mutated channel values via Tritanopia()

**Evidence:** fn Tritanopia(Red: Option<&mut f32>, ...) takes Option parameters rather than mandatory references; let R: f32 = *Red.as_deref().unwrap(); (and similarly for Green/Blue) panics if any is None; *Red.unwrap() = ... (and similarly for Green/Blue) repeats the all-Some requirement; pub extern "C" fn tritanopia() calls Tritanopia(Some(&mut RGBNorm.R), Some(&mut RGBNorm.G), Some(&mut RGBNorm.B)), indicating the intended invariant is 'always Some'

**Implementation:** Change signature to `fn tritanopia_in_place(rgb: &mut cb_rgb)` or `fn Tritanopia(rgb: &mut cb_rgb)` so the compiler enforces presence of all channels. If optionality is truly required, encode it as a separate API (e.g., `fn Tritanopia_partial(mask: ChannelMask, rgb: &mut cb_rgb)`), avoiding `Option`+unwrap.

---

## Protocol Invariants

### 1. cb_rgb color-space/range protocol (Linear 0..1 <-> sRGB 0..1 <-> Denormalized 0..255)

**Location**: `/data/test_case/lib.rs:1-114`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The functions treat cb_rgb as being in specific implicit color-space/range states. cbRemoveGammaRGB() assumes its cb_rgb input is gamma-encoded (sRGB-ish) in the 0..1 range and produces linear 0..1 output. cbApplyGammaRGB() assumes linear 0..1 input and produces gamma-encoded 0..1 output. However, cb_rgb is just three f32s with no type-level distinction, so nothing prevents calling these functions in the wrong order or with out-of-range values (negative, >1). This matters because cbDenorm() then performs C-like wrapping narrowing to u8, so out-of-range values silently wrap rather than clamp, producing nonsensical colors.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cb_rgb_255, 3 free function(s); struct cb_rgb, 2 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cb_rgb {
    pub R: f32,
    pub G: f32,
    pub B: f32,
}

#[inline]
fn cbRemoveGammaRGB(RGB: cb_rgb) -> cb_rgb {
    #[inline]
    fn remove_gamma(c: f32) -> f32 {
        let c64 = c as f64;
        if c64 > 0.04045f64 {
            (((c64 + 0.055f64) / 1.055f64).powf(2.4f64)) as f32
        } else {
            (c64 / 12.92f64) as f32
        }
    }

    cb_rgb {
        R: remove_gamma(RGB.R),
        G: remove_gamma(RGB.G),
        B: remove_gamma(RGB.B),
    }
}

#[inline]
fn cbNorm(RGB: cb_rgb_255) -> cb_rgb {
    cb_rgb {
        R: RGB.R as f32 / 255.0f32,
        G: RGB.G as f32 / 255.0f32,
        B: RGB.B as f32 / 255.0f32,
    }
}

#[inline]
fn cbDenorm(RGB: cb_rgb) -> cb_rgb_255 {
    // Match C-style float->u8 conversion behavior more closely:
    // - add 0.5 then truncate toward zero
    // - wrap on overflow (as C does for unsigned narrowing conversions)
    #[inline]
    fn to_u8_wrapping(x: f32) -> u8 {
        let t = (x * 255.0f32 + 0.5f32) as i32;
        (t as u32 as u8)
    }

    cb_rgb_255 {
        R: to_u8_wrapping(RGB.R),
        G: to_u8_wrapping(RGB.G),
        B: to_u8_wrapping(RGB.B),
    }
}

#[inline]
fn cbApplyGammaRGB(RGB: cb_rgb) -> cb_rgb {
    #[inline]
    fn apply_gamma(c: f32) -> f32 {
        let c64 = c as f64;
        if c64 > 0.003_130_804_953_560_371_3_f64 {
            (1.055f64 * c64.powf(0.4166666666f64) - 0.055f64) as f32
        } else {
            (c64 * 12.92f64) as f32
        }
    }

    cb_rgb {
        R: apply_gamma(RGB.R),
        G: apply_gamma(RGB.G),
        B: apply_gamma(RGB.B),
    }
}

#[inline]
fn Tritanopia(Red: Option<&mut f32>, Green: Option<&mut f32>, Blue: Option<&mut f32>) {
    let R: f32 = *Red.as_deref().unwrap();
    let G: f32 = *Green.as_deref().unwrap();
    let B: f32 = *Blue.as_deref().unwrap();
    *Red.unwrap() = R + 0.127_398_86_f32 * G - 0.127_398_86_f32 * B;
    *Green.unwrap() = -4.486E-11f32 * R + 0.873_909_3_f32 * G + 0.126_090_7_f32 * B;
    *Blue.unwrap() = 3.1113E-10f32 * R + 0.873_909_3_f32 * G + 0.126_090_7_f32 * B;
}

#[no_mangle]
pub extern "C" fn tritanopia(RGB: cb_rgb_255) -> cb_rgb_255 {
    let mut RGBNorm: cb_rgb = cbRemoveGammaRGB(cbNorm(RGB));
    Tritanopia(
        Some(&mut RGBNorm.R),
        Some(&mut RGBNorm.G),
        Some(&mut RGBNorm.B),
    );
    let Result: cb_rgb_255 = cbDenorm(cbApplyGammaRGB(RGBNorm));
    Result
}
```

**Entity:** cb_rgb

**States:** LinearNormalized(0..1), GammaEncodedNormalized(0..1), Unconstrained(f32)

**Transitions:**
- GammaEncodedNormalized -> LinearNormalized via cbRemoveGammaRGB(RGB)
- LinearNormalized -> GammaEncodedNormalized via cbApplyGammaRGB(RGB)
- LinearNormalized -> Unconstrained (possible) via Tritanopia mutating channels
- GammaEncodedNormalized -> cb_rgb_255 via cbDenorm(RGB) (expects 0..1 but will wrap otherwise)

**Evidence:** fn cbNorm(RGB: cb_rgb_255) -> cb_rgb divides by 255.0f32, implying normalized 0..1 semantics; fn cbRemoveGammaRGB(RGB: cb_rgb) uses sRGB threshold 0.04045 and powf(2.4), implying input is gamma-encoded normalized; fn cbApplyGammaRGB(RGB: cb_rgb) uses linear threshold 0.003_130_804... and powf(0.4166..), implying input is linear normalized; fn cbDenorm(RGB: cb_rgb) comment: "wrap on overflow (as C does for unsigned narrowing conversions)" and implementation to_u8_wrapping(x) shows out-of-range values are not rejected/clamped; pub extern "C" fn tritanopia() composes cbRemoveGammaRGB(cbNorm(RGB)) then cbDenorm(cbApplyGammaRGB(RGBNorm)), implying a required order/protocol

**Implementation:** Introduce distinct wrapper types around cb_rgb, e.g. `struct LinearRgb01(cb_rgb); struct Srgb01(cb_rgb);` and make `cb_remove_gamma(Srgb01)->LinearRgb01`, `cb_apply_gamma(LinearRgb01)->Srgb01`, `cb_denorm(Srgb01)->cb_rgb_255` (or `cb_rgb_255`). Provide fallible constructors like `LinearRgb01::try_from(cb_rgb)` that validate 0..=1 (or clamp explicitly) to prevent accidental wrapping at denorm.

---

