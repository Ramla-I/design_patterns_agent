# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 1
- **Protocol**: 2
- **Modules analyzed**: 1

## State Machine Invariants

### 2. Scanner input cursor protocol (HasMore / Exhausted)

**Location**: `/data/test_case/main.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Scanner maintains a cursor (idx) into an in-memory byte buffer (input). Methods like next_i32() are only able to produce values while idx < input.len(); once idx reaches the end, parsing is exhausted and next_i32() returns None. This is an implicit state machine encoded by idx relative to input.len(), but the type system does not distinguish or prevent calls in the Exhausted state; callers must handle None correctly and avoid assuming progress.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct buffer_t, 9 free function(s); struct buffer_array_t, 2 free function(s); 2 free function(s)

    0
}

struct Scanner {
    input: Vec<u8>,
    idx: usize,
}
impl Scanner {
    fn new() -> Self {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        Self { input, idx: 0 }
    }

    fn next_i32(&mut self) -> Option<i32> {
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
        if self.idx >= self.input.len() {
            return None;
        }

        let mut sign: i32 = 1;
        if self.input[self.idx] == b'-' {
            sign = -1;
            self.idx += 1;
        }

        let mut val: i32 = 0;
        let mut any = false;
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_digit() {
            any = true;
            let digit = (self.input[self.idx] - b'0') as i32;
            // Must not saturate: C's scanf into int overflows (wraps) on typical targets.
            val = val.wrapping_mul(10).wrapping_add(digit);
            self.idx += 1;
        }
        any.then_some(val.wrapping_mul(sign))
    }
}

```

**Entity:** Scanner

**States:** HasMoreInput, Exhausted

**Transitions:**
- HasMoreInput -> HasMoreInput via next_i32() advancing idx while skipping whitespace / consuming digits
- HasMoreInput -> Exhausted via next_i32() when idx advances to input.len() (or starts at/after end)
- Exhausted -> Exhausted via next_i32() returning None

**Evidence:** field `idx: usize` is a runtime cursor into `input: Vec<u8>`; next_i32(): `while self.idx < self.input.len() && ... { self.idx += 1; }` (cursor advances); next_i32(): `if self.idx >= self.input.len() { return None; }` defines the exhausted state; next_i32(): loop consumes digits and increments `self.idx` until a non-digit/end is reached

**Implementation:** Model the cursor state at the type level, e.g. `Scanner<S>` with `HasMore` and `Exhausted` states; have `next_i32(self) -> (Option<i32>, Scanner<...>)` or split into an iterator type `struct Tokens<'a> { ... }` where `next()` returns `Option<i32>` and ownership/lifetimes make exhaustion explicit. Alternatively, provide `impl Iterator for Scanner` to make the 'may be exhausted' contract explicit in the API.

---

### 3. buffer_t validity invariant (length-bound + checksum-consistent)

**Location**: `/data/test_case/main.rs:1-476`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Many operations implicitly assume a buffer is in a Valid state where (1) length is within 0..=256 and (2) checksum matches calculate_checksum(data, length). This is only partially checked at runtime (validate_buffer) and is otherwise maintained by convention (each mutating operation recomputes checksum). The type system allows constructing or mutating buffer_t into Invalid states (e.g., arbitrary length/checksum), and most functions accept &buffer_t directly without proving validity.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct buffer_t, 9 free function(s); struct buffer_array_t, 2 free function(s); struct Scanner, impl Scanner (2 methods)

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

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

#[derive(Copy, Clone)]
pub struct buffer_t {
    pub data: [u8; 256],
    pub length: usize,
    pub checksum: u32,
}

#[derive(Clone)]
pub struct buffer_array_t {
    pub buffers: Vec<buffer_t>,
    pub count: i32,
    pub capacity: i32,
}

fn calculate_checksum(data: &[u8], length: usize) -> u32 {
    let mut sum: u32 = 0;
    for &b in data.iter().take(length) {
        sum = (sum << 3) ^ (b as u32);
    }
    sum
}

fn validate_buffer(buf: Option<&mut buffer_t>) -> bool {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer");
        return false_0 != 0;
    };

    if buf.length > 256 {
        eprintln!(
            "Error: Buffer length {0} exceeds maximum 256",
            buf.length as u64
        );
        return false_0 != 0;
    }

    let expected = calculate_checksum(&buf.data, buf.length);
    if buf.checksum != expected {
        eprintln!(
            "Warning: Checksum mismatch. Expected {0}, got {1}",
            expected, buf.checksum
        );
    }
    true_0 != 0
}

fn init_buffer_array(initial_capacity: i32) -> Option<buffer_array_t> {
    if initial_capacity <= 0 {
        eprintln!("Error: Invalid capacity {initial_capacity}");
        return None;
    }

    let cap = initial_capacity as usize;
    let mut buffers = Vec::with_capacity(cap);
    buffers.resize(
        cap,
        buffer_t {
            data: [0; 256],
            length: 0,
            checksum: 0,
        },
    );

    Some(buffer_array_t {
        buffers,
        count: 0,
        capacity: initial_capacity,
    })
}

fn free_buffer_array(_: Option<&buffer_array_t>) {
    // No-op in safe Rust; kept for structure parity.
}

