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

### 1. Mode selection requires non-negative indexing into fixed mode table

**Location**: `/data/test_case/lib.rs:1-193`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `modeselect` computes `mode_index = mode_selector % 4` as an i32 to mimic C behavior. In Rust, a negative `mode_selector` yields a negative `mode_index`, but the code then casts `mode_index as usize` to index `modes[...]`, which can panic or access the wrong element. The function implicitly relies on a precondition that `mode_selector` be non-negative (or otherwise yield an index 0..=3), but this is not enforced by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Xu32, impl core :: fmt :: UpperHex for Xu32 (1 methods)

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
        extern "C" {
            fn strcmp(__s1: *const i8, __s2: *const i8) -> i32;
            fn time(__timer: *mut time_t) -> time_t;
        }

        pub type __time_t = i64;
        pub type time_t = __time_t;

        // Replacement for the missing `crate::c_lib::Xu32` used only for formatting.
        // Keep it local and lightweight; it preserves the `{:#X}`-style formatting usage.
        struct Xu32(u32);
        impl core::fmt::UpperHex for Xu32 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        pub(crate) unsafe fn classify_mode(mode: &[i8]) -> i32 {
            if strcmp(mode.as_ptr(), b"standard\0" as *const u8 as *const i8) == 0 {
                0x10
            } else if strcmp(mode.as_ptr(), b"enhanced\0" as *const u8 as *const i8) == 0 {
                0x20
            } else if strcmp(mode.as_ptr(), b"turbo\0" as *const u8 as *const i8) == 0 {
                0x30
            } else if strcmp(mode.as_ptr(), b"extreme\0" as *const u8 as *const i8) == 0 {
                0x40
            } else {
                0
            }
        }

        pub(crate) fn apply_multiplier(mut base: i32, level: i32) -> i32 {
            // Preserve original C2Rust fallthrough-via-state behavior.
            let mut state: u64;
            match level {
                4 => {
                    base += 0xff;
                    state = 17978390595819632496;
                }
                3 => {
                    state = 17978390595819632496;
                }
                2 => {
                    state = 16013381478083460001;
                }
                1 => {
                    state = 10344925754825801646;
                }
                0 => {
                    state = 6995965253482708452;
                }
                _ => {
                    base = 0xdead;
                    state = 6937071982253665452;
                }
            }

            if state == 17978390595819632496 {
                base += 0xab;
                state = 16013381478083460001;
            }
            if state == 16013381478083460001 {
                base += 0x7e;
                state = 10344925754825801646;
            }
            if state == 10344925754825801646 {
                base += 0x1c;
                state = 6995965253482708452;
            }
            if state == 6995965253482708452 {
                base += 0x5;
            }
            base
        }

        pub(crate) fn convert_time_factor(factor: f64) -> i32 {
            (factor * 1e12f64) as i32
        }

        pub(crate) fn convert_negative_overflow(value: f64) -> i32 {
            (value * -1e15f64) as i32
        }

        pub(crate) unsafe fn get_modified_time(offset_days: i32, offset_hours: i32) -> time_t {
            let mut current: time_t = time(core::ptr::null_mut());
            current >>= 29;
            let offset: time_t = (offset_days * 86400 + offset_hours * 3600) as i64;
            current + offset
        }

        pub(crate) unsafe fn hash_time_value(t: time_t) -> i32 {
            // Avoid UB: only read the actual bytes of `t`.
            let mut hash: i32 = 0x5a5a5a5a;
            let bytes: &[u8] = core::slice::from_raw_parts(
                core::ptr::addr_of!(t).cast::<u8>(),
                core::mem::size_of::<time_t>(),
            );

            for (i, &b) in bytes.iter().enumerate() {
                let shift = i.wrapping_rem(4).wrapping_mul(8);
                hash ^= (b as i32) << shift;
                hash = hash.wrapping_mul(0x1f);
            }
            hash & 0x7fffffff
        }

        #[no_mangle]
        pub unsafe extern "C" fn modeselect(
            mode_selector: i32,
            time_offset: i32,
            complexity: i32,
            seed: i32,
        ) -> i32 {
            let mut result: i32 = 0;

            let modes: [*const i8; 4] = [
                b"standard\0" as *const u8 as *const i8,
                b"enhanced\0" as *const u8 as *const i8,
                b"turbo\0" as *const u8 as *const i8,
                b"extreme\0" as *const u8 as *const i8,
            ];

            // Preserve original `% 4` behavior (including negatives) by keeping it as i32.
            let mode_index: i32 = mode_selector % 4;
            let mode_ptr: *const i8 = modes[mode_index as usize];

            let selected_mode: &[i8] = if mode_ptr.is_null() {
                &[]
            } else {
                // Keep the original oversized slice length; strcmp stops at NUL anyway.
                core::slice::from_raw_parts(mode_ptr, 100000)
            };

            let mode_value: i32 = classify_mode(selected_mode);
            println!(
                "Selected mode: {0} (0x{1:X})",
                core::ffi::CStr::from_ptr(mode_ptr).to_str().unwrap(),
                Xu32(mode_value as u32)
            );
            result += mode_value;

            let complexity_level: i32 = complexity % 5;
            let multiplier: i32 = apply_multiplier(0xa0, complexity_level);
            println!(
                "Complexity level: {0}, Multiplier: 0x{1:X}",
                complexity_level,
                Xu32(multiplier as u32)
            );
            result += multiplier;

            let modified_time: time_t = get_modified_time(time_offset, seed % 24);
            let time_hash: i32 = hash_time_value(modified_time);
            println!(
                "Modified time: {0}, Hash: 0x{1:X}",
                modified_time,
                Xu32(time_hash as u32)
            );
            result += time_hash % 0x1000;

            let factor1: f64 = seed as f64 * 1e8f64;
            let factor2: f64 = time_offset as f64 * -1e7f64;

            println!("Converting double {0:.2e} to int (may overflow)...", factor1);
            let result1: i32 = convert_time_factor(factor1);
            println!("Result 1: {0} (0x{1:X})", result1, Xu32(result1 as u32));

            println!("Converting double {0:.2e} to int (may underflow)...", factor2);
            let result2: i32 = convert_negative_overflow(factor2);
            println!("Result 2: {0} (0x{1:X})", result2, Xu32(result2 as u32));

            result ^= result1 & 0xff;
            result ^= result2 & 0xff00;
            result = result * 0x10 + 0xbeef;

            print!("\nFinal result: {0} (0x{1:X})\n", result, Xu32(result as u32));
            result
        }
    }
}
```

**Entity:** modeselect (mode_selector/mode_index/modes)

**States:** ValidModeIndex(0..=3), InvalidModeIndex(<0 or >3 via negative % result)

**Transitions:**
- InvalidModeIndex -> ValidModeIndex via validating/clamping mode_selector before indexing

**Evidence:** modeselect: comment: "Preserve original `% 4` behavior (including negatives) by keeping it as i32."; modeselect: `let mode_index: i32 = mode_selector % 4;`; modeselect: `let mode_ptr: *const i8 = modes[mode_index as usize];` (casts possibly-negative i32 to usize for indexing)

**Implementation:** Introduce a `ModeIndex(usize)` newtype with `TryFrom<i32>` (or `TryFrom<isize>`) that validates 0..4, or represent the mode as an enum `Mode { Standard, Enhanced, Turbo, Extreme }` derived from `mode_selector` with checked mapping; only allow indexing/mode lookup given a validated `ModeIndex`/`Mode`.

---

## Protocol Invariants

### 2. C string protocol: NUL-terminated, valid pointer, and lifetime for strcmp/CStr usage

**Location**: `/data/test_case/lib.rs:1-193`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `classify_mode` and `modeselect` treat a Rust slice/pointer as a C string. `classify_mode` calls `strcmp(mode.as_ptr(), ...)`, which requires `mode.as_ptr()` to point to a NUL-terminated string. Separately, `modeselect` builds `selected_mode` using `from_raw_parts(mode_ptr, 100000)` and later prints using `CStr::from_ptr(mode_ptr)`, both of which require `mode_ptr` to be non-null, properly NUL-terminated, and valid to read up to the NUL. These are implicit safety/validity requirements not captured in the types (the function accepts `&[i8]` / raw pointers rather than `&CStr`/`NonNull<CChar>`).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Xu32, impl core :: fmt :: UpperHex for Xu32 (1 methods)

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
        extern "C" {
            fn strcmp(__s1: *const i8, __s2: *const i8) -> i32;
            fn time(__timer: *mut time_t) -> time_t;
        }

        pub type __time_t = i64;
        pub type time_t = __time_t;

        // Replacement for the missing `crate::c_lib::Xu32` used only for formatting.
        // Keep it local and lightweight; it preserves the `{:#X}`-style formatting usage.
        struct Xu32(u32);
        impl core::fmt::UpperHex for Xu32 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        pub(crate) unsafe fn classify_mode(mode: &[i8]) -> i32 {
            if strcmp(mode.as_ptr(), b"standard\0" as *const u8 as *const i8) == 0 {
                0x10
            } else if strcmp(mode.as_ptr(), b"enhanced\0" as *const u8 as *const i8) == 0 {
                0x20
            } else if strcmp(mode.as_ptr(), b"turbo\0" as *const u8 as *const i8) == 0 {
                0x30
            } else if strcmp(mode.as_ptr(), b"extreme\0" as *const u8 as *const i8) == 0 {
                0x40
            } else {
                0
            }
        }

        pub(crate) fn apply_multiplier(mut base: i32, level: i32) -> i32 {
            // Preserve original C2Rust fallthrough-via-state behavior.
            let mut state: u64;
            match level {
                4 => {
                    base += 0xff;
                    state = 17978390595819632496;
                }
                3 => {
                    state = 17978390595819632496;
                }
                2 => {
                    state = 16013381478083460001;
                }
                1 => {
                    state = 10344925754825801646;
                }
                0 => {
                    state = 6995965253482708452;
                }
                _ => {
                    base = 0xdead;
                    state = 6937071982253665452;
                }
            }

            if state == 17978390595819632496 {
                base += 0xab;
                state = 16013381478083460001;
            }
            if state == 16013381478083460001 {
                base += 0x7e;
                state = 10344925754825801646;
            }
            if state == 10344925754825801646 {
                base += 0x1c;
                state = 6995965253482708452;
            }
            if state == 6995965253482708452 {
                base += 0x5;
            }
            base
        }

        pub(crate) fn convert_time_factor(factor: f64) -> i32 {
            (factor * 1e12f64) as i32
        }

        pub(crate) fn convert_negative_overflow(value: f64) -> i32 {
            (value * -1e15f64) as i32
        }

        pub(crate) unsafe fn get_modified_time(offset_days: i32, offset_hours: i32) -> time_t {
            let mut current: time_t = time(core::ptr::null_mut());
            current >>= 29;
            let offset: time_t = (offset_days * 86400 + offset_hours * 3600) as i64;
            current + offset
        }

        pub(crate) unsafe fn hash_time_value(t: time_t) -> i32 {
            // Avoid UB: only read the actual bytes of `t`.
            let mut hash: i32 = 0x5a5a5a5a;
            let bytes: &[u8] = core::slice::from_raw_parts(
                core::ptr::addr_of!(t).cast::<u8>(),
                core::mem::size_of::<time_t>(),
            );

            for (i, &b) in bytes.iter().enumerate() {
                let shift = i.wrapping_rem(4).wrapping_mul(8);
                hash ^= (b as i32) << shift;
                hash = hash.wrapping_mul(0x1f);
            }
            hash & 0x7fffffff
        }

        #[no_mangle]
        pub unsafe extern "C" fn modeselect(
            mode_selector: i32,
            time_offset: i32,
            complexity: i32,
            seed: i32,
        ) -> i32 {
            let mut result: i32 = 0;

            let modes: [*const i8; 4] = [
                b"standard\0" as *const u8 as *const i8,
                b"enhanced\0" as *const u8 as *const i8,
                b"turbo\0" as *const u8 as *const i8,
                b"extreme\0" as *const u8 as *const i8,
            ];

            // Preserve original `% 4` behavior (including negatives) by keeping it as i32.
            let mode_index: i32 = mode_selector % 4;
            let mode_ptr: *const i8 = modes[mode_index as usize];

            let selected_mode: &[i8] = if mode_ptr.is_null() {
                &[]
            } else {
                // Keep the original oversized slice length; strcmp stops at NUL anyway.
                core::slice::from_raw_parts(mode_ptr, 100000)
            };

            let mode_value: i32 = classify_mode(selected_mode);
            println!(
                "Selected mode: {0} (0x{1:X})",
                core::ffi::CStr::from_ptr(mode_ptr).to_str().unwrap(),
                Xu32(mode_value as u32)
            );
            result += mode_value;

            let complexity_level: i32 = complexity % 5;
            let multiplier: i32 = apply_multiplier(0xa0, complexity_level);
            println!(
                "Complexity level: {0}, Multiplier: 0x{1:X}",
                complexity_level,
                Xu32(multiplier as u32)
            );
            result += multiplier;

            let modified_time: time_t = get_modified_time(time_offset, seed % 24);
            let time_hash: i32 = hash_time_value(modified_time);
            println!(
                "Modified time: {0}, Hash: 0x{1:X}",
                modified_time,
                Xu32(time_hash as u32)
            );
            result += time_hash % 0x1000;

            let factor1: f64 = seed as f64 * 1e8f64;
            let factor2: f64 = time_offset as f64 * -1e7f64;

            println!("Converting double {0:.2e} to int (may overflow)...", factor1);
            let result1: i32 = convert_time_factor(factor1);
            println!("Result 1: {0} (0x{1:X})", result1, Xu32(result1 as u32));

            println!("Converting double {0:.2e} to int (may underflow)...", factor2);
            let result2: i32 = convert_negative_overflow(factor2);
            println!("Result 2: {0} (0x{1:X})", result2, Xu32(result2 as u32));

            result ^= result1 & 0xff;
            result ^= result2 & 0xff00;
            result = result * 0x10 + 0xbeef;

            print!("\nFinal result: {0} (0x{1:X})\n", result, Xu32(result as u32));
            result
        }
    }
}
```

**Entity:** classify_mode (mode: &[i8]) / selected_mode construction

**States:** ValidCStrPointer(NUL-terminated, non-null, accessible), InvalidCStrPointer(null/non-NUL-terminated/dangling)

**Transitions:**
- InvalidCStrPointer -> ValidCStrPointer via constructing/accepting `&CStr` (or validating input pointer/termination) before calling strcmp/CStr::from_ptr

**Evidence:** classify_mode signature: `pub(crate) unsafe fn classify_mode(mode: &[i8]) -> i32` (takes arbitrary slice, but uses it as C string); classify_mode: `strcmp(mode.as_ptr(), b"standard\0" ...)` (requires NUL-terminated `mode`); modeselect: `core::slice::from_raw_parts(mode_ptr, 100000)` with comment: "Keep the original oversized slice length; strcmp stops at NUL anyway."; modeselect: `core::ffi::CStr::from_ptr(mode_ptr).to_str().unwrap()` (requires non-null, NUL-terminated, valid memory)

**Implementation:** Change `classify_mode` to accept `&core::ffi::CStr` (or a local `struct ModeCStr<'a>(&'a CStr)` newtype) and in `modeselect` avoid `from_raw_parts(..., 100000)`. Keep the mode table as `&'static CStr` (e.g., `CStr::from_bytes_with_nul(b"standard\0").unwrap()`) and pass those references through, eliminating the need for unsafe pointer/slice assumptions at call sites.

---

