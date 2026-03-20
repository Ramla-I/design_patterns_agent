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

### 1. C string validity invariant for printLine (NUL-terminated, valid pointer/lifetime)

**Location**: `/data/test_case/lib.rs:1-43`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: printLine assumes the incoming &[i8] represents a valid NUL-terminated C string (i.e., contains a terminating 0 byte and is safe to read until that terminator). This is not enforced by the type system because &[i8] does not guarantee NUL-termination or that reading past the slice length won't occur. The function converts the slice pointer to a CStr using unsafe CStr::from_ptr, which will read memory until it finds NUL; if the slice is not properly terminated within bounds, this can cause out-of-bounds reads/UB. The empty-slice check only avoids calling from_ptr on an empty slice; it does not establish C string validity.

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

            // SAFETY: `line` is expected to be a NUL-terminated C string buffer.
            let cstr = unsafe { CStr::from_ptr(line.as_ptr() as *const i8) };
            println!("{}", cstr.to_string_lossy());
        }

        #[no_mangle]
        pub extern "C" fn driver(data: i32) {
            let mut source = [0i8; 100];
            let mut dest = [0i8; 100];

            source[..99].fill(b'A' as i8);
            source[99] = 0;

            if data < 100 {
                // Preserve original behavior: negative `data` leads to huge usize and would panic.
                let n = data as usize;
                dest[..n].copy_from_slice(&source[..n]);
                dest[n] = 0;
            }

            printLine(&dest);
        }
    }
}
```

**Entity:** printLine(line: &[i8]) / NUL-terminated C string buffer passed as slice

**States:** InvalidCStrBuffer, ValidCStrBuffer

**Transitions:**
- InvalidCStrBuffer -> ValidCStrBuffer by constructing/validating as CStr/CString before calling printLine()

**Evidence:** printLine(line: &[i8]) takes a raw byte slice with no C-string guarantees; comment: "SAFETY: `line` is expected to be a NUL-terminated C string buffer."; unsafe { CStr::from_ptr(line.as_ptr() as *const i8) } relies on NUL-termination beyond the slice type's guarantees; if line.is_empty() { return; } is a runtime guard but does not ensure NUL termination

**Implementation:** Change the API to accept &CStr (or &CString) instead of &[i8]. If callers naturally have fixed buffers, introduce a validated wrapper like struct NulTerminated<'a>(&'a [i8]); impl TryFrom<&'a [i8]> for NulTerminated<'a> { /* check contains 0 */ } and have printLine take NulTerminated or directly &CStr (constructed safely).

---

## Protocol Invariants

### 2. Bounded copy + NUL-termination protocol for dest (Uninitialized/NotTerminated -> Terminated)

**Location**: `/data/test_case/lib.rs:1-43`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: driver intends to build a NUL-terminated string in dest by copying n bytes and then writing dest[n] = 0. This requires a compound invariant: (1) n must be within dest capacity minus 1 to leave room for the terminator, and (2) the 'else' path (data >= 100) leaves dest as all zeros (currently effectively a valid empty C string), while the 'then' path must ensure n is a safe index. The code only checks data < 100, but allows negative data; casting a negative i32 to usize yields a huge value, which can panic during slicing/copy and would be UB if it were done with unchecked indexing. The correctness of later printLine(&dest) relies on dest being NUL-terminated after these operations, but that is maintained by runtime logic rather than enforced by types.

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

            // SAFETY: `line` is expected to be a NUL-terminated C string buffer.
            let cstr = unsafe { CStr::from_ptr(line.as_ptr() as *const i8) };
            println!("{}", cstr.to_string_lossy());
        }

        #[no_mangle]
        pub extern "C" fn driver(data: i32) {
            let mut source = [0i8; 100];
            let mut dest = [0i8; 100];

            source[..99].fill(b'A' as i8);
            source[99] = 0;

            if data < 100 {
                // Preserve original behavior: negative `data` leads to huge usize and would panic.
                let n = data as usize;
                dest[..n].copy_from_slice(&source[..n]);
                dest[n] = 0;
            }

            printLine(&dest);
        }
    }
}
```

**Entity:** driver(data: i32) buffer copy protocol (source/dest arrays)

**States:** DestUnterminatedOrUninitialized, DestProperlyTerminated

**Transitions:**
- DestUnterminatedOrUninitialized -> DestProperlyTerminated via the 'data < 100' branch performing copy and writing dest[n] = 0
- DestUnterminatedOrUninitialized -> DestProperlyTerminated via the 'data >= 100' branch leaving dest as zero-initialized (empty C string)

**Evidence:** driver: let mut dest = [0i8; 100]; and later printLine(&dest) assumes it is a C string buffer; driver: if data < 100 { let n = data as usize; dest[..n].copy_from_slice(&source[..n]); dest[n] = 0; }; comment: "negative `data` leads to huge usize and would panic" describes a precondition on data (must be non-negative) that is not enforced; source[99] = 0 and dest[n] = 0 show the implicit NUL-termination protocol

**Implementation:** Introduce a validated index/capacity type (e.g., struct CopyLen<const N: usize>(usize); impl TryFrom<i32> for CopyLen<100> { /* require 0 <= data && data < N */ }). Use it so the branch can be expressed as fn driver(data: CopyLen<100>) or internally convert with TryFrom and only proceed on Ok. Alternatively accept u8/u16 (non-negative) and clamp/check to <= 99, and encapsulate the 'write C string into buffer' logic in a helper returning a &CStr or a dedicated NulTerminatedBuffer<100> wrapper.

---

