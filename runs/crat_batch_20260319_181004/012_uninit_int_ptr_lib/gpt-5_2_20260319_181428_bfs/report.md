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

### 1. Driver mode protocol (Good-path initialized data vs Bad-path None)

**Location**: `/data/test_case/lib.rs:1-39`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: driver uses an integer flag as a protocol selector: in GoodMode it constructs a valid reference and calls printIntPtrLine; in BadMode it routes to bad(), which deterministically passes None and triggers the unwrap panic. This relies on callers providing a valid mode value to avoid runtime failure, but the FFI signature `extern "C" fn driver(useGood: i32)` cannot enforce the allowed modes or prevent selecting the failing path.

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

pub mod src {
    pub mod lib {
        // === driver.rs ===
        pub(crate) fn printIntPtrLine(intNumber: Option<&i32>) {
            // Preserve original behavior: unwrap and panic on None
            println!("{0}", *intNumber.unwrap());
        }

        pub(crate) fn bad() {
            let data: Option<&i32> = None;
            printIntPtrLine(data);
        }

        pub(crate) fn good() {
            let data: i32 = 5;
            let data_addr: Option<&i32> = Some(&data);
            printIntPtrLine(data_addr);
        }

        #[no_mangle]
        pub extern "C" fn driver(useGood: i32) {
            if useGood != 0 {
                good();
            } else {
                bad();
            };
        }
    }
}
```

**Entity:** driver(useGood: i32) control flag

**States:** GoodMode (useGood != 0), BadMode (useGood == 0)

**Transitions:**
- BadMode -> GoodMode by passing a nonzero `useGood` when calling driver()

**Evidence:** driver(): `if useGood != 0 { good(); } else { bad(); }` encodes the two modes; bad(): passes `None` into `printIntPtrLine`, which unwrap-panics; good(): passes `Some(&data)` into `printIntPtrLine`

**Implementation:** Expose a safe Rust API like `enum Mode { Good, Bad }` or `struct UseGood(bool)` and provide an FFI wrapper that converts `i32` into `Result<Mode, _>` (rejecting invalid/undesired values) before calling the internal implementation; keep the `extern "C"` entrypoint thin and validated.

---

