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

### 1. C-string slice preconditions (NUL-terminated, UTF-8, and in-bounds start/stop)

**Location**: `/data/test_case/lib.rs:1-71`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: slice_internal assumes multiple input validity conditions that are only enforced by panics / runtime checks: (1) `mystr` must contain a NUL terminator within the provided buffer so it can be interpreted as a C string; (2) the slice from `start` must also contain a NUL terminator so `tail_cstr` can be formed; (3) the bytes after `start` must be valid UTF-8 (`to_str().unwrap()`), otherwise it panics; (4) `start` and `stop` must satisfy 0 <= start < stop <= len, otherwise it prints an error and returns 1. None of these requirements are expressed in the type system: `&[i8]` does not guarantee NUL-termination or UTF-8, and `i32` indices allow negative/out-of-range values.

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

        // === slicing.rs ===
        pub(crate) fn slice_internal(mystr: &[i8], start_ptr: Option<&i32>, stop_ptr: Option<&i32>) -> i32 {
            // Interpret `mystr` as a C string (NUL-terminated) within the provided buffer.
            let bytes: &[u8] = bytemuck::cast_slice(mystr);
            let cstr = match CStr::from_bytes_until_nul(bytes) {
                Ok(s) => s,
                Err(_) => {
                    // Preserve behavior: original code would unwrap and panic; return error instead is a logic change.
                    // So keep the unwrap-like behavior by printing nothing and returning nonzero? No: preserve as close as possible.
                    // We'll mimic unwrap by panicking.
                    panic!("input is not NUL-terminated within provided buffer");
                }
            };
            let len = cstr.count_bytes();

            let start = start_ptr.copied().unwrap_or(0);
            if start < 0 || (start as usize) > len {
                println!("Error: start is off the end of the string!");
                return 1;
            }

            let stop = stop_ptr.copied().unwrap_or(len as i32);
            if stop < 0 || (stop as usize) > len {
                println!("Error: stop is off the end of the string!");
                return 1;
            }
            if stop <= start {
                println!("Error: stop must come after start!");
                return 1;
            }

            let start_usize = start as usize;
            let width = (stop - start) as usize;

            let tail_bytes: &[u8] = bytemuck::cast_slice(&mystr[start_usize..]);
            let tail_cstr = CStr::from_bytes_until_nul(tail_bytes).unwrap();
            let tail_str = tail_cstr.to_str().unwrap();

            println!("{1:.0$}", width, tail_str);
            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn slice(
            mystr: *const i8,
            start_ptr: Option<&i32>,
            stop_ptr: Option<&i32>,
        ) -> i32 {
            let buf: &[i8] = if mystr.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(mystr, 1024)
            };
            slice_internal(buf, start_ptr, stop_ptr)
        }
    }
}
```

**Entity:** slice_internal(mystr: &[i8], start_ptr: Option<&i32>, stop_ptr: Option<&i32>)

**States:** ValidInputs, InvalidInputs

**Transitions:**
- InvalidInputs -> ValidInputs via providing a NUL-terminated buffer, UTF-8 data, and valid start/stop bounds

**Evidence:** slice_internal: CStr::from_bytes_until_nul(bytes) with panic!("input is not NUL-terminated within provided buffer"); slice_internal: start_ptr.copied().unwrap_or(0) followed by `if start < 0 || (start as usize) > len { ... return 1; }`; slice_internal: stop_ptr.copied().unwrap_or(len as i32) followed by `if stop < 0 || (stop as usize) > len { ... return 1; }`; slice_internal: `if stop <= start { println!("Error: stop must come after start!"); return 1; }`; slice_internal: `let tail_cstr = CStr::from_bytes_until_nul(tail_bytes).unwrap();` (requires a NUL after start); slice_internal: `let tail_str = tail_cstr.to_str().unwrap();` (requires UTF-8)

**Implementation:** Introduce validated wrapper types, e.g. `struct NulTerminated<'a>(&'a CStr)` (constructed via `CStr::from_bytes_until_nul` once) and `struct SliceRange { start: usize, stop: usize }` (constructed via `try_from((start_i32, stop_i32, len))`). Change `slice_internal` to accept `cstr: &CStr` (or `&str` if UTF-8 is required) plus a `SliceRange`, eliminating negative/out-of-bounds cases and moving validation to constructors.

---

## Protocol Invariants

### 2. FFI pointer validity protocol for `slice` (null vs readable buffer of at least 1024)

**Location**: `/data/test_case/lib.rs:1-71`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The FFI entrypoint implicitly requires that when `mystr` is non-null it points to a readable allocation of at least 1024 bytes (because it unconditionally forms `from_raw_parts(mystr, 1024)`). This is a safety precondition not represented in the signature: `*const i8` can be dangling/short/unreadable. The function also has a distinct 'null pointer means empty buffer' behavior that is encoded with a runtime check rather than a type-level distinction.

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

        // === slicing.rs ===
        pub(crate) fn slice_internal(mystr: &[i8], start_ptr: Option<&i32>, stop_ptr: Option<&i32>) -> i32 {
            // Interpret `mystr` as a C string (NUL-terminated) within the provided buffer.
            let bytes: &[u8] = bytemuck::cast_slice(mystr);
            let cstr = match CStr::from_bytes_until_nul(bytes) {
                Ok(s) => s,
                Err(_) => {
                    // Preserve behavior: original code would unwrap and panic; return error instead is a logic change.
                    // So keep the unwrap-like behavior by printing nothing and returning nonzero? No: preserve as close as possible.
                    // We'll mimic unwrap by panicking.
                    panic!("input is not NUL-terminated within provided buffer");
                }
            };
            let len = cstr.count_bytes();

            let start = start_ptr.copied().unwrap_or(0);
            if start < 0 || (start as usize) > len {
                println!("Error: start is off the end of the string!");
                return 1;
            }

            let stop = stop_ptr.copied().unwrap_or(len as i32);
            if stop < 0 || (stop as usize) > len {
                println!("Error: stop is off the end of the string!");
                return 1;
            }
            if stop <= start {
                println!("Error: stop must come after start!");
                return 1;
            }

            let start_usize = start as usize;
            let width = (stop - start) as usize;

            let tail_bytes: &[u8] = bytemuck::cast_slice(&mystr[start_usize..]);
            let tail_cstr = CStr::from_bytes_until_nul(tail_bytes).unwrap();
            let tail_str = tail_cstr.to_str().unwrap();

            println!("{1:.0$}", width, tail_str);
            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn slice(
            mystr: *const i8,
            start_ptr: Option<&i32>,
            stop_ptr: Option<&i32>,
        ) -> i32 {
            let buf: &[i8] = if mystr.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(mystr, 1024)
            };
            slice_internal(buf, start_ptr, stop_ptr)
        }
    }
}
```

**Entity:** unsafe extern "C" fn slice(mystr: *const i8, start_ptr: Option<&i32>, stop_ptr: Option<&i32>)

**States:** NullPointer, NonNullValidPointer, NonNullInvalidPointer

**Transitions:**
- NullPointer -> NonNullValidPointer via passing a valid pointer to >=1024 readable bytes

**Evidence:** slice: `pub unsafe extern "C" fn slice(mystr: *const i8, ...)` (unsafe FFI boundary implies caller-upheld invariants); slice: `if mystr.is_null() { &[] } else { std::slice::from_raw_parts(mystr, 1024) }` (non-null path assumes 1024 readable bytes)

**Implementation:** Expose a safe Rust wrapper that requires a proof/capability of validity, e.g. `struct Buffer1024(*const i8); unsafe fn new(ptr: *const i8) -> Self` (documenting the >=1024 readable-bytes precondition) and make the safe API take `Buffer1024` instead of `*const i8`. Keep the raw `extern "C"` as a thin adapter that immediately constructs the capability (remaining `unsafe`) and forwards to a fully safe internal function.

---

