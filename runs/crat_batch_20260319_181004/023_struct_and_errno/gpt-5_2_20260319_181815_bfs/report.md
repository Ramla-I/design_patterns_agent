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

### 1. house_t numeric validity invariant (non-negative counts; integer bathrooms step)

**Location**: `/data/test_case/main.rs:1-78`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The code implicitly treats `floors` and `bedrooms` as counts that should not become negative, and treats `bathrooms` as a quantity that is incremented in 1.0 steps. None of these constraints are enforced by the type system: `add_bedrooms()` accepts any `i32` (including negative), and `bathrooms` is an unconstrained `f64` that can be set to NaN/Inf or arbitrary fractional values. If `extra_bedrooms` is negative enough, `house.bedrooms` can become negative, producing nonsensical output in `print_house()`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 4 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[derive(Copy, Clone)]
struct house_t {
    floors: i32,
    bedrooms: i32,
    bathrooms: f64,
}

fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

fn print_house(house: &house_t) {
    println!(
        "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
        house.floors, house.bedrooms, house.bathrooms
    );
}

fn run(the_house: &mut house_t, extra_bedrooms: i32) {
    print_house(the_house);
    add_floor(the_house);
    print_house(the_house);
    the_house.bathrooms += 1.0;
    print_house(the_house);
    add_bedrooms(the_house, extra_bedrooms);
    print_house(the_house);
}

fn main() {
    use std::io::{self, Read};

    // Mimic fgets into a 100-byte buffer: read up to a newline (or EOF) and keep at most 99 chars.
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);

    let line = input.lines().next().unwrap_or("");
    let mut trimmed = line.trim_start();

    // In case the first line is empty but there are leading newlines, try to find the first non-empty line.
    if trimmed.is_empty() {
        if let Some(first_nonempty) = input.lines().find(|l| !l.trim_start().is_empty()) {
            trimmed = first_nonempty.trim_start();
        }
    }

    let token = trimmed.split_whitespace().next().unwrap_or("");

    match token.parse::<i32>() {
        Ok(x) => {
            let mut the_house = house_t {
                floors: 2,
                bedrooms: 5,
                bathrooms: 2.5,
            };
            run(&mut the_house, x);
            run(&mut the_house, x);
        }
        Err(_) => {
            println!("An error occurred");
        }
    }
}
```

**Entity:** house_t

**States:** ValidCounts, InvalidCounts

**Transitions:**
- ValidCounts -> InvalidCounts via add_bedrooms(house, extra_bedrooms) when extra_bedrooms is negative enough
- ValidCounts -> InvalidCounts via direct mutation of bathrooms (the_house.bathrooms += 1.0) if bathrooms was already NaN/Inf or otherwise invalid

**Evidence:** struct house_t { floors: i32, bedrooms: i32, bathrooms: f64 } uses unconstrained primitives for count-like fields; fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) { house.bedrooms += extra_bedrooms; } allows negative adjustments without checks; fn run(...) { the_house.bathrooms += 1.0; } mutates a raw f64 directly, with no invariant enforcement; fn print_house(...) prints counts directly, implying they are meaningful as non-negative quantities

**Implementation:** Introduce validated newtypes such as `struct Floors(u32)`, `struct Bedrooms(u32)`, and possibly `struct Bathrooms(NonNaNF64)` (or a fixed-point type like tenths). Provide constructors that validate (or use unsigned types) and only expose operations like `add_floor(&mut self)` / `add_bedrooms(&mut self, extra: u32)` that preserve invariants.

---

## Protocol Invariants

### 2. House modification protocol (ordered sequence of mutations and prints)

**Location**: `/data/test_case/main.rs:1-78`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: `run()` encodes a specific multi-step interaction with a `house_t`: print initial state, then add a floor, print, then add 1.0 bathroom, print, then add bedrooms, print. This ordered protocol is enforced only by convention inside `run()`; callers can bypass it by calling `add_floor()`, directly mutating `bathrooms`, or calling `add_bedrooms()` in arbitrary orders. If the intent is that house updates happen through a controlled sequence (e.g., auditing/logging every change), the type system currently cannot ensure that updates happen only through that protocol or that each step has occurred before the next.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 4 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[derive(Copy, Clone)]
struct house_t {
    floors: i32,
    bedrooms: i32,
    bathrooms: f64,
}

fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

fn print_house(house: &house_t) {
    println!(
        "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
        house.floors, house.bedrooms, house.bathrooms
    );
}

fn run(the_house: &mut house_t, extra_bedrooms: i32) {
    print_house(the_house);
    add_floor(the_house);
    print_house(the_house);
    the_house.bathrooms += 1.0;
    print_house(the_house);
    add_bedrooms(the_house, extra_bedrooms);
    print_house(the_house);
}

fn main() {
    use std::io::{self, Read};

    // Mimic fgets into a 100-byte buffer: read up to a newline (or EOF) and keep at most 99 chars.
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);

    let line = input.lines().next().unwrap_or("");
    let mut trimmed = line.trim_start();

    // In case the first line is empty but there are leading newlines, try to find the first non-empty line.
    if trimmed.is_empty() {
        if let Some(first_nonempty) = input.lines().find(|l| !l.trim_start().is_empty()) {
            trimmed = first_nonempty.trim_start();
        }
    }

    let token = trimmed.split_whitespace().next().unwrap_or("");

    match token.parse::<i32>() {
        Ok(x) => {
            let mut the_house = house_t {
                floors: 2,
                bedrooms: 5,
                bathrooms: 2.5,
            };
            run(&mut the_house, x);
            run(&mut the_house, x);
        }
        Err(_) => {
            println!("An error occurred");
        }
    }
}
```

**Entity:** run(the_house: &mut house_t, extra_bedrooms: i32)

**States:** BeforeRun, After1stFloorAdded, AfterBathroomAdded, AfterBedroomsAdded

**Transitions:**
- BeforeRun -> After1stFloorAdded via add_floor(the_house) inside run()
- After1stFloorAdded -> AfterBathroomAdded via direct `the_house.bathrooms += 1.0` inside run()
- AfterBathroomAdded -> AfterBedroomsAdded via add_bedrooms(the_house, extra_bedrooms) inside run()

**Evidence:** fn run(...) calls print_house(the_house) between each mutation, implying an intended sequence; run(): add_floor(the_house); then later `the_house.bathrooms += 1.0;` then add_bedrooms(the_house, extra_bedrooms); bathrooms is mutated directly in run() rather than via a dedicated function, making the protocol easy to violate elsewhere

**Implementation:** Model the sequence as typestate: `struct House<S> { inner: house_t, _s: PhantomData<S> }` with states like `Start`, `FloorAdded`, `BathroomAdded`, `BedroomsAdded`. Expose transitions `add_floor(self) -> House<FloorAdded>`, `add_bathroom(self) -> House<BathroomAdded>`, `add_bedrooms(self, ...) -> House<BedroomsAdded>`, and only implement `print_house()` for all (or specific) states depending on intent.

---