fn buffer_copy(src: Option<&mut buffer_t>, dst: Option<&mut buffer_t>) -> i32 {
    let (Some(src), Some(dst)) = (src, dst) else {
        eprintln!("Error: NULL pointer in buffer_copy");
        return -1;
    };

    if !validate_buffer(Some(src)) {
        return -1;
    }

    dst.data[..src.length].copy_from_slice(&src.data[..src.length]);
    dst.length = src.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_reverse(buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in reverse");
        return -1;
    };
    if buf.length == 0 {
        return 0;
    }

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    for i in 0..buf.length {
        buf.data[i] = temp[buf.length - 1 - i];
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn buffer_merge(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_merge");
        return -1;
    };

    if src1.length + src2.length > 256 {
        eprintln!(
            "Error: Merged length {0} exceeds maximum",
            (src1.length + src2.length) as u64
        );
        return -1;
    }

    dst.data[..src1.length].copy_from_slice(&src1.data[..src1.length]);
    dst.data[src1.length..src1.length + src2.length].copy_from_slice(&src2.data[..src2.length]);
    dst.length = src1.length + src2.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_split(
    src: Option<&mut buffer_t>,
    split_pos: usize,
    dst1: Option<&mut buffer_t>,
    dst2: Option<&mut buffer_t>,
) -> i32 {
    let (Some(src), Some(dst1), Some(dst2)) = (src, dst1, dst2) else {
        eprintln!("Error: NULL pointer in buffer_split");
        return -1;
    };

    if split_pos > src.length {
        eprintln!(
            "Error: Split position {0} exceeds length {1}",
            split_pos as u64,
            src.length as u64
        );
        return -1;
    }

    if split_pos != 0 {
        dst1.data[..split_pos].copy_from_slice(&src.data[..split_pos]);
    }
    dst1.length = split_pos;
    dst1.checksum = calculate_checksum(&dst1.data, dst1.length);

    let remaining = src.length - split_pos;
    if remaining != 0 {
        dst2.data[..remaining].copy_from_slice(&src.data[split_pos..split_pos + remaining]);
    }
    dst2.length = remaining;
    dst2.checksum = calculate_checksum(&dst2.data, dst2.length);

    0
}

fn buffer_interleave(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_interleave");
        return -1;
    };

    let max_len = src1.length.max(src2.length);
    if src1.length + src2.length > 256 {
        eprintln!("Error: Interleaved length exceeds maximum");
        return -1;
    }

    let mut dst_pos = 0usize;
    for i in 0..max_len {
        if i < src1.length {
            dst.data[dst_pos] = src1.data[i];
            dst_pos += 1;
        }
        if i < src2.length {
            dst.data[dst_pos] = src2.data[i];
            dst_pos += 1;
        }
    }

    dst.length = dst_pos;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_rotate(buf: Option<&mut buffer_t>, mut positions: i32) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in rotate");
        return -1;
    };

    if buf.length == 0 || positions == 0 {
        return 0;
    }

    // C behavior: positions %= len; if positions < 0 then positions += len (with wrap)
    positions %= buf.length as i32;
    if positions < 0 {
        positions = positions.wrapping_add(buf.length as i32);
    }
    let positions = positions as usize;

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    // IMPORTANT: Match the original C2Rust logic exactly:
    // data[0..len-positions] = temp[positions..len]
    // data[len-positions..len] = temp[0..positions]
    // This is a LEFT rotation by `positions`.
    let left_len = buf.length - positions;
    buf.data[..left_len].copy_from_slice(&temp[positions..positions + left_len]);
    buf.data[left_len..left_len + positions].copy_from_slice(&temp[..positions]);

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

struct Scanner {
    input: Vec<u8>,
    idx: usize,
}
impl Scanner {
    fn new() -> Self {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        Self { input, idx: 0 }
    }

    fn next_i32(&mut self) -> Option<i32> {
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
        if self.idx >= self.input.len() {
            return None;
        }

        let mut sign: i32 = 1;
        if self.input[self.idx] == b'-' {
            sign = -1;
            self.idx += 1;
        }

        let mut val: i32 = 0;
        let mut any = false;
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_digit() {
            any = true;
            let digit = (self.input[self.idx] - b'0') as i32;
            // Must not saturate: C's scanf into int overflows (wraps) on typical targets.
            val = val.wrapping_mul(10).wrapping_add(digit);
            self.idx += 1;
        }
        any.then_some(val.wrapping_mul(sign))
    }
}

fn read_buffer(scan: &mut Scanner, buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in read_buffer");
        return -1;
    };

    let Some(length) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer length");
        return -1;
    };

    if !(0..=256).contains(&length) {
        eprintln!("Error: Invalid buffer length {length}");
        return -1;
    }

    buf.length = length as usize;
    for i in 0..buf.length {
        let Some(byte) = scan.next_i32() else {
            eprintln!("Error: Failed to read byte {0}", i as u64);
            return -1;
        };
        buf.data[i] = byte as u8;
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn write_buffer(buf: Option<&buffer_t>) {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in write_buffer");
        return;
    };

    print!("{0}", buf.length as u64);
    for i in 0..buf.length {
        print!(" {0}", buf.data[i] as u32);
    }
    println!();
}

