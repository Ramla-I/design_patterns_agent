# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 0
- **Resource lifecycle**: 2
- **State machine**: 1
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 3. os_data ownership/initialization protocol (Empty -> PartiallyFilled -> Filled; must be freed)

**Location**: `/data/test_case/lib.rs:1-287`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: parse_uname_string_internal writes multiple fields of os_data by allocating C strings (via strdup/malloc) and storing raw pointers. Which fields are set depends on which parsing branch matched; others remain in their prior state. Callers must (1) pass an os_data whose pointer fields are in a known initial state (typically null), (2) treat non-null fields as owning allocations that must eventually be freed, and (3) avoid double-free/leaks across repeated calls. None of this is represented in the type system because os_data is Copy, holds raw pointers, and there is no Drop/ownership wrapper.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct os_data, 2 free function(s); struct regmatch_t, 2 free function(s); struct re_pattern_buffer

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[macro_use]
extern crate c2rust_bitfields;

extern "C" {
    pub type re_dfa_t;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strdup(__s: *const i8) -> *mut i8;
    fn strstr(__haystack: *const i8, __needle: *const i8) -> *mut i8;
    fn regcomp(__preg: *mut regex_t, __pattern: *const i8, __cflags: i32) -> i32;
    fn regexec(
        __preg: *const regex_t,
        __String: *const i8,
        __nmatch: usize,
        __pmatch: *mut regmatch_t,
        __eflags: i32,
    ) -> i32;
    fn regfree(__preg: *mut regex_t);
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct os_data {
    pub os_name: *mut i8,
    pub os_version: *mut i8,
    pub os_major: *mut i8,
    pub os_minor: *mut i8,
    pub os_codename: *mut i8,
    pub os_platform: *mut i8,
    pub os_build: *mut i8,
    pub os_uname: *mut i8,
    pub os_arch: *mut i8,
}

pub type regoff_t = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}

pub type regex_t = re_pattern_buffer;

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(BitfieldStruct)]
pub struct re_pattern_buffer {
    pub __buffer: *mut re_dfa_t,
    pub __allocated: __re_long_size_t,
    pub __used: __re_long_size_t,
    pub __syntax: reg_syntax_t,
    pub __fastmap: *mut i8,
    pub __translate: *mut u8,
    pub re_nsub: usize,
    pub __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
        [u8; 1],
    pub c2rust_padding: [u8; 7],
}

pub type reg_syntax_t = u64;
pub type __re_long_size_t = u64;

pub const REG_EXTENDED: i32 = 1;

#[inline]
unsafe fn bytes_as_i8_slice(bytes: &[u8]) -> &[i8] {
    core::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len())
}

