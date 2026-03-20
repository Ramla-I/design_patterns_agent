# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. Computation history lifecycle & bounds protocol (Null/Allocated, Count within Capacity)

**Location**: `/data/test_case/lib.rs:1-174`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The thread-local history buffer is a manually-managed heap allocation whose initialization and valid-use conditions are enforced only by runtime null checks and a parallel count variable. The code assumes: (1) the pointer is either null (unallocated) or points to an array of exactly 10 ComputationResult elements allocated by allocate_results(10); (2) history_count tracks how many entries have been written and must stay within [0,10]; (3) allocation happens at most once per thread and is never freed (leak). None of these are represented in the type system because the state is carried as a raw pointer (*mut ComputationResult) plus an i32 count, both behind RefCell.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ComputationResult, 2 free function(s)

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
    fn calloc(__nmemb: usize, __size: usize) -> *mut core::ffi::c_void;
    fn time(__timer: *mut time_t) -> time_t;
}

pub type __time_t = i64;
pub type time_t = __time_t;

pub type Operation = u32;
pub type StatusCode = i32;

pub const STATUS_SUCCESS: StatusCode = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputationResult {
    pub value: i32,
    pub timestamp: time_t,
    pub status: StatusCode,
}

pub type MathOperation = Option<unsafe extern "C" fn(i32, i32, i32) -> i32>;

pub(crate) fn is_valid_operation(op_char: i8) -> bool {
    // Avoid non-const expressions in patterns; keep it simple and portable.
    op_char >= b'1' as i8 && op_char <= b'5' as i8
}

pub(crate) fn get_operation_priority(op: Operation) -> i32 {
    op.wrapping_mul(10) as i32
}

pub(crate) extern "C" fn add_operation(a: i32, b: i32, _: i32) -> i32 {
    a + b
}
pub(crate) extern "C" fn multiply_operation(a: i32, b: i32, _: i32) -> i32 {
    a * b
}
pub(crate) extern "C" fn subtract_operation(a: i32, b: i32, _: i32) -> i32 {
    a - b
}
pub(crate) extern "C" fn divide_operation(a: i32, b: i32, _: i32) -> i32 {
    if b == 0 { 0 } else { a / b }
}
pub(crate) extern "C" fn modulo_operation(a: i32, b: i32, _: i32) -> i32 {
    if b == 0 { 0 } else { a % b }
}

pub(crate) fn select_operation(op: Operation) -> MathOperation {
    let f: unsafe extern "C" fn(i32, i32, i32) -> i32 = match op {
        1 => add_operation,
        2 => multiply_operation,
        3 => subtract_operation,
        4 => divide_operation,
        5 => modulo_operation,
        _ => add_operation,
    };
    Some(f)
}

pub(crate) unsafe fn get_computation_timestamp() -> time_t {
    let mut current_time: time_t = 0;
    time(&raw mut current_time);
    current_time >>= 29;
    current_time
}

pub(crate) unsafe fn allocate_results(count: i32) -> *mut ComputationResult {
    let ptr = calloc(count as usize, core::mem::size_of::<ComputationResult>());
    ptr as *mut ComputationResult
}

