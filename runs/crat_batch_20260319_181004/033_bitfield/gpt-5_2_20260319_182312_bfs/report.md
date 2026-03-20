# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 1

## Precondition Invariants

### 1. foo_t packed-bitfield validity (x:2 bits, y:3 bits, b:1 bit)

**Location**: `/data/test_case/main.rs:1-40`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: foo_t stores three logical fields (x, y, b) packed into a single byte (x_y_b[0]). The intended invariant is that only bits 0..=5 carry meaning (x: bits 0..=1, y: bits 2..=4, b: bit 5) and that x and y are within their declared bit-widths. The type system does not expose or enforce these logical domains: x() and y() return u32 (not range-limited types), and the raw storage x_y_b allows construction/copying of values with arbitrary/unknown bits (including bits 6..=7) and arbitrary x/y ranges if written through raw byte access or FFI. The setters mask inputs to fit, but this is only a runtime convention and does not prevent invalid representations from existing in foo_t.

**Evidence**:

```rust
// Note: Other parts of this module contain: 2 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct foo_t {
    // Bit layout (within this byte):
    // bits 0..=1: x (2 bits)
    // bits 2..=4: y (3 bits)
    // bit 5:      b (1 bit)
    pub x_y_b: [u8; 1],
    pub c2rust_padding: [u8; 3],
    pub z: i32,
}

impl foo_t {
    fn x(&self) -> u32 {
        (self.x_y_b[0] & 0b0000_0011) as u32
    }
    fn set_x(&mut self, x: u32) {
        let x = (x & 0b11) as u8;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0000_0011) | x;
    }

    fn y(&self) -> u32 {
        ((self.x_y_b[0] >> 2) & 0b0000_0111) as u32
    }
    fn set_y(&mut self, y: u32) {
        let y = ((y & 0b111) as u8) << 2;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0001_1100) | y;
    }

    fn b(&self) -> bool {
        ((self.x_y_b[0] >> 5) & 0b1) != 0
    }
    fn set_b(&mut self, b: bool) {
        let bit = (b as u8) << 5;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0010_0000) | bit;
    }
}

```

**Entity:** foo_t

**States:** ValidBitLayout, Invalid/OutOfSpecBitLayout

**Transitions:**
- Invalid/OutOfSpecBitLayout -> ValidBitLayout via set_x()/set_y()/set_b() (they overwrite only the relevant masked bits)

**Evidence:** comment on foo_t.x_y_b: "Bit layout ... bits 0..=1: x ... bits 2..=4: y ... bit 5: b" documents an implicit representation invariant; field: pub x_y_b: [u8; 1] exposes the raw packed storage publicly (can contain any byte value, including setting unused bits 6..=7); method: fn set_x(&mut self, x: u32) masks with (x & 0b11) implying the precondition/domain 'x fits in 2 bits' is not a type-level guarantee; method: fn set_y(&mut self, y: u32) masks with (y & 0b111) implying the precondition/domain 'y fits in 3 bits' is not a type-level guarantee; method: fn b(&self) reads bit 5; method fn set_b(&mut self, b: bool) writes bit 5, implying only a single-bit boolean is meaningful there

**Implementation:** Make x_y_b private and expose typed accessors using range-restricted newtypes: e.g., struct X2(u8); impl TryFrom<u8> for X2 { /* ensure <4 */ }; struct Y3(u8) { /* ensure <8 */ }. Change set_x/set_y to take X2/Y3, and optionally provide a checked constructor for foo_t that zeroes/validates reserved bits (6..=7). For FFI, keep #[repr(C)] on an inner raw struct and wrap it in a safe Rust type that enforces the invariant.

---

### 2. foo_t packed-bitfield validity invariant (x/y range + b flag)

