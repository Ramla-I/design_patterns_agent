# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 9
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 4
- **Precondition**: 4
- **Protocol**: 1
- **Modules analyzed**: 2

## State Machine Invariants

### 3. c2Simplex validity invariant (count selects active vertices; div must match simplex geometry)

**Location**: `/data/test_case/lib.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: c2Simplex stores up to 4 simplex vertices (a,b,c,d) but also carries runtime metadata (count, div) that implicitly defines which vertices are active and whether derived quantities are valid. Typical usage relies on the invariant that only the first `count` vertices are meaningful (e.g., count=1 => only `a` is used; count=2 => `a,b`; etc.), and that `count` is within 0..=4. Additionally, `div` appears to be a derived scalar (e.g., determinant/normalization factor) that is only meaningful when consistent with the active vertices. None of these constraints are enforced by the type system: all fields are public, and any `i32` can be assigned to `count`, allowing invalid states (negative, >4, mismatched `div`/vertices).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h; 1 free function(s)


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

**States:** Uninitialized/Invalid, Valid(1-vertex), Valid(2-vertex), Valid(3-vertex), Valid(4-vertex)

**Transitions:**
- Uninitialized/Invalid -> Valid(n-vertex) by setting count to n and populating the corresponding vertices (a..)
- Valid(k-vertex) -> Valid(k±1-vertex) by mutating count and updating vertices/div

**Evidence:** pub struct c2Simplex { ... pub a: c2sv, pub b: c2sv, pub c: c2sv, pub d: c2sv, pub div: f32, pub count: i32 } — fixed storage for 4 vertices plus runtime `count` selecting how many are active; field `count: i32` — unconstrained integer likely intended to be in a small range (0..=4) to index/select a,b,c,d; field `div: f32` — extra derived/normalization value whose correctness depends on the active vertices, but can be set independently because it is public

**Implementation:** Make `c2Simplex` opaque (private fields) and represent the active-vertex count at the type level: `struct Simplex<const N: usize> { verts: [c2sv; N], div: f32 }` with `N` in 1..=4, plus conversion/transition APIs like `fn add_vertex(self, v: c2sv) -> Simplex<{N+1}>` (or an enum `Simplex1|2|3|4`). Alternatively, use a `newtype` for `count` (e.g., `struct SimplexCount(u8);` validated to 0..=4) and expose safe constructors that keep `div` consistent.

---

### 6. c2Proxy initialization + valid-count protocol (Uninitialized -> Initialized[Circle/AABB/Capsule])

**Location**: `/data/test_case/lib.rs:1-725`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2Proxy has an implicit validity state encoded by the runtime field `count`. Many operations (notably `c2Support` and later indexing into `verts`) assume `count > 0` and that the first `count` entries of `verts` are initialized with the shape's support points. This is established by calling `c2MakeProxy` with a matching `(shape pointer, type)` pair. The type system does not couple `type_0` to the pointee type of `shape`, nor does it prevent using a proxy whose `count` is still 0 or inconsistent with the initialized verts.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h

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
pub const C2_TYPE_POLY: C2_TYPE = 3;
pub const C2_TYPE_CIRCLE: C2_TYPE = 1;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Manifold {
    pub count: i32,
    pub depths: [f32; 2],
    pub contact_points: [c2v; 2],
    pub n: c2v,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2h {
    pub n: c2v,
    pub d: f32,
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
pub(crate) fn c2Dist(h: c2h, p: c2v) -> f32 {
    c2Dot(h.n, p) - h.d
}

pub(crate) fn c2PlaneAt(p: Option<&c2Poly>, i: i32) -> c2h {
    let p = p.unwrap();
    let n = p.norms[i as usize];
    c2h {
        n,
        d: c2Dot(n, p.verts[i as usize]),
    }
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
        2 => {
            let bb = (shape as *const c2AABB).as_ref().unwrap();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, Some(bb));
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
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
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

#[inline]
pub(crate) fn c2MulxvT(a: c2x, b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

#[inline]
pub(crate) fn c2Intersect(a: c2v, b: c2v, da: f32, db: f32) -> c2v {
    c2Add(a, c2Mulvs(c2Sub(b, a), da / (da - db)))
}

fn c2Clip(seg: &mut [c2v], h: c2h) -> i32 {
    let mut out = [c2v { x: 0.0, y: 0.0 }; 2];
    let mut sp: usize = 0;

    let d0 = c2Dist(h, seg[0]);
    if d0 < 0.0 {
        out[sp] = seg[0];
        sp += 1;
    }

    let d1 = c2Dist(h, seg[1]);
    if d1 < 0.0 {
        if sp < 2 {
            out[sp] = seg[1];
            sp += 1;
        }
    }

    if d0 == 0.0 && d1 == 0.0 {
        out[0] = seg[0];
        out[1] = seg[1];
        sp = 2;
    } else if d0 * d1 <= 0.0 {
        if sp < 2 {
            out[sp] = c2Intersect(seg[0], seg[1], d0, d1);
            sp += 1;
        }
    }

    seg[0] = out[0];
    seg[1] = out[1];
    sp as i32
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

fn c2SidePlanes(seg: &mut [c2v], ra: c2v, rb: c2v, mut h: Option<&mut c2h>) -> i32 {
    let in_0 = c2Norm(c2Sub(rb, ra));
    let left = c2h {
        n: c2Neg(in_0),
        d: c2Dot(c2Neg(in_0), ra),
    };
    let right = c2h {
        n: in_0,
        d: c2Dot(in_0, rb),
    };

    if c2Clip(seg, left) < 2 {
        return 0;
    }
    if c2Clip(seg, right) < 2 {
        return 0;
    }

    if let Some(h) = h.as_deref_mut() {
        let n = c2CCW90(in_0);
        h.n = n;
        h.d = c2Dot(n, ra);
    }
    1
}

fn c2SidePlanesFromPoly(
    seg: &mut [c2v],
    x: c2x,
    p: Option<&c2Poly>,
    e: i32,
    h: Option<&mut c2h>,
) -> i32 {
    let p = p.unwrap();
    let ra = c2Mulxv(x, p.verts[e as usize]);
    let next = if e + 1 == p.count { 0 } else { e + 1 };
    let rb = c2Mulxv(x, p.verts[next as usize]);
    c2SidePlanes(seg, ra, rb, h)
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
    }
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
    }
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
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
    let count = count as usize;
    let mut imax = 0usize;
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

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;

        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).as_ref().unwrap();
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
        for i in 0..(save_count as usize) {
            if iA == saveA[i] && iB == saveB[i] {
            
// ... (truncated) ...
```

