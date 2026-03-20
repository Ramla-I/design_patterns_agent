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

### 2. Raw pointer validity + minimum length protocol for creating a Rust slice

**Location**: `/data/test_case/lib.rs:1-54`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: driver() converts an incoming raw pointer into a Rust slice with `from_raw_parts(in_0, 1024)` unless it is null. This encodes an implicit protocol: callers must either pass null (meaning empty input) or pass a non-null pointer to at least 1024 readable bytes for the duration of the call. This requirement is not expressed in the type system; it is only partially checked at runtime (null check) and otherwise relies on caller correctness.

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
        use std::ptr;

        extern "C" {
            fn strchr(__s: *const i8, __c: i32) -> *mut i8;
        }

        pub(crate) unsafe fn foo(mut in_0: &[i8], c: i8) -> i32 {
            let mut res: i32 = 0;

            while !in_0.is_empty() {
                let found = strchr(in_0.as_ptr(), c as i32);
                if found.is_null() {
                    break;
                }

                res += 1;

                // Advance past the found character. We don't know the true remaining length,
                // so keep the original "large slice" behavior but avoid raw pointer arithmetic.
                in_0 = std::slice::from_raw_parts(found, 100000);
                in_0 = &in_0[1..];
            }

            res
        }

        pub(crate) unsafe fn driver_internal(in_0: &[i8]) {
            println!("A: {0}", foo(in_0, b'A' as i8));
            println!("x: {0}", foo(in_0, b'x' as i8));
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *const i8) {
            let slice = if in_0.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(in_0, 1024)
            };
            driver_internal(slice)
        }
    }
}
```

**Entity:** driver(in_0: *const i8)

**States:** NullPointer, ValidReadablePointerAtLeast1024

**Transitions:**
- NullPointer -> ValidReadablePointerAtLeast1024 via passing a non-null pointer to a 1024-byte readable region when calling driver()

**Evidence:** driver(): `let slice = if in_0.is_null() { &[] } else { std::slice::from_raw_parts(in_0, 1024) };` (null-vs-non-null state split + length assumption); driver(): signature `pub unsafe extern "C" fn driver(in_0: *const i8)` (unsafe raw pointer contract is external to the type system)

**Implementation:** Expose a safe Rust-facing wrapper that takes `Option<core::ptr::NonNull<i8>>` plus a length newtype (or directly `&[i8]`) and keep the raw-pointer entrypoint thin. For FFI, use a struct argument like `{ ptr: *const i8, len: usize }` (or two args) to avoid the hard-coded 1024 and make the length an explicit part of the protocol.

---

### 1. C-string / NUL-terminated buffer precondition for strchr-based scanning

**Location**: `/data/test_case/lib.rs:1-54`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: foo() passes in_0.as_ptr() to the C function strchr, which expects a valid NUL-terminated C string (or at least a readable memory region ending in '\0') starting at that pointer. However, foo() accepts an arbitrary Rust slice &[i8] with an explicit length, and nothing ensures (at compile time) that the pointer is followed by a terminator within accessible memory. The loop also 're-slices' using from_raw_parts(found, 100000), implicitly assuming that memory from `found` for 100000 bytes is readable. These are runtime/FFI preconditions not represented in the type system; violating them yields UB rather than a Rust error.

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
        use std::ptr;

        extern "C" {
            fn strchr(__s: *const i8, __c: i32) -> *mut i8;
        }

        pub(crate) unsafe fn foo(mut in_0: &[i8], c: i8) -> i32 {
            let mut res: i32 = 0;

            while !in_0.is_empty() {
                let found = strchr(in_0.as_ptr(), c as i32);
                if found.is_null() {
                    break;
                }

                res += 1;

                // Advance past the found character. We don't know the true remaining length,
                // so keep the original "large slice" behavior but avoid raw pointer arithmetic.
                in_0 = std::slice::from_raw_parts(found, 100000);
                in_0 = &in_0[1..];
            }

            res
        }

        pub(crate) unsafe fn driver_internal(in_0: &[i8]) {
            println!("A: {0}", foo(in_0, b'A' as i8));
            println!("x: {0}", foo(in_0, b'x' as i8));
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *const i8) {
            let slice = if in_0.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(in_0, 1024)
            };
            driver_internal(slice)
        }
    }
}
```

**Entity:** foo(in_0: &[i8], c: i8) (and its input slice)

**States:** ValidCStrRegion, InvalidOrNonTerminatedRegion

**Transitions:**
- InvalidOrNonTerminatedRegion -> ValidCStrRegion via constructing a proper CStr/terminated buffer before calling foo()

**Evidence:** foo(): `let found = strchr(in_0.as_ptr(), c as i32);` (FFI call requires C-string semantics); foo(): comment: "We don't know the true remaining length" indicates unknown length/readability assumptions; foo(): `in_0 = std::slice::from_raw_parts(found, 100000);` hard-codes an assumed readable region size; foo(): loop termination depends on `found.is_null()` (i.e., strchr finding a terminator / failing), not on Rust slice length

**Implementation:** Change foo to accept a validated FFI string/buffer type such as `&std::ffi::CStr` (or `*const c_char` wrapped in a newtype guaranteeing NUL-termination and validity). If the intent is bounded search, accept `&[u8]` and implement a bounded scan in Rust (or call `memchr`-like FFI with an explicit length) to avoid assuming a terminator/readable tail.

---

