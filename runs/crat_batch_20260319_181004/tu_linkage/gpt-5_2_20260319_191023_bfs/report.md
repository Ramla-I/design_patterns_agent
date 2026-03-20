# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 3
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## State Machine Invariants

### 1. Program execution cursor protocol (InBounds / Exhausted / InvalidLength)

**Location**: `/data/test_case/main.rs:1-25`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Program has an implicit execution state determined by (ip, n) relative to code. `fetch()` is only meaningful while `ip < n` (InBounds); once `ip >= n` it becomes Exhausted and always returns None. Additionally, correctness implicitly requires `n <= code.len()`: if `n` exceeds the slice length, `fetch()` may return None early due to `code.get(self.ip)?` even when `ip < n`, conflating 'end of program' with 'invalid program length'. None of these states/constraints are encoded in the type system; they are enforced by runtime checks and `Option` short-circuiting.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct VM, 3 free function(s); 15 free function(s)

}

#[derive(Clone, Copy)]
struct Program<'a> {
    code: &'a [i32],
    n: usize,
    ip: usize,
}

impl<'a> Program<'a> {
    fn new(code: &'a [i32], n: usize) -> Self {
        Self { code, n, ip: 0 }
    }

    fn fetch(&mut self) -> Option<i32> {
        if self.ip >= self.n {
            return None;
        }
        let v = *self.code.get(self.ip)?;
        self.ip = self.ip.wrapping_add(1);
        Some(v)
    }
}

```

**Entity:** Program<'a>

**States:** InBounds (ip < n), Exhausted (ip >= n), InvalidLength (n > code.len())

**Transitions:**
- InBounds -> InBounds via fetch() when ip + 1 < n
- InBounds -> Exhausted via fetch() when ip becomes >= n
- InvalidLength -> (observed as premature None) via fetch() when code.get(ip) returns None despite ip < n

**Evidence:** field `n: usize` and `ip: usize` encode remaining length and cursor position; Program::new(code, n) stores `n` without validating it against `code.len()`; Program::fetch(): `if self.ip >= self.n { return None; }` defines the Exhausted state boundary; Program::fetch(): `let v = *self.code.get(self.ip)?;` can return None if `ip` is out of bounds for `code`, independent of `n`; Program::fetch(): `self.ip = self.ip.wrapping_add(1);` advances state (cursor) and introduces wraparound behavior not represented in the type

**Implementation:** Validate `n <= code.len()` at construction by changing `Program::new` to return `Result<Program, Error>` or by accepting a `ProgramLen` newtype created via `ProgramLen::new(n, code_len) -> Option/Result`. Alternatively, remove `n` and use `code.len()` as the bound. If wraparound is undesired, replace `wrapping_add(1)` with checked/saturating increment (or make `ip` a newtype that cannot overflow in this context).

---

### 2. VM execution protocol (Configured/Reset -> Running -> Halted/Finished)

**Location**: `/data/test_case/main.rs:1-10`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: VM appears to model an interpreter with runtime state split across `stack`, `trace`, and a `steps` counter. This implies an execution protocol where the VM is reset/initialized (empty stack/trace, steps at baseline), then transitions through a running state as steps increase and stack/trace mutate, and eventually reaches a halted/finished state. None of these states (e.g., 'ready to step', 'has valid stack', 'has started executing', 'finished') are represented in the type system; the struct is always constructible (via `Default`) and clonable regardless of whether its internal fields represent a valid VM state for the next operation. As a result, any invariants like "stack must be non-empty before pop", "steps must be non-negative", or "trace corresponds to executed steps" must be enforced by runtime checks elsewhere (not shown here) rather than by method availability tied to state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Program, impl Program < 'a > (2 methods); 15 free function(s)

}

#[derive(Clone, Default)]
struct VM {
    stack: Vec<i32>,
    trace: Vec<i32>,
    steps: i32,
}

```

**Entity:** VM

**States:** Empty/Reset, Running, Halted/Finished

**Transitions:**
- Empty/Reset -> Running via execution/step methods (not shown in snippet)
- Running -> Halted/Finished via program completion/halting condition (not shown in snippet)
- Any -> Empty/Reset via reset/reinitialize (not shown in snippet)

**Evidence:** struct VM { stack: Vec<i32>, trace: Vec<i32>, steps: i32 } — multiple fields encode evolving runtime execution state; #[derive(Default)] on VM — allows constructing a VM in a baseline/empty state without proving it is configured/ready for execution; steps: i32 — signed counter suggests an implicit invariant like 'steps >= 0' and monotonic progression during execution, but it is not enforced by types

**Implementation:** Introduce `VM<S>` with zero-sized state types like `Reset`, `Running`, `Halted`. Provide constructors such as `VM<Reset>::new()` / `Default for VM<Reset>`. Make stepping/execution methods available only on `VM<Running>` (or have `start(self, program) -> VM<Running>`). If there is a minimum stack requirement for some ops, use newtypes like `NonEmptyStack` or split operations into safe variants that require capabilities/tokens produced by successful checks.

---

### 3. Program instruction-stream protocol (ip in-bounds / exhausted) and code-length agreement

**Location**: `/data/test_case/main.rs:1-428`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Program encodes a cursor over an instruction stream using (code, n, ip). Correctness relies on the latent invariant that `n <= code.len()` and that `ip` stays within `[0, n]`. `fetch()` enforces this at runtime by returning None when `ip >= n` (and also via `code.get(ip)?`), and callers interpret `None` as end-of-program or as an error depending on opcode context. The type system does not prevent constructing a Program with an inconsistent `n` (larger than `code.len()`), nor does it distinguish 'exhausted' programs from 'ready' programs to prevent opcode handlers from assuming an immediate operand exists.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct VM, 3 free function(s); struct Program, impl Program < 'a > (2 methods)

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
use std::io::{self, Write};

