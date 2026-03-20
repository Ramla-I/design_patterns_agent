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

### 1. FFI pointer validity/length protocol (Null vs NonNull, and buffer length >= n)

**Location**: `/data/test_case/lib.rs:1-50`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The unsafe FFI entrypoint encodes a protocol around raw pointers: a null dest/src pointer is treated as an empty slice, while a non-null pointer is assumed to be valid for reads/writes of 1024 f32 elements (because from_raw_parts[_mut](..., 1024) is used unconditionally). Additionally, the logical element count used is n = max(size, 0), and the function only operates on the first n elements (bounded by dest/src slice lengths). None of these requirements (non-null implies valid for 1024 elements; pointer provenance; alignment; not dangling) are enforced by the type system at the call site; they are implicit in the unsafe conversion and the chosen fixed length.

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

pub(crate) fn normalize_internal(dest: &mut [f32], src: &[f32], size: i32) {
    let n = size.max(0) as usize;

    let sum_sq: f32 = src
        .get(..n)
        .unwrap_or(src)
        .iter()
        .map(|&x| x * x)
        .sum();

    if sum_sq > 0.0 {
        let inv_norm = 1.0f32 / sum_sq.sqrt();
        if let (Some(d), Some(s)) = (dest.get_mut(..n), src.get(..n)) {
            for (out, &x) in d.iter_mut().zip(s.iter()) {
                *out = x * inv_norm;
            }
        }
    } else if dest.as_mut_ptr() != src.as_ptr() as *mut f32 {
        if let Some(d) = dest.get_mut(..n) {
            d.fill(0.0);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn normalize(dest: *mut f32, src: *const f32, size: i32) {
    normalize_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
        size,
    )
}
```

**Entity:** normalize (extern "C" API: dest/src pointers + size)

**States:** NullPointer (treated as empty slice), NonNullPointer (must point to at least 1024 f32s, and at least n used)

**Transitions:**
- NullPointer -> NonNullPointer via caller providing a non-null pointer
- Any -> UB via caller providing non-null but not valid for 1024 f32 elements

**Evidence:** normalize: `if dest.is_null() { &mut [] } else { std::slice::from_raw_parts_mut(dest, 1024) }` (non-null implies a 1024-element writable allocation); normalize: `if src.is_null() { &[] } else { std::slice::from_raw_parts(src, 1024) }` (non-null implies a 1024-element readable allocation); normalize_internal: `let n = size.max(0) as usize;` (logical length derived from size; negative treated as 0); normalize_internal: `dest.get_mut(..n)` / `src.get(..n)` gates use on slices having at least n elements (runtime check rather than type-level contract)

**Implementation:** Expose a safe wrapper that takes `dest: Option<NonNull<[f32; 1024]>>` / `src: Option<NonNull<[f32; 1024]>>` (or `NonNull<f32>` plus a `const LEN: usize = 1024` newtype), and performs the `from_raw_parts[_mut]` internally. This makes the 'non-null => valid for 1024 f32' precondition explicit. If variable-length is intended, accept a length parameter and use `NonNull<[f32]>` fat pointers or pass `(ptr, len)` and wrap it in a validated slice newtype.

---

## Protocol Invariants

### 2. Aliasing-sensitive behavior (in-place vs out-of-place) depends on pointer equality

**Location**: `/data/test_case/lib.rs:1-50`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The algorithm has an implicit protocol regarding whether the operation is in-place (dest aliases src) or out-of-place. When `sum_sq == 0.0`, it only zero-fills `dest[..n]` if dest and src do not alias, detected via raw pointer comparison. If they alias, it intentionally does nothing (to avoid clobbering src, or to preserve behavior). This aliasing-dependent behavior is not expressible in the current signature (`&mut [f32]`, `&[f32]`), and the code relies on a runtime pointer equality check; callers through FFI can violate Rust aliasing rules by providing overlapping regions that are not exactly the same pointer, leading to surprising results or UB at the FFI boundary.

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

pub(crate) fn normalize_internal(dest: &mut [f32], src: &[f32], size: i32) {
    let n = size.max(0) as usize;

    let sum_sq: f32 = src
        .get(..n)
        .unwrap_or(src)
        .iter()
        .map(|&x| x * x)
        .sum();

    if sum_sq > 0.0 {
        let inv_norm = 1.0f32 / sum_sq.sqrt();
        if let (Some(d), Some(s)) = (dest.get_mut(..n), src.get(..n)) {
            for (out, &x) in d.iter_mut().zip(s.iter()) {
                *out = x * inv_norm;
            }
        }
    } else if dest.as_mut_ptr() != src.as_ptr() as *mut f32 {
        if let Some(d) = dest.get_mut(..n) {
            d.fill(0.0);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn normalize(dest: *mut f32, src: *const f32, size: i32) {
    normalize_internal(
        if dest.is_null() {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(dest, 1024)
        },
        if src.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(src, 1024)
        },
        size,
    )
}
```

**Entity:** normalize_internal (interaction between dest/src aliasing and sum_sq==0 branch)

**States:** Aliased (dest and src same base address), NonAliased (dest and src different base address)

**Transitions:**
- NonAliased -> Aliased via caller passing same buffer for dest and src (FFI)
- Aliased -> NonAliased via caller passing distinct buffers (FFI)

**Evidence:** normalize_internal: `} else if dest.as_mut_ptr() != src.as_ptr() as *mut f32 { ... d.fill(0.0); }` (branch behavior depends on aliasing detected by pointer equality); normalize_internal signature: `dest: &mut [f32], src: &[f32]` (type system does not encode 'may alias' vs 'must not alias' for the FFI use-case)

**Implementation:** Split APIs into two safe entrypoints: one for out-of-place normalization that requires non-aliasing via a dedicated wrapper (e.g., `struct DistinctSlices<'a> { dest: &'a mut [f32], src: &'a [f32] }` constructed only after checking `!ptr::eq(...)` and non-overlap), and one for in-place normalization taking `&mut [f32]` only. The FFI layer can decide which to call based on pointer/overlap checks, making the protocol explicit and preventing accidental overlap usage in the 'distinct' path.

---

