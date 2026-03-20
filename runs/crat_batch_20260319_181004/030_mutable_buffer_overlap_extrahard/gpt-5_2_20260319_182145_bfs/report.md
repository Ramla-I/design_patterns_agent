# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## State Machine Invariants

### 2. Parsed-prefix validity protocol for `data[..count]` (Count tracks initialized elements)

**Location**: `/data/test_case/main.rs:1-51`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `data` is a fixed-size buffer whose valid/initialized portion is tracked by the runtime variable `count`. The code relies on the invariant that only `data[..count]` contains meaningful parsed integers, while `data[count..]` is not part of the logical dataset. This is enforced manually by incrementing `count` only after a successful parse and then slicing with `..count` when calling `driver`. The type system does not encode that `count` is the length of the valid prefix, nor that `count <= 100` always holds (it is maintained by runtime checks).

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

fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32]) {
    let len = out.len().min(mul1.len()).min(mul2.len()).min(add.len());
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

fn driver(out: &mut [i32]) {
    // Equivalent to: fma_array(out, out, out, out, len)
    // Compute using the original values (as if reading from the same buffer).
    let original = out.to_vec();
    fma_array(out, &original, &original, &original);

    for &v in out.iter() {
        println!("{0}", v);
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut data = [0i32; 100];
    let mut count = 0usize;

    for token in input.split_whitespace() {
        if count >= 100 {
            break;
        }
        if let Ok(v) = token.parse::<i32>() {
            data[count] = v;
            count += 1;
        } else {
            break;
        }
    }

    driver(&mut data[..count]);
}
```

**Entity:** main() input parsing into `data` and `count`

**States:** UninitializedTailPresent, ValidPrefixOnly

**Transitions:**
- UninitializedTailPresent -> ValidPrefixOnly via `driver(&mut data[..count])` using `count` as the logical length

**Evidence:** main(): `let mut data = [0i32; 100]; let mut count = 0usize;` uses a separate length tracker; main(): `if count >= 100 { break; }` runtime bound check to maintain `count <= 100`; main(): `if let Ok(v) = token.parse::<i32>() { data[count] = v; count += 1; } else { break; }` ensures `count` advances only when an element is initialized/valid; main(): `driver(&mut data[..count]);` relies on the prefix-length invariant

**Implementation:** Replace `(data, count)` with a single type that maintains the invariant, e.g. `struct ParsedPrefix<const N: usize> { buf: [i32; N], len: usize }` with methods `push(i32) -> Result<(), Full>` and `as_mut_slice(&mut self) -> &mut [i32]`. Then `driver(parsed.as_mut_slice())` cannot accidentally use an incorrect `count` or an out-of-bounds slice.

---

## Protocol Invariants

### 1. Aliasing protocol for FMA inputs (in-place vs out-of-place semantics)

**Location**: `/data/test_case/main.rs:1-51`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The computation is intended to behave as if `mul1`, `mul2`, and `add` were read from the original input values even when they conceptually refer to the same buffer as `out` (in-place operation). `fma_array` itself does not enforce or document any aliasing constraints; it will produce different results if the input slices overlap with `out` in ways that cause reads to observe already-updated `out[i]`. `driver` relies on an implicit protocol: if inputs alias `out`, you must snapshot the original values first (out-of-place) to preserve the intended semantics. This is enforced only by convention and the explicit `to_vec()` copy in `driver`, not by types.

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

fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32]) {
    let len = out.len().min(mul1.len()).min(mul2.len()).min(add.len());
    for i in 0..len {
        out[i] = mul1[i] * mul2[i] + add[i];
    }
}

fn driver(out: &mut [i32]) {
    // Equivalent to: fma_array(out, out, out, out, len)
    // Compute using the original values (as if reading from the same buffer).
    let original = out.to_vec();
    fma_array(out, &original, &original, &original);

    for &v in out.iter() {
        println!("{0}", v);
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut data = [0i32; 100];
    let mut count = 0usize;

    for token in input.split_whitespace() {
        if count >= 100 {
            break;
        }
        if let Ok(v) = token.parse::<i32>() {
            data[count] = v;
            count += 1;
        } else {
            break;
        }
    }

    driver(&mut data[..count]);
}
```

**Entity:** driver(out: &mut [i32]) / fma_array(out, mul1, mul2, add)

**States:** NonAliasedInputs, AliasedInputsNeedingSnapshot

**Transitions:**
- AliasedInputsNeedingSnapshot -> NonAliasedInputs via `let original = out.to_vec()` in driver()

**Evidence:** fn fma_array(out: &mut [i32], mul1: &[i32], mul2: &[i32], add: &[i32]) takes independent slices with no aliasing/overlap contract; driver(): comment: "Equivalent to: fma_array(out, out, out, out, len)" and "Compute using the original values"; driver(): `let original = out.to_vec(); fma_array(out, &original, &original, &original);` implements the snapshot requirement

**Implementation:** Introduce a wrapper representing a snapshot/readonly view, e.g. `struct Snapshot(Vec<i32>);` and change the API to make in-place semantics explicit: `fn fma_in_place(out: &mut [i32], input: SnapshotRef<'_>)` or provide two entry points `fma_out_of_place(out, a,b,c)` and `fma_in_place(out)` where the in-place version internally snapshots. This makes it impossible to accidentally call the out-of-place kernel with aliased slices while expecting snapshot semantics.

---

