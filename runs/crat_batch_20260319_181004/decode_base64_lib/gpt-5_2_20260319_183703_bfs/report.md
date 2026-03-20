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

### 1. FFI-owned decoded C-string lifetime/ownership protocol (Null | OwnedAllocated)

**Location**: `/data/test_case/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: decode_base64_internal returns a raw pointer that is either null (on empty/NUL input or allocation failure) or points to a heap allocation produced by calloc and intended to be treated as a NUL-terminated C string. The caller implicitly owns this allocation and must eventually free it (with the matching allocator) to avoid leaks. None of this ownership/lifetime information is represented in the type system: the return type is *const i8 with no indication of nullability, allocation source, required deallocation, or that the memory is NUL-terminated.

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

extern "C" {
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn calloc(__nmemb: usize, __size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
}

pub const TRUE: i32 = 1;
pub const FALSE: i32 = 0;

#[inline]
fn decode(c: i8) -> u8 {
    let c = c as u8;
    match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        _ => 63,
    }
}

#[inline]
fn is_base64(c: i8) -> i32 {
    let c = c as u8;
    if matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'=') {
        TRUE
    } else {
        FALSE
    }
}

pub(crate) unsafe fn decode_base64_internal(src: &[i8]) -> *const i8 {
    if src.is_empty() || src[0] == 0 {
        return core::ptr::null();
    }

    // C-string semantics: stop at first NUL (or end of provided slice).
    let nul_pos = src.iter().position(|&b| b == 0).unwrap_or(src.len());
    // Original code: l = strlen(src) + 1
    let l_with_nul = nul_pos + 1;

    // Allocate output: calloc(sizeof(i8), l + 13)
    let dest_len = l_with_nul + 13;
    let dest_ptr = calloc(core::mem::size_of::<i8>(), dest_len) as *mut i8;
    if dest_ptr.is_null() {
        return core::ptr::null();
    }

    // Allocate temp buffer: malloc(l)
    let buf_ptr = malloc(l_with_nul) as *mut u8;
    if buf_ptr.is_null() {
        free(dest_ptr as *mut core::ffi::c_void);
        return core::ptr::null();
    }
    let buf = core::slice::from_raw_parts_mut(buf_ptr, l_with_nul);

    // Filter base64 chars into buf.
    let mut filtered_len: usize = 0;
    for &b in &src[..nul_pos] {
        if is_base64(b) != 0 {
            buf[filtered_len] = b as u8;
            filtered_len += 1;
        }
    }

    // Decode into dest.
    let mut out = core::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len);
    let mut k: usize = 0;
    while k < filtered_len {
        let c1: i8 = buf[k] as i8;
        let c2: i8 = if k + 1 < filtered_len {
            buf[k + 1] as i8
        } else {
            b'A' as i8
        };
        let c3: i8 = if k + 2 < filtered_len {
            buf[k + 2] as i8
        } else {
            b'A' as i8
        };
        let c4: i8 = if k + 3 < filtered_len {
            buf[k + 3] as i8
        } else {
            b'A' as i8
        };

        let b1 = decode(c1);
        let b2 = decode(c2);
        let b3 = decode(c3);
        let b4 = decode(c4);

        out[0] = ((b1 as u32) << 2 | (b2 as u32) >> 4) as u8;
        out = &mut out[1..];

        if c3 != b'=' as i8 {
            out[0] = (((b2 as u32) & 0x0f) << 4 | (b3 as u32) >> 2) as u8;
            out = &mut out[1..];
        }
        if c4 != b'=' as i8 {
            out[0] = (((b3 as u32) & 0x03) << 6 | (b4 as u32)) as u8;
            out = &mut out[1..];
        }

        k += 4;
    }

    // Ensure NUL termination at the actual end of produced output.
    // (calloc already zeroed, but this matches C-string expectations precisely)
    if !out.is_empty() {
        out[0] = 0;
    } else {
        // Extremely unlikely, but keep within bounds.
        *dest_ptr.add(dest_len - 1) = 0;
    }

    free(buf_ptr as *mut core::ffi::c_void);
    dest_ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn decode_base64(src: *const i8) -> *const i8 {
    decode_base64_internal(if src.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(src, 1024)
    })
}
```

**Entity:** decode_base64_internal (returned *const i8)

**States:** Null (no allocation / allocation failure), OwnedAllocated (heap-allocated, NUL-terminated C string)

**Transitions:**
- Null -> OwnedAllocated via successful calloc/malloc + decode loop
- OwnedAllocated -> (freed) via caller calling free() (implicit, not provided in API)

**Evidence:** decode_base64_internal: `if src.is_empty() || src[0] == 0 { return core::ptr::null(); }` encodes the Null state; decode_base64_internal: `let dest_ptr = calloc(...) as *mut i8; if dest_ptr.is_null() { return core::ptr::null(); }` encodes allocation-failure -> Null; decode_base64_internal: comment `// Ensure NUL termination ... matches C-string expectations` plus writes `out[0] = 0` / `*dest_ptr.add(dest_len - 1) = 0` encodes 'NUL-terminated string' invariant; decode_base64_internal: `free(buf_ptr ...)` frees only the temp buffer, while `dest_ptr` is returned and never freed here (implying caller must free); decode_base64 (FFI): signature `pub unsafe extern "C" fn decode_base64(...) -> *const i8` exposes the raw pointer with no ownership semantics

**Implementation:** For Rust callers, return an owned RAII wrapper like `struct CStringOwned(NonNull<c_char>); impl Drop for CStringOwned { fn drop(&mut self){ unsafe{ free(self.0.as_ptr().cast()) } } }` and expose `as_ptr()` for FFI. For C API, provide an explicit `decode_base64_free(ptr: *mut c_char)` function and document that only pointers returned by decode_base64 may be freed with it; alternatively return `Option<NonNull<i8>>` internally to encode non-null at the type level.

