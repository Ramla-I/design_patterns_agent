# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 2
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 1. C-string slice validity protocol (NUL-terminated, readable, non-empty)

**Location**: `/data/test_case/lib.rs:1-97`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `open_with_cleanup` assumes `filename` is a valid C string (NUL-terminated) and that `filename.as_ptr()` points to readable memory. This is not enforced by the type system: `&[i8]` can be empty (as passed when `filename.is_null()` in `driver`), not NUL-terminated, or not actually backed by at least one readable byte. Passing an empty slice or non-NUL-terminated buffer into `CStr::from_ptr(filename.as_ptr())` is UB and/or can read past bounds. The API relies on `unsafe` and caller discipline instead of a type that proves C string validity.

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

// === goto.rs ===

pub(crate) fn forward_goto_example(x: i32) -> i32 {
    if x < 0 {
        eprintln!("Error: negative input");
        return -1;
    }
    println!("Processing: {x}");
    x * 2
}

pub(crate) unsafe fn open_with_cleanup(filename: &[i8]) -> Option<std::fs::File> {
    // Interpret `filename` as a C string (NUL-terminated).
    let c_filename = std::ffi::CStr::from_ptr(filename.as_ptr());
    let path = match c_filename.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: opening or processing file <invalid utf-8>");
            return None;
        }
    };

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error: opening or processing file {path}");
            return None;
        }
    };

    let mut reader = std::io::BufReader::new(file);
    let mut error: i32 = 0;
    let mut buffer = [0i8; 100];

    // Read line-by-line using Rust I/O (avoids dependency on missing `crate::c_lib::rs_fgets`).
    loop {
        buffer.fill(0);

        let mut line = String::new();
        match std::io::BufRead::read_line(&mut reader, &mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Mimic C fgets behavior: print the line as-is.
                print!("{line}");
            }
            Err(_) => {
                error = 1;
                break;
            }
        }
    }

    if error == 0 {
        Some(reader.into_inner())
    } else {
        eprintln!("Error: opening or processing file {path}");
        None
    }
}

pub(crate) unsafe fn driver_internal(num: i32, filename: &[i8]) -> i32 {
    let res = forward_goto_example(num);
    if res == -1 {
        return -1;
    }
    println!("Goto output: {res}");

    let mut out = match open_with_cleanup(filename) {
        Some(f) => f,
        None => return -2,
    };

    let _flush_rc = std::io::Write::flush(&mut out).map_or(-1, |_| 0);
    0
}

