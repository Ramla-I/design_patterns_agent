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

### 2. powf domain/range checking pipeline (Parsed -> Computed -> DomainChecked -> RangeChecked)

**Location**: `/data/test_case/main.rs:1-109`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The program follows an implicit ordering: parse base/exponent strictly, compute `base.powf(exponent)`, then check `is_nan()` (domain error) and `is_infinite()` (range error) before printing. This pipeline is enforced by runtime checks and early exits; the type system does not distinguish 'unchecked result' from 'domain+range checked result', so other code could accidentally print/consume NaN/∞ without checks if this logic were refactored/reused.

**Evidence**:

```rust
// Note: Other parts of this module contain: enum ParseKind, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::path::Path;

enum ParseKind {
    Invalid,
    Range,
}

fn program_name_for_usage(arg0: &str) -> String {
    // The original C version prints argv[0] as provided by the harness.
    // In this test environment, argv[0] is expected to end with "/driver".
    // Use the basename and prefix with "/driver" to match the expected regex.
    let base = Path::new(arg0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(arg0);

    if base == "driver" {
        "/driver".to_string()
    } else {
        arg0.to_string()
    }
}

fn parse_f64_strict(s: &str) -> Result<f64, ParseKind> {
    // strtod-like: allow surrounding whitespace, but reject any trailing junk.
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ParseKind::Invalid);
    }

    match trimmed.parse::<f64>() {
        Ok(v) => {
            // Approximate strtod ERANGE: treat infinities as range errors.
            if v.is_infinite() {
                Err(ParseKind::Range)
            } else {
                Ok(v)
            }
        }
        Err(_) => Err(ParseKind::Invalid),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        let prog = program_name_for_usage(args.get(0).map(String::as_str).unwrap_or(""));
        eprintln!("Usage: {} base exponent", prog);
        std::process::exit(1);
    }

    let base_str = &args[1];
    let exponent_str = &args[2];

    let base = match parse_f64_strict(base_str) {
        Ok(v) => v,
        Err(ParseKind::Range) => {
            eprintln!("Range error while converting base '{}'", base_str);
            std::process::exit(1);
        }
        Err(ParseKind::Invalid) => {
            eprintln!("Invalid numeric input for base: '{}'", base_str);
            std::process::exit(1);
        }
    };

    let exponent = match parse_f64_strict(exponent_str) {
        Ok(v) => v,
        Err(ParseKind::Range) => {
            eprintln!("Range error while converting exponent '{}'", exponent_str);
            std::process::exit(1);
        }
        Err(ParseKind::Invalid) => {
            eprintln!("Invalid numeric input for exponent: '{}'", exponent_str);
            std::process::exit(1);
        }
    };

    let result = base.powf(exponent);

    if result.is_nan() {
        eprintln!(
            "Domain error: pow({0:.2}, {1:.2}) is undefined in the real number domain.",
            base, exponent
        );
        std::process::exit(1);
    } else if result.is_infinite() {
        eprintln!(
            "Range error: pow({0:.2}, {1:.2}) caused overflow or underflow.",
            base, exponent
        );
        std::process::exit(1);
    }

    println!("Result: {0:.2}", result);
}
```

**Entity:** main (pow computation pipeline)

**States:** ParsedInputs, ComputedResult, DomainError, RangeError, PrintableResult

**Transitions:**
- ParsedInputs -> ComputedResult via `let result = base.powf(exponent);`
- ComputedResult -> DomainError via `if result.is_nan()`
- ComputedResult -> RangeError via `else if result.is_infinite()`
- ComputedResult -> PrintableResult via falling through checks (not NaN and not infinite)

**Evidence:** `let result = base.powf(exponent);`: creates an unchecked floating-point result; `if result.is_nan()` => prints "Domain error: pow(...) is undefined..." and exits: NaN is an invalid/terminal state; `else if result.is_infinite()` => prints "Range error: pow(...) caused overflow or underflow." and exits: infinity is an invalid/terminal state; `println!("Result: {0:.2}", result);` occurs only after both checks

**Implementation:** Wrap the result in a checked type, e.g. `struct RealPowResult(f64);` with `fn new(base: FiniteF64, exp: FiniteF64) -> Result<RealPowResult, PowError>` that performs `powf` and rejects NaN/∞. Expose `Display`/`as_f64()` only on `RealPowResult` so printing/consumption requires the checks to have occurred.

---

## Protocol Invariants

