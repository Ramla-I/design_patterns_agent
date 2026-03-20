# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 2
- **Modules analyzed**: 1

## Protocol Invariants

### 1. Buffer-processing pipeline protocol (valid-length prefix + flag-gated stages)

**Location**: `/data/test_case/main.rs:1-282`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The code relies on an implicit protocol where `length` denotes the active prefix of `buffer` that may be read/written by each stage, and stages must be executed in a specific order with guards (flags and `new_len` thresholds) so helpers never observe out-of-range indices or invalid parameters. This is enforced by runtime checks (e.g., `new_len >= 2`) and by the caller convention that `buffer.len() == length` (in main) and that each transformation returns an updated `new_len` to be used by later stages. None of these relationships are enforced by the type system: helpers accept an arbitrary `&mut [u8]` plus a separate `len` that may not match, and `flags` is a raw bitmask that can request incompatible/meaningless operations (e.g., interleave when `new_len < 2`, reverse_segments when `seg_size > new_len`), relying on internal conditionals to no-op safely.

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

use std::io::{self, Read};

fn process_buffer(buffer: &mut [u8], length: usize, flags: u32, param1: i32, param2: i32) -> usize {
    let mut new_len: usize = length;
    if buffer.is_empty() || length == 0 {
        return 0;
    }

    if flags & 0x1 != 0 {
        // Must match original: offset computed with i32 % i32 (can be negative),
        // then rotate_buffer normalizes negative via wrapping_add(len as u64).
        let offset: i32 = param1 % length as i32;
        if offset != 0 {
            rotate_buffer(buffer, length, offset);
        }
    }
    if flags & 0x2 != 0 {
        let threshold: u8 = (if param1 > 0 && param1 <= 255 { param1 } else { 3 }) as u8;
        new_len = compact_runs(buffer, new_len, threshold);
    }
    if flags & 0x4 != 0 {
        let preserve: i32 = (param2 != 0) as i32;
        new_len = remove_duplicates(buffer, new_len, preserve);
    }
    if flags & 0x8 != 0 && new_len >= 2 {
        interleave_halves(buffer, new_len);
    }
    if flags & 0x10 != 0 && new_len >= 4 {
        let seg_size: usize = if param1 > 0 { param1 as usize } else { 4 };
        if seg_size <= new_len {
            reverse_segments(buffer, new_len, seg_size);
        }
    }
    new_len
}

// Matches the original C2Rust rotate_buffer behavior:
// It performs a RIGHT rotation by `offset` (after normalization).
fn rotate_buffer(buf: &mut [u8], len: usize, mut offset: i32) {
    if len <= 1 {
        return;
    }
    offset %= len as i32;
    if offset < 0 {
        // Match original: offset = (offset as u64).wrapping_add(len as u64) as i32;
        offset = ((offset as u32 as u64).wrapping_add(len as u64)) as i32;
    }
    if offset == 0 {
        return;
    }
    let off = offset as usize;
    buf[..len].rotate_right(off);
}

fn compact_runs(buf: &mut [u8], mut len: usize, threshold: u8) -> usize {
    let mut read: usize = 0;
    let mut write: usize = 0;

    while read < len {
        let current: u8 = buf[read];
        let mut run_len: usize = 1;
        while read + run_len < len && buf[read + run_len] == current {
            run_len += 1;
        }

        if run_len >= threshold as usize {
            if run_len > 255 {
                run_len = 255;
            }
            buf[write] = current;
            write += 1;
            buf[write] = run_len as u8;
            write += 1;

            if read + run_len < len {
                let remaining = len - (read + run_len);
                buf.copy_within(read + run_len..read + run_len + remaining, write);
            }
            len = write + (len - (read + run_len));
            read = write;
        } else {
            if write != read {
                buf.copy_within(read..read + run_len, write);
            }
            write += run_len;
            read += run_len;
        }
    }

    len
}

