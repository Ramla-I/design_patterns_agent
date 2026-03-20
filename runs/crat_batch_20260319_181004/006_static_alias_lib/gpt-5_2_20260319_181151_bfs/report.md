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

### 1. Aliased-pointer protocol for running_sum (Local outer ref vs thread-local INNER ref)

**Location**: `/data/test_case/lib.rs:1-49`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `driver` maintains `running_sum` as an `Option<&mut i32>` but updates it by calling `static_alias(...)` which returns a raw `*mut i32` that may point either to the caller-provided `outer_ref` (stack local `initial_value`) or to the thread-local `INNER`'s `RefCell` contents. The result is then converted back to `Option<&mut i32>` via `.as_mut()`. This creates an implicit protocol: the returned pointer must be non-null and must remain valid/mutably borrowable for the duration of its use in the loop iteration. None of that is enforced by the type system because the raw pointer erases provenance/lifetime, and the `Option<&mut i32>` state (Some/None) is not tied to a proof that the pointer is valid. In particular, the code relies on `static_alias` never returning null when given `Some`, and on the thread-local borrow not escaping (even though a pointer into `inner.borrow_mut()` is returned).

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
        use std::cell::RefCell;

        thread_local! {
            static INNER: RefCell<i32> = const { RefCell::new(1) };
        }

        // === staticalias.rs ===
        #[no_mangle]
        pub extern "C" fn static_alias(mut outer: Option<&mut i32>) -> *mut i32 {
            let Some(outer_ref) = outer.as_deref_mut() else {
                return std::ptr::null_mut();
            };

            INNER.with(|inner| {
                let mut inner_val = inner.borrow_mut();
                if *outer_ref >= *inner_val {
                    *inner_val += *outer_ref;
                    (&mut *inner_val) as *mut i32
                } else {
                    *outer_ref += *inner_val;
                    outer_ref as *mut i32
                }
            })
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(mut initial_value: i32, iterations: i32) {
            let mut running_sum: Option<&mut i32> = Some(&mut initial_value);

            for _ in 0..iterations {
                running_sum = static_alias(running_sum.as_deref_mut()).as_mut();
                println!("{0}", *running_sum.as_deref().unwrap());
            }
        }
    }
}
```

**Entity:** driver / static_alias interaction via running_sum: Option<&mut i32> and *mut i32

**States:** OuterPtr, InnerPtr, NullPtr (invalid)

**Transitions:**
- OuterPtr -> InnerPtr via static_alias() when *outer_ref >= *inner_val
- OuterPtr -> OuterPtr via static_alias() when *outer_ref < *inner_val
- InnerPtr -> InnerPtr via static_alias() (subsequent iterations may keep returning INNER depending on comparisons)
- Any -> NullPtr (invalid) if static_alias(None) is called (currently avoided by driver but not represented in types)

**Evidence:** static_alias signature: `pub extern "C" fn static_alias(mut outer: Option<&mut i32>) -> *mut i32` returns a raw pointer that may be null; `let Some(outer_ref) = outer.as_deref_mut() else { return std::ptr::null_mut(); };` encodes the NullPtr state; Branch returns two different pointer origins: `(&mut *inner_val) as *mut i32` (thread-local INNER) vs `outer_ref as *mut i32` (caller stack); driver converts raw pointer back into `Option<&mut i32>`: `running_sum = static_alias(...).as_mut();` (lifetime/provenance unchecked); driver assumes non-null by unwrapping each iteration: `running_sum.as_deref().unwrap()`

**Implementation:** Replace `*mut i32` with a tagged, lifetime-carrying enum to preserve provenance: `enum Alias<'a> { Outer(&'a mut i32), Inner(InnerRef<'a>) }`, where `InnerRef<'a>` is a wrapper that does not outlive the `INNER.with` borrow (or redesign to return only within a closure). Alternatively, make `static_alias` take a closure `fn with_alias<R>(outer: &mut i32, f: impl for<'a> FnOnce(Alias<'a>) -> R) -> R` so the inner borrow cannot escape.

---

