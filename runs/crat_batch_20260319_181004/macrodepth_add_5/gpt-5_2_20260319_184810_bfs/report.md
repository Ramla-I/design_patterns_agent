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

### 2. accum_add input range invariant (only 1..=6 is meaningful)

**Location**: `/data/test_case/main.rs:1-123`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: accum_add only performs meaningful accumulation for specific small values (1..=6); all other inputs fall through to the default arm and return INIT_ADD (0). Callers implicitly rely on supplying an in-range value to get a nontrivial result, but this is not expressed in the function signature and is only encoded by the match arms.

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

const INIT_ADD: i32 = 0;
const REPEAT: i32 = 5;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}

#[allow(dead_code)]
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}

#[allow(dead_code)]
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_add(n: i32) -> i32 {
    let mut acc = INIT_ADD;
    match n {
        1 => {
            acc += 0;
        }
        2 => {
            acc += 0;
            acc += 1;
        }
        3 => {
            acc += 0;
            acc += 1;
            acc += 2;
        }
        4 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
        }
        5 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
            acc += 4;
        }
        6 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
            acc += 4;
            acc += 5;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_add;
static G_OP_NAME: &str = "add";

fn helper_call(a: i32, b: i32) -> i32 {
    let r = op_add(a, b);
    let mut acc = INIT_ADD;
    acc += 0;
    acc += 1;
    acc += 2;
    acc += 3;
    acc += 4;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_add;
    let r = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r = accum_add(n);
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

    let r_call = op_add(a, b);

    let mut acc = INIT_ADD;
    acc += 0;
    acc += 1;
    acc += 2;
    acc += 3;
    acc += 4;

    let x1 = helper_call(a, b);
    let x2 = helper_ptr(a, b);
    let x3 = use_generated(REPEAT);
    let g = (G_OP)(a, b);

    println!("op={} call={} acc={} g.call={}", G_OP_NAME, r_call, acc, g);
    println!("summary={}", r_call + acc + x1 + x2 + x3 + g);
}
```

**Entity:** accum_add(n: i32) input domain

**States:** InRange(1..=6), OutOfRange

**Transitions:**
- OutOfRange -> InRange via caller providing a constrained value (e.g., REPEAT)

**Evidence:** accum_add(n): `match n { 1 => ..., 2 => ..., ... 6 => ..., 0 | _ => {} }` shows only 1..=6 has defined behavior; everything else is treated as default; use_generated(REPEAT): `let r = accum_add(n);` relies on REPEAT being a supported value (REPEAT is 5)

**Implementation:** Define `struct RepeatCount(u8);` with `TryFrom<i32>` (or `NonZeroU8` + range check) to guarantee 1..=6, and change `fn accum_add(n: RepeatCount) -> i32`. Alternatively, use an enum `enum N { One, Two, Three, Four, Five, Six }` and match on that.

---

## Protocol Invariants

### 1. Command-line argument arity + parse validity protocol

**Location**: `/data/test_case/main.rs:1-123`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The program assumes at least two positional arguments (A and B) exist and are valid i32s. This is currently enforced by a runtime length check (else exit) and by lossy parsing (parse failures silently become 0 via unwrap_or(0)). The type system does not distinguish "not enough args", "args present but not yet validated", and "validated numeric inputs"; nor does it prevent accidentally treating parse failure as a legitimate 0 value.

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

const INIT_ADD: i32 = 0;
const REPEAT: i32 = 5;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}

#[allow(dead_code)]
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}

#[allow(dead_code)]
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_add(n: i32) -> i32 {
    let mut acc = INIT_ADD;
    match n {
        1 => {
            acc += 0;
        }
        2 => {
            acc += 0;
            acc += 1;
        }
        3 => {
            acc += 0;
            acc += 1;
            acc += 2;
        }
        4 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
        }
        5 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
            acc += 4;
        }
        6 => {
            acc += 0;
            acc += 1;
            acc += 2;
            acc += 3;
            acc += 4;
            acc += 5;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_add;
static G_OP_NAME: &str = "add";

fn helper_call(a: i32, b: i32) -> i32 {
    let r = op_add(a, b);
    let mut acc = INIT_ADD;
    acc += 0;
    acc += 1;
    acc += 2;
    acc += 3;
    acc += 4;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_add;
    let r = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r = accum_add(n);
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

    let r_call = op_add(a, b);

    let mut acc = INIT_ADD;
    acc += 0;
    acc += 1;
    acc += 2;
    acc += 3;
    acc += 4;

    let x1 = helper_call(a, b);
    let x2 = helper_ptr(a, b);
    let x3 = use_generated(REPEAT);
    let g = (G_OP)(a, b);

    println!("op={} call={} acc={} g.call={}", G_OP_NAME, r_call, acc, g);
    println!("summary={}", r_call + acc + x1 + x2 + x3 + g);
}
```

**Entity:** CLI arguments (args: Vec<String>)

**States:** InsufficientArgs, SufficientArgsButUnvalidated, ValidatedI32Inputs

**Transitions:**
- InsufficientArgs -> (process exit) via main() args.len() < 3 branch
- SufficientArgsButUnvalidated -> ValidatedI32Inputs via parse() of args[1], args[2]

**Evidence:** main(): `if args.len() < 3 { ... std::process::exit(2); }` enforces minimum arity at runtime; main(): `let a: i32 = args[1].parse().unwrap_or(0);` and same for b encodes 'invalid number => 0' fallback; main(): direct indexing `args[1]` and `args[2]` relies on the prior length check

**Implementation:** Introduce a parsed-args type, e.g. `struct Inputs { a: i32, b: i32 }` (or `struct A(i32); struct B(i32);`) with `impl TryFrom<&[String]> for Inputs` returning a Result on both arity and parse errors. This makes "validated inputs" an explicit type and prevents silently mapping parse errors to 0.

---

