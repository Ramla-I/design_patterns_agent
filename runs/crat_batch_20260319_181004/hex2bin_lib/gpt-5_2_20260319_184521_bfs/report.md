# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## State Machine Invariants

### 1. Hex decoding state machine (NibbleBoundary / HalfByteAccumulated) and ignore policy

**Location**: `/data/test_case/lib.rs:1-148`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The decoder maintains an implicit two-state automaton in `state`: at a nibble boundary it may (a) accept an ignored character (via `strchr(ignore, c)`) and stay in the boundary state, or (b) accept a hex digit and transition to the half-byte-accumulated state by storing the high nibble in `c_acc`. In the half-byte-accumulated state it must accept exactly one hex digit next, produce an output byte, and transition back to nibble boundary. If input ends (or breaks) while in HalfByteAccumulated, that is an error (odd number of hex digits) and the function rewinds `hex_pos` by 1. None of this protocol is represented in the types; it is enforced by the `state` byte and runtime branches.

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

extern "C" {
    fn strchr(__s: *const i8, __c: i32) -> *mut i8;
}

pub(crate) unsafe fn hex2bin_internal(
    bin: &mut [u8],
    bin_maxlen: usize,
    hex: &[i8],
    hex_len: usize,
    ignore: &[i8],
    hex_end_p: Option<&mut *const i8>,
) -> i32 {
    let mut bin_pos: usize = 0;
    let mut hex_pos: usize = 0;
    let mut ret: i32 = 0;

    let mut c_acc: u8 = 0;
    let mut state: u8 = 0;

    while hex_pos < hex_len {
        let c = hex[hex_pos] as u8;

        // Branchless-ish hex digit decode (preserve original logic)
        let c_num = (c as u32 ^ 48) as u8;
        let c_num0 = ((c_num as u32).wrapping_sub(10) >> 8) as u8;

        let c_alpha = (c as u32 & !32u32).wrapping_sub(55) as u8;
        let c_alpha0 =
            (((c_alpha as u32).wrapping_sub(10) ^ (c_alpha as u32).wrapping_sub(16)) >> 8) as u8;

        let is_hex_digit = ((c_num0 as i32 | c_alpha0 as i32) as u32) != 0;

        if !is_hex_digit {
            // If we're at a nibble boundary (state==0), allow skipping ignored chars.
            let can_ignore = !ignore.is_empty()
                && state == 0
                && !strchr(ignore.as_ptr(), c as i32).is_null();

            if !can_ignore {
                break;
            }
            hex_pos = hex_pos.wrapping_add(1);
            continue;
        }

        let c_val = (c_num0 as i32 & c_num as i32 | c_alpha0 as i32 & c_alpha as i32) as u8;

        if bin_pos >= bin_maxlen {
            ret = -1;
            break;
        }

        if state == 0 {
            c_acc = (c_val as u32).wrapping_mul(16) as u8;
        } else {
            bin[bin_pos] = (c_acc as i32 | c_val as i32) as u8;
            bin_pos = bin_pos.wrapping_add(1);
        }

        state = (!state) as u8;
        hex_pos = hex_pos.wrapping_add(1);
    }

    // Odd number of hex digits: error, and point to the last processed char.
    if state != 0 {
        hex_pos = hex_pos.wrapping_sub(1);
        ret = -1;
    }

    if ret != 0 {
        bin_pos = 0;
    }

    if let Some(endp) = hex_end_p {
        *endp = hex[hex_pos..].as_ptr();
    } else if hex_pos != hex_len {
        ret = -1;
    }

    if ret != 0 {
        ret
    } else {
        bin_pos as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn hex2bin(
    bin: *mut u8,
    bin_maxlen: usize,
    hex: *const i8,
    hex_len: usize,
    ignore: *const i8,
    hex_end_p: Option<&mut *const i8>,
) -> i32 {
    // Create slices sized to the maximum lengths we may touch.
    // We must not read/write beyond `hex_len`/`bin_maxlen`, but we also must not
    // create a slice longer than the actual allocation. The original code used
    // a fixed 1024; keep that conservative cap while also respecting provided maxes.
    let bin_slice_len = if bin.is_null() {
        0
    } else {
        core::cmp::min(1024, bin_maxlen)
    };
    let hex_slice_len = if hex.is_null() {
        0
    } else {
        core::cmp::min(1024, hex_len)
    };

    let bin_slice: &mut [u8] = if bin.is_null() {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(bin, bin_slice_len)
    };

    let hex_slice: &[i8] = if hex.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(hex, hex_slice_len)
    };

    // `ignore` is a C string for strchr; keep the original conservative cap.
    let ignore_slice: &[i8] = if ignore.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(ignore, 1024)
    };

    hex2bin_internal(
        bin_slice,
        bin_maxlen,
        hex_slice,
        hex_len,
        ignore_slice,
        hex_end_p,
    )
}
```

**Entity:** hex2bin_internal

**States:** NibbleBoundary (state==0), HalfByteAccumulated (state!=0)

**Transitions:**
- NibbleBoundary -> HalfByteAccumulated via reading a hex digit (sets c_acc = c_val*16)
- HalfByteAccumulated -> NibbleBoundary via reading a hex digit (writes bin[bin_pos] = c_acc|c_val and increments bin_pos)
- NibbleBoundary -> NibbleBoundary via skipping an ignored char (only when state==0 and strchr matches)
- HalfByteAccumulated -> Error via loop end/break with state!=0 (odd number of digits)

**Evidence:** `let mut c_acc: u8 = 0; let mut state: u8 = 0;` encodes the automaton state in a byte; `let can_ignore = !ignore.is_empty() && state == 0 && !strchr(ignore.as_ptr(), c as i32).is_null();` shows ignore is only permitted at nibble boundary; `if state == 0 { c_acc = (c_val as u32).wrapping_mul(16) as u8; } else { bin[bin_pos] = (c_acc as i32 | c_val as i32) as u8; bin_pos += 1; }` shows distinct behaviors per state; `state = (!state) as u8;` toggles between the two states after each hex digit; `if state != 0 { hex_pos = hex_pos.wrapping_sub(1); ret = -1; }` enforces the “must end on nibble boundary” invariant at runtime

**Implementation:** Factor the loop body into a small decoder type `Decoder<S>` where `S` is `AtBoundary` or `HaveHighNibble(u8)`; `push_hex_digit` on `Decoder<AtBoundary>` returns `Decoder<HaveHighNibble>`; `push_hex_digit` on `Decoder<HaveHighNibble>` returns `Decoder<AtBoundary>` plus an output byte. Only `Decoder<AtBoundary>` would offer `try_ignore_char(...)`, making the “ignore only at boundary” rule compile-time visible within Rust code.

---

## Precondition Invariants

### 2. FFI pointer/length/allocation precondition protocol (non-null pointers must be valid for advertised lengths)

**Location**: `/data/test_case/lib.rs:1-148`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The wrapper attempts to be defensive by creating slices capped at 1024, but the function still requires an implicit FFI protocol: if `bin`/`hex`/`ignore` are non-null, they must point to allocations readable/writable for at least the slice lengths created (and semantically for at least `hex_len` / `bin_maxlen` as used by `hex2bin_internal`). This validity relationship between raw pointers and the provided lengths is not enforced by the type system; it relies on caller discipline and runtime null checks. Additionally, `ignore` is treated as a C string for `strchr`, implying it must be NUL-terminated somewhere in the accessible range.

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

extern "C" {
    fn strchr(__s: *const i8, __c: i32) -> *mut i8;
}

pub(crate) unsafe fn hex2bin_internal(
    bin: &mut [u8],
    bin_maxlen: usize,
    hex: &[i8],
    hex_len: usize,
    ignore: &[i8],
    hex_end_p: Option<&mut *const i8>,
) -> i32 {
    let mut bin_pos: usize = 0;
    let mut hex_pos: usize = 0;
    let mut ret: i32 = 0;

    let mut c_acc: u8 = 0;
    let mut state: u8 = 0;

    while hex_pos < hex_len {
        let c = hex[hex_pos] as u8;

        // Branchless-ish hex digit decode (preserve original logic)
        let c_num = (c as u32 ^ 48) as u8;
        let c_num0 = ((c_num as u32).wrapping_sub(10) >> 8) as u8;

        let c_alpha = (c as u32 & !32u32).wrapping_sub(55) as u8;
        let c_alpha0 =
            (((c_alpha as u32).wrapping_sub(10) ^ (c_alpha as u32).wrapping_sub(16)) >> 8) as u8;

        let is_hex_digit = ((c_num0 as i32 | c_alpha0 as i32) as u32) != 0;

        if !is_hex_digit {
            // If we're at a nibble boundary (state==0), allow skipping ignored chars.
            let can_ignore = !ignore.is_empty()
                && state == 0
                && !strchr(ignore.as_ptr(), c as i32).is_null();

            if !can_ignore {
                break;
            }
            hex_pos = hex_pos.wrapping_add(1);
            continue;
        }

        let c_val = (c_num0 as i32 & c_num as i32 | c_alpha0 as i32 & c_alpha as i32) as u8;

        if bin_pos >= bin_maxlen {
            ret = -1;
            break;
        }

        if state == 0 {
            c_acc = (c_val as u32).wrapping_mul(16) as u8;
        } else {
            bin[bin_pos] = (c_acc as i32 | c_val as i32) as u8;
            bin_pos = bin_pos.wrapping_add(1);
        }

        state = (!state) as u8;
        hex_pos = hex_pos.wrapping_add(1);
    }

    // Odd number of hex digits: error, and point to the last processed char.
    if state != 0 {
        hex_pos = hex_pos.wrapping_sub(1);
        ret = -1;
    }

    if ret != 0 {
        bin_pos = 0;
    }

    if let Some(endp) = hex_end_p {
        *endp = hex[hex_pos..].as_ptr();
    } else if hex_pos != hex_len {
        ret = -1;
    }

    if ret != 0 {
        ret
    } else {
        bin_pos as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn hex2bin(
    bin: *mut u8,
    bin_maxlen: usize,
    hex: *const i8,
    hex_len: usize,
    ignore: *const i8,
    hex_end_p: Option<&mut *const i8>,
) -> i32 {
    // Create slices sized to the maximum lengths we may touch.
    // We must not read/write beyond `hex_len`/`bin_maxlen`, but we also must not
    // create a slice longer than the actual allocation. The original code used
    // a fixed 1024; keep that conservative cap while also respecting provided maxes.
    let bin_slice_len = if bin.is_null() {
        0
    } else {
        core::cmp::min(1024, bin_maxlen)
    };
    let hex_slice_len = if hex.is_null() {
        0
    } else {
        core::cmp::min(1024, hex_len)
    };

    let bin_slice: &mut [u8] = if bin.is_null() {
        &mut []
    } else {
        core::slice::from_raw_parts_mut(bin, bin_slice_len)
    };

    let hex_slice: &[i8] = if hex.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(hex, hex_slice_len)
    };

    // `ignore` is a C string for strchr; keep the original conservative cap.
    let ignore_slice: &[i8] = if ignore.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(ignore, 1024)
    };

    hex2bin_internal(
        bin_slice,
        bin_maxlen,
        hex_slice,
        hex_len,
        ignore_slice,
        hex_end_p,
    )
}
```

