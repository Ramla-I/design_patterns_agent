# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## State Machine Invariants

### 1. Arithmetic engine protocol via implicit thread-local state (Accumulator/Multiplier/Count)

**Location**: `/data/test_case/lib.rs:1-202`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The arithmetic operations are not pure functions: they mutate thread-local hidden state (accumulator, multiplier, operation_count). Correct interpretation of results depends on call history and ordering (e.g., operation_count must reflect number of executed ops; accumulator threshold gates whether subtraction runs; divide_multiplier silently no-ops when b==0). None of this statefulness or the required temporal ordering is visible in types—callers just see extern "C" functions taking i32s. The operations array is typed as Option<fn>, but the code assumes all entries are Some and panics otherwise.

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

extern "C" {
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn memchr(__s: *const core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

thread_local! {
    static accumulator: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
thread_local! {
    static multiplier: std::cell::Cell<i32> = const { std::cell::Cell::new(1) };
}
thread_local! {
    static operation_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) extern "C" fn add_to_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() + (a + b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn multiply_with_multiplier(a: i32, b: i32) -> i32 {
    multiplier.set(multiplier.get() * (a * b));
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) extern "C" fn subtract_from_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() - (a - b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn divide_multiplier(_: i32, b: i32) -> i32 {
    if b != 0 {
        multiplier.set(multiplier.get() / b);
    }
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) unsafe fn process_octal_string(dest: &mut [i8], octal_val: i32) {
    let mut buffer: [i8; 50] = [0; 50];
    sprintf(
        buffer.as_mut_ptr(),
        b"Octal: 0%o, Decimal: %d\0" as *const u8 as *const i8,
        octal_val,
        octal_val,
    );
    strcpy(dest.as_mut_ptr(), buffer.as_ptr());
}

#[inline]
unsafe fn c_strlen(mut s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s != 0 {
        n += 1;
        s = s.add(1);
    }
    n
}

pub(crate) unsafe fn find_and_replace_char(str: *mut i8, search_char: i32) {
    if str.is_null() {
        return;
    }
    // Match original: CStr::from_ptr(str).count_bytes()
    let len = c_strlen(str as *const i8);
    let found = memchr(str as *const core::ffi::c_void, search_char, len) as *mut i8;
    if let Some(ch) = found.as_mut() {
        *ch = b'X' as i8;
    }
}

pub(crate) fn validate_and_normalize(value: i32) -> i32 {
    let is_nonzero: i32 = (value != 0) as i32;
    let lower_threshold: i32 = 0o100;
    let upper_threshold: i32 = 0o777;
    if is_nonzero != 0 && value > 0 {
        if value < lower_threshold {
            return lower_threshold;
        } else if value > upper_threshold {
            return upper_threshold;
        }
    }
    value
}

static operations: [operation_func; 4] = unsafe {
    [
        Some(add_to_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(multiply_with_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
        Some(subtract_from_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(divide_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
    ]
};

#[no_mangle]
pub unsafe extern "C" fn findrep(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    let p1_valid: i32 = (param1 != 0) as i32;
    let p2_valid: i32 = (param2 != 0) as i32;
    let p3_valid: i32 = (param3 != 0) as i32;
    let p4_valid: i32 = (param4 != 0) as i32;
    let active_params: i32 = p1_valid + p2_valid + p3_valid + p4_valid;

    let mode_add: i32 = 0o1;
    let mode_multiply: i32 = 0o2;

    let normalized_p1: i32 = validate_and_normalize(param1);
    let normalized_p2: i32 = validate_and_normalize(param2);
    let normalized_p3: i32 = validate_and_normalize(param3);
    let normalized_p4: i32 = validate_and_normalize(param4);

    let mut message: [i8; 100] = [0; 100];
    let mut search_buffer: [i8; 100] = [0; 100];

    process_octal_string(&mut message, 0o123);
    strcpy(
        search_buffer.as_mut_ptr(),
        b"Function pointer example with static vars\0" as *const u8 as *const i8,
    );

    // IMPORTANT: match original C2Rust bug/behavior:
    // It created a slice from the found pointer with length 100000, then did
    // offset_from(search_buffer) on that slice's pointer (which is the found pointer),
    // and added that offset to result. That offset is always 0 when found.
    let search_len = c_strlen(search_buffer.as_ptr());
    let found_ptr = memchr(
        search_buffer.as_ptr() as *const core::ffi::c_void,
        b'p' as i32,
        search_len,
    ) as *const i8;

    if !found_ptr.is_null() {
        // offset_from(found_ptr, search_buffer) in original was effectively 0 due to slice base.
        result += 0;
    }

    let mut selected_op: operation_func;

    if active_params >= mode_add {
        selected_op = operations[0];
        result += selected_op.expect("non-null function pointer")(normalized_p1, normalized_p2);
    }

    if active_params >= mode_multiply {
        selected_op = operations[1];
        result += selected_op.expect("non-null function pointer")(normalized_p3, normalized_p4);
    }

    if accumulator.get() > 0o150 {
        selected_op = operations[2];
        let subtract_result =
            selected_op.expect("non-null function pointer")(normalized_p1, normalized_p3);
        result += subtract_result;
    }

    find_and_replace_char(message.as_mut_ptr(), b'O' as i32);

    let mut final_message: [i8; 100] = [0; 100];
    strcpy(final_message.as_mut_ptr(), message.as_ptr());

    let has_accumulator: i32 = (accumulator.get() != 0) as i32;
    let has_multiplier: i32 = (multiplier.get() != 0) as i32;
    let both_active: i32 = (has_accumulator != 0 && has_multiplier != 0) as i32;
    if both_active != 0 {
        result += accumulator.get() + multiplier.get();
    }

    if multiplier.get() > 0o100 {
        selected_op = operations[3];
        selected_op.expect("non-null function pointer")(multiplier.get(), 2);
    }

    result += operation_count.get() * 0o10;

    let result_exists: i32 = (result != 0) as i32;
    if result_exists == 0 {
        result = 0o777;
    }

    result
}
```

**Entity:** thread_local! accumulator/multiplier/operation_count + operation_func (operations table)

**States:** Fresh (accumulator=0, multiplier=1, operation_count=0), Mutated (after one or more operations), Div-by-zero ignored (divide_multiplier called with b=0)

**Transitions:**
- Fresh -> Mutated via add_to_accumulator()/multiply_with_multiplier()/subtract_from_accumulator()/divide_multiplier()
- Mutated -> Mutated via any subsequent operation
- Mutated -> Div-by-zero ignored via divide_multiplier(_, b) when b==0 (multiplier unchanged but operation_count increments)

**Evidence:** thread_local! static accumulator: Cell<i32> = ...Cell::new(0); thread_local! static multiplier: Cell<i32> = ...Cell::new(1); thread_local! static operation_count: Cell<i32> = ...Cell::new(0); fn add_to_accumulator: accumulator.set(...); operation_count.set(operation_count.get() + 1); fn multiply_with_multiplier: multiplier.set(...); operation_count.set(operation_count.get() + 1); fn divide_multiplier: if b != 0 { multiplier.set(multiplier.get() / b); } ... operation_count++ (silent 'b!=0' precondition); findrep: if accumulator.get() > 0o150 { selected_op = operations[2]; ... } (state-dependent branching); findrep: result += operation_count.get() * 0o10 (semantic dependence on correct counting); static operations: [operation_func; 4] = [Some(...), ...]; findrep: selected_op.expect("non-null function pointer")(...) (assumes Some)

**Implementation:** Introduce an explicit engine type, e.g. `struct Engine<State> { accumulator: i32, multiplier: i32, count: i32, _s: PhantomData<State> }`. Provide constructors for `Engine<Fresh>` and transition methods returning new states. Move the function pointer table to `fn(&mut Engine<Running>, a, b) -> i32` (no Option), so calling code must hold a mutable engine to perform operations and counting/threshold gating becomes explicit and testable.

---

## Precondition Invariants

### 2. C-string buffer capacity + NUL-termination precondition for process_octal_string

**Location**: `/data/test_case/lib.rs:1-202`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: process_octal_string uses `sprintf` into a fixed local buffer and then `strcpy` into `dest`. This implicitly requires that `dest` has enough space for the formatted string including the trailing NUL byte. The function signature `&mut [i8]` does not encode the minimum length, and `strcpy` will overrun `dest` if it's too small.

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

extern "C" {
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn memchr(__s: *const core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

thread_local! {
    static accumulator: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
thread_local! {
    static multiplier: std::cell::Cell<i32> = const { std::cell::Cell::new(1) };
}
thread_local! {
    static operation_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) extern "C" fn add_to_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() + (a + b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn multiply_with_multiplier(a: i32, b: i32) -> i32 {
    multiplier.set(multiplier.get() * (a * b));
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) extern "C" fn subtract_from_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() - (a - b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn divide_multiplier(_: i32, b: i32) -> i32 {
    if b != 0 {
        multiplier.set(multiplier.get() / b);
    }
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) unsafe fn process_octal_string(dest: &mut [i8], octal_val: i32) {
    let mut buffer: [i8; 50] = [0; 50];
    sprintf(
        buffer.as_mut_ptr(),
        b"Octal: 0%o, Decimal: %d\0" as *const u8 as *const i8,
        octal_val,
        octal_val,
    );
    strcpy(dest.as_mut_ptr(), buffer.as_ptr());
}

#[inline]
unsafe fn c_strlen(mut s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s != 0 {
        n += 1;
        s = s.add(1);
    }
    n
}

pub(crate) unsafe fn find_and_replace_char(str: *mut i8, search_char: i32) {
    if str.is_null() {
        return;
    }
    // Match original: CStr::from_ptr(str).count_bytes()
    let len = c_strlen(str as *const i8);
    let found = memchr(str as *const core::ffi::c_void, search_char, len) as *mut i8;
    if let Some(ch) = found.as_mut() {
        *ch = b'X' as i8;
    }
}

pub(crate) fn validate_and_normalize(value: i32) -> i32 {
    let is_nonzero: i32 = (value != 0) as i32;
    let lower_threshold: i32 = 0o100;
    let upper_threshold: i32 = 0o777;
    if is_nonzero != 0 && value > 0 {
        if value < lower_threshold {
            return lower_threshold;
        } else if value > upper_threshold {
            return upper_threshold;
        }
    }
    value
}

static operations: [operation_func; 4] = unsafe {
    [
        Some(add_to_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(multiply_with_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
        Some(subtract_from_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(divide_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
    ]
};

#[no_mangle]
pub unsafe extern "C" fn findrep(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    let p1_valid: i32 = (param1 != 0) as i32;
    let p2_valid: i32 = (param2 != 0) as i32;
    let p3_valid: i32 = (param3 != 0) as i32;
    let p4_valid: i32 = (param4 != 0) as i32;
    let active_params: i32 = p1_valid + p2_valid + p3_valid + p4_valid;

    let mode_add: i32 = 0o1;
    let mode_multiply: i32 = 0o2;

    let normalized_p1: i32 = validate_and_normalize(param1);
    let normalized_p2: i32 = validate_and_normalize(param2);
    let normalized_p3: i32 = validate_and_normalize(param3);
    let normalized_p4: i32 = validate_and_normalize(param4);

    let mut message: [i8; 100] = [0; 100];
    let mut search_buffer: [i8; 100] = [0; 100];

    process_octal_string(&mut message, 0o123);
    strcpy(
        search_buffer.as_mut_ptr(),
        b"Function pointer example with static vars\0" as *const u8 as *const i8,
    );

    // IMPORTANT: match original C2Rust bug/behavior:
    // It created a slice from the found pointer with length 100000, then did
    // offset_from(search_buffer) on that slice's pointer (which is the found pointer),
    // and added that offset to result. That offset is always 0 when found.
    let search_len = c_strlen(search_buffer.as_ptr());
    let found_ptr = memchr(
        search_buffer.as_ptr() as *const core::ffi::c_void,
        b'p' as i32,
        search_len,
    ) as *const i8;

    if !found_ptr.is_null() {
        // offset_from(found_ptr, search_buffer) in original was effectively 0 due to slice base.
        result += 0;
    }

    let mut selected_op: operation_func;

    if active_params >= mode_add {
        selected_op = operations[0];
        result += selected_op.expect("non-null function pointer")(normalized_p1, normalized_p2);
    }

    if active_params >= mode_multiply {
        selected_op = operations[1];
        result += selected_op.expect("non-null function pointer")(normalized_p3, normalized_p4);
    }

    if accumulator.get() > 0o150 {
        selected_op = operations[2];
        let subtract_result =
            selected_op.expect("non-null function pointer")(normalized_p1, normalized_p3);
        result += subtract_result;
    }

    find_and_replace_char(message.as_mut_ptr(), b'O' as i32);

    let mut final_message: [i8; 100] = [0; 100];
    strcpy(final_message.as_mut_ptr(), message.as_ptr());

    let has_accumulator: i32 = (accumulator.get() != 0) as i32;
    let has_multiplier: i32 = (multiplier.get() != 0) as i32;
    let both_active: i32 = (has_accumulator != 0 && has_multiplier != 0) as i32;
    if both_active != 0 {
        result += accumulator.get() + multiplier.get();
    }

    if multiplier.get() > 0o100 {
        selected_op = operations[3];
        selected_op.expect("non-null function pointer")(multiplier.get(), 2);
    }

    result += operation_count.get() * 0o10;

    let result_exists: i32 = (result != 0) as i32;
    if result_exists == 0 {
        result = 0o777;
    }

    result
}
```

**Entity:** process_octal_string(dest: &mut [i8], octal_val: i32)

**States:** Sufficient capacity (dest large enough), Insufficient capacity (overflow/UB risk)

**Transitions:**
- Insufficient capacity -> memory corruption via strcpy()
- Sufficient capacity -> valid C string written into dest

**Evidence:** process_octal_string: let mut buffer: [i8; 50] = [0; 50]; process_octal_string: sprintf(buffer.as_mut_ptr(), b"Octal: 0%o, Decimal: %d\0"..., octal_val, octal_val); process_octal_string: strcpy(dest.as_mut_ptr(), buffer.as_ptr()) (unbounded copy into dest)

**Implementation:** Accept a sized output buffer type that encodes capacity, e.g. `fn process_octal_string(dest: &mut [i8; 50], ...)` or `struct CBuf<const N: usize>([i8; N]);` and implement writing with bounds checking (`snprintf`-style) so the capacity constraint is enforced by the type/const generic.

---

## Protocol Invariants

### 3. Valid C-string pointer protocol (non-null, NUL-terminated, readable/writable memory)

**Location**: `/data/test_case/lib.rs:1-202`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The function is written in a C style: it accepts a raw pointer, checks only for null, computes length via `c_strlen` by scanning until NUL, then uses `memchr` over that length and mutates the found byte. This implicitly requires that `str` points to a valid, readable NUL-terminated buffer and that the found location is writable. These requirements are not captured by the signature and are only partially guarded at runtime (null check only).

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

extern "C" {
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn memchr(__s: *const core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

pub type operation_func = Option<unsafe extern "C" fn(i32, i32) -> i32>;

thread_local! {
    static accumulator: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}
thread_local! {
    static multiplier: std::cell::Cell<i32> = const { std::cell::Cell::new(1) };
}
thread_local! {
    static operation_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) extern "C" fn add_to_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() + (a + b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn multiply_with_multiplier(a: i32, b: i32) -> i32 {
    multiplier.set(multiplier.get() * (a * b));
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) extern "C" fn subtract_from_accumulator(a: i32, b: i32) -> i32 {
    accumulator.set(accumulator.get() - (a - b));
    operation_count.set(operation_count.get() + 1);
    accumulator.get()
}

pub(crate) extern "C" fn divide_multiplier(_: i32, b: i32) -> i32 {
    if b != 0 {
        multiplier.set(multiplier.get() / b);
    }
    operation_count.set(operation_count.get() + 1);
    multiplier.get()
}

pub(crate) unsafe fn process_octal_string(dest: &mut [i8], octal_val: i32) {
    let mut buffer: [i8; 50] = [0; 50];
    sprintf(
        buffer.as_mut_ptr(),
        b"Octal: 0%o, Decimal: %d\0" as *const u8 as *const i8,
        octal_val,
        octal_val,
    );
    strcpy(dest.as_mut_ptr(), buffer.as_ptr());
}

#[inline]
unsafe fn c_strlen(mut s: *const i8) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *s != 0 {
        n += 1;
        s = s.add(1);
    }
    n
}

pub(crate) unsafe fn find_and_replace_char(str: *mut i8, search_char: i32) {
    if str.is_null() {
        return;
    }
    // Match original: CStr::from_ptr(str).count_bytes()
    let len = c_strlen(str as *const i8);
    let found = memchr(str as *const core::ffi::c_void, search_char, len) as *mut i8;
    if let Some(ch) = found.as_mut() {
        *ch = b'X' as i8;
    }
}

pub(crate) fn validate_and_normalize(value: i32) -> i32 {
    let is_nonzero: i32 = (value != 0) as i32;
    let lower_threshold: i32 = 0o100;
    let upper_threshold: i32 = 0o777;
    if is_nonzero != 0 && value > 0 {
        if value < lower_threshold {
            return lower_threshold;
        } else if value > upper_threshold {
            return upper_threshold;
        }
    }
    value
}

static operations: [operation_func; 4] = unsafe {
    [
        Some(add_to_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(multiply_with_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
        Some(subtract_from_accumulator as unsafe extern "C" fn(i32, i32) -> i32),
        Some(divide_multiplier as unsafe extern "C" fn(i32, i32) -> i32),
    ]
};

#[no_mangle]
pub unsafe extern "C" fn findrep(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    let p1_valid: i32 = (param1 != 0) as i32;
    let p2_valid: i32 = (param2 != 0) as i32;
    let p3_valid: i32 = (param3 != 0) as i32;
    let p4_valid: i32 = (param4 != 0) as i32;
    let active_params: i32 = p1_valid + p2_valid + p3_valid + p4_valid;

    let mode_add: i32 = 0o1;
    let mode_multiply: i32 = 0o2;

    let normalized_p1: i32 = validate_and_normalize(param1);
    let normalized_p2: i32 = validate_and_normalize(param2);
    let normalized_p3: i32 = validate_and_normalize(param3);
    let normalized_p4: i32 = validate_and_normalize(param4);

    let mut message: [i8; 100] = [0; 100];
    let mut search_buffer: [i8; 100] = [0; 100];

    process_octal_string(&mut message, 0o123);
    strcpy(
        search_buffer.as_mut_ptr(),
        b"Function pointer example with static vars\0" as *const u8 as *const i8,
    );

    // IMPORTANT: match original C2Rust bug/behavior:
    // It created a slice from the found pointer with length 100000, then did
    // offset_from(search_buffer) on that slice's pointer (which is the found pointer),
    // and added that offset to result. That offset is always 0 when found.
    let search_len = c_strlen(search_buffer.as_ptr());
    let found_ptr = memchr(
        search_buffer.as_ptr() as *const core::ffi::c_void,
        b'p' as i32,
        search_len,
    ) as *const i8;

    if !found_ptr.is_null() {
        // offset_from(found_ptr, search_buffer) in original was effectively 0 due to slice base.
        result += 0;
    }

    let mut selected_op: operation_func;

    if active_params >= mode_add {
        selected_op = operations[0];
        result += selected_op.expect("non-null function pointer")(normalized_p1, normalized_p2);
    }

    if active_params >= mode_multiply {
        selected_op = operations[1];
        result += selected_op.expect("non-null function pointer")(normalized_p3, normalized_p4);
    }

    if accumulator.get() > 0o150 {
        selected_op = operations[2];
        let subtract_result =
            selected_op.expect("non-null function pointer")(normalized_p1, normalized_p3);
        result += subtract_result;
    }

    find_and_replace_char(message.as_mut_ptr(), b'O' as i32);

    let mut final_message: [i8; 100] = [0; 100];
    strcpy(final_message.as_mut_ptr(), message.as_ptr());

    let has_accumulator: i32 = (accumulator.get() != 0) as i32;
    let has_multiplier: i32 = (multiplier.get() != 0) as i32;
    let both_active: i32 = (has_accumulator != 0 && has_multiplier != 0) as i32;
    if both_active != 0 {
        result += accumulator.get() + multiplier.get();
    }

    if multiplier.get() > 0o100 {
        selected_op = operations[3];
        selected_op.expect("non-null function pointer")(multiplier.get(), 2);
    }

    result += operation_count.get() * 0o10;

    let result_exists: i32 = (result != 0) as i32;
    if result_exists == 0 {
        result = 0o777;
    }

    result
}
```

**Entity:** find_and_replace_char(str: *mut i8, search_char: i32) + c_strlen/memchr usage

**States:** Null pointer (treated as no-op), Non-null but not NUL-terminated / invalid memory (UB risk), Valid NUL-terminated mutable C string

**Transitions:**
- Null pointer -> returns early (no-op)
- Valid NUL-terminated mutable C string -> possibly replaces first occurrence with 'X'
- Non-null but invalid/not terminated -> potential out-of-bounds read in c_strlen and/or memchr, and potential invalid write

**Evidence:** find_and_replace_char(str: *mut i8, ...): if str.is_null() { return; } (only null is checked); find_and_replace_char: let len = c_strlen(str as *const i8); (requires NUL termination and readable memory); c_strlen: while *s != 0 { ... s = s.add(1); } (unbounded scan if no NUL); find_and_replace_char: let found = memchr(str as *const c_void, search_char, len) as *mut i8; (assumes `len` is correct/within allocation); find_and_replace_char: if let Some(ch) = found.as_mut() { *ch = b'X' as i8; } (writes through raw pointer)

**Implementation:** Replace `*mut i8` with `&mut CStr`-like wrapper or a custom `struct MutCStr<'a>(&'a mut [u8]);` that guarantees NUL-termination and mutability. Expose a safe constructor from `&'a mut [u8]` that validates exactly one trailing NUL (or at least presence of NUL) before allowing `find_and_replace_char` to run safely.

---

