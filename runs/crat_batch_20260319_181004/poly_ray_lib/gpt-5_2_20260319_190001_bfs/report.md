# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 7
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 5
- **Protocol**: 2
- **Modules analyzed**: 2

## Precondition Invariants

### 5. c2Poly validity invariant (count bounds + initialized prefix of verts/norms)

**Location**: `/data/test_case/lib.rs:1-528`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Several algorithms treat `c2Poly.count` as the number of active vertices/normals and then index `verts[i]`/`norms[i]` for `i in 0..count`. This requires (1) `count` to be within the fixed backing array capacity (<= 8 and non-negative) and (2) the first `count` entries of `verts` and `norms` to be initialized consistently. These are semantic/validity requirements but the type system permits any `i32` count and does not tie `count` to which array elements are valid, so out-of-bounds indexing and nonsensical geometry are possible if a malformed `c2Poly` is passed in.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_POLY: C2_TYPE = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) fn c2V(mut x: f32, mut y: f32) -> c2v {
    c2v { x, y }
}

#[inline]
pub(crate) fn c2Dot(mut a: c2v, mut b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(mut a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, mut b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, mut b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, mut b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(mut a: c2v, mut b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(mut a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(mut a: c2v) -> c2v {
    c2v { x: -a.y, y: a.x }
}

#[inline]
pub(crate) fn c2Absv(mut a: c2v) -> c2v {
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
fn c2SignedDistPointToPlane_OneDimensional(mut p: f32, mut n: f32, mut d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(mut da: f32, mut db: f32) -> f32 {
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

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let dist = c2Dot(n, c2Sub(p0, center_of_b_box)).abs();
    let d = dist - c2Dot(abs_n, half_extents);
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

    let hit0 = (t0 <= 1.0f32) as i32;
    let hit1 = (t1 <= 1.0f32) as i32;
    let hit2 = (t2 <= 1.0f32) as i32;
    let hit3 = (t3 <= 1.0f32) as i32;

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
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(mut a: c2v) -> c2v {
    c2v { x: a.y, y: -a.x }
}

#[inline]
pub(crate) fn c2MulmvT(mut a: c2m, mut b: c2v) -> c2v {
    c2v {
        x: a.x.x * b.x + a.x.y * b.y,
        y: a.y.x * b.x + a.y.y * b.y,
    }
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
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0f32;
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

    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if yAe.x * yAp.x < 0.0f32 || min_abs_x < B.r {
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

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
            return 1;
        }
    }

    0
}

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0f32, s: 0.0f32 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0f32, 0.0f32),
        r: c2RotIdentity(),
    }
}

#[inline]
pub(crate) fn c2Mulrv(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulrvT(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulxvT(mut a: c2x, mut b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

pub(crate) fn c2RaytoPoly(
    mut A: c2Ray,
    mut B: Option<&c2Poly>,
    mut bx_ptr: Option<&c2x>,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let p = c2MulxvT(bx, A.p);
    let d = c2MulrvT(bx.r, A.d);

    let mut lo = 0.0f32;
    let mut hi = A.t;
    let mut index: i32 = !0;

    let poly = match B {
        Some(p) => p,
        None => return 0,
    };

    for i in 0..(poly.count as usize) {
        let num = c2Dot(poly.norms[i], c2Sub(poly.verts[i], p));
        let den = c2Dot(poly.norms[i], d);

        if den == 0.0f32 && num < 0.0f32 {
            return 0;
        } else if den < 0.0f32 && num < lo * den {
            lo = num / den;
            index = i as i32;
        } else if den > 0.0f32 && num < hi * den {
            hi = num / den;
        }

        if hi < lo {
            return 0;
        }
    }

    if index != !0 {
        if let Some(out) = out.as_deref_mut() {
            out.t = lo;
            out.n = c2Mulrv(bx.r, poly.norms[index as usize]);
        }
        return 1;
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut bx: Option<&c2x>,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        3 => c2RaytoPoly(A, (B as *const c2Poly).as_ref(), bx.as_deref(), out.as_deref_mut()),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn poly_ray(mut cast1: Option<&mut c2Raycast>, mut cast2: Option<&mut c2Raycast>) -> i32 {
    let mut hit: i32 = 0;

    let mut p: c2Poly = c2Poly {
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
        norms: [c2v { x: 0.0, y: 0.0 }; 8],
    };

    p.verts[0] = c2V(0.875f32, -11.5f32);
    p.verts[1] = c2V(0.875f32, 11.5f32);
    p.verts[2] = c2V(-0.875f32, 11.5f32);
    p.verts[3] = c2V(-0.875f32, -11.5f32);

    p.norms[0] = c2V(1.0f32, 0.0f32);
    p.norms[1] = c2V(0.0f32, 1.0f32);
    p.norms[2] = c2V(-1.0f32, 0.0f32);
    p.norms[3] = c2V(0.0f32, -1.0f32);

    p.count = 4;

    let ray0 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 1.0f32, y: 0.0f32 },
        t: 4.0f32,
    };

    let ray1 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 0.0f32, y: -1.0f32 },
        t: 4.0f32,
    };

    hit += c2CastRay(
        ray0,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast1.as_deref_mut(),
    );
    hit += c2CastRay(
        ray1,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast2.as_deref_mut(),
    ) << 1;

    hit
}
```

**Entity:** c2Poly

**States:** ValidPoly (0 <= count <= 8, verts/norms[0..count) initialized), InvalidPoly (count out of range or verts/norms missing/garbage)

**Transitions:**
- InvalidPoly -> ValidPoly via constructing with checked count and filling verts/norms
- ValidPoly -> InvalidPoly via mutating count without keeping arrays consistent (possible today due to public fields)

**Evidence:** struct c2Poly { count: i32, verts: [c2v; 8], norms: [c2v; 8] } uses an unvalidated runtime count to describe an array prefix; c2RaytoPoly: `for i in 0..(poly.count as usize)` then indexes `poly.norms[i]` and `poly.verts[i]` (requires count <= 8 and count >= 0); poly_ray: manually sets `p.verts[0..4]`, `p.norms[0..4]`, then sets `p.count = 4` (demonstrates the intended protocol: fill prefix then set count)

**Implementation:** Make `c2Poly` fields private and expose a constructor like `C2Poly::new(verts: [c2v; N], norms: [c2v; N]) -> C2Poly` using const generics (N<=8) or store `verts: ArrayVec<c2v, 8>`/`norms: ArrayVec<c2v, 8>` so the length is tracked by the container, eliminating the separate `count` field. Alternatively use `struct ValidCount(u8)` with `TryFrom<i32>` and only allow `count: ValidCount`.

---

### 4. c2Circle geometric validity (non-negative / finite radius, finite center)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: A circle used in geometry/collision code typically assumes its center point and radius are valid numeric values, and that the radius is non-negative (often strictly positive) and finite. As defined, `c2Circle` allows constructing circles with `r < 0.0`, `r == NaN`, `r == +/-INF`, and potentially non-finite components in `p` (depending on `c2v`). These invalid states are not prevented by the type system and would have to be handled by runtime checks in algorithms that consume `c2Circle` (not shown in this snippet).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2AABB, 1 free function(s); 1 free function(s)


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
- Invalid -> Valid via validation/constructor (not present in snippet)

**Evidence:** pub struct c2Circle { pub p: c2v, pub r: f32 } — `r: f32` permits negative/NaN/Inf values and there is no constructor/validation shown; #[repr(C)] and #[derive(Copy, Clone)] indicate this is a plain data/FFI-friendly type, so invariants are likely relied upon externally rather than enforced here

**Implementation:** Introduce a validated radius type, e.g. `struct Radius(f32);` with `TryFrom<f32>` enforcing `is_finite() && v >= 0.0` (or `> 0.0`), and/or provide `impl c2Circle { pub fn new(p: c2v, r: Radius) -> Self }`. If `c2v` can be non-finite, similarly add a `FiniteVec2` newtype or a `ValidatedCircle` wrapper used by algorithms that require validity.

---

### 1. c2Poly vertex-count validity (count bounds & initialized prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Poly encodes a variable-length polygon using a fixed-size array. The runtime field `count` determines how many entries in `verts` and `norms` are logically part of the polygon. Implicitly, `count` must stay within the array capacity (0..=8), and only the first `count` elements of `verts`/`norms` are considered initialized/meaningful. The type system does not prevent constructing a c2Poly with `count` > 8, negative `count`, or with mismatched/garbage data in the arrays beyond the logical prefix; any code that iterates based on `count` would rely on this invariant being upheld externally.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle; struct c2AABB, 1 free function(s); 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
}

```

**Entity:** c2Poly

**States:** Valid (0..=8, verts/norms meaningful up to count), Invalid (count out of range or unused slots uninitialized/garbage)

**Transitions:**
- Invalid -> Valid via construction/normalization that clamps/validates count and fills verts/norms[0..count)

**Evidence:** line 8: `pub count: i32` is a runtime length for the polygon; line 9: `pub verts: [c2v; 8]` fixed capacity implies `count` must be <= 8; line 10: `pub norms: [c2v; 8]` must correspond to the same logical prefix as `verts`; line 4: `#[repr(C)]` suggests FFI-style layout where validity is expected by convention, not enforced

**Implementation:** Make `count` a validated type like `struct VertCount(u8);` with `TryFrom<i32>` ensuring 0..=8, or redesign as `struct Poly { verts: ArrayVec<c2v, 8>, norms: ArrayVec<c2v, 8> }` so length is tracked by the container. If FFI requires the C layout, provide a safe constructor `c2Poly::new(verts: &[c2v], norms: &[c2v]) -> Result<c2Poly, Error>` that sets `count` and copies into arrays, and keep raw fields private.

---

### 2. c2Ray validity preconditions (normalized direction, non-negative ray length)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The raw fields of c2Ray implicitly represent a geometric ray that typical raycast/intersection routines assume is well-formed: the direction vector `d` is usually expected to be non-zero (often normalized), and the ray extent/maximum time `t` is expected to be non-negative (and typically finite). Because `c2Ray` is a plain `#[repr(C)]` POD with all public fields and `Copy`, any code can construct an 'invalid' ray (e.g., `d == 0`, `t < 0`, NaNs/Infs), and those preconditions are not enforced by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Capsule; struct c2m; struct c2Circle; struct c2AABB, 1 free function(s); 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Ray {
    pub p: c2v,
    pub d: c2v,
    pub t: f32,
}

```

**Entity:** c2Ray

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via validated construction/normalization (not present in this snippet)

**Evidence:** struct c2Ray { pub p: c2v, pub d: c2v, pub t: f32 } — all fields are public, allowing construction of potentially invalid rays; field `d: c2v` is a direction vector; by convention this must be non-zero / normalized for many ray algorithms, but no type-level guarantee exists; field `t: f32` is a scalar extent/time parameter; negative or non-finite values are representable and not prevented

**Implementation:** Make fields private and provide constructors like `c2Ray::new(p, d, t) -> Option<Self>` that reject invalid inputs; encode invariants with newtypes such as `UnitVec2(c2v)` (guaranteed normalized, non-zero) and `NonNegativeF32(f32)` (reject NaN/Inf and < 0). Expose getters returning these validated types to downstream algorithms.

---

### 3. c2Capsule geometric validity invariant (non-negative radius / non-degenerate endpoints)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Capsule is a plain data struct used to represent a capsule by two endpoints (a, b) and a radius (r). The type system does not enforce common geometric validity requirements: (1) radius should be non-negative (typically > 0), and (2) the segment endpoints should usually be distinct (a != b) unless a 'degenerate capsule' is intentionally allowed (which effectively becomes a circle/sphere). As written, any f32 value (including negative, NaN, +/-inf) can be stored in r, and a/b can be equal; any invariants are therefore implicit and must be upheld by convention or downstream runtime checks.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Ray, 1 free function(s); struct c2m; struct c2Circle; struct c2AABB, 1 free function(s); 1 free function(s)


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
- Invalid -> Valid via construction/validation (not present in snippet)

**Evidence:** pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 } — raw fields expose an unconstrained geometric representation; field `r: f32` allows negative values and NaN/inf, but capsule radius is typically required to be finite and >= 0; fields `a: c2v` and `b: c2v` encode a segment; many capsule algorithms assume a != b (non-zero segment length)

**Implementation:** Introduce a validated wrapper, e.g. `struct Radius(f32);` with `TryFrom<f32>` ensuring finite and >= 0 (or > 0). Optionally add `struct Segment { a: c2v, b: c2v }` with a constructor that rejects a == b (or models degeneracy explicitly with an enum `Capsule = DegenerateCircle{center,r} | SegmentCapsule{a,b,r}`). Then make `c2Capsule` store `Radius` (and possibly `Segment`) and provide fallible constructors instead of public fields.

---

## Protocol Invariants

### 6. Raycast output protocol (only valid when hit==1; otherwise contents undefined/stale)

**Location**: `/data/test_case/lib.rs:1-528`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Raycast functions take an optional out-parameter and return an `i32` hit flag. Callers must respect a protocol: only if the function returns 1 is the output (t, n) meaningful. The type system does not connect the return value to the initialization/validity of `out`, so misuse (reading `out` after a 0 return, or relying on previous values) is possible. This is especially visible because several functions early-return 0 without writing to `out`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_POLY: C2_TYPE = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) fn c2V(mut x: f32, mut y: f32) -> c2v {
    c2v { x, y }
}

#[inline]
pub(crate) fn c2Dot(mut a: c2v, mut b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(mut a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, mut b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, mut b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, mut b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(mut a: c2v, mut b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(mut a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(mut a: c2v) -> c2v {
    c2v { x: -a.y, y: a.x }
}

#[inline]
pub(crate) fn c2Absv(mut a: c2v) -> c2v {
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
fn c2SignedDistPointToPlane_OneDimensional(mut p: f32, mut n: f32, mut d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(mut da: f32, mut db: f32) -> f32 {
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

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let dist = c2Dot(n, c2Sub(p0, center_of_b_box)).abs();
    let d = dist - c2Dot(abs_n, half_extents);
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

    let hit0 = (t0 <= 1.0f32) as i32;
    let hit1 = (t1 <= 1.0f32) as i32;
    let hit2 = (t2 <= 1.0f32) as i32;
    let hit3 = (t3 <= 1.0f32) as i32;

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
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(mut a: c2v) -> c2v {
    c2v { x: a.y, y: -a.x }
}

#[inline]
pub(crate) fn c2MulmvT(mut a: c2m, mut b: c2v) -> c2v {
    c2v {
        x: a.x.x * b.x + a.x.y * b.y,
        y: a.y.x * b.x + a.y.y * b.y,
    }
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
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0f32;
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

    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if yAe.x * yAp.x < 0.0f32 || min_abs_x < B.r {
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

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
            return 1;
        }
    }

    0
}

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0f32, s: 0.0f32 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0f32, 0.0f32),
        r: c2RotIdentity(),
    }
}

#[inline]
pub(crate) fn c2Mulrv(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulrvT(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulxvT(mut a: c2x, mut b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

pub(crate) fn c2RaytoPoly(
    mut A: c2Ray,
    mut B: Option<&c2Poly>,
    mut bx_ptr: Option<&c2x>,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let p = c2MulxvT(bx, A.p);
    let d = c2MulrvT(bx.r, A.d);

    let mut lo = 0.0f32;
    let mut hi = A.t;
    let mut index: i32 = !0;

    let poly = match B {
        Some(p) => p,
        None => return 0,
    };

    for i in 0..(poly.count as usize) {
        let num = c2Dot(poly.norms[i], c2Sub(poly.verts[i], p));
        let den = c2Dot(poly.norms[i], d);

        if den == 0.0f32 && num < 0.0f32 {
            return 0;
        } else if den < 0.0f32 && num < lo * den {
            lo = num / den;
            index = i as i32;
        } else if den > 0.0f32 && num < hi * den {
            hi = num / den;
        }

        if hi < lo {
            return 0;
        }
    }

    if index != !0 {
        if let Some(out) = out.as_deref_mut() {
            out.t = lo;
            out.n = c2Mulrv(bx.r, poly.norms[index as usize]);
        }
        return 1;
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut bx: Option<&c2x>,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        3 => c2RaytoPoly(A, (B as *const c2Poly).as_ref(), bx.as_deref(), out.as_deref_mut()),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn poly_ray(mut cast1: Option<&mut c2Raycast>, mut cast2: Option<&mut c2Raycast>) -> i32 {
    let mut hit: i32 = 0;

    let mut p: c2Poly = c2Poly {
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
        norms: [c2v { x: 0.0, y: 0.0 }; 8],
    };

    p.verts[0] = c2V(0.875f32, -11.5f32);
    p.verts[1] = c2V(0.875f32, 11.5f32);
    p.verts[2] = c2V(-0.875f32, 11.5f32);
    p.verts[3] = c2V(-0.875f32, -11.5f32);

    p.norms[0] = c2V(1.0f32, 0.0f32);
    p.norms[1] = c2V(0.0f32, 1.0f32);
    p.norms[2] = c2V(-1.0f32, 0.0f32);
    p.norms[3] = c2V(0.0f32, -1.0f32);

    p.count = 4;

    let ray0 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 1.0f32, y: 0.0f32 },
        t: 4.0f32,
    };

    let ray1 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 0.0f32, y: -1.0f32 },
        t: 4.0f32,
    };

    hit += c2CastRay(
        ray0,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast1.as_deref_mut(),
    );
    hit += c2CastRay(
        ray1,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast2.as_deref_mut(),
    ) << 1;

    hit
}
```

**Entity:** c2Raycast (as used via out: Option<&mut c2Raycast>)

**States:** NoHit (function returns 0; out must be ignored), Hit (function returns 1; out.t/out.n written and meaningful)

**Transitions:**
- NoHit -> Hit via calling c2RaytoCircle/c2RaytoAABB/c2RaytoCapsule/c2RaytoPoly and receiving return value 1
- Hit -> NoHit via receiving return value 0 on a later call (out may remain unchanged)

**Evidence:** c2RaytoCircle: returns 0 on `disc < 0` and on t outside [0, A.t] without writing to `out`; writes `out.t`/`out.n` only inside `if t >= 0 && t <= A.t { if let Some(out) ... } return 1;`; c2RaytoAABB: multiple early `return 0;` paths before `if let Some(out) { ... }` write; c2RaytoPoly: returns 0 for `B: None` and several separating-axis failures; writes `out` only when `index != !0` and returns 1; poly_ray: accumulates hit flags from `c2CastRay(..., cast1)` / `c2CastRay(..., cast2)` demonstrating the 'check integer return' convention

**Implementation:** Replace `(i32, Option<&mut c2Raycast>)` with `Option<c2Raycast>` (or `Result<c2Raycast, NoHit>`) so the presence of a value encodes the Hit/NoHit state. If avoiding allocation/copies, return `bool` plus a dedicated `HitRef<'a>(&'a mut c2Raycast)` capability type that can only be constructed on hit, or use `MaybeUninit<c2Raycast>` in the signature and return `Option<InitializedRaycast>` wrapper.

---

### 7. Tagged-union protocol for shape pointer (typeB must match pointee type and bx usage)

**Location**: `/data/test_case/lib.rs:1-528`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2CastRay implements a manual tagged union: the `typeB` integer selects which concrete shape type `B: *const c_void` is cast to and dereferenced as. Correctness relies on an implicit invariant that `typeB` matches the actual allocation/type behind `B` (Circle/AABB/Capsule/Poly), and that `B` is valid for reads for that type. This is not enforced by the type system (hence `unsafe`), and passing a mismatched tag/pointer yields immediate UB. Additionally, only the Poly path consumes the optional transform `bx` (`c2RaytoPoly(..., bx, ...)`), implying another implicit rule: `bx` is meaningful only for polygon casts in this API.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 19 free function(s); struct c2Raycast, 6 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Poly; struct c2Ray, 1 free function(s); struct c2Capsule; struct c2m; struct c2Circle; struct c2AABB, 1 free function(s)

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
pub const C2_TYPE_POLY: C2_TYPE = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) fn c2V(mut x: f32, mut y: f32) -> c2v {
    c2v { x, y }
}

#[inline]
pub(crate) fn c2Dot(mut a: c2v, mut b: c2v) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline]
pub(crate) fn c2Len(mut a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, mut b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Sub(mut a: c2v, mut b: c2v) -> c2v {
    a.x -= b.x;
    a.y -= b.y;
    a
}

#[inline]
pub(crate) fn c2Mulvs(mut a: c2v, mut b: f32) -> c2v {
    a.x *= b;
    a.y *= b;
    a
}

#[inline]
pub(crate) fn c2Div(mut a: c2v, mut b: f32) -> c2v {
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(mut a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Minv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub(crate) fn c2Maxv(mut a: c2v, mut b: c2v) -> c2v {
    c2V(a.x.max(b.x), a.y.max(b.y))
}

#[inline]
pub(crate) fn c2Skew(mut a: c2v) -> c2v {
    c2v { x: -a.y, y: a.x }
}

#[inline]
pub(crate) fn c2Absv(mut a: c2v) -> c2v {
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
fn c2SignedDistPointToPlane_OneDimensional(mut p: f32, mut n: f32, mut d: f32) -> f32 {
    p * n - d * n
}

#[inline]
fn c2RayToPlane_OneDimensional(mut da: f32, mut db: f32) -> f32 {
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

    let half_extents = c2Mulvs(c2Sub(B.max, B.min), 0.5f32);
    let center_of_b_box = c2Mulvs(c2Add(B.min, B.max), 0.5f32);

    let dist = c2Dot(n, c2Sub(p0, center_of_b_box)).abs();
    let d = dist - c2Dot(abs_n, half_extents);
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

    let hit0 = (t0 <= 1.0f32) as i32;
    let hit1 = (t1 <= 1.0f32) as i32;
    let hit2 = (t2 <= 1.0f32) as i32;
    let hit3 = (t3 <= 1.0f32) as i32;

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
    }

    1
}

#[inline]
pub(crate) fn c2CCW90(mut a: c2v) -> c2v {
    c2v { x: a.y, y: -a.x }
}

#[inline]
pub(crate) fn c2MulmvT(mut a: c2m, mut b: c2v) -> c2v {
    c2v {
        x: a.x.x * b.x + a.x.y * b.y,
        y: a.y.x * b.x + a.y.y * b.y,
    }
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
    capsule_bb.min = c2V(-B.r, 0.0f32);
    capsule_bb.max = c2V(B.r, yBb.y);

    if let Some(out) = out.as_deref_mut() {
        out.n = c2Norm(cap_n);
        out.t = 0.0f32;
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

    let min_abs_x = yAe.x.abs().min(yAp.x.abs());
    if yAe.x * yAp.x < 0.0f32 || min_abs_x < B.r {
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

            if let Some(out) = out.as_deref_mut() {
                out.n = if c > 0.0f32 { M.x } else { c2Skew(M.y) };
                out.t = t * A.t;
            }
            return 1;
        }
    }

    0
}

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0f32, s: 0.0f32 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0f32, 0.0f32),
        r: c2RotIdentity(),
    }
}

#[inline]
pub(crate) fn c2Mulrv(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulrvT(mut a: c2r, mut b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2MulxvT(mut a: c2x, mut b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

pub(crate) fn c2RaytoPoly(
    mut A: c2Ray,
    mut B: Option<&c2Poly>,
    mut bx_ptr: Option<&c2x>,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let p = c2MulxvT(bx, A.p);
    let d = c2MulrvT(bx.r, A.d);

    let mut lo = 0.0f32;
    let mut hi = A.t;
    let mut index: i32 = !0;

    let poly = match B {
        Some(p) => p,
        None => return 0,
    };

    for i in 0..(poly.count as usize) {
        let num = c2Dot(poly.norms[i], c2Sub(poly.verts[i], p));
        let den = c2Dot(poly.norms[i], d);

        if den == 0.0f32 && num < 0.0f32 {
            return 0;
        } else if den < 0.0f32 && num < lo * den {
            lo = num / den;
            index = i as i32;
        } else if den > 0.0f32 && num < hi * den {
            hi = num / den;
        }

        if hi < lo {
            return 0;
        }
    }

    if index != !0 {
        if let Some(out) = out.as_deref_mut() {
            out.t = lo;
            out.n = c2Mulrv(bx.r, poly.norms[index as usize]);
        }
        return 1;
    }

    0
}

pub(crate) unsafe fn c2CastRay(
    mut A: c2Ray,
    mut B: *const std::ffi::c_void,
    mut bx: Option<&c2x>,
    mut typeB: C2_TYPE,
    mut out: Option<&mut c2Raycast>,
) -> i32 {
    match typeB {
        0 => c2RaytoCircle(A, *(B as *mut c2Circle), out.as_deref_mut()),
        1 => c2RaytoAABB(A, *(B as *mut c2AABB), out.as_deref_mut()),
        2 => c2RaytoCapsule(A, *(B as *mut c2Capsule), out.as_deref_mut()),
        3 => c2RaytoPoly(A, (B as *const c2Poly).as_ref(), bx.as_deref(), out.as_deref_mut()),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn poly_ray(mut cast1: Option<&mut c2Raycast>, mut cast2: Option<&mut c2Raycast>) -> i32 {
    let mut hit: i32 = 0;

    let mut p: c2Poly = c2Poly {
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
        norms: [c2v { x: 0.0, y: 0.0 }; 8],
    };

    p.verts[0] = c2V(0.875f32, -11.5f32);
    p.verts[1] = c2V(0.875f32, 11.5f32);
    p.verts[2] = c2V(-0.875f32, 11.5f32);
    p.verts[3] = c2V(-0.875f32, -11.5f32);

    p.norms[0] = c2V(1.0f32, 0.0f32);
    p.norms[1] = c2V(0.0f32, 1.0f32);
    p.norms[2] = c2V(-1.0f32, 0.0f32);
    p.norms[3] = c2V(0.0f32, -1.0f32);

    p.count = 4;

    let ray0 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 1.0f32, y: 0.0f32 },
        t: 4.0f32,
    };

    let ray1 = c2Ray {
        p: c2v {
            x: -3.869416f32,
            y: 13.0693407f32,
        },
        d: c2v { x: 0.0f32, y: -1.0f32 },
        t: 4.0f32,
    };

    hit += c2CastRay(
        ray0,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast1.as_deref_mut(),
    );
    hit += c2CastRay(
        ray1,
        &raw const p as *const std::ffi::c_void,
        None,
        C2_TYPE_POLY,
        cast2.as_deref_mut(),
    ) << 1;

    hit
}
```

**Entity:** c2CastRay (typeB: C2_TYPE, B: *const c_void)

**States:** WellTyped (typeB tag matches the concrete object behind B), IllTyped (typeB/tag mismatch or invalid pointer)

**Transitions:**
- IllTyped -> WellTyped via constructing a typed handle that couples the pointer with its tag
- WellTyped -> IllTyped via changing tag without changing pointer (possible today because they are separate parameters)

**Evidence:** signature: `pub(crate) unsafe fn c2CastRay(A: c2Ray, B: *const c_void, bx: Option<&c2x>, typeB: C2_TYPE, out: Option<&mut c2Raycast>)` separates the tag from the pointer; match arms perform unchecked casts/dereferences: `*(B as *mut c2Circle)`, `*(B as *mut c2AABB)`, `*(B as *mut c2Capsule)`, and `(B as *const c2Poly).as_ref()`; poly_ray calls `c2CastRay(ray0, &raw const p as *const c_void, None, C2_TYPE_POLY, ...)` demonstrating the intended coupling between `typeB` and the actual pointee type

**Implementation:** Introduce a safe enum `ShapeRef<'a> { Circle(&'a c2Circle), AABB(&'a c2AABB), Capsule(&'a c2Capsule), Poly(&'a c2Poly, Option<&'a c2x>) }` (or separate wrapper types per shape) and implement `cast_ray(ray, shape, out)`. This couples the tag and reference at the type level, eliminating `*const c_void` and `typeB` mismatches and allowing `bx` only where relevant.

---

