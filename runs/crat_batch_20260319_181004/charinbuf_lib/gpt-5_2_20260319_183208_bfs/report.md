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

### 1. C buffer ownership + NUL-termination protocol (Null / Allocated)

**Location**: `/data/test_case/lib.rs:1-221`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The raw pointer returned by create_buffer() encodes an implicit resource state: it is either null (meaning 'no buffer') or a heap allocation that must be freed exactly once with free(). Callers must branch on is_null() before using it as a C string (CStr::from_ptr) or before passing it to find_char_in_buffer, and must later call free() on the same pointer. None of this ownership/validity is represented in the type system because the API exposes *mut i8 and uses manual null checks and manual free().

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
#![feature(as_array_of_cells)]

pub mod src {
    pub mod lib {
        use core::ffi::c_void;
        use core::ptr;

        extern "C" {
            fn malloc(__size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn memchr(__s: *const c_void, __c: i32, __n: usize) -> *mut c_void;
            fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
        }

        pub type operation_func = Option<unsafe extern "C" fn(i32) -> i32>;

        thread_local! {
            static counter: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
        }

        pub(crate) extern "C" fn increment_counter(value: i32) -> i32 {
            counter.set(counter.get() + value);
            counter.get()
        }

        pub const UINT16_MAX: i32 = 65535;

        pub(crate) extern "C" fn decrement_counter(value: i32) -> i32 {
            counter.set(counter.get() - value);
            counter.get()
        }

        pub(crate) extern "C" fn multiply_counter(value: i32) -> i32 {
            counter.set(counter.get() * value);
            counter.get()
        }

        pub(crate) extern "C" fn reset_counter(value: i32) -> i32 {
            counter.set(value);
            counter.get()
        }

        pub(crate) fn is_string_empty(str: Option<&i8>) -> i32 {
            match str {
                None => 1,
                Some(first) => (*first == 0) as i32,
            }
        }

        pub(crate) unsafe fn find_char_in_buffer(
            buffer: Option<&mut i8>,
            size: usize,
            target: i8,
        ) -> *const i8 {
            let Some(b) = buffer else {
                return ptr::null();
            };
            memchr(b as *mut i8 as *const c_void, target as i32, size) as *const i8
        }

        pub(crate) unsafe fn create_buffer(initial: &[i8]) -> *mut i8 {
            if initial.is_empty() {
                return ptr::null_mut();
            }

            // Determine C-string length (up to NUL). If no NUL, treat whole slice as content.
            let len = initial.iter().position(|&c| c == 0).unwrap_or(initial.len());
            let total = len.saturating_add(1);

            let raw = malloc(total) as *mut i8;
            if raw.is_null() {
                return ptr::null_mut();
            }

            // Copy using the original C routine to preserve behavior.
            strcpy(raw, initial.as_ptr());
            raw
        }

        pub(crate) fn validate_uint16_range(value: i32) -> i32 {
            (0..=UINT16_MAX).contains(&value) as i32
        }

        pub(crate) unsafe fn apply_operation(op: operation_func, value: i32) -> i32 {
            let Some(f) = op else { return -1 };
            f(value)
        }

        #[no_mangle]
        pub unsafe extern "C" fn charinbuf(mode: i32, value: i32, opt1: i32, opt2: i32) -> i32 {
            let mut result: i32 = 0;

            // Keep the original "Option<&i8>" style checks.
            let test_string: Option<&i8> = bytemuck::cast_slice(b"\0").first();
            let non_empty_string: Option<&i8> = bytemuck::cast_slice(b"Hello, World!\0").first();

            counter.set(0);

            match mode {
                0 => {
                    println!("Mode 0: UINT16_MAX validation");
                    println!("Checking if value {value} is within uint16_t range...");
                    if validate_uint16_range(value) != 0 {
                        println!(
                            "Value {0} is valid (0 <= value <= {1})",
                            value,
                            UINT16_MAX as u32
                        );
                        result = value;
                    } else {
                        println!("Value {value} is out of range for uint16_t");
                        result = -1;
                    }
                    println!("UINT16_MAX constant value: {0}", UINT16_MAX as u32);
                }
                1 => {
                    println!("Mode 1: String empty check by dereference");
                    if is_string_empty(test_string) != 0 {
                        println!("Test string is empty (checked with *string)");
                        result = 0;
                    } else {
                        println!("Test string is not empty");
                        result = 1;
                    }
                    if is_string_empty(non_empty_string) != 0 {
                        println!("Non-empty string check failed!");
                    } else {
                        println!("Non-empty string correctly identified");
                        result += 10;
                    }
                }
                2 => {
                    println!("Mode 2: Dynamic memory allocation and free");
                    let raw = create_buffer(bytemuck::cast_slice(b"Testing malloc and free\0"));
                    if !raw.is_null() {
                        println!(
                            "Buffer allocated: \'{0}\'",
                            std::ffi::CStr::from_ptr(raw as _).to_str().unwrap()
                        );
                        let len = std::ffi::CStr::from_ptr(raw as _).to_bytes().len();
                        println!("Buffer length: {0}", len as u64);
                        result = len as i32;
                        free(raw as *mut _);
                        println!("Buffer freed successfully");
                    } else {
                        println!("Failed to allocate buffer");
                        result = -1;
                    }
                }
                3 => {
                    println!("Mode 3: Function pointers with static counter");
                    let mut current_op: operation_func;

                    current_op = Some(reset_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, value);
                    println!("Counter reset to: {result}");

                    current_op = Some(increment_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, opt1);
                    println!("Counter after increment by {opt1}: {result}");

                    current_op = Some(multiply_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, opt2);
                    println!("Counter after multiply by {opt2}: {result}");

                    current_op = Some(decrement_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, 5);
                    println!("Counter after decrement by 5: {result}");

                    println!("Final static counter value: {0}", counter.get());
                }
                4 => {
                    println!("Mode 4: Using memchr to find character");
                    let raw = create_buffer(bytemuck::cast_slice(
                        b"Search for character X in this buffer\0",
                    ));
                    if !raw.is_null() {
                        let cstr = std::ffi::CStr::from_ptr(raw as _);
                        let buf_size = cstr.to_bytes().len();
                        let search_char: i8 = b'X' as i8;

                        println!(
                            "Searching for \'{0}\' in: \'{1}\'",
                            search_char as u8 as char,
                            cstr.to_str().unwrap()
                        );

                        let found_ptr = find_char_in_buffer(Some(&mut *raw), buf_size, search_char);
                        if !found_ptr.is_null() {
                            result = found_ptr.offset_from(raw) as i32;
                            println!(
                                "Found \'{0}\' at position: {1}",
                                search_char as u8 as char, result
                            );
                        } else {
                            println!("Character \'{0}\' not found", search_char as u8 as char);
                            result = -1;
                        }

                        free(raw as *mut _);
                    }
                }
                _ => {
                    println!("Invalid mode: {mode}");
                    result = -1;
                }
            }

            result
        }
    }
}
```

**Entity:** create_buffer() return value (*mut i8)

**States:** Null (allocation failed or empty input), Allocated (malloc-owned C string)

**Transitions:**
- Null -> (terminal) by returning ptr::null_mut() from create_buffer(initial.is_empty() or malloc failure)
- Allocated -> Freed via free(raw as *mut _)

**Evidence:** fn create_buffer(initial: &[i8]) -> *mut i8 returns ptr::null_mut() when initial.is_empty(); create_buffer(): `let raw = malloc(total) as *mut i8; if raw.is_null() { return ptr::null_mut(); }`; mode 2: `if !raw.is_null() { ... CStr::from_ptr(raw as _) ... free(raw as *mut _); } else { ... }`; mode 4: `if !raw.is_null() { let cstr = CStr::from_ptr(raw as _); ... free(raw as *mut _); }`; create_buffer(): `strcpy(raw, initial.as_ptr());` implies required NUL-termination of `initial` for safety/semantic correctness

**Implementation:** Introduce an owning wrapper `struct CBuffer(NonNull<i8>); impl Drop for CBuffer { fn drop(&mut self){ unsafe{ free(self.0.as_ptr() as *mut c_void) }}}` and make `create_buffer` return `Option<CBuffer>` (or `Result<CBuffer, AllocError>`). Provide safe methods like `as_c_str(&self) -> &CStr` and `as_mut_ptr(&mut self) -> *mut i8` to control raw access.

---

## Precondition Invariants

### 2. Buffer validity preconditions for memchr (Non-null + correct length + stable allocation)

**Location**: `/data/test_case/lib.rs:1-221`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: find_char_in_buffer relies on implicit preconditions: when `Some(&mut i8)` is passed, it must point to a live allocation covering at least `size` bytes for memchr to read; additionally the pointer must remain valid for the duration of the call. The function uses Option to encode only 'null vs non-null', but cannot enforce at compile time that `size` matches the actual buffer length or that the pointer is derived from a live allocation (e.g., the malloc buffer before free).

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
#![feature(as_array_of_cells)]

pub mod src {
    pub mod lib {
        use core::ffi::c_void;
        use core::ptr;

        extern "C" {
            fn malloc(__size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn memchr(__s: *const c_void, __c: i32, __n: usize) -> *mut c_void;
            fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
        }

        pub type operation_func = Option<unsafe extern "C" fn(i32) -> i32>;

        thread_local! {
            static counter: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
        }

        pub(crate) extern "C" fn increment_counter(value: i32) -> i32 {
            counter.set(counter.get() + value);
            counter.get()
        }

        pub const UINT16_MAX: i32 = 65535;

        pub(crate) extern "C" fn decrement_counter(value: i32) -> i32 {
            counter.set(counter.get() - value);
            counter.get()
        }

        pub(crate) extern "C" fn multiply_counter(value: i32) -> i32 {
            counter.set(counter.get() * value);
            counter.get()
        }

        pub(crate) extern "C" fn reset_counter(value: i32) -> i32 {
            counter.set(value);
            counter.get()
        }

        pub(crate) fn is_string_empty(str: Option<&i8>) -> i32 {
            match str {
                None => 1,
                Some(first) => (*first == 0) as i32,
            }
        }

        pub(crate) unsafe fn find_char_in_buffer(
            buffer: Option<&mut i8>,
            size: usize,
            target: i8,
        ) -> *const i8 {
            let Some(b) = buffer else {
                return ptr::null();
            };
            memchr(b as *mut i8 as *const c_void, target as i32, size) as *const i8
        }

        pub(crate) unsafe fn create_buffer(initial: &[i8]) -> *mut i8 {
            if initial.is_empty() {
                return ptr::null_mut();
            }

            // Determine C-string length (up to NUL). If no NUL, treat whole slice as content.
            let len = initial.iter().position(|&c| c == 0).unwrap_or(initial.len());
            let total = len.saturating_add(1);

            let raw = malloc(total) as *mut i8;
            if raw.is_null() {
                return ptr::null_mut();
            }

            // Copy using the original C routine to preserve behavior.
            strcpy(raw, initial.as_ptr());
            raw
        }

        pub(crate) fn validate_uint16_range(value: i32) -> i32 {
            (0..=UINT16_MAX).contains(&value) as i32
        }

        pub(crate) unsafe fn apply_operation(op: operation_func, value: i32) -> i32 {
            let Some(f) = op else { return -1 };
            f(value)
        }

        #[no_mangle]
        pub unsafe extern "C" fn charinbuf(mode: i32, value: i32, opt1: i32, opt2: i32) -> i32 {
            let mut result: i32 = 0;

            // Keep the original "Option<&i8>" style checks.
            let test_string: Option<&i8> = bytemuck::cast_slice(b"\0").first();
            let non_empty_string: Option<&i8> = bytemuck::cast_slice(b"Hello, World!\0").first();

            counter.set(0);

            match mode {
                0 => {
                    println!("Mode 0: UINT16_MAX validation");
                    println!("Checking if value {value} is within uint16_t range...");
                    if validate_uint16_range(value) != 0 {
                        println!(
                            "Value {0} is valid (0 <= value <= {1})",
                            value,
                            UINT16_MAX as u32
                        );
                        result = value;
                    } else {
                        println!("Value {value} is out of range for uint16_t");
                        result = -1;
                    }
                    println!("UINT16_MAX constant value: {0}", UINT16_MAX as u32);
                }
                1 => {
                    println!("Mode 1: String empty check by dereference");
                    if is_string_empty(test_string) != 0 {
                        println!("Test string is empty (checked with *string)");
                        result = 0;
                    } else {
                        println!("Test string is not empty");
                        result = 1;
                    }
                    if is_string_empty(non_empty_string) != 0 {
                        println!("Non-empty string check failed!");
                    } else {
                        println!("Non-empty string correctly identified");
                        result += 10;
                    }
                }
                2 => {
                    println!("Mode 2: Dynamic memory allocation and free");
                    let raw = create_buffer(bytemuck::cast_slice(b"Testing malloc and free\0"));
                    if !raw.is_null() {
                        println!(
                            "Buffer allocated: \'{0}\'",
                            std::ffi::CStr::from_ptr(raw as _).to_str().unwrap()
                        );
                        let len = std::ffi::CStr::from_ptr(raw as _).to_bytes().len();
                        println!("Buffer length: {0}", len as u64);
                        result = len as i32;
                        free(raw as *mut _);
                        println!("Buffer freed successfully");
                    } else {
                        println!("Failed to allocate buffer");
                        result = -1;
                    }
                }
                3 => {
                    println!("Mode 3: Function pointers with static counter");
                    let mut current_op: operation_func;

                    current_op = Some(reset_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, value);
                    println!("Counter reset to: {result}");

                    current_op = Some(increment_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, opt1);
                    println!("Counter after increment by {opt1}: {result}");

                    current_op = Some(multiply_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, opt2);
                    println!("Counter after multiply by {opt2}: {result}");

                    current_op = Some(decrement_counter as unsafe extern "C" fn(i32) -> i32);
                    result = apply_operation(current_op, 5);
                    println!("Counter after decrement by 5: {result}");

                    println!("Final static counter value: {0}", counter.get());
                }
                4 => {
                    println!("Mode 4: Using memchr to find character");
                    let raw = create_buffer(bytemuck::cast_slice(
                        b"Search for character X in this buffer\0",
                    ));
                    if !raw.is_null() {
                        let cstr = std::ffi::CStr::from_ptr(raw as _);
                        let buf_size = cstr.to_bytes().len();
                        let search_char: i8 = b'X' as i8;

                        println!(
                            "Searching for \'{0}\' in: \'{1}\'",
                            search_char as u8 as char,
                            cstr.to_str().unwrap()
                        );

                        let found_ptr = find_char_in_buffer(Some(&mut *raw), buf_size, search_char);
                        if !found_ptr.is_null() {
                            result = found_ptr.offset_from(raw) as i32;
                            println!(
                                "Found \'{0}\' at position: {1}",
                                search_char as u8 as char, result
                            );
                        } else {
                            println!("Character \'{0}\' not found", search_char as u8 as char);
                            result = -1;
                        }

                        free(raw as *mut _);
                    }
                }
                _ => {
                    println!("Invalid mode: {mode}");
                    result = -1;
                }
            }

            result
        }
    }
}
```

**Entity:** find_char_in_buffer(buffer: Option<&mut i8>, size: usize, target: i8)

**States:** Invalid (None / dangling / wrong size), Valid (points to at least `size` bytes)

**Transitions:**
- Invalid -> Valid by constructing a correctly-sized borrowed slice (conceptually) before calling memchr
- Valid -> Invalid if the backing allocation is freed or if `size` exceeds the actual allocation

**Evidence:** fn find_char_in_buffer(buffer: Option<&mut i8>, size: usize, target: i8) -> *const i8; find_char_in_buffer(): `memchr(b as *mut i8 as *const c_void, target as i32, size)` reads `size` bytes from `b`; mode 4: `let buf_size = cstr.to_bytes().len(); ... find_char_in_buffer(Some(&mut *raw), buf_size, search_char);` depends on buf_size matching the allocation and on `raw` still being allocated; mode 4: `free(raw as *mut _)` after search shows the temporal dependency (must not use found_ptr/raw after free)

**Implementation:** Change the API to accept a slice: `fn find_char_in_buffer(buf: &mut [u8], target: u8) -> Option<usize>` (or `&[u8]` if no mutation needed). This makes the length part of the type and removes the need for a separate `size` parameter and Option-as-null. If raw C interop is needed, accept `NonNull<u8>` plus a `usize` in a `struct BufferView<'a> { ptr: NonNull<u8>, len: usize, _lt: PhantomData<&'a mut [u8]> }` constructed only from slices/owners.

---

