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

### 1. cp_image_t raw pointer validity + dimension/pixel-buffer consistency

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: cp_image_t contains a raw pointer `pix` that is expected (by C-FFI convention) to either be null (representing no pixel buffer) or point to a valid contiguous pixel buffer whose length matches the image dimensions (typically w*h). The Rust type system does not enforce non-nullness, initialization, ownership/lifetime, or that `w` and `h` are consistent with the allocation behind `pix`. Additionally, `Copy, Clone` allows duplicating the struct, implicitly duplicating the raw pointer without tracking aliasing, ownership, or borrowing rules, which can lead to use-after-free/double-free in surrounding code that treats `pix` as owned.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_pixel_t


#[repr(C)]
#[derive(Copy, Clone)]
pub struct cp_image_t {
    pub w: i32,
    pub h: i32,
    pub pix: *mut cp_pixel_t,
}

```

**Entity:** cp_image_t

**States:** Null/Uninitialized pix, Valid pixel buffer

**Transitions:**
- Null/Uninitialized pix -> Valid pixel buffer via external (FFI) allocation/initialization
- Valid pixel buffer -> Null/Uninitialized pix via external (FFI) free/deinit

**Evidence:** line 8: pub pix: *mut cp_pixel_t (raw mutable pointer; may be null/dangling; no lifetime/ownership encoded); line 6: pub w: i32 and line 7: pub h: i32 (dimensions exist but no link to pixel buffer length/capacity); line 5: #[derive(Copy, Clone)] (copies the raw pointer without enforcing any aliasing/ownership protocol)

**Implementation:** Introduce a safe wrapper that encodes invariants: e.g., `struct ImageBuf<'a> { w: NonZeroU32, h: NonZeroU32, pix: NonNull<cp_pixel_t>, _lt: PhantomData<&'a mut [cp_pixel_t]> }` (or `&'a mut [cp_pixel_t]` directly). Use `Option<NonNull<_>>` to represent the nullable state. If ownership is intended, use RAII: `struct OwnedImage { w, h, pix: NonNull<_> }` with `Drop` calling the correct FFI free, and remove `Copy`.

---

