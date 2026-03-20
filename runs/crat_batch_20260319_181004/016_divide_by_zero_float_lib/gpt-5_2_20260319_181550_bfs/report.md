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

### 2. Non-zero divisor requirement for division (unchecked vs checked path)

**Location**: `/data/test_case/lib.rs:1-87`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on an implicit precondition that `data` must be non-zero (or at least not near zero) before computing `100.0 / data`. `bad(data)` performs the division unconditionally, so its correctness depends on callers never passing 0.0/near-0.0 (a latent invariant). `goodB2G(data)` enforces the invariant at runtime by checking an epsilon and taking an alternate path if violated. The type system does not distinguish 'validated non-zero float' from arbitrary float, so misuse is only caught (sometimes) by runtime checks or manifests as problematic results.

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

        pub(crate) fn printLine(line: &[i8]) {
            if line.is_empty() {
                return;
            }

            // The original C2Rust used `from_bytes_until_nul(...).unwrap()`.
            // In the harness, some callers may pass slices that *don't* include the NUL.
            // To preserve behavior without panicking, accept both NUL-terminated and
            // non-NUL-terminated slices.
            let bytes: &[u8] =
                unsafe { std::slice::from_raw_parts(line.as_ptr() as *const u8, line.len()) };

            let s = match CStr::from_bytes_until_nul(bytes) {
                Ok(cstr) => cstr.to_string_lossy(),
                Err(_) => String::from_utf8_lossy(bytes),
            };
            println!("{s}");
        }

        pub(crate) fn printIntLine(intNumber: i32) {
            println!("{intNumber}");
        }

        pub(crate) fn bad(data: f32) {
            let result: i32 = (100.0f64 / data as f64) as i32;
            printIntLine(result);
        }

        fn goodG2B() {
            let data: f32 = 2.0f32;
            let result: i32 = (100.0f64 / data as f64) as i32;
            printIntLine(result);
        }

        fn goodB2G(data: f32) {
            // Match original: compare in f64 space against a f64 epsilon.
            if (data as f64).abs() > 0.000001f64 {
                let result: i32 = (100.0f64 / data as f64) as i32;
                printIntLine(result);
            } else {
                printLine(unsafe {
                    std::slice::from_raw_parts(
                        b"This would result in a divide by zero\0".as_ptr() as *const i8,
                        b"This would result in a divide by zero\0".len(),
                    )
                });
            }
        }

        pub(crate) fn good(data: f32) {
            goodG2B();
            goodB2G(data);
        }

        #[no_mangle]
        pub extern "C" fn driver(goodData: f32, badData: f32) {
            printLine(unsafe {
                std::slice::from_raw_parts(b"Calling good()...\0".as_ptr() as *const i8, 18)
            });
            good(goodData);
            printLine(unsafe {
                std::slice::from_raw_parts(b"Finished good()\0".as_ptr() as *const i8, 16)
            });
            printLine(unsafe {
                std::slice::from_raw_parts(b"Calling bad()...\0".as_ptr() as *const i8, 17)
            });
            bad(badData);
            printLine(unsafe {
                std::slice::from_raw_parts(b"Finished bad()\0".as_ptr() as *const i8, 15)
            });
        }
    }
}
```

**Entity:** src::lib::bad / src::lib::goodB2G (and their `data: f32` parameter)

**States:** UncheckedDivisor (may be zero/near-zero), CheckedNonZeroDivisor, RejectedAsZero

**Transitions:**
- UncheckedDivisor -> CheckedNonZeroDivisor via `goodB2G` epsilon check `(data as f64).abs() > 0.000001f64`
- UncheckedDivisor -> RejectedAsZero via the `else` branch printing "This would result in a divide by zero"
- UncheckedDivisor -> (uses division anyway) via `bad(data)`

**Evidence:** bad: `let result: i32 = (100.0f64 / data as f64) as i32;` (no guard); goodB2G: `if (data as f64).abs() > 0.000001f64 { ... 100.0f64 / data as f64 ... } else { printLine(... "This would result in a divide by zero\0" ...) }`; error message string literal: "This would result in a divide by zero" indicates the intended precondition

**Implementation:** Define `struct NonZeroF64(f64);` (or `NonZeroF32`) with `TryFrom<f32>` that performs the epsilon/non-zero validation. Change division sites to accept `NonZeroF64` (or `Result<NonZeroF64, ...>`), so `bad` becomes impossible without explicit validation, and `goodB2G` becomes the constructor/validator.

---

### 1. C-string slice validity protocol (NUL-terminated or raw-bytes fallback)

**Location**: `/data/test_case/lib.rs:1-87`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: printLine implicitly expects its input to be a valid memory region and, ideally, a NUL-terminated C string. It uses an unsafe cast from &[i8] to &[u8] and then attempts CStr parsing; if parsing fails (likely due to missing NUL), it falls back to interpreting the full slice as UTF-8 (lossy). The type system does not enforce that the slice is backed by valid memory (FFI-style requirement), nor does it encode whether the slice is NUL-terminated vs raw bytes; callers can pass any &[i8], including one constructed from raw parts, and the function relies on runtime parsing/fallback and on the unsafe caller-side construction being correct.

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

        pub(crate) fn printLine(line: &[i8]) {
            if line.is_empty() {
                return;
            }

            // The original C2Rust used `from_bytes_until_nul(...).unwrap()`.
            // In the harness, some callers may pass slices that *don't* include the NUL.
            // To preserve behavior without panicking, accept both NUL-terminated and
            // non-NUL-terminated slices.
            let bytes: &[u8] =
                unsafe { std::slice::from_raw_parts(line.as_ptr() as *const u8, line.len()) };

            let s = match CStr::from_bytes_until_nul(bytes) {
                Ok(cstr) => cstr.to_string_lossy(),
                Err(_) => String::from_utf8_lossy(bytes),
            };
            println!("{s}");
        }

        pub(crate) fn printIntLine(intNumber: i32) {
            println!("{intNumber}");
        }

        pub(crate) fn bad(data: f32) {
            let result: i32 = (100.0f64 / data as f64) as i32;
            printIntLine(result);
        }

        fn goodG2B() {
            let data: f32 = 2.0f32;
            let result: i32 = (100.0f64 / data as f64) as i32;
            printIntLine(result);
        }

        fn goodB2G(data: f32) {
            // Match original: compare in f64 space against a f64 epsilon.
            if (data as f64).abs() > 0.000001f64 {
                let result: i32 = (100.0f64 / data as f64) as i32;
                printIntLine(result);
            } else {
                printLine(unsafe {
                    std::slice::from_raw_parts(
                        b"This would result in a divide by zero\0".as_ptr() as *const i8,
                        b"This would result in a divide by zero\0".len(),
                    )
                });
            }
        }

        pub(crate) fn good(data: f32) {
            goodG2B();
            goodB2G(data);
        }

        #[no_mangle]
        pub extern "C" fn driver(goodData: f32, badData: f32) {
            printLine(unsafe {
                std::slice::from_raw_parts(b"Calling good()...\0".as_ptr() as *const i8, 18)
            });
            good(goodData);
            printLine(unsafe {
                std::slice::from_raw_parts(b"Finished good()\0".as_ptr() as *const i8, 16)
            });
            printLine(unsafe {
                std::slice::from_raw_parts(b"Calling bad()...\0".as_ptr() as *const i8, 17)
            });
            bad(badData);
            printLine(unsafe {
                std::slice::from_raw_parts(b"Finished bad()\0".as_ptr() as *const i8, 15)
            });
        }
    }
}
```

