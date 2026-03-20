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

### 3. FFI buffer size and NUL-termination expectations for input string

**Location**: `/data/test_case/lib.rs:1-211`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `driver` converts a raw C pointer `in_0: *mut i8` into a Rust slice of fixed length 1024 via `from_raw_parts_mut`, regardless of the actual allocation size. For non-null inputs, correctness implicitly requires that `in_0` points to at least 1024 writable bytes, and that within those bytes there is a NUL terminator so that `strtol_local`'s terminator scan will not read out of bounds. These are FFI-level preconditions not represented in types; the code only checks for null and otherwise assumes the buffer contract.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 4 free function(s)

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
        use core::ptr;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct house_t {
            pub floors: i32,
            pub bedrooms: i32,
            pub bathrooms: f64,
        }

        fn add_floor(house: &mut house_t) {
            house.floors += 1;
        }

        fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
            house.bedrooms += extra_bedrooms;
        }

        fn print_house(house: &house_t) {
            println!(
                "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
                house.floors, house.bedrooms, house.bathrooms
            );
        }

        pub(crate) fn run(the_house: &mut house_t, extra_bedrooms: i32) {
            print_house(the_house);
            add_floor(the_house);
            print_house(the_house);

            the_house.bathrooms += 1.0;
            print_house(the_house);

            add_bedrooms(the_house, extra_bedrooms);
            print_house(the_house);
        }

        // Local replacement for `crate::c_lib::strtol` used by the original C2Rust output.
        // Only base-10 is needed here.
        unsafe fn strtol_local(
            s: *const i8,
            endp: *mut *const u8,
            base: i32,
            mut error: Option<&mut bool>,
        ) -> i64 {
            if let Some(ref mut e) = error {
                **e = false;
            }

            if s.is_null() {
                if !endp.is_null() {
                    *endp = ptr::null();
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            if base != 10 {
                if !endp.is_null() {
                    *endp = s as *const u8;
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            // Find NUL terminator to bound the input.
            let mut len: usize = 0;
            loop {
                let ch = *s.add(len) as u8;
                if ch == 0 {
                    break;
                }
                len += 1;
            }
            let bytes = core::slice::from_raw_parts(s as *const u8, len);

            // Skip leading ASCII whitespace.
            let mut i = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            // Optional sign.
            let mut sign: i64 = 1;
            if i < bytes.len() {
                match bytes[i] {
                    b'+' => i += 1,
                    b'-' => {
                        sign = -1;
                        i += 1;
                    }
                    _ => {}
                }
            }

            let start_digits = i;

            // Accumulate digits with overflow detection.
            let mut acc: i64 = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !(b'0'..=b'9').contains(&b) {
                    break;
                }
                let digit = (b - b'0') as i64;

                match acc.checked_mul(10).and_then(|v| v.checked_add(digit)) {
                    Some(v) => acc = v,
                    None => {
                        if let Some(ref mut e) = error {
                            **e = true;
                        }
                        if !endp.is_null() {
                            *endp = s.add(i) as *const u8;
                        }
                        return if sign < 0 { i64::MIN } else { i64::MAX };
                    }
                }

                i += 1;
            }

            // If no digits were consumed, endp points to original string.
            if !endp.is_null() {
                let end_ptr = if i == start_digits { s } else { s.add(i) };
                *endp = end_ptr as *const u8;
            }

            // Apply sign with overflow check.
            match acc.checked_mul(sign) {
                Some(v) => v,
                None => {
                    if let Some(ref mut e) = error {
                        **e = true;
                    }
                    if sign < 0 { i64::MIN } else { i64::MAX }
                }
            }
        }

        unsafe fn parse_val(str_: *const i8, val: &mut i32) -> bool {
            let mut error0 = false;
            let mut endp: *const u8 = str_ as *const u8;

            let tmp: i64 = strtol_local(str_, &mut endp as *mut *const u8, 10, Some(&mut error0));

            let ok = endp != (str_ as *const u8)
                && !error0
                && tmp >= INT_MIN as i64
                && tmp <= INT_MAX as i64;

            if ok {
                *val = tmp as i32;
                true_0 != 0
            } else {
                false_0 != 0
            }
        }

        pub(crate) unsafe fn driver_internal(in_0: &mut [i8]) {
            let mut x: i32 = 0;

            if parse_val(in_0.as_ptr(), &mut x) {
                let mut the_house = house_t {
                    floors: 2,
                    bedrooms: 5,
                    bathrooms: 2.5,
                };

                run(&mut the_house, x);
                run(&mut the_house, x);
            } else {
                println!("An error occurred");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *mut i8) {
            driver_internal(if in_0.is_null() {
                &mut []
            } else {
                core::slice::from_raw_parts_mut(in_0, 1024)
            })
        }

        pub const true_0: i32 = 1;
        pub const false_0: i32 = 0;
        pub const INT_MAX: i32 = __INT_MAX__;
        pub const INT_MIN: i32 = -__INT_MAX__ - 1;
        pub const __INT_MAX__: i32 = 2147483647;
    }
}
```

**Entity:** driver (pub unsafe extern "C" fn) / driver_internal

**States:** NullOrInvalidBuffer, ValidBuffer(<=1024, readable, NUL-terminated)

**Transitions:**
- NullOrInvalidBuffer -> ValidBuffer by providing a non-null pointer to a >=1024-byte writable region containing a NUL-terminated string before calling driver()

**Evidence:** driver(in_0: *mut i8): `if in_0.is_null() { &mut [] } else { core::slice::from_raw_parts_mut(in_0, 1024) }` assumes non-null implies a valid 1024-byte region; driver_internal(in_0: &mut [i8]): passes `in_0.as_ptr()` to parse_val, which calls strtol_local that scans for NUL via `*s.add(len)`

**Implementation:** Expose a safe Rust entrypoint that accepts `&core::ffi::CStr` (or `&[u8]` with an explicit NUL guarantee type) instead of a raw pointer/guessed length, and keep `driver` as a thin unsafe FFI shim. Alternatively, model the input as `struct InputBuf1024(*mut i8); unsafe fn new(ptr) -> Option<InputBuf1024>` that checks non-null and (where possible) length/termination, then only allow `driver_internal` to accept that capability.

---

### 1. C-string parsing preconditions (non-null, NUL-terminated, base-10-only)

**Location**: `/data/test_case/lib.rs:1-211`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `strtol_local` assumes C-string conventions and uses raw pointers. Correct behavior requires (1) `s` to be non-null, (2) `s` to point to a NUL-terminated buffer readable up to the terminator, and (3) `base` must be 10 (anything else is treated as an error). These requirements are enforced by runtime pointer checks and a `base != 10` branch, not by the type system; the function is `unsafe` but still accepts any `*const i8` and any `i32` base, leaving the protocol implicit.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 4 free function(s)

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
        use core::ptr;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct house_t {
            pub floors: i32,
            pub bedrooms: i32,
            pub bathrooms: f64,
        }

        fn add_floor(house: &mut house_t) {
            house.floors += 1;
        }

        fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
            house.bedrooms += extra_bedrooms;
        }

        fn print_house(house: &house_t) {
            println!(
                "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
                house.floors, house.bedrooms, house.bathrooms
            );
        }

        pub(crate) fn run(the_house: &mut house_t, extra_bedrooms: i32) {
            print_house(the_house);
            add_floor(the_house);
            print_house(the_house);

            the_house.bathrooms += 1.0;
            print_house(the_house);

            add_bedrooms(the_house, extra_bedrooms);
            print_house(the_house);
        }

        // Local replacement for `crate::c_lib::strtol` used by the original C2Rust output.
        // Only base-10 is needed here.
        unsafe fn strtol_local(
            s: *const i8,
            endp: *mut *const u8,
            base: i32,
            mut error: Option<&mut bool>,
        ) -> i64 {
            if let Some(ref mut e) = error {
                **e = false;
            }

            if s.is_null() {
                if !endp.is_null() {
                    *endp = ptr::null();
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            if base != 10 {
                if !endp.is_null() {
                    *endp = s as *const u8;
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            // Find NUL terminator to bound the input.
            let mut len: usize = 0;
            loop {
                let ch = *s.add(len) as u8;
                if ch == 0 {
                    break;
                }
                len += 1;
            }
            let bytes = core::slice::from_raw_parts(s as *const u8, len);

            // Skip leading ASCII whitespace.
            let mut i = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            // Optional sign.
            let mut sign: i64 = 1;
            if i < bytes.len() {
                match bytes[i] {
                    b'+' => i += 1,
                    b'-' => {
                        sign = -1;
                        i += 1;
                    }
                    _ => {}
                }
            }

            let start_digits = i;

            // Accumulate digits with overflow detection.
            let mut acc: i64 = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !(b'0'..=b'9').contains(&b) {
                    break;
                }
                let digit = (b - b'0') as i64;

                match acc.checked_mul(10).and_then(|v| v.checked_add(digit)) {
                    Some(v) => acc = v,
                    None => {
                        if let Some(ref mut e) = error {
                            **e = true;
                        }
                        if !endp.is_null() {
                            *endp = s.add(i) as *const u8;
                        }
                        return if sign < 0 { i64::MIN } else { i64::MAX };
                    }
                }

                i += 1;
            }

            // If no digits were consumed, endp points to original string.
            if !endp.is_null() {
                let end_ptr = if i == start_digits { s } else { s.add(i) };
                *endp = end_ptr as *const u8;
            }

            // Apply sign with overflow check.
            match acc.checked_mul(sign) {
                Some(v) => v,
                None => {
                    if let Some(ref mut e) = error {
                        **e = true;
                    }
                    if sign < 0 { i64::MIN } else { i64::MAX }
                }
            }
        }

        unsafe fn parse_val(str_: *const i8, val: &mut i32) -> bool {
            let mut error0 = false;
            let mut endp: *const u8 = str_ as *const u8;

            let tmp: i64 = strtol_local(str_, &mut endp as *mut *const u8, 10, Some(&mut error0));

            let ok = endp != (str_ as *const u8)
                && !error0
                && tmp >= INT_MIN as i64
                && tmp <= INT_MAX as i64;

            if ok {
                *val = tmp as i32;
                true_0 != 0
            } else {
                false_0 != 0
            }
        }

        pub(crate) unsafe fn driver_internal(in_0: &mut [i8]) {
            let mut x: i32 = 0;

            if parse_val(in_0.as_ptr(), &mut x) {
                let mut the_house = house_t {
                    floors: 2,
                    bedrooms: 5,
                    bathrooms: 2.5,
                };

                run(&mut the_house, x);
                run(&mut the_house, x);
            } else {
                println!("An error occurred");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *mut i8) {
            driver_internal(if in_0.is_null() {
                &mut []
            } else {
                core::slice::from_raw_parts_mut(in_0, 1024)
            })
        }

        pub const true_0: i32 = 1;
        pub const false_0: i32 = 0;
        pub const INT_MAX: i32 = __INT_MAX__;
        pub const INT_MIN: i32 = -__INT_MAX__ - 1;
        pub const __INT_MAX__: i32 = 2147483647;
    }
}
```

**Entity:** strtol_local (unsafe fn)

**States:** InvalidInput, ValidInput

**Transitions:**
- InvalidInput -> ValidInput by ensuring `s` is non-null, NUL-terminated, and `base == 10` before calling strtol_local()

**Evidence:** strtol_local(s, endp, base, ...): parameter `s: *const i8` is a raw pointer and is dereferenced via `*s.add(len)` in a loop to find NUL terminator; strtol_local: `if s.is_null() { ... **e = true; return 0; }` encodes a non-null precondition as a runtime check; strtol_local: `if base != 10 { ... **e = true; return 0; }` hard-codes the invariant “only base-10 is supported”; strtol_local: comment `// Find NUL terminator to bound the input.` indicates reliance on NUL termination

**Implementation:** Introduce a safe wrapper type like `struct CStrPtr<'a>(&'a core::ffi::CStr);` (or accept `&CStr` directly) to enforce non-null + NUL-terminated at the type level, and replace `base: i32` with a `const`/zero-sized type (or remove it) to encode base-10-only. Expose a safe `fn parse_i32_decimal(s: &CStr) -> Result<i32, ParseIntError>` that internally calls the unsafe routine if needed.

---

## Protocol Invariants

### 2. Validated integer parsing protocol (ParsedI32 token vs Unparsed/Invalid)

**Location**: `/data/test_case/lib.rs:1-211`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `parse_val` implements a multi-step validation protocol to turn a raw `*const i8` into an `i32`: it calls `strtol_local`, checks that at least one digit was consumed (`endp != str_`), checks `!error0`, and enforces `INT_MIN..=INT_MAX` bounds before writing to `*val`. Callers must respect the implied contract: `val` is only initialized/meaningful when the function returns true. This is encoded via an out-parameter plus boolean return, rather than returning a typed value that would make the “only valid on success” state explicit.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 4 free function(s)

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
        use core::ptr;

        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct house_t {
            pub floors: i32,
            pub bedrooms: i32,
            pub bathrooms: f64,
        }

        fn add_floor(house: &mut house_t) {
            house.floors += 1;
        }

        fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
            house.bedrooms += extra_bedrooms;
        }

        fn print_house(house: &house_t) {
            println!(
                "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
                house.floors, house.bedrooms, house.bathrooms
            );
        }

        pub(crate) fn run(the_house: &mut house_t, extra_bedrooms: i32) {
            print_house(the_house);
            add_floor(the_house);
            print_house(the_house);

            the_house.bathrooms += 1.0;
            print_house(the_house);

            add_bedrooms(the_house, extra_bedrooms);
            print_house(the_house);
        }

        // Local replacement for `crate::c_lib::strtol` used by the original C2Rust output.
        // Only base-10 is needed here.
        unsafe fn strtol_local(
            s: *const i8,
            endp: *mut *const u8,
            base: i32,
            mut error: Option<&mut bool>,
        ) -> i64 {
            if let Some(ref mut e) = error {
                **e = false;
            }

            if s.is_null() {
                if !endp.is_null() {
                    *endp = ptr::null();
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            if base != 10 {
                if !endp.is_null() {
                    *endp = s as *const u8;
                }
                if let Some(ref mut e) = error {
                    **e = true;
                }
                return 0;
            }

            // Find NUL terminator to bound the input.
            let mut len: usize = 0;
            loop {
                let ch = *s.add(len) as u8;
                if ch == 0 {
                    break;
                }
                len += 1;
            }
            let bytes = core::slice::from_raw_parts(s as *const u8, len);

            // Skip leading ASCII whitespace.
            let mut i = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            // Optional sign.
            let mut sign: i64 = 1;
            if i < bytes.len() {
                match bytes[i] {
                    b'+' => i += 1,
                    b'-' => {
                        sign = -1;
                        i += 1;
                    }
                    _ => {}
                }
            }

            let start_digits = i;

            // Accumulate digits with overflow detection.
            let mut acc: i64 = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !(b'0'..=b'9').contains(&b) {
                    break;
                }
                let digit = (b - b'0') as i64;

                match acc.checked_mul(10).and_then(|v| v.checked_add(digit)) {
                    Some(v) => acc = v,
                    None => {
                        if let Some(ref mut e) = error {
                            **e = true;
                        }
                        if !endp.is_null() {
                            *endp = s.add(i) as *const u8;
                        }
                        return if sign < 0 { i64::MIN } else { i64::MAX };
                    }
                }

                i += 1;
            }

            // If no digits were consumed, endp points to original string.
            if !endp.is_null() {
                let end_ptr = if i == start_digits { s } else { s.add(i) };
                *endp = end_ptr as *const u8;
            }

            // Apply sign with overflow check.
            match acc.checked_mul(sign) {
                Some(v) => v,
                None => {
                    if let Some(ref mut e) = error {
                        **e = true;
                    }
                    if sign < 0 { i64::MIN } else { i64::MAX }
                }
            }
        }

        unsafe fn parse_val(str_: *const i8, val: &mut i32) -> bool {
            let mut error0 = false;
            let mut endp: *const u8 = str_ as *const u8;

            let tmp: i64 = strtol_local(str_, &mut endp as *mut *const u8, 10, Some(&mut error0));

            let ok = endp != (str_ as *const u8)
                && !error0
                && tmp >= INT_MIN as i64
                && tmp <= INT_MAX as i64;

            if ok {
                *val = tmp as i32;
                true_0 != 0
            } else {
                false_0 != 0
            }
        }

        pub(crate) unsafe fn driver_internal(in_0: &mut [i8]) {
            let mut x: i32 = 0;

            if parse_val(in_0.as_ptr(), &mut x) {
                let mut the_house = house_t {
                    floors: 2,
                    bedrooms: 5,
                    bathrooms: 2.5,
                };

                run(&mut the_house, x);
                run(&mut the_house, x);
            } else {
                println!("An error occurred");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *mut i8) {
            driver_internal(if in_0.is_null() {
                &mut []
            } else {
                core::slice::from_raw_parts_mut(in_0, 1024)
            })
        }

        pub const true_0: i32 = 1;
        pub const false_0: i32 = 0;
        pub const INT_MAX: i32 = __INT_MAX__;
        pub const INT_MIN: i32 = -__INT_MAX__ - 1;
        pub const __INT_MAX__: i32 = 2147483647;
    }
}
```

**Entity:** parse_val (unsafe fn)

**States:** UnvalidatedInput, ValidatedI32

**Transitions:**
- UnvalidatedInput -> ValidatedI32 via parse_val() returning true (and writing to `*val`)

**Evidence:** parse_val(str_, val): uses out-param `val: &mut i32` and returns `bool`, implying `val` is only valid if return is true; parse_val: `let tmp: i64 = strtol_local(..., Some(&mut error0));` then `let ok = endp != (str_ as *const u8) && !error0 && tmp >= INT_MIN as i64 && tmp <= INT_MAX as i64;` encodes the full validation gate; parse_val: `if ok { *val = tmp as i32; ... }` shows `*val` is only written in the success state

**Implementation:** Replace `(out i32, bool)` with `fn parse_val(str_: &CStr) -> Result<i32, ParseError>`, optionally returning a `struct ValidI32(i32);` newtype if additional invariants are desired. This makes it impossible to observe an uninitialized/invalid `val` and forces callers to handle the failure state explicitly.

---

