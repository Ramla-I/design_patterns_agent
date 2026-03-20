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

### 1. C-string slice precondition (NUL-terminated within bounds)

**Location**: `/data/test_case/lib.rs:1-132`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: searchAndReplace_internal treats its slice inputs as C strings and scans them with c_strlen by reading until a NUL byte. This implicitly requires that each slice's backing memory contains a NUL terminator before the end of the accessible allocation. The type system only knows these are Rust slices, not that they are NUL-terminated C strings; if the NUL is missing within the actually accessible memory, c_strlen will read out of bounds (UB). This precondition is also relied upon before calling C APIs strstr/strdup which require valid NUL-terminated pointers.

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

use core::ffi::c_void;
use core::ptr;

extern "C" {
    fn malloc(__size: usize) -> *mut c_void;
    fn strdup(__s: *const i8) -> *mut i8;
    fn strstr(__haystack: *const i8, __needle: *const i8) -> *mut i8;
}

unsafe fn c_strlen(mut s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s != 0 {
        n = n.wrapping_add(1);
        s = s.add(1);
    }
    n
}

pub(crate) unsafe fn searchAndReplace_internal(orig: &[i8], search: &[i8], value: &[i8]) -> *const i8 {
    let orig_ptr = orig.as_ptr();
    let search_ptr = search.as_ptr();
    let value_ptr = value.as_ptr();

    // Inputs are treated as C strings (NUL-terminated).
    let orig_len = c_strlen(orig_ptr);
    let search_len = c_strlen(search_ptr);
    let value_len = c_strlen(value_ptr);

    // Match original behavior: if no match (or empty search), return strdup(orig).
    // Empty search would otherwise loop forever with strstr.
    if search_len == 0 {
        return strdup(orig_ptr) as *const i8;
    }

    let first = strstr(orig_ptr, search_ptr);
    if first.is_null() {
        return strdup(orig_ptr) as *const i8;
    }

    // Build output bytes (excluding final NUL) then NUL-terminate.
    let mut out: Vec<i8> = Vec::new();

    // Copy prefix before first match.
    let mut inx_start = first.offset_from(orig_ptr) as usize;
    if inx_start != 0 {
        out.extend_from_slice(&orig[..inx_start]);
    }

    // Iterate matches.
    let mut from = inx_start + search_len;
    let mut p = first;

    while !p.is_null() {
        // Insert replacement.
        if value_len != 0 {
            out.extend_from_slice(&value[..value_len]);
        }

        // Find next match starting after current match.
        let next = if from <= orig_len {
            strstr(orig_ptr.add(from), search_ptr)
        } else {
            ptr::null_mut()
        };

        if next.is_null() {
            break;
        }

        let inx_start2 = next.offset_from(orig_ptr) as usize;

        // Copy gap between end of current match and start of next match.
        if inx_start2 > from {
            out.extend_from_slice(&orig[from..inx_start2]);
        }

        inx_start = inx_start2;
        from = inx_start + search_len;
        p = next;
    }

    // Copy tail after last match.
    if from < orig_len {
        out.extend_from_slice(&orig[from..orig_len]);
    }

    // NUL-terminate.
    out.push(0);

    // Return C-allocated buffer so caller can free with C free().
    let size = out.len();
    let mem = malloc(size) as *mut i8;
    if mem.is_null() {
        return ptr::null();
    }
    ptr::copy_nonoverlapping(out.as_ptr(), mem, size);
    mem as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn searchAndReplace(orig: *const i8, search: *const i8, value: *const i8) -> *const i8 {
    searchAndReplace_internal(
        if orig.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(orig, 1024)
        },
        if search.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(search, 1024)
        },
        if value.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(value, 1024)
        },
    )
}
```

**Entity:** searchAndReplace_internal(orig: &[i8], search: &[i8], value: &[i8]) -> *const i8

**States:** Valid C-string slices, Invalid/unterminated or too-short slices

**Transitions:**
- Invalid/unterminated or too-short slices -> Valid C-string slices via validation/creation (e.g., constructing a CStr-backed wrapper)

**Evidence:** comment in searchAndReplace_internal: "Inputs are treated as C strings (NUL-terminated)."; unsafe fn c_strlen(mut s: *const i8) loops `while *s != 0` and increments pointer, with no slice length bound; searchAndReplace_internal: `let orig_len = c_strlen(orig_ptr);` similarly for search/value; searchAndReplace_internal calls `strstr(orig_ptr, search_ptr)` and `strdup(orig_ptr)` which require NUL-terminated C strings

**Implementation:** Accept `&CStr` (or a custom `struct CStrSlice<'a>(&'a CStr)` / `NonNull<c_char>` wrapper) instead of `&[i8]`. Construct it at the FFI boundary by validating `orig/search/value` are non-null and NUL-terminated (e.g., `CStr::from_ptr`). This makes the NUL-termination invariant explicit and removes the need for `c_strlen` over raw pointers.

---

## Protocol Invariants

### 2. FFI boundary protocol (pointer validity + output ownership via C allocator)

**Location**: `/data/test_case/lib.rs:1-132`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The exported FFI function has an implicit protocol: inputs may be null (interpreted as empty), but if non-null they must point to readable NUL-terminated C strings. Additionally, the return value is either null (malloc failure) or a pointer to C-allocated memory that the caller must free with C free(). None of these ownership and validity requirements are expressed in the signature (`*const i8` / `*const i8` return), so misuse (passing non-C strings, assuming Rust ownership, forgetting to free, freeing with the wrong allocator) is possible.

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

use core::ffi::c_void;
use core::ptr;

extern "C" {
    fn malloc(__size: usize) -> *mut c_void;
    fn strdup(__s: *const i8) -> *mut i8;
    fn strstr(__haystack: *const i8, __needle: *const i8) -> *mut i8;
}

unsafe fn c_strlen(mut s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s != 0 {
        n = n.wrapping_add(1);
        s = s.add(1);
    }
    n
}

pub(crate) unsafe fn searchAndReplace_internal(orig: &[i8], search: &[i8], value: &[i8]) -> *const i8 {
    let orig_ptr = orig.as_ptr();
    let search_ptr = search.as_ptr();
    let value_ptr = value.as_ptr();

    // Inputs are treated as C strings (NUL-terminated).
    let orig_len = c_strlen(orig_ptr);
    let search_len = c_strlen(search_ptr);
    let value_len = c_strlen(value_ptr);

    // Match original behavior: if no match (or empty search), return strdup(orig).
    // Empty search would otherwise loop forever with strstr.
    if search_len == 0 {
        return strdup(orig_ptr) as *const i8;
    }

    let first = strstr(orig_ptr, search_ptr);
    if first.is_null() {
        return strdup(orig_ptr) as *const i8;
    }

    // Build output bytes (excluding final NUL) then NUL-terminate.
    let mut out: Vec<i8> = Vec::new();

    // Copy prefix before first match.
    let mut inx_start = first.offset_from(orig_ptr) as usize;
    if inx_start != 0 {
        out.extend_from_slice(&orig[..inx_start]);
    }

    // Iterate matches.
    let mut from = inx_start + search_len;
    let mut p = first;

    while !p.is_null() {
        // Insert replacement.
        if value_len != 0 {
            out.extend_from_slice(&value[..value_len]);
        }

        // Find next match starting after current match.
        let next = if from <= orig_len {
            strstr(orig_ptr.add(from), search_ptr)
        } else {
            ptr::null_mut()
        };

        if next.is_null() {
            break;
        }

        let inx_start2 = next.offset_from(orig_ptr) as usize;

        // Copy gap between end of current match and start of next match.
        if inx_start2 > from {
            out.extend_from_slice(&orig[from..inx_start2]);
        }

        inx_start = inx_start2;
        from = inx_start + search_len;
        p = next;
    }

    // Copy tail after last match.
    if from < orig_len {
        out.extend_from_slice(&orig[from..orig_len]);
    }

    // NUL-terminate.
    out.push(0);

    // Return C-allocated buffer so caller can free with C free().
    let size = out.len();
    let mem = malloc(size) as *mut i8;
    if mem.is_null() {
        return ptr::null();
    }
    ptr::copy_nonoverlapping(out.as_ptr(), mem, size);
    mem as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn searchAndReplace(orig: *const i8, search: *const i8, value: *const i8) -> *const i8 {
    searchAndReplace_internal(
        if orig.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(orig, 1024)
        },
        if search.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(search, 1024)
        },
        if value.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(value, 1024)
        },
    )
}
```

