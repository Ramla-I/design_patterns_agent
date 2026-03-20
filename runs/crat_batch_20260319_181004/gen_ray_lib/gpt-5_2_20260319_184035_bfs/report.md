# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 6
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 5
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 6. c2Circle validity invariant (Non-negative / finite radius; finite position)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Circle is a plain C-compatible data struct with no constructors or validation. Implicitly, geometric code typically assumes the radius is non-negative (often strictly > 0 for some operations) and that both the center point and radius are finite (not NaN/Inf). These constraints are not enforced by the type system because `r: f32` permits negative/NaN/Inf values and `p: c2v` may also contain non-finite components. Any downstream algorithms likely rely on these as preconditions, but violations would only be caught (if at all) via runtime checks elsewhere.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; 1 free function(s)


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
- Invalid -> Valid via external validation/normalization (not present in this snippet)
- Valid -> Invalid via direct field mutation/FFI write (no API restriction)

**Evidence:** struct c2Circle { pub p: c2v, pub r: f32 } — public fields allow constructing/mutating potentially invalid circles; r: f32 — allows negative, NaN, and infinite values; no local validation/construction API present; #[repr(C)] — suggests FFI usage where unchecked inputs are common and invariants are often assumed by consumers

**Implementation:** Make fields private and provide constructors like `c2Circle::new(p, r: NonNegativeF32)` where `NonNegativeF32` (or `FiniteF32`) is a validated newtype (possibly using `TryFrom<f32>`). Optionally expose `c2Circle<Valid>` via a typestate marker if you need to represent both raw/untrusted and validated circles at the type level.

---

### 3. Raycast output must be provided when a hit is possible (Option used but treated as required)

**Location**: `/data/test_case/lib.rs:1-449`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Several raycast functions accept `out: Option<&mut c2Raycast>` but unconditionally `unwrap()` it on code paths where a hit is computed or even before hit testing completes. This means callers must pass `Some(&mut ...)` whenever the function might reach those paths; passing `None` will panic. The API advertises optional output, but the implementation relies on a stronger precondition that is not enforced by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Raycast {
    pub t: f32,
    pub n: c2v,
}

