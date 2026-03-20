# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 1
- **Modules analyzed**: 2

## Precondition Invariants

### 1. C-string slice protocol for process_strings (valid pointers + NUL-terminated target)

**Location**: `/data/test_case/lib.rs:1-170`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: process_strings relies on a bundle of FFI/string validity preconditions that are only partially checked at runtime. It assumes: (1) each element pointer in `strings[..limit]` is either NULL (skipped) or points to a readable NUL-terminated C string (at least first byte readable, and enough readable bytes for `strncmp` up to `target_len`), and (2) `target` contains a NUL terminator so `CStr::from_bytes_until_nul` succeeds. If these are violated, behavior ranges from filtering out NULL/empty strings to panicking (`unwrap()`) or undefined behavior from reading invalid memory in `*(s as *const i8)` / calling `strncmp` on non-C-strings. None of these requirements are expressed in the type system: raw pointers and `&[i8]` allow invalid states.

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

extern "C" {
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn strncmp(__s1: *const i8, __s2: *const i8, __n: usize) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub i: i32,
    pub f: f32,
}

fn memchra(buf: &[i8], c: i32, n: usize) -> i32 {
    let needle = c as i8;
    buf.iter()
        .take(n.min(buf.len()))
        .filter(|&&b| b == needle)
        .count() as i32
}

fn process_buffer(buffer: &[i8], len: usize) -> i32 {
    if buffer.is_empty() || buffer[0] == 0 {
        return -1;
    }
    buffer
        .iter()
        .take(len.min(buffer.len()))
        .take_while(|&&b| b != 0)
        .map(|&b| b as i32)
        .sum()
}

unsafe fn int_to_float_bits(value: i32) -> f32 {
    // Preserve original union-based bit reinterpretation.
    let mut converter: C2RustUnnamed = C2RustUnnamed { i: 0 };
    converter.i = value;
    converter.f
}

unsafe fn process_strings(strings: &[*mut i8], count: i32, target: &[i8]) -> i32 {
    if strings.is_empty() || count <= 0 {
        return 0;
    }

    let limit = (count as usize).min(strings.len());

    // Determine comparison length: C-string length of `target` up to NUL.
    let target_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(target))
        .unwrap()
        .count_bytes();

    strings
        .iter()
        .take(limit)
        .filter(|&&s| {
            if s.is_null() {
                return false;
            }
            // Check first byte not NUL (as original did).
            if unsafe { *(s as *const i8) } == 0 {
                return false;
            }
            unsafe { strncmp(s as *const i8, target.as_ptr(), target_len) == 0 }
        })
        .count() as i32
}

fn safe_sum_array(arr: &[i32], size: usize) -> i32 {
    if arr.is_empty() || size == 0 {
        return 0;
    }
    arr.iter().take(size.min(arr.len())).copied().sum()
}

unsafe fn interpret_as_int(bytes: Option<&u8>, len: usize) -> i32 {
    if bytes.is_none() || len < core::mem::size_of::<i32>() {
        return 0;
    }
    let p = bytes.unwrap() as *const u8 as *const i32;
    // Preserve original behavior (potentially unaligned read).
    core::ptr::read_unaligned(p)
}

fn count_occurrences(text: &[i8], ch: i8) -> i32 {
    if text.is_empty() || text[0] == 0 {
        return 0;
    }
    let len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(text))
        .unwrap()
        .count_bytes();
    memchra(text, ch as i32, len)
}

fn complex_iteration(data: &[i32], count: usize) -> i32 {
    if data.is_empty() || count == 0 {
        return -1;
    }
    data.iter()
        .take(count.min(data.len()))
        .fold(0i32, |acc, &v| acc ^ ((v as u32 & 0xff) as i32))
}