---

## Precondition Invariants

### 2. Input pointer validity/termination precondition (ValidReadableUpToNULWithin1024)

**Location**: `/data/test_case/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: decode_base64 constructs a Rust slice from a raw C pointer with a fixed length of 1024 bytes and then applies C-string semantics (search for NUL). This relies on an unstated precondition: if `src` is non-null, it must point to at least 1024 readable bytes (or at minimum be safely readable until the first NUL within those 1024 bytes). Otherwise, `from_raw_parts(src, 1024)` is immediate undefined behavior. The type system does not express this requirement; it is only implied by the unsafe FFI signature and the fixed-length slice creation.

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

extern "C" {
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn calloc(__nmemb: usize, __size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
}

pub const TRUE: i32 = 1;
pub const FALSE: i32 = 0;

#[inline]
fn decode(c: i8) -> u8 {
    let c = c as u8;
    match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        _ => 63,
    }
}

#[inline]
fn is_base64(c: i8) -> i32 {
    let c = c as u8;
    if matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'=') {
        TRUE
    } else {
        FALSE
    }
}

pub(crate) unsafe fn decode_base64_internal(src: &[i8]) -> *const i8 {
    if src.is_empty() || src[0] == 0 {
        return core::ptr::null();
    }

    // C-string semantics: stop at first NUL (or end of provided slice).
    let nul_pos = src.iter().position(|&b| b == 0).unwrap_or(src.len());
    // Original code: l = strlen(src) + 1
    let l_with_nul = nul_pos + 1;

    // Allocate output: calloc(sizeof(i8), l + 13)
    let dest_len = l_with_nul + 13;
    let dest_ptr = calloc(core::mem::size_of::<i8>(), dest_len) as *mut i8;
    if dest_ptr.is_null() {
        return core::ptr::null();
    }

    // Allocate temp buffer: malloc(l)
    let buf_ptr = malloc(l_with_nul) as *mut u8;
    if buf_ptr.is_null() {
        free(dest_ptr as *mut core::ffi::c_void);
        return core::ptr::null();
    }
    let buf = core::slice::from_raw_parts_mut(buf_ptr, l_with_nul);

    // Filter base64 chars into buf.
    let mut filtered_len: usize = 0;
    for &b in &src[..nul_pos] {
        if is_base64(b) != 0 {
            buf[filtered_len] = b as u8;
            filtered_len += 1;
        }
    }

    // Decode into dest.
    let mut out = core::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len);
    let mut k: usize = 0;
    while k < filtered_len {
        let c1: i8 = buf[k] as i8;
        let c2: i8 = if k + 1 < filtered_len {
            buf[k + 1] as i8
        } else {
            b'A' as i8
        };
        let c3: i8 = if k + 2 < filtered_len {
            buf[k + 2] as i8
        } else {
            b'A' as i8
        };
        let c4: i8 = if k + 3 < filtered_len {
            buf[k + 3] as i8
        } else {
            b'A' as i8
        };

        let b1 = decode(c1);
        let b2 = decode(c2);
        let b3 = decode(c3);
        let b4 = decode(c4);

        out[0] = ((b1 as u32) << 2 | (b2 as u32) >> 4) as u8;
        out = &mut out[1..];

        if c3 != b'=' as i8 {
            out[0] = (((b2 as u32) & 0x0f) << 4 | (b3 as u32) >> 2) as u8;
            out = &mut out[1..];
        }
        if c4 != b'=' as i8 {
            out[0] = (((b3 as u32) & 0x03) << 6 | (b4 as u32)) as u8;
            out = &mut out[1..];
        }

        k += 4;
    }

    // Ensure NUL termination at the actual end of produced output.
    // (calloc already zeroed, but this matches C-string expectations precisely)
    if !out.is_empty() {
        out[0] = 0;
    } else {
        // Extremely unlikely, but keep within bounds.
        *dest_ptr.add(dest_len - 1) = 0;
    }

    free(buf_ptr as *mut core::ffi::c_void);
    dest_ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn decode_base64(src: *const i8) -> *const i8 {
    decode_base64_internal(if src.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(src, 1024)
    })
}
```

**Entity:** decode_base64 (FFI boundary: src pointer + fixed-length slice creation)

**States:** NullInput, NonNullButPossiblyInvalid, ValidReadableAndNULTerminatedWithin1024

**Transitions:**
- NullInput -> (treated as empty slice) via `if src.is_null() { &[] }`
- NonNullButPossiblyInvalid -> ValidReadableAndNULTerminatedWithin1024 when caller upholds safety precondition (implicit)

**Evidence:** decode_base64: `pub unsafe extern "C" fn decode_base64(src: *const i8) -> *const i8` exposes raw pointer with safety requirements; decode_base64: `core::slice::from_raw_parts(src, 1024)` requires `src` be valid for reads of 1024 bytes; decode_base64_internal: comment `// C-string semantics: stop at first NUL (or end of provided slice).` relies on existence of a NUL within the provided readable region

**Implementation:** Split the API: keep the raw FFI `unsafe` function but implement safe Rust entrypoints that take `&CStr` (or `*const c_char` plus a checked `CStr::from_ptr`) to enforce NUL-termination, and/or take a `&[u8]` with an explicit length from the caller rather than assuming 1024. Internally, use `Option<NonNull<i8>>` for nullability and accept a `&CStr` capability token for 'valid NUL-terminated input'.

---