fn main() {
    let mut scan = Scanner::new();

    let Some(operation) = scan.next_i32() else {
        eprintln!("Error: Failed to read operation");
        std::process::exit(1);
    };
    let Some(buffer_count) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer count");
        std::process::exit(1);
    };

    if buffer_count <= 0 || buffer_count > 100 {
        eprintln!("Error: Invalid buffer count {buffer_count}");
        std::process::exit(1);
    }

    let mut buffers = match init_buffer_array(buffer_count) {
        Some(b) => b,
        None => std::process::exit(1),
    };

    for i in 0..(buffer_count as usize) {
        if read_buffer(&mut scan, buffers.buffers.get_mut(i)) != 0 {
            free_buffer_array(Some(&buffers));
            std::process::exit(1);
        }
        buffers.count += 1;
    }

    let mut result: i32 = 0;
    match operation {
        0 => {
            if buffer_count >= 2 {
                let mut temp = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                result = buffer_copy(buffers.buffers.get_mut(0), Some(&mut temp));
                if result == 0 {
                    write_buffer(Some(&temp));
                }
            } else {
                eprintln!("Error: Copy needs at least 2 buffers");
                result = -1;
            }
        }
        1 => {
            for i in 0..(buffer_count as usize) {
                result = buffer_reverse(buffers.buffers.get_mut(i));
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        2 => {
            if buffer_count >= 2 {
                let mut merged = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_merge(&src1, &src2, Some(&mut merged));
                if result == 0 {
                    write_buffer(Some(&merged));
                }
            } else {
                eprintln!("Error: Merge needs at least 2 buffers");
                result = -1;
            }
        }
        3 => {
            if buffer_count >= 1 {
                let Some(split_pos) = scan.next_i32() else {
                    eprintln!("Error: Failed to read split position");
                    std::process::exit(1);
                };

                let mut part1 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let mut part2 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };

                result = buffer_split(
                    buffers.buffers.get_mut(0),
                    split_pos as usize,
                    Some(&mut part1),
                    Some(&mut part2),
                );
                if result == 0 {
                    write_buffer(Some(&part1));
                    write_buffer(Some(&part2));
                }
            }
        }
        4 => {
            if buffer_count >= 2 {
                let mut interleaved = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_interleave(&src1, &src2, Some(&mut interleaved));
                if result == 0 {
                    write_buffer(Some(&interleaved));
                }
            } else {
                eprintln!("Error: Interleave needs at least 2 buffers");
                result = -1;
            }
        }
        5 => {
            let Some(positions) = scan.next_i32() else {
                eprintln!("Error: Failed to read rotation amount");
                std::process::exit(1);
            };

            for i in 0..(buffer_count as usize) {
                result = buffer_rotate(buffers.buffers.get_mut(i), positions);
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        6 => {
            for i in 0..(buffer_count as usize) {
                println!("{0}", buffers.buffers[i].checksum);
            }
        }
        _ => {
            eprintln!("Error: Unknown operation {operation}");
            result = -1;
        }
    }

    free_buffer_array(Some(&buffers));
    std::process::exit(if result != 0 { 1 } else { 0 });
}
```

**Entity:** buffer_t

**States:** Valid, Invalid

**Transitions:**
- Invalid -> Valid via read_buffer() (sets length, fills bytes, recomputes checksum)
- Invalid -> Valid via buffer_copy()/buffer_reverse()/buffer_merge()/buffer_split()/buffer_interleave()/buffer_rotate() (recomputes checksum after writing)
- Valid -> Invalid via external mutation of public fields length/checksum/data (no encapsulation)

**Evidence:** buffer_t fields are public: `pub data: [u8; 256]`, `pub length: usize`, `pub checksum: u32` (external code can violate invariants); `validate_buffer(..)` checks `if buf.length > 256 { ... }` and compares `buf.checksum != expected` (runtime validity definition); `buffer_copy(..)` calls `if !validate_buffer(Some(src)) { return -1; }` before copying; Mutators recompute checksum: `buf.checksum = calculate_checksum(&buf.data, buf.length)` in read_buffer/buffer_copy/buffer_reverse/buffer_merge/buffer_split/buffer_interleave/buffer_rotate; Error messages reveal invariants: "Buffer length {0} exceeds maximum 256", "Warning: Checksum mismatch"

**Implementation:** Make buffer_t fields private and expose a `struct ValidBuffer(buffer_t)` (or `Buffer` with private fields). Provide constructors/mutators that maintain invariants and return `Result<ValidBuffer, Error>` when parsing/validating. Functions like merge/interleave take `&ValidBuffer` and produce `ValidBuffer`, eliminating the need for validate_buffer checks and preventing construction of invalid length/checksum.

---

## Precondition Invariants

### 1. buffer_array_t length/capacity invariants (count/capacity must match buffers)

**Location**: `/data/test_case/main.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: buffer_array_t carries redundant size metadata: `buffers: Vec<buffer_t>` plus separate `count: i32` and `capacity: i32`. Correctness likely depends on implicit invariants such as: count == buffers.len(), capacity == buffers.capacity() (or >= count), count/capacity are non-negative, and count <= capacity. None of these are enforced by the type system; because the fields are public, any code can construct an Inconsistent state (e.g., count not matching buffers length, negative values, capacity smaller than count), which can later lead to out-of-bounds logic or incorrect iteration/allocation decisions in code that trusts `count`/`capacity`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct buffer_t, 9 free function(s); struct Scanner, impl Scanner (2 methods); 2 free function(s)

}

#[derive(Clone)]
pub struct buffer_array_t {
    pub buffers: Vec<buffer_t>,
    pub count: i32,
    pub capacity: i32,
}

```

**Entity:** buffer_array_t

**States:** Consistent, Inconsistent

**Transitions:**
- Consistent -> Inconsistent via public field mutation (e.g., writing `count`/`capacity` independently of `buffers`)

**Evidence:** struct buffer_array_t { pub buffers: Vec<buffer_t>, pub count: i32, pub capacity: i32 } — redundant runtime-tracked size fields; `pub` on all three fields allows external code to break the relationship between `buffers`, `count`, and `capacity`

**Implementation:** Make fields private and derive count/capacity from Vec (remove `count`/`capacity` entirely), or wrap them in validated newtypes (e.g., `NonNegativeI32`) and provide constructors/methods that maintain invariants (e.g., `fn len(&self)->usize { self.buffers.len() }`, `fn push(&mut self, b: buffer_t)` updates metadata). If distinct semantics are intended (e.g., FFI-style fields), use a private representation plus accessor methods and/or a `TryFrom` constructor that validates `count <= capacity` and `count as usize == buffers.len()`.

---

## Protocol Invariants

### 4. buffer_array_t initialization/occupancy protocol (Allocated -> PartiallyFilled -> Filled)

**Location**: `/data/test_case/main.rs:1-476`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: buffer_array_t encodes a container protocol where `capacity` and `count` must stay consistent with `buffers.len()` and how many entries have been initialized by input. This is tracked manually with `count: i32` alongside `buffers: Vec<buffer_t>`, and the correctness relies on the convention that `count` is incremented exactly once per successful `read_buffer`. The type system does not prevent mismatches (e.g., count not matching actual filled elements, capacity not matching buffers.len(), or using elements beyond `count`).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct buffer_t, 9 free function(s); struct buffer_array_t, 2 free function(s); struct Scanner, impl Scanner (2 methods)

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

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

#[derive(Copy, Clone)]
pub struct buffer_t {
    pub data: [u8; 256],
    pub length: usize,
    pub checksum: u32,
}

#[derive(Clone)]
pub struct buffer_array_t {
    pub buffers: Vec<buffer_t>,
    pub count: i32,
    pub capacity: i32,
}

fn calculate_checksum(data: &[u8], length: usize) -> u32 {
    let mut sum: u32 = 0;
    for &b in data.iter().take(length) {
        sum = (sum << 3) ^ (b as u32);
    }
    sum
}

fn validate_buffer(buf: Option<&mut buffer_t>) -> bool {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer");
        return false_0 != 0;
    };

    if buf.length > 256 {
        eprintln!(
            "Error: Buffer length {0} exceeds maximum 256",
            buf.length as u64
        );
        return false_0 != 0;
    }

    let expected = calculate_checksum(&buf.data, buf.length);
    if buf.checksum != expected {
        eprintln!(
            "Warning: Checksum mismatch. Expected {0}, got {1}",
            expected, buf.checksum
        );
    }
    true_0 != 0
}

fn init_buffer_array(initial_capacity: i32) -> Option<buffer_array_t> {
    if initial_capacity <= 0 {
        eprintln!("Error: Invalid capacity {initial_capacity}");
        return None;
    }

    let cap = initial_capacity as usize;
    let mut buffers = Vec::with_capacity(cap);
    buffers.resize(
        cap,
        buffer_t {
            data: [0; 256],
            length: 0,
            checksum: 0,
        },
    );

    Some(buffer_array_t {
        buffers,
        count: 0,
        capacity: initial_capacity,
    })
}

fn free_buffer_array(_: Option<&buffer_array_t>) {
    // No-op in safe Rust; kept for structure parity.
}

fn buffer_copy(src: Option<&mut buffer_t>, dst: Option<&mut buffer_t>) -> i32 {
    let (Some(src), Some(dst)) = (src, dst) else {
        eprintln!("Error: NULL pointer in buffer_copy");
        return -1;
    };

    if !validate_buffer(Some(src)) {
        return -1;
    }

    dst.data[..src.length].copy_from_slice(&src.data[..src.length]);
    dst.length = src.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_reverse(buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in reverse");
        return -1;
    };
    if buf.length == 0 {
        return 0;
    }

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    for i in 0..buf.length {
        buf.data[i] = temp[buf.length - 1 - i];
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn buffer_merge(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_merge");
        return -1;
    };

    if src1.length + src2.length > 256 {
        eprintln!(
            "Error: Merged length {0} exceeds maximum",
            (src1.length + src2.length) as u64
        );
        return -1;
    }

    dst.data[..src1.length].copy_from_slice(&src1.data[..src1.length]);
    dst.data[src1.length..src1.length + src2.length].copy_from_slice(&src2.data[..src2.length]);
    dst.length = src1.length + src2.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_split(
    src: Option<&mut buffer_t>,
    split_pos: usize,
    dst1: Option<&mut buffer_t>,
    dst2: Option<&mut buffer_t>,
) -> i32 {
    let (Some(src), Some(dst1), Some(dst2)) = (src, dst1, dst2) else {
        eprintln!("Error: NULL pointer in buffer_split");
        return -1;
    };

    if split_pos > src.length {
        eprintln!(
            "Error: Split position {0} exceeds length {1}",
            split_pos as u64,
            src.length as u64
        );
        return -1;
    }

    if split_pos != 0 {
        dst1.data[..split_pos].copy_from_slice(&src.data[..split_pos]);
    }
    dst1.length = split_pos;
    dst1.checksum = calculate_checksum(&dst1.data, dst1.length);

    let remaining = src.length - split_pos;
    if remaining != 0 {
        dst2.data[..remaining].copy_from_slice(&src.data[split_pos..split_pos + remaining]);
    }
    dst2.length = remaining;
    dst2.checksum = calculate_checksum(&dst2.data, dst2.length);

    0
}

fn buffer_interleave(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_interleave");
        return -1;
    };

    let max_len = src1.length.max(src2.length);
    if src1.length + src2.length > 256 {
        eprintln!("Error: Interleaved length exceeds maximum");
        return -1;
    }

    let mut dst_pos = 0usize;
    for i in 0..max_len {
        if i < src1.length {
            dst.data[dst_pos] = src1.data[i];
            dst_pos += 1;
        }
        if i < src2.length {
            dst.data[dst_pos] = src2.data[i];
            dst_pos += 1;
        }
    }

    dst.length = dst_pos;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_rotate(buf: Option<&mut buffer_t>, mut positions: i32) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in rotate");
        return -1;
    };

    if buf.length == 0 || positions == 0 {
        return 0;
    }

    // C behavior: positions %= len; if positions < 0 then positions += len (with wrap)
    positions %= buf.length as i32;
    if positions < 0 {
        positions = positions.wrapping_add(buf.length as i32);
    }
    let positions = positions as usize;

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    // IMPORTANT: Match the original C2Rust logic exactly:
    // data[0..len-positions] = temp[positions..len]
    // data[len-positions..len] = temp[0..positions]
    // This is a LEFT rotation by `positions`.
    let left_len = buf.length - positions;
    buf.data[..left_len].copy_from_slice(&temp[positions..positions + left_len]);
    buf.data[left_len..left_len + positions].copy_from_slice(&temp[..positions]);

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

struct Scanner {
    input: Vec<u8>,
    idx: usize,
}
impl Scanner {
    fn new() -> Self {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        Self { input, idx: 0 }
    }

    fn next_i32(&mut self) -> Option<i32> {
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
        if self.idx >= self.input.len() {
            return None;
        }

        let mut sign: i32 = 1;
        if self.input[self.idx] == b'-' {
            sign = -1;
            self.idx += 1;
        }

        let mut val: i32 = 0;
        let mut any = false;
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_digit() {
            any = true;
            let digit = (self.input[self.idx] - b'0') as i32;
            // Must not saturate: C's scanf into int overflows (wraps) on typical targets.
            val = val.wrapping_mul(10).wrapping_add(digit);
            self.idx += 1;
        }
        any.then_some(val.wrapping_mul(sign))
    }
}

fn read_buffer(scan: &mut Scanner, buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in read_buffer");
        return -1;
    };

    let Some(length) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer length");
        return -1;
    };

    if !(0..=256).contains(&length) {
        eprintln!("Error: Invalid buffer length {length}");
        return -1;
    }

    buf.length = length as usize;
    for i in 0..buf.length {
        let Some(byte) = scan.next_i32() else {
            eprintln!("Error: Failed to read byte {0}", i as u64);
            return -1;
        };
        buf.data[i] = byte as u8;
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn write_buffer(buf: Option<&buffer_t>) {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in write_buffer");
        return;
    };

    print!("{0}", buf.length as u64);
    for i in 0..buf.length {
        print!(" {0}", buf.data[i] as u32);
    }
    println!();
}

fn main() {
    let mut scan = Scanner::new();

    let Some(operation) = scan.next_i32() else {
        eprintln!("Error: Failed to read operation");
        std::process::exit(1);
    };
    let Some(buffer_count) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer count");
        std::process::exit(1);
    };

    if buffer_count <= 0 || buffer_count > 100 {
        eprintln!("Error: Invalid buffer count {buffer_count}");
        std::process::exit(1);
    }

    let mut buffers = match init_buffer_array(buffer_count) {
        Some(b) => b,
        None => std::process::exit(1),
    };

    for i in 0..(buffer_count as usize) {
        if read_buffer(&mut scan, buffers.buffers.get_mut(i)) != 0 {
            free_buffer_array(Some(&buffers));
            std::process::exit(1);
        }
        buffers.count += 1;
    }

    let mut result: i32 = 0;
    match operation {
        0 => {
            if buffer_count >= 2 {
                let mut temp = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                result = buffer_copy(buffers.buffers.get_mut(0), Some(&mut temp));
                if result == 0 {
                    write_buffer(Some(&temp));
                }
            } else {
                eprintln!("Error: Copy needs at least 2 buffers");
                result = -1;
            }
        }
        1 => {
            for i in 0..(buffer_count as usize) {
                result = buffer_reverse(buffers.buffers.get_mut(i));
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        2 => {
            if buffer_count >= 2 {
                let mut merged = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_merge(&src1, &src2, Some(&mut merged));
                if result == 0 {
                    write_buffer(Some(&merged));
                }
            } else {
                eprintln!("Error: Merge needs at least 2 buffers");
                result = -1;
            }
        }
        3 => {
            if buffer_count >= 1 {
                let Some(split_pos) = scan.next_i32() else {
                    eprintln!("Error: Failed to read split position");
                    std::process::exit(1);
                };

                let mut part1 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let mut part2 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };

                result = buffer_split(
                    buffers.buffers.get_mut(0),
                    split_pos as usize,
                    Some(&mut part1),
                    Some(&mut part2),
                );
                if result == 0 {
                    write_buffer(Some(&part1));
                    write_buffer(Some(&part2));
                }
            }
        }
        4 => {
            if buffer_count >= 2 {
                let mut interleaved = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_interleave(&src1, &src2, Some(&mut interleaved));
                if result == 0 {
                    write_buffer(Some(&interleaved));
                }
            } else {
                eprintln!("Error: Interleave needs at least 2 buffers");
                result = -1;
            }
        }
        5 => {
            let Some(positions) = scan.next_i32() else {
                eprintln!("Error: Failed to read rotation amount");
                std::process::exit(1);
            };

            for i in 0..(buffer_count as usize) {
                result = buffer_rotate(buffers.buffers.get_mut(i), positions);
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        6 => {
            for i in 0..(buffer_count as usize) {
                println!("{0}", buffers.buffers[i].checksum);
            }
        }
        _ => {
            eprintln!("Error: Unknown operation {operation}");
            result = -1;
        }
    }

    free_buffer_array(Some(&buffers));
    std::process::exit(if result != 0 { 1 } else { 0 });
}
```

**Entity:** buffer_array_t

**States:** Allocated(capacity set, count=0), PartiallyFilled(0<count<capacity), Filled(count==capacity)

**Transitions:**
- Allocated -> PartiallyFilled via successful loop iterations that call read_buffer() then `buffers.count += 1`
- PartiallyFilled -> Filled when `count` reaches `capacity`/expected buffer_count
- Any state -> inconsistent state via arbitrary mutation of public `count`/`capacity` or direct access to `buffers` ignoring `count`

**Evidence:** buffer_array_t has redundant runtime state: `pub buffers: Vec<buffer_t>, pub count: i32, pub capacity: i32`; `init_buffer_array(initial_capacity)` sets `buffers.resize(cap, buffer_t{...})`, `count: 0`, `capacity: initial_capacity` (protocol start); main fills using `read_buffer(&mut scan, buffers.buffers.get_mut(i))` then `buffers.count += 1` (manual state transition); No check ties `count`/`capacity` to `buffers.len()` after construction; fields are public so invariants can be violated from outside

**Implementation:** Hide fields and represent stages as types, e.g. `BufferArray<Allocated>` returned from `init`, with a method `read_next(self, &mut Scanner) -> Result<BufferArray<...>, ...>` that increments an internal index and eventually yields `BufferArray<Filled>`. Alternatively, remove `count`/`capacity` fields entirely and rely on `Vec` length plus an internal private cursor, exposing only safe iteration over initialized buffers.

---

### 5. Scanner consumption protocol (UnreadInput -> PartiallyConsumed -> Exhausted)

**Location**: `/data/test_case/main.rs:1-476`

**Confidence**: medium

**Suggested Pattern**: session_type

**Description**: Scanner is a stateful token stream over stdin: calls to next_i32 advance `idx` and eventually exhaust the input. Many call sites treat missing tokens as fatal (exit) or error (-1), so the implicit protocol is that the required number of integers must be present and consumed in the correct order for the chosen `operation`. This ordering and availability is not expressed in types; it is enforced by runtime Option checks and process exits.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct buffer_t, 9 free function(s); struct buffer_array_t, 2 free function(s); struct Scanner, impl Scanner (2 methods)

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

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

#[derive(Copy, Clone)]
pub struct buffer_t {
    pub data: [u8; 256],
    pub length: usize,
    pub checksum: u32,
}

#[derive(Clone)]
pub struct buffer_array_t {
    pub buffers: Vec<buffer_t>,
    pub count: i32,
    pub capacity: i32,
}

fn calculate_checksum(data: &[u8], length: usize) -> u32 {
    let mut sum: u32 = 0;
    for &b in data.iter().take(length) {
        sum = (sum << 3) ^ (b as u32);
    }
    sum
}

fn validate_buffer(buf: Option<&mut buffer_t>) -> bool {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer");
        return false_0 != 0;
    };

    if buf.length > 256 {
        eprintln!(
            "Error: Buffer length {0} exceeds maximum 256",
            buf.length as u64
        );
        return false_0 != 0;
    }

    let expected = calculate_checksum(&buf.data, buf.length);
    if buf.checksum != expected {
        eprintln!(
            "Warning: Checksum mismatch. Expected {0}, got {1}",
            expected, buf.checksum
        );
    }
    true_0 != 0
}