#[no_mangle]
pub unsafe extern "C" fn driver(num: i32, filename: *const i8) -> i32 {
    driver_internal(
        num,
        if filename.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(filename, 1024)
        },
    )
}
```

**Entity:** open_with_cleanup(filename: &[i8]) (and its caller driver_internal/driver)

**States:** ValidCStringSlice, InvalidCStringSlice

**Transitions:**
- InvalidCStringSlice -> ValidCStringSlice via validation/construction of a CStr (or CString) before calling open_with_cleanup()

**Evidence:** open_with_cleanup: comment `Interpret `filename` as a C string (NUL-terminated).` documents the required invariant; open_with_cleanup: `let c_filename = std::ffi::CStr::from_ptr(filename.as_ptr());` requires a valid NUL-terminated pointer; not guaranteed by `&[i8]`; driver: `if filename.is_null() { &[] } else { std::slice::from_raw_parts(filename, 1024) }` can pass an empty slice to open_with_cleanup; `&[].as_ptr()` is not a valid C string pointer

**Implementation:** Expose a safe wrapper type like `struct ValidCStr<'a>(&'a std::ffi::CStr);` and change `open_with_cleanup` to take `&CStr` (or `ValidCStr`). In `driver`, perform pointer checks and conversion once: if `filename` is null return an error; otherwise `let c = CStr::from_ptr(filename);` and pass `c` downward. This makes the precondition explicit and removes the need for `&[i8]` guessing.

---

## Protocol Invariants

### 2. Error-code protocol for control flow (-1 / -2 / 0) across calls

**Location**: `/data/test_case/lib.rs:1-97`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `driver_internal` encodes a multi-step protocol using sentinel integer return codes: `forward_goto_example` returns `-1` on negative input; `open_with_cleanup` returns `None` on failure which `driver_internal` maps to `-2`; otherwise success is `0`. The correctness of the calling code depends on remembering and propagating these special values (temporal dependency: check `res == -1` before proceeding). The type system does not distinguish these outcomes, so misuse (e.g., forgetting to check `-1`, mixing up `-1` vs `-2`) is possible.

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

// === goto.rs ===

pub(crate) fn forward_goto_example(x: i32) -> i32 {
    if x < 0 {
        eprintln!("Error: negative input");
        return -1;
    }
    println!("Processing: {x}");
    x * 2
}

pub(crate) unsafe fn open_with_cleanup(filename: &[i8]) -> Option<std::fs::File> {
    // Interpret `filename` as a C string (NUL-terminated).
    let c_filename = std::ffi::CStr::from_ptr(filename.as_ptr());
    let path = match c_filename.to_str() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: opening or processing file <invalid utf-8>");
            return None;
        }
    };

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error: opening or processing file {path}");
            return None;
        }
    };

    let mut reader = std::io::BufReader::new(file);
    let mut error: i32 = 0;
    let mut buffer = [0i8; 100];

    // Read line-by-line using Rust I/O (avoids dependency on missing `crate::c_lib::rs_fgets`).
    loop {
        buffer.fill(0);

        let mut line = String::new();
        match std::io::BufRead::read_line(&mut reader, &mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Mimic C fgets behavior: print the line as-is.
                print!("{line}");
            }
            Err(_) => {
                error = 1;
                break;
            }
        }
    }

    if error == 0 {
        Some(reader.into_inner())
    } else {
        eprintln!("Error: opening or processing file {path}");
        None
    }
}

pub(crate) unsafe fn driver_internal(num: i32, filename: &[i8]) -> i32 {
    let res = forward_goto_example(num);
    if res == -1 {
        return -1;
    }
    println!("Goto output: {res}");

    let mut out = match open_with_cleanup(filename) {
        Some(f) => f,
        None => return -2,
    };

    let _flush_rc = std::io::Write::flush(&mut out).map_or(-1, |_| 0);
    0
}

#[no_mangle]
pub unsafe extern "C" fn driver(num: i32, filename: *const i8) -> i32 {
    driver_internal(
        num,
        if filename.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(filename, 1024)
        },
    )
}
```

**Entity:** driver(num: i32, filename: *const i8) / driver_internal(num: i32, filename: &[i8])

**States:** Ok, NegativeInput, FileOpenOrReadFailed

**Transitions:**
- Ok -> NegativeInput via forward_goto_example() returning -1
- Ok -> FileOpenOrReadFailed via open_with_cleanup() returning None (mapped to -2)
- Ok -> Ok via successful forward_goto_example() and open_with_cleanup()

**Evidence:** forward_goto_example: `if x < 0 { ...; return -1; }` uses -1 as an error sentinel; driver_internal: `let res = forward_goto_example(num); if res == -1 { return -1; }` relies on that sentinel to decide whether to continue; driver_internal: `match open_with_cleanup(filename) { Some(f) => f, None => return -2 }` introduces a second distinct error code (-2); open_with_cleanup: multiple `return None;` sites with error logging `Error: opening or processing file ...` indicate a distinct failure mode being encoded via Option

**Implementation:** Replace the `i32` error-code protocol with `Result<Success, DriverError>` where `enum DriverError { NegativeInput, FileOpenOrReadFailed }`. For the C ABI boundary, map `DriverError` to the desired numeric codes only in `driver` (the extern "C" function), keeping the internal API type-safe.

---