**Entity:** hex2bin (extern "C" wrapper)

**States:** NullInputs (some pointers null), ValidInputs (non-null pointers point to enough accessible memory)

**Transitions:**
- NullInputs -> ValidInputs via caller providing non-null pointers with sufficient backing storage (FFI-side transition, not represented in Rust)

**Evidence:** `pub unsafe extern "C" fn hex2bin(bin: *mut u8, ..., hex: *const i8, ..., ignore: *const i8, ...)` uses raw pointers and is `unsafe`, indicating required caller-side invariants; Null checks: `if bin.is_null() { ... } else { from_raw_parts_mut(bin, bin_slice_len) }` and similarly for `hex` and `ignore`; Comment: `We must not read/write beyond hex_len/bin_maxlen, but we also must not create a slice longer than the actual allocation.` describes a protocol between pointer validity and lengths; `core::slice::from_raw_parts(ignore, 1024)` plus `strchr(ignore.as_ptr(), ...)` implies `ignore` must be valid memory and contain a terminating NUL for `strchr` to stop

**Implementation:** Introduce safe Rust entrypoints that accept `&mut [u8]` and `&[u8]`/`&[i8]` (or `CStr` for `ignore`) and call `hex2bin_internal` directly. Keep the `extern "C"` function as a thin `unsafe` adapter that validates (or at least documents) the required pointer/length invariants before constructing slices. Optionally use a `NonNull<T>`-based newtype like `struct FfiBufMut<'a>(NonNull<u8>, usize, PhantomData<&'a mut [u8]>)` to make “non-null + length go together” explicit on the Rust side.

---

