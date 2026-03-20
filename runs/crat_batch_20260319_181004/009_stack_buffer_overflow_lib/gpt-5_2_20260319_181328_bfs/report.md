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

### 1. Array index validity protocol for buffer[0..10)

**Location**: `/data/test_case/lib.rs:1-90`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Indexing into the fixed-size `buffer: [i32; 10]` implicitly requires `data` to be within 0..10. In `bad`, only the lower bound is checked (`data >= 0`) and the upper bound is assumed, then `get_unchecked_mut` is used, making the program rely on a runtime precondition that is not enforced by the type system. In `goodB2G`, the full in-bounds check `(0..10).contains(&data)` is performed before indexing. This is a latent invariant: `data` must be a valid index (0..10) before any write to `buffer[data]` occurs.

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
            let bytes: &[u8] = bytemuck::cast_slice(line);
            let cstr = CStr::from_bytes_until_nul(bytes).unwrap();
            println!("{}", cstr.to_str().unwrap());
        }

        pub(crate) fn printIntLine(intNumber: i32) {
            println!("{intNumber}");
        }

        pub(crate) fn bad(data: i32) {
            let mut buffer: [i32; 10] = [0; 10];

            if data >= 0 {
                // Preserve original behavior: no upper-bound check.
                // In the original C, out-of-bounds is UB; here we keep the same
                // "write then print" semantics by using unchecked indexing.
                unsafe {
                    *buffer.get_unchecked_mut(data as usize) = 1;
                }
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(b"ERROR: Array index is negative.\0"));
            }
        }

        fn goodG2B() {
            let data: i32 = 7;
            let mut buffer: [i32; 10] = [0; 10];

            if data >= 0 {
                buffer[data as usize] = 1;
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(b"ERROR: Array index is negative.\0"));
            }
        }

        fn goodB2G(data: i32) {
            let mut buffer: [i32; 10] = [0; 10];

            if (0..10).contains(&data) {
                buffer[data as usize] = 1;
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(
                    b"ERROR: Array index is out-of-bounds\0",
                ));
            }
        }

        pub(crate) fn good(data: i32) {
            goodG2B();
            goodB2G(data);
        }

        #[no_mangle]
        pub extern "C" fn driver(goodData: i32, badData: i32) {
            printLine(bytemuck::cast_slice(b"Calling good()...\0"));
            good(goodData);
            printLine(bytemuck::cast_slice(b"Finished good()\0"));
            printLine(bytemuck::cast_slice(b"Calling bad()...\0"));
            bad(badData);
            printLine(bytemuck::cast_slice(b"Finished bad()\0"));
        }
    }
}
```

**Entity:** bad(data: i32) / goodB2G(data: i32) array indexing into buffer: [i32; 10]

**States:** UncheckedIndex, CheckedInBounds

**Transitions:**
- UncheckedIndex -> CheckedInBounds via (0..10).contains(&data) guard (goodB2G)

**Evidence:** bad(): `if data >= 0 { ... *buffer.get_unchecked_mut(data as usize) = 1; }` (only lower-bound checked; unchecked indexing used); bad(): comment: "no upper-bound check" and "out-of-bounds is UB; ... using unchecked indexing"; goodB2G(): `if (0..10).contains(&data) { buffer[data as usize] = 1; } else { printLine("ERROR: Array index is out-of-bounds") }`; Error messages: "ERROR: Array index is negative." and "ERROR: Array index is out-of-bounds" encode the intended preconditions

**Implementation:** Introduce a validated index type, e.g. `struct Index10(u8); impl TryFrom<i32> for Index10 { ... }`, guaranteeing 0..10. Then change APIs to accept `Index10` (or `Option/Result<Index10, _>` from parsing) and use `buffer[idx.0 as usize]` without unchecked indexing. Alternatively, take `usize` and validate at the boundary (FFI/driver) so internal functions cannot be called with invalid indices.

---

### 2. C-string validity invariant for printLine input

**Location**: `/data/test_case/lib.rs:1-90`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `printLine` assumes its `&[i8]` argument is a NUL-terminated C string and also valid UTF-8. The function performs conversions that will panic (`unwrap()`) if these invariants do not hold. The type system does not express that callers must supply a NUL-terminated buffer (with a NUL byte present) or that the bytes before NUL are UTF-8; these are latent input validity requirements.

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
            let bytes: &[u8] = bytemuck::cast_slice(line);
            let cstr = CStr::from_bytes_until_nul(bytes).unwrap();
            println!("{}", cstr.to_str().unwrap());
        }

        pub(crate) fn printIntLine(intNumber: i32) {
            println!("{intNumber}");
        }

        pub(crate) fn bad(data: i32) {
            let mut buffer: [i32; 10] = [0; 10];

            if data >= 0 {
                // Preserve original behavior: no upper-bound check.
                // In the original C, out-of-bounds is UB; here we keep the same
                // "write then print" semantics by using unchecked indexing.
                unsafe {
                    *buffer.get_unchecked_mut(data as usize) = 1;
                }
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(b"ERROR: Array index is negative.\0"));
            }
        }

        fn goodG2B() {
            let data: i32 = 7;
            let mut buffer: [i32; 10] = [0; 10];

            if data >= 0 {
                buffer[data as usize] = 1;
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(b"ERROR: Array index is negative.\0"));
            }
        }

        fn goodB2G(data: i32) {
            let mut buffer: [i32; 10] = [0; 10];

            if (0..10).contains(&data) {
                buffer[data as usize] = 1;
                for v in buffer {
                    printIntLine(v);
                }
            } else {
                printLine(bytemuck::cast_slice(
                    b"ERROR: Array index is out-of-bounds\0",
                ));
            }
        }

        pub(crate) fn good(data: i32) {
            goodG2B();
            goodB2G(data);
        }

        #[no_mangle]
        pub extern "C" fn driver(goodData: i32, badData: i32) {
            printLine(bytemuck::cast_slice(b"Calling good()...\0"));
            good(goodData);
            printLine(bytemuck::cast_slice(b"Finished good()\0"));
            printLine(bytemuck::cast_slice(b"Calling bad()...\0"));
            bad(badData);
            printLine(bytemuck::cast_slice(b"Finished bad()\0"));
        }
    }
}
```

**Entity:** printLine(line: &[i8])

**States:** NotNulTerminatedOrInvalidUtf8, ValidNulTerminatedUtf8CString

**Transitions:**
- NotNulTerminatedOrInvalidUtf8 -> ValidNulTerminatedUtf8CString via runtime parsing in CStr::from_bytes_until_nul(...).unwrap() and to_str().unwrap()

**Evidence:** printLine(): `let cstr = CStr::from_bytes_until_nul(bytes).unwrap();` requires a NUL byte in `bytes`; printLine(): `println!("{}", cstr.to_str().unwrap());` requires UTF-8 contents before NUL; printLine(): accepts `line: &[i8]` and uses `bytemuck::cast_slice(line)` to treat it as bytes (caller-controlled data representation)

**Implementation:** Change the signature to accept a type that carries the invariant, e.g. `&CStr` (for NUL-termination) or `&str` (for UTF-8), and perform conversion at the boundary once. If the API must accept raw bytes, return `Result<()>` instead of unwrapping, or introduce `struct NulTerminated<'a>(&'a [u8]);` constructed only via a checked constructor.

---

