# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 6
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 4
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 3. c2Simplex validity invariant (count governs active vertices and div)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex encodes a GJK-style simplex whose runtime state is determined by the integer `count` (how many of {a,b,c,d} are active). The type system does not enforce that `count` is within the valid range (typically 0..=4) nor that only the first `count` vertices are considered initialized/meaningful. The `div` field also appears to be a cached derived quantity (e.g., a divisor/barycentric denominator) whose validity is coupled to the current simplex contents/count; nothing prevents constructing or mutating a c2Simplex where `div` is stale, zero when it must be non-zero, or otherwise inconsistent with {a,b,c,d,count}. Because the struct is `Copy`, these potentially-invalid combinations can be freely duplicated, further weakening any intended protocol about updating `count`/`div` together.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Capsule; struct c2AABB; struct c2sv; struct c2Proxy, 1 free function(s); struct c2Circle


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

**States:** Empty/Invalid (count=0 or out of range), 1-vertex simplex, 2-vertex simplex, 3-vertex simplex, 4-vertex simplex

**Transitions:**
- Empty/Invalid -> N-vertex simplex by setting count to N and initializing corresponding vertices (a..d)
- N-vertex simplex -> M-vertex simplex by changing count (and updating div accordingly)

**Evidence:** field: `pub count: i32` — runtime integer selects the active simplex size/state; fields: `pub a: c2sv, pub b: c2sv, pub c: c2sv, pub d: c2sv` — four fixed slots imply only a prefix is meaningful depending on `count`; field: `pub div: f32` — cached numeric value likely dependent on the current simplex state, but not tied to it by types; derive: `#[derive(Copy, Clone)]` — allows copying potentially invalid/intermediate states without enforcing an update protocol

**Implementation:** Model the simplex as an enum or typestate by count, e.g. `enum Simplex { One{a,div}, Two{a,b,div}, Three{a,b,c,div}, Four{a,b,c,d,div} }` or `struct Simplex<const N: usize> { verts: [c2sv; N], div: NonZeroF32 }` with constructors ensuring `N` is 1..=4 and `div` computed/validated at creation; provide transition methods that consume `self` and return the next-state type (e.g., `add_vertex(self, v) -> Simplex<{N+1}>`).

---

### 6. c2GJKCache validity protocol (Empty/Invalid vs Populated/Valid simplex cache)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2GJKCache is a C-ABI cache struct whose fields implicitly encode whether the cache is usable. The `count` field (and associated index arrays `iA`/`iB`) suggest a cache that is either empty/invalid (e.g., count == 0 or out of range) or populated/valid (count in 1..=3 with only the first `count` entries of `iA`/`iB` meaningful). These constraints are not enforced by the type system: nothing prevents constructing a cache with `count` outside 0..=3, mismatched/garbage indices, or nonsensical `metric/div` values. Because the struct is `Copy`, invalid states can also be freely duplicated and propagated.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Capsule; struct c2AABB; struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Circle


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
- EmptyOrInvalid -> PopulatedValid via writing fields (e.g., setting count and filling iA/iB)
- PopulatedValid -> EmptyOrInvalid via resetting/overwriting fields (e.g., setting count=0 or corrupting indices)

**Evidence:** line 8: `pub count: i32` alongside fixed-size arrays `iA: [i32; 3]`, `iB: [i32; 3]` implies an invariant `0 <= count <= 3` and that only the first `count` indices are initialized/meaningful; line 6: `pub metric: f32` likely only meaningful when the cache is populated; otherwise it is effectively undefined; line 9-10: index arrays are public and unvalidated, allowing out-of-range/garbage indices inconsistent with `count`; line 4-5: `#[repr(C)]` and `#[derive(Copy, Clone)]` indicate a plain data container where correctness relies on external protocol rather than Rust invariants

**Implementation:** Hide fields behind a safe wrapper: `struct GjkCache { inner: c2GJKCache }` with constructors like `GjkCache::empty()` and `GjkCache::from_indices(metric, indices_a: [i32; N], indices_b: [i32; N])` where `N: 1..=3` is encoded via distinct types (e.g., enums or const generics `N`) and only exposes getters returning `&[i32]` slices of length `count`. Alternatively encode `count` as `u8` plus `TryFrom<i32>` validation and make `iA/iB` private.

---

## Precondition Invariants

### 1. c2Capsule geometric validity (non-negative radius; non-degenerate/normalized endpoints)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The struct is a raw geometry container for a capsule, but it implicitly relies on geometric validity constraints that are not enforced by the type system. At minimum, the radius `r` is expected to be non-negative (and typically finite). Additionally, many capsule algorithms assume endpoints `a` and `b` form a meaningful segment (often `a != b` for non-degenerate capsules) and that all components are finite. As written, any `f32` is accepted (including NaN/Inf/negative), and any endpoints are allowed, so invalid capsules can be constructed and passed to downstream collision/geometry routines.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2AABB; struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Circle


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
- Invalid -> Valid via construction/validation step (not present in this snippet)

