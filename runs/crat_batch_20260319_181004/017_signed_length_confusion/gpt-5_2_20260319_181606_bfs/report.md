# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Precondition Invariants

### 1. Parsed-length validity protocol (Unparsed/Invalid -> ValidLength)

**Location**: `/data/test_case/main.rs:1-124`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `data` is initialized to -1, then conditionally set from input via `atoi_c_like`. Later it is used as a length (`n = data as usize`) for slicing/copying into `dest`. This is only safe if `data` is within 0..=99; negative values would underflow on cast and/or make slicing invalid. The code enforces this with runtime checks and even process termination, but the type system does not distinguish a validated length from an unvalidated `i32`.

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

fn printLine(line: &[u8]) {
    if line.is_empty() {
        return;
    }
    let end = line.iter().position(|&b| b == 0).unwrap_or(line.len());
    println!("{}", String::from_utf8_lossy(&line[..end]));
}

/// C-like atoi:
/// - skips leading whitespace
/// - parses optional sign
/// - consumes consecutive digits
/// - returns 0 if no digits
/// - wraps on overflow like C (two's complement)
fn atoi_c_like(bytes: &[u8]) -> i32 {
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c => i += 1,
            _ => break,
        }
    }

    let mut sign: i32 = 1;
    if i < bytes.len() {
        if bytes[i] == b'-' {
            sign = -1;
            i += 1;
        } else if bytes[i] == b'+' {
            i += 1;
        }
    }

    let mut acc: i32 = 0;
    let mut any = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_digit() {
            any = true;
            let d = (b - b'0') as i32;
            acc = acc.wrapping_mul(10).wrapping_add(d);
            i += 1;
        } else {
            break;
        }
    }

    if !any {
        0
    } else if sign == -1 {
        acc.wrapping_neg()
    } else {
        acc
    }
}

