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

### 1. C-string parsing preconditions (non-null, NUL-terminated, in-range i32)

**Location**: `/data/test_case/lib.rs:1-155`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: parse_val encodes several required input validity conditions at runtime: the output pointer must be present (Some), the input C string pointer must be non-null, and the pointed-to bytes must contain a NUL terminator within the probed range and represent a base-10 i32 in range [INT_MIN, INT_MAX]. These are enforced by returning false_0 at runtime, but the signature allows calling with null/absent pointers and non-C-string buffers, and it performs raw pointer reads with an arbitrary maximum length (100000) that assumes the memory is valid to read. The type system does not capture the 'valid C string' and 'has output slot' requirements.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 2 free function(s)

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

pub mod src {
    pub mod lib {
        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct house_t {
            pub floors: i32,
            pub bedrooms: i32,
            pub bathrooms: f64,
        }

        thread_local! {
            static the_house: std::cell::RefCell<house_t> = const {
                std::cell::RefCell::new(house_t { floors: 2, bedrooms: 5, bathrooms: 2.5f64 })
            };
        }

        fn add_floor(house: &mut house_t) {
            house.floors += 1;
        }

        fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
            house.bedrooms += extra_bedrooms;
        }

        fn add_floor_to_the_house() {
            the_house.with_borrow_mut(add_floor);
        }

        fn print_the_house() {
            the_house.with_borrow(|h| {
                println!(
                    "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
                    h.floors, h.bedrooms, h.bathrooms
                )
            });
        }

        pub(crate) fn run(extra_bedrooms: i32) {
            print_the_house();

            add_floor_to_the_house();
            print_the_house();

            the_house.with_borrow_mut(|h| h.bathrooms += 1.0f64);
            print_the_house();

            the_house.with_borrow_mut(|h| add_bedrooms(h, extra_bedrooms));
            print_the_house();
        }

        /// Parse a base-10 i32 from a C string pointer.
        /// Keeps the original signature for compatibility with the C2Rust output.
        unsafe fn parse_val(str_: *const i8, val: Option<&mut i32>) -> bool {
            let Some(out) = val else {
                return false_0 != 0;
            };
            if str_.is_null() {
                return false_0 != 0;
            }

            // Avoid external libc dependency: parse from bytes up to NUL.
            // This matches the original intent (strtol with range checking).
            let bytes = std::slice::from_raw_parts(str_ as *const u8, 100000);
            let nul_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            let bytes = &bytes[..nul_pos];

            // Skip leading ASCII whitespace like strtol.
            let mut i = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i == bytes.len() {
                return false_0 != 0;
            }

            // Optional sign.
            let mut sign: i64 = 1;
            if bytes[i] == b'+' {
                i += 1;
            } else if bytes[i] == b'-' {
                sign = -1;
                i += 1;
            }

            // Must have at least one digit.
            if i == bytes.len() || !bytes[i].is_ascii_digit() {
                return false_0 != 0;
            }

            // Accumulate digits with overflow detection in i64.
            let mut acc: i64 = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !b.is_ascii_digit() {
                    break;
                }
                let digit = (b - b'0') as i64;

                // Check acc*10 + digit overflow for i64.
                if acc > (i64::MAX - digit) / 10 {
                    return false_0 != 0;
                }
                acc = acc * 10 + digit;
                i += 1;
            }

            let value = acc.saturating_mul(sign);
            if value < INT_MIN as i64 || value > INT_MAX as i64 {
                return false_0 != 0;
            }

