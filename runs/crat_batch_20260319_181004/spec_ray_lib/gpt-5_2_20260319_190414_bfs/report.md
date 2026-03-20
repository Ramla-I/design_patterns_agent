# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 3
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 4. AABB canonical-form invariant (min <= max per axis)

**Location**: `/data/test_case/lib.rs:1-401`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Many AABB operations implicitly assume `min.x <= max.x` and `min.y <= max.y` (a canonical box). The struct `c2AABB` does not enforce this, and functions like `c2AABBtoAABB` and `c2AABBtoPoint` compare against `min`/`max` assuming they bound an interval. If callers provide an inverted box, the tests can yield incorrect results. Some code constructs canonical AABBs explicitly (e.g., in `c2RaytoAABB` it uses componentwise min/max of endpoints), but other call sites can pass arbitrary `c2AABB` values.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2Circle; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
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
    if disc < 0.0 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0 && t <= A.t {
        if let Some(out) = out.as_deref_mut() {
            out.t = t;
            let impact = c2Add(A.p, c2Mulvs(A.d, t));
            out.n = c2Norm(c2Sub(impact, p));
        }
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0 {
        0.0
    } else if da * db > 0.0 {
        1.0
    } else {
        let d = da - db;
        if d != 0.0 { da / d } else { 0.0 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0 = (t0 <= 1.0) as i32;
    let hit1 = (t1 <= 1.0) as i32;
    let hit2 = (t2 <= 1.0) as i32;
    let hit3 = (t3 <= 1.0) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    if let Some(out) = out.as_deref_mut() {
        if t0 >= t1 && t0 >= t2 && t0 >= t3 {
            out.t = t0 * A.t;
            out.n = c2V(-1.0, 0.0);
        } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
            out.t = t1 * A.t;
            out.n = c2V(1.0, 0.0);
        } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
            out.t = t2 * A.t;
            out.n = c2V(0.0, -1.0);
        } else {
            out.t = t3 * A.t;
            out.n = c2V(0.0, 1.0);
        }
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
    let d0 = (B.x < A.min.x) as i32;
    let d1 = (B.y < A.min.y) as i32;
    let d2 = (B.x > A.max.x) as i32;
    let d3 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(mut A: c2Ray, mut B: c2Capsule, mut out: Option<&mut c2Raycast>) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0.0, y: 0.0 },
        y: c2v { x: 0.0, y: 0.0 },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    capsule_bb.min = c2V(-B.r, 0.0);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0;
    }

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

    let crosses_x_axis = yAe.x * yAp.x < 0.0;
    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if crosses_x_axis || min_abs_x < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
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
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn spec_ray(
    mut cast: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
) -> i32 {
    let mp = c2V(mp_x, mp_y);

    let c = c2Circle {
        p: c2V(c_p_x, c_p_y),
        r: c_r,
    };

    let mut ray = c2Ray {
        p: c2V(r_p_x, r_p_y),
        d: c2v { x: 0.0, y: 0.0 },
        t: 0.0,
    };
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast.as_deref_mut(),
    )
}
```

**Entity:** c2AABB

**States:** CanonicalAABB, NonCanonicalAABB

**Transitions:**
- NonCanonicalAABB -> CanonicalAABB via canonicalization (min/max swap) at construction time

**Evidence:** c2AABB fields are raw: `min: c2v, max: c2v` with no validation; c2AABBtoAABB uses comparisons like `B.max.x < A.min.x` and `A.max.x < B.min.x`, which assume min/max ordering; c2AABBtoPoint checks `B.x < A.min.x` and `B.x > A.max.x`, which is only meaningful when min <= max; c2RaytoAABB constructs `a_box.min = c2Minv(p0, p1); a_box.max = c2Maxv(p0, p1);`, indicating the expected canonical form

**Implementation:** Provide `struct Aabb { min: c2v, max: c2v }` with `fn new(a: c2v, b: c2v) -> Self { min = min(a,b); max = max(a,b) }` and keep fields private. Expose `as_raw()` for FFI if needed. Update APIs to take `Aabb` (canonical) instead of `c2AABB` where possible.

---

### 2. Ray validity invariants (normalized direction; non-negative length/parameterization)

**Location**: `/data/test_case/lib.rs:1-401`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The geometry routines assume a ray is in a mathematically valid form: direction `d` should be a unit vector (or at least non-zero), and the ray extent/parameter `t` is treated as a non-negative distance/extent along `d`. These constraints are not encoded in the `c2Ray` type (all fields are plain `f32`), so callers can pass a zero vector for `d`, NaNs/infinities, or a negative `t`, which can trigger divisions by zero and incorrect intersection results. The code relies on callers to construct rays correctly (e.g., `spec_ray` normalizes and computes `t`), but other call sites (including `c2CastRay`) accept arbitrary `c2Ray` values.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2Circle; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
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
    if disc < 0.0 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0 && t <= A.t {
        if let Some(out) = out.as_deref_mut() {
            out.t = t;
            let impact = c2Add(A.p, c2Mulvs(A.d, t));
            out.n = c2Norm(c2Sub(impact, p));
        }
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0 {
        0.0
    } else if da * db > 0.0 {
        1.0
    } else {
        let d = da - db;
        if d != 0.0 { da / d } else { 0.0 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0 = (t0 <= 1.0) as i32;
    let hit1 = (t1 <= 1.0) as i32;
    let hit2 = (t2 <= 1.0) as i32;
    let hit3 = (t3 <= 1.0) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    if let Some(out) = out.as_deref_mut() {
        if t0 >= t1 && t0 >= t2 && t0 >= t3 {
            out.t = t0 * A.t;
            out.n = c2V(-1.0, 0.0);
        } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
            out.t = t1 * A.t;
            out.n = c2V(1.0, 0.0);
        } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
            out.t = t2 * A.t;
            out.n = c2V(0.0, -1.0);
        } else {
            out.t = t3 * A.t;
            out.n = c2V(0.0, 1.0);
        }
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
    let d0 = (B.x < A.min.x) as i32;
    let d1 = (B.y < A.min.y) as i32;
    let d2 = (B.x > A.max.x) as i32;
    let d3 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(mut A: c2Ray, mut B: c2Capsule, mut out: Option<&mut c2Raycast>) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0.0, y: 0.0 },
        y: c2v { x: 0.0, y: 0.0 },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    capsule_bb.min = c2V(-B.r, 0.0);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0;
    }

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

    let crosses_x_axis = yAe.x * yAp.x < 0.0;
    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if crosses_x_axis || min_abs_x < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
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
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn spec_ray(
    mut cast: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
) -> i32 {
    let mp = c2V(mp_x, mp_y);

    let c = c2Circle {
        p: c2V(c_p_x, c_p_y),
        r: c_r,
    };

    let mut ray = c2Ray {
        p: c2V(r_p_x, r_p_y),
        d: c2v { x: 0.0, y: 0.0 },
        t: 0.0,
    };
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast.as_deref_mut(),
    )
}
```

