# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 2
- **Modules analyzed**: 2

## Precondition Invariants

### 1. C-string slice validity precondition (NUL-terminated within bounds, valid UTF-8)

**Location**: `/data/test_case/lib.rs:1-109`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: printLine assumes that the provided &[i8] is a valid C string buffer: it must contain a NUL terminator within the slice bounds and the bytes up to the NUL must be valid UTF-8. These requirements are not represented in the type system (it accepts any &[i8]) and are instead enforced by panicking unwrap() calls at runtime.

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

// === driver.rs ===
pub(crate) fn printLine(line: &[i8]) {
    if !line.is_empty() {
        println!(
            "{0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(line))
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}

fn helperBad() -> *mut i8 {
    let mut charString: [i8; 17] = [
        b'h' as i8,
        b'e' as i8,
        b'l' as i8,
        b'p' as i8,
        b'e' as i8,
        b'r' as i8,
        b'B' as i8,
        b'a' as i8,
        b'd' as i8,
        b' ' as i8,
        b's' as i8,
        b't' as i8,
        b'r' as i8,
        b'i' as i8,
        b'n' as i8,
        b'g' as i8,
        b'\0' as i8,
    ];
    charString.as_mut_ptr()
}

pub(crate) unsafe fn bad() {
    printLine({
        let _x = helperBad();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

fn helperGood1() -> *mut i8 {
    thread_local! {
        static charString: std::cell::RefCell<[i8; 19]> = const {
            std::cell::RefCell::new([
                b'h' as i8,
                b'e' as i8,
                b'l' as i8,
                b'p' as i8,
                b'e' as i8,
                b'r' as i8,
                b'G' as i8,
                b'o' as i8,
                b'o' as i8,
                b'd' as i8,
                b'1' as i8,
                b' ' as i8,
                b's' as i8,
                b't' as i8,
                b'r' as i8,
                b'i' as i8,
                b'n' as i8,
                b'g' as i8,
                b'\0' as i8,
            ])
        };
    };

    // Return a stable pointer to the thread-local buffer without borrowing it.
    // This matches the intended C behavior (pointer remains valid for the thread).
    charString.with(|cell| cell.as_ptr() as *mut i8)
}

pub(crate) unsafe fn good() {
    printLine({
        let _x = helperGood1();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn driver(useGood: i32) {
    if useGood != 0 {
        good();
    } else {
        bad();
    };
}
```

**Entity:** printLine(line: &[i8])

**States:** ValidCStringSlice, InvalidCStringSlice

**Transitions:**
- InvalidCStringSlice -> panic via unwrap() chain in printLine

**Evidence:** printLine signature: pub(crate) fn printLine(line: &[i8]) accepts arbitrary i8 slice (no CString/CStr wrapper); printLine body: CStr::from_bytes_until_nul(bytemuck::cast_slice(line)).unwrap() requires a NUL within bounds; printLine body: .to_str().unwrap() requires UTF-8 content

**Implementation:** Change API to accept &CStr (or a newtype like struct NulTerminatedI8Slice<'a>(&'a [i8]) with a constructor that checks for an in-bounds NUL). If UTF-8 is required, accept &str or a validated newtype that performs CStr::to_str() once at the boundary.

---

## Protocol Invariants

### 3. Thread-local buffer access protocol (must be NUL-terminated and not concurrently mutably borrowed when used as C string)

**Location**: `/data/test_case/lib.rs:1-109`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: helperGood1 returns a raw pointer into a thread-local RefCell buffer by using RefCell::as_ptr() and casting it to *mut i8. The intended protocol is that this pointer remains valid for the thread and points to a NUL-terminated C string. However, the type system does not enforce (a) that the contents remain NUL-terminated, (b) that no one mutably borrows/modifies the RefCell while the raw pointer is being used as a C string, or (c) that consumers only read within the actual buffer length (good() still builds a 100000-length slice).

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

// === driver.rs ===
pub(crate) fn printLine(line: &[i8]) {
    if !line.is_empty() {
        println!(
            "{0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(line))
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}

fn helperBad() -> *mut i8 {
    let mut charString: [i8; 17] = [
        b'h' as i8,
        b'e' as i8,
        b'l' as i8,
        b'p' as i8,
        b'e' as i8,
        b'r' as i8,
        b'B' as i8,
        b'a' as i8,
        b'd' as i8,
        b' ' as i8,
        b's' as i8,
        b't' as i8,
        b'r' as i8,
        b'i' as i8,
        b'n' as i8,
        b'g' as i8,
        b'\0' as i8,
    ];
    charString.as_mut_ptr()
}

pub(crate) unsafe fn bad() {
    printLine({
        let _x = helperBad();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

fn helperGood1() -> *mut i8 {
    thread_local! {
        static charString: std::cell::RefCell<[i8; 19]> = const {
            std::cell::RefCell::new([
                b'h' as i8,
                b'e' as i8,
                b'l' as i8,
                b'p' as i8,
                b'e' as i8,
                b'r' as i8,
                b'G' as i8,
                b'o' as i8,
                b'o' as i8,
                b'd' as i8,
                b'1' as i8,
                b' ' as i8,
                b's' as i8,
                b't' as i8,
                b'r' as i8,
                b'i' as i8,
                b'n' as i8,
                b'g' as i8,
                b'\0' as i8,
            ])
        };
    };

    // Return a stable pointer to the thread-local buffer without borrowing it.
    // This matches the intended C behavior (pointer remains valid for the thread).
    charString.with(|cell| cell.as_ptr() as *mut i8)
}

pub(crate) unsafe fn good() {
    printLine({
        let _x = helperGood1();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn driver(useGood: i32) {
    if useGood != 0 {
        good();
    } else {
        bad();
    };
}
```

**Entity:** helperGood1() -> *mut i8 / good()

**States:** TLSBufferStable, TLSBufferInvalidatedOrNonNulTerminated

**Transitions:**
- TLSBufferStable -> TLSBufferInvalidatedOrNonNulTerminated via any mutation of the thread-local RefCell contents that removes/moves the NUL terminator
- TLSBufferStable -> UB via std::slice::from_raw_parts(_x, 100000) if read exceeds the 19-byte TLS buffer

**Evidence:** helperGood1: thread_local! static charString: RefCell<[i8; 19]>; helperGood1 comment: "Return a stable pointer to the thread-local buffer without borrowing it" indicates an implicit lifetime/protocol assumption; helperGood1: charString.with(|cell| cell.as_ptr() as *mut i8) exposes raw pointer without an active borrow; good: std::slice::from_raw_parts(_x, 100000) assumes pointer valid for far more than the TLS buffer size (19); buffer literal in helperGood1 includes '\0' terminator, implying required invariant: NUL-terminated

**Implementation:** Expose a safe accessor that returns a capability tied to a borrow, e.g. charString.with(|cell| { let borrow = cell.borrow(); let cstr = CStr::from_bytes_until_nul(bytemuck::cast_slice(&*borrow))?; ... }) so the borrow guards against concurrent mutation while the data is used. Alternatively, store a &'static CStr in TLS (or a newtype guaranteeing NUL termination) and return *const c_char with an API that also returns the correct length.

---

### 2. Pointer provenance & lifetime protocol (pointer must outlive its use)

**Location**: `/data/test_case/lib.rs:1-109`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: bad() implicitly relies on helperBad() returning a pointer to memory that remains valid long enough to build a slice and print it. In reality helperBad() returns a pointer to a stack-allocated array, which becomes dangling as soon as helperBad returns. The type system cannot express the required lifetime/provenance constraints because a raw pointer is returned and later turned into a slice with from_raw_parts.

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

// === driver.rs ===
pub(crate) fn printLine(line: &[i8]) {
    if !line.is_empty() {
        println!(
            "{0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(line))
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}

fn helperBad() -> *mut i8 {
    let mut charString: [i8; 17] = [
        b'h' as i8,
        b'e' as i8,
        b'l' as i8,
        b'p' as i8,
        b'e' as i8,
        b'r' as i8,
        b'B' as i8,
        b'a' as i8,
        b'd' as i8,
        b' ' as i8,
        b's' as i8,
        b't' as i8,
        b'r' as i8,
        b'i' as i8,
        b'n' as i8,
        b'g' as i8,
        b'\0' as i8,
    ];
    charString.as_mut_ptr()
}

pub(crate) unsafe fn bad() {
    printLine({
        let _x = helperBad();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

fn helperGood1() -> *mut i8 {
    thread_local! {
        static charString: std::cell::RefCell<[i8; 19]> = const {
            std::cell::RefCell::new([
                b'h' as i8,
                b'e' as i8,
                b'l' as i8,
                b'p' as i8,
                b'e' as i8,
                b'r' as i8,
                b'G' as i8,
                b'o' as i8,
                b'o' as i8,
                b'd' as i8,
                b'1' as i8,
                b' ' as i8,
                b's' as i8,
                b't' as i8,
                b'r' as i8,
                b'i' as i8,
                b'n' as i8,
                b'g' as i8,
                b'\0' as i8,
            ])
        };
    };

    // Return a stable pointer to the thread-local buffer without borrowing it.
    // This matches the intended C behavior (pointer remains valid for the thread).
    charString.with(|cell| cell.as_ptr() as *mut i8)
}

pub(crate) unsafe fn good() {
    printLine({
        let _x = helperGood1();
        if _x.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(_x, 100000)
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn driver(useGood: i32) {
    if useGood != 0 {
        good();
    } else {
        bad();
    };
}
```

**Entity:** helperBad() -> *mut i8 / bad()

**States:** PointerValid, PointerDangling

**Transitions:**
- PointerValid -> PointerDangling via returning from helperBad() (stack frame ends)
- PointerDangling -> UB via std::slice::from_raw_parts(_x, 100000) in bad()

**Evidence:** helperBad: let mut charString: [i8; 17] = [...] creates stack buffer; helperBad: charString.as_mut_ptr() returns raw pointer to stack buffer; bad: let _x = helperBad(); then uses _x after helperBad has returned; bad: std::slice::from_raw_parts(_x, 100000) assumes pointer is valid for 100000 elements

**Implementation:** Return an owning buffer (e.g., [i8; 17], Vec<i8>, or CString) or a reference with a lifetime (fn helper_bad<'a>(buf: &'a mut [i8; 17]) -> &'a mut [i8]) so the borrow checker enforces that the storage outlives its use. If a stable pointer is required, use a pinned/owned allocation and return a wrapper that owns it and exposes as_ptr().

---

