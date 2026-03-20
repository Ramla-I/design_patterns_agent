# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 0
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 1. Per-thread running-sum accumulator protocol (Initialized-at-0 / Updated)

**Location**: `/data/test_case/lib.rs:1-36`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: SUM is a per-thread mutable accumulator whose current value is implicit state. Calls to static_sum(update) read-modify-write this state (wrapping_add) and return the new value. The type system does not express that static_sum/driver have side effects, that the value is per-thread (not global), or that the accumulator starts at 0 and then evolves only through wrapping addition. Any caller can invoke static_sum in any order; the meaning of results depends on the entire history of prior calls on that thread.

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

pub mod src {
    pub mod lib {
        use std::cell::Cell;

        thread_local! {
            static SUM: Cell<i32> = const { Cell::new(0) };
        }

        #[no_mangle]
        pub unsafe extern "C" fn static_sum(update: i32) -> i32 {
            SUM.with(|sum| {
                let new_val = sum.get().wrapping_add(update);
                sum.set(new_val);
                new_val
            })
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(stride: i32) {
            for i in 0..10i32 {
                println!("{0}", static_sum(i.wrapping_mul(stride)));
            }
        }
    }
}
```

**Entity:** thread_local! SUM: Cell<i32>

**States:** Initialized(0), Updated(n)

**Transitions:**
- Initialized(0) -> Updated(update) via static_sum(update)
- Updated(n) -> Updated(n.wrapping_add(update)) via static_sum(update)

**Evidence:** thread_local! { static SUM: Cell<i32> = const { Cell::new(0) }; } defines implicit mutable state with initial value 0; static_sum(update): let new_val = sum.get().wrapping_add(update); sum.set(new_val); encodes the state transition; driver(stride): calls static_sum(...) in a loop, relying on accumulated history for printed values

**Implementation:** Make the state explicit and require a token/capability to mutate it: e.g., struct SumAcc { cell: Cell<i32> } stored in TLS, and expose a safe API like fn with_sum<R>(f: impl FnOnce(&SumHandle) -> R) where SumHandle is a non-Send, thread-bound capability that provides add_wrapping(&self, update: i32) -> i32. This makes the protocol explicit (per-thread handle, side effects) and removes the need for unsafe extern access for Rust callers.

---

