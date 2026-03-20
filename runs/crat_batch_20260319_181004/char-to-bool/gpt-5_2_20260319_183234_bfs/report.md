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

### 2. C-like bool-bytes protocol (sequence bytes must represent 0/1 booleans with additional ordering rules)

**Location**: `/data/test_case/main.rs:1-357`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The function conceptually treats input `sequence: &[i8]` as a boolean array with C-like representation (bytes storing 0/1), then applies extra structural rules (first element must be true, last must be false for len>1, no more than 3 identical consecutive values) before classifying by number of transitions. These assumptions are implicit: the input type is just `&[i8]`, and the boolean encoding plus structural constraints are enforced only by runtime checks and return codes, not by a validated/typed representation.

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

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

fn parse_bool(c: i8) -> bool {
    if c == b'y' as i8 || c == b'Y' as i8 {
        true_0 != 0
    } else if c == b'n' as i8 || c == b'N' as i8 {
        false_0 != 0
    } else {
        false_0 != 0
    }
}

fn apply_permissions(read: bool, write: bool, execute: bool) -> i32 {
    let mut permission_value: i32 = 0;
    if read {
        permission_value += 4;
    }
    if write {
        permission_value += 2;
    }
    if execute {
        permission_value += 1;
    }
    if read as i32 != 0 && write as i32 != 0 && execute as i32 != 0 {
        return 100 + permission_value;
    } else if read as i32 != 0 && write as i32 != 0 {
        if permission_value == 6 {
            return 50 + permission_value;
        }
    } else if read as i32 != 0 && execute as i32 != 0 {
        return 30 + permission_value;
    } else if write as i32 != 0 && execute as i32 != 0 {
        return 20 + permission_value;
    } else if read {
        return 10 + permission_value;
    } else if write {
        return -10;
    } else if execute {
        return -20;
    }
    0
}

fn evaluate_conditions(cond1: bool, cond2: bool, cond3: bool, logic_op: i32) -> i32 {
    let result: bool;
    match logic_op {
        0 => {
            result = cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0;
            if result {
                100
            } else {
                if cond1 as i32 != 0 && cond2 as i32 != 0 {
                    return 50;
                }
                if cond1 as i32 != 0 && cond3 as i32 != 0 {
                    return 51;
                }
                if cond2 as i32 != 0 && cond3 as i32 != 0 {
                    return 52;
                }
                if cond1 {
                    return 10;
                }
                if cond2 {
                    return 11;
                }
                if cond3 {
                    return 12;
                }
                0
            }
        }
        1 => {
            result = cond1 as i32 != 0 || cond2 as i32 != 0 || cond3 as i32 != 0;
            if result {
                let mut count: i32 = 0;
                if cond1 {
                    count += 1;
                }
                if cond2 {
                    count += 1;
                }
                if cond3 {
                    count += 1;
                }
                return 100 + count;
            }
            0
        }
        2 => {
            result = (cond1 as i32 ^ cond2 as i32 ^ cond3 as i32) != 0;
            if result {
                if cond1 as i32 != 0 && !cond2 && !cond3 {
                    return 1;
                }
                if !cond1 && cond2 as i32 != 0 && !cond3 {
                    return 2;
                }
                if !cond1 && !cond2 && cond3 as i32 != 0 {
                    return 3;
                }
                if cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0 {
                    return 7;
                }
                return 90;
            }
            0
        }
        3 => {
            result = !(cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0);
            if result {
                if !cond1 && !cond2 && !cond3 {
                    return 200;
                }
                if !cond1 {
                    return 150;
                }
                if !cond2 {
                    return 151;
                }
                if !cond3 {
                    return 152;
                }
                return 100;
            }
            0
        }
        _ => -1,
    }
}

