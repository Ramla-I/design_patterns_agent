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

### 1. C-string slice validity precondition (NUL-terminated within bounds)

**Location**: `/data/test_case/lib.rs:1-46`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: tool_basename_internal passes path.as_ptr() to the C function strrchr, which requires a valid NUL-terminated C string. The Rust type &[i8] does not encode NUL-termination or a maximum search bound, so correctness relies on the caller providing a slice that contains a terminating '\0' before any invalid memory. If the slice is not NUL-terminated (or points to non-C-string data), strrchr may read past the intended buffer, causing UB. The function also returns a pointer (start) that is only meaningful if it points into the same valid C string; this is likewise not enforced by the signature.

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
    fn strrchr(__s: *const i8, __c: i32) -> *mut i8;
}

pub(crate) unsafe fn tool_basename_internal(mut path: &[i8]) -> *const i8 {
    if path.is_empty() {
        return path.as_ptr();
    }

    let slash = strrchr(path.as_ptr(), b'/' as i32);
    let backslash = strrchr(path.as_ptr(), b'\\' as i32);

    let start = match (slash.is_null(), backslash.is_null()) {
        (true, true) => path.as_ptr(),
        (false, true) => slash.add(1),
        (true, false) => backslash.add(1),
        (false, false) => {
            if slash > backslash {
                slash.add(1)
            } else {
                backslash.add(1)
            }
        }
    };

    start
}

#[no_mangle]
pub unsafe extern "C" fn tool_basename(path: *const i8) -> *const i8 {
    tool_basename_internal(if path.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(path, 1024)
    })
}
```

**Entity:** tool_basename_internal(path: &[i8])

**States:** ValidCStrSlice, InvalidOrUnboundedSlice

**Transitions:**
- InvalidOrUnboundedSlice -> ValidCStrSlice via caller ensuring NUL-termination (not represented in types)

**Evidence:** tool_basename_internal: signature uses path: &[i8] (does not guarantee NUL termination); tool_basename_internal: calls strrchr(path.as_ptr(), ...) twice; tool_basename_internal: pointer arithmetic on results: slash.add(1), backslash.add(1)

**Implementation:** Change tool_basename_internal to accept &std::ffi::CStr (or a custom newtype wrapping *const c_char with a validated length/terminator). For FFI inputs, convert with CStr::from_ptr(path) (unsafe but makes the precondition explicit at the boundary) and then operate on the bytes via cstr.to_bytes_with_nul().

---

## Protocol Invariants

### 2. FFI pointer + lifetime/ownership protocol (NULL vs non-NULL, must outlive result)

**Location**: `/data/test_case/lib.rs:1-46`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: tool_basename is an unsafe extern "C" API that treats NULL as a special case (empty slice) and otherwise assumes the pointer refers to a readable, NUL-terminated C string. It fabricates a Rust slice with std::slice::from_raw_parts(path, 1024), which encodes an implicit protocol: the pointer must be valid for reading 1024 bytes (or at least up to the first NUL that strrchr will find) and must remain alive for as long as the returned pointer is used. The returned *const i8 is a borrowed pointer into the original buffer (path or path+offset), but the type does not express that borrow/lifetime relationship, so users can easily violate it (use-after-free, passing a pointer not valid for 1024 bytes, missing NUL).

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
    fn strrchr(__s: *const i8, __c: i32) -> *mut i8;
}

pub(crate) unsafe fn tool_basename_internal(mut path: &[i8]) -> *const i8 {
    if path.is_empty() {
        return path.as_ptr();
    }

    let slash = strrchr(path.as_ptr(), b'/' as i32);
    let backslash = strrchr(path.as_ptr(), b'\\' as i32);

    let start = match (slash.is_null(), backslash.is_null()) {
        (true, true) => path.as_ptr(),
        (false, true) => slash.add(1),
        (true, false) => backslash.add(1),
        (false, false) => {
            if slash > backslash {
                slash.add(1)
            } else {
                backslash.add(1)
            }
        }
    };

    start
}

#[no_mangle]
pub unsafe extern "C" fn tool_basename(path: *const i8) -> *const i8 {
    tool_basename_internal(if path.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(path, 1024)
    })
}
```

**Entity:** tool_basename(path: *const i8)

**States:** NullPath, NonNullCStrPointer

**Transitions:**
- NullPath -> (returns empty.as_ptr())
- NonNullCStrPointer -> (returns path + offset into same buffer) via tool_basename()

**Evidence:** tool_basename: checks path.is_null() to choose between &[] and from_raw_parts(path, 1024); tool_basename: uses std::slice::from_raw_parts(path, 1024) (hard-coded implicit readable length requirement); tool_basename: returns *const i8 produced by tool_basename_internal (pointer into the input buffer)

**Implementation:** Provide a safe Rust wrapper that takes &CStr (or a validated newtype like ValidCStrPtr) and returns a pointer or &CStr tied to the input lifetime. For the raw FFI entrypoint, keep it unsafe but route through the wrapper after validating with CStr::from_ptr and avoiding from_raw_parts(1024). If a fixed max is required, accept an explicit length parameter from C and use memchr-style bounded search instead of strrchr.

---