### 1. Strict finite-f64 parsing contract (NonEmptyNumeric -> FiniteF64 | Invalid | Range)

**Location**: `/data/test_case/main.rs:1-109`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code relies on a two-phase protocol: (1) validate a user-provided string with parse_f64_strict(), which trims whitespace, rejects empty input, rejects parse failures, and classifies infinities as Range errors; (2) only after this validation, treat the returned f64 as a 'finite numeric argument' suitable for powf. This validity is not represented in the type system; callers must remember to route errors and only proceed with Ok(v).

**Evidence**:

```rust
// Note: Other parts of this module contain: enum ParseKind, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::path::Path;

enum ParseKind {
    Invalid,
    Range,
}

fn program_name_for_usage(arg0: &str) -> String {
    // The original C version prints argv[0] as provided by the harness.
    // In this test environment, argv[0] is expected to end with "/driver".
    // Use the basename and prefix with "/driver" to match the expected regex.
    let base = Path::new(arg0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(arg0);

    if base == "driver" {
        "/driver".to_string()
    } else {
        arg0.to_string()
    }
}

fn parse_f64_strict(s: &str) -> Result<f64, ParseKind> {
    // strtod-like: allow surrounding whitespace, but reject any trailing junk.
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ParseKind::Invalid);
    }

    match trimmed.parse::<f64>() {
        Ok(v) => {
            // Approximate strtod ERANGE: treat infinities as range errors.
            if v.is_infinite() {
                Err(ParseKind::Range)
            } else {
                Ok(v)
            }
        }
        Err(_) => Err(ParseKind::Invalid),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        let prog = program_name_for_usage(args.get(0).map(String::as_str).unwrap_or(""));
        eprintln!("Usage: {} base exponent", prog);
        std::process::exit(1);
    }

    let base_str = &args[1];
    let exponent_str = &args[2];

    let base = match parse_f64_strict(base_str) {
        Ok(v) => v,
        Err(ParseKind::Range) => {
            eprintln!("Range error while converting base '{}'", base_str);
            std::process::exit(1);
        }
        Err(ParseKind::Invalid) => {
            eprintln!("Invalid numeric input for base: '{}'", base_str);
            std::process::exit(1);
        }
    };

    let exponent = match parse_f64_strict(exponent_str) {
        Ok(v) => v,
        Err(ParseKind::Range) => {
            eprintln!("Range error while converting exponent '{}'", exponent_str);
            std::process::exit(1);
        }
        Err(ParseKind::Invalid) => {
            eprintln!("Invalid numeric input for exponent: '{}'", exponent_str);
            std::process::exit(1);
        }
    };

    let result = base.powf(exponent);

    if result.is_nan() {
        eprintln!(
            "Domain error: pow({0:.2}, {1:.2}) is undefined in the real number domain.",
            base, exponent
        );
        std::process::exit(1);
    } else if result.is_infinite() {
        eprintln!(
            "Range error: pow({0:.2}, {1:.2}) caused overflow or underflow.",
            base, exponent
        );
        std::process::exit(1);
    }

    println!("Result: {0:.2}", result);
}
```

**Entity:** parse_f64_strict (returned f64 value)

**States:** UnvalidatedInput, FiniteF64, Invalid, Range

**Transitions:**
- UnvalidatedInput -> FiniteF64 via parse_f64_strict(s) returning Ok(v) where !v.is_infinite()
- UnvalidatedInput -> Invalid via parse_f64_strict(s) returning Err(ParseKind::Invalid)
- UnvalidatedInput -> Range via parse_f64_strict(s) returning Err(ParseKind::Range)

**Evidence:** fn parse_f64_strict(s: &str) -> Result<f64, ParseKind>: returns raw f64 with a validity protocol encoded in Result; trimmed.is_empty() => Err(ParseKind::Invalid): empty/whitespace-only is invalid; trimmed.parse::<f64>() Err(_) => Err(ParseKind::Invalid): parse failures are invalid; if v.is_infinite() { Err(ParseKind::Range) } else { Ok(v) }: 'finite' is an additional required property for success; main(): base/exponent are only used after matching on parse_f64_strict and exiting on Err

**Implementation:** Introduce `struct FiniteF64(f64);` with `impl TryFrom<&str> for FiniteF64` (or `FromStr`) performing the trim/empty/parse/infinite checks. Change `parse_f64_strict` to return `Result<FiniteF64, ParseKind>` so downstream code cannot accidentally call `powf` with an unvalidated/possibly-infinite value.

---

