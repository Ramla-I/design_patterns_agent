# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 2. c2AABB validity invariant (min must be component-wise <= max)

**Location**: `/data/test_case/lib.rs:1-112`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Several computations assume `c2AABB` represents a real axis-aligned box where `min` is the lower corner and `max` is the upper corner (component-wise). This is not enforced by the struct fields (`min: c2v, max: c2v`). If a caller constructs an AABB with swapped/inverted bounds, `c2Clampv(A.p, B.min, B.max)` and the separation test in `c2AABBtoAABB` no longer have the intended semantics (e.g., clamp with lo>hi per-axis is ill-defined for the geometric meaning). This could be enforced by a constructor/newtype that normalizes or validates ordering.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); struct c2Circle, 1 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
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

#[inline]
pub(crate) fn c2V(x: f32, y: f32) -> c2v {
    c2v { x, y }
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

#[inline]
pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    let r2 = r * r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let separated = (B.max.x < A.min.x) || (A.max.x < B.min.x) || (B.max.y < A.min.y) || (A.max.y < B.min.y);
    (!separated) as i32
}

#[no_mangle]
pub unsafe extern "C" fn collided(
    A: *const core::ffi::c_void,
    typeA: C2_TYPE,
    B: *const core::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    match typeA {
        0 => match typeB {
            0 => c2CircletoCircle(*(A as *const c2Circle), *(B as *const c2Circle)),
            1 => c2CircletoAABB(*(A as *const c2Circle), *(B as *const c2AABB)),
            _ => 0,
        },
        1 => match typeB {
            0 => c2CircletoAABB(*(B as *const c2Circle), *(A as *const c2AABB)),
            1 => c2AABBtoAABB(*(A as *const c2AABB), *(B as *const c2AABB)),
            _ => 0,
        },
        _ => 0,
    }
}
```

**Entity:** c2AABB

**States:** ValidAABB(min<=max), InvalidAABB(min>max in some axis)

**Transitions:**
- InvalidAABB -> ValidAABB via normalize/sort-bounds constructor (not present in code)
- ValidAABB -> InvalidAABB via direct field construction/FFI writes (currently possible)

**Evidence:** struct field names: `c2AABB { min: c2v, max: c2v }` imply ordered bounds but do not enforce them; c2CircletoAABB: `let L = c2Clampv(A.p, B.min, B.max);` assumes `B.min` is the per-axis lower bound and `B.max` is the upper bound; c2AABBtoAABB: separation logic compares `B.max.x < A.min.x` etc., which only matches geometric intent when min/max are ordered

**Implementation:** Introduce `struct ValidAABB(c2AABB);` with `impl ValidAABB { fn new(min: c2v, max: c2v) -> Self { let min2 = c2Minv(min, max); let max2 = c2Maxv(min, max); Self(c2AABB{min:min2,max:max2}) } }` (or `try_new` returning `Option/Result`). Update collision functions to accept `ValidAABB` (or `&ValidAABB`) so ordering is guaranteed for internal callers.

---

### 3. c2Circle geometric validity (NonNegativeRadius / InvalidRadius)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2Circle encodes a geometric circle with center `p` and radius `r`. A latent invariant is that the radius should be non-negative (and often finite). As defined, `r: f32` allows negative values and NaN/inf, which can silently break downstream geometry algorithms that assume a valid radius. This validity condition is not enforced by the type system; any code constructing or mutating `c2Circle` can place it into an invalid state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
}

```

**Entity:** c2Circle

**States:** Valid (r >= 0), Invalid (r < 0)

**Transitions:**
- Valid -> Invalid via writing a negative/NaN/inf value into field `r`
- Invalid -> Valid via writing a non-negative finite value into field `r`

**Evidence:** line 10: `pub r: f32` is an unconstrained floating-point radius; line 8-11: `pub struct c2Circle { pub p: c2v, pub r: f32 }` exposes fields publicly, allowing construction of invalid circles; line 6-7: `#[repr(C)]` suggests FFI/ABI usage, increasing likelihood that consumers assume C-side invariant 'radius >= 0'

