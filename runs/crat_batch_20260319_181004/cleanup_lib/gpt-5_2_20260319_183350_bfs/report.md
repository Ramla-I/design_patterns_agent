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

### 1. C-heap buffer lifecycle (Unallocated/Null -> Allocated -> Freed)

**Location**: `/data/test_case/lib.rs:1-89`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The code encodes a manual ownership protocol for a C-heap allocation: the pointer starts null, may transition to an allocated buffer via malloc, and must be freed exactly once via cleanup_resources. Correctness relies on runtime null checks and call ordering; the type system does not prevent forgetting to free, double-freeing, or using the pointer after it has been freed (UAF).

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
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn strncmp(__s1: *const i8, __s2: *const i8, __n: usize) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let numbers = [a, b, c, d];
    let mut result: i32 = 0;

    // Keep the original FFI validation behavior.
    let expected_str: &[i8] = bytemuck::cast_slice(b"VALID\0");
    let input_str: &[i8] = bytemuck::cast_slice(b"VALID\0");
    let cmp_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(expected_str))
        .unwrap()
        .count_bytes();

    // We'll keep a raw pointer for the allocation so we can free it correctly.
    let mut dynamic_ptr: *mut i8 = std::ptr::null_mut();

    if strncmp(input_str.as_ptr(), expected_str.as_ptr(), cmp_len) != 0 {
        println!("Input string validation failed.");
    } else {
        for &n in &numbers {
            match n {
                10 => {
                    result += 10;
                    result += 20;
                }
                20 => {
                    result += 20;
                }
                30 => {
                    result += 30;
                    result += 40;
                }
                40 => {
                    result += 40;
                }
                _ => {
                    result += n;
                }
            }
        }

        dynamic_ptr = malloc(50 * core::mem::size_of::<i8>()) as *mut i8;
        if dynamic_ptr.is_null() {
            println!("Memory allocation failed.");
        } else {
            // Use a properly-sized slice for the allocated buffer.
            let dynamic_buf: &mut [i8] = std::slice::from_raw_parts_mut(dynamic_ptr, 50);

            snprintf(
                dynamic_buf.as_mut_ptr(),
                50,
                b"Processed numbers: %s\0" as *const u8 as *const i8,
                b"numbers\0" as *const u8 as *const i8,
            );

            println!(
                "{0}",
                std::ffi::CStr::from_ptr(dynamic_buf.as_ptr())
                    .to_str()
                    .unwrap()
            );
        }
    }

    cleanup_resources(dynamic_ptr);
    result
}