mod lib {
    pub fn target(code: i32) -> i32 {
        if code < 0 {
            return 7;
        }
        let m: i32 = code % 10;
        if m == 0 {
            return 0;
        }
        if m <= 3 {
            return 1;
        }
        if m <= 6 {
            return 2;
        }
        if m == 7 {
            return 3;
        }
        4
    }
}

mod a {
    use super::Cell;

    #[inline]
    fn a_bias_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f((x ^ 0x55) + 7)
    }

    thread_local! {
        static STATE_A: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        if code < 0 {
            return STATE_A.with(|s| if (s.get() & 1) != 0 { 6 } else { 5 });
        }
        STATE_A.with(|s| s.set(s.get() ^ (code << 1)));
        let k: i32 = STATE_A.with(|s| ((code >> 2) ^ s.get()) & 7);
        match k {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 1,
            4 => 3,
            5 | 6 => 5,
            _ => 7,
        }
    }

    #[inline]
    fn wrap(x: i32) -> i32 {
        target(x - 5)
    }

    pub fn call_a_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = wrap(a);
        let c: i32 = target(b ^ 3);
        let d: i32 = a_bias_call(target, b);
        a ^ (b << 1) ^ (c << 2) ^ (d << 3)
    }

    pub fn process_a_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: usize = 0;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut j: i32 = 0;
            while j < 3 {
                let t: i32 = target(v + j);
                if (t & 1) == 0 {
                    acc = (acc as u64).wrapping_add(t as u64) as usize;
                } else {
                    acc = (acc as u64 ^ ((t << j) as u64)) as usize;
                    if t == 5 {
                        break;
                    }
                }
                j += 1;
            }
            i = i.wrapping_add(1);
        }
        if (acc as u64) > 0x7fffffff {
            acc = 0x7fffffff;
        }
        if (acc as u64) < 0xffffffff80000000u64 {
            acc = 0xffffffff80000000usize;
        }
        acc as i32
    }
}

mod b {
    use super::Cell;

    #[inline]
    fn b_twist_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f(((x + 9) ^ 0x2222) - 17)
    }

    thread_local! {
        static FLIPFLOP: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        let ff: i32 = FLIPFLOP.with(|f| {
            f.set(f.get() ^ 1);
            f.get()
        });

        if code < 0 {
            return if ff != 0 { 2 } else { 6 };
        }
        let mask: i32 = if ff != 0 { 0x7f } else { 0x1f };
        let z: i32 = (code ^ mask) % 8;
        if z == 0 || z == 7 {
            return 4;
        }
        if z == 1 || z == 2 {
            return 3;
        }
        if z == 3 {
            return 1;
        }
        if z == 4 {
            return 0;
        }
        if z == 5 {
            return 5;
        }
        7
    }

    #[inline]
    fn w2(x: i32) -> i32 {
        target(x + 9)
    }

    pub fn call_b_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = w2(a);
        let c: i32 = b_twist_call(target, a);
        let d: i32 = target(c ^ x);
        (a << 1) ^ (b << 2) ^ (c << 3) ^ (d << 4)
    }

    pub fn process_b_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: i32 = 1;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut iter: i32 = 0;
            loop {
                iter += 1;
                if iter > 4 {
                    break;
                }
                let t: i32 = target(v - iter);
                if t == 6 {
                    acc -= t;
                    break;
                } else {
                    if t == 3 {
                        continue;
                    }
                    acc = (acc * 3) ^ t;
                }
            }
            i = i.wrapping_add(1);
        }
        acc
    }
}

#[derive(Clone, Default)]
struct VM {
    stack: Vec<i32>,
    trace: Vec<i32>,
    steps: i32,
}

#[derive(Clone, Copy)]
struct Program<'a> {
    code: &'a [i32],
    n: usize,
    ip: usize,
}

impl<'a> Program<'a> {
    fn new(code: &'a [i32], n: usize) -> Self {
        Self { code, n, ip: 0 }
    }

    fn fetch(&mut self) -> Option<i32> {
        if self.ip >= self.n {
            return None;
        }
        let v = *self.code.get(self.ip)?;
        self.ip = self.ip.wrapping_add(1);
        Some(v)
    }
}

#[inline]
fn classify(impl_id: i32, x: i32) -> i32 {
    if impl_id == 0 {
        return a::call_a_once(x);
    }
    if impl_id == 1 {
        return b::call_b_once(x + 1);
    }
    lib::target(lib::target(x + 1))
}

fn process_stream(impl_id: i32, buf: &[i32], n: usize) -> i32 {
    if impl_id == 0 {
        return a::process_a_stream(buf, n);
    }
    if impl_id == 1 {
        return b::process_b_stream(buf, n);
    }
    let mut acc: i32 = 0;
    let mut i: usize = 0;
    while i < n {
        let t: i32 = lib::target(buf[i]);
        if (t & 1) == 0 {
            acc += t * 2;
        } else {
            acc ^= t + 7;
        }
        i = i.wrapping_add(1);
    }
    acc
}

#[inline]
fn vm_trace(vm: &mut VM, t: i32) {
    vm.trace.push(t);
}

#[inline]
fn iv_peek(stack: &[i32], def: i32) -> i32 {
    stack.last().copied().unwrap_or(def)
}

