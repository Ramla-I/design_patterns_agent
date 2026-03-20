# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 0
- **Protocol**: 1
- **Modules analyzed**: 2

## Protocol Invariants

### 1. driver step-control protocol (x-step / y-step scheduling via do_x_step)

**Location**: `/data/test_case/lib.rs:1-50`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The function implements an implicit two-phase stepping protocol controlled by the local flag `do_x_step`: when `do_x_step` is true it performs at most one `x` decrement (if `x > 0`) and then forces a transition to the `y` phase; when `do_x_step` is false it performs `y` decrements until either `y == 0` (terminating the inner loop) or until `x < 3` triggers a transition back to allow another `x` step. There is also a special-case scheduling rule (`x==1 && y==4`) that skips the initial `x` step and begins in the `y` phase. These phase/transition rules are enforced only by runtime control flow and a boolean, not by any type-level representation of the protocol (e.g., a stepper state type), so incorrect rewrites/refactors could easily violate the intended sequencing without compiler help.

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

pub mod src {
    pub mod lib {
        // === driver.rs ===
        #[no_mangle]
        pub extern "C" fn driver(mut x: i32, mut y: i32) {
            while x > 0 || y > 0 {
                println!("loop");

                // Preserve the original control-flow behavior:
                // - If (x==1 && y==4): skip the "x" step and go directly to the "y" step.
                // - Otherwise: do one "x" step (if possible), then proceed to "y" steps.
                let mut do_x_step = !(x == 1 && y == 4);

                loop {
                    if do_x_step {
                        if x > 0 {
                            println!("x");
                            x -= 1;
                        }
                        do_x_step = false;
                        continue;
                    }

                    if y == 0 {
                        break;
                    }

                    println!("y");
                    y -= 1;

                    if x < 3 {
                        do_x_step = true;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
```

**Entity:** driver (extern "C" fn)

**States:** XStepAllowed, YStepOnly, Terminated

**Transitions:**
- XStepAllowed -> YStepOnly via setting `do_x_step = false` after attempting the x-step
- YStepOnly -> XStepAllowed via `if x < 3 { do_x_step = true; }` after a y-step
- YStepOnly -> Terminated via `if y == 0 { break; }` (inner loop ends)
- XStepAllowed -> Terminated indirectly when outer condition `while x > 0 || y > 0` becomes false after steps
- Start -> YStepOnly when `do_x_step` initialized to false by special case `!(x == 1 && y == 4)`

**Evidence:** function `driver(mut x: i32, mut y: i32)` contains an implicit state machine encoded in local control flow; `let mut do_x_step = !(x == 1 && y == 4);` encodes a special-case initial state (skip x-step when x==1 && y==4); inner `loop { if do_x_step { ... x -= 1; do_x_step = false; continue; } ... }` encodes the XStepAllowed -> YStepOnly transition; `if y == 0 { break; }` defines a termination condition for the y-phase (inner loop termination); after `y -= 1;` the branch `if x < 3 { do_x_step = true; } else { break; }` encodes YStepOnly -> XStepAllowed or YStepOnly -> Terminated transitions; comment: "Preserve the original control-flow behavior" documents the protocol requirements and special-case rule

**Implementation:** Refactor the inner stepping logic into a small stepper type, e.g. `struct Driver { x: i32, y: i32 }` plus state types `struct XStep; struct YStep;` and methods like `fn step(self, state: XStep) -> (Self, YStep)` / `fn step(self, state: YStep) -> StepResult` that make the allowed transitions explicit. The special-case start state can be represented by choosing the initial state type (`XStep` vs `YStep`) based on the `(x,y)` inputs, rather than a boolean flag.

---

