# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. Malloc-owned pointer array protocol (Allocated / Freed-on-error / Leaked-to-caller)

**Location**: `/data/test_case/lib.rs:1-81`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The functions allocate a C-compatible array of `*const i8` using `malloc` and return it to the caller on success. On failure paths they call `free` internally and return null. On success, the returned pointer must be freed by the caller with `free` (and with the same allocator). This ownership/cleanup protocol is only described implicitly by comments and by the use of raw pointers; Rust’s type system does not express 'must free with free() exactly once' or prevent misuse (double-free, forgetting to free, freeing with the wrong allocator).

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
        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
        }

        pub(crate) unsafe fn UTIL_createLinePointers_internal(
            buffer: &[i8],
            numLines: usize,
            bufferSize: usize,
        ) -> *const *const i8 {
            let mut line_index: usize = 0;
            let mut pos: usize = 0;

            // Allocate C-compatible array of pointers (caller expects malloc-allocated memory).
            let alloc_size = numLines.wrapping_mul(core::mem::size_of::<*const i8>());
            let buffer_ptrs = malloc(alloc_size);
            if buffer_ptrs.is_null() {
                return core::ptr::null();
            }
            let line_pointers = buffer_ptrs as *mut *const i8;

            // Never index beyond the provided slice; C code assumes bufferSize is valid for buffer.
            let effective_size = core::cmp::min(bufferSize, buffer.len());

            while line_index < numLines && pos < effective_size {
                // Record pointer to start of current line.
                *line_pointers.add(line_index) = buffer[pos..].as_ptr();
                line_index = line_index.wrapping_add(1);

                // Find NUL terminator within bounds.
                let mut len: usize = 0;
                while pos.wrapping_add(len) < effective_size && buffer[pos.wrapping_add(len)] != 0 {
                    len = len.wrapping_add(1);
                }

                // Advance past the string and its NUL if present.
                pos = pos.wrapping_add(len);
                if pos < effective_size {
                    pos = pos.wrapping_add(1);
                }
            }

            if line_index != numLines {
                free(buffer_ptrs);
                return core::ptr::null();
            }

            line_pointers as *const *const i8
        }

        #[no_mangle]
        pub unsafe extern "C" fn UTIL_createLinePointers(
            buffer: *const i8,
            numLines: usize,
            bufferSize: usize,
        ) -> *const *const i8 {
            UTIL_createLinePointers_internal(
                if buffer.is_null() || bufferSize == 0 {
                    &[]
                } else {
                    // Use bufferSize as the slice length to avoid out-of-bounds indexing.
                    core::slice::from_raw_parts(buffer, bufferSize)
                },
                numLines,
                bufferSize,
            )
        }
    }
}
```

**Entity:** UTIL_createLinePointers_internal / UTIL_createLinePointers (returned *const *const i8)

**States:** NotAllocated, Allocated, FreedOnError, ReturnedToCaller

**Transitions:**
- NotAllocated -> Allocated via malloc(alloc_size)
- Allocated -> FreedOnError via free(buffer_ptrs) when line_index != numLines
- Allocated -> ReturnedToCaller via returning line_pointers as *const *const i8 (caller must free)

**Evidence:** extern "C" { fn malloc(...); fn free(...); }; comment: "Allocate C-compatible array of pointers (caller expects malloc-allocated memory)."; code: let buffer_ptrs = malloc(alloc_size); if buffer_ptrs.is_null() { return core::ptr::null(); }; code: if line_index != numLines { free(buffer_ptrs); return core::ptr::null(); }; code: line_pointers as *const *const i8 (raw pointer returned with no RAII)

**Implementation:** Introduce an internal RAII wrapper like `struct MallocPtr<T>(*mut T); impl Drop for MallocPtr<T> { fn drop(&mut self){ unsafe{ free(self.0 as *mut c_void) }}}` and return it from the Rust-internal API. For the `extern "C"` boundary, keep returning `*const *const i8`, but implement it by `let p = MallocPtr::new(...)?; core::mem::forget(p);` so Rust code cannot accidentally leak or double-free on error paths.

---

## Precondition Invariants

### 2. C-string-lines parsing preconditions (ValidBuffer/ValidLayout for numLines)

**Location**: `/data/test_case/lib.rs:1-81`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Correct operation depends on an implicit input contract: `buffer` must point to at least `bufferSize` bytes (when non-null), and within the first `min(bufferSize, buffer.len())` bytes there must be at least `numLines` NUL-terminated strings (or else the function frees and returns null). The function also treats `buffer == null` or `bufferSize == 0` as an empty slice, which will fail unless `numLines == 0`. These are protocol/precondition requirements typical of C APIs but not represented in Rust types (e.g., a validated slice length, or a type representing "N NUL-terminated strings").

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
        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
        }

        pub(crate) unsafe fn UTIL_createLinePointers_internal(
            buffer: &[i8],
            numLines: usize,
            bufferSize: usize,
        ) -> *const *const i8 {
            let mut line_index: usize = 0;
            let mut pos: usize = 0;

            // Allocate C-compatible array of pointers (caller expects malloc-allocated memory).
            let alloc_size = numLines.wrapping_mul(core::mem::size_of::<*const i8>());
            let buffer_ptrs = malloc(alloc_size);
            if buffer_ptrs.is_null() {
                return core::ptr::null();
            }
            let line_pointers = buffer_ptrs as *mut *const i8;

            // Never index beyond the provided slice; C code assumes bufferSize is valid for buffer.
            let effective_size = core::cmp::min(bufferSize, buffer.len());

            while line_index < numLines && pos < effective_size {
                // Record pointer to start of current line.
                *line_pointers.add(line_index) = buffer[pos..].as_ptr();
                line_index = line_index.wrapping_add(1);

                // Find NUL terminator within bounds.
                let mut len: usize = 0;
                while pos.wrapping_add(len) < effective_size && buffer[pos.wrapping_add(len)] != 0 {
                    len = len.wrapping_add(1);
                }

                // Advance past the string and its NUL if present.
                pos = pos.wrapping_add(len);
                if pos < effective_size {
                    pos = pos.wrapping_add(1);
                }
            }

            if line_index != numLines {
                free(buffer_ptrs);
                return core::ptr::null();
            }

            line_pointers as *const *const i8
        }

        #[no_mangle]
        pub unsafe extern "C" fn UTIL_createLinePointers(
            buffer: *const i8,
            numLines: usize,
            bufferSize: usize,
        ) -> *const *const i8 {
            UTIL_createLinePointers_internal(
                if buffer.is_null() || bufferSize == 0 {
                    &[]
                } else {
                    // Use bufferSize as the slice length to avoid out-of-bounds indexing.
                    core::slice::from_raw_parts(buffer, bufferSize)
                },
                numLines,
                bufferSize,
            )
        }
    }
}
```

