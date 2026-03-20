# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 2. Leaked output buffer ownership protocol (caller must eventually free, but no mechanism provided)

**Location**: `/data/test_case/lib.rs:1-96`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The function allocates an output `Vec<i8>` and then intentionally leaks it (`mem::forget`) to return a stable pointer across FFI. This creates an implicit ownership/cleanup protocol: after returning, the allocation is no longer managed by Rust and must be freed by some external mechanism. However, the type signature `-> *const i8` does not encode ownership transfer, the required deallocator, or even that the pointer is heap-allocated and unique. The lifecycle transition is enforced only by `mem::forget` and a comment; without a paired `free` API or an owning wrapper type, this is a latent invariant that can easily lead to leaks or mismatched frees.

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

use core::ptr;

fn encode(u: u8) -> i8 {
    match u {
        0..=25 => (b'A' + u) as i8,
        26..=51 => (b'a' + (u - 26)) as i8,
        52..=61 => (b'0' + (u - 52)) as i8,
        62 => b'+' as i8,
        _ => b'/' as i8,
    }
}

pub(crate) unsafe fn encode_base64_internal(mut size: i32, src: &[i8]) -> *const i8 {
    if src.is_empty() {
        return ptr::null();
    }

    // If size==0, treat src as a C string and compute length up to NUL.
    // IMPORTANT: Do not unwrap; if no NUL is present in the provided window,
    // fall back to using the full slice length (matches typical C behavior
    // within the provided buffer).
    if size == 0 {
        let mut len: usize = 0;
        while len < src.len() && src[len] != 0 {
            len += 1;
        }
        size = len as i32;
    }

    if size < 0 {
        return ptr::null();
    }

    let size_usize = size as usize;

    // Allocate output: 4 chars per 3 bytes, plus 4 extra like original.
    // Original code used calloc and returned a pointer to a NUL-filled buffer;
    // it did not explicitly NUL-terminate at the end, but the buffer is zeroed.
    let out_len = size_usize.saturating_mul(4) / 3 + 4;
    let mut out: Vec<i8> = vec![0; out_len];

    let mut p: usize = 0;
    let mut i: usize = 0;

    while i < size_usize {
        // Preserve original semantics: b1 always read; b2/b3 only if within `size`.
        // Wrapper provides a fixed window; we keep behavior consistent with that.
        let b1: u8 = src[i] as u8;
        let b2: u8 = if i + 1 < size_usize { src[i + 1] as u8 } else { 0 };
        let b3: u8 = if i + 2 < size_usize { src[i + 2] as u8 } else { 0 };

        let b4: u8 = b1 >> 2;
        let b5: u8 = ((b1 & 0x03) << 4) | (b2 >> 4);
        let b6: u8 = ((b2 & 0x0f) << 2) | (b3 >> 6);
        let b7: u8 = b3 & 0x3f;

        // Ensure we have room for 4 output bytes.
        if p + 3 >= out.len() {
            return ptr::null();
        }

        out[p] = encode(b4);
        out[p + 1] = encode(b5);
        out[p + 2] = if i + 1 < size_usize { encode(b6) } else { b'=' as i8 };
        out[p + 3] = if i + 2 < size_usize { encode(b7) } else { b'=' as i8 };

        p += 4;
        i += 3;
    }

    let ptr_out = out.as_ptr();
    core::mem::forget(out); // leak to preserve C-style ownership across FFI
    ptr_out
}