fn run_engine_internal(impl_id: i32, code: &[i32], n: usize, vm: &mut VM) -> i32 {
    let mut p: Program<'_> = Program::new(code, n);
    while let Some(op) = p.fetch() {
        vm.steps += 1;
        match op {
            0 => {
                let Some(imm) = p.fetch() else { return 1 };
                vm.stack.push(imm);
                vm_trace(vm, 0);
            }
            1 => {
                let Some(bv) = vm.stack.pop() else { return 2 };
                let Some(av) = vm.stack.pop() else { return 2 };
                vm.stack.push(av + bv);
                vm_trace(vm, 1);
            }
            2 => {
                let Some(bv) = vm.stack.pop() else { return 3 };
                let Some(av) = vm.stack.pop() else { return 3 };
                vm.stack.push(av * bv);
                vm_trace(vm, 2);
            }
            3 => {
                let a1: i32 = iv_peek(&vm.stack, 0);
                vm.stack.push(a1);
                vm_trace(vm, 3);
            }
            4 => {
                if vm.stack.pop().is_none() {
                    return 4;
                }
                vm_trace(vm, 4);
            }
            5 => {
                let x: i32 = iv_peek(&vm.stack, 0);
                let bucket: i32 = classify(impl_id, x);
                vm.stack.push(bucket);
                match bucket {
                    0 => vm_trace(vm, 5),
                    1 => vm_trace(vm, 6),
                    2 => vm_trace(vm, 7),
                    3 | 4 => vm_trace(vm, 8),
                    _ => vm_trace(vm, 9),
                }
            }
            6 => {
                let Some(k) = p.fetch() else { return 5 };
                let Some(cond) = vm.stack.pop() else { return 6 };
                if cond != 0 {
                    if (k as usize) > p.n.wrapping_sub(p.ip) {
                        return 7;
                    }
                    p.ip = (p.ip as u64).wrapping_add(k as u64) as usize;
                    vm_trace(vm, 10);
                } else {
                    vm_trace(vm, 11);
                }
            }
            7 => {
                let Some(times) = p.fetch() else { return 8 };
                if p.ip >= p.n {
                    return 9;
                }
                let saved_ip: usize = p.ip;
                let mut i: i32 = 0;
                while i < times {
                    // In the original, inner.code points to the same backing array,
                    // but the slice passed starts at inner.ip, and n=1.
                    let start = saved_ip;
                    let rc: i32 = if start <= code.len() {
                        run_engine_internal(impl_id, &code[start..], 1, vm)
                    } else {
                        // If out of bounds, mimic fetch failure => loop ends with rc=0 in C? Here, return 0.
                        0
                    };
                    if rc != 0 {
                        p.ip = saved_ip.wrapping_add(1);
                        vm_trace(vm, 12);
                        break;
                    } else {
                        i += 1;
                    }
                }
                p.ip = saved_ip.wrapping_add(1);
            }
            8 => {
                let x0: i32 = iv_peek(&vm.stack, 0);
                let y: i32 = classify(impl_id, x0);
                vm.stack.push(y);
                vm_trace(vm, 13);
            }
            9 => {
                let Some(m) = p.fetch() else { return 10 };
                if m < 0 || (m as usize) > vm.stack.len() {
                    return 11;
                }
                let m_usize = m as usize;
                let mut tmp: Vec<i32> = vec![0; m_usize];

                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }
                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }

                let s: i32 = process_stream(impl_id, &tmp, m_usize);
                vm.stack.push(s);
                vm_trace(vm, 14);
            }
            10 => return 0,
            _ => return 99,
        }
    }
    0
}

fn vm_print<W: Write>(mut fp: W, label: &str, vm: &VM) {
    let _ = write!(
        fp,
        "{}STACK_TOP={} STEPS={} TRACE=",
        label,
        iv_peek(&vm.stack, -777),
        vm.steps
    );
    for &t in &vm.trace {
        let idx: usize = (t & 25) as usize;
        let ch: u8 = b"abcdefghijklmnopqrstuvwxyz"[idx];
        let _ = fp.write_all(&[ch]);
    }
    let _ = fp.write_all(b"\n");
}

