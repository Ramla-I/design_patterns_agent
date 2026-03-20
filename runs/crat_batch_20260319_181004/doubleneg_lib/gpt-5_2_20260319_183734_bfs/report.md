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

### 1. Float-to-int conversion validity (Finite & InRange f64 -> i32)

**Location**: `/data/test_case/lib.rs:1-140`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on an implicit precondition that the f64 passed to convert_double_to_int must be finite (not NaN/Inf) and within the representable range of i32. This is not enforced by the type system: doubleneg intentionally constructs values that violate it (very large magnitude, INFINITY, NAN) and still converts them. A safer API would require callers to prove/obtain a validated value before conversion, turning the current 'UB likely/undefined behavior' cases into impossible states at compile time (or explicit error handling).

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

extern "C" {
    fn memchr(__s: *const core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
}

pub(crate) fn convert_double_to_int(value: f64) -> i32 {
    value as i32
}

pub(crate) unsafe fn find_value_in_buffer(buffer: &mut [i8], size: usize, search_val: i32) -> i32 {
    let size = size.min(buffer.len());
    let target = search_val as i8;

    // Prefer safe Rust search; keep behavior equivalent for i8 bytes.
    if let Some(pos) = buffer[..size].iter().position(|&b| b == target) {
        return pos as i32;
    }
    -1
}

pub(crate) fn create_numeric_buffer(buffer: &mut [i8], size: i32, seed: i32) {
    let size = (size.max(0) as usize).min(buffer.len());
    for (i, slot) in buffer.iter_mut().take(size).enumerate() {
        let i = i as i32;
        *slot = ((seed + i * 7) % 256) as i8;
    }
}

pub(crate) fn calculate_with_doubles(a: i32, b: i32, c: i32) -> f64 {
    let mut result = if b != 0 { a as f64 / b as f64 } else { 0.0 };
    result *= 10.0f64.powf((c % 10) as f64);
    result
}

#[no_mangle]
pub unsafe extern "C" fn doubleneg(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;
    let mut buffer: [i8; 256] = [0; 256];

    println!("=== Starting foo() execution ===");
    println!("Parameters: {param1}, {param2}, {param3}, {param4}");

    print!("\n--- Integer Negation Test ---\n");
    let negation_test: i32 = param1;
    let negation_result: i32 = (negation_test != 0) as i32;
    println!("Original value: {negation_test}");
    println!("After !!negation: {negation_result}");
    result += negation_result * 10;

    let neg_p2: i32 = (param2 != 0) as i32;
    let neg_p3: i32 = (param3 != 0) as i32;
    let neg_p4: i32 = (param4 != 0) as i32;
    println!("Double negation results: {neg_p2}, {neg_p3}, {neg_p4}");
    result += neg_p2 + neg_p3 + neg_p4;

    print!("\n--- Double to Int Conversion Test ---\n");
    let large_double: f64 = calculate_with_doubles(param1, param2, param3);
    println!("Calculated double value: {0:e}", { large_double });
    let converted_int: i32 = convert_double_to_int(large_double);
    println!("Converted to int (may be UB): {converted_int}");

    let negative_large: f64 = -2.0f64.powf(40_f64);
    println!("Very large negative double: {0:e}", { negative_large });
    let converted_neg: i32 = convert_double_to_int(negative_large);
    println!("Converted to int (UB likely): {converted_neg}");
    result += converted_int % 1000 + converted_neg % 1000;

    print!("\n--- Memchr Search Test ---\n");
    create_numeric_buffer(&mut buffer, 256, param1);

    let search_values: [i32; 4] = [param2 % 256, param3 % 256, param4 % 256, 42];
    let num_searches: i32 = search_values.len() as i32;

    println!("Searching buffer for values...");
    for i in 0..num_searches {
        let pos: i32 = find_value_in_buffer(&mut buffer, 256, search_values[i as usize]);
        if pos >= 0 {
            println!(
                "Found value {0} at position {1}",
                search_values[i as usize], pos
            );
            result += pos;
        } else {
            println!("Value {0} not found", search_values[i as usize]);
        }
    }

    // Keep the original FFI memchr usage and pointer-diff behavior.
    let direct_search: &[i8] = {
        let p = memchr(buffer.as_ptr() as *const _, 100, 256);
        if p.is_null() {
            &[]
        } else {
            // Preserve original (odd) behavior: create a huge slice from the found pointer.
            std::slice::from_raw_parts(p as *const i8, 100000)
        }
    };

    if !direct_search.is_empty() {
        let offset = direct_search
            .as_ptr()
            .cast_mut()
            .offset_from(buffer.as_mut_ptr()) as i64;
        println!("Direct memchr found byte 100 at offset: {0}", offset);
        result += offset as i32;
    }

    print!("\n--- Combined Feature Test ---\n");
    for i in 0..10 {
        let search_byte: i32 = (param1 + i * param2) % 256;
        let found: *mut std::ffi::c_void = memchr(buffer.as_ptr() as *const _, search_byte, 256);
        let found_flag: i32 = (!found.is_null()) as i32;
        println!("Search {i}: byte={search_byte}, found={found_flag}");
        result += found_flag;
    }

    let infinity_val: f64 = core::f32::INFINITY as f64;
    let nan_val: f64 = core::f32::NAN as f64;

    print!("\n--- Special Double Values ---\n");
    print!("Converting INFINITY to int: ");
    let inf_as_int: i32 = convert_double_to_int(infinity_val);
    println!("{inf_as_int} (undefined behavior)");
    print!("Converting NAN to int: ");
    let nan_as_int: i32 = convert_double_to_int(nan_val);
    println!("{nan_as_int} (undefined behavior)");

    print!("\n=== Final Result ===\n");
    println!("Accumulated result: {result}");
    result
}
```

**Entity:** convert_double_to_int (and its callers in doubleneg)

**States:** ValidConvertibleF64, InvalidF64 (NaN/Inf/OutOfRange)

**Transitions:**
- InvalidF64 -> ValidConvertibleF64 via explicit validation/clamping step before conversion

**Evidence:** fn convert_double_to_int(value: f64) -> i32 { value as i32 } (unchecked cast); doubleneg: println!("Converted to int (may be UB): {converted_int}") after convert_double_to_int(large_double); doubleneg: negative_large = -2.0f64.powf(40_f64); then println!("Converted to int (UB likely): {converted_neg}"); doubleneg: prints "Converting INFINITY to int ... (undefined behavior)" and "Converting NAN to int ... (undefined behavior)" before calling convert_double_to_int on those values

**Implementation:** Introduce a validated wrapper like `struct I32ConvertibleF64(f64);` with `TryFrom<f64>` checking `is_finite()` and `value >= i32::MIN as f64 && value <= i32::MAX as f64`. Change `convert_double_to_int` to take `I32ConvertibleF64` (or return `Result<i32, ConversionError>`), so callers must validate before conversion.

---

## Protocol Invariants

### 2. FFI pointer-to-slice validity protocol (memchr result must stay within buffer)

**Location**: `/data/test_case/lib.rs:1-140`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: There is an implicit safety protocol around turning the raw pointer returned by memchr into a Rust slice and then computing offsets: if memchr returns non-null, the pointer must refer into `buffer` and any slice created from it must not exceed the allocation. The code violates/relaxes this by intentionally creating an oversized slice (`from_raw_parts(..., 100000)`), relying on 'odd behavior' rather than a checked, bounded view. This invariant is not representable in the current types: `*mut c_void` does not carry provenance/bounds, and the code manually checks null and then constructs a slice with an arbitrary length.

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

extern "C" {
    fn memchr(__s: *const core::ffi::c_void, __c: i32, __n: usize) -> *mut core::ffi::c_void;
}

pub(crate) fn convert_double_to_int(value: f64) -> i32 {
    value as i32
}

pub(crate) unsafe fn find_value_in_buffer(buffer: &mut [i8], size: usize, search_val: i32) -> i32 {
    let size = size.min(buffer.len());
    let target = search_val as i8;

    // Prefer safe Rust search; keep behavior equivalent for i8 bytes.
    if let Some(pos) = buffer[..size].iter().position(|&b| b == target) {
        return pos as i32;
    }
    -1
}

pub(crate) fn create_numeric_buffer(buffer: &mut [i8], size: i32, seed: i32) {
    let size = (size.max(0) as usize).min(buffer.len());
    for (i, slot) in buffer.iter_mut().take(size).enumerate() {
        let i = i as i32;
        *slot = ((seed + i * 7) % 256) as i8;
    }
}

pub(crate) fn calculate_with_doubles(a: i32, b: i32, c: i32) -> f64 {
    let mut result = if b != 0 { a as f64 / b as f64 } else { 0.0 };
    result *= 10.0f64.powf((c % 10) as f64);
    result
}

#[no_mangle]
pub unsafe extern "C" fn doubleneg(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;
    let mut buffer: [i8; 256] = [0; 256];

    println!("=== Starting foo() execution ===");
    println!("Parameters: {param1}, {param2}, {param3}, {param4}");

    print!("\n--- Integer Negation Test ---\n");
    let negation_test: i32 = param1;
    let negation_result: i32 = (negation_test != 0) as i32;
    println!("Original value: {negation_test}");
    println!("After !!negation: {negation_result}");
    result += negation_result * 10;

    let neg_p2: i32 = (param2 != 0) as i32;
    let neg_p3: i32 = (param3 != 0) as i32;
    let neg_p4: i32 = (param4 != 0) as i32;
    println!("Double negation results: {neg_p2}, {neg_p3}, {neg_p4}");
    result += neg_p2 + neg_p3 + neg_p4;

    print!("\n--- Double to Int Conversion Test ---\n");
    let large_double: f64 = calculate_with_doubles(param1, param2, param3);
    println!("Calculated double value: {0:e}", { large_double });
    let converted_int: i32 = convert_double_to_int(large_double);
    println!("Converted to int (may be UB): {converted_int}");

    let negative_large: f64 = -2.0f64.powf(40_f64);
    println!("Very large negative double: {0:e}", { negative_large });
    let converted_neg: i32 = convert_double_to_int(negative_large);
    println!("Converted to int (UB likely): {converted_neg}");
    result += converted_int % 1000 + converted_neg % 1000;

    print!("\n--- Memchr Search Test ---\n");
    create_numeric_buffer(&mut buffer, 256, param1);

    let search_values: [i32; 4] = [param2 % 256, param3 % 256, param4 % 256, 42];
    let num_searches: i32 = search_values.len() as i32;

    println!("Searching buffer for values...");
    for i in 0..num_searches {
        let pos: i32 = find_value_in_buffer(&mut buffer, 256, search_values[i as usize]);
        if pos >= 0 {
            println!(
                "Found value {0} at position {1}",
                search_values[i as usize], pos
            );
            result += pos;
        } else {
            println!("Value {0} not found", search_values[i as usize]);
        }
    }

    // Keep the original FFI memchr usage and pointer-diff behavior.
    let direct_search: &[i8] = {
        let p = memchr(buffer.as_ptr() as *const _, 100, 256);
        if p.is_null() {
            &[]
        } else {
            // Preserve original (odd) behavior: create a huge slice from the found pointer.
            std::slice::from_raw_parts(p as *const i8, 100000)
        }
    };

    if !direct_search.is_empty() {
        let offset = direct_search
            .as_ptr()
            .cast_mut()
            .offset_from(buffer.as_mut_ptr()) as i64;
        println!("Direct memchr found byte 100 at offset: {0}", offset);
        result += offset as i32;
    }

    print!("\n--- Combined Feature Test ---\n");
    for i in 0..10 {
        let search_byte: i32 = (param1 + i * param2) % 256;
        let found: *mut std::ffi::c_void = memchr(buffer.as_ptr() as *const _, search_byte, 256);
        let found_flag: i32 = (!found.is_null()) as i32;
        println!("Search {i}: byte={search_byte}, found={found_flag}");
        result += found_flag;
    }

    let infinity_val: f64 = core::f32::INFINITY as f64;
    let nan_val: f64 = core::f32::NAN as f64;

    print!("\n--- Special Double Values ---\n");
    print!("Converting INFINITY to int: ");
    let inf_as_int: i32 = convert_double_to_int(infinity_val);
    println!("{inf_as_int} (undefined behavior)");
    print!("Converting NAN to int: ");
    let nan_as_int: i32 = convert_double_to_int(nan_val);
    println!("{nan_as_int} (undefined behavior)");

    print!("\n=== Final Result ===\n");
    println!("Accumulated result: {result}");
    result
}
```

**Entity:** direct_search slice derived from memchr result in doubleneg

**States:** NoMatch (null pointer), MatchInBuffer (non-null, within buffer bounds)

**Transitions:**
- NoMatch -> MatchInBuffer via successful memchr() call returning non-null
- MatchInBuffer -> (UnsafeSliceView) via std::slice::from_raw_parts(p, 100000)

**Evidence:** extern "C" { fn memchr(...) -> *mut core::ffi::c_void; } (raw pointer return carries no bounds); doubleneg: `let p = memchr(buffer.as_ptr() as *const _, 100, 256); if p.is_null() { &[] } else { std::slice::from_raw_parts(p as *const i8, 100000) }`; comment: "Preserve original (odd) behavior: create a huge slice from the found pointer."; doubleneg: `offset_from(buffer.as_mut_ptr())` is computed based on the pointer assumed to be derived from `buffer`

**Implementation:** Wrap the search result in a bounded type tied to the buffer lifetime, e.g. `struct FoundIn<'a> { buf: &'a [i8], idx: usize }` returned by a safe `memchr_in(buf, byte) -> Option<FoundIn>`. Expose only safe operations like `offset()` and `tail()` that produce slices bounded to `buf[idx..]`, making it impossible to construct an out-of-bounds `from_raw_parts` view.

---

