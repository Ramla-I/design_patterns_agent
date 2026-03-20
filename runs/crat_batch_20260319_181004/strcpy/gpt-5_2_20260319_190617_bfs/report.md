# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Temporal Ordering Invariants

### 3. Structured input protocol (token stream must match declared lengths and bounds)

**Location**: `/data/test_case/main.rs:1-478`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: `main` implements an implicit multi-step parser over a whitespace token stream: it must read `operation`, then `flags`, then `input_len`, then exactly `input_len` i8 tokens, then `ref_len`, then exactly `ref_len` i8 tokens. Correctness depends on the temporal ordering and on `input_len/ref_len <= MAX_LEN`. This is enforced via runtime indexing (`idx`) and `exit(1)` on failures. The type system does not model the parser state, so it is easy to accidentally reuse `idx` incorrectly or call parsing steps out of order in future refactors; the only protection is ad-hoc runtime checks and process termination.

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

const MAX_LEN: usize = 1024;

fn c_strlen_i8(buf: &[i8]) -> usize {
    buf.iter().position(|&b| b == 0).unwrap_or(buf.len())
}

fn c_strcmp_i8(a: &[i8], b: &[i8]) -> i32 {
    let mut i = 0usize;
    loop {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
        i += 1;
    }
}

fn c_strncmp_i8(a: &[i8], b: &[i8], n: usize) -> i32 {
    for i in 0..n {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
    }
    0
}

fn c_strncat_i8(dest: &mut [i8], src: &[i8], n: usize) {
    let dlen = c_strlen_i8(dest);
    if dlen >= dest.len() {
        return;
    }
    let mut copied = 0usize;
    while copied < n && dlen + copied + 1 < dest.len() {
        let c = *src.get(copied).unwrap_or(&0);
        if c == 0 {
            break;
        }
        dest[dlen + copied] = c;
        copied += 1;
    }
    if dlen + copied < dest.len() {
        dest[dlen + copied] = 0;
    } else if !dest.is_empty() {
        dest[dest.len() - 1] = 0;
    }
}

fn validate_token(token: &[i8], expected: &[i8]) -> i32 {
    if c_strcmp_i8(token, expected) == 0 {
        return 1;
    }
    const VALID: &[i8] = &[b'V' as i8, b'A' as i8, b'L' as i8, b'I' as i8, b'D' as i8, 0];
    const OK: &[i8] = &[b'O' as i8, b'K' as i8, 0];
    if c_strcmp_i8(token, VALID) == 0 || c_strcmp_i8(token, OK) == 0 {
        return 1;
    }
    0
}

fn parse_command(buffer: &[i8], buf_size: usize, cmd_list: &[&[i8]], list_size: i32) -> i32 {
    for i in 0..(list_size as usize) {
        let cmd = cmd_list[i];
        let cmd_len = c_strlen_i8(cmd);

        if buf_size >= cmd_len
            && c_strncmp_i8(buffer, cmd, cmd_len) == 0
            && (*buffer.get(cmd_len).unwrap_or(&0) == 0
                || *buffer.get(cmd_len).unwrap_or(&0) == b' ' as i8)
        {
            return i as i32;
        }
        if c_strcmp_i8(buffer, cmd) == 0 {
            return i as i32;
        }
    }

    const ADMIN: &[i8] = &[b'A' as i8, b'D' as i8, b'M' as i8, b'I' as i8, b'N' as i8, 0];
    if c_strcmp_i8(buffer, ADMIN) == 0 {
        return 99;
    }
    -1
}

fn compare_prefix(s: &[i8], prefix: &[i8], exact_match: i32) -> i32 {
    let prefix_len = c_strlen_i8(prefix);

    if exact_match != 0 {
        if c_strcmp_i8(s, prefix) == 0 {
            return 1;
        }

        let variations: [[i8; 32]; 5] = [
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'1' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'2' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'o' as i8;
                a[2] = b'l' as i8;
                a[3] = b'd' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'n' as i8;
                a[2] = b'e' as i8;
                a[3] = b'w' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b't' as i8;
                a[2] = b'm' as i8;
                a[3] = b'p' as i8;
                a
            },
        ];

        for (i, var) in variations.iter().enumerate() {
            let mut expected = [0i8; 64];

            // Mimic C: copy up to 63 bytes from prefix (may include NUL and beyond),
            // then force NUL at the end.
            for j in 0..63 {
                expected[j] = *prefix.get(j).unwrap_or(&0);
            }
            expected[63] = 0;

            let cur_len = c_strlen_i8(&expected);
            let remaining = 63usize.wrapping_sub(cur_len);
            c_strncat_i8(&mut expected, var, remaining);

            if c_strcmp_i8(s, &expected) == 0 {
                return 2 + i as i32;
            }
        }
        0
    } else {
        if c_strncmp_i8(s, prefix, prefix_len) == 0 {
            return 1;
        }
        0
    }
}