**Location**: `/data/test_case/main.rs:1-80`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: foo_t encodes three logical fields (x:2 bits, y:3 bits, b:1 bit) into the single byte x_y_b[0]. The setters silently mask/truncate inputs to fit the available bits (x & 0b11, y & 0b111). Callers may assume x and y preserve the provided values, but if x > 3 or y > 7 those values are irreversibly truncated; this is an implicit 'inputs must be in range' precondition not enforced by the type system. The valid logical state is 'values are within representable range' versus 'caller supplied out-of-range values that got truncated'.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct foo_t, 1 free function(s), impl foo_t (6 methods)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::io::{self, Read};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct foo_t {
    // Bit layout (within this byte):
    // bits 0..=1: x (2 bits)
    // bits 2..=4: y (3 bits)
    // bit 5:      b (1 bit)
    pub x_y_b: [u8; 1],
    pub c2rust_padding: [u8; 3],
    pub z: i32,
}

impl foo_t {
    fn x(&self) -> u32 {
        (self.x_y_b[0] & 0b0000_0011) as u32
    }
    fn set_x(&mut self, x: u32) {
        let x = (x & 0b11) as u8;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0000_0011) | x;
    }

    fn y(&self) -> u32 {
        ((self.x_y_b[0] >> 2) & 0b0000_0111) as u32
    }
    fn set_y(&mut self, y: u32) {
        let y = ((y & 0b111) as u8) << 2;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0001_1100) | y;
    }

    fn b(&self) -> bool {
        ((self.x_y_b[0] >> 5) & 0b1) != 0
    }
    fn set_b(&mut self, b: bool) {
        let bit = (b as u8) << 5;
        self.x_y_b[0] = (self.x_y_b[0] & !0b0010_0000) | bit;
    }
}

fn print_foo(foo: &mut foo_t) {
    println!("{} {} {} {}", foo.x(), foo.y(), foo.b() as i32, foo.z);
}

fn driver(x: u32, y: u32, b: bool, z: i32) {
    let mut foo = foo_t {
        x_y_b: [0; 1],
        c2rust_padding: [0; 3],
        z,
    };
    foo.set_x(x);
    foo.set_y(y);
    foo.set_b(b);
    print_foo(&mut foo);
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();

    let x: u32 = it.next().unwrap().parse().unwrap();
    let y: u32 = it.next().unwrap().parse().unwrap();
    let b_i: i32 = it.next().unwrap().parse().unwrap();
    let z: i32 = it.next().unwrap().parse().unwrap();

    driver(x, y, b_i != 0, z);
}
```

**Entity:** foo_t

**States:** ValidBitfieldValues, Truncated/OutOfRangeInput

**Transitions:**
- ValidBitfieldValues -> Truncated/OutOfRangeInput via set_x(x) when x has bits outside 0b11
- ValidBitfieldValues -> Truncated/OutOfRangeInput via set_y(y) when y has bits outside 0b111
- Truncated/OutOfRangeInput -> ValidBitfieldValues via set_x/set_y with in-range values

**Evidence:** comment on foo_t.x_y_b: "bits 0..=1: x (2 bits)", "bits 2..=4: y (3 bits)", "bit 5: b (1 bit)"; set_x(): `let x = (x & 0b11) as u8;` (masks to 2 bits); set_y(): `let y = ((y & 0b111) as u8) << 2;` (masks to 3 bits); x(): `(self.x_y_b[0] & 0b0000_0011) as u32` and y(): `((self.x_y_b[0] >> 2) & 0b0000_0111) as u32` decode only those bit ranges

**Implementation:** Introduce newtypes `struct X2(u8); struct Y3(u8);` with `TryFrom<u32>` validating ranges (0..=3, 0..=7). Change `set_x(&mut self, x: X2)` / `set_y(&mut self, y: Y3)` so out-of-range values are impossible to pass without an explicit fallible conversion. Optionally provide `fn with_fields(x: X2, y: Y3, b: bool, z: i32) -> foo_t` as a safe constructor and keep raw bit operations private.

---

