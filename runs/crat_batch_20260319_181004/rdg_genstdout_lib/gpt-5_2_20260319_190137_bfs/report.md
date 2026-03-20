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

### 2. Heap allocation ownership protocol for returned filename (must be freed by caller)

**Location**: `/data/test_case/lib.rs:1-165`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The function allocates a new C buffer with `calloc` and returns a raw pointer to it. This implies an ownership transfer to the caller, who must later release it using the appropriate deallocator (likely `free` in the same C runtime). This lifecycle/ownership is not represented in the type system (it is just `*const i8`), so callers can easily leak it, double-free it, or free it with the wrong allocator. Additionally, on allocation failure the function prints an error and calls `exit(30)`, meaning it never returns an error to the caller; this is a control-flow protocol that is not expressed in the signature.

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
use std::ffi::CStr;

extern "C" {
    fn calloc(__nmemb: usize, __size: usize) -> *mut c_void;
    fn exit(__status: i32) -> !;
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
    fn strrchr(__s: *const i8, __c: i32) -> *mut i8;
    fn strerror(__errnum: i32) -> *mut i8;

    // On macOS, the symbol is __error (not __errno_location).
    // Keep this internal and portable by selecting the right one per target.
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd"))]
    fn __error() -> *mut i32;

    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    fn __errno_location() -> *mut i32;
}

