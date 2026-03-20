# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. Non-null modifier callback requirement

**Location**: `/data/test_case/lib.rs:1-199`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `hatch` stores callbacks in `modifier_func` (an `Option`) and then immediately calls `.expect("non-null function pointer")`, implying the invariant that the callback must be non-null at the call point. This is enforced only by panicking, not by the type system. Since `hatch` itself always sets `Some(...)` before calling, the `None` state is effectively unreachable but still representable.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataRecord, 1 free function(s)

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

extern "C" {
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memmove(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
    fn memset(__s: *mut core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
    fn time(__timer: *mut time_t) -> time_t;
    fn difftime(__time1: time_t, __time0: time_t) -> f64;
}

pub type __time_t = i64;
pub type time_t = __time_t;

pub type operation_func = Option<unsafe extern "C" fn(i32, i32, i32) -> i32>;
pub type modifier_func = Option<unsafe extern "C" fn(i32, i32) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataRecord {
    pub id: i32,
    pub value: i32,
    pub timestamp: time_t,
    pub name: [i8; 32],
}

thread_local! {
    static global_counter: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
thread_local! {
    static global_accumulator: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) extern "C" fn increment_counter(value: i32, _: i32) {
    global_counter.set(global_counter.get() + value);
}

pub(crate) extern "C" fn update_accumulator(value: i32, _: i32) {
    global_accumulator.set(global_accumulator.get() * 2 + value);
}

pub(crate) unsafe fn apply_operation(op: operation_func, a: i32, b: i32, c: i32) -> i32 {
    op.expect("non-null function pointer")(a, b, c)
}

pub(crate) extern "C" fn add_three(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

pub(crate) extern "C" fn multiply_add(a: i32, b: i32, c: i32) -> i32 {
    a * b + c
}

pub(crate) extern "C" fn complex_calc(a: i32, b: i32, c: i32) -> i32 {
    (a - b) * c + global_counter.get()
}

pub(crate) unsafe fn shift_array_data(arr: *mut i32, size: i32, shift_by: i32) {
    if shift_by > 0 && shift_by < size {
        memmove(
            arr as *mut core::ffi::c_void,
            arr.offset(shift_by as isize) as *const core::ffi::c_void as *const std::ffi::c_void,
            ((size - shift_by) as usize).wrapping_mul(core::mem::size_of::<i32>()),
        );
        memset(
            arr.offset((size - shift_by) as isize) as *mut core::ffi::c_void,
            0,
            (shift_by as usize).wrapping_mul(core::mem::size_of::<i32>()),
        );
    }
}

pub(crate) fn process_pointer_data(ptr: Option<&i32>, multiplier: i32) -> i32 {
    let value: i32 = *ptr.unwrap();
    value * multiplier + global_accumulator.get()
}

pub(crate) unsafe fn compute_with_dynamic_memory(base: i32, count: i32) -> i32 {
    // Idiomatic: use Vec instead of malloc/free, but preserve logic.
    if count <= 0 {
        return 0;
    }
    let count_usize = count as usize;
    let mut temp_array = vec![0i32; count_usize];

    for i in 0..count {
        temp_array[i as usize] = base + i * 3;
    }

    temp_array.iter().take(count_usize).sum()
}

pub(crate) unsafe fn get_time_based_value(seed: i32) -> i32 {
    let mut current_time: time_t = 0;
    time(&raw mut current_time);
    let reference_time: time_t = current_time - (seed * 3600) as i64;
    let diff: f64 = difftime(current_time, reference_time);
    (diff / 100_f64) as i32 + seed
}

pub(crate) unsafe fn manipulate_records(
    records: *mut DataRecord,
    num_records: i32,
    shift: i32,
) -> i32 {
    let mut total: i32 = 0;
    if shift > 0 && shift < num_records {
        memmove(
            records as *mut core::ffi::c_void,
            records.offset(shift as isize) as *const core::ffi::c_void as *const std::ffi::c_void,
            ((num_records - shift) as usize).wrapping_mul(core::mem::size_of::<DataRecord>()),
        );
    }
    let mut i: i32 = 0;
    while i < num_records - shift {
        total += (*(records.offset(i as isize) as *const DataRecord)).value;
        i += 1;
    }
    total
}

#[no_mangle]
pub unsafe extern "C" fn hatch(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    let mut mod_func: modifier_func;
    mod_func = Some(increment_counter as unsafe extern "C" fn(i32, i32) -> ()) as modifier_func;
    mod_func.expect("non-null function pointer")(param1, 999);

    mod_func = Some(update_accumulator as unsafe extern "C" fn(i32, i32) -> ()) as modifier_func;
    mod_func.expect("non-null function pointer")(param2, 888);

    let mut op_func: operation_func;
    op_func = Some(add_three as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param1, param2, param3);

    op_func = Some(multiply_add as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param2, param3, param4);

    op_func = Some(complex_calc as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param1, param3, param4);

    // Use Vec instead of malloc/free; preserve values and shifting behavior.
    let mut dynamic_data: Vec<i32> = (0..10).map(|i| param1 + i).collect();

    result += process_pointer_data(dynamic_data.get(5), param2);

    shift_array_data(dynamic_data.as_mut_ptr(), 10, 3);
    result += dynamic_data[0];

    result += get_time_based_value(param3);

    // Use Vec for records; still call C time/snprintf to preserve formatting/FFI behavior.
    let mut records: Vec<DataRecord> = vec![
        DataRecord {
            id: 0,
            value: 0,
            timestamp: 0,
            name: [0; 32],
        };
        5
    ];

    for i in 0..5i32 {
        let rec = &mut records[i as usize];
        rec.id = i;
        rec.value = param4 + i * 10;
        time(&raw mut rec.timestamp);
        snprintf(
            rec.name.as_mut_ptr(),
            32,
            b"Record_%d\0" as *const u8 as *const i8,
            i,
        );
    }

    result += manipulate_records(records.as_mut_ptr(), 5, 2);

    result += compute_with_dynamic_memory(param1, 8);
    result += global_counter.get() + global_accumulator.get();

    result
}
```

**Entity:** modifier_func (Option<unsafe extern "C" fn(i32,i32)->()>) usage in hatch

**States:** Null (None), Valid (Some(fn))

**Transitions:**
- Null -> Valid by assigning `Some(increment_counter/update_accumulator)` before invocation

**Evidence:** type alias: `pub type modifier_func = Option<unsafe extern "C" fn(i32, i32) -> ()>;`; `hatch`: `mod_func = Some(increment_counter as ... ) as modifier_func; mod_func.expect("non-null function pointer")(param1, 999);`; `hatch`: `mod_func = Some(update_accumulator as ... ) as modifier_func; mod_func.expect("non-null function pointer")(param2, 888);`

**Implementation:** Replace `modifier_func` at internal call sites with a non-optional `unsafe extern "C" fn(i32,i32)` or wrap as `struct NonNullModifier(...)`. Keep the `Option` only at the FFI boundary if nullable pointers are genuinely possible.

---

### 2. Pointer + length + shift coupling (bounds/aliasing preconditions)

**Location**: `/data/test_case/lib.rs:1-199`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `shift_array_data` and `manipulate_records` take raw pointers plus a `size/num_records` and a `shift` and then perform pointer arithmetic and `memmove/memset`. Correctness requires an implicit invariant: the pointer must refer to at least `size` (or `num_records`) initialized elements, and `shift` must be in-bounds. The functions partially guard `shift` (`if shift_by > 0 && shift_by < size`), but they cannot ensure the pointer actually has the required provenance/length; this is left to the caller. These are slice-shaped APIs that could be made safe by taking `&mut [T]` and encoding the relationship between pointer and length in the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataRecord, 1 free function(s)

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

extern "C" {
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memmove(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
    fn memset(__s: *mut core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
    fn time(__timer: *mut time_t) -> time_t;
    fn difftime(__time1: time_t, __time0: time_t) -> f64;
}

pub type __time_t = i64;
pub type time_t = __time_t;

pub type operation_func = Option<unsafe extern "C" fn(i32, i32, i32) -> i32>;
pub type modifier_func = Option<unsafe extern "C" fn(i32, i32) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataRecord {
    pub id: i32,
    pub value: i32,
    pub timestamp: time_t,
    pub name: [i8; 32],
}

thread_local! {
    static global_counter: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
thread_local! {
    static global_accumulator: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) extern "C" fn increment_counter(value: i32, _: i32) {
    global_counter.set(global_counter.get() + value);
}

pub(crate) extern "C" fn update_accumulator(value: i32, _: i32) {
    global_accumulator.set(global_accumulator.get() * 2 + value);
}

pub(crate) unsafe fn apply_operation(op: operation_func, a: i32, b: i32, c: i32) -> i32 {
    op.expect("non-null function pointer")(a, b, c)
}

pub(crate) extern "C" fn add_three(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

pub(crate) extern "C" fn multiply_add(a: i32, b: i32, c: i32) -> i32 {
    a * b + c
}

pub(crate) extern "C" fn complex_calc(a: i32, b: i32, c: i32) -> i32 {
    (a - b) * c + global_counter.get()
}

pub(crate) unsafe fn shift_array_data(arr: *mut i32, size: i32, shift_by: i32) {
    if shift_by > 0 && shift_by < size {
        memmove(
            arr as *mut core::ffi::c_void,
            arr.offset(shift_by as isize) as *const core::ffi::c_void as *const std::ffi::c_void,
            ((size - shift_by) as usize).wrapping_mul(core::mem::size_of::<i32>()),
        );
        memset(
            arr.offset((size - shift_by) as isize) as *mut core::ffi::c_void,
            0,
            (shift_by as usize).wrapping_mul(core::mem::size_of::<i32>()),
        );
    }
}

pub(crate) fn process_pointer_data(ptr: Option<&i32>, multiplier: i32) -> i32 {
    let value: i32 = *ptr.unwrap();
    value * multiplier + global_accumulator.get()
}

pub(crate) unsafe fn compute_with_dynamic_memory(base: i32, count: i32) -> i32 {
    // Idiomatic: use Vec instead of malloc/free, but preserve logic.
    if count <= 0 {
        return 0;
    }
    let count_usize = count as usize;
    let mut temp_array = vec![0i32; count_usize];

    for i in 0..count {
        temp_array[i as usize] = base + i * 3;
    }

    temp_array.iter().take(count_usize).sum()
}

pub(crate) unsafe fn get_time_based_value(seed: i32) -> i32 {
    let mut current_time: time_t = 0;
    time(&raw mut current_time);
    let reference_time: time_t = current_time - (seed * 3600) as i64;
    let diff: f64 = difftime(current_time, reference_time);
    (diff / 100_f64) as i32 + seed
}

pub(crate) unsafe fn manipulate_records(
    records: *mut DataRecord,
    num_records: i32,
    shift: i32,
) -> i32 {
    let mut total: i32 = 0;
    if shift > 0 && shift < num_records {
        memmove(
            records as *mut core::ffi::c_void,
            records.offset(shift as isize) as *const core::ffi::c_void as *const std::ffi::c_void,
            ((num_records - shift) as usize).wrapping_mul(core::mem::size_of::<DataRecord>()),
        );
    }
    let mut i: i32 = 0;
    while i < num_records - shift {
        total += (*(records.offset(i as isize) as *const DataRecord)).value;
        i += 1;
    }
    total
}

#[no_mangle]
pub unsafe extern "C" fn hatch(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    let mut mod_func: modifier_func;
    mod_func = Some(increment_counter as unsafe extern "C" fn(i32, i32) -> ()) as modifier_func;
    mod_func.expect("non-null function pointer")(param1, 999);

    mod_func = Some(update_accumulator as unsafe extern "C" fn(i32, i32) -> ()) as modifier_func;
    mod_func.expect("non-null function pointer")(param2, 888);

    let mut op_func: operation_func;
    op_func = Some(add_three as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param1, param2, param3);

    op_func = Some(multiply_add as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param2, param3, param4);

    op_func = Some(complex_calc as unsafe extern "C" fn(i32, i32, i32) -> i32) as operation_func;
    result += apply_operation(op_func, param1, param3, param4);

    // Use Vec instead of malloc/free; preserve values and shifting behavior.
    let mut dynamic_data: Vec<i32> = (0..10).map(|i| param1 + i).collect();

    result += process_pointer_data(dynamic_data.get(5), param2);

    shift_array_data(dynamic_data.as_mut_ptr(), 10, 3);
    result += dynamic_data[0];

    result += get_time_based_value(param3);

    // Use Vec for records; still call C time/snprintf to preserve formatting/FFI behavior.
    let mut records: Vec<DataRecord> = vec![
        DataRecord {
            id: 0,
            value: 0,
            timestamp: 0,
            name: [0; 32],
        };
        5
    ];

    for i in 0..5i32 {
        let rec = &mut records[i as usize];
        rec.id = i;
        rec.value = param4 + i * 10;
        time(&raw mut rec.timestamp);
        snprintf(
            rec.name.as_mut_ptr(),
            32,
            b"Record_%d\0" as *const u8 as *const i8,
            i,
        );
    }

    result += manipulate_records(records.as_mut_ptr(), 5, 2);

    result += compute_with_dynamic_memory(param1, 8);
    result += global_counter.get() + global_accumulator.get();

    result
}
```

**Entity:** shift_array_data / manipulate_records pointer+length API

**States:** No-op (shift<=0 or shift>=size), Shifted (0 < shift < size with sufficient backing storage)

**Transitions:**
- No-op -> Shifted by providing shift such that 0 < shift < size/num_records
- Shifted -> Shifted via repeated calls with valid parameters (state is in the memory region, not the type)

**Evidence:** `shift_array_data(arr: *mut i32, size: i32, shift_by: i32)`: uses `arr.offset(shift_by as isize)` and `memmove(...)`/`memset(...)` based on `(size - shift_by)`; `shift_array_data`: guard `if shift_by > 0 && shift_by < size { ... }` indicates an in-bounds requirement for the shift; `manipulate_records(records: *mut DataRecord, num_records: i32, shift: i32)`: uses `records.offset(shift as isize)` inside `memmove(...)` and then reads `*(records.offset(i as isize) ...)` up to `num_records - shift`; call sites in `hatch`: `shift_array_data(dynamic_data.as_mut_ptr(), 10, 3);` and `manipulate_records(records.as_mut_ptr(), 5, 2);` rely on Vec providing enough backing storage

**Implementation:** Make these APIs accept slices: `fn shift_array_data(arr: &mut [i32], shift_by: usize)` and `fn manipulate_records(records: &mut [DataRecord], shift: usize)`. This enforces pointer/length coupling and removes most `unsafe`. If shift constraints are richer, use a validated newtype like `struct InBoundsShift(usize)` created via `TryFrom<(usize, usize)>` (shift, len).

---

