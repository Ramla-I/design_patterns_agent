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

### 1. C-string slice validity precondition (NUL-terminated, non-empty, stable pointer)

**Location**: `/data/test_case/lib.rs:1-49`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: printLine assumes its `line: &[i8]` argument actually represents a valid C string: it must be non-empty, NUL-terminated, and backed by memory that remains valid for the duration of the call. The function partially checks this at runtime (`is_empty()`), but it does not enforce (at the type level) the stronger invariant required by `CStr::from_ptr`: that `line.as_ptr()` points to a NUL-terminated sequence. Callers rely on convention/comments (passing `b"...\0"` cast to `[i8]`) to satisfy this. Misuse (e.g., missing trailing NUL, interior NUL expectations, or non-UTF8 expectations) would be UB or surprising output, and the type signature does not prevent it.

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

            // SAFETY: `line` originates from C-style NUL-terminated bytes in this crate.
            // We interpret it as a C string and print it as UTF-8.
            let s = unsafe { CStr::from_ptr(line.as_ptr() as *const i8) };
            println!("{}", s.to_string_lossy());
        }

        pub(crate) fn bad() {
            printLine(bytemuck::cast_slice(b"bad()\0"));
        }

        fn helperGood() {
            printLine(bytemuck::cast_slice(b"helperGood()\0"));
        }

        pub(crate) fn good() {
            printLine(bytemuck::cast_slice(b"good()\0"));
            helperGood();
        }

        #[no_mangle]
        pub extern "C" fn driver() {
            printLine(bytemuck::cast_slice(b"Calling good()...\0"));
            good();
            printLine(bytemuck::cast_slice(b"Finished good()\0"));
            printLine(bytemuck::cast_slice(b"Calling bad()...\0"));
            bad();
            printLine(bytemuck::cast_slice(b"Finished bad()\0"));
        }
    }
}
```

**Entity:** printLine(line: &[i8])

**States:** Invalid/unknown bytes, Valid C string bytes (NUL-terminated, non-empty)

**Transitions:**
- Invalid/unknown bytes -> Valid C string bytes by constructing/passing a NUL-terminated buffer (e.g., b"...\0")

**Evidence:** fn printLine(line: &[i8]) takes a raw slice of i8 rather than a validated C string type; printLine: `if line.is_empty() { return; }` is a runtime guard for one precondition; comment in printLine: `SAFETY: line originates from C-style NUL-terminated bytes in this crate.` documents an implicit invariant; printLine: `unsafe { CStr::from_ptr(line.as_ptr() as *const i8) }` requires a NUL-terminated string at that pointer; call sites: `bytemuck::cast_slice(b"...\0")` rely on a trailing `\0` convention to satisfy the invariant

**Implementation:** Change `printLine` to accept `&CStr` (or a crate-local newtype like `struct NulTerminatedI8<'a>(&'a CStr)` / `struct Ci8Str<'a>(&'a CStr)`). Provide constructors like `fn from_bytes_with_nul(bytes: &'a [u8]) -> Result<Self, _>` (using `CStr::from_bytes_with_nul`) so the NUL-termination invariant is checked once at construction; then `printLine` becomes safe and `unsafe CStr::from_ptr` disappears.

---

