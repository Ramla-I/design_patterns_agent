# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 2

## Temporal Ordering Invariants

### 2. C RNG global-state protocol (Seeded -> Random draws)

**Location**: `/data/test_case/lib.rs:1-66`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The C RNG functions `rand()`/`srand()` have an implicit global state: `srand(seed)` must be called before `rand()` draws if deterministic seeding is required. The code relies on the ordering inside `long_exec` (call `srand` then fill the array with `rand()`), but nothing in the type system prevents other code from calling `rand()` without a prior `srand`, nor does it encapsulate the seeded capability as a token. This is a latent invariant about correct ordering at the FFI boundary.

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

// === long.rs ===
extern "C" {
    fn rand() -> i32;
    fn srand(__seed: u32);
}

pub const ARRAY_SIZE: i32 = 256 * 1024;
pub const ITERATIONS: i32 = 2000;

thread_local! {
    static array: std::cell::Cell<[i32; 262144]> = const { std::cell::Cell::new([0; 262144]) };
}

pub(crate) fn perform_expensive_operations() {
    array.with(|cell| {
        let cells = cell.as_array_of_cells();
        for i in 0..(ARRAY_SIZE as usize) {
            let mut x = cells[i].get();
            for _ in 0..100 {
                x = x * 3 + 7;
                x ^= x >> 3;
                x -= x << 1;
                x = x / 2 + x % 7;
            }
            cells[i].set(x);
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn long_exec(seed: u32) {
    srand(seed);

    array.with(|cell| {
        let cells = cell.as_array_of_cells();
        for i in 0..(ARRAY_SIZE as usize) {
            cells[i].set(rand());
        }
    });

    for _ in 0..ITERATIONS {
        perform_expensive_operations();
    }

    let xor_result = array.with(|cell| {
        let cells = cell.as_array_of_cells();
        let mut acc: i32 = 0;
        for i in 0..(ARRAY_SIZE as usize) {
            acc ^= cells[i].get();
        }
        acc
    });

    println!("{xor_result}");
}
```

**Entity:** unsafe extern "C" fn long_exec(seed: u32) / FFI boundary to rand/srand

**States:** Unseeded C RNG, Seeded C RNG

**Transitions:**
- Unseeded C RNG -> Seeded C RNG via long_exec(): srand(seed)

**Evidence:** extern "C" { fn rand() -> i32; fn srand(__seed: u32); } indicates reliance on external global RNG state; long_exec(seed): calls srand(seed); then immediately uses rand() in a loop: cells[i].set(rand())

**Implementation:** Introduce a safe wrapper that returns a seeding token/capability: `struct SeededRng(PhantomData<*mut ()>); fn seed(seed: u32) -> SeededRng { unsafe { srand(seed) }; SeededRng(...) }` and require `&SeededRng` to call `fn rand_i32(_: &SeededRng) -> i32`. Then `long_exec` must obtain the capability before filling the array, and other code cannot call `rand_i32` without it.

---

## Protocol Invariants

### 1. Thread-local array initialization & usage protocol (Uninitialized/Unknown -> Seeded -> Mutated -> Reduced)

**Location**: `/data/test_case/lib.rs:1-66`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The thread-local `array` has an implicit workflow: it must be seeded (filled with `rand()` values after `srand(seed)`) before running expensive operations, and the final XOR reduction is intended to happen after all iterations. None of these phases are represented in the type system: `perform_expensive_operations()` can be called at any time (including before seeding), and nothing prevents computing the XOR at any point or multiple times. The correctness/meaningfulness of the output relies on callers following this temporal protocol.

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

// === long.rs ===
extern "C" {
    fn rand() -> i32;
    fn srand(__seed: u32);
}

pub const ARRAY_SIZE: i32 = 256 * 1024;
pub const ITERATIONS: i32 = 2000;

thread_local! {
    static array: std::cell::Cell<[i32; 262144]> = const { std::cell::Cell::new([0; 262144]) };
}

pub(crate) fn perform_expensive_operations() {
    array.with(|cell| {
        let cells = cell.as_array_of_cells();
        for i in 0..(ARRAY_SIZE as usize) {
            let mut x = cells[i].get();
            for _ in 0..100 {
                x = x * 3 + 7;
                x ^= x >> 3;
                x -= x << 1;
                x = x / 2 + x % 7;
            }
            cells[i].set(x);
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn long_exec(seed: u32) {
    srand(seed);

    array.with(|cell| {
        let cells = cell.as_array_of_cells();
        for i in 0..(ARRAY_SIZE as usize) {
            cells[i].set(rand());
        }
    });

    for _ in 0..ITERATIONS {
        perform_expensive_operations();
    }

    let xor_result = array.with(|cell| {
        let cells = cell.as_array_of_cells();
        let mut acc: i32 = 0;
        for i in 0..(ARRAY_SIZE as usize) {
            acc ^= cells[i].get();
        }
        acc
    });

    println!("{xor_result}");
}
```

**Entity:** thread_local! static array: Cell<[i32; 262144]>

**States:** Uninitialized/Unknown contents, Seeded (filled by rand()), Mutated (after perform_expensive_operations iterations), Reduced (xor computed/consumed)

**Transitions:**
- Uninitialized/Unknown contents -> Seeded (filled by rand()) via long_exec(): array.with(...) { cells[i].set(rand()) }
- Seeded -> Mutated via perform_expensive_operations() (repeated ITERATIONS times in long_exec)
- Mutated -> Reduced via long_exec(): xor_result = array.with(...) { acc ^= cells[i].get() }

**Evidence:** thread_local! static array: Cell<[i32; 262144]> = ... Cell::new([0; 262144]) (shared mutable state whose contents represent the phase); long_exec(seed): calls srand(seed) then fills array with rand(): cells[i].set(rand()); perform_expensive_operations(): reads and writes array elements via cells[i].get()/set() without any check that seeding happened; long_exec(): for _ in 0..ITERATIONS { perform_expensive_operations(); } then computes xor_result by reading all cells

**Implementation:** Wrap the thread-local storage behind a typed API that encodes phases: e.g., `struct ArrayState<S>(PhantomData<S>);` with `Unseeded`, `Seeded`, `Mutated` markers. Expose `fn seed(self, seed: u32) -> ArrayState<Seeded>`, `fn step(self) -> ArrayState<Mutated>` (or `fn run(self, iters: NonZeroU32) -> ArrayState<Mutated>`), and `fn reduce(self) -> (i32, ArrayState<...>)` so that `perform_expensive_operations` is only callable on `Seeded/Mutated` and the reduction is sequenced after mutation.

---