**Implementation:** Introduce `struct Radius(f32);` with a smart constructor `Radius::new(r: f32) -> Option<Radius>` (or `Result`) enforcing `r.is_finite() && r >= 0.0`. Then define `pub struct c2Circle { pub p: c2v, pub r: Radius }`. If `#[repr(C)]` must be preserved for FFI, keep `c2Circle` as an FFI struct and provide a safe wrapper `struct Circle { raw: c2Circle }` that validates on creation and only exposes `Radius`-validated accessors.

---

## Protocol Invariants

### 1. Tagged-void-pointer protocol for collision shapes (type tag must match pointed-to layout)

**Location**: `/data/test_case/lib.rs:1-112`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The unsafe extern "C" function `collided` implements a tagged-pointer protocol: `typeA` and `typeB` determine how to interpret the raw `*const c_void` pointers `A` and `B`. Correctness requires that when `typeA == 0`, `A` must point to a valid `c2Circle`; when `typeA == 1`, `A` must point to a valid `c2AABB` (same for B/typeB). This invariant is not enforced by the type system: the function performs unchecked casts and dereferences (`*(A as *const c2Circle)` etc.). Invalid tags, null/dangling pointers, or mismatched tags lead to UB, while unknown tags silently return 0 ("no collision"), which can mask errors. A safer design would encode the shape kind at the type level (or via an enum) and avoid `c_void`/integer tags for Rust callers; for C callers, provide separate entry points per shape pair or require a validated handle/capability.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); struct c2Circle, 1 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
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

#[inline]
pub(crate) fn c2V(x: f32, y: f32) -> c2v {
    c2v { x, y }
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

#[inline]
pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    let r2 = r * r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let separated = (B.max.x < A.min.x) || (A.max.x < B.min.x) || (B.max.y < A.min.y) || (A.max.y < B.min.y);
    (!separated) as i32
}

#[no_mangle]
pub unsafe extern "C" fn collided(
    A: *const core::ffi::c_void,
    typeA: C2_TYPE,
    B: *const core::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    match typeA {
        0 => match typeB {
            0 => c2CircletoCircle(*(A as *const c2Circle), *(B as *const c2Circle)),
            1 => c2CircletoAABB(*(A as *const c2Circle), *(B as *const c2AABB)),
            _ => 0,
        },
        1 => match typeB {
            0 => c2CircletoAABB(*(B as *const c2Circle), *(A as *const c2AABB)),
            1 => c2AABBtoAABB(*(A as *const c2AABB), *(B as *const c2AABB)),
            _ => 0,
        },
        _ => 0,
    }
}
```

**Entity:** collided (FFI API: (A, typeA, B, typeB))

**States:** CirclePtr(tag=0), AABBPtr(tag=1), UnknownTag(other)

**Transitions:**
- UnknownTag(other) -> (returns 0) via default match arms
- CirclePtr(tag=0) x CirclePtr(tag=0) -> dispatch c2CircletoCircle()
- CirclePtr(tag=0) x AABBPtr(tag=1) -> dispatch c2CircletoAABB()
- AABBPtr(tag=1) x CirclePtr(tag=0) -> dispatch c2CircletoAABB() with swapped args
- AABBPtr(tag=1) x AABBPtr(tag=1) -> dispatch c2AABBtoAABB()

**Evidence:** signature: `unsafe extern "C" fn collided(A: *const c_void, typeA: C2_TYPE, B: *const c_void, typeB: C2_TYPE)` uses raw void pointers plus integer tags; match on `typeA` and `typeB` with literals `0` and `1` determines interpretation of pointers; unchecked deref/cast: `*(A as *const c2Circle)`, `*(B as *const c2AABB)`, etc.; default arms: `_ => 0` for unknown `typeA/typeB` silently treat invalid states as non-collision

**Implementation:** Define typed wrappers for Rust-side use, e.g. `#[repr(transparent)] struct CircleRef(*const c2Circle); struct AABBRef(*const c2AABB);` with constructors `unsafe fn from_raw(ptr) -> Option<Self>` that validate non-null/alignment. Replace `(ptr, tag)` with an enum for Rust callers: `enum ShapeRef<'a> { Circle(&'a c2Circle), AABB(&'a c2AABB) }` and dispatch on the enum. For the C ABI, consider exposing separate functions (`collided_circle_circle`, `collided_circle_aabb`, `collided_aabb_aabb`) to make mismatched-layout calls impossible at compile time on the Rust side and harder to misuse from C.

---