fn find_delimiter(data: &[i8], len: usize, delim: i8) -> i32 {
    if len == 0 {
        return -1;
    }
    let mut i = 0usize;
    while i < len {
        let c = *data.get(i).unwrap_or(&0);
        if c == delim {
            return i as i32;
        }
        if c == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    const NONE: &[i8] = &[b'N' as i8, b'O' as i8, b'N' as i8, b'E' as i8, 0];
    const EMPTY: &[i8] = &[b'E' as i8, b'M' as i8, b'P' as i8, b'T' as i8, b'Y' as i8, 0];

    if delim == b'|' as i8 && c_strcmp_i8(data, NONE) == 0 {
        return -2;
    }
    if delim == b':' as i8 && c_strcmp_i8(data, EMPTY) == 0 {
        return -3;
    }
    -1
}

fn match_pattern(text: &[i8], pattern: &[i8], case_sensitive: i32) -> i32 {
    if case_sensitive != 0 {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }

        // Build wildcard strings exactly like snprintf("*%s*", pattern) etc.
        let pat_len = c_strlen_i8(pattern);
        let pat = &pattern[..pat_len];

        fn build_wildcard(prefix: &[i8], pat: &[i8], suffix: &[i8]) -> [i8; 64] {
            let mut out = [0i8; 64];
            let mut idx = 0usize;

            for &c in prefix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in pat {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in suffix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            out[idx.min(out.len() - 1)] = 0;
            out
        }

        let w0 = build_wildcard(&[b'*' as i8], pat, &[b'*' as i8]);
        let w1 = build_wildcard(&[], pat, &[b'*' as i8]);
        let w2 = build_wildcard(&[b'*' as i8], pat, &[]);

        for (i, w) in [w0, w1, w2].iter().enumerate() {
            if c_strcmp_i8(text, w) == 0 {
                return 2 + i as i32;
            }
        }

        let text_len = c_strlen_i8(text);
        let pattern_len = c_strlen_i8(pattern);

        // Important: original C2Rust uses wrapping_sub; if pattern_len > text_len,
        // the loop still runs and may match at i=0 due to strncmp stopping at NUL.
        let mut i0 = 0usize;
        while i0 <= text_len.wrapping_sub(pattern_len) {
            if c_strncmp_i8(&text[i0..], pattern, pattern_len) == 0 {
                return 10usize.wrapping_add(i0) as i32;
            }
            i0 = i0.wrapping_add(1);
        }
    } else {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }
        let pattern_len = c_strlen_i8(pattern);
        let text_len = c_strlen_i8(text);

        if text_len != pattern_len && c_strncmp_i8(text, pattern, pattern_len) == 0 {
            return 5;
        }

        if text_len == pattern_len {
            let mut m = 1i32;
            let mut i = 0usize;
            while i < pattern_len {
                let mut c1 = text[i];
                let mut c2 = pattern[i];
                if (b'A' as i8..=b'Z' as i8).contains(&c1) {
                    c1 = (c1 as i32 + 32) as i8;
                }
                if (b'A' as i8..=b'Z' as i8).contains(&c2) {
                    c2 = (c2 as i32 + 32) as i8;
                }
                if c1 != c2 {
                    m = 0;
                    break;
                }
                i = i.wrapping_add(1);
            }
            if m != 0 {
                return 6;
            }
        }
    }
    0
}

fn process_strings(
    input: &[i8],
    input_len: usize,
    reference: &[i8],
    ref_len: usize,
    operation: i32,
    flags: u32,
) -> i32 {
    if input.is_empty() {
        return -1;
    }
    match operation {
        0 => {
            // C2Rust checks reference.is_empty() (null pointer). In our harness, ref_len==0
            // still provides a valid (possibly empty) slice, so do NOT reject.
            validate_token(input, reference)
        }
        1 => {
            const START: &[i8] =
                &[b'S' as i8, b'T' as i8, b'A' as i8, b'R' as i8, b'T' as i8, 0];
            const STOP: &[i8] = &[b'S' as i8, b'T' as i8, b'O' as i8, b'P' as i8, 0];
            const PAUSE: &[i8] =
                &[b'P' as i8, b'A' as i8, b'U' as i8, b'S' as i8, b'E' as i8, 0];
            const RESUME: &[i8] = &[
                b'R' as i8,
                b'E' as i8,
                b'S' as i8,
                b'U' as i8,
                b'M' as i8,
                b'E' as i8,
                0,
            ];
            const RESET: &[i8] =
                &[b'R' as i8, b'E' as i8, b'S' as i8, b'E' as i8, b'T' as i8, 0];
            let commands: [&[i8]; 5] = [START, STOP, PAUSE, RESUME, RESET];
            parse_command(input, input_len, &commands, 5)
        }
        2 => {
            // Same: do not reject empty reference slice; C would only reject null pointer.
            let exact = (flags & 0x1) as i32;
            compare_prefix(input, reference, exact)
        }
        3 => {
            let delim = if !reference.is_empty() && ref_len != 0 {
                reference[0]
            } else {
                b':' as i8
            };
            find_delimiter(input, input_len, delim)
        }
        4 => {
            // Same: do not reject empty reference slice.
            let case_sens = (flags & 0x2) as i32;
            match_pattern(input, reference, case_sens)
        }
        _ => -3,
    }
}