fn remove_duplicates(buf: &mut [u8], len: usize, preserve_order: i32) -> usize {
    if len <= 1 {
        return len;
    }

    if preserve_order != 0 {
        let mut write: usize = 1;
        let mut i: usize = 1;
        while i < len {
            let mut j: usize = 0;
            while j < write {
                if buf[i] == buf[j] {
                    break;
                }
                j += 1;
            }
            if j == write {
                if write != i {
                    buf[write] = buf[i];
                }
                write += 1;
            }
            i += 1;
        }
        write
    } else {
        let mut seen = [0u8; 256];
        let mut write: usize = 0;
        let mut i: usize = 0;
        while i < len {
            let idx = buf[i] as usize;
            if seen[idx] == 0 {
                seen[idx] = 1;
                if write != i {
                    buf.swap(write, i);
                }
                write += 1;
            }
            i += 1;
        }
        write
    }
}

fn interleave_halves(buf: &mut [u8], len: usize) {
    if len < 2 {
        return;
    }
    let half: usize = len / 2;
    let odd: usize = len % 2;

    if half <= 256 {
        // Match original: temp holds first half; second-half bytes are read from buf[half+i]
        // which are not overwritten during the loop.
        let mut temp = [0u8; 512];
        temp[..half].copy_from_slice(&buf[..half]);

        for i in 0..half {
            let b = buf[half + i];
            buf[i * 2 + 1] = b;
            buf[i * 2] = temp[i];
        }
        if odd != 0 {
            buf[len - 1] = buf[half];
        }
    } else {
        let mut i: usize = 0;
        while i < half {
            let src: usize = half + i;
            let dst: usize = i * 2 + 1;
            if dst < src {
                let val: u8 = buf[src];
                buf.copy_within(dst..src, dst + 1);
                buf[dst] = val;
            }
            i += 1;
        }
    }
}

fn reverse_segments(buf: &mut [u8], len: usize, seg_size: usize) {
    if seg_size <= 1 || len < seg_size {
        return;
    }
    let num_segments: usize = len / seg_size;
    let remainder: usize = len % seg_size;

    for seg in 0..num_segments {
        let base = seg * seg_size;
        buf[base..base + seg_size].reverse();
    }

    if remainder > 1 {
        let base = num_segments * seg_size;
        buf[base..base + remainder].reverse();
    }
}