fn configure_flags(decisions: &[bool], count: usize) -> i32 {
    let mut flags: u32 = 0;
    let mut special_count: i32 = 0;
    let mut i: usize = 0;
    while i < count && i < 32 {
        if decisions[i] {
            flags |= 1u32 << i;
            special_count += 1;
        }
        i = i.wrapping_add(1);
    }
    if special_count == 0 {
        return 0;
    } else if special_count as usize == count {
        return 1000 + count as i32;
    } else if special_count == 1 {
        let mut i_0: usize = 0;
        while i_0 < count {
            if decisions[i_0] {
                return 100 + i_0 as i32;
            }
            i_0 = i_0.wrapping_add(1);
        }
    } else if special_count as usize == count.wrapping_sub(1) {
        let mut i_1: usize = 0;
        while i_1 < count {
            if !decisions[i_1] {
                return 200 + i_1 as i32;
            }
            i_1 = i_1.wrapping_add(1);
        }
    }
    let mut alternating: bool = true_0 != 0;
    let mut i_2: usize = 1;
    while i_2 < count {
        if decisions[i_2] as i32 == decisions[i_2.wrapping_sub(1)] as i32 {
            alternating = false_0 != 0;
            break;
        } else {
            i_2 = i_2.wrapping_add(1);
        }
    }
    if alternating {
        return 500 + special_count;
    }
    let mut max_consecutive: i32 = 0;
    let mut current_consecutive: i32 = 0;
    let mut i_3: usize = 0;
    while i_3 < count {
        if decisions[i_3] {
            current_consecutive += 1;
            if current_consecutive > max_consecutive {
                max_consecutive = current_consecutive;
            }
        } else {
            current_consecutive = 0;
        }
        i_3 = i_3.wrapping_add(1);
    }
    if max_consecutive >= 3 {
        return 300 + max_consecutive;
    }
    let _ = flags;
    special_count
}

fn validate_sequence_bytes_like_c(sequence: &[i8]) -> i32 {
    let len = sequence.len();
    if len == 0 {
        return 0;
    }

    // In the original C2Rust, `bools` aliases the same memory as `sequence` but as `bool*`.
    // Writes are `*bools[i] = parse_bool(sequence[i])`, which in practice stores 0/1 bytes.
    // Subsequent reads use `bools[i]`, i.e., those stored 0/1 bytes.
    let mut bool_bytes: Vec<u8> = Vec::with_capacity(len);
    for &b in sequence {
        bool_bytes.push(if parse_bool(b) { 1 } else { 0 });
    }

    if bool_bytes[0] == 0 {
        return -10;
    }
    if len > 1 && bool_bytes[len - 1] != 0 {
        return -11;
    }

    let mut consecutive: i32 = 1;
    for i in 1..len {
        if bool_bytes[i] == bool_bytes[i - 1] {
            consecutive += 1;
            if consecutive > 3 {
                return -12;
            }
        } else {
            consecutive = 1;
        }
    }

    let mut transitions: i32 = 0;
    for i in 1..len {
        if bool_bytes[i] != bool_bytes[i - 1] {
            transitions += 1;
        }
    }

    if len <= 3 {
        if transitions == 0 {
            return 1;
        }
        if transitions as usize == len.wrapping_sub(1) {
            return 2;
        }
        10 + transitions
    } else if len <= 10 {
        if (transitions as usize) < len.wrapping_div(3) {
            return 20;
        }
        if transitions as usize > len.wrapping_div(2) {
            return 30;
        }
        25
    } else {
        if transitions < 3 {
            return 40;
        }
        if transitions as usize > len.wrapping_sub(3) {
            return 50;
        }
        45
    }
}

