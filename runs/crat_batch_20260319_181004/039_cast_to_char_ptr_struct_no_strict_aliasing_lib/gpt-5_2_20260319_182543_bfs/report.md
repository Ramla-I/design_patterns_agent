# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 2

## Protocol Invariants

### 1. FFI memcpy protocol (valid pointers + correct size/representation)

**Location**: `/data/test_case/lib.rs:1-62`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The function relies on an implicit FFI protocol: calling `memcpy` is only sound if the destination pointer is valid for writes of `size_of::<house_t>()` bytes, the source pointer is valid for reads of the same size, and `house_t` has a stable C representation compatible with raw byte copying. After the call, only the first `size_of::<house_t>()` bytes of `raw` are initialized; printing assumes the whole 16-byte buffer is valid to read. These requirements are enforced by `unsafe` and conventions, not by the type system (e.g., there is no type-level link between the destination buffer length and `size_of::<house_t>()`, and the use of raw pointers bypasses Rust’s usual initialization/aliasing checks).

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
extern "C" {
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

#[inline]
fn xu32_identity(x: u32) -> u32 {
    // The original C2Rust used `crate::c_lib::Xu32`, but that module isn't available here.
    // Preserve behavior needed for formatting by using an identity mapping.
    x
}

fn print_hex(bytes: &[u8]) {
    for &b in bytes {
        print!("{:02x}", xu32_identity(b as u32));
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

    let mut raw = [0u8; 16];

    // Keep the original FFI memcpy call.
    memcpy(
        raw.as_mut_ptr() as *mut core::ffi::c_void,
        (&house as *const house_t) as *const core::ffi::c_void,
        core::mem::size_of::<house_t>(),
    );

    print_hex(&raw);
}
```

**Entity:** driver (unsafe extern "C" fn)

**States:** Prepared (valid source/dest + correct layout), Copied (bytes initialized in destination buffer)

**Transitions:**
- Prepared -> Copied via memcpy(...)

**Evidence:** fn driver is declared `pub unsafe extern "C" fn driver(...)` (unsafe boundary implies unchecked preconditions); `memcpy(raw.as_mut_ptr() as *mut c_void, (&house as *const house_t) as *const c_void, size_of::<house_t>())` uses raw pointers and a runtime byte count; `let mut raw = [0u8; 16];` hard-codes buffer length independently of `core::mem::size_of::<house_t>()`; `print_hex(&raw);` prints all 16 bytes regardless of how many bytes were copied

**Implementation:** Replace raw `memcpy` usage with a safe, size-coupled API: e.g., `let raw: [u8; core::mem::size_of::<house_t>()] = unsafe { core::mem::transmute(house) };` for `Copy` POD, or a helper `fn as_bytes<T: Copy>(t: &T) -> &[u8; size_of::<T>()]` built on `bytemuck::bytes_of`/`Pod` so the buffer length is type-level. If FFI `memcpy` must remain, wrap it in `fn memcpy_into_array<T: Copy>(src: &T) -> [u8; size_of::<T>()]` so the destination length cannot diverge from `T`.

---