fn main() {
    // The original executable is argv-driven: argv[1..] are the program ints, and n is the last int.
    // argv[0] is ignored (program name).
    let args: Vec<String> = std::env::args().collect();
    let mut nums: Vec<i32> = Vec::new();
    for s in args.iter().skip(1) {
        nums.push(s.parse::<i32>().unwrap_or(0));
    }

    if nums.is_empty() {
        return;
    }

    let n: usize = nums.last().copied().unwrap_or(0).max(0) as usize;
    let code_all: &[i32] = &nums[..nums.len().saturating_sub(1)];
    let n_eff: usize = n.min(code_all.len());

    let mut vm_a = VM::default();
    let mut vm_b = VM::default();
    let mut vm_ext = VM::default();

    let rc_a = run_engine_internal(0, code_all, n_eff, &mut vm_a);
    let rc_b = run_engine_internal(1, code_all, n_eff, &mut vm_b);
    let rc_ext = run_engine_internal(2, code_all, n_eff, &mut vm_ext);

    print!("RC:A={} B={} EXT={}\n", rc_a, rc_b, rc_ext);

    let mut out = io::stdout().lock();
    vm_print(&mut out, "A:", &vm_a);
    vm_print(&mut out, "B:", &vm_b);
    vm_print(&mut out, "EXT:", &vm_ext);
}
```

**Entity:** Program<'a>

**States:** Ready (ip < n), Exhausted (ip >= n)

**Transitions:**
- Ready -> Ready via fetch() when ip increments and remains < n
- Ready -> Exhausted via fetch() when ip reaches n
- Exhausted -> Exhausted via fetch() returning None

**Evidence:** struct Program { code: &'a [i32], n: usize, ip: usize } encodes stream state in plain integers; Program::new(code, n) stores n without checking n <= code.len(); Program::fetch(): `if self.ip >= self.n { return None; }` and `let v = *self.code.get(self.ip)?;` are runtime guards for in-bounds access; run_engine_internal(): many opcodes treat missing immediates as errors, e.g. op 0: `let Some(imm) = p.fetch() else { return 1 };`, op 6: `let Some(k) = p.fetch() else { return 5 };`, op 7: `let Some(times) = p.fetch() else { return 8 };`, op 9: `let Some(m) = p.fetch() else { return 10 };`

**Implementation:** Introduce a validated program type that encodes `n <= code.len()` at construction (e.g., `ProgramSlice<'a> { code: &'a [i32] }` where `n` is always `code.len()`, or `Program<'a> { code: &'a [i32], ip: usize }` and slice the input to length `n` up-front). Optionally split into typestates like `Program<Ready>`/`Program<Exhausted>` returned by `fetch()` (or an iterator API) so opcode decoding cannot ignore exhaustion without handling it.

---

## Precondition Invariants

### 5. Relative-jump and subprogram execution bounds protocol (k/times must keep ip in-range)

**Location**: `/data/test_case/main.rs:1-428`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Control-flow opcodes rely on latent bounds invariants tying immediates to the remaining instruction count. Opcode 6 (conditional relative jump) requires that the immediate `k` not advance the instruction pointer beyond the remaining program (`p.n - p.ip`), checked at runtime. Opcode 7 performs a form of recursive single-instruction execution using `run_engine_internal` on a subslice starting at `saved_ip` with `n=1`, relying on `saved_ip <= code.len()` and on carefully restoring `p.ip`. These are temporal/ordering constraints on when and how `ip` may be modified that are not represented in types; violations become numeric rc errors or special-case behavior.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct VM, 3 free function(s); struct Program, impl Program < 'a > (2 methods)

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
use std::io::{self, Write};

mod lib {
    pub fn target(code: i32) -> i32 {
        if code < 0 {
            return 7;
        }
        let m: i32 = code % 10;
        if m == 0 {
            return 0;
        }
        if m <= 3 {
            return 1;
        }
        if m <= 6 {
            return 2;
        }
        if m == 7 {
            return 3;
        }
        4
    }
}

mod a {
    use super::Cell;

    #[inline]
    fn a_bias_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f((x ^ 0x55) + 7)
    }

    thread_local! {
        static STATE_A: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        if code < 0 {
            return STATE_A.with(|s| if (s.get() & 1) != 0 { 6 } else { 5 });
        }
        STATE_A.with(|s| s.set(s.get() ^ (code << 1)));
        let k: i32 = STATE_A.with(|s| ((code >> 2) ^ s.get()) & 7);
        match k {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 1,
            4 => 3,
            5 | 6 => 5,
            _ => 7,
        }
    }

    #[inline]
    fn wrap(x: i32) -> i32 {
        target(x - 5)
    }

    pub fn call_a_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = wrap(a);
        let c: i32 = target(b ^ 3);
        let d: i32 = a_bias_call(target, b);
        a ^ (b << 1) ^ (c << 2) ^ (d << 3)
    }

    pub fn process_a_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: usize = 0;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut j: i32 = 0;
            while j < 3 {
                let t: i32 = target(v + j);
                if (t & 1) == 0 {
                    acc = (acc as u64).wrapping_add(t as u64) as usize;
                } else {
                    acc = (acc as u64 ^ ((t << j) as u64)) as usize;
                    if t == 5 {
                        break;
                    }
                }
                j += 1;
            }
            i = i.wrapping_add(1);
        }
        if (acc as u64) > 0x7fffffff {
            acc = 0x7fffffff;
        }
        if (acc as u64) < 0xffffffff80000000u64 {
            acc = 0xffffffff80000000usize;
        }
        acc as i32
    }
}

mod b {
    use super::Cell;

    #[inline]
    fn b_twist_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f(((x + 9) ^ 0x2222) - 17)
    }

    thread_local! {
        static FLIPFLOP: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        let ff: i32 = FLIPFLOP.with(|f| {
            f.set(f.get() ^ 1);
            f.get()
        });

        if code < 0 {
            return if ff != 0 { 2 } else { 6 };
        }
        let mask: i32 = if ff != 0 { 0x7f } else { 0x1f };
        let z: i32 = (code ^ mask) % 8;
        if z == 0 || z == 7 {
            return 4;
        }
        if z == 1 || z == 2 {
            return 3;
        }
        if z == 3 {
            return 1;
        }
        if z == 4 {
            return 0;
        }
        if z == 5 {
            return 5;
        }
        7
    }

    #[inline]
    fn w2(x: i32) -> i32 {
        target(x + 9)
    }

    pub fn call_b_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = w2(a);
        let c: i32 = b_twist_call(target, a);
        let d: i32 = target(c ^ x);
        (a << 1) ^ (b << 2) ^ (c << 3) ^ (d << 4)
    }

    pub fn process_b_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: i32 = 1;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut iter: i32 = 0;
            loop {
                iter += 1;
                if iter > 4 {
                    break;
                }
                let t: i32 = target(v - iter);
                if t == 6 {
                    acc -= t;
                    break;
                } else {
                    if t == 3 {
                        continue;
                    }
                    acc = (acc * 3) ^ t;
                }
            }
            i = i.wrapping_add(1);
        }
        acc
    }
}

#[derive(Clone, Default)]
struct VM {
    stack: Vec<i32>,
    trace: Vec<i32>,
    steps: i32,
}

#[derive(Clone, Copy)]
struct Program<'a> {
    code: &'a [i32],
    n: usize,
    ip: usize,
}

impl<'a> Program<'a> {
    fn new(code: &'a [i32], n: usize) -> Self {
        Self { code, n, ip: 0 }
    }

    fn fetch(&mut self) -> Option<i32> {
        if self.ip >= self.n {
            return None;
        }
        let v = *self.code.get(self.ip)?;
        self.ip = self.ip.wrapping_add(1);
        Some(v)
    }
}

#[inline]
fn classify(impl_id: i32, x: i32) -> i32 {
    if impl_id == 0 {
        return a::call_a_once(x);
    }
    if impl_id == 1 {
        return b::call_b_once(x + 1);
    }
    lib::target(lib::target(x + 1))
}

fn process_stream(impl_id: i32, buf: &[i32], n: usize) -> i32 {
    if impl_id == 0 {
        return a::process_a_stream(buf, n);
    }
    if impl_id == 1 {
        return b::process_b_stream(buf, n);
    }
    let mut acc: i32 = 0;
    let mut i: usize = 0;
    while i < n {
        let t: i32 = lib::target(buf[i]);
        if (t & 1) == 0 {
            acc += t * 2;
        } else {
            acc ^= t + 7;
        }
        i = i.wrapping_add(1);
    }
    acc
}

#[inline]
fn vm_trace(vm: &mut VM, t: i32) {
    vm.trace.push(t);
}

#[inline]
fn iv_peek(stack: &[i32], def: i32) -> i32 {
    stack.last().copied().unwrap_or(def)
}

fn run_engine_internal(impl_id: i32, code: &[i32], n: usize, vm: &mut VM) -> i32 {
    let mut p: Program<'_> = Program::new(code, n);
    while let Some(op) = p.fetch() {
        vm.steps += 1;
        match op {
            0 => {
                let Some(imm) = p.fetch() else { return 1 };
                vm.stack.push(imm);
                vm_trace(vm, 0);
            }
            1 => {
                let Some(bv) = vm.stack.pop() else { return 2 };
                let Some(av) = vm.stack.pop() else { return 2 };
                vm.stack.push(av + bv);
                vm_trace(vm, 1);
            }
            2 => {
                let Some(bv) = vm.stack.pop() else { return 3 };
                let Some(av) = vm.stack.pop() else { return 3 };
                vm.stack.push(av * bv);
                vm_trace(vm, 2);
            }
            3 => {
                let a1: i32 = iv_peek(&vm.stack, 0);
                vm.stack.push(a1);
                vm_trace(vm, 3);
            }
            4 => {
                if vm.stack.pop().is_none() {
                    return 4;
                }
                vm_trace(vm, 4);
            }
            5 => {
                let x: i32 = iv_peek(&vm.stack, 0);
                let bucket: i32 = classify(impl_id, x);
                vm.stack.push(bucket);
                match bucket {
                    0 => vm_trace(vm, 5),
                    1 => vm_trace(vm, 6),
                    2 => vm_trace(vm, 7),
                    3 | 4 => vm_trace(vm, 8),
                    _ => vm_trace(vm, 9),
                }
            }
            6 => {
                let Some(k) = p.fetch() else { return 5 };
                let Some(cond) = vm.stack.pop() else { return 6 };
                if cond != 0 {
                    if (k as usize) > p.n.wrapping_sub(p.ip) {
                        return 7;
                    }
                    p.ip = (p.ip as u64).wrapping_add(k as u64) as usize;
                    vm_trace(vm, 10);
                } else {
                    vm_trace(vm, 11);
                }
            }
            7 => {
                let Some(times) = p.fetch() else { return 8 };
                if p.ip >= p.n {
                    return 9;
                }
                let saved_ip: usize = p.ip;
                let mut i: i32 = 0;
                while i < times {
                    // In the original, inner.code points to the same backing array,
                    // but the slice passed starts at inner.ip, and n=1.
                    let start = saved_ip;
                    let rc: i32 = if start <= code.len() {
                        run_engine_internal(impl_id, &code[start..], 1, vm)
                    } else {
                        // If out of bounds, mimic fetch failure => loop ends with rc=0 in C? Here, return 0.
                        0
                    };
                    if rc != 0 {
                        p.ip = saved_ip.wrapping_add(1);
                        vm_trace(vm, 12);
                        break;
                    } else {
                        i += 1;
                    }
                }
                p.ip = saved_ip.wrapping_add(1);
            }
            8 => {
                let x0: i32 = iv_peek(&vm.stack, 0);
                let y: i32 = classify(impl_id, x0);
                vm.stack.push(y);
                vm_trace(vm, 13);
            }
            9 => {
                let Some(m) = p.fetch() else { return 10 };
                if m < 0 || (m as usize) > vm.stack.len() {
                    return 11;
                }
                let m_usize = m as usize;
                let mut tmp: Vec<i32> = vec![0; m_usize];

                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }
                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }

                let s: i32 = process_stream(impl_id, &tmp, m_usize);
                vm.stack.push(s);
                vm_trace(vm, 14);
            }
            10 => return 0,
            _ => return 99,
        }
    }
    0
}

fn vm_print<W: Write>(mut fp: W, label: &str, vm: &VM) {
    let _ = write!(
        fp,
        "{}STACK_TOP={} STEPS={} TRACE=",
        label,
        iv_peek(&vm.stack, -777),
        vm.steps
    );
    for &t in &vm.trace {
        let idx: usize = (t & 25) as usize;
        let ch: u8 = b"abcdefghijklmnopqrstuvwxyz"[idx];
        let _ = fp.write_all(&[ch]);
    }
    let _ = fp.write_all(b"\n");
}

fn main() {
    // The original executable is argv-driven: argv[1..] are the program ints, and n is the last int.
    // argv[0] is ignored (program name).
    let args: Vec<String> = std::env::args().collect();
    let mut nums: Vec<i32> = Vec::new();
    for s in args.iter().skip(1) {
        nums.push(s.parse::<i32>().unwrap_or(0));
    }

    if nums.is_empty() {
        return;
    }

    let n: usize = nums.last().copied().unwrap_or(0).max(0) as usize;
    let code_all: &[i32] = &nums[..nums.len().saturating_sub(1)];
    let n_eff: usize = n.min(code_all.len());

    let mut vm_a = VM::default();
    let mut vm_b = VM::default();
    let mut vm_ext = VM::default();

    let rc_a = run_engine_internal(0, code_all, n_eff, &mut vm_a);
    let rc_b = run_engine_internal(1, code_all, n_eff, &mut vm_b);
    let rc_ext = run_engine_internal(2, code_all, n_eff, &mut vm_ext);

    print!("RC:A={} B={} EXT={}\n", rc_a, rc_b, rc_ext);

    let mut out = io::stdout().lock();
    vm_print(&mut out, "A:", &vm_a);
    vm_print(&mut out, "B:", &vm_b);
    vm_print(&mut out, "EXT:", &vm_ext);
}
```

