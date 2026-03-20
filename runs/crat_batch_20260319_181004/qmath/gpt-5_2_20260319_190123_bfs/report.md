# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Precondition Invariants

### 2. Vector normalization precondition (non-zero, finite length before inverse sqrt)

**Location**: `/data/test_case/main.rs:1-54`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: VectorNormalizeFast_internal computes an inverse square root of the squared length. This implicitly assumes the length-squared is positive and finite; otherwise results can become NaN/Inf or nonsensical (e.g., normalizing the zero vector). This precondition is not represented in types: `vec3_t` is just `[f32; 3]` and `Q_rsqrt` accepts any `f32` without guarding against 0.0, negative, NaN, or infinity.

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
#![feature(as_array_of_cells)]

use std::process;

pub type vec_t = f32;
pub type vec3_t = [vec_t; 3];

#[inline]
fn Q_rsqrt(number: f32) -> f32 {
    // Quake III fast inverse square root (one Newton iteration)
    let x2 = number * 0.5f32;
    let mut y = number;
    let mut i = y.to_bits();
    i = 0x5f3759dfu32.wrapping_sub(i >> 1);
    y = f32::from_bits(i);
    let threehalfs = 1.5f32;
    y * (threehalfs - x2 * y * y)
}

#[inline]
fn VectorNormalizeFast_internal(v: &mut vec3_t) {
    let ilength = Q_rsqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    v[0] *= ilength;
    v[1] *= ilength;
    v[2] *= ilength;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprint!("{} requires 4 inputs\n", args.get(0).map(String::as_str).unwrap_or(""));
        process::exit(1);
    }

    let mut inputs: vec3_t = [
        args[1].parse::<f32>().unwrap_or(0.0),
        args[2].parse::<f32>().unwrap_or(0.0),
        args[3].parse::<f32>().unwrap_or(0.0),
    ];

    VectorNormalizeFast_internal(&mut inputs);

    print!("{:.6} {:.6} {:.6}\n", inputs[0] as f64, inputs[1] as f64, inputs[2] as f64);
}
```

**Entity:** fn VectorNormalizeFast_internal(v: &mut vec3_t) / Q_rsqrt(number: f32)

**States:** PossiblyZeroOrNonFinite, SafeToNormalize(nonzero & finite length)

**Transitions:**
- PossiblyZeroOrNonFinite -> SafeToNormalize(nonzero & finite length) via an explicit validation step (not present in code)

**Evidence:** VectorNormalizeFast_internal(): `let ilength = Q_rsqrt(v[0]*v[0] + v[1]*v[1] + v[2]*v[2]);` passes computed length^2 directly to Q_rsqrt; Q_rsqrt(number: f32): operates on raw bits (`to_bits`, magic constant, `from_bits`) with no domain checks; VectorNormalizeFast_internal(): `v[i] *= ilength;` blindly applies the scaling factor even if ilength is NaN/Inf

**Implementation:** Use a validated wrapper such as `struct NonZeroFiniteVec3([f32; 3]);` with `TryFrom<vec3_t>` that checks `len_sq.is_finite()` and `len_sq > 0.0`. Provide `fn normalize(self) -> UnitVec3` (another newtype) so only validated vectors can be normalized and the output type encodes 'unit length'.

---

## Protocol Invariants

### 1. CLI input protocol (3 numeric components required before normalization)

**Location**: `/data/test_case/main.rs:1-54`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program assumes a protocol: exactly three numeric components must be provided on the command line before building a vec3_t and normalizing it. This is enforced with a runtime length check and by parsing strings at runtime. Additionally, parse failures are silently converted to 0.0 via unwrap_or(0.0), meaning 'invalid numeric input' becomes indistinguishable from a legitimate 0.0 and the type system does not express 'validated numeric input'.

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
#![feature(as_array_of_cells)]

use std::process;

pub type vec_t = f32;
pub type vec3_t = [vec_t; 3];

#[inline]
fn Q_rsqrt(number: f32) -> f32 {
    // Quake III fast inverse square root (one Newton iteration)
    let x2 = number * 0.5f32;
    let mut y = number;
    let mut i = y.to_bits();
    i = 0x5f3759dfu32.wrapping_sub(i >> 1);
    y = f32::from_bits(i);
    let threehalfs = 1.5f32;
    y * (threehalfs - x2 * y * y)
}

#[inline]
fn VectorNormalizeFast_internal(v: &mut vec3_t) {
    let ilength = Q_rsqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    v[0] *= ilength;
    v[1] *= ilength;
    v[2] *= ilength;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprint!("{} requires 4 inputs\n", args.get(0).map(String::as_str).unwrap_or(""));
        process::exit(1);
    }

    let mut inputs: vec3_t = [
        args[1].parse::<f32>().unwrap_or(0.0),
        args[2].parse::<f32>().unwrap_or(0.0),
        args[3].parse::<f32>().unwrap_or(0.0),
    ];

    VectorNormalizeFast_internal(&mut inputs);

    print!("{:.6} {:.6} {:.6}\n", inputs[0] as f64, inputs[1] as f64, inputs[2] as f64);
}
```

**Entity:** fn main (CLI argument vector: args/inputs)

**States:** ArgsUnchecked, ArgsValidated(len==4), VecParsed(3 floats)

**Transitions:**
- ArgsUnchecked -> ArgsValidated(len==4) via `if args.len() != 4 { ... exit(1) }`
- ArgsValidated(len==4) -> VecParsed(3 floats) via `parse::<f32>().unwrap_or(0.0)`

**Evidence:** main(): `let args: Vec<String> = std::env::args().collect();` builds an unchecked, variable-length argument list; main(): `if args.len() != 4 { ... process::exit(1); }` runtime check enforces required arity; main(): `args[1].parse::<f32>().unwrap_or(0.0)` (and similarly for args[2], args[3]) encodes a 'must be numeric' precondition but falls back silently

**Implementation:** Introduce a `struct Args3([f32; 3]);` with `impl TryFrom<std::env::Args> for Args3` (or `TryFrom<&[String]>`) that performs the arity + numeric parsing once and returns `Result<Args3, ParseError>`. `main()` then operates only on `Args3`, making 'validated 3-vector input' explicit and removing the silent 0.0 fallback.

---