**Entity:** searchAndReplace(orig: *const i8, search: *const i8, value: *const i8) -> *const i8

**States:** Null inputs allowed / treated as empty, Non-null inputs must be valid C strings, Returned pointer is C-owned and must be freed by caller

**Transitions:**
- Null input pointer -> Empty-string semantics via `if orig.is_null() { &[] } else { ... }`
- Non-null input pointer -> Interpreted as C string via `from_raw_parts(ptr, 1024)` + `c_strlen`/`strstr` usage
- Allocated result -> Freed by caller via external `free()` (implied by comment)

**Evidence:** searchAndReplace: `if orig.is_null() { &[] } else { std::slice::from_raw_parts(orig, 1024) }` (same for search/value); searchAndReplace_internal comment: "Return C-allocated buffer so caller can free with C free()."; searchAndReplace_internal: `let mem = malloc(size) as *mut i8; ... mem as *const i8` returning a raw pointer allocated by C malloc; searchAndReplace_internal: `if mem.is_null() { return ptr::null(); }` encodes the null-on-OOM convention

**Implementation:** Expose/encourage a safe wrapper returning an owning RAII type (e.g., `struct CMallocStr(NonNull<c_char>); impl Drop for CMallocStr { free(ptr) }`) and take `Option<&CStr>` for inputs. Keep the raw `extern "C"` function as a thin shim, but make the internal API typed around `CStr` and `CMallocStr` to encode validity and ownership.

---