**Entity:** run_engine_internal (control-flow over Program/VM)

**States:** Executing, Branching/Recursing, Error (out-of-bounds / invalid operand)

**Transitions:**
- Executing -> Branching/Recursing via opcode 6 or 7
- Branching/Recursing -> Executing via updating/restoring p.ip
- Executing/Branching -> Error via returning rc (e.g., 7, 9) when bounds checks fail

**Evidence:** opcode 6: `let Some(k) = p.fetch() else { return 5 }; ... if (k as usize) > p.n.wrapping_sub(p.ip) { return 7; } p.ip = ... + k` is a runtime check encoding a jump-bounds precondition; opcode 7: uses `saved_ip = p.ip;` then `run_engine_internal(impl_id, &code[start..], 1, vm)` with comment describing the intended slice/ip relationship; also checks `if start <= code.len()` else returns 0; opcode 7: multiple assignments `p.ip = saved_ip.wrapping_add(1);` show a required restore protocol around the recursive call

**Implementation:** Introduce validated immediate types for control flow, e.g. `RelOffset` or `InBoundsOffset` created only after checking against the current `Program` remaining length, and make the jump API accept only that newtype. Similarly, encapsulate 'execute one-instruction subprogram at current ip' as a safe method on `Program` that internally guarantees slice creation and ip restoration, rather than open-coding it with raw indices.