pub(crate) unsafe fn get_os_arch(os_header: &[i8]) -> *mut i8 {
    const ARCHS: [*const i8; 13] = [
        b"x86_64\0" as *const u8 as *const i8,
        b"i386\0" as *const u8 as *const i8,
        b"i686\0" as *const u8 as *const i8,
        b"sparc\0" as *const u8 as *const i8,
        b"amd64\0" as *const u8 as *const i8,
        b"i86pc\0" as *const u8 as *const i8,
        b"ia64\0" as *const u8 as *const i8,
        b"AIX\0" as *const u8 as *const i8,
        b"armv6\0" as *const u8 as *const i8,
        b"armv7\0" as *const u8 as *const i8,
        b"aarch64\0" as *const u8 as *const i8,
        b"arm64\0" as *const u8 as *const i8,
        core::ptr::null(),
    ];

    if os_header.is_empty() {
        return core::ptr::null_mut();
    }

    for &arch in ARCHS.iter() {
        if arch.is_null() {
            break;
        }
        if !strstr(os_header.as_ptr(), arch).is_null() {
            return strdup(arch);
        }
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn w_regexec(
    pattern: &[i8],
    string: Option<&i8>,
    nmatch: usize,
    pmatch: Option<&regmatch_t>,
) -> i32 {
    let mut regex: regex_t = regex_t {
        __buffer: core::ptr::null_mut(),
        __allocated: 0,
        __used: 0,
        __syntax: 0,
        __fastmap: core::ptr::null_mut(),
        __translate: core::ptr::null_mut(),
        re_nsub: 0,
        __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
            [0; 1],
        c2rust_padding: [0; 7],
    };

    if pattern.is_empty() || string.is_none() {
        return 0;
    }

    if regcomp(&raw mut regex, pattern.as_ptr(), REG_EXTENDED) != 0 {
        // pattern is expected to be NUL-terminated
        let pat = core::ffi::CStr::from_ptr(pattern.as_ptr());
        eprint!(
            "Couldn\'t compile regular expression \'{0}\'\n",
            pat.to_string_lossy()
        );
        return 0;
    }

    let result = regexec(
        &raw const regex,
        string.map_or(core::ptr::null::<i8>(), |p| p),
        nmatch,
        pmatch
            .map_or(core::ptr::null::<regmatch_t>(), |p| p)
            .cast_mut(),
        0,
    );

    regfree(&raw mut regex);
    (result == 0) as i32
}

#[inline]
unsafe fn trim_trailing_last_char_to_nul(s: *mut i8) {
    let len = core::ffi::CStr::from_ptr(s).count_bytes();
    if len > 0 {
        *s.add(len).sub(1) = 0;
    }
}

#[inline]
unsafe fn alloc_and_copy_match(src: *const i8, m: regmatch_t) -> *mut i8 {
    let match_size = m.rm_eo - m.rm_so;
    if match_size <= 0 {
        return core::ptr::null_mut();
    }
    let dst = malloc((match_size + 1) as usize) as *mut i8;
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    snprintf(
        dst,
        (match_size + 1) as usize,
        b"%.*s\0" as *const u8 as *const i8,
        match_size,
        src.offset(m.rm_so as isize),
    );
    dst
}

pub(crate) unsafe fn parse_uname_string_internal(uname: &mut [i8], osd: Option<&mut os_data>) {
    let Some(osd) = osd else { return };

    let re_major = bytes_as_i8_slice(b"^([0-9]+)\\.*\0");
    let re_minor = bytes_as_i8_slice(b"^[0-9]+\\.([0-9]+)\\.*\0");
    let re_build = bytes_as_i8_slice(b"^[0-9]+\\.[0-9]+\\.([0-9]+(\\.[0-9]+)*)\\.*\0");

    let mut match_0: [regmatch_t; 2] = [
        regmatch_t { rm_so: 0, rm_eo: 0 },
        regmatch_t { rm_so: 0, rm_eo: 0 },
    ];

    let mut str_tmp = strstr(uname.as_mut_ptr(), b" [Ver: \0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(7);

        osd.os_name = strdup(uname.as_mut_ptr());

        trim_trailing_last_char_to_nul(str_tmp);

        if w_regexec(re_major, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_major = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_minor, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_minor = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_build, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_build = alloc_and_copy_match(str_tmp, match_0[1]);
        }

        osd.os_version = strdup(str_tmp);
        osd.os_platform = strdup(b"windows\0" as *const u8 as *const i8);
        return;
    }

    str_tmp = strstr(uname.as_mut_ptr(), b" [\0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(2);

        osd.os_name = strdup(str_tmp);

        str_tmp = strstr(osd.os_name as *const i8, b": \0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(2);

            osd.os_version = strdup(str_tmp);
            trim_trailing_last_char_to_nul(osd.os_version);

            str_tmp = strstr(osd.os_version as *const i8, b" (\0" as *const u8 as *const i8);
            if !str_tmp.is_null() {
                *str_tmp = 0;
                str_tmp = str_tmp.add(2);

                osd.os_codename = strdup(str_tmp);
                trim_trailing_last_char_to_nul(osd.os_codename);
            }

            if w_regexec(re_major, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_major = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
            if w_regexec(re_minor, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_minor = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
        } else {
            trim_trailing_last_char_to_nul(osd.os_name);
        }

        str_tmp = strstr(osd.os_name as *const i8, b"|\0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(1);
            osd.os_platform = strdup(str_tmp);
        }
    }

    let arch = get_os_arch(uname);
    if !arch.is_null() {
        osd.os_arch = strdup(arch);
        free(arch as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn parse_uname_string(mut uname: *mut i8, mut osd: Option<&mut os_data>) {
    parse_uname_string_internal(
        if uname.is_null() {
            &mut []
        } else {
            core::slice::from_raw_parts_mut(uname, 1024)
        },
        osd,
    )
}
```

**Entity:** os_data

**States:** Empty (all pointers null/invalid), PartiallyFilled (some fields point to allocated C strings), Filled (all expected fields allocated), Freed (pointers no longer valid)

**Transitions:**
- Empty -> PartiallyFilled via parse_uname_string_internal()/parse_uname_string() writing some fields (e.g., os_name/os_version)
- PartiallyFilled -> Filled via parse_uname_string_internal() writing more fields (os_major/os_minor/os_build/os_codename/os_platform/os_arch)
- Filled/PartiallyFilled -> Freed via caller freeing all non-null fields (not present in this snippet, but required by the allocations performed)

**Evidence:** struct os_data fields are raw pointers: os_name, os_version, os_major, os_minor, os_codename, os_platform, os_build, os_uname, os_arch; os_data derives Copy, Clone (so pointer-owning values can be trivially duplicated, risking double-free if freed later); parse_uname_string_internal: osd.os_name = strdup(...); osd.os_version = strdup(...); osd.os_platform = strdup(...); osd.os_codename = strdup(...); osd.os_major/os_minor/os_build set from alloc_and_copy_match() (malloc + snprintf); alloc_and_copy_match(): uses malloc() to allocate dst and returns *mut i8 that must be freed by someone; parse_uname_string_internal: osd.os_arch = strdup(arch); free(arch ...) (shows explicit ownership transfer expectations)

**Implementation:** Wrap owned C strings in a non-Copy RAII type (e.g., struct OwnedCStr(*mut c_char) with Drop calling free). Then define struct OsData { os_name: Option<OwnedCStr>, ... } (or use CString when allocation comes from Rust). This makes ownership explicit, prevents accidental Copy, and can enforce initialization by using Option fields rather than raw pointers.

---

### 2. re_pattern_buffer initialization & ownership protocol (Uninitialized/Empty -> Compiled/Ready -> Freed)

**Location**: `/data/test_case/lib.rs:1-17`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: This is an FFI-facing struct that encodes internal allocation/initialization state with raw pointers and size counters. Correct usage implicitly requires: (1) pointers like __buffer/__fastmap/__translate are either null/unused or point to valid allocated memory compatible with the associated metadata fields (__allocated/__used and bitflags), (2) the struct must be properly initialized/compiled before consumers dereference/use those pointers or trust counters, and (3) any owned allocations must be released exactly once, after which the struct must not be used again. None of these states or transitions are represented in the type system because all resources are raw pointers and scalar fields (Copy/Clone even allows accidental duplication of ownership).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct os_data, 2 free function(s); struct regmatch_t, 2 free function(s); 3 free function(s)

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(BitfieldStruct)]
pub struct re_pattern_buffer {
    pub __buffer: *mut re_dfa_t,
    pub __allocated: __re_long_size_t,
    pub __used: __re_long_size_t,
    pub __syntax: reg_syntax_t,
    pub __fastmap: *mut i8,
    pub __translate: *mut u8,
    pub re_nsub: usize,
    pub __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
        [u8; 1],
    pub c2rust_padding: [u8; 7],
}

```

**Entity:** re_pattern_buffer

**States:** Uninitialized/Empty, Compiled/Ready, Freed/Invalid

**Transitions:**
- Uninitialized/Empty -> Compiled/Ready via (external/FFI) regex compile/initialize routine that populates __buffer/__allocated/__used/__syntax and optionally __fastmap/__translate
- Compiled/Ready -> Freed/Invalid via (external/FFI) free/cleanup routine that releases __buffer/__fastmap/__translate and invalidates associated metadata

**Evidence:** line 6: pub __buffer: *mut re_dfa_t (raw owning/borrowing pointer; validity depends on initialization and lifetime); line 7-8: pub __allocated / pub __used size counters (imply invariant __used <= __allocated and that __buffer is valid for that extent); line 10-11: pub __fastmap: *mut i8 and pub __translate: *mut u8 (optional auxiliary allocations; require correct allocation/free discipline); line 15: field name '__can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor' indicates packed flags controlling whether fastmap/regs/etc. are allocated/accurate, but they are not type-checked; line 4: #[derive(Copy, Clone)] on a struct containing raw pointers makes it easy to duplicate a value that implicitly refers to the same underlying allocations, risking double-free/use-after-free

**Implementation:** Wrap the FFI struct in a safe Rust API with explicit states, e.g. `struct PatternBuf<S> { inner: re_pattern_buffer, _s: PhantomData<S> }` with zero-sized states `Uninit`, `Compiled`. Provide constructors that produce `PatternBuf<Uninit>` (zeroed/null pointers), a `compile(self, ...) -> Result<PatternBuf<Compiled>, Error>` transition that performs FFI initialization, and implement `Drop` only for `PatternBuf<Compiled>` to call the FFI free routine. Avoid `Copy/Clone` on the safe wrapper; if cloning is needed, make it explicit and define whether it shares or deep-copies the underlying resources.

---

## State Machine Invariants

### 4. regex_t compile/free lifecycle (Uncompiled -> Compiled -> Freed) tied to regcomp/regfree

**Location**: `/data/test_case/lib.rs:1-287`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: w_regexec manually manages a C regex object: it must be in an initialized/zeroed state before regcomp; regexec must only be called after successful regcomp; and regfree must be called exactly once after regcomp to release internal allocations. The function encodes this with runtime checks/early returns and explicit regfree. The type system does not prevent calling regexec on an uncompiled regex or forgetting regfree in other usages of regex_t.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct os_data, 2 free function(s); struct regmatch_t, 2 free function(s); struct re_pattern_buffer

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[macro_use]
extern crate c2rust_bitfields;

extern "C" {
    pub type re_dfa_t;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strdup(__s: *const i8) -> *mut i8;
    fn strstr(__haystack: *const i8, __needle: *const i8) -> *mut i8;
    fn regcomp(__preg: *mut regex_t, __pattern: *const i8, __cflags: i32) -> i32;
    fn regexec(
        __preg: *const regex_t,
        __String: *const i8,
        __nmatch: usize,
        __pmatch: *mut regmatch_t,
        __eflags: i32,
    ) -> i32;
    fn regfree(__preg: *mut regex_t);
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct os_data {
    pub os_name: *mut i8,
    pub os_version: *mut i8,
    pub os_major: *mut i8,
    pub os_minor: *mut i8,
    pub os_codename: *mut i8,
    pub os_platform: *mut i8,
    pub os_build: *mut i8,
    pub os_uname: *mut i8,
    pub os_arch: *mut i8,
}

pub type regoff_t = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}

pub type regex_t = re_pattern_buffer;

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(BitfieldStruct)]
pub struct re_pattern_buffer {
    pub __buffer: *mut re_dfa_t,
    pub __allocated: __re_long_size_t,
    pub __used: __re_long_size_t,
    pub __syntax: reg_syntax_t,
    pub __fastmap: *mut i8,
    pub __translate: *mut u8,
    pub re_nsub: usize,
    pub __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
        [u8; 1],
    pub c2rust_padding: [u8; 7],
}

pub type reg_syntax_t = u64;
pub type __re_long_size_t = u64;

pub const REG_EXTENDED: i32 = 1;

#[inline]
unsafe fn bytes_as_i8_slice(bytes: &[u8]) -> &[i8] {
    core::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len())
}

pub(crate) unsafe fn get_os_arch(os_header: &[i8]) -> *mut i8 {
    const ARCHS: [*const i8; 13] = [
        b"x86_64\0" as *const u8 as *const i8,
        b"i386\0" as *const u8 as *const i8,
        b"i686\0" as *const u8 as *const i8,
        b"sparc\0" as *const u8 as *const i8,
        b"amd64\0" as *const u8 as *const i8,
        b"i86pc\0" as *const u8 as *const i8,
        b"ia64\0" as *const u8 as *const i8,
        b"AIX\0" as *const u8 as *const i8,
        b"armv6\0" as *const u8 as *const i8,
        b"armv7\0" as *const u8 as *const i8,
        b"aarch64\0" as *const u8 as *const i8,
        b"arm64\0" as *const u8 as *const i8,
        core::ptr::null(),
    ];

    if os_header.is_empty() {
        return core::ptr::null_mut();
    }

    for &arch in ARCHS.iter() {
        if arch.is_null() {
            break;
        }
        if !strstr(os_header.as_ptr(), arch).is_null() {
            return strdup(arch);
        }
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn w_regexec(
    pattern: &[i8],
    string: Option<&i8>,
    nmatch: usize,
    pmatch: Option<&regmatch_t>,
) -> i32 {
    let mut regex: regex_t = regex_t {
        __buffer: core::ptr::null_mut(),
        __allocated: 0,
        __used: 0,
        __syntax: 0,
        __fastmap: core::ptr::null_mut(),
        __translate: core::ptr::null_mut(),
        re_nsub: 0,
        __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
            [0; 1],
        c2rust_padding: [0; 7],
    };

    if pattern.is_empty() || string.is_none() {
        return 0;
    }

    if regcomp(&raw mut regex, pattern.as_ptr(), REG_EXTENDED) != 0 {
        // pattern is expected to be NUL-terminated
        let pat = core::ffi::CStr::from_ptr(pattern.as_ptr());
        eprint!(
            "Couldn\'t compile regular expression \'{0}\'\n",
            pat.to_string_lossy()
        );
        return 0;
    }

    let result = regexec(
        &raw const regex,
        string.map_or(core::ptr::null::<i8>(), |p| p),
        nmatch,
        pmatch
            .map_or(core::ptr::null::<regmatch_t>(), |p| p)
            .cast_mut(),
        0,
    );

    regfree(&raw mut regex);
    (result == 0) as i32
}

#[inline]
unsafe fn trim_trailing_last_char_to_nul(s: *mut i8) {
    let len = core::ffi::CStr::from_ptr(s).count_bytes();
    if len > 0 {
        *s.add(len).sub(1) = 0;
    }
}

#[inline]
unsafe fn alloc_and_copy_match(src: *const i8, m: regmatch_t) -> *mut i8 {
    let match_size = m.rm_eo - m.rm_so;
    if match_size <= 0 {
        return core::ptr::null_mut();
    }
    let dst = malloc((match_size + 1) as usize) as *mut i8;
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    snprintf(
        dst,
        (match_size + 1) as usize,
        b"%.*s\0" as *const u8 as *const i8,
        match_size,
        src.offset(m.rm_so as isize),
    );
    dst
}

pub(crate) unsafe fn parse_uname_string_internal(uname: &mut [i8], osd: Option<&mut os_data>) {
    let Some(osd) = osd else { return };

    let re_major = bytes_as_i8_slice(b"^([0-9]+)\\.*\0");
    let re_minor = bytes_as_i8_slice(b"^[0-9]+\\.([0-9]+)\\.*\0");
    let re_build = bytes_as_i8_slice(b"^[0-9]+\\.[0-9]+\\.([0-9]+(\\.[0-9]+)*)\\.*\0");

    let mut match_0: [regmatch_t; 2] = [
        regmatch_t { rm_so: 0, rm_eo: 0 },
        regmatch_t { rm_so: 0, rm_eo: 0 },
    ];

    let mut str_tmp = strstr(uname.as_mut_ptr(), b" [Ver: \0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(7);

        osd.os_name = strdup(uname.as_mut_ptr());

        trim_trailing_last_char_to_nul(str_tmp);

        if w_regexec(re_major, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_major = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_minor, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_minor = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_build, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_build = alloc_and_copy_match(str_tmp, match_0[1]);
        }

        osd.os_version = strdup(str_tmp);
        osd.os_platform = strdup(b"windows\0" as *const u8 as *const i8);
        return;
    }

    str_tmp = strstr(uname.as_mut_ptr(), b" [\0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(2);

        osd.os_name = strdup(str_tmp);

        str_tmp = strstr(osd.os_name as *const i8, b": \0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(2);

            osd.os_version = strdup(str_tmp);
            trim_trailing_last_char_to_nul(osd.os_version);

            str_tmp = strstr(osd.os_version as *const i8, b" (\0" as *const u8 as *const i8);
            if !str_tmp.is_null() {
                *str_tmp = 0;
                str_tmp = str_tmp.add(2);

                osd.os_codename = strdup(str_tmp);
                trim_trailing_last_char_to_nul(osd.os_codename);
            }

            if w_regexec(re_major, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_major = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
            if w_regexec(re_minor, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_minor = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
        } else {
            trim_trailing_last_char_to_nul(osd.os_name);
        }

        str_tmp = strstr(osd.os_name as *const i8, b"|\0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(1);
            osd.os_platform = strdup(str_tmp);
        }
    }

    let arch = get_os_arch(uname);
    if !arch.is_null() {
        osd.os_arch = strdup(arch);
        free(arch as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn parse_uname_string(mut uname: *mut i8, mut osd: Option<&mut os_data>) {
    parse_uname_string_internal(
        if uname.is_null() {
            &mut []
        } else {
            core::slice::from_raw_parts_mut(uname, 1024)
        },
        osd,
    )
}
```

**Entity:** w_regexec (regex_t / re_pattern_buffer)

**States:** Uncompiled (zeroed regex_t), Compiled (regcomp succeeded; internal allocations active), Freed (regfree called; internal pointers invalid)

**Transitions:**
- Uncompiled -> Compiled via regcomp(&mut regex, ...)
- Compiled -> Freed via regfree(&mut regex)
- Compiled -> Freed after regexec via regfree(&mut regex) (normal path)

**Evidence:** w_regexec: creates let mut regex: regex_t = ... with null pointers/__allocated=0 etc. (manual 'uncompiled' initialization); w_regexec: if regcomp(&raw mut regex, ...) != 0 { ... return 0; } (only on success does it proceed to regexec); w_regexec: let result = regexec(&raw const regex, ...); regfree(&raw mut regex); (regfree required after regexec); comment in w_regexec: "pattern is expected to be NUL-terminated" indicates additional precondition coupled to regcomp/regexec usage

**Implementation:** Introduce a safe wrapper like struct CompiledRegex { inner: regex_t } with a constructor compile(pattern: &CStr) -> Result<CompiledRegex, ...>; implement Drop for CompiledRegex calling regfree(&mut inner). Expose exec(&self, ...) only on CompiledRegex so regexec cannot be used before compilation and regfree cannot be forgotten/doubled.

---

## Precondition Invariants

### 1. FFI string-pointer validity/ownership invariant (NULL / Valid CStr / Valid CString-owned)

**Location**: `/data/test_case/lib.rs:1-16`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: All fields in os_data are raw mutable pointers (*mut i8) that are implicitly expected to either be null (meaning 'no value') or point to valid NUL-terminated C strings for the duration they are used. The type does not encode whether each pointer is null, whether it points to a properly terminated string, whether the memory is initialized, or who owns/frees it. Copy/Clone further allows duplicating these pointers without tracking aliasing or ownership, making double-free/dangling-pointer bugs possible if any consumer treats them as owned strings.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct regmatch_t, 2 free function(s); struct re_pattern_buffer; 3 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct os_data {
    pub os_name: *mut i8,
    pub os_version: *mut i8,
    pub os_major: *mut i8,
    pub os_minor: *mut i8,
    pub os_codename: *mut i8,
    pub os_platform: *mut i8,
    pub os_build: *mut i8,
    pub os_uname: *mut i8,
    pub os_arch: *mut i8,
}

```

**Entity:** os_data

**States:** Null (absent), Non-null pointer to valid NUL-terminated C string, Dangling/invalid pointer (UB if read)

**Transitions:**
- Null -> Non-null via FFI/population code writing pointers into fields (not shown)
- Non-null -> Dangling/invalid via freeing underlying allocation while copies of os_data still exist (not prevented)

**Evidence:** struct os_data fields: os_name/os_version/... are *mut i8 (raw pointers with no validity/ownership encoded); #[derive(Copy, Clone)] on os_data enables bitwise copies of the raw pointers

**Implementation:** Replace *mut i8 fields with safer wrappers encoding invariants, e.g. Option<NonNull<c_char>> for borrowed C strings, or Option<CString> (or Box<CStr>) for owned strings. If these are borrowed from FFI, use *const c_char (not mut) plus lifetimes: struct OsData<'a> { os_name: Option<&'a CStr>, ... }. If owned, drop Copy/Clone and implement Drop to free allocations (RAII).

---

### 5. uname buffer protocol (NUL-terminated, writable, sized >= needed; input gets mutated)

**Location**: `/data/test_case/lib.rs:1-287`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The parsing functions assume the uname input is a writable, NUL-terminated C string stored in a buffer large enough for operations like strstr and CStr::from_ptr. The code also mutates the buffer in place by writing NUL bytes at delimiter locations. The extern API accepts a raw pointer and internally treats it as a 1024-byte mutable slice, but the type system does not enforce that the pointer is valid for 1024 bytes, is writable, or is NUL-terminated.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct os_data, 2 free function(s); struct regmatch_t, 2 free function(s); struct re_pattern_buffer

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

#[macro_use]
extern crate c2rust_bitfields;

extern "C" {
    pub type re_dfa_t;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strdup(__s: *const i8) -> *mut i8;
    fn strstr(__haystack: *const i8, __needle: *const i8) -> *mut i8;
    fn regcomp(__preg: *mut regex_t, __pattern: *const i8, __cflags: i32) -> i32;
    fn regexec(
        __preg: *const regex_t,
        __String: *const i8,
        __nmatch: usize,
        __pmatch: *mut regmatch_t,
        __eflags: i32,
    ) -> i32;
    fn regfree(__preg: *mut regex_t);
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct os_data {
    pub os_name: *mut i8,
    pub os_version: *mut i8,
    pub os_major: *mut i8,
    pub os_minor: *mut i8,
    pub os_codename: *mut i8,
    pub os_platform: *mut i8,
    pub os_build: *mut i8,
    pub os_uname: *mut i8,
    pub os_arch: *mut i8,
}

pub type regoff_t = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}

pub type regex_t = re_pattern_buffer;

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(BitfieldStruct)]
pub struct re_pattern_buffer {
    pub __buffer: *mut re_dfa_t,
    pub __allocated: __re_long_size_t,
    pub __used: __re_long_size_t,
    pub __syntax: reg_syntax_t,
    pub __fastmap: *mut i8,
    pub __translate: *mut u8,
    pub re_nsub: usize,
    pub __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
        [u8; 1],
    pub c2rust_padding: [u8; 7],
}

pub type reg_syntax_t = u64;
pub type __re_long_size_t = u64;

pub const REG_EXTENDED: i32 = 1;

#[inline]
unsafe fn bytes_as_i8_slice(bytes: &[u8]) -> &[i8] {
    core::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len())
}

pub(crate) unsafe fn get_os_arch(os_header: &[i8]) -> *mut i8 {
    const ARCHS: [*const i8; 13] = [
        b"x86_64\0" as *const u8 as *const i8,
        b"i386\0" as *const u8 as *const i8,
        b"i686\0" as *const u8 as *const i8,
        b"sparc\0" as *const u8 as *const i8,
        b"amd64\0" as *const u8 as *const i8,
        b"i86pc\0" as *const u8 as *const i8,
        b"ia64\0" as *const u8 as *const i8,
        b"AIX\0" as *const u8 as *const i8,
        b"armv6\0" as *const u8 as *const i8,
        b"armv7\0" as *const u8 as *const i8,
        b"aarch64\0" as *const u8 as *const i8,
        b"arm64\0" as *const u8 as *const i8,
        core::ptr::null(),
    ];

    if os_header.is_empty() {
        return core::ptr::null_mut();
    }

    for &arch in ARCHS.iter() {
        if arch.is_null() {
            break;
        }
        if !strstr(os_header.as_ptr(), arch).is_null() {
            return strdup(arch);
        }
    }
    core::ptr::null_mut()
}

pub(crate) unsafe fn w_regexec(
    pattern: &[i8],
    string: Option<&i8>,
    nmatch: usize,
    pmatch: Option<&regmatch_t>,
) -> i32 {
    let mut regex: regex_t = regex_t {
        __buffer: core::ptr::null_mut(),
        __allocated: 0,
        __used: 0,
        __syntax: 0,
        __fastmap: core::ptr::null_mut(),
        __translate: core::ptr::null_mut(),
        re_nsub: 0,
        __can_be_null___regs_allocated___fastmap_accurate___no_sub___not_bol___not_eol___newline_anchor:
            [0; 1],
        c2rust_padding: [0; 7],
    };

    if pattern.is_empty() || string.is_none() {
        return 0;
    }

    if regcomp(&raw mut regex, pattern.as_ptr(), REG_EXTENDED) != 0 {
        // pattern is expected to be NUL-terminated
        let pat = core::ffi::CStr::from_ptr(pattern.as_ptr());
        eprint!(
            "Couldn\'t compile regular expression \'{0}\'\n",
            pat.to_string_lossy()
        );
        return 0;
    }

    let result = regexec(
        &raw const regex,
        string.map_or(core::ptr::null::<i8>(), |p| p),
        nmatch,
        pmatch
            .map_or(core::ptr::null::<regmatch_t>(), |p| p)
            .cast_mut(),
        0,
    );

    regfree(&raw mut regex);
    (result == 0) as i32
}

#[inline]
unsafe fn trim_trailing_last_char_to_nul(s: *mut i8) {
    let len = core::ffi::CStr::from_ptr(s).count_bytes();
    if len > 0 {
        *s.add(len).sub(1) = 0;
    }
}

#[inline]
unsafe fn alloc_and_copy_match(src: *const i8, m: regmatch_t) -> *mut i8 {
    let match_size = m.rm_eo - m.rm_so;
    if match_size <= 0 {
        return core::ptr::null_mut();
    }
    let dst = malloc((match_size + 1) as usize) as *mut i8;
    if dst.is_null() {
        return core::ptr::null_mut();
    }
    snprintf(
        dst,
        (match_size + 1) as usize,
        b"%.*s\0" as *const u8 as *const i8,
        match_size,
        src.offset(m.rm_so as isize),
    );
    dst
}

pub(crate) unsafe fn parse_uname_string_internal(uname: &mut [i8], osd: Option<&mut os_data>) {
    let Some(osd) = osd else { return };

    let re_major = bytes_as_i8_slice(b"^([0-9]+)\\.*\0");
    let re_minor = bytes_as_i8_slice(b"^[0-9]+\\.([0-9]+)\\.*\0");
    let re_build = bytes_as_i8_slice(b"^[0-9]+\\.[0-9]+\\.([0-9]+(\\.[0-9]+)*)\\.*\0");

    let mut match_0: [regmatch_t; 2] = [
        regmatch_t { rm_so: 0, rm_eo: 0 },
        regmatch_t { rm_so: 0, rm_eo: 0 },
    ];

    let mut str_tmp = strstr(uname.as_mut_ptr(), b" [Ver: \0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(7);

        osd.os_name = strdup(uname.as_mut_ptr());

        trim_trailing_last_char_to_nul(str_tmp);

        if w_regexec(re_major, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_major = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_minor, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_minor = alloc_and_copy_match(str_tmp, match_0[1]);
        }
        if w_regexec(re_build, str_tmp.as_ref(), 2, Some(&match_0[0])) != 0 {
            osd.os_build = alloc_and_copy_match(str_tmp, match_0[1]);
        }

        osd.os_version = strdup(str_tmp);
        osd.os_platform = strdup(b"windows\0" as *const u8 as *const i8);
        return;
    }

    str_tmp = strstr(uname.as_mut_ptr(), b" [\0" as *const u8 as *const i8);
    if !str_tmp.is_null() {
        *str_tmp = 0;
        str_tmp = str_tmp.add(2);

        osd.os_name = strdup(str_tmp);

        str_tmp = strstr(osd.os_name as *const i8, b": \0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(2);

            osd.os_version = strdup(str_tmp);
            trim_trailing_last_char_to_nul(osd.os_version);

            str_tmp = strstr(osd.os_version as *const i8, b" (\0" as *const u8 as *const i8);
            if !str_tmp.is_null() {
                *str_tmp = 0;
                str_tmp = str_tmp.add(2);

                osd.os_codename = strdup(str_tmp);
                trim_trailing_last_char_to_nul(osd.os_codename);
            }

            if w_regexec(re_major, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_major = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
            if w_regexec(re_minor, osd.os_version.as_ref(), 2, Some(&match_0[0])) != 0 {
                osd.os_minor = alloc_and_copy_match(osd.os_version, match_0[1]);
            }
        } else {
            trim_trailing_last_char_to_nul(osd.os_name);
        }

        str_tmp = strstr(osd.os_name as *const i8, b"|\0" as *const u8 as *const i8);
        if !str_tmp.is_null() {
            *str_tmp = 0;
            str_tmp = str_tmp.add(1);
            osd.os_platform = strdup(str_tmp);
        }
    }

    let arch = get_os_arch(uname);
    if !arch.is_null() {
        osd.os_arch = strdup(arch);
        free(arch as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn parse_uname_string(mut uname: *mut i8, mut osd: Option<&mut os_data>) {
    parse_uname_string_internal(
        if uname.is_null() {
            &mut []
        } else {
            core::slice::from_raw_parts_mut(uname, 1024)
        },
        osd,
    )
}
```

**Entity:** parse_uname_string / parse_uname_string_internal

**States:** ValidWritableCStrBuffer, Invalid/Null/NotNULTerminated, MutatedInPlace (delimiters replaced by NUL)

**Transitions:**
- ValidWritableCStrBuffer -> MutatedInPlace via parse_uname_string_internal setting *str_tmp = 0 and trim_trailing_last_char_to_nul()
- Invalid/Null/NotNULTerminated -> (UB or incorrect behavior) via calls to strstr/CStr::from_ptr (no compile-time prevention)

**Evidence:** parse_uname_string: converts uname pointer to core::slice::from_raw_parts_mut(uname, 1024) (assumes at least 1024 writable bytes); parse_uname_string_internal: uses strstr(uname.as_mut_ptr(), ...) (requires NUL-terminated C string semantics); parse_uname_string_internal: writes through the input buffer: *str_tmp = 0; and trim_trailing_last_char_to_nul(str_tmp) which does *s.add(len).sub(1) = 0; trim_trailing_last_char_to_nul: uses core::ffi::CStr::from_ptr(s).count_bytes() (requires NUL-termination or else reads past bounds)

**Implementation:** Provide a safe entrypoint that takes &mut CStr/`&mut [u8]` plus an explicit length, validates there is a NUL within bounds, and wraps it in a newtype like struct WritableCStrBuf<'a>(&'a mut [c_char]); expose parse_uname_string_internal only for this validated wrapper. Alternatively accept &mut CString and operate on its internal buffer.

---

