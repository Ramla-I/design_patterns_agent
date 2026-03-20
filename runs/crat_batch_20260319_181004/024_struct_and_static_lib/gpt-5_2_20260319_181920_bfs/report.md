# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 2. house_t value-domain invariants (nonnegative counts; integral quantities)

**Location**: `/data/test_case/lib.rs:1-75`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `house_t` fields represent real-world counts/quantities (floors, bedrooms, bathrooms). The code assumes arithmetic updates keep the house in a sensible domain (e.g., floors/bedrooms not negative; bathrooms not negative; bedrooms changed by `extra_bedrooms`). None of these constraints are encoded in the type system: `i32` allows negative floors/bedrooms and `f64` allows NaN/inf/negative bathrooms, and `add_bedrooms` accepts any `i32` including negative, potentially producing an invalid house state.

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

// === driver.rs ===
#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

thread_local! {
    static the_house: std::cell::RefCell<house_t> = const {
        std::cell::RefCell::new(house_t {
            floors: 2,
            bedrooms: 5,
            bathrooms: 2.5f64,
        })
    };
}

#[inline]
fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

#[inline]
fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

#[inline]
fn add_floor_to_the_house() {
    the_house.with_borrow_mut(add_floor);
}

#[inline]
fn print_the_house() {
    the_house.with_borrow(|h| {
        println!(
            "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
            h.floors, h.bedrooms, h.bathrooms
        )
    });
}

pub(crate) fn run(extra_bedrooms: i32) {
    print_the_house();

    add_floor_to_the_house();
    print_the_house();

    the_house.with_borrow_mut(|h| h.bathrooms += 1.0f64);
    print_the_house();

    the_house.with_borrow_mut(|h| add_bedrooms(h, extra_bedrooms));
    print_the_house();
}

#[no_mangle]
pub extern "C" fn driver(x: i32) {
    run(x);
    run(x);
}
```

**Entity:** house_t

**States:** ValidHouse, InvalidHouse

**Transitions:**
- ValidHouse -> InvalidHouse via add_bedrooms(h, extra_bedrooms) when extra_bedrooms is negative enough
- ValidHouse -> InvalidHouse via direct mutation `h.bathrooms += 1.0f64` if bathrooms was NaN/inf or becomes otherwise invalid
- ValidHouse -> InvalidHouse via any future direct assignment to floors/bedrooms/bathrooms (all fields are pub)

**Evidence:** pub struct house_t { pub floors: i32, pub bedrooms: i32, pub bathrooms: f64 } — raw numeric types with no validity constraints; fn add_floor(house: &mut house_t) { house.floors += 1; } — assumes floors is a count; no lower-bound/overflow checks; fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) { house.bedrooms += extra_bedrooms; } — allows negative deltas; no validation; run(extra_bedrooms: i32) passes an unconstrained i32 into add_bedrooms via `the_house.with_borrow_mut(|h| add_bedrooms(h, extra_bedrooms));`

**Implementation:** Replace raw fields with validated newtypes: `struct Floors(NonZeroU32 or u32)`, `struct Bedrooms(u32)`, `struct Bathrooms(NonNaN<f64> + constrained >= 0.0)`. Make fields private and provide constructors and methods like `fn add_bedrooms(&mut self, extra: BedroomsDelta)` where `BedroomsDelta` encodes allowed changes (e.g., only nonnegative). This makes invalid states unrepresentable and forces validation at boundaries (e.g., in `driver(x)` / `run(x)`).

---

## Protocol Invariants

### 1. Thread-local mutable singleton protocol (borrowed immutably vs mutably)

**Location**: `/data/test_case/lib.rs:1-75`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `the_house` is a per-thread global mutable `house_t` stored behind `RefCell`. Correctness relies on a dynamic borrowing protocol: at any moment it must be either unborrowed, immutably borrowed (any number of readers), or mutably borrowed (exactly one writer, no readers). This is enforced by `RefCell` at runtime (panic on violation), not at compile time, because the state is hidden behind a thread-local and closures (`with_borrow`/`with_borrow_mut`).

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

// === driver.rs ===
#[repr(C)]
#[derive(Copy, Clone)]
pub struct house_t {
    pub floors: i32,
    pub bedrooms: i32,
    pub bathrooms: f64,
}

thread_local! {
    static the_house: std::cell::RefCell<house_t> = const {
        std::cell::RefCell::new(house_t {
            floors: 2,
            bedrooms: 5,
            bathrooms: 2.5f64,
        })
    };
}

#[inline]
fn add_floor(house: &mut house_t) {
    house.floors += 1;
}

#[inline]
fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
    house.bedrooms += extra_bedrooms;
}

#[inline]
fn add_floor_to_the_house() {
    the_house.with_borrow_mut(add_floor);
}

#[inline]
fn print_the_house() {
    the_house.with_borrow(|h| {
        println!(
            "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
            h.floors, h.bedrooms, h.bathrooms
        )
    });
}

pub(crate) fn run(extra_bedrooms: i32) {
    print_the_house();

    add_floor_to_the_house();
    print_the_house();

    the_house.with_borrow_mut(|h| h.bathrooms += 1.0f64);
    print_the_house();

    the_house.with_borrow_mut(|h| add_bedrooms(h, extra_bedrooms));
    print_the_house();
}

#[no_mangle]
pub extern "C" fn driver(x: i32) {
    run(x);
    run(x);
}
```

**Entity:** the_house (thread_local RefCell<house_t>)

**States:** Unborrowed, ImmutablyBorrowed, MutablyBorrowed

**Transitions:**
- Unborrowed -> ImmutablyBorrowed via the_house.with_borrow(...)
- Unborrowed -> MutablyBorrowed via the_house.with_borrow_mut(...)
- ImmutablyBorrowed -> Unborrowed when with_borrow closure returns
- MutablyBorrowed -> Unborrowed when with_borrow_mut closure returns

**Evidence:** thread_local! { static the_house: std::cell::RefCell<house_t> = ... } — interior mutability with runtime borrow checking; fn add_floor_to_the_house() { the_house.with_borrow_mut(add_floor); } — acquires a mutable borrow to mutate floors; fn print_the_house() { the_house.with_borrow(|h| { println!(...) }) } — acquires an immutable borrow to read fields; run(): multiple uses of the_house.with_borrow_mut(|h| ...) to mutate bathrooms and bedrooms

**Implementation:** Hide `the_house` behind an API that vends explicit capabilities/tokens: e.g., `fn with_house<R>(f: impl FnOnce(HouseRead<'_>) -> R)` and `fn with_house_mut<R>(f: impl FnOnce(HouseWrite<'_>) -> R)` where `HouseRead/HouseWrite` are newtypes around `Ref<'_, house_t>` / `RefMut<'_, house_t>`. This makes the read/write capability explicit and prevents accidentally exposing raw `RefCell`-like borrow operations elsewhere. (It still uses runtime borrow checks, but moves the protocol into typed capabilities and limits misuse.)

---

