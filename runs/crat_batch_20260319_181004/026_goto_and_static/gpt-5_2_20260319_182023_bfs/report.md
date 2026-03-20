# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 1

## Temporal Ordering Invariants

### 1. Thread-local y initialization/update protocol (Default=123 -> Overwritten from stdin)

**Location**: `/data/test_case/main.rs:1-52`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The program relies on a temporal ordering where the thread-local `y` must be set from parsed input (`y_in`) before calling `multi_stage`, otherwise `multi_stage` will observe the default value 123 and fail its `y == 2` check. This ordering is enforced only by call sequence in `main`, not by the type system; `multi_stage` can be called at any time and reads `y` implicitly via TLS.

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

use std::cell::RefCell;
use std::io::{self, Read};

thread_local! {
    static y: RefCell<i32> = const { RefCell::new(123) };
}

fn multi_stage(x: i32, z: i32) -> i32 {
    let mut result: i32 = 0;
    if x != 1 {
        println!("Error: x != 1");
        result = 1;
    } else if y.with_borrow(|y_ref| *y_ref != 2) {
        println!("Error: x == 1 but y != 2");
        result = 2;
    } else if z != 3 {
        println!("Error: x == 1 and y == 2, but z != 3");
        result = 3;
    } else {
        println!("Ok!");
        return result;
    }
    println!("Operation failed");
    result
}

fn main() {
    // Read all stdin, parse three integers: x, y, z
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut it = input.split_whitespace();
    let x: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let y_in: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let z: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);

    y.with_borrow_mut(|y_ref| *y_ref = y_in);

    let result: i32 = multi_stage(x, z);
    println!("Result: {result}");
}
```

**Entity:** thread_local! static y: RefCell<i32>

**States:** DefaultValue(123), OverwrittenFromInput(y_in)

**Transitions:**
- DefaultValue(123) -> OverwrittenFromInput(y_in) via y.with_borrow_mut(|y_ref| *y_ref = y_in)

**Evidence:** thread_local! { static y: RefCell<i32> = const { RefCell::new(123) }; } defines an implicit default state; main(): y.with_borrow_mut(|y_ref| *y_ref = y_in); performs the overwrite transition; multi_stage(): y.with_borrow(|y_ref| *y_ref != 2) branches on the current TLS value; failure message "Error: x == 1 but y != 2" reveals the required condition; main(): let result: i32 = multi_stage(x, z); depends on y having been set earlier

**Implementation:** Make `multi_stage` take an explicit capability/handle proving `y` has been set, e.g. `struct YSetToken; fn set_y(val: i32) -> YSetToken { ... }` and `fn multi_stage(token: &YSetToken, x: i32, z: i32) -> i32` (or pass `y` as an explicit parameter). This removes reliance on implicit TLS state and enforces call ordering at compile time.

---

## Precondition Invariants

### 2. multi_stage input-validation protocol (x==1, y==2, z==3)

**Location**: `/data/test_case/main.rs:1-52`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `multi_stage` encodes a strict precondition chain: it only succeeds (prints "Ok!" and returns 0) when `x == 1`, the thread-local `y == 2`, and `z == 3`. Otherwise it returns an error code (1/2/3) after printing which precondition failed. These constraints are runtime-only checks; the type system treats `x`, `y`, and `z` as arbitrary `i32` so invalid values can always be passed/observed.

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

use std::cell::RefCell;
use std::io::{self, Read};

thread_local! {
    static y: RefCell<i32> = const { RefCell::new(123) };
}

fn multi_stage(x: i32, z: i32) -> i32 {
    let mut result: i32 = 0;
    if x != 1 {
        println!("Error: x != 1");
        result = 1;
    } else if y.with_borrow(|y_ref| *y_ref != 2) {
        println!("Error: x == 1 but y != 2");
        result = 2;
    } else if z != 3 {
        println!("Error: x == 1 and y == 2, but z != 3");
        result = 3;
    } else {
        println!("Ok!");
        return result;
    }
    println!("Operation failed");
    result
}

fn main() {
    // Read all stdin, parse three integers: x, y, z
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut it = input.split_whitespace();
    let x: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let y_in: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let z: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);

    y.with_borrow_mut(|y_ref| *y_ref = y_in);

    let result: i32 = multi_stage(x, z);
    println!("Result: {result}");
}
```

**Entity:** fn multi_stage(x: i32, z: i32) -> i32 (and its inputs x, y(TLS), z)

**States:** InvalidInput, ValidInput

**Transitions:**
- InvalidInput -> ValidInput via supplying values x=1, y(TLS)=2, z=3 before call

**Evidence:** multi_stage(): if x != 1 { println!("Error: x != 1"); result = 1; }; multi_stage(): else if y.with_borrow(|y_ref| *y_ref != 2) { println!("Error: x == 1 but y != 2"); result = 2; }; multi_stage(): else if z != 3 { println!("Error: x == 1 and y == 2, but z != 3"); result = 3; }; multi_stage(): else { println!("Ok!"); return result; }

**Implementation:** Introduce validated newtypes for the required values, e.g. `struct X1; struct Z3; struct Y2;` with constructors that check/parse and return `Option/Result`, then change the signature to `fn multi_stage(_x: X1, _y: Y2, _z: Z3) -> i32` (and pass `y` explicitly rather than reading TLS). This makes invalid states unrepresentable at call sites.

---

