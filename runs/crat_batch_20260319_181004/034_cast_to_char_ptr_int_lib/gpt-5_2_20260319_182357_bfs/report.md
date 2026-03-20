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

### 1. FFI pointer validity & lifetime precondition for from_raw_parts

**Location**: `/data/test_case/lib.rs:1-32`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `driver` constructs a byte slice using `core::slice::from_raw_parts` from a raw pointer derived from `x`. This is only valid if the pointer is non-null, properly aligned for `i32`, points to an actual `i32` for the duration of the slice use, and is safe to read as raw bytes. These requirements are implicit and enforced only by `unsafe`, not by the type system; violating them would be UB. In the current code the pointer comes from `&raw const x` (a local), so it is valid for the duration of the function body, but the invariant remains latent at the API boundary because the function is `extern "C"` and `unsafe` without encoding the safety contract in types.

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
        // The original C2Rust code referenced `crate::c_lib::Xu32`, but this crate
        // does not provide `c_lib`. Keep behavior (formatting each byte as 2 hex digits)
        // without relying on missing symbols.
        fn print_hex(bytes: &[u8]) {
            for &b in bytes {
                print!("{:02x}", b);
            }
            println!();
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(x: i32) {
            let bytes = core::slice::from_raw_parts(
                (&raw const x as *const i32).cast::<u8>(),
                core::mem::size_of::<i32>(),
            );
            print_hex(bytes);
        }
    }
}
```

**Entity:** src::lib::driver (unsafe extern "C" fn)

**States:** Valid i32 object addressable as u8 bytes, Invalid/unaddressable pointer or wrong lifetime

**Transitions:**
- Valid i32 object addressable as u8 bytes -> (creates) &[u8] via core::slice::from_raw_parts(...)

**Evidence:** driver is declared `pub unsafe extern "C" fn driver(x: i32)`; uses `core::slice::from_raw_parts(((&raw const x as *const i32).cast::<u8>()), core::mem::size_of::<i32>())`; `from_raw_parts` is an unsafe API requiring a valid pointer/length pair

**Implementation:** Wrap the raw-byte-view precondition in a safe helper/newtype, e.g. `struct I32Bytes([u8; 4]); impl From<i32> for I32Bytes { ... }`, and have `driver` call a safe conversion (`let bytes = x.to_ne_bytes(); print_hex(&bytes);`) eliminating `from_raw_parts` and its pointer validity requirements from the API surface.

---

