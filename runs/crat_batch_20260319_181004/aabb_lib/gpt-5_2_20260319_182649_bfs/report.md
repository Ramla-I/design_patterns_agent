# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 9
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 6
- **Protocol**: 2
- **Modules analyzed**: 2

## State Machine Invariants

### 8. c2Simplex state machine (by vertex count 0/1/2/3 with matching initialized vertices and div)

**Location**: `/data/test_case/lib.rs:1-705`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2Simplex uses `count: i32` to determine which subset of vertices (a/b/c) are initialized and which operations are valid. Multiple functions branch on `s.count` and assume certain fields are meaningful (e.g., `c2D` assumes a/b are valid when count==2; `c2Witness` divides by `s.div` and assumes `div` matches the barycentric weights stored in `u`). These invariants are maintained by convention and runtime branching, not enforced by the type system; it is possible to call helpers with a simplex whose `count` does not match the initialized vertices or whose `div` is zero, leading to wrong math or division-by-zero.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s)

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
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2GJKCache {
    pub metric: f32,
    pub count: i32,
    pub iA: [i32; 3],
    pub iB: [i32; 3],
    pub div: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2sv {
    pub sA: c2v,
    pub sB: c2v,
    pub p: c2v,
    pub u: f32,
    pub iA: i32,
    pub iB: i32,
}

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

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0, s: 0.0 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0, 0.0),
        r: c2RotIdentity(),
    }
}

pub(crate) fn c2BBVerts(out: &mut [c2v], bb: Option<&c2AABB>) {
    let bb = bb.unwrap();
    out[0] = bb.min;
    out[1] = c2V(bb.max.x, bb.min.y);
    out[2] = bb.max;
    out[3] = c2V(bb.min.x, bb.max.y);
}

pub(crate) unsafe fn c2MakeProxy(
    shape: *const std::ffi::c_void,
    type_0: C2_TYPE,
    mut p: Option<&mut c2Proxy>,
) {
    let p = p.as_deref_mut().unwrap();
    match type_0 {
        C2_TYPE_CIRCLE => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        C2_TYPE_AABB => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        C2_TYPE_CAPSULE => {
            let c = (shape as *const c2Capsule).as_ref().unwrap();
            p.radius = c.r;
            p.count = 2;
            p.verts[0] = c.a;
            p.verts[1] = c.b;
        }
        _ => {}
    }
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Det2(a: c2v, b: c2v) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn c2GJKSimplexMetric(s: Option<&c2Simplex>) -> f32 {
    let s = s.unwrap();
    match s.count {
        2 => c2Len(c2Sub(s.b.p, s.a.p)),
        3 => c2Det2(c2Sub(s.b.p, s.a.p), c2Sub(s.c.p, s.a.p)),
        _ => 0.0,
    }
}

#[inline]
pub(crate) fn c2Mulrv(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Mulxv(a: c2x, b: c2v) -> c2v {
    c2Add(c2Mulrv(a.r, b), a.p)
}

pub(crate) fn c22(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    if v <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if u <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else {
        s.a.u = u;
        s.b.u = v;
        s.div = u + v;
        s.count = 2;
    }
}

pub(crate) fn c23(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;
    let c = s.c.p;

    let uAB = c2Dot(b, c2Sub(b, a));
    let vAB = c2Dot(a, c2Sub(a, b));
    let uBC = c2Dot(c, c2Sub(c, b));
    let vBC = c2Dot(b, c2Sub(b, c));
    let uCA = c2Dot(a, c2Sub(a, c));
    let vCA = c2Dot(c, c2Sub(c, a));

    let area = c2Det2(c2Sub(b, a), c2Sub(c, a));
    let uABC = c2Det2(b, c) * area;
    let vABC = c2Det2(c, a) * area;
    let wABC = c2Det2(a, b) * area;

    if vAB <= 0.0 && uCA <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB <= 0.0 && vBC <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uBC <= 0.0 && vCA <= 0.0 {
        s.a = s.c;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB > 0.0 && vAB > 0.0 && wABC <= 0.0 {
        s.a.u = uAB;
        s.b.u = vAB;
        s.div = uAB + vAB;
        s.count = 2;
    } else if uBC > 0.0 && vBC > 0.0 && uABC <= 0.0 {
        s.a = s.b;
        s.b = s.c;
        s.a.u = uBC;
        s.b.u = vBC;
        s.div = uBC + vBC;
        s.count = 2;
    } else if uCA > 0.0 && vCA > 0.0 && vABC <= 0.0 {
        s.b = s.a;
        s.a = s.c;
        s.a.u = uCA;
        s.b.u = vCA;
        s.div = uCA + vCA;
        s.count = 2;
    } else {
        s.a.u = uABC;
        s.b.u = vABC;
        s.c.u = wABC;
        s.div = uABC + vABC + wABC;
        s.count = 3;
    }
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

pub(crate) fn c2D(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    match s.count {
        1 => c2Neg(s.a.p),
        2 => {
            let ab = c2Sub(s.b.p, s.a.p);
            if c2Det2(ab, c2Neg(s.a.p)) > 0.0 {
                c2Skew(ab)
            } else {
                c2CCW90(ab)
            }
        }
        _ => c2V(0.0, 0.0),
    }
}

pub(crate) fn c2Support(verts: &[c2v], count: i32, d: c2v) -> i32 {
    let count = count.max(0) as usize;
    let mut imax: usize = 0;
    let mut dmax = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i;
            dmax = dot;
        }
    }
    imax as i32
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0 / s.div;

    match s.count {
        1 => {
            *a.as_deref_mut().unwrap() = s.a.sA;
            *b.as_deref_mut().unwrap() = s.a.sB;
        }
        2 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sA, den * s.a.u),
                c2Mulvs(s.b.sA, den * s.b.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sB, den * s.a.u),
                c2Mulvs(s.b.sB, den * s.b.u),
            );
        }
        3 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sA, den * s.a.u),
                    c2Mulvs(s.b.sA, den * s.b.u),
                ),
                c2Mulvs(s.c.sA, den * s.c.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sB, den * s.a.u),
                    c2Mulvs(s.b.sB, den * s.b.u),
                ),
                c2Mulvs(s.c.sB, den * s.c.u),
            );
        }
        _ => {
            *a.unwrap() = c2V(0.0, 0.0);
            *b.unwrap() = c2V(0.0, 0.0);
        }
    }
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0 / s.div;
    match s.count {
        1 => s.a.p,
        2 => c2Add(
            c2Mulvs(s.a.p, den * s.a.u),
            c2Mulvs(s.b.p, den * s.b.u),
        ),
        _ => c2V(0.0, 0.0),
    }
}

