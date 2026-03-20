# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 0
- **Protocol**: 2
- **Modules analyzed**: 2

## State Machine Invariants

### 2. Thread-local error reason validity (Null / Set-to-static-CStr)

**Location**: `/data/test_case/lib.rs:1-306`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The thread-local cp_error_reason stores a raw `*const i8` which implicitly must either be null (no error) or point to a valid NUL-terminated string that outlives all reads (typically a static C string). The type system cannot ensure non-nullness when used, nor lifetime/validity of the pointed-to string; this is a latent invariant typical of FFI error channels.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_state_t, 11 free function(s)

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
        use core::ffi::c_void;

        extern "C" {
            fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
            fn memset(__s: *mut c_void, __c: i32, __n: usize) -> *mut c_void;
            fn calloc(__nmemb: usize, __size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn __assert_fail(
                __assertion: *const i8,
                __file: *const i8,
                __line: u32,
                __function: *const i8,
            ) -> !;
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_state_t {
            pub bits: u64,
            pub count: i32,
            pub words: *mut u32,
            pub word_count: i32,
            pub word_index: i32,
            pub bits_left: i32,
            pub final_word_available: i32,
            pub final_word: u32,
            pub out: *mut i8,
            pub out_end: *mut i8,
            pub begin: *mut i8,
            pub lookup: [u16; 512],
            pub lit: [u32; 288],
            pub dst: [u32; 32],
            pub len: [u32; 19],
            pub nlit: u32,
            pub ndst: u32,
            pub nlen: u32,
        }

        thread_local! {
            static cp_error_reason: std::cell::Cell<*const i8> = const { std::cell::Cell::new(core::ptr::null()) };
        }

        // Keep these statics unchanged (no #[no_mangle] added).
        pub static cp_fixed_table: [u8; 320] = [
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];

        pub static cp_permutation_order: [u8; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ];
        pub static cp_len_extra_bits: [u8; 31] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
        ];
        pub static cp_len_base: [u32; 31] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
            163, 195, 227, 258, 0, 0,
        ];
        pub static cp_dist_extra_bits: [u8; 32] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
            13, 0, 0,
        ];
        pub static cp_dist_base: [u32; 32] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049,
            3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
        ];

        #[inline]
        fn cp_would_overflow(s: Option<&cp_state_t>, num_bits: i32) -> i32 {
            (s.unwrap().bits_left + s.unwrap().count - num_bits < 0) as i32
        }

        unsafe fn cp_ptr(s: Option<&cp_state_t>) -> *const i8 {
            if s.unwrap().bits_left & 7 != 0 {
                __assert_fail(
                    b"!(s->bits_left & 7)\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    95,
                    [
                        b'c' as i8, b'h' as i8, b'a' as i8, b'r' as i8, b' ' as i8, b'*' as i8,
                        b'c' as i8, b'p' as i8, b'_' as i8, b'p' as i8, b't' as i8, b'r' as i8,
                        b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8, b's' as i8, b't' as i8,
                        b'a' as i8, b't' as i8, b'e' as i8, b'_' as i8, b't' as i8, b' ' as i8,
                        b'*' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            (s.unwrap().words.offset(s.unwrap().word_index as isize) as *mut i8)
                .offset(-((s.unwrap().count / 8) as isize)) as *const i8
        }

        unsafe fn cp_peak_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u64 {
            if s.as_deref().unwrap().count < num_bits_to_read {
                if s.as_deref().unwrap().word_index < s.as_deref().unwrap().word_count {
                    let idx = s.as_deref().unwrap().word_index;
                    s.as_deref_mut().unwrap().word_index = idx + 1;

                    let word: u32 = *s.as_deref().unwrap().words.add(idx as usize);
                    let shift = s.as_deref().unwrap().count;
                    s.as_deref_mut().unwrap().bits |= (word as u64) << shift;
                    s.as_deref_mut().unwrap().count += 32;

                    if s.as_deref().unwrap().word_index > s.as_deref().unwrap().word_count {
                        __assert_fail(
                            b"s->word_index <= s->word_count\0" as *const u8 as *const i8,
                            b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                                as *const u8 as *const i8,
                            104,
                            [
                                b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'6' as i8,
                                b'4' as i8, b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8,
                                b'p' as i8, b'_' as i8, b'p' as i8, b'e' as i8, b'a' as i8,
                                b'k' as i8, b'_' as i8, b'b' as i8, b'i' as i8, b't' as i8,
                                b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                                b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                                b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8,
                                b' ' as i8, b'i' as i8, b'n' as i8, b't' as i8, b')' as i8,
                                b'\0' as i8,
                            ]
                            .as_ptr(),
                        );
                    }
                } else if s.as_deref().unwrap().final_word_available != 0 {
                    let word = s.as_deref().unwrap().final_word;
                    let shift = s.as_deref().unwrap().count;
                    s.as_deref_mut().unwrap().bits |= (word as u64) << shift;
                    s.as_deref_mut().unwrap().count += s.as_deref().unwrap().bits_left;
                    s.as_deref_mut().unwrap().final_word_available = 0;
                }
            }
            s.as_deref().unwrap().bits
        }

        unsafe fn cp_consume_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u32 {
            if s.as_deref().unwrap().count < num_bits_to_read {
                __assert_fail(
                    b"s->count >= num_bits_to_read\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    115,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'c' as i8, b'o' as i8, b'n' as i8, b's' as i8, b'u' as i8, b'm' as i8,
                        b'e' as i8, b'_' as i8, b'b' as i8, b'i' as i8, b't' as i8, b's' as i8,
                        b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8, b's' as i8, b't' as i8,
                        b'a' as i8, b't' as i8, b'e' as i8, b'_' as i8, b't' as i8, b' ' as i8,
                        b'*' as i8, b',' as i8, b' ' as i8, b'i' as i8, b'n' as i8, b't' as i8,
                        b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            let mask = (1u64 << num_bits_to_read).wrapping_sub(1);
            let bits = (s.as_deref().unwrap().bits & mask) as u32;
            s.as_deref_mut().unwrap().bits >>= num_bits_to_read;
            s.as_deref_mut().unwrap().count -= num_bits_to_read;
            s.unwrap().bits_left -= num_bits_to_read;
            bits
        }

        unsafe fn cp_read_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u32 {
            if num_bits_to_read > 32 {
                __assert_fail(
                    b"num_bits_to_read <= 32\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    123,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if num_bits_to_read < 0 {
                __assert_fail(
                    b"num_bits_to_read >= 0\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    124,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if s.as_deref().unwrap().bits_left <= 0 {
                __assert_fail(
                    b"s->bits_left > 0\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    125,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if s.as_deref().unwrap().count > 64 {
                __assert_fail(
                    b"s->count <= 64\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    126,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if cp_would_overflow(s.as_deref(), num_bits_to_read) != 0 {
                __assert_fail(
                    b"!cp_would_overflow(s, num_bits_to_read)\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    127,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }

            cp_peak_bits(s.as_deref_mut(), num_bits_to_read);
            cp_consume_bits(s, num_bits_to_read)
        }

        #[inline]
        fn cp_rev16(mut a: u32) -> u32 {
            a = (a & 0xaaaau32) >> 1 | (a & 0x5555u32) << 1;
            a = (a & 0xccccu32) >> 2 | (a & 0x3333u32) << 2;
            a = (a & 0xf0f0u32) >> 4 | (a & 0x0f0fu32) << 4;
            a = (a & 0xff00u32) >> 8 | (a & 0x00ffu32) << 8;
            a
        }

        unsafe fn cp_build(
            mut s: Option<&mut cp_state_t>,
            tree: &mut [u32],
            lens: &[u8],
            sym_count: i32,
        ) -> i32 {
            let mut codes: [i32; 16] = [0; 16];
            let mut first: [i32; 16] = [0; 16];
            let mut counts: [i32; 16] = [0; 16];

            for &l in &lens[..(sym_count as usize)] {
                counts
// ... (truncated) ...
```

**Entity:** cp_error_reason (thread_local Cell<*const i8>)

**States:** NoReason (null()), HasReason (non-null pointer to a valid, long-lived C string)

**Transitions:**
- NoReason -> HasReason via setting cp_error_reason to a non-null C string pointer (not shown here, but implied by its existence as an error channel)
- HasReason -> NoReason via resetting to null (implied by initialization `Cell::new(null())`)

**Evidence:** thread_local! `static cp_error_reason: Cell<*const i8> = ... Cell::new(core::ptr::null())` (null represents 'no error'); The stored type is `*const i8` (raw pointer with no lifetime / validity guarantees)

**Implementation:** Introduce `struct ErrorReason(NonNull<c_char>);` and store `Option<ErrorReason>` (or `Cell<Option<NonNull<c_char>>>` via `Cell<usize>` indirection). Provide setters that only accept `&'static CStr` (or an enum of known static reasons) to make the lifetime invariant explicit, and getters returning `Option<&'static CStr>` when applicable.

---

## Protocol Invariants

### 1. cp_state_t bitstream cursor protocol (Aligned / Unaligned, InputRemaining / ExhaustedFinal)

**Location**: `/data/test_case/lib.rs:1-306`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: cp_state_t encodes a bitstream reader with a hidden protocol across its fields (bits/count/words/word_index/word_count/bits_left/final_word_available/final_word). Callers and internal helpers assume (1) certain alignment when taking a raw byte pointer view, (2) that reads only occur when bits remain (bits_left > 0) and do not overflow available bits, and (3) that refilling progresses monotonically through words and optionally a one-time final_word. These invariants are enforced via runtime asserts and ad-hoc checks, not by the type system; nothing prevents constructing a cp_state_t with inconsistent counters or calling functions in the wrong state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct cp_state_t, 11 free function(s)

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
        use core::ffi::c_void;

        extern "C" {
            fn memcpy(__dest: *mut c_void, __src: *const c_void, __n: usize) -> *mut c_void;
            fn memset(__s: *mut c_void, __c: i32, __n: usize) -> *mut c_void;
            fn calloc(__nmemb: usize, __size: usize) -> *mut c_void;
            fn free(__ptr: *mut c_void);
            fn __assert_fail(
                __assertion: *const i8,
                __file: *const i8,
                __line: u32,
                __function: *const i8,
            ) -> !;
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_state_t {
            pub bits: u64,
            pub count: i32,
            pub words: *mut u32,
            pub word_count: i32,
            pub word_index: i32,
            pub bits_left: i32,
            pub final_word_available: i32,
            pub final_word: u32,
            pub out: *mut i8,
            pub out_end: *mut i8,
            pub begin: *mut i8,
            pub lookup: [u16; 512],
            pub lit: [u32; 288],
            pub dst: [u32; 32],
            pub len: [u32; 19],
            pub nlit: u32,
            pub ndst: u32,
            pub nlen: u32,
        }

        thread_local! {
            static cp_error_reason: std::cell::Cell<*const i8> = const { std::cell::Cell::new(core::ptr::null()) };
        }

        // Keep these statics unchanged (no #[no_mangle] added).
        pub static cp_fixed_table: [u8; 320] = [
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];

        pub static cp_permutation_order: [u8; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ];
        pub static cp_len_extra_bits: [u8; 31] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
        ];
        pub static cp_len_base: [u32; 31] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
            163, 195, 227, 258, 0, 0,
        ];
        pub static cp_dist_extra_bits: [u8; 32] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
            13, 0, 0,
        ];
        pub static cp_dist_base: [u32; 32] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049,
            3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
        ];

        #[inline]
        fn cp_would_overflow(s: Option<&cp_state_t>, num_bits: i32) -> i32 {
            (s.unwrap().bits_left + s.unwrap().count - num_bits < 0) as i32
        }

        unsafe fn cp_ptr(s: Option<&cp_state_t>) -> *const i8 {
            if s.unwrap().bits_left & 7 != 0 {
                __assert_fail(
                    b"!(s->bits_left & 7)\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    95,
                    [
                        b'c' as i8, b'h' as i8, b'a' as i8, b'r' as i8, b' ' as i8, b'*' as i8,
                        b'c' as i8, b'p' as i8, b'_' as i8, b'p' as i8, b't' as i8, b'r' as i8,
                        b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8, b's' as i8, b't' as i8,
                        b'a' as i8, b't' as i8, b'e' as i8, b'_' as i8, b't' as i8, b' ' as i8,
                        b'*' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            (s.unwrap().words.offset(s.unwrap().word_index as isize) as *mut i8)
                .offset(-((s.unwrap().count / 8) as isize)) as *const i8
        }

        unsafe fn cp_peak_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u64 {
            if s.as_deref().unwrap().count < num_bits_to_read {
                if s.as_deref().unwrap().word_index < s.as_deref().unwrap().word_count {
                    let idx = s.as_deref().unwrap().word_index;
                    s.as_deref_mut().unwrap().word_index = idx + 1;

                    let word: u32 = *s.as_deref().unwrap().words.add(idx as usize);
                    let shift = s.as_deref().unwrap().count;
                    s.as_deref_mut().unwrap().bits |= (word as u64) << shift;
                    s.as_deref_mut().unwrap().count += 32;

                    if s.as_deref().unwrap().word_index > s.as_deref().unwrap().word_count {
                        __assert_fail(
                            b"s->word_index <= s->word_count\0" as *const u8 as *const i8,
                            b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                                as *const u8 as *const i8,
                            104,
                            [
                                b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'6' as i8,
                                b'4' as i8, b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8,
                                b'p' as i8, b'_' as i8, b'p' as i8, b'e' as i8, b'a' as i8,
                                b'k' as i8, b'_' as i8, b'b' as i8, b'i' as i8, b't' as i8,
                                b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                                b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                                b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8,
                                b' ' as i8, b'i' as i8, b'n' as i8, b't' as i8, b')' as i8,
                                b'\0' as i8,
                            ]
                            .as_ptr(),
                        );
                    }
                } else if s.as_deref().unwrap().final_word_available != 0 {
                    let word = s.as_deref().unwrap().final_word;
                    let shift = s.as_deref().unwrap().count;
                    s.as_deref_mut().unwrap().bits |= (word as u64) << shift;
                    s.as_deref_mut().unwrap().count += s.as_deref().unwrap().bits_left;
                    s.as_deref_mut().unwrap().final_word_available = 0;
                }
            }
            s.as_deref().unwrap().bits
        }

        unsafe fn cp_consume_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u32 {
            if s.as_deref().unwrap().count < num_bits_to_read {
                __assert_fail(
                    b"s->count >= num_bits_to_read\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    115,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'c' as i8, b'o' as i8, b'n' as i8, b's' as i8, b'u' as i8, b'm' as i8,
                        b'e' as i8, b'_' as i8, b'b' as i8, b'i' as i8, b't' as i8, b's' as i8,
                        b'(' as i8, b'c' as i8, b'p' as i8, b'_' as i8, b's' as i8, b't' as i8,
                        b'a' as i8, b't' as i8, b'e' as i8, b'_' as i8, b't' as i8, b' ' as i8,
                        b'*' as i8, b',' as i8, b' ' as i8, b'i' as i8, b'n' as i8, b't' as i8,
                        b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            let mask = (1u64 << num_bits_to_read).wrapping_sub(1);
            let bits = (s.as_deref().unwrap().bits & mask) as u32;
            s.as_deref_mut().unwrap().bits >>= num_bits_to_read;
            s.as_deref_mut().unwrap().count -= num_bits_to_read;
            s.unwrap().bits_left -= num_bits_to_read;
            bits
        }

        unsafe fn cp_read_bits(mut s: Option<&mut cp_state_t>, num_bits_to_read: i32) -> u32 {
            if num_bits_to_read > 32 {
                __assert_fail(
                    b"num_bits_to_read <= 32\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    123,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if num_bits_to_read < 0 {
                __assert_fail(
                    b"num_bits_to_read >= 0\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    124,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if s.as_deref().unwrap().bits_left <= 0 {
                __assert_fail(
                    b"s->bits_left > 0\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    125,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if s.as_deref().unwrap().count > 64 {
                __assert_fail(
                    b"s->count <= 64\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    126,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }
            if cp_would_overflow(s.as_deref(), num_bits_to_read) != 0 {
                __assert_fail(
                    b"!cp_would_overflow(s, num_bits_to_read)\0" as *const u8 as *const i8,
                    b"/home/ubuntu/Test-Corpus/Public-Tests/B02_organic/pinflate_lib/src/pinflate_lib/test_case/src/lib.c\0"
                        as *const u8 as *const i8,
                    127,
                    [
                        b'u' as i8, b'i' as i8, b'n' as i8, b't' as i8, b'3' as i8, b'2' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'c' as i8, b'p' as i8, b'_' as i8,
                        b'r' as i8, b'e' as i8, b'a' as i8, b'd' as i8, b'_' as i8, b'b' as i8,
                        b'i' as i8, b't' as i8, b's' as i8, b'(' as i8, b'c' as i8, b'p' as i8,
                        b'_' as i8, b's' as i8, b't' as i8, b'a' as i8, b't' as i8, b'e' as i8,
                        b'_' as i8, b't' as i8, b' ' as i8, b'*' as i8, b',' as i8, b' ' as i8,
                        b'i' as i8, b'n' as i8, b't' as i8, b')' as i8, b'\0' as i8,
                    ]
                    .as_ptr(),
                );
            }

            cp_peak_bits(s.as_deref_mut(), num_bits_to_read);
            cp_consume_bits(s, num_bits_to_read)
        }

        #[inline]
        fn cp_rev16(mut a: u32) -> u32 {
            a = (a & 0xaaaau32) >> 1 | (a & 0x5555u32) << 1;
            a = (a & 0xccccu32) >> 2 | (a & 0x3333u32) << 2;
            a = (a & 0xf0f0u32) >> 4 | (a & 0x0f0fu32) << 4;
            a = (a & 0xff00u32) >> 8 | (a & 0x00ffu32) << 8;
            a
        }

        unsafe fn cp_build(
            mut s: Option<&mut cp_state_t>,
            tree: &mut [u32],
            lens: &[u8],
            sym_count: i32,
        ) -> i32 {
            let mut codes: [i32; 16] = [0; 16];
            let mut first: [i32; 16] = [0; 16];
            let mut counts: [i32; 16] = [0; 16];

            for &l in &lens[..(sym_count as usize)] {
                counts
// ... (truncated) ...
```

**Entity:** cp_state_t

**States:** ByteAligned (bits_left % 8 == 0), ByteUnaligned (bits_left % 8 != 0), Buffered (count >= needed bits), NeedsRefill (count < needed bits && word_index < word_count), FinalWordAvailable (count < needed bits && word_index >= word_count && final_word_available != 0), Exhausted (word_index >= word_count && final_word_available == 0)

**Transitions:**
- ByteAligned -> ByteUnaligned via cp_consume_bits()/cp_read_bits() consuming non-multiple-of-8 bits
- NeedsRefill -> Buffered via cp_peak_bits() when word_index < word_count (pull 32 bits from words[word_index] and increment word_index)
- FinalWordAvailable -> Buffered via cp_peak_bits() (pull final_word once, then set final_word_available = 0)
- FinalWordAvailable -> Exhausted via cp_peak_bits() (after consuming final_word, final_word_available becomes 0)
- Buffered -> NeedsRefill via cp_consume_bits()/cp_read_bits() decreasing count below future requested size
- Any (with remaining input) -> Exhausted as word_index reaches word_count and final_word_available becomes 0

**Evidence:** cp_state_t fields: bits, count, words, word_count, word_index, bits_left, final_word_available, final_word (state encoded in integers/pointers); cp_ptr(s): asserts byte alignment: `if s.unwrap().bits_left & 7 != 0 { __assert_fail("!(s->bits_left & 7)") }`; cp_peak_bits(s, n): conditional refill depends on `count < n` and either `word_index < word_count` or `final_word_available != 0`; also mutates `word_index` and clears `final_word_available`; cp_peak_bits: assertion that `s->word_index <= s->word_count` after increment (monotonic index / bounds invariant); cp_consume_bits(s, n): asserts `s->count >= num_bits_to_read` before shifting; updates `bits`, `count`, and `bits_left` together (coupled-field invariant); cp_read_bits(s, n): asserts `0 <= n <= 32`, `s->bits_left > 0`, `s->count <= 64`, and `!cp_would_overflow(s, n)` before calling cp_peak_bits + cp_consume_bits; cp_would_overflow(s, n): computes `s.bits_left + s.count - n < 0` (implicit availability invariant)

**Implementation:** Wrap cp_state_t in a safe Rust bit-reader with typestates/capabilities: e.g. `struct BitReader<'a, S> { st: cp_state_t, _s: PhantomData<S>, _buf: PhantomData<&'a [u32]> }`. Provide `fn new_aligned(...) -> BitReader<Aligned>` (enforces bits_left%8==0), `fn read_bits<const N: u8>(&mut self) -> u32` for `N<=32` (compile-time bound), and use a private state transition API so only the reader can mutate `word_index/count/bits_left`. Expose `fn as_ptr(&self) -> *const i8` only for `BitReader<Aligned>` (or via an `Aligned` capability token).

---

### 3. cp_state_t decoding/output buffer protocol (Uninitialized pointers -> Active -> Finished)

**Location**: `/data/test_case/lib.rs:1-25`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: cp_state_t is a C-layout, Copy-able struct that carries raw pointers and cursor/capacity fields (words/word_count/word_index/bits_left and out/out_end/begin) that implicitly form a multi-step decoding protocol. Correctness requires that pointers be non-null and in-bounds before use, that indices/counters remain consistent with the backing buffers, and that output writes only occur while out < out_end. None of these requirements are enforced by the type system because the buffers are represented as *mut pointers plus integer lengths, and the struct is Copy so it can be duplicated, creating multiple mutable aliases to the same underlying buffers.

**Evidence**:

```rust
// Note: Other parts of this module contain: 2 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct cp_state_t {
            pub bits: u64,
            pub count: i32,
            pub words: *mut u32,
            pub word_count: i32,
            pub word_index: i32,
            pub bits_left: i32,
            pub final_word_available: i32,
            pub final_word: u32,
            pub out: *mut i8,
            pub out_end: *mut i8,
            pub begin: *mut i8,
            pub lookup: [u16; 512],
            pub lit: [u32; 288],
            pub dst: [u32; 32],
            pub len: [u32; 19],
            pub nlit: u32,
            pub ndst: u32,
            pub nlen: u32,
        }

```

**Entity:** cp_state_t

**States:** Uninitialized, Active, Finished

**Transitions:**
- Uninitialized -> Active by setting words/out/begin/out_end and initializing cursor fields (word_index, bits_left, word_count, count, bits, etc.)
- Active -> Finished when decoding completes and final_word_available/final_word indicate completion state

**Evidence:** field: words: *mut u32 + word_count: i32 + word_index: i32 (raw buffer + length + cursor; requires word_index within [0, word_count]); field: out: *mut i8 + out_end: *mut i8 + begin: *mut i8 (raw output range; requires begin <= out <= out_end and writes stop at out_end); field: bits: u64 + bits_left: i32 (bit-buffer with remaining count; requires bits_left kept in a valid range for the algorithm); field: final_word_available: i32 + final_word: u32 (flag/value pair encoding a state: whether a final word can/must be consumed); derive: Copy, Clone on cp_state_t (allows duplicating a struct containing *mut pointers, creating implicit aliasing/multiple 'owners' of the same cursors)

**Implementation:** Wrap the raw C struct in a safe Rust API: e.g., `struct CpState<S> { raw: cp_state_t, _s: PhantomData<S> }` with states like `Uninit`, `Active`, `Finished`. Provide constructors that take `&mut [u32]` and `&mut [u8]` (or `NonNull<T>` + lengths) and set pointers/lengths consistently. Only expose decode/step methods on `CpState<Active>`, and return `CpState<Finished>` (or a separate result type) when done. Remove/avoid Copy for the safe wrapper to prevent duplicating mutable cursor state.

---