**Evidence:** struct c2Capsule { a: c2v, b: c2v, r: f32 } — `r: f32` permits negative/NaN/Inf radii with no validation; #[repr(C)] and #[derive(Copy, Clone)] — encourages treating this as a plain data blob (likely FFI), with no invariants enforced on creation or mutation

**Implementation:** Introduce a validated capsule wrapper, e.g. `struct Capsule { a: c2v, b: c2v, r: NonNegativeF32 }` where `NonNegativeF32` is a newtype that only constructs from finite `f32 >= 0.0` (or `> 0.0`). Optionally provide `TryFrom<c2Capsule> for Capsule` for FFI interop. If degenerate capsules are disallowed, also validate `a != b` (or encode with a `NonDegenerateSegment` newtype).

---

### 5. c2Circle geometric validity (non-negative finite radius)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Circle is a plain data struct intended to represent a geometric circle. Implicitly, its radius should be non-negative (and typically finite), and its center coordinates should be finite. As written, the type system allows constructing circles with negative/NaN/infinite `r` (and potentially invalid `p` depending on `c2v`). Any algorithms operating on circles usually assume a valid radius; this is an invariant not enforced at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Capsule; struct c2AABB; struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
}

```

**Entity:** c2Circle

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via validated construction (not present in this snippet)

**Evidence:** struct c2Circle { pub p: c2v, pub r: f32 } — raw `f32` radius permits negative/NaN/∞ values; #[repr(C)] and `pub` fields indicate FFI/plain-data usage without constructor validation

**Implementation:** Introduce `struct Radius(f32);` with `TryFrom<f32>`/`new(r: f32) -> Option/Result<Radius>` enforcing `r.is_finite() && r >= 0.0`, and use `c2Circle { p: c2v, r: Radius }` (or provide a `c2Circle::new(p, r: Radius)` constructor) so invalid radii cannot be represented.

---

### 4. c2Proxy vertex-count validity invariant (count matches verts prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2Proxy encodes a variable-length list of vertices in a fixed-size array `verts: [c2v; 8]`, with `count: i32` indicating how many entries are logically used. The type system does not enforce that `count` is within the array capacity (0..=8), non-negative, or that only the first `count` vertices are considered initialized/meaningful. Any consumer that indexes or iterates using `count` is implicitly relying on these constraints to avoid out-of-bounds access or using garbage/unintended vertices.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Capsule; struct c2AABB; struct c2Simplex, 3 free function(s); struct c2sv; struct c2Circle


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

```

**Entity:** c2Proxy

**States:** Valid(count in 0..=8 and verts[0..count] initialized), Invalid(count out of range or verts prefix not meaningful)

**Transitions:**
- Invalid -> Valid by constructing/updating c2Proxy such that count is clamped/validated and verts prefix is populated

**Evidence:** line 9: `pub count: i32` stores the logical number of vertices at runtime (can be negative or > 8); line 10: `pub verts: [c2v; 8]` fixed capacity implies an implicit invariant `0 <= count <= 8` and that only a prefix is used; line 6: `#[repr(C)]` suggests FFI/layout-driven usage, increasing reliance on runtime conventions rather than Rust-enforced invariants

**Implementation:** Make `count` a validated type, e.g. `struct VertCount(u8);` with constructor `VertCount::new(n: usize) -> Option<VertCount>` that enforces `n <= 8`. Alternatively store vertices as `([c2v; 8], VertCount)` privately and expose safe accessors like `fn verts(&self) -> &[c2v] { &self.verts[..self.count.get() as usize] }` so callers cannot create invalid `count`/prefix combinations.

---

### 2. c2AABB geometric validity invariant (min <= max per axis)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2AABB encodes an axis-aligned bounding box using two corners (min and max). Many AABB operations implicitly require that min is component-wise <= max; otherwise the box is inverted and downstream computations (overlap tests, expansions, area/perimeter, etc.) may be incorrect. The struct is a plain POD with public fields, Copy/Clone, and no constructor/validation, so the type system does not prevent constructing or mutating an invalid AABB.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 22 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Capsule; struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Circle


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

```

**Entity:** c2AABB

**States:** Valid (min.x <= max.x and min.y <= max.y), Invalid (inverted/degenerate bounds)

**Transitions:**
- Invalid -> Valid via validated construction/normalization (not present in snippet)
- Valid -> Invalid via direct field mutation/copy of bad data

**Evidence:** line 7-10: `pub struct c2AABB { pub min: c2v, pub max: c2v }` exposes raw corners with no validation; line 6: `#[derive(Copy, Clone)]` makes it easy to duplicate potentially-invalid boxes without checks; line 5: `#[repr(C)]` suggests FFI/POD usage, reinforcing that invariants are currently purely conventional rather than type-enforced

**Implementation:** Make `c2AABB` fields private and provide `impl c2AABB { pub fn new(min: c2v, max: c2v) -> Result<Self, Error> { /* check min<=max */ } }`. Optionally add `pub struct ValidAabb(c2AABB);` with `TryFrom<c2AABB>`/`new_unchecked` for FFI, and only expose AABB operations on `ValidAabb`.

---

