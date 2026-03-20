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

### 1. CLI argument presence & parse validity precondition (Missing/Invalid -> Valid)

**Location**: `/data/test_case/main.rs:1-123`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program assumes two positional CLI arguments exist and are valid i32 numbers. This is enforced at runtime by checking args.len() and by parsing with unwrap_or(0), which silently maps parse failures into the value 0. The type system does not distinguish (a) missing args, (b) present-but-non-numeric args, and (c) successfully parsed integers, so downstream computations run even when inputs are invalid.

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

pub const INIT_mul: i32 = 1;
pub const REPEAT: i32 = 4;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_mul(n: i32) -> i32 {
    let mut acc = INIT_mul;
    match n {
        1 => {
            acc *= 1;
        }
        2 => {
            acc *= 1;
            acc *= 1 + 1;
        }
        3 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
        }
        4 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
        }
        5 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
            acc *= 4 + 1;
        }
        6 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
            acc *= 4 + 1;
            acc *= 5 + 1;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_mul;
static G_OP_NAME: &str = "mul";

fn helper_call(a: i32, b: i32) -> i32 {
    let r = op_mul(a, b);
    let mut acc = INIT_mul;
    acc *= 1;
    acc *= 1 + 1;
    acc *= 2 + 1;
    acc *= 3 + 1;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_mul;
    let r = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r = accum_mul(n);
    println!("gen.acc={r}");
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} A B", args.get(0).map(|s| s.as_str()).unwrap_or(""));
        std::process::exit(2);
    }

    let a: i32 = args[1].parse().unwrap_or(0);
    let b: i32 = args[2].parse().unwrap_or(0);

    let r_call: i32 = op_mul(a, b);

    let mut acc: i32 = INIT_mul;
    acc *= 1;
    acc *= 1 + 1;
    acc *= 2 + 1;
    acc *= 3 + 1;

    let x1: i32 = helper_call(a, b);
    let x2: i32 = helper_ptr(a, b);
    let x3: i32 = use_generated(REPEAT);

    let g: i32 = (G_OP)(a, b);

    println!(
        "op={0} call={1} acc={2} g.call={3}",
        G_OP_NAME, r_call, acc, g
    );
    println!("summary={0}", (r_call + acc + x1 + x2 + x3 + g));

    std::process::exit(0);
}
```

**Entity:** CLI arguments (A, B) in main()

**States:** MissingArgs, PresentButInvalid, Valid

**Transitions:**
- MissingArgs -> (process exit) via args.len() < 3 check
- PresentButInvalid -> Valid (but semantically wrong) via parse().unwrap_or(0)
- Valid -> Valid via successful parse()

**Evidence:** main(): `if args.len() < 3 { eprintln!("usage: {} A B", ...); std::process::exit(2); }` enforces presence at runtime; main(): `let a: i32 = args[1].parse().unwrap_or(0);` and `let b: i32 = args[2].parse().unwrap_or(0);` encode an implicit 'parsed successfully' precondition but fall back to 0 on failure

**Implementation:** Introduce a `struct ParsedI32(i32);` with `TryFrom<&str>` (or a `struct Args { a: i32, b: i32 }` with `TryFrom<std::env::Args>`). Return `Result<Args, Error>` from a parsing function so main only proceeds with a statically-known 'Valid' value, avoiding the `unwrap_or(0)` invalid-state conflation.

---

## Protocol Invariants

### 2. Operation function/name coherence invariant

**Location**: `/data/test_case/main.rs:1-123`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code implicitly relies on `G_OP` and `G_OP_NAME` representing the same logical operation (e.g., op_mul and "mul"). This relationship is maintained only by convention: they are two separate `static` items with no type-level coupling, so they can drift (e.g., `G_OP = op_add` while `G_OP_NAME = "mul"`) without compiler errors.

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

pub const INIT_mul: i32 = 1;
pub const REPEAT: i32 = 4;

fn op_add(a: i32, b: i32) -> i32 {
    a + b
}
fn op_sub(a: i32, b: i32) -> i32 {
    a - b
}
fn op_mul(a: i32, b: i32) -> i32 {
    a * b
}

fn accum_mul(n: i32) -> i32 {
    let mut acc = INIT_mul;
    match n {
        1 => {
            acc *= 1;
        }
        2 => {
            acc *= 1;
            acc *= 1 + 1;
        }
        3 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
        }
        4 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
        }
        5 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
            acc *= 4 + 1;
        }
        6 => {
            acc *= 1;
            acc *= 1 + 1;
            acc *= 2 + 1;
            acc *= 3 + 1;
            acc *= 4 + 1;
            acc *= 5 + 1;
        }
        0 | _ => {}
    }
    acc
}

static G_OP: fn(i32, i32) -> i32 = op_mul;
static G_OP_NAME: &str = "mul";

fn helper_call(a: i32, b: i32) -> i32 {
    let r = op_mul(a, b);
    let mut acc = INIT_mul;
    acc *= 1;
    acc *= 1 + 1;
    acc *= 2 + 1;
    acc *= 3 + 1;
    println!("helper.call={r} helper.acc={acc}");
    r + acc
}

fn helper_ptr(a: i32, b: i32) -> i32 {
    let fp: fn(i32, i32) -> i32 = op_mul;
    let r = fp(a, b);
    println!("helper.ptr={r}");
    r
}

fn use_generated(n: i32) -> i32 {
    let r = accum_mul(n);
    println!("gen.acc={r}");
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} A B", args.get(0).map(|s| s.as_str()).unwrap_or(""));
        std::process::exit(2);
    }

    let a: i32 = args[1].parse().unwrap_or(0);
    let b: i32 = args[2].parse().unwrap_or(0);

    let r_call: i32 = op_mul(a, b);

    let mut acc: i32 = INIT_mul;
    acc *= 1;
    acc *= 1 + 1;
    acc *= 2 + 1;
    acc *= 3 + 1;

    let x1: i32 = helper_call(a, b);
    let x2: i32 = helper_ptr(a, b);
    let x3: i32 = use_generated(REPEAT);

    let g: i32 = (G_OP)(a, b);

    println!(
        "op={0} call={1} acc={2} g.call={3}",
        G_OP_NAME, r_call, acc, g
    );
    println!("summary={0}", (r_call + acc + x1 + x2 + x3 + g));

    std::process::exit(0);
}
```

**Entity:** Global operation selection (G_OP, G_OP_NAME)

**States:** CoherentPair, IncoherentPair

**Transitions:**
- CoherentPair -> IncoherentPair via independent edits to `static G_OP` or `static G_OP_NAME`

**Evidence:** `static G_OP: fn(i32, i32) -> i32 = op_mul;` defines the callable operation; `static G_OP_NAME: &str = "mul";` separately defines its name; main(): `println!("op={0} ...", G_OP_NAME, ...)` assumes `G_OP_NAME` describes the operation used in `(G_OP)(a, b)`

**Implementation:** Define a single `static OP: Operation = Operation { name: "mul", f: op_mul };` where `struct Operation { name: &'static str, f: fn(i32,i32)->i32 }`. Alternatively use an `enum OpKind { Mul, Add, Sub }` with methods `fn name(&self)->&'static str` and `fn apply(&self,a,b)->i32` to make mismatches unrepresentable.

---