#[no_mangle]
pub unsafe extern "C" fn memchra2(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut result: i32 = 0;

    let mut buffer: [i8; 64] = [0; 64];
    snprintf(
        buffer.as_mut_ptr(),
        core::mem::size_of::<[i8; 64]>(),
        b"test%d-%d-%d-%d\0" as *const u8 as *const i8,
        a,
        b,
        c,
        d,
    );

    let dash_count: i32 = count_occurrences(&buffer, b'-' as i8);
    result += dash_count * 10;

    let values: [i32; 4] = [a, b, c, d];
    let sum: i32 = safe_sum_array(&values, 4);
    result += sum;

    let test_strings: [*mut i8; 4] = [
        b"test1\0" as *const u8 as *const i8 as *mut i8,
        b"test2\0" as *const u8 as *const i8 as *mut i8,
        b"testing\0" as *const u8 as *const i8 as *mut i8,
        b"other\0" as *const u8 as *const i8 as *mut i8,
    ];
    let matches: i32 = process_strings(&test_strings, 4, bytemuck::cast_slice(b"test\0"));
    result += matches * 5;

    let f: f32 = int_to_float_bits(a);
    if f > 0.0f32 && f < 1000.0f32 {
        result += f as i32;
    }

    let buf_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
        .unwrap()
        .count_bytes();
    let buf_sum: i32 = process_buffer(&buffer, buf_len);
    if buf_sum > 0 {
        result += buf_sum % 256;
    }

    let mut bytes: [u8; 4] = [0; 4];
    bytes[0] = (b & 0xff) as u8;
    bytes[1] = (c & 0xff) as u8;
    bytes[2] = (d & 0xff) as u8;
    bytes[3] = 0;

    let interpreted: i32 = interpret_as_int(Some(&bytes[0]), 4);
    result ^= interpreted;

    let complex_result: i32 = complex_iteration(&values, 4);
    result += complex_result;

    result
}
```

**Entity:** process_strings(strings: &[*mut i8], count: i32, target: &[i8])

**States:** InvalidInputs, ValidInputs

**Transitions:**
- InvalidInputs -> ValidInputs via constructing validated inputs (CStr/CString/NonNull) before calling process_strings

**Evidence:** process_strings: `if strings.is_empty() || count <= 0 { return 0; }` gates behavior on runtime checks; process_strings: `let limit = (count as usize).min(strings.len());` mixes signed `count` with slice length (implicit contract that count matches number of usable pointers); process_strings: `CStr::from_bytes_until_nul(bytemuck::cast_slice(target)).unwrap()` requires `target` to contain a NUL byte; otherwise panics; process_strings: `if s.is_null() { return false; }` raw pointer nullability handled at runtime; process_strings: `if unsafe { *(s as *const i8) } == 0 { return false; }` dereferences raw pointer (requires readable memory); process_strings: `unsafe { strncmp(s as *const i8, target.as_ptr(), target_len) == 0 }` assumes `s` points to a valid C string readable for `target_len` bytes

**Implementation:** Change the API to accept validated string types: `strings: &[Option<core::ptr::NonNull<c_char>>]` (or `&[&CStr]` if ownership/lifetimes allow) and `target: &CStr`. This makes nullability explicit and enforces NUL-termination at construction. If `count` is required by a C ABI, wrap it in a `CountWithinSlice` newtype produced by a checked constructor tying it to `strings.len()`.

---

### 2. Byte buffer validity protocol for interpret_as_int (present + >=4 bytes + alignment/aliasing assumptions)

**Location**: `/data/test_case/lib.rs:1-170`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: interpret_as_int implicitly requires a present pointer to at least 4 bytes of memory. It enforces presence and length via runtime checks, then performs an unaligned i32 read. The type signature `Option<&u8>` cannot express the requirement that the referenced memory extends for `len` bytes (and is a coherent byte buffer). As written, callers can pass `Some(&u8)` with an arbitrary `len`, and only `len` is checked; the actual backing allocation size is not tracked by the type system.

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

extern "C" {
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn strncmp(__s1: *const i8, __s2: *const i8, __n: usize) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub i: i32,
    pub f: f32,
}

fn memchra(buf: &[i8], c: i32, n: usize) -> i32 {
    let needle = c as i8;
    buf.iter()
        .take(n.min(buf.len()))
        .filter(|&&b| b == needle)
        .count() as i32
}

fn process_buffer(buffer: &[i8], len: usize) -> i32 {
    if buffer.is_empty() || buffer[0] == 0 {
        return -1;
    }
    buffer
        .iter()
        .take(len.min(buffer.len()))
        .take_while(|&&b| b != 0)
        .map(|&b| b as i32)
        .sum()
}

unsafe fn int_to_float_bits(value: i32) -> f32 {
    // Preserve original union-based bit reinterpretation.
    let mut converter: C2RustUnnamed = C2RustUnnamed { i: 0 };
    converter.i = value;
    converter.f
}

unsafe fn process_strings(strings: &[*mut i8], count: i32, target: &[i8]) -> i32 {
    if strings.is_empty() || count <= 0 {
        return 0;
    }

    let limit = (count as usize).min(strings.len());

    // Determine comparison length: C-string length of `target` up to NUL.
    let target_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(target))
        .unwrap()
        .count_bytes();

    strings
        .iter()
        .take(limit)
        .filter(|&&s| {
            if s.is_null() {
                return false;
            }
            // Check first byte not NUL (as original did).
            if unsafe { *(s as *const i8) } == 0 {
                return false;
            }
            unsafe { strncmp(s as *const i8, target.as_ptr(), target_len) == 0 }
        })
        .count() as i32
}

fn safe_sum_array(arr: &[i32], size: usize) -> i32 {
    if arr.is_empty() || size == 0 {
        return 0;
    }
    arr.iter().take(size.min(arr.len())).copied().sum()
}

unsafe fn interpret_as_int(bytes: Option<&u8>, len: usize) -> i32 {
    if bytes.is_none() || len < core::mem::size_of::<i32>() {
        return 0;
    }
    let p = bytes.unwrap() as *const u8 as *const i32;
    // Preserve original behavior (potentially unaligned read).
    core::ptr::read_unaligned(p)
}

fn count_occurrences(text: &[i8], ch: i8) -> i32 {
    if text.is_empty() || text[0] == 0 {
        return 0;
    }
    let len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(text))
        .unwrap()
        .count_bytes();
    memchra(text, ch as i32, len)
}

fn complex_iteration(data: &[i32], count: usize) -> i32 {
    if data.is_empty() || count == 0 {
        return -1;
    }
    data.iter()
        .take(count.min(data.len()))
        .fold(0i32, |acc, &v| acc ^ ((v as u32 & 0xff) as i32))
}

#[no_mangle]
pub unsafe extern "C" fn memchra2(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut result: i32 = 0;

    let mut buffer: [i8; 64] = [0; 64];
    snprintf(
        buffer.as_mut_ptr(),
        core::mem::size_of::<[i8; 64]>(),
        b"test%d-%d-%d-%d\0" as *const u8 as *const i8,
        a,
        b,
        c,
        d,
    );

    let dash_count: i32 = count_occurrences(&buffer, b'-' as i8);
    result += dash_count * 10;

    let values: [i32; 4] = [a, b, c, d];
    let sum: i32 = safe_sum_array(&values, 4);
    result += sum;

    let test_strings: [*mut i8; 4] = [
        b"test1\0" as *const u8 as *const i8 as *mut i8,
        b"test2\0" as *const u8 as *const i8 as *mut i8,
        b"testing\0" as *const u8 as *const i8 as *mut i8,
        b"other\0" as *const u8 as *const i8 as *mut i8,
    ];
    let matches: i32 = process_strings(&test_strings, 4, bytemuck::cast_slice(b"test\0"));
    result += matches * 5;

    let f: f32 = int_to_float_bits(a);
    if f > 0.0f32 && f < 1000.0f32 {
        result += f as i32;
    }

    let buf_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
        .unwrap()
        .count_bytes();
    let buf_sum: i32 = process_buffer(&buffer, buf_len);
    if buf_sum > 0 {
        result += buf_sum % 256;
    }

    let mut bytes: [u8; 4] = [0; 4];
    bytes[0] = (b & 0xff) as u8;
    bytes[1] = (c & 0xff) as u8;
    bytes[2] = (d & 0xff) as u8;
    bytes[3] = 0;

    let interpreted: i32 = interpret_as_int(Some(&bytes[0]), 4);
    result ^= interpreted;

    let complex_result: i32 = complex_iteration(&values, 4);
    result += complex_result;

    result
}
```

