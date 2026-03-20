# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 3. FFI temporary buffer lifecycle (malloc/memcpy/strtod/free) must be balanced and non-null

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: parse_number manually manages a temporary C string buffer with malloc/free and relies on explicit early-return paths to free it. Correctness requires that the allocation succeeds (non-null) before memcpy/strtod, and that free is called exactly once on all paths after allocation. This is enforced by control-flow discipline, not by RAII or types, and is easy to break with future edits.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct parse_buffer, 1 free function(s); struct cJSON

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

pub type cJSON_bool = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct parse_buffer {
    pub content: *const u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cJSON {
    pub type_0: i32,
    pub valueint: i32,
    pub valuedouble: f64,
}

pub const true_0: cJSON_bool = 1;
pub const false_0: cJSON_bool = 0;

pub const __INT_MAX__: i32 = 2147483647;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const INT_MAX: i32 = __INT_MAX__;
pub const cJSON_Number: i32 = 1i32 << 3;

extern "C" {
    fn strtod(__nptr: *const i8, __endptr: *mut *mut i8) -> f64;
}

#[inline]
fn is_number_char(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' | b'.')
}

#[no_mangle]
pub unsafe extern "C" fn parse_number(
    mut item: Option<&mut cJSON>,
    input_buffer: Option<&mut parse_buffer>,
) -> cJSON_bool {
    let Some(input) = input_buffer.as_deref() else {
        return false_0;
    };
    if input.content.is_null() {
        return false_0;
    }

    // Determine maximal prefix length that can belong to a number.
    let mut number_string_length: usize = 0;
    let mut has_decimal_point: cJSON_bool = false_0;

    while input.offset + number_string_length < input.length {
        let b = *input.content.add(input.offset + number_string_length);
        if !is_number_char(b) {
            break;
        }
        if b == b'.' {
            has_decimal_point = true_0;
        }
        number_string_length += 1;
    }

    // Allocate temporary C string buffer.
    let raw = malloc(number_string_length + 1) as *mut u8;
    if raw.is_null() {
        return false_0;
    }
    let number_c_string = core::slice::from_raw_parts_mut(raw, number_string_length + 1);

    memcpy(
        number_c_string.as_mut_ptr() as *mut core::ffi::c_void,
        input.content.add(input.offset) as *const core::ffi::c_void,
        number_string_length,
    );
    number_c_string[number_string_length] = b'\0';

    // Preserve original behavior: normalize '.' to decimal_point (which is also '.').
    if has_decimal_point != 0 {
        let decimal_point: u8 = b'.';
        for b in &mut number_c_string[..number_string_length] {
            if *b == b'.' {
                *b = decimal_point;
            }
        }
    }

    // Parse using libc-like strtod.
    let mut endptr: *mut i8 = core::ptr::null_mut();
    let number = strtod(number_c_string.as_ptr() as *const i8, &mut endptr);

    if endptr == number_c_string.as_mut_ptr() as *mut i8 {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    }

    let Some(out_item) = item.as_deref_mut() else {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    };

    out_item.valuedouble = number;
    out_item.valueint = if number >= INT_MAX as f64 {
        INT_MAX
    } else if number <= INT_MIN as f64 {
        INT_MIN
    } else {
        number as i32
    };
    out_item.type_0 = cJSON_Number;

    // Advance input offset by the number of bytes consumed.
    let consumed = (endptr as *mut u8).offset_from(number_c_string.as_mut_ptr()) as usize;
    if let Some(input_mut) = input_buffer {
        input_mut.offset = input_mut.offset.wrapping_add(consumed);
    }

    free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
    true_0
}
```

**Entity:** parse_number

**States:** Unallocated, Allocated (must be freed exactly once), Freed

**Transitions:**
- Unallocated -> Allocated via malloc(number_string_length + 1)
- Allocated -> Freed via free(number_c_string.as_mut_ptr() as *mut c_void) on all exit paths after allocation

**Evidence:** parse_number: `let raw = malloc(number_string_length + 1) as *mut u8; if raw.is_null() { return false_0; }` allocation + null check; parse_number: `memcpy(... input.content.add(input.offset) ..., number_string_length)` uses allocated buffer and assumes it is valid; parse_number: multiple early returns that manually `free(...)` (e.g. when `endptr == ...` and when `item` is None); parse_number: final `free(...)` before returning true_0

**Implementation:** Introduce a small RAII guard for the allocation: `struct MallocBuf(NonNull<u8>, usize); impl Drop for MallocBuf { fn drop(&mut self){ unsafe{ free(self.0.as_ptr() as *mut c_void) }}}` so all returns automatically free. Or avoid malloc entirely by parsing from a Rust slice (e.g., via `from_utf8` + `f64::from_str`) when acceptable.

---

## State Machine Invariants

### 4. cJSON Number variant initialization protocol (type_0 controls which value fields are valid)

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: parse_number mutates a cJSON to represent a number by setting valuedouble/valueint and then setting type_0 = cJSON_Number. Implicitly, consumers are expected to interpret valueint/valuedouble only when type_0 indicates the Number variant. This is a runtime tag protocol encoded as an i32 and is not enforced by Rust types (a safe API would prevent reading number fields when not a number, and would prevent partially-initialized states).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct parse_buffer, 1 free function(s); struct cJSON

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

pub type cJSON_bool = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct parse_buffer {
    pub content: *const u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cJSON {
    pub type_0: i32,
    pub valueint: i32,
    pub valuedouble: f64,
}

pub const true_0: cJSON_bool = 1;
pub const false_0: cJSON_bool = 0;

pub const __INT_MAX__: i32 = 2147483647;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const INT_MAX: i32 = __INT_MAX__;
pub const cJSON_Number: i32 = 1i32 << 3;

extern "C" {
    fn strtod(__nptr: *const i8, __endptr: *mut *mut i8) -> f64;
}

#[inline]
fn is_number_char(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' | b'.')
}

#[no_mangle]
pub unsafe extern "C" fn parse_number(
    mut item: Option<&mut cJSON>,
    input_buffer: Option<&mut parse_buffer>,
) -> cJSON_bool {
    let Some(input) = input_buffer.as_deref() else {
        return false_0;
    };
    if input.content.is_null() {
        return false_0;
    }

    // Determine maximal prefix length that can belong to a number.
    let mut number_string_length: usize = 0;
    let mut has_decimal_point: cJSON_bool = false_0;

    while input.offset + number_string_length < input.length {
        let b = *input.content.add(input.offset + number_string_length);
        if !is_number_char(b) {
            break;
        }
        if b == b'.' {
            has_decimal_point = true_0;
        }
        number_string_length += 1;
    }

    // Allocate temporary C string buffer.
    let raw = malloc(number_string_length + 1) as *mut u8;
    if raw.is_null() {
        return false_0;
    }
    let number_c_string = core::slice::from_raw_parts_mut(raw, number_string_length + 1);

    memcpy(
        number_c_string.as_mut_ptr() as *mut core::ffi::c_void,
        input.content.add(input.offset) as *const core::ffi::c_void,
        number_string_length,
    );
    number_c_string[number_string_length] = b'\0';

    // Preserve original behavior: normalize '.' to decimal_point (which is also '.').
    if has_decimal_point != 0 {
        let decimal_point: u8 = b'.';
        for b in &mut number_c_string[..number_string_length] {
            if *b == b'.' {
                *b = decimal_point;
            }
        }
    }

    // Parse using libc-like strtod.
    let mut endptr: *mut i8 = core::ptr::null_mut();
    let number = strtod(number_c_string.as_ptr() as *const i8, &mut endptr);

    if endptr == number_c_string.as_mut_ptr() as *mut i8 {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    }

    let Some(out_item) = item.as_deref_mut() else {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    };

    out_item.valuedouble = number;
    out_item.valueint = if number >= INT_MAX as f64 {
        INT_MAX
    } else if number <= INT_MIN as f64 {
        INT_MIN
    } else {
        number as i32
    };
    out_item.type_0 = cJSON_Number;

    // Advance input offset by the number of bytes consumed.
    let consumed = (endptr as *mut u8).offset_from(number_c_string.as_mut_ptr()) as usize;
    if let Some(input_mut) = input_buffer {
        input_mut.offset = input_mut.offset.wrapping_add(consumed);
    }

    free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
    true_0
}
```

**Entity:** cJSON

**States:** Uninitialized/OtherType, Number (type_0 == cJSON_Number; valueint/valuedouble meaningful)

**Transitions:**
- Uninitialized/OtherType -> Number via parse_number() writing valuedouble/valueint and setting type_0 = cJSON_Number

**Evidence:** cJSON fields: `type_0: i32` plus `valueint: i32`, `valuedouble: f64` indicates a tagged-union-like layout; constant `pub const cJSON_Number: i32 = 1i32 << 3;` is used as the tag value; parse_number: `out_item.valuedouble = number; ... out_item.valueint = ...; out_item.type_0 = cJSON_Number;` establishes the tag/data relationship

**Implementation:** Expose a safe wrapper enum around cJSON, e.g. `enum JsonValue { Number { int: i32, double: f64 }, ... }`, or typestate wrapper `struct CJson<S> { raw: cJSON, _s: PhantomData<S> }` where `CJson<Number>` guarantees `type_0==cJSON_Number` and provides number accessors; parsing returns `CJson<Number>` (or `Result<CJson<Number>, _>`).

---

## Precondition Invariants

### 1. parse_buffer validity + cursor protocol (non-null buffer, in-bounds offset, depth tracking)

**Location**: `/data/test_case/lib.rs:1-11`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: parse_buffer is a C-ABI parsing cursor over a byte slice. Correct use implicitly requires that `content` is non-null and points to at least `length` readable bytes for the lifetime of the buffer, and that `offset` stays within `[0, length]` as the cursor advances. `depth` likely tracks nested parsing depth and is expected to be updated in sync with parse transitions (enter/exit nested structures) and to remain within some reasonable bound. None of these invariants are enforced by the type system because raw pointers and plain `usize` fields permit construction of invalid states and out-of-bounds cursor movement.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cJSON; 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct parse_buffer {
    pub content: *const u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
}

```

**Entity:** parse_buffer

**States:** Invalid (null/dangling content or inconsistent fields), Valid (content points to readable bytes; offset/length/depth consistent)

**Transitions:**
- Invalid -> Valid via constructing from a real backing buffer (not representable here; currently any values can be assigned)
- Valid -> Invalid via advancing `offset` beyond `length` or using `content` after its backing storage is freed/moved
- Valid -> Valid via cursor movement (updating offset) and nesting changes (updating depth) while preserving invariants

**Evidence:** line 8: `pub content: *const u8` raw pointer allows null/dangling; no lifetime ties to backing storage; line 9: `pub length: usize` intended buffer size is not coupled to `content`; line 10: `pub offset: usize` cursor position not constrained to be <= length; line 11: `pub depth: usize` nesting depth tracked as an unconstrained integer

**Implementation:** Replace `*const u8 + length` with `&'a [u8]` (or `NonNull<u8>` + `PhantomData<&'a [u8]>` for FFI), and wrap `offset` in a newtype that can only be constructed/advanced with bounds checks (e.g., `struct Offset(usize);` with `fn advance(&mut self, n, len) -> Result<()>`). Consider splitting into `ParseBuffer<'a>` (safe) and an FFI `repr(C)` wrapper that can be validated via `TryFrom<parse_buffer>` before use.

---

## Protocol Invariants

### 2. parse_buffer validity + cursor protocol (NonNull content, offset within length, monotonic consumption)

**Location**: `/data/test_case/lib.rs:1-142`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: parse_number assumes an input parse_buffer is in a valid readable state: content must be non-null, and offset/length must describe a readable slice. It then treats offset as a cursor into content and advances it by the number of bytes consumed from a temporary C string parse. These requirements are enforced with runtime null checks and implicit pointer arithmetic, and offset advancement uses wrapping_add (allowing silent overflow). None of these invariants (non-null pointer, in-bounds cursor, non-overflowing advancement, monotonic progression) are expressed in the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct parse_buffer, 1 free function(s); struct cJSON

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
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

pub type cJSON_bool = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct parse_buffer {
    pub content: *const u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cJSON {
    pub type_0: i32,
    pub valueint: i32,
    pub valuedouble: f64,
}

pub const true_0: cJSON_bool = 1;
pub const false_0: cJSON_bool = 0;

pub const __INT_MAX__: i32 = 2147483647;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const INT_MAX: i32 = __INT_MAX__;
pub const cJSON_Number: i32 = 1i32 << 3;

extern "C" {
    fn strtod(__nptr: *const i8, __endptr: *mut *mut i8) -> f64;
}

#[inline]
fn is_number_char(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' | b'.')
}

#[no_mangle]
pub unsafe extern "C" fn parse_number(
    mut item: Option<&mut cJSON>,
    input_buffer: Option<&mut parse_buffer>,
) -> cJSON_bool {
    let Some(input) = input_buffer.as_deref() else {
        return false_0;
    };
    if input.content.is_null() {
        return false_0;
    }

    // Determine maximal prefix length that can belong to a number.
    let mut number_string_length: usize = 0;
    let mut has_decimal_point: cJSON_bool = false_0;

    while input.offset + number_string_length < input.length {
        let b = *input.content.add(input.offset + number_string_length);
        if !is_number_char(b) {
            break;
        }
        if b == b'.' {
            has_decimal_point = true_0;
        }
        number_string_length += 1;
    }

    // Allocate temporary C string buffer.
    let raw = malloc(number_string_length + 1) as *mut u8;
    if raw.is_null() {
        return false_0;
    }
    let number_c_string = core::slice::from_raw_parts_mut(raw, number_string_length + 1);

    memcpy(
        number_c_string.as_mut_ptr() as *mut core::ffi::c_void,
        input.content.add(input.offset) as *const core::ffi::c_void,
        number_string_length,
    );
    number_c_string[number_string_length] = b'\0';

    // Preserve original behavior: normalize '.' to decimal_point (which is also '.').
    if has_decimal_point != 0 {
        let decimal_point: u8 = b'.';
        for b in &mut number_c_string[..number_string_length] {
            if *b == b'.' {
                *b = decimal_point;
            }
        }
    }

    // Parse using libc-like strtod.
    let mut endptr: *mut i8 = core::ptr::null_mut();
    let number = strtod(number_c_string.as_ptr() as *const i8, &mut endptr);

    if endptr == number_c_string.as_mut_ptr() as *mut i8 {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    }

    let Some(out_item) = item.as_deref_mut() else {
        free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
        return false_0;
    };

    out_item.valuedouble = number;
    out_item.valueint = if number >= INT_MAX as f64 {
        INT_MAX
    } else if number <= INT_MIN as f64 {
        INT_MIN
    } else {
        number as i32
    };
    out_item.type_0 = cJSON_Number;

    // Advance input offset by the number of bytes consumed.
    let consumed = (endptr as *mut u8).offset_from(number_c_string.as_mut_ptr()) as usize;
    if let Some(input_mut) = input_buffer {
        input_mut.offset = input_mut.offset.wrapping_add(consumed);
    }

    free(number_c_string.as_mut_ptr() as *mut core::ffi::c_void);
    true_0
}
```

**Entity:** parse_buffer

**States:** Invalid (null content and/or out-of-bounds offset), Valid (non-null content, offset<=length; readable region), Advanced (valid after consuming a prefix)

**Transitions:**
- Invalid -> Valid by constructing parse_buffer with non-null content and consistent length/offset
- Valid -> Advanced via parse_number() updating input_mut.offset += consumed

**Evidence:** parse_buffer fields: content: *const u8, length: usize, offset: usize encode slice+cursor state at runtime; parse_number: `if input.content.is_null() { return false_0; }` guards null content; parse_number: reads `*input.content.add(input.offset + number_string_length)` relying on `offset + number_string_length < length` loop condition; parse_number: `input_mut.offset = input_mut.offset.wrapping_add(consumed);` updates cursor with wrapping semantics (implicit 'no overflow' invariant)

**Implementation:** Replace raw pointer/len/offset with safe representations: e.g. `struct Input<'a> { bytes: &'a [u8], cursor: usize }` (or `NonNull<u8>` + `usize` with checked methods). Provide a method `fn take_number_prefix(&mut self) -> (&[u8], ConsumedLen)` that returns the consumed prefix and advances cursor using checked_add, making out-of-bounds/overflow impossible without unsafe.

---

