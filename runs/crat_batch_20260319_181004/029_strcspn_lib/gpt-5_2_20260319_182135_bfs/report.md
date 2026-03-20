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

### 1. C string pointer/slice validity + NUL-termination precondition

**Location**: `/data/test_case/lib.rs:1-50`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The FFI entrypoint `driver` and the helper `driver_internal` rely on an implicit contract about the inputs: if pointers are non-null they must point to at least 1024 readable `i8` values and contain a NUL terminator within that range so `CStr::from_bytes_until_nul(...).unwrap()` will succeed. These requirements are not enforced by the type system; violations lead to UB (invalid memory) or panics (no NUL). The `NullOrEmpty` state is handled by passing `&[]`, but `driver_internal` still assumes it can successfully parse a CStr from the provided slices (which will panic for empty slices).

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
        use std::ffi::CStr;

        // === driver.rs ===
        pub(crate) fn driver_internal(s1: &[i8], s2: &[i8]) {
            let s1_bytes = unsafe { std::slice::from_raw_parts(s1.as_ptr().cast::<u8>(), s1.len()) };
            let s2_bytes = unsafe { std::slice::from_raw_parts(s2.as_ptr().cast::<u8>(), s2.len()) };

            let cstr1 = CStr::from_bytes_until_nul(s1_bytes).unwrap();
            let cstr2 = CStr::from_bytes_until_nul(s2_bytes).unwrap();

            let bytes1 = cstr1.to_bytes();
            let bytes2 = cstr2.to_bytes();

            let count = bytes1
                .iter()
                .take_while(|b| !bytes2.contains(b))
                .count() as u64;

            println!("{count}");
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(s1: *const i8, s2: *const i8) {
            driver_internal(
                if s1.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(s1, 1024)
                },
                if s2.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(s2, 1024)
                },
            )
        }
    }
}
```

**Entity:** driver (extern "C" API) / driver_internal input contract

**States:** NullOrEmpty, NonNullValidCStrWithin1024, NonNullInvalid (no NUL / invalid memory)

**Transitions:**
- NullOrEmpty -> panic via CStr::from_bytes_until_nul(...).unwrap() in driver_internal
- NonNullValidCStrWithin1024 -> success path via CStr::from_bytes_until_nul(...).unwrap() in driver_internal
- NonNullInvalid (no NUL / invalid memory) -> panic (no NUL) or UB (invalid memory) via from_raw_parts + unwrap

**Evidence:** fn driver(s1: *const i8, s2: *const i8): raw pointers encode optional/unknown-validity inputs; driver: if s1.is_null() { &[] } else { std::slice::from_raw_parts(s1, 1024) } (same for s2) hardcodes a 1024-byte readable-memory precondition for non-null pointers; driver_internal: unsafe { std::slice::from_raw_parts(s1.as_ptr().cast::<u8>(), s1.len()) } reinterprets i8 slice as u8 bytes; driver_internal: CStr::from_bytes_until_nul(s1_bytes).unwrap() and same for s2: requires a NUL within the provided slice; unwrap panics if missing

**Implementation:** Introduce a validated wrapper like `struct CStrBuf<'a>(&'a CStr);` (and/or `struct NulTerminatedBytes<'a>(&'a [u8]);`) with a constructor `fn try_from_1024(ptr: *const i8) -> Result<Option<CStrBuf<'a>>, Error>` that (a) treats null as None, (b) safely bounds to 1024 bytes, and (c) performs `CStr::from_bytes_until_nul` returning a Result instead of panicking. Then make `driver_internal` accept `Option<&CStr>` (or `Option<CStrBuf>`) so the precondition is enforced at the boundary and the empty-slice panic state becomes unrepresentable.

---