**Entity:** c2Ray

**States:** ValidRay, InvalidRay

**Transitions:**
- InvalidRay -> ValidRay via normalization + extent computation (as done in spec_ray)

**Evidence:** c2Ray has fields `d: c2v` and `t: f32` with no type-level restrictions; fn c2Norm(a: c2v) -> c2v computes `c2Div(a, c2Len(a))` (division by length; zero-length vector invalid); spec_ray: `ray.d = c2Norm(c2Sub(mp, ray.p));` constructs a normalized direction; spec_ray: `ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);` computes an extent used as the allowed max in intersection tests; c2RaytoCircle: checks `t >= 0.0 && t <= A.t`, implicitly assuming `A.t` is a meaningful bound (typically non-negative); c2RaytoAABB: computes `p1 = A.p + A.d * A.t`, implicitly assuming `A.t` parameterizes the segment/extent along `d`

**Implementation:** Introduce constructors that validate invariants: `struct UnitVec2(c2v)` (ensuring non-zero and normalized) and `struct NonNegF32(f32)` (or `NonNegative<f32>`). Then define `struct Ray { p: c2v, d: UnitVec2, t: NonNegF32 }` and only expose `Ray::new(p, dir, t)` / `Ray::from_points(p, target)` returning `Option/Result` if invalid.

---

### 1. c2Capsule geometric validity (finite radius, distinct endpoints)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2Capsule likely represents a capsule defined by endpoints (a,b) and radius r. For most geometry operations, implicit preconditions typically apply: r should be finite and non-negative, and (often) endpoints should be distinct (or if coincident, the shape degenerates to a circle). The current type allows constructing values with negative/NaN/infinite r or otherwise-degenerate geometry, and there is no type-level enforcement that a capsule is valid for downstream collision/raycast computations.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2Circle; struct c2Ray, 1 free function(s); struct c2m; struct c2AABB, 1 free function(s); 1 free function(s)


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
- Invalid -> Valid via validated constructor (not present in this snippet)
- Valid -> Invalid via direct field mutation/struct literal construction (possible because fields are pub)

**Evidence:** pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 } — radius is an unconstrained f32 and can be negative/NaN/inf; pub fields a/b/r — callers can create/mutate capsules without validation; #[derive(Copy, Clone)] — easy to duplicate and pass around invalid values without any checks

**Implementation:** Make fields private and provide constructors returning Result/Option. Use a newtype for the radius like `struct NonNegativeF32(f32);` (or `FiniteF32` + `NonNegative`) and store `r: NonNegativeF32`. Optionally provide a separate `DegenerateCapsule`/`Circle` type or encode the distinct-endpoints requirement by constructing from `Segment` that enforces `a != b` (if desired).