**Entity:** src::lib::printLine (and its `line: &[i8]` parameter)

**States:** ValidNulTerminatedCStrBytes, NonNulTerminatedRawBytes, InvalidPointerOrLength (UB)

**Transitions:**
- NonNulTerminatedRawBytes -> ValidNulTerminatedCStrBytes by ensuring a trailing NUL before calling
- Any -> InvalidPointerOrLength (UB) if constructed from_raw_parts with incorrect ptr/len

**Evidence:** printLine signature: `fn printLine(line: &[i8])` accepts arbitrary i8 slices rather than `&CStr`/`&[u8]`; comment: "some callers may pass slices that *don't* include the NUL" and "accept both NUL-terminated and non-NUL-terminated slices"; unsafe cast: `std::slice::from_raw_parts(line.as_ptr() as *const u8, line.len())`; runtime branch on protocol: `match CStr::from_bytes_until_nul(bytes) { Ok(..) => .., Err(_) => .. }`

**Implementation:** Introduce distinct wrapper types to make the protocol explicit: e.g., `struct NulTerminated<'a>(&'a CStr); struct RawBytes<'a>(&'a [u8]);` and provide `print_cstr(NulTerminated)` and `print_bytes(RawBytes)`. For FFI-facing callers, accept `*const c_char` and require `unsafe fn` or `Option<&CStr>` to make pointer validity/NUL-termination expectations explicit.

---

