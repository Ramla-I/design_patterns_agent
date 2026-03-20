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

### 1. Raw-pointer slice validity precondition (NULL-or-valid, length matches allocation)

**Location**: `/data/test_case/lib.rs:1-168`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: These functions accept a raw pointer `p: *mut c_void` and a `len` and immediately form a slice via `from_raw_parts(p as *const u8, len)` unless `p.is_null()` or `len == 0`. Correctness/safety relies on an implicit precondition: either (a) `p` is null or `len == 0` (so an empty slice is used), or (b) `p` is non-null and points to at least `len` readable bytes for the duration of the call. The type system does not enforce this pointer/length coupling; it is enforced by `unsafe` plus a runtime `is_null`/`len==0` guard.

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

use core::mem;

#[inline(always)]
unsafe fn stbds_siphash_bytes(p: *mut core::ffi::c_void, len: usize, seed: usize) -> usize {
    // Use the actual `len` for the slice to avoid out-of-bounds reads.
    let mut d: &[u8] = if p.is_null() || len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(p as *const u8, len)
    };

    let word_bytes = mem::size_of::<usize>();
    let word_bits = word_bytes * 8;
    let half_bits = word_bits / 2;

    #[inline(always)]
    fn rotl(x: usize, r: usize) -> usize {
        x.rotate_left(r as u32)
    }

    #[inline(always)]
    fn sip_round(v0: &mut usize, v1: &mut usize, v2: &mut usize, v3: &mut usize, half_bits: usize) {
        *v0 = v0.wrapping_add(*v1);
        *v1 = rotl(*v1, 13);
        *v1 ^= *v0;
        *v0 = rotl(*v0, half_bits);

        *v2 = v2.wrapping_add(*v3);
        *v3 = rotl(*v3, 16);
        *v3 ^= *v2;

        *v2 = v2.wrapping_add(*v1);
        *v1 = rotl(*v1, 17);
        *v1 ^= *v2;
        *v2 = rotl(*v2, half_bits);

        *v0 = v0.wrapping_add(*v3);
        *v3 = rotl(*v3, 21);
        *v3 ^= *v0;
    }

    let mut v0: usize = (((0x736f6d65usize << 16) << 16).wrapping_add(0x70736575)) ^ seed;
    let mut v1: usize = (((0x646f7261usize << 16) << 16).wrapping_add(0x6e646f6d)) ^ !seed;
    let mut v2: usize = (((0x6c796765usize << 16) << 16).wrapping_add(0x6e657261)) ^ seed;
    let mut v3: usize = (((0x74656462usize << 16) << 16).wrapping_add(0x79746573)) ^ !seed;

    v0 = ((v0 as u64) ^ (0x0706050403020100u64 ^ seed as u64)) as usize;
    v1 = ((v1 as u64) ^ (0x0f0e0d0c0b0a0908u64 ^ (!seed) as u64)) as usize;
    v2 = ((v2 as u64) ^ (0x0706050403020100u64 ^ seed as u64)) as usize;
    v3 = ((v3 as u64) ^ (0x0f0e0d0c0b0a0908u64 ^ (!seed) as u64)) as usize;

    let mut i = 0usize;

    while i + word_bytes <= len {
        // The original code reads 8 bytes per block (usize assumed 64-bit in that codegen).
        // Keep behavior but guard against non-64-bit by only using the first 8 bytes when available.
        let mut data = (d[0] as usize)
            | ((d[1] as usize) << 8)
            | ((d[2] as usize) << 16)
            | ((d[3] as usize) << 24)
            | ((d[4] as usize) << 32)
            | ((d[5] as usize) << 40)
            | ((d[6] as usize) << 48)
            | ((d[7] as usize) << 56);

        v3 ^= data;
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
        v0 ^= data;

        i += word_bytes;
        d = &d[word_bytes..];
    }

    // Final partial block
    let mut data = len << (word_bits - 8);
    match len - i {
        7 => {
            data |= (d[6] as usize) << 48;
            data |= (d[5] as usize) << 40;
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        6 => {
            data |= (d[5] as usize) << 40;
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        5 => {
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        4 => {
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        3 => {
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        2 => {
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        1 => {
            data |= d[0] as usize;
        }
        0 | _ => {}
    }

    v3 ^= data;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    v0 ^= data;

    v2 ^= 0xff;
    for _ in 0..4 {
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    }

    v0 ^ v1 ^ v2 ^ v3
}

pub(crate) unsafe fn stbds_hash_bytes(p: *mut core::ffi::c_void, len: usize, seed: usize) -> usize {
    stbds_siphash_bytes(p, len, seed)
}

#[no_mangle]
pub unsafe extern "C" fn siphash(mut init: i32) {
    let mut mem: [u8; 64] = [0; 64];

    for b in mem.iter_mut() {
        *b = init as u8;
        init += 1;
    }

    for i in 0..64i32 {
        let hash: usize = stbds_hash_bytes(mem.as_mut_ptr() as *mut _, i as usize, 0);
        print!("  {{ ");
        for j in 0..8i32 {
            // Avoid dependency on `crate::c_lib::Xu32` (not present in this crate).
            print!("0x{0:>02x}, ", ((hash >> (j * 8)) & 0xff) as u8);
        }
        println!(" }},");
    }
}
```

**Entity:** stbds_siphash_bytes / stbds_hash_bytes (unsafe pointer+length API)

**States:** NullOrEmptyInput, ValidReadableBuffer

**Transitions:**
- NullOrEmptyInput -> ValidReadableBuffer by providing non-null p and len>0 at call site

**Evidence:** stbds_siphash_bytes signature: `unsafe fn stbds_siphash_bytes(p: *mut core::ffi::c_void, len: usize, seed: usize)`; runtime check: `if p.is_null() || len == 0 { &[] } else { core::slice::from_raw_parts(p as *const u8, len) }`; slice creation from raw parts: `core::slice::from_raw_parts(p as *const u8, len)` requires `p` be valid for `len` bytes

**Implementation:** Replace `(p: *mut c_void, len: usize)` with a safe wrapper parameter such as `&[u8]` for internal callers; for FFI, accept `NonNull<u8>` + `usize` in a validated newtype (e.g., `struct ByteSpan<'a>(&'a [u8]);` or `struct ValidPtrLen { ptr: NonNull<u8>, len: usize }` with `unsafe fn new(ptr,len)->Self`), then make the hashing function take that wrapper rather than raw parts.

---

### 2. Block size/word-size protocol (expects 8-byte blocks even when word_bytes != 8)

**Location**: `/data/test_case/lib.rs:1-168`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The loop condition uses `word_bytes = size_of::<usize>()` and iterates while `i + word_bytes <= len`, but the body unconditionally reads `d[0]..d[7]` to build `data`. This implicitly assumes each iteration has at least 8 bytes available (i.e., `word_bytes >= 8` and/or the code is running on a 64-bit-usize target where `word_bytes == 8`). On 32-bit targets (`word_bytes == 4`), the loop guard would allow entering with only 4 bytes remaining, yet the body would index up to `d[7]` (out of bounds). The code comments acknowledge an intent to “read 8 bytes per block”, but the actual guard is tied to `usize` size, not 8. This is an implicit target/protocol assumption not enforced by the type system.

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

use core::mem;

#[inline(always)]
unsafe fn stbds_siphash_bytes(p: *mut core::ffi::c_void, len: usize, seed: usize) -> usize {
    // Use the actual `len` for the slice to avoid out-of-bounds reads.
    let mut d: &[u8] = if p.is_null() || len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(p as *const u8, len)
    };

    let word_bytes = mem::size_of::<usize>();
    let word_bits = word_bytes * 8;
    let half_bits = word_bits / 2;

    #[inline(always)]
    fn rotl(x: usize, r: usize) -> usize {
        x.rotate_left(r as u32)
    }

    #[inline(always)]
    fn sip_round(v0: &mut usize, v1: &mut usize, v2: &mut usize, v3: &mut usize, half_bits: usize) {
        *v0 = v0.wrapping_add(*v1);
        *v1 = rotl(*v1, 13);
        *v1 ^= *v0;
        *v0 = rotl(*v0, half_bits);

        *v2 = v2.wrapping_add(*v3);
        *v3 = rotl(*v3, 16);
        *v3 ^= *v2;

        *v2 = v2.wrapping_add(*v1);
        *v1 = rotl(*v1, 17);
        *v1 ^= *v2;
        *v2 = rotl(*v2, half_bits);

        *v0 = v0.wrapping_add(*v3);
        *v3 = rotl(*v3, 21);
        *v3 ^= *v0;
    }

    let mut v0: usize = (((0x736f6d65usize << 16) << 16).wrapping_add(0x70736575)) ^ seed;
    let mut v1: usize = (((0x646f7261usize << 16) << 16).wrapping_add(0x6e646f6d)) ^ !seed;
    let mut v2: usize = (((0x6c796765usize << 16) << 16).wrapping_add(0x6e657261)) ^ seed;
    let mut v3: usize = (((0x74656462usize << 16) << 16).wrapping_add(0x79746573)) ^ !seed;

    v0 = ((v0 as u64) ^ (0x0706050403020100u64 ^ seed as u64)) as usize;
    v1 = ((v1 as u64) ^ (0x0f0e0d0c0b0a0908u64 ^ (!seed) as u64)) as usize;
    v2 = ((v2 as u64) ^ (0x0706050403020100u64 ^ seed as u64)) as usize;
    v3 = ((v3 as u64) ^ (0x0f0e0d0c0b0a0908u64 ^ (!seed) as u64)) as usize;

    let mut i = 0usize;

    while i + word_bytes <= len {
        // The original code reads 8 bytes per block (usize assumed 64-bit in that codegen).
        // Keep behavior but guard against non-64-bit by only using the first 8 bytes when available.
        let mut data = (d[0] as usize)
            | ((d[1] as usize) << 8)
            | ((d[2] as usize) << 16)
            | ((d[3] as usize) << 24)
            | ((d[4] as usize) << 32)
            | ((d[5] as usize) << 40)
            | ((d[6] as usize) << 48)
            | ((d[7] as usize) << 56);

        v3 ^= data;
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
        v0 ^= data;

        i += word_bytes;
        d = &d[word_bytes..];
    }

    // Final partial block
    let mut data = len << (word_bits - 8);
    match len - i {
        7 => {
            data |= (d[6] as usize) << 48;
            data |= (d[5] as usize) << 40;
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        6 => {
            data |= (d[5] as usize) << 40;
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        5 => {
            data |= (d[4] as usize) << 32;
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        4 => {
            data |= (d[3] as usize) << 24;
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        3 => {
            data |= (d[2] as usize) << 16;
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        2 => {
            data |= (d[1] as usize) << 8;
            data |= d[0] as usize;
        }
        1 => {
            data |= d[0] as usize;
        }
        0 | _ => {}
    }

    v3 ^= data;
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    v0 ^= data;

    v2 ^= 0xff;
    for _ in 0..4 {
        sip_round(&mut v0, &mut v1, &mut v2, &mut v3, half_bits);
    }

    v0 ^ v1 ^ v2 ^ v3
}

pub(crate) unsafe fn stbds_hash_bytes(p: *mut core::ffi::c_void, len: usize, seed: usize) -> usize {
    stbds_siphash_bytes(p, len, seed)
}

#[no_mangle]
pub unsafe extern "C" fn siphash(mut init: i32) {
    let mut mem: [u8; 64] = [0; 64];

    for b in mem.iter_mut() {
        *b = init as u8;
        init += 1;
    }

    for i in 0..64i32 {
        let hash: usize = stbds_hash_bytes(mem.as_mut_ptr() as *mut _, i as usize, 0);
        print!("  {{ ");
        for j in 0..8i32 {
            // Avoid dependency on `crate::c_lib::Xu32` (not present in this crate).
            print!("0x{0:>02x}, ", ((hash >> (j * 8)) & 0xff) as u8);
        }
        println!(" }},");
    }
}
```

**Entity:** stbds_siphash_bytes (block decoding assumes 8 accessible bytes per iteration)

**States:** AlignedFor8ByteBlockRead, InsufficientBytesFor8ByteBlockRead

**Transitions:**
- InsufficientBytesFor8ByteBlockRead -> AlignedFor8ByteBlockRead by restricting compilation/usage to targets where `size_of::<usize>() == 8` or by changing the loop guard to `i + 8 <= len`

**Evidence:** `let word_bytes = mem::size_of::<usize>();` and `while i + word_bytes <= len { ... }`; unconditional 8-byte indexing inside loop: `d[0] ... d[7]`; comment indicates 8-byte block assumption: `// The original code reads 8 bytes per block (usize assumed 64-bit in that codegen).`

**Implementation:** Enforce the target invariant at compile time with `#[cfg(not(target_pointer_width = "64"))] compile_error!("requires 64-bit usize");` or decouple from `usize` by using a fixed block size (`const BLOCK: usize = 8; while i + BLOCK <= len { ... d[0..8] ... }`). If you want a type-level signal, introduce a private `struct U64Target;` gated behind `cfg(target_pointer_width="64")` and only expose the function when that type exists.

---