fn parse_i32(tok: &str) -> Option<i32> {
    tok.parse::<i32>().ok()
}
fn parse_u32(tok: &str) -> Option<u32> {
    tok.parse::<u32>().ok()
}
fn parse_usize(tok: &str) -> Option<usize> {
    tok.parse::<usize>().ok()
}
fn parse_i8(tok: &str) -> Option<i8> {
    let v = tok.parse::<i32>().ok()?;
    Some(v as i8)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut idx = 0usize;

    let operation = match tokens.get(idx).and_then(|t| parse_i32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading operation\n");
            std::process::exit(1);
        }
    };

    let flags = match tokens.get(idx).and_then(|t| parse_u32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading flags\n");
            std::process::exit(1);
        }
    };

    let input_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading input length\n");
            std::process::exit(1);
        }
    };
    if input_len > MAX_LEN {
        eprint!("Error: input length {} exceeds maximum 1024\n", input_len);
        std::process::exit(1);
    }

    let mut input_bytes = Vec::with_capacity(input_len);
    for i in 0..input_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                input_bytes.push(v);
            }
            None => {
                eprint!("Error reading input byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let ref_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading reference length\n");
            std::process::exit(1);
        }
    };
    if ref_len > MAX_LEN {
        eprint!("Error: reference length {} exceeds maximum 1024\n", ref_len);
        std::process::exit(1);
    }

    let mut reference_bytes = Vec::with_capacity(ref_len);
    for i in 0..ref_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                reference_bytes.push(v);
            }
            None => {
                eprint!("Error reading reference byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let result = process_strings(
        &input_bytes,
        input_len,
        &reference_bytes,
        ref_len,
        operation,
        flags,
    );
    print!("{result}\n");
}
```

**Entity:** main input decoding (operation/flags/input_len/input_bytes/ref_len/reference_bytes)

**States:** ReadingHeader(operation,flags,input_len,ref_len), ReadingInputBytes(expecting input_len items), ReadingReferenceBytes(expecting ref_len items), Complete, InvalidInput

**Transitions:**
- ReadingHeader -> ReadingInputBytes via successfully parsing `input_len`
- ReadingInputBytes -> ReadingReferenceBytes via consuming `input_len` bytes
- ReadingReferenceBytes -> Complete via consuming `ref_len` bytes
- Any -> InvalidInput via `eprint!("Error reading ...")` + `std::process::exit(1)`
- Any(length) -> InvalidInput via `if input_len > MAX_LEN { exit(1) }` and `if ref_len > MAX_LEN { exit(1) }`

**Evidence:** main: `let tokens: Vec<&str> = input.split_whitespace().collect(); let mut idx = 0usize;` manual cursor encodes parser state; main: sequential parses advancing `idx` for operation/flags/input_len/ref_len (`idx += 1` after each successful parse); main: bounded-length invariant enforced at runtime: `if input_len > MAX_LEN { ... exit(1) }` and `if ref_len > MAX_LEN { ... exit(1) }`; main: loops that assume the next N tokens exist and are parseable: `for i in 0..input_len { ... tokens.get(idx) ... idx += 1 ... }` and similarly for `ref_len`; error handling encodes the protocol: `eprint!("Error reading input byte {}\n", i); exit(1);` and analogous messages for each stage

**Implementation:** Create a small parsing type `struct TokenCursor<'a> { toks: Vec<&'a str>, idx: usize }` with typed methods `read_i32()`, `read_u32()`, `read_len_max<const MAX: usize>() -> BoundedUsize<MAX>`, and `read_i8_vec(len: BoundedUsize<MAX>)`. Return `Result<ParsedInput, ParseError>` instead of exiting. This models the staged protocol in APIs and centralizes the invariants (bounds + exact token counts).

---

## Precondition Invariants

### 2. NUL-terminated C-string invariant for &[i8] inputs (CStr-like validity)

**Location**: `/data/test_case/main.rs:1-478`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Many functions implicitly treat `&[i8]` as C strings: scanning for a NUL terminator (`c_strlen_i8`), comparing until NUL (`c_strcmp_i8`/`c_strncmp_i8`), and concatenating while reserving space for a trailing NUL (`c_strncat_i8`). This protocol is not enforced: callers can pass arbitrary byte slices that are not NUL-terminated. The code partially compensates by using `get(i).unwrap_or(&0)` (treating out-of-bounds as NUL), which changes semantics compared to real C strings and can lead to surprising matches/lengths (e.g., comparisons may succeed because missing bytes are considered 0). A type-level wrapper for 'valid C string in i8 encoding' would make the intended invariant explicit and prevent misuse.

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

const MAX_LEN: usize = 1024;

fn c_strlen_i8(buf: &[i8]) -> usize {
    buf.iter().position(|&b| b == 0).unwrap_or(buf.len())
}

fn c_strcmp_i8(a: &[i8], b: &[i8]) -> i32 {
    let mut i = 0usize;
    loop {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
        i += 1;
    }
}

fn c_strncmp_i8(a: &[i8], b: &[i8], n: usize) -> i32 {
    for i in 0..n {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
    }
    0
}

fn c_strncat_i8(dest: &mut [i8], src: &[i8], n: usize) {
    let dlen = c_strlen_i8(dest);
    if dlen >= dest.len() {
        return;
    }
    let mut copied = 0usize;
    while copied < n && dlen + copied + 1 < dest.len() {
        let c = *src.get(copied).unwrap_or(&0);
        if c == 0 {
            break;
        }
        dest[dlen + copied] = c;
        copied += 1;
    }
    if dlen + copied < dest.len() {
        dest[dlen + copied] = 0;
    } else if !dest.is_empty() {
        dest[dest.len() - 1] = 0;
    }
}

fn validate_token(token: &[i8], expected: &[i8]) -> i32 {
    if c_strcmp_i8(token, expected) == 0 {
        return 1;
    }
    const VALID: &[i8] = &[b'V' as i8, b'A' as i8, b'L' as i8, b'I' as i8, b'D' as i8, 0];
    const OK: &[i8] = &[b'O' as i8, b'K' as i8, 0];
    if c_strcmp_i8(token, VALID) == 0 || c_strcmp_i8(token, OK) == 0 {
        return 1;
    }
    0
}

fn parse_command(buffer: &[i8], buf_size: usize, cmd_list: &[&[i8]], list_size: i32) -> i32 {
    for i in 0..(list_size as usize) {
        let cmd = cmd_list[i];
        let cmd_len = c_strlen_i8(cmd);

        if buf_size >= cmd_len
            && c_strncmp_i8(buffer, cmd, cmd_len) == 0
            && (*buffer.get(cmd_len).unwrap_or(&0) == 0
                || *buffer.get(cmd_len).unwrap_or(&0) == b' ' as i8)
        {
            return i as i32;
        }
        if c_strcmp_i8(buffer, cmd) == 0 {
            return i as i32;
        }
    }

    const ADMIN: &[i8] = &[b'A' as i8, b'D' as i8, b'M' as i8, b'I' as i8, b'N' as i8, 0];
    if c_strcmp_i8(buffer, ADMIN) == 0 {
        return 99;
    }
    -1
}

fn compare_prefix(s: &[i8], prefix: &[i8], exact_match: i32) -> i32 {
    let prefix_len = c_strlen_i8(prefix);

    if exact_match != 0 {
        if c_strcmp_i8(s, prefix) == 0 {
            return 1;
        }

        let variations: [[i8; 32]; 5] = [
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'1' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'2' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'o' as i8;
                a[2] = b'l' as i8;
                a[3] = b'd' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'n' as i8;
                a[2] = b'e' as i8;
                a[3] = b'w' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b't' as i8;
                a[2] = b'm' as i8;
                a[3] = b'p' as i8;
                a
            },
        ];

        for (i, var) in variations.iter().enumerate() {
            let mut expected = [0i8; 64];

            // Mimic C: copy up to 63 bytes from prefix (may include NUL and beyond),
            // then force NUL at the end.
            for j in 0..63 {
                expected[j] = *prefix.get(j).unwrap_or(&0);
            }
            expected[63] = 0;

            let cur_len = c_strlen_i8(&expected);
            let remaining = 63usize.wrapping_sub(cur_len);
            c_strncat_i8(&mut expected, var, remaining);

            if c_strcmp_i8(s, &expected) == 0 {
                return 2 + i as i32;
            }
        }
        0
    } else {
        if c_strncmp_i8(s, prefix, prefix_len) == 0 {
            return 1;
        }
        0
    }
}

fn find_delimiter(data: &[i8], len: usize, delim: i8) -> i32 {
    if len == 0 {
        return -1;
    }
    let mut i = 0usize;
    while i < len {
        let c = *data.get(i).unwrap_or(&0);
        if c == delim {
            return i as i32;
        }
        if c == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    const NONE: &[i8] = &[b'N' as i8, b'O' as i8, b'N' as i8, b'E' as i8, 0];
    const EMPTY: &[i8] = &[b'E' as i8, b'M' as i8, b'P' as i8, b'T' as i8, b'Y' as i8, 0];

    if delim == b'|' as i8 && c_strcmp_i8(data, NONE) == 0 {
        return -2;
    }
    if delim == b':' as i8 && c_strcmp_i8(data, EMPTY) == 0 {
        return -3;
    }
    -1
}

fn match_pattern(text: &[i8], pattern: &[i8], case_sensitive: i32) -> i32 {
    if case_sensitive != 0 {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }

        // Build wildcard strings exactly like snprintf("*%s*", pattern) etc.
        let pat_len = c_strlen_i8(pattern);
        let pat = &pattern[..pat_len];

        fn build_wildcard(prefix: &[i8], pat: &[i8], suffix: &[i8]) -> [i8; 64] {
            let mut out = [0i8; 64];
            let mut idx = 0usize;

            for &c in prefix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in pat {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in suffix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            out[idx.min(out.len() - 1)] = 0;
            out
        }

        let w0 = build_wildcard(&[b'*' as i8], pat, &[b'*' as i8]);
        let w1 = build_wildcard(&[], pat, &[b'*' as i8]);
        let w2 = build_wildcard(&[b'*' as i8], pat, &[]);

        for (i, w) in [w0, w1, w2].iter().enumerate() {
            if c_strcmp_i8(text, w) == 0 {
                return 2 + i as i32;
            }
        }

        let text_len = c_strlen_i8(text);
        let pattern_len = c_strlen_i8(pattern);

        // Important: original C2Rust uses wrapping_sub; if pattern_len > text_len,
        // the loop still runs and may match at i=0 due to strncmp stopping at NUL.
        let mut i0 = 0usize;
        while i0 <= text_len.wrapping_sub(pattern_len) {
            if c_strncmp_i8(&text[i0..], pattern, pattern_len) == 0 {
                return 10usize.wrapping_add(i0) as i32;
            }
            i0 = i0.wrapping_add(1);
        }
    } else {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }
        let pattern_len = c_strlen_i8(pattern);
        let text_len = c_strlen_i8(text);

        if text_len != pattern_len && c_strncmp_i8(text, pattern, pattern_len) == 0 {
            return 5;
        }

        if text_len == pattern_len {
            let mut m = 1i32;
            let mut i = 0usize;
            while i < pattern_len {
                let mut c1 = text[i];
                let mut c2 = pattern[i];
                if (b'A' as i8..=b'Z' as i8).contains(&c1) {
                    c1 = (c1 as i32 + 32) as i8;
                }
                if (b'A' as i8..=b'Z' as i8).contains(&c2) {
                    c2 = (c2 as i32 + 32) as i8;
                }
                if c1 != c2 {
                    m = 0;
                    break;
                }
                i = i.wrapping_add(1);
            }
            if m != 0 {
                return 6;
            }
        }
    }
    0
}

fn process_strings(
    input: &[i8],
    input_len: usize,
    reference: &[i8],
    ref_len: usize,
    operation: i32,
    flags: u32,
) -> i32 {
    if input.is_empty() {
        return -1;
    }
    match operation {
        0 => {
            // C2Rust checks reference.is_empty() (null pointer). In our harness, ref_len==0
            // still provides a valid (possibly empty) slice, so do NOT reject.
            validate_token(input, reference)
        }
        1 => {
            const START: &[i8] =
                &[b'S' as i8, b'T' as i8, b'A' as i8, b'R' as i8, b'T' as i8, 0];
            const STOP: &[i8] = &[b'S' as i8, b'T' as i8, b'O' as i8, b'P' as i8, 0];
            const PAUSE: &[i8] =
                &[b'P' as i8, b'A' as i8, b'U' as i8, b'S' as i8, b'E' as i8, 0];
            const RESUME: &[i8] = &[
                b'R' as i8,
                b'E' as i8,
                b'S' as i8,
                b'U' as i8,
                b'M' as i8,
                b'E' as i8,
                0,
            ];
            const RESET: &[i8] =
                &[b'R' as i8, b'E' as i8, b'S' as i8, b'E' as i8, b'T' as i8, 0];
            let commands: [&[i8]; 5] = [START, STOP, PAUSE, RESUME, RESET];
            parse_command(input, input_len, &commands, 5)
        }
        2 => {
            // Same: do not reject empty reference slice; C would only reject null pointer.
            let exact = (flags & 0x1) as i32;
            compare_prefix(input, reference, exact)
        }
        3 => {
            let delim = if !reference.is_empty() && ref_len != 0 {
                reference[0]
            } else {
                b':' as i8
            };
            find_delimiter(input, input_len, delim)
        }
        4 => {
            // Same: do not reject empty reference slice.
            let case_sens = (flags & 0x2) as i32;
            match_pattern(input, reference, case_sens)
        }
        _ => -3,
    }
}

fn parse_i32(tok: &str) -> Option<i32> {
    tok.parse::<i32>().ok()
}
fn parse_u32(tok: &str) -> Option<u32> {
    tok.parse::<u32>().ok()
}
fn parse_usize(tok: &str) -> Option<usize> {
    tok.parse::<usize>().ok()
}
fn parse_i8(tok: &str) -> Option<i8> {
    let v = tok.parse::<i32>().ok()?;
    Some(v as i8)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut idx = 0usize;

    let operation = match tokens.get(idx).and_then(|t| parse_i32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading operation\n");
            std::process::exit(1);
        }
    };

    let flags = match tokens.get(idx).and_then(|t| parse_u32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading flags\n");
            std::process::exit(1);
        }
    };

    let input_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading input length\n");
            std::process::exit(1);
        }
    };
    if input_len > MAX_LEN {
        eprint!("Error: input length {} exceeds maximum 1024\n", input_len);
        std::process::exit(1);
    }

    let mut input_bytes = Vec::with_capacity(input_len);
    for i in 0..input_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                input_bytes.push(v);
            }
            None => {
                eprint!("Error reading input byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let ref_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading reference length\n");
            std::process::exit(1);
        }
    };
    if ref_len > MAX_LEN {
        eprint!("Error: reference length {} exceeds maximum 1024\n", ref_len);
        std::process::exit(1);
    }

    let mut reference_bytes = Vec::with_capacity(ref_len);
    for i in 0..ref_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                reference_bytes.push(v);
            }
            None => {
                eprint!("Error reading reference byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let result = process_strings(
        &input_bytes,
        input_len,
        &reference_bytes,
        ref_len,
        operation,
        flags,
    );
    print!("{result}\n");
}
```

