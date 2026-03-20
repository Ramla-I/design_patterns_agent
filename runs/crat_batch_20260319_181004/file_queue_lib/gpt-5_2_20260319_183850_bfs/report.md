# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. file_queue file-handle initialization/lifecycle (NoFile / FileOpen)

**Location**: `/data/test_case/lib.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: file_queue encodes (at runtime) whether it currently owns an open file handle via `fp: Option<BufReader<File>>`. When `fp` is `None`, any logic that expects to read/seek/operate on the file must not run; when `fp` is `Some(_)`, the queue owns an open file resource and related metadata fields (e.g., `file_name`, `f_status`, timestamps) are presumably meaningful/consistent with that open file. The type system does not distinguish the 'no file attached' vs 'file attached' states, so callers must rely on runtime `Option` checking and conventions to avoid using file-dependent fields/operations in the `None` state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct tm, 2 free function(s); struct timespec; struct stat; struct alert_data, 4 free function(s); struct timeval; struct fd_set; trait AsRawFd, 11 free function(s), impl AsRawFd for std :: io :: BufReader < std :: fs :: File > (1 methods)

        }

        #[repr(C)]
        pub struct file_queue {
            pub last_change: time_t,
            pub year: i32,
            pub day: i32,
            pub flags: i32,
            pub mon: [i8; 4],
            pub file_name: [i8; 257],
            pub fp: Option<std::io::BufReader<std::fs::File>>,
            pub f_status: stat,
        }

```

**Entity:** file_queue

**States:** NoFile, FileOpen

**Transitions:**
- NoFile -> FileOpen by setting `fp` from `None` to `Some(BufReader<File>)`
- FileOpen -> NoFile by setting `fp` to `None` (dropping the reader/file)

**Evidence:** field `fp: Option<std::io::BufReader<std::fs::File>>` encodes presence/absence of an owned open file at runtime; fields `file_name: [i8; 257]` and `f_status: stat` suggest file-related metadata that is only meaningful when a file is present (but not tied to `fp` by types)

**Implementation:** Represent the two states explicitly: `struct FileQueue<S> { last_change: time_t, year: i32, day: i32, flags: i32, mon: [i8; 4], file_name: [i8; 257], f_status: stat, fp: S }` where `S` is either `NoFile` (no `fp` field) or `FileOpen { fp: BufReader<File> }` (or use `PhantomData` plus separate storage). Provide constructors/transition methods like `attach_file(self, fp: BufReader<File>) -> FileQueue<FileOpen>` and `detach_file(self) -> FileQueue<NoFile>`. File-dependent APIs would only be implemented for `FileQueue<FileOpen>`.

---

## State Machine Invariants

### 2. file_queue lifecycle (Name set -> File opened -> Positioned/Reading)

