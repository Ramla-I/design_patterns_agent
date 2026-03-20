# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 8
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 3
- **Precondition**: 4
- **Protocol**: 1
- **Modules analyzed**: 2

## State Machine Invariants

### 3. c2GJKCache validity protocol (Empty/Uninitialized vs Populated cache)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache is a plain C-repr POD cache struct intended to be filled/updated by algorithms (e.g., GJK). The type exposes raw scalar fields (metric, count, iA/iB, div) with no constructors or invariants enforced by the Rust type system. Implicitly, the fields must satisfy a consistency contract for the cache to be meaningful: count likely determines how many entries in iA/iB are valid (0..=3), and metric/div likely have expected ranges/relationships. As written, any bit-pattern can be constructed, copied, and passed around (Copy, Clone), including invalid combinations (e.g., count outside 0..=3, using iA/iB entries beyond count).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); 3 free function(s)


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

**States:** EmptyOrUninitialized, PopulatedValid

**Transitions:**
- EmptyOrUninitialized -> PopulatedValid via external initialization/fill (not represented in this snippet)

**Evidence:** derive(Copy, Clone): the struct can be duplicated freely, preserving potentially-invalid states; field `count: i32`: runtime integer likely encodes how many of `iA`/`iB` entries are valid, but not type-checked; fields `iA: [i32; 3]` and `iB: [i32; 3]`: fixed-size arrays whose partially-valid prefix is implicitly governed by `count`; field `metric: f32` and `div: f32`: raw floats suggest additional numeric validity expectations not encoded in types

**Implementation:** Make `c2GJKCache` fields private and provide a safe wrapper `GjkCache` with validated construction/update APIs. Use a newtype for `count` like `struct SimplexCount(u8);` validated to 0..=3, and expose indices as `[u8; 3]` (or `Option<NonNegativeIndex>` for unused slots). If an 'empty' cache is meaningful, represent it explicitly (e.g., `enum GjkCache { Empty, Populated { ... } }`) or `GjkCache<Empty/Populated>` typestate if you can model initialization steps.

---

### 7. c2Simplex internal state machine (count-driven active vertices + valid div/u invariants)

**Location**: `/data/test_case/lib.rs:1-698`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `c2Simplex` uses the runtime field `count: i32` to indicate which of `{a,b,c}` vertices are valid and how other fields (`div`, barycentric weights `u`) must be interpreted. Many functions branch on `s.count` and assume supporting invariants (e.g., `div != 0` when computing `1.0 / s.div`, `a/b/c` contain initialized witness data, only the first `count` vertices are read/written). These constraints are not represented in the type system; incorrect `count`/`div` combinations would cause invalid math (division by zero) or logic errors. In `c2GJK`, correctness relies on a temporal protocol: initialize simplex (either from cache or fresh) -> iteratively update via `c22`/`c23` -> then call `c2Witness`/`c2L`/`c2D` with a consistent simplex state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

extern "C" {
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
}

