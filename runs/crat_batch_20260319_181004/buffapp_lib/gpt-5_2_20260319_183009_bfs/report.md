# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. StringBuffer allocation/validity + lifecycle protocol (Null/Allocated/Freed)

**Location**: `/data/test_case/lib.rs:1-201`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The buffer is managed manually through malloc/realloc/free and raw pointers. Callers must (1) check allocation success (non-null), (2) only call append_to_buffer/destroy_buffer on an allocated buffer, (3) call destroy_buffer exactly once to free both the internal data pointer and the StringBuffer header, and (4) never use the buffer again after destroy_buffer (UAF). None of this is expressed in the type system: create_buffer returns *mut StringBuffer (nullable), append/destroy accept Option-wrapped references and use -1/no-op to signal invalid state, and buffapp uses unwrap() assuming allocation succeeded.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StringBuffer, 3 free function(s)

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
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn realloc(__ptr: *mut core::ffi::c_void, __size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
    fn strcmp(__s1: *const i8, __s2: *const i8) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StringBuffer {
    pub data: *mut i8,
    pub capacity: i32,
    pub length: i32,
}

pub(crate) unsafe fn create_buffer(initial_capacity: i32) -> *mut StringBuffer {
    if initial_capacity <= 0 {
        return core::ptr::null_mut();
    }

    let buf_ptr = malloc(core::mem::size_of::<StringBuffer>()) as *mut StringBuffer;
    let Some(buf) = buf_ptr.as_mut() else {
        return core::ptr::null_mut();
    };

    let data_ptr = malloc(initial_capacity as usize) as *mut i8;
    if data_ptr.is_null() {
        free(buf_ptr as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    buf.data = data_ptr;
    buf.capacity = initial_capacity;
    buf.length = 0;
    *buf.data = 0;

    buf_ptr
}

pub(crate) unsafe fn append_to_buffer(mut buffer: Option<&mut StringBuffer>, str: &[i8]) -> i32 {
    let Some(buf) = buffer.as_deref_mut() else {
        return -1;
    };

    // Determine C-string length (up to NUL). If no NUL, treat as full slice.
    let str_len: i32 = match str.iter().position(|&c| c == 0) {
        Some(n) => n as i32,
        None => str.len() as i32,
    };

    let required_capacity: i32 = buf.length.saturating_add(str_len).saturating_add(1);
    if required_capacity > buf.capacity {
        let new_capacity: i32 = required_capacity.saturating_mul(2);
        let new_ptr =
            realloc(buf.data as *mut core::ffi::c_void, new_capacity as usize) as *mut i8;
        if new_ptr.is_null() {
            return -1;
        }
        buf.data = new_ptr;
        buf.capacity = new_capacity;
    }

    // Copy including terminating NUL from `str` (callers provide NUL-terminated buffers).
    strcpy(buf.data.add(buf.length as usize), str.as_ptr());
    buf.length += str_len;
    0
}

pub(crate) unsafe fn destroy_buffer(buffer: Option<&StringBuffer>) {
    let Some(buf) = buffer else { return };

    if !buf.data.is_null() {
        free(buf.data as *mut core::ffi::c_void);
    }
    free(buf as *const _ as *mut core::ffi::c_void);
}

pub(crate) fn get_operation_name(op_code: i32) -> *const i8 {
    match op_code {
        0 => b"add\0".as_ptr() as *const i8,
        1 => b"subtract\0".as_ptr() as *const i8,
        2 => b"multiply\0".as_ptr() as *const i8,
        3 => b"divide\0".as_ptr() as *const i8,
        _ => b"unknown\0".as_ptr() as *const i8,
    }
}

pub(crate) unsafe fn perform_operation(a: i32, b: i32, operation: &[i8]) -> i32 {
    if strcmp(operation.as_ptr(), b"add\0".as_ptr() as *const i8) == 0 {
        a + b
    } else if strcmp(operation.as_ptr(), b"subtract\0".as_ptr() as *const i8) == 0 {
        a - b
    } else if strcmp(operation.as_ptr(), b"multiply\0".as_ptr() as *const i8) == 0 {
        a * b
    } else if strcmp(operation.as_ptr(), b"divide\0".as_ptr() as *const i8) == 0 {
        if b != 0 { a / b } else { 0 }
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn buffapp(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut log_buffer: Option<&mut StringBuffer> = create_buffer(32).as_mut();
    let mut result: i32 = 0;
    let mut temp: [i8; 64] = [0; 64];

    log_buffer.as_deref_mut().unwrap().length = 0;

    sprintf(
        temp.as_mut_ptr(),
        b"Starting computation with %d parameters\n\0".as_ptr() as *const i8,
        4,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let op1_ptr = get_operation_name(param1 % 4);
    let op1: &[i8] = if op1_ptr.is_null() {
        &[]
    } else {
        // Convert CStr bytes (&[u8]) to &[i8] without allocation.
        bytemuck::cast_slice(core::ffi::CStr::from_ptr(op1_ptr).to_bytes_with_nul())
    };
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 1: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op1.as_ptr(),
        param1,
        param2,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate1: i32 = perform_operation(param1, param2, op1);
    result += intermediate1;

    let op2_ptr = get_operation_name(param3 % 4);
    let op2: &[i8] = if op2_ptr.is_null() {
        &[]
    } else {
        bytemuck::cast_slice(core::ffi::CStr::from_ptr(op2_ptr).to_bytes_with_nul())
    };
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 2: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op2.as_ptr(),
        param3,
        param4,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate2: i32 = perform_operation(param3, param4, op2);
    result += intermediate2;

    let op3: &[i8] = bytemuck::cast_slice(b"multiply\0");
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 3: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op3.as_ptr(),
        intermediate1,
        intermediate2,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate3: i32 = perform_operation(intermediate1, intermediate2, op3);
    if intermediate3 != 0 {
        result /= intermediate3;
    } else {
        result = param1 + param2 + param3 + param4;
    }

    sprintf(
        temp.as_mut_ptr(),
        b"Final result: %d\n\0".as_ptr() as *const i8,
        result,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    print!(
        "Computation Log:\n{0}\n",
        core::ffi::CStr::from_ptr(log_buffer.as_deref().unwrap().data as _)
            .to_str()
            .unwrap()
    );

    destroy_buffer(log_buffer.as_deref());
    result
}
```

**Entity:** StringBuffer (and the raw pointer returned by create_buffer)

**States:** Null (allocation failed), Allocated (owns heap data + header), Freed (must not be used)

**Transitions:**
- Null -> (no valid transitions; all operations invalid)
- Allocated -> Allocated via append_to_buffer() (may realloc)
- Allocated -> Freed via destroy_buffer()

**Evidence:** create_buffer(initial_capacity) returns core::ptr::null_mut() when initial_capacity <= 0 or malloc fails (nullable pointer encodes 'Null' state); create_buffer: allocates buf_ptr via malloc(size_of::<StringBuffer>()) and data_ptr via malloc(initial_capacity as usize); on data_ptr failure frees buf_ptr (manual resource management); append_to_buffer(buffer: Option<&mut StringBuffer>) returns -1 when buffer is None (runtime validity check for 'not allocated'/'missing buffer'); append_to_buffer: calls realloc(buf.data, new_capacity) and updates buf.data/buf.capacity (requires buf.data to be a valid allocated pointer); destroy_buffer(buffer: Option<&StringBuffer>) early-returns on None, and frees buf.data then frees buf itself (two-level free; requires exactly-once destruction); buffapp: create_buffer(32).as_mut() stored in Option<&mut StringBuffer> and then log_buffer.as_deref_mut().unwrap().length = 0 (unwrap relies on non-null allocation at runtime); buffapp: destroy_buffer(log_buffer.as_deref()) called at end (manual 'must call' cleanup requirement)

**Implementation:** Introduce an owning safe wrapper, e.g. `struct OwnedStringBuffer { ptr: NonNull<StringBuffer> }` with `impl Drop` calling the current destroy logic. Make `create_buffer` return `Result<OwnedStringBuffer, AllocError>` (or `Option<OwnedStringBuffer>`), and expose `append(&mut self, cstr: &CStr) -> Result<(), Error>` so callers cannot forget to free, cannot double-free, and cannot unwrap a null allocation.

---

## Precondition Invariants

### 2. C-string argument validity invariant (NUL-terminated, non-null pointer, no interior truncation surprises)

**Location**: `/data/test_case/lib.rs:1-201`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Several functions take `&[i8]` but then treat it as a C string. This creates a latent precondition: the slice must represent a valid NUL-terminated C string and its pointer must be safe to pass to C (`strcmp`, `strcpy`, `sprintf %s`). The code partially compensates by searching for a NUL to compute `str_len`, but still unconditionally calls `strcpy(..., str.as_ptr())`, which requires a terminating NUL in memory. In buffapp, when `op*_ptr` is null it uses `&[]` and then passes `op1.as_ptr()` to `sprintf` with `%s` and to `perform_operation`/`strcmp`, which is an invalid C-string pointer. These requirements are not encoded in the types; using `&CStr`/`&CString` would enforce them at compile time for Rust callers.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StringBuffer, 3 free function(s)

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
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn realloc(__ptr: *mut core::ffi::c_void, __size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
    fn strcmp(__s1: *const i8, __s2: *const i8) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StringBuffer {
    pub data: *mut i8,
    pub capacity: i32,
    pub length: i32,
}

pub(crate) unsafe fn create_buffer(initial_capacity: i32) -> *mut StringBuffer {
    if initial_capacity <= 0 {
        return core::ptr::null_mut();
    }

    let buf_ptr = malloc(core::mem::size_of::<StringBuffer>()) as *mut StringBuffer;
    let Some(buf) = buf_ptr.as_mut() else {
        return core::ptr::null_mut();
    };

    let data_ptr = malloc(initial_capacity as usize) as *mut i8;
    if data_ptr.is_null() {
        free(buf_ptr as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    buf.data = data_ptr;
    buf.capacity = initial_capacity;
    buf.length = 0;
    *buf.data = 0;

    buf_ptr
}

pub(crate) unsafe fn append_to_buffer(mut buffer: Option<&mut StringBuffer>, str: &[i8]) -> i32 {
    let Some(buf) = buffer.as_deref_mut() else {
        return -1;
    };

    // Determine C-string length (up to NUL). If no NUL, treat as full slice.
    let str_len: i32 = match str.iter().position(|&c| c == 0) {
        Some(n) => n as i32,
        None => str.len() as i32,
    };

    let required_capacity: i32 = buf.length.saturating_add(str_len).saturating_add(1);
    if required_capacity > buf.capacity {
        let new_capacity: i32 = required_capacity.saturating_mul(2);
        let new_ptr =
            realloc(buf.data as *mut core::ffi::c_void, new_capacity as usize) as *mut i8;
        if new_ptr.is_null() {
            return -1;
        }
        buf.data = new_ptr;
        buf.capacity = new_capacity;
    }

    // Copy including terminating NUL from `str` (callers provide NUL-terminated buffers).
    strcpy(buf.data.add(buf.length as usize), str.as_ptr());
    buf.length += str_len;
    0
}

pub(crate) unsafe fn destroy_buffer(buffer: Option<&StringBuffer>) {
    let Some(buf) = buffer else { return };

    if !buf.data.is_null() {
        free(buf.data as *mut core::ffi::c_void);
    }
    free(buf as *const _ as *mut core::ffi::c_void);
}

pub(crate) fn get_operation_name(op_code: i32) -> *const i8 {
    match op_code {
        0 => b"add\0".as_ptr() as *const i8,
        1 => b"subtract\0".as_ptr() as *const i8,
        2 => b"multiply\0".as_ptr() as *const i8,
        3 => b"divide\0".as_ptr() as *const i8,
        _ => b"unknown\0".as_ptr() as *const i8,
    }
}

pub(crate) unsafe fn perform_operation(a: i32, b: i32, operation: &[i8]) -> i32 {
    if strcmp(operation.as_ptr(), b"add\0".as_ptr() as *const i8) == 0 {
        a + b
    } else if strcmp(operation.as_ptr(), b"subtract\0".as_ptr() as *const i8) == 0 {
        a - b
    } else if strcmp(operation.as_ptr(), b"multiply\0".as_ptr() as *const i8) == 0 {
        a * b
    } else if strcmp(operation.as_ptr(), b"divide\0".as_ptr() as *const i8) == 0 {
        if b != 0 { a / b } else { 0 }
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn buffapp(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut log_buffer: Option<&mut StringBuffer> = create_buffer(32).as_mut();
    let mut result: i32 = 0;
    let mut temp: [i8; 64] = [0; 64];

    log_buffer.as_deref_mut().unwrap().length = 0;

    sprintf(
        temp.as_mut_ptr(),
        b"Starting computation with %d parameters\n\0".as_ptr() as *const i8,
        4,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let op1_ptr = get_operation_name(param1 % 4);
    let op1: &[i8] = if op1_ptr.is_null() {
        &[]
    } else {
        // Convert CStr bytes (&[u8]) to &[i8] without allocation.
        bytemuck::cast_slice(core::ffi::CStr::from_ptr(op1_ptr).to_bytes_with_nul())
    };
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 1: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op1.as_ptr(),
        param1,
        param2,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate1: i32 = perform_operation(param1, param2, op1);
    result += intermediate1;

    let op2_ptr = get_operation_name(param3 % 4);
    let op2: &[i8] = if op2_ptr.is_null() {
        &[]
    } else {
        bytemuck::cast_slice(core::ffi::CStr::from_ptr(op2_ptr).to_bytes_with_nul())
    };
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 2: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op2.as_ptr(),
        param3,
        param4,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate2: i32 = perform_operation(param3, param4, op2);
    result += intermediate2;

    let op3: &[i8] = bytemuck::cast_slice(b"multiply\0");
    sprintf(
        temp.as_mut_ptr(),
        b"Operation 3: %s(%d, %d)\n\0".as_ptr() as *const i8,
        op3.as_ptr(),
        intermediate1,
        intermediate2,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    let intermediate3: i32 = perform_operation(intermediate1, intermediate2, op3);
    if intermediate3 != 0 {
        result /= intermediate3;
    } else {
        result = param1 + param2 + param3 + param4;
    }

    sprintf(
        temp.as_mut_ptr(),
        b"Final result: %d\n\0".as_ptr() as *const i8,
        result,
    );
    append_to_buffer(log_buffer.as_deref_mut(), &temp);

    print!(
        "Computation Log:\n{0}\n",
        core::ffi::CStr::from_ptr(log_buffer.as_deref().unwrap().data as _)
            .to_str()
            .unwrap()
    );

    destroy_buffer(log_buffer.as_deref());
    result
}
```

**Entity:** append_to_buffer / perform_operation (string arguments as &[i8])

**States:** Valid C string (NUL-terminated, pointer usable), Invalid/Non-C string (missing NUL or non-dereferenceable)

**Transitions:**
- Invalid/Non-C string -> Valid C string via constructing/receiving a CStr (e.g., CStr::from_ptr or CString)
- Valid C string -> (consumed by) C APIs strcmp/strcpy/sprintf

**Evidence:** append_to_buffer: comment 'Copy including terminating NUL from `str` (callers provide NUL-terminated buffers).' (comment-based protocol); append_to_buffer: computes str_len by scanning for 0, but still calls `strcpy(buf.data.add(buf.length as usize), str.as_ptr())` (requires NUL termination regardless of scan result); perform_operation(operation: &[i8]) calls `strcmp(operation.as_ptr(), b"add\0".as_ptr() as *const i8)` etc. (requires operation.as_ptr() to be a valid C string pointer); buffapp: `let op1: &[i8] = if op1_ptr.is_null() { &[] } else { ... }` then passes `op1.as_ptr()` into `sprintf(..., b"Operation 1: %s...", op1.as_ptr(), ...)` (%s requires NUL-terminated string, but &[] is not); buffapp: same pattern for op2; op3 uses `bytemuck::cast_slice(b"multiply\0")` (explicitly NUL-terminated, showing the intended invariant)

**Implementation:** Change APIs to accept `&core::ffi::CStr` (or a wrapper `struct CiStr<'a>(&'a CStr)` if you need `[i8]` interop). For `append_to_buffer`, take `&CStr` and use its bytes-with-nul length, avoiding the scan+strcpy mismatch. For `perform_operation`, accept an `Operation` enum or `&CStr` and compare against known `CStr` constants. In buffapp, represent missing operation as `None` and avoid calling `%s`/strcmp when absent (or use a fallback `CStr` like "unknown\0").

---

### 3. StringBuffer raw-pointer validity + length/capacity invariants

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: StringBuffer is a C-ABI buffer descriptor that relies on implicit invariants not enforced by the type system: when the buffer is usable, `data` must be non-null and point to a valid writable allocation sized for `capacity`; `length` must be within bounds (typically 0 <= length <= capacity). Additionally, the struct is `Copy`, so copies alias the same `data` pointer; this implies an implicit protocol that mutation/freeing must be coordinated externally to avoid double-free, use-after-free, and aliased-mutable access. None of these requirements are represented in the type system (raw pointer + i32 metadata).

**Evidence**:

```rust
// Note: Other parts of this module contain: 3 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct StringBuffer {
    pub data: *mut i8,
    pub capacity: i32,
    pub length: i32,
}

```

**Entity:** StringBuffer

**States:** Unallocated/Null, Allocated/Valid

**Transitions:**
- Unallocated/Null -> Allocated/Valid via external allocation/initialization (not shown in snippet)
- Allocated/Valid -> Unallocated/Null via external deallocation/reset (not shown in snippet)

**Evidence:** line 7: `pub data: *mut i8` is a raw nullable pointer with no lifetime/ownership tracking; line 8: `pub capacity: i32` is metadata that must match the allocation size but is unchecked; line 9: `pub length: i32` must be <= capacity but is unchecked; line 5-6: `#[derive(Copy, Clone)]` allows implicit duplication/aliasing of the same `data` pointer

**Implementation:** Replace `StringBuffer` (for internal Rust use) with a safe wrapper that enforces invariants, e.g. `struct OwnedStringBuffer { buf: Vec<i8> }` or `struct BorrowedStringBuffer<'a> { data: NonNull<i8>, capacity: usize, length: usize, _lt: PhantomData<&'a mut [i8]> }`. Use `NonNull<i8>` and `usize` for sizes, enforce `length <= capacity` on construction, and avoid `Copy` for owning variants. Keep the `#[repr(C)]` raw struct only at the FFI boundary with explicit `unsafe` constructors/converters.

---