pub(crate) fn process_decisions(
    decision_string: &mut [i8],
    length: usize,
    operation: i32,
    param: i32,
) -> i32 {
    if decision_string.is_empty() || length == 0 {
        return -1;
    }
    let length = length.min(decision_string.len());

    match operation {
        0 => {
            if length < 3 {
                return -2;
            }
            let read: bool = parse_bool(decision_string[0]);
            let write: bool = parse_bool(decision_string[1]);
            let execute: bool = parse_bool(decision_string[2]);
            apply_permissions(read, write, execute)
        }
        1 => {
            if length < 3 {
                return -2;
            }
            let cond1: bool = parse_bool(decision_string[0]);
            let cond2: bool = parse_bool(decision_string[1]);
            let cond3: bool = parse_bool(decision_string[2]);
            evaluate_conditions(cond1, cond2, cond3, param)
        }
        2 => {
            let mut decisions: [bool; 32] = [false; 32];
            let count: usize = if length < 32 { length } else { 32 };
            let mut i: usize = 0;
            while i < count {
                decisions[i] = parse_bool(decision_string[i]);
                i = i.wrapping_add(1);
            }
            configure_flags(&decisions, count)
        }
        3 => validate_sequence_bytes_like_c(&decision_string[..length]),
        _ => -3,
    }
}

