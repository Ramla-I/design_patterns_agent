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

### 1. FFI precondition: `floors` must be a valid domain value for a `house_t`

**Location**: `/data/test_case/lib.rs:1-43`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The FFI entrypoint `driver(floors: i32)` blindly constructs a `house_t` using the provided `floors` value. There is an implicit assumption that `floors` is within an acceptable domain (e.g., non-negative / reasonable range). This precondition is not enforced by the type system: any `i32` can be passed across the FFI boundary, and `unsafe` signals that the caller must uphold requirements, but those requirements are not encoded as types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

// === driver.rs ===
#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

fn print_hex(bytes: &[u8]) {
    for &b in bytes {
        // Avoid depending on a missing `crate::c_lib::Xu32`; formatting already expects u32.
        print!("{:02x}", b as u32);
    }
    println!();
}

#[no_mangle]
pub unsafe extern "C" fn driver(floors: i32) {
    let house = house_t {
        floors,
        bedrooms: 3,
        bathrooms: 2.0f64,
    };

    let bytes = std::slice::from_raw_parts(
        std::ptr::from_ref(&house).cast::<u8>(),
        ::core::mem::size_of::<house_t>(),
    );
    print_hex(bytes);
}
```

**Entity:** driver (unsafe extern "C" fn)

**States:** ValidInput, InvalidInput

**Transitions:**
- InvalidInput -> ValidInput via validation before constructing `house_t` (not present in code)

**Evidence:** `pub unsafe extern "C" fn driver(floors: i32)` takes an unconstrained `i32` from FFI; `let house = house_t { floors, bedrooms: 3, bathrooms: 2.0f64 }` directly embeds `floors` without validation

**Implementation:** Introduce a `Floors` newtype with a checked constructor, e.g. `struct Floors(i32); impl TryFrom<i32> for Floors { ... }`, and have an internal safe function `fn driver_checked(floors: Floors)`; keep the extern `driver` as a thin wrapper that validates/returns early.

---

## Protocol Invariants

### 2. C-layout/bytes protocol: only safe to treat `house_t` as raw bytes under stable repr(C) + POD assumptions

**Location**: `/data/test_case/lib.rs:1-43`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The code reinterprets a `&house_t` as a `&[u8]` and prints its raw memory representation. This relies on an implicit protocol: `house_t` must remain a plain-old-data (POD) type with `#[repr(C)]`, no padding-dependent semantics, and no internal pointers/references that would make raw-byte output meaningless or potentially sensitive. Rust does not enforce 'POD/NoPaddingMeaningful' as a trait bound here, nor does it prevent future edits to `house_t` (adding non-POD fields) from silently breaking the intended behavior.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

// === driver.rs ===
#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

fn print_hex(bytes: &[u8]) {
    for &b in bytes {
        // Avoid depending on a missing `crate::c_lib::Xu32`; formatting already expects u32.
        print!("{:02x}", b as u32);
    }
    println!();
}

#[no_mangle]
pub unsafe extern "C" fn driver(floors: i32) {
    let house = house_t {
        floors,
        bedrooms: 3,
        bathrooms: 2.0f64,
    };

    let bytes = std::slice::from_raw_parts(
        std::ptr::from_ref(&house).cast::<u8>(),
        ::core::mem::size_of::<house_t>(),
    );
    print_hex(bytes);
}
```

**Entity:** house_t

**States:** ByteSerializable, NotByteSerializable

**Transitions:**
- NotByteSerializable -> ByteSerializable via restricting `house_t` to POD-only fields and asserting POD-ness at compile time

**Evidence:** `#[repr(C)] pub struct house_t { ... }` indicates intent to match a C ABI/layout; `std::slice::from_raw_parts(std::ptr::from_ref(&house).cast::<u8>(), ::core::mem::size_of::<house_t>())` reinterprets the struct as bytes; `print_hex(bytes);` consumes the raw memory representation as a serialization/diagnostic format

**Implementation:** Gate the byte-view behind a trait/capability implemented only for POD types, e.g. using `bytemuck::Pod`/`Zeroable`: `fn as_bytes<T: Pod>(t: &T) -> &[u8] { bytemuck::bytes_of(t) }`. This makes the 'safe-to-view-as-bytes' invariant explicit and compile-time checked for `house_t`.

---

