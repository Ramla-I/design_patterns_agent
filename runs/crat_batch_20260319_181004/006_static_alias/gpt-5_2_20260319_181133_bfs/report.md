# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## Temporal Ordering Invariants

### 2. CLI input validation protocol (ArgCount=3 and i32-parseable args)

**Location**: `/data/test_case/main.rs:1-77`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: main() assumes a validated command-line shape before proceeding: exactly two user arguments must be present (args.len()==3 including program name) and both must parse as i32 after trimming. This is enforced by runtime checks followed by process exit, but the rest of main is written as if in a "validated" state. The type system does not reflect the transition from unchecked strings to validated numeric arguments, so later code relies on having performed the checks (and on early exits).

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

use std::cell::RefCell;

fn static_alias(outer: &mut i32) -> *mut i32 {
    thread_local! {
        static INNER: RefCell<i32> = const { RefCell::new(1) };
    }

    if INNER.with_borrow(|inner_ref| *outer >= *inner_ref) {
        INNER.with_borrow_mut(|inner_ref| *inner_ref += *outer);
        INNER.with_borrow_mut(|inner_ref| inner_ref as *mut i32)
    } else {
        INNER.with_borrow(|inner_ref| *outer += *inner_ref);
        outer as *mut i32
    }
}

fn parse_i32_arg(s: &str, err_msg: &str) -> Result<i32, ()> {
    // Match C strtol behavior used here: accept leading/trailing whitespace,
    // but reject if no digits were parsed at all.
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(());
    }
    match trimmed.parse::<i32>() {
        Ok(v) => Ok(v),
        Err(_) => Err(()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Error: should only be two (integer) arguments!");
        std::process::exit(1);
    }

    let mut initial_value = match parse_i32_arg(&args[1], "Error: first argument must be an integer!") {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    let iterations = match parse_i32_arg(&args[2], "Error: second argument must be an integer!") {
        Ok(v) => v,
        Err(_) => {
            println!("Error: second argument must be an integer!");
            std::process::exit(1);
        }
    };

    let mut running_ptr: *mut i32 = &mut initial_value;

    let mut i = 0i32;
    while i < iterations {
        unsafe {
            running_ptr = static_alias(&mut *running_ptr);
            println!("{0}", *running_ptr);
        }
        i += 1;
    }

    std::process::exit(0);
}
```

**Entity:** main (CLI argument parsing contract)

**States:** ArgsUnchecked, ArgsValidated

**Transitions:**
- ArgsUnchecked -> ArgsValidated via `if args.len() != 3 { ... exit(1) }` and successful `parse_i32_arg` for both arguments

**Evidence:** main: `if args.len() != 3 { println!("Error: should only be two (integer) arguments!"); std::process::exit(1); }` encodes the required argument count as a runtime check; main: `match parse_i32_arg(&args[1], "Error: first argument must be an integer!") { ... Err(_) => { println!(...); exit(1) } }` runtime validation that arg1 is an i32; main: `match parse_i32_arg(&args[2], "Error: second argument must be an integer!") { ... Err(_) => { println!(...); exit(1) } }` runtime validation that arg2 is an i32; parse_i32_arg comment: `accept leading/trailing whitespace, but reject if no digits were parsed at all` documents an input protocol beyond plain `parse()`

**Implementation:** Introduce a parsed-args struct/newtypes: `struct ParsedArgs { initial: i32, iterations: NonNegativeI32 }` (or `usize` for iterations) with `impl TryFrom<Vec<String>> for ParsedArgs`. After construction, main operates only on `ParsedArgs`, making the "validated" state explicit and preventing use of raw `args[1]`/`args[2]` without checks.

---

## Protocol Invariants

### 1. Pointer-provenance / aliasing protocol for returned raw pointer (Outer vs INNER)

**Location**: `/data/test_case/main.rs:1-77`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: static_alias() returns a raw mutable pointer that is conditionally either (a) a pointer derived from the caller-provided &mut i32 (outer), or (b) a pointer into a thread-local RefCell<i32> (INNER). Subsequent code treats the pointer uniformly (dereference, reborrow as &mut i32, and pass back into static_alias), but correctness relies on an implicit protocol: the pointer must be dereferenceable, must refer to an i32 with appropriate lifetime (either the caller stack slot or the thread-local), and must not be used in ways that violate aliasing rules when converting between *mut i32 and &mut i32. The type system does not encode which storage the pointer refers to, nor the provenance/lifetime constraints, so callers rely on unsafe and discipline.

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

use std::cell::RefCell;

fn static_alias(outer: &mut i32) -> *mut i32 {
    thread_local! {
        static INNER: RefCell<i32> = const { RefCell::new(1) };
    }

    if INNER.with_borrow(|inner_ref| *outer >= *inner_ref) {
        INNER.with_borrow_mut(|inner_ref| *inner_ref += *outer);
        INNER.with_borrow_mut(|inner_ref| inner_ref as *mut i32)
    } else {
        INNER.with_borrow(|inner_ref| *outer += *inner_ref);
        outer as *mut i32
    }
}

fn parse_i32_arg(s: &str, err_msg: &str) -> Result<i32, ()> {
    // Match C strtol behavior used here: accept leading/trailing whitespace,
    // but reject if no digits were parsed at all.
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(());
    }
    match trimmed.parse::<i32>() {
        Ok(v) => Ok(v),
        Err(_) => Err(()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Error: should only be two (integer) arguments!");
        std::process::exit(1);
    }

    let mut initial_value = match parse_i32_arg(&args[1], "Error: first argument must be an integer!") {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    let iterations = match parse_i32_arg(&args[2], "Error: second argument must be an integer!") {
        Ok(v) => v,
        Err(_) => {
            println!("Error: second argument must be an integer!");
            std::process::exit(1);
        }
    };

    let mut running_ptr: *mut i32 = &mut initial_value;

    let mut i = 0i32;
    while i < iterations {
        unsafe {
            running_ptr = static_alias(&mut *running_ptr);
            println!("{0}", *running_ptr);
        }
        i += 1;
    }

    std::process::exit(0);
}
```

**Entity:** static_alias (thread-local INNER + returned *mut i32)

**States:** PtrToOuter, PtrToThreadLocalInner

**Transitions:**
- PtrToOuter -> PtrToThreadLocalInner via static_alias() when `*outer >= *inner_ref`
- PtrToThreadLocalInner -> PtrToOuter via static_alias() when `*outer < *inner_ref` (because `outer` is then updated and its pointer is returned)

**Evidence:** fn static_alias(outer: &mut i32) -> *mut i32: returns a raw mutable pointer rather than a reference, losing lifetime/provenance information; thread_local! { static INNER: RefCell<i32> = ... }: introduces a second storage location that the returned pointer may refer to; if INNER.with_borrow(|inner_ref| *outer >= *inner_ref) { ... inner_ref as *mut i32 } else { ... outer as *mut i32 }: explicitly returns pointers to two different origins depending on a runtime condition; main: `running_ptr = static_alias(&mut *running_ptr);` reconstitutes `&mut i32` from a `*mut i32` (`&mut *running_ptr`) inside `unsafe`, relying on an implicit validity/uniqueness invariant for that pointer

**Implementation:** Return a tagged/typed handle instead of `*mut i32`, e.g. `enum Alias<'a> { Outer(&'a mut i32), Inner(ThreadLocalInnerHandle) }` where `ThreadLocalInnerHandle` provides controlled access to INNER (e.g. methods that borrow via RefCell internally). Alternatively use `enum AliasPtr { Outer(NonNull<i32>), Inner(NonNull<i32>) }` plus separate APIs for reborrowing to avoid `&mut *ptr` and keep the protocol explicit.

---