#[inline]
unsafe fn errno_ptr() -> *mut i32 {
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd"))]
    {
        __error()
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    {
        __errno_location()
    }
}

#[inline]
unsafe fn c_strlen_bounded(ptr: *const i8, max: usize) -> usize {
    if ptr.is_null() || max == 0 {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, max);
    bytes.iter().position(|&b| b == 0).unwrap_or(max)
}

pub(crate) unsafe fn extractFilename(path: &[i8], separator: i8) -> *const i8 {
    let last = strrchr(path.as_ptr(), separator as i32);
    if last.is_null() {
        path.as_ptr()
    } else {
        last.add(1)
    }
}

pub(crate) unsafe fn FIO_createFilename_fromOutDir_internal(
    path: &[i8],
    outDirName: &[i8],
    suffixLen: usize,
) -> *const i8 {
    let separator: i8 = b'/' as i8;

    let filename_start_ptr = extractFilename(path, separator);
    let filenameStart: &[i8] = if filename_start_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(filename_start_ptr, 100000)
    };

    let out_len = if outDirName.is_empty() {
        0
    } else {
        c_strlen_bounded(outDirName.as_ptr(), outDirName.len())
    };
    let file_len = if filenameStart.is_empty() {
        0
    } else {
        c_strlen_bounded(filenameStart.as_ptr(), filenameStart.len())
    };

    // Preserve original allocation sizing behavior.
    let total_size = out_len
        .wrapping_add(1)
        .wrapping_add(file_len)
        .wrapping_add(suffixLen)
        .wrapping_add(1);

    let buf_ptr = calloc(1, total_size) as *mut i8;
    if buf_ptr.is_null() {
        eprint!(
            "zstd: FIO_createFilename_fromOutDir: {0}",
            CStr::from_ptr(strerror(*errno_ptr()) as _)
                .to_str()
                .unwrap()
        );
        exit(30);
    }

    if out_len != 0 {
        memcpy(
            buf_ptr as *mut c_void,
            outDirName.as_ptr() as *const c_void,
            out_len,
        );
    }

    // Match original logic: if outDirName doesn't end with '/', insert it.
    // If outDirName is empty, insert '/'.
    let needs_sep = if out_len == 0 {
        true
    } else {
        *outDirName.as_ptr().add(out_len - 1) != separator
    };

    let mut write_pos = out_len;
    if needs_sep {
        *buf_ptr.add(write_pos) = separator;
        write_pos += 1;
    }

    if file_len != 0 {
        memcpy(
            buf_ptr.add(write_pos) as *mut c_void,
            filenameStart.as_ptr() as *const c_void,
            file_len,
        );
    }

    buf_ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn FIO_createFilename_fromOutDir(
    path: *const i8,
    outDirName: *const i8,
    suffixLen: usize,
) -> *const i8 {
    FIO_createFilename_fromOutDir_internal(
        if path.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(path, 1024)
        },
        if outDirName.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(outDirName, 1024)
        },
        suffixLen,
    )
}
```

**Entity:** FIO_createFilename_fromOutDir_internal (returned buffer)

**States:** Allocated buffer returned (caller owns), Allocation failed (process exits; no return)

**Transitions:**
- Start -> Allocated buffer returned via `calloc` (ownership transferred) -> Must be freed by caller
- Start -> OOM -> `exit(30)` (non-returning)

**Evidence:** FIO_createFilename_fromOutDir_internal(): `let buf_ptr = calloc(1, total_size) as *mut i8;`; FIO_createFilename_fromOutDir_internal(): `if buf_ptr.is_null() { ...; exit(30); }` indicates non-returning failure path; FIO_createFilename_fromOutDir_internal(): returns `buf_ptr as *const i8` with no accompanying free function/token; Comment: "Preserve original allocation sizing behavior." and use of C allocation APIs implies C-side ownership expectations

**Implementation:** Return an owning RAII type on the Rust side, e.g. `struct CStringOwned(*mut c_char); impl Drop for CStringOwned { fn drop(&mut self){ unsafe{ free(self.0 as *mut c_void) }}}` and expose a paired `extern "C" fn ..._free(ptr: *mut c_char)` for FFI consumers. If the C ABI must remain `*const i8`, provide a separate safe Rust wrapper `fn create_filename(...) -> Result<CStringOwned, AllocError>` and keep the `exit(30)` behavior only in the C shim.

---

## Precondition Invariants

### 1. C-string pointer validity protocol (null-terminated within bounded region)

**Location**: `/data/test_case/lib.rs:1-165`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: These functions treat raw `*const i8` inputs as C strings and assume they are either null or point to readable memory that contains a NUL terminator within a fixed maximum (1024 bytes at the FFI boundary; later 100000 bytes for the extracted filename slice). This is enforced only by runtime null checks and bounded scanning; the type system does not express (1) non-nullness, (2) NUL-termination, or (3) the relationship between the pointer and the maximum readable length. As written, a non-null pointer that is not valid for reads up to the bound is UB, and a non-null pointer without a NUL within the bound is treated as a truncated string. The API could instead require/accept validated C strings (or explicit lengths) to make these preconditions explicit and avoid creating arbitrary slices from raw pointers.

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
use std::ffi::CStr;

extern "C" {
    fn calloc(__nmemb: usize, __size: usize) -> *mut c_void;
    fn exit(__status: i32) -> !;
    fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
    fn strrchr(__s: *const i8, __c: i32) -> *mut i8;
    fn strerror(__errnum: i32) -> *mut i8;

    // On macOS, the symbol is __error (not __errno_location).
    // Keep this internal and portable by selecting the right one per target.
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd"))]
    fn __error() -> *mut i32;

    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    fn __errno_location() -> *mut i32;
}

#[inline]
unsafe fn errno_ptr() -> *mut i32 {
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd"))]
    {
        __error()
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    {
        __errno_location()
    }
}

#[inline]
unsafe fn c_strlen_bounded(ptr: *const i8, max: usize) -> usize {
    if ptr.is_null() || max == 0 {
        return 0;
    }
    let bytes = std::slice::from_raw_parts(ptr as *const u8, max);
    bytes.iter().position(|&b| b == 0).unwrap_or(max)
}

pub(crate) unsafe fn extractFilename(path: &[i8], separator: i8) -> *const i8 {
    let last = strrchr(path.as_ptr(), separator as i32);
    if last.is_null() {
        path.as_ptr()
    } else {
        last.add(1)
    }
}

pub(crate) unsafe fn FIO_createFilename_fromOutDir_internal(
    path: &[i8],
    outDirName: &[i8],
    suffixLen: usize,
) -> *const i8 {
    let separator: i8 = b'/' as i8;

    let filename_start_ptr = extractFilename(path, separator);
    let filenameStart: &[i8] = if filename_start_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(filename_start_ptr, 100000)
    };

    let out_len = if outDirName.is_empty() {
        0
    } else {
        c_strlen_bounded(outDirName.as_ptr(), outDirName.len())
    };
    let file_len = if filenameStart.is_empty() {
        0
    } else {
        c_strlen_bounded(filenameStart.as_ptr(), filenameStart.len())
    };

    // Preserve original allocation sizing behavior.
    let total_size = out_len
        .wrapping_add(1)
        .wrapping_add(file_len)
        .wrapping_add(suffixLen)
        .wrapping_add(1);

    let buf_ptr = calloc(1, total_size) as *mut i8;
    if buf_ptr.is_null() {
        eprint!(
            "zstd: FIO_createFilename_fromOutDir: {0}",
            CStr::from_ptr(strerror(*errno_ptr()) as _)
                .to_str()
                .unwrap()
        );
        exit(30);
    }

    if out_len != 0 {
        memcpy(
            buf_ptr as *mut c_void,
            outDirName.as_ptr() as *const c_void,
            out_len,
        );
    }

    // Match original logic: if outDirName doesn't end with '/', insert it.
    // If outDirName is empty, insert '/'.
    let needs_sep = if out_len == 0 {
        true
    } else {
        *outDirName.as_ptr().add(out_len - 1) != separator
    };

    let mut write_pos = out_len;
    if needs_sep {
        *buf_ptr.add(write_pos) = separator;
        write_pos += 1;
    }

    if file_len != 0 {
        memcpy(
            buf_ptr.add(write_pos) as *mut c_void,
            filenameStart.as_ptr() as *const c_void,
            file_len,
        );
    }

    buf_ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn FIO_createFilename_fromOutDir(
    path: *const i8,
    outDirName: *const i8,
    suffixLen: usize,
) -> *const i8 {
    FIO_createFilename_fromOutDir_internal(
        if path.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(path, 1024)
        },
        if outDirName.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(outDirName, 1024)
        },
        suffixLen,
    )
}
```