**Entity:** C-style string slices used across c_strlen_i8/c_strcmp_i8/c_strncmp_i8/c_strncat_i8 and higher-level functions

**States:** NulTerminated(contains 0 terminator within provided slice), NonTerminated(no 0 terminator within slice; treated as implicitly 0 beyond end)

**Transitions:**
- NonTerminated -> NulTerminated via constructing a buffer with an explicit trailing 0 (e.g., after c_strncat_i8 ensures NUL termination)
- Any -> (interpreted-as-C-string) via passing &[i8] into c_strlen_i8/c_strcmp_i8/c_strncmp_i8

**Evidence:** fn c_strlen_i8(buf: &[i8]) -> usize: `position(|&b| b == 0)` assumes a NUL terminator is meaningful; fn c_strcmp_i8: `let ac = *a.get(i).unwrap_or(&0);` and `if ac == 0 { return 0; }` uses 0 as terminator and treats OOB as 0; fn c_strncmp_i8: same `unwrap_or(&0)` + `if ac == 0 { return 0; }` terminator semantics; fn c_strncat_i8: computes `dlen = c_strlen_i8(dest)` and then enforces `dest[dlen + copied] = 0` or `dest[dest.len()-1]=0`, encoding a 'must stay NUL-terminated' invariant; higher-level fns rely on terminators: `parse_command` uses `cmd_len = c_strlen_i8(cmd)` and checks `buffer[cmd_len] == 0 || buffer[cmd_len] == ' '`; `match_pattern` uses `pat_len = c_strlen_i8(pattern)` and slices `&pattern[..pat_len]`

