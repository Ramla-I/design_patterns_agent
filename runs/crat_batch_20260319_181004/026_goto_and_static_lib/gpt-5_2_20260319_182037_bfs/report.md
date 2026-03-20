# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Temporal Ordering Invariants

### 1. Thread-local 'y' configuration protocol for multi_stage (y must be set to 2 for success)

**Location**: `/data/test_case/lib.rs:1-49`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: `multi_stage(x, z)` implicitly depends on hidden thread-local state `y`. For the successful path, the caller must have previously set `y` to 2 (in the current thread) and must pass `x==1` and `z==3`. This dependency is not reflected in `multi_stage`'s signature (it takes only `x` and `z`) and is enforced only via runtime matching/printing. The API relies on `driver()` to establish the required `y` state via `y.set(local_y)` before calling `multi_stage`.

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
        thread_local! {
            static y: std::cell::Cell<i32> = const { std::cell::Cell::new(123) };
        }

        fn multi_stage(x: i32, z: i32) -> i32 {
            match (x, y.get(), z) {
                (1, 2, 3) => {
                    println!("Ok!");
                    0
                }
                (x, _, _) if x != 1 => {
                    println!("Error: x != 1");
                    println!("Operation failed");
                    1
                }
                (_, local_y, _) if local_y != 2 => {
                    println!("Error: x == 1 but y != 2");
                    println!("Operation failed");
                    2
                }
                _ => {
                    println!("Error: x == 1 and y == 2, but z != 3");
                    println!("Operation failed");
                    3
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn driver(x: i32, local_y: i32, z: i32) {
            y.set(local_y);
            let result = multi_stage(x, z);
            println!("Result: {result}");
        }
    }
}
```

**Entity:** thread_local! static y: Cell<i32>

**States:** Unconfigured/OtherValue(y!=2), Configured(y==2)

**Transitions:**
- Unconfigured/OtherValue -> Configured via y.set(2) (called from driver())
- Configured -> Unconfigured/OtherValue via y.set(value!=2) (called from driver())

**Evidence:** thread_local! { static y: std::cell::Cell<i32> ... } defines hidden mutable state; multi_stage(): match (x, y.get(), z) reads y via y.get() to decide success/failure; multi_stage(): arm (_, local_y, _) if local_y != 2 => prints "Error: x == 1 but y != 2" and returns 2; driver(): y.set(local_y); establishes y before calling multi_stage(x, z)

**Implementation:** Make the dependency explicit by removing the global and threading it through the type system: e.g., define a `struct StageCtx { y: i32 }` and require `multi_stage(x, z, ctx: &StageCtx)` or introduce a capability token `struct YIs2(())` returned only by a function that sets/validates `y==2`; then `multi_stage` takes `YIs2` (or `&ConfiguredCtx`) so it cannot be called unless the precondition has been established.

---

## Precondition Invariants

### 2. Multi-stage input/state precondition (x==1, y==2, z==3) encoded as runtime branching

**Location**: `/data/test_case/lib.rs:1-49`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `multi_stage` is effectively a validation pipeline with a single success condition `(x==1, y.get()==2, z==3)` and multiple distinct failure modes. The required values are not represented in the types (all are plain `i32`), so invalid combinations are only detected at runtime and mapped to numeric error codes (1/2/3) with printed diagnostics.

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
        thread_local! {
            static y: std::cell::Cell<i32> = const { std::cell::Cell::new(123) };
        }

        fn multi_stage(x: i32, z: i32) -> i32 {
            match (x, y.get(), z) {
                (1, 2, 3) => {
                    println!("Ok!");
                    0
                }
                (x, _, _) if x != 1 => {
                    println!("Error: x != 1");
                    println!("Operation failed");
                    1
                }
                (_, local_y, _) if local_y != 2 => {
                    println!("Error: x == 1 but y != 2");
                    println!("Operation failed");
                    2
                }
                _ => {
                    println!("Error: x == 1 and y == 2, but z != 3");
                    println!("Operation failed");
                    3
                }
            }
        }

        #[no_mangle]
        pub extern "C" fn driver(x: i32, local_y: i32, z: i32) {
            y.set(local_y);
            let result = multi_stage(x, z);
            println!("Result: {result}");
        }
    }
}
```

**Entity:** fn multi_stage(x: i32, z: i32) -> i32

**States:** Success(x=1,y=2,z=3), Failure(x!=1), Failure(x=1,y!=2), Failure(x=1,y=2,z!=3)

**Transitions:**
- AnyState -> Success by calling multi_stage with x=1,z=3 and having y set to 2
- AnyState -> Failure(...) by calling multi_stage with non-matching values

**Evidence:** multi_stage(): match (x, y.get(), z) explicitly enumerates required triple (1,2,3); multi_stage(): prints "Error: x != 1" and returns 1 for x!=1; multi_stage(): prints "Error: x == 1 but y != 2" and returns 2 for y!=2; multi_stage(): prints "Error: x == 1 and y == 2, but z != 3" and returns 3 for z!=3

**Implementation:** Encode required values as validated types/capabilities: e.g., `struct X1; struct Z3; struct Y2;` produced only by constructors that check the runtime integers (or by `TryFrom<i32>`). Then expose `multi_stage(x: X1, z: Z3, _y: Y2) -> ()` (or a richer result type) so impossible states are unrepresentable at the call site.

---

