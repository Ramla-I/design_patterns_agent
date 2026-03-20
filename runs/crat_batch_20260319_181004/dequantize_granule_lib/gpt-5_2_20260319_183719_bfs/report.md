# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. Byte-slice cursor validity invariant (non-null / in-bounds / ordered pos<=limit)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: bs_t is a C-style buffer cursor. Safe use implicitly requires (1) buf to be non-null and point to a live allocation for the duration of use, and (2) pos/limit to describe a valid range within that allocation (typically 0 <= pos <= limit <= len). These requirements are not enforced: buf is a raw pointer with no lifetime/length, and pos/limit are signed i32 with no ordering or bounds guarantees, so callers can construct invalid cursors leading to out-of-bounds reads or use-after-free in downstream code.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct L12_scale_info


#[repr(C)]
#[derive(Copy, Clone)]
pub struct bs_t {
    pub buf: *const u8,
    pub pos: i32,
    pub limit: i32,
}

```

**Entity:** bs_t

**States:** ValidCursor, InvalidCursor

**Transitions:**
- InvalidCursor -> ValidCursor via constructing/initializing fields with correct pointer + bounds (not encoded in types)

**Evidence:** line 7: pub buf: *const u8 — raw pointer allows null/dangling and has no lifetime/length; line 8: pub pos: i32 — signed index can be negative and has no relation enforced to limit; line 9: pub limit: i32 — signed bound can be negative and has no relation enforced to pos or underlying allocation size; line 5-10: #[repr(C)] + Copy, Clone suggests FFI/plain-data usage where validity is a runtime convention rather than a typed guarantee

**Implementation:** Introduce a safe wrapper (e.g., struct Bs<'a> { buf: &'a [u8], pos: usize, limit: usize }) that enforces 0<=pos<=limit<=buf.len() in constructors; keep bs_t only for FFI and provide checked conversions: impl<'a> TryFrom<bs_t> for Bs<'a> (or an unsafe from_raw_parts requiring caller proof). Use usize for indices to remove negative values.

---

