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

### 2. Command-line arity protocol (InvalidArity / ValidArity[2..=4])

**Location**: `/data/test_case/main.rs:1-78`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: The rest of `main` assumes `args[1]` exists and that optional indices are at `args[2]`/`args[3]` depending on `argc`. This is enforced by an early arity check and `exit(1)`, but the type system does not encode that the program is in a 'valid arity' state when later indexing occurs. The implicit protocol is: check arity first; only then index into `args` at fixed positions.

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
    let args: Vec<String> = std::env::args().collect();
    let argc = args.len();

    if argc > 4 || argc == 1 {
        println!("Error: there should be one to three arguments passed:");
        println!("<string> [start] [stop]");
        std::process::exit(1);
    }

    let s = &args[1];
    let bytes = s.as_bytes();
    let len = bytes.len();

    let start: usize = if argc >= 3 {
        match args[2].parse::<isize>() {
            Ok(v) => {
                if v < 0 {
                    // C would treat negative as a huge unsigned when compared to len.
                    println!("Error: start is off the end of the string!");
                    std::process::exit(1);
                }
                let v = v as usize;
                if v > len {
                    println!("Error: start is off the end of the string!");
                    std::process::exit(1);
                }
                v
            }
            Err(_) => {
                print!("Second argument must be an integer!");
                std::process::exit(1);
            }
        }
    } else {
        0
    };

    let stop: usize = if argc == 4 {
        // Match the original (buggy) C2Rust behavior: it never properly validates
        // that the third argument is numeric, and instead proceeds with whatever
        // value results (effectively 0 on non-numeric), then checks ordering.
        let parsed = args[3].parse::<isize>().unwrap_or(0);

        if parsed < 0 {
            println!("Error: stop is off the end of the string!");
            std::process::exit(1);
        }
        let v = parsed as usize;

        if v > len {
            println!("Error: stop is off the end of the string!");
            std::process::exit(1);
        }
        if v <= start {
            println!("Error: stop must come after start!");
            std::process::exit(1);
        }
        v
    } else {
        len
    };

    // Byte-based slicing to match C pointer offset + precision behavior.
    let slice = &bytes[start..stop];
    let out = String::from_utf8_lossy(slice);
    println!("{:.width$}", out, width = stop - start);
}
```

**Entity:** CLI argument vector `args` / `argc`

**States:** InvalidArity, ValidArity

**Transitions:**
- InvalidArity -> ValidArity via `if argc > 4 || argc == 1 { ... exit(1) }` (else continue)

**Evidence:** Arity gate: `if argc > 4 || argc == 1 { ... std::process::exit(1); }`; Assumption after the gate: `let s = &args[1];` would panic on missing arg without the prior check; Optional indexing depends on arity: `if argc >= 3 { ... args[2] ... }` and `if argc == 4 { ... args[3] ... }`

**Implementation:** Parse into a typed config first: `enum Cli { One { s: String }, Two { s: String, start: isize }, Three { s: String, start: isize, stop: isize } }` (or `struct ParsedArgs { s: String, start: Option<isize>, stop: Option<isize> }`) returned by `fn parse_args() -> Result<..., Error>`. This makes later code consume a variant/struct that guarantees required positions exist, eliminating raw `args[i]` indexing outside the parser.

---

## Protocol Invariants

### 1. Validated slice bounds protocol (Unchecked -> StartValidated -> RangeValidated)

**Location**: `/data/test_case/main.rs:1-78`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program relies on a multi-step validation protocol before using `start`/`stop` for `&bytes[start..stop]`. Inputs begin as unchecked CLI strings, then `start` is validated to be numeric, non-negative, and within `len`. After `start` is established, `stop` is validated (or defaulted) to be non-negative, within `len`, and strictly greater than `start`. Only in the fully validated state is slicing performed safely. These invariants are currently enforced by runtime checks + `exit(1)`, and the type system does not distinguish unchecked from validated indices or a proven-valid range.

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
    let args: Vec<String> = std::env::args().collect();
    let argc = args.len();

    if argc > 4 || argc == 1 {
        println!("Error: there should be one to three arguments passed:");
        println!("<string> [start] [stop]");
        std::process::exit(1);
    }

    let s = &args[1];
    let bytes = s.as_bytes();
    let len = bytes.len();

    let start: usize = if argc >= 3 {
        match args[2].parse::<isize>() {
            Ok(v) => {
                if v < 0 {
                    // C would treat negative as a huge unsigned when compared to len.
                    println!("Error: start is off the end of the string!");
                    std::process::exit(1);
                }
                let v = v as usize;
                if v > len {
                    println!("Error: start is off the end of the string!");
                    std::process::exit(1);
                }
                v
            }
            Err(_) => {
                print!("Second argument must be an integer!");
                std::process::exit(1);
            }
        }
    } else {
        0
    };

    let stop: usize = if argc == 4 {
        // Match the original (buggy) C2Rust behavior: it never properly validates
        // that the third argument is numeric, and instead proceeds with whatever
        // value results (effectively 0 on non-numeric), then checks ordering.
        let parsed = args[3].parse::<isize>().unwrap_or(0);

        if parsed < 0 {
            println!("Error: stop is off the end of the string!");
            std::process::exit(1);
        }
        let v = parsed as usize;

        if v > len {
            println!("Error: stop is off the end of the string!");
            std::process::exit(1);
        }
        if v <= start {
            println!("Error: stop must come after start!");
            std::process::exit(1);
        }
        v
    } else {
        len
    };

    // Byte-based slicing to match C pointer offset + precision behavior.
    let slice = &bytes[start..stop];
    let out = String::from_utf8_lossy(slice);
    println!("{:.width$}", out, width = stop - start);
}
```

**Entity:** start/stop indices (usize) used for slicing bytes

**States:** UncheckedInputs, StartValidated, RangeValidated

**Transitions:**
- UncheckedInputs -> StartValidated via parsing/validating args[2] (or default start=0 when argc<3)
- StartValidated -> RangeValidated via parsing/validating args[3] (or default stop=len when argc!=4)

**Evidence:** `let start: usize = if argc >= 3 { match args[2].parse::<isize>() { ... } } else { 0 }` encodes a state transition from unchecked to validated start; Error messages and exits: `println!("Error: start is off the end of the string!"); std::process::exit(1);` and `print!("Second argument must be an integer!"); std::process::exit(1);` are runtime enforcement of start preconditions; `let stop: usize = if argc == 4 { ... } else { len }` encodes a second transition to a validated stop (or default); Stop validation checks: `if v > len { println!("Error: stop is off the end of the string!"); ... }` and ordering check `if v <= start { println!("Error: stop must come after start!"); ... }`; The safety-critical use site: `let slice = &bytes[start..stop];` requires the invariant `start <= stop <= len` but this is not represented in types

**Implementation:** Introduce validated index/range types, e.g. `struct Start(usize); struct Stop(usize); struct ValidRange { start: usize, stop: usize }` with constructors `Start::parse(arg, len) -> Result<Start, Error>` and `ValidRange::new(Start, Stop, len) -> Result<ValidRange, Error>`. Only accept `ValidRange` at the slicing site (`fn slice_bytes(bytes: &[u8], r: ValidRange) -> &[u8]`). This makes it impossible to call slicing without having passed validation.

---