fn parse_i32(tok: &str) -> Option<i32> {
    tok.parse::<i32>().ok()
}
fn parse_u32(tok: &str) -> Option<u32> {
    tok.parse::<u32>().ok()
}
fn parse_isize(tok: &str) -> Option<isize> {
    tok.parse::<isize>().ok()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();

    let flags: u32 = match it.next().and_then(parse_u32) {
        Some(v) => v,
        None => {
            eprint!("Error reading flags\n");
            std::process::exit(1);
        }
    };
    let param1: i32 = match it.next().and_then(parse_i32) {
        Some(v) => v,
        None => {
            eprint!("Error reading param1\n");
            std::process::exit(1);
        }
    };
    let param2: i32 = match it.next().and_then(parse_i32) {
        Some(v) => v,
        None => {
            eprint!("Error reading param2\n");
            std::process::exit(1);
        }
    };

    // Important: original accepts negative length token, then later fails while reading bytes.
    // So parse as signed first, then cast to usize (wrapping like C).
    let length_signed: isize = match it.next().and_then(parse_isize) {
        Some(v) => v,
        None => {
            eprint!("Error reading length\n");
            std::process::exit(1);
        }
    };
    let length: usize = length_signed as usize;

    // Allocate buffer of that length (if huge, this may OOM; matches C-like behavior).
    let mut buffer = vec![0u8; length];

    for i in 0..length {
        let tok = match it.next() {
            Some(v) => v,
            None => {
                eprint!("Error reading byte {}\n", i);
                std::process::exit(1);
            }
        };
        let val_i32 = match parse_i32(tok) {
            Some(v) => v,
            None => {
                eprint!("Error reading byte {}\n", i);
                std::process::exit(1);
            }
        };
        buffer[i] = val_i32 as u8; // wrap modulo 256
    }

    let new_len = process_buffer(&mut buffer, length, flags, param1, param2);

    if new_len == 0 {
        print!("0\n");
        return;
    }

    print!("{}", new_len);
    for i in 0..new_len {
        print!(" {}", buffer[i] as u32);
    }
    print!("\n");
}
```

**Entity:** process_buffer (and its helper pipeline over (buffer, length, flags, param1, param2))

**States:** InputPrefixValid, InputPrefixInvalid, Stage1_RotatedOrSkipped, Stage2_CompactedOrSkipped, Stage3_DedupedOrSkipped, Stage4_InterleavedOrSkipped, Stage5_SegReversedOrSkipped, OutputPrefixValid

**Transitions:**
- InputPrefixValid -> Stage1_RotatedOrSkipped via rotate_buffer() when flags&0x1!=0
- Stage1_RotatedOrSkipped -> Stage2_CompactedOrSkipped via compact_runs() when flags&0x2!=0 (updates new_len)
- Stage2_CompactedOrSkipped -> Stage3_DedupedOrSkipped via remove_duplicates() when flags&0x4!=0 (updates new_len)
- Stage3_DedupedOrSkipped -> Stage4_InterleavedOrSkipped via interleave_halves() when flags&0x8!=0 && new_len>=2
- Stage4_InterleavedOrSkipped -> Stage5_SegReversedOrSkipped via reverse_segments() when flags&0x10!=0 && new_len>=4 && seg_size<=new_len
- Stage5_SegReversedOrSkipped -> OutputPrefixValid via returning new_len

**Evidence:** process_buffer signature: `fn process_buffer(buffer: &mut [u8], length: usize, flags: u32, ...) -> usize` keeps `buffer` and `length` separate (latent invariant: `length <= buffer.len()` and operations only touch `[0..length)`/`[0..new_len)`).; process_buffer: `let mut new_len: usize = length;` then `new_len = compact_runs(...)` / `new_len = remove_duplicates(...)` — later stages must use updated `new_len`.; process_buffer: guards encode stage preconditions: `if flags & 0x8 != 0 && new_len >= 2 { interleave_halves(...) }` and `if flags & 0x10 != 0 && new_len >= 4 { ... if seg_size <= new_len { reverse_segments(...) } }`.; rotate_buffer: normalizes `offset` derived from `param1 % length as i32` and early-returns on `len<=1` / `offset==0` — indicates required domain constraints for meaningful rotation.; compact_runs/remove_duplicates/interleave_halves/reverse_segments all take `(buf, len)` and index into `buf[..len]` (e.g., `buf[..len].rotate_right(off)`, `while read < len { ... buf[read] ... }`, `buf[base..base+seg_size]`) assuming `len` is a valid prefix length.

**Implementation:** Model the processing as a typed pipeline over a `BufPrefix<'a>` newtype that carries `&'a mut [u8]` plus an invariant `active_len <= buf.len()`. Expose constructors like `BufPrefix::new(buf: &mut [u8]) -> BufPrefix` (active_len = buf.len()) and stage methods `rotate(self, offset: Offset) -> Self`, `compact(self, threshold: Threshold) -> BufPrefix` (returns updated active_len), etc. Encode `flags` as an enum/bitflags builder that produces a concrete pipeline type (or a builder that conditionally applies stages) so illegal/meaningless combinations can be rejected earlier and helpers can drop the separate `len` parameter (use slice splitting: `let (active, _) = buf.split_at_mut(active_len)`).

---

### 2. Input format protocol (token order + signed-length wrapping semantics)

**Location**: `/data/test_case/main.rs:1-282`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: The program assumes a strict token protocol on stdin: flags, param1, param2, length (parsed as signed), then exactly `length` byte tokens. It also intentionally permits negative `length` tokens and then casts to `usize` (wrapping like C), which can lead to huge allocations and later failures; this behavior is an implicit semantic requirement captured only by comments and by the specific parse/cast sequence. The type system does not express the input state machine or the 'signed length then wrapping cast' rule; correctness is maintained by runtime branching to `std::process::exit(1)` on missing/invalid tokens and by the comment documenting why `isize` is used first.

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

use std::io::{self, Read};

fn process_buffer(buffer: &mut [u8], length: usize, flags: u32, param1: i32, param2: i32) -> usize {
    let mut new_len: usize = length;
    if buffer.is_empty() || length == 0 {
        return 0;
    }

    if flags & 0x1 != 0 {
        // Must match original: offset computed with i32 % i32 (can be negative),
        // then rotate_buffer normalizes negative via wrapping_add(len as u64).
        let offset: i32 = param1 % length as i32;
        if offset != 0 {
            rotate_buffer(buffer, length, offset);
        }
    }
    if flags & 0x2 != 0 {
        let threshold: u8 = (if param1 > 0 && param1 <= 255 { param1 } else { 3 }) as u8;
        new_len = compact_runs(buffer, new_len, threshold);
    }
    if flags & 0x4 != 0 {
        let preserve: i32 = (param2 != 0) as i32;
        new_len = remove_duplicates(buffer, new_len, preserve);
    }
    if flags & 0x8 != 0 && new_len >= 2 {
        interleave_halves(buffer, new_len);
    }
    if flags & 0x10 != 0 && new_len >= 4 {
        let seg_size: usize = if param1 > 0 { param1 as usize } else { 4 };
        if seg_size <= new_len {
            reverse_segments(buffer, new_len, seg_size);
        }
    }
    new_len
}

// Matches the original C2Rust rotate_buffer behavior:
// It performs a RIGHT rotation by `offset` (after normalization).
fn rotate_buffer(buf: &mut [u8], len: usize, mut offset: i32) {
    if len <= 1 {
        return;
    }
    offset %= len as i32;
    if offset < 0 {
        // Match original: offset = (offset as u64).wrapping_add(len as u64) as i32;
        offset = ((offset as u32 as u64).wrapping_add(len as u64)) as i32;
    }
    if offset == 0 {
        return;
    }
    let off = offset as usize;
    buf[..len].rotate_right(off);
}

fn compact_runs(buf: &mut [u8], mut len: usize, threshold: u8) -> usize {
    let mut read: usize = 0;
    let mut write: usize = 0;

    while read < len {
        let current: u8 = buf[read];
        let mut run_len: usize = 1;
        while read + run_len < len && buf[read + run_len] == current {
            run_len += 1;
        }

        if run_len >= threshold as usize {
            if run_len > 255 {
                run_len = 255;
            }
            buf[write] = current;
            write += 1;
            buf[write] = run_len as u8;
            write += 1;

            if read + run_len < len {
                let remaining = len - (read + run_len);
                buf.copy_within(read + run_len..read + run_len + remaining, write);
            }
            len = write + (len - (read + run_len));
            read = write;
        } else {
            if write != read {
                buf.copy_within(read..read + run_len, write);
            }
            write += run_len;
            read += run_len;
        }
    }

    len
}

fn remove_duplicates(buf: &mut [u8], len: usize, preserve_order: i32) -> usize {
    if len <= 1 {
        return len;
    }

    if preserve_order != 0 {
        let mut write: usize = 1;
        let mut i: usize = 1;
        while i < len {
            let mut j: usize = 0;
            while j < write {
                if buf[i] == buf[j] {
                    break;
                }
                j += 1;
            }
            if j == write {
                if write != i {
                    buf[write] = buf[i];
                }
                write += 1;
            }
            i += 1;
        }
        write
    } else {
        let mut seen = [0u8; 256];
        let mut write: usize = 0;
        let mut i: usize = 0;
        while i < len {
            let idx = buf[i] as usize;
            if seen[idx] == 0 {
                seen[idx] = 1;
                if write != i {
                    buf.swap(write, i);
                }
                write += 1;
            }
            i += 1;
        }
        write
    }
}

fn interleave_halves(buf: &mut [u8], len: usize) {
    if len < 2 {
        return;
    }
    let half: usize = len / 2;
    let odd: usize = len % 2;

    if half <= 256 {
        // Match original: temp holds first half; second-half bytes are read from buf[half+i]
        // which are not overwritten during the loop.
        let mut temp = [0u8; 512];
        temp[..half].copy_from_slice(&buf[..half]);

        for i in 0..half {
            let b = buf[half + i];
            buf[i * 2 + 1] = b;
            buf[i * 2] = temp[i];
        }
        if odd != 0 {
            buf[len - 1] = buf[half];
        }
    } else {
        let mut i: usize = 0;
        while i < half {
            let src: usize = half + i;
            let dst: usize = i * 2 + 1;
            if dst < src {
                let val: u8 = buf[src];
                buf.copy_within(dst..src, dst + 1);
                buf[dst] = val;
            }
            i += 1;
        }
    }
}

fn reverse_segments(buf: &mut [u8], len: usize, seg_size: usize) {
    if seg_size <= 1 || len < seg_size {
        return;
    }
    let num_segments: usize = len / seg_size;
    let remainder: usize = len % seg_size;

    for seg in 0..num_segments {
        let base = seg * seg_size;
        buf[base..base + seg_size].reverse();
    }

    if remainder > 1 {
        let base = num_segments * seg_size;
        buf[base..base + remainder].reverse();
    }
}

fn parse_i32(tok: &str) -> Option<i32> {
    tok.parse::<i32>().ok()
}
fn parse_u32(tok: &str) -> Option<u32> {
    tok.parse::<u32>().ok()
}
fn parse_isize(tok: &str) -> Option<isize> {
    tok.parse::<isize>().ok()
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();

    let flags: u32 = match it.next().and_then(parse_u32) {
        Some(v) => v,
        None => {
            eprint!("Error reading flags\n");
            std::process::exit(1);
        }
    };
    let param1: i32 = match it.next().and_then(parse_i32) {
        Some(v) => v,
        None => {
            eprint!("Error reading param1\n");
            std::process::exit(1);
        }
    };
    let param2: i32 = match it.next().and_then(parse_i32) {
        Some(v) => v,
        None => {
            eprint!("Error reading param2\n");
            std::process::exit(1);
        }
    };

    // Important: original accepts negative length token, then later fails while reading bytes.
    // So parse as signed first, then cast to usize (wrapping like C).
    let length_signed: isize = match it.next().and_then(parse_isize) {
        Some(v) => v,
        None => {
            eprint!("Error reading length\n");
            std::process::exit(1);
        }
    };
    let length: usize = length_signed as usize;

    // Allocate buffer of that length (if huge, this may OOM; matches C-like behavior).
    let mut buffer = vec![0u8; length];

    for i in 0..length {
        let tok = match it.next() {
            Some(v) => v,
            None => {
                eprint!("Error reading byte {}\n", i);
                std::process::exit(1);
            }
        };
        let val_i32 = match parse_i32(tok) {
            Some(v) => v,
            None => {
                eprint!("Error reading byte {}\n", i);
                std::process::exit(1);
            }
        };
        buffer[i] = val_i32 as u8; // wrap modulo 256
    }

    let new_len = process_buffer(&mut buffer, length, flags, param1, param2);

    if new_len == 0 {
        print!("0\n");
        return;
    }

    print!("{}", new_len);
    for i in 0..new_len {
        print!(" {}", buffer[i] as u32);
    }
    print!("\n");
}
```