**Location**: `/data/test_case/lib.rs:1-556`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `file_queue` is treated as moving through implicit runtime states. `GetFile_Queue()` initializes `file_name` and chooses a name based on `flags` (stdin vs alerts file). `Handle_Queue()` then conditionally opens `fp` based on flags and expects `fp` to be `Some` for later operations like seeking/reading. Failures are represented by returning 0 or null pointers and by setting `fp = None`, but the type system does not distinguish a queue whose `file_name` is valid, whose `fp` is open, or whose state is EOF/error; callers must respect an ordering protocol (call `GetFile_Queue` before `Handle_Queue`, and only use `fp` if it is `Some`).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct tm, 2 free function(s); struct timespec; struct stat; struct file_queue, 2 free function(s); struct alert_data, 4 free function(s); struct timeval; struct fd_set

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
    pub mod lib {}

    // Helpers expected as `crate::src::c_lib::*`.
    pub mod c_lib {
        use std::io::{BufRead, Seek, SeekFrom};

        pub trait AsRawFd {
            fn as_raw_fd(&self) -> i32;
        }

        impl AsRawFd for std::io::BufReader<std::fs::File> {
            fn as_raw_fd(&self) -> i32 {
                use std::os::fd::AsRawFd as _;
                self.get_ref().as_raw_fd()
            }
        }

        pub unsafe fn rs_fseek<T: Seek>(fp: &mut T, offset: i64, whence: i32) -> i32 {
            let from = match whence {
                0 => SeekFrom::Start(offset as u64),
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => return -1,
            };
            fp.seek(from).map(|_| 0).unwrap_or(-1)
        }

        // C-like fgets: reads up to buf.len()-1 bytes, NUL-terminates, includes '\n' if present.
        // Returns buf.as_mut_ptr() on success, NULL on EOF/error.
        pub unsafe fn rs_fgets<T: BufRead>(
            buf: &mut [i8],
            fp: &mut T,
            _unused: Option<*mut core::ffi::c_void>,
            eof: Option<&mut i32>,
        ) -> *mut i8 {
            if buf.is_empty() {
                if let Some(e) = eof {
                    *e = 1;
                }
                return core::ptr::null_mut();
            }

            let max = buf.len().saturating_sub(1);
            let mut out_len = 0usize;

            while out_len < max {
                let available = match fp.fill_buf() {
                    Ok(b) => b,
                    Err(_) => {
                        if let Some(e) = eof {
                            *e = 1;
                        }
                        return core::ptr::null_mut();
                    }
                };

                if available.is_empty() {
                    if let Some(e) = eof {
                        *e = 1;
                    }
                    break;
                }

                let mut consumed = 0usize;
                while consumed < available.len() && out_len < max {
                    let ch = available[consumed];
                    buf[out_len] = ch as i8;
                    out_len += 1;
                    consumed += 1;
                    if ch == b'\n' {
                        break;
                    }
                }
                fp.consume(consumed);

                if out_len > 0 && buf[out_len - 1] == b'\n' as i8 {
                    break;
                }
            }

            if out_len == 0 {
                return core::ptr::null_mut();
            }

            buf[out_len] = 0;
            buf.as_mut_ptr()
        }

        pub unsafe fn rs_perror(msg: *const i8) {
            if msg.is_null() {
                eprintln!("(null)");
                return;
            }
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            eprintln!("{s}");
        }

        pub unsafe fn atoi(s: &[u8]) -> i32 {
            if s.is_empty() {
                return 0;
            }
            let mut i = 0usize;
            while i < s.len()
                && matches!(s[i], b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
            {
                i += 1;
            }
            let mut sign = 1i32;
            if i < s.len() && s[i] == b'-' {
                sign = -1;
                i += 1;
            } else if i < s.len() && s[i] == b'+' {
                i += 1;
            }
            let mut val: i32 = 0;
            while i < s.len() {
                let c = s[i];
                if !(b'0'..=b'9').contains(&c) {
                    break;
                }
                val = val.saturating_mul(10).saturating_add((c - b'0') as i32);
                i += 1;
            }
            val.saturating_mul(sign)
        }
    }

    // === driver.rs ===
    pub mod driver {
        pub type __dev_t = u64;
        pub type __uid_t = u32;
        pub type __gid_t = u32;
        pub type __ino_t = u64;
        pub type __mode_t = u32;
        pub type __nlink_t = u64;
        pub type __time_t = i64;
        pub type __blksize_t = i64;
        pub type __blkcnt_t = i64;
        pub type __syscall_slong_t = i64;
        pub type time_t = __time_t;

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct tm {
            pub tm_sec: i32,
            pub tm_min: i32,
            pub tm_hour: i32,
            pub tm_mday: i32,
            pub tm_mon: i32,
            pub tm_year: i32,
            pub tm_wday: i32,
            pub tm_yday: i32,
            pub tm_isdst: i32,
            pub tm_gmtoff: i64,
            pub tm_zone: *const i8,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct timespec {
            pub tv_sec: __time_t,
            pub tv_nsec: __syscall_slong_t,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct stat {
            pub st_dev: __dev_t,
            pub st_ino: __ino_t,
            pub st_nlink: __nlink_t,
            pub st_mode: __mode_t,
            pub st_uid: __uid_t,
            pub st_gid: __gid_t,
            pub __pad0: i32,
            pub st_rdev: __dev_t,
            pub st_size: i64,
            pub st_blksize: __blksize_t,
            pub st_blocks: __blkcnt_t,
            pub st_atim: timespec,
            pub st_mtim: timespec,
            pub st_ctim: timespec,
            pub __glibc_reserved: [__syscall_slong_t; 3],
        }

        #[repr(C)]
        pub struct file_queue {
            pub last_change: time_t,
            pub year: i32,
            pub day: i32,
            pub flags: i32,
            pub mon: [i8; 4],
            pub file_name: [i8; 257],
            pub fp: Option<std::io::BufReader<std::fs::File>>,
            pub f_status: stat,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct alert_data {
            pub rule: u32,
            pub level: u32,
            pub alertid: *mut i8,
            pub date: *mut i8,
            pub location: *mut i8,
            pub comment: *mut i8,
            pub group: *mut i8,
            pub srcip: *mut i8,
            pub srcport: i32,
            pub dstip: *mut i8,
            pub dstport: i32,
            pub user: *mut i8,
            pub filename: *mut i8,
        }
    }

    // === file_queue.rs ===
    pub mod file_queue {
        use crate::src::driver::__time_t;
        use crate::src::driver::alert_data;
        use crate::src::driver::file_queue;
        use crate::src::driver::stat;
        use crate::src::driver::tm;
        use crate::src::read_alert::GetAlertData;

        extern "C" {
            fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
            fn fstat(__fd: i32, __buf: *mut stat) -> i32;
            fn strerror(__errnum: i32) -> *mut i8;
            fn select(
                __nfds: i32,
                __readfds: *mut fd_set,
                __writefds: *mut fd_set,
                __exceptfds: *mut fd_set,
                __timeout: *mut timeval,
            ) -> i32;
        }

        // Fix for macOS: __errno_location is glibc-specific; use libc's __error on Apple.
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos"))]
        extern "C" {
            fn __error() -> *mut i32;
        }
        #[cfg(not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "tvos",
            target_os = "watchos"
        )))]
        extern "C" {
            fn __errno_location() -> *mut i32;
        }

        #[inline]
        unsafe fn errno_location() -> *mut i32 {
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos"))]
            {
                __error()
            }
            #[cfg(not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "watchos"
            )))]
            {
                __errno_location()
            }
        }

        pub type __suseconds_t = i64;

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct timeval {
            pub tv_sec: __time_t,
            pub tv_usec: __suseconds_t,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct fd_set {
            pub __fds_bits: [__fd_mask; 16],
        }

        pub type __fd_mask = i64;

        pub const SEEK_END: i32 = 2;
        pub const MAX_FQUEUE: i32 = 256;
        pub const FQ_TIMEOUT: i32 = 5;

        pub const ALERTS_DAILY: [i8; 11] = [
            b'a' as i8,
            b'l' as i8,
            b'e' as i8,
            b'r' as i8,
            b't' as i8,
            b's' as i8,
            b'.' as i8,
            b'l' as i8,
            b'o' as i8,
            b'g' as i8,
            0,
        ];

        pub const CRALERT_READ_ALL: i32 = 0x4;
        pub const CRALERT_FP_SET: i32 = 0x10;

        pub const FSTAT_ERROR: [i8; 72] = [
            b'(' as i8,
            b'1' as i8,
            b'1' as i8,
            b'1' as i8,
            b'8' as i8,
            b')' as i8,
            b':' as i8,
            b' ' as i8,
            b'C' as i8,
            b'o' as i8,
            b'u' as i8,
            b'l' as i8,
            b'd' as i8,
            b' ' as i8,
            b'n' as i8,
            b'o' as i8,
            b't' as i8,
            b' ' as i8,
            b'r' as i8,
            b'e' as i8,
            b't' as i8,
            b'r' as i8,
            b'i' as i8,
            b'e' as i8,
            b'v' as i8,
            b'e' as i8,
            b' ' as i8,
            b'i' as i8,
            b'n' as i8,
            b'f' as i8,
            b'o' as i8,
            b'r' as i8,
            b'm' as i8,
            b'a' as i8,
            b't' as i8,
            b'i' as i8,
            b'o' as i8,
            b'n' as i8,
            b' ' as i8,
            b'o' as i8,
            b'f' as i8,
            b' ' as i8,
            b'f' as i8,
            b'i' as i8,
            b'l' as i8,
            b'e' as i8,
            b' ' as i8,
            b'\'' as i8,
            b'%' as i8,
            b's' as i8,
            b'\'' as i8,
            b' ' as i8,
            b'd' as i8,
            b'u' as i8,
            b'e' as i8,
            b' ' as i8,
            b't' as i8,
            b'o' as i8,
            b' ' as i8,
            b'[' as i8,
            b'(' as i8,
            b'%' as i8,
            b'd' as i8,
            b')' as i8,
            b'-' as i8,
            b'(' as i8,
            b'%' as i8,
            b's' as i8,
            b')' as i8,
            b']' as i8,
            b'.' as i8,
            0,
        ];

        pub const FSEEK_ERROR: [i8; 64] = [
            b'(' as i8,
            b'1' as i8,
            b'1' as i8,
            b'1' as i8,
            b'6' as i8,
            b')' as i8,
            b':' as i8,
            b' ' as i8,
            b'C' as i8,
            b'o' as i8,
            b'u' as i8,
            b'l' as i8,
            b'd' as i8,
            b' ' as i8,
            b'n' as i8,
            b'o' as i8,
            b't' as i8,
            b' ' as i8,
            b's' as i8,
            b'e' as i8,
            b't' as i8,
            b' ' as i8,
            b'p' as i8,
            b'o' as i8,
            b's' as i8,
            b'i' as i8,
            b't' as i8,
            b'i' as i8,
            b'o' as i8,
            b'n' as i8,
            b' ' as i8,
            b'i' as i8,
            b'n' as i8,
            b' ' as i8,
            b'f' as i8,
            b'i' as i8,
            b'l' as i8,
            b'e' as i8,
            b' ' as i8,
            b'\'' as i8,
            b'%' as i8,
            b's' as i8,
            b'\'' as i8,
            b' ' as i8,
            b'd' as i8,
            b'u' as i8,
            b'e' as i8,
            b' ' as i8,
            b't' as i8,
            b'o' as i8,
            b' ' as i8,
            b'[' as i8,
            b'(' as i8,
            b'%' as i8,
            b'd' as i8,
            b')' as i8,
            b'-' as i8,
            b'(' as i8,
            b'%' as i8,
            b's' as i8,
            b')' as i8,
            b']' as i8,
            b'.' as i8,
            0,
        ];

        pub(crate) unsafe fn merror(
            err_template: Option<&i8>,
            file_name: Option<&i8>,
            err: i32,
            err_msg: Option<&i8>,
        ) {
            let mut buffer: [i8; 256] = [0; 256];
            snprintf(
                buffer.as_mut_ptr(),
                core::mem::size_of_val(&buffer),
                err_template.map_or(core::ptr::null(), |p| p),
                file_name.map_or(core::ptr::null(), |p| p),
                err,
                err_msg.map_or(core::ptr::null(), |p| p),
            );
            eprintln!("{}", std::ffi::CStr::from_ptr(buffer.as_ptr()).to_string_lossy());
        }

        static s_month: [&[i8]; 12] = [
            &[b'J' as i8, b'a' as i8, b'n' as i8, 0],
            &[b'F' as i8, b'e' as i8, b'b' as i8, 0],
            &[b'M' as i8, b'a' as i8, b'r' as i8, 0],
            &[b'A' as i8, b'p' as i8, b'r' as i8, 0],
            &[b'M' as i8, b'a' as i8, b'y' as i8, 0],
            &[b'J' as i8, b'u' as i8, b'n' as i8, 0],
            &[b'J' as i8, b'u' as i8, b'l' as i8, 0],
            &[b'A' as i8, b'u' as i8, b'g' as i8, 0],
            &[b'S' as i8, b'e' as i8, b'p' as i8, 0],
            &[b'O' as i8, b'c' as i8, b't' as i8, 0],
            &[b'N' as i8, b'o' as i8, b'v' as i8, 0],
            &[b'D' as i8, b'e' as i8, b'c' as i8, 0],
        ];

        unsafe fn file_sleep() {
            let mut fp_timeout = timeval {
                tv_sec: FQ_TIMEOUT as i64,
                tv_usec: 0,
            };
            select(
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut fp_timeout,
            );
        }

        unsafe fn GetFile_Queue(fileq: Option<&mut file_queue>) {
            let fileq = fileq.unwrap();
            fileq.file_name[0] = 0;
            fileq.file_name[MAX_FQUEUE as usize] = 0;

            let name = if fileq.flags & CRALERT_FP_SET != 0 {
                b"<stdin>\0" as *const u8 as *const i8
            } else {
                ALERTS_DAILY.as_ptr()
            };

            snprintf(
                fileq.file_name.as_mut_ptr(),
                MAX_FQUEUE as usize,
                b"%s\0" as *const u8 as *const i8,
                name,
            );
        }

        unsafe fn Handle_Queue(fileq: Option<&mut file_queue>, flags: i32) -> i32 {
            let fileq = fileq.unwrap();

            if flags & CRALERT_FP_SET == 0 {
                fileq.fp = None;

                let path = std::ffi::CStr::from_ptr(fileq.file_name.as_ptr())
                    .to_string_lossy()
                    .into_owned();

                fileq.fp = std::fs::File::open(path).ok().map(std::io::BufReader::new);
                if fileq.fp.is_none() {
                    return 0;
                }
            }

            if flags & CRALERT_READ_ALL == 0 {
                let Some(fp) = fileq.fp.as_mut() else {
                    return 0;
                };
                if crate::src::c_lib::rs_fseek(fp, 0, SEEK_END) < 0 {
                    let errno = *errno_location();
                    merror(
                        Some(&FSEEK_ERROR[0]),
                        Some(&fileq.file_name[0]),
                        errno,
                        strerro
// ... (truncated) ...
```

**Entity:** crate::src::driver::file_queue

**States:** Uninitialized, Named, FileHandleReady, EOFOrError

**Transitions:**
- Uninitialized -> Named via GetFile_Queue(fileq)
- Named -> FileHandleReady via Handle_Queue(fileq, flags) when (flags & CRALERT_FP_SET)==0 and File::open succeeds
- Named -> FileHandleReady via Handle_Queue(fileq, flags) when (flags & CRALERT_FP_SET)!=0 (implies stdin-mode; file is not opened here)
- Named/FileHandleReady -> EOFOrError via Handle_Queue returning 0 (e.g., open failure, fp missing, seek failure)

**Evidence:** struct file_queue fields: `flags: i32`, `file_name: [i8; 257]`, `fp: Option<std::io::BufReader<std::fs::File>>` encode runtime state; GetFile_Queue(): `fileq.file_name[0] = 0; fileq.file_name[MAX_FQUEUE as usize] = 0;` then `snprintf(fileq.file_name...)` initializes name buffer; GetFile_Queue(): chooses name based on `fileq.flags & CRALERT_FP_SET != 0` (stdin vs `ALERTS_DAILY`), implying a mode-dependent protocol; Handle_Queue(): `if flags & CRALERT_FP_SET == 0 { fileq.fp = None; ... fileq.fp = File::open(...).ok().map(BufReader::new); if fileq.fp.is_none() { return 0; } }` open/closed state is handled at runtime; Handle_Queue(): `let Some(fp) = fileq.fp.as_mut() else { return 0; };` gates subsequent operations on runtime presence of `fp`; Handle_Queue(): `rs_fseek(fp, 0, SEEK_END) < 0` error path uses errno/merror; indicates an operation only valid in the FileHandleReady state

**Implementation:** Introduce `FileQueue<S>` with marker states like `Uninit`, `Named`, `Ready` (and optionally `StdinReady`). Make `get_file_queue(self/ &mut) -> FileQueue<Named>` set the name, then `open(self, flags) -> Result<FileQueue<Ready>, OpenError>`. Only implement read/seek methods for `FileQueue<Ready>`. Represent stdin vs file-open as separate state types or via an enum held in the Ready state instead of `flags`/Option checks.

---

## Precondition Invariants

### 4. FFI string/ownership validity protocol for alert_data pointer fields

**Location**: `/data/test_case/lib.rs:1-20`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: alert_data contains multiple raw `*mut i8` fields that are implicitly treated as pointers to C strings (and/or writable buffers) with a required lifetime. The code relies on callers to ensure each pointer is either null or points to valid memory for the duration of use, and (if treated as a C string) that it is NUL-terminated. Because they are raw mutable pointers inside a `Copy, Clone` struct, the type system cannot prevent: using dangling pointers, double-free scenarios (if any field is actually owned), aliasing mutable pointers across copies, or passing non-NUL-terminated/non-UTF8 data to string consumers. These are latent invariants about pointer validity, mutability, and ownership that are not enforced at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct tm, 2 free function(s); struct timespec; struct stat; struct file_queue, 2 free function(s); struct timeval; struct fd_set; trait AsRawFd, 11 free function(s), impl AsRawFd for std :: io :: BufReader < std :: fs :: File > (1 methods)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct alert_data {
            pub rule: u32,
            pub level: u32,
            pub alertid: *mut i8,
            pub date: *mut i8,
            pub location: *mut i8,
            pub comment: *mut i8,
            pub group: *mut i8,
            pub srcip: *mut i8,
            pub srcport: i32,
            pub dstip: *mut i8,
            pub dstport: i32,
            pub user: *mut i8,
            pub filename: *mut i8,
        }

```

**Entity:** alert_data

**States:** Invalid/Uninitialized pointers, Valid C strings (NUL-terminated) + correct lifetime/ownership, Freed/Dangling pointers

**Transitions:**
- Invalid/Uninitialized pointers -> Valid C strings (NUL-terminated) + correct lifetime/ownership via FFI initialization / manual assignment to fields
- Valid C strings (NUL-terminated) + correct lifetime/ownership -> Freed/Dangling pointers via freeing underlying allocations while copies of alert_data still exist (enabled by Copy/Clone)

**Evidence:** `#[derive(Copy, Clone)]` on `alert_data` enables bitwise copying of raw pointers without tracking ownership/lifetimes; fields: `alertid: *mut i8`, `date: *mut i8`, `location: *mut i8`, `comment: *mut i8`, `group: *mut i8`, `srcip: *mut i8`, `dstip: *mut i8`, `user: *mut i8`, `filename: *mut i8` are raw pointers with no lifetime/validity encoding; `#[repr(C)]` indicates FFI layout, implying these pointers participate in an external (C) protocol not represented in Rust types

**Implementation:** Replace `*mut i8` fields with wrappers expressing intent and safety, e.g. `Option<NonNull<c_char>>` for nullable pointers; or `*const c_char` if they are read-only. For owned data, use `CString` (or `Box<CStr>`/`Vec<c_char>`) and make the Rust-side struct non-`Copy`. If the data must borrow external memory, introduce a lifetime: `struct AlertData<'a> { alertid: Option<&'a CStr>, ... }` and provide an explicit conversion to/from the `#[repr(C)]` FFI struct.

---

## Protocol Invariants

### 3. C fgets buffer protocol (NonEmpty buffer -> NUL-terminated output / NULL on EOF)

**Location**: `/data/test_case/lib.rs:1-556`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `rs_fgets` assumes a C-style output contract: the caller must supply a non-empty `buf` so the function can write at most `len-1` bytes and then NUL-terminate at `buf[out_len]`. It returns a raw pointer that is either NULL (EOF/error/empty read) or equals `buf.as_mut_ptr()` on success, and it may set an `eof` out-parameter. These requirements are checked at runtime (`buf.is_empty()`, `out_len==0`) and are not expressible in the signature, so callers can still pass an empty slice and must remember to interpret NULL/eof correctly.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct tm, 2 free function(s); struct timespec; struct stat; struct file_queue, 2 free function(s); struct alert_data, 4 free function(s); struct timeval; struct fd_set

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
    pub mod lib {}

    // Helpers expected as `crate::src::c_lib::*`.
    pub mod c_lib {
        use std::io::{BufRead, Seek, SeekFrom};

        pub trait AsRawFd {
            fn as_raw_fd(&self) -> i32;
        }

        impl AsRawFd for std::io::BufReader<std::fs::File> {
            fn as_raw_fd(&self) -> i32 {
                use std::os::fd::AsRawFd as _;
                self.get_ref().as_raw_fd()
            }
        }

        pub unsafe fn rs_fseek<T: Seek>(fp: &mut T, offset: i64, whence: i32) -> i32 {
            let from = match whence {
                0 => SeekFrom::Start(offset as u64),
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => return -1,
            };
            fp.seek(from).map(|_| 0).unwrap_or(-1)
        }

        // C-like fgets: reads up to buf.len()-1 bytes, NUL-terminates, includes '\n' if present.
        // Returns buf.as_mut_ptr() on success, NULL on EOF/error.
        pub unsafe fn rs_fgets<T: BufRead>(
            buf: &mut [i8],
            fp: &mut T,
            _unused: Option<*mut core::ffi::c_void>,
            eof: Option<&mut i32>,
        ) -> *mut i8 {
            if buf.is_empty() {
                if let Some(e) = eof {
                    *e = 1;
                }
                return core::ptr::null_mut();
            }

            let max = buf.len().saturating_sub(1);
            let mut out_len = 0usize;

            while out_len < max {
                let available = match fp.fill_buf() {
                    Ok(b) => b,
                    Err(_) => {
                        if let Some(e) = eof {
                            *e = 1;
                        }
                        return core::ptr::null_mut();
                    }
                };

                if available.is_empty() {
                    if let Some(e) = eof {
                        *e = 1;
                    }
                    break;
                }

                let mut consumed = 0usize;
                while consumed < available.len() && out_len < max {
                    let ch = available[consumed];
                    buf[out_len] = ch as i8;
                    out_len += 1;
                    consumed += 1;
                    if ch == b'\n' {
                        break;
                    }
                }
                fp.consume(consumed);

                if out_len > 0 && buf[out_len - 1] == b'\n' as i8 {
                    break;
                }
            }

            if out_len == 0 {
                return core::ptr::null_mut();
            }

            buf[out_len] = 0;
            buf.as_mut_ptr()
        }

        pub unsafe fn rs_perror(msg: *const i8) {
            if msg.is_null() {
                eprintln!("(null)");
                return;
            }
            let s = std::ffi::CStr::from_ptr(msg).to_string_lossy();
            eprintln!("{s}");
        }

        pub unsafe fn atoi(s: &[u8]) -> i32 {
            if s.is_empty() {
                return 0;
            }
            let mut i = 0usize;
            while i < s.len()
                && matches!(s[i], b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
            {
                i += 1;
            }
            let mut sign = 1i32;
            if i < s.len() && s[i] == b'-' {
                sign = -1;
                i += 1;
            } else if i < s.len() && s[i] == b'+' {
                i += 1;
            }
            let mut val: i32 = 0;
            while i < s.len() {
                let c = s[i];
                if !(b'0'..=b'9').contains(&c) {
                    break;
                }
                val = val.saturating_mul(10).saturating_add((c - b'0') as i32);
                i += 1;
            }
            val.saturating_mul(sign)
        }
    }

    // === driver.rs ===
    pub mod driver {
        pub type __dev_t = u64;
        pub type __uid_t = u32;
        pub type __gid_t = u32;
        pub type __ino_t = u64;
        pub type __mode_t = u32;
        pub type __nlink_t = u64;
        pub type __time_t = i64;
        pub type __blksize_t = i64;
        pub type __blkcnt_t = i64;
        pub type __syscall_slong_t = i64;
        pub type time_t = __time_t;

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct tm {
            pub tm_sec: i32,
            pub tm_min: i32,
            pub tm_hour: i32,
            pub tm_mday: i32,
            pub tm_mon: i32,
            pub tm_year: i32,
            pub tm_wday: i32,
            pub tm_yday: i32,
            pub tm_isdst: i32,
            pub tm_gmtoff: i64,
            pub tm_zone: *const i8,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct timespec {
            pub tv_sec: __time_t,
            pub tv_nsec: __syscall_slong_t,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct stat {
            pub st_dev: __dev_t,
            pub st_ino: __ino_t,
            pub st_nlink: __nlink_t,
            pub st_mode: __mode_t,
            pub st_uid: __uid_t,
            pub st_gid: __gid_t,
            pub __pad0: i32,
            pub st_rdev: __dev_t,
            pub st_size: i64,
            pub st_blksize: __blksize_t,
            pub st_blocks: __blkcnt_t,
            pub st_atim: timespec,
            pub st_mtim: timespec,
            pub st_ctim: timespec,
            pub __glibc_reserved: [__syscall_slong_t; 3],
        }

        #[repr(C)]
        pub struct file_queue {
            pub last_change: time_t,
            pub year: i32,
            pub day: i32,
            pub flags: i32,
            pub mon: [i8; 4],
            pub file_name: [i8; 257],
            pub fp: Option<std::io::BufReader<std::fs::File>>,
            pub f_status: stat,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct alert_data {
            pub rule: u32,
            pub level: u32,
            pub alertid: *mut i8,
            pub date: *mut i8,
            pub location: *mut i8,
            pub comment: *mut i8,
            pub group: *mut i8,
            pub srcip: *mut i8,
            pub srcport: i32,
            pub dstip: *mut i8,
            pub dstport: i32,
            pub user: *mut i8,
            pub filename: *mut i8,
        }
    }

    // === file_queue.rs ===
    pub mod file_queue {
        use crate::src::driver::__time_t;
        use crate::src::driver::alert_data;
        use crate::src::driver::file_queue;
        use crate::src::driver::stat;
        use crate::src::driver::tm;
        use crate::src::read_alert::GetAlertData;

        extern "C" {
            fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
            fn fstat(__fd: i32, __buf: *mut stat) -> i32;
            fn strerror(__errnum: i32) -> *mut i8;
            fn select(
                __nfds: i32,
                __readfds: *mut fd_set,
                __writefds: *mut fd_set,
                __exceptfds: *mut fd_set,
                __timeout: *mut timeval,
            ) -> i32;
        }

        // Fix for macOS: __errno_location is glibc-specific; use libc's __error on Apple.
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos"))]
        extern "C" {
            fn __error() -> *mut i32;
        }
        #[cfg(not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "tvos",
            target_os = "watchos"
        )))]
        extern "C" {
            fn __errno_location() -> *mut i32;
        }

        #[inline]
        unsafe fn errno_location() -> *mut i32 {
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos"))]
            {
                __error()
            }
            #[cfg(not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "watchos"
            )))]
            {
                __errno_location()
            }
        }

        pub type __suseconds_t = i64;

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct timeval {
            pub tv_sec: __time_t,
            pub tv_usec: __suseconds_t,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct fd_set {
            pub __fds_bits: [__fd_mask; 16],
        }

        pub type __fd_mask = i64;

        pub const SEEK_END: i32 = 2;
        pub const MAX_FQUEUE: i32 = 256;
        pub const FQ_TIMEOUT: i32 = 5;

        pub const ALERTS_DAILY: [i8; 11] = [
            b'a' as i8,
            b'l' as i8,
            b'e' as i8,
            b'r' as i8,
            b't' as i8,
            b's' as i8,
            b'.' as i8,
            b'l' as i8,
            b'o' as i8,
            b'g' as i8,
            0,
        ];

        pub const CRALERT_READ_ALL: i32 = 0x4;
        pub const CRALERT_FP_SET: i32 = 0x10;

        pub const FSTAT_ERROR: [i8; 72] = [
            b'(' as i8,
            b'1' as i8,
            b'1' as i8,
            b'1' as i8,
            b'8' as i8,
            b')' as i8,
            b':' as i8,
            b' ' as i8,
            b'C' as i8,
            b'o' as i8,
            b'u' as i8,
            b'l' as i8,
            b'd' as i8,
            b' ' as i8,
            b'n' as i8,
            b'o' as i8,
            b't' as i8,
            b' ' as i8,
            b'r' as i8,
            b'e' as i8,
            b't' as i8,
            b'r' as i8,
            b'i' as i8,
            b'e' as i8,
            b'v' as i8,
            b'e' as i8,
            b' ' as i8,
            b'i' as i8,
            b'n' as i8,
            b'f' as i8,
            b'o' as i8,
            b'r' as i8,
            b'm' as i8,
            b'a' as i8,
            b't' as i8,
            b'i' as i8,
            b'o' as i8,
            b'n' as i8,
            b' ' as i8,
            b'o' as i8,
            b'f' as i8,
            b' ' as i8,
            b'f' as i8,
            b'i' as i8,
            b'l' as i8,
            b'e' as i8,
            b' ' as i8,
            b'\'' as i8,
            b'%' as i8,
            b's' as i8,
            b'\'' as i8,
            b' ' as i8,
            b'd' as i8,
            b'u' as i8,
            b'e' as i8,
            b' ' as i8,
            b't' as i8,
            b'o' as i8,
            b' ' as i8,
            b'[' as i8,
            b'(' as i8,
            b'%' as i8,
            b'd' as i8,
            b')' as i8,
            b'-' as i8,
            b'(' as i8,
            b'%' as i8,
            b's' as i8,
            b')' as i8,
            b']' as i8,
            b'.' as i8,
            0,
        ];

        pub const FSEEK_ERROR: [i8; 64] = [
            b'(' as i8,
            b'1' as i8,
            b'1' as i8,
            b'1' as i8,
            b'6' as i8,
            b')' as i8,
            b':' as i8,
            b' ' as i8,
            b'C' as i8,
            b'o' as i8,
            b'u' as i8,
            b'l' as i8,
            b'd' as i8,
            b' ' as i8,
            b'n' as i8,
            b'o' as i8,
            b't' as i8,
            b' ' as i8,
            b's' as i8,
            b'e' as i8,
            b't' as i8,
            b' ' as i8,
            b'p' as i8,
            b'o' as i8,
            b's' as i8,
            b'i' as i8,
            b't' as i8,
            b'i' as i8,
            b'o' as i8,
            b'n' as i8,
            b' ' as i8,
            b'i' as i8,
            b'n' as i8,
            b' ' as i8,
            b'f' as i8,
            b'i' as i8,
            b'l' as i8,
            b'e' as i8,
            b' ' as i8,
            b'\'' as i8,
            b'%' as i8,
            b's' as i8,
            b'\'' as i8,
            b' ' as i8,
            b'd' as i8,
            b'u' as i8,
            b'e' as i8,
            b' ' as i8,
            b't' as i8,
            b'o' as i8,
            b' ' as i8,
            b'[' as i8,
            b'(' as i8,
            b'%' as i8,
            b'd' as i8,
            b')' as i8,
            b'-' as i8,
            b'(' as i8,
            b'%' as i8,
            b's' as i8,
            b')' as i8,
            b']' as i8,
            b'.' as i8,
            0,
        ];

        pub(crate) unsafe fn merror(
            err_template: Option<&i8>,
            file_name: Option<&i8>,
            err: i32,
            err_msg: Option<&i8>,
        ) {
            let mut buffer: [i8; 256] = [0; 256];
            snprintf(
                buffer.as_mut_ptr(),
                core::mem::size_of_val(&buffer),
                err_template.map_or(core::ptr::null(), |p| p),
                file_name.map_or(core::ptr::null(), |p| p),
                err,
                err_msg.map_or(core::ptr::null(), |p| p),
            );
            eprintln!("{}", std::ffi::CStr::from_ptr(buffer.as_ptr()).to_string_lossy());
        }

        static s_month: [&[i8]; 12] = [
            &[b'J' as i8, b'a' as i8, b'n' as i8, 0],
            &[b'F' as i8, b'e' as i8, b'b' as i8, 0],
            &[b'M' as i8, b'a' as i8, b'r' as i8, 0],
            &[b'A' as i8, b'p' as i8, b'r' as i8, 0],
            &[b'M' as i8, b'a' as i8, b'y' as i8, 0],
            &[b'J' as i8, b'u' as i8, b'n' as i8, 0],
            &[b'J' as i8, b'u' as i8, b'l' as i8, 0],
            &[b'A' as i8, b'u' as i8, b'g' as i8, 0],
            &[b'S' as i8, b'e' as i8, b'p' as i8, 0],
            &[b'O' as i8, b'c' as i8, b't' as i8, 0],
            &[b'N' as i8, b'o' as i8, b'v' as i8, 0],
            &[b'D' as i8, b'e' as i8, b'c' as i8, 0],
        ];

        unsafe fn file_sleep() {
            let mut fp_timeout = timeval {
                tv_sec: FQ_TIMEOUT as i64,
                tv_usec: 0,
            };
            select(
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &raw mut fp_timeout,
            );
        }

        unsafe fn GetFile_Queue(fileq: Option<&mut file_queue>) {
            let fileq = fileq.unwrap();
            fileq.file_name[0] = 0;
            fileq.file_name[MAX_FQUEUE as usize] = 0;

            let name = if fileq.flags & CRALERT_FP_SET != 0 {
                b"<stdin>\0" as *const u8 as *const i8
            } else {
                ALERTS_DAILY.as_ptr()
            };

            snprintf(
                fileq.file_name.as_mut_ptr(),
                MAX_FQUEUE as usize,
                b"%s\0" as *const u8 as *const i8,
                name,
            );
        }

        unsafe fn Handle_Queue(fileq: Option<&mut file_queue>, flags: i32) -> i32 {
            let fileq = fileq.unwrap();

            if flags & CRALERT_FP_SET == 0 {
                fileq.fp = None;

                let path = std::ffi::CStr::from_ptr(fileq.file_name.as_ptr())
                    .to_string_lossy()
                    .into_owned();

                fileq.fp = std::fs::File::open(path).ok().map(std::io::BufReader::new);
                if fileq.fp.is_none() {
                    return 0;
                }
            }

            if flags & CRALERT_READ_ALL == 0 {
                let Some(fp) = fileq.fp.as_mut() else {
                    return 0;
                };
                if crate::src::c_lib::rs_fseek(fp, 0, SEEK_END) < 0 {
                    let errno = *errno_location();
                    merror(
                        Some(&FSEEK_ERROR[0]),
                        Some(&fileq.file_name[0]),
                        errno,
                        strerro
// ... (truncated) ...
```

**Entity:** crate::src::c_lib::rs_fgets

**States:** InvalidBuffer, Ready, ReturnedNull(EofOrError), ReturnedPtr(NulTerminated)

**Transitions:**
- Ready -> ReturnedPtr(NulTerminated) when out_len > 0 (writes `buf[out_len]=0` and returns `buf.as_mut_ptr()`)
- Ready -> ReturnedNull(EofOrError) when fill_buf errors/EOF and no bytes were read (`out_len==0`), or when `buf.is_empty()`

**Evidence:** rs_fgets signature: `pub unsafe fn rs_fgets(buf: &mut [i8], fp: &mut T, ..., eof: Option<&mut i32>) -> *mut i8` returns raw pointer with C-like NULL protocol; rs_fgets(): `if buf.is_empty() { ... return core::ptr::null_mut(); }` encodes a precondition at runtime; rs_fgets(): `let max = buf.len().saturating_sub(1);` and later `buf[out_len] = 0;` requires at least 1 byte for NUL terminator; rs_fgets(): `if out_len == 0 { return core::ptr::null_mut(); }` defines NULL return meaning 'no data read'; comment above rs_fgets: `C-like fgets ... NUL-terminates ... Returns ... NULL on EOF/error.` documents the implicit protocol

**Implementation:** Accept `buf: &mut NonEmptyI8Slice` (a newtype that can only be constructed from a non-empty slice) or `&mut [core::ffi::c_char; N]` via const generics `N: usize` with `N>0` ensured by construction. Return `Option<NonNull<i8>>` (or better, `Result<&CStr, ReadError>`) to encode the NULL/non-NULL contract without raw pointers, and model EOF as `Ok(None)`.

---

