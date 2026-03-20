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

### 1. C-string pointer validity & NUL-termination protocol (Null / NonNull valid CStr)

**Location**: `/data/test_case/main.rs:1-94`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Callers of helperBad()/helperGood1() treat the returned *mut i8 as either null (special-cased) or a valid pointer to a NUL-terminated character buffer. This protocol is enforced with a runtime null check and by relying on the thread-local array having a trailing 0 byte. The type system does not express (1) non-nullness, (2) provenance/lifetime of the pointed-to storage, or (3) that the buffer is a valid C string.

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

use std::io::{self, Read};

pub(crate) fn printLine(line: &[i8]) {
    if line.is_empty() {
        return;
    }

    // Interpret as a C string: print up to the first NUL byte.
    let bytes: Vec<u8> = line
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();

    println!("{}", String::from_utf8_lossy(&bytes));
}

fn helperBad() -> *mut i8 {
    // In the original C2Rust, this returns a pointer to a stack buffer (dangling).
    // We preserve the intent (a pointer to a NUL-terminated string) safely by
    // returning a pointer to thread-local storage.
    thread_local! {
        static CHAR_STRING: std::cell::RefCell<[i8; 17]> = const {
            std::cell::RefCell::new([
                b'h' as i8, b'e' as i8, b'l' as i8, b'p' as i8, b'e' as i8, b'r' as i8,
                b'B' as i8, b'a' as i8, b'd' as i8, b' ' as i8, b's' as i8, b't' as i8,
                b'r' as i8, b'i' as i8, b'n' as i8, b'g' as i8, 0,
            ])
        };
    }

    CHAR_STRING.with_borrow_mut(|s| s.as_mut_ptr())
}

pub(crate) unsafe fn bad() {
    let ptr = helperBad();
    if ptr.is_null() {
        printLine(&[]);
    } else {
        // Preserve the original "oversized slice" behavior; printLine stops at NUL.
        let slice = std::slice::from_raw_parts(ptr, 100000);
        printLine(slice);
    }
}

fn helperGood1() -> *mut i8 {
    thread_local! {
        static CHAR_STRING: std::cell::RefCell<[i8; 19]> = const {
            std::cell::RefCell::new([
                b'h' as i8, b'e' as i8, b'l' as i8, b'p' as i8, b'e' as i8, b'r' as i8,
                b'G' as i8, b'o' as i8, b'o' as i8, b'd' as i8, b'1' as i8, b' ' as i8,
                b's' as i8, b't' as i8, b'r' as i8, b'i' as i8, b'n' as i8, b'g' as i8, 0,
            ])
        };
    }

    CHAR_STRING.with_borrow_mut(|s| s.as_mut_ptr())
}

pub(crate) unsafe fn good() {
    let ptr = helperGood1();
    if ptr.is_null() {
        printLine(&[]);
    } else {
        let slice = std::slice::from_raw_parts(ptr, 100000);
        printLine(slice);
    }
}

fn main() {
    // Read an integer from stdin (like scanf("%d", &x)).
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);

    unsafe {
        if x != 0 {
            good();
        } else {
            bad();
        }
    }
}
```

**Entity:** helperBad/helperGood1 return value (*mut i8)

**States:** Null, NonNullValidCStr

**Transitions:**
- Null -> NonNullValidCStr via helperBad()/helperGood1() returning a non-null pointer (conceptual)

**Evidence:** fn helperBad() -> *mut i8: returns a raw pointer intended to represent a NUL-terminated string; fn helperGood1() -> *mut i8: same pattern, raw pointer return; bad(): `if ptr.is_null() { ... } else { ... }` runtime check gates use; good(): `if ptr.is_null() { ... } else { ... }` runtime check gates use; helperBad thread_local CHAR_STRING ends with `0,` (explicit NUL terminator); helperGood1 thread_local CHAR_STRING ends with `0,` (explicit NUL terminator); comment in helperBad: "pointer to a NUL-terminated string" and discussion of dangling pointer intent

**Implementation:** Return `Option<std::ffi::CString>` or `Option<&'static std::ffi::CStr>` (if storage can be made 'static), or a newtype like `struct NonNullCStr(NonNull<i8>);` constructed only from known-NUL-terminated buffers; eliminate the null branch by returning `NonNull<i8>` (or `&CStr`) when null is impossible.

---

## Protocol Invariants