---

## Protocol Invariants

### 3. Tagged-void* dispatch protocol (type tag must match pointed-to shape)

**Location**: `/data/test_case/lib.rs:1-401`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The `c2CastRay` API encodes a dynamic sum type using `(typeB: C2_TYPE, B: *const c_void)`. Correctness requires that `typeB` precisely matches the concrete object stored at `B` (Circle/AABB/Capsule) and that `B` points to valid memory of that type for the duration of the call. This invariant is enforced only by `unsafe` + convention. If the tag mismatches, the function will reinterpret-cast `B` to the wrong type and dereference it, causing undefined behavior. Additionally, unknown tags cause a panic. This could be represented as a typed enum at the Rust level to remove the tag/pointer coupling from callers.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 16 free function(s); struct c2Raycast, 5 free function(s); struct c2Circle; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_CIRCLE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
    pub r: f32,
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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
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
    if disc < 0.0 {
        return 0;
    }

    let t = -b - disc.sqrt();
    if t >= 0.0 && t <= A.t {
        if let Some(out) = out.as_deref_mut() {
            out.t = t;
            let impact = c2Add(A.p, c2Mulvs(A.d, t));
            out.n = c2Norm(c2Sub(impact, p));
        }
        return 1;
    }

    0
}

pub(crate) fn c2AABBtoAABB(mut A: c2AABB, mut B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

#[inline]
fn c2SignedDistPointToPlane_OneDimensional(p: f32, n: f32, d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(da: f32, db: f32) -> f32 {
    if da < 0.0 {
        0.0
    } else if da * db > 0.0 {
        1.0
    } else {
        let d = da - db;
        if d != 0.0 { da / d } else { 0.0 }
    }
}

pub(crate) fn c2RaytoAABB(mut A: c2Ray, mut B: c2AABB, mut out: Option<&mut c2Raycast>) -> i32 {
    let p0 = A.p;
    let p1 = c2Add(A.p, c2Mulvs(A.d, A.t));

    let mut a_box = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    a_box.min = c2Minv(p0, p1);
    a_box.max = c2Maxv(p0, p1);

    if c2AABBtoAABB(a_box, B) == 0 {
        return 0;
    }

    let ab = c2Sub(p1, p0);
    let n = c2Skew(ab);
    let abs_n = c2Absv(n);

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5);

    let p0_to_center = c2Sub(p0, center_of_b_box);
    let d = c2Dot(n, p0_to_center).abs() - c2Dot(abs_n, half_extents);
    if d > 0.0 {
        return 0;
    }

    let da0 = c2SignedDistPointToPlane_OneDimensional(p0.x, -1.0, B.min.x);
    let db0 = c2SignedDistPointToPlane_OneDimensional(p1.x, -1.0, B.min.x);
    let da1 = c2SignedDistPointToPlane_OneDimensional(p0.x, 1.0, B.max.x);
    let db1 = c2SignedDistPointToPlane_OneDimensional(p1.x, 1.0, B.max.x);
    let da2 = c2SignedDistPointToPlane_OneDimensional(p0.y, -1.0, B.min.y);
    let db2 = c2SignedDistPointToPlane_OneDimensional(p1.y, -1.0, B.min.y);
    let da3 = c2SignedDistPointToPlane_OneDimensional(p0.y, 1.0, B.max.y);
    let db3 = c2SignedDistPointToPlane_OneDimensional(p1.y, 1.0, B.max.y);

    let mut t0 = c2RayToPlane_OneDimensional(da0, db0);
    let mut t1 = c2RayToPlane_OneDimensional(da1, db1);
    let mut t2 = c2RayToPlane_OneDimensional(da2, db2);
    let mut t3 = c2RayToPlane_OneDimensional(da3, db3);

    let hit0 = (t0 <= 1.0) as i32;
    let hit1 = (t1 <= 1.0) as i32;
    let hit2 = (t2 <= 1.0) as i32;
    let hit3 = (t3 <= 1.0) as i32;

    let hit = hit0 | hit1 | hit2 | hit3;
    if hit == 0 {
        return 0;
    }

    t0 *= hit0 as f32;
    t1 *= hit1 as f32;
    t2 *= hit2 as f32;
    t3 *= hit3 as f32;

    if let Some(out) = out.as_deref_mut() {
        if t0 >= t1 && t0 >= t2 && t0 >= t3 {
            out.t = t0 * A.t;
            out.n = c2V(-1.0, 0.0);
        } else if t1 >= t0 && t1 >= t2 && t1 >= t3 {
            out.t = t1 * A.t;
            out.n = c2V(1.0, 0.0);
        } else if t2 >= t0 && t2 >= t1 && t2 >= t3 {
            out.t = t2 * A.t;
            out.n = c2V(0.0, -1.0);
        } else {
            out.t = t3 * A.t;
            out.n = c2V(0.0, 1.0);
        }
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
    let d0 = (B.x < A.min.x) as i32;
    let d1 = (B.y < A.min.y) as i32;
    let d2 = (B.x > A.max.x) as i32;
    let d3 = (B.y > A.max.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) fn c2CircleToPoint(mut A: c2Circle, mut B: c2v) -> i32 {
    let n = c2Sub(A.p, B);
    let d2 = c2Dot(n, n);
    (d2 < A.r * A.r) as i32
}

pub(crate) fn c2RaytoCapsule(mut A: c2Ray, mut B: c2Capsule, mut out: Option<&mut c2Raycast>) -> i32 {
    let mut M = c2m {
        x: c2v { x: 0.0, y: 0.0 },
        y: c2v { x: 0.0, y: 0.0 },
    };
    M.y = c2Norm(c2Sub(B.b, B.a));
    M.x = c2CCW90(M.y);

    let cap_n = c2Sub(B.b, B.a);
    let yBb = c2MulmvT(M, cap_n);
    let yAp = c2MulmvT(M, c2Sub(A.p, B.a));
    let yAd = c2MulmvT(M, A.d);
    let yAe = c2Add(yAp, c2Mulvs(yAd, A.t));

    let mut capsule_bb = c2AABB {
        min: c2v { x: 0.0, y: 0.0 },
        max: c2v { x: 0.0, y: 0.0 },
    };
    capsule_bb.min = c2V(-B.r, 0.0);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0;
    }

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

    let crosses_x_axis = yAe.x * yAp.x < 0.0;
    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if crosses_x_axis || min_abs_x < B.r {
        let Ca = c2Circle { p: B.a, r: B.r };
        let Cb = c2Circle { p: B.b, r: B.r };

        if yAp.x.abs() < B.r {
            if yAp.y < 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            } else {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }
        } else {
            let c = if yAp.x > 0.0 { B.r } else { -B.r };
            let d = yAe.x - yAp.x;
            let t = (c - yAp.x) / d;
            let y = yAp.y + (yAe.y - yAp.y) * t;

            if y <= 0.0 {
                return c2RaytoCircle(A, Ca, out.as_deref_mut());
            }
            if y >= yBb.y {
                return c2RaytoCircle(A, Cb, out.as_deref_mut());
            }

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
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
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        _ => panic!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn spec_ray(
    mut cast: Option<&mut c2Raycast>,
    mut mp_x: f32,
    mut mp_y: f32,
    mut c_p_x: f32,
    mut c_p_y: f32,
    mut c_r: f32,
    mut r_p_x: f32,
    mut r_p_y: f32,
) -> i32 {
    let mp = c2V(mp_x, mp_y);

    let c = c2Circle {
        p: c2V(c_p_x, c_p_y),
        r: c_r,
    };

    let mut ray = c2Ray {
        p: c2V(r_p_x, r_p_y),
        d: c2v { x: 0.0, y: 0.0 },
        t: 0.0,
    };
    ray.d = c2Norm(c2Sub(mp, ray.p));
    ray.t = c2Dot(mp, ray.d) - c2Dot(ray.p, ray.d);

    c2CastRay(
        ray,
        &raw const c as *const std::ffi::c_void,
        C2_TYPE_CIRCLE,
        cast.as_deref_mut(),
    )
}
```

**Entity:** c2CastRay / (typeB, B pointer) pair

**States:** TagMatchesPayload, TagMismatchOrNull

**Transitions:**
- TagMismatchOrNull -> TagMatchesPayload via constructing a typed shape enum/union before calling c2CastRay

**Evidence:** unsafe fn c2CastRay(A: c2Ray, B: *const c_void, typeB: C2_TYPE, ...); c2CastRay match arms dereference based on tag: `*(B as *mut c2Circle)`, `*(B as *mut c2AABB)`, `*(B as *mut c2Capsule)`; c2CastRay: `_ => panic!()` on unknown `typeB` indicates a required finite tag set and a precondition on `typeB`; spec_ray calls `c2CastRay(ray, &raw const c as *const c_void, C2_TYPE_CIRCLE, ...)`, demonstrating the intended 'tag + pointer' coupling

**Implementation:** Replace `(typeB, *const c_void)` with a Rust enum: `enum ShapeRef<'a> { Circle(&'a c2Circle), AABB(&'a c2AABB), Capsule(&'a c2Capsule) }`. Then `fn cast_ray(ray: c2Ray, shape: ShapeRef<'_>, out: Option<&mut c2Raycast>) -> i32` dispatches without casts. If the C ABI must remain, add a safe wrapper that accepts `ShapeRef` and internally calls the unsafe C-like function.

---