pub type C2_TYPE = u32;
pub const C2_TYPE_CAPSULE: C2_TYPE = 2;
pub const C2_TYPE_AABB: C2_TYPE = 1;
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Ray {
    pub p: c2v,
    pub d: c2v,
    pub t: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2m {
    pub x: c2v,
    pub y: c2v,
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
pub(crate) fn c2Dot(a: c2v, b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2Absv(a: c2v) -> c2v {
    c2V(a.x.abs(), a.y.abs())
}

pub(crate) fn c2RaytoCircle(mut A: c2Ray, mut B: c2Circle, mut out: Option<&mut c2Raycast>) -> i32 {
    let p = B.p;
    let m = c2Sub(A.p, p);
    let c = c2Dot(m, m) - B.r * B.r;
    let b = c2Dot(m, A.d);
    let disc = b * b - c;
    if disc < 0.0f32 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0f32 && t <= A.t {
        let out = out.as_deref_mut().unwrap();
        out.t = t;
        let impact = c2Add(A.p, c2Mulvs(A.d, t));
        out.n = c2Norm(c2Sub(impact, p));
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0f32 {
        0.0f32
    } else if da * db > 0.0f32 {
        1.0f32
    } else {
        let d = da - db;
        if d != 0.0f32 { da / d } else { 0.0f32 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0f32 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0f32, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0f32, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0f32, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0f32, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0f32, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0f32, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0f32, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0f32, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0: i32 = (t0 <= 1.0f32) as i32;
    let hit1: i32 = (t1 <= 1.0f32) as i32;
    let hit2: i32 = (t2 <= 1.0f32) as i32;
    let hit3: i32 = (t3 <= 1.0f32) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    let out = out.as_deref_mut().unwrap();
    if t0 >= t1 && t0 >= t2 && t0 >= t3 {
        out.t = t0 * A.t;
        out.n = c2V(-1.0f32, 0.0f32);
    } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
        out.t = t1 * A.t;
        out.n = c2V(1.0f32, 0.0f32);
    } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
        out.t = t2 * A.t;
        out.n = c2V(0.0f32, -1.0f32);
    } else {
        out.t = t3 * A.t;
        out.n = c2V(0.0f32, 1.0f32);
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

#[inline]
pub(crate) fn c2MulmvT(a: c2m, b: c2v) -> c2v {
    c2V(a.x.x * b.x + a.x.y * b.y, a.y.x * b.x + a.y.y * b.y)
}

pub(crate) fn c2AABBtoPoint(mut A: c2AABB, mut B: c2v) -> i32 {
    let d0: i32 = (B.x < A.min.x) as i32;
    let d1: i32 = (B.y < A.min.y) as i32;
    let d2: i32 = (B.x > A.max.x) as i32;
    let d3: i32 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(
    mut A: c2Ray,
    mut B: c2Capsule,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0., y: 0. },
        y: c2v { x: 0., y: 0. },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    let out_ref = out.as_deref_mut().unwrap();
    out_ref.n = c2Norm(cap_n);
    out_ref.t = 0.0f32;

    if c2AABBtoPoint(capsule_bb, yAp) != 0 {
        return 1;
    } else {
        let capsule_a = c2Circle { p: B.a, r: B.r };
        let capsule_b = c2Circle { p: B.b, r: B.r };
        if c2CircleToPoint(capsule_a, A.p) != 0 {
            return 1;
        } else if c2CircleToPoint(capsule_b, A.p) != 0 {
            return 1;
        }
    }

    if yAe.x * yAp.x < 0.0f32 || yAe.x.abs().min(yAp.x.abs()) < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0f32 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            let out_ref = out.as_deref_mut().unwrap();
            out_ref.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
            out_ref.t = t * A.t;
            return 1;
        }
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        C2_TYPE_CIRCLE => c2RaytoCircle(A, *(B as *const c2Circle), out.as_deref_mut()),
        C2_TYPE_AABB => c2RaytoAABB(A, *(B as *const c2AABB), out.as_deref_mut()),
        C2_TYPE_CAPSULE => c2RaytoCapsule(A, *(B as *const c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn gen_ray(
    mut cast1: Option<&mut c2Raycast>,
    mut cast2: Option<&mut c2Raycast>,
    mut cast3: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut cap_a_x: f32,
    mut cap_a_y: f32,
    mut cap_b_x: f32,
    mut cap_b_y: f32,
    mut cap_r: f32,
    mut bb_min_x: f32,
    mut bb_min_y: f32,
    mut bb_max_x: f32,
    mut bb_max_y: f32,
) -> i32 {
    let mut hit: i32 = 0;

    let mp = c2V(mp_x, mp_y);

    let mut ray = c2Ray {
        p: c2v { x: 0., y: 0. },
        d: c2v { x: 0., y: 0. },
        t: 0.,
    };
    ray.p = c2V(r_p_x, r_p_y);
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    let mut c = c2Circle {
        p: c2v { x: 0., y: 0. },
        r: 0.,
    };
    c.p = c2V(c_p_x, c_p_y);
    c.r = c_r;

    hit += c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast1.as_deref_mut(),
    );

    let mut cap = c2Capsule {
        a: c2v { x: 0., y: 0. },
        b: c2v { x: 0., y: 0. },
        r: 0.,
    };
    cap.a = c2V(cap_a_x, cap_a_y);
    cap.b = c2V(cap_b_x, cap_b_y);
    cap.r = cap_r;

    hit += c2CastRay(
        ray,
        &raw const cap as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        cast2.as_deref_mut(),
    ) << 1;

    let mut bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    bb.min = c2V(bb_min_x, bb_min_y);
    bb.max = c2V(bb_max_x, bb_max_y);

    hit += c2CastRay(
        ray,
        &raw const bb as *const std::ffi::c_void,
        C2_TYPE_AABB,
        cast3.as_deref_mut(),
    ) << 2;

    hit
}
```

**Entity:** c2Raycast output parameter (Option<&mut c2Raycast>)

**States:** OutProvided, OutMissing

**Transitions:**
- OutMissing -> panic via out.as_deref_mut().unwrap() on hit/computation paths

**Evidence:** fn c2RaytoCircle(..., out: Option<&mut c2Raycast>) -> i32: `let out = out.as_deref_mut().unwrap();` executed when `t >= 0 && t <= A.t` (hit path); fn c2RaytoAABB(..., out: Option<&mut c2Raycast>) -> i32: `let out = out.as_deref_mut().unwrap();` executed after `hit != 0` (hit path); fn c2RaytoCapsule(..., out: Option<&mut c2Raycast>) -> i32: `let out_ref = out.as_deref_mut().unwrap();` executed unconditionally before early returns; later uses additional unwraps; fn c2CastRay(..., out: Option<&mut c2Raycast>) -> i32: forwards `out.as_deref_mut()` into the above functions, preserving the possibility of `None` reaching an `unwrap()`

**Implementation:** Split APIs into two variants: (1) `*_hit(ray, shape) -> Option<c2Raycast>` returning the computed hit record by value, and/or (2) `*_into(ray, shape, out: &mut c2Raycast) -> bool/i32` where output is required. Use distinct functions or a wrapper type like `struct Out<'a>(&'a mut c2Raycast);` to make the capability explicit and eliminate `Option`+`unwrap`.

---

### 2. c2Capsule geometric validity invariant (non-negative radius; non-degenerate endpoints)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Capsule likely represents a capsule shape defined by endpoints a and b and radius r. There are implicit validity requirements not enforced by the type system: (1) radius r should be non-negative (and usually finite), and (2) the segment endpoints should not be degenerate in ways the algorithms don’t expect (commonly a != b, or at least that downstream code can handle the 'circle' degeneracy when a == b). As written, any f32 (including negative, NaN, or infinity) can be stored in r, and any values can be stored in a/b, so invalid shapes can be constructed and passed around unchecked.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Ray, 1 free function(s); struct c2m; struct c2Circle; 1 free function(s)


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
- Invalid -> Valid via constructing/validating inputs (not present in this snippet)

**Evidence:** pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 } — unconstrained f32 radius and unconstrained endpoints allow invalid capsules; #[repr(C)] and #[derive(Copy, Clone)] suggest this is a plain data/FFI geometry type with no constructors enforcing invariants

**Implementation:** Introduce a validated wrapper for radius (e.g., struct Radius(f32) with TryFrom<f32> ensuring r.is_finite() && r >= 0.0). Optionally provide a smart constructor for capsules: impl c2Capsule { fn try_new(a: c2v, b: c2v, r: Radius) -> Result<Self, Error> { ... } } or a separate ValidCapsule newtype that can only be obtained via validation; keep the repr(C) raw type for FFI and convert into the validated type at boundaries.

---

### 1. c2Ray validity invariant (direction normalized + nonnegative finite ray length)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Ray is a plain data struct with no constructor enforcing geometric validity. In typical raycasting math, the direction `d` is expected to be normalized (or at least non-zero), and the maximum distance/parameter `t` is expected to be finite and nonnegative. These constraints are not enforced by the type system: users can set `d` to the zero vector or NaN/Inf components, and can set `t` to negative/NaN/Inf, which may break downstream raycast computations or produce incorrect results. The code relies on callers to uphold these preconditions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle; 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Ray {
    pub p: c2v,
    pub d: c2v,
    pub t: f32,
}

```

**Entity:** c2Ray

**States:** ValidRay, InvalidRay

**Transitions:**
- InvalidRay -> ValidRay via (not present here) a validated constructor/normalization step

**Evidence:** struct c2Ray fields: `pub d: c2v` is an unconstrained direction vector (can be zero/NaN/Inf); struct c2Ray fields: `pub t: f32` is unconstrained (can be negative/NaN/Inf); `pub` fields allow constructing arbitrary, potentially invalid rays without checks

**Implementation:** Keep the raw FFI layout as `#[repr(C)] struct c2RayRaw { p: c2v, d: c2v, t: f32 }` and expose a safe wrapper `struct Ray { raw: c2RayRaw }` constructed via `Ray::new(p, dir, max_t)` that (1) rejects/handles zero-length dir, (2) normalizes dir, (3) enforces `max_t.is_finite() && max_t >= 0.0`. Alternatively use newtypes like `UnitVec2(c2v)` and `NonNegFiniteF32(f32)` for `d` and `t`.

---

### 5. Geometric parameter validity invariants (normalized directions, non-negative radii, ordered AABB mins/maxes, non-degenerate segments)

**Location**: `/data/test_case/lib.rs:1-449`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The algorithms assume various numeric invariants (non-zero lengths, non-negative radii, consistent min/max ordering) but accept raw `f32` fields. Violating these can cause division by zero, NaNs, or nonsensical results. These invariants are not enforced by the type system; they are implicit in how helper functions are used.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Raycast {
    pub t: f32,
    pub n: c2v,
}

pub type C2_TYPE = u32;
pub const C2_TYPE_CAPSULE: C2_TYPE = 2;
pub const C2_TYPE_AABB: C2_TYPE = 1;
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Ray {
    pub p: c2v,
    pub d: c2v,
    pub t: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2m {
    pub x: c2v,
    pub y: c2v,
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
pub(crate) fn c2Dot(a: c2v, b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2Absv(a: c2v) -> c2v {
    c2V(a.x.abs(), a.y.abs())
}

pub(crate) fn c2RaytoCircle(mut A: c2Ray, mut B: c2Circle, mut out: Option<&mut c2Raycast>) -> i32 {
    let p = B.p;
    let m = c2Sub(A.p, p);
    let c = c2Dot(m, m) - B.r * B.r;
    let b = c2Dot(m, A.d);
    let disc = b * b - c;
    if disc < 0.0f32 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0f32 && t <= A.t {
        let out = out.as_deref_mut().unwrap();
        out.t = t;
        let impact = c2Add(A.p, c2Mulvs(A.d, t));
        out.n = c2Norm(c2Sub(impact, p));
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0f32 {
        0.0f32
    } else if da * db > 0.0f32 {
        1.0f32
    } else {
        let d = da - db;
        if d != 0.0f32 { da / d } else { 0.0f32 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0f32 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0f32, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0f32, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0f32, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0f32, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0f32, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0f32, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0f32, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0f32, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0: i32 = (t0 <= 1.0f32) as i32;
    let hit1: i32 = (t1 <= 1.0f32) as i32;
    let hit2: i32 = (t2 <= 1.0f32) as i32;
    let hit3: i32 = (t3 <= 1.0f32) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    let out = out.as_deref_mut().unwrap();
    if t0 >= t1 && t0 >= t2 && t0 >= t3 {
        out.t = t0 * A.t;
        out.n = c2V(-1.0f32, 0.0f32);
    } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
        out.t = t1 * A.t;
        out.n = c2V(1.0f32, 0.0f32);
    } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
        out.t = t2 * A.t;
        out.n = c2V(0.0f32, -1.0f32);
    } else {
        out.t = t3 * A.t;
        out.n = c2V(0.0f32, 1.0f32);
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

#[inline]
pub(crate) fn c2MulmvT(a: c2m, b: c2v) -> c2v {
    c2V(a.x.x * b.x + a.x.y * b.y, a.y.x * b.x + a.y.y * b.y)
}

pub(crate) fn c2AABBtoPoint(mut A: c2AABB, mut B: c2v) -> i32 {
    let d0: i32 = (B.x < A.min.x) as i32;
    let d1: i32 = (B.y < A.min.y) as i32;
    let d2: i32 = (B.x > A.max.x) as i32;
    let d3: i32 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(
    mut A: c2Ray,
    mut B: c2Capsule,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0., y: 0. },
        y: c2v { x: 0., y: 0. },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    let out_ref = out.as_deref_mut().unwrap();
    out_ref.n = c2Norm(cap_n);
    out_ref.t = 0.0f32;

    if c2AABBtoPoint(capsule_bb, yAp) != 0 {
        return 1;
    } else {
        let capsule_a = c2Circle { p: B.a, r: B.r };
        let capsule_b = c2Circle { p: B.b, r: B.r };
        if c2CircleToPoint(capsule_a, A.p) != 0 {
            return 1;
        } else if c2CircleToPoint(capsule_b, A.p) != 0 {
            return 1;
        }
    }

    if yAe.x * yAp.x < 0.0f32 || yAe.x.abs().min(yAp.x.abs()) < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0f32 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            let out_ref = out.as_deref_mut().unwrap();
            out_ref.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
            out_ref.t = t * A.t;
            return 1;
        }
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        C2_TYPE_CIRCLE => c2RaytoCircle(A, *(B as *const c2Circle), out.as_deref_mut()),
        C2_TYPE_AABB => c2RaytoAABB(A, *(B as *const c2AABB), out.as_deref_mut()),
        C2_TYPE_CAPSULE => c2RaytoCapsule(A, *(B as *const c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn gen_ray(
    mut cast1: Option<&mut c2Raycast>,
    mut cast2: Option<&mut c2Raycast>,
    mut cast3: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut cap_a_x: f32,
    mut cap_a_y: f32,
    mut cap_b_x: f32,
    mut cap_b_y: f32,
    mut cap_r: f32,
    mut bb_min_x: f32,
    mut bb_min_y: f32,
    mut bb_max_x: f32,
    mut bb_max_y: f32,
) -> i32 {
    let mut hit: i32 = 0;

    let mp = c2V(mp_x, mp_y);

    let mut ray = c2Ray {
        p: c2v { x: 0., y: 0. },
        d: c2v { x: 0., y: 0. },
        t: 0.,
    };
    ray.p = c2V(r_p_x, r_p_y);
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    let mut c = c2Circle {
        p: c2v { x: 0., y: 0. },
        r: 0.,
    };
    c.p = c2V(c_p_x, c_p_y);
    c.r = c_r;

    hit += c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast1.as_deref_mut(),
    );

    let mut cap = c2Capsule {
        a: c2v { x: 0., y: 0. },
        b: c2v { x: 0., y: 0. },
        r: 0.,
    };
    cap.a = c2V(cap_a_x, cap_a_y);
    cap.b = c2V(cap_b_x, cap_b_y);
    cap.r = cap_r;

    hit += c2CastRay(
        ray,
        &raw const cap as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        cast2.as_deref_mut(),
    ) << 1;

    let mut bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    bb.min = c2V(bb_min_x, bb_min_y);
    bb.max = c2V(bb_max_x, bb_max_y);

    hit += c2CastRay(
        ray,
        &raw const bb as *const std::ffi::c_void,
        C2_TYPE_AABB,
        cast3.as_deref_mut(),
    ) << 2;

    hit
}
```

**Entity:** Geometric primitives (c2Ray, c2Circle, c2Capsule, c2AABB) numeric validity

**States:** ValidGeometry, InvalidGeometry

**Transitions:**
- InvalidGeometry -> NaN/inf propagation or incorrect hit tests via normalization/division

**Evidence:** fn c2Norm(a: c2v) -> c2v: `c2Div(a, c2Len(a))` divides by length without checking for zero (requires non-zero vector); gen_ray: `ray.d = c2Norm(c2Sub(mp, ray.p));` requires `mp != ray.p` to avoid normalizing a zero vector; c2RaytoCapsule: `M.y = c2Norm(c2Sub(B.b, B.a));` requires capsule segment `B.b != B.a` (non-degenerate) to avoid zero-length normalization; c2RaytoCircle/c2CircleToPoint: use `B.r * B.r` and `A.r * A.r` implicitly assuming radius is non-negative and finite; c2RaytoAABB/c2AABBtoAABB/c2AABBtoPoint: comparisons assume `A.min <= A.max` componentwise; no enforcement when constructing `bb` in gen_ray from raw inputs

**Implementation:** Introduce validated wrappers: `struct UnitVec(c2v)` (constructed via `try_new` returning Option/Result), `struct Radius(f32)` enforcing `>= 0` and finite, and `struct Aabb { min: c2v, max: c2v }` with constructor that sorts/validates. Use these in safe entrypoints (keep raw `repr(C)` structs for FFI) so internal algorithms can assume invariants without unchecked division/NaN risk.

---

## Protocol Invariants

### 4. Void-pointer + manual tag protocol (typeB must match B's actual pointee type and be non-null)

**Location**: `/data/test_case/lib.rs:1-449`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `c2CastRay` implements a manual tagged-union over `*const c_void` plus a numeric `typeB`. Correctness requires that (a) `B` points to a valid instance of the type indicated by `typeB`, properly aligned and alive for the call, and (b) `typeB` is one of the known constants. None of this is enforced by the type system; a mismatch triggers UB due to dereferencing `B as *const T`, and an unknown tag panics.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2AABB, 1 free function(s); struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Raycast {
    pub t: f32,
    pub n: c2v,
}

pub type C2_TYPE = u32;
pub const C2_TYPE_CAPSULE: C2_TYPE = 2;
pub const C2_TYPE_AABB: C2_TYPE = 1;
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Ray {
    pub p: c2v,
    pub d: c2v,
    pub t: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2m {
    pub x: c2v,
    pub y: c2v,
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
pub(crate) fn c2Dot(a: c2v, b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(a: c2v, b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2Absv(a: c2v) -> c2v {
    c2V(a.x.abs(), a.y.abs())
}

pub(crate) fn c2RaytoCircle(mut A: c2Ray, mut B: c2Circle, mut out: Option<&mut c2Raycast>) -> i32 {
    let p = B.p;
    let m = c2Sub(A.p, p);
    let c = c2Dot(m, m) - B.r * B.r;
    let b = c2Dot(m, A.d);
    let disc = b * b - c;
    if disc < 0.0f32 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0f32 && t <= A.t {
        let out = out.as_deref_mut().unwrap();
        out.t = t;
        let impact = c2Add(A.p, c2Mulvs(A.d, t));
        out.n = c2Norm(c2Sub(impact, p));
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0f32 {
        0.0f32
    } else if da * db > 0.0f32 {
        1.0f32
    } else {
        let d = da - db;
        if d != 0.0f32 { da / d } else { 0.0f32 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0f32 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0f32, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0f32, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0f32, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0f32, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0f32, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0f32, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0f32, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0f32, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0: i32 = (t0 <= 1.0f32) as i32;
    let hit1: i32 = (t1 <= 1.0f32) as i32;
    let hit2: i32 = (t2 <= 1.0f32) as i32;
    let hit3: i32 = (t3 <= 1.0f32) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    let out = out.as_deref_mut().unwrap();
    if t0 >= t1 && t0 >= t2 && t0 >= t3 {
        out.t = t0 * A.t;
        out.n = c2V(-1.0f32, 0.0f32);
    } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
        out.t = t1 * A.t;
        out.n = c2V(1.0f32, 0.0f32);
    } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
        out.t = t2 * A.t;
        out.n = c2V(0.0f32, -1.0f32);
    } else {
        out.t = t3 * A.t;
        out.n = c2V(0.0f32, 1.0f32);
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

#[inline]
pub(crate) fn c2MulmvT(a: c2m, b: c2v) -> c2v {
    c2V(a.x.x * b.x + a.x.y * b.y, a.y.x * b.x + a.y.y * b.y)
}

pub(crate) fn c2AABBtoPoint(mut A: c2AABB, mut B: c2v) -> i32 {
    let d0: i32 = (B.x < A.min.x) as i32;
    let d1: i32 = (B.y < A.min.y) as i32;
    let d2: i32 = (B.x > A.max.x) as i32;
    let d3: i32 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(
    mut A: c2Ray,
    mut B: c2Capsule,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0., y: 0. },
        y: c2v { x: 0., y: 0. },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    let out_ref = out.as_deref_mut().unwrap();
    out_ref.n = c2Norm(cap_n);
    out_ref.t = 0.0f32;

    if c2AABBtoPoint(capsule_bb, yAp) != 0 {
        return 1;
    } else {
        let capsule_a = c2Circle { p: B.a, r: B.r };
        let capsule_b = c2Circle { p: B.b, r: B.r };
        if c2CircleToPoint(capsule_a, A.p) != 0 {
            return 1;
        } else if c2CircleToPoint(capsule_b, A.p) != 0 {
            return 1;
        }
    }

    if yAe.x * yAp.x < 0.0f32 || yAe.x.abs().min(yAp.x.abs()) < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0f32 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0f32 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            let out_ref = out.as_deref_mut().unwrap();
            out_ref.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
            out_ref.t = t * A.t;
            return 1;
        }
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        C2_TYPE_CIRCLE => c2RaytoCircle(A, *(B as *const c2Circle), out.as_deref_mut()),
        C2_TYPE_AABB => c2RaytoAABB(A, *(B as *const c2AABB), out.as_deref_mut()),
        C2_TYPE_CAPSULE => c2RaytoCapsule(A, *(B as *const c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn gen_ray(
    mut cast1: Option<&mut c2Raycast>,
    mut cast2: Option<&mut c2Raycast>,
    mut cast3: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut cap_a_x: f32,
    mut cap_a_y: f32,
    mut cap_b_x: f32,
    mut cap_b_y: f32,
    mut cap_r: f32,
    mut bb_min_x: f32,
    mut bb_min_y: f32,
    mut bb_max_x: f32,
    mut bb_max_y: f32,
) -> i32 {
    let mut hit: i32 = 0;

    let mp = c2V(mp_x, mp_y);

    let mut ray = c2Ray {
        p: c2v { x: 0., y: 0. },
        d: c2v { x: 0., y: 0. },
        t: 0.,
    };
    ray.p = c2V(r_p_x, r_p_y);
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    let mut c = c2Circle {
        p: c2v { x: 0., y: 0. },
        r: 0.,
    };
    c.p = c2V(c_p_x, c_p_y);
    c.r = c_r;

    hit += c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast1.as_deref_mut(),
    );

    let mut cap = c2Capsule {
        a: c2v { x: 0., y: 0. },
        b: c2v { x: 0., y: 0. },
        r: 0.,
    };
    cap.a = c2V(cap_a_x, cap_a_y);
    cap.b = c2V(cap_b_x, cap_b_y);
    cap.r = cap_r;

    hit += c2CastRay(
        ray,
        &raw const cap as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        cast2.as_deref_mut(),
    ) << 1;

    let mut bb = c2AABB {
        min: c2v { x: 0., y: 0. },
        max: c2v { x: 0., y: 0. },
    };
    bb.min = c2V(bb_min_x, bb_min_y);
    bb.max = c2V(bb_max_x, bb_max_y);

    hit += c2CastRay(
        ray,
        &raw const bb as *const std::ffi::c_void,
        C2_TYPE_AABB,
        cast3.as_deref_mut(),
    ) << 2;

    hit
}
```

**Entity:** c2CastRay (B: *const c_void, typeB: C2_TYPE)

**States:** ValidTaggedPointer, MismatchedTagOrInvalidPointer

**Transitions:**
- ValidTaggedPointer -> dispatched cast via match(typeB)
- MismatchedTagOrInvalidPointer -> UB via `*(B as *const c2Circle/c2AABB/c2Capsule)`
- UnknownTag -> panic via `_ => panic!()`

**Evidence:** pub(crate) unsafe fn c2CastRay(A: c2Ray, B: *const c_void, typeB: C2_TYPE, ...): signature encodes 'tag + void*' protocol; match typeB { C2_TYPE_CIRCLE => c2RaytoCircle(A, *(B as *const c2Circle), ...), ... }: unchecked reinterpret casts and dereference; _ => panic!() in c2CastRay: unknown `typeB` is a runtime failure mode; const C2_TYPE_*: C2_TYPE = 0/1/2: tag values are integers rather than a Rust enum carrying the data

**Implementation:** Replace `(ptr, tag)` with a typed enum: `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB), Capsule(&'a c2Capsule) }` and `fn cast_ray(ray: c2Ray, shape: ShapeRef<'_>, out: &mut c2Raycast) -> i32`. If FFI requires `void*`, keep the unsafe extern entrypoint but immediately convert into the safe enum after validating the tag, and avoid deref on unknown tags.

---

