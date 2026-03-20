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

### 1. Signed-byte formatting protocol (C2Rust sign-extension vs intended 8-bit hex)

**Location**: `/data/test_case/lib.rs:1-30`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The function relies on an implicit formatting protocol: it intentionally sign-extends the i8 input to u32 via i32 so that negative i8 values become 0xffff_ffxx and are printed without truncation, matching original C2Rust behavior. This is a semantic choice about how to interpret the input byte (signed vs unsigned / full-width vs masked) that is enforced only by an in-function casting sequence and comments; callers cannot express at the type level which interpretation they intend, and accidental refactors (e.g., casting directly to u8/u32) would silently change behavior.

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

        pub(crate) fn printHexCharLine(charHex: i8) {
            // Match the original C2Rust behavior:
            // println!("{0:>02x}", crate::c_lib::Xu32(charHex as u32));
            // i8 -> u32 sign-extends via i32, so negative values become 0xffff_ffxx
            // and are printed without truncation.
            let v: u32 = charHex as i32 as u32;
            println!("{0:>02x}", v);
        }

        #[no_mangle]
        pub extern "C" fn driver(data: i8) {
            let result: i8 = (data as i32 + 1) as i8;
            printHexCharLine(result);
        }
    }
}
```

**Entity:** printHexCharLine(charHex: i8)

**States:** C2Rust-compatible sign-extended printing, 8-bit-truncated hex printing

**Transitions:**
- i8 (possibly negative) -> u32 sign-extended via `as i32 as u32` before formatting

**Evidence:** printHexCharLine signature: `fn printHexCharLine(charHex: i8)` takes a signed byte; comment: "i8 -> u32 sign-extends via i32, so negative values become 0xffff_ffxx and are printed without truncation."; code: `let v: u32 = charHex as i32 as u32;` encodes the protocol choice

**Implementation:** Introduce a newtype capturing the intended interpretation, e.g. `struct SignExtendedByte(i8);` with `impl From<i8> for SignExtendedByte` and `impl Display/LowerHex` (or a dedicated `print()` method). Alternatively use `struct TruncatedByte(u8)` for the other semantics; `printHexCharLine` would accept the newtype so the choice is explicit at compile time.

---

