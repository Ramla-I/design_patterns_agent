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

### 1. C-string slice validity precondition (NUL-terminated UTF-8)

**Location**: `/data/test_case/lib.rs:1-76`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: printLine assumes its input slice of i8 is a C string payload: it must contain a NUL terminator and the bytes before the NUL must be valid UTF-8. These requirements are not expressed in the type of `line` (`&[i8]`), so callers can pass arbitrary byte slices and cause panics at runtime via unwraps.

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
        pub const __SCHAR_MAX__: i32 = 127;
        pub const CHAR_MAX: i32 = __SCHAR_MAX__;

        pub(crate) fn printLine(line: &[i8]) {
            if line.is_empty() {
                return;
            }
            let bytes: &[u8] = bytemuck::cast_slice(line);
            let cstr = std::ffi::CStr::from_bytes_until_nul(bytes).unwrap();
            println!("{}", cstr.to_str().unwrap());
        }

        pub(crate) fn printHexCharLine(charHex: i8) {
            // Avoid dependency on a non-existent `crate::c_lib::Xu32`.
            // The original intent is to print the value as an unsigned byte in hex.
            let v: u8 = charHex as u8;
            println!("{0:>02x}", v);
        }

        pub(crate) fn bad() {
            let data: i8 = CHAR_MAX as i8;
            if data as i32 > 0 {
                let result: i8 = (data as i32 * 2) as i8;
                printHexCharLine(result);
            }
        }

        fn goodG2B() {
            let data: i8 = 2;
            if data as i32 > 0 {
                let result: i8 = (data as i32 * 2) as i8;
                printHexCharLine(result);
            }
        }

        fn goodB2G() {
            let data: i8 = CHAR_MAX as i8;
            if data as i32 > 0 {
                if (data as i32) < CHAR_MAX / 2 {
                    let result: i8 = (data as i32 * 2) as i8;
                    printHexCharLine(result);
                } else {
                    printLine(bytemuck::cast_slice(
                        b"data value is too large to perform arithmetic safely.\0",
                    ));
                }
            }
        }

        pub(crate) fn good() {
            goodG2B();
            goodB2G();
        }

        #[no_mangle]
        pub extern "C" fn driver(useGood: i32) {
            if useGood != 0 {
                good();
            } else {
                bad();
            };
        }
    }
}
```

**Entity:** printLine(line: &[i8])

**States:** ValidCStrBytes, Invalid/NonCStrBytes

**Transitions:**
- Invalid/NonCStrBytes -> panic via CStr::from_bytes_until_nul(...).unwrap() or to_str().unwrap()

**Evidence:** fn printLine(line: &[i8]) takes a raw slice with no encoding/termination guarantee; let cstr = std::ffi::CStr::from_bytes_until_nul(bytes).unwrap(); (panics if no NUL); println!("{}", cstr.to_str().unwrap()); (panics if not UTF-8); bytemuck::cast_slice(line) reinterprets &[i8] as &[u8] without validating contents

**Implementation:** Change API to accept `&CStr` (or `&std::ffi::CString`) to enforce NUL-termination, and/or accept `&str` if UTF-8 is required. If the input truly originates as `&[i8]`, introduce a validated wrapper like `struct NulTerminatedI8Slice<'a>(&'a [i8]);` with a `TryFrom<&'a [i8]> for ...` that checks for a NUL and (optionally) UTF-8, then make `printLine` take that wrapper.

---

## Protocol Invariants

### 2. Safe-doubling protocol for signed byte (unchecked vs checked multiply)

**Location**: `/data/test_case/lib.rs:1-76`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on a protocol: before doubling a positive `i8` value, it must be proven that `data * 2` fits in the `i8` range. `bad()` and `goodG2B()` perform an unchecked multiply by widening to i32 and then truncating back to i8, which can overflow/wrap; `goodB2G()` performs a runtime bounds check `(data as i32) < CHAR_MAX/2` before multiplying. This safety requirement is not encoded in types; it is enforced only by control flow and constants.

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
        pub const __SCHAR_MAX__: i32 = 127;
        pub const CHAR_MAX: i32 = __SCHAR_MAX__;

        pub(crate) fn printLine(line: &[i8]) {
            if line.is_empty() {
                return;
            }
            let bytes: &[u8] = bytemuck::cast_slice(line);
            let cstr = std::ffi::CStr::from_bytes_until_nul(bytes).unwrap();
            println!("{}", cstr.to_str().unwrap());
        }

        pub(crate) fn printHexCharLine(charHex: i8) {
            // Avoid dependency on a non-existent `crate::c_lib::Xu32`.
            // The original intent is to print the value as an unsigned byte in hex.
            let v: u8 = charHex as u8;
            println!("{0:>02x}", v);
        }

        pub(crate) fn bad() {
            let data: i8 = CHAR_MAX as i8;
            if data as i32 > 0 {
                let result: i8 = (data as i32 * 2) as i8;
                printHexCharLine(result);
            }
        }

        fn goodG2B() {
            let data: i8 = 2;
            if data as i32 > 0 {
                let result: i8 = (data as i32 * 2) as i8;
                printHexCharLine(result);
            }
        }

        fn goodB2G() {
            let data: i8 = CHAR_MAX as i8;
            if data as i32 > 0 {
                if (data as i32) < CHAR_MAX / 2 {
                    let result: i8 = (data as i32 * 2) as i8;
                    printHexCharLine(result);
                } else {
                    printLine(bytemuck::cast_slice(
                        b"data value is too large to perform arithmetic safely.\0",
                    ));
                }
            }
        }

        pub(crate) fn good() {
            goodG2B();
            goodB2G();
        }

        #[no_mangle]
        pub extern "C" fn driver(useGood: i32) {
            if useGood != 0 {
                good();
            } else {
                bad();
            };
        }
    }
}
```

**Entity:** bad()/goodG2B()/goodB2G() arithmetic on i8

**States:** UncheckedMultiply, CheckedMultiply

**Transitions:**
- UncheckedMultiply -> potential overflow/wrap via `(data as i32 * 2) as i8` in bad()
- CheckedMultiply -> safe multiply via guard `(data as i32) < CHAR_MAX / 2` then `(data as i32 * 2) as i8` in goodB2G()

**Evidence:** bad(): `let data: i8 = CHAR_MAX as i8;` then `let result: i8 = (data as i32 * 2) as i8;` (no bounds check); goodG2B(): `let data: i8 = 2;` then `let result: i8 = (data as i32 * 2) as i8;` (relies on chosen constant being safe); goodB2G(): runtime precondition `if (data as i32) < CHAR_MAX / 2 { ... } else { printLine("data value is too large...") }`

**Implementation:** Introduce a wrapper representing values proven safe to double, e.g. `struct DoubleSafeI8(i8);` with `TryFrom<i8>` that enforces `0 < x && x <= i8::MAX/2` (or a more general range). Provide `fn double(self) -> i8` or `-> DoubleSafeI8` without runtime checks. Alternatively use `i8::checked_mul(2)` to force callers to handle `None`, and wrap the `Some` case into a newtype to make the checked state explicit.

---

