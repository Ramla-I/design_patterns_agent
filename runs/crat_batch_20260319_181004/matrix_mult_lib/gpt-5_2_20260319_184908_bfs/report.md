# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 2
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 2. matrix_t ownership & validity protocol (Allocated/Valid -> Freed/Invalid; plus NULL-as-error)

**Location**: `/data/test_case/lib.rs:1-366`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: matrix_t is managed manually via malloc/free and passed around as raw pointers. Several APIs return null pointers to signal allocation/parse failure, and callers are expected to treat null as an error state. After successful creation, the caller must eventually free the matrix via free_matrix; after freeing, any further use is UB but not prevented by the type system. The type system also doesn’t encode whether a matrix pointer is non-null, uniquely owned, or still alive; this is instead handled by raw pointers, Option<&matrix_t> parameters, and runtime null checks.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct matrix_t, 7 free function(s)

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
        use core::ffi::c_void;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct matrix_t {
            pub matrix: *mut *mut i32,
            pub width: i32,
            pub height: i32,
        }

        // Provide the missing crate::c_lib module used by the original C2Rust output.
        // Keep it minimal and C-compatible.
        pub mod c_lib {
            extern "C" {
                pub fn rs_perror(msg: *const i8);
                pub fn atoi(s: *const i8) -> i32;
            }
        }

        // === matrix.rs / write.rs externs ===
        extern "C" {
            fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
            fn malloc(__size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn strcat(__dest: *mut i8, __src: *const i8) -> *mut i8;
            fn strdup(__s: *const i8) -> *mut i8;
            fn strtok_r(__s: *mut i8, __delim: *const i8, __save_ptr: *mut *mut i8) -> *mut i8;

            fn __errno_location() -> *mut i32;
            fn strerror(__errnum: i32) -> *mut i8;
        }

        pub const EINVAL: i32 = 22;

        #[inline]
        unsafe fn errno_get() -> i32 {
            *__errno_location()
        }

        unsafe fn free_matrix_raw(mat: *mut matrix_t) {
            if mat.is_null() {
                return;
            }
            let height = (*mat).height;
            let rows = (*mat).matrix;

            if !rows.is_null() {
                for i in 0..(height.max(0) as usize) {
                    let row_ptr = *rows.add(i);
                    if !row_ptr.is_null() {
                        free(row_ptr as *mut c_void);
                    }
                }
                free(rows as *mut c_void);
            }
            free(mat as *mut c_void);
        }

        pub(crate) unsafe fn allocate_matrix(width: i32, height: i32) -> *mut matrix_t {
            let mat_ptr = malloc(core::mem::size_of::<matrix_t>()) as *mut matrix_t;
            let Some(mat) = mat_ptr.as_mut() else {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix struct\0" as *const u8 as *const i8,
                );
                return core::ptr::null_mut();
            };

            mat.width = width;
            mat.height = height;

            let rows_ptr = malloc((height as usize).wrapping_mul(core::mem::size_of::<*mut i32>()))
                as *mut *mut i32;
            if rows_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix rows\0" as *const u8 as *const i8,
                );
                free(mat_ptr as *mut c_void);
                return core::ptr::null_mut();
            }
            mat.matrix = rows_ptr;

            for i in 0..(height.max(0) as usize) {
                let row_ptr =
                    malloc((width as usize).wrapping_mul(core::mem::size_of::<i32>())) as *mut i32;
                if row_ptr.is_null() {
                    c_lib::rs_perror(
                        b"Failed to allocate memory for matrix columns\0" as *const u8 as *const i8,
                    );
                    for j in 0..=i {
                        let p = *rows_ptr.add(j);
                        if !p.is_null() {
                            free(p as *mut c_void);
                        }
                    }
                    free(rows_ptr as *mut c_void);
                    free(mat_ptr as *mut c_void);
                    return core::ptr::null_mut();
                }
                *rows_ptr.add(i) = row_ptr;
            }

            mat_ptr
        }

        #[no_mangle]
        pub unsafe extern "C" fn free_matrix(mat: Option<&matrix_t>) {
            let Some(mat_ref) = mat else { return };
            free_matrix_raw(mat_ref as *const matrix_t as *mut matrix_t);
        }

        pub(crate) unsafe fn initialize_matrix_from_string_internal(
            input: &[i8],
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            let mat_ptr = allocate_matrix(width, height);
            if mat_ptr.is_null() {
                return core::ptr::null();
            }

            let dup_ptr = strdup(input.as_ptr());
            if dup_ptr.is_null() {
                c_lib::rs_perror(b"Failed to duplicate input string\0" as *const u8 as *const i8);
                free_matrix_raw(mat_ptr);
                return core::ptr::null();
            }

            let mut saveptr_row: *mut i8 = core::ptr::null_mut();
            let mut row_tok =
                strtok_r(dup_ptr, b"\n\0" as *const u8 as *const i8, &mut saveptr_row);

            for i in 0..(height.max(0) as usize) {
                if row_tok.is_null() {
                    eprintln!("Insufficient rows in input string.");
                    free(dup_ptr as *mut c_void);
                    free_matrix_raw(mat_ptr);
                    return core::ptr::null();
                }

                let mut saveptr_col: *mut i8 = core::ptr::null_mut();
                let mut col_tok =
                    strtok_r(row_tok, b" \0" as *const u8 as *const i8, &mut saveptr_col);

                for j in 0..(width.max(0) as usize) {
                    if col_tok.is_null() {
                        eprintln!("Insufficient columns in row {0}.", (i as i32) + 1);
                        free(dup_ptr as *mut c_void);
                        free_matrix_raw(mat_ptr);
                        return core::ptr::null();
                    }

                    let value = c_lib::atoi(col_tok as *const i8);
                    *(*(*mat_ptr).matrix.add(i)).add(j) = value;

                    col_tok = strtok_r(
                        core::ptr::null_mut(),
                        b" \0" as *const u8 as *const i8,
                        &mut saveptr_col,
                    );
                }

                row_tok = strtok_r(
                    core::ptr::null_mut(),
                    b"\n\0" as *const u8 as *const i8,
                    &mut saveptr_row,
                );
            }

            free(dup_ptr as *mut c_void);
            mat_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_matrix_from_string(
            input: *const i8,
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            initialize_matrix_from_string_internal(
                if input.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(input, 1024)
                },
                width,
                height,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn multiply_matrices(
            mat_a: Option<&matrix_t>,
            mat_b: Option<&matrix_t>,
        ) -> *const matrix_t {
            let a = mat_a.unwrap();
            let b = mat_b.unwrap();

            if a.width != b.height {
                eprintln!("Matrix dimensions do not allow multiplication.");
                return core::ptr::null();
            }

            let result_ptr = allocate_matrix(b.width, a.height);
            if result_ptr.is_null() {
                return core::ptr::null();
            }

            for i in 0..(a.height.max(0) as usize) {
                for j in 0..(b.width.max(0) as usize) {
                    let mut acc: i32 = 0;
                    for k in 0..(a.width.max(0) as usize) {
                        let av = *(*a.matrix.add(i)).add(k);
                        let bv = *(*b.matrix.add(k)).add(j);
                        acc = acc.wrapping_add(av.wrapping_mul(bv));
                    }
                    *(*(*result_ptr).matrix.add(i)).add(j) = acc;
                }
            }

            result_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrix_to_string(mat: Option<&matrix_t>) -> *const i8 {
            let Some(mat) = mat else {
                eprintln!("Error: Matrix is NULL.");
                return core::ptr::null();
            };

            let buffer_size: i32 = mat.height * (mat.width * 10 + mat.width) + mat.height + 1;

            let out_ptr = malloc(buffer_size as usize) as *mut i8;
            if out_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix string\0" as *const u8 as *const i8,
                );
                return core::ptr::null();
            }

            *out_ptr = 0;

            for i in 0..(mat.height.max(0) as usize) {
                for j in 0..(mat.width.max(0) as usize) {
                    let mut tmp: [i8; 12] = [0; 12];
                    snprintf(
                        tmp.as_mut_ptr(),
                        core::mem::size_of::<[i8; 12]>(),
                        b"%d\0" as *const u8 as *const i8,
                        *(*mat.matrix.add(i)).add(j),
                    );
                    strcat(out_ptr, tmp.as_ptr());
                    if (j as i32) < mat.width - 1 {
                        strcat(out_ptr, b" \0" as *const u8 as *const i8);
                    }
                }
                strcat(out_ptr, b"\n\0" as *const u8 as *const i8);
            }

            out_ptr as *const i8
        }

        pub(crate) unsafe fn write_to_file_internal(filename: &[i8], content: &[i8]) -> i32 {
            if content.is_empty() {
                eprintln!("Error: Content is NULL.");
                return EINVAL;
            }

            let path = match std::ffi::CStr::from_ptr(filename.as_ptr() as _).to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Error: Filename is not valid UTF-8.");
                    return EINVAL;
                }
            };

            let mut file = match std::fs::File::create(path)
                .ok()
                .map::<std::io::BufWriter<std::fs::File>, _>(std::io::BufWriter::new)
            {
                Some(f) => f,
                None => {
                    eprintln!(
                        "Error opening file '{0}': {1}",
                        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                            .unwrap()
                            .to_str()
                            .unwrap(),
                        std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                            .to_str()
                            .unwrap()
                    );
                    return errno_get();
                }
            };

            use std::io::Write;
            let string_to_print = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(content))
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            if write!(file, "{string_to_print}").is_err() {
                eprintln!(
                    "Error writing to file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                let _ = file.flush();
                return errno_get();
            }

            if file.flush().is_err() {
                eprintln!(
                    "Error closing file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                return errno_get();
            }

            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn write_to_file(filename: *const i8, content: *const i8) -> i32 {
            write_to_file_internal(
                if filename.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(filename, 1024)
                },
                if content.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(content, 1024)
                },
            )
        }
    }
}
```

**Entity:** matrix_t

**States:** Null (error / absent), Allocated+Initialized (valid), Freed (invalid/dangling)

**Transitions:**
- Null -> Allocated+Initialized via allocate_matrix() / initialize_matrix_from_string_internal() on success
- Allocated+Initialized -> Null via allocation/parse failure returning null (core::ptr::null / null_mut)
- Allocated+Initialized -> Freed via free_matrix()/free_matrix_raw()

**Evidence:** struct matrix_t { matrix: *mut *mut i32, width: i32, height: i32 } uses raw pointers with no lifetime/ownership tracking; allocate_matrix(): returns *mut matrix_t; on allocation failure returns core::ptr::null_mut() after rs_perror(...); initialize_matrix_from_string_internal(): if strdup(...) fails or tokenization finds insufficient rows/cols, calls free_matrix_raw(mat_ptr) and returns core::ptr::null(); free_matrix_raw(mat: *mut matrix_t): frees rows, then rows pointer, then mat pointer; early-return if mat.is_null(); free_matrix(mat: Option<&matrix_t>): accepts a borrowed reference but then casts to *mut and frees it (free_matrix_raw(mat_ref as *const matrix_t as *mut matrix_t)), implying an implicit ownership transfer not expressed in the signature

**Implementation:** Introduce an owning Rust handle, e.g. `struct Matrix(NonNull<matrix_t>); impl Drop for Matrix { fn drop(&mut self){ unsafe{ free_matrix_raw(self.0.as_ptr()) } } }`. Make constructors return `Result<Matrix, Error>` instead of null. Expose safe methods on `&Matrix` for operations, and only allow raw pointer escape via `fn as_ptr(&self) -> *const matrix_t` for FFI. Use `NonNull` to encode non-null at the type level.

---

### 1. FFI matrix pointer validity + shape invariant (Allocated / Freed; Rectangular)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: `matrix_t` is an FFI-facing struct that exposes raw pointers (`matrix: *mut *mut i32`) plus dimensions (`width`, `height`). Correct use relies on implicit invariants not enforced by Rust's type system: the `matrix` pointer must either be null (or otherwise treated as uninitialized) or point to a valid 2D allocation whose outer length matches `height` and whose inner rows each have length `width`. Additionally, because the type is `Copy`, values can be duplicated freely, which makes it easy to accidentally double-free or use-after-free if any external code treats `matrix_t` as owning the allocation.

**Evidence**:

```rust
// Note: Other parts of this module contain: 3 free function(s)

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct matrix_t {
            pub matrix: *mut *mut i32,
            pub width: i32,
            pub height: i32,
        }