**Entity:** interpret_as_int(bytes: Option<&u8>, len: usize)

**States:** NoDataOrTooShort, SufficientBytes

**Transitions:**
- NoDataOrTooShort -> SufficientBytes via providing a real byte slice of length >= 4

**Evidence:** interpret_as_int: `if bytes.is_none() || len < core::mem::size_of::<i32>() { return 0; }` runtime check encodes the precondition; interpret_as_int: `let p = bytes.unwrap() as *const u8 as *const i32;` erases provenance from a single byte reference into an i32 pointer; interpret_as_int: `core::ptr::read_unaligned(p)` relies on the existence of 4 readable bytes starting at `p`

**Implementation:** Replace `(Option<&u8>, len)` with `Option<&[u8]>` (or just `&[u8]` if `None` isn’t needed). Then require `bytes.len() >= 4` at the call site using a constructor like `struct AtLeast4<'a>(&'a [u8]); impl<'a> TryFrom<&'a [u8]> for AtLeast4<'a> { ... }`, and implement `fn interpret_as_int(buf: AtLeast4) -> i32` using `read_unaligned` on `buf.0.as_ptr()`.

---

## Protocol Invariants

### 3. NUL-terminated buffer protocol for CStr-based scanning (initialized-by-snprintf -> scan until NUL)

