# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 1

## State Machine Invariants

### 1. Thread-local accumulator protocol (Uninitialized-at-thread-start / Accumulating)

**Location**: `/data/test_case/main.rs:1-43`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: static_sum relies on implicit per-thread mutable state stored in a thread-local Cell<i32> (SUM). Each thread starts with SUM = 0, and every call to static_sum(update) mutates this hidden state by adding update. Callers cannot see or control which logical 'accumulator instance' they are updating, and the API does not encode that the result depends on call history and thread identity (per-thread state). This protocol (thread start at zero, then repeated updates) is enforced only by the use of thread_local! + Cell, not by explicit types passed through the call chain.

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

use std::cell::Cell;

pub(crate) fn static_sum(update: i32) -> i32 {
    thread_local! {
        static SUM: Cell<i32> = const { Cell::new(0) };
    }
    SUM.with(|s| {
        s.set(s.get() + update);
        s.get()
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Error: should only be a single (integer) argument!");
        std::process::exit(1);
    }

    let stride: i32 = match args[1].parse::<i32>() {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    for i in 0..10 {
        println!("{}", static_sum(i * stride));
    }
}
```

**Entity:** static_sum / thread-local SUM: Cell<i32>

**States:** ThreadStart(Zero), Accumulating

**Transitions:**
- ThreadStart(Zero) -> Accumulating via first call to static_sum(update) (SUM.with + set/get)
- Accumulating -> Accumulating via subsequent calls to static_sum(update)

**Evidence:** static_sum(): `thread_local! { static SUM: Cell<i32> = const { Cell::new(0) }; }` encodes implicit per-thread state initialized to 0; static_sum(): `SUM.with(|s| { s.set(s.get() + update); s.get() })` shows stateful update and dependence on prior calls

**Implementation:** Make the state explicit by introducing an accumulator type and passing it around: `struct Sum(Cell<i32>); impl Sum { fn update(&self, delta: i32) -> i32 { ... } }`. If you still want TLS, return a capability/token that grants access: `fn sum_handle() -> &'static Sum` (or `fn with_sum<R>(f: impl FnOnce(&Sum)->R)->R`), so callers must explicitly opt into using the thread-local accumulator rather than calling a globally stateful function.

---

## Precondition Invariants

### 2. CLI argument validity precondition (ExactlyOneArg + ParseableI32)

**Location**: `/data/test_case/main.rs:1-43`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: main assumes a specific input protocol: exactly one CLI argument (besides argv[0]) and that it parses as i32. These validity requirements are enforced by runtime checks and process termination, not by types. Downstream code (the loop calling static_sum) implicitly depends on having a valid `stride: i32` produced by successful argument validation/parsing.

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

use std::cell::Cell;

pub(crate) fn static_sum(update: i32) -> i32 {
    thread_local! {
        static SUM: Cell<i32> = const { Cell::new(0) };
    }
    SUM.with(|s| {
        s.set(s.get() + update);
        s.get()
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Error: should only be a single (integer) argument!");
        std::process::exit(1);
    }

    let stride: i32 = match args[1].parse::<i32>() {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    for i in 0..10 {
        println!("{}", static_sum(i * stride));
    }
}
```

**Entity:** stride (parsed from CLI argument)

**States:** ArgsInvalid, ArgsValid(StrideI32)

**Transitions:**
- ArgsInvalid -> process::exit(1) via `if args.len() != 2` branch
- ArgsInvalid -> process::exit(1) via `Err(_) => { ... exit(1) }` parse branch
- ArgsValid(StrideI32) -> ArgsValid(StrideI32) via successful `parse::<i32>()` producing `stride`

**Evidence:** main(): `if args.len() != 2 { println!("Error: should only be a single (integer) argument!"); std::process::exit(1); }` encodes 'exactly one argument' invariant; main(): `match args[1].parse::<i32>() { Ok(v) => v, Err(_) => { println!("Error: first argument must be an integer!"); std::process::exit(1); } }` encodes 'must parse as i32' invariant

**Implementation:** Factor argument handling into a parser that returns a typed value: `struct Stride(i32); impl TryFrom<&[String]> for Stride { type Error = ...; fn try_from(args: &[String]) -> Result<Self, _> { ... } }`. Then `main` can be `let stride: Stride = args.as_slice().try_into()?;` and the rest of the program only accepts `Stride`, preventing accidental use without validation.

---