fn init_buffer_array(initial_capacity: i32) -> Option<buffer_array_t> {
    if initial_capacity <= 0 {
        eprintln!("Error: Invalid capacity {initial_capacity}");
        return None;
    }

    let cap = initial_capacity as usize;
    let mut buffers = Vec::with_capacity(cap);
    buffers.resize(
        cap,
        buffer_t {
            data: [0; 256],
            length: 0,
            checksum: 0,
        },
    );

    Some(buffer_array_t {
        buffers,
        count: 0,
        capacity: initial_capacity,
    })
}

fn free_buffer_array(_: Option<&buffer_array_t>) {
    // No-op in safe Rust; kept for structure parity.
}

fn buffer_copy(src: Option<&mut buffer_t>, dst: Option<&mut buffer_t>) -> i32 {
    let (Some(src), Some(dst)) = (src, dst) else {
        eprintln!("Error: NULL pointer in buffer_copy");
        return -1;
    };

    if !validate_buffer(Some(src)) {
        return -1;
    }

    dst.data[..src.length].copy_from_slice(&src.data[..src.length]);
    dst.length = src.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_reverse(buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in reverse");
        return -1;
    };
    if buf.length == 0 {
        return 0;
    }

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    for i in 0..buf.length {
        buf.data[i] = temp[buf.length - 1 - i];
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn buffer_merge(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_merge");
        return -1;
    };

    if src1.length + src2.length > 256 {
        eprintln!(
            "Error: Merged length {0} exceeds maximum",
            (src1.length + src2.length) as u64
        );
        return -1;
    }

    dst.data[..src1.length].copy_from_slice(&src1.data[..src1.length]);
    dst.data[src1.length..src1.length + src2.length].copy_from_slice(&src2.data[..src2.length]);
    dst.length = src1.length + src2.length;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_split(
    src: Option<&mut buffer_t>,
    split_pos: usize,
    dst1: Option<&mut buffer_t>,
    dst2: Option<&mut buffer_t>,
) -> i32 {
    let (Some(src), Some(dst1), Some(dst2)) = (src, dst1, dst2) else {
        eprintln!("Error: NULL pointer in buffer_split");
        return -1;
    };

    if split_pos > src.length {
        eprintln!(
            "Error: Split position {0} exceeds length {1}",
            split_pos as u64,
            src.length as u64
        );
        return -1;
    }

    if split_pos != 0 {
        dst1.data[..split_pos].copy_from_slice(&src.data[..split_pos]);
    }
    dst1.length = split_pos;
    dst1.checksum = calculate_checksum(&dst1.data, dst1.length);

    let remaining = src.length - split_pos;
    if remaining != 0 {
        dst2.data[..remaining].copy_from_slice(&src.data[split_pos..split_pos + remaining]);
    }
    dst2.length = remaining;
    dst2.checksum = calculate_checksum(&dst2.data, dst2.length);

    0
}

fn buffer_interleave(src1: &buffer_t, src2: &buffer_t, dst: Option<&mut buffer_t>) -> i32 {
    let Some(dst) = dst else {
        eprintln!("Error: NULL pointer in buffer_interleave");
        return -1;
    };

    let max_len = src1.length.max(src2.length);
    if src1.length + src2.length > 256 {
        eprintln!("Error: Interleaved length exceeds maximum");
        return -1;
    }

    let mut dst_pos = 0usize;
    for i in 0..max_len {
        if i < src1.length {
            dst.data[dst_pos] = src1.data[i];
            dst_pos += 1;
        }
        if i < src2.length {
            dst.data[dst_pos] = src2.data[i];
            dst_pos += 1;
        }
    }

    dst.length = dst_pos;
    dst.checksum = calculate_checksum(&dst.data, dst.length);
    0
}

fn buffer_rotate(buf: Option<&mut buffer_t>, mut positions: i32) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in rotate");
        return -1;
    };

    if buf.length == 0 || positions == 0 {
        return 0;
    }

    // C behavior: positions %= len; if positions < 0 then positions += len (with wrap)
    positions %= buf.length as i32;
    if positions < 0 {
        positions = positions.wrapping_add(buf.length as i32);
    }
    let positions = positions as usize;

    let mut temp = [0u8; 256];
    temp[..buf.length].copy_from_slice(&buf.data[..buf.length]);

    // IMPORTANT: Match the original C2Rust logic exactly:
    // data[0..len-positions] = temp[positions..len]
    // data[len-positions..len] = temp[0..positions]
    // This is a LEFT rotation by `positions`.
    let left_len = buf.length - positions;
    buf.data[..left_len].copy_from_slice(&temp[positions..positions + left_len]);
    buf.data[left_len..left_len + positions].copy_from_slice(&temp[..positions]);

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

struct Scanner {
    input: Vec<u8>,
    idx: usize,
}
impl Scanner {
    fn new() -> Self {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).unwrap();
        Self { input, idx: 0 }
    }

    fn next_i32(&mut self) -> Option<i32> {
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_whitespace() {
            self.idx += 1;
        }
        if self.idx >= self.input.len() {
            return None;
        }

        let mut sign: i32 = 1;
        if self.input[self.idx] == b'-' {
            sign = -1;
            self.idx += 1;
        }

        let mut val: i32 = 0;
        let mut any = false;
        while self.idx < self.input.len() && self.input[self.idx].is_ascii_digit() {
            any = true;
            let digit = (self.input[self.idx] - b'0') as i32;
            // Must not saturate: C's scanf into int overflows (wraps) on typical targets.
            val = val.wrapping_mul(10).wrapping_add(digit);
            self.idx += 1;
        }
        any.then_some(val.wrapping_mul(sign))
    }
}

