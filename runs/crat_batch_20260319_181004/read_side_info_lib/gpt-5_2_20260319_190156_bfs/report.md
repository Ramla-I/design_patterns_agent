# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 2. L3_gr_info_t pointer validity + table/field coherence (FFI preconditions)

**Location**: `/data/test_case/lib.rs:1-23`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: L3_gr_info_t is an FFI-style, #[repr(C)] Copy struct whose fields are expected to satisfy non-local invariants before use (e.g., sfbtab must point to a valid table; various small integer fields are indices/counts that must be within ranges, and arrays like table_select/region_count/subblock_gain must be interpreted consistently with other mode fields like block_type and mixed_block_flag). None of these preconditions are enforced by the type system: sfbtab is a raw pointer (can be null/dangling), and many fields are plain integers that can encode invalid values. Because the type is Copy/Clone, invalid states can be trivially duplicated and propagated.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct bs_t, 3 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct L3_gr_info_t {
            pub sfbtab: *const u8,
            pub part_23_length: u16,
            pub big_values: u16,
            pub scalefac_compress: u16,
            pub global_gain: u8,
            pub block_type: u8,
            pub mixed_block_flag: u8,
            pub n_long_sfb: u8,
            pub n_short_sfb: u8,
            pub table_select: [u8; 3],
            pub region_count: [u8; 3],
            pub subblock_gain: [u8; 3],
            pub preflag: u8,
            pub scalefac_scale: u8,
            pub count1_table: u8,
            pub scfsi: u8,
        }

```

**Entity:** L3_gr_info_t

**States:** Invalid/Uninitialized (may contain null/garbage), Valid (all internal pointers/fields coherent for decoding)

**Transitions:**
- Invalid/Uninitialized -> Valid via an (external) initialization/fill step (not shown in snippet)

**Evidence:** L3_gr_info_t: #[repr(C)] indicates FFI layout expectations; L3_gr_info_t: #[derive(Copy, Clone)] allows duplicating whatever state (valid or invalid) the struct currently holds; field sfbtab: *const u8 is a raw pointer with no non-null / lifetime / bounds guarantees; fields block_type: u8 and mixed_block_flag: u8 suggest mode-dependent interpretation of other fields (e.g., n_long_sfb/n_short_sfb, region_count, subblock_gain) but are not encoded as an enum/typestate; fields table_select: [u8; 3], region_count: [u8; 3], subblock_gain: [u8; 3] are index-like values with implicit range constraints not represented in types

**Implementation:** Wrap raw/index-like fields in validated types (e.g., NonNull<u8> for sfbtab, BlockType enum for block_type, boolean for mixed_block_flag, newtypes for counts/indices with constructors that validate ranges). Optionally provide a safe Rust constructor (or builder) that returns a validated L3GrInfo wrapper, while keeping the raw #[repr(C)] struct only for FFI interchange.

---

### 1. bs_t buffer cursor validity (Null/Invalid vs Valid range)

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: bs_t is an FFI-style buffer cursor: `buf` is a raw pointer and `pos`/`limit` describe the readable range. Correct use implicitly requires `buf` to be non-null (or otherwise point to valid memory), and `pos`/`limit` to satisfy an invariant like `0 <= pos <= limit` and `limit` not exceeding the backing buffer length. None of these are enforced by the type system because the fields are public and use raw pointer + plain integers; the type permits constructing states where `buf` is null/dangling or `pos`/`limit` are negative or inverted, which would make any consumer doing pointer arithmetic/reads unsafe at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct L3_gr_info_t

    pub mod lib {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct bs_t {
            pub buf: *const u8,
            pub pos: i32,
            pub limit: i32,
        }

```

**Entity:** lib::bs_t

**States:** Invalid (null or out-of-bounds cursor), Valid (buf non-null, pos/limit define in-bounds range)

**Transitions:**
- Invalid -> Valid via constructing/initializing fields with a real buffer and consistent pos/limit
- Valid -> Invalid via mutation of public fields (e.g., setting buf to null or pos > limit)

**Evidence:** struct bs_t fields are all public: `pub buf: *const u8`, `pub pos: i32`, `pub limit: i32`; `buf: *const u8` is a raw pointer; can be null/dangling and lacks lifetime/length information; `pos: i32` and `limit: i32` allow negative values and do not enforce `pos <= limit`

**Implementation:** Make bs_t an internal struct with private fields and provide a safe constructor like `fn new(buf: NonNull<u8>, len: usize) -> Bs<Valid>` that stores a slice/lifetime (`&'a [u8]` or `NonNull<u8> + len`) and uses `usize` for `pos/limit`. Optionally use a typestate `Bs<Unchecked>` -> `Bs<Checked>` if validation must be separated for FFI inputs.

---