```

**Entity:** matrix_t

**States:** UninitializedOrNull, AllocatedValid, FreedOrDangling

**Transitions:**
- UninitializedOrNull -> AllocatedValid via external/FFI allocation/initialization (not shown)
- AllocatedValid -> FreedOrDangling via external/FFI deallocation/free (not shown)

**Evidence:** line 6: `pub matrix: *mut *mut i32` is a raw pointer-to-pointer with no lifetime/ownership tracking; line 7-8: `pub width: i32`, `pub height: i32` are separate fields that must match the allocation behind `matrix` (shape invariant not encoded); line 4: `#[derive(Copy, Clone)]` allows implicit duplication of a value that contains a raw pointer, which can violate single-owner/dropped-once expectations

**Implementation:** Introduce a safe wrapper that encodes validity and shape, e.g. `struct Matrix<'a> { rows: NonNull<[NonNull<[i32]>]> }` or a flat `NonNull<[i32]>` with `(width, height)` and indexing; if ownership is intended, make it `struct OwnedMatrix { ptr: NonNull<i32>, width: usize, height: usize }` with `Drop` to free exactly once and remove `Copy`. If it's borrowed, tie it to a lifetime `MatrixRef<'a>` and keep the raw `matrix_t` only for FFI boundaries.

---

## Precondition Invariants

### 3. matrix_t dimension compatibility precondition (multiplication requires a.width == b.height)

**Location**: `/data/test_case/lib.rs:1-366`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Matrix multiplication requires the width/height relationship `a.width == b.height`. This is checked at runtime and otherwise returns null, but nothing in the type system prevents calling multiplication with incompatible matrices. Dimensions are stored as runtime i32 fields, so compatibility is purely dynamic.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct matrix_t, 7 free function(s)

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
        use core::ffi::c_void;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct matrix_t {
            pub matrix: *mut *mut i32,
            pub width: i32,
            pub height: i32,
        }

        // Provide the missing crate::c_lib module used by the original C2Rust output.
        // Keep it minimal and C-compatible.
        pub mod c_lib {
            extern "C" {
                pub fn rs_perror(msg: *const i8);
                pub fn atoi(s: *const i8) -> i32;
            }
        }

        // === matrix.rs / write.rs externs ===
        extern "C" {
            fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
            fn malloc(__size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn strcat(__dest: *mut i8, __src: *const i8) -> *mut i8;
            fn strdup(__s: *const i8) -> *mut i8;
            fn strtok_r(__s: *mut i8, __delim: *const i8, __save_ptr: *mut *mut i8) -> *mut i8;

            fn __errno_location() -> *mut i32;
            fn strerror(__errnum: i32) -> *mut i8;
        }

        pub const EINVAL: i32 = 22;

        #[inline]
        unsafe fn errno_get() -> i32 {
            *__errno_location()
        }

        unsafe fn free_matrix_raw(mat: *mut matrix_t) {
            if mat.is_null() {
                return;
            }
            let height = (*mat).height;
            let rows = (*mat).matrix;

            if !rows.is_null() {
                for i in 0..(height.max(0) as usize) {
                    let row_ptr = *rows.add(i);
                    if !row_ptr.is_null() {
                        free(row_ptr as *mut c_void);
                    }
                }
                free(rows as *mut c_void);
            }
            free(mat as *mut c_void);
        }

        pub(crate) unsafe fn allocate_matrix(width: i32, height: i32) -> *mut matrix_t {
            let mat_ptr = malloc(core::mem::size_of::<matrix_t>()) as *mut matrix_t;
            let Some(mat) = mat_ptr.as_mut() else {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix struct\0" as *const u8 as *const i8,
                );
                return core::ptr::null_mut();
            };

            mat.width = width;
            mat.height = height;

            let rows_ptr = malloc((height as usize).wrapping_mul(core::mem::size_of::<*mut i32>()))
                as *mut *mut i32;
            if rows_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix rows\0" as *const u8 as *const i8,
                );
                free(mat_ptr as *mut c_void);
                return core::ptr::null_mut();
            }
            mat.matrix = rows_ptr;

            for i in 0..(height.max(0) as usize) {
                let row_ptr =
                    malloc((width as usize).wrapping_mul(core::mem::size_of::<i32>())) as *mut i32;
                if row_ptr.is_null() {
                    c_lib::rs_perror(
                        b"Failed to allocate memory for matrix columns\0" as *const u8 as *const i8,
                    );
                    for j in 0..=i {
                        let p = *rows_ptr.add(j);
                        if !p.is_null() {
                            free(p as *mut c_void);
                        }
                    }
                    free(rows_ptr as *mut c_void);
                    free(mat_ptr as *mut c_void);
                    return core::ptr::null_mut();
                }
                *rows_ptr.add(i) = row_ptr;
            }

            mat_ptr
        }

        #[no_mangle]
        pub unsafe extern "C" fn free_matrix(mat: Option<&matrix_t>) {
            let Some(mat_ref) = mat else { return };
            free_matrix_raw(mat_ref as *const matrix_t as *mut matrix_t);
        }

        pub(crate) unsafe fn initialize_matrix_from_string_internal(
            input: &[i8],
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            let mat_ptr = allocate_matrix(width, height);
            if mat_ptr.is_null() {
                return core::ptr::null();
            }

            let dup_ptr = strdup(input.as_ptr());
            if dup_ptr.is_null() {
                c_lib::rs_perror(b"Failed to duplicate input string\0" as *const u8 as *const i8);
                free_matrix_raw(mat_ptr);
                return core::ptr::null();
            }

            let mut saveptr_row: *mut i8 = core::ptr::null_mut();
            let mut row_tok =
                strtok_r(dup_ptr, b"\n\0" as *const u8 as *const i8, &mut saveptr_row);

            for i in 0..(height.max(0) as usize) {
                if row_tok.is_null() {
                    eprintln!("Insufficient rows in input string.");
                    free(dup_ptr as *mut c_void);
                    free_matrix_raw(mat_ptr);
                    return core::ptr::null();
                }

                let mut saveptr_col: *mut i8 = core::ptr::null_mut();
                let mut col_tok =
                    strtok_r(row_tok, b" \0" as *const u8 as *const i8, &mut saveptr_col);

                for j in 0..(width.max(0) as usize) {
                    if col_tok.is_null() {
                        eprintln!("Insufficient columns in row {0}.", (i as i32) + 1);
                        free(dup_ptr as *mut c_void);
                        free_matrix_raw(mat_ptr);
                        return core::ptr::null();
                    }

                    let value = c_lib::atoi(col_tok as *const i8);
                    *(*(*mat_ptr).matrix.add(i)).add(j) = value;

                    col_tok = strtok_r(
                        core::ptr::null_mut(),
                        b" \0" as *const u8 as *const i8,
                        &mut saveptr_col,
                    );
                }

                row_tok = strtok_r(
                    core::ptr::null_mut(),
                    b"\n\0" as *const u8 as *const i8,
                    &mut saveptr_row,
                );
            }

            free(dup_ptr as *mut c_void);
            mat_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_matrix_from_string(
            input: *const i8,
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            initialize_matrix_from_string_internal(
                if input.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(input, 1024)
                },
                width,
                height,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn multiply_matrices(
            mat_a: Option<&matrix_t>,
            mat_b: Option<&matrix_t>,
        ) -> *const matrix_t {
            let a = mat_a.unwrap();
            let b = mat_b.unwrap();

            if a.width != b.height {
                eprintln!("Matrix dimensions do not allow multiplication.");
                return core::ptr::null();
            }

            let result_ptr = allocate_matrix(b.width, a.height);
            if result_ptr.is_null() {
                return core::ptr::null();
            }

            for i in 0..(a.height.max(0) as usize) {
                for j in 0..(b.width.max(0) as usize) {
                    let mut acc: i32 = 0;
                    for k in 0..(a.width.max(0) as usize) {
                        let av = *(*a.matrix.add(i)).add(k);
                        let bv = *(*b.matrix.add(k)).add(j);
                        acc = acc.wrapping_add(av.wrapping_mul(bv));
                    }
                    *(*(*result_ptr).matrix.add(i)).add(j) = acc;
                }
            }

            result_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrix_to_string(mat: Option<&matrix_t>) -> *const i8 {
            let Some(mat) = mat else {
                eprintln!("Error: Matrix is NULL.");
                return core::ptr::null();
            };

            let buffer_size: i32 = mat.height * (mat.width * 10 + mat.width) + mat.height + 1;

            let out_ptr = malloc(buffer_size as usize) as *mut i8;
            if out_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix string\0" as *const u8 as *const i8,
                );
                return core::ptr::null();
            }

            *out_ptr = 0;

            for i in 0..(mat.height.max(0) as usize) {
                for j in 0..(mat.width.max(0) as usize) {
                    let mut tmp: [i8; 12] = [0; 12];
                    snprintf(
                        tmp.as_mut_ptr(),
                        core::mem::size_of::<[i8; 12]>(),
                        b"%d\0" as *const u8 as *const i8,
                        *(*mat.matrix.add(i)).add(j),
                    );
                    strcat(out_ptr, tmp.as_ptr());
                    if (j as i32) < mat.width - 1 {
                        strcat(out_ptr, b" \0" as *const u8 as *const i8);
                    }
                }
                strcat(out_ptr, b"\n\0" as *const u8 as *const i8);
            }

            out_ptr as *const i8
        }

        pub(crate) unsafe fn write_to_file_internal(filename: &[i8], content: &[i8]) -> i32 {
            if content.is_empty() {
                eprintln!("Error: Content is NULL.");
                return EINVAL;
            }

            let path = match std::ffi::CStr::from_ptr(filename.as_ptr() as _).to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Error: Filename is not valid UTF-8.");
                    return EINVAL;
                }
            };

            let mut file = match std::fs::File::create(path)
                .ok()
                .map::<std::io::BufWriter<std::fs::File>, _>(std::io::BufWriter::new)
            {
                Some(f) => f,
                None => {
                    eprintln!(
                        "Error opening file '{0}': {1}",
                        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                            .unwrap()
                            .to_str()
                            .unwrap(),
                        std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                            .to_str()
                            .unwrap()
                    );
                    return errno_get();
                }
            };

            use std::io::Write;
            let string_to_print = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(content))
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            if write!(file, "{string_to_print}").is_err() {
                eprintln!(
                    "Error writing to file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                let _ = file.flush();
                return errno_get();
            }

            if file.flush().is_err() {
                eprintln!(
                    "Error closing file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                return errno_get();
            }

            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn write_to_file(filename: *const i8, content: *const i8) -> i32 {
            write_to_file_internal(
                if filename.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(filename, 1024)
                },
                if content.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(content, 1024)
                },
            )
        }
    }
}
```