pub(crate) unsafe fn cleanup_resources(dynamic_str: *mut i8) {
    if !dynamic_str.is_null() {
        free(dynamic_str as *mut core::ffi::c_void);
    }
}
```

**Entity:** dynamic_ptr (allocated C buffer passed across cleanup/cleanup_resources)

**States:** Null (not allocated), Allocated (malloc-owned), Freed (must not be reused)

**Transitions:**
- Null (not allocated) -> Allocated (malloc-owned) via malloc(...) in cleanup()
- Allocated (malloc-owned) -> Freed via cleanup_resources(dynamic_ptr) calling free(...)
- Null (not allocated) -> Null (not allocated) via cleanup_resources(dynamic_ptr) no-op on null

**Evidence:** in cleanup(): `let mut dynamic_ptr: *mut i8 = std::ptr::null_mut();` encodes 'unallocated' state as null; in cleanup(): `dynamic_ptr = malloc(50 * core::mem::size_of::<i8>()) as *mut i8;` allocates and transitions to 'allocated'; in cleanup(): `if dynamic_ptr.is_null() { println!("Memory allocation failed."); } else { ... from_raw_parts_mut(dynamic_ptr, 50) ... }` gates use on runtime null check; in cleanup(): `cleanup_resources(dynamic_ptr);` relies on caller to always perform cleanup after allocation attempt; in cleanup_resources(): `if !dynamic_str.is_null() { free(dynamic_str as *mut core::ffi::c_void); }` frees based on runtime null check (no compile-time ownership)

**Implementation:** Introduce an owning wrapper for the allocation, e.g. `struct CBuf(*mut i8); impl Drop for CBuf { fn drop(&mut self){ if !self.0.is_null(){ unsafe{ free(self.0.cast()) }}}}`; construct it from `malloc` returning `Option<CBuf>` or `Result<CBuf, AllocError>`, and expose safe accessors like `as_mut_slice(&mut self, len)` so `from_raw_parts_mut` is only callable when allocated.

---

## Precondition Invariants

### 2. FFI pointer/string validity preconditions (CStr/format buffer protocol)

**Location**: `/data/test_case/lib.rs:1-89`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Several operations assume C FFI invariants that are not represented in the type system: (1) `snprintf` requires a writable, sufficiently-sized, valid pointer and a NUL-terminated format string; (2) `CStr::from_ptr(dynamic_buf.as_ptr())` assumes the produced buffer is NUL-terminated; (3) `to_str().unwrap()` assumes valid UTF-8. These are enforced only by runtime checks/unwraps and by relying on C library behavior, not by Rust types.

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
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn strncmp(__s1: *const i8, __s2: *const i8, __n: usize) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let numbers = [a, b, c, d];
    let mut result: i32 = 0;

    // Keep the original FFI validation behavior.
    let expected_str: &[i8] = bytemuck::cast_slice(b"VALID\0");
    let input_str: &[i8] = bytemuck::cast_slice(b"VALID\0");
    let cmp_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(expected_str))
        .unwrap()
        .count_bytes();

    // We'll keep a raw pointer for the allocation so we can free it correctly.
    let mut dynamic_ptr: *mut i8 = std::ptr::null_mut();

    if strncmp(input_str.as_ptr(), expected_str.as_ptr(), cmp_len) != 0 {
        println!("Input string validation failed.");
    } else {
        for &n in &numbers {
            match n {
                10 => {
                    result += 10;
                    result += 20;
                }
                20 => {
                    result += 20;
                }
                30 => {
                    result += 30;
                    result += 40;
                }
                40 => {
                    result += 40;
                }
                _ => {
                    result += n;
                }
            }
        }

        dynamic_ptr = malloc(50 * core::mem::size_of::<i8>()) as *mut i8;
        if dynamic_ptr.is_null() {
            println!("Memory allocation failed.");
        } else {
            // Use a properly-sized slice for the allocated buffer.
            let dynamic_buf: &mut [i8] = std::slice::from_raw_parts_mut(dynamic_ptr, 50);

            snprintf(
                dynamic_buf.as_mut_ptr(),
                50,
                b"Processed numbers: %s\0" as *const u8 as *const i8,
                b"numbers\0" as *const u8 as *const i8,
            );

            println!(
                "{0}",
                std::ffi::CStr::from_ptr(dynamic_buf.as_ptr())
                    .to_str()
                    .unwrap()
            );
        }
    }

    cleanup_resources(dynamic_ptr);
    result
}

pub(crate) unsafe fn cleanup_resources(dynamic_str: *mut i8) {
    if !dynamic_str.is_null() {
        free(dynamic_str as *mut core::ffi::c_void);
    }
}
```

**Entity:** cleanup (FFI entrypoint)

**States:** Inputs satisfy C/CStr invariants, Inputs violate C/CStr invariants (UB/panic risk)

**Transitions:**
- Inputs satisfy C/CStr invariants -> safe printing path via snprintf(...) then CStr::from_ptr(...).to_str().unwrap()
- Inputs violate C/CStr invariants -> panic/UB via from_ptr/to_str unwrap assumptions

**Evidence:** in cleanup(): `snprintf(dynamic_buf.as_mut_ptr(), 50, b"Processed numbers: %s\0" as *const u8 as *const i8, ...)` relies on a correct writable buffer and NUL-terminated format string; in cleanup(): `std::ffi::CStr::from_ptr(dynamic_buf.as_ptr())` assumes the buffer is NUL-terminated by snprintf; in cleanup(): `.to_str().unwrap()` encodes a UTF-8 validity precondition as a panic-on-violation check; in cleanup(): `std::slice::from_raw_parts_mut(dynamic_ptr, 50)` assumes `dynamic_ptr` is non-null and points to at least 50 bytes (guarded only by `if dynamic_ptr.is_null()`)

**Implementation:** Wrap C strings/buffers in validated types: e.g., use `CString` for format/argument strings instead of raw `*const i8`, and represent the output as `CStr`/`CString` only after checking `snprintf`'s return and ensuring NUL termination. A small newtype like `struct NulTerminatedBuf<const N: usize>([u8; N]);` (or an RAII `CBuf` that provides `as_mut_c_char_ptr()` plus a checked `as_cstr()` method) can centralize the invariants and remove `from_ptr(...).to_str().unwrap()` from the main logic.

---

