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

### 1. tflac_bitwriter buffer/position validity invariant (Uninitialized vs Initialized)

**Location**: `/data/test_case/lib.rs:1-11`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: tflac_bitwriter encodes a bitstream writer whose internal fields imply a runtime validity contract: `buffer` must point to writable memory and `len` must describe its usable capacity; `pos` must remain within bounds (and likely tracks current byte/bit position alongside `bits`/`val`). Because `buffer` is a raw pointer and the struct is `Copy`, nothing in the type system prevents creating/duplicating instances with a null/dangling buffer, mismatched `len`, or out-of-range `pos`, nor does it prevent using a writer before it has been properly initialized with a backing buffer.

**Evidence**:

```rust

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct tflac_bitwriter {
            pub val: tflac_uint,
            pub bits: tflac_u32,
            pub pos: tflac_u32,
            pub len: tflac_u32,
            pub tot: tflac_u32,
            pub buffer: *mut tflac_u8,
        }

```

**Entity:** tflac_bitwriter

**States:** Uninitialized (buffer == null/len==0), Initialized (buffer non-null with valid capacity, pos<=len)

**Transitions:**
- Uninitialized -> Initialized via external initialization that sets buffer/len/pos (not shown in snippet)

**Evidence:** struct field `buffer: *mut tflac_u8` is a raw mutable pointer with no lifetime/aliasing/null guarantees; struct fields `pos: tflac_u32` and `len: tflac_u32` imply a bounds relationship (pos within len) that is not encoded; `#[derive(Copy, Clone)]` on `tflac_bitwriter` allows duplicating the writer, weakening any uniqueness/ownership expectations for `buffer`

**Implementation:** Represent initialization as a typestate: `struct BitWriter<S> { val: u32, bits: u32, pos: u32, tot: u32, buf: NonNull<u8>, len: usize, _s: PhantomData<S> }` with `Uninit`/`Init` states. Provide `BitWriter<Uninit>::with_buffer(buf: &mut [u8]) -> BitWriter<Init>`; only implement write/flush methods for `BitWriter<Init>`. Use `NonNull<u8>` + `usize` for len, and avoid `Copy` (or gate cloning behind explicit semantics).

---

