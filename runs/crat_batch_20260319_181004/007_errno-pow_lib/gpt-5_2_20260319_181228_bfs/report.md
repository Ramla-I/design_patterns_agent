# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. my_pow domain/range precondition (RealFinite inputs -> ValidReal output vs ErrorSentinel)

**Location**: `/data/test_case/lib.rs:1-37`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: my_pow relies on runtime checks of the computed result to distinguish a 'valid real-number result' from domain/range errors. On error it prints to stderr and returns the sentinel value -1.0. This creates an implicit protocol: callers must avoid inputs that lead to NaN/Infinity (or must treat -1.0 as an error indicator and/or consult stderr). The type system does not enforce that inputs are within a valid domain, nor that the output is not a sentinel collision (a legitimate pow result can also be -1.0).

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
        // === pow.rs ===
        #[no_mangle]
        pub extern "C" fn my_pow(base: f64, exponent: f64) -> f64 {
            let result = base.powf(exponent);

            if result.is_nan() {
                eprintln!(
                    "Domain error: pow({0:.2}, {1:.2}) is undefined in the real number domain.",
                    base, exponent
                );
                return -1.0;
            }

            if result.is_infinite() {
                eprintln!(
                    "Range error: pow({0:.2}, {1:.2}) caused overflow or underflow.",
                    base, exponent
                );
                return -1.0;
            }

            result
        }
    }
}
```

**Entity:** my_pow (extern "C" fn)

**States:** ValidReal, DomainOrRangeError

**Transitions:**
- ValidReal -> DomainOrRangeError via result.is_nan() / result.is_infinite() branches (return -1.0)
- ValidReal -> ValidReal via normal return result

**Evidence:** fn my_pow(base: f64, exponent: f64) -> f64: plain f64 inputs/outputs carry no validity information; let result = base.powf(exponent);: uses powf which can yield NaN/Infinity for some inputs; if result.is_nan() { eprintln!("Domain error: pow({0:.2}, {1:.2}) is undefined in the real number domain.", ...); return -1.0; }; if result.is_infinite() { eprintln!("Range error: pow({0:.2}, {1:.2}) caused overflow or underflow.", ...); return -1.0; }; return -1.0;: sentinel error value not distinguished from legitimate -1.0 results

**Implementation:** Introduce input newtypes like `FiniteF64` / `NonNaNF64` (validated on construction) and return a tagged result instead of a sentinel, e.g. `Result<FiniteF64, PowError>` (or a `#[repr(C)]` error code + out-parameter for the C ABI). This makes the 'error vs valid' state explicit and prevents NaN/Inf at the boundary.

---