fn main() {
    let mut data: i32 = -1;

    // Mimic fgets(inputBuffer, 14, stdin):
    // read up to 13 bytes (or until newline), then NUL-terminate.
    let mut all_input = Vec::<u8>::new();
    let read_ok = io::stdin().read_to_end(&mut all_input).is_ok();

    let mut input_buffer = [0u8; 14];
    let mut got_any = false;

    if read_ok && !all_input.is_empty() {
        let mut n = 0usize;
        while n < all_input.len() && n < 13 {
            input_buffer[n] = all_input[n];
            n += 1;
            if all_input[n - 1] == b'\n' {
                break;
            }
        }
        // NUL terminate (already zeroed, but keep explicit)
        if n < input_buffer.len() {
            input_buffer[n] = 0;
        } else {
            input_buffer[13] = 0;
        }
        got_any = n > 0;
    }

    if got_any {
        data = atoi_c_like(&input_buffer);
    } else {
        printLine(b"fgets() failed.\0");
    }

    let mut source = [0u8; 100];
    let mut dest = [0u8; 100];

    source[..99].fill(b'A');
    source[99] = 0;

    // Original C checks only `data < 100` (no non-negative check).
    // If data is negative, the C code has undefined behavior.
    // Here we terminate with a non-zero exit code to avoid Rust UB.
    if data < 100 {
        if data < 0 {
            std::process::exit(1);
        }
        let n = data as usize;
        dest[..n].copy_from_slice(&source[..n]);
        dest[n] = 0;
    }

    printLine(&dest);
}
```

**Entity:** data: i32 (parsed length used for copy)

**States:** UnparsedOrInvalid, ValidLength(0..=99), TooLarge(>=100)

**Transitions:**
- UnparsedOrInvalid -> ValidLength(0..=99) via `data = atoi_c_like(&input_buffer)` plus runtime checks `if data < 100 { if data < 0 { exit(1) } ... }`
- UnparsedOrInvalid -> TooLarge(>=100) via `data = atoi_c_like(&input_buffer)` (then copy is skipped by `if data < 100` guard)

**Evidence:** `let mut data: i32 = -1;` sentinel default indicates an invalid/unparsed state; `if got_any { data = atoi_c_like(&input_buffer); }` shows `data` is only sometimes initialized from input; comment: `Original C checks only data < 100 (no non-negative check). If data is negative ... undefined behavior. Here we terminate ...` documents the latent precondition `0 <= data`; `if data < 100 { if data < 0 { std::process::exit(1); } let n = data as usize; dest[..n].copy_from_slice(&source[..n]); dest[n] = 0; }` runtime gating + cast to usize reveals the required invariant

**Implementation:** Introduce a validated length type, e.g. `struct CopyLen(u8);` or `struct CopyLen(usize);` with `TryFrom<i32>`/`TryFrom<usize>` ensuring `<= 99`. Parse into `Result<CopyLen, _>` and only perform `copy_from_slice` with `CopyLen`, eliminating the negative/too-large states at the call site.

---

## Protocol Invariants

### 2. C-string buffer protocol (NUL-terminated within capacity / may be empty)

**Location**: `/data/test_case/main.rs:1-124`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `input_buffer` is intended to mimic `fgets`: it should contain up to 13 bytes of input and be NUL-terminated so it can be treated like a C string / C-style numeric input. The code relies on a `got_any` flag and manual NUL termination to establish this invariant before passing the buffer to `atoi_c_like`. This protocol (non-empty + NUL termination) is maintained by convention and runtime logic, not by types.

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

fn printLine(line: &[u8]) {
    if line.is_empty() {
        return;
    }
    let end = line.iter().position(|&b| b == 0).unwrap_or(line.len());
    println!("{}", String::from_utf8_lossy(&line[..end]));
}

/// C-like atoi:
/// - skips leading whitespace
/// - parses optional sign
/// - consumes consecutive digits
/// - returns 0 if no digits
/// - wraps on overflow like C (two's complement)
fn atoi_c_like(bytes: &[u8]) -> i32 {
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c => i += 1,
            _ => break,
        }
    }

    let mut sign: i32 = 1;
    if i < bytes.len() {
        if bytes[i] == b'-' {
            sign = -1;
            i += 1;
        } else if bytes[i] == b'+' {
            i += 1;
        }
    }

    let mut acc: i32 = 0;
    let mut any = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_digit() {
            any = true;
            let d = (b - b'0') as i32;
            acc = acc.wrapping_mul(10).wrapping_add(d);
            i += 1;
        } else {
            break;
        }
    }

    if !any {
        0
    } else if sign == -1 {
        acc.wrapping_neg()
    } else {
        acc
    }
}

fn main() {
    let mut data: i32 = -1;

    // Mimic fgets(inputBuffer, 14, stdin):
    // read up to 13 bytes (or until newline), then NUL-terminate.
    let mut all_input = Vec::<u8>::new();
    let read_ok = io::stdin().read_to_end(&mut all_input).is_ok();

    let mut input_buffer = [0u8; 14];
    let mut got_any = false;

    if read_ok && !all_input.is_empty() {
        let mut n = 0usize;
        while n < all_input.len() && n < 13 {
            input_buffer[n] = all_input[n];
            n += 1;
            if all_input[n - 1] == b'\n' {
                break;
            }
        }
        // NUL terminate (already zeroed, but keep explicit)
        if n < input_buffer.len() {
            input_buffer[n] = 0;
        } else {
            input_buffer[13] = 0;
        }
        got_any = n > 0;
    }

    if got_any {
        data = atoi_c_like(&input_buffer);
    } else {
        printLine(b"fgets() failed.\0");
    }

    let mut source = [0u8; 100];
    let mut dest = [0u8; 100];

    source[..99].fill(b'A');
    source[99] = 0;

    // Original C checks only `data < 100` (no non-negative check).
    // If data is negative, the C code has undefined behavior.
    // Here we terminate with a non-zero exit code to avoid Rust UB.
    if data < 100 {
        if data < 0 {
            std::process::exit(1);
        }
        let n = data as usize;
        dest[..n].copy_from_slice(&source[..n]);
        dest[n] = 0;
    }

    printLine(&dest);
}
```

**Entity:** input_buffer: [u8; 14] (C-string-like buffer)

**States:** Empty, NonEmptyNulTerminated

**Transitions:**
- Empty -> NonEmptyNulTerminated via successful read/copy loop plus explicit NUL write, and `got_any = n > 0`
- NonEmptyNulTerminated -> Empty is possible conceptually when `read_ok` is false or `all_input.is_empty()` (then `got_any` remains false)

**Evidence:** comment: `Mimic fgets(inputBuffer, 14, stdin): read up to 13 bytes ... then NUL-terminate.` documents the required buffer shape; `let mut input_buffer = [0u8; 14];` fixed-capacity buffer intended for C-style use; `while n < all_input.len() && n < 13 { input_buffer[n] = all_input[n]; ... if all_input[n - 1] == b'\n' { break; } }` enforces the 13-byte max and newline stopping behavior; `input_buffer[n] = 0;` / `input_buffer[13] = 0;` explicit NUL termination step; `let mut got_any = false;` and `got_any = n > 0;` is a runtime state flag controlling whether parsing happens; `if got_any { data = atoi_c_like(&input_buffer); } else { printLine(b"fgets() failed.\0"); }` shows method availability depends on the flag/protocol

**Implementation:** Wrap the buffer in a type that guarantees NUL termination and tracks length, e.g. `struct NulTerminatedBuf<const N: usize> { buf: [u8; N], len: usize }` with a constructor `fn from_stdin_fgets_style(...) -> Option<Self>` that returns `None` for empty. Expose `as_bytes_with_nul()`/`as_bytes()` to consumers; `atoi_c_like` could accept this newtype instead of `&[u8]`.

---

