# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 6
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Temporal Ordering Invariants

### 5. c2Proxy initialization + count/verts invariant (MakeProxy must set a valid vertex count for Support/GJK)

**Location**: `/data/test_case/lib.rs:1-711`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2Proxy is created with dummy fields (count=0, verts all zeros) and is only made valid after `c2MakeProxy` fills `radius`, `count`, and the first `count` entries in `verts` according to the shape type. Downstream code assumes the proxy is valid: `c2Support` indexes `verts[0]` unconditionally and iterates `0..count`, and `c2GJK` passes `pA.count/pB.count` to `c2Support`. The type system does not prevent calling `c2Support` with `count==0` or a proxy that hasn't been initialized (or was initialized with a mismatched `type_0`/shape pointer).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 1 free function(s)

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
pub struct c2Circle {
    pub p: c2v,
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
pub struct c2Simplex {
    pub a: c2sv,
    pub b: c2sv,
    pub c: c2sv,
    pub d: c2sv,
    pub div: f32,
    pub count: i32,
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
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) const fn c2V(x: f32, y: f32) -> c2v {
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
    };
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
    };
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
    };
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
    };
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
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    // Keep the original memory layout trick: treat a,b,c,d as a contiguous array.
    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache) = cache.as_deref() {
        let cache_was_good = (cache.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache.count as isize) {
                let iA = cache.iA[i as usize];
                let iB = cache.iB[i as usize];
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

            s.count = cache.count;
            s.div = cache.div;

            let metric_old = cache.metric;
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
        let eps = 1.192_092_9e-7_f32;
        if c2Dot(d, d) < eps * eps {
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

    let mut a = c2V(0.0, 0.0);
    let mut b = c2V(0.0, 0.0);
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        let eps = 1.192_092_9e-7_f32;

        if dist > rA + rB && dist > eps {
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

    if let Some(cache) = cache.as_deref_mut() {
        cache.metric = c2GJKSimplexMetric(Some(&s));
        cache.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache.iA[i as usize] = v.iA;
            cache.iB[i as usize] = v.iB;
        }
        cache.div = s.div;
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
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.
// ... (truncated) ...
```

**Entity:** c2Proxy

**States:** Uninitialized/Invalid, CircleProxy(count=1), AABBProxy(count=4), CapsuleProxy(count=2)

**Transitions:**
- Uninitialized/Invalid -> CircleProxy via c2MakeProxy(type=C2_TYPE_CIRCLE)
- Uninitialized/Invalid -> AABBProxy via c2MakeProxy(type=C2_TYPE_AABB)
- Uninitialized/Invalid -> CapsuleProxy via c2MakeProxy(type=C2_TYPE_CAPSULE)

**Evidence:** struct c2Proxy fields: `count: i32`, `verts: [c2v; 8]` encode how many vertices are valid at runtime; c2GJK(): initializes `pA`/`pB` with `count: 0` then calls `c2MakeProxy(A, typeA, Some(&mut pA))` / `c2MakeProxy(B, typeB, Some(&mut pB))`; c2MakeProxy(): sets `p.count` to 1/4/2 depending on `type_0` and fills corresponding `p.verts[...]`; c2Support(): uses `let mut dmax = c2Dot(verts[0], d);` (requires count>=1 and verts[0] initialized); c2Support(): loops `for i in 1..count` where `count` comes from `pA.count` / `pB.count`

**Implementation:** Replace `(shape_ptr, C2_TYPE)` + mutable out-parameter with typed constructors returning a validated proxy: `impl From<&c2Circle> for Proxy<Circle>` etc., or `enum Proxy { Circle{...}, Aabb{...}, Capsule{...} }` with an internal `NonZeroUsize` count. Provide `support(&self, d) -> usize` as a method on the validated proxy so `count>=1` is guaranteed and `verts[0..count]` are always initialized.

---

## State Machine Invariants

### 1. c2GJKCache validity/initialization protocol (Empty/Uninitialized vs Populated)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache is a plain C-layout data bag intended to carry cached simplex/index data for a GJK iteration. Several fields implicitly encode whether the cache is usable: `count` likely indicates how many entries of `iA`/`iB` are valid (e.g., 0..=3), and `metric`/`div` likely must be consistent with that count. As written, the type system allows constructing arbitrary bit-patterns/values (including negative or >3 `count`, or leaving fields uninitialized via FFI), and there is no API-level way to ensure the cache is in a coherent state before it is consumed by algorithms elsewhere in the module.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 1 free function(s); 2 free function(s)


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
- EmptyOrUninitialized -> Populated via filling/updating fields (not represented in this snippet)

**Evidence:** field `count: i32` together with fixed-size arrays `iA: [i32; 3]` and `iB: [i32; 3]` implies only the first `count` elements are meaningful, but this is not enforced; field `metric: f32` suggests an additional validity constraint tied to the cached simplex/iteration, but it is just a public float; field `div: f32` suggests a derived/normalization value that likely must be non-zero/finite when `count > 0`, but this is not enforced; `#[repr(C)]` and `pub` fields indicate this may be shared with C/FFI code where uninitialized/invalid states are possible

**Implementation:** Make `c2GJKCache` construction go through checked constructors and wrap `count` in a bounded type (e.g., `struct CacheCount(u8)` with invariant 0..=3). Store indices in `[(i32,i32); 3]` plus a `CacheCount`, and keep `metric/div` private with a `fn new_empty() -> Self` and `fn set_entry(&mut self, idx: CacheCount, a: i32, b: i32)`/`fn finalize(&mut self, metric: f32, div: NonZeroF32)` to prevent incoherent states. If FFI requires the exact layout, provide a separate `#[repr(C)]` raw struct and a safe validated wrapper `struct GjkCache(GjkCacheRaw)`.

---

### 4. c2Simplex vertex-count protocol (1/2/3 vertices with matching div/u invariants)

**Location**: `/data/test_case/lib.rs:1-711`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Multiple functions treat c2Simplex as being in one of a few discrete algorithmic states determined by `count` (number of active simplex vertices). For each count, only a prefix of (a,b,c) is considered initialized/meaningful, and additional fields (`div`, barycentric weights `u`) must be set consistently. This is enforced only by runtime `match s.count` branching and by carefully-written mutation routines (c22/c23). The type system does not prevent calling operations with an incompatible `count` (e.g., `c2Witness` computes `den = 1.0 / s.div` before checking count, so `div` must be nonzero in Count1/2/3 states).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 1 free function(s)

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
pub struct c2Circle {
    pub p: c2v,
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
pub struct c2Simplex {
    pub a: c2sv,
    pub b: c2sv,
    pub c: c2sv,
    pub d: c2sv,
    pub div: f32,
    pub count: i32,
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
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) const fn c2V(x: f32, y: f32) -> c2v {
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
    };
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
    };
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
    };
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
    };
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
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    // Keep the original memory layout trick: treat a,b,c,d as a contiguous array.
    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache) = cache.as_deref() {
        let cache_was_good = (cache.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache.count as isize) {
                let iA = cache.iA[i as usize];
                let iB = cache.iB[i as usize];
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

            s.count = cache.count;
            s.div = cache.div;

            let metric_old = cache.metric;
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
        let eps = 1.192_092_9e-7_f32;
        if c2Dot(d, d) < eps * eps {
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

    let mut a = c2V(0.0, 0.0);
    let mut b = c2V(0.0, 0.0);
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        let eps = 1.192_092_9e-7_f32;

        if dist > rA + rB && dist > eps {
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

    if let Some(cache) = cache.as_deref_mut() {
        cache.metric = c2GJKSimplexMetric(Some(&s));
        cache.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache.iA[i as usize] = v.iA;
            cache.iB[i as usize] = v.iB;
        }
        cache.div = s.div;
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
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.
// ... (truncated) ...
```

**Entity:** c2Simplex

**States:** Count1, Count2, Count3, Invalid/Other

**Transitions:**
- Count2 -> Count1 or Count2 via c22() (updates a/b.u, div, count)
- Count3 -> Count1 or Count2 or Count3 via c23() (updates a/b/c.u, div, count)
- Count1/Count2 -> Count3 via c2GJK loop (s.count += 1 when adding a new vertex)
- Any -> Count1 via c2GJK initialization (sets count=1, div=1.0, a.u=1.0)

**Evidence:** field: c2Simplex::count used as a runtime discriminator in c2GJKSimplexMetric(), c2D(), c2L(), c2Witness(); field: c2Simplex::div used in c2Witness() and c2L() as `let den = 1.0 / s.div` (requires div != 0 in valid states); function c22(Some(&mut s)): sets `s.div` and `s.count` to 1 or 2 depending on region tests; function c23(Some(&mut s)): sets `s.div` and `s.count` to 1,2, or 3; in c2GJK(): `match s.count { 2 => c22(...), 3 => c23(...), _ => {} }` and later `s.count += 1` to add a new vertex

**Implementation:** Represent simplex as an enum or typestate: `enum Simplex { One(V1), Two(V2), Three(V3) }` where each variant stores only the active vertices and guarantees `div != 0` and valid barycentric weights. Expose `reduce()` transitions returning the appropriate variant (e.g., `Two::reduce(self) -> Simplex` for c22 logic, `Three::reduce(self) -> Simplex` for c23). Make `witness()`/`closest_point()` methods exist only on the valid variants so `div` cannot be zero by construction.

---

## Precondition Invariants

### 3. c2AABB validity invariant (min <= max on each axis)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: An axis-aligned bounding box typically requires that min.x <= max.x and min.y <= max.y (and similarly for other dimensions if applicable). As defined, c2AABB is a plain POD with public fields, so callers can construct an 'Invalid' AABB (swapped extents, NaNs, etc.). Any downstream algorithms likely assume the 'Valid' state but the type system does not enforce it, and there are no constructors here that normalize or validate.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); 2 free function(s)


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
- Invalid -> Valid via normalization/validation constructor (not present in this snippet)

**Evidence:** pub struct c2AABB { pub min: c2v, pub max: c2v } — public fields allow constructing inconsistent bounds; #[repr(C)] and #[derive(Copy, Clone)] indicate this is intended as a simple FFI/value type with no enforced invariants

**Implementation:** Make fields private and provide constructors like `c2AABB::new(min, max)` that either validates (returns Result) or normalizes (swaps components so min<=max). Expose `min()`/`max()` accessors. Alternatively create `struct ValidAabb(c2AABB);` with `TryFrom<c2AABB>` performing validation for code that requires the invariant.

---

### 2. c2Proxy vertex-count and radius validity invariant

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2Proxy encodes a variable-length list of vertices using a fixed-size array `verts: [c2v; 8]` plus a runtime `count: i32`. Correct use requires `count` to be within the array capacity (and non-negative) so consumers only read the first `count` vertices. Additionally, `radius: f32` likely represents a geometric radius and is implicitly expected to be non-negative and finite. None of these constraints are enforced by the type system because `count` is an unrestricted `i32` and `radius` is an unrestricted `f32`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2AABB, 1 free function(s); 2 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

```

**Entity:** c2Proxy

**States:** Valid (count within verts capacity, radius non-negative), Invalid (count out of range and/or radius negative/NaN)

**Transitions:**
- Invalid -> Valid via constructing/setting count and radius to satisfy the constraints
- Valid -> Invalid via mutating count/radius to out-of-range/negative/NaN values

**Evidence:** line 7: field `pub count: i32` is an unrestricted runtime length for the vertex set; line 8: field `pub verts: [c2v; 8]` has fixed capacity 8, implying `count` must be in 0..=8 to be safe/meaningful; line 6: field `pub radius: f32` (unrestricted float) implies an unstated non-negative/finite geometric constraint

**Implementation:** Make construction go through a validating API: e.g., `struct VertexCount(u8);` with `TryFrom<i32>` ensuring 0..=8, and `struct Radius(f32);` with `TryFrom<f32>` ensuring finite and >= 0. Then redefine `c2Proxy` as `struct c2Proxy { radius: Radius, count: VertexCount, verts: [c2v; 8] }`, or alternatively store `verts_len: u8` and keep fields private with a constructor `fn new(radius: Radius, verts: impl IntoIterator<Item=c2v>) -> Result<Self, Error>`.

---

## Protocol Invariants

### 6. Shape pointer/type-tag coupling protocol (void* must match C2_TYPE and be non-null)

**Location**: `/data/test_case/lib.rs:1-711`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Several APIs accept `*const c_void` plus a runtime `C2_TYPE` tag and then cast the pointer based on the tag. Correctness requires that the pointer is non-null, properly aligned, and actually points to the shape corresponding to `type_0`/`typeA`/`typeB`. This coupling is not enforced by Rust types; misuse leads to UB (invalid cast/deref) or panics (unwrap on null).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Circle, 3 free function(s); struct c2v, 24 free function(s); struct c2Capsule, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 1 free function(s)

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
pub struct c2Circle {
    pub p: c2v,
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
pub struct c2Simplex {
    pub a: c2sv,
    pub b: c2sv,
    pub c: c2sv,
    pub d: c2sv,
    pub div: f32,
    pub count: i32,
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
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[inline]
pub(crate) const fn c2V(x: f32, y: f32) -> c2v {
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
    };
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
    };
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
    };
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
    };
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
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        b: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        c: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        d: c2sv {
            sA: c2V(0.0, 0.0),
            sB: c2V(0.0, 0.0),
            p: c2V(0.0, 0.0),
            u: 0.0,
            iA: 0,
            iB: 0,
        },
        div: 0.0,
        count: 0,
    };

    // Keep the original memory layout trick: treat a,b,c,d as a contiguous array.
    let verts: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache) = cache.as_deref() {
        let cache_was_good = (cache.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache.count as isize) {
                let iA = cache.iA[i as usize];
                let iB = cache.iB[i as usize];
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

            s.count = cache.count;
            s.div = cache.div;

            let metric_old = cache.metric;
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
        let eps = 1.192_092_9e-7_f32;
        if c2Dot(d, d) < eps * eps {
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

    let mut a = c2V(0.0, 0.0);
    let mut b = c2V(0.0, 0.0);
    c2Witness(Some(&s), Some(&mut a), Some(&mut b));

    let mut dist = c2Len(c2Sub(a, b));
    if hit != 0 {
        a = b;
        dist = 0.0;
    } else if use_radius != 0 {
        let rA = pA.radius;
        let rB = pB.radius;
        let eps = 1.192_092_9e-7_f32;

        if dist > rA + rB && dist > eps {
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

    if let Some(cache) = cache.as_deref_mut() {
        cache.metric = c2GJKSimplexMetric(Some(&s));
        cache.count = s.count;
        for i in 0..(s.count as isize) {
            let v = verts.offset(i).as_ref().unwrap();
            cache.iA[i as usize] = v.iA;
            cache.iB[i as usize] = v.iB;
        }
        cache.div = s.div;
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
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.
// ... (truncated) ...
```

**Entity:** C2_TYPE + c2MakeProxy/c2GJK raw shape pointers

**States:** ValidCirclePtr, ValidAABBPtr, ValidCapsulePtr, Invalid/Mismatched

**Transitions:**
- Invalid/Mismatched -> Valid{Shape}Ptr only by constructing calls with matching pointer and C2_TYPE tag

**Evidence:** function signature: `unsafe fn c2MakeProxy(shape: *const c_void, type_0: C2_TYPE, ...)`; c2MakeProxy(): casts based on `type_0`: `(shape as *const c2Circle).as_ref().unwrap()` and `(shape as *const c2Capsule).as_ref().unwrap()` (requires non-null + correct pointee type); c2MakeProxy(): for AABB uses `let bb = (shape as *const c2AABB).as_ref();` and then `c2BBVerts(&mut p.verts, bb)` where `c2BBVerts` does `let bb = bb.unwrap();` (also requires non-null); function signature: `unsafe fn c2GJK(A: *const c_void, typeA: C2_TYPE, ..., B: *const c_void, typeB: C2_TYPE, ...)` and calls `c2MakeProxy(A, typeA, ...)` / `c2MakeProxy(B, typeB, ...)`

**Implementation:** Introduce a typed shape enum or trait object instead of `(void*, C2_TYPE)`: `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB), Capsule(&'a c2Capsule) }`. Then define `fn make_proxy(shape: ShapeRef) -> Proxy` and `fn gjk(a: ShapeRef, ax: c2x, b: ShapeRef, bx: c2x, ...)`. If FFI constraints require raw pointers, wrap them in validated newtypes like `struct CirclePtr(NonNull<c2Circle>)` etc., eliminating the tag/pointer mismatch at compile time.

---