### 2. Raw pointer -> slice creation safety protocol (ValidRange for len / InBoundsUntilNul)

**Location**: `/data/test_case/main.rs:1-94`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: bad()/good() create an oversized slice (`from_raw_parts(ptr, 100000)`) from a raw pointer and then iterate it in printLine until a NUL byte is found. This implicitly requires that the memory region `[ptr, ptr + 100000)` is readable (or at least readable up to the first NUL), and that a NUL occurs before any invalid memory. The code relies on comments/intent and on the thread-local backing arrays, but the type system cannot ensure the slice length is within the allocated object nor that scanning will not read out of bounds.

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

use std::io::{self, Read};

pub(crate) fn printLine(line: &[i8]) {
    if line.is_empty() {
        return;
    }

    // Interpret as a C string: print up to the first NUL byte.
    let bytes: Vec<u8> = line
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();

    println!("{}", String::from_utf8_lossy(&bytes));
}

fn helperBad() -> *mut i8 {
    // In the original C2Rust, this returns a pointer to a stack buffer (dangling).
    // We preserve the intent (a pointer to a NUL-terminated string) safely by
    // returning a pointer to thread-local storage.
    thread_local! {
        static CHAR_STRING: std::cell::RefCell<[i8; 17]> = const {
            std::cell::RefCell::new([
                b'h' as i8, b'e' as i8, b'l' as i8, b'p' as i8, b'e' as i8, b'r' as i8,
                b'B' as i8, b'a' as i8, b'd' as i8, b' ' as i8, b's' as i8, b't' as i8,
                b'r' as i8, b'i' as i8, b'n' as i8, b'g' as i8, 0,
            ])
        };
    }

    CHAR_STRING.with_borrow_mut(|s| s.as_mut_ptr())
}

pub(crate) unsafe fn bad() {
    let ptr = helperBad();
    if ptr.is_null() {
        printLine(&[]);
    } else {
        // Preserve the original "oversized slice" behavior; printLine stops at NUL.
        let slice = std::slice::from_raw_parts(ptr, 100000);
        printLine(slice);
    }
}

fn helperGood1() -> *mut i8 {
    thread_local! {
        static CHAR_STRING: std::cell::RefCell<[i8; 19]> = const {
            std::cell::RefCell::new([
                b'h' as i8, b'e' as i8, b'l' as i8, b'p' as i8, b'e' as i8, b'r' as i8,
                b'G' as i8, b'o' as i8, b'o' as i8, b'd' as i8, b'1' as i8, b' ' as i8,
                b's' as i8, b't' as i8, b'r' as i8, b'i' as i8, b'n' as i8, b'g' as i8, 0,
            ])
        };
    }

    CHAR_STRING.with_borrow_mut(|s| s.as_mut_ptr())
}

pub(crate) unsafe fn good() {
    let ptr = helperGood1();
    if ptr.is_null() {
        printLine(&[]);
    } else {
        let slice = std::slice::from_raw_parts(ptr, 100000);
        printLine(slice);
    }
}

fn main() {
    // Read an integer from stdin (like scanf("%d", &x)).
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    let x: i32 = input.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);

    unsafe {
        if x != 0 {
            good();
        } else {
            bad();
        }
    }
}
```

**Entity:** bad()/good() unsafe raw slice creation from pointer

**States:** UncheckedPointer, SliceBackedByValidMemory

**Transitions:**
- UncheckedPointer -> SliceBackedByValidMemory via `std::slice::from_raw_parts(ptr, 100000)` in bad()/good()

**Evidence:** bad(): `let slice = std::slice::from_raw_parts(ptr, 100000);` creates a slice with a length unrelated to the backing array size; good(): `let slice = std::slice::from_raw_parts(ptr, 100000);` same oversized-slice construction; printLine(): `take_while(|&&c| c != 0)` depends on encountering a NUL before reading invalid memory; comment in bad(): "Preserve the original \"oversized slice\" behavior; printLine stops at NUL."

**Implementation:** Avoid fabricating oversized slices; instead return/accept `&std::ffi::CStr` (or `CString`) and print via `CStr::to_string_lossy()`. If raw pointers must be used, wrap them in a type that can only be constructed alongside a proven length/capability (e.g., `struct BoundedPtr { ptr: NonNull<i8>, len: usize }`) and only create slices using the correct `len` (here, 17 or 19).

---