**Implementation:** Introduce a wrapper like `struct CStrI8<'a>(&'a [i8]);` with `TryFrom<&'a [i8]>` validating 'contains at least one 0' (or ensuring last byte is 0), and provide safe methods `len_to_nul()`, `cmp()`, `prefix_cmp()`. Update APIs to accept `CStrI8` (or `Option<CStrI8>` where 'null pointer vs empty' matters) so non-terminated slices are rejected at construction time instead of being silently treated as terminated by `unwrap_or(&0)`.

---

## Protocol Invariants

### 1. process_strings operation/flags protocol (operation selects interpretation of reference and flags)

**Location**: `/data/test_case/main.rs:1-478`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The meaning and required shape of inputs depends on `operation`. `process_strings` is effectively a tagged protocol driven by a runtime `i32`: each operation interprets `reference`/`ref_len` and specific bits in `flags` differently (e.g., op2 uses flags bit 0 as `exact`, op4 uses bit 1 as `case_sensitive`, op3 treats `reference[0]` as a delimiter only if `reference` is non-empty and `ref_len != 0`). None of these couplings are enforced by the type system because `operation` is an untyped integer and `flags` is a generic bitfield; callers can pass meaningless flag bits for a given operation, or pass a reference slice whose length/semantics don't match the operation, with behavior only defined by runtime branching.

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