---

## Protocol Invariants

### 4. VM stack-effect protocol (opcodes require specific stack heights) and error-code state signaling

**Location**: `/data/test_case/main.rs:1-428`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The VM executes a stack-based bytecode where each opcode has implicit preconditions on the stack height (e.g., add/mul require 2 values; pop requires 1; conditional jump requires a condition value; stream op requires m elements available). These preconditions are enforced via runtime `pop()` checks and error returns (rc values 2/3/4/6/11 etc.). The type system does not encode stack height or opcode stack effects, so invalid programs are only detected at runtime, and different error codes represent different violated preconditions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct VM, 3 free function(s); struct Program, impl Program < 'a > (2 methods)

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
use std::io::{self, Write};

mod lib {
    pub fn target(code: i32) -> i32 {
        if code < 0 {
            return 7;
        }
        let m: i32 = code % 10;
        if m == 0 {
            return 0;
        }
        if m <= 3 {
            return 1;
        }
        if m <= 6 {
            return 2;
        }
        if m == 7 {
            return 3;
        }
        4
    }
}

mod a {
    use super::Cell;

    #[inline]
    fn a_bias_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f((x ^ 0x55) + 7)
    }

    thread_local! {
        static STATE_A: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        if code < 0 {
            return STATE_A.with(|s| if (s.get() & 1) != 0 { 6 } else { 5 });
        }
        STATE_A.with(|s| s.set(s.get() ^ (code << 1)));
        let k: i32 = STATE_A.with(|s| ((code >> 2) ^ s.get()) & 7);
        match k {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 1,
            4 => 3,
            5 | 6 => 5,
            _ => 7,
        }
    }

    #[inline]
    fn wrap(x: i32) -> i32 {
        target(x - 5)
    }

    pub fn call_a_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = wrap(a);
        let c: i32 = target(b ^ 3);
        let d: i32 = a_bias_call(target, b);
        a ^ (b << 1) ^ (c << 2) ^ (d << 3)
    }

    pub fn process_a_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: usize = 0;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut j: i32 = 0;
            while j < 3 {
                let t: i32 = target(v + j);
                if (t & 1) == 0 {
                    acc = (acc as u64).wrapping_add(t as u64) as usize;
                } else {
                    acc = (acc as u64 ^ ((t << j) as u64)) as usize;
                    if t == 5 {
                        break;
                    }
                }
                j += 1;
            }
            i = i.wrapping_add(1);
        }
        if (acc as u64) > 0x7fffffff {
            acc = 0x7fffffff;
        }
        if (acc as u64) < 0xffffffff80000000u64 {
            acc = 0xffffffff80000000usize;
        }
        acc as i32
    }
}

