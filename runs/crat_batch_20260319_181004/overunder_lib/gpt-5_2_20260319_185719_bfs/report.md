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

### 1. Single-element copy protocol (expects non-empty dest/src slices)

**Location**: `/data/test_case/lib.rs:1-195`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: copy_data_block() is written to copy exactly one DataBlock (element 0). It silently does nothing if either slice is empty. Callers that expect a copy must ensure both dest and src have length >= 1, but this is only enforced by a runtime conditional (get/get_mut) rather than the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataBlock, 1 free function(s)

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

        extern "C" {
            fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
            fn strncpy(__dest: *mut i8, __src: *const i8, __n: usize) -> *mut i8;
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct DataBlock {
            pub id: i32,
            pub value: f64,
            pub label: [i8; 20],
        }

        pub const __INT_MAX__: i32 = 2147483647;
        pub const INT_MAX: i32 = __INT_MAX__;
        pub const INT_MIN: i32 = -__INT_MAX__ - 1;

        pub(crate) fn safe_double_to_int(d: f64) -> i32 {
            if d.is_nan() {
                0
            } else if d > INT_MAX as f64 {
                INT_MAX
            } else if d < INT_MIN as f64 {
                INT_MIN
            } else {
                d as i32
            }
        }

        pub(crate) fn process_with_fallthrough(code: i32, mut base_value: i32) -> i32 {
            // Preserve original fall-through behavior:
            // 5 -> +50 then fallthrough to 4,3,2,1
            // 4 -> fallthrough to 3,2,1
            // 3 -> fallthrough to 2,1
            // 2 -> fallthrough to 1
            // 1 -> +10
            // 0 -> base=0
            // default -> base=-1
            match code {
                5 => {
                    base_value += 50;
                    base_value += 40;
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                4 => {
                    base_value += 40;
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                3 => {
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                2 => {
                    base_value += 20;
                    base_value += 10;
                }
                1 => {
                    base_value += 10;
                }
                0 => {
                    base_value = 0;
                }
                _ => {
                    base_value = -1;
                }
            }
            base_value
        }

        pub(crate) unsafe fn copy_data_block(dest: &mut [DataBlock], src: &[DataBlock]) {
            // Original code copies exactly one DataBlock.
            // Keep behavior but use safe Rust copy for the element.
            if let (Some(d), Some(s)) = (dest.get_mut(0), src.get(0)) {
                *d = *s;
            }
        }

        pub(crate) fn handle_pointer_operations(value: i32) -> i32 {
            let local_value = value * 2;
            let ptr: Option<&i32> = Some(&local_value);
            *ptr.unwrap() + 100
        }

        #[no_mangle]
        pub unsafe extern "C" fn overunder(a: i32, b: i32, c: i32, d: i32) -> i32 {
            let mut total: i32;

            let result_1: i32 = a;
            let result_2: i32 = b;
            println!("result_1 = {result_1}");
            println!("result_2 = {result_2}");

            let temp1: f64 = a as f64 * 1.5f64;
            let temp2: f64 = b as f64 * 2.7f64;
            let temp3: f64 = c as f64 / 3.3f64;
            let temp4: f64 = ((d * d + a * a) as f64).sqrt();

            let conv1: i32 = safe_double_to_int(temp1);
            let conv2: i32 = safe_double_to_int(temp2);
            let conv3: i32 = safe_double_to_int(temp3);
            let conv4: i32 = safe_double_to_int(temp4);
            println!("Converted values: {conv1}, {conv2}, {conv3}, {conv4}");

            let switch_result: i32 = process_with_fallthrough(a % 6, b);
            println!("Switch fall-through result: {switch_result}");

            let mut source_block: DataBlock = DataBlock {
                id: a,
                value: temp1,
                label: [0; 20],
            };

            // Keep FFI call for strncpy as in original.
            strncpy(
                source_block.label.as_mut_ptr(),
                b"Source\0" as *const u8 as *const i8,
                core::mem::size_of::<[i8; 20]>().wrapping_sub(1),
            );
            source_block.label[core::mem::size_of::<[i8; 20]>().wrapping_sub(1)] = 0;

            let mut dest_block: DataBlock = DataBlock {
                id: 0,
                value: 0.0,
                label: [0; 20],
            };

            copy_data_block(
                core::slice::from_mut(&mut dest_block),
                core::slice::from_ref(&source_block),
            );

            // Convert i8 label to bytes for CStr parsing.
            let label_bytes: [u8; 20] = core::array::from_fn(|i| dest_block.label[i] as u8);
            println!(
                "Copied block: id={0}, value={1:.2}, label={2}",
                dest_block.id,
                dest_block.value,
                core::ffi::CStr::from_bytes_until_nul(&label_bytes)
                    .unwrap()
                    .to_str()
                    .unwrap()
            );

            let ptr_result: i32 = handle_pointer_operations(c);
            println!("Pointer operation result: {ptr_result}");

            total = conv1 + conv2 + conv3 + conv4 + switch_result + ptr_result;
            total += dest_block.id;

            let overflow_test: f64 = 1e15f64;
            let safe_conv: i32 = safe_double_to_int(overflow_test);
            println!("Overflow protected conversion: {safe_conv}");

            let underflow_test: f64 = -1e15f64;
            let safe_conv2: i32 = safe_double_to_int(underflow_test);
            println!("Underflow protected conversion: {safe_conv2}");

            let array1: [i32; 5] = [a, b, c, d, a + b];
            let mut array2: [i32; 5] = [0; 5];

            // Replace memcpy-bytes with direct copy.
            array2.copy_from_slice(&array1);

            print!("Array copied via memcpy: ");
            for &v in &array2 {
                print!("{v} ");
                total += v;
            }
            println!();

            total
        }
    }
}
```

**Entity:** copy_data_block

**States:** EmptySlices, NonEmptySlices

**Transitions:**
- EmptySlices -> NonEmptySlices via caller providing slices with len >= 1

**Evidence:** copy_data_block comment: "Original code copies exactly one DataBlock."; copy_data_block: `if let (Some(d), Some(s)) = (dest.get_mut(0), src.get(0)) { *d = *s; }` — runtime gating on element 0 existing; overunder: passes `core::slice::from_mut(&mut dest_block)` and `core::slice::from_ref(&source_block)` which are guaranteed len==1 (relies on caller-side construction)

**Implementation:** Change signature to take `&mut DataBlock` and `&DataBlock` (since exactly one element is ever used), or accept `&mut [DataBlock; 1]` / `&[DataBlock; 1]` to encode the length-at-least-1 (and exactly-1) requirement at compile time.

---

## Protocol Invariants

### 2. C-string label validity invariant (NUL-terminated, valid bytes for display)

**Location**: `/data/test_case/lib.rs:1-195`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: DataBlock.label is treated as a C string in overunder(): it is expected to contain a NUL terminator within 20 bytes, and then to be valid UTF-8 for printing. These requirements are not enforced by the type of `label: [i8; 20]`; they are maintained by an ad-hoc protocol (strncpy + manual NUL + unwraps during parsing). Any other producer of DataBlock could violate these assumptions and cause panics (unwrap) or incorrect interpretation.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataBlock, 1 free function(s)

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

        extern "C" {
            fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
            fn strncpy(__dest: *mut i8, __src: *const i8, __n: usize) -> *mut i8;
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct DataBlock {
            pub id: i32,
            pub value: f64,
            pub label: [i8; 20],
        }

        pub const __INT_MAX__: i32 = 2147483647;
        pub const INT_MAX: i32 = __INT_MAX__;
        pub const INT_MIN: i32 = -__INT_MAX__ - 1;

        pub(crate) fn safe_double_to_int(d: f64) -> i32 {
            if d.is_nan() {
                0
            } else if d > INT_MAX as f64 {
                INT_MAX
            } else if d < INT_MIN as f64 {
                INT_MIN
            } else {
                d as i32
            }
        }

        pub(crate) fn process_with_fallthrough(code: i32, mut base_value: i32) -> i32 {
            // Preserve original fall-through behavior:
            // 5 -> +50 then fallthrough to 4,3,2,1
            // 4 -> fallthrough to 3,2,1
            // 3 -> fallthrough to 2,1
            // 2 -> fallthrough to 1
            // 1 -> +10
            // 0 -> base=0
            // default -> base=-1
            match code {
                5 => {
                    base_value += 50;
                    base_value += 40;
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                4 => {
                    base_value += 40;
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                3 => {
                    base_value += 30;
                    base_value += 20;
                    base_value += 10;
                }
                2 => {
                    base_value += 20;
                    base_value += 10;
                }
                1 => {
                    base_value += 10;
                }
                0 => {
                    base_value = 0;
                }
                _ => {
                    base_value = -1;
                }
            }
            base_value
        }

        pub(crate) unsafe fn copy_data_block(dest: &mut [DataBlock], src: &[DataBlock]) {
            // Original code copies exactly one DataBlock.
            // Keep behavior but use safe Rust copy for the element.
            if let (Some(d), Some(s)) = (dest.get_mut(0), src.get(0)) {
                *d = *s;
            }
        }

        pub(crate) fn handle_pointer_operations(value: i32) -> i32 {
            let local_value = value * 2;
            let ptr: Option<&i32> = Some(&local_value);
            *ptr.unwrap() + 100
        }

        #[no_mangle]
        pub unsafe extern "C" fn overunder(a: i32, b: i32, c: i32, d: i32) -> i32 {
            let mut total: i32;

            let result_1: i32 = a;
            let result_2: i32 = b;
            println!("result_1 = {result_1}");
            println!("result_2 = {result_2}");

            let temp1: f64 = a as f64 * 1.5f64;
            let temp2: f64 = b as f64 * 2.7f64;
            let temp3: f64 = c as f64 / 3.3f64;
            let temp4: f64 = ((d * d + a * a) as f64).sqrt();

            let conv1: i32 = safe_double_to_int(temp1);
            let conv2: i32 = safe_double_to_int(temp2);
            let conv3: i32 = safe_double_to_int(temp3);
            let conv4: i32 = safe_double_to_int(temp4);
            println!("Converted values: {conv1}, {conv2}, {conv3}, {conv4}");

            let switch_result: i32 = process_with_fallthrough(a % 6, b);
            println!("Switch fall-through result: {switch_result}");

            let mut source_block: DataBlock = DataBlock {
                id: a,
                value: temp1,
                label: [0; 20],
            };

            // Keep FFI call for strncpy as in original.
            strncpy(
                source_block.label.as_mut_ptr(),
                b"Source\0" as *const u8 as *const i8,
                core::mem::size_of::<[i8; 20]>().wrapping_sub(1),
            );
            source_block.label[core::mem::size_of::<[i8; 20]>().wrapping_sub(1)] = 0;

            let mut dest_block: DataBlock = DataBlock {
                id: 0,
                value: 0.0,
                label: [0; 20],
            };

            copy_data_block(
                core::slice::from_mut(&mut dest_block),
                core::slice::from_ref(&source_block),
            );

            // Convert i8 label to bytes for CStr parsing.
            let label_bytes: [u8; 20] = core::array::from_fn(|i| dest_block.label[i] as u8);
            println!(
                "Copied block: id={0}, value={1:.2}, label={2}",
                dest_block.id,
                dest_block.value,
                core::ffi::CStr::from_bytes_until_nul(&label_bytes)
                    .unwrap()
                    .to_str()
                    .unwrap()
            );

            let ptr_result: i32 = handle_pointer_operations(c);
            println!("Pointer operation result: {ptr_result}");

            total = conv1 + conv2 + conv3 + conv4 + switch_result + ptr_result;
            total += dest_block.id;

            let overflow_test: f64 = 1e15f64;
            let safe_conv: i32 = safe_double_to_int(overflow_test);
            println!("Overflow protected conversion: {safe_conv}");

            let underflow_test: f64 = -1e15f64;
            let safe_conv2: i32 = safe_double_to_int(underflow_test);
            println!("Underflow protected conversion: {safe_conv2}");

            let array1: [i32; 5] = [a, b, c, d, a + b];
            let mut array2: [i32; 5] = [0; 5];

            // Replace memcpy-bytes with direct copy.
            array2.copy_from_slice(&array1);

            print!("Array copied via memcpy: ");
            for &v in &array2 {
                print!("{v} ");
                total += v;
            }
            println!();

            total
        }
    }
}
```

**Entity:** DataBlock

**States:** LabelMayBeNonTerminatedOrInvalid, LabelIsNulTerminatedCString

**Transitions:**
- LabelMayBeNonTerminatedOrInvalid -> LabelIsNulTerminatedCString via `strncpy(...); label[19] = 0`

**Evidence:** DataBlock field: `pub label: [i8; 20]` — raw fixed buffer with no string validity encoded; overunder: `strncpy(source_block.label.as_mut_ptr(), b"Source\0"..., size_of::<[i8; 20]>() - 1)` followed by `source_block.label[... - 1] = 0` — manual NUL-termination protocol; overunder: converts and parses as C string: `CStr::from_bytes_until_nul(&label_bytes).unwrap().to_str().unwrap()` — panics if no NUL is found or if bytes are not UTF-8

**Implementation:** Represent the label as a dedicated type, e.g. `struct Label([u8; 20]);` with constructors that enforce NUL-termination (`CString`-like) and possibly UTF-8 (`struct Utf8Label(...)`). Expose `as_c_str()` returning `&CStr` (or fallible). Alternatively, store `label` as `[u8; 20]` and use `CStr`/`c_char` semantics explicitly to avoid `i8`/`u8` reinterpretation.

---