const MAX_LEN: usize = 1024;

fn c_strlen_i8(buf: &[i8]) -> usize {
    buf.iter().position(|&b| b == 0).unwrap_or(buf.len())
}

fn c_strcmp_i8(a: &[i8], b: &[i8]) -> i32 {
    let mut i = 0usize;
    loop {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
        i += 1;
    }
}

fn c_strncmp_i8(a: &[i8], b: &[i8], n: usize) -> i32 {
    for i in 0..n {
        let ac = *a.get(i).unwrap_or(&0);
        let bc = *b.get(i).unwrap_or(&0);
        if ac != bc {
            return ac as i32 - bc as i32;
        }
        if ac == 0 {
            return 0;
        }
    }
    0
}

fn c_strncat_i8(dest: &mut [i8], src: &[i8], n: usize) {
    let dlen = c_strlen_i8(dest);
    if dlen >= dest.len() {
        return;
    }
    let mut copied = 0usize;
    while copied < n && dlen + copied + 1 < dest.len() {
        let c = *src.get(copied).unwrap_or(&0);
        if c == 0 {
            break;
        }
        dest[dlen + copied] = c;
        copied += 1;
    }
    if dlen + copied < dest.len() {
        dest[dlen + copied] = 0;
    } else if !dest.is_empty() {
        dest[dest.len() - 1] = 0;
    }
}

fn validate_token(token: &[i8], expected: &[i8]) -> i32 {
    if c_strcmp_i8(token, expected) == 0 {
        return 1;
    }
    const VALID: &[i8] = &[b'V' as i8, b'A' as i8, b'L' as i8, b'I' as i8, b'D' as i8, 0];
    const OK: &[i8] = &[b'O' as i8, b'K' as i8, 0];
    if c_strcmp_i8(token, VALID) == 0 || c_strcmp_i8(token, OK) == 0 {
        return 1;
    }
    0
}

fn parse_command(buffer: &[i8], buf_size: usize, cmd_list: &[&[i8]], list_size: i32) -> i32 {
    for i in 0..(list_size as usize) {
        let cmd = cmd_list[i];
        let cmd_len = c_strlen_i8(cmd);

        if buf_size >= cmd_len
            && c_strncmp_i8(buffer, cmd, cmd_len) == 0
            && (*buffer.get(cmd_len).unwrap_or(&0) == 0
                || *buffer.get(cmd_len).unwrap_or(&0) == b' ' as i8)
        {
            return i as i32;
        }
        if c_strcmp_i8(buffer, cmd) == 0 {
            return i as i32;
        }
    }

    const ADMIN: &[i8] = &[b'A' as i8, b'D' as i8, b'M' as i8, b'I' as i8, b'N' as i8, 0];
    if c_strcmp_i8(buffer, ADMIN) == 0 {
        return 99;
    }
    -1
}

