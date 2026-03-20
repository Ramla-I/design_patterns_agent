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

### 1. cp_image_t FFI pointer validity + dimension/pixel-buffer consistency

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: cp_image_t encodes an image as width/height plus a raw mutable pointer to pixels. Correct usage implicitly requires that `pix` is either null (representing no buffer) or points to a valid, writable allocation of at least `w*h` pixels (and that `w`/`h` are non-negative and consistent with the buffer). None of these invariants are enforced by the type system because `pix` is `*mut cp_pixel_t` and the struct is `Copy`, so it can be duplicated without tracking ownership, lifetime, aliasing, or allocation size. This allows use-after-free, double-free (if someone treats it as owning), out-of-bounds access (if `w*h` exceeds allocation), and violating Rust aliasing rules (multiple copies containing the same `*mut`).

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

**States:** NullPixels (no backing buffer), HasPixels (points to writable pixel buffer)

**Transitions:**
- NullPixels -> HasPixels via external allocation/initialization of `pix` (not represented in this snippet)
- HasPixels -> NullPixels via external deallocation/reset of `pix` (not represented in this snippet)

**Evidence:** line 9: `pub pix: *mut cp_pixel_t` raw mutable pointer implies an implicit validity/initialization requirement; line 7-8: `pub w: i32`, `pub h: i32` imply implicit constraints (non-negative; buffer sized for `w*h` pixels); line 6: `#[derive(Copy, Clone)]` allows duplicating a raw mutable pointer without encoding lifetime/ownership/uniqueness; line 5: `#[repr(C)]` indicates FFI layout, suggesting the pointer is managed externally and must follow an FFI protocol

**Implementation:** Provide a safe Rust wrapper type around `cp_image_t` that stores pixels as a slice/Vec (or `NonNull<cp_pixel_t>` + length) and uses a newtype for validated dimensions (e.g., `NonNegativeI32` or `u32`). Example: `struct Image<'a> { w: u32, h: u32, pix: &'a mut [cp_pixel_t] }` for borrowed buffers, or `struct OwnedImage { w: u32, h: u32, pix: Vec<cp_pixel_t> }` for owned buffers. Keep `cp_image_t` as the raw FFI representation and only expose constructors that enforce `pix` non-null and `pix.len() == w*h` when converting to the safe wrapper.

---