**Entity:** main input decoding (flags/params/length/buffer bytes)

**States:** ExpectFlags, ExpectParam1, ExpectParam2, ExpectLengthSigned, ExpectBytes, ReadyToProcess, ErrorExit

**Transitions:**
- ExpectFlags -> ExpectParam1 via `it.next().and_then(parse_u32)`
- ExpectParam1 -> ExpectParam2 via `it.next().and_then(parse_i32)`
- ExpectParam2 -> ExpectLengthSigned via `it.next().and_then(parse_i32)`
- ExpectLengthSigned -> ExpectBytes via `it.next().and_then(parse_isize)` then `as usize` cast
- ExpectBytes -> ReadyToProcess after reading exactly `length` byte tokens in `for i in 0..length`
- Any state -> ErrorExit via `eprint!("Error reading ...")` + `std::process::exit(1)`

**Evidence:** main: sequential parsing with exits: `Error reading flags/param1/param2/length/byte {}` followed by `std::process::exit(1)` encodes the protocol states.; comment in main: `Important: original accepts negative length token, then later fails ... parse as signed first, then cast to usize (wrapping like C).`; main: `let length_signed: isize = ...; let length: usize = length_signed as usize;` encodes the signed-then-wrapping invariant.; main: `let mut buffer = vec![0u8; length];` relies on that length semantics (possible OOM) and couples buffer.len() to `length` for later `process_buffer(&mut buffer, length, ...)`.

**Implementation:** Create a small parser/stateful builder type like `struct InputBuilder { it: SplitWhitespace<'a> }` with methods `read_flags() -> Result<Flags,..>`, `read_param1() -> Result<Param1,..>`, `read_length_signed() -> Result<LengthToken,..>`, `alloc_buffer(len: LengthToken) -> Result<BufferOwned,..>`, `read_bytes(buf: &mut BufferOwned) -> Result<(),..>`. Use newtypes `LengthToken(isize)` and `WrappedLen(usize)` to make the signed-to-wrapped conversion explicit (`impl From<LengthToken> for WrappedLen`). This enforces ordering at the API level and makes the special length semantics unmissable at compile time.

---

