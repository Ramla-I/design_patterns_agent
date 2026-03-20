# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 2. Quantizer index invariant (3-bit code within a fixed 8-wide block, with neighbor clamping)

**Location**: `/data/test_case/lib.rs:1-86`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `encode_quant` treats `uni` as a quantizer index whose low 3 bits (`uni & 7`) are used to compute a signed delta, and whose higher bits define an 8-wide block that must not change when considering neighbors `uni±1`. The function performs explicit runtime checks to avoid crossing block boundaries by clamping `uni1/uni2` back to `uni` when `±1` would change bits outside the low 3. This is an implicit validity/domain rule for `uni` (and for the notion of “neighbor within the same block”) that is not represented in the type system; callers can pass any `i32` and only the internal masking/clamping preserves the intended semantics.

**Evidence**:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[inline]
fn abs_i32_branchless(x: i32) -> i32 {
    x ^ (x >> 31)
}

#[inline]
fn apply_lsbit(mut u: i32, lsbit: i32) -> i32 {
    if lsbit == 0 {
        return u;
    }

    if lsbit == 4 {
        u &= !1;
        u |= (u >> 1) & (u >> 2) & 1;
        u
    } else if (lsbit & 1) != 0 {
        u | 1
    } else {
        u & !1
    }
}

#[inline]
fn quant_diff(uni: i32, step: i32) -> i32 {
    let mut diff = (2 * (uni & 7) + 1) * step / 8;
    if (uni & 8) != 0 {
        diff = -diff;
    }
    diff
}