mod b {
    use super::Cell;

    #[inline]
    fn b_twist_call(f: fn(i32) -> i32, x: i32) -> i32 {
        f(((x + 9) ^ 0x2222) - 17)
    }

    thread_local! {
        static FLIPFLOP: Cell<i32> = const { Cell::new(0) };
    }

    fn target(code: i32) -> i32 {
        let ff: i32 = FLIPFLOP.with(|f| {
            f.set(f.get() ^ 1);
            f.get()
        });

        if code < 0 {
            return if ff != 0 { 2 } else { 6 };
        }
        let mask: i32 = if ff != 0 { 0x7f } else { 0x1f };
        let z: i32 = (code ^ mask) % 8;
        if z == 0 || z == 7 {
            return 4;
        }
        if z == 1 || z == 2 {
            return 3;
        }
        if z == 3 {
            return 1;
        }
        if z == 4 {
            return 0;
        }
        if z == 5 {
            return 5;
        }
        7
    }

    #[inline]
    fn w2(x: i32) -> i32 {
        target(x + 9)
    }

    pub fn call_b_once(x: i32) -> i32 {
        let a: i32 = target(x);
        let b: i32 = w2(a);
        let c: i32 = b_twist_call(target, a);
        let d: i32 = target(c ^ x);
        (a << 1) ^ (b << 2) ^ (c << 3) ^ (d << 4)
    }

    pub fn process_b_stream(xs: &[i32], n: usize) -> i32 {
        let mut acc: i32 = 1;
        let mut i: usize = 0;
        while i < n {
            let v: i32 = xs[i];
            let mut iter: i32 = 0;
            loop {
                iter += 1;
                if iter > 4 {
                    break;
                }
                let t: i32 = target(v - iter);
                if t == 6 {
                    acc -= t;
                    break;
                } else {
                    if t == 3 {
                        continue;
                    }
                    acc = (acc * 3) ^ t;
                }
            }
            i = i.wrapping_add(1);
        }
        acc
    }
}

#[derive(Clone, Default)]
struct VM {
    stack: Vec<i32>,
    trace: Vec<i32>,
    steps: i32,
}

#[derive(Clone, Copy)]
struct Program<'a> {
    code: &'a [i32],
    n: usize,
    ip: usize,
}

impl<'a> Program<'a> {
    fn new(code: &'a [i32], n: usize) -> Self {
        Self { code, n, ip: 0 }
    }

    fn fetch(&mut self) -> Option<i32> {
        if self.ip >= self.n {
            return None;
        }
        let v = *self.code.get(self.ip)?;
        self.ip = self.ip.wrapping_add(1);
        Some(v)
    }
}

#[inline]
fn classify(impl_id: i32, x: i32) -> i32 {
    if impl_id == 0 {
        return a::call_a_once(x);
    }
    if impl_id == 1 {
        return b::call_b_once(x + 1);
    }
    lib::target(lib::target(x + 1))
}

fn process_stream(impl_id: i32, buf: &[i32], n: usize) -> i32 {
    if impl_id == 0 {
        return a::process_a_stream(buf, n);
    }
    if impl_id == 1 {
        return b::process_b_stream(buf, n);
    }
    let mut acc: i32 = 0;
    let mut i: usize = 0;
    while i < n {
        let t: i32 = lib::target(buf[i]);
        if (t & 1) == 0 {
            acc += t * 2;
        } else {
            acc ^= t + 7;
        }
        i = i.wrapping_add(1);
    }
    acc
}

#[inline]
fn vm_trace(vm: &mut VM, t: i32) {
    vm.trace.push(t);
}

#[inline]
fn iv_peek(stack: &[i32], def: i32) -> i32 {
    stack.last().copied().unwrap_or(def)
}