fn compare_prefix(s: &[i8], prefix: &[i8], exact_match: i32) -> i32 {
    let prefix_len = c_strlen_i8(prefix);

    if exact_match != 0 {
        if c_strcmp_i8(s, prefix) == 0 {
            return 1;
        }

        let variations: [[i8; 32]; 5] = [
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'1' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'v' as i8;
                a[2] = b'2' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'o' as i8;
                a[2] = b'l' as i8;
                a[3] = b'd' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b'n' as i8;
                a[2] = b'e' as i8;
                a[3] = b'w' as i8;
                a
            },
            {
                let mut a = [0i8; 32];
                a[0] = b'_' as i8;
                a[1] = b't' as i8;
                a[2] = b'm' as i8;
                a[3] = b'p' as i8;
                a
            },
        ];

        for (i, var) in variations.iter().enumerate() {
            let mut expected = [0i8; 64];

            // Mimic C: copy up to 63 bytes from prefix (may include NUL and beyond),
            // then force NUL at the end.
            for j in 0..63 {
                expected[j] = *prefix.get(j).unwrap_or(&0);
            }
            expected[63] = 0;

            let cur_len = c_strlen_i8(&expected);
            let remaining = 63usize.wrapping_sub(cur_len);
            c_strncat_i8(&mut expected, var, remaining);

            if c_strcmp_i8(s, &expected) == 0 {
                return 2 + i as i32;
            }
        }
        0
    } else {
        if c_strncmp_i8(s, prefix, prefix_len) == 0 {
            return 1;
        }
        0
    }
}

fn find_delimiter(data: &[i8], len: usize, delim: i8) -> i32 {
    if len == 0 {
        return -1;
    }
    let mut i = 0usize;
    while i < len {
        let c = *data.get(i).unwrap_or(&0);
        if c == delim {
            return i as i32;
        }
        if c == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }

    const NONE: &[i8] = &[b'N' as i8, b'O' as i8, b'N' as i8, b'E' as i8, 0];
    const EMPTY: &[i8] = &[b'E' as i8, b'M' as i8, b'P' as i8, b'T' as i8, b'Y' as i8, 0];

    if delim == b'|' as i8 && c_strcmp_i8(data, NONE) == 0 {
        return -2;
    }
    if delim == b':' as i8 && c_strcmp_i8(data, EMPTY) == 0 {
        return -3;
    }
    -1
}

fn match_pattern(text: &[i8], pattern: &[i8], case_sensitive: i32) -> i32 {
    if case_sensitive != 0 {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }

        // Build wildcard strings exactly like snprintf("*%s*", pattern) etc.
        let pat_len = c_strlen_i8(pattern);
        let pat = &pattern[..pat_len];

        fn build_wildcard(prefix: &[i8], pat: &[i8], suffix: &[i8]) -> [i8; 64] {
            let mut out = [0i8; 64];
            let mut idx = 0usize;

            for &c in prefix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in pat {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            for &c in suffix {
                if idx + 1 >= out.len() {
                    out[out.len() - 1] = 0;
                    return out;
                }
                out[idx] = c;
                idx += 1;
            }
            out[idx.min(out.len() - 1)] = 0;
            out
        }

        let w0 = build_wildcard(&[b'*' as i8], pat, &[b'*' as i8]);
        let w1 = build_wildcard(&[], pat, &[b'*' as i8]);
        let w2 = build_wildcard(&[b'*' as i8], pat, &[]);

        for (i, w) in [w0, w1, w2].iter().enumerate() {
            if c_strcmp_i8(text, w) == 0 {
                return 2 + i as i32;
            }
        }

        let text_len = c_strlen_i8(text);
        let pattern_len = c_strlen_i8(pattern);

        // Important: original C2Rust uses wrapping_sub; if pattern_len > text_len,
        // the loop still runs and may match at i=0 due to strncmp stopping at NUL.
        let mut i0 = 0usize;
        while i0 <= text_len.wrapping_sub(pattern_len) {
            if c_strncmp_i8(&text[i0..], pattern, pattern_len) == 0 {
                return 10usize.wrapping_add(i0) as i32;
            }
            i0 = i0.wrapping_add(1);
        }
    } else {
        if c_strcmp_i8(text, pattern) == 0 {
            return 1;
        }
        let pattern_len = c_strlen_i8(pattern);
        let text_len = c_strlen_i8(text);

        if text_len != pattern_len && c_strncmp_i8(text, pattern, pattern_len) == 0 {
            return 5;
        }

        if text_len == pattern_len {
            let mut m = 1i32;
            let mut i = 0usize;
            while i < pattern_len {
                let mut c1 = text[i];
                let mut c2 = pattern[i];
                if (b'A' as i8..=b'Z' as i8).contains(&c1) {
                    c1 = (c1 as i32 + 32) as i8;
                }
                if (b'A' as i8..=b'Z' as i8).contains(&c2) {
                    c2 = (c2 as i32 + 32) as i8;
                }
                if c1 != c2 {
                    m = 0;
                    break;
                }
                i = i.wrapping_add(1);
            }
            if m != 0 {
                return 6;
            }
        }
    }
    0
}

