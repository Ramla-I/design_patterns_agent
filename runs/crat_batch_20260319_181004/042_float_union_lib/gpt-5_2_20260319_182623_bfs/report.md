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

### 1. raw_double_t union active-field / initialization invariant (f64 bits view)

**Location**: `/data/test_case/lib.rs:1-31`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The union `raw_double_t` is used for type-punning between `f64` and `u64`. Correct use relies on an implicit invariant: a caller must only read the union field corresponding to the most recently written field (the 'active' variant), or otherwise accept that reading another field is a raw bit reinterpretation. This is not tracked by the type system; instead it is enforced by `unsafe` usage and convention. In `driver`, the code writes `f` then reads `x`, relying on the implicit protocol that this reinterpretation is intended and that the union has been initialized.

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
        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub union raw_double_t {
            pub x: u64,
            pub f: f64,
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(f: f64) {
            let u = raw_double_t { f };
            let bits = unsafe { u.x };

            // Avoid referencing a non-existent `crate::c_lib` module.
            // Preserve output shape: hex bits, float, float with 4 decimals.
            println!("{0:x} {1} {2:.4}", bits, f, f);
        }
    }
}
```

**Entity:** raw_double_t

**States:** Initialized with f (f set), Initialized with x (x set)

**Transitions:**
- (uninitialized) -> Initialized with f via `raw_double_t { f }`
- (uninitialized) -> Initialized with x via `raw_double_t { x }` (not shown, but implied by union field)

**Evidence:** definition: `pub union raw_double_t { pub x: u64, pub f: f64 }` encodes multiple possible active fields; driver(): `let u = raw_double_t { f };` initializes the union through field `f`; driver(): `let bits = unsafe { u.x };` reads a different union field (`x`) under `unsafe`, indicating reliance on an implicit invariant/protocol

**Implementation:** Replace the `union` with a transparent newtype wrapper around the canonical representation and provide explicit conversions, e.g. `#[repr(transparent)] struct F64Bits(u64); impl From<f64> for F64Bits { ... } impl From<F64Bits> for f64 { ... }`, using `f64::to_bits` / `f64::from_bits` to make the reinterpretation explicit and safe.

---