**Entity:** matrix_t

**States:** Dimension-compatible, Dimension-incompatible

**Transitions:**
- Dimension-incompatible -> (error) via multiply_matrices() returning core::ptr::null()
- Dimension-compatible -> (produces result) via multiply_matrices() allocating result and computing entries

**Evidence:** multiply_matrices(mat_a, mat_b): `if a.width != b.height { eprintln!("Matrix dimensions do not allow multiplication."); return core::ptr::null(); }`; matrix_t stores `width: i32, height: i32` as runtime fields used in loops and compatibility checks

**Implementation:** Encode dimensions at the type level for safe Rust-facing APIs, e.g. `struct Matrix<const W: usize, const H: usize> { ... }` (backed by owned allocation), and implement `impl<const WA: usize, const HA: usize, const WB: usize, const HB: usize> Mul<Matrix<WB, HB>> for Matrix<WA, HA> where WA == HB` (via const generics tricks or a helper trait). Keep an FFI layer that converts raw `matrix_t` to typed matrices only after validating dimensions.

---

## Protocol Invariants

### 4. C-string / NUL-termination protocol for filename and content buffers

**Location**: `/data/test_case/lib.rs:1-366`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The file-writing functions expect `filename` and `content` buffers to be valid NUL-terminated C strings (and additionally `filename` must be valid UTF-8 for conversion to a Rust path). This is not expressed in the types: the code accepts `*const i8` and constructs `&[i8]` slices of fixed length 1024, then uses CStr parsing that will panic or mis-handle data if there is no NUL within the slice. The empty-slice sentinel `&[]` is also used to represent NULL pointers, which is a runtime convention rather than a type-level guarantee.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct matrix_t, 7 free function(s)

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
        use core::ffi::c_void;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct matrix_t {
            pub matrix: *mut *mut i32,
            pub width: i32,
            pub height: i32,
        }

        // Provide the missing crate::c_lib module used by the original C2Rust output.
        // Keep it minimal and C-compatible.
        pub mod c_lib {
            extern "C" {
                pub fn rs_perror(msg: *const i8);
                pub fn atoi(s: *const i8) -> i32;
            }
        }

        // === matrix.rs / write.rs externs ===
        extern "C" {
            fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
            fn malloc(__size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn strcat(__dest: *mut i8, __src: *const i8) -> *mut i8;
            fn strdup(__s: *const i8) -> *mut i8;
            fn strtok_r(__s: *mut i8, __delim: *const i8, __save_ptr: *mut *mut i8) -> *mut i8;

            fn __errno_location() -> *mut i32;
            fn strerror(__errnum: i32) -> *mut i8;
        }

        pub const EINVAL: i32 = 22;

        #[inline]
        unsafe fn errno_get() -> i32 {
            *__errno_location()
        }

        unsafe fn free_matrix_raw(mat: *mut matrix_t) {
            if mat.is_null() {
                return;
            }
            let height = (*mat).height;
            let rows = (*mat).matrix;

            if !rows.is_null() {
                for i in 0..(height.max(0) as usize) {
                    let row_ptr = *rows.add(i);
                    if !row_ptr.is_null() {
                        free(row_ptr as *mut c_void);
                    }
                }
                free(rows as *mut c_void);
            }
            free(mat as *mut c_void);
        }

        pub(crate) unsafe fn allocate_matrix(width: i32, height: i32) -> *mut matrix_t {
            let mat_ptr = malloc(core::mem::size_of::<matrix_t>()) as *mut matrix_t;
            let Some(mat) = mat_ptr.as_mut() else {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix struct\0" as *const u8 as *const i8,
                );
                return core::ptr::null_mut();
            };

            mat.width = width;
            mat.height = height;

            let rows_ptr = malloc((height as usize).wrapping_mul(core::mem::size_of::<*mut i32>()))
                as *mut *mut i32;
            if rows_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix rows\0" as *const u8 as *const i8,
                );
                free(mat_ptr as *mut c_void);
                return core::ptr::null_mut();
            }
            mat.matrix = rows_ptr;

            for i in 0..(height.max(0) as usize) {
                let row_ptr =
                    malloc((width as usize).wrapping_mul(core::mem::size_of::<i32>())) as *mut i32;
                if row_ptr.is_null() {
                    c_lib::rs_perror(
                        b"Failed to allocate memory for matrix columns\0" as *const u8 as *const i8,
                    );
                    for j in 0..=i {
                        let p = *rows_ptr.add(j);
                        if !p.is_null() {
                            free(p as *mut c_void);
                        }
                    }
                    free(rows_ptr as *mut c_void);
                    free(mat_ptr as *mut c_void);
                    return core::ptr::null_mut();
                }
                *rows_ptr.add(i) = row_ptr;
            }

            mat_ptr
        }

        #[no_mangle]
        pub unsafe extern "C" fn free_matrix(mat: Option<&matrix_t>) {
            let Some(mat_ref) = mat else { return };
            free_matrix_raw(mat_ref as *const matrix_t as *mut matrix_t);
        }

        pub(crate) unsafe fn initialize_matrix_from_string_internal(
            input: &[i8],
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            let mat_ptr = allocate_matrix(width, height);
            if mat_ptr.is_null() {
                return core::ptr::null();
            }

            let dup_ptr = strdup(input.as_ptr());
            if dup_ptr.is_null() {
                c_lib::rs_perror(b"Failed to duplicate input string\0" as *const u8 as *const i8);
                free_matrix_raw(mat_ptr);
                return core::ptr::null();
            }

            let mut saveptr_row: *mut i8 = core::ptr::null_mut();
            let mut row_tok =
                strtok_r(dup_ptr, b"\n\0" as *const u8 as *const i8, &mut saveptr_row);

            for i in 0..(height.max(0) as usize) {
                if row_tok.is_null() {
                    eprintln!("Insufficient rows in input string.");
                    free(dup_ptr as *mut c_void);
                    free_matrix_raw(mat_ptr);
                    return core::ptr::null();
                }

                let mut saveptr_col: *mut i8 = core::ptr::null_mut();
                let mut col_tok =
                    strtok_r(row_tok, b" \0" as *const u8 as *const i8, &mut saveptr_col);

                for j in 0..(width.max(0) as usize) {
                    if col_tok.is_null() {
                        eprintln!("Insufficient columns in row {0}.", (i as i32) + 1);
                        free(dup_ptr as *mut c_void);
                        free_matrix_raw(mat_ptr);
                        return core::ptr::null();
                    }

                    let value = c_lib::atoi(col_tok as *const i8);
                    *(*(*mat_ptr).matrix.add(i)).add(j) = value;

                    col_tok = strtok_r(
                        core::ptr::null_mut(),
                        b" \0" as *const u8 as *const i8,
                        &mut saveptr_col,
                    );
                }

                row_tok = strtok_r(
                    core::ptr::null_mut(),
                    b"\n\0" as *const u8 as *const i8,
                    &mut saveptr_row,
                );
            }

            free(dup_ptr as *mut c_void);
            mat_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_matrix_from_string(
            input: *const i8,
            width: i32,
            height: i32,
        ) -> *const matrix_t {
            initialize_matrix_from_string_internal(
                if input.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(input, 1024)
                },
                width,
                height,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn multiply_matrices(
            mat_a: Option<&matrix_t>,
            mat_b: Option<&matrix_t>,
        ) -> *const matrix_t {
            let a = mat_a.unwrap();
            let b = mat_b.unwrap();

            if a.width != b.height {
                eprintln!("Matrix dimensions do not allow multiplication.");
                return core::ptr::null();
            }

            let result_ptr = allocate_matrix(b.width, a.height);
            if result_ptr.is_null() {
                return core::ptr::null();
            }

            for i in 0..(a.height.max(0) as usize) {
                for j in 0..(b.width.max(0) as usize) {
                    let mut acc: i32 = 0;
                    for k in 0..(a.width.max(0) as usize) {
                        let av = *(*a.matrix.add(i)).add(k);
                        let bv = *(*b.matrix.add(k)).add(j);
                        acc = acc.wrapping_add(av.wrapping_mul(bv));
                    }
                    *(*(*result_ptr).matrix.add(i)).add(j) = acc;
                }
            }

            result_ptr as *const matrix_t
        }

        #[no_mangle]
        pub unsafe extern "C" fn matrix_to_string(mat: Option<&matrix_t>) -> *const i8 {
            let Some(mat) = mat else {
                eprintln!("Error: Matrix is NULL.");
                return core::ptr::null();
            };

            let buffer_size: i32 = mat.height * (mat.width * 10 + mat.width) + mat.height + 1;

            let out_ptr = malloc(buffer_size as usize) as *mut i8;
            if out_ptr.is_null() {
                c_lib::rs_perror(
                    b"Failed to allocate memory for matrix string\0" as *const u8 as *const i8,
                );
                return core::ptr::null();
            }

            *out_ptr = 0;

            for i in 0..(mat.height.max(0) as usize) {
                for j in 0..(mat.width.max(0) as usize) {
                    let mut tmp: [i8; 12] = [0; 12];
                    snprintf(
                        tmp.as_mut_ptr(),
                        core::mem::size_of::<[i8; 12]>(),
                        b"%d\0" as *const u8 as *const i8,
                        *(*mat.matrix.add(i)).add(j),
                    );
                    strcat(out_ptr, tmp.as_ptr());
                    if (j as i32) < mat.width - 1 {
                        strcat(out_ptr, b" \0" as *const u8 as *const i8);
                    }
                }
                strcat(out_ptr, b"\n\0" as *const u8 as *const i8);
            }

            out_ptr as *const i8
        }

        pub(crate) unsafe fn write_to_file_internal(filename: &[i8], content: &[i8]) -> i32 {
            if content.is_empty() {
                eprintln!("Error: Content is NULL.");
                return EINVAL;
            }

            let path = match std::ffi::CStr::from_ptr(filename.as_ptr() as _).to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Error: Filename is not valid UTF-8.");
                    return EINVAL;
                }
            };

            let mut file = match std::fs::File::create(path)
                .ok()
                .map::<std::io::BufWriter<std::fs::File>, _>(std::io::BufWriter::new)
            {
                Some(f) => f,
                None => {
                    eprintln!(
                        "Error opening file '{0}': {1}",
                        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                            .unwrap()
                            .to_str()
                            .unwrap(),
                        std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                            .to_str()
                            .unwrap()
                    );
                    return errno_get();
                }
            };

            use std::io::Write;
            let string_to_print = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(content))
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            if write!(file, "{string_to_print}").is_err() {
                eprintln!(
                    "Error writing to file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                let _ = file.flush();
                return errno_get();
            }

            if file.flush().is_err() {
                eprintln!(
                    "Error closing file '{0}': {1}",
                    std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(filename))
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    std::ffi::CStr::from_ptr(strerror(errno_get()) as _)
                        .to_str()
                        .unwrap()
                );
                return errno_get();
            }

            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn write_to_file(filename: *const i8, content: *const i8) -> i32 {
            write_to_file_internal(
                if filename.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(filename, 1024)
                },
                if content.is_null() {
                    &[]
                } else {
                    core::slice::from_raw_parts(content, 1024)
                },
            )
        }
    }
}
```

**Entity:** write_to_file_internal / write_to_file

**States:** Valid NUL-terminated C string (and valid UTF-8 for filename path), Invalid/unterminated/empty (error)

**Transitions:**
- Invalid/unterminated/empty -> (error) via returning EINVAL / errno_get() and printing error messages
- Valid NUL-terminated C string -> (writes file) via successful create + write + flush returning 0

**Evidence:** write_to_file(filename: *const i8, content: *const i8): converts pointers to `core::slice::from_raw_parts(ptr, 1024)` (fixed-length, not proven NUL-terminated); write_to_file_internal(): `if content.is_empty() { eprintln!("Error: Content is NULL."); return EINVAL; }` uses empty slice as NULL sentinel; write_to_file_internal(): `std::ffi::CStr::from_ptr(filename.as_ptr() as _).to_str()` requires `filename` be NUL-terminated and UTF-8; otherwise returns Err mapped to EINVAL; write_to_file_internal(): `CStr::from_bytes_until_nul(bytemuck::cast_slice(content)).unwrap()` and similar unwrap chains for filename in error messages assume a NUL occurs within the provided slice and will panic if not

**Implementation:** For Rust-facing entrypoints, accept `&CStr` (or `CString`) for `filename`/`content` to enforce NUL-termination, and a `Utf8Path`/`PathBuf`-validated wrapper for filenames requiring UTF-8. For FFI, keep `extern "C" fn write_to_file(filename: *const c_char, content: *const c_char)` but immediately validate into `Option<&CStr>` using `CStr::from_ptr` only after checking non-null, returning an error code instead of `unwrap()` panics.

---