pub type C2_TYPE = u32;
pub const C2_TYPE_AABB: C2_TYPE = 2;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
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
        1 => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        2 => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        0 => {
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    let s = s.as_deref_mut().unwrap();
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;
    let c = sref.c.p;

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

    let s = s.as_deref_mut().unwrap();
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
    let mut imax: i32 = 0;
    let mut dmax: f32 = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i as i32;
            dmax = dot;
        }
    }
    imax
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;

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
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;
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
            for i in 0..cache_ref.count {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i as isize).as_mut().unwrap();
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

            if min_metric < max_metric * 2.0 && metric < -1.0e8 {
                // keep cache_was_read = 0
            } else {
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

    let mut saveA: [i32; 3] = [0; 3];
    let mut saveB: [i32; 3] = [0; 3];
    let mut d0: f32 = f32::MAX;
    let mut iter: i32 = 0;
    let mut hit: i32 = 0;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..save_count {
            let v = &*(verts.offset(i as isize) as *const c2sv);
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
        for i in 0..s.count {
            let v = verts.offset(i as isize).as_ref().unwrap();
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
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
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
        N
// ... (truncated) ...
```

**Entity:** c2Simplex

**States:** Count=0 (uninitialized), Count=1 (point), Count=2 (segment), Count=3 (triangle)

**Transitions:**
- Count=0 -> Count=1 via initialization in `c2GJK` (cache miss path sets `s.count = 1; s.div = 1.0; s.a.u = 1.0`)
- Count=1/2 -> Count=1/2 via `c22` (sets `s.count` and `s.div` based on region tests)
- Count=2/3 -> Count=1/2/3 via `c23` (sets `s.count` and `s.div` based on region tests)
- Count=1/2/3 -> Count=3 (terminal) when `c2GJK` loop reaches `s.count == 3` (hit)

**Evidence:** Struct field: `pub struct c2Simplex { ... pub div: f32, pub count: i32 }`; `c2GJKSimplexMetric` matches on `s.count` and reads `s.b`/`s.c` only for counts 2/3; `c2Witness`: `let den = 1.0f32 / s.div; match s.count { 1 => ..., 2 => uses den and s.a.u/s.b.u, 3 => uses den and s.a.u/s.b.u/s.c.u }` (implicit invariant: `div != 0` for count 2/3); `c2L`: `let den = 1.0f32 / s.div; match s.count { 1 => ..., 2 => uses den and u weights }`; `c22` and `c23` both mutate `s.count`, `s.div`, and barycentric `u` weights (`s.a.u`, `s.b.u`, `s.c.u`) to maintain region invariants; `c2GJK` loop: `match s.count { 2 => c22(Some(&mut s)), 3 => c23(Some(&mut s)), _ => {} }` followed by calls that assume a consistent simplex (`c2L(Some(&s))`, `c2D(Some(&s))`, `c2Witness(Some(&s), ...)`)

**Implementation:** Encode the simplex dimension in the type: `c2Simplex<D>` where `D` is `Dim1|Dim2|Dim3`, and store only the active vertices for that dimension. Provide transitions like `fn reduce(self) -> Either<c2Simplex<Dim1>, c2Simplex<Dim2>>` for `c22`, and similarly for `c23`. Alternatively use `enum Simplex { S1{...}, S2{...}, S3{...} }` so functions like `witness()` can take `&Simplex` and the compiler enforces that `div`/`u` are present and meaningful for each case.

---

### 4. c2Simplex vertex-count protocol (count determines which vertices/fields are valid)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex stores up to four simplex vertices (a,b,c,d) along with a runtime `count` that implicitly determines which of those vertex fields are meaningful. The type system does not prevent constructing or using a c2Simplex with an invalid `count` (e.g., negative or >4), nor does it prevent reading/using `b/c/d` when `count` indicates they are not initialized/valid. The `div` field also appears to be a derived accumulator/normalization factor for the simplex and is implicitly tied to the current set of active vertices (as determined by `count`), but nothing enforces that relationship at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); 3 free function(s)


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

**States:** Empty (count=0), 1-vertex (count=1), 2-vertex (count=2), 3-vertex (count=3), 4-vertex (count=4)

**Transitions:**
- Empty -> 1-vertex by setting count=1 and initializing `a`
- 1-vertex -> 2-vertex by setting count=2 and initializing `b`
- 2-vertex -> 3-vertex by setting count=3 and initializing `c`
- 3-vertex -> 4-vertex by setting count=4 and initializing `d`
- Any -> smaller simplex by decreasing count (dropping higher vertices)

**Evidence:** field `count: i32` encodes how many of `a/b/c/d` are active/valid at runtime; fields `a`, `b`, `c`, `d` are always present, implying a protocol is needed to know which ones are meaningful; field `div: f32` is a shared derived value for the simplex but is not coupled to `count`/vertices by types

**Implementation:** Model the simplex as `enum Simplex { Empty, V1{a,div}, V2{a,b,div}, V3{a,b,c,div}, V4{a,b,c,d,div} }` (or `struct Simplex<const N: usize>` with `[c2sv; N]` and a const-generic N in 0..=4). Expose only operations valid for each arity, and make transitions return the next arity type rather than mutating a shared `count`.

---

## Precondition Invariants

### 8. c2Proxy initialization + vertex-count validity (count bounds and nonzero count)

**Location**: `/data/test_case/lib.rs:1-698`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `c2Proxy` is constructed with `count = 0` and a default `verts` array, then must be initialized by `c2MakeProxy` before it is used (e.g., by `c2Support`, which assumes at least one vertex and indexes `verts[0]`). Additionally, `count` must be within the capacity of `verts` (8) and reflect the number of meaningful vertices written. This protocol is enforced only by call ordering inside `c2GJK` and by assuming only known shape types produce valid counts.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

extern "C" {
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
}

pub type C2_TYPE = u32;
pub const C2_TYPE_AABB: C2_TYPE = 2;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
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
        1 => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        2 => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        0 => {
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    let s = s.as_deref_mut().unwrap();
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;
    let c = sref.c.p;

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

    let s = s.as_deref_mut().unwrap();
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
    let mut imax: i32 = 0;
    let mut dmax: f32 = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i as i32;
            dmax = dot;
        }
    }
    imax
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;

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
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;
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
            for i in 0..cache_ref.count {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i as isize).as_mut().unwrap();
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

            if min_metric < max_metric * 2.0 && metric < -1.0e8 {
                // keep cache_was_read = 0
            } else {
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

    let mut saveA: [i32; 3] = [0; 3];
    let mut saveB: [i32; 3] = [0; 3];
    let mut d0: f32 = f32::MAX;
    let mut iter: i32 = 0;
    let mut hit: i32 = 0;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..save_count {
            let v = &*(verts.offset(i as isize) as *const c2sv);
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
        for i in 0..s.count {
            let v = verts.offset(i as isize).as_ref().unwrap();
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
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
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
        N
// ... (truncated) ...
```

**Entity:** c2Proxy

**States:** Uninitialized (count=0), Initialized (count in 1..=4, verts[0..count) valid)

**Transitions:**
- Uninitialized -> Initialized via `c2MakeProxy(...)`

**Evidence:** `c2GJK` creates proxies with `count: 0` then calls `c2MakeProxy(A, typeA, Some(&mut pA)); c2MakeProxy(B, typeB, Some(&mut pB));`; `c2Support(verts: &[c2v], count: i32, d: c2v)`: computes `let count = count.max(0) as usize;` then immediately does `let mut dmax: f32 = c2Dot(verts[0], d);` (implicit precondition: `count >= 1` and `verts` non-empty); `c2MakeProxy` sets `p.count = 1` for circles, `p.count = 4` for AABBs, `p.count = 2` for capsules; default/unknown tag `_ => {}` leaves `count` unchanged (can remain 0)

**Implementation:** Split into `c2Proxy<Uninit>` and `c2Proxy<Init>` (or `Option<NonZeroU8>` for count with stronger typing). Provide `fn make_proxy(shape: ShapeRef) -> c2Proxy<Init>` returning a fully-initialized proxy with `count: NonZeroU8` and `verts: [c2v; 8]` plus a method `fn verts(&self) -> &[c2v]` that returns the correctly sized slice `&self.verts[..count]`. Then make `c2Support` take `&[c2v]` (already sized) or `(&ProxyInit, d)` so it cannot be called with an uninitialized/empty proxy.

---

### 2. c2Circle geometric validity (non-negative, finite radius)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Circle represents a geometric circle with center `p` and radius `r`. The API exposes `r: f32` publicly, so callers can construct circles with negative radius, NaN, or infinite values. Many geometric algorithms implicitly require a non-negative, finite radius; this invariant is not enforced by the type system because `f32` permits invalid values and the fields are public.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); 3 free function(s)


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
- Invalid -> Valid via validated construction (not present in snippet)
- Valid -> Invalid via direct field mutation (public `r: f32`)

**Evidence:** pub struct c2Circle { pub p: c2v, pub r: f32 } — `r` is an unconstrained `f32` and publicly writable; #[repr(C)] — suggests FFI interop where callers may assume C-like validity contracts but Rust type system does not encode them

**Implementation:** Make fields private and introduce `struct Radius(f32);` with `TryFrom<f32>`/constructor enforcing `is_finite()` and `>= 0.0`. Expose `c2Circle::new(p: c2v, r: Radius)` (or `NonNegativeF32`) so only validated circles can be constructed in safe Rust. Keep an `#[repr(C)]` raw mirror type for FFI if needed.

---

### 1. c2Capsule geometric validity (non-negative radius; non-degenerate segment)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Capsule is a plain data carrier for a capsule defined by endpoints a/b and radius r. The type system does not prevent constructing capsules with invalid geometric parameters (e.g., negative radius, NaN/inf radius, or degenerate endpoints a == b if the rest of the API assumes a true segment). Downstream geometric routines typically rely on these being valid; without enforcement, invalid values can lead to incorrect math, panics (e.g., normalization of zero-length vectors), or FFI/ABI assumptions being violated at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); 3 free function(s)


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

**Evidence:** pub struct c2Capsule { pub a: c2v, pub b: c2v, pub r: f32 } — raw f32 radius has no non-negativity/finite constraint; #[repr(C)] — suggests FFI/ABI usage where invalid numeric values can be particularly problematic, but no validation is encoded

**Implementation:** Introduce validated wrappers like `struct Radius(f32);` with `TryFrom<f32>` enforcing `is_finite() && r >= 0.0`, and/or a `ValidatedCapsule` newtype created via `try_new(a, b, r)` that checks `a != b` (if required by algorithms) and stores `r: Radius`. Keep `c2Capsule` as the raw FFI struct if needed, but only expose safe APIs in terms of the validated type.

---

### 5. c2Proxy vertex-count validity invariant (count bounds + active verts prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `c2Proxy` encodes a variable-length list of vertices using a fixed-size array `verts: [c2v; 8]` plus a runtime `count: i32`. The implicit invariant is that only the first `count` entries in `verts` are considered initialized/active, and `count` must be within the capacity (0..=8). The type system does not prevent constructing a `c2Proxy` with negative `count`, `count > 8`, or with meaningless trailing vertices being interpreted incorrectly by downstream code that trusts `count`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; 3 free function(s)


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
- Invalid -> Valid via validation/constructor that clamps or rejects bad count and enforces prefix semantics

**Evidence:** struct c2Proxy field: `pub count: i32` (runtime length, can be negative/out of range); struct c2Proxy field: `pub verts: [c2v; 8]` (fixed capacity that count must not exceed); struct c2Proxy field: `pub radius: f32` (likely coupled to geometry/verts but no type-level relation enforced)

**Implementation:** Make construction go through a checked API: e.g., `struct VertexCount(u8); impl TryFrom<i32> for VertexCount { ... }` ensuring `<= 8`, and store `count: VertexCount`. Optionally wrap vertices as `struct ProxyVerts { verts: [c2v; 8], len: VertexCount }` and expose `fn verts(&self) -> &[c2v]` that returns only the active prefix.

---

## Protocol Invariants

### 6. Tagged-void* shape protocol (type tag must match pointee layout)

**Location**: `/data/test_case/lib.rs:1-698`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `c2MakeProxy` and `c2GJK` take `*const c_void` plus a runtime `C2_TYPE` tag and then cast the pointer to a concrete shape type based on the tag. Correctness requires that `typeA/typeB` exactly match the actual pointee type behind `A/B`. This is a latent invariant: the compiler cannot prevent passing `C2_TYPE_AABB` with a pointer to `c2Capsule` (or a dangling/unaligned pointer), which would lead to UB via `.as_ref().unwrap()` on an invalid cast. The code currently enforces this only by conventions and runtime branching, not by types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

extern "C" {
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
}

pub type C2_TYPE = u32;
pub const C2_TYPE_AABB: C2_TYPE = 2;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

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
pub struct c2AABB {
    pub min: c2v,
    pub max: c2v,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Circle {
    pub p: c2v,
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
        1 => {
            let c = (shape as *const c2Circle).as_ref().unwrap();
            p.radius = c.r;
            p.count = 1;
            p.verts[0] = c.p;
        }
        2 => {
            let bb = (shape as *const c2AABB).as_ref();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, bb);
        }
        0 => {
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;

    let u = c2Dot(b, c2Sub(b, a));
    let v = c2Dot(a, c2Sub(a, b));

    let s = s.as_deref_mut().unwrap();
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
    let sref = s.as_deref().unwrap();
    let a = sref.a.p;
    let b = sref.b.p;
    let c = sref.c.p;

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

    let s = s.as_deref_mut().unwrap();
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
    let mut imax: i32 = 0;
    let mut dmax: f32 = c2Dot(verts[0], d);
    for i in 1..count {
        let dot = c2Dot(verts[i], d);
        if dot > dmax {
            imax = i as i32;
            dmax = dot;
        }
    }
    imax
}

pub(crate) fn c2Witness(s: Option<&c2Simplex>, mut a: Option<&mut c2v>, mut b: Option<&mut c2v>) {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;

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
    c2Mulvs(a, 1.0f32 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

pub(crate) fn c2L(s: Option<&c2Simplex>) -> c2v {
    let s = s.unwrap();
    let den = 1.0f32 / s.div;
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
            for i in 0..cache_ref.count {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];
                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts.offset(i as isize).as_mut().unwrap();
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

            if min_metric < max_metric * 2.0 && metric < -1.0e8 {
                // keep cache_was_read = 0
            } else {
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

    let mut saveA: [i32; 3] = [0; 3];
    let mut saveB: [i32; 3] = [0; 3];
    let mut d0: f32 = f32::MAX;
    let mut iter: i32 = 0;
    let mut hit: i32 = 0;

    while iter < 20 {
        let save_count = s.count;
        for i in 0..save_count {
            let v = &*(verts.offset(i as isize) as *const c2sv);
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
        for i in 0..s.count {
            let v = verts.offset(i as isize).as_ref().unwrap();
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
    let d0: i32 = (B.max.x < A.min.x) as i32;
    let d1: i32 = (A.max.x < B.min.x) as i32;
    let d2: i32 = (B.max.y < A.min.y) as i32;
    let d3: i32 = (A.max.y < B.min.y) as i32;
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
        N
// ... (truncated) ...
```

**Entity:** c2MakeProxy / c2GJK (shape pointer + C2_TYPE tag)

**States:** Circle, AABB, Capsule, Unknown/Invalid

**Transitions:**
- Unknown/Invalid -> {Circle|AABB|Capsule} via choosing a matching C2_TYPE at call sites

**Evidence:** C2_TYPE constants: `pub const C2_TYPE_AABB: C2_TYPE = 2;` and `pub const C2_TYPE_CAPSULE: C2_TYPE = 0;` (and match arms use numeric tags); `unsafe fn c2MakeProxy(shape: *const c_void, type_0: C2_TYPE, ...)` then `match type_0 { 1 => (shape as *const c2Circle).as_ref().unwrap(), 2 => (shape as *const c2AABB).as_ref(), 0 => (shape as *const c2Capsule).as_ref().unwrap(), _ => {} }`; `unsafe fn c2GJK(A: *const c_void, typeA: C2_TYPE, ..., B: *const c_void, typeB: C2_TYPE, ...)` calls `c2MakeProxy(A, typeA, ...)` and `c2MakeProxy(B, typeB, ...)`; Call site example: `c2AABBtoCapsule` passes `&raw const A as *const c_void` with `C2_TYPE_AABB` and `&raw const B as *const c_void` with `C2_TYPE_CAPSULE`

**Implementation:** Replace `(ptr: *const c_void, tag: C2_TYPE)` with a typed enum or trait-object-free sum type, e.g. `enum ShapeRef<'a> { Circle(&'a c2Circle), AABB(&'a c2AABB), Capsule(&'a c2Capsule) }`. Then `c2MakeProxy(shape: ShapeRef, p: &mut c2Proxy)` and `c2GJK(A: ShapeRef, ax: Option<&c2x>, B: ShapeRef, bx: Option<&c2x>, ...)` remove the cast/tag mismatch class entirely. If FFI requires `c_void`, introduce `struct TaggedShapePtr<T> { ptr: NonNull<T> }` constructors per shape that also supply the correct tag, so call sites cannot mix them.

---

