# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 4. c2GJKCache validity protocol (Empty/Uninitialized vs Populated cache)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache appears to be a cache record whose fields must be mutually consistent to be meaningful (e.g., count determines which entries in iA/iB are valid, and metric/div are expected to correspond to that cached simplex). As a plain #[repr(C)] Copy struct with all fields public, the type system does not prevent creating nonsensical combinations such as count outside 0..=3, partially-initialized indices, or div values that are invalid for the cached state. Any code consuming this cache must therefore rely on implicit conventions about which fields are valid in which states.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle; struct c2AABB; struct c2Capsule; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2GJKCache {
    pub metric: f32,
    pub count: i32,
    pub iA: [i32; 3],
    pub iB: [i32; 3],
    pub div: f32,
}

```

**Entity:** c2GJKCache

**States:** EmptyOrInvalid, PopulatedValid

**Transitions:**
- EmptyOrInvalid -> PopulatedValid via external initialization/update (not shown in snippet)

**Evidence:** pub struct c2GJKCache { ... } is a plain data container with all fields `pub` (no constructors/validation); field `count: i32` suggests a bounded number of active entries, while `iA: [i32; 3]` and `iB: [i32; 3]` imply only the first `count` slots are meaningful; fields `metric: f32` and `div: f32` imply additional derived/auxiliary values tied to the cached indices, but are not coupled/validated by the type system; #[derive(Copy, Clone)] allows easy duplication of potentially-invalid intermediate states

**Implementation:** Make fields private and expose a validated representation: e.g., `struct GjkCache { metric: f32, div: NonZeroF32OrFinite, entries: ArrayVec<(IndexA, IndexB), 3> }` (or `[(i32,i32);3]` plus `count: u8`), with `TryFrom<c2GJKCache>`/`From<GjkCache>` for FFI. Use a `u8` count newtype enforcing 0..=3, and only allow construction through functions that keep `metric/div` consistent with the chosen entries.

---

### 3. c2Simplex validity invariant (active vertices count + divisor coherence)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex stores up to four support vertices (a,b,c,d) and a runtime `count` indicating how many are active. Many simplex algorithms also rely on `div` being consistent with the active set (e.g., non-zero when dividing barycentric coordinates, and corresponding to the computed determinant/denominator for the current simplex). The type system does not prevent constructing a c2Simplex with an out-of-range `count`, with inactive vertices containing garbage/meaningless values, or with a `div` that does not match the current active simplex—these are latent states encoded only by integers/floats and convention.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle; struct c2AABB; struct c2Capsule; struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Simplex {
    pub a: c2sv,
    pub b: c2sv,
    pub c: c2sv,
    pub d: c2sv,
    pub div: f32,
    pub count: i32,
}

```

**Entity:** c2Simplex

**States:** Empty/Uninitialized (count = 0), 1-vertex simplex (count = 1), 2-vertex simplex (count = 2), 3-vertex simplex (count = 3), 4-vertex simplex (count = 4), Invalid (count outside 0..=4 or div inconsistent)

**Transitions:**
- Empty/Uninitialized -> N-vertex simplex by setting count and populating a..d
- N-vertex simplex -> M-vertex simplex by updating count and recomputing div (and possibly reordering a..d)

**Evidence:** field `count: i32` encodes the number of active simplex vertices at runtime; fields `a`, `b`, `c`, `d` exist unconditionally, implying only a prefix/subset is meaningful depending on `count`; field `div: f32` is a separate scalar that must stay consistent with the current simplex but is not tied to `count`/vertices by the type system; `#[derive(Copy, Clone)]` allows duplicating potentially-invalid combinations of (a,b,c,d,div,count) without validation

**Implementation:** Represent the simplex as an enum or typestate by arity: `enum Simplex { Empty, V1{a,div}, V2{a,b,div}, V3{a,b,c,div}, V4{a,b,c,d,div} }` (or `Simplex<const N: usize>` with `[c2sv; N]`). Make transitions explicit via constructors like `Simplex::from_1(a)` / `add_vertex(self, v) -> Simplex` that recompute `div` internally, eliminating invalid `count` values and keeping `div` coherent with the active vertices.

---

## Precondition Invariants

### 1. c2AABB geometric validity invariant (min <= max per axis)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: An AABB implicitly assumes that for each axis, min <= max (and typically that both are finite). This validity is not enforced by the type system: c2AABB is a plain #[repr(C)] POD with public fields, so callers can construct an 'inverted' or NaN/Inf box that downstream algorithms likely assume cannot happen.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle; struct c2Capsule; struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

```

**Entity:** c2AABB

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via caller reordering/fixing min/max (not provided in this snippet)
- Valid -> Invalid via direct public field mutation/initialization

**Evidence:** struct c2AABB has public fields `min: c2v` and `max: c2v` with no constructor/validation in this snippet; `#[repr(C)]` and `#[derive(Copy, Clone)]` indicate a C-FFI/POD style type where invariants are convention-based rather than enforced

**Implementation:** Make fields private and provide a constructor like `fn new(min: c2v, max: c2v) -> ValidAabb` that enforces/normalizes ordering (swap per-axis as needed) and optionally rejects non-finite values. Expose a `struct ValidAabb(c2AABB);` newtype (or `c2AABB<Valid>` typestate) for internal APIs that require the invariant, while keeping a raw `c2AABB` for FFI boundaries.

---

### 2. c2Proxy vertex-count validity (count bounds and active prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Proxy stores a fixed-capacity vertex array `verts: [c2v; 8]` along with a runtime `count: i32` indicating how many entries are logically active. The type system does not enforce that `count` is within 0..=8, non-negative, or that only the first `count` vertices are considered initialized/meaningful. Any code that iterates `0..count` or assumes `count` matches the active prefix relies on runtime discipline; an out-of-range `count` can lead to panics, incorrect geometry, or (in FFI contexts) UB if passed to C expecting the invariant.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle; struct c2AABB; struct c2Capsule; struct c2GJKCache; struct c2sv; struct c2Simplex, 3 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

```

**Entity:** c2Proxy

**States:** Valid(count in 0..=8), Invalid(count out of range / inconsistent with verts)

**Transitions:**
- Invalid -> Valid by setting count to a value in 0..=8 and ensuring verts[0..count] are the active vertices

**Evidence:** struct c2Proxy: field `count: i32` is a runtime length separate from storage; struct c2Proxy: field `verts: [c2v; 8]` is a fixed-capacity buffer implying an upper bound of 8 active vertices; line with `#[repr(C)]`: suggests FFI layout where C-side code likely assumes `count` respects the array capacity

**Implementation:** Replace `count: i32` with a validated length type (e.g., `struct VertCount(u8)` with constructor enforcing `<= 8`), or use `usize` plus `TryFrom<i32>` at the boundary. Consider representing vertices as `([c2v; 8], VertCount)` with helpers `active_verts(&self) -> &[c2v]` that slices using the validated count.

---

