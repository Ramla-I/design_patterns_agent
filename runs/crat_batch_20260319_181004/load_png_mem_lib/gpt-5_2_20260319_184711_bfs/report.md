# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 3. cp_image_t raw buffer validity (null/allocated, size-consistent, lifetime/ownership)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: cp_image_t is an FFI-style image descriptor where `pix` is a raw pointer to pixel storage. Correct usage implicitly depends on runtime/semantic invariants not enforced by the type system: (1) whether `pix` is null or points to a valid allocation, (2) whether that allocation is large enough for `w*h` pixels, and (3) who owns the allocation and how long it lives. Because the struct is `Copy`, the pointer can be duplicated freely, which can silently duplicate the 'handle' to the same allocation and make ownership/freeing protocols (if any exist elsewhere in the crate) impossible to express safely at compile time. There is also an implicit size/shape invariant: when `pix` is non-null, `w` and `h` should be non-negative and correspond to the actual buffer length.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_pixel_t, 4 free function(s); struct cp_state_t, 11 free function(s); struct cp_raw_png_t, 2 free function(s); 7 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_image_t {
            pub w: i32,
            pub h: i32,
            pub pix: *mut cp_pixel_t,
        }

```

**Entity:** cp_image_t

**States:** NullPixels (no backing buffer), HasPixels (backing buffer present)

**Transitions:**
- NullPixels -> HasPixels via external/FFI allocation and assignment to `pix`
- HasPixels -> NullPixels via external/FFI free/release and clearing `pix` (if done)

**Evidence:** line 8: `pub pix: *mut cp_pixel_t` is a raw pointer that may be null/dangling and carries no lifetime/length information; line 6-7: `pub w: i32`, `pub h: i32` imply a length/shape that must match the allocation behind `pix`; line 4: `#[derive(Copy, Clone)]` allows duplicating `cp_image_t` (and thus `pix`) without tracking ownership or aliasing constraints

**Implementation:** Introduce a safe wrapper that encodes validity: e.g., `struct Image<'a> { w: NonZeroU32/usize, h: NonZeroU32/usize, pix: NonNull<cp_pixel_t>, _buf: PhantomData<&'a mut [cp_pixel_t]> }` (borrowed) or an owning `struct OwnedImage { w: usize, h: usize, buf: Vec<cp_pixel_t> }`. Provide explicit constructors from raw parts (`unsafe fn from_raw_parts(...)`) that validate `w/h` and require a `NonNull` pointer, and avoid `Copy` on owning/unique types to prevent accidental double-free protocols.

---

## Precondition Invariants

### 2. Raw PNG byte-range invariant (valid non-null contiguous range p..end)

**Location**: `/data/test_case/lib.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: cp_raw_png_t encodes a byte slice as two raw pointers (start `p` and end `end`). Correct use implicitly requires that both pointers are either a well-formed half-open range into the same allocated object (typically a PNG buffer) with `p <= end`, and that the memory remains alive/immutable for the duration of use. None of these invariants (non-null, ordering, provenance/same allocation, lifetime) are enforced by the type system because the fields are `*const u8`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_pixel_t, 4 free function(s); struct cp_image_t, 3 free function(s); struct cp_state_t, 11 free function(s); 7 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_raw_png_t {
            pub p: *const u8,
            pub end: *const u8,
        }

```

**Entity:** cp_raw_png_t

**States:** Invalid (null/out-of-bounds/misordered pointers), Valid (non-null, p <= end, points to same allocation)

**Transitions:**
- Invalid -> Valid via construction that sets `p`/`end` from a real buffer (implicit; not enforced here)

**Evidence:** struct cp_raw_png_t { p: *const u8, end: *const u8 } uses raw pointers to represent a range; #[repr(C)] indicates FFI layout expectations, increasing reliance on external code to uphold pointer validity; #[derive(Copy, Clone)] allows duplicating the struct without tying it to the backing buffer lifetime, implying an unenforced 'must outlive uses' requirement