fn read_buffer(scan: &mut Scanner, buf: Option<&mut buffer_t>) -> i32 {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in read_buffer");
        return -1;
    };

    let Some(length) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer length");
        return -1;
    };

    if !(0..=256).contains(&length) {
        eprintln!("Error: Invalid buffer length {length}");
        return -1;
    }

    buf.length = length as usize;
    for i in 0..buf.length {
        let Some(byte) = scan.next_i32() else {
            eprintln!("Error: Failed to read byte {0}", i as u64);
            return -1;
        };
        buf.data[i] = byte as u8;
    }

    buf.checksum = calculate_checksum(&buf.data, buf.length);
    0
}

fn write_buffer(buf: Option<&buffer_t>) {
    let Some(buf) = buf else {
        eprintln!("Error: NULL buffer in write_buffer");
        return;
    };

    print!("{0}", buf.length as u64);
    for i in 0..buf.length {
        print!(" {0}", buf.data[i] as u32);
    }
    println!();
}

fn main() {
    let mut scan = Scanner::new();

    let Some(operation) = scan.next_i32() else {
        eprintln!("Error: Failed to read operation");
        std::process::exit(1);
    };
    let Some(buffer_count) = scan.next_i32() else {
        eprintln!("Error: Failed to read buffer count");
        std::process::exit(1);
    };

    if buffer_count <= 0 || buffer_count > 100 {
        eprintln!("Error: Invalid buffer count {buffer_count}");
        std::process::exit(1);
    }

    let mut buffers = match init_buffer_array(buffer_count) {
        Some(b) => b,
        None => std::process::exit(1),
    };

    for i in 0..(buffer_count as usize) {
        if read_buffer(&mut scan, buffers.buffers.get_mut(i)) != 0 {
            free_buffer_array(Some(&buffers));
            std::process::exit(1);
        }
        buffers.count += 1;
    }

    let mut result: i32 = 0;
    match operation {
        0 => {
            if buffer_count >= 2 {
                let mut temp = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                result = buffer_copy(buffers.buffers.get_mut(0), Some(&mut temp));
                if result == 0 {
                    write_buffer(Some(&temp));
                }
            } else {
                eprintln!("Error: Copy needs at least 2 buffers");
                result = -1;
            }
        }
        1 => {
            for i in 0..(buffer_count as usize) {
                result = buffer_reverse(buffers.buffers.get_mut(i));
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        2 => {
            if buffer_count >= 2 {
                let mut merged = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_merge(&src1, &src2, Some(&mut merged));
                if result == 0 {
                    write_buffer(Some(&merged));
                }
            } else {
                eprintln!("Error: Merge needs at least 2 buffers");
                result = -1;
            }
        }
        3 => {
            if buffer_count >= 1 {
                let Some(split_pos) = scan.next_i32() else {
                    eprintln!("Error: Failed to read split position");
                    std::process::exit(1);
                };

                let mut part1 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let mut part2 = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };

                result = buffer_split(
                    buffers.buffers.get_mut(0),
                    split_pos as usize,
                    Some(&mut part1),
                    Some(&mut part2),
                );
                if result == 0 {
                    write_buffer(Some(&part1));
                    write_buffer(Some(&part2));
                }
            }
        }
        4 => {
            if buffer_count >= 2 {
                let mut interleaved = buffer_t {
                    data: [0; 256],
                    length: 0,
                    checksum: 0,
                };
                let src1 = buffers.buffers[0];
                let src2 = buffers.buffers[1];
                result = buffer_interleave(&src1, &src2, Some(&mut interleaved));
                if result == 0 {
                    write_buffer(Some(&interleaved));
                }
            } else {
                eprintln!("Error: Interleave needs at least 2 buffers");
                result = -1;
            }
        }
        5 => {
            let Some(positions) = scan.next_i32() else {
                eprintln!("Error: Failed to read rotation amount");
                std::process::exit(1);
            };

            for i in 0..(buffer_count as usize) {
                result = buffer_rotate(buffers.buffers.get_mut(i), positions);
                if result != 0 {
                    break;
                }
                write_buffer(buffers.buffers.get(i));
            }
        }
        6 => {
            for i in 0..(buffer_count as usize) {
                println!("{0}", buffers.buffers[i].checksum);
            }
        }
        _ => {
            eprintln!("Error: Unknown operation {operation}");
            result = -1;
        }
    }

    free_buffer_array(Some(&buffers));
    std::process::exit(if result != 0 { 1 } else { 0 });
}
```

**Entity:** Scanner

**States:** HasMoreTokens, Exhausted

**Transitions:**
- HasMoreTokens -> HasMoreTokens via next_i32() consuming whitespace/digits and advancing idx
- HasMoreTokens -> Exhausted when `idx >= input.len()` leading next_i32() to return None

**Evidence:** Scanner stores runtime cursor: `idx: usize` and buffer `input: Vec<u8>`; `next_i32` advances idx while parsing and returns `None` when `self.idx >= self.input.len()`; main enforces ordering/availability via runtime checks: `let Some(operation) = scan.next_i32() else { ... exit(1) }`, similarly for buffer_count, split_pos, positions; read_buffer depends on token availability/ordering: `Failed to read buffer length` / `Failed to read byte {i}` errors triggered when next_i32() returns None

**Implementation:** Model the input format as a typed protocol: parse into an enum like `Command::Copy { buffers: Vec<ValidBuffer> } | Command::Split { buffer: ValidBuffer, split_pos: usize } | ...` via a single parsing step. After parsing, execution functions operate on fully-typed data (no further token consumption), eliminating partial-consumption states and scattered `Option` checks.

---

