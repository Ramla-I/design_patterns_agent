# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 3
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 2. c2Simplex validity protocol (count-gated active vertices + div invariant)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex encodes a small state machine in the runtime field `count`: it determines which of {a,b,c,d} are logically present/active. Code using this struct must branch on `count` to know which vertices are valid to read/update; the type system does not prevent reading inactive members. Additionally `div` is a precomputed denominator/normalization factor whose meaningfulness likely depends on the active simplex size (and must typically be non-zero in the states where it is used), but this coupling is not enforced at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); 2 free function(s)


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

**States:** Empty/Uninitialized (count = 0), 1-vertex simplex (count = 1), 2-vertex simplex (count = 2), 3-vertex simplex (count = 3), 4-vertex simplex (count = 4)

**Transitions:**
- count = 0 -> 1 via adding vertex a
- 1 -> 2 via adding vertex b
- 2 -> 3 via adding vertex c
- 3 -> 4 via adding vertex d
- N -> M via reduction/closest-feature selection (dropping vertices) (implied by presence of 4 slots + count field)

**Evidence:** struct c2Simplex fields `a`, `b`, `c`, `d`: four fixed vertex slots implying a variable-active-set representation; field `count: i32`: runtime cardinality/state indicator not reflected in the type; field `div: f32`: extra derived value that is meaningful only under certain geometric/active-set conditions, but unconstrained by types

**Implementation:** Represent the simplex as an enum/typestate: `enum Simplex { Empty, One([c2sv;1], Div), Two([c2sv;2], Div), Three([c2sv;3], Div), Four([c2sv;4], Div) }` (or `c2Simplex<S>` with `S=Empty/One/Two/Three/Four`). Expose operations like `add_vertex(self, v) -> SimplexNext` and `reduce(self, ...) -> SimplexSmaller` so code cannot access `c`/`d` unless in the corresponding state; ensure `div` is constructed/validated alongside the state (e.g., non-zero newtype).

---

### 4. c2Simplex validity protocol (count-dependent initialized vertices + nonzero div)

