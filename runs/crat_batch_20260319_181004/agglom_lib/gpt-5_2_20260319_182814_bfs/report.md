# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 3. RNG seeding/valid-handle invariant (cn_rnd_t must be present and initialized)

**Location**: `/data/test_case/lib.rs:1-356`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: f4 requires a mutable RNG state to be provided; it panics if `None` is passed. Additionally, cn_rnd_next assumes `cn_rnd_t.state` contains a valid (seeded) internal state for the xorshift-like transition; nothing in the type ensures the state was ever seeded to a non-degenerate value. The API encodes handle presence and readiness as runtime conventions (Option + panic) rather than types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct lm_vec2, 4 free function(s); struct cn_rnd_t, 2 free function(s); struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); struct c2Circle, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type tflac_u32 = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub flt: f32,
    pub num: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lm_vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cn_rnd_t {
    pub state: [u64; 2],
}

pub type C2_TYPE = u32;
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
pub struct c2v {
    pub x: f32,
    pub y: f32,
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
pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    let r2 = r * r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let separated = (B.max.x < A.min.x) || (A.max.x < B.min.x) || (B.max.y < A.min.y) || (A.max.y < B.min.y);
    (!separated) as i32
}

pub(crate) unsafe fn f2(
    A: *const std::ffi::c_void,
    typeA: C2_TYPE,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    unsafe fn read<T: Copy>(p: *const std::ffi::c_void) -> T {
        // Preserve original behavior (unaligned reads possible in C); use read_unaligned.
        (p as *const T).read_unaligned()
    }

    match (typeA, typeB) {
        (C2_TYPE_CIRCLE, C2_TYPE_CIRCLE) => c2CircletoCircle(read::<c2Circle>(A), read::<c2Circle>(B)),
        (C2_TYPE_CIRCLE, C2_TYPE_AABB) => c2CircletoAABB(read::<c2Circle>(A), read::<c2AABB>(B)),
        (C2_TYPE_AABB, C2_TYPE_CIRCLE) => c2CircletoAABB(read::<c2Circle>(B), read::<c2AABB>(A)),
        (C2_TYPE_AABB, C2_TYPE_AABB) => c2AABBtoAABB(read::<c2AABB>(A), read::<c2AABB>(B)),
        _ => 0,
    }
}

pub(crate) fn f3(v1: i32, v2: i32) -> i32 {
    if v2 == 0 {
        return 0;
    }
    let q: i32;
    let r: i32;
    if v1 >= 0 {
        if v2 >= 0 {
            return v1 / v2;
        } else if v2 != i32::MIN {
            q = -(v1 / -v2);
            r = v1 % -v2;
        } else {
            q = 0;
            r = v1;
        }
    } else if v1 != i32::MIN {
        if v2 >= 0 {
            q = -(-v1 / v2);
            r = -(-v1 % v2);
        } else if v2 != i32::MIN {
            q = -v1 / -v2;
            r = -(-v1 % -v2);
        } else {
            q = 1;
            r = v1 - q * v2;
        }
    } else if v2 >= 0 {
        q = -(-(v1 + v2) / v2) - 1;
        r = -(-(v1 + v2) % v2);
    } else if v2 != i32::MIN {
        q = -(v1 - v2) / -v2 + 1;
        r = -(-(v1 - v2) % -v2);
    } else {
        q = 1;
        r = 0;
    }
    if r >= 0 {
        q
    } else {
        q + if v2 > 0 { -1 } else { 1 }
    }
}

#[inline]
fn cn_rnd_next(rnd: &mut cn_rnd_t) -> u64 {
    let mut x = rnd.state[0];
    let y = rnd.state[1];
    rnd.state[0] = y;
    x ^= x << 23;
    x ^= x >> 17;
    x ^= y ^ (y >> 26);
    rnd.state[1] = x;
    x.wrapping_add(y)
}

pub(crate) fn f4(rnd: Option<&mut cn_rnd_t>) -> f64 {
    let rnd = rnd.expect("cn_rnd_t must be provided");
    let value = cn_rnd_next(rnd);
    let exponent: u64 = 1023;
    let mantissa: u64 = value >> 12;
    let bits: u64 = (exponent << 52) | mantissa;
    f64::from_bits(bits) - 1.0
}

pub(crate) fn f5(mut a: u32) -> u32 {
    a = (a & 0xaaaau32) >> 1 | (a & 0x5555u32) << 1;
    a = (a & 0xccccu32) >> 2 | (a & 0x3333u32) << 2;
    a = (a & 0xf0f0u32) >> 4 | (a & 0x0f0fu32) << 4;
    a = (a & 0xff00u32) >> 8 | (a & 0x00ffu32) << 8;
    a
}

pub(crate) fn f7(blocksize: tflac_u32, channels: tflac_u32, bitdepth: tflac_u32) -> tflac_u32 {
    18u32.wrapping_add(channels).wrapping_add(
        blocksize
            .wrapping_mul(bitdepth)
            .wrapping_mul(channels.wrapping_mul((channels != 2) as u32))
            .wrapping_add(blocksize.wrapping_mul(bitdepth).wrapping_mul((channels == 2) as u32))
            .wrapping_add(
                blocksize
                    .wrapping_mul(bitdepth.wrapping_add((bitdepth != 32) as u32))
                    .wrapping_mul((channels == 2) as u32),
            )
            .wrapping_add(7)
            .wrapping_div(8),
    )
}

#[inline]
fn lm_v2(x: f32, y: f32) -> lm_vec2 {
    lm_vec2 { x, y }
}

#[inline]
fn lm_sub2(a: lm_vec2, b: lm_vec2) -> lm_vec2 {
    lm_v2(a.x - b.x, a.y - b.y)
}