**Entity:** UTIL_createLinePointers / UTIL_createLinePointers_internal (buffer/numLines/bufferSize contract)

**States:** InvalidInput, ValidInputButInsufficientLines, ValidInputWithExactlyNumLines

**Transitions:**
- InvalidInput -> ValidInputButInsufficientLines via mapping null/size to &[] and attempting parse
- ValidInputButInsufficientLines -> InvalidInput (reported) via returning null after free when line_index != numLines
- ValidInputWithExactlyNumLines -> (success) via returning non-null pointer array

**Evidence:** comment: "Never index beyond the provided slice; C code assumes bufferSize is valid for buffer."; code: UTIL_createLinePointers: if buffer.is_null() || bufferSize == 0 { &[] } else { core::slice::from_raw_parts(buffer, bufferSize) }; code: effective_size = core::cmp::min(bufferSize, buffer.len()); code: while line_index < numLines ... find NUL terminator within bounds ... advance past ...; code: if line_index != numLines { free(buffer_ptrs); return core::ptr::null(); } (signals 'must contain numLines lines')

**Implementation:** For Rust-facing callers, introduce validated input types such as `struct Buf<'a>(&'a [i8]); struct NulSeparatedLines<'a> { buf: &'a [i8], starts: Vec<usize> }` created by a fallible constructor that checks `numLines` and NUL terminators. Then `UTIL_createLinePointers_internal` can accept `NulSeparatedLines` (or a `&[&CStr]`) so the parsing/"exactly numLines" invariant is established once and reused without runtime re-checking inside the allocator routine.

---