**Location**: `/data/test_case/lib.rs:1-170`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Several functions treat `&[i8]` buffers as C strings: they assume there is a NUL terminator within the slice and that the content is initialized up to that terminator. This is relied on when computing lengths via `CStr::from_bytes_until_nul(...).unwrap()` and then iterating up to that length. In memchra2, the buffer becomes a 'C string' only after `snprintf` has been called; this temporal dependency is not represented in types, and failures to NUL-terminate (or passing arbitrary non-NUL-terminated `&[i8]` to count_occurrences) will panic at runtime due to `unwrap()`.

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

extern "C" {
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn strncmp(__s1: *const i8, __s2: *const i8, __n: usize) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub i: i32,
    pub f: f32,
}

fn memchra(buf: &[i8], c: i32, n: usize) -> i32 {
    let needle = c as i8;
    buf.iter()
        .take(n.min(buf.len()))
        .filter(|&&b| b == needle)
        .count() as i32
}

fn process_buffer(buffer: &[i8], len: usize) -> i32 {
    if buffer.is_empty() || buffer[0] == 0 {
        return -1;
    }
    buffer
        .iter()
        .take(len.min(buffer.len()))
        .take_while(|&&b| b != 0)
        .map(|&b| b as i32)
        .sum()
}

unsafe fn int_to_float_bits(value: i32) -> f32 {
    // Preserve original union-based bit reinterpretation.
    let mut converter: C2RustUnnamed = C2RustUnnamed { i: 0 };
    converter.i = value;
    converter.f
}

unsafe fn process_strings(strings: &[*mut i8], count: i32, target: &[i8]) -> i32 {
    if strings.is_empty() || count <= 0 {
        return 0;
    }

    let limit = (count as usize).min(strings.len());

    // Determine comparison length: C-string length of `target` up to NUL.
    let target_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(target))
        .unwrap()
        .count_bytes();

    strings
        .iter()
        .take(limit)
        .filter(|&&s| {
            if s.is_null() {
                return false;
            }
            // Check first byte not NUL (as original did).
            if unsafe { *(s as *const i8) } == 0 {
                return false;
            }
            unsafe { strncmp(s as *const i8, target.as_ptr(), target_len) == 0 }
        })
        .count() as i32
}

fn safe_sum_array(arr: &[i32], size: usize) -> i32 {
    if arr.is_empty() || size == 0 {
        return 0;
    }
    arr.iter().take(size.min(arr.len())).copied().sum()
}

unsafe fn interpret_as_int(bytes: Option<&u8>, len: usize) -> i32 {
    if bytes.is_none() || len < core::mem::size_of::<i32>() {
        return 0;
    }
    let p = bytes.unwrap() as *const u8 as *const i32;
    // Preserve original behavior (potentially unaligned read).
    core::ptr::read_unaligned(p)
}