fn main() {
    // Read all stdin; tests provide three lines: operation, param, decision string.
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();

    fn next_line(buf: &[u8], idx: &mut usize) -> Vec<u8> {
        if *idx >= buf.len() {
            return Vec::new();
        }
        let start = *idx;
        while *idx < buf.len() && buf[*idx] != b'\n' {
            *idx += 1;
        }
        let line = buf[start..*idx].to_vec();
        if *idx < buf.len() && buf[*idx] == b'\n' {
            *idx += 1;
        }
        line
    }

    let mut idx = 0usize;
    let op_line = next_line(&input, &mut idx);
    let param_line = next_line(&input, &mut idx);
    let decision_line = next_line(&input, &mut idx);

    let operation: i32 = String::from_utf8_lossy(&op_line).trim().parse().unwrap_or(0);
    let param: i32 = String::from_utf8_lossy(&param_line).trim().parse().unwrap_or(0);

    // Keep decision bytes exactly as provided on the line (no trimming).
    let mut decision_bytes: Vec<i8> = decision_line.into_iter().map(|b| b as i8).collect();
    let length = decision_bytes.len();

    let result = process_decisions(&mut decision_bytes, length, operation, param);
    print!("{result}\n");
}
```

**Entity:** validate_sequence_bytes_like_c (byte-as-bool encoding contract)

**States:** EmptySequence, StartsWithTrue, EndsWithFalseIfLen>1, NoRunLongerThan3, TransitionCountClassified

**Transitions:**
- EmptySequence -> returns 0
- NonEmpty -> reject if first decoded bool is false (returns -10)
- Len>1 -> reject if last decoded bool is true (returns -11)
- Decoded -> reject if any run length > 3 (returns -12)
- ValidStructure -> classified into result bands based on len and transitions

**Evidence:** validate_sequence_bytes_like_c: comment explains C2Rust aliasing where `sequence` memory is treated as `bool*` storing 0/1 bytes; validate_sequence_bytes_like_c: builds `bool_bytes` with `push(if parse_bool(b) { 1 } else { 0 })` (0/1 encoding); validate_sequence_bytes_like_c: `if bool_bytes[0] == 0 { return -10; }` requires first=true; validate_sequence_bytes_like_c: `if len > 1 && bool_bytes[len - 1] != 0 { return -11; }` requires last=false when len>1; validate_sequence_bytes_like_c: run-length check `if consecutive > 3 { return -12; }` encodes max-consecutive invariant; validate_sequence_bytes_like_c: later logic relies on `transitions` computed from adjacent inequality

**Implementation:** Introduce a validated wrapper like `struct BoolByteSeq(Vec<u8>);` (or `struct ValidatedSeq<'a>(&'a [u8]);`) with `TryFrom<&[i8]>` that performs `parse_bool` + structural checks (first/last/run-length). Only allow the classification logic to accept `ValidatedSeq`, eliminating the need for -10/-11/-12 paths and making the boolean-byte representation explicit in the type.

---

## Protocol Invariants

### 1. process_decisions operation protocol (Op-specific parsing + length requirements)

**Location**: `/data/test_case/main.rs:1-357`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The meaning of `decision_string[..length]` and `param` depends on the runtime value of `operation`. Each operation has distinct preconditions (notably minimum lengths and a restricted `logic_op` domain for op=1) and produces different interpretations of the same bytes. These requirements are currently enforced by runtime branching and sentinel return codes (-1/-2/-3), rather than by distinct types/entrypoints that make illegal combinations unrepresentable.

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

pub const true_0: i32 = 1;
pub const false_0: i32 = 0;

fn parse_bool(c: i8) -> bool {
    if c == b'y' as i8 || c == b'Y' as i8 {
        true_0 != 0
    } else if c == b'n' as i8 || c == b'N' as i8 {
        false_0 != 0
    } else {
        false_0 != 0
    }
}

fn apply_permissions(read: bool, write: bool, execute: bool) -> i32 {
    let mut permission_value: i32 = 0;
    if read {
        permission_value += 4;
    }
    if write {
        permission_value += 2;
    }
    if execute {
        permission_value += 1;
    }
    if read as i32 != 0 && write as i32 != 0 && execute as i32 != 0 {
        return 100 + permission_value;
    } else if read as i32 != 0 && write as i32 != 0 {
        if permission_value == 6 {
            return 50 + permission_value;
        }
    } else if read as i32 != 0 && execute as i32 != 0 {
        return 30 + permission_value;
    } else if write as i32 != 0 && execute as i32 != 0 {
        return 20 + permission_value;
    } else if read {
        return 10 + permission_value;
    } else if write {
        return -10;
    } else if execute {
        return -20;
    }
    0
}

fn evaluate_conditions(cond1: bool, cond2: bool, cond3: bool, logic_op: i32) -> i32 {
    let result: bool;
    match logic_op {
        0 => {
            result = cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0;
            if result {
                100
            } else {
                if cond1 as i32 != 0 && cond2 as i32 != 0 {
                    return 50;
                }
                if cond1 as i32 != 0 && cond3 as i32 != 0 {
                    return 51;
                }
                if cond2 as i32 != 0 && cond3 as i32 != 0 {
                    return 52;
                }
                if cond1 {
                    return 10;
                }
                if cond2 {
                    return 11;
                }
                if cond3 {
                    return 12;
                }
                0
            }
        }
        1 => {
            result = cond1 as i32 != 0 || cond2 as i32 != 0 || cond3 as i32 != 0;
            if result {
                let mut count: i32 = 0;
                if cond1 {
                    count += 1;
                }
                if cond2 {
                    count += 1;
                }
                if cond3 {
                    count += 1;
                }
                return 100 + count;
            }
            0
        }
        2 => {
            result = (cond1 as i32 ^ cond2 as i32 ^ cond3 as i32) != 0;
            if result {
                if cond1 as i32 != 0 && !cond2 && !cond3 {
                    return 1;
                }
                if !cond1 && cond2 as i32 != 0 && !cond3 {
                    return 2;
                }
                if !cond1 && !cond2 && cond3 as i32 != 0 {
                    return 3;
                }
                if cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0 {
                    return 7;
                }
                return 90;
            }
            0
        }
        3 => {
            result = !(cond1 as i32 != 0 && cond2 as i32 != 0 && cond3 as i32 != 0);
            if result {
                if !cond1 && !cond2 && !cond3 {
                    return 200;
                }
                if !cond1 {
                    return 150;
                }
                if !cond2 {
                    return 151;
                }
                if !cond3 {
                    return 152;
                }
                return 100;
            }
            0
        }
        _ => -1,
    }
}

fn configure_flags(decisions: &[bool], count: usize) -> i32 {
    let mut flags: u32 = 0;
    let mut special_count: i32 = 0;
    let mut i: usize = 0;
    while i < count && i < 32 {
        if decisions[i] {
            flags |= 1u32 << i;
            special_count += 1;
        }
        i = i.wrapping_add(1);
    }
    if special_count == 0 {
        return 0;
    } else if special_count as usize == count {
        return 1000 + count as i32;
    } else if special_count == 1 {
        let mut i_0: usize = 0;
        while i_0 < count {
            if decisions[i_0] {
                return 100 + i_0 as i32;
            }
            i_0 = i_0.wrapping_add(1);
        }
    } else if special_count as usize == count.wrapping_sub(1) {
        let mut i_1: usize = 0;
        while i_1 < count {
            if !decisions[i_1] {
                return 200 + i_1 as i32;
            }
            i_1 = i_1.wrapping_add(1);
        }
    }
    let mut alternating: bool = true_0 != 0;
    let mut i_2: usize = 1;
    while i_2 < count {
        if decisions[i_2] as i32 == decisions[i_2.wrapping_sub(1)] as i32 {
            alternating = false_0 != 0;
            break;
        } else {
            i_2 = i_2.wrapping_add(1);
        }
    }
    if alternating {
        return 500 + special_count;
    }
    let mut max_consecutive: i32 = 0;
    let mut current_consecutive: i32 = 0;
    let mut i_3: usize = 0;
    while i_3 < count {
        if decisions[i_3] {
            current_consecutive += 1;
            if current_consecutive > max_consecutive {
                max_consecutive = current_consecutive;
            }
        } else {
            current_consecutive = 0;
        }
        i_3 = i_3.wrapping_add(1);
    }
    if max_consecutive >= 3 {
        return 300 + max_consecutive;
    }
    let _ = flags;
    special_count
}

fn validate_sequence_bytes_like_c(sequence: &[i8]) -> i32 {
    let len = sequence.len();
    if len == 0 {
        return 0;
    }

    // In the original C2Rust, `bools` aliases the same memory as `sequence` but as `bool*`.
    // Writes are `*bools[i] = parse_bool(sequence[i])`, which in practice stores 0/1 bytes.
    // Subsequent reads use `bools[i]`, i.e., those stored 0/1 bytes.
    let mut bool_bytes: Vec<u8> = Vec::with_capacity(len);
    for &b in sequence {
        bool_bytes.push(if parse_bool(b) { 1 } else { 0 });
    }

    if bool_bytes[0] == 0 {
        return -10;
    }
    if len > 1 && bool_bytes[len - 1] != 0 {
        return -11;
    }

    let mut consecutive: i32 = 1;
    for i in 1..len {
        if bool_bytes[i] == bool_bytes[i - 1] {
            consecutive += 1;
            if consecutive > 3 {
                return -12;
            }
        } else {
            consecutive = 1;
        }
    }

    let mut transitions: i32 = 0;
    for i in 1..len {
        if bool_bytes[i] != bool_bytes[i - 1] {
            transitions += 1;
        }
    }

    if len <= 3 {
        if transitions == 0 {
            return 1;
        }
        if transitions as usize == len.wrapping_sub(1) {
            return 2;
        }
        10 + transitions
    } else if len <= 10 {
        if (transitions as usize) < len.wrapping_div(3) {
            return 20;
        }
        if transitions as usize > len.wrapping_div(2) {
            return 30;
        }
        25
    } else {
        if transitions < 3 {
            return 40;
        }
        if transitions as usize > len.wrapping_sub(3) {
            return 50;
        }
        45
    }
}

pub(crate) fn process_decisions(
    decision_string: &mut [i8],
    length: usize,
    operation: i32,
    param: i32,
) -> i32 {
    if decision_string.is_empty() || length == 0 {
        return -1;
    }
    let length = length.min(decision_string.len());

    match operation {
        0 => {
            if length < 3 {
                return -2;
            }
            let read: bool = parse_bool(decision_string[0]);
            let write: bool = parse_bool(decision_string[1]);
            let execute: bool = parse_bool(decision_string[2]);
            apply_permissions(read, write, execute)
        }
        1 => {
            if length < 3 {
                return -2;
            }
            let cond1: bool = parse_bool(decision_string[0]);
            let cond2: bool = parse_bool(decision_string[1]);
            let cond3: bool = parse_bool(decision_string[2]);
            evaluate_conditions(cond1, cond2, cond3, param)
        }
        2 => {
            let mut decisions: [bool; 32] = [false; 32];
            let count: usize = if length < 32 { length } else { 32 };
            let mut i: usize = 0;
            while i < count {
                decisions[i] = parse_bool(decision_string[i]);
                i = i.wrapping_add(1);
            }
            configure_flags(&decisions, count)
        }
        3 => validate_sequence_bytes_like_c(&decision_string[..length]),
        _ => -3,
    }
}

fn main() {
    // Read all stdin; tests provide three lines: operation, param, decision string.
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();

    fn next_line(buf: &[u8], idx: &mut usize) -> Vec<u8> {
        if *idx >= buf.len() {
            return Vec::new();
        }
        let start = *idx;
        while *idx < buf.len() && buf[*idx] != b'\n' {
            *idx += 1;
        }
        let line = buf[start..*idx].to_vec();
        if *idx < buf.len() && buf[*idx] == b'\n' {
            *idx += 1;
        }
        line
    }

    let mut idx = 0usize;
    let op_line = next_line(&input, &mut idx);
    let param_line = next_line(&input, &mut idx);
    let decision_line = next_line(&input, &mut idx);

    let operation: i32 = String::from_utf8_lossy(&op_line).trim().parse().unwrap_or(0);
    let param: i32 = String::from_utf8_lossy(&param_line).trim().parse().unwrap_or(0);

    // Keep decision bytes exactly as provided on the line (no trimming).
    let mut decision_bytes: Vec<i8> = decision_line.into_iter().map(|b| b as i8).collect();
    let length = decision_bytes.len();

    let result = process_decisions(&mut decision_bytes, length, operation, param);
    print!("{result}\n");
}
```

