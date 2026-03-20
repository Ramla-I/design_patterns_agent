# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 1
- **Precondition**: 2
- **Protocol**: 0
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 2. Logger initialization lifecycle (Uninitialized / Initialized / Finalized)

**Location**: `/data/test_case/lib.rs:1-365`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Logging functions silently depend on a thread-local global file writer being initialized. Before initialize_logger(), log_*_internal() no-ops; after finalize_logger(), the writer is taken (set back to None) and further logging again no-ops. This protocol (initialize before any log_* calls; finalize at the end; avoid double-init/finalize ordering issues) is enforced only via runtime Option checks and global state, not by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Task, 5 free function(s); struct TaskManager

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
    // Keep module structure expected by the harness: src::driver, src::logger, src::task_manager
    pub mod driver {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct Task {
            pub description: [i8; 256],
            pub priority: i32,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct TaskManager {
            pub tasks: *mut Task,
            pub max_tasks: i32,
            pub task_count: i32,
        }
    }

    pub mod logger {
        extern "C" {
            fn getenv(__name: *const i8) -> *mut i8;
        }

        thread_local! {
            static log_file:
            std::cell::RefCell<Option<std::io::BufWriter<std::fs::File>>> = const
            { std::cell::RefCell::new(None) };
        }

        #[inline]
        fn cstr_from_i8_ptr(ptr: *const i8) -> Option<&'static std::ffi::CStr> {
            if ptr.is_null() {
                None
            } else {
                // SAFETY: caller promises ptr is a valid NUL-terminated C string.
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) })
            }
        }

        #[inline]
        fn cstr_from_i8_slice_until_nul(bytes: &[i8]) -> Option<&std::ffi::CStr> {
            if bytes.is_empty() {
                return None;
            }
            // SAFETY: i8/u8 have identical layout.
            let u8s = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
            std::ffi::CStr::from_bytes_until_nul(u8s).ok()
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_logger() -> i32 {
            let env_ptr = getenv(b"LOG_FILE\0" as *const u8 as *const i8);

            let path_cstr = if let Some(c) = cstr_from_i8_ptr(env_ptr) {
                c
            } else {
                std::ffi::CStr::from_bytes_with_nul(b"default.log\0").unwrap()
            };

            let path_str = match path_cstr.to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Failed to open log file: invalid UTF-8 path");
                    return -1;
                }
            };

            log_file.with_borrow_mut(|log_file_ref| {
                *log_file_ref = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path_str)
                    .ok()
                    .map(std::io::BufWriter::new);
            });

            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_none()) {
                eprintln!("Failed to open log file: {0}", path_str);
                return -1;
            }

            log_info_internal(bytemuck::cast_slice(b"Logger initialized.\0"));
            0
        }

        pub(crate) fn log_info_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[INFO] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_info(message: *const i8) {
            log_info_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_warning_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[WARNING] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_warning(message: *const i8) {
            log_warning_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_error_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[ERROR] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_error(message: *const i8) {
            log_error_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        #[no_mangle]
        pub extern "C" fn finalize_logger() {
            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                log_info_internal(bytemuck::cast_slice(b"Logger finalized.\0"));
                {
                    let mut __x = log_file.with_borrow_mut(|log_file_ref| log_file_ref.take().unwrap());
                    let __v = std::io::Write::flush(&mut __x).map_or(-1, |_| 0);
                    drop(__x);
                    __v
                };
            }
        }
    }

    pub mod task_manager {
        use crate::src::driver::{Task, TaskManager};
        use crate::src::logger::{log_error_internal, log_info_internal, log_warning_internal};

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
            fn getenv(__name: *const i8) -> *mut i8;
        }

        const TASK_DESC_LEN: usize = 256;

        #[inline]
        unsafe fn max_tasks_from_env_or_default() -> i32 {
            let env_ptr = getenv(b"MAX_TASKS\0" as *const u8 as *const i8);
            if env_ptr.is_null() {
                return 10;
            }
            // SAFETY: getenv returns a NUL-terminated C string.
            let cstr = std::ffi::CStr::from_ptr(env_ptr);
            let s = cstr.to_str().ok().unwrap_or("");
            s.trim().parse::<i32>().ok().unwrap_or(10)
        }

        #[no_mangle]
        pub unsafe extern "C" fn create_task_manager() -> *mut TaskManager {
            let manager_ptr = malloc(core::mem::size_of::<TaskManager>()) as *mut TaskManager;
            let Some(manager) = manager_ptr.as_mut() else {
                log_error_internal(bytemuck::cast_slice(
                    b"Failed to allocate memory for TaskManager.\0",
                ));
                return core::ptr::null_mut();
            };

            manager.max_tasks = max_tasks_from_env_or_default();
            manager.task_count = 0;

            let tasks_bytes = (manager.max_tasks as usize).wrapping_mul(core::mem::size_of::<Task>());
            manager.tasks = malloc(tasks_bytes) as *mut Task;

            if manager.tasks.is_null() {
                log_error_internal(bytemuck::cast_slice(b"Failed to allocate memory for tasks.\0"));
                free(manager_ptr as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }

            log_info_internal(bytemuck::cast_slice(b"TaskManager created successfully.\0"));
            manager_ptr
        }

        pub(crate) unsafe fn add_task_internal(
            mut manager: Option<&mut TaskManager>,
            description: &[i8],
            priority: i32,
        ) {
            let mgr = manager.as_deref_mut().unwrap();

            if mgr.task_count >= mgr.max_tasks {
                log_warning_internal(bytemuck::cast_slice(
                    b"Cannot add task: Maximum task limit reached.\0",
                ));
                return;
            }

            let idx = mgr.task_count as usize;
            mgr.task_count += 1;

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts_mut(mgr.tasks, mgr.max_tasks as usize);
            let task = &mut tasks[idx];

            // Match original semantics: always copy exactly 255 bytes from `description` slice
            // (caller provides 1024 bytes), then set last byte to NUL.
            let copy_len = TASK_DESC_LEN - 1;
            task.description[..copy_len].copy_from_slice(&description[..copy_len]);
            task.description[copy_len] = 0;
            task.priority = priority;

            log_info_internal(bytemuck::cast_slice(b"Task added successfully.\0"));
        }

        #[no_mangle]
        pub unsafe extern "C" fn add_task(
            manager: Option<&mut TaskManager>,
            description: *const i8,
            priority: i32,
        ) {
            add_task_internal(
                manager,
                if description.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(description, 1024)
                },
                priority,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn print_tasks(manager: Option<&mut TaskManager>) {
            println!("Tasks:");
            let mgr = manager.as_deref().unwrap();

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts(mgr.tasks, mgr.max_tasks as usize);

            for i in 0..(mgr.task_count as usize) {
                let task = &tasks[i];
                let desc_u8 = std::slice::from_raw_parts(
                    task.description.as_ptr() as *const u8,
                    task.description.len(),
                );
                let desc = std::ffi::CStr::from_bytes_until_nul(desc_u8)
                    .ok()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                println!("  [{0}] {1} (Priority: {2})", i + 1, desc, task.priority);
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn destroy_task_manager(manager: Option<&TaskManager>) {
            let mgr = manager.unwrap();
            free(mgr.tasks as *mut core::ffi::c_void);
            free((mgr as *const TaskManager as *mut TaskManager) as *mut core::ffi::c_void);
            log_info_internal(bytemuck::cast_slice(
                b"TaskManager destroyed successfully.\0",
            ));
        }
    }
}

// Added by Claude Code after initial translation: driver entry point expected by cando2 test harness
#[no_mangle]
pub unsafe extern "C" fn driver(tasks: *const std::ffi::c_char) -> std::ffi::c_int {
    if tasks.is_null() {
        return 0;
    }

    if src::logger::initialize_logger() != 0 {
        return 1; // EXIT_FAILURE
    }

    let manager = src::task_manager::create_task_manager();
    if manager.is_null() {
        return 1;
    }

    let mut start = tasks;
    let mut priority: std::ffi::c_int = 1;

    while *start != 0 {
        let end = libc::strchr(start, '\n' as std::ffi::c_int);
        let end = if end.is_null() {
            start.add(libc::strlen(start))
        } else {
            end
        };

        let length = (end as usize) - (start as usize);
        let task = libc::malloc(length + 1) as *mut std::ffi::c_char;
        if task.is_null() {
            eprintln!("Error: Failed to allocate memory for task.");
            src::task_manager::destroy_task_manager(manager.as_ref());
            src::logger::finalize_logger();
            return 1;
        }
        libc::strncpy(task, start, length);
        *task.add(length) = 0;

        src::task_manager::add_task(manager.as_mut(), task, priority);
        priority += 1;
        libc::free(task as *mut std::ffi::c_void);

        start = if *end == '\n' as i8 { end.add(1) } else { end };
    }

    src::task_manager::print_tasks(manager.as_mut());
    src::task_manager::destroy_task_manager(manager.as_ref());
    src::logger::finalize_logger();

    0
}
```

**Entity:** src::logger (thread_local! static log_file)

**States:** Uninitialized, Initialized, Finalized

**Transitions:**
- Uninitialized -> Initialized via initialize_logger() (sets log_file to Some(BufWriter<File>))
- Initialized -> Finalized via finalize_logger() (take()s writer, flushes, leaving None)
- Finalized -> Initialized via initialize_logger() (possible re-init; not prevented)

**Evidence:** thread_local! static log_file: RefCell<Option<BufWriter<File>>> (None/Some encodes state); initialize_logger(): writes *log_file_ref = OpenOptions...open(...).ok().map(BufWriter::new); initialize_logger(): checks log_file...is_none() then prints "Failed to open log file" and returns -1; log_info_internal()/log_warning_internal()/log_error_internal(): `if ... is_some()` else return (no-op when uninitialized/finalized); finalize_logger(): `log_file_ref.take().unwrap()` then flushes; leaves log_file as None

**Implementation:** Replace the implicit global state with an explicit Logger handle: `struct Logger { writer: BufWriter<File> }`. Make `initialize_logger() -> Result<Logger, _>` and require `&Logger` (or `Logger` passed into TaskManager/driver) for `log_*` calls. Implement `Drop` for Logger to flush/close automatically; keep a separate `NoLogger`/optional wrapper if needed for FFI.

---

## State Machine Invariants

### 3. TaskManager allocation/validity & capacity protocol (Null / Alive / Freed; and WithinCapacity)

**Location**: `/data/test_case/lib.rs:1-365`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: TaskManager is manually heap-allocated and returned as a raw pointer; methods assume the pointer is non-null and points to a valid allocation whose `tasks` points to an allocation of `max_tasks` Tasks. add_task_internal/print_tasks unwrap the Option<&mut TaskManager>/Option<&TaskManager> and then create slices from raw parts, which is only valid in the Alive state. destroy_task_manager frees both `mgr.tasks` and the manager itself, transitioning to Freed; using the pointer after that is UB but not prevented. Additionally, adding tasks must respect a capacity invariant (`task_count < max_tasks`) which is enforced at runtime via a check.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Task, 5 free function(s); struct TaskManager

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
    // Keep module structure expected by the harness: src::driver, src::logger, src::task_manager
    pub mod driver {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct Task {
            pub description: [i8; 256],
            pub priority: i32,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct TaskManager {
            pub tasks: *mut Task,
            pub max_tasks: i32,
            pub task_count: i32,
        }
    }

    pub mod logger {
        extern "C" {
            fn getenv(__name: *const i8) -> *mut i8;
        }

        thread_local! {
            static log_file:
            std::cell::RefCell<Option<std::io::BufWriter<std::fs::File>>> = const
            { std::cell::RefCell::new(None) };
        }

        #[inline]
        fn cstr_from_i8_ptr(ptr: *const i8) -> Option<&'static std::ffi::CStr> {
            if ptr.is_null() {
                None
            } else {
                // SAFETY: caller promises ptr is a valid NUL-terminated C string.
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) })
            }
        }

        #[inline]
        fn cstr_from_i8_slice_until_nul(bytes: &[i8]) -> Option<&std::ffi::CStr> {
            if bytes.is_empty() {
                return None;
            }
            // SAFETY: i8/u8 have identical layout.
            let u8s = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
            std::ffi::CStr::from_bytes_until_nul(u8s).ok()
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_logger() -> i32 {
            let env_ptr = getenv(b"LOG_FILE\0" as *const u8 as *const i8);

            let path_cstr = if let Some(c) = cstr_from_i8_ptr(env_ptr) {
                c
            } else {
                std::ffi::CStr::from_bytes_with_nul(b"default.log\0").unwrap()
            };

            let path_str = match path_cstr.to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Failed to open log file: invalid UTF-8 path");
                    return -1;
                }
            };

            log_file.with_borrow_mut(|log_file_ref| {
                *log_file_ref = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path_str)
                    .ok()
                    .map(std::io::BufWriter::new);
            });

            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_none()) {
                eprintln!("Failed to open log file: {0}", path_str);
                return -1;
            }

            log_info_internal(bytemuck::cast_slice(b"Logger initialized.\0"));
            0
        }

        pub(crate) fn log_info_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[INFO] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_info(message: *const i8) {
            log_info_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_warning_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[WARNING] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_warning(message: *const i8) {
            log_warning_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_error_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[ERROR] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_error(message: *const i8) {
            log_error_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        #[no_mangle]
        pub extern "C" fn finalize_logger() {
            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                log_info_internal(bytemuck::cast_slice(b"Logger finalized.\0"));
                {
                    let mut __x = log_file.with_borrow_mut(|log_file_ref| log_file_ref.take().unwrap());
                    let __v = std::io::Write::flush(&mut __x).map_or(-1, |_| 0);
                    drop(__x);
                    __v
                };
            }
        }
    }

    pub mod task_manager {
        use crate::src::driver::{Task, TaskManager};
        use crate::src::logger::{log_error_internal, log_info_internal, log_warning_internal};

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
            fn getenv(__name: *const i8) -> *mut i8;
        }

        const TASK_DESC_LEN: usize = 256;

        #[inline]
        unsafe fn max_tasks_from_env_or_default() -> i32 {
            let env_ptr = getenv(b"MAX_TASKS\0" as *const u8 as *const i8);
            if env_ptr.is_null() {
                return 10;
            }
            // SAFETY: getenv returns a NUL-terminated C string.
            let cstr = std::ffi::CStr::from_ptr(env_ptr);
            let s = cstr.to_str().ok().unwrap_or("");
            s.trim().parse::<i32>().ok().unwrap_or(10)
        }

        #[no_mangle]
        pub unsafe extern "C" fn create_task_manager() -> *mut TaskManager {
            let manager_ptr = malloc(core::mem::size_of::<TaskManager>()) as *mut TaskManager;
            let Some(manager) = manager_ptr.as_mut() else {
                log_error_internal(bytemuck::cast_slice(
                    b"Failed to allocate memory for TaskManager.\0",
                ));
                return core::ptr::null_mut();
            };

            manager.max_tasks = max_tasks_from_env_or_default();
            manager.task_count = 0;

            let tasks_bytes = (manager.max_tasks as usize).wrapping_mul(core::mem::size_of::<Task>());
            manager.tasks = malloc(tasks_bytes) as *mut Task;

            if manager.tasks.is_null() {
                log_error_internal(bytemuck::cast_slice(b"Failed to allocate memory for tasks.\0"));
                free(manager_ptr as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }

            log_info_internal(bytemuck::cast_slice(b"TaskManager created successfully.\0"));
            manager_ptr
        }

        pub(crate) unsafe fn add_task_internal(
            mut manager: Option<&mut TaskManager>,
            description: &[i8],
            priority: i32,
        ) {
            let mgr = manager.as_deref_mut().unwrap();

            if mgr.task_count >= mgr.max_tasks {
                log_warning_internal(bytemuck::cast_slice(
                    b"Cannot add task: Maximum task limit reached.\0",
                ));
                return;
            }

            let idx = mgr.task_count as usize;
            mgr.task_count += 1;

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts_mut(mgr.tasks, mgr.max_tasks as usize);
            let task = &mut tasks[idx];

            // Match original semantics: always copy exactly 255 bytes from `description` slice
            // (caller provides 1024 bytes), then set last byte to NUL.
            let copy_len = TASK_DESC_LEN - 1;
            task.description[..copy_len].copy_from_slice(&description[..copy_len]);
            task.description[copy_len] = 0;
            task.priority = priority;

            log_info_internal(bytemuck::cast_slice(b"Task added successfully.\0"));
        }

        #[no_mangle]
        pub unsafe extern "C" fn add_task(
            manager: Option<&mut TaskManager>,
            description: *const i8,
            priority: i32,
        ) {
            add_task_internal(
                manager,
                if description.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(description, 1024)
                },
                priority,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn print_tasks(manager: Option<&mut TaskManager>) {
            println!("Tasks:");
            let mgr = manager.as_deref().unwrap();

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts(mgr.tasks, mgr.max_tasks as usize);

            for i in 0..(mgr.task_count as usize) {
                let task = &tasks[i];
                let desc_u8 = std::slice::from_raw_parts(
                    task.description.as_ptr() as *const u8,
                    task.description.len(),
                );
                let desc = std::ffi::CStr::from_bytes_until_nul(desc_u8)
                    .ok()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                println!("  [{0}] {1} (Priority: {2})", i + 1, desc, task.priority);
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn destroy_task_manager(manager: Option<&TaskManager>) {
            let mgr = manager.unwrap();
            free(mgr.tasks as *mut core::ffi::c_void);
            free((mgr as *const TaskManager as *mut TaskManager) as *mut core::ffi::c_void);
            log_info_internal(bytemuck::cast_slice(
                b"TaskManager destroyed successfully.\0",
            ));
        }
    }
}

// Added by Claude Code after initial translation: driver entry point expected by cando2 test harness
#[no_mangle]
pub unsafe extern "C" fn driver(tasks: *const std::ffi::c_char) -> std::ffi::c_int {
    if tasks.is_null() {
        return 0;
    }

    if src::logger::initialize_logger() != 0 {
        return 1; // EXIT_FAILURE
    }

    let manager = src::task_manager::create_task_manager();
    if manager.is_null() {
        return 1;
    }

    let mut start = tasks;
    let mut priority: std::ffi::c_int = 1;

    while *start != 0 {
        let end = libc::strchr(start, '\n' as std::ffi::c_int);
        let end = if end.is_null() {
            start.add(libc::strlen(start))
        } else {
            end
        };

        let length = (end as usize) - (start as usize);
        let task = libc::malloc(length + 1) as *mut std::ffi::c_char;
        if task.is_null() {
            eprintln!("Error: Failed to allocate memory for task.");
            src::task_manager::destroy_task_manager(manager.as_ref());
            src::logger::finalize_logger();
            return 1;
        }
        libc::strncpy(task, start, length);
        *task.add(length) = 0;

        src::task_manager::add_task(manager.as_mut(), task, priority);
        priority += 1;
        libc::free(task as *mut std::ffi::c_void);

        start = if *end == '\n' as i8 { end.add(1) } else { end };
    }

    src::task_manager::print_tasks(manager.as_mut());
    src::task_manager::destroy_task_manager(manager.as_ref());
    src::logger::finalize_logger();

    0
}
```

**Entity:** src::driver::TaskManager

**States:** Null, Alive, Freed

**Transitions:**
- Null -> Alive via create_task_manager() success (malloc manager, malloc tasks, initialize fields)
- Alive -> Null via create_task_manager() failure (returns null_mut() after logging/freeing partial allocation)
- Alive -> Freed via destroy_task_manager() (free tasks, free manager)
- Alive (WithinCapacity) -> Alive (WithinCapacity or Full) via add_task_internal() (increments task_count until max_tasks)
- Alive (Any) -> UB if print_tasks()/add_task_internal() called with Freed pointer (not prevented)

**Evidence:** driver::TaskManager fields: `tasks: *mut Task`, `max_tasks: i32`, `task_count: i32` (raw pointers + counters encode validity/capacity at runtime); create_task_manager(): `let manager_ptr = malloc(...) as *mut TaskManager; ... else { ... return null_mut(); }`; create_task_manager(): `manager.tasks = malloc(tasks_bytes) as *mut Task; if manager.tasks.is_null() { ... free(manager_ptr); return null_mut(); }`; add_task_internal(): `let mgr = manager.as_deref_mut().unwrap();` (requires non-None manager / non-null from FFI); add_task_internal(): `if mgr.task_count >= mgr.max_tasks { ... return; }` (runtime capacity gate); add_task_internal()/print_tasks(): `std::slice::from_raw_parts(_mut)(mgr.tasks, mgr.max_tasks as usize)` relies on tasks allocation being valid and alive; destroy_task_manager(): `free(mgr.tasks ...)` then `free(mgr as *const TaskManager as *mut TaskManager ...)` (explicit transition to Freed)

**Implementation:** Wrap the raw allocation in an owning Rust type: `struct OwnedTaskManager { inner: NonNull<TaskManager> }` (or avoid separate malloc by storing Vec<Task>). Implement `Drop` to call free exactly once. Expose safe methods on `&mut OwnedTaskManager` that cannot be called on Null/Freed. Represent tasks storage as `Vec<Task>` (capacity known) to remove `from_raw_parts` and encode length/capacity safely; enforce `task_count <= max_tasks` by using `Vec::len()` or a bounded collection.

---

## Precondition Invariants

### 4. FFI description buffer protocol (must be at least 255 bytes and NUL-terminated semantics)

**Location**: `/data/test_case/lib.rs:1-365`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: add_task_internal assumes the incoming `description` slice has at least 255 bytes and blindly copies `TASK_DESC_LEN-1` bytes from it. The public FFI wrapper constructs a 1024-byte slice from a raw pointer, assuming the pointed-to buffer is valid for that length. These are strong preconditions on the caller (pointer validity + minimum length) that are not expressed in the types; violating them can cause out-of-bounds reads/UB.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Task, 5 free function(s); struct TaskManager

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
    // Keep module structure expected by the harness: src::driver, src::logger, src::task_manager
    pub mod driver {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct Task {
            pub description: [i8; 256],
            pub priority: i32,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct TaskManager {
            pub tasks: *mut Task,
            pub max_tasks: i32,
            pub task_count: i32,
        }
    }

    pub mod logger {
        extern "C" {
            fn getenv(__name: *const i8) -> *mut i8;
        }

        thread_local! {
            static log_file:
            std::cell::RefCell<Option<std::io::BufWriter<std::fs::File>>> = const
            { std::cell::RefCell::new(None) };
        }

        #[inline]
        fn cstr_from_i8_ptr(ptr: *const i8) -> Option<&'static std::ffi::CStr> {
            if ptr.is_null() {
                None
            } else {
                // SAFETY: caller promises ptr is a valid NUL-terminated C string.
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) })
            }
        }

        #[inline]
        fn cstr_from_i8_slice_until_nul(bytes: &[i8]) -> Option<&std::ffi::CStr> {
            if bytes.is_empty() {
                return None;
            }
            // SAFETY: i8/u8 have identical layout.
            let u8s = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
            std::ffi::CStr::from_bytes_until_nul(u8s).ok()
        }

        #[no_mangle]
        pub unsafe extern "C" fn initialize_logger() -> i32 {
            let env_ptr = getenv(b"LOG_FILE\0" as *const u8 as *const i8);

            let path_cstr = if let Some(c) = cstr_from_i8_ptr(env_ptr) {
                c
            } else {
                std::ffi::CStr::from_bytes_with_nul(b"default.log\0").unwrap()
            };

            let path_str = match path_cstr.to_str() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Failed to open log file: invalid UTF-8 path");
                    return -1;
                }
            };

            log_file.with_borrow_mut(|log_file_ref| {
                *log_file_ref = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path_str)
                    .ok()
                    .map(std::io::BufWriter::new);
            });

            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_none()) {
                eprintln!("Failed to open log file: {0}", path_str);
                return -1;
            }

            log_info_internal(bytemuck::cast_slice(b"Logger initialized.\0"));
            0
        }

        pub(crate) fn log_info_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[INFO] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_info(message: *const i8) {
            log_info_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_warning_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[WARNING] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_warning(message: *const i8) {
            log_warning_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        pub(crate) fn log_error_internal(message: &[i8]) {
            if !log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                return;
            }
            log_file.with_borrow_mut(|log_file_ref| {
                use std::io::Write;
                let Some(writer) = log_file_ref.as_mut() else { return };
                let msg = cstr_from_i8_slice_until_nul(message)
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                let _ = writeln!(writer, "[ERROR] {0}", msg);
            });
        }

        #[no_mangle]
        pub unsafe extern "C" fn log_error(message: *const i8) {
            log_error_internal(if message.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(message, 1024)
            })
        }

        #[no_mangle]
        pub extern "C" fn finalize_logger() {
            if log_file.with_borrow(|log_file_ref| (*log_file_ref).is_some()) {
                log_info_internal(bytemuck::cast_slice(b"Logger finalized.\0"));
                {
                    let mut __x = log_file.with_borrow_mut(|log_file_ref| log_file_ref.take().unwrap());
                    let __v = std::io::Write::flush(&mut __x).map_or(-1, |_| 0);
                    drop(__x);
                    __v
                };
            }
        }
    }

    pub mod task_manager {
        use crate::src::driver::{Task, TaskManager};
        use crate::src::logger::{log_error_internal, log_info_internal, log_warning_internal};

        extern "C" {
            fn malloc(__size: usize) -> *mut core::ffi::c_void;
            fn free(__ptr: *mut core::ffi::c_void);
            fn getenv(__name: *const i8) -> *mut i8;
        }

        const TASK_DESC_LEN: usize = 256;

        #[inline]
        unsafe fn max_tasks_from_env_or_default() -> i32 {
            let env_ptr = getenv(b"MAX_TASKS\0" as *const u8 as *const i8);
            if env_ptr.is_null() {
                return 10;
            }
            // SAFETY: getenv returns a NUL-terminated C string.
            let cstr = std::ffi::CStr::from_ptr(env_ptr);
            let s = cstr.to_str().ok().unwrap_or("");
            s.trim().parse::<i32>().ok().unwrap_or(10)
        }

        #[no_mangle]
        pub unsafe extern "C" fn create_task_manager() -> *mut TaskManager {
            let manager_ptr = malloc(core::mem::size_of::<TaskManager>()) as *mut TaskManager;
            let Some(manager) = manager_ptr.as_mut() else {
                log_error_internal(bytemuck::cast_slice(
                    b"Failed to allocate memory for TaskManager.\0",
                ));
                return core::ptr::null_mut();
            };

            manager.max_tasks = max_tasks_from_env_or_default();
            manager.task_count = 0;

            let tasks_bytes = (manager.max_tasks as usize).wrapping_mul(core::mem::size_of::<Task>());
            manager.tasks = malloc(tasks_bytes) as *mut Task;

            if manager.tasks.is_null() {
                log_error_internal(bytemuck::cast_slice(b"Failed to allocate memory for tasks.\0"));
                free(manager_ptr as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }

            log_info_internal(bytemuck::cast_slice(b"TaskManager created successfully.\0"));
            manager_ptr
        }

        pub(crate) unsafe fn add_task_internal(
            mut manager: Option<&mut TaskManager>,
            description: &[i8],
            priority: i32,
        ) {
            let mgr = manager.as_deref_mut().unwrap();

            if mgr.task_count >= mgr.max_tasks {
                log_warning_internal(bytemuck::cast_slice(
                    b"Cannot add task: Maximum task limit reached.\0",
                ));
                return;
            }

            let idx = mgr.task_count as usize;
            mgr.task_count += 1;

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts_mut(mgr.tasks, mgr.max_tasks as usize);
            let task = &mut tasks[idx];

            // Match original semantics: always copy exactly 255 bytes from `description` slice
            // (caller provides 1024 bytes), then set last byte to NUL.
            let copy_len = TASK_DESC_LEN - 1;
            task.description[..copy_len].copy_from_slice(&description[..copy_len]);
            task.description[copy_len] = 0;
            task.priority = priority;

            log_info_internal(bytemuck::cast_slice(b"Task added successfully.\0"));
        }

        #[no_mangle]
        pub unsafe extern "C" fn add_task(
            manager: Option<&mut TaskManager>,
            description: *const i8,
            priority: i32,
        ) {
            add_task_internal(
                manager,
                if description.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(description, 1024)
                },
                priority,
            )
        }

        #[no_mangle]
        pub unsafe extern "C" fn print_tasks(manager: Option<&mut TaskManager>) {
            println!("Tasks:");
            let mgr = manager.as_deref().unwrap();

            // SAFETY: tasks points to an allocation of max_tasks Tasks.
            let tasks = std::slice::from_raw_parts(mgr.tasks, mgr.max_tasks as usize);

            for i in 0..(mgr.task_count as usize) {
                let task = &tasks[i];
                let desc_u8 = std::slice::from_raw_parts(
                    task.description.as_ptr() as *const u8,
                    task.description.len(),
                );
                let desc = std::ffi::CStr::from_bytes_until_nul(desc_u8)
                    .ok()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("");
                println!("  [{0}] {1} (Priority: {2})", i + 1, desc, task.priority);
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn destroy_task_manager(manager: Option<&TaskManager>) {
            let mgr = manager.unwrap();
            free(mgr.tasks as *mut core::ffi::c_void);
            free((mgr as *const TaskManager as *mut TaskManager) as *mut core::ffi::c_void);
            log_info_internal(bytemuck::cast_slice(
                b"TaskManager destroyed successfully.\0",
            ));
        }
    }
}

// Added by Claude Code after initial translation: driver entry point expected by cando2 test harness
#[no_mangle]
pub unsafe extern "C" fn driver(tasks: *const std::ffi::c_char) -> std::ffi::c_int {
    if tasks.is_null() {
        return 0;
    }

    if src::logger::initialize_logger() != 0 {
        return 1; // EXIT_FAILURE
    }

    let manager = src::task_manager::create_task_manager();
    if manager.is_null() {
        return 1;
    }

    let mut start = tasks;
    let mut priority: std::ffi::c_int = 1;

    while *start != 0 {
        let end = libc::strchr(start, '\n' as std::ffi::c_int);
        let end = if end.is_null() {
            start.add(libc::strlen(start))
        } else {
            end
        };

        let length = (end as usize) - (start as usize);
        let task = libc::malloc(length + 1) as *mut std::ffi::c_char;
        if task.is_null() {
            eprintln!("Error: Failed to allocate memory for task.");
            src::task_manager::destroy_task_manager(manager.as_ref());
            src::logger::finalize_logger();
            return 1;
        }
        libc::strncpy(task, start, length);
        *task.add(length) = 0;

        src::task_manager::add_task(manager.as_mut(), task, priority);
        priority += 1;
        libc::free(task as *mut std::ffi::c_void);

        start = if *end == '\n' as i8 { end.add(1) } else { end };
    }

    src::task_manager::print_tasks(manager.as_mut());
    src::task_manager::destroy_task_manager(manager.as_ref());
    src::logger::finalize_logger();

    0
}
```

**Entity:** src::task_manager::add_task_internal (and FFI add_task)

**States:** ValidDescriptionBuffer, InvalidDescriptionBuffer

**Transitions:**
- InvalidDescriptionBuffer -> UB via add_task_internal() copy_from_slice of `description[..255]`
- ValidDescriptionBuffer -> ValidDescriptionBuffer via add_task_internal() (copies, then writes trailing NUL)

**Evidence:** const TASK_DESC_LEN: usize = 256; and `let copy_len = TASK_DESC_LEN - 1;`; add_task_internal(): `task.description[..copy_len].copy_from_slice(&description[..copy_len]);` (requires description.len() >= 255); comment in add_task_internal(): "always copy exactly 255 bytes from `description` slice (caller provides 1024 bytes)"; FFI add_task(): `std::slice::from_raw_parts(description, 1024)` (requires pointer valid for 1024 bytes even if string shorter); FFI log_info/log_warning/log_error similarly use `from_raw_parts(message, 1024)` (same latent precondition pattern)

**Implementation:** Introduce a validated wrapper for inbound FFI buffers, e.g. `struct FfiMsg<'a>(&'a [u8]);` constructed only from `NonNull<c_char>` plus an explicit length (passed from C) or via `CStr::from_ptr` (scan to NUL). Then make `add_task_internal` take `&CStr` or `&[u8; 255]`-like bounded input to make the minimum-length/validity requirement explicit and checked at the boundary.

---

### 1. TaskManager pointer validity & capacity/count invariant

**Location**: `/data/test_case/lib.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: TaskManager encodes an implicit initialization and validity invariant: `tasks` must point to a valid allocation for `max_tasks` Tasks (or be null when uninitialized), and `task_count` must remain within `[0, max_tasks]`. These constraints are not enforced by the type system because raw pointers and plain integers carry no provenance/ownership or range guarantees. Any API that reads/writes through `tasks` or indexes it implicitly relies on these preconditions to avoid UB/out-of-bounds access.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Task, 5 free function(s); 12 free function(s)


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct TaskManager {
            pub tasks: *mut Task,
            pub max_tasks: i32,
            pub task_count: i32,
        }

```

**Entity:** TaskManager

**States:** Uninitialized/Null, Initialized/Allocated

**Transitions:**
- Uninitialized/Null -> Initialized/Allocated via an allocator/initializer (not shown)
- Initialized/Allocated -> Uninitialized/Null via a deallocator/teardown (not shown)

**Evidence:** field `tasks: *mut Task` is a raw pointer with no lifetime/ownership/nullability guarantees; field `max_tasks: i32` implies a capacity that must match the allocation behind `tasks`; field `task_count: i32` implies a runtime-tracked number of active tasks that must be <= `max_tasks`; `#[derive(Copy, Clone)]` on TaskManager allows duplicating the raw pointer and counters, implying an implicit aliasing/ownership protocol that is not type-checked

**Implementation:** Replace `tasks: *mut Task` + `max_tasks` with `Vec<Task>` or `Box<[MaybeUninit<Task>]>` (or `NonNull<Task>` plus a length newtype) owned by TaskManager; make TaskManager non-Copy; encode counts/capacity as `usize` and keep `task_count` derived from the container length (or use a `TaskCount` newtype enforcing `<= max_tasks`). If FFI layout is required, wrap the `#[repr(C)]` struct in a safe Rust owner type that manages allocation/deallocation and exposes safe methods.

---

