# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 1

## Temporal Ordering Invariants

### 2. Array initialization-before-use protocol (Uninitialized/Zeroed -> Seeded -> Transformed)

**Location**: `/data/test_case/main.rs:1-91`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The program implicitly assumes a temporal ordering on the thread-local ARRAY contents: it starts as an all-zero default, must be seeded with PRNG output before running the expensive transform loop, and only then is it reduced (xor_result). This ordering is maintained only by the sequence in main(); the type system does not prevent calling `perform_expensive_operations()` or the final XOR reduction on the default-zero array, nor does it encode that seeding has occurred.

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
#![feature(as_array_of_cells)]

use std::cell::Cell;

pub const __INT_MAX__: i32 = 2147483647;
pub const UINT_MAX: u32 = (__INT_MAX__ as u32).wrapping_mul(2).wrapping_add(1);
pub const ARRAY_SIZE: usize = 256 * 1024;
pub const ITERATIONS: i32 = 2000;

thread_local! {
    static ARRAY: Cell<[i32; ARRAY_SIZE]> = const { Cell::new([0; ARRAY_SIZE]) };
}

fn perform_expensive_operations() {
    ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        for i in 0..ARRAY_SIZE {
            let mut x = cells[i].get();
            for _ in 0..100 {
                x = x.wrapping_mul(3).wrapping_add(7);
                x ^= x >> 3;
                x = x.wrapping_sub(x << 1);
                x = x / 2 + x % 7;
            }
            cells[i].set(x);
        }
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <seed>", args.get(0).map(String::as_str).unwrap_or(""));
        std::process::exit(1);
    }

    let seed_str = &args[1];
    let temp_seed: u64 = match seed_str.parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid seed: '{}'", seed_str);
            std::process::exit(1);
        }
    };

    if temp_seed > UINT_MAX as u64 {
        eprintln!("Invalid seed: '{}'", seed_str);
        std::process::exit(1);
    }

    // Deterministic PRNG to replace C rand()/srand() without libc/FFI.
    // LCG parameters are common and sufficient for this benchmark-style program.
    let mut state: u32 = temp_seed as u32;
    let mut next_rand_i32 = || -> i32 {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        // Produce a 31-bit non-negative value like many rand() implementations.
        ((state >> 1) as i32) & 0x7fff_ffff
    };

    ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        for i in 0..ARRAY_SIZE {
            cells[i].set(next_rand_i32());
        }
    });

    for _ in 0..ITERATIONS {
        perform_expensive_operations();
    }

    let xor_result: i32 = ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        let mut acc: i32 = 0;
        for i in 0..ARRAY_SIZE {
            acc ^= cells[i].get();
        }
        acc
    });

    println!("{xor_result}");
}
```

**Entity:** thread_local `ARRAY: Cell<[i32; ARRAY_SIZE]>` usage

**States:** ZeroedDefault, SeededWithPRNG, TransformedAfterIterations

**Transitions:**
- ZeroedDefault -> SeededWithPRNG via the `for i in 0..ARRAY_SIZE { cells[i].set(next_rand_i32()); }` loop in main()
- SeededWithPRNG -> TransformedAfterIterations via `for _ in 0..ITERATIONS { perform_expensive_operations(); }`

**Evidence:** thread_local!: `static ARRAY: Cell<[i32; ARRAY_SIZE]> = const { Cell::new([0; ARRAY_SIZE]) };` establishes an implicit 'zeroed' starting state; main(): seeding step: `ARRAY.with(|arr| { ... for i in 0..ARRAY_SIZE { cells[i].set(next_rand_i32()); } })`; main(): transformation step: `for _ in 0..ITERATIONS { perform_expensive_operations(); }`; main(): reduction step reads contents: `acc ^= cells[i].get();` to compute `xor_result`

**Implementation:** Wrap the TLS array in an API that returns typed handles, e.g. `struct ArrayHandle<S>(PhantomData<S>);` with `Zeroed`, `Seeded`, `Transformed` states. Provide `fn seed(self, seed: SeedU32) -> ArrayHandle<Seeded>`, `fn transform(self, iters: NonZeroU32) -> ArrayHandle<Transformed>`, and `fn xor(&self) -> i32` only on `ArrayHandle<Transformed>` (or at least on `Seeded`). Internally the handle uses `ARRAY.with(...)` but callers cannot skip steps.

---

## Protocol Invariants

### 1. Seed validation protocol (RawArg -> ParsedU64 -> InRangeU32)

**Location**: `/data/test_case/main.rs:1-91`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The program relies on a multi-step validation protocol for the seed: (1) there must be exactly one CLI argument; (2) it must parse as u64; (3) it must be <= UINT_MAX so it can be safely narrowed to u32. These requirements are enforced with runtime branching and process::exit(), and only then is the seed converted to u32 to initialize the PRNG state. The type system does not distinguish an unchecked seed string from a validated seed usable to initialize the PRNG.

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
#![feature(as_array_of_cells)]

use std::cell::Cell;

pub const __INT_MAX__: i32 = 2147483647;
pub const UINT_MAX: u32 = (__INT_MAX__ as u32).wrapping_mul(2).wrapping_add(1);
pub const ARRAY_SIZE: usize = 256 * 1024;
pub const ITERATIONS: i32 = 2000;

thread_local! {
    static ARRAY: Cell<[i32; ARRAY_SIZE]> = const { Cell::new([0; ARRAY_SIZE]) };
}

fn perform_expensive_operations() {
    ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        for i in 0..ARRAY_SIZE {
            let mut x = cells[i].get();
            for _ in 0..100 {
                x = x.wrapping_mul(3).wrapping_add(7);
                x ^= x >> 3;
                x = x.wrapping_sub(x << 1);
                x = x / 2 + x % 7;
            }
            cells[i].set(x);
        }
    });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <seed>", args.get(0).map(String::as_str).unwrap_or(""));
        std::process::exit(1);
    }

    let seed_str = &args[1];
    let temp_seed: u64 = match seed_str.parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid seed: '{}'", seed_str);
            std::process::exit(1);
        }
    };

    if temp_seed > UINT_MAX as u64 {
        eprintln!("Invalid seed: '{}'", seed_str);
        std::process::exit(1);
    }

    // Deterministic PRNG to replace C rand()/srand() without libc/FFI.
    // LCG parameters are common and sufficient for this benchmark-style program.
    let mut state: u32 = temp_seed as u32;
    let mut next_rand_i32 = || -> i32 {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        // Produce a 31-bit non-negative value like many rand() implementations.
        ((state >> 1) as i32) & 0x7fff_ffff
    };

    ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        for i in 0..ARRAY_SIZE {
            cells[i].set(next_rand_i32());
        }
    });

    for _ in 0..ITERATIONS {
        perform_expensive_operations();
    }

    let xor_result: i32 = ARRAY.with(|arr| {
        let cells = arr.as_array_of_cells();
        let mut acc: i32 = 0;
        for i in 0..ARRAY_SIZE {
            acc ^= cells[i].get();
        }
        acc
    });

    println!("{xor_result}");
}
```

