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

### 1. FFI raw memory protocol (valid pointers / sizes / type-punning preconditions)

**Location**: `/data/test_case/lib.rs:1-40`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: driver() relies on an unsafe FFI memcpy call to populate a stack buffer with the byte representation of an f32, then reinterprets that buffer as u8 bytes for printing. The correctness depends on implicit preconditions: the destination pointer must be valid and writable for size_of::<f32>() bytes, the source pointer must be valid and readable for the same size, and the copy size must match the actual buffer length. After memcpy completes, raw is assumed initialized and safe to view as bytes. None of these preconditions/transition (uninitialized -> initialized) is expressed in the type system; safety is ensured only by manual reasoning inside an unsafe function.

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

// === driver.rs ===
extern "C" {
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
}

#[inline]
fn print_hex(p: &[u8], len: i32) {
    let n = (len.max(0) as usize).min(p.len());
    for &b in &p[..n] {
        print!("{:02x}", b);
    }
    println!();
}

#[no_mangle]
pub unsafe extern "C" fn driver(x: f32) {
    let mut raw: [i8; 4] = [0; 4];
    memcpy(
        raw.as_mut_ptr().cast::<core::ffi::c_void>(),
        (&x as *const f32).cast::<core::ffi::c_void>(),
        core::mem::size_of::<f32>(),
    );

    let bytes: &[u8] = bytemuck::cast_slice(&raw);
    print_hex(bytes, raw.len() as i32);
}
```

**Entity:** driver (unsafe extern "C" fn)

**States:** Raw bytes not yet initialized, Raw bytes initialized with f32 representation

**Transitions:**
- Raw bytes not yet initialized -> Raw bytes initialized with f32 representation via memcpy(...)

**Evidence:** extern "C" fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) introduces raw-pointer/size preconditions; driver(): let mut raw: [i8; 4] = [0; 4]; then memcpy(raw.as_mut_ptr().cast::<c_void>(), (&x as *const f32).cast::<c_void>(), core::mem::size_of::<f32>()); driver(): bytemuck::cast_slice(&raw) assumes the raw buffer can be safely reinterpreted as bytes after the copy

**Implementation:** Avoid FFI memcpy and encode the transition using safe Rust: let bytes: [u8; 4] = x.to_ne_bytes(); then pass &bytes to print_hex. If the intent is explicitly 'bytes of an f32', introduce a newtype like struct F32Bytes([u8; 4]); impl From<f32> for F32Bytes { ... } so byte-extraction is always size-correct without raw pointers.

---