fn count_occurrences(text: &[i8], ch: i8) -> i32 {
    if text.is_empty() || text[0] == 0 {
        return 0;
    }
    let len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(text))
        .unwrap()
        .count_bytes();
    memchra(text, ch as i32, len)
}

fn complex_iteration(data: &[i32], count: usize) -> i32 {
    if data.is_empty() || count == 0 {
        return -1;
    }
    data.iter()
        .take(count.min(data.len()))
        .fold(0i32, |acc, &v| acc ^ ((v as u32 & 0xff) as i32))
}

#[no_mangle]
pub unsafe extern "C" fn memchra2(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut result: i32 = 0;

    let mut buffer: [i8; 64] = [0; 64];
    snprintf(
        buffer.as_mut_ptr(),
        core::mem::size_of::<[i8; 64]>(),
        b"test%d-%d-%d-%d\0" as *const u8 as *const i8,
        a,
        b,
        c,
        d,
    );

    let dash_count: i32 = count_occurrences(&buffer, b'-' as i8);
    result += dash_count * 10;

    let values: [i32; 4] = [a, b, c, d];
    let sum: i32 = safe_sum_array(&values, 4);
    result += sum;

    let test_strings: [*mut i8; 4] = [
        b"test1\0" as *const u8 as *const i8 as *mut i8,
        b"test2\0" as *const u8 as *const i8 as *mut i8,
        b"testing\0" as *const u8 as *const i8 as *mut i8,
        b"other\0" as *const u8 as *const i8 as *mut i8,
    ];
    let matches: i32 = process_strings(&test_strings, 4, bytemuck::cast_slice(b"test\0"));
    result += matches * 5;

    let f: f32 = int_to_float_bits(a);
    if f > 0.0f32 && f < 1000.0f32 {
        result += f as i32;
    }

    let buf_len = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
        .unwrap()
        .count_bytes();
    let buf_sum: i32 = process_buffer(&buffer, buf_len);
    if buf_sum > 0 {
        result += buf_sum % 256;
    }

    let mut bytes: [u8; 4] = [0; 4];
    bytes[0] = (b & 0xff) as u8;
    bytes[1] = (c & 0xff) as u8;
    bytes[2] = (d & 0xff) as u8;
    bytes[3] = 0;

    let interpreted: i32 = interpret_as_int(Some(&bytes[0]), 4);
    result ^= interpreted;

    let complex_result: i32 = complex_iteration(&values, 4);
    result += complex_result;

    result
}
```

**Entity:** C-string buffer handling in memchra2 / count_occurrences / process_buffer

**States:** NonCStrBuffer, CStrBuffer

**Transitions:**
- NonCStrBuffer -> CStrBuffer via snprintf(...) writing a NUL terminator (or by constructing a CStr/CString)

**Evidence:** memchra2: `let mut buffer: [i8; 64] = [0; 64];` then `snprintf(buffer.as_mut_ptr(), ..., b"test%d-%d-%d-%d\0" ...)` implicitly establishes C-string formatting/termination before later scans; count_occurrences: `CStr::from_bytes_until_nul(bytemuck::cast_slice(text)).unwrap()` requires a NUL terminator; panics otherwise; memchra2: `let buf_len = CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer)).unwrap().count_bytes();` assumes snprintf produced a NUL within 64 bytes; process_buffer: treats `len` as a C-string length boundary (`take_while(|&&b| b != 0)`), relying on NUL-terminated semantics; process_buffer: `if buffer.is_empty() || buffer[0] == 0 { return -1; }` runtime guard encodes 'non-empty C string' precondition

**Implementation:** Introduce a wrapper like `struct NulTerminatedI8<'a>(&'a [i8]);` constructed via a checked function that searches for NUL (or by converting from `&CStr`/`CString`). Make `count_occurrences`/`process_buffer` accept `&CStr` (or `NulTerminatedI8`) instead of `&[i8]`, and in `memchra2` convert the snprintf result into a `CStr` view using `CStr::from_ptr(buffer.as_ptr())` only after ensuring termination (or check snprintf’s return value and enforce bounds).

---