**Entity:** Program argument `seed` (parsed from args[1])

**States:** RawString, ParsedU64, InRangeForU32

**Transitions:**
- RawString -> ParsedU64 via seed_str.parse::<u64>()
- ParsedU64 -> InRangeForU32 via `if temp_seed > UINT_MAX as u64 { exit(1) }` then `temp_seed as u32`

**Evidence:** main(): `if args.len() != 2 { eprintln!("Usage: {} <seed>", ...); std::process::exit(1); }` enforces presence/arity at runtime; main(): `match seed_str.parse::<u64>() { ... Err(_) => { eprintln!("Invalid seed: '{}'", seed_str); std::process::exit(1); } }` enforces parseability at runtime; main(): `if temp_seed > UINT_MAX as u64 { eprintln!("Invalid seed: '{}'", seed_str); std::process::exit(1); }` enforces range constraint at runtime; main(): `let mut state: u32 = temp_seed as u32;` relies on prior range check to make narrowing sound

**Implementation:** Introduce `struct SeedU32(u32); impl TryFrom<&str> for SeedU32` that performs parsing + range check, returning Result instead of exiting. Then PRNG initialization takes `SeedU32` (or `NonZeroU32` if desired), eliminating the need to remember to check `<= UINT_MAX` before casting.

---