**Entity:** c2Proxy (as produced/consumed by c2MakeProxy/c2Support/c2GJK)

**States:** Uninitialized (count = 0, verts undefined/unused), Initialized: Circle (count = 1), Initialized: Capsule (count = 2), Initialized: AABB (count = 4)

**Transitions:**
- Uninitialized -> Initialized:* via unsafe fn c2MakeProxy(shape, type_0, &mut c2Proxy)
- Initialized:* -> UB/panic-risk if used with count == 0 or mismatched count/verts assumptions

**Evidence:** c2Proxy fields: `count: i32`, `verts: [c2v; 8]` encode how many vertices are valid at runtime; in c2GJK: `let mut pA = c2Proxy { ... count: 0, verts: ... }` then `c2MakeProxy(A, typeA, Some(&mut pA));` (proxy must be initialized before use); c2MakeProxy: `match type_0 { C2_TYPE_CIRCLE => { p.count = 1; p.verts[0] = c.p; } ... C2_TYPE_CAPSULE => { p.count = 2; ... } 2 => { p.count = 4; c2BBVerts(&mut p.verts, Some(bb)); } _ => {} }` (default `_` leaves proxy potentially uninitialized with count=0); c2Support(verts, count, d): uses `verts[0]` unconditionally and loops `for i in 1..count` after casting `count as usize` (requires `count >= 1` and consistent initialized prefix); in c2GJK: calls `c2Support(&pA.verts, pA.count, ...)` and later indexes `pA.verts[iA as usize]` (requires `0 <= iA < pA.count` and count valid)

