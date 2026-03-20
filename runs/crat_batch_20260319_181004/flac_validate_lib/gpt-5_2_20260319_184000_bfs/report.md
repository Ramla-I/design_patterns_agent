# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 0
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 1. tflac configuration vs per-block decoding state (static header fields + dynamic current block)

**Location**: `/data/test_case/lib.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The struct mixes long-lived stream/header parameters (blocksize, samplerate, channels, bitdepth, channel_mode, min/max partition order, max_rice_value) with per-block mutable decoding state (cur_blocksize, partition_order). This implies an implicit protocol: first the configuration fields must be set to valid stream values; then during decoding, cur_blocksize and partition_order change per block/frame but must remain consistent with the configured bounds (e.g., partition_order should stay within [min_partition_order, max_partition_order], and cur_blocksize likely should not exceed blocksize). None of these relationships or the separation between configured vs in-progress decoding state are enforced by the type system; all fields are public and the type is Copy, so invalid combinations and stale/aliased copies of 'current' state are easy to create.

**Evidence**:

```rust

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tflac {
    pub blocksize: tflac_u32,
    pub samplerate: tflac_u32,
    pub channels: tflac_u32,
    pub bitdepth: tflac_u32,
    pub channel_mode: tflac_u8,
    pub max_rice_value: tflac_u8,
    pub min_partition_order: tflac_u8,
    pub max_partition_order: tflac_u8,
    pub partition_order: tflac_u8,
    pub cur_blocksize: tflac_u32,
}

```

**Entity:** tflac

**States:** Configured (stream parameters set), DecodingBlock (cur_blocksize/partition_order reflect current frame/block)

**Transitions:**
- Configured -> DecodingBlock by updating cur_blocksize and partition_order for the current block/frame
- DecodingBlock -> DecodingBlock by updating cur_blocksize/partition_order for the next block/frame

**Evidence:** pub struct tflac { ... } has both configuration-like fields (blocksize, samplerate, channels, bitdepth, channel_mode, min_partition_order, max_partition_order) and per-block fields (partition_order, cur_blocksize); field names: min_partition_order/max_partition_order imply a bounds invariant for partition_order; field names: blocksize vs cur_blocksize imply a relationship (current block size derived from or limited by configured block size); #[derive(Copy, Clone)] on tflac allows duplicating the struct including 'current' fields (cur_blocksize/partition_order), suggesting implicit expectations about when copying is safe

**Implementation:** Split into distinct types: e.g., TflacConfig { blocksize, samplerate, channels, bitdepth, channel_mode, max_rice_value, min_partition_order, max_partition_order } and TflacDecoder<'a> { cfg: &'a TflacConfig, cur_blocksize: ..., partition_order: ... }. Alternatively use typestate: struct Tflac<S> { cfg: ..., state: S }, with S = Configured vs InBlock. Enforce bounds by making partition_order a newtype validated against min/max at construction.

---

