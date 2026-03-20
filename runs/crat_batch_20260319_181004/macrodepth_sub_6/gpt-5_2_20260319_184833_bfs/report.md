# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Precondition Invariants

### 1. CLI argument arity + parse validity preconditions (ArgsPresent/ArgsMissing, Parsed/Defaulted)

**Location**: `/data/test_case/main.rs:1-121`

**Confidence**: high

**Suggested Pattern**: builder

**Description**: main() implicitly requires at least two positional arguments (A and B). If fewer are provided, the program exits early. It also implicitly expects args[1] and args[2] to be valid i32 strings, but instead of enforcing successful parsing it silently defaults to 0 on parse failure. These are runtime-enforced (length check + exit) and convention-enforced (expect numeric strings), not enforced by the type system.

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

pub const INIT_sub: i32 = 0;
pub const REPEAT: i32 = 6;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_sub(n: i32) -> i32 {
    let mut acc: i32 = INIT_sub;
    match n {
        1 => {
            acc -= 0;
        }
        2 => {
            acc -= 0;
            acc -= 1;
        }
        3 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
        }
        4 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
        }
        5 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
            acc -= 4;
        }
        6 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
            acc -= 4;
            acc -= 5;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_sub;
static G_OP_NAME: &str = "sub";

fn helper_call(a: i32, b: i32) -> i32 {
    let r: i32 = op_sub(a, b);
    let mut acc: i32 = INIT_sub;
    acc -= 0;
    acc -= 1;
    acc -= 2;
    acc -= 3;
    acc -= 4;
    acc -= 5;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_sub;
    let r: i32 = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r: i32 = accum_sub(n);
    println!("gen.acc={r}");
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} A B", args.get(0).map(String::as_str).unwrap_or(""));
        std::process::exit(2);
    }

    let a: i32 = args[1].parse().unwrap_or(0);
    let b: i32 = args[2].parse().unwrap_or(0);

    let r_call: i32 = op_sub(a, b);

    let mut acc: i32 = INIT_sub;
    acc -= 0;
    acc -= 1;
    acc -= 2;
    acc -= 3;
    acc -= 4;
    acc -= 5;

    let x1: i32 = helper_call(a, b);
    let x2: i32 = helper_ptr(a, b);
    let x3: i32 = use_generated(REPEAT);
    let g: i32 = (G_OP)(a, b);

    println!("op={} call={} acc={} g.call={}", G_OP_NAME, r_call, acc, g);
    println!("summary={}", r_call + acc + x1 + x2 + x3 + g);
}
```

**Entity:** CLI arguments (args: Vec<String> in main)

**States:** ArgsMissing, ArgsPresent, ParsedI32, DefaultedToZeroOnParseFail

**Transitions:**
- ArgsMissing -> (process termination) via std::process::exit(2)
- ArgsPresent -> ParsedI32 via args[i].parse() success
- ArgsPresent -> DefaultedToZeroOnParseFail via unwrap_or(0)

**Evidence:** main(): `if args.len() < 3 { ... std::process::exit(2); }` enforces arity at runtime; main(): `let a: i32 = args[1].parse().unwrap_or(0);` defaults on parse failure; main(): `let b: i32 = args[2].parse().unwrap_or(0);` defaults on parse failure; main(): usage string `"usage: {} A B"` documents the positional-argument protocol

**Implementation:** Introduce a small parser type: `struct Cli { a: i32, b: i32 }` with `impl TryFrom<&[String]> for Cli` returning `Result<Cli, UsageError>`. This makes "ArgsPresent + Parsed" a constructed state; main can operate only on `Cli` (no indexing, no defaulting).

---

## Protocol Invariants

### 2. Paired global configuration invariant (function pointer must match its name)

**Location**: `/data/test_case/main.rs:1-121`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on an implicit invariant that the global function pointer `G_OP` and its descriptive string `G_OP_NAME` refer to the same operation. This relationship is not represented in the type system: `static G_OP: fn(i32,i32)->i32` and `static G_OP_NAME: &str` can be changed independently, producing misleading output (e.g., name says "sub" while function actually adds).

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

pub const INIT_sub: i32 = 0;
pub const REPEAT: i32 = 6;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_sub(n: i32) -> i32 {
    let mut acc: i32 = INIT_sub;
    match n {
        1 => {
            acc -= 0;
        }
        2 => {
            acc -= 0;
            acc -= 1;
        }
        3 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
        }
        4 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
        }
        5 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
            acc -= 4;
        }
        6 => {
            acc -= 0;
            acc -= 1;
            acc -= 2;
            acc -= 3;
            acc -= 4;
            acc -= 5;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_sub;
static G_OP_NAME: &str = "sub";

fn helper_call(a: i32, b: i32) -> i32 {
    let r: i32 = op_sub(a, b);
    let mut acc: i32 = INIT_sub;
    acc -= 0;
    acc -= 1;
    acc -= 2;
    acc -= 3;
    acc -= 4;
    acc -= 5;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_sub;
    let r: i32 = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r: i32 = accum_sub(n);
    println!("gen.acc={r}");
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} A B", args.get(0).map(String::as_str).unwrap_or(""));
        std::process::exit(2);
    }

    let a: i32 = args[1].parse().unwrap_or(0);
    let b: i32 = args[2].parse().unwrap_or(0);

    let r_call: i32 = op_sub(a, b);

    let mut acc: i32 = INIT_sub;
    acc -= 0;
    acc -= 1;
    acc -= 2;
    acc -= 3;
    acc -= 4;
    acc -= 5;

    let x1: i32 = helper_call(a, b);
    let x2: i32 = helper_ptr(a, b);
    let x3: i32 = use_generated(REPEAT);
    let g: i32 = (G_OP)(a, b);

    println!("op={} call={} acc={} g.call={}", G_OP_NAME, r_call, acc, g);
    println!("summary={}", r_call + acc + x1 + x2 + x3 + g);
}
```

**Entity:** Operation selection globals (G_OP, G_OP_NAME)

**States:** ConsistentPair, InconsistentPair

**Transitions:**
- ConsistentPair -> InconsistentPair via independent modification of `G_OP` or `G_OP_NAME` (no typed coupling)

**Evidence:** `static G_OP: fn(i32, i32) -> i32 = op_sub;` selects the operation implementation; `static G_OP_NAME: &str = "sub";` separately selects the printed name; main(): `let g: i32 = (G_OP)(a, b);` uses the function pointer; main(): `println!("op={} ...", G_OP_NAME, ...)` prints the independent name

**Implementation:** Define a single typed config: `struct Op { name: &'static str, f: fn(i32,i32)->i32 }` and a single `static G_OP: Op = Op { name: "sub", f: op_sub };`. This makes it impossible to update the name without the function (and vice versa) because they are stored together.

---

