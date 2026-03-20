# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 2

## Precondition Invariants

### 1. C-string validity precondition for printLine (Empty / Valid CStr / Invalid)

**Location**: `/data/test_case/lib.rs:1-55`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: printLine implicitly expects either (a) an empty slice (in which case it is a no-op) or (b) a non-empty slice whose bytes contain a NUL terminator and represent valid UTF-8 up to the first NUL. These requirements are enforced only by runtime early-return and unwrap()-based panics. The type system currently accepts any &[i8], including non-NUL-terminated buffers or buffers with non-UTF-8 contents, which will panic at runtime.

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
        use std::ffi::CStr;

        pub(crate) fn printLine(line: &[i8]) {
            if line.is_empty() {
                // Match original behavior: do nothing for empty input.
                return;
            }

            // SAFETY: We only reinterpret the same memory as u8 for CStr parsing.
            let bytes: &[u8] =
                unsafe { std::slice::from_raw_parts(line.as_ptr() as *const u8, line.len()) };

            // Preserve original semantics: unwrap on missing NUL / invalid UTF-8.
            let s = CStr::from_bytes_until_nul(bytes)
                .unwrap()
                .to_str()
                .unwrap();
            println!("{0}", s);
        }

        pub(crate) fn bad() {
            let data: &[i8] = &[];
            printLine(data);
        }

        pub(crate) fn good() {
            // Provide a proper NUL-terminated C string.
            let bytes = b"string\0";
            let data: &[i8] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len()) };
            printLine(data);
        }

        #[no_mangle]
        pub extern "C" fn driver(useGood: i32) {
            if useGood != 0 {
                good();
            } else {
                bad();
            };
        }
    }
}
```

**Entity:** printLine(line: &[i8]) input buffer

**States:** Empty, NonEmptyValidCStr, NonEmptyInvalidCStr

**Transitions:**
- Empty -> (returns) via early return in printLine()
- NonEmptyValidCStr -> (prints) via CStr::from_bytes_until_nul(...).to_str()
- NonEmptyInvalidCStr -> (panic) via unwrap() in CStr::from_bytes_until_nul / to_str

**Evidence:** fn printLine(line: &[i8]): accepts arbitrary &[i8] without expressing C-string/UTF-8 constraints; printLine: `if line.is_empty() { return; }` special-cases Empty state; printLine: `CStr::from_bytes_until_nul(bytes).unwrap()` panics if no NUL terminator exists in the provided length; printLine: `.to_str().unwrap()` panics if bytes before NUL are not valid UTF-8; printLine comment: "Preserve original semantics: unwrap on missing NUL / invalid UTF-8."; good(): uses `b"string\0"` to satisfy the implicit NUL-terminated invariant; bad(): passes `&[]`, relying on the empty-slice special case

**Implementation:** Introduce a validated wrapper like `struct Utf8CStr<'a>(&'a CStr);` or accept `&CStr`/`&std::ffi::CStr` directly. Provide a conversion `impl<'a> TryFrom<&'a [i8]> for Utf8CStr<'a>` that checks for NUL and UTF-8 once, then make `printLine` take `Option<Utf8CStr<'_>>` (or overload) to represent the Empty case explicitly.

---

