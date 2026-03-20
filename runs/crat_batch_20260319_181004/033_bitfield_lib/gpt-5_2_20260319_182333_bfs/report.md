# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. foo_t packed-bitfield validity (x/y ranges, reserved bits, and bool normalization)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: foo_t encodes multiple logical fields (x, y, b) into a single backing byte `x_y_b`. The code implicitly relies on `x` fitting in 2 bits, `y` fitting in 3 bits, `b` being a normalized boolean, and (often for C interop) any remaining/unused bits being in an expected state. Because the raw storage is exposed as `[u8; 1]` and the struct is `Copy, Clone`, it is possible to construct or mutate `foo_t` with arbitrary bit patterns that violate these logical constraints; the type system does not distinguish a "validly-encoded" foo_t from an arbitrary byte pattern.

**Evidence**:

```rust
// Note: Other parts of this module contain: 1 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone, BitfieldStruct)]
        pub struct foo_t {
            #[bitfield(name = "x", ty = "u32", bits = "0..=1")]
            #[bitfield(name = "y", ty = "u32", bits = "2..=4")]
            #[bitfield(name = "b", ty = "bool", bits = "5..=5")]
            pub x_y_b: [u8; 1],
            pub c2rust_padding: [u8; 3],
            pub z: i32,
        }

```

**Entity:** foo_t

**States:** ValidBitfieldEncoding, InvalidBitfieldEncoding

**Transitions:**
- InvalidBitfieldEncoding -> ValidBitfieldEncoding via setting x/y/b through generated BitfieldStruct accessors (not shown here)

**Evidence:** struct foo_t: #[derive(Copy, Clone, BitfieldStruct)] implies logical fields are packed into raw storage; field x_y_b: [u8; 1] is the raw backing store for bitfields, allowing arbitrary byte values; #[bitfield(name = "x", ty = "u32", bits = "0..=1")] restricts x to 2 bits but is not enforced by the Rust type of x_y_b; #[bitfield(name = "y", ty = "u32", bits = "2..=4")] restricts y to 3 bits but is not enforced by the Rust type of x_y_b; #[bitfield(name = "b", ty = "bool", bits = "5..=5")] expects a boolean in a single bit, but x_y_b can contain any value

**Implementation:** Make the raw storage private and expose only validated constructors/accessors. For example, wrap the backing byte in a `struct FooBits(u8);` with `TryFrom<u8>`/`from_parts(x: X2, y: Y3, b: bool)` constructors, where `X2` and `Y3` are newtypes that validate ranges (or use const generics / bounded integer types). Keep `foo_t` construction behind these APIs so invalid encodings cannot be created in safe Rust.

---

### 2. foo_t bitfield validity invariant (x/y must fit declared bit widths)

**Location**: `/data/test_case/lib.rs:1-51`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: foo_t encodes fields x (2 bits) and y (3 bits) inside the single-byte x_y_b bitfield storage. The API accepts unconstrained u32 values for x/y (e.g., driver(x: u32, y: u32, ...)), then writes them via set_x/set_y into the bitfield. This implies an invariant that callers should only provide values that fit the allocated bit widths; otherwise values will be truncated/masked, which is a silent semantic change not represented in the types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct foo_t, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[macro_use]
extern crate c2rust_bitfields;

pub mod src {
    pub mod lib {
        use ::c2rust_bitfields;

        #[repr(C)]
        #[derive(Copy, Clone, BitfieldStruct)]
        pub struct foo_t {
            #[bitfield(name = "x", ty = "u32", bits = "0..=1")]
            #[bitfield(name = "y", ty = "u32", bits = "2..=4")]
            #[bitfield(name = "b", ty = "bool", bits = "5..=5")]
            pub x_y_b: [u8; 1],
            pub c2rust_padding: [u8; 3],
            pub z: i32,
        }

        pub(crate) fn print_foo(foo: Option<&mut foo_t>) {
            if let Some(foo) = foo {
                println!("{} {} {} {}", foo.x(), foo.y(), foo.b() as i32, foo.z);
            }
        }

        #[no_mangle]
        pub extern "C" fn driver(x: u32, y: u32, b: bool, z: i32) {
            let mut foo = foo_t {
                x_y_b: [0; 1],
                c2rust_padding: [0; 3],
                z,
            };
            foo.set_x(x);
            foo.set_y(y);
            foo.set_b(b);

            print_foo(Some(&mut foo));
        }
    }
}
```

**Entity:** foo_t

**States:** ValidBits(x in 0..=3, y in 0..=7), OutOfRangeInput(x or y exceeds bit width)

**Transitions:**
- OutOfRangeInput -> ValidBits via caller-side validation before set_x/set_y
- ValidBits -> ValidBits via set_x/set_y with in-range values

**Evidence:** foo_t bitfield definitions: #[bitfield(name = "x", ty = "u32", bits = "0..=1")] and #[bitfield(name = "y", ty = "u32", bits = "2..=4")]; driver(x: u32, y: u32, ...): accepts unconstrained u32 for x and y; driver: foo.set_x(x); foo.set_y(y); writes unchecked inputs into bitfield storage

**Implementation:** Introduce newtypes like `struct X2(u8); struct Y3(u8);` with `TryFrom<u32>` performing range checks (0..=3 and 0..=7). Change `driver`/constructors to accept `X2`/`Y3` (or a `FooBuilder` that validates) so out-of-range values are rejected at compile-time API boundaries (types) rather than silently truncated at runtime.

---