            *out = value as i32;
            true_0 != 0
        }

        pub(crate) unsafe fn driver_internal(in_0: &mut [i8]) {
            let mut x: i32 = 0;
            if parse_val(in_0.as_ptr(), Some(&mut x)) {
                run(x);
                run(x);
            } else {
                println!("An error occurred");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *mut i8) {
            driver_internal(if in_0.is_null() {
                &mut []
            } else {
                std::slice::from_raw_parts_mut(in_0, 1024)
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

**Entity:** parse_val(str_: *const i8, val: Option<&mut i32>)

**States:** InvalidInput, ValidInput

**Transitions:**
- InvalidInput -> ValidInput via satisfying: val=Some(&mut _), str_ != null, parseable digits, no overflow, and INT_MIN..=INT_MAX
- ValidInput -> InvalidInput via any violated check returning false_0

**Evidence:** function signature: unsafe fn parse_val(str_: *const i8, val: Option<&mut i32>) -> bool (accepts nullable raw pointer and optional out param); early return: let Some(out) = val else { return false_0 != 0; } (requires output slot); null check: if str_.is_null() { return false_0 != 0; }; unsafe read assumption: std::slice::from_raw_parts(str_ as *const u8, 100000) (assumes readable memory and eventual NUL terminator within range); range check: if value < INT_MIN as i64 || value > INT_MAX as i64 { return false_0 != 0; }; success writes out-param: *out = value as i32; true_0 != 0

**Implementation:** Replace (str_: *const i8) with a validated wrapper such as struct CStrPtr<'a>(&'a std::ffi::CStr) obtained via unsafe constructor at FFI boundary; replace Option<&mut i32> with &mut i32 so 'has output slot' is guaranteed; expose a safe fn parse_i32(s: &CStr) -> Option<i32> (or Result<i32, ParseError>) and keep the unsafe raw-pointer shim only at the extern boundary.

---

## Protocol Invariants

### 2. FFI buffer validity protocol (non-null pointer must reference >=1024 writable bytes + NUL-terminated string)

**Location**: `/data/test_case/lib.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The FFI entrypoint builds a &mut [i8] slice from a raw pointer assuming a fixed length of 1024 bytes when non-null. This implicitly requires that any non-null pointer passed from C points to at least 1024 writable bytes. Additionally, driver_internal passes in_0.as_ptr() to parse_val, which then reads up to 100000 bytes searching for NUL, implicitly requiring a NUL terminator to appear before unreadable memory (and before the probe limit). None of these requirements are represented in the type system; they are implicit in slice creation and raw reads.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct house_t, 2 free function(s)

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

pub mod src {
    pub mod lib {
        // === driver.rs ===
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct house_t {
            pub floors: i32,
            pub bedrooms: i32,
            pub bathrooms: f64,
        }

        thread_local! {
            static the_house: std::cell::RefCell<house_t> = const {
                std::cell::RefCell::new(house_t { floors: 2, bedrooms: 5, bathrooms: 2.5f64 })
            };
        }

        fn add_floor(house: &mut house_t) {
            house.floors += 1;
        }

        fn add_bedrooms(house: &mut house_t, extra_bedrooms: i32) {
            house.bedrooms += extra_bedrooms;
        }

        fn add_floor_to_the_house() {
            the_house.with_borrow_mut(add_floor);
        }

        fn print_the_house() {
            the_house.with_borrow(|h| {
                println!(
                    "The house has {0} floors, {1} bedrooms, and {2:.1} bathrooms",
                    h.floors, h.bedrooms, h.bathrooms
                )
            });
        }

        pub(crate) fn run(extra_bedrooms: i32) {
            print_the_house();

            add_floor_to_the_house();
            print_the_house();

            the_house.with_borrow_mut(|h| h.bathrooms += 1.0f64);
            print_the_house();

            the_house.with_borrow_mut(|h| add_bedrooms(h, extra_bedrooms));
            print_the_house();
        }

        /// Parse a base-10 i32 from a C string pointer.
        /// Keeps the original signature for compatibility with the C2Rust output.
        unsafe fn parse_val(str_: *const i8, val: Option<&mut i32>) -> bool {
            let Some(out) = val else {
                return false_0 != 0;
            };
            if str_.is_null() {
                return false_0 != 0;
            }

            // Avoid external libc dependency: parse from bytes up to NUL.
            // This matches the original intent (strtol with range checking).
            let bytes = std::slice::from_raw_parts(str_ as *const u8, 100000);
            let nul_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            let bytes = &bytes[..nul_pos];

            // Skip leading ASCII whitespace like strtol.
            let mut i = 0usize;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i == bytes.len() {
                return false_0 != 0;
            }

            // Optional sign.
            let mut sign: i64 = 1;
            if bytes[i] == b'+' {
                i += 1;
            } else if bytes[i] == b'-' {
                sign = -1;
                i += 1;
            }

            // Must have at least one digit.
            if i == bytes.len() || !bytes[i].is_ascii_digit() {
                return false_0 != 0;
            }

            // Accumulate digits with overflow detection in i64.
            let mut acc: i64 = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if !b.is_ascii_digit() {
                    break;
                }
                let digit = (b - b'0') as i64;

                // Check acc*10 + digit overflow for i64.
                if acc > (i64::MAX - digit) / 10 {
                    return false_0 != 0;
                }
                acc = acc * 10 + digit;
                i += 1;
            }

            let value = acc.saturating_mul(sign);
            if value < INT_MIN as i64 || value > INT_MAX as i64 {
                return false_0 != 0;
            }

            *out = value as i32;
            true_0 != 0
        }

        pub(crate) unsafe fn driver_internal(in_0: &mut [i8]) {
            let mut x: i32 = 0;
            if parse_val(in_0.as_ptr(), Some(&mut x)) {
                run(x);
                run(x);
            } else {
                println!("An error occurred");
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn driver(in_0: *mut i8) {
            driver_internal(if in_0.is_null() {
                &mut []
            } else {
                std::slice::from_raw_parts_mut(in_0, 1024)
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

**Entity:** driver(in_0: *mut i8) / driver_internal(in_0: &mut [i8])

**States:** NullPointer, NonNullButInvalidBuffer, ValidBuffer

**Transitions:**
- NullPointer -> (no parsing) via driver mapping null to &mut []
- ValidBuffer -> Parsed/NotParsed via parse_val(in_0.as_ptr(), ...)
- NonNullButInvalidBuffer -> UndefinedBehavior via std::slice::from_raw_parts_mut(in_0, 1024) if backing memory is smaller/invalid

**Evidence:** extern boundary: pub unsafe extern "C" fn driver(in_0: *mut i8); fixed-length slice creation: std::slice::from_raw_parts_mut(in_0, 1024); null handling: if in_0.is_null() { &mut [] } else { ... }; driver_internal: parse_val(in_0.as_ptr(), Some(&mut x)) (parsing assumes C-string bytes); parse_val raw probe: std::slice::from_raw_parts(str_ as *const u8, 100000) (assumes readable range and NUL termination)

**Implementation:** At the safe boundary, require a capability/wrapper proving buffer validity, e.g. struct InputBuf<'a>(&'a mut [u8; 1024]); provide an unsafe constructor `unsafe fn from_ptr(p: *mut i8) -> Option<InputBuf<'static>>` that checks non-null and documents required allocation; alternatively change the ABI to accept (ptr, len) and use `&mut [u8]` with explicit length, then parse using `CStr` (requires NUL termination) or length-bounded parsing without over-reading.

---