#[no_mangle]
pub extern "C" fn encode_quant(
    mut uni: i32,
    step: i32,
    pred: i32,
    tgt: i32,
    tgt2: i32,
    lsbit: i32,
) -> i32 {
    let mut uni1 = uni + 1;
    let mut uni2 = uni - 1;

    if ((uni ^ uni1) & !7) != 0 {
        uni1 = uni;
    }
    if ((uni ^ uni2) & !7) != 0 {
        uni2 = uni;
    }

    uni = apply_lsbit(uni, lsbit);
    uni1 = apply_lsbit(uni1, lsbit);
    uni2 = apply_lsbit(uni2, lsbit);

    let p0 = pred + quant_diff(uni, step);
    let p1 = pred + quant_diff(uni1, step);
    let p2 = pred + quant_diff(uni2, step);

    let mut d0 = abs_i32_branchless(tgt - p0);
    let mut d1 = abs_i32_branchless(tgt - p1);
    let mut d2 = abs_i32_branchless(tgt - p2);

    d0 += abs_i32_branchless(tgt2 - p0) >> 5;
    d1 += abs_i32_branchless(tgt2 - p1) >> 5;
    d2 += abs_i32_branchless(tgt2 - p2) >> 5;

    if d1 < d0 {
        uni = uni1;
        d0 = d1;
    }
    if d2 < d0 {
        uni = uni2;
    }

    uni
}
```

**Entity:** uni (i32 parameter to encode_quant())

**States:** InBlock (neighbor differs only in low 3 bits), EdgeOfBlock (neighbor would cross block so clamp to self)

**Transitions:**
- InBlock -> EdgeOfBlock via clamping when `((uni ^ (uni±1)) & !7) != 0`

**Evidence:** encode_quant: `let mut uni1 = uni + 1; let mut uni2 = uni - 1;` (neighbor exploration); encode_quant: `if ((uni ^ uni1) & !7) != 0 { uni1 = uni; }` (clamp if crossing 8-wide block); encode_quant: `if ((uni ^ uni2) & !7) != 0 { uni2 = uni; }` (clamp if crossing 8-wide block); quant_diff: `let mut diff = (2 * (uni & 7) + 1) * step / 8;` (only low 3 bits are semantically used)

**Implementation:** Introduce a `struct UniIndex { raw: i32 }` (or `struct BlockedIndex { block: i32, offset: u8 }` where `offset: u8` is constrained to 0..=7). Provide safe constructors and a `neighbor_within_block(delta: i32) -> Option<Self>` (or clamped variant) so the “same-block neighbor” rule is enforced by APIs rather than ad-hoc bit checks.

---

## Protocol Invariants

### 1. lsbit mode protocol (None / ForceLSB / ClearLSB / DerivedLSB)

**Location**: `/data/test_case/lib.rs:1-86`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `lsbit` is used as a mode selector controlling how the least-significant bit of `u` is manipulated. The code implicitly relies on specific magic values/bit-patterns of `lsbit` (0 and 4 are special; otherwise behavior depends on odd/even). This protocol is enforced only by runtime branching on integer values; callers can pass arbitrary `i32` values that will be interpreted via these conventions, but the type system does not encode the allowed modes or their meaning.

**Evidence**:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[inline]
fn abs_i32_branchless(x: i32) -> i32 {
    x ^ (x >> 31)
}

#[inline]
fn apply_lsbit(mut u: i32, lsbit: i32) -> i32 {
    if lsbit == 0 {
        return u;
    }

    if lsbit == 4 {
        u &= !1;
        u |= (u >> 1) & (u >> 2) & 1;
        u
    } else if (lsbit & 1) != 0 {
        u | 1
    } else {
        u & !1
    }
}

#[inline]
fn quant_diff(uni: i32, step: i32) -> i32 {
    let mut diff = (2 * (uni & 7) + 1) * step / 8;
    if (uni & 8) != 0 {
        diff = -diff;
    }
    diff
}

#[no_mangle]
pub extern "C" fn encode_quant(
    mut uni: i32,
    step: i32,
    pred: i32,
    tgt: i32,
    tgt2: i32,
    lsbit: i32,
) -> i32 {
    let mut uni1 = uni + 1;
    let mut uni2 = uni - 1;

    if ((uni ^ uni1) & !7) != 0 {
        uni1 = uni;
    }
    if ((uni ^ uni2) & !7) != 0 {
        uni2 = uni;
    }

    uni = apply_lsbit(uni, lsbit);
    uni1 = apply_lsbit(uni1, lsbit);
    uni2 = apply_lsbit(uni2, lsbit);

    let p0 = pred + quant_diff(uni, step);
    let p1 = pred + quant_diff(uni1, step);
    let p2 = pred + quant_diff(uni2, step);

    let mut d0 = abs_i32_branchless(tgt - p0);
    let mut d1 = abs_i32_branchless(tgt - p1);
    let mut d2 = abs_i32_branchless(tgt - p2);

    d0 += abs_i32_branchless(tgt2 - p0) >> 5;
    d1 += abs_i32_branchless(tgt2 - p1) >> 5;
    d2 += abs_i32_branchless(tgt2 - p2) >> 5;

    if d1 < d0 {
        uni = uni1;
        d0 = d1;
    }
    if d2 < d0 {
        uni = uni2;
    }

    uni
}
```

**Entity:** lsbit (i32 parameter to apply_lsbit()/encode_quant())

**States:** None (0), DerivedLSB (4), ForceLSB (odd), ClearLSB (even nonzero, non-4)

**Transitions:**
- N/A (mode is selected per call, not stored)

**Evidence:** apply_lsbit(lsbit): `if lsbit == 0 { return u; }` (special case 0); apply_lsbit(lsbit): `if lsbit == 4 { ... }` (special case 4); apply_lsbit(lsbit): `else if (lsbit & 1) != 0 { u | 1 } else { u & !1 }` (odd/even interpretation)

**Implementation:** Replace `lsbit: i32` with an enum, e.g. `enum LsbitMode { None, Derived, ForceOne, ForceZero }` (or a `TryFrom<i32>` newtype that validates/normalizes). Then `apply_lsbit(u, mode)` matches on the enum, eliminating magic numbers and invalid modes at compile time (or at least centralizing validation).

---