fn process_strings(
    input: &[i8],
    input_len: usize,
    reference: &[i8],
    ref_len: usize,
    operation: i32,
    flags: u32,
) -> i32 {
    if input.is_empty() {
        return -1;
    }
    match operation {
        0 => {
            // C2Rust checks reference.is_empty() (null pointer). In our harness, ref_len==0
            // still provides a valid (possibly empty) slice, so do NOT reject.
            validate_token(input, reference)
        }
        1 => {
            const START: &[i8] =
                &[b'S' as i8, b'T' as i8, b'A' as i8, b'R' as i8, b'T' as i8, 0];
            const STOP: &[i8] = &[b'S' as i8, b'T' as i8, b'O' as i8, b'P' as i8, 0];
            const PAUSE: &[i8] =
                &[b'P' as i8, b'A' as i8, b'U' as i8, b'S' as i8, b'E' as i8, 0];
            const RESUME: &[i8] = &[
                b'R' as i8,
                b'E' as i8,
                b'S' as i8,
                b'U' as i8,
                b'M' as i8,
                b'E' as i8,
                0,
            ];
            const RESET: &[i8] =
                &[b'R' as i8, b'E' as i8, b'S' as i8, b'E' as i8, b'T' as i8, 0];
            let commands: [&[i8]; 5] = [START, STOP, PAUSE, RESUME, RESET];
            parse_command(input, input_len, &commands, 5)
        }
        2 => {
            // Same: do not reject empty reference slice; C would only reject null pointer.
            let exact = (flags & 0x1) as i32;
            compare_prefix(input, reference, exact)
        }
        3 => {
            let delim = if !reference.is_empty() && ref_len != 0 {
                reference[0]
            } else {
                b':' as i8
            };
            find_delimiter(input, input_len, delim)
        }
        4 => {
            // Same: do not reject empty reference slice.
            let case_sens = (flags & 0x2) as i32;
            match_pattern(input, reference, case_sens)
        }
        _ => -3,
    }
}

fn parse_i32(tok: &str) -> Option<i32> {
    tok.parse::<i32>().ok()
}
fn parse_u32(tok: &str) -> Option<u32> {
    tok.parse::<u32>().ok()
}
fn parse_usize(tok: &str) -> Option<usize> {
    tok.parse::<usize>().ok()
}
fn parse_i8(tok: &str) -> Option<i8> {
    let v = tok.parse::<i32>().ok()?;
    Some(v as i8)
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut idx = 0usize;

    let operation = match tokens.get(idx).and_then(|t| parse_i32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading operation\n");
            std::process::exit(1);
        }
    };

    let flags = match tokens.get(idx).and_then(|t| parse_u32(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading flags\n");
            std::process::exit(1);
        }
    };

    let input_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading input length\n");
            std::process::exit(1);
        }
    };
    if input_len > MAX_LEN {
        eprint!("Error: input length {} exceeds maximum 1024\n", input_len);
        std::process::exit(1);
    }

    let mut input_bytes = Vec::with_capacity(input_len);
    for i in 0..input_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                input_bytes.push(v);
            }
            None => {
                eprint!("Error reading input byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let ref_len = match tokens.get(idx).and_then(|t| parse_usize(t)) {
        Some(v) => {
            idx += 1;
            v
        }
        None => {
            eprint!("Error reading reference length\n");
            std::process::exit(1);
        }
    };
    if ref_len > MAX_LEN {
        eprint!("Error: reference length {} exceeds maximum 1024\n", ref_len);
        std::process::exit(1);
    }

    let mut reference_bytes = Vec::with_capacity(ref_len);
    for i in 0..ref_len {
        match tokens.get(idx).and_then(|t| parse_i8(t)) {
            Some(v) => {
                idx += 1;
                reference_bytes.push(v);
            }
            None => {
                eprint!("Error reading reference byte {}\n", i);
                std::process::exit(1);
            }
        }
    }

    let result = process_strings(
        &input_bytes,
        input_len,
        &reference_bytes,
        ref_len,
        operation,
        flags,
    );
    print!("{result}\n");
}
```

**Entity:** process_strings (operation: i32, flags: u32, reference/ref_len)

**States:** Op0(TokenValidation), Op1(CommandParse), Op2(PrefixCompare), Op3(DelimiterFind), Op4(PatternMatch), InvalidOp

**Transitions:**
- InvalidOp -> (error) via default arm `_ => -3`
- Any -> Op0/Op1/Op2/Op3/Op4 via `match operation { 0..4 => ... }`

**Evidence:** fn process_strings(..., operation: i32, flags: u32) -> i32: `match operation { 0 => ..., 1 => ..., 2 => ..., 3 => ..., 4 => ..., _ => -3 }` defines a runtime-tagged protocol; operation 2: `let exact = (flags & 0x1) as i32; compare_prefix(input, reference, exact)` (flag bit 0 only meaningful here); operation 4: `let case_sens = (flags & 0x2) as i32; match_pattern(input, reference, case_sens)` (flag bit 1 only meaningful here); operation 3: `let delim = if !reference.is_empty() && ref_len != 0 { reference[0] } else { b':' as i8 };` (reference used as delimiter only under a runtime condition); operation 0/2/4 comments: 'do not reject empty reference slice; C would only reject null pointer' indicates subtle preconditions/semantics depend on operation

**Implementation:** Replace `(operation: i32, flags: u32, reference/ref_len)` with a typed enum describing each operation and its parameters, e.g. `enum Operation<'a> { ValidateToken { expected: &'a CStrI8 }, ParseCommand, ComparePrefix { prefix: &'a CStrI8, exact: bool }, FindDelimiter { delim: i8 }, MatchPattern { pattern: &'a CStrI8, case_sensitive: bool } }` and expose `process_strings(input: &CStrI8, op: Operation) -> i32`. This makes invalid flag/parameter combinations unrepresentable and removes the `_ => -3` invalid-op path.

---

