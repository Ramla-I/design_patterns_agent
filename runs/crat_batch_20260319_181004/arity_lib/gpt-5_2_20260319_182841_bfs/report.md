# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 3
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 3. Matrix output buffer sizing protocol (NonNull + at least 3 rows)

**Location**: `/data/test_case/lib.rs:1-215`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `init_matrix` writes exactly 3 rows of `[c_int; 4]` into the memory starting at `matrix`. It checks only for null, but relies on a comment-based safety contract that the pointer is valid for at least 3 rows. The type system does not encode the required row count for the output buffer; passing a smaller allocation would make `from_raw_parts_mut(matrix, 3)` and `copy_from_slice` write out of bounds (UB).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataBlock

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

pub mod src {
    pub mod lib {
        use core::ffi;
        use core::ptr;

        pub type size_t = usize;

        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct DataBlock {
            pub values: [ffi::c_int; 4],
            pub count: ffi::c_int,
            pub label: *mut ffi::c_char,
        }

        pub const NULL: *mut ffi::c_void = 0 as *mut ffi::c_void;

        #[no_mangle]
        pub unsafe extern "C" fn shift_array(
            mut arr: *mut ffi::c_int,
            mut size: ffi::c_int,
            mut positions: ffi::c_int,
        ) {
            if arr.is_null() || size <= 0 || positions <= 0 || positions >= size {
                return;
            }

            let size_usize = size as usize;
            let pos_usize = positions as usize;

            // SAFETY: caller promises `arr` points to at least `size` elements.
            let slice = core::slice::from_raw_parts_mut(arr, size_usize);

            // memmove semantics for overlapping regions.
            slice.copy_within(0..(size_usize - pos_usize), pos_usize);

            for x in &mut slice[..pos_usize] {
                *x = 0;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn process_string(mut str: *const ffi::c_char) -> ffi::c_int {
            if str.is_null() || *str == 0 {
                return 0;
            }
            // SAFETY: C string is assumed NUL-terminated.
            let len = libc_strlen(str);
            len as ffi::c_int
        }

        // Local strlen to avoid relying on external declarations while keeping behavior.
        unsafe fn libc_strlen(s: *const ffi::c_char) -> usize {
            let mut p = s;
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(s) as usize
        }

        #[no_mangle]
        pub unsafe extern "C" fn apply_bitmask(
            mut value: ffi::c_int,
            mut operation: ffi::c_int,
        ) -> ffi::c_int {
            const MASK1: ffi::c_int = 0o360;
            const MASK2: ffi::c_int = 0o17;
            const MASK3: ffi::c_int = 0o252;
            const MASK4: ffi::c_int = 0o125;

            match operation {
                0 => value & MASK1,
                1 => value & MASK2,
                2 => value | MASK3,
                3 => value ^ MASK4,
                _ => value,
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn init_matrix(mut matrix: *mut [ffi::c_int; 4]) {
            if matrix.is_null() {
                return;
            }

            let temp: [[ffi::c_int; 4]; 3] = [
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 10, 11, 12],
            ];

            // SAFETY: caller promises `matrix` points to at least 3 rows.
            let out = core::slice::from_raw_parts_mut(matrix, 3);
            out.copy_from_slice(&temp);
        }

        #[no_mangle]
        pub unsafe extern "C" fn compare_allocations(
            mut val1: ffi::c_int,
            mut val2: ffi::c_int,
        ) -> ffi::c_int {
            // Use Rust allocation; preserve the original "compare addresses" behavior.
            let mut b1 = Box::new(val1);
            let mut b2 = Box::new(val2);

            let ptr1: *mut ffi::c_int = (&mut *b1) as *mut ffi::c_int;
            let ptr2: *mut ffi::c_int = (&mut *b2) as *mut ffi::c_int;

            let mut result: ffi::c_int = if ptr1 < ptr2 {
                1
            } else if ptr1 > ptr2 {
                2
            } else {
                3
            };

            let uninit_ptr: *mut ffi::c_int = ptr1;
            result += if *uninit_ptr > 0 { 10 } else { 0 };

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity4(
            mut param1: ffi::c_int,
            mut param2: ffi::c_int,
            mut param3: ffi::c_int,
            mut param4: ffi::c_int,
        ) -> ffi::c_int {
            let mut result: ffi::c_int = 0;

            let mut block = DataBlock {
                values: [param1, param2, param3, param4],
                count: 4,
                label: ptr::null_mut(),
            };

            let test_str: [ffi::c_char; 6] = [b'H' as _, b'e' as _, b'l' as _, b'l' as _, b'o' as _, 0];
            let empty_str: [ffi::c_char; 1] = [0];

            let len1 = process_string(test_str.as_ptr());
            let len2 = process_string(empty_str.as_ptr());
            result += len1 + len2;

            shift_array(block.values.as_mut_ptr(), 4, 1);

            for &v in block.values.iter().take(block.count as usize) {
                result += v;
            }

            result = apply_bitmask(result, param1 % 4);

            let mut matrix: [[ffi::c_int; 4]; 3] = [[0; 4]; 3];
            init_matrix(matrix.as_mut_ptr());
            result += matrix[0][0] + matrix[2][3];

            let alloc_result = compare_allocations(param1, param2);
            result += alloc_result;

            if param3 != 0 {
                result = result * param3 / 100;
            }
            if param4 != 0 {
                result += param4;
            }

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity2(mut p1: ffi::c_int, mut p2: ffi::c_int) -> ffi::c_int {
            arity4(p1, p2, 0, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity3(
            mut p1: ffi::c_int,
            mut p2: ffi::c_int,
            mut p3: ffi::c_int,
        ) -> ffi::c_int {
            arity4(p1, p2, p3, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity(
            mut len: ffi::c_uchar,
            mut params: *mut ffi::c_int,
        ) -> ffi::c_int {
            if (len as ffi::c_int) < 2 {
                return -1;
            }
            if params.is_null() {
                return -1;
            }

            // SAFETY: caller promises `params` has at least `len` elements.
            let args = core::slice::from_raw_parts(params, len as usize);

            match len as ffi::c_int {
                2 => arity2(args[0], args[1]),
                3 => arity3(args[0], args[1], args[2]),
                _ => arity4(args[0], args[1], args[2], args[3]),
            }
        }
    }
}
```

**Entity:** init_matrix(matrix: *mut [c_int; 4])

**States:** NullOrTooSmallBuffer, Writable3x4Buffer

**Transitions:**
- NullOrTooSmallBuffer -> (returns early only for null) via is_null check
- Writable3x4Buffer -> (writes 3x4 constants) via from_raw_parts_mut + copy_from_slice

**Evidence:** init_matrix: `if matrix.is_null() { return; }` only checks null, not capacity; init_matrix: comment `// SAFETY: caller promises `matrix` points to at least 3 rows.`; init_matrix: `let out = core::slice::from_raw_parts_mut(matrix, 3); out.copy_from_slice(&temp);` assumes exactly 3 writable rows

**Implementation:** Offer a safe wrapper `fn init_matrix_safe(out: &mut [[c_int; 4]; 3])` (or `&mut [[c_int;4]]` with a length check) and implement the copy on that. For FFI, accept `NonNull<[c_int;4]>` plus a separate `rows: usize` capability/newtype proving `rows >= 3`, or expose a C ABI that takes `*mut c_int` plus explicit dimensions and validates them.

---

### 4. DataBlock FFI validity protocol (label pointer + count/value invariants)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: DataBlock is an FFI-facing C-layout struct whose safe use depends on runtime validity conditions that are not enforced by the type system. In particular: (1) `label` is a raw `*mut c_char` and may be null, dangling, or non-NUL-terminated; it also implies an ownership/lifetime protocol (who allocates/frees) that Rust cannot enforce from this type. (2) `count` likely constrains how many entries in `values: [c_int; 4]` are logically initialized/meaningful (e.g., 0..=4), but nothing prevents out-of-range counts. Because the struct is `Copy, Clone`, any implicit ownership of `label` would be unsafe (copies would duplicate the pointer without duplicating the allocation), so correct usage must treat `label` as non-owning/borrowed or otherwise ensure an external ownership protocol. These invariants are currently entirely implicit and rely on callers to uphold them.

**Evidence**:

```rust
// Note: Other parts of this module contain: 10 free function(s)


        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct DataBlock {
            pub values: [ffi::c_int; 4],
            pub count: ffi::c_int,
            pub label: *mut ffi::c_char,
        }

```

**Entity:** DataBlock

**States:** ValidFFI, InvalidFFI

**Transitions:**
- InvalidFFI -> ValidFFI via external initialization that sets `label` to a valid C string pointer and `count` within bounds

**Evidence:** line 6: `pub label: *mut ffi::c_char` raw pointer requires non-null/dangling and C-string validity to be usable safely; line 5: `pub count: ffi::c_int` alongside `values: [ffi::c_int; 4]` implies a bound/interpretation relationship not encoded in the type; line 2: `#[derive(Copy, Clone)]` makes any ownership/lifetime expectations for `label` impossible to enforce and easy to violate by accidental copying; line 3: `#[repr(C)]` indicates this is intended for FFI where such pointer/count protocols are common but not type-checked

**Implementation:** Introduce a safe wrapper/newtypes that encode the invariants: e.g., `struct Count0to4(u8)` (validated on construction) and `struct BorrowedCString<'a>(*const c_char, PhantomData<&'a CStr>)` or `Option<NonNull<c_char>>` for `label` (if nullable). Provide constructors like `DataBlockSafe::new(values: [c_int; 4], count: Count0to4, label: Option<&CStr>)` and an `into_ffi()` method to produce the raw `DataBlock` for FFI boundaries, keeping the raw struct as an internal/unsafe representation.

---

### 1. shift_array pointer+length validity precondition (NonNull + in-bounds shift)

**Location**: `/data/test_case/lib.rs:1-215`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `shift_array` is only correct when `arr` is non-null, `size > 0`, and `positions` satisfies `0 < positions < size`. Additionally, the caller must ensure `arr` points to at least `size` initialized `c_int` elements. The function enforces some of this with runtime checks and a comment-based safety contract, but the type system does not express non-nullness, element count, or valid shift range; violation would make the raw-to-slice conversion and copy operations UB.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataBlock

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

pub mod src {
    pub mod lib {
        use core::ffi;
        use core::ptr;

        pub type size_t = usize;

        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct DataBlock {
            pub values: [ffi::c_int; 4],
            pub count: ffi::c_int,
            pub label: *mut ffi::c_char,
        }

        pub const NULL: *mut ffi::c_void = 0 as *mut ffi::c_void;

        #[no_mangle]
        pub unsafe extern "C" fn shift_array(
            mut arr: *mut ffi::c_int,
            mut size: ffi::c_int,
            mut positions: ffi::c_int,
        ) {
            if arr.is_null() || size <= 0 || positions <= 0 || positions >= size {
                return;
            }

            let size_usize = size as usize;
            let pos_usize = positions as usize;

            // SAFETY: caller promises `arr` points to at least `size` elements.
            let slice = core::slice::from_raw_parts_mut(arr, size_usize);

            // memmove semantics for overlapping regions.
            slice.copy_within(0..(size_usize - pos_usize), pos_usize);

            for x in &mut slice[..pos_usize] {
                *x = 0;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn process_string(mut str: *const ffi::c_char) -> ffi::c_int {
            if str.is_null() || *str == 0 {
                return 0;
            }
            // SAFETY: C string is assumed NUL-terminated.
            let len = libc_strlen(str);
            len as ffi::c_int
        }

        // Local strlen to avoid relying on external declarations while keeping behavior.
        unsafe fn libc_strlen(s: *const ffi::c_char) -> usize {
            let mut p = s;
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(s) as usize
        }

        #[no_mangle]
        pub unsafe extern "C" fn apply_bitmask(
            mut value: ffi::c_int,
            mut operation: ffi::c_int,
        ) -> ffi::c_int {
            const MASK1: ffi::c_int = 0o360;
            const MASK2: ffi::c_int = 0o17;
            const MASK3: ffi::c_int = 0o252;
            const MASK4: ffi::c_int = 0o125;

            match operation {
                0 => value & MASK1,
                1 => value & MASK2,
                2 => value | MASK3,
                3 => value ^ MASK4,
                _ => value,
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn init_matrix(mut matrix: *mut [ffi::c_int; 4]) {
            if matrix.is_null() {
                return;
            }

            let temp: [[ffi::c_int; 4]; 3] = [
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 10, 11, 12],
            ];

            // SAFETY: caller promises `matrix` points to at least 3 rows.
            let out = core::slice::from_raw_parts_mut(matrix, 3);
            out.copy_from_slice(&temp);
        }

        #[no_mangle]
        pub unsafe extern "C" fn compare_allocations(
            mut val1: ffi::c_int,
            mut val2: ffi::c_int,
        ) -> ffi::c_int {
            // Use Rust allocation; preserve the original "compare addresses" behavior.
            let mut b1 = Box::new(val1);
            let mut b2 = Box::new(val2);

            let ptr1: *mut ffi::c_int = (&mut *b1) as *mut ffi::c_int;
            let ptr2: *mut ffi::c_int = (&mut *b2) as *mut ffi::c_int;

            let mut result: ffi::c_int = if ptr1 < ptr2 {
                1
            } else if ptr1 > ptr2 {
                2
            } else {
                3
            };

            let uninit_ptr: *mut ffi::c_int = ptr1;
            result += if *uninit_ptr > 0 { 10 } else { 0 };

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity4(
            mut param1: ffi::c_int,
            mut param2: ffi::c_int,
            mut param3: ffi::c_int,
            mut param4: ffi::c_int,
        ) -> ffi::c_int {
            let mut result: ffi::c_int = 0;

            let mut block = DataBlock {
                values: [param1, param2, param3, param4],
                count: 4,
                label: ptr::null_mut(),
            };

            let test_str: [ffi::c_char; 6] = [b'H' as _, b'e' as _, b'l' as _, b'l' as _, b'o' as _, 0];
            let empty_str: [ffi::c_char; 1] = [0];

            let len1 = process_string(test_str.as_ptr());
            let len2 = process_string(empty_str.as_ptr());
            result += len1 + len2;

            shift_array(block.values.as_mut_ptr(), 4, 1);

            for &v in block.values.iter().take(block.count as usize) {
                result += v;
            }

            result = apply_bitmask(result, param1 % 4);

            let mut matrix: [[ffi::c_int; 4]; 3] = [[0; 4]; 3];
            init_matrix(matrix.as_mut_ptr());
            result += matrix[0][0] + matrix[2][3];

            let alloc_result = compare_allocations(param1, param2);
            result += alloc_result;

            if param3 != 0 {
                result = result * param3 / 100;
            }
            if param4 != 0 {
                result += param4;
            }

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity2(mut p1: ffi::c_int, mut p2: ffi::c_int) -> ffi::c_int {
            arity4(p1, p2, 0, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity3(
            mut p1: ffi::c_int,
            mut p2: ffi::c_int,
            mut p3: ffi::c_int,
        ) -> ffi::c_int {
            arity4(p1, p2, p3, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity(
            mut len: ffi::c_uchar,
            mut params: *mut ffi::c_int,
        ) -> ffi::c_int {
            if (len as ffi::c_int) < 2 {
                return -1;
            }
            if params.is_null() {
                return -1;
            }

            // SAFETY: caller promises `params` has at least `len` elements.
            let args = core::slice::from_raw_parts(params, len as usize);

            match len as ffi::c_int {
                2 => arity2(args[0], args[1]),
                3 => arity3(args[0], args[1], args[2]),
                _ => arity4(args[0], args[1], args[2], args[3]),
            }
        }
    }
}
```

**Entity:** shift_array(arr: *mut c_int, size: c_int, positions: c_int)

**States:** InvalidInput, ValidInput

**Transitions:**
- InvalidInput -> (returns early) via input validation
- ValidInput -> (performs in-place shift) via from_raw_parts_mut + copy_within

**Evidence:** shift_array: `if arr.is_null() || size <= 0 || positions <= 0 || positions >= size { return; }` encodes required input relations; shift_array: comment `// SAFETY: caller promises `arr` points to at least `size` elements.`; shift_array: `core::slice::from_raw_parts_mut(arr, size_usize)` requires a valid pointer to `size_usize` elements

**Implementation:** Expose a safe wrapper that takes `arr: NonNull<c_int>` and `slice: &mut [c_int]` (or `&mut [c_int; N]`), plus a `ShiftBy` newtype that can only be constructed when `0 < positions < len`. Internally call the `extern "C"` raw function or implement the logic on slices; keep the raw API `unsafe` for FFI only.

---

## Protocol Invariants

### 2. C-string protocol (NonNull + NUL-terminated + readable memory)

**Location**: `/data/test_case/lib.rs:1-215`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `process_string`/`libc_strlen` assume the input is a readable, NUL-terminated C string. The function checks for null and for an immediate NUL byte (treating that as empty), but it cannot enforce that the pointer is valid for reads up to the terminator. `libc_strlen` walks memory until it finds `0`, which is UB if the pointer is dangling or not properly terminated. These are classic C-string protocol requirements not represented in the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataBlock

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

pub mod src {
    pub mod lib {
        use core::ffi;
        use core::ptr;

        pub type size_t = usize;

        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct DataBlock {
            pub values: [ffi::c_int; 4],
            pub count: ffi::c_int,
            pub label: *mut ffi::c_char,
        }

        pub const NULL: *mut ffi::c_void = 0 as *mut ffi::c_void;

        #[no_mangle]
        pub unsafe extern "C" fn shift_array(
            mut arr: *mut ffi::c_int,
            mut size: ffi::c_int,
            mut positions: ffi::c_int,
        ) {
            if arr.is_null() || size <= 0 || positions <= 0 || positions >= size {
                return;
            }

            let size_usize = size as usize;
            let pos_usize = positions as usize;

            // SAFETY: caller promises `arr` points to at least `size` elements.
            let slice = core::slice::from_raw_parts_mut(arr, size_usize);

            // memmove semantics for overlapping regions.
            slice.copy_within(0..(size_usize - pos_usize), pos_usize);

            for x in &mut slice[..pos_usize] {
                *x = 0;
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn process_string(mut str: *const ffi::c_char) -> ffi::c_int {
            if str.is_null() || *str == 0 {
                return 0;
            }
            // SAFETY: C string is assumed NUL-terminated.
            let len = libc_strlen(str);
            len as ffi::c_int
        }

        // Local strlen to avoid relying on external declarations while keeping behavior.
        unsafe fn libc_strlen(s: *const ffi::c_char) -> usize {
            let mut p = s;
            while *p != 0 {
                p = p.add(1);
            }
            p.offset_from(s) as usize
        }

        #[no_mangle]
        pub unsafe extern "C" fn apply_bitmask(
            mut value: ffi::c_int,
            mut operation: ffi::c_int,
        ) -> ffi::c_int {
            const MASK1: ffi::c_int = 0o360;
            const MASK2: ffi::c_int = 0o17;
            const MASK3: ffi::c_int = 0o252;
            const MASK4: ffi::c_int = 0o125;

            match operation {
                0 => value & MASK1,
                1 => value & MASK2,
                2 => value | MASK3,
                3 => value ^ MASK4,
                _ => value,
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn init_matrix(mut matrix: *mut [ffi::c_int; 4]) {
            if matrix.is_null() {
                return;
            }

            let temp: [[ffi::c_int; 4]; 3] = [
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 10, 11, 12],
            ];

            // SAFETY: caller promises `matrix` points to at least 3 rows.
            let out = core::slice::from_raw_parts_mut(matrix, 3);
            out.copy_from_slice(&temp);
        }

        #[no_mangle]
        pub unsafe extern "C" fn compare_allocations(
            mut val1: ffi::c_int,
            mut val2: ffi::c_int,
        ) -> ffi::c_int {
            // Use Rust allocation; preserve the original "compare addresses" behavior.
            let mut b1 = Box::new(val1);
            let mut b2 = Box::new(val2);

            let ptr1: *mut ffi::c_int = (&mut *b1) as *mut ffi::c_int;
            let ptr2: *mut ffi::c_int = (&mut *b2) as *mut ffi::c_int;

            let mut result: ffi::c_int = if ptr1 < ptr2 {
                1
            } else if ptr1 > ptr2 {
                2
            } else {
                3
            };

            let uninit_ptr: *mut ffi::c_int = ptr1;
            result += if *uninit_ptr > 0 { 10 } else { 0 };

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity4(
            mut param1: ffi::c_int,
            mut param2: ffi::c_int,
            mut param3: ffi::c_int,
            mut param4: ffi::c_int,
        ) -> ffi::c_int {
            let mut result: ffi::c_int = 0;

            let mut block = DataBlock {
                values: [param1, param2, param3, param4],
                count: 4,
                label: ptr::null_mut(),
            };

            let test_str: [ffi::c_char; 6] = [b'H' as _, b'e' as _, b'l' as _, b'l' as _, b'o' as _, 0];
            let empty_str: [ffi::c_char; 1] = [0];

            let len1 = process_string(test_str.as_ptr());
            let len2 = process_string(empty_str.as_ptr());
            result += len1 + len2;

            shift_array(block.values.as_mut_ptr(), 4, 1);

            for &v in block.values.iter().take(block.count as usize) {
                result += v;
            }

            result = apply_bitmask(result, param1 % 4);

            let mut matrix: [[ffi::c_int; 4]; 3] = [[0; 4]; 3];
            init_matrix(matrix.as_mut_ptr());
            result += matrix[0][0] + matrix[2][3];

            let alloc_result = compare_allocations(param1, param2);
            result += alloc_result;

            if param3 != 0 {
                result = result * param3 / 100;
            }
            if param4 != 0 {
                result += param4;
            }

            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity2(mut p1: ffi::c_int, mut p2: ffi::c_int) -> ffi::c_int {
            arity4(p1, p2, 0, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity3(
            mut p1: ffi::c_int,
            mut p2: ffi::c_int,
            mut p3: ffi::c_int,
        ) -> ffi::c_int {
            arity4(p1, p2, p3, 0)
        }

        #[no_mangle]
        pub unsafe extern "C" fn arity(
            mut len: ffi::c_uchar,
            mut params: *mut ffi::c_int,
        ) -> ffi::c_int {
            if (len as ffi::c_int) < 2 {
                return -1;
            }
            if params.is_null() {
                return -1;
            }

            // SAFETY: caller promises `params` has at least `len` elements.
            let args = core::slice::from_raw_parts(params, len as usize);

            match len as ffi::c_int {
                2 => arity2(args[0], args[1]),
                3 => arity3(args[0], args[1], args[2]),
                _ => arity4(args[0], args[1], args[2], args[3]),
            }
        }
    }
}
```

**Entity:** process_string(str: *const c_char) / libc_strlen(s: *const c_char)

**States:** NullOrEmpty, ValidCStr

**Transitions:**
- NullOrEmpty -> (returns 0) via runtime checks
- ValidCStr -> (scans to terminator) via libc_strlen loop

**Evidence:** process_string: `if str.is_null() || *str == 0 { return 0; }` encodes null/empty handling; process_string: comment `// SAFETY: C string is assumed NUL-terminated.`; libc_strlen: `while *p != 0 { p = p.add(1); }` requires a valid readable region until a NUL byte

**Implementation:** Provide a safe Rust-facing API taking `&core::ffi::CStr` (or `*const c_char` wrapped as `NonNull<c_char>` plus validation to `CStr::from_ptr`). Keep the raw `extern "C"` function `unsafe`, but inside Rust code prefer the typed `CStr`-based entrypoints so NUL-termination is a checked precondition.

---

