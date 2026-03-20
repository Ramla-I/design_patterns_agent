# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## Protocol Invariants

### 1. C-string style buffer protocol (Filled -> NUL-terminated -> NUL-delimited write)

**Location**: `/data/test_case/main.rs:1-54`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The code relies on a C-string-like invariant for `buf`: after reading, it must be NUL-terminated at index `i` (and `i` must be within bounds), and writing must only emit bytes up to (but not including) the first NUL. This is enforced by local control flow and indexing, not by the type system. If the termination step or the out_len computation were changed/misused elsewhere, `write_all` could emit unintended bytes (including leftover bytes from prior iterations) or violate the intended 'fputs'-like semantics. The type system only sees a `[u8; 128]` slice; it does not capture the notion 'this buffer is NUL-terminated at position i and valid up to that terminator'.

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

use std::io::{self, Read, Write};

fn main() {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut buf = [0u8; 128];

    loop {
        // Read like fgets into a fixed buffer:
        // - reads at most 127 bytes
        // - stops after newline (included)
        // - DOES include NUL bytes if present in input (binary-safe)
        // - always NUL-terminates the buffer
        let mut i = 0usize;
        let mut read_any = false;

        while i < buf.len() - 1 {
            let mut byte = [0u8; 1];
            match stdin.read_exact(&mut byte) {
                Ok(()) => {
                    read_any = true;
                    buf[i] = byte[0];
                    i += 1;
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // EOF
                Err(_) => return,
            }
        }

        if !read_any {
            break; // EOF with no data read
        }

        buf[i] = 0; // NUL-terminate like fgets

        // Write like fputs: output bytes up to the first NUL in the buffer.
        let out_len = buf[..=i].iter().position(|&b| b == 0).unwrap_or(i);
        let _ = stdout.write_all(&buf[..out_len]);
    }
}
```

**Entity:** buf: [u8; 128] (as used in main loop)

**States:** Filling, Terminated, Emitted

**Transitions:**
- Filling -> Terminated via `buf[i] = 0`
- Terminated -> Emitted via `stdout.write_all(&buf[..out_len])`

**Evidence:** comment: "always NUL-terminates the buffer" and "Write like fputs: output bytes up to the first NUL in the buffer."; `while i < buf.len() - 1` ensures space for terminator before `buf[i] = 0`; `buf[i] = 0; // NUL-terminate like fgets`; `let out_len = buf[..=i].iter().position(|&b| b == 0).unwrap_or(i);` computes length based on first NUL; `stdout.write_all(&buf[..out_len])` assumes `out_len` corresponds to the intended valid prefix

**Implementation:** Introduce a small newtype representing a NUL-terminated, NUL-delimited buffer, e.g. `struct NulTerminatedLine<'a> { buf: &'a [u8] }` created only by a constructor that performs the read loop and appends the terminator. Expose `as_bytes_to_nul(&self) -> &[u8]` for writing. This makes it impossible to call the 'emit' step without first producing a terminated buffer.

---

