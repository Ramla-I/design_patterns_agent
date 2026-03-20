# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## State Machine Invariants

### 3. Closed set of shape kinds encoded as integers (invalid values possible)

**Location**: `/data/test_case/lib.rs:1-175`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `C2_TYPE` is a `u32` with constants representing a small closed set of shape kinds. The code relies on these exact numeric values to drive dispatch. Because it is not an enum, any other `u32` can be passed, creating an 'Unknown' state that is only handled by the `_ => 0` fallback at runtime (and may mask caller bugs).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 1 free function(s); struct c2v, 7 free function(s); struct c2Circle, 2 free function(s); struct c2AABB

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type C2_TYPE = u32;
pub const C2_TYPE_CAPSULE: C2_TYPE = 2;
pub const C2_TYPE_AABB: C2_TYPE = 1;
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) fn c2V(x: f32, y: f32) -> c2v {
    c2v { x, y }
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Maxv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Minv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Clampv(a: c2v, lo: c2v, hi: c2v) -> c2v {
    c2Maxv(lo, c2Minv(a, hi))
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Dot(a: c2v, b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    (d2 < r * r) as i32
}

pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

pub(crate) fn c2CircletoCapsule(A: c2Circle, B: c2Capsule) -> i32 {
    let n = c2Sub(B.b, B.a);
    let ap = c2Sub(A.p, B.a);
    let da = c2Dot(ap, n);

    let d2 = if da < 0.0 {
        c2Dot(ap, ap)
    } else {
        let db = c2Dot(c2Sub(A.p, B.b), n);
        if db < 0.0 {
            let nn = c2Dot(n, n);
            let e = c2Sub(ap, c2Mulvs(n, da / nn));
            c2Dot(e, e)
        } else {
            let bp = c2Sub(A.p, B.b);
            c2Dot(bp, bp)
        }
    };

    let r = A.r + B.r;
    (d2 < r * r) as i32
}

pub(crate) unsafe fn c2Collided(
    A: *const std::ffi::c_void,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    match typeB {
        C2_TYPE_CIRCLE => c2CircletoCircle(*(A as *const c2Circle), *(B as *const c2Circle)),
        C2_TYPE_AABB => c2CircletoAABB(*(A as *const c2Circle), *(B as *const c2AABB)),
        C2_TYPE_CAPSULE => c2CircletoCapsule(*(A as *const c2Circle), *(B as *const c2Capsule)),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn circle_collide(x: f32, y: f32, r: f32) -> i32 {
    let circle_in = c2Circle { p: c2V(x, y), r };

    let circle = c2Circle {
        p: c2V(-70.0, 0.0),
        r: 20.0,
    };

    let aabb = c2AABB {
        min: c2V(-40.0, -40.0),
        max: c2V(-15.0, -15.0),
    };

    let capsule = c2Capsule {
        a: c2V(-40.0, 40.0),
        b: c2V(-20.0, 100.0),
        r: 10.0,
    };

    let mut result = 0;
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const circle).cast::<std::ffi::c_void>(),
        C2_TYPE_CIRCLE,
    );
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const aabb).cast::<std::ffi::c_void>(),
        C2_TYPE_AABB,
    ) << 1;
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const capsule).cast::<std::ffi::c_void>(),
        C2_TYPE_CAPSULE,
    ) << 2;

    result
}
```

**Entity:** C2_TYPE / C2_TYPE_* constants

**States:** Circle (0), AABB (1), Capsule (2), Unknown/invalid (any other u32)

**Transitions:**
- Valid numeric constant -> used in `c2Collided` match arm
- Invalid numeric value -> handled by `_ => 0`

**Evidence:** `pub type C2_TYPE = u32;` allows arbitrary integers; `pub const C2_TYPE_CIRCLE: C2_TYPE = 0;`, `C2_TYPE_AABB = 1;`, `C2_TYPE_CAPSULE = 2;` define an intended closed set; `match typeB { ... _ => 0 }` in `c2Collided` is a runtime handling of invalid/unknown values

**Implementation:** Define `#[repr(u32)] enum C2Type { Circle = 0, Aabb = 1, Capsule = 2 }` and change `c2Collided` to take `typeB: C2Type` (or `TryFrom<u32>` at the FFI boundary). This removes the 'unknown integer' state from internal APIs.

---

## Precondition Invariants

### 1. c2Capsule geometric validity invariants (non-negative radius, finite values, non-degenerate axis)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Capsule represents a geometric capsule defined by endpoints a/b and radius r. The type allows construction of physically/geomtrically invalid values (e.g., negative radius, NaN/inf components, or degenerate capsules where a==b if the rest of the library assumes a segment). These are latent preconditions that downstream algorithms typically rely on but are not enforced by the type system because fields are all public and r is a plain f32.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 7 free function(s); struct c2Circle, 2 free function(s); struct c2AABB; 2 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

```

**Entity:** c2Capsule

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via validated constructor (not present in snippet)
- Valid -> Invalid via direct field mutation (a/b/r are public)

**Evidence:** line 7-11: `pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 }` — all fields are public, so any values (including NaN/inf and negative r) are representable; line 4-5: `#[repr(C)]` + `Copy, Clone` suggests this is a plain FFI/geometry POD where invariants are expected externally, not encoded in types