**Implementation:** Replace `(shape: *const c_void, type_0: C2_TYPE)` with a typed enum `enum ShapeRef<'a> { Circle(&'a c2Circle), Capsule(&'a c2Capsule), Aabb(&'a c2AABB) }` and construct `Proxy<Circle>|Proxy<Capsule>|Proxy<Aabb>` where the vertex count is a const generic (e.g., `Proxy<const N: usize> { radius: f32, verts: [c2v; N] }`). Then `support()` can take `&Proxy<N>` and never accept `count=0` or mismatched shape/type.

---

### 7. c2Simplex internal-count invariants (count-driven variant + div != 0)

**Location**: `/data/test_case/lib.rs:1-725`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: c2Simplex is treated as a tagged union controlled by the runtime integer `count`. Multiple functions branch on `count` and then assume a specific subset of fields (`a`, `b`, `c`) are initialized and that `div` is non-zero for barycentric normalization. This protocol is maintained by carefully setting `count`/`div` in `c22` and `c23`, but the type system cannot prevent calling `c2L/c2D/c2Witness/c2GJKSimplexMetric` with an invalid `count` (e.g., 0 or >3) or with `div == 0.0` when `count` is 2/3, which would cause division-by-zero/NaNs and out-of-protocol reads.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h

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
pub const C2_TYPE_POLY: C2_TYPE = 3;
pub const C2_TYPE_CIRCLE: C2_TYPE = 1;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Manifold {
    pub count: i32,
    pub depths: [f32; 2],
    pub contact_points: [c2v; 2],
    pub n: c2v,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2h {
    pub n: c2v,
    pub d: f32,
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
pub(crate) fn c2Dist(h: c2h, p: c2v) -> f32 {
    c2Dot(h.n, p) - h.d
}

pub(crate) fn c2PlaneAt(p: Option<&c2Poly>, i: i32) -> c2h {
    let p = p.unwrap();
    let n = p.norms[i as usize];
    c2h {
        n,
        d: c2Dot(n, p.verts[i as usize]),
    }
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
        2 => {
            let bb = (shape as *const c2AABB).as_ref().unwrap();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, Some(bb));
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
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
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

#[inline]
pub(crate) fn c2MulxvT(a: c2x, b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

#[inline]
pub(crate) fn c2Intersect(a: c2v, b: c2v, da: f32, db: f32) -> c2v {
    c2Add(a, c2Mulvs(c2Sub(b, a), da / (da - db)))
}

fn c2Clip(seg: &mut [c2v], h: c2h) -> i32 {
    let mut out = [c2v { x: 0.0, y: 0.0 }; 2];
    let mut sp: usize = 0;

    let d0 = c2Dist(h, seg[0]);
    if d0 < 0.0 {
        out[sp] = seg[0];
        sp += 1;
    }

    let d1 = c2Dist(h, seg[1]);
    if d1 < 0.0 {
        if sp < 2 {
            out[sp] = seg[1];
            sp += 1;
        }
    }

    if d0 == 0.0 && d1 == 0.0 {
        out[0] = seg[0];
        out[1] = seg[1];
        sp = 2;
    } else if d0 * d1 <= 0.0 {
        if sp < 2 {
            out[sp] = c2Intersect(seg[0], seg[1], d0, d1);
            sp += 1;
        }
    }

    seg[0] = out[0];
    seg[1] = out[1];
    sp as i32
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

fn c2SidePlanes(seg: &mut [c2v], ra: c2v, rb: c2v, mut h: Option<&mut c2h>) -> i32 {
    let in_0 = c2Norm(c2Sub(rb, ra));
    let left = c2h {
        n: c2Neg(in_0),
        d: c2Dot(c2Neg(in_0), ra),
    };
    let right = c2h {
        n: in_0,
        d: c2Dot(in_0, rb),
    };

    if c2Clip(seg, left) < 2 {
        return 0;
    }
    if c2Clip(seg, right) < 2 {
        return 0;
    }

    if let Some(h) = h.as_deref_mut() {
        let n = c2CCW90(in_0);
        h.n = n;
        h.d = c2Dot(n, ra);
    }
    1
}

fn c2SidePlanesFromPoly(
    seg: &mut [c2v],
    x: c2x,
    p: Option<&c2Poly>,
    e: i32,
    h: Option<&mut c2h>,
) -> i32 {
    let p = p.unwrap();
    let ra = c2Mulxv(x, p.verts[e as usize]);
    let next = if e + 1 == p.count { 0 } else { e + 1 };
    let rb = c2Mulxv(x, p.verts[next as usize]);
    c2SidePlanes(seg, ra, rb, h)
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
    }
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
    }
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
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
    let count = count as usize;
    let mut imax = 0usize;
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

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;

        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).as_ref().unwrap();
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
        for i in 0..(save_count as usize) {
            if iA == saveA[i] && iB == saveB[i] {
            
// ... (truncated) ...
```

**Entity:** c2Simplex (as mutated by c22/c23 and read by c2GJKSimplexMetric/c2D/c2L/c2Witness)

**States:** Point simplex (count = 1), Segment simplex (count = 2), Triangle simplex (count = 3)

**Transitions:**
- Point (1) -> Segment (2) via c22() setting `s.count = 2`
- Segment (2) -> Point (1) via c22() setting `s.count = 1` (dropping a vertex)
- Triangle (3) -> Point/Segment/Triangle via c23() setting `s.count` to 1/2/3
- Any -> Invalid if `count`/`div` become inconsistent (not prevented by types)

**Evidence:** c2Simplex fields: `div: f32`, `count: i32` plus fixed slots `a,b,c,d` (runtime tag + payload); c2GJKSimplexMetric: `match s.count { 2 => ..., 3 => ..., _ => 0.0 }` (count interpreted as variant tag); c2D: `match s.count { 1 => ..., 2 => ..., _ => c2V(0.0,0.0) }` (behavior depends on count); c2L: computes `let den = 1.0 / s.div; match s.count { 1 => ..., 2 => ... }` (requires `s.div != 0` when count==2); c2Witness: `let den = 1.0 / s.div; match s.count { 2 => ... den*..., 3 => ... den*... }` (requires `s.div != 0` for count 2/3); c22 and c23 explicitly set `s.div` and `s.count` together (e.g., `s.div = 1.0; s.count = 1;` and `s.div = u+v; s.count = 2;`), indicating an intended invariant coupling the fields

**Implementation:** Model simplex as an enum with variants carrying only valid fields and non-zero denominators, e.g. `enum Simplex { Pt{a: c2sv}, Seg{a: c2sv,b: c2sv, div: NonZeroF32}, Tri{a: c2sv,b: c2sv,c: c2sv, div: NonZeroF32} }`. Then make `c22/c23` consume and return a new `Simplex` variant, and restrict `witness()/l()/d()/metric()` to the variants that support them without runtime `count` checks or `1.0/s.div` on potentially-zero values.

---

### 1. c2Manifold contact-count validity (0..=2) and active-slice protocol

**Location**: `/data/test_case/lib.rs:1-11`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Manifold encodes how many contact points are valid via the runtime integer field `count`. Only the first `count` entries of `depths` and `contact_points` are logically initialized/meaningful, and `n` is the contact normal associated with those points. The type system does not prevent constructing a manifold with an out-of-range `count` (e.g., 3 or -1), nor does it prevent reading/writing elements beyond the active prefix implied by `count`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h; 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Manifold {
    pub count: i32,
    pub depths: [f32; 2],
    pub contact_points: [c2v; 2],
    pub n: c2v,
}

```

**Entity:** c2Manifold

**States:** Empty (count=0), SingleContact (count=1), TwoContacts (count=2), Invalid (count<0 or count>2)

**Transitions:**
- Empty -> SingleContact by setting count from 0 to 1
- SingleContact -> TwoContacts by setting count from 1 to 2
- AnyValid -> Empty by setting count to 0
- AnyValid -> Invalid by setting count outside 0..=2

**Evidence:** struct c2Manifold: `pub count: i32` — runtime field indicating how many contacts are present; struct c2Manifold: `pub depths: [f32; 2]` and `pub contact_points: [c2v; 2]` — fixed-capacity arrays whose logical length must match `count`; struct c2Manifold: `pub n: c2v` — normal vector implicitly tied to the active contacts but not conditioned by type on `count`

**Implementation:** Replace `count: i32` with a validated domain type, e.g. `struct ContactCount(u8);` that can only be 0..=2 (or an enum `enum ContactCount { Zero, One, Two }`). Provide APIs returning slices: `fn points(&self) -> &[c2v]` and `fn depths(&self) -> &[f32]` that slice to `count` (and constructors that require consistent data), eliminating out-of-range and over-read/over-write usage.

---

## Precondition Invariants

### 2. c2AABB validity invariant (min <= max per axis; non-negative extents)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2AABB implicitly represents an axis-aligned bounding box where `min` is expected to be component-wise <= `max` (x and y), yielding non-negative extents. The struct is `pub` with unconstrained fields, so callers can construct an 'invalid' AABB (e.g., min.x > max.x), and nothing in the type system prevents passing such a value to other geometry routines that likely assume validity. This is a value-level invariant (not a lifecycle), and it is currently entirely unenforced at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h; 1 free function(s)


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
- Invalid -> Valid via constructing/normalizing/sorting endpoints (not present in snippet)

**Evidence:** pub struct c2AABB { pub min: c2v, pub max: c2v } — field naming implies an ordering invariant between `min` and `max`; #[repr(C)] on c2AABB — suggests this is a data layout type used across FFI/geometry code where such invariants are typically assumed rather than rechecked; #[derive(Copy, Clone)] — cheap copying increases likelihood invalid values propagate unless validity is encoded in the type

**Implementation:** Make fields private and expose `pub struct Aabb(c2AABB);` or `pub struct ValidAabb { min: c2v, max: c2v }` with a smart constructor `try_new(min, max) -> Result<ValidAabb, Error>` that checks component-wise ordering (or a `new_sorted(a, b)` that sorts per-axis). Provide `as_raw(&self) -> c2AABB` for FFI. This keeps raw `c2AABB` for interop but encourages internal code to require `ValidAabb`.

---

### 5. c2Poly vertex-count validity (count bounds + active prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Poly encodes how many vertices/normals are logically present via the runtime field `count`, while `verts`/`norms` are fixed-size arrays of length 8. Correct usage implicitly requires that `count` is within 0..=8 and that only the prefix 0..count of `verts` and `norms` is considered initialized/meaningful (and likely paired by index). None of these constraints are enforced by the type system; any i32 value can be stored in `count`, including negative values or values > 8, and nothing ties `count` to which elements of the arrays are valid to read.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2h; 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
}

```

**Entity:** c2Poly

**States:** Valid(count in 0..=8, verts/norms initialized for 0..count), Invalid(count out of range or active data not initialized)

**Transitions:**
- Invalid -> Valid by setting count into 0..=8 and initializing verts/norms[0..count] consistently

**Evidence:** struct field `count: i32` (runtime length, can be negative/out of range); struct fields `verts: [c2v; 8]` and `norms: [c2v; 8]` (fixed-capacity storage implies `count` selects an active prefix)

**Implementation:** Replace `count: i32` with a validated count type (e.g., `struct VertexCount(u8)` with `TryFrom<i32>` ensuring 0..=8), and/or model vertices as a sized wrapper like `struct Poly<const N: usize> { verts: [c2v; N], norms: [c2v; N] }` plus constructors that only allow N<=8; alternatively store `count: u8` and keep `verts/norms` in `[MaybeUninit<c2v>; 8]` with safe accessors returning `&[c2v]` sliced to `count`.

---

### 4. c2Proxy shape-vertex validity invariant (count bounds + active prefix)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2Proxy encodes a variable-length vertex list using a fixed-size array `verts: [c2v; 8]` plus a runtime `count: i32`. The intended invariant is that `count` is within 0..=8 and only the prefix `verts[0..count)` is considered part of the shape; the remaining elements are ignored. None of this is enforced by the type system: `count` can be negative or > 8, and all 8 vertices are always present so code may accidentally read/write vertices that are not logically part of the proxy. This also implies a secondary invariant that `radius` should be non-negative, but that is likewise not enforced.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Poly, 1 free function(s); struct c2h; 1 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Proxy {
    pub radius: f32,
    pub count: i32,
    pub verts: [c2v; 8],
}

```

**Entity:** c2Proxy

**States:** Valid(count in 0..=8, verts[0..count) initialized), Invalid(count out of range or verts beyond count relied upon)

**Transitions:**
- Invalid -> Valid via construction/initialization that clamps/validates `count` and `radius`
- Valid -> Invalid via mutation setting `count` out of bounds or `radius` negative

**Evidence:** `pub count: i32` runtime length field for a vertex list; `pub verts: [c2v; 8]` fixed-capacity backing storage implying only a prefix is meaningful; `pub radius: f32` likely required to be >= 0.0 but unconstrained by type

**Implementation:** Replace `count: i32` with a constrained type like `u8` (or `NonZeroU8` if empty disallowed) and enforce `<= 8` at construction: e.g., `struct VertexCount(u8); impl TryFrom<usize> for VertexCount` that rejects > 8. Expose vertices as `&[c2v]`/`&mut [c2v]` derived from the count, or store `Vec<c2v>`/`SmallVec<[c2v; 8]>` to avoid the split length+array invariant. Wrap `radius` in a `NonNegativeF32` newtype validated at creation.

---

### 9. c2Circle validity invariant (non-negative, finite radius; finite position)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: c2Circle is a plain C-layout POD with public fields. Callers are implicitly expected to maintain geometric validity: the center `p` should contain finite coordinates and the radius `r` should be finite and non-negative (and typically > 0 for many algorithms). None of these constraints are enforced by the type system because `r` is an unconstrained `f32` and fields are public. This allows construction of invalid circles (NaN/inf radius, negative radius) that downstream collision/math code likely assumes cannot happen.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h; 1 free function(s)


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
- Invalid -> Valid via constructing/validating a circle (not present in snippet)
- Valid -> Invalid via direct field mutation of `p` or `r` (public fields)

**Evidence:** line 7-11: `pub struct c2Circle { pub p: c2v, pub r: f32 }` exposes unconstrained `f32` radius and public mutation of invariants; line 5: `#[repr(C)]` indicates FFI/POD usage where invariants are typically maintained by convention rather than types

**Implementation:** Make fields private and provide `c2Circle::new(p, r: Radius)` where `Radius` is a newtype validated to be finite and >= 0 (or > 0 as required). Optionally introduce `FiniteF32` (or `NonNaNF32`) wrappers for components used by `c2v`, and expose getters/setters that preserve the invariant.

---

## Protocol Invariants

### 8. c2GJKCache validity protocol (Empty/Invalid -> Valid and index-bounds coupling)

**Location**: `/data/test_case/lib.rs:1-725`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: c2GJKCache is an optional acceleration structure whose usability is encoded by `count` and additional coupled fields (`iA/iB` indices into proxy verts, `div`, `metric`). When `cache.count != 0`, c2GJK assumes `0 <= count <= 3` and that each `iA[i]`/`iB[i]` is in-bounds for `pA.count`/`pB.count` (and non-negative), because it immediately indexes `pA.verts[iA as usize]` and `pB.verts[iB as usize]`. Then it runs a metric consistency check and conditionally treats the cache as read (`cache_was_read = 1`) or ignores it and reinitializes simplex. None of these coupling/bounds requirements are expressed in types; they're enforced only by runtime integer checks and implicit assumptions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct c2v, 34 free function(s); struct c2Manifold, 9 free function(s); struct c2Capsule; struct c2AABB; struct c2Circle; struct c2GJKCache; struct c2x, 1 free function(s); struct c2r, 1 free function(s); struct c2Simplex, 3 free function(s); struct c2sv; struct c2Proxy, 1 free function(s); struct c2Poly, 1 free function(s); struct c2h

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
pub const C2_TYPE_POLY: C2_TYPE = 3;
pub const C2_TYPE_CIRCLE: C2_TYPE = 1;
pub const C2_TYPE_CAPSULE: C2_TYPE = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2v {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Manifold {
    pub count: i32,
    pub depths: [f32; 2],
    pub contact_points: [c2v; 2],
    pub n: c2v,
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2Poly {
    pub count: i32,
    pub verts: [c2v; 8],
    pub norms: [c2v; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct c2h {
    pub n: c2v,
    pub d: f32,
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
pub(crate) fn c2Dist(h: c2h, p: c2v) -> f32 {
    c2Dot(h.n, p) - h.d
}

pub(crate) fn c2PlaneAt(p: Option<&c2Poly>, i: i32) -> c2h {
    let p = p.unwrap();
    let n = p.norms[i as usize];
    c2h {
        n,
        d: c2Dot(n, p.verts[i as usize]),
    }
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
        2 => {
            let bb = (shape as *const c2AABB).as_ref().unwrap();
            p.radius = 0.0;
            p.count = 4;
            c2BBVerts(&mut p.verts, Some(bb));
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
pub(crate) fn c2MulrvT(a: c2r, b: c2v) -> c2v {
    c2V(a.c * b.x + a.s * b.y, -a.s * b.x + a.c * b.y)
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

#[inline]
pub(crate) fn c2MulxvT(a: c2x, b: c2v) -> c2v {
    c2MulrvT(a.r, c2Sub(b, a.p))
}

#[inline]
pub(crate) fn c2Intersect(a: c2v, b: c2v, da: f32, db: f32) -> c2v {
    c2Add(a, c2Mulvs(c2Sub(b, a), da / (da - db)))
}

fn c2Clip(seg: &mut [c2v], h: c2h) -> i32 {
    let mut out = [c2v { x: 0.0, y: 0.0 }; 2];
    let mut sp: usize = 0;

    let d0 = c2Dist(h, seg[0]);
    if d0 < 0.0 {
        out[sp] = seg[0];
        sp += 1;
    }

    let d1 = c2Dist(h, seg[1]);
    if d1 < 0.0 {
        if sp < 2 {
            out[sp] = seg[1];
            sp += 1;
        }
    }

    if d0 == 0.0 && d1 == 0.0 {
        out[0] = seg[0];
        out[1] = seg[1];
        sp = 2;
    } else if d0 * d1 <= 0.0 {
        if sp < 2 {
            out[sp] = c2Intersect(seg[0], seg[1], d0, d1);
            sp += 1;
        }
    }

    seg[0] = out[0];
    seg[1] = out[1];
    sp as i32
}

#[inline]
pub(crate) fn c2Div(a: c2v, b: f32) -> c2v {
    c2Mulvs(a, 1.0 / b)
}

#[inline]
pub(crate) fn c2Norm(a: c2v) -> c2v {
    c2Div(a, c2Len(a))
}

#[inline]
pub(crate) fn c2Neg(a: c2v) -> c2v {
    c2V(-a.x, -a.y)
}

#[inline]
pub(crate) fn c2CCW90(a: c2v) -> c2v {
    c2V(a.y, -a.x)
}

fn c2SidePlanes(seg: &mut [c2v], ra: c2v, rb: c2v, mut h: Option<&mut c2h>) -> i32 {
    let in_0 = c2Norm(c2Sub(rb, ra));
    let left = c2h {
        n: c2Neg(in_0),
        d: c2Dot(c2Neg(in_0), ra),
    };
    let right = c2h {
        n: in_0,
        d: c2Dot(in_0, rb),
    };

    if c2Clip(seg, left) < 2 {
        return 0;
    }
    if c2Clip(seg, right) < 2 {
        return 0;
    }

    if let Some(h) = h.as_deref_mut() {
        let n = c2CCW90(in_0);
        h.n = n;
        h.d = c2Dot(n, ra);
    }
    1
}

fn c2SidePlanesFromPoly(
    seg: &mut [c2v],
    x: c2x,
    p: Option<&c2Poly>,
    e: i32,
    h: Option<&mut c2h>,
) -> i32 {
    let p = p.unwrap();
    let ra = c2Mulxv(x, p.verts[e as usize]);
    let next = if e + 1 == p.count { 0 } else { e + 1 };
    let rb = c2Mulxv(x, p.verts[next as usize]);
    c2SidePlanes(seg, ra, rb, h)
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
    }
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
    }
}

#[inline]
pub(crate) fn c2Skew(a: c2v) -> c2v {
    c2V(-a.y, a.x)
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
    let count = count as usize;
    let mut imax = 0usize;
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

    let mut saveA = [0i32; 3];
    let mut saveB = [0i32; 3];

    let mut d0 = f32::MAX;
    let mut iter = 0i32;
    let mut hit = 0i32;

    while iter < 20 {
        let save_count = s.count;

        for i in 0..(save_count as isize) {
            let v = (verts.offset(i) as *const c2sv).as_ref().unwrap();
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
        for i in 0..(save_count as usize) {
            if iA == saveA[i] && iB == saveB[i] {
            
// ... (truncated) ...
```

**Entity:** c2GJKCache (as consumed/produced by c2GJK)

**States:** Empty/Invalid (count = 0), Candidate cache (count != 0 but may be rejected by metric check), Accepted/Applied cache (cache_was_read = 1; simplex seeded from cache)

**Transitions:**
- Empty/Invalid -> Candidate via user providing a nonzero `cache.count`
- Candidate -> Accepted/Applied via metric check setting `cache_was_read = 1` and copying indices into simplex verts
- Candidate -> Rejected/ignored when metric check fails and `cache_was_read` remains 0
- Rejected/Empty -> Seeded simplex via fallback initialization in c2GJK (`s.count = 1`, `s.div = 1.0`)

**Evidence:** c2GJKCache fields: `count: i32`, `iA: [i32; 3]`, `iB: [i32; 3]`, `div: f32`, `metric: f32` (multiple values must be mutually consistent); in c2GJK: `let cache_was_good = (cache.count != 0) as i32; if cache_was_good != 0 { for i in 0..(cache.count as isize) { let iA = cache.iA[i]; let iB = cache.iB[i]; ... let sA = ... pA.verts[iA as usize]; let sB = ... pB.verts[iB as usize]; ... } s.count = cache.count; s.div = cache.div; ... }` (requires bounds: cache.count <= 3, iA/iB within proxy counts, and non-negative); in c2GJK: cache acceptance is tracked by `cache_was_read` integer flag and metric comparisons using `cache.metric` and `c2GJKSimplexMetric(Some(&s))`; fallback when cache not read: `if cache_was_read == 0 { ... s.div = 1.0; s.count = 1; }` (explicit protocol step when cache is absent/invalid)

**Implementation:** Introduce a validated cache type constructed only through checked APIs: `struct ValidGjkCache { count: NonZeroU8 /*1..=3*/, iA: [u8;3], iB: [u8;3], div: NonZeroF32, metric: f32 }` plus a `fn validate(&c2GJKCache, pA_count: u8, pB_count: u8) -> Option<ValidGjkCache>` that enforces bounds/non-negativity. Then `c2GJK` accepts `Option<ValidGjkCache>` instead of `Option<&mut c2GJKCache>`, eliminating unchecked indexing from raw cache data.

---