#[inline]
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

pub(crate) unsafe fn c2GJK(
    A: *const std::ffi::c_void,
    typeA: C2_TYPE,
    ax_ptr: Option<&c2x>,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
    bx_ptr: Option<&c2x>,
    outA: Option<&mut c2v>,
    outB: Option<&mut c2v>,
    use_radius: i32,
    iterations: Option<&mut i32>,
    mut cache: Option<&mut c2GJKCache>,
) -> f32 {
    let ax = ax_ptr.copied().unwrap_or_else(c2xIdentity);
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let mut pA = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    let mut pB = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    c2MakeProxy(A, typeA, Some(&mut pA));
    c2MakeProxy(B, typeB, Some(&mut pB));

    let mut s = c2Simplex {
        a: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i).as_mut().unwrap();
                v.iA = iA;
                v.sA = sA;
                v.iB = iB;
                v.sB = sB;
                v.p = c2Sub(v.sB, v.sA);
                v.u = 0.0;
            }

            s.count = cache_ref.count;
            s.div = cache_ref.div;

            let metric_old = cache_ref.metric;
            let metric = c2GJKSimplexMetric(Some(&s));
            let min_metric = metric.min(metric_old);
            let max_metric = metric.max(metric_old);

            if !(min_metric < max_metric * 2.0 && metric < -1.0e8) {
                cache_was_read = 1;
            }
        }
    }

    if cache_was_read == 0 {
        s.a.iA = 0;
        s.a.iB = 0;
        s.a.sA = c2Mulxv(ax, pA.verts[0]);
        s.a.sB = c2Mulxv(bx, pB.verts[0]);
        s.a.p = c2Sub(s.a.sB, s.a.sA);
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    }

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).read();
            saveA[i as usize] = v.iA;
            saveB[i as usize] = v.iB;
        }

        match s.count {
            2 => c22(Some(&mut s)),
            3 => c23(Some(&mut s)),
            _ => {}
        }

        if s.count == 3 {
            hit = 1;
            break;
        }

        let p = c2L(Some(&s));
        let d1 = c2Dot(p, p);
        if d1 > d0 {
            break;
        }
        d0 = d1;

        let d = c2D(Some(&s));
        if c2Dot(d, d) < 1.192_092_9e-7_f32 * 1.192_092_9e-7_f32 {
            break;
        }

        let iA = c2Support(&pA.verts, pA.count, c2MulrvT(ax.r, c2Neg(d)));
        let sA = c2Mulxv(ax, pA.verts[iA as usize]);
        let iB = c2Support(&pB.verts, pB.count, c2MulrvT(bx.r, d));
        let sB = c2Mulxv(bx, pB.verts[iB as usize]);

        let v = verts.offset(s.count as isize).as_mut().unwrap();
        v.iA = iA;
        v.sA = sA;
        v.iB = iB;
        v.sB = sB;
        v.p = c2Sub(v.sB, v.sA);

        let mut dup = 0;
        for i in 0..save_count {
            if iA == saveA[i as usize] && iB == saveB[i as usize] {
                dup = 1;
                break;
            }
        }
        if dup != 0 {
            break;
        }

        s.count += 1;
        iter += 1;
    }

    let mut a = c2v { x: 0.0, y: 0.0 };
    let mut b = c2v { x: 0.0, y: 0.0 };
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        if dist > rA + rB && dist > 1.192_092_9e-7_f32 {
            dist -= rA + rB;
            let n = c2Norm(c2Sub(b, a));
            a = c2Add(a, c2Mulvs(n, rA));
            b = c2Sub(b, c2Mulvs(n, rB));
            if a.x == b.x && a.y == b.y {
                dist = 0.0;
            }
        } else {
            let p = c2Mulvs(c2Add(a, b), 0.5);
            a = p;
            b = p;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache_mut.iA[i as usize] = v.iA;
            cache_mut.iB[i as usize] = v.iB;
        }
        cache_mut.div = s.div;
    }

    if let Some(outA) = outA {
        *outA = a;
    }
    if let Some(outB) = outB {
        *outB = b;
    }
    if let Some(iterations) = iterations {
        *iterations = iter;
    }

    dist
}

pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) unsafe fn c2AABBtoCapsule(A: c2AABB, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_AABB,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) unsafe fn c2CapsuletoCapsule(A: c2Capsule, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(
// ... (truncated) ...
```

**Entity:** c2Simplex

**States:** Empty(count=0), Point(count=1), Segment(count=2), Triangle(count=3)

**Transitions:**
- Empty(count=0) -> Point(count=1) via c2GJK() initialization path setting s.a.*, s.div=1, s.count=1
- Point(count=1) -> Segment(count=2) via c22() setting weights and s.count=2
- Segment(count=2) -> Point(count=1) via c22() reducing to a single vertex when u<=0 or v<=0
- Segment(count=2) -> Triangle(count=3) via c23() setting weights and s.count=3
- Triangle(count=3) -> Segment(count=2) or Point(count=1) via c23() reductions depending on region tests

**Evidence:** struct c2Simplex { a,b,c,d: c2sv, div: f32, count: i32 } uses `count` as a runtime state discriminator; c2GJKSimplexMetric(): `match s.count { 2 => ..., 3 => ..., _ => 0.0 }` depends on count to select valid geometry; c22(): updates `s.count` to 1 or 2 and sets `s.div` accordingly; assumes inputs are a/b initialized (`let a = s.a.p; let b = s.b.p;`); c23(): updates `s.count` to 1/2/3 and rewrites vertices (e.g., `s.a = s.b;`, `s.b = s.c;`) depending on region; assumes a/b/c initialized (`let a = s.a.p; let b = s.b.p; let c = s.c.p;`); c2D(): `match s.count { 1 => ..., 2 => { let ab = c2Sub(s.b.p, s.a.p); ... }, _ => ... }` implies only certain fields are meaningful per state; c2Witness(): computes `let den = 1.0 / s.div;` and then uses weights depending on `s.count` (requires div!=0 and weights consistent with count); c2L(): also does `let den = 1.0 / s.div;` and only defines results for count 1/2

**Implementation:** Represent simplex as an enum or typestate: `enum Simplex { Point{a, div}, Segment{a,b,div}, Triangle{a,b,c,div} }` (or `Simplex<S>` with `S=Point/Segment/Triangle`). Make c22 take `Segment` and return `Point|Segment`, and c23 take `Triangle` and return `Point|Segment|Triangle`. Expose c2D/c2L/c2Witness only for the states where they are mathematically defined, eliminating reliance on `count` and preventing `div==0` by construction.

---

## Precondition Invariants

### 9. Shape-pointer/type-tag protocol (C2_TYPE must match actual pointed-to shape; proxy.count must be in 1/2/4)

**Location**: `/data/test_case/lib.rs:1-705`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: c2MakeProxy() takes a raw `*const c_void` plus a runtime type tag `C2_TYPE` and uses it to cast the pointer to a concrete shape type. Correctness requires that `type_0` matches the actual allocation behind `shape`, and that the pointer is non-null and properly aligned for that type; otherwise `.as_ref().unwrap()` will panic or result in UB if the pointer is invalid. The resulting c2Proxy also has an implicit invariant that `count` matches the number of initialized vertices (1 for circle, 2 for capsule, 4 for AABB) and that subsequent code (e.g., c2Support) only reads `verts[0..count]`. None of this is captured in types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s)

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
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2GJKCache {
    pub metric: f32,
    pub count: i32,
    pub iA: [i32; 3],
    pub iB: [i32; 3],
    pub div: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2sv {
    pub sA: c2v,
    pub sB: c2v,
    pub p: c2v,
    pub u: f32,
    pub iA: i32,
    pub iB: i32,
}

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

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0, s: 0.0 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0, 0.0),
        r: c2RotIdentity(),
    }
}

pub(crate) fn c2BBVerts(out: &mut [c2v], bb: Option<&c2AABB>) {
    let bb = bb.unwrap();
    out[0] = bb.min;
    out[1] = c2V(bb.max.x, bb.min.y);
    out[2] = bb.max;
    out[3] = c2V(bb.min.x, bb.max.y);
}

pub(crate) unsafe fn c2MakeProxy(
    shape: *const std::ffi::c_void,
    type_0: C2_TYPE,
    mut p: Option<&mut c2Proxy>,
) {
    let p = p.as_deref_mut().unwrap();
    match type_0 {
        C2_TYPE_CIRCLE => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        C2_TYPE_AABB => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        C2_TYPE_CAPSULE => {
            let c = (shape as *const c2Capsule).as_ref().unwrap();
            p.radius = c.r;
            p.count = 2;
            p.verts[0] = c.a;
            p.verts[1] = c.b;
        }
        _ => {}
    }
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Det2(a: c2v, b: c2v) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn c2GJKSimplexMetric(s: Option<&c2Simplex>) -> f32 {
    let s = s.unwrap();
    match s.count {
        2 => c2Len(c2Sub(s.b.p, s.a.p)),
        3 => c2Det2(c2Sub(s.b.p, s.a.p), c2Sub(s.c.p, s.a.p)),
        _ => 0.0,
    }
}

#[inline]
pub(crate) fn c2Mulrv(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Mulxv(a: c2x, b: c2v) -> c2v {
    c2Add(c2Mulrv(a.r, b), a.p)
}

pub(crate) fn c22(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    if v <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if u <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else {
        s.a.u = u;
        s.b.u = v;
        s.div = u + v;
        s.count = 2;
    }
}

pub(crate) fn c23(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;
    let c = s.c.p;

    let uAB = c2Dot(b, c2Sub(b, a));
    let vAB = c2Dot(a, c2Sub(a, b));
    let uBC = c2Dot(c, c2Sub(c, b));
    let vBC = c2Dot(b, c2Sub(b, c));
    let uCA = c2Dot(a, c2Sub(a, c));
    let vCA = c2Dot(c, c2Sub(c, a));

    let area = c2Det2(c2Sub(b, a), c2Sub(c, a));
    let uABC = c2Det2(b, c) * area;
    let vABC = c2Det2(c, a) * area;
    let wABC = c2Det2(a, b) * area;

    if vAB <= 0.0 && uCA <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB <= 0.0 && vBC <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uBC <= 0.0 && vCA <= 0.0 {
        s.a = s.c;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB > 0.0 && vAB > 0.0 && wABC <= 0.0 {
        s.a.u = uAB;
        s.b.u = vAB;
        s.div = uAB + vAB;
        s.count = 2;
    } else if uBC > 0.0 && vBC > 0.0 && uABC <= 0.0 {
        s.a = s.b;
        s.b = s.c;
        s.a.u = uBC;
        s.b.u = vBC;
        s.div = uBC + vBC;
        s.count = 2;
    } else if uCA > 0.0 && vCA > 0.0 && vABC <= 0.0 {
        s.b = s.a;
        s.a = s.c;
        s.a.u = uCA;
        s.b.u = vCA;
        s.div = uCA + vCA;
        s.count = 2;
    } else {
        s.a.u = uABC;
        s.b.u = vABC;
        s.c.u = wABC;
        s.div = uABC + vABC + wABC;
        s.count = 3;
    }
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

pub(crate) fn c2D(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    match s.count {
        1 => c2Neg(s.a.p),
        2 => {
            let ab = c2Sub(s.b.p, s.a.p);
            if c2Det2(ab, c2Neg(s.a.p)) > 0.0 {
                c2Skew(ab)
            } else {
                c2CCW90(ab)
            }
        }
        _ => c2V(0.0, 0.0),
    }
}

pub(crate) fn c2Support(verts: &[c2v], count: i32, d: c2v) -> i32 {
    let count = count.max(0) as usize;
    let mut imax: usize = 0;
    let mut dmax = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i;
            dmax = dot;
        }
    }
    imax as i32
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0 / s.div;

    match s.count {
        1 => {
            *a.as_deref_mut().unwrap() = s.a.sA;
            *b.as_deref_mut().unwrap() = s.a.sB;
        }
        2 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sA, den * s.a.u),
                c2Mulvs(s.b.sA, den * s.b.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sB, den * s.a.u),
                c2Mulvs(s.b.sB, den * s.b.u),
            );
        }
        3 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sA, den * s.a.u),
                    c2Mulvs(s.b.sA, den * s.b.u),
                ),
                c2Mulvs(s.c.sA, den * s.c.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sB, den * s.a.u),
                    c2Mulvs(s.b.sB, den * s.b.u),
                ),
                c2Mulvs(s.c.sB, den * s.c.u),
            );
        }
        _ => {
            *a.unwrap() = c2V(0.0, 0.0);
            *b.unwrap() = c2V(0.0, 0.0);
        }
    }
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0 / s.div;
    match s.count {
        1 => s.a.p,
        2 => c2Add(
            c2Mulvs(s.a.p, den * s.a.u),
            c2Mulvs(s.b.p, den * s.b.u),
        ),
        _ => c2V(0.0, 0.0),
    }
}

#[inline]
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

pub(crate) unsafe fn c2GJK(
    A: *const std::ffi::c_void,
    typeA: C2_TYPE,
    ax_ptr: Option<&c2x>,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
    bx_ptr: Option<&c2x>,
    outA: Option<&mut c2v>,
    outB: Option<&mut c2v>,
    use_radius: i32,
    iterations: Option<&mut i32>,
    mut cache: Option<&mut c2GJKCache>,
) -> f32 {
    let ax = ax_ptr.copied().unwrap_or_else(c2xIdentity);
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let mut pA = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    let mut pB = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    c2MakeProxy(A, typeA, Some(&mut pA));
    c2MakeProxy(B, typeB, Some(&mut pB));

    let mut s = c2Simplex {
        a: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i).as_mut().unwrap();
                v.iA = iA;
                v.sA = sA;
                v.iB = iB;
                v.sB = sB;
                v.p = c2Sub(v.sB, v.sA);
                v.u = 0.0;
            }

            s.count = cache_ref.count;
            s.div = cache_ref.div;

            let metric_old = cache_ref.metric;
            let metric = c2GJKSimplexMetric(Some(&s));
            let min_metric = metric.min(metric_old);
            let max_metric = metric.max(metric_old);

            if !(min_metric < max_metric * 2.0 && metric < -1.0e8) {
                cache_was_read = 1;
            }
        }
    }

    if cache_was_read == 0 {
        s.a.iA = 0;
        s.a.iB = 0;
        s.a.sA = c2Mulxv(ax, pA.verts[0]);
        s.a.sB = c2Mulxv(bx, pB.verts[0]);
        s.a.p = c2Sub(s.a.sB, s.a.sA);
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    }

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).read();
            saveA[i as usize] = v.iA;
            saveB[i as usize] = v.iB;
        }

        match s.count {
            2 => c22(Some(&mut s)),
            3 => c23(Some(&mut s)),
            _ => {}
        }

        if s.count == 3 {
            hit = 1;
            break;
        }

        let p = c2L(Some(&s));
        let d1 = c2Dot(p, p);
        if d1 > d0 {
            break;
        }
        d0 = d1;

        let d = c2D(Some(&s));
        if c2Dot(d, d) < 1.192_092_9e-7_f32 * 1.192_092_9e-7_f32 {
            break;
        }

        let iA = c2Support(&pA.verts, pA.count, c2MulrvT(ax.r, c2Neg(d)));
        let sA = c2Mulxv(ax, pA.verts[iA as usize]);
        let iB = c2Support(&pB.verts, pB.count, c2MulrvT(bx.r, d));
        let sB = c2Mulxv(bx, pB.verts[iB as usize]);

        let v = verts.offset(s.count as isize).as_mut().unwrap();
        v.iA = iA;
        v.sA = sA;
        v.iB = iB;
        v.sB = sB;
        v.p = c2Sub(v.sB, v.sA);

        let mut dup = 0;
        for i in 0..save_count {
            if iA == saveA[i as usize] && iB == saveB[i as usize] {
                dup = 1;
                break;
            }
        }
        if dup != 0 {
            break;
        }

        s.count += 1;
        iter += 1;
    }

    let mut a = c2v { x: 0.0, y: 0.0 };
    let mut b = c2v { x: 0.0, y: 0.0 };
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        if dist > rA + rB && dist > 1.192_092_9e-7_f32 {
            dist -= rA + rB;
            let n = c2Norm(c2Sub(b, a));
            a = c2Add(a, c2Mulvs(n, rA));
            b = c2Sub(b, c2Mulvs(n, rB));
            if a.x == b.x && a.y == b.y {
                dist = 0.0;
            }
        } else {
            let p = c2Mulvs(c2Add(a, b), 0.5);
            a = p;
            b = p;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache_mut.iA[i as usize] = v.iA;
            cache_mut.iB[i as usize] = v.iB;
        }
        cache_mut.div = s.div;
    }

    if let Some(outA) = outA {
        *outA = a;
    }
    if let Some(outB) = outB {
        *outB = b;
    }
    if let Some(iterations) = iterations {
        *iterations = iter;
    }

    dist
}

pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) unsafe fn c2AABBtoCapsule(A: c2AABB, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_AABB,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) unsafe fn c2CapsuletoCapsule(A: c2Capsule, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(
// ... (truncated) ...
```

**Entity:** c2MakeProxy / c2Proxy (in combination with C2_TYPE and raw shape pointers)

**States:** UntrustedRaw(shape_ptr,type_tag), ProxyBuiltForCircle, ProxyBuiltForAABB, ProxyBuiltForCapsule

**Transitions:**
- UntrustedRaw(shape_ptr,type_tag) -> ProxyBuiltForCircle via c2MakeProxy(..., C2_TYPE_CIRCLE, ...)
- UntrustedRaw(shape_ptr,type_tag) -> ProxyBuiltForAABB via c2MakeProxy(..., C2_TYPE_AABB, ...)
- UntrustedRaw(shape_ptr,type_tag) -> ProxyBuiltForCapsule via c2MakeProxy(..., C2_TYPE_CAPSULE, ...)

**Evidence:** type tag constants: `pub const C2_TYPE_CIRCLE`, `C2_TYPE_AABB`, `C2_TYPE_CAPSULE` used to select casts; c2MakeProxy signature: `unsafe fn c2MakeProxy(shape: *const c_void, type_0: C2_TYPE, p: Option<&mut c2Proxy>)` accepts untyped pointer plus tag; c2MakeProxy(): `let c = (shape as *const c2Circle).as_ref().unwrap();` and similarly for capsule uses unwrap() requiring non-null + correct type; c2MakeProxy() AABB branch: `let bb = (shape as *const c2AABB).as_ref(); ... c2BBVerts(&mut p.verts, bb);` passes an Option into c2BBVerts which then does `let bb = bb.unwrap();` (still requiring non-null and correct pointer/type); c2MakeProxy() sets `p.count = 1` (circle), `4` (AABB), `2` (capsule), and initializes only that many vertices; c2Support(verts, count, d): reads `verts[0]` unconditionally and then iterates `for i in 1..count` (requires count>=1 and vertices initialized up to count)

**Implementation:** Replace `(shape: *const c_void, type_0: C2_TYPE)` with a typed sum type, e.g. `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB), Capsule(&'a c2Capsule) }` and a safe `impl From<&c2Circle> for ShapeRef` etc. Then `make_proxy(shape: ShapeRef) -> c2Proxy` can set `count` and vertices without casts/unwraps. If FFI requires raw pointers, create a `ValidatedShapePtr` newtype produced by an unsafe constructor `unsafe fn from_raw(ptr, tag) -> Option<ValidatedShapePtr>` that checks non-null and returns a typed wrapper used by safe Rust code.

---

### 1. c2AABB validity invariant (min <= max per axis)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: An AABB is only geometrically meaningful when, for each axis, min is less than or equal to max (and typically both are finite). The struct exposes `min`/`max` as public fields with no constructor or validation, so callers can create an 'Invalid' box (e.g., swapped corners) and pass it into other algorithms. This invariant is relied on implicitly by typical AABB operations but is not enforced by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s); 2 free function(s)


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
- Invalid -> Valid via a validating/normalizing constructor (not present in snippet)

**Evidence:** pub struct c2AABB { pub min: c2v, pub max: c2v } — public fields allow constructing any ordering; #[derive(Copy, Clone)] on c2AABB — cheap copying encourages passing around without validation; #[repr(C)] — suggests FFI/plain-data usage where invariants are expected but not checked

**Implementation:** Make fields private and provide `impl c2AABB { pub fn new(min: c2v, max: c2v) -> ValidAabb { ... } }` that validates/normalizes (swap components as needed, optionally reject NaN/inf). Expose a `pub struct ValidAabb(c2AABB);` newtype so downstream APIs accept only `ValidAabb` rather than raw `c2AABB`.

---

### 6. c2Simplex validity invariant (active vertices count and div normalization)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex appears to encode a geometric simplex with up to 4 support vertices (a,b,c,d) and metadata `count` indicating how many vertices are active. The type system does not enforce that `count` is within the representable range (likely 0..=4 or 1..=4), nor that `div` is non-zero/normalized when used (commonly as a denominator/normalization factor). Consumers must implicitly respect that only the first `count` of {a,b,c,d} are valid/initialized for the current simplex and must avoid operations requiring `div != 0` unless that holds.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; 2 free function(s)


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

**States:** Valid(count ∈ 1..=4, div != 0, only first `count` vertices are meaningful), Invalid/Uninitialized(count outside 1..=4 and/or div == 0, extra vertices meaningless)

**Transitions:**
- Invalid/Uninitialized -> Valid via constructing/filling fields so that `count` matches the number of meaningful vertices and `div` is set to a usable value

**Evidence:** pub struct c2Simplex { pub a: c2sv, pub b: c2sv, pub c: c2sv, pub d: c2sv, pub div: f32, pub count: i32 } — fixed 4 slots plus a runtime `count` field implies only some slots are active; field `count: i32` — runtime integer encodes which simplex size/state is in use; not constrained to 0..=4 by the type system; field `div: f32` — standalone scalar typically used as a divisor/normalizer; not constrained away from 0/NaN/Inf by the type system

**Implementation:** Model the simplex as an enum over sizes or a typestate: `enum Simplex { One{a, div}, Two{a,b,div}, Three{a,b,c,div}, Four{a,b,c,d,div} }` (or `Simplex<N>` with const generics), eliminating the need for a runtime `count` and ensuring only the relevant vertices exist. Additionally wrap `div` in a `NonZeroF32`-like newtype (or store precomputed reciprocal) to ensure division preconditions are met.

---

### 2. c2Capsule geometric validity invariants (non-negative radius, non-degenerate endpoints)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Capsule represents a capsule defined by segment endpoints `a` and `b` and radius `r`. The type allows construction of geometrically invalid capsules (e.g., negative radius, NaN radius, or degenerate/NaN endpoints). Downstream geometry algorithms typically assume the capsule is valid; however, this module-level struct is `Copy` and exposes public fields, so there is no enforced validation step or invariant preservation by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s); 2 free function(s)


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
- Invalid -> Valid via validated construction (not present in snippet)
- Valid -> Invalid via direct public field mutation (e.g., setting r < 0.0)

**Evidence:** pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 } — all fields are public and unconstrained; derive(Copy, Clone) — values are trivially duplicated with no validation hook; r: f32 — permits negative values and NaN/Inf; no newtype or constructor enforces constraints

**Implementation:** Make fields private and provide a constructor like `c2Capsule::new(a, b, r: Radius)` where `Radius` is a newtype enforcing `r.is_finite() && r >= 0.0`. Optionally introduce `ValidatedCapsule(c2Capsule)` or `c2Capsule<Valid>` as a typestate to ensure only validated instances are passed into algorithms.

---

### 4. c2Proxy shape definition validity (count within verts capacity; only first `count` verts are used)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `c2Proxy` encodes a variable-length vertex list using a fixed-size array `verts: [c2v; 8]` plus a runtime `count: i32`. The implicit invariant is that `count` must be within the array capacity (and typically non-negative), and that only the first `count` entries in `verts` are considered part of the shape. The type system does not prevent constructing a `c2Proxy` with an out-of-range/negative `count`, nor does it guarantee that unused entries are ignored/initialized consistently. Any code that iterates `0..count` or assumes the active vertex slice is valid is relying on this runtime invariant.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2sv; struct c2Simplex, 3 free function(s); 2 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

```

**Entity:** c2Proxy

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via construction/normalization that clamps/validates `count` (not present in snippet)

**Evidence:** pub count: i32 field — runtime length/arity not tied to the type; pub verts: [c2v; 8] field — fixed capacity array implying `count` must be <= 8; #[repr(C)] on c2Proxy — suggests FFI-style "struct with length + buffer" pattern where invariants are typically maintained externally

**Implementation:** Hide the fields and provide a constructor returning `Result<c2Proxy, Error>` that validates `0..=8` and then exposes an accessor `fn verts(&self) -> &[c2v]` using `&self.verts[..count as usize]`. Optionally make `count` a `u8` newtype `VertCount(NonZeroU8/Range)` (or `u8` constrained to 0..=8) to eliminate negative/out-of-range values at compile time.

---

### 5. c2sv simplex vertex invariant (indices + barycentric/weight validity)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2sv appears to represent a simplex/support-vertex record (two support points sA/sB, a derived point p, weight u, and indices iA/iB). There are latent validity requirements that are not enforced by the type system: indices iA/iB are expected to be valid indices into some external vertex arrays/proxies (or sentinel values), and u is expected to be within a meaningful numeric range for a weight/barycentric coefficient (typically finite and often within [0,1]). Additionally, p is presumably derived from sA/sB (e.g., p = sB - sA or another relation), but nothing enforces consistency between these fields. Because the struct is Copy and all fields are public, any combination of values can be constructed, including out-of-range indices, NaN u, or inconsistent p.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2Simplex, 3 free function(s); 2 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2sv {
    pub sA: c2v,
    pub sB: c2v,
    pub p: c2v,
    pub u: f32,
    pub iA: i32,
    pub iB: i32,
}

```

**Entity:** c2sv

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via constructing/validating c2sv (not expressed in this snippet)

**Evidence:** pub struct c2sv { ... pub u: f32, pub iA: i32, pub iB: i32 } — raw scalar fields with no range/validity type; #[derive(Copy, Clone)] and all fields are `pub` — allows unchecked copying/construction of potentially invalid states; field names `iA`/`iB` strongly imply 'index A/B' into external data; `u` implies a weight/parameter

**Implementation:** Make fields private and provide a constructor returning Result that validates invariants. Use newtypes like `struct VertexIndex(u32);` (or `Option<VertexIndex>` for sentinel) and `struct Weight(f32);` where `Weight::new(x)` checks `x.is_finite()` (and optionally `0.0..=1.0`). If `p` must be derived, omit it from storage or compute it in a method to ensure consistency.

---

## Protocol Invariants

### 7. c2GJKCache validity protocol (Empty/Invalid vs Warm/Valid cache)

**Location**: `/data/test_case/lib.rs:1-705`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2GJKCache is treated as an optional warm-start cache for c2GJK(). At runtime, the algorithm decides whether the cache is usable based on cache.count and a metric consistency check; if it is usable, it seeds the simplex from cache.iA/iB and uses cache.div. Otherwise it ignores the cache and reinitializes the simplex. The type system does not prevent constructing a cache with inconsistent fields (e.g., count out of range, iA/iB out of bounds for the proxies, div == 0, metric unrelated), nor does it distinguish a validated cache from an unvalidated one.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2GJKCache; struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s)

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
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2r {
    pub c: f32,
    pub s: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2x {
    pub p: c2v,
    pub r: c2r,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Capsule {
    pub a: c2v,
    pub b: c2v,
    pub r: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2GJKCache {
    pub metric: f32,
    pub count: i32,
    pub iA: [i32; 3],
    pub iB: [i32; 3],
    pub div: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2sv {
    pub sA: c2v,
    pub sB: c2v,
    pub p: c2v,
    pub u: f32,
    pub iA: i32,
    pub iB: i32,
}

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

#[inline]
pub(crate) fn c2RotIdentity() -> c2r {
    c2r { c: 1.0, s: 0.0 }
}

#[inline]
pub(crate) fn c2xIdentity() -> c2x {
    c2x {
        p: c2V(0.0, 0.0),
        r: c2RotIdentity(),
    }
}

pub(crate) fn c2BBVerts(out: &mut [c2v], bb: Option<&c2AABB>) {
    let bb = bb.unwrap();
    out[0] = bb.min;
    out[1] = c2V(bb.max.x, bb.min.y);
    out[2] = bb.max;
    out[3] = c2V(bb.min.x, bb.max.y);
}

pub(crate) unsafe fn c2MakeProxy(
    shape: *const std::ffi::c_void,
    type_0: C2_TYPE,
    mut p: Option<&mut c2Proxy>,
) {
    let p = p.as_deref_mut().unwrap();
    match type_0 {
        C2_TYPE_CIRCLE => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        C2_TYPE_AABB => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        C2_TYPE_CAPSULE => {
            let c = (shape as *const c2Capsule).as_ref().unwrap();
            p.radius = c.r;
            p.count = 2;
            p.verts[0] = c.a;
            p.verts[1] = c.b;
        }
        _ => {}
    }
}

#[inline]
pub(crate) fn c2Len(a: c2v) -> f32 {
    c2Dot(a, a).sqrt()
}

#[inline]
pub(crate) fn c2Det2(a: c2v, b: c2v) -> f32 {
    a.x * b.y - a.y * b.x
}

pub(crate) fn c2GJKSimplexMetric(s: Option<&c2Simplex>) -> f32 {
    let s = s.unwrap();
    match s.count {
        2 => c2Len(c2Sub(s.b.p, s.a.p)),
        3 => c2Det2(c2Sub(s.b.p, s.a.p), c2Sub(s.c.p, s.a.p)),
        _ => 0.0,
    }
}

#[inline]
pub(crate) fn c2Mulrv(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x - a.s * b.y, a.s * b.x + a.c * b.y)
}

#[inline]
pub(crate) fn c2Add(mut a: c2v, b: c2v) -> c2v {
    a.x += b.x;
    a.y += b.y;
    a
}

#[inline]
pub(crate) fn c2Mulxv(a: c2x, b: c2v) -> c2v {
    c2Add(c2Mulrv(a.r, b), a.p)
}

pub(crate) fn c22(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    if v <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if u <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else {
        s.a.u = u;
        s.b.u = v;
        s.div = u + v;
        s.count = 2;
    }
}

pub(crate) fn c23(mut s: Option<&mut c2Simplex>) {
    let s = s.as_deref_mut().unwrap();
    let a = s.a.p;
    let b = s.b.p;
    let c = s.c.p;

    let uAB = c2Dot(b, c2Sub(b, a));
    let vAB = c2Dot(a, c2Sub(a, b));
    let uBC = c2Dot(c, c2Sub(c, b));
    let vBC = c2Dot(b, c2Sub(b, c));
    let uCA = c2Dot(a, c2Sub(a, c));
    let vCA = c2Dot(c, c2Sub(c, a));

    let area = c2Det2(c2Sub(b, a), c2Sub(c, a));
    let uABC = c2Det2(b, c) * area;
    let vABC = c2Det2(c, a) * area;
    let wABC = c2Det2(a, b) * area;

    if vAB <= 0.0 && uCA <= 0.0 {
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB <= 0.0 && vBC <= 0.0 {
        s.a = s.b;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uBC <= 0.0 && vCA <= 0.0 {
        s.a = s.c;
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    } else if uAB > 0.0 && vAB > 0.0 && wABC <= 0.0 {
        s.a.u = uAB;
        s.b.u = vAB;
        s.div = uAB + vAB;
        s.count = 2;
    } else if uBC > 0.0 && vBC > 0.0 && uABC <= 0.0 {
        s.a = s.b;
        s.b = s.c;
        s.a.u = uBC;
        s.b.u = vBC;
        s.div = uBC + vBC;
        s.count = 2;
    } else if uCA > 0.0 && vCA > 0.0 && vABC <= 0.0 {
        s.b = s.a;
        s.a = s.c;
        s.a.u = uCA;
        s.b.u = vCA;
        s.div = uCA + vCA;
        s.count = 2;
    } else {
        s.a.u = uABC;
        s.b.u = vABC;
        s.c.u = wABC;
        s.div = uABC + vABC + wABC;
        s.count = 3;
    }
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

pub(crate) fn c2D(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    match s.count {
        1 => c2Neg(s.a.p),
        2 => {
            let ab = c2Sub(s.b.p, s.a.p);
            if c2Det2(ab, c2Neg(s.a.p)) > 0.0 {
                c2Skew(ab)
            } else {
                c2CCW90(ab)
            }
        }
        _ => c2V(0.0, 0.0),
    }
}

pub(crate) fn c2Support(verts: &[c2v], count: i32, d: c2v) -> i32 {
    let count = count.max(0) as usize;
    let mut imax: usize = 0;
    let mut dmax = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i;
            dmax = dot;
        }
    }
    imax as i32
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0 / s.div;

    match s.count {
        1 => {
            *a.as_deref_mut().unwrap() = s.a.sA;
            *b.as_deref_mut().unwrap() = s.a.sB;
        }
        2 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sA, den * s.a.u),
                c2Mulvs(s.b.sA, den * s.b.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Mulvs(s.a.sB, den * s.a.u),
                c2Mulvs(s.b.sB, den * s.b.u),
            );
        }
        3 => {
            *a.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sA, den * s.a.u),
                    c2Mulvs(s.b.sA, den * s.b.u),
                ),
                c2Mulvs(s.c.sA, den * s.c.u),
            );
            *b.as_deref_mut().unwrap() = c2Add(
                c2Add(
                    c2Mulvs(s.a.sB, den * s.a.u),
                    c2Mulvs(s.b.sB, den * s.b.u),
                ),
                c2Mulvs(s.c.sB, den * s.c.u),
            );
        }
        _ => {
            *a.unwrap() = c2V(0.0, 0.0);
            *b.unwrap() = c2V(0.0, 0.0);
        }
    }
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0 / s.div;
    match s.count {
        1 => s.a.p,
        2 => c2Add(
            c2Mulvs(s.a.p, den * s.a.u),
            c2Mulvs(s.b.p, den * s.b.u),
        ),
        _ => c2V(0.0, 0.0),
    }
}