pub(crate) unsafe fn perform_computation_with_history(
    a: i32,
    b: i32,
    op: Operation,
    mut history: Option<&mut *mut ComputationResult>,
    mut history_count: Option<&mut i32>,
) -> i32 {
    let math_func: MathOperation = select_operation(op);
    let result: i32 = math_func.expect("non-null function pointer")(a, b, 0);

    let history_ptr_ref: &mut *mut ComputationResult = history.as_deref_mut().unwrap();
    let history_count_ref: &mut i32 = history_count.as_deref_mut().unwrap();

    if (*history_ptr_ref).is_null() {
        *history_ptr_ref = allocate_results(10);
        *history_count_ref = 0;
    }

    if *history_count_ref < 10 {
        let idx = *history_count_ref as isize;
        let entry = &mut *(*history_ptr_ref).offset(idx);
        entry.value = result;
        entry.timestamp = get_computation_timestamp();
        entry.status = STATUS_SUCCESS;
        *history_count_ref += 1;
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn mathop(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    thread_local! {
        static computation_history: std::cell::RefCell<*mut ComputationResult> = const {
            std::cell::RefCell::new(core::ptr::null_mut())
        };
    };
    thread_local! {
        static history_count: std::cell::RefCell<i32> = const {
            std::cell::RefCell::new(0)
        };
    };

    let validation_char: i8 = (param1 % 128) as i8;
    let _is_valid: bool = is_valid_operation(validation_char);

    let selected_op: Operation = (param3 % 5 + 1) as u32;
    let operation_priority: i32 = get_operation_priority(selected_op);

    let intermediate_result: i32 = computation_history.with_borrow_mut(|computation_history_ref| {
        history_count.with_borrow_mut(|history_count_ref| {
            perform_computation_with_history(
                param1,
                param2,
                selected_op,
                Some(computation_history_ref),
                Some(history_count_ref),
            )
        })
    });

    let second_op: Operation = ((param4 + 1) % 5 + 1) as u32;
    let mut final_result: i32 = computation_history.with_borrow_mut(|computation_history_ref| {
        history_count.with_borrow_mut(|history_count_ref| {
            perform_computation_with_history(
                intermediate_result,
                param4,
                second_op,
                Some(computation_history_ref),
                Some(history_count_ref),
            )
        })
    });

    final_result += operation_priority;

    let computation_time: time_t = get_computation_timestamp();
    let time_modifier: i32 = (computation_time % 100) as i32;
    final_result += time_modifier;

    println!("Computation performed at timestamp: {computation_time}");
    println!("Operation priority: {operation_priority}");
    history_count.with_borrow(|history_count_ref| {
        println!("History entries: {0}", *history_count_ref)
    });
    println!("Final result: {final_result}");

    final_result
}
```

**Entity:** thread_local computation_history: RefCell<*mut ComputationResult> + history_count: RefCell<i32>

**States:** Unallocated (ptr = null, count = 0), Allocated (ptr != null, 0 <= count <= 10), Saturated (ptr != null, count = 10; no further writes)

**Transitions:**
- Unallocated -> Allocated via perform_computation_with_history(): if (*history_ptr_ref).is_null() { *history_ptr_ref = allocate_results(10); *history_count_ref = 0; }
- Allocated -> Saturated via perform_computation_with_history(): repeated calls increment *history_count_ref until it reaches 10
- Allocated/Saturated -> Allocated/Saturated via perform_computation_with_history(): subsequent calls reuse the same allocation; writes only if count < 10

**Evidence:** thread_local! static computation_history: RefCell<*mut ComputationResult> initialized to core::ptr::null_mut(); thread_local! static history_count: RefCell<i32> initialized to 0; perform_computation_with_history(): history.as_deref_mut().unwrap() and history_count.as_deref_mut().unwrap() assume these are always Some(...) at call sites; perform_computation_with_history(): if (*history_ptr_ref).is_null() { *history_ptr_ref = allocate_results(10); *history_count_ref = 0; } encodes the Unallocated -> Allocated transition; perform_computation_with_history(): if *history_count_ref < 10 { ... entry = &mut *(*history_ptr_ref).offset(idx); ... *history_count_ref += 1; } encodes bounds + write protocol and relies on count being consistent with allocation capacity; allocate_results(count): uses calloc(count as usize, size_of::<ComputationResult>()) and returns a raw pointer with no ownership/freeing mechanism

**Implementation:** Replace the raw pointer + count with an owned buffer type stored in TLS, e.g. RefCell<Option<HistoryBuf>> where HistoryBuf holds Box<[ComputationResult; 10]> (or Vec<ComputationResult> with fixed capacity) and a usize len. Expose methods push(result) that enforces len <= 10. Drop for HistoryBuf handles cleanup; eliminating null checks and pointer arithmetic. If FFI requires a pointer, provide a method as_ptr()/as_mut_ptr() that borrows the internal buffer.

---