**Implementation:** Wrap the pair in a safe Rust type that carries a lifetime and enforces ordering, e.g. `struct RawPng<'a>(&'a [u8]);` and (if FFI requires the C layout) provide `impl<'a> From<&'a [u8]> for cp_raw_png_t` plus an unsafe `fn as_slice<'a>(&self) -> &'a [u8]` only when the caller can prove validity. Alternatively store `NonNull<u8>` + `len: usize` (or `NonNull<u8>` + `NonNull<u8>` with a constructor that checks `p <= end`) and expose only checked constructors.

---

## Protocol Invariants

### 1. cp_state_t decode context protocol (Uninitialized/Configured -> Active -> Finished)

**Location**: `/data/test_case/lib.rs:1-25`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: cp_state_t is a C-FFI style mutable decoding context whose fields implicitly encode multiple phases of use. Several raw pointers and index/counter fields must be set up consistently before decoding starts (e.g., input word buffer pointers + indices, output buffer begin/end), then mutated while decoding (bit buffer, word index, bits_left, output cursor), and finally treated as finished once input/output is exhausted or a final word is consumed. None of these phases are represented in the type system: raw pointers may be null/dangling, indices can be out of range, and counters can be inconsistent with the pointed-to buffers. Safe Rust could enforce these via typestate and lifetimes tying pointers to actual slices/buffers, preventing calling ‘decode steps’ before proper initialization and preventing out-of-bounds index usage.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_pixel_t, 4 free function(s); struct cp_image_t, 3 free function(s); struct cp_raw_png_t, 2 free function(s); 7 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_state_t {
            pub bits: u64,
            pub count: i32,
            pub words: *mut u32,
            pub word_count: i32,
            pub word_index: i32,
            pub bits_left: i32,
            pub final_word_available: i32,
            pub final_word: u32,
            pub out: *mut i8,
            pub out_end: *mut i8,
            pub begin: *mut i8,
            pub lookup: [u16; 512],
            pub lit: [u32; 288],
            pub dst: [u32; 32],
            pub len: [u32; 19],
            pub nlit: u32,
            pub ndst: u32,
            pub nlen: u32,
        }

```

**Entity:** cp_state_t

**States:** Uninitialized, Configured, ActiveDecoding, FinishedOrErrored

**Transitions:**
- Uninitialized -> Configured by initializing pointers/counters (e.g., words/word_count/out/out_end/begin) and tables (lookup/lit/dst/len + nlit/ndst/nlen)
- Configured -> ActiveDecoding by starting bit/word consumption (bits/bits_left/word_index/count)
- ActiveDecoding -> FinishedOrErrored when input exhausted / final_word consumed (final_word_available/final_word) or output cursor reaches out_end

**Evidence:** cp_state_t.words: *mut u32 and cp_state_t.word_count: i32 imply an external input buffer with a required bounds relationship (word_index < word_count); cp_state_t.word_index: i32, cp_state_t.bits_left: i32, cp_state_t.bits: u64, cp_state_t.count: i32 are typical streaming/bitbuffer state that must be updated in a defined order during decoding; cp_state_t.final_word_available: i32 and cp_state_t.final_word: u32 encode a two-state flag+payload protocol (only valid to read final_word when final_word_available != 0); cp_state_t.out: *mut i8, cp_state_t.out_end: *mut i8, cp_state_t.begin: *mut i8 imply an output range and a current cursor; requires begin <= out <= out_end and out must not advance past out_end; cp_state_t.lookup: [u16; 512], lit: [u32; 288], dst: [u32; 32], len: [u32; 19] plus nlit/ndst/nlen counters imply ‘tables configured’ state where counts must match valid ranges before use

**Implementation:** Wrap cp_state_t in a safe Rust API: struct State<'a, S> { raw: cp_state_t, _s: PhantomData<S>, _in: PhantomData<&'a mut [u32]>, _out: PhantomData<&'a mut [u8]> }. Provide constructors like State<Uninit>::new(), then configure(self, input: &'a mut [u32], output: &'a mut [u8], tables: Tables) -> State<Configured>, then start(self) -> State<Active>. During Active, expose safe accessors that use usize indices (word_index: usize) and enforce bounds. Encode final_word_available as Option<u32> (or a small enum) instead of (i32,u32).

---

