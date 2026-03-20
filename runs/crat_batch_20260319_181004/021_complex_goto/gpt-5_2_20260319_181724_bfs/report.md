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

### 1. foo loop-progress protocol (x/y must monotonically decrease to terminate)

**Location**: `/data/test_case/main.rs:1-68`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The logic in foo() relies on an implicit progress/termination protocol: while (x > 0 || y > 0) holds, each iteration must eventually decrement x and/or y so the outer loop can reach the termination condition (x <= 0 && y <= 0). This is enforced only by the specific arrangement of runtime checks and decrements; the type system does not encode that x/y are non-negative, that they monotonically decrease, or that the inner loops always make progress toward termination. Passing negative values also violates the intended meaning of the guards (x>0/y>0) without any type-level restriction.

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

fn foo(mut x: i32, mut y: i32) {
    while x > 0 || y > 0 {
        println!("loop");

        if x == 1 && y == 4 {
            // Start in the "y" branch.
            loop {
                if y == 0 {
                    break;
                }
                println!("y");
                y -= 1;
                if x < 3 {
                    if x > 0 {
                        println!("x");
                        x -= 1;
                    }
                    continue;
                } else {
                    break;
                }
            }
        } else {
            // Start in the "x" branch, then proceed to the "y" branch.
            loop {
                if x > 0 {
                    println!("x");
                    x -= 1;
                }

                if y == 0 {
                    break;
                }
                println!("y");
                y -= 1;

                if x < 3 {
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();

    let x: i32 = it.next().unwrap_or("0").parse().unwrap();
    let y: i32 = it.next().unwrap_or("0").parse().unwrap();

    foo(x, y);
}
```

**Entity:** foo(x: i32, y: i32) loop variables

**States:** Running, Terminated

**Transitions:**
- Running -> Terminated via reaching outer-loop condition x <= 0 && y <= 0 (after repeated x -= 1 / y -= 1 steps)

**Evidence:** foo(): while x > 0 || y > 0 { ... } establishes the intended termination condition; both inner loops: y -= 1 and x -= 1 are the only progress steps ("y" branch: y -= 1; optionally x -= 1; "x then y" branch: x -= 1 then y -= 1); guards that control whether decrements happen: if x > 0 { x -= 1 }, if y == 0 { break }, if x < 3 { continue } else { break }

**Implementation:** Introduce a newtype for inputs such as NonNegativeI32 (or use u32/usize) to encode the precondition that x and y represent counts. Optionally model the algorithm as operating on a DecreasingCounter type whose API only allows decrementing toward zero, making invalid negative states unrepresentable.

---