#[no_mangle]
pub unsafe extern "C" fn encode_base64(size: i32, src: *const i8) -> *const i8 {
    encode_base64_internal(
        size,
        if src.is_null() {
            &[]
        } else {
            core::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** encode_base64_internal (returned pointer ownership)

**States:** AllocatedAndOwnedByRustVec, LeakedToCallerAsRawPointer

**Transitions:**
- AllocatedAndOwnedByRustVec -> LeakedToCallerAsRawPointer via `core::mem::forget(out)`

**Evidence:** encode_base64_internal: `let mut out: Vec<i8> = vec![0; out_len];` allocates owned buffer; encode_base64_internal: `let ptr_out = out.as_ptr(); core::mem::forget(out); // leak to preserve C-style ownership across FFI` explicitly leaks and documents the protocol; encode_base64_internal returns `*const i8` (raw pointer carries no ownership/deallocation info)

**Implementation:** On the Rust side, return an owning type (e.g., `CString`/`Vec<u8>`) for safe callers. For FFI, export a paired `#[no_mangle] extern "C" fn encode_base64_free(ptr: *mut i8, len: usize)` (or return a `struct { ptr: *mut c_char, len: usize }`) so ownership transfer is explicit and the correct allocator is used. Internally, use `Vec::into_raw_parts`/`CString::into_raw` to make ownership transfer explicit rather than relying on `mem::forget`.

---

## Protocol Invariants

### 1. FFI input validity + length protocol (Null/Empty vs Valid pointer; Explicit size vs C-string length)

**Location**: `/data/test_case/lib.rs:1-96`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The API encodes an implicit protocol for how `src` and `size` must be interpreted. If `src` is null (or slice is empty), the function returns null. If `size > 0`, it treats the first `size` bytes of `src` as the input window. If `size == 0`, it treats `src` as a C string and computes the length up to the first NUL *within the provided window*. If `size < 0`, it returns null. None of these states/interpretations are represented in the type system: the public FFI takes a raw pointer plus an `i32` flag-like length, and internally relies on runtime checks and comments to enforce meaning. The wrapper further hard-codes a 1024-byte readable window, implicitly requiring that `src` (when non-null) points to at least 1024 readable bytes, which is an unstated precondition for safety/correctness at the boundary.

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

use core::ptr;

fn encode(u: u8) -> i8 {
    match u {
        0..=25 => (b'A' + u) as i8,
        26..=51 => (b'a' + (u - 26)) as i8,
        52..=61 => (b'0' + (u - 52)) as i8,
        62 => b'+' as i8,
        _ => b'/' as i8,
    }
}

pub(crate) unsafe fn encode_base64_internal(mut size: i32, src: &[i8]) -> *const i8 {
    if src.is_empty() {
        return ptr::null();
    }

    // If size==0, treat src as a C string and compute length up to NUL.
    // IMPORTANT: Do not unwrap; if no NUL is present in the provided window,
    // fall back to using the full slice length (matches typical C behavior
    // within the provided buffer).
    if size == 0 {
        let mut len: usize = 0;
        while len < src.len() && src[len] != 0 {
            len += 1;
        }
        size = len as i32;
    }

    if size < 0 {
        return ptr::null();
    }

    let size_usize = size as usize;

    // Allocate output: 4 chars per 3 bytes, plus 4 extra like original.
    // Original code used calloc and returned a pointer to a NUL-filled buffer;
    // it did not explicitly NUL-terminate at the end, but the buffer is zeroed.
    let out_len = size_usize.saturating_mul(4) / 3 + 4;
    let mut out: Vec<i8> = vec![0; out_len];

    let mut p: usize = 0;
    let mut i: usize = 0;

    while i < size_usize {
        // Preserve original semantics: b1 always read; b2/b3 only if within `size`.
        // Wrapper provides a fixed window; we keep behavior consistent with that.
        let b1: u8 = src[i] as u8;
        let b2: u8 = if i + 1 < size_usize { src[i + 1] as u8 } else { 0 };
        let b3: u8 = if i + 2 < size_usize { src[i + 2] as u8 } else { 0 };

        let b4: u8 = b1 >> 2;
        let b5: u8 = ((b1 & 0x03) << 4) | (b2 >> 4);
        let b6: u8 = ((b2 & 0x0f) << 2) | (b3 >> 6);
        let b7: u8 = b3 & 0x3f;

        // Ensure we have room for 4 output bytes.
        if p + 3 >= out.len() {
            return ptr::null();
        }

        out[p] = encode(b4);
        out[p + 1] = encode(b5);
        out[p + 2] = if i + 1 < size_usize { encode(b6) } else { b'=' as i8 };
        out[p + 3] = if i + 2 < size_usize { encode(b7) } else { b'=' as i8 };

        p += 4;
        i += 3;
    }

    let ptr_out = out.as_ptr();
    core::mem::forget(out); // leak to preserve C-style ownership across FFI
    ptr_out
}

#[no_mangle]
pub unsafe extern "C" fn encode_base64(size: i32, src: *const i8) -> *const i8 {
    encode_base64_internal(
        size,
        if src.is_null() {
            &[]
        } else {
            core::slice::from_raw_parts(src, 1024)
        },
    )
}
```

**Entity:** encode_base64_internal / encode_base64 (FFI boundary)

**States:** NullOrEmptyInput, ValidPointerWithExplicitSize, ValidPointerWithCStringLength(size==0)

**Transitions:**
- ValidPointerWithCStringLength(size==0) -> ValidPointerWithExplicitSize via computed `len` (size becomes len as i32)
- AnyInput -> NullOrEmptyInput via src.is_null() (wrapper passes &[]) or src.is_empty() (internal returns null)
- AnyInput -> NullOrEmptyInput via size < 0 (internal returns null)

**Evidence:** encode_base64: `if src.is_null() { &[] } else { core::slice::from_raw_parts(src, 1024) }` hard-codes a 1024-byte readable window requirement; encode_base64_internal: `if src.is_empty() { return ptr::null(); }` defines the NullOrEmptyInput outcome; comment in encode_base64_internal: `If size==0, treat src as a C string and compute length up to NUL.`; encode_base64_internal: `if size == 0 { ... while len < src.len() && src[len] != 0 { ... } size = len as i32; }` implements the size==0 C-string-length mode; encode_base64_internal: `if size < 0 { return ptr::null(); }` defines negative sizes as invalid

**Implementation:** Model the interpretation of `size` explicitly: e.g., `enum InputLen { Explicit(NonZeroUsize), CString }` and a safe Rust entry point `fn encode_base64_safe(input: &[u8]) -> CString` plus a separate FFI shim that validates `src`/`size` and converts into the enum. For the raw-pointer API, use a newtype like `struct FfiSrcWindow<'a>(&'a [u8]);` constructed only after validating the pointer+available length (rather than unconditionally assuming 1024).

---

