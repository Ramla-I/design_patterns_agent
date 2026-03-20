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

### 2. Validated-integer parsing protocol (Must validate before use)

**Location**: `/data/test_case/main.rs:1-155`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The code uses a two-step protocol for parsing: first `parse_val(&line)` checks whether the string looks like a valid in-range i32 per a C `strtol`-like rule; only then `parse_i32_value(&line)` is expected to succeed and produce the numeric value. This ordering is an implicit invariant: calling `parse_i32_value` without having established the same predicate can yield `None` and requires error handling. The type system does not encode that a string has been validated, so the protocol is enforced by control flow and duplication of parsing logic.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 2 free function(s)

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
use std::io::{self, Read};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

pub const __INT_MAX__: i32 = 2147483647;
pub const INT_MAX: i32 = __INT_MAX__;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;

thread_local! {
    static the_house: RefCell<house_t> = const {
        RefCell::new(house_t { floors: 2, bedrooms: 5, bathrooms: 2.5f64 })
    };
}

fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

fn add_floor_to_the_house() {
    the_house.with(|h| add_floor(&mut *h.borrow_mut()));
}

fn print_the_house() {
    the_house.with(|h| {
        let h = h.borrow();
        println!(
            "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
            h.floors, h.bedrooms, h.bathrooms
        );
    });
}

pub(crate) fn run(extra_bedrooms: i32) {
    print_the_house();
    add_floor_to_the_house();
    print_the_house();
    the_house.with(|h| h.borrow_mut().bathrooms += 1.0f64);
    print_the_house();
    the_house.with(|h| add_bedrooms(&mut *h.borrow_mut(), extra_bedrooms));
    print_the_house();
}

fn parse_val(s: &str) -> bool {
    // Match C strtol behavior loosely: accept leading whitespace, optional sign, digits;
    // reject if no digits; reject overflow beyond i32.
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 0usize;

    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }

    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return false;
    }

    let num_str = &trimmed[..i];
    match num_str.parse::<i64>() {
        Ok(v) if v >= INT_MIN as i64 && v <= INT_MAX as i64 => true,
        _ => false,
    }
}

fn parse_i32_value(s: &str) -> Option<i32> {
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 0usize;

    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }

    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return None;
    }

    let num_str = &trimmed[..i];
    let v = num_str.parse::<i64>().ok()?;
    if v < INT_MIN as i64 || v > INT_MAX as i64 {
        return None;
    }
    Some(v as i32)
}

fn main() {
    // C version reads up to 99 chars via fgets; emulate by reading a line and truncating.
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    let mut line = input.lines().next().unwrap_or("").to_string();
    if line.len() > 99 {
        line.truncate(99);
    }

    let mut x: i32 = 0;
    if parse_val(&line) {
        if let Some(v) = parse_i32_value(&line) {
            x = v;
            run(x);
            run(x);
        } else {
            println!("An error occurred");
        }
    } else {
        println!("An error occurred");
    }

    // Return code 0 (implicit).
    let _ = x; // keep variable used similarly to original flow
    let _ = true_0;
    let _ = false_0;
}
```

**Entity:** Input parsing in main (parse_val + parse_i32_value)

**States:** UnvalidatedInput, ValidatedAsI32, Rejected

**Transitions:**
- UnvalidatedInput -> ValidatedAsI32 via `if parse_val(&line) { if let Some(v) = parse_i32_value(&line) { ... } }` in main()
- UnvalidatedInput -> Rejected via `if !parse_val(&line) { println!("An error occurred") }` in main()
- Validated predicate -> Rejected via `else { println!("An error occurred") }` if `parse_i32_value` returns None despite prior check

**Evidence:** main(): `if parse_val(&line) { if let Some(v) = parse_i32_value(&line) { ... } else { println!("An error occurred"); } } else { println!("An error occurred"); }` shows required ordering and fallback error path; parse_val(): comment `Match C strtol behavior loosely... reject overflow beyond i32.` describes a validation precondition separate from extracting the value; parse_val(): returns bool (no value), forcing a second parse step; parse_i32_value(): returns `Option<i32>` and repeats the same trimming/sign/digits/range checks, indicating the protocol is not encoded in types

**Implementation:** Replace the two-step API with a single checked parse that returns a typed result, e.g. `struct ValidI32(i32); impl TryFrom<&str> for ValidI32 { type Error = ParseIntErrorLike; ... }`. Then `main` becomes `let x: ValidI32 = line.as_str().try_into()?; run(x.get());` (or make `run` accept `ValidI32`). This encodes 'validated' at the type level and removes the temporal dependency and duplicate parsing.

---

## Protocol Invariants

### 1. Thread-local house mutation protocol (borrow discipline + domain validity)

**Location**: `/data/test_case/main.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The global `the_house` is a thread-local mutable singleton accessed through `RefCell`. Correct usage relies on the dynamic borrow protocol: you must not hold an immutable borrow while taking a mutable borrow (or take two mutable borrows). The code follows a temporal pattern of taking short-lived borrows inside `with` closures. Additionally, `house_t` fields (floors/bedrooms/bathrooms) are mutated without any type-level guarantee that values remain within a valid domain (e.g., non-negative floors/bedrooms, bathrooms in sensible increments). Both the borrow protocol and the domain validity are enforced only at runtime (RefCell panics for borrow violations; no checks for domain constraints).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 2 free function(s)

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
use std::io::{self, Read};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

