# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 3. Allocated C string ownership/lifetime protocol (must be freed, non-null indicates allocation)

**Location**: `/data/test_case/lib.rs:1-46`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: A non-null return value implies ownership of newly allocated memory from `malloc(len)` and requires a corresponding deallocation by the caller (typically `free`). A null return implies no allocation occurred (or failure/invalid input). This ownership protocol is implicit and not represented in the return type (`*const i8`), so Rust callers can easily leak or mis-free, and nothing ties the pointer to the allocator used.

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
}

pub const NULL: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;

pub(crate) unsafe fn custom_strdup_internal(mut str: &[i8]) -> *const i8 {
    if str.is_empty() {
        return core::ptr::null();
    }

    // Find NUL terminator within provided slice.
    let nul_pos = match str.iter().position(|&b| b == 0) {
        Some(p) => p,
        None => return core::ptr::null(),
    };
    let len = nul_pos + 1;

    let ptr = malloc(len) as *mut i8;
    if ptr.is_null() {
        return core::ptr::null();
    }

    core::ptr::copy_nonoverlapping(str.as_ptr(), ptr, len);
    ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn custom_strdup(mut str: *const i8) -> *const i8 {
    custom_strdup_internal(if str.is_null() {
        &[]
    } else {
        // Preserve original behavior: only scan up to 1024 bytes.
        std::slice::from_raw_parts(str, 1024)
    })
}
```

**Entity:** Returned pointer from custom_strdup_internal/custom_strdup

**States:** NullReturnNoAllocation, NonNullReturnOwnedHeapCString

**Transitions:**
- NullReturnNoAllocation -> (no-op) via early returns or malloc failure
- NonNullReturnOwnedHeapCString -> (must be freed) via external free() by caller

**Evidence:** custom_strdup_internal: `let ptr = malloc(len) as *mut i8;` allocates heap memory; custom_strdup_internal: `if ptr.is_null() { return core::ptr::null(); }` distinguishes allocation failure vs success; custom_strdup_internal returns `ptr as *const i8` with no accompanying free/Drop mechanism

**Implementation:** For Rust-side APIs, return an owning RAII type (e.g., `struct MallocCString(*mut c_char); impl Drop for MallocCString { free(self.0) }`) or return `Option<NonNull<c_char>>` plus a dedicated `unsafe fn free_malloced(ptr: NonNull<c_char>)`. For the extern "C" boundary keep `*const i8`, but offer a parallel safe Rust API that returns the RAII wrapper.

---

## Precondition Invariants

### 1. C-string slice validity preconditions (NUL-terminated within bounds, non-empty)

**Location**: `/data/test_case/lib.rs:1-46`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: custom_strdup_internal relies on an implicit protocol for its input slice: the slice must contain a NUL terminator (0) somewhere within the provided bounds, and callers typically intend it to represent a C string. If the slice is empty or lacks a NUL terminator, the function returns null. If malloc fails, it returns null. These states (invalid vs valid C-string slice) are enforced via runtime scanning/branching rather than by accepting a type that guarantees NUL-termination (and possibly non-emptiness).

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
}

pub const NULL: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;

pub(crate) unsafe fn custom_strdup_internal(mut str: &[i8]) -> *const i8 {
    if str.is_empty() {
        return core::ptr::null();
    }

    // Find NUL terminator within provided slice.
    let nul_pos = match str.iter().position(|&b| b == 0) {
        Some(p) => p,
        None => return core::ptr::null(),
    };
    let len = nul_pos + 1;

    let ptr = malloc(len) as *mut i8;
    if ptr.is_null() {
        return core::ptr::null();
    }

    core::ptr::copy_nonoverlapping(str.as_ptr(), ptr, len);
    ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn custom_strdup(mut str: *const i8) -> *const i8 {
    custom_strdup_internal(if str.is_null() {
        &[]
    } else {
        // Preserve original behavior: only scan up to 1024 bytes.
        std::slice::from_raw_parts(str, 1024)
    })
}
```

**Entity:** custom_strdup_internal (unsafe fn)

**States:** InvalidInput, ValidCStrSlice, AllocatedAndCopied

**Transitions:**
- InvalidInput -> (return null) via early returns on str.is_empty() / no NUL found
- ValidCStrSlice -> AllocatedAndCopied via malloc(len) + copy_nonoverlapping

**Evidence:** custom_strdup_internal: `if str.is_empty() { return core::ptr::null(); }`; custom_strdup_internal: `match str.iter().position(|&b| b == 0) { Some(p) => p, None => return core::ptr::null(), }` enforces 'NUL must exist within slice'; custom_strdup_internal: `let len = nul_pos + 1;` relies on NUL position to define copy length; custom_strdup_internal: `let ptr = malloc(len) as *mut i8; if ptr.is_null() { return core::ptr::null(); }` encodes allocation-success state; custom_strdup_internal: `core::ptr::copy_nonoverlapping(str.as_ptr(), ptr, len);` assumes `len` bytes are valid to read from `str`

**Implementation:** Introduce an input type that carries the invariant, e.g. accept `&core::ffi::CStr` (or a crate-local `NulTerminatedSlice<'a>(&'a [i8])` validated constructor). Then `custom_strdup_internal` no longer needs to scan/branch for NUL (except perhaps to compute length), and invalid inputs become unrepresentable at the call site.

---

## Protocol Invariants

### 2. Raw pointer / bounded-scan protocol (NULL vs non-NULL, readable for 1024 bytes, NUL within 1024)

**Location**: `/data/test_case/lib.rs:1-46`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: custom_strdup encodes a C-FFI protocol: if the input pointer is NULL it is treated as an empty slice and results in null. If non-NULL, it unsafely creates a slice of length 1024 and searches for a NUL terminator; this implicitly requires that the pointer be valid to read for 1024 bytes and that a NUL occurs within those 1024 bytes, otherwise it returns null (or may UB if the 1024-byte read is invalid). This is a temporal/usage protocol on the caller: provide either NULL or a pointer to at least 1024 readable bytes with a terminator before that bound. The type system does not express any of these constraints.

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
}