#[inline]
fn lm_dot2(a: lm_vec2, b: lm_vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

pub(crate) fn f9(p1: lm_vec2, p2: lm_vec2, p3: lm_vec2, p: lm_vec2) -> lm_vec2 {
    let v0 = lm_sub2(p3, p1);
    let v1 = lm_sub2(p2, p1);
    let v2 = lm_sub2(p, p1);
    let dot00 = lm_dot2(v0, v0);
    let dot01 = lm_dot2(v0, v1);
    let dot02 = lm_dot2(v0, v2);
    let dot11 = lm_dot2(v1, v1);
    let dot12 = lm_dot2(v1, v2);
    let invDenom = 1.0f32 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * invDenom;
    let v = (dot00 * dot12 - dot01 * dot02) * invDenom;
    lm_v2(u, v)
}

static m__mantissa: [u32; 2048] = [
    0, 0x33800000, 0x34000000, 0x34400000, 0x34800000, 0x34a00000, 0x34c00000, 0x34e00000,
    0x35000000, 0x35100000, 0x35200000, 0x35300000, 0x35400000, 0x35500000, 0x35600000, 0x35700000,
    0x35800000, 0x35880000, 0x35900000, 0x35980000, 0x35a00000, 0x35a80000, 0x35b00000, 0x35b80000,
    0x35c00000, 0x35c80000, 0x35d00000, 0x35d80000, 0x35e00000, 0x35e80000, 0x35f00000, 0x35f80000,
    0x36000000, 0x36040000, 0x36080000, 0x360c0000, 0x36100000, 0x36140000, 0x36180000, 0x361c0000,
    0x36200000, 0x36240000, 0x36280000, 0x362c0000, 0x36300000, 0x36340000, 0x36380000, 0x363c0000,
    0x36400000, 0x36440000, 0x36480000, 0x364c0000, 0x36500000, 0x36540000, 0x36580000, 0x365c0000,
    0x36600000, 0x36640000, 0x36680000, 0x366c0000, 0x36700000, 0x36740000, 0x36780000, 0x367c0000,
    0x36800000, 0x36820000, 0x36840000, 0x36860000, 0x36880000, 0x368a0000, 0x368c0000, 0x368e0000,
    0x36900000, 0x36920000, 0x36940000, 0x36960000, 0x36980000, 0x369a0000, 0x369c0000, 0x369e0000,
    0x36a00000, 0x36a20000, 0x36a40000, 0x36a60000, 0x36a80000, 0x36aa0000, 0x36ac0000, 0x36ae0000,
    0x36b00000, 0x36b20000, 0x36b40000, 0x36b60000, 0x36b80000, 0x36ba0000, 0x36bc0000, 0x36be0000,
    0x36c00000, 0x36c20000, 0x36c40000, 0x36c60000, 0x36c80000, 0x36ca0000, 0x36cc0000, 0x36ce0000,
    0x36d00000, 0x36d20000, 0x36d40000, 0x36d60000, 0x36d80000, 0x36da0000, 0x36dc0000, 0x36de0000,
    0x36e00000, 0x36e20000, 0x36e40000, 0x36e60000, 0x36e80000, 0x36ea0000, 0x36ec0000, 0x36ee0000,
    0x36f00000, 0x36f20000, 0x36f40000, 0x36f60000, 0x36f80000, 0x36fa0000, 0x36fc0000, 0x36fe0000,
    0x37000000, 0x37010000, 0x37020000, 0x37030000, 0x37040000, 0x37050000, 0x37060000, 0x37070000,
    0x37080000, 0x37090000, 0x370a0000, 0x370b0000, 0x370c0000, 0x370d0000, 0x370e0000, 0x370f0000,
    0x37100000, 0x37110000, 0x37120000, 0x37130000, 0x37140000, 0x37150000, 0x37160000, 0x37170000,
    0x37180000, 0x37190000, 0x371a0000, 0x371b0000, 0x371c0000, 0x371d0000, 0x371e0000, 0x371f0000,
    0x37200000, 0x37210000, 0x37220000, 0x37230000, 0x37240000, 0x37250000, 0x37260000, 0x37270000,
    0x37280000, 0x37290000, 0x372a0000, 0x372b0000, 0x372c0000, 0x372d0000, 0x372e0000, 0x372f0000,
    0x37300000, 0x37310000, 0x37320000, 0x37330000, 0x37340000, 0x37350000, 0x37360000, 0x37370000,
    0x37380000, 0x37390000, 0x373a0000, 0x373b0000, 0x373c0000, 0x373d0000, 0x373e0000, 0x373f0000,
    0x37400000, 0x37410000, 0x37420000, 0x37430000, 0x37440000, 0x37450000, 0x37460000, 0x37470000,
    0x37480000, 0x37490000, 0x374a0000, 0x374b0000, 0x374c0000, 0x374d0000, 0x374e0000, 0x374f0000,
    0x37500000, 0x37510000, 0x37520000, 0x37530000, 0x37540000, 0x37550000, 0x37560000, 0x37570000,
    0x37580000, 0x37590000, 0x375a0000, 0x375b0000, 0x375c0000, 0x375d0000, 0x375e0000, 0x375f0000,
    0x37600000, 0x37610000, 0x37620000, 0x37630000, 0x37640000, 0x37650000, 0x37660000, 0x37670000,
    0x37680000, 0x37690000, 0x376a0000, 0x376b0000, 0x376c0000, 0x376d0000, 0x376e0000, 0x376f0000,
    0x37700000, 0x37710000, 0x37720000, 0x37730000, 0x37740000, 0x37750000, 0x37760000, 0x37770000,
    0x37780000, 0x37790000, 0x377a0000, 0x377b0000, 0x377c0000, 0x377d0000, 0x377e0000, 0x377f0000,
    0x37800000, 0x37808000, 0x37810000, 0x37818000, 0x37820000, 0x37828000, 0x37830000, 0x37838000,
    0x37840000, 0x37848000, 0x37850000, 0x37858000, 0x37860000, 0x37868000, 0x37870000, 0x37878000,
    0x37880000, 0x37888000, 0x37890000, 0x37898000, 0x378a0000, 0x378a8000, 0x378b0000, 0x378b8000,
    0x378c0000, 0x378c8000, 0x378d0000, 0x378d8000, 0x378e0000, 0x378e8000, 0x378f0000, 0x378f8000,
    0x37900000, 0x37908000, 0x37910000, 0x37918000, 0x37920000, 0x37928000, 0x37930000, 0x37938000,
    0x37940000, 0x37948000, 0x37950000, 0x37958000, 0x37960000, 0x37968000, 0x37970000, 0x37978000,
    0x37980000, 0x37988000, 0x37990000, 0x37998000, 0x379a0000, 0x379a8000, 0x379b0000, 0x379b8000,
    0x379c0000, 0x379c8000, 0x379d0000, 0x379d8000, 0x379e0000, 0x379e8000, 0x379f0000, 0x379f8000,
    0x37a00000, 0x37a08000, 0x37a10000, 0x37a18000, 0x37a20000, 0x37a28000, 0x37a30000, 0x37a38000,
    0x37a40000, 0x37a48000, 0x37a50000, 0x37a58000, 0x37a60000, 0x37a68000, 0x37a70000, 0x37a78000,
    0x37a80000, 0x37a88000, 0x37a90000, 0x37a98000, 0x37aa0000, 0x37aa8000, 0x37ab0000, 0x37ab8000,
    0x37ac0000, 0x37ac8000, 0x37ad0000, 0x37ad8000, 0x37ae0000, 0x37ae8000, 0x37af0000, 0x37af8000,
    0x37b00000, 0x37b08000, 0x37b10000, 0x37b18000, 0x37b20000, 0x37b28000, 0x37b30000, 0x37b38000,
    0x37b40000, 0x37b48000, 0x37b50000, 0x37b58000, 0x37b60000, 0x37b68000, 0x37b70000, 0x37b78000,
    0x37b80000, 0x37b88000, 0x37b90000, 0x37b98000, 0x37ba0000, 0x37ba8000, 0x37bb0000, 0x37bb8000,
    0x37bc0000, 0x37bc8000, 0x37bd0000, 0x37bd8000, 0x37be0000, 0x37be8000, 0x37bf0000, 0x37bf8000,
    0x37c00000, 0x37c08000, 0x37c10000, 0x37c18000, 0x37c20000, 0x37c28000, 0x37c30000, 0x37c38000,
    0x37c40000, 0x37c48000, 0x37c50000, 0x37c58000, 0x37c60000, 0x37c68000, 0x37c70000, 0x37c78000,
    0x37c80000, 0x37c88000, 0x37c90000, 0x37c98000, 0x37ca0000, 0x37ca8000, 0x37cb0000, 0x37cb8000,
    0x37cc0000, 0x37cc8000, 0x37cd0000, 0x37cd8000, 0x37ce0000, 0x37ce8000, 0x37cf0000, 0x37cf8000,
    0x37d00000, 0x37d08000, 0x37d10000, 0x37d18000, 0x37d20000, 0x37d28000, 0x37d30000, 0x37d38000,
    0x37d40000, 0x37d48000, 0x37d50000, 0x37d58000, 0x37d60000, 0x37d68000, 0x37d70000, 0x37d78000,
    0x37d80000, 0x37d88000, 0x37d90000, 0x37d98000, 0x37da0000, 0x37da8000, 0x37db0000, 0x37db8000,
    0x37dc0000, 0x37dc8000, 0x37dd0000, 0x37dd8000, 0x37de0000, 0x37de8000, 0x37df0000, 0x37df8000,
    0x37e00000, 0x37e08000, 0x37e10000, 0x37e18000, 0x37e20000, 0x37e28000, 0x37e30000, 0x37e38000,
    0x37e40000, 0x37e48000, 0x37e50000, 0x37e58000, 0x37e60000, 0x37e68000, 0x37e70000, 0x37e78000,
    0x37e80000, 0x37e88000, 0x37e90000, 0x37e98000, 0x37ea0000, 0x37ea8000, 0x37eb0000, 0x37eb8000,
    0x37ec0000, 0x37ec8000, 0x37ed0000, 0x37ed8000, 0x37ee0000, 0x37ee8000, 0x37ef0000, 0x37ef8000,
    0x37f00000, 0x37f08000, 0x37f10000, 0x37f18000, 0x37f20000, 0x37f28000, 0x37f30000, 0x37f38000,
    0x37f40000, 0x37f48000, 0x37f50000, 0x37f58000, 0x37f60000, 0x37f68000, 0x37f70000, 0x37f78000,
    0x37f80000, 0x37f88000, 0x37f90000, 0x37f98000, 0x37fa0000, 0x37fa8000, 0x37fb0000, 0x37fb8000,
    0x37fc0000, 0x37fc8000, 0x37fd0000, 0x37fd8000, 0x37fe0000, 0x37fe8000, 0x37ff0000, 0x37ff8000,
    0x38000000, 0x38004000, 0x38008000, 0x3800c000, 0x38010000, 0x38014000, 0x38018000, 0x3801c000,
    0x38020000, 0x38024000, 0x38028000, 0x3802c000, 0x38030000, 0x38034000, 0x38038000, 0x3803c000,
    0x38040000, 0x38044000, 0x38048000, 0x3804c000, 0x38050000, 0x38054000, 0x38058000, 0x3805c000,
    0x38060000, 0x38064000, 0x38068000, 0x3806c000, 0x38070000, 0x38074000, 0x38078000, 0x3807c000,
    0x38080000, 0x38084000, 0x38088000, 0x3808c000, 0x38090000, 0x38094000, 0x38098000, 0x3809c000,
    0x380a0000, 0x380a4000, 0x380a8000, 0x380ac000, 0x380b0000, 0x380b4000, 0x380b8000, 0x380bc000,
    0x380c0000, 0x380c4000, 0x380c8000, 0x380cc000, 0x380d0000, 0x380d4000, 0x380d8000, 0x380dc000,
    0x380e0000, 0x380e4000, 0x380e8000, 0x380ec000, 0x380f0000, 0x380f4000, 0x380f8000, 0x380fc000,
    0x38100000, 0x38104000, 0x38108000, 0x3810c000, 0x38110000, 0x38114000, 0x38118000, 0x3811c000,
    0x38120000, 0x38124000, 0x38128000, 0x3812c000, 0x38130000, 0x38134000, 0x38138000, 0x3813c000,
    0x38140000, 0x38144000, 0x38148000, 0x3814c000, 0x38150000, 0x38154000, 0x38158000, 0x3815c000,
    0x38160000, 0x38164000, 0x38168000, 0x3816c000, 0x38170000, 0x38174000, 0x38178000, 0x3817c000,
    0x38180000, 0x38184000, 0x38188000, 0x3818c000, 0x38190000, 0x38194000, 0x38198000, 0x3819c000,
    0x381a0000, 0x381a4000, 0x381a8000, 0x381ac000, 0x381b0000, 0x381b4000, 0x381b8000, 0x381bc000,
    0x381c0000, 0x381c4000, 0x381c8000, 0x381cc000, 0x381d0000, 0x381d4000, 0x381d8000, 0x381dc000,
    0x381e0000, 0x381e4000, 0x381e8000, 0x381ec000, 0x381f0000, 0x381f4000, 0x381f8000, 0x381fc000,
    0x38200000, 0x38204000, 0x38208000, 0x3820c000, 0x38210000, 0x38214000, 0x38218000, 0x3821c000,
    0x38220000, 0x38224000, 0x38228000, 0x3822c000, 0x38230000, 0x38234000, 0x38238000, 0x3823c000,
    0x38240000, 0x38244000, 0x38248000, 0x3824c000, 0x38250000, 0x38254000, 0x38258000, 0x3825c000,
    0x38260000, 0x38264000, 0x38268000, 0x3826c000, 0x38270000, 0x38274000, 0x38278000, 0x3827c000,
    0x38280000, 0x38284000, 0x38288000, 0x3828c000, 0x38290000, 0x38294000, 0x38298000, 0x3829c000,
    0x382a0000, 0x382a4000, 0x382a8000, 0x382ac000, 0x382b0000, 0x382b4000, 0x382b8000, 0x382bc000,
    0x382c0000, 0x382c4000, 0x382c8000, 0x382cc000, 0x382d0000, 0x382d4000, 0x382d8000, 0x382dc000,
    0x382e0000, 0x382e4000, 0x382e8000, 0x382ec000, 0x382f0000, 0x382f4000, 0x382f8000, 0x382fc000,
    0x38300000, 0x38304000, 0x38308000, 0x3830c000, 0x38310000, 0x38314000, 0x38318000, 0x3831c000,
    0x38320000, 0x38324000, 0x38328000, 0x3832c000, 0x38330000, 0x38334000, 0x38338000, 0x3833c000,
    0x38340000, 0x38344000, 0x38348000, 0x3834c000, 0x38350000, 0x38354000, 0x38358000, 0x3835c000,
    0x38360000, 0x38364000, 0x38368000, 0x3836c000, 0x38370000, 0x38374000, 0x38378000, 0x3837c000,
    0x38380000, 0x38384000, 0x38388000, 0x3838c000, 0x38390000, 0x38394000, 0x38398000, 0x3839c000,
    0x383a0000, 0x383a4000, 0x383a8000, 0x383ac000, 0x383b0000, 0x383b4000, 0x383b8000, 0x383bc000,
    0x383c0000, 0x383c4000, 0x383c8000, 0x383cc000, 0x383d0000, 0x383d4000, 0x383d8000, 0x383dc000,
    0x383e0000, 0x383e4000, 0x383e8000, 0x383ec000, 0x383f0000, 0x383f4000, 0x383f8000, 0x383fc000,
    0x38400000, 0x38404000, 0x38408000, 0x3840c000, 0x38410000, 0x38414000, 0x38418000, 0x3841c000,
    0x38420000, 0x38424000, 0x38428000, 0x3842c000, 0x38430000, 0x38434000, 0x38438000, 0x3843c000,
    0x38440000, 0x38444000, 0x38448000, 0x3844c000, 0x38450000, 0x38454000, 0x38458000, 0x3845c000,
    0x38460000, 0x38464000, 0x38468000, 0x3846c
// ... (truncated) ...
```

**Entity:** cn_rnd_t (used by cn_rnd_next and f4)

**States:** ValidRngHandle, MissingRngHandle, PossiblyUnseededState

**Transitions:**
- MissingRngHandle -> panic via f4(None)
- ValidRngHandle -> ValidRngHandle via cn_rnd_next(&mut cn_rnd_t) state transition

**Evidence:** pub struct cn_rnd_t { pub state: [u64; 2] }; pub(crate) fn f4(rnd: Option<&mut cn_rnd_t>) -> f64 { let rnd = rnd.expect("cn_rnd_t must be provided"); ... }; fn cn_rnd_next(rnd: &mut cn_rnd_t) -> u64 { let mut x = rnd.state[0]; let y = rnd.state[1]; rnd.state[0] = y; ... rnd.state[1] = x; ... }

**Implementation:** Make `f4` take `&mut cn_rnd_t` (no Option) and provide a seeding constructor/builder that returns a `SeededRng` newtype: `struct SeededRng(cn_rnd_t); impl SeededRng { fn from_seed(seed: u128) -> Self { ... } fn next_f64(&mut self) -> f64 { ... } }`. This makes "must be provided" and "must be seeded" unrepresentable in safe code.

---

### 1. PRNG seed/initialization validity (Unseeded/Invalid vs Seeded/Valid)

**Location**: `/data/test_case/lib.rs:1-8`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: cn_rnd_t appears to represent a random-number-generator state as two u64 words. There is an implicit validity requirement for PRNG states (typically: must be seeded, and often must not be the all-zero state) before it is used to generate random numbers. As written, cn_rnd_t is Copy + public with a public `state` field, so any bit pattern (including an invalid/unseeded one) can be constructed and freely duplicated, and the type system does not enforce that the state has been properly initialized/seeded or remains within whatever invariants the RNG algorithm expects.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct lm_vec2, 4 free function(s); struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); struct c2Circle, 1 free function(s); 9 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct cn_rnd_t {
    pub state: [u64; 2],
}

```

**Entity:** cn_rnd_t

**States:** Invalid/Unseeded, Seeded/Valid

**Transitions:**
- Invalid/Unseeded -> Seeded/Valid via (external) seeding/initialization routine (not shown in snippet)

**Evidence:** pub struct cn_rnd_t { pub state: [u64; 2] } exposes raw internal RNG state directly; #[derive(Copy, Clone)] allows duplicating the RNG state, which is often semantically distinct from advancing a single RNG stream; #[repr(C)] suggests FFI/ABI usage where raw bytes are passed across boundaries, increasing likelihood of needing a 'valid state' invariant

**Implementation:** Make the internal state private (e.g., `state: [u64; 2]`), provide constructors like `cn_rnd_t::from_seed(seed: u128)` / `::from_state(ValidState)`, and (if needed) a validated `NonZero`-like wrapper ensuring disallowed states (e.g., all-zero) cannot be created. If copying is not intended, drop `Copy` and only allow cloning via an explicit `fork()` that documents semantics.

---

## Protocol Invariants

### 2. Untyped shape-pointer protocol (type tag must match pointed-to layout)

**Location**: `/data/test_case/lib.rs:1-356`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: f2 implements a manual sum-type over raw void pointers by pairing each pointer with a runtime C2_TYPE tag. Correctness relies on an implicit invariant: when typeA == C2_TYPE_CIRCLE, A must point to a valid (and sufficiently sized) c2Circle; when typeA == C2_TYPE_AABB, A must point to a valid c2AABB; similarly for B/typeB. The function then does unchecked (but unaligned) reads into those concrete types. The type system cannot prevent callers from passing a mismatched tag/pointer combination or dangling/invalid pointers, which would yield UB. Unknown tags fall through to `_ => 0`, silently masking misuse instead of making it unrepresentable.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct lm_vec2, 4 free function(s); struct cn_rnd_t, 2 free function(s); struct c2AABB, 2 free function(s); struct c2v, 6 free function(s); struct c2Circle, 1 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type tflac_u32 = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub flt: f32,
    pub num: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct lm_vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct cn_rnd_t {
    pub state: [u64; 2],
}

pub type C2_TYPE = u32;
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
pub struct c2v {
    pub x: f32,
    pub y: f32,
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
pub(crate) fn c2CircletoCircle(A: c2Circle, B: c2Circle) -> i32 {
    let c = c2Sub(B.p, A.p);
    let d2 = c2Dot(c, c);
    let r = A.r + B.r;
    let r2 = r * r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2CircletoAABB(A: c2Circle, B: c2AABB) -> i32 {
    let L = c2Clampv(A.p, B.min, B.max);
    let ab = c2Sub(A.p, L);
    let d2 = c2Dot(ab, ab);
    let r2 = A.r * A.r;
    (d2 < r2) as i32
}

#[inline]
pub(crate) fn c2AABBtoAABB(A: c2AABB, B: c2AABB) -> i32 {
    let separated = (B.max.x < A.min.x) || (A.max.x < B.min.x) || (B.max.y < A.min.y) || (A.max.y < B.min.y);
    (!separated) as i32
}

pub(crate) unsafe fn f2(
    A: *const std::ffi::c_void,
    typeA: C2_TYPE,
    B: *const std::ffi::c_void,
    typeB: C2_TYPE,
) -> i32 {
    unsafe fn read<T: Copy>(p: *const std::ffi::c_void) -> T {
        // Preserve original behavior (unaligned reads possible in C); use read_unaligned.
        (p as *const T).read_unaligned()
    }

    match (typeA, typeB) {
        (C2_TYPE_CIRCLE, C2_TYPE_CIRCLE) => c2CircletoCircle(read::<c2Circle>(A), read::<c2Circle>(B)),
        (C2_TYPE_CIRCLE, C2_TYPE_AABB) => c2CircletoAABB(read::<c2Circle>(A), read::<c2AABB>(B)),
        (C2_TYPE_AABB, C2_TYPE_CIRCLE) => c2CircletoAABB(read::<c2Circle>(B), read::<c2AABB>(A)),
        (C2_TYPE_AABB, C2_TYPE_AABB) => c2AABBtoAABB(read::<c2AABB>(A), read::<c2AABB>(B)),
        _ => 0,
    }
}

pub(crate) fn f3(v1: i32, v2: i32) -> i32 {
    if v2 == 0 {
        return 0;
    }
    let q: i32;
    let r: i32;
    if v1 >= 0 {
        if v2 >= 0 {
            return v1 / v2;
        } else if v2 != i32::MIN {
            q = -(v1 / -v2);
            r = v1 % -v2;
        } else {
            q = 0;
            r = v1;
        }
    } else if v1 != i32::MIN {
        if v2 >= 0 {
            q = -(-v1 / v2);
            r = -(-v1 % v2);
        } else if v2 != i32::MIN {
            q = -v1 / -v2;
            r = -(-v1 % -v2);
        } else {
            q = 1;
            r = v1 - q * v2;
        }
    } else if v2 >= 0 {
        q = -(-(v1 + v2) / v2) - 1;
        r = -(-(v1 + v2) % v2);
    } else if v2 != i32::MIN {
        q = -(v1 - v2) / -v2 + 1;
        r = -(-(v1 - v2) % -v2);
    } else {
        q = 1;
        r = 0;
    }
    if r >= 0 {
        q
    } else {
        q + if v2 > 0 { -1 } else { 1 }
    }
}

#[inline]
fn cn_rnd_next(rnd: &mut cn_rnd_t) -> u64 {
    let mut x = rnd.state[0];
    let y = rnd.state[1];
    rnd.state[0] = y;
    x ^= x << 23;
    x ^= x >> 17;
    x ^= y ^ (y >> 26);
    rnd.state[1] = x;
    x.wrapping_add(y)
}

pub(crate) fn f4(rnd: Option<&mut cn_rnd_t>) -> f64 {
    let rnd = rnd.expect("cn_rnd_t must be provided");
    let value = cn_rnd_next(rnd);
    let exponent: u64 = 1023;
    let mantissa: u64 = value >> 12;
    let bits: u64 = (exponent << 52) | mantissa;
    f64::from_bits(bits) - 1.0
}

pub(crate) fn f5(mut a: u32) -> u32 {
    a = (a & 0xaaaau32) >> 1 | (a & 0x5555u32) << 1;
    a = (a & 0xccccu32) >> 2 | (a & 0x3333u32) << 2;
    a = (a & 0xf0f0u32) >> 4 | (a & 0x0f0fu32) << 4;
    a = (a & 0xff00u32) >> 8 | (a & 0x00ffu32) << 8;
    a
}

pub(crate) fn f7(blocksize: tflac_u32, channels: tflac_u32, bitdepth: tflac_u32) -> tflac_u32 {
    18u32.wrapping_add(channels).wrapping_add(
        blocksize
            .wrapping_mul(bitdepth)
            .wrapping_mul(channels.wrapping_mul((channels != 2) as u32))
            .wrapping_add(blocksize.wrapping_mul(bitdepth).wrapping_mul((channels == 2) as u32))
            .wrapping_add(
                blocksize
                    .wrapping_mul(bitdepth.wrapping_add((bitdepth != 32) as u32))
                    .wrapping_mul((channels == 2) as u32),
            )
            .wrapping_add(7)
            .wrapping_div(8),
    )
}

#[inline]
fn lm_v2(x: f32, y: f32) -> lm_vec2 {
    lm_vec2 { x, y }
}

#[inline]
fn lm_sub2(a: lm_vec2, b: lm_vec2) -> lm_vec2 {
    lm_v2(a.x - b.x, a.y - b.y)
}

#[inline]
fn lm_dot2(a: lm_vec2, b: lm_vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

pub(crate) fn f9(p1: lm_vec2, p2: lm_vec2, p3: lm_vec2, p: lm_vec2) -> lm_vec2 {
    let v0 = lm_sub2(p3, p1);
    let v1 = lm_sub2(p2, p1);
    let v2 = lm_sub2(p, p1);
    let dot00 = lm_dot2(v0, v0);
    let dot01 = lm_dot2(v0, v1);
    let dot02 = lm_dot2(v0, v2);
    let dot11 = lm_dot2(v1, v1);
    let dot12 = lm_dot2(v1, v2);
    let invDenom = 1.0f32 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * invDenom;
    let v = (dot00 * dot12 - dot01 * dot02) * invDenom;
    lm_v2(u, v)
}

static m__mantissa: [u32; 2048] = [
    0, 0x33800000, 0x34000000, 0x34400000, 0x34800000, 0x34a00000, 0x34c00000, 0x34e00000,
    0x35000000, 0x35100000, 0x35200000, 0x35300000, 0x35400000, 0x35500000, 0x35600000, 0x35700000,
    0x35800000, 0x35880000, 0x35900000, 0x35980000, 0x35a00000, 0x35a80000, 0x35b00000, 0x35b80000,
    0x35c00000, 0x35c80000, 0x35d00000, 0x35d80000, 0x35e00000, 0x35e80000, 0x35f00000, 0x35f80000,
    0x36000000, 0x36040000, 0x36080000, 0x360c0000, 0x36100000, 0x36140000, 0x36180000, 0x361c0000,
    0x36200000, 0x36240000, 0x36280000, 0x362c0000, 0x36300000, 0x36340000, 0x36380000, 0x363c0000,
    0x36400000, 0x36440000, 0x36480000, 0x364c0000, 0x36500000, 0x36540000, 0x36580000, 0x365c0000,
    0x36600000, 0x36640000, 0x36680000, 0x366c0000, 0x36700000, 0x36740000, 0x36780000, 0x367c0000,
    0x36800000, 0x36820000, 0x36840000, 0x36860000, 0x36880000, 0x368a0000, 0x368c0000, 0x368e0000,
    0x36900000, 0x36920000, 0x36940000, 0x36960000, 0x36980000, 0x369a0000, 0x369c0000, 0x369e0000,
    0x36a00000, 0x36a20000, 0x36a40000, 0x36a60000, 0x36a80000, 0x36aa0000, 0x36ac0000, 0x36ae0000,
    0x36b00000, 0x36b20000, 0x36b40000, 0x36b60000, 0x36b80000, 0x36ba0000, 0x36bc0000, 0x36be0000,
    0x36c00000, 0x36c20000, 0x36c40000, 0x36c60000, 0x36c80000, 0x36ca0000, 0x36cc0000, 0x36ce0000,
    0x36d00000, 0x36d20000, 0x36d40000, 0x36d60000, 0x36d80000, 0x36da0000, 0x36dc0000, 0x36de0000,
    0x36e00000, 0x36e20000, 0x36e40000, 0x36e60000, 0x36e80000, 0x36ea0000, 0x36ec0000, 0x36ee0000,
    0x36f00000, 0x36f20000, 0x36f40000, 0x36f60000, 0x36f80000, 0x36fa0000, 0x36fc0000, 0x36fe0000,
    0x37000000, 0x37010000, 0x37020000, 0x37030000, 0x37040000, 0x37050000, 0x37060000, 0x37070000,
    0x37080000, 0x37090000, 0x370a0000, 0x370b0000, 0x370c0000, 0x370d0000, 0x370e0000, 0x370f0000,
    0x37100000, 0x37110000, 0x37120000, 0x37130000, 0x37140000, 0x37150000, 0x37160000, 0x37170000,
    0x37180000, 0x37190000, 0x371a0000, 0x371b0000, 0x371c0000, 0x371d0000, 0x371e0000, 0x371f0000,
    0x37200000, 0x37210000, 0x37220000, 0x37230000, 0x37240000, 0x37250000, 0x37260000, 0x37270000,
    0x37280000, 0x37290000, 0x372a0000, 0x372b0000, 0x372c0000, 0x372d0000, 0x372e0000, 0x372f0000,
    0x37300000, 0x37310000, 0x37320000, 0x37330000, 0x37340000, 0x37350000, 0x37360000, 0x37370000,
    0x37380000, 0x37390000, 0x373a0000, 0x373b0000, 0x373c0000, 0x373d0000, 0x373e0000, 0x373f0000,
    0x37400000, 0x37410000, 0x37420000, 0x37430000, 0x37440000, 0x37450000, 0x37460000, 0x37470000,
    0x37480000, 0x37490000, 0x374a0000, 0x374b0000, 0x374c0000, 0x374d0000, 0x374e0000, 0x374f0000,
    0x37500000, 0x37510000, 0x37520000, 0x37530000, 0x37540000, 0x37550000, 0x37560000, 0x37570000,
    0x37580000, 0x37590000, 0x375a0000, 0x375b0000, 0x375c0000, 0x375d0000, 0x375e0000, 0x375f0000,
    0x37600000, 0x37610000, 0x37620000, 0x37630000, 0x37640000, 0x37650000, 0x37660000, 0x37670000,
    0x37680000, 0x37690000, 0x376a0000, 0x376b0000, 0x376c0000, 0x376d0000, 0x376e0000, 0x376f0000,
    0x37700000, 0x37710000, 0x37720000, 0x37730000, 0x37740000, 0x37750000, 0x37760000, 0x37770000,
    0x37780000, 0x37790000, 0x377a0000, 0x377b0000, 0x377c0000, 0x377d0000, 0x377e0000, 0x377f0000,
    0x37800000, 0x37808000, 0x37810000, 0x37818000, 0x37820000, 0x37828000, 0x37830000, 0x37838000,
    0x37840000, 0x37848000, 0x37850000, 0x37858000, 0x37860000, 0x37868000, 0x37870000, 0x37878000,
    0x37880000, 0x37888000, 0x37890000, 0x37898000, 0x378a0000, 0x378a8000, 0x378b0000, 0x378b8000,
    0x378c0000, 0x378c8000, 0x378d0000, 0x378d8000, 0x378e0000, 0x378e8000, 0x378f0000, 0x378f8000,
    0x37900000, 0x37908000, 0x37910000, 0x37918000, 0x37920000, 0x37928000, 0x37930000, 0x37938000,
    0x37940000, 0x37948000, 0x37950000, 0x37958000, 0x37960000, 0x37968000, 0x37970000, 0x37978000,
    0x37980000, 0x37988000, 0x37990000, 0x37998000, 0x379a0000, 0x379a8000, 0x379b0000, 0x379b8000,
    0x379c0000, 0x379c8000, 0x379d0000, 0x379d8000, 0x379e0000, 0x379e8000, 0x379f0000, 0x379f8000,
    0x37a00000, 0x37a08000, 0x37a10000, 0x37a18000, 0x37a20000, 0x37a28000, 0x37a30000, 0x37a38000,
    0x37a40000, 0x37a48000, 0x37a50000, 0x37a58000, 0x37a60000, 0x37a68000, 0x37a70000, 0x37a78000,
    0x37a80000, 0x37a88000, 0x37a90000, 0x37a98000, 0x37aa0000, 0x37aa8000, 0x37ab0000, 0x37ab8000,
    0x37ac0000, 0x37ac8000, 0x37ad0000, 0x37ad8000, 0x37ae0000, 0x37ae8000, 0x37af0000, 0x37af8000,
    0x37b00000, 0x37b08000, 0x37b10000, 0x37b18000, 0x37b20000, 0x37b28000, 0x37b30000, 0x37b38000,
    0x37b40000, 0x37b48000, 0x37b50000, 0x37b58000, 0x37b60000, 0x37b68000, 0x37b70000, 0x37b78000,
    0x37b80000, 0x37b88000, 0x37b90000, 0x37b98000, 0x37ba0000, 0x37ba8000, 0x37bb0000, 0x37bb8000,
    0x37bc0000, 0x37bc8000, 0x37bd0000, 0x37bd8000, 0x37be0000, 0x37be8000, 0x37bf0000, 0x37bf8000,
    0x37c00000, 0x37c08000, 0x37c10000, 0x37c18000, 0x37c20000, 0x37c28000, 0x37c30000, 0x37c38000,
    0x37c40000, 0x37c48000, 0x37c50000, 0x37c58000, 0x37c60000, 0x37c68000, 0x37c70000, 0x37c78000,
    0x37c80000, 0x37c88000, 0x37c90000, 0x37c98000, 0x37ca0000, 0x37ca8000, 0x37cb0000, 0x37cb8000,
    0x37cc0000, 0x37cc8000, 0x37cd0000, 0x37cd8000, 0x37ce0000, 0x37ce8000, 0x37cf0000, 0x37cf8000,
    0x37d00000, 0x37d08000, 0x37d10000, 0x37d18000, 0x37d20000, 0x37d28000, 0x37d30000, 0x37d38000,
    0x37d40000, 0x37d48000, 0x37d50000, 0x37d58000, 0x37d60000, 0x37d68000, 0x37d70000, 0x37d78000,
    0x37d80000, 0x37d88000, 0x37d90000, 0x37d98000, 0x37da0000, 0x37da8000, 0x37db0000, 0x37db8000,
    0x37dc0000, 0x37dc8000, 0x37dd0000, 0x37dd8000, 0x37de0000, 0x37de8000, 0x37df0000, 0x37df8000,
    0x37e00000, 0x37e08000, 0x37e10000, 0x37e18000, 0x37e20000, 0x37e28000, 0x37e30000, 0x37e38000,
    0x37e40000, 0x37e48000, 0x37e50000, 0x37e58000, 0x37e60000, 0x37e68000, 0x37e70000, 0x37e78000,
    0x37e80000, 0x37e88000, 0x37e90000, 0x37e98000, 0x37ea0000, 0x37ea8000, 0x37eb0000, 0x37eb8000,
    0x37ec0000, 0x37ec8000, 0x37ed0000, 0x37ed8000, 0x37ee0000, 0x37ee8000, 0x37ef0000, 0x37ef8000,
    0x37f00000, 0x37f08000, 0x37f10000, 0x37f18000, 0x37f20000, 0x37f28000, 0x37f30000, 0x37f38000,
    0x37f40000, 0x37f48000, 0x37f50000, 0x37f58000, 0x37f60000, 0x37f68000, 0x37f70000, 0x37f78000,
    0x37f80000, 0x37f88000, 0x37f90000, 0x37f98000, 0x37fa0000, 0x37fa8000, 0x37fb0000, 0x37fb8000,
    0x37fc0000, 0x37fc8000, 0x37fd0000, 0x37fd8000, 0x37fe0000, 0x37fe8000, 0x37ff0000, 0x37ff8000,
    0x38000000, 0x38004000, 0x38008000, 0x3800c000, 0x38010000, 0x38014000, 0x38018000, 0x3801c000,
    0x38020000, 0x38024000, 0x38028000, 0x3802c000, 0x38030000, 0x38034000, 0x38038000, 0x3803c000,
    0x38040000, 0x38044000, 0x38048000, 0x3804c000, 0x38050000, 0x38054000, 0x38058000, 0x3805c000,
    0x38060000, 0x38064000, 0x38068000, 0x3806c000, 0x38070000, 0x38074000, 0x38078000, 0x3807c000,
    0x38080000, 0x38084000, 0x38088000, 0x3808c000, 0x38090000, 0x38094000, 0x38098000, 0x3809c000,
    0x380a0000, 0x380a4000, 0x380a8000, 0x380ac000, 0x380b0000, 0x380b4000, 0x380b8000, 0x380bc000,
    0x380c0000, 0x380c4000, 0x380c8000, 0x380cc000, 0x380d0000, 0x380d4000, 0x380d8000, 0x380dc000,
    0x380e0000, 0x380e4000, 0x380e8000, 0x380ec000, 0x380f0000, 0x380f4000, 0x380f8000, 0x380fc000,
    0x38100000, 0x38104000, 0x38108000, 0x3810c000, 0x38110000, 0x38114000, 0x38118000, 0x3811c000,
    0x38120000, 0x38124000, 0x38128000, 0x3812c000, 0x38130000, 0x38134000, 0x38138000, 0x3813c000,
    0x38140000, 0x38144000, 0x38148000, 0x3814c000, 0x38150000, 0x38154000, 0x38158000, 0x3815c000,
    0x38160000, 0x38164000, 0x38168000, 0x3816c000, 0x38170000, 0x38174000, 0x38178000, 0x3817c000,
    0x38180000, 0x38184000, 0x38188000, 0x3818c000, 0x38190000, 0x38194000, 0x38198000, 0x3819c000,
    0x381a0000, 0x381a4000, 0x381a8000, 0x381ac000, 0x381b0000, 0x381b4000, 0x381b8000, 0x381bc000,
    0x381c0000, 0x381c4000, 0x381c8000, 0x381cc000, 0x381d0000, 0x381d4000, 0x381d8000, 0x381dc000,
    0x381e0000, 0x381e4000, 0x381e8000, 0x381ec000, 0x381f0000, 0x381f4000, 0x381f8000, 0x381fc000,
    0x38200000, 0x38204000, 0x38208000, 0x3820c000, 0x38210000, 0x38214000, 0x38218000, 0x3821c000,
    0x38220000, 0x38224000, 0x38228000, 0x3822c000, 0x38230000, 0x38234000, 0x38238000, 0x3823c000,
    0x38240000, 0x38244000, 0x38248000, 0x3824c000, 0x38250000, 0x38254000, 0x38258000, 0x3825c000,
    0x38260000, 0x38264000, 0x38268000, 0x3826c000, 0x38270000, 0x38274000, 0x38278000, 0x3827c000,
    0x38280000, 0x38284000, 0x38288000, 0x3828c000, 0x38290000, 0x38294000, 0x38298000, 0x3829c000,
    0x382a0000, 0x382a4000, 0x382a8000, 0x382ac000, 0x382b0000, 0x382b4000, 0x382b8000, 0x382bc000,
    0x382c0000, 0x382c4000, 0x382c8000, 0x382cc000, 0x382d0000, 0x382d4000, 0x382d8000, 0x382dc000,
    0x382e0000, 0x382e4000, 0x382e8000, 0x382ec000, 0x382f0000, 0x382f4000, 0x382f8000, 0x382fc000,
    0x38300000, 0x38304000, 0x38308000, 0x3830c000, 0x38310000, 0x38314000, 0x38318000, 0x3831c000,
    0x38320000, 0x38324000, 0x38328000, 0x3832c000, 0x38330000, 0x38334000, 0x38338000, 0x3833c000,
    0x38340000, 0x38344000, 0x38348000, 0x3834c000, 0x38350000, 0x38354000, 0x38358000, 0x3835c000,
    0x38360000, 0x38364000, 0x38368000, 0x3836c000, 0x38370000, 0x38374000, 0x38378000, 0x3837c000,
    0x38380000, 0x38384000, 0x38388000, 0x3838c000, 0x38390000, 0x38394000, 0x38398000, 0x3839c000,
    0x383a0000, 0x383a4000, 0x383a8000, 0x383ac000, 0x383b0000, 0x383b4000, 0x383b8000, 0x383bc000,
    0x383c0000, 0x383c4000, 0x383c8000, 0x383cc000, 0x383d0000, 0x383d4000, 0x383d8000, 0x383dc000,
    0x383e0000, 0x383e4000, 0x383e8000, 0x383ec000, 0x383f0000, 0x383f4000, 0x383f8000, 0x383fc000,
    0x38400000, 0x38404000, 0x38408000, 0x3840c000, 0x38410000, 0x38414000, 0x38418000, 0x3841c000,
    0x38420000, 0x38424000, 0x38428000, 0x3842c000, 0x38430000, 0x38434000, 0x38438000, 0x3843c000,
    0x38440000, 0x38444000, 0x38448000, 0x3844c000, 0x38450000, 0x38454000, 0x38458000, 0x3845c000,
    0x38460000, 0x38464000, 0x38468000, 0x3846c
// ... (truncated) ...
```

**Entity:** f2 (collision dispatch API over *const c_void + C2_TYPE)

**States:** CirclePtr, AABBPtr, InvalidTagOrMismatchedPtr

**Transitions:**
- CirclePtr/AABBPtr selection via (typeA, typeB) match arms in f2()

**Evidence:** pub(crate) unsafe fn f2(A: *const c_void, typeA: C2_TYPE, B: *const c_void, typeB: C2_TYPE) -> i32; type tags: pub const C2_TYPE_CIRCLE: C2_TYPE = 0; pub const C2_TYPE_AABB: C2_TYPE = 1; unsafe fn read<T: Copy>(p: *const c_void) -> T { (p as *const T).read_unaligned() }; match (typeA, typeB) { (C2_TYPE_CIRCLE, C2_TYPE_CIRCLE) => read::<c2Circle>(A)...; (C2_TYPE_CIRCLE, C2_TYPE_AABB) => read::<c2AABB>(B)...; ...; _ => 0 }

**Implementation:** Replace (ptr, tag) pairs with a typed enum, e.g. `enum ShapeRef<'a> { Circle(&'a c2Circle), Aabb(&'a c2AABB) }` (or owning variants if needed). Then implement `fn collide(a: ShapeRef<'_>, b: ShapeRef<'_>) -> i32` with exhaustive matching. If FFI requires pointers, use `#[repr(C)] struct TaggedShape { tag: C2_TYPE, ptr: *const c_void }` plus safe constructors `TaggedShape::from_circle(&c2Circle)` / `from_aabb(&c2AABB)` so mismatches are unconstructible in safe Rust.

---