**Entity:** FIO_createFilename_fromOutDir_internal / FIO_createFilename_fromOutDir (C ABI)

**States:** Null pointer treated as empty string, Non-null pointer to a NUL-terminated C string within an assumed max bound, Non-null pointer not NUL-terminated within bound (logic silently truncates)

**Transitions:**
- Null pointer -> treated as empty slice via FIO_createFilename_fromOutDir()
- Non-null pointer -> assumed-valid bounded C string via from_raw_parts(..., 1024) + c_strlen_bounded()
- Extracted filename pointer -> assumed-valid bounded C string via from_raw_parts(..., 100000) + c_strlen_bounded()

**Evidence:** FIO_createFilename_fromOutDir(): `if path.is_null() { &[] } else { std::slice::from_raw_parts(path, 1024) }`; FIO_createFilename_fromOutDir(): same pattern for `outDirName` with `from_raw_parts(outDirName, 1024)`; extractFilename(): uses `strrchr(path.as_ptr(), separator as i32)` and returns either `path.as_ptr()` or `last.add(1)` (pointer arithmetic assumes valid C string memory); FIO_createFilename_fromOutDir_internal(): `std::slice::from_raw_parts(filename_start_ptr, 100000)` creates an arbitrary-length slice from a raw pointer; c_strlen_bounded(): scans for NUL with `from_raw_parts(ptr as *const u8, max)`; this assumes `ptr..ptr+max` is readable when ptr is non-null

**Implementation:** Introduce a wrapper like `struct BoundedCStr<'a> { bytes: &'a [u8] }` constructed via a single unsafe validator that checks (a) non-null, (b) readable up to `max`, and (c) finds NUL position; or accept `&CStr`/`Option<&CStr>` at the Rust boundary (and provide a separate `extern "C"` shim that immediately converts `*const c_char` to `Option<&CStr>` using `CStr::from_ptr` only after checking a max length). Alternatively, change the FFI to accept `(ptr, len)` and avoid implicit scanning.

---

