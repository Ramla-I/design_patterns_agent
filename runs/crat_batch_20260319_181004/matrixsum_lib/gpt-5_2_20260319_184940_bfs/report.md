# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. DynamicArray heap-allocation lifecycle + initialization invariant (Allocated/Live -> Freed)

**Location**: `/data/test_case/lib.rs:1-155`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: DynamicArray is managed manually via malloc/realloc/free and raw pointers. Correct usage relies on a temporal protocol: init_array() must succeed before any use; add_element()/expand_array() assume the struct and its data buffer are valid and exclusively mutable; free_array() must be called exactly once at the end, and the array must not be used after freeing. None of this is enforced by the type system because the API traffics in *mut DynamicArray, Option<&mut DynamicArray>, and Option<&DynamicArray>, and free_array() can be called on borrowed references without consuming an owning handle (allowing double-free/use-after-free by construction).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DynamicArray, 4 free function(s)

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
        use core::ptr;

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn realloc(__ptr: *mut core::ffi::c_void, __size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct DynamicArray {
            pub data: *mut i32,
            pub size: usize,
            pub capacity: usize,
        }

        #[no_mangle]
        pub static matrix: [[i32; 4]; 3] = [
            [0x1, 0x2, 0x3, 0x4],
            [0x10, 0x20, 0x30, 0x40],
            [0xa1, 0xb2, 0xc3, 0xd4],
        ];

        pub const FLAG_READ: i32 = 0o1;
        pub const FLAG_WRITE: i32 = 0o2;
        pub const FLAG_EXECUTE: i32 = 0o4;
        pub const FLAG_DELETE: i32 = 0o10;

        pub(crate) unsafe fn init_array(initial_capacity: usize) -> *mut DynamicArray {
            let arr_ptr = malloc(core::mem::size_of::<DynamicArray>()) as *mut DynamicArray;
            let Some(arr) = arr_ptr.as_mut() else {
                return ptr::null_mut();
            };

            let bytes = initial_capacity.saturating_mul(core::mem::size_of::<i32>());
            let data_ptr = malloc(bytes) as *mut i32;
            if data_ptr.is_null() {
                free(arr_ptr as *mut core::ffi::c_void);
                return ptr::null_mut();
            }

            arr.data = data_ptr;
            arr.size = 0;
            arr.capacity = initial_capacity;
            arr_ptr
        }

        pub(crate) unsafe fn expand_array(mut arr: Option<&mut DynamicArray>) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            let new_capacity = a.capacity.saturating_mul(2);
            let new_bytes = new_capacity.saturating_mul(core::mem::size_of::<i32>());

            let new_ptr = realloc(a.data as *mut core::ffi::c_void, new_bytes) as *mut i32;
            if new_ptr.is_null() {
                return 0;
            }

            a.data = new_ptr;
            a.capacity = new_capacity;
            1
        }

        pub(crate) unsafe fn add_element(mut arr: Option<&mut DynamicArray>, value: i32) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            if a.size >= a.capacity && expand_array(Some(a)) == 0 {
                return 0;
            }

            let idx = a.size;
            a.size = a.size.wrapping_add(1);
            *a.data.add(idx) = value;
            1
        }

        pub(crate) unsafe fn free_array(arr: Option<&DynamicArray>) {
            if let Some(a) = arr {
                free(a.data as *mut core::ffi::c_void);
                free(a as *const DynamicArray as *mut core::ffi::c_void);
            }
        }

        pub(crate) fn process_flags(flags: i32) -> i32 {
            let read_enabled = ((flags & FLAG_READ) != 0) as i32;
            let write_enabled = ((flags & FLAG_WRITE) != 0) as i32;
            let execute_enabled = ((flags & FLAG_EXECUTE) != 0) as i32;
            let delete_enabled = ((flags & FLAG_DELETE) != 0) as i32;
            read_enabled + write_enabled + execute_enabled + delete_enabled
        }

        pub(crate) fn calculate_matrix_checksum() -> i32 {
            matrix.iter().flatten().copied().sum()
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrixsum(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
            let hex_base: i32 = 0xff;
            let hex_multiplier: i32 = 0x10;

            let mut permissions: i32 = 0;
            if param1 != 0 {
                permissions |= FLAG_READ;
            }
            if param2 != 0 {
                permissions |= FLAG_WRITE;
            }
            if param3 != 0 {
                permissions |= FLAG_EXECUTE;
            }
            if param4 != 0 {
                permissions |= FLAG_DELETE;
            }

            let mut arr: Option<&mut DynamicArray> = init_array(2usize).as_mut();
            if arr.is_none() {
                return -1;
            }

            add_element(arr.as_deref_mut(), param1);
            add_element(arr.as_deref_mut(), param2);
            add_element(arr.as_deref_mut(), param3);
            add_element(arr.as_deref_mut(), param4);

            let a = arr.as_deref().unwrap();
            let values = core::slice::from_raw_parts(a.data, a.size);
            let sum: i32 = values.iter().copied().sum();

            let flag_count: i32 = process_flags(permissions);
            let matrix_sum: i32 = calculate_matrix_checksum();
            let result: i32 = sum * hex_multiplier + flag_count * hex_base + (matrix_sum & 0xfff);

            free_array(arr.as_deref());
            result
        }
    }
}
```

**Entity:** DynamicArray

**States:** Unallocated/Null, Allocated (struct+buffer live), Freed (dangling/invalid)

**Transitions:**
- Unallocated/Null -> Allocated via init_array(initial_capacity) returning non-null *mut DynamicArray
- Allocated -> Allocated (capacity grow) via expand_array(Some(&mut DynamicArray)) calling realloc
- Allocated -> Freed via free_array(Some(&DynamicArray)) calling free() on both data and struct

**Evidence:** struct DynamicArray { data: *mut i32, size: usize, capacity: usize } — raw pointer + manual size/capacity bookkeeping; init_array(): malloc DynamicArray, then malloc data buffer; on data_ptr.is_null() it calls free(arr_ptr) and returns null_mut(); add_element(): writes with *a.data.add(idx) after possible expand_array(); assumes data is allocated and idx < capacity; expand_array(): uses realloc(a.data, new_bytes) and updates a.data/a.capacity; assumes a.data is a valid allocation; free_array(arr: Option<&DynamicArray>): calls free(a.data) and free(a as *const DynamicArray as *mut c_void) despite only having a shared borrow; does not consume an owning pointer/handle

**Implementation:** Introduce an owning wrapper (e.g., struct DynamicArrayBox { ptr: NonNull<DynamicArray> }) that is the only type allowed to call free in Drop. Provide safe methods on &mut self for push/expand that maintain size<=capacity, and hide raw pointers. If FFI needs a raw pointer, expose as_ptr()/as_mut_ptr() temporarily without transferring ownership.

---

## State Machine Invariants

### 2. DynamicArray internal bounds invariant (size <= capacity, no overflow) during push/expand

**Location**: `/data/test_case/lib.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: DynamicArray operations rely on an implicit validity state: (1) size must never exceed capacity, (2) data must point to an allocation large enough for capacity i32s, and (3) size increments must not overflow. The code attempts to maintain this with runtime checks and saturating arithmetic, but it is not guaranteed: add_element() uses wrapping_add on size, and expand_array() can saturate capacity multiplication, potentially leaving capacity unchanged while add_element continues. These invariants could be enforced by making growth and indexing safe and by using checked arithmetic and slice-based storage.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DynamicArray, 4 free function(s)

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
        use core::ptr;

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn realloc(__ptr: *mut core::ffi::c_void, __size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct DynamicArray {
            pub data: *mut i32,
            pub size: usize,
            pub capacity: usize,
        }

        #[no_mangle]
        pub static matrix: [[i32; 4]; 3] = [
            [0x1, 0x2, 0x3, 0x4],
            [0x10, 0x20, 0x30, 0x40],
            [0xa1, 0xb2, 0xc3, 0xd4],
        ];

        pub const FLAG_READ: i32 = 0o1;
        pub const FLAG_WRITE: i32 = 0o2;
        pub const FLAG_EXECUTE: i32 = 0o4;
        pub const FLAG_DELETE: i32 = 0o10;

        pub(crate) unsafe fn init_array(initial_capacity: usize) -> *mut DynamicArray {
            let arr_ptr = malloc(core::mem::size_of::<DynamicArray>()) as *mut DynamicArray;
            let Some(arr) = arr_ptr.as_mut() else {
                return ptr::null_mut();
            };

            let bytes = initial_capacity.saturating_mul(core::mem::size_of::<i32>());
            let data_ptr = malloc(bytes) as *mut i32;
            if data_ptr.is_null() {
                free(arr_ptr as *mut core::ffi::c_void);
                return ptr::null_mut();
            }

            arr.data = data_ptr;
            arr.size = 0;
            arr.capacity = initial_capacity;
            arr_ptr
        }

        pub(crate) unsafe fn expand_array(mut arr: Option<&mut DynamicArray>) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            let new_capacity = a.capacity.saturating_mul(2);
            let new_bytes = new_capacity.saturating_mul(core::mem::size_of::<i32>());

            let new_ptr = realloc(a.data as *mut core::ffi::c_void, new_bytes) as *mut i32;
            if new_ptr.is_null() {
                return 0;
            }

            a.data = new_ptr;
            a.capacity = new_capacity;
            1
        }

        pub(crate) unsafe fn add_element(mut arr: Option<&mut DynamicArray>, value: i32) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            if a.size >= a.capacity && expand_array(Some(a)) == 0 {
                return 0;
            }

            let idx = a.size;
            a.size = a.size.wrapping_add(1);
            *a.data.add(idx) = value;
            1
        }

        pub(crate) unsafe fn free_array(arr: Option<&DynamicArray>) {
            if let Some(a) = arr {
                free(a.data as *mut core::ffi::c_void);
                free(a as *const DynamicArray as *mut core::ffi::c_void);
            }
        }

        pub(crate) fn process_flags(flags: i32) -> i32 {
            let read_enabled = ((flags & FLAG_READ) != 0) as i32;
            let write_enabled = ((flags & FLAG_WRITE) != 0) as i32;
            let execute_enabled = ((flags & FLAG_EXECUTE) != 0) as i32;
            let delete_enabled = ((flags & FLAG_DELETE) != 0) as i32;
            read_enabled + write_enabled + execute_enabled + delete_enabled
        }

        pub(crate) fn calculate_matrix_checksum() -> i32 {
            matrix.iter().flatten().copied().sum()
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrixsum(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
            let hex_base: i32 = 0xff;
            let hex_multiplier: i32 = 0x10;

            let mut permissions: i32 = 0;
            if param1 != 0 {
                permissions |= FLAG_READ;
            }
            if param2 != 0 {
                permissions |= FLAG_WRITE;
            }
            if param3 != 0 {
                permissions |= FLAG_EXECUTE;
            }
            if param4 != 0 {
                permissions |= FLAG_DELETE;
            }

            let mut arr: Option<&mut DynamicArray> = init_array(2usize).as_mut();
            if arr.is_none() {
                return -1;
            }

            add_element(arr.as_deref_mut(), param1);
            add_element(arr.as_deref_mut(), param2);
            add_element(arr.as_deref_mut(), param3);
            add_element(arr.as_deref_mut(), param4);

            let a = arr.as_deref().unwrap();
            let values = core::slice::from_raw_parts(a.data, a.size);
            let sum: i32 = values.iter().copied().sum();

            let flag_count: i32 = process_flags(permissions);
            let matrix_sum: i32 = calculate_matrix_checksum();
            let result: i32 = sum * hex_multiplier + flag_count * hex_base + (matrix_sum & 0xfff);

            free_array(arr.as_deref());
            result
        }
    }
}
```

**Entity:** DynamicArray

**States:** Valid (size <= capacity, data points to capacity elements), Invalid (size > capacity and/or data insufficient)

**Transitions:**
- Valid -> Valid via add_element() when (size < capacity) or expand_array() succeeds
- Valid -> Invalid via add_element() if size wraps (wrapping_add) or if capacity growth saturates and cannot actually accommodate more elements

**Evidence:** add_element(): `if a.size >= a.capacity && expand_array(Some(a)) == 0 { return 0; }` — runtime guard for size/capacity; add_element(): `a.size = a.size.wrapping_add(1);` — allows silent overflow of size; add_element(): `*a.data.add(idx) = value;` — unchecked write based on idx=size prior to increment; expand_array(): `let new_capacity = a.capacity.saturating_mul(2);` — capacity growth may saturate, affecting the size<capacity invariant

**Implementation:** Keep DynamicArray behind an owning safe wrapper that stores capacity/len as usize but uses checked_add/checked_mul for growth and len increments. Expose push(&mut self, i32) -> Result<(), AllocError> that ensures len < cap before writing. Internally use `slice::from_raw_parts_mut(data, capacity)` for bounds-checked indexing (or wrap with Vec<i32> if FFI constraints allow).

---

## Precondition Invariants

### 3. Permission flags validity protocol (bitmask domain vs arbitrary i32)

**Location**: `/data/test_case/lib.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: process_flags() implicitly assumes `flags` is a bitmask composed only of FLAG_READ/FLAG_WRITE/FLAG_EXECUTE/FLAG_DELETE. The function will silently ignore unknown bits, and callers can pass any i32. This is a latent domain invariant (allowed bits set) that could be made unrepresentable by introducing a dedicated flags type instead of raw i32.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DynamicArray, 4 free function(s)

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
        use core::ptr;

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn realloc(__ptr: *mut core::ffi::c_void, __size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct DynamicArray {
            pub data: *mut i32,
            pub size: usize,
            pub capacity: usize,
        }

        #[no_mangle]
        pub static matrix: [[i32; 4]; 3] = [
            [0x1, 0x2, 0x3, 0x4],
            [0x10, 0x20, 0x30, 0x40],
            [0xa1, 0xb2, 0xc3, 0xd4],
        ];

        pub const FLAG_READ: i32 = 0o1;
        pub const FLAG_WRITE: i32 = 0o2;
        pub const FLAG_EXECUTE: i32 = 0o4;
        pub const FLAG_DELETE: i32 = 0o10;

        pub(crate) unsafe fn init_array(initial_capacity: usize) -> *mut DynamicArray {
            let arr_ptr = malloc(core::mem::size_of::<DynamicArray>()) as *mut DynamicArray;
            let Some(arr) = arr_ptr.as_mut() else {
                return ptr::null_mut();
            };

            let bytes = initial_capacity.saturating_mul(core::mem::size_of::<i32>());
            let data_ptr = malloc(bytes) as *mut i32;
            if data_ptr.is_null() {
                free(arr_ptr as *mut core::ffi::c_void);
                return ptr::null_mut();
            }

            arr.data = data_ptr;
            arr.size = 0;
            arr.capacity = initial_capacity;
            arr_ptr
        }

        pub(crate) unsafe fn expand_array(mut arr: Option<&mut DynamicArray>) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            let new_capacity = a.capacity.saturating_mul(2);
            let new_bytes = new_capacity.saturating_mul(core::mem::size_of::<i32>());

            let new_ptr = realloc(a.data as *mut core::ffi::c_void, new_bytes) as *mut i32;
            if new_ptr.is_null() {
                return 0;
            }

            a.data = new_ptr;
            a.capacity = new_capacity;
            1
        }

        pub(crate) unsafe fn add_element(mut arr: Option<&mut DynamicArray>, value: i32) -> i32 {
            let Some(a) = arr.as_deref_mut() else {
                return 0;
            };

            if a.size >= a.capacity && expand_array(Some(a)) == 0 {
                return 0;
            }

            let idx = a.size;
            a.size = a.size.wrapping_add(1);
            *a.data.add(idx) = value;
            1
        }

        pub(crate) unsafe fn free_array(arr: Option<&DynamicArray>) {
            if let Some(a) = arr {
                free(a.data as *mut core::ffi::c_void);
                free(a as *const DynamicArray as *mut core::ffi::c_void);
            }
        }

        pub(crate) fn process_flags(flags: i32) -> i32 {
            let read_enabled = ((flags & FLAG_READ) != 0) as i32;
            let write_enabled = ((flags & FLAG_WRITE) != 0) as i32;
            let execute_enabled = ((flags & FLAG_EXECUTE) != 0) as i32;
            let delete_enabled = ((flags & FLAG_DELETE) != 0) as i32;
            read_enabled + write_enabled + execute_enabled + delete_enabled
        }

        pub(crate) fn calculate_matrix_checksum() -> i32 {
            matrix.iter().flatten().copied().sum()
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrixsum(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
            let hex_base: i32 = 0xff;
            let hex_multiplier: i32 = 0x10;

            let mut permissions: i32 = 0;
            if param1 != 0 {
                permissions |= FLAG_READ;
            }
            if param2 != 0 {
                permissions |= FLAG_WRITE;
            }
            if param3 != 0 {
                permissions |= FLAG_EXECUTE;
            }
            if param4 != 0 {
                permissions |= FLAG_DELETE;
            }

            let mut arr: Option<&mut DynamicArray> = init_array(2usize).as_mut();
            if arr.is_none() {
                return -1;
            }

            add_element(arr.as_deref_mut(), param1);
            add_element(arr.as_deref_mut(), param2);
            add_element(arr.as_deref_mut(), param3);
            add_element(arr.as_deref_mut(), param4);

            let a = arr.as_deref().unwrap();
            let values = core::slice::from_raw_parts(a.data, a.size);
            let sum: i32 = values.iter().copied().sum();

            let flag_count: i32 = process_flags(permissions);
            let matrix_sum: i32 = calculate_matrix_checksum();
            let result: i32 = sum * hex_multiplier + flag_count * hex_base + (matrix_sum & 0xfff);

            free_array(arr.as_deref());
            result
        }
    }
}
```

**Entity:** flags: i32 (FLAG_READ/WRITE/EXECUTE/DELETE as bitmask)

**States:** ValidFlags (subset of defined FLAG_*), InvalidFlags (contains unknown bits)

**Transitions:**
- InvalidFlags -> ValidFlags via masking/validation before calling process_flags()
- ValidFlags -> ValidFlags via bitwise or/and operations on the flags type

**Evidence:** const FLAG_READ/FLAG_WRITE/FLAG_EXECUTE/FLAG_DELETE define the intended bit domain; process_flags(flags: i32): checks bits via `(flags & FLAG_READ) != 0` etc.; no validation that flags has no other bits set; matrixsum(): builds `permissions` by OR-ing only these FLAG_* constants, indicating the intended construction protocol

**Implementation:** Define `struct Permissions(i32);` with constructors like `Permissions::empty()` and setters, and/or use the `bitflags` crate. Implement `TryFrom<i32>` to validate no unknown bits (`flags & !ALL == 0`). Change process_flags to take `Permissions` instead of `i32`.

---

