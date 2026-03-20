# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. FFI heap allocation & initialization lifecycle for ComputeState (Allocated -> Initialized -> Freed)

**Location**: `/data/test_case/lib.rs:1-228`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: checkshift manually manages a heap-allocated ComputeState via malloc/free and then treats it as a 1-element slice. Correctness relies on (1) malloc succeeding, (2) init_state being called before any reads/writes of state[0], and (3) free being called exactly once on the same pointer after all uses. These are enforced with runtime null checks and call ordering in checkshift, not by the type system; nothing prevents forgetting init_state, using after free, double-free, or early returns that leak (future changes could add new early returns).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ComputeState, 2 free function(s)

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputeState {
    pub accumulator: i32,
    pub operation_count: i32,
    pub checksum: u32,
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

pub const MAGIC_NUMBER: u32 = 0xdeadbeefu32;
pub const MASK_LOWER: i32 = 0xffff;

static static_multiplier: i32 = 3;
static static_addend: i32 = 100;
static static_shift_amount: i32 = 2;

pub(crate) extern "C" fn multiply_with_static(a: i32, b: i32) -> i32 {
    a * b * static_multiplier
}
pub(crate) extern "C" fn add_with_static(a: i32, b: i32) -> i32 {
    a + b + static_addend
}
pub(crate) extern "C" fn xor_operation(a: i32, b: i32) -> i32 {
    a ^ b ^ 0xabcd
}
pub(crate) extern "C" fn shift_with_static(a: i32, b: i32) -> i32 {
    a << static_shift_amount | b >> static_shift_amount
}

pub(crate) fn get_operation(opcode: i32) -> operation_func {
    thread_local! {
        static OPS: std::cell::Cell<[operation_func; 4]> = const {
            std::cell::Cell::new([None, None, None, None])
        };
    }

    OPS.with(|cell| {
        let ops = cell.as_array_of_cells();
        if ops[0].get().is_none() {
            ops[0].set(Some(multiply_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[1].set(Some(add_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[2].set(Some(xor_operation as unsafe extern "C" fn(i32, i32) -> i32));
            ops[3].set(Some(shift_with_static as unsafe extern "C" fn(i32, i32) -> i32));
        }

        if (0..4).contains(&opcode) {
            ops[opcode as usize].get()
        } else {
            None
        }
    })
}

pub(crate) unsafe fn execute_operation(
    func: operation_func,
    a: i32,
    b: i32,
    op_name: &[i8],
) -> i32 {
    let Some(func) = func else {
        println!(
            "Error: Operation function pointer is NULL for {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return 0;
    };

    println!("Variable a = {a}");
    println!("Variable b = {b}");
    let result = func(a, b);
    println!(
        "Result of {0}: {1}",
        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
            .unwrap()
            .to_str()
            .unwrap(),
        result
    );
    result
}

pub(crate) fn compute_checksum(values: &[i32], count: i32) -> u32 {
    let mut checksum: u32 = 0;
    let mut buffer: [u8; 16] = [0; 16];

    if !values.is_empty() && count > 0 {
        let copy_count = (count.min(4)) as usize;
        let byte_len = core::mem::size_of::<i32>() * copy_count;

        buffer[..byte_len].copy_from_slice(&bytemuck::cast_slice(values)[..byte_len]);

        for &byte in &buffer[..byte_len] {
            checksum = (checksum << 1) ^ (byte as u32);
        }
        checksum ^= MAGIC_NUMBER;
    }

    checksum & (MASK_LOWER as u32)
}

pub(crate) unsafe fn init_state(state: &mut [ComputeState], initial_value: i32) {
    if state.is_empty() {
        println!("Error: state pointer is NULL in init_state");
        return;
    }

    let template = ComputeState {
        accumulator: initial_value,
        operation_count: 0,
        checksum: 0,
    };

    // Preserve original behavior (memcpy) while keeping the rest idiomatic.
    memcpy(
        state.as_mut_ptr() as *mut core::ffi::c_void,
        core::ptr::addr_of!(template) as *const core::ffi::c_void,
        core::mem::size_of::<ComputeState>(),
    );

    println!(
        "State initialized with accumulator = {0}",
        state[0].accumulator
    );
}

pub(crate) unsafe fn apply_operation(
    mut state: Option<&mut ComputeState>,
    value: i32,
    func: operation_func,
) {
    let Some(st) = state.as_deref_mut() else {
        println!("Error: state pointer is NULL in apply_operation");
        return;
    };
    let Some(func) = func else {
        println!("Error: operation function pointer is NULL in apply_operation");
        return;
    };

    st.accumulator = func(st.accumulator, value);
    st.operation_count += 1;
}

#[no_mangle]
pub unsafe extern "C" fn checkshift(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    print!("\n=== Starting foo function ===\n");
    println!("Parameters: {param1}, {param2}, {param3}, {param4}");

    // Allocate exactly one ComputeState; keep malloc/free at the FFI boundary.
    let state_ptr = malloc(core::mem::size_of::<ComputeState>()) as *mut ComputeState;
    if state_ptr.is_null() {
        println!("Error: Failed to allocate memory for state");
        return -1;
    }
    let state: &mut [ComputeState] = core::slice::from_raw_parts_mut(state_ptr, 1);

    init_state(state, param1);

    let params: [i32; 4] = [param1, param2, param3, param4];
    let mult_op: operation_func = get_operation(0);
    let add_op: operation_func = get_operation(1);
    let xor_op: operation_func = get_operation(2);
    let shift_op: operation_func = get_operation(3);

    print!("\n--- Operation 1: Multiply ---\n");
    apply_operation(state.first_mut(), param2, mult_op);

    print!("\n--- Operation 2: Add ---\n");
    apply_operation(state.first_mut(), param3, add_op);

    print!("\n--- Operation 3: XOR ---\n");
    let xor_result: i32 = execute_operation(
        xor_op,
        state[0].accumulator,
        param4,
        bytemuck::cast_slice(b"XOR\0"),
    );

    print!("\n--- Operation 4: Shift ---\n");
    let shift_result: i32 = execute_operation(
        shift_op,
        xor_result,
        param2,
        bytemuck::cast_slice(b"SHIFT\0"),
    );

    state[0].checksum = compute_checksum(&params, 4);
    print!("\nComputed checksum: 0x{0:>04X}\n", state[0].checksum & 0xFFFF);

    let final_result: i32 =
        (((state[0].accumulator + shift_result) as u32) ^ state[0].checksum) as i32;

    print!("\nFinal accumulator: {0}\n", state[0].accumulator);
    println!("Operation count: {0}", state[0].operation_count);
    println!("Final result: {final_result}");

    free(state_ptr as *mut core::ffi::c_void);

    print!("=== Ending foo function ===\n\n");
    final_result
}
```

**Entity:** checkshift (and its raw state_ptr allocation)

**States:** Unallocated, Allocated(Uninitialized), Initialized, Freed

**Transitions:**
- Unallocated -> Allocated(Uninitialized) via malloc(size_of::<ComputeState>()) in checkshift
- Allocated(Uninitialized) -> Initialized via init_state(state, param1)
- Initialized -> Freed via free(state_ptr as *mut c_void)

**Evidence:** checkshift: `let state_ptr = malloc(size_of::<ComputeState>()) as *mut ComputeState;` (manual allocation); checkshift: `if state_ptr.is_null() { ... return -1; }` (runtime allocation failure path); checkshift: `let state: &mut [ComputeState] = core::slice::from_raw_parts_mut(state_ptr, 1);` (raw pointer -> slice, assumes valid for len 1); checkshift: `init_state(state, param1);` must occur before later `state[0].accumulator` / `state[0].checksum` accesses; checkshift: `free(state_ptr as *mut core::ffi::c_void);` (manual free must match malloc)

**Implementation:** Introduce an owning wrapper like `struct StateBox(NonNull<ComputeState>); impl Drop for StateBox { free(...) }` and provide safe constructors: `StateBox::new(initial_value) -> Result<StateBox, AllocError>` that calls malloc + init_state internally. Expose `fn as_mut(&mut self) -> &mut ComputeState`. This makes initialization and freeing automatic and prevents use-after-free via borrow checking.

---

## Precondition Invariants

### 3. Opcode-to-operation capability (Valid opcode -> Non-null function pointer)

**Location**: `/data/test_case/lib.rs:1-228`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Operations are selected by an i32 opcode and represented as `Option<unsafe extern "C" fn(...)>`. Callers must ensure the opcode is in 0..4 (or otherwise ensure the function pointer is Some) before attempting to execute/apply the operation. The code currently relies on runtime checks (`if (0..4).contains(&opcode)` and `let Some(func) = func else { ... }`) plus error-print-and-return behavior, rather than making invalid opcodes unrepresentable.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ComputeState, 2 free function(s)

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputeState {
    pub accumulator: i32,
    pub operation_count: i32,
    pub checksum: u32,
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

pub const MAGIC_NUMBER: u32 = 0xdeadbeefu32;
pub const MASK_LOWER: i32 = 0xffff;

static static_multiplier: i32 = 3;
static static_addend: i32 = 100;
static static_shift_amount: i32 = 2;

pub(crate) extern "C" fn multiply_with_static(a: i32, b: i32) -> i32 {
    a * b * static_multiplier
}
pub(crate) extern "C" fn add_with_static(a: i32, b: i32) -> i32 {
    a + b + static_addend
}
pub(crate) extern "C" fn xor_operation(a: i32, b: i32) -> i32 {
    a ^ b ^ 0xabcd
}
pub(crate) extern "C" fn shift_with_static(a: i32, b: i32) -> i32 {
    a << static_shift_amount | b >> static_shift_amount
}

pub(crate) fn get_operation(opcode: i32) -> operation_func {
    thread_local! {
        static OPS: std::cell::Cell<[operation_func; 4]> = const {
            std::cell::Cell::new([None, None, None, None])
        };
    }

    OPS.with(|cell| {
        let ops = cell.as_array_of_cells();
        if ops[0].get().is_none() {
            ops[0].set(Some(multiply_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[1].set(Some(add_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[2].set(Some(xor_operation as unsafe extern "C" fn(i32, i32) -> i32));
            ops[3].set(Some(shift_with_static as unsafe extern "C" fn(i32, i32) -> i32));
        }

        if (0..4).contains(&opcode) {
            ops[opcode as usize].get()
        } else {
            None
        }
    })
}

pub(crate) unsafe fn execute_operation(
    func: operation_func,
    a: i32,
    b: i32,
    op_name: &[i8],
) -> i32 {
    let Some(func) = func else {
        println!(
            "Error: Operation function pointer is NULL for {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return 0;
    };

    println!("Variable a = {a}");
    println!("Variable b = {b}");
    let result = func(a, b);
    println!(
        "Result of {0}: {1}",
        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
            .unwrap()
            .to_str()
            .unwrap(),
        result
    );
    result
}

pub(crate) fn compute_checksum(values: &[i32], count: i32) -> u32 {
    let mut checksum: u32 = 0;
    let mut buffer: [u8; 16] = [0; 16];

    if !values.is_empty() && count > 0 {
        let copy_count = (count.min(4)) as usize;
        let byte_len = core::mem::size_of::<i32>() * copy_count;

        buffer[..byte_len].copy_from_slice(&bytemuck::cast_slice(values)[..byte_len]);

        for &byte in &buffer[..byte_len] {
            checksum = (checksum << 1) ^ (byte as u32);
        }
        checksum ^= MAGIC_NUMBER;
    }

    checksum & (MASK_LOWER as u32)
}

pub(crate) unsafe fn init_state(state: &mut [ComputeState], initial_value: i32) {
    if state.is_empty() {
        println!("Error: state pointer is NULL in init_state");
        return;
    }

    let template = ComputeState {
        accumulator: initial_value,
        operation_count: 0,
        checksum: 0,
    };

    // Preserve original behavior (memcpy) while keeping the rest idiomatic.
    memcpy(
        state.as_mut_ptr() as *mut core::ffi::c_void,
        core::ptr::addr_of!(template) as *const core::ffi::c_void,
        core::mem::size_of::<ComputeState>(),
    );

    println!(
        "State initialized with accumulator = {0}",
        state[0].accumulator
    );
}

pub(crate) unsafe fn apply_operation(
    mut state: Option<&mut ComputeState>,
    value: i32,
    func: operation_func,
) {
    let Some(st) = state.as_deref_mut() else {
        println!("Error: state pointer is NULL in apply_operation");
        return;
    };
    let Some(func) = func else {
        println!("Error: operation function pointer is NULL in apply_operation");
        return;
    };

    st.accumulator = func(st.accumulator, value);
    st.operation_count += 1;
}

#[no_mangle]
pub unsafe extern "C" fn checkshift(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    print!("\n=== Starting foo function ===\n");
    println!("Parameters: {param1}, {param2}, {param3}, {param4}");

    // Allocate exactly one ComputeState; keep malloc/free at the FFI boundary.
    let state_ptr = malloc(core::mem::size_of::<ComputeState>()) as *mut ComputeState;
    if state_ptr.is_null() {
        println!("Error: Failed to allocate memory for state");
        return -1;
    }
    let state: &mut [ComputeState] = core::slice::from_raw_parts_mut(state_ptr, 1);

    init_state(state, param1);

    let params: [i32; 4] = [param1, param2, param3, param4];
    let mult_op: operation_func = get_operation(0);
    let add_op: operation_func = get_operation(1);
    let xor_op: operation_func = get_operation(2);
    let shift_op: operation_func = get_operation(3);

    print!("\n--- Operation 1: Multiply ---\n");
    apply_operation(state.first_mut(), param2, mult_op);

    print!("\n--- Operation 2: Add ---\n");
    apply_operation(state.first_mut(), param3, add_op);

    print!("\n--- Operation 3: XOR ---\n");
    let xor_result: i32 = execute_operation(
        xor_op,
        state[0].accumulator,
        param4,
        bytemuck::cast_slice(b"XOR\0"),
    );

    print!("\n--- Operation 4: Shift ---\n");
    let shift_result: i32 = execute_operation(
        shift_op,
        xor_result,
        param2,
        bytemuck::cast_slice(b"SHIFT\0"),
    );

    state[0].checksum = compute_checksum(&params, 4);
    print!("\nComputed checksum: 0x{0:>04X}\n", state[0].checksum & 0xFFFF);

    let final_result: i32 =
        (((state[0].accumulator + shift_result) as u32) ^ state[0].checksum) as i32;

    print!("\nFinal accumulator: {0}\n", state[0].accumulator);
    println!("Operation count: {0}", state[0].operation_count);
    println!("Final result: {final_result}");

    free(state_ptr as *mut core::ffi::c_void);

    print!("=== Ending foo function ===\n\n");
    final_result
}
```

**Entity:** operation_func (and get_operation/execute_operation/apply_operation)

**States:** Unknown/Unchecked(opcode or func may be invalid), ValidOperation(guaranteed Some(fn)), InvalidOperation(None)

**Transitions:**
- Unknown/Unchecked -> ValidOperation via `get_operation(opcode)` when `(0..4).contains(&opcode)` producing `Some(fn)`
- Unknown/Unchecked -> InvalidOperation via `get_operation(opcode)` when opcode out of range producing `None`
- ValidOperation -> (used) via `execute_operation(func, ...)` / `apply_operation(..., func)`

**Evidence:** type alias: `pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;` (nullability encoded at runtime); get_operation: `if (0..4).contains(&opcode) { ops[opcode as usize].get() } else { None }` (opcode range precondition); execute_operation: `let Some(func) = func else { println!("Error: Operation function pointer is NULL ..."); return 0; };` (runtime guard + sentinel return); apply_operation: `let Some(func) = func else { println!("Error: operation function pointer is NULL in apply_operation"); return; };` (runtime guard)

**Implementation:** Define `enum Opcode { Mul, Add, Xor, Shift }` (or `struct Opcode(u8)` with `TryFrom<i32>`), and return a non-optional function pointer: `fn get_operation(op: Opcode) -> unsafe extern "C" fn(i32,i32)->i32`. Alternatively return a capability `struct Operation(unsafe extern "C" fn(i32,i32)->i32);` constructed only from valid opcodes, eliminating `Option` and the NULL-path at call sites.

---

## Protocol Invariants

### 2. ComputeState usage protocol (Initialized before use; operation_count/checksum coherence)

**Location**: `/data/test_case/lib.rs:1-228`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ComputeState has an implicit protocol: it must be initialized (accumulator set, counters reset) before any operations are applied; then apply_operation mutates accumulator and increments operation_count; later checksum is computed and written before producing the final_result. The type system does not distinguish these phases, so code could accidentally call apply_operation on uninitialized memory or compute final_result before checksum is set, relying only on the current call ordering in checkshift.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ComputeState, 2 free function(s)

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputeState {
    pub accumulator: i32,
    pub operation_count: i32,
    pub checksum: u32,
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

pub const MAGIC_NUMBER: u32 = 0xdeadbeefu32;
pub const MASK_LOWER: i32 = 0xffff;

static static_multiplier: i32 = 3;
static static_addend: i32 = 100;
static static_shift_amount: i32 = 2;

pub(crate) extern "C" fn multiply_with_static(a: i32, b: i32) -> i32 {
    a * b * static_multiplier
}
pub(crate) extern "C" fn add_with_static(a: i32, b: i32) -> i32 {
    a + b + static_addend
}
pub(crate) extern "C" fn xor_operation(a: i32, b: i32) -> i32 {
    a ^ b ^ 0xabcd
}
pub(crate) extern "C" fn shift_with_static(a: i32, b: i32) -> i32 {
    a << static_shift_amount | b >> static_shift_amount
}

pub(crate) fn get_operation(opcode: i32) -> operation_func {
    thread_local! {
        static OPS: std::cell::Cell<[operation_func; 4]> = const {
            std::cell::Cell::new([None, None, None, None])
        };
    }

    OPS.with(|cell| {
        let ops = cell.as_array_of_cells();
        if ops[0].get().is_none() {
            ops[0].set(Some(multiply_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[1].set(Some(add_with_static as unsafe extern "C" fn(i32, i32) -> i32));
            ops[2].set(Some(xor_operation as unsafe extern "C" fn(i32, i32) -> i32));
            ops[3].set(Some(shift_with_static as unsafe extern "C" fn(i32, i32) -> i32));
        }

        if (0..4).contains(&opcode) {
            ops[opcode as usize].get()
        } else {
            None
        }
    })
}

pub(crate) unsafe fn execute_operation(
    func: operation_func,
    a: i32,
    b: i32,
    op_name: &[i8],
) -> i32 {
    let Some(func) = func else {
        println!(
            "Error: Operation function pointer is NULL for {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return 0;
    };

    println!("Variable a = {a}");
    println!("Variable b = {b}");
    let result = func(a, b);
    println!(
        "Result of {0}: {1}",
        std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(op_name))
            .unwrap()
            .to_str()
            .unwrap(),
        result
    );
    result
}

pub(crate) fn compute_checksum(values: &[i32], count: i32) -> u32 {
    let mut checksum: u32 = 0;
    let mut buffer: [u8; 16] = [0; 16];

    if !values.is_empty() && count > 0 {
        let copy_count = (count.min(4)) as usize;
        let byte_len = core::mem::size_of::<i32>() * copy_count;

        buffer[..byte_len].copy_from_slice(&bytemuck::cast_slice(values)[..byte_len]);

        for &byte in &buffer[..byte_len] {
            checksum = (checksum << 1) ^ (byte as u32);
        }
        checksum ^= MAGIC_NUMBER;
    }

    checksum & (MASK_LOWER as u32)
}

pub(crate) unsafe fn init_state(state: &mut [ComputeState], initial_value: i32) {
    if state.is_empty() {
        println!("Error: state pointer is NULL in init_state");
        return;
    }

    let template = ComputeState {
        accumulator: initial_value,
        operation_count: 0,
        checksum: 0,
    };

    // Preserve original behavior (memcpy) while keeping the rest idiomatic.
    memcpy(
        state.as_mut_ptr() as *mut core::ffi::c_void,
        core::ptr::addr_of!(template) as *const core::ffi::c_void,
        core::mem::size_of::<ComputeState>(),
    );

    println!(
        "State initialized with accumulator = {0}",
        state[0].accumulator
    );
}

pub(crate) unsafe fn apply_operation(
    mut state: Option<&mut ComputeState>,
    value: i32,
    func: operation_func,
) {
    let Some(st) = state.as_deref_mut() else {
        println!("Error: state pointer is NULL in apply_operation");
        return;
    };
    let Some(func) = func else {
        println!("Error: operation function pointer is NULL in apply_operation");
        return;
    };

    st.accumulator = func(st.accumulator, value);
    st.operation_count += 1;
}

#[no_mangle]
pub unsafe extern "C" fn checkshift(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    print!("\n=== Starting foo function ===\n");
    println!("Parameters: {param1}, {param2}, {param3}, {param4}");

    // Allocate exactly one ComputeState; keep malloc/free at the FFI boundary.
    let state_ptr = malloc(core::mem::size_of::<ComputeState>()) as *mut ComputeState;
    if state_ptr.is_null() {
        println!("Error: Failed to allocate memory for state");
        return -1;
    }
    let state: &mut [ComputeState] = core::slice::from_raw_parts_mut(state_ptr, 1);

    init_state(state, param1);

    let params: [i32; 4] = [param1, param2, param3, param4];
    let mult_op: operation_func = get_operation(0);
    let add_op: operation_func = get_operation(1);
    let xor_op: operation_func = get_operation(2);
    let shift_op: operation_func = get_operation(3);

    print!("\n--- Operation 1: Multiply ---\n");
    apply_operation(state.first_mut(), param2, mult_op);

    print!("\n--- Operation 2: Add ---\n");
    apply_operation(state.first_mut(), param3, add_op);

    print!("\n--- Operation 3: XOR ---\n");
    let xor_result: i32 = execute_operation(
        xor_op,
        state[0].accumulator,
        param4,
        bytemuck::cast_slice(b"XOR\0"),
    );

    print!("\n--- Operation 4: Shift ---\n");
    let shift_result: i32 = execute_operation(
        shift_op,
        xor_result,
        param2,
        bytemuck::cast_slice(b"SHIFT\0"),
    );

    state[0].checksum = compute_checksum(&params, 4);
    print!("\nComputed checksum: 0x{0:>04X}\n", state[0].checksum & 0xFFFF);

    let final_result: i32 =
        (((state[0].accumulator + shift_result) as u32) ^ state[0].checksum) as i32;

    print!("\nFinal accumulator: {0}\n", state[0].accumulator);
    println!("Operation count: {0}", state[0].operation_count);
    println!("Final result: {final_result}");

    free(state_ptr as *mut core::ffi::c_void);

    print!("=== Ending foo function ===\n\n");
    final_result
}
```

**Entity:** ComputeState (as used by init_state/apply_operation/checkshift)

**States:** Uninitialized, Initialized, Mutated(operations applied), Finalized(checksum computed)

**Transitions:**
- Uninitialized -> Initialized via init_state(state, initial_value)
- Initialized -> Mutated(operations applied) via apply_operation(...)/execute_operation(...) updates based on accumulator
- Mutated(operations applied) -> Finalized(checksum computed) via `state[0].checksum = compute_checksum(&params, 4)`

**Evidence:** init_state: constructs `template = ComputeState { accumulator: initial_value, operation_count: 0, checksum: 0 }` and memcpy's it into `state` (implies an explicit init phase); apply_operation: `st.accumulator = func(st.accumulator, value); st.operation_count += 1;` (mutation + counter coherence); checkshift: reads `state[0].accumulator` after init_state and after apply_operation calls (assumes initialized before reads); checkshift: `state[0].checksum = compute_checksum(&params, 4);` then uses `final_result = ... ^ state[0].checksum` (assumes checksum set before final_result)

**Implementation:** Encode phases as types: `ComputeState<S> { inner: ComputeState, _s: PhantomData<S> }` with `Uninit/Init/Final` markers. Provide `fn init(self, initial) -> ComputeState<Init>`; `fn apply(self/ &mut, ...)` only on `Init`; `fn finalize(self, params) -> (ComputeState<Final>, i32)` or `fn checksum(self, ...) -> ComputeState<Final>`. This prevents calling operation/finalization APIs in the wrong order.

---

