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

### 1. FFI-bytes-view precondition (valid representation/lifetime of input)

**Location**: `/data/test_case/lib.rs:1-39`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `driver` forms a byte slice by taking the address of the parameter `x` and reinterpreting it as `[u8; size_of::<f32>()]`. This relies on an implicit precondition: the pointed-to memory must be valid for reads of `size_of::<f32>()` bytes for the duration of the slice use, and the representation being inspected is the in-memory `f32` representation (including endianness and any platform-specific NaN payload details). The type system does not express these requirements; they are instead pushed into `unsafe` and the FFI boundary.

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
        // Keep the original call site working even when `crate::c_lib` doesn't exist in this crate.
        // If the real project provides `crate::c_lib::Xu32`, this local module can be removed.
        mod c_lib {
            #[inline]
            pub fn Xu32(x: u32) -> u32 {
                x
            }
        }

        #[inline]
        fn print_hex(bytes: &[u8]) {
            for &b in bytes {
                print!("{:02x}", c_lib::Xu32(b as u32));
            }
            println!();
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(x: f32) {
            let bytes = core::slice::from_raw_parts(
                core::ptr::addr_of!(x).cast::<u8>(),
                core::mem::size_of::<f32>(),
            );
            print_hex(bytes);
        }
    }
}
```

**Entity:** driver (unsafe extern "C" fn)

**States:** SafeToViewBytes, UBRisk

**Transitions:**
- UBRisk -> SafeToViewBytes via upholding `unsafe` preconditions when calling driver

**Evidence:** pub unsafe extern "C" fn driver(x: f32) — caller must uphold safety contract (no explicit contract in types); core::ptr::addr_of!(x).cast::<u8>() — reinterprets `&f32` as `*const u8`; core::slice::from_raw_parts(..., core::mem::size_of::<f32>()) — constructs slice from raw pointer (requires validity preconditions); print_hex(bytes) — consumes the raw slice immediately, assuming it is valid

**Implementation:** Expose a safe Rust API that takes a byte-oriented newtype (e.g., `#[repr(transparent)] struct F32Bytes([u8; 4]);`) or returns bytes from a safe conversion (`x.to_ne_bytes()`/`to_le_bytes()`), and keep the `unsafe extern "C" fn` as a thin wrapper that performs the conversion before calling an internal safe function.

---