**Implementation:** Make fields private and provide constructors returning `Result<c2Capsule, Error>` (or `Option`). Use a `NonNegativeF32`/`FiniteF32` newtype for `r` (and possibly for `c2v` components) to prevent negative/NaN/inf at compile time where possible. If FFI requires `repr(C)`/public layout, keep `c2Capsule` as the raw POD and introduce `ValidatedCapsule(c2Capsule)` newtype that can only be created through validation and is the input to geometry algorithms.

---

## Protocol Invariants

### 2. FFI shape dispatch protocol (type tag must match pointee layout)

**Location**: `/data/test_case/lib.rs:1-175`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: c2Collided implements a tagged-union-style dispatch where the runtime value `typeB` determines how the raw `*const c_void` pointers are reinterpreted. Correctness requires that `A` always points to a `c2Circle`, and that `B` points to the concrete shape indicated by `typeB`. This invariant is not enforced by the type system because the API accepts `*const c_void` plus an integer tag; mismatches can lead to invalid reads when doing `*(B as *const ...)`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 1 free function(s); struct c2v, 7 free function(s); struct c2Circle, 2 free function(s); struct c2AABB

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type C2_TYPE = u32;
pub const C2_TYPE_CAPSULE: C2_TYPE = 2;
pub const C2_TYPE_AABB: C2_TYPE = 1;
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) fn c2V(x: f32, y: f32) -> c2v {
    c2v { x, y }
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Maxv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Minv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Clampv(a: c2v, lo: c2v, hi: c2v) -> c2v {
    c2Maxv(lo, c2Minv(a, hi))
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Dot(a: c2v, b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    (d2 < r * r) as i32
}

pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

pub(crate) fn c2CircletoCapsule(A: c2Circle, B: c2Capsule) -> i32 {
    let n = c2Sub(B.b, B.a);
    let ap = c2Sub(A.p, B.a);
    let da = c2Dot(ap, n);

    let d2 = if da < 0.0 {
        c2Dot(ap, ap)
    } else {
        let db = c2Dot(c2Sub(A.p, B.b), n);
        if db < 0.0 {
            let nn = c2Dot(n, n);
            let e = c2Sub(ap, c2Mulvs(n, da / nn));
            c2Dot(e, e)
        } else {
            let bp = c2Sub(A.p, B.b);
            c2Dot(bp, bp)
        }
    };

    let r = A.r + B.r;
    (d2 < r * r) as i32
}

pub(crate) unsafe fn c2Collided(
    A: *const std::ffi::c_void,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    match typeB {
        C2_TYPE_CIRCLE => c2CircletoCircle(*(A as *const c2Circle), *(B as *const c2Circle)),
        C2_TYPE_AABB => c2CircletoAABB(*(A as *const c2Circle), *(B as *const c2AABB)),
        C2_TYPE_CAPSULE => c2CircletoCapsule(*(A as *const c2Circle), *(B as *const c2Capsule)),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn circle_collide(x: f32, y: f32, r: f32) -> i32 {
    let circle_in = c2Circle { p: c2V(x, y), r };

    let circle = c2Circle {
        p: c2V(-70.0, 0.0),
        r: 20.0,
    };

    let aabb = c2AABB {
        min: c2V(-40.0, -40.0),
        max: c2V(-15.0, -15.0),
    };

    let capsule = c2Capsule {
        a: c2V(-40.0, 40.0),
        b: c2V(-20.0, 100.0),
        r: 10.0,
    };

    let mut result = 0;
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const circle).cast::<std::ffi::c_void>(),
        C2_TYPE_CIRCLE,
    );
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const aabb).cast::<std::ffi::c_void>(),
        C2_TYPE_AABB,
    ) << 1;
    result += c2Collided(
        (&raw const circle_in).cast::<std::ffi::c_void>(),
        (&raw const capsule).cast::<std::ffi::c_void>(),
        C2_TYPE_CAPSULE,
    ) << 2;

    result
}
```

**Entity:** c2Collided (and the implicit 'shape pointer + type tag' pair)

**States:** B points to c2Circle and typeB==C2_TYPE_CIRCLE, B points to c2AABB and typeB==C2_TYPE_AABB, B points to c2Capsule and typeB==C2_TYPE_CAPSULE, Invalid/unknown typeB or mismatched pointer/type (UB risk)

**Transitions:**
- Well-typed tag/pointer pairing -> (match typeB) -> concrete collision routine
- Unknown typeB -> returns 0 via `_ => 0` (silent fallback)

**Evidence:** function `pub(crate) unsafe fn c2Collided(A: *const c_void, B: *const c_void, typeB: C2_TYPE)` uses raw void pointers plus a numeric tag; `match typeB { C2_TYPE_CIRCLE => ... *(B as *const c2Circle), C2_TYPE_AABB => ... *(B as *const c2AABB), C2_TYPE_CAPSULE => ... *(B as *const c2Capsule) }` reinterprets `B` based on `typeB`; all match arms cast `A as *const c2Circle` (implicit precondition: A must always be a circle); `_ => 0` indicates an 'invalid/unknown type' state handled only at runtime

**Implementation:** Replace `(ptr: *const c_void, tag: C2_TYPE)` with an enum carrying typed pointers, e.g. `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB), Capsule(&'a c2Capsule) }`; then `fn collided(a: &c2Circle, b: ShapeRef) -> i32` has no casts. For FFI, provide constructors `ShapeRef::from_raw_circle(ptr)` etc. so the only `unsafe` is at the boundary and the tag/pointer pairing becomes unrepresentable in safe code.

---

