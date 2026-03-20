# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. FFI boundary safety precondition for exported driver()

**Location**: `/data/test_case/lib.rs:1-23`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: `driver` is exported as `extern "C"` and marked `unsafe`, implying a latent contract: it should only be invoked in a context that satisfies the FFI preconditions (correct ABI, no unwinding across the boundary, and any global/runtime initialization required by printing). The function body itself does not enforce or encode these preconditions; the only indicator is the `unsafe extern "C"` signature and `#[no_mangle]` export. This is a protocol at the API boundary that could be made harder to misuse from Rust by providing a safe wrapper and limiting direct access to the unsafe symbol.

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
fn print_hex(p: &[u8], len: i32) {
    for &b in p.iter().take(len as usize) {
        print!("{0:>02x}", b as u32);
    }
    println!();
}

#[no_mangle]
pub unsafe extern "C" fn driver(x: i32) {
    let bytes = x.to_ne_bytes();
    print_hex(&bytes, bytes.len() as i32);
}
```

**Entity:** driver (unsafe extern "C" fn)

**States:** Rust-only call context (safe assumptions hold), FFI call context (caller-controlled, safety assumptions required)

**Transitions:**
- Rust safe wrapper -> calls unsafe driver() after establishing FFI preconditions

**Evidence:** `#[no_mangle] pub unsafe extern "C" fn driver(x: i32)` indicates an FFI boundary with an implicit safety contract; `print!`/`println!` used inside `driver` suggests reliance on Rust runtime/I/O behavior not representable in the C ABI contract

**Implementation:** Expose a safe Rust API like `pub fn driver_safe(x: i32)` and keep `pub unsafe extern "C" fn driver` behind a module boundary; optionally require a capability token (e.g., `struct FfiReady(())`) created by an initialization function, and accept `FfiReady` as an argument to the safe wrapper to encode 'initialized/allowed to call' at the type level.

---