pub const NULL: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;

pub(crate) unsafe fn custom_strdup_internal(mut str: &[i8]) -> *const i8 {
    if str.is_empty() {
        return core::ptr::null();
    }

    // Find NUL terminator within provided slice.
    let nul_pos = match str.iter().position(|&b| b == 0) {
        Some(p) => p,
        None => return core::ptr::null(),
    };
    let len = nul_pos + 1;

    let ptr = malloc(len) as *mut i8;
    if ptr.is_null() {
        return core::ptr::null();
    }

    core::ptr::copy_nonoverlapping(str.as_ptr(), ptr, len);
    ptr as *const i8
}

#[no_mangle]
pub unsafe extern "C" fn custom_strdup(mut str: *const i8) -> *const i8 {
    custom_strdup_internal(if str.is_null() {
        &[]
    } else {
        // Preserve original behavior: only scan up to 1024 bytes.
        std::slice::from_raw_parts(str, 1024)
    })
}
```

**Entity:** custom_strdup (unsafe extern "C" fn)

**States:** NullPtrInput, NonNullPtrButInvalidMemory, NonNullPtrNoNulIn1024, NonNullPtrValidCStrWithin1024

**Transitions:**
- NullPtrInput -> (delegates empty slice) via `if str.is_null() { &[] }`
- NonNullPtrValidCStrWithin1024 -> (delegates to allocation/copy) via `from_raw_parts(str, 1024)` then `custom_strdup_internal(...)`
- NonNullPtrNoNulIn1024 -> (return null) via `custom_strdup_internal` returning null when no NUL is found
- NonNullPtrButInvalidMemory -> (UB risk) via `std::slice::from_raw_parts(str, 1024)` requiring 1024 readable bytes

**Evidence:** custom_strdup: `if str.is_null() { &[] } else { std::slice::from_raw_parts(str, 1024) }` encodes NULL vs non-NULL behavior and a fixed readable bound; comment in custom_strdup: `// Preserve original behavior: only scan up to 1024 bytes.` documents the bounded-scan protocol; custom_strdup delegates to `custom_strdup_internal(...)`, which returns null if no NUL is found in the provided slice

**Implementation:** For Rust callers, provide a safe wrapper that accepts `Option<&CStr>` (or `*const c_char` validated into `Option<&CStr>` via `CStr::from_ptr` in an unsafe constructor) and removes the `from_raw_parts(..., 1024)` requirement. If the 1024-byte cap must remain, expose a `BoundedCStr<'a, const N: usize>` newtype created by an unsafe validator that checks readability/terminator within N, and have the internal logic accept that type.

---