fn run_engine_internal(impl_id: i32, code: &[i32], n: usize, vm: &mut VM) -> i32 {
    let mut p: Program<'_> = Program::new(code, n);
    while let Some(op) = p.fetch() {
        vm.steps += 1;
        match op {
            0 => {
                let Some(imm) = p.fetch() else { return 1 };
                vm.stack.push(imm);
                vm_trace(vm, 0);
            }
            1 => {
                let Some(bv) = vm.stack.pop() else { return 2 };
                let Some(av) = vm.stack.pop() else { return 2 };
                vm.stack.push(av + bv);
                vm_trace(vm, 1);
            }
            2 => {
                let Some(bv) = vm.stack.pop() else { return 3 };
                let Some(av) = vm.stack.pop() else { return 3 };
                vm.stack.push(av * bv);
                vm_trace(vm, 2);
            }
            3 => {
                let a1: i32 = iv_peek(&vm.stack, 0);
                vm.stack.push(a1);
                vm_trace(vm, 3);
            }
            4 => {
                if vm.stack.pop().is_none() {
                    return 4;
                }
                vm_trace(vm, 4);
            }
            5 => {
                let x: i32 = iv_peek(&vm.stack, 0);
                let bucket: i32 = classify(impl_id, x);
                vm.stack.push(bucket);
                match bucket {
                    0 => vm_trace(vm, 5),
                    1 => vm_trace(vm, 6),
                    2 => vm_trace(vm, 7),
                    3 | 4 => vm_trace(vm, 8),
                    _ => vm_trace(vm, 9),
                }
            }
            6 => {
                let Some(k) = p.fetch() else { return 5 };
                let Some(cond) = vm.stack.pop() else { return 6 };
                if cond != 0 {
                    if (k as usize) > p.n.wrapping_sub(p.ip) {
                        return 7;
                    }
                    p.ip = (p.ip as u64).wrapping_add(k as u64) as usize;
                    vm_trace(vm, 10);
                } else {
                    vm_trace(vm, 11);
                }
            }
            7 => {
                let Some(times) = p.fetch() else { return 8 };
                if p.ip >= p.n {
                    return 9;
                }
                let saved_ip: usize = p.ip;
                let mut i: i32 = 0;
                while i < times {
                    // In the original, inner.code points to the same backing array,
                    // but the slice passed starts at inner.ip, and n=1.
                    let start = saved_ip;
                    let rc: i32 = if start <= code.len() {
                        run_engine_internal(impl_id, &code[start..], 1, vm)
                    } else {
                        // If out of bounds, mimic fetch failure => loop ends with rc=0 in C? Here, return 0.
                        0
                    };
                    if rc != 0 {
                        p.ip = saved_ip.wrapping_add(1);
                        vm_trace(vm, 12);
                        break;
                    } else {
                        i += 1;
                    }
                }
                p.ip = saved_ip.wrapping_add(1);
            }
            8 => {
                let x0: i32 = iv_peek(&vm.stack, 0);
                let y: i32 = classify(impl_id, x0);
                vm.stack.push(y);
                vm_trace(vm, 13);
            }
            9 => {
                let Some(m) = p.fetch() else { return 10 };
                if m < 0 || (m as usize) > vm.stack.len() {
                    return 11;
                }
                let m_usize = m as usize;
                let mut tmp: Vec<i32> = vec![0; m_usize];

                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }
                for idx in (0..m_usize).rev() {
                    tmp[idx] = vm.stack.pop().unwrap_or(0);
                }

                let s: i32 = process_stream(impl_id, &tmp, m_usize);
                vm.stack.push(s);
                vm_trace(vm, 14);
            }
            10 => return 0,
            _ => return 99,
        }
    }
    0
}

fn vm_print<W: Write>(mut fp: W, label: &str, vm: &VM) {
    let _ = write!(
        fp,
        "{}STACK_TOP={} STEPS={} TRACE=",
        label,
        iv_peek(&vm.stack, -777),
        vm.steps
    );
    for &t in &vm.trace {
        let idx: usize = (t & 25) as usize;
        let ch: u8 = b"abcdefghijklmnopqrstuvwxyz"[idx];
        let _ = fp.write_all(&[ch]);
    }
    let _ = fp.write_all(b"\n");
}

fn main() {
    // The original executable is argv-driven: argv[1..] are the program ints, and n is the last int.
    // argv[0] is ignored (program name).
    let args: Vec<String> = std::env::args().collect();
    let mut nums: Vec<i32> = Vec::new();
    for s in args.iter().skip(1) {
        nums.push(s.parse::<i32>().unwrap_or(0));
    }

    if nums.is_empty() {
        return;
    }

    let n: usize = nums.last().copied().unwrap_or(0).max(0) as usize;
    let code_all: &[i32] = &nums[..nums.len().saturating_sub(1)];
    let n_eff: usize = n.min(code_all.len());

    let mut vm_a = VM::default();
    let mut vm_b = VM::default();
    let mut vm_ext = VM::default();

    let rc_a = run_engine_internal(0, code_all, n_eff, &mut vm_a);
    let rc_b = run_engine_internal(1, code_all, n_eff, &mut vm_b);
    let rc_ext = run_engine_internal(2, code_all, n_eff, &mut vm_ext);

    print!("RC:A={} B={} EXT={}\n", rc_a, rc_b, rc_ext);

    let mut out = io::stdout().lock();
    vm_print(&mut out, "A:", &vm_a);
    vm_print(&mut out, "B:", &vm_b);
    vm_print(&mut out, "EXT:", &vm_ext);
}
```

**Entity:** VM

**States:** Running (invariants satisfied for next opcode), Trapped/Error (returned nonzero rc)

**Transitions:**
- Running -> Trapped/Error via returning rc when a stack precondition fails (e.g., insufficient operands)
- Running -> Running via successful opcode execution that mutates stack/trace/steps
- Running -> (implicit) Halt via opcode 10 returning 0

**Evidence:** struct VM { stack: Vec<i32>, trace: Vec<i32>, steps: i32 } uses a plain Vec with no compile-time stack-depth tracking; op 1 (add): `let Some(bv) = vm.stack.pop() else { return 2 }; let Some(av) = vm.stack.pop() else { return 2 };` requires 2 stack values; op 2 (mul): analogous runtime checks returning 3; op 4 (pop): `if vm.stack.pop().is_none() { return 4; }` requires 1 stack value; op 6 (cond jump): `let Some(cond) = vm.stack.pop() else { return 6 };` requires 1 stack value; op 9 (stream op): `if m < 0 || (m as usize) > vm.stack.len() { return 11; }` encodes a dynamic precondition on stack length

**Implementation:** If feasible, model bytecode as a typed IR with stack effects (e.g., GADT-like encoding or a validated `Instruction` enum produced by a verifier pass) so `run_engine_internal` operates on verified instructions rather than raw i32 opcodes. A lighter alternative is a `VerifiedProgram` newtype produced by a verifier that checks stack-height constraints and branch bounds ahead of execution, making `run_engine_internal` take `VerifiedProgram` and allowing many runtime error paths to be eliminated.

---