**Entity:** process_decisions (operation/param/decision_string contract)

**States:** InvalidInput (empty slice or length==0), Op0_Permissions (needs >=3 decision bytes), Op1_Conditions (needs >=3 decision bytes + logic_op domain), Op2_Flags (uses up to 32 decision bytes), Op3_ValidateSequence (uses provided length prefix), UnknownOperation

**Transitions:**
- InvalidInput -> (return -1) via initial guard in process_decisions()
- Op0_Permissions -> (return -2) if length < 3; otherwise parse_bool[0..3] then apply_permissions()
- Op1_Conditions -> (return -2) if length < 3; otherwise parse_bool[0..3] then evaluate_conditions(..., param)
- Op2_Flags -> fill decisions[0..min(length,32)] then configure_flags(...)
- Op3_ValidateSequence -> validate_sequence_bytes_like_c(&decision_string[..length])
- UnknownOperation -> (return -3) via match default arm

**Evidence:** process_decisions: signature includes `operation: i32` and `param: i32` which are interpreted differently per match arm; process_decisions: `if decision_string.is_empty() || length == 0 { return -1; }` encodes an input-validity state; process_decisions op=0/op=1: `if length < 3 { return -2; }` minimum-length precondition; process_decisions op=1: calls `evaluate_conditions(cond1, cond2, cond3, param)` where `param` is used as `logic_op`; evaluate_conditions: `match logic_op { 0 | 1 | 2 | 3 => ..., _ => -1 }` reveals restricted domain for `param` in op=1; process_decisions op=2: `let mut decisions: [bool; 32]` and `count: usize = if length < 32 { length } else { 32 };` shows implicit max-length/cap; process_decisions: `_ => -3` indicates an operation-domain invariant (only 0..=3 meaningful)

**Implementation:** Replace `(operation: i32, param: i32)` with an enum describing the operation and its typed parameters, e.g. `enum Operation { Permissions, Conditions{ logic: LogicOp }, Flags, ValidateSequence }` where `LogicOp` is a Rust enum for 0..=3. Split into typed entrypoints or a single `process(op: Operation, decisions: DecisionsInput)`; encode length requirements via newtypes like `struct AtLeast3<'a>(&'a [i8]);` constructed with `TryFrom<&[i8]>` so op0/op1 cannot be called without >=3 bytes.

---