**Location**: `/data/test_case/lib.rs:1-702`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Multiple functions interpret `c2Simplex` differently depending on `s.count` and assume that the corresponding vertices (a/b/c) and barycentric weights (`u`) are initialized consistently, and that `s.div` is nonzero when used as a divisor. These invariants are maintained by the GJK update functions (`c22`, `c23`) and initialization in `c2GJK`, but not enforced by the type system: any caller can construct a `c2Simplex` with arbitrary `count/div` and call `c2Witness`, `c2L`, `c2D`, or `c2GJKSimplexMetric`, which can lead to division-by-zero (`1.0 / s.div`) or reading meaningless fields.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s)

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

    // Keep the original memory layout trick (a,b,c contiguous) but avoid repeated pointer math.
    let verts_ptr: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];

                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts_ptr.offset(i).as_mut().unwrap();
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
                // keep behavior
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

        for i in 0..(save_count as isize) {
            let v = (verts_ptr.offset(i) as *const c2sv).read();
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

        let v_new = verts_ptr.offset(s.count as isize).as_mut().unwrap();
        v_new.iA = iA;
        v_new.sA = sA;
        v_new.iB = iB;
        v_new.sB = sB;
        v_new.p = c2Sub(v_new.sB, v_new.sA);

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
            let p_mid = c2Mulvs(c2Add(a, b), 0.5);
            a = p_mid;
            b = p_mid;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;

        for i in 0..(s.count as isize) {
            let v = verts_ptr.offset(i).as_ref().unwrap();
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
        &raw const B as *c
// ... (truncated) ...
```

**Entity:** c2Simplex (used by c22/c23/c2D/c2L/c2Witness/c2GJKSimplexMetric)

**States:** Count0OrInvalid, Simplex1, Simplex2, Simplex3

**Transitions:**
- Count0OrInvalid -> Simplex1 via c2GJK initial simplex setup (sets count=1, div=1, a.u=1)
- Simplex2 -> Simplex1/Simplex2 via c22 (updates u/div/count)
- Simplex3 -> Simplex1/Simplex2/Simplex3 via c23 (updates u/div/count)
- Simplex1/Simplex2 -> Simplex3 via c2GJK loop when s.count increments and update yields count==3

**Evidence:** struct c2Simplex { ... div: f32, count: i32 } (runtime state field `count` gates meaning of fields); c2Witness: `let den = 1.0 / s.div;` then uses den when s.count is 2 or 3 (requires s.div != 0 and u fields set); c2L: `let den = 1.0 / s.div;` used when s.count==2; c2D: match s.count { 1 => ..., 2 => ... } (assumes a/b are valid when count==2); c2GJKSimplexMetric: match s.count { 2 => ..., 3 => ... }; c22/c23 mutate `s.count`, `s.div`, and per-vertex `u` based on geometric tests (they are the protocol transitions)

**Implementation:** Model simplex variants as distinct types: `Simplex1 { a: c2sv }`, `Simplex2 { a: c2sv, b: c2sv, div: NonZeroF32 }`, `Simplex3 { a: c2sv, b: c2sv, c: c2sv, div: NonZeroF32 }`. Provide transition functions `fn reduce(self) -> Simplex1|Simplex2` etc. Then `witness()`/`l()`/`d()` can be implemented only for the states where they are valid, eliminating `count` checks and preventing `div==0` at compile time (using a `NonZeroF32` newtype or storing `inv_div`).

---

## Precondition Invariants

### 1. c2GJKCache validity invariants (simplex count/index arrays/metric coherence)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache encodes a GJK (Gilbert–Johnson–Keerthi) warm-start cache. Several fields imply coherence/validity rules that are not enforced by the type system: (1) `count` determines how many entries of `iA`/`iB` are meaningful (typically 0..=3), (2) indices in `iA`/`iB` should be within the vertex ranges of the shapes they reference, and (3) `metric` and `div` are meaningful only when the cache is in a valid, normalized state. Because all fields are public and `Copy`, any code can construct or mutate a cache into an invalid combination (e.g., `count=5`, or stale indices), and the type system cannot distinguish a cache that is safe to use from one that needs initialization/normalization.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s); 2 free function(s)


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

**States:** EmptyOrUninitialized, ValidCache(count=1..=3), Invalid(count outside 0..=3 or indices out of range)

**Transitions:**
- EmptyOrUninitialized -> ValidCache(count=1..=3) via construction/population by GJK algorithm (not shown in snippet)
- ValidCache(count=1..=3) -> Invalid via arbitrary public-field writes/copies

**Evidence:** pub struct c2GJKCache { ... } with all fields `pub` allows creating incoherent states; field `count: i32` implies a runtime cardinality for `iA: [i32; 3]` and `iB: [i32; 3]` (only first `count` entries should be used); fixed-size arrays `iA: [i32; 3]` and `iB: [i32; 3]` imply an implicit bound `count <= 3` that is not encoded in the type; derived `Copy, Clone` enables duplicating potentially invalid/stale caches without any validation step; fields `metric: f32` and `div: f32` suggest derived/normalized values that depend on `count`/indices but have no type-level coupling

**Implementation:** Make fields private and expose a validated constructor, e.g. `struct GjkCache { inner: c2GJKCache }` with `TryFrom<c2GJKCache>` enforcing `0..=3` count and any additional invariants; alternatively replace `count: i32` with a bounded type like `u8` plus an internal `enum Count { Zero, One, Two, Three }` or `NonZeroU8` for non-empty caches, and provide accessor methods that only yield `&[(i32,i32)]` slices of length `count`.

---

### 3. Shape pointer + type tag agreement (Circle/AABB/Capsule) and initialized-proxy guarantee

**Location**: `/data/test_case/lib.rs:1-702`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: c2MakeProxy assumes the caller passes (1) a non-null `shape: *const c_void` that actually points to a value of the concrete Rust repr(C) type indicated by `type_0`, and (2) `p: Option<&mut c2Proxy>` is `Some` so it can be written. It then partially initializes `c2Proxy` fields (radius/count/verts[0..count]) based on the runtime tag. If `type_0` is not one of the known constants or `shape` is not of the promised dynamic type, the function either leaves `p` unchanged (default `_ => {}` branch) or performs UB by casting/dereferencing the wrong type. None of these requirements are enforced by the type system because the API uses `*const c_void` + `C2_TYPE` tag + `Option` wrappers and `unwrap()`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s)

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

    // Keep the original memory layout trick (a,b,c contiguous) but avoid repeated pointer math.
    let verts_ptr: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];

                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts_ptr.offset(i).as_mut().unwrap();
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
                // keep behavior
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

        for i in 0..(save_count as isize) {
            let v = (verts_ptr.offset(i) as *const c2sv).read();
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

        let v_new = verts_ptr.offset(s.count as isize).as_mut().unwrap();
        v_new.iA = iA;
        v_new.sA = sA;
        v_new.iB = iB;
        v_new.sB = sB;
        v_new.p = c2Sub(v_new.sB, v_new.sA);

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
            let p_mid = c2Mulvs(c2Add(a, b), 0.5);
            a = p_mid;
            b = p_mid;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;

        for i in 0..(s.count as isize) {
            let v = verts_ptr.offset(i).as_ref().unwrap();
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
        &raw const B as *c
// ... (truncated) ...
```

**Entity:** c2MakeProxy / (shape pointer, type_0, c2Proxy)

**States:** UninitializedProxy, InitializedProxy(Circle), InitializedProxy(AABB), InitializedProxy(Capsule), InvalidTypeOrNull

**Transitions:**
- UninitializedProxy -> InitializedProxy(Circle) via c2MakeProxy(..., C2_TYPE_CIRCLE, Some(&mut p))
- UninitializedProxy -> InitializedProxy(AABB) via c2MakeProxy(..., C2_TYPE_AABB, Some(&mut p))
- UninitializedProxy -> InitializedProxy(Capsule) via c2MakeProxy(..., C2_TYPE_CAPSULE, Some(&mut p))
- Any -> InvalidTypeOrNull when `type_0` is unknown or `shape`/`p` are null/None

**Evidence:** fn c2MakeProxy(shape: *const c_void, type_0: C2_TYPE, mut p: Option<&mut c2Proxy>); let p = p.as_deref_mut().unwrap(); (requires p is Some); C2_TYPE_CIRCLE arm: (shape as *const c2Circle).as_ref().unwrap(); (requires correct pointee type + non-null); C2_TYPE_AABB arm: let bb = (shape as *const c2AABB).as_ref(); then c2BBVerts(..., bb); (Option passed through; later unwrapped in c2BBVerts); C2_TYPE_CAPSULE arm: (shape as *const c2Capsule).as_ref().unwrap();; _ => {} (unknown type leaves proxy potentially uninitialized/stale)

**Implementation:** Replace the `(shape: *const c_void, type: C2_TYPE)` pair with a typed enum carrying references: `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB), Capsule(&'a c2Capsule) }`. Then `fn make_proxy(shape: ShapeRef<'_>) -> c2Proxy` (or `&mut c2Proxy`) becomes safe and total (no `_ => {}`), and removes the need for runtime tag agreement and pointer casts.

---

### 5. Non-empty vertex set requirement (count > 0 and within verts.len())

**Location**: `/data/test_case/lib.rs:1-702`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Support assumes there is at least one valid vertex to evaluate and that `count` does not exceed `verts.len()`. It unconditionally reads `verts[0]` to initialize `dmax`, so `count <= 0` or `verts` being empty will panic. It also iterates `for i in 1..count` after clamping `count` to >=0, but does not clamp to `verts.len()`, so an oversized `count` can also cause out-of-bounds indexing. In practice, callers (notably `c2GJK`) rely on `c2MakeProxy` to set `pA.count`/`pB.count` consistently with the fixed-size `verts` array, but the function signature itself cannot express the non-empty/in-range requirement.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2Capsule, 3 free function(s); struct c2v, 24 free function(s); struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2AABB, 2 free function(s); struct c2Circle, 1 free function(s)

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

    // Keep the original memory layout trick (a,b,c contiguous) but avoid repeated pointer math.
    let verts_ptr: *mut c2sv = &raw mut s.a;

    let mut cache_was_read = 0;
    if let Some(cache_ref) = cache.as_deref() {
        let cache_was_good = (cache_ref.count != 0) as i32;
        if cache_was_good != 0 {
            for i in 0..(cache_ref.count as isize) {
                let iA = cache_ref.iA[i as usize];
                let iB = cache_ref.iB[i as usize];

                let sA = c2Mulxv(ax, pA.verts[iA as usize]);
                let sB = c2Mulxv(bx, pB.verts[iB as usize]);

                let v = verts_ptr.offset(i).as_mut().unwrap();
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
                // keep behavior
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

        for i in 0..(save_count as isize) {
            let v = (verts_ptr.offset(i) as *const c2sv).read();
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

        let v_new = verts_ptr.offset(s.count as isize).as_mut().unwrap();
        v_new.iA = iA;
        v_new.sA = sA;
        v_new.iB = iB;
        v_new.sB = sB;
        v_new.p = c2Sub(v_new.sB, v_new.sA);

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
            let p_mid = c2Mulvs(c2Add(a, b), 0.5);
            a = p_mid;
            b = p_mid;
            dist = 0.0;
        }
    }

    if let Some(cache_mut) = cache.as_deref_mut() {
        cache_mut.metric = c2GJKSimplexMetric(Some(&s));
        cache_mut.count = s.count;

        for i in 0..(s.count as isize) {
            let v = verts_ptr.offset(i).as_ref().unwrap();
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
        &raw const B as *c
// ... (truncated) ...
```

**Entity:** c2Support(verts: &[c2v], count: i32, d: c2v)

**States:** EmptyOrOutOfRange, NonEmptyInRange

**Transitions:**
- EmptyOrOutOfRange -> NonEmptyInRange by ensuring count in 1..=verts.len() before calling c2Support

**Evidence:** c2Support: `let mut dmax = c2Dot(verts[0], d);` (requires verts non-empty); c2Support: `let count = count.max(0) as usize;` (allows 0, but still reads verts[0]); c2Support: `for i in 1..count { let dot = c2Dot(verts[i], d); ... }` (requires count <= verts.len()); c2GJK calls: `c2Support(&pA.verts, pA.count, ...)` and `pA.count` comes from c2MakeProxy

**Implementation:** Introduce a `NonEmptySlice<'a, T>` (or use `std::slice::SplitFirst`-style API) and a bounded count type. For example: `fn support(verts: &[c2v], d: c2v) -> usize` taking the full slice (no separate count), or `struct ProxyVerts<'a> { verts: &'a [c2v] }` created only with non-empty slices. For fixed capacity proxies, store `verts: [c2v; 8]` plus `count: NonZeroUsize` and expose `fn as_slice(&self) -> &[c2v]` returning `&self.verts[..count.get()]`.

---