pub const __INT_MAX__: i32 = 2147483647;
pub const INT_MAX: i32 = __INT_MAX__;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;

thread_local! {
    static the_house: RefCell<house_t> = const {
        RefCell::new(house_t { floors: 2, bedrooms: 5, bathrooms: 2.5f64 })
    };
}

fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

fn add_floor_to_the_house() {
    the_house.with(|h| add_floor(&mut *h.borrow_mut()));
}

fn print_the_house() {
    the_house.with(|h| {
        let h = h.borrow();
        println!(
            "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
            h.floors, h.bedrooms, h.bathrooms
        );
    });
}

pub(crate) fn run(extra_bedrooms: i32) {
    print_the_house();
    add_floor_to_the_house();
    print_the_house();
    the_house.with(|h| h.borrow_mut().bathrooms += 1.0f64);
    print_the_house();
    the_house.with(|h| add_bedrooms(&mut *h.borrow_mut(), extra_bedrooms));
    print_the_house();
}

fn parse_val(s: &str) -> bool {
    // Match C strtol behavior loosely: accept leading whitespace, optional sign, digits;
    // reject if no digits; reject overflow beyond i32.
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 0usize;

    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }

    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return false;
    }

    let num_str = &trimmed[..i];
    match num_str.parse::<i64>() {
        Ok(v) if v >= INT_MIN as i64 && v <= INT_MAX as i64 => true,
        _ => false,
    }
}

fn parse_i32_value(s: &str) -> Option<i32> {
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 0usize;

    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }

    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return None;
    }

    let num_str = &trimmed[..i];
    let v = num_str.parse::<i64>().ok()?;
    if v < INT_MIN as i64 || v > INT_MAX as i64 {
        return None;
    }
    Some(v as i32)
}

fn main() {
    // C version reads up to 99 chars via fgets; emulate by reading a line and truncating.
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    let mut line = input.lines().next().unwrap_or("").to_string();
    if line.len() > 99 {
        line.truncate(99);
    }

    let mut x: i32 = 0;
    if parse_val(&line) {
        if let Some(v) = parse_i32_value(&line) {
            x = v;
            run(x);
            run(x);
        } else {
            println!("An error occurred");
        }
    } else {
        println!("An error occurred");
    }

    // Return code 0 (implicit).
    let _ = x; // keep variable used similarly to original flow
    let _ = true_0;
    let _ = false_0;
}
```

**Entity:** house_t / the_house (thread_local RefCell<house_t>)

**States:** NotBorrowed, ImmutablyBorrowed, MutablyBorrowed

**Transitions:**
- NotBorrowed -> ImmutablyBorrowed via the_house.with(|h| h.borrow()) in print_the_house()
- NotBorrowed -> MutablyBorrowed via the_house.with(|h| h.borrow_mut()) in add_floor_to_the_house(), run()
- MutablyBorrowed -> NotBorrowed when the RefMut temporary is dropped at end of closure/statement
- ImmutablyBorrowed -> NotBorrowed when the Ref temporary is dropped at end of closure scope

**Evidence:** thread_local!: `static the_house: RefCell<house_t>` encodes runtime borrow-checked global mutable state; print_the_house(): `let h = h.borrow();` (immutable borrow) must not overlap any mutable borrow; add_floor_to_the_house(): `h.borrow_mut()` (mutable borrow) relies on no outstanding borrows; run(): `the_house.with(|h| h.borrow_mut().bathrooms += 1.0f64);` and later `the_house.with(|h| add_bedrooms(&mut *h.borrow_mut(), extra_bedrooms));` mutate fields with no validity newtypes/guards

**Implementation:** Hide `the_house` behind an API that passes an explicit capability/handle to mutate, e.g. `struct HouseToken(PhantomData<*mut ()>); fn with_house<R>(f: impl FnOnce(HouseToken, &mut House) -> R) -> R`. This prevents arbitrary borrowing outside the controlled entrypoint. For domain validity, wrap fields in newtypes like `NonNegativeI32` / `NonNegativeF64` (or `u32` where appropriate) and expose only checked mutation methods (`add_floor`, `add_bedrooms_checked`).

---