#[inline]
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
}

pub(crate) unsafe fn c2GJK(
    A: *const std::ffi::c_void,
    typeA: C2_TYPE,
    ax_ptr: Option<&c2x>,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
    bx_ptr: Option<&c2x>,
    outA: Option<&mut c2v>,
    outB: Option<&mut c2v>,
    use_radius: i32,
    iterations: Option<&mut i32>,
    mut cache: Option<&mut c2GJKCache>,
) -> f32 {
    let ax = ax_ptr.copied().unwrap_or_else(c2xIdentity);
    let bx = bx_ptr.copied().unwrap_or_else(c2xIdentity);

    let mut pA = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    let mut pB = c2Proxy {
        radius: 0.0,
        count: 0,
        verts: [c2v { x: 0.0, y: 0.0 }; 8],
    };
    c2MakeProxy(A, typeA, Some(&mut pA));
    c2MakeProxy(B, typeB, Some(&mut pB));

    let mut s = c2Simplex {
        a: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2v { x: 0.0, y: 0.0 },
            sB: c2v { x: 0.0, y: 0.0 },
            p: c2v { x: 0.0, y: 0.0 },
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i).as_mut().unwrap();
                v.iA = iA;
                v.sA = sA;
                v.iB = iB;
                v.sB = sB;
                v.p = c2Sub(v.sB, v.sA);
                v.u = 0.0;
            }

            s.count = cache_ref.count;
            s.div = cache_ref.div;

            let metric_old = cache_ref.metric;
            let metric = c2GJKSimplexMetric(Some(&s));
            let min_metric = metric.min(metric_old);
            let max_metric = metric.max(metric_old);

            if !(min_metric < max_metric * 2.0 && metric < -1.0e8) {
                cache_was_read = 1;
            }
        }
    }

    if cache_was_read == 0 {
        s.a.iA = 0;
        s.a.iB = 0;
        s.a.sA = c2Mulxv(ax, pA.verts[0]);
        s.a.sB = c2Mulxv(bx, pB.verts[0]);
        s.a.p = c2Sub(s.a.sB, s.a.sA);
        s.a.u = 1.0;
        s.div = 1.0;
        s.count = 1;
    }

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).read();
            saveA[i as usize] = v.iA;
            saveB[i as usize] = v.iB;
        }

        match s.count {
            2 => c22(Some(&mut s)),
            3 => c23(Some(&mut s)),
            _ => {}
        }

        if s.count == 3 {
            hit = 1;
            break;
        }

        let p = c2L(Some(&s));
        let d1 = c2Dot(p, p);
        if d1 > d0 {
            break;
        }
        d0 = d1;

        let d = c2D(Some(&s));
        if c2Dot(d, d) < 1.192_092_9e-7_f32 * 1.192_092_9e-7_f32 {
            break;
        }

        let iA = c2Support(&pA.verts, pA.count, c2MulrvT(ax.r, c2Neg(d)));
        let sA = c2Mulxv(ax, pA.verts[iA as usize]);
        let iB = c2Support(&pB.verts, pB.count, c2MulrvT(bx.r, d));
        let sB = c2Mulxv(bx, pB.verts[iB as usize]);

        let v = verts.offset(s.count as isize).as_mut().unwrap();
        v.iA = iA;
        v.sA = sA;
        v.iB = iB;
        v.sB = sB;
        v.p = c2Sub(v.sB, v.sA);

        let mut dup = 0;
        for i in 0..save_count {
            if iA == saveA[i as usize] && iB == saveB[i as usize] {
                dup = 1;
                break;
            }
        }
        if dup != 0 {
            break;
        }

        s.count += 1;
        iter += 1;
    }

    let mut a = c2v { x: 0.0, y: 0.0 };
    let mut b = c2v { x: 0.0, y: 0.0 };
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        if dist > rA + rB && dist > 1.192_092_9e-7_f32 {
            dist -= rA + rB;
            let n = c2Norm(c2Sub(b, a));
            a = c2Add(a, c2Mulvs(n, rA));
            b = c2Sub(b, c2Mulvs(n, rB));
            if a.x == b.x && a.y == b.y {
                dist = 0.0;
            }
        } else {
            let p = c2Mulvs(c2Add(a, b), 0.5);
            a = p;
            b = p;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache_mut.iA[i as usize] = v.iA;
            cache_mut.iB[i as usize] = v.iB;
        }
        cache_mut.div = s.div;
    }

    if let Some(outA) = outA {
        *outA = a;
    }
    if let Some(outB) = outB {
        *outB = b;
    }
    if let Some(iterations) = iterations {
        *iterations = iter;
    }

    dist
}

pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let d0 = (B.max.x < A.min.x) as i32;
    let d1 = (A.max.x < B.min.x) as i32;
    let d2 = (B.max.y < A.min.y) as i32;
    let d3 = (A.max.y < B.min.y) as i32;
    ((d0 | d1 | d2 | d3) == 0) as i32
}

pub(crate) unsafe fn c2AABBtoCapsule(A: c2AABB, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_AABB,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) unsafe fn c2CapsuletoCapsule(A: c2Capsule, B: c2Capsule) -> i32 {
    if c2GJK(
        &raw const A as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        &raw const B as *const std::ffi::c_void,
        C2_TYPE_CAPSULE,
        None,
        None,
        None,
        1,
        None,
        None,
    ) != 0.0
    {
        return 0;
    }
    1
}

pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(
// ... (truncated) ...
```

**Entity:** c2GJKCache

**States:** EmptyOrInvalid, WarmValid

**Transitions:**
- EmptyOrInvalid -> WarmValid via successful c2GJK() writing cache_mut.{metric,count,iA,iB,div}
- WarmValid -> EmptyOrInvalid when c2GJK() rejects cache due to runtime metric check / count==0

**Evidence:** struct c2GJKCache { metric: f32, count: i32, iA: [i32; 3], iB: [i32; 3], div: f32 } encodes cache contents but has no validity marker; in c2GJK(): `let cache_was_good = (cache_ref.count != 0) as i32;` treats count==0 as 'no cache'; in c2GJK(): for i in 0..cache_ref.count: indexes `pA.verts[iA as usize]` and `pB.verts[iB as usize]` coming from cache_ref.iA/iB (requires iA/iB to be in-bounds for the proxy); in c2GJK(): `s.count = cache_ref.count; s.div = cache_ref.div;` assumes div/count are consistent for later `1.0 / s.div` in c2Witness()/c2L(); in c2GJK(): runtime acceptance gate `if !(min_metric < max_metric * 2.0 && metric < -1.0e8) { cache_was_read = 1; }` decides whether cache is trusted; in c2GJK(): on exit, writes `cache_mut.metric = ...; cache_mut.count = s.count; cache_mut.iA[i]=...; cache_mut.iB[i]=...; cache_mut.div = s.div;` establishing the 'WarmValid' state

**Implementation:** Introduce `struct GjkCache<State> { inner: c2GJKCache, _s: PhantomData<State> }` with `Empty` and `Validated` states. Provide constructors that create `GjkCache<Empty>` and a `validate(self, pA: &c2Proxy, pB: &c2Proxy) -> Option<GjkCache<Validated>>` that bounds-checks count (0..=3), iA/iB ranges against proxy.count, and div != 0. Make the warm-start path accept only `Option<&GjkCache<Validated>>`, and return an updated `GjkCache<Validated>` from c2GJK().

---

### 3. c2GJKCache validity protocol (Empty/Uninitialized vs Populated cache)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache is a plain #[repr(C)] Copy struct whose fields encode whether a GJK cache is meaningful. The struct likely relies on an implicit protocol where `count` determines how many entries of `iA`/`iB` are valid, and where `metric`/`div` are only meaningful when the cache is populated. None of these relationships are enforced by the type system: any `count` value can be stored (including negative or >3), and callers can read or write `iA`/`iB` entries regardless of `count`, creating invalid caches that downstream algorithms must defensively handle (or will misbehave on).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2r, 1 free function(s); struct c2x, 1 free function(s); struct c2Circle, 3 free function(s); struct c2AABB, 2 free function(s); struct c2Capsule, 1 free function(s); struct c2Proxy, 1 free function(s); struct c2sv; struct c2Simplex, 3 free function(s); 2 free function(s)


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

**States:** EmptyOrUninitialized, Populated

**Transitions:**
- EmptyOrUninitialized -> Populated via writing fields (especially setting `count` and filling `iA`/`iB`)

**Evidence:** pub struct c2GJKCache { ... } is a C-FFI-style data bag with no constructors/validators shown; field `count: i32` alongside fixed-size arrays `iA: [i32; 3]` and `iB: [i32; 3]` suggests `count` is a runtime 'active length' for those arrays; fields `metric: f32` and `div: f32` appear to be derived/valid only under certain cache-populated conditions, but are always present and freely writable

**Implementation:** Introduce a validated Rust-side wrapper that enforces invariants before exposing it to algorithms: e.g., `struct GjkCache { metric: f32, div: f32, indices: SmallVec<[(i32,i32); 3]> }` or `enum GjkCache { Empty, Populated { metric: f32, div: NonZeroF32, pairs: [(i32,i32); 3], count: ValidCount3 } }`. Use a `ValidCount3` newtype (0..=3) and provide safe constructors; keep `c2GJKCache` as the raw FFI representation and implement `TryFrom<c2GJKCache>` / `From<GjkCache>` for boundary checks.

---

