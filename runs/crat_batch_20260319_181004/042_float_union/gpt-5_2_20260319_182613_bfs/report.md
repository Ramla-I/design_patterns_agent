# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## Protocol Invariants

### 1. raw_double_t active-union-variant protocol (write f then read x)

**Location**: `/data/test_case/main.rs:1-125`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: raw_double_t is a C-style union used for bit reinterpretation between f64 and u64. Correctness relies on a protocol: the field you read must correspond to the field most recently written (or you must be intentionally doing type-punning with an explicit representation guarantee). In this code, the union is constructed with { f } and then read via unsafe { u.x }, relying on the implicit invariant that reading x after writing f is valid for the intended bit-level conversion. The type system does not track which variant is active, so misuse (e.g., reading f after writing x elsewhere) would compile and could be UB depending on the rules relied upon.

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

use std::io::{self, Read};

#[repr(C)]
#[derive(Copy, Clone)]
pub union raw_double_t {
    pub x: u64,
    pub f: f64,
}

fn format_hex_float_c99(f: f64) -> String {
    // Match C99 printf("%a") / Rust "{:a}"-like output:
    // - "0x1.<hex>p+E" for normals
    // - "0x0p+0" for zero
    // - preserve sign, including -0.0
    // - lowercase hex digits
    if f.is_nan() {
        // Not exercised by tests; keep reasonable C-like spelling.
        return "nan".to_string();
    }
    if f.is_infinite() {
        return if f.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }

    let bits = f.to_bits();
    let sign_neg = (bits >> 63) != 0;
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    let frac_bits = bits & ((1u64 << 52) - 1);

    if exp_bits == 0 && frac_bits == 0 {
        // Zero (preserve -0.0)
        return if sign_neg {
            "-0x0p+0".to_string()
        } else {
            "0x0p+0".to_string()
        };
    }

    let mut out = String::new();
    if sign_neg {
        out.push('-');
    }

    if exp_bits == 0 {
        // Subnormal: value = (-1)^s * 0.frac * 2^(1-bias)
        // Normalize: find highest set bit in frac, shift to make leading 1 at bit 52.
        let msb = 63 - frac_bits.leading_zeros() as i32; // 0..51
        let shift = 52 - msb; // 1..52
        let mant = frac_bits << shift; // now has leading 1 at bit 52
        let frac = mant & ((1u64 << 52) - 1);
        let exp = (1 - 1023) - shift; // adjust exponent due to normalization shift

        out.push_str("0x1");
        if frac != 0 {
            out.push('.');
            out.push_str(&format!("{:013x}", frac));
            while out.ends_with('0') {
                out.pop();
            }
            if out.ends_with('.') {
                out.pop();
            }
        }
        out.push('p');
        if exp >= 0 {
            out.push('+');
        }
        out.push_str(&exp.to_string());
        return out;
    }

    // Normal
    let exp = exp_bits - 1023;
    out.push_str("0x1");
    if frac_bits != 0 {
        out.push('.');
        out.push_str(&format!("{:013x}", frac_bits));
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
    }
    out.push('p');
    if exp >= 0 {
        out.push('+');
    }
    out.push_str(&exp.to_string());
    out
}

fn driver(f: f64) {
    let u = raw_double_t { f };
    let bits = unsafe { u.x };
    let a = format_hex_float_c99(f);
    println!("{:x} {} {:.4}", bits, a, f);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let f: f64 = input
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    driver(f);
}
```

**Entity:** raw_double_t

**States:** LastWrittenAsF64, LastWrittenAsU64

**Transitions:**
- LastWrittenAsF64 -> LastWrittenAsU64 via unsafe read of u.x after constructing raw_double_t { f } (type-punning use)
- LastWrittenAsU64 -> LastWrittenAsF64 via unsafe read of u.f after constructing/assigning raw_double_t { x } (not shown, but enabled by the union API)

**Evidence:** definition: `pub union raw_double_t { pub x: u64, pub f: f64 }` — union with two overlapping representations and no tracked active field; driver(): `let u = raw_double_t { f };` followed by `let bits = unsafe { u.x };` — unsafe cross-field read implies an active-variant/type-pun protocol

**Implementation:** Replace the union use with safe, explicit conversion wrappers, e.g. `struct F64Bits(u64); impl From<f64> for F64Bits { fn from(f: f64)->Self{Self(f.to_bits())}}` and/or use `f64::to_bits()` / `f64::from_bits()` directly; if a union is required for FFI, wrap it in safe constructors `from_f64`/`from_u64` and only expose matching accessors to enforce the intended direction.

---

