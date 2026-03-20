# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## State Machine Invariants

### 2. Increment-to-next-9 loop invariant (ArbitraryI32 -> EndsWith9)

**Location**: `/data/test_case/main.rs:1-40`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `val` is treated as moving through a small state machine: it starts as an arbitrary parsed i32, then the loop increments until it reaches a value whose last decimal digit is 9 (`val % 10 == 9`), at which point it stops. The code relies on the runtime check `val % 10 == 9` to decide termination; the type system does not encode the 'ends with 9' property, so nothing prevents using a value assumed to be in the terminal state elsewhere without re-checking.

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

fn main() {
    let mut args = std::env::args();
    let _prog = args.next();
    let first = args.next();
    let extra = args.next();

    if first.is_none() || extra.is_some() {
        println!("Error: should only be a single (integer) argument!");
        std::process::exit(1);
    }

    let s = first.unwrap();
    let mut val: i32 = match s.parse::<i32>() {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    loop {
        println!("{val}");
        if val % 10 == 9 {
            break;
        }
        val += 1;
    }

    std::process::exit(0);
}
```

**Entity:** Loop variable `val: i32` in main()

**States:** ArbitraryI32, EndsWith9

**Transitions:**
- ArbitraryI32 -> ArbitraryI32 via `val += 1` when `val % 10 != 9`
- ArbitraryI32 -> EndsWith9 via loop reaching `if val % 10 == 9 { break; }`

**Evidence:** loop { println!("{val}"); if val % 10 == 9 { break; } val += 1; } (runtime predicate defines terminal state)

**Implementation:** Model the terminal property with a validated newtype, e.g. `struct EndsWith9I32(i32); impl TryFrom<i32> for EndsWith9I32 { ... }` (checking `% 10 == 9`), and/or a function `fn advance_to_ends_with_9(v: i32) -> EndsWith9I32` to make the transition explicit and type-visible.

---

## Protocol Invariants

### 1. CLI arity & integer-parse precondition (ValidArgs -> ValidInt)

**Location**: `/data/test_case/main.rs:1-40`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program relies on a multi-step runtime protocol for input validity: it first checks that exactly one non-program argument exists (arity validation), then unwraps that argument, then parses it as i32. These steps are enforced via runtime checks plus early process exit; the type system does not distinguish unchecked vs validated/parsed input, so `unwrap()` and subsequent logic assume the earlier checks ran and succeeded.

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

fn main() {
    let mut args = std::env::args();
    let _prog = args.next();
    let first = args.next();
    let extra = args.next();

    if first.is_none() || extra.is_some() {
        println!("Error: should only be a single (integer) argument!");
        std::process::exit(1);
    }

    let s = first.unwrap();
    let mut val: i32 = match s.parse::<i32>() {
        Ok(v) => v,
        Err(_) => {
            println!("Error: first argument must be an integer!");
            std::process::exit(1);
        }
    };

    loop {
        println!("{val}");
        if val % 10 == 9 {
            break;
        }
        val += 1;
    }

    std::process::exit(0);
}
```

**Entity:** Program CLI argument handling in main()

**States:** ArgsUnchecked, ArityValidated, ParsedInt

**Transitions:**
- ArgsUnchecked -> ArityValidated via `if first.is_none() || extra.is_some() { ... exit(1) }`
- ArityValidated -> ParsedInt via `first.unwrap()` + `s.parse::<i32>()` success (else exit(1))

**Evidence:** let first = args.next(); let extra = args.next(); (Option-based unchecked presence); if first.is_none() || extra.is_some() { println!("Error: should only be a single (integer) argument!"); std::process::exit(1); } (arity gate + error message); let s = first.unwrap(); (assumes arity validation already happened); match s.parse::<i32>() { Ok(v) => v, Err(_) => { println!("Error: first argument must be an integer!"); std::process::exit(1); } } (parse gate + error message)

**Implementation:** Introduce a parsing layer that returns a newtype capturing validated input, e.g. `struct SingleI32Arg(i32); impl TryFrom<std::env::Args> for SingleI32Arg { ... }`, so `main()` receives `SingleI32Arg` (or `Result<SingleI32Arg, Error>`) and cannot call `unwrap()` on unchecked `Option`/`Result`.

---

