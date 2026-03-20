# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Temporal Ordering Invariants

### 2. ConfigFlags presence + initialization-before-use protocol

**Location**: `/data/test_case/lib.rs:1-292`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Multiple functions accept `Option<&mut ConfigFlags>`/`Option<&ConfigFlags>` but then immediately `unwrap()` it, meaning callers must always pass `Some(...)` (presence invariant) and, implicitly, must have had flags configured before being read (initialization-before-use). The code relies on a specific call order in `envy`: `init_config_from_env(Some(&mut state.flags))` must occur before `perform_operation(..., Some(&mut state.flags))` and `apply_bit_operations(..., Some(&mut state.flags))`, otherwise flags would be at default/unknown values. None of this is enforced by the type system; violations panic at runtime (`unwrap`) or silently change behavior by reading default bits.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ConfigFlags, 3 free function(s); struct ProcessState

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
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn getenv(__name: *const i8) -> *mut i8;
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
    fn strchr(__s: *const i8, __c: i32) -> *mut i8;
}

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct ConfigFlags {
    #[bitfield(name = "verbose", ty = "u32", bits = "0..=0")]
    #[bitfield(name = "debug", ty = "u32", bits = "1..=1")]
    #[bitfield(name = "optimize", ty = "u32", bits = "2..=2")]
    #[bitfield(name = "cache_enabled", ty = "u32", bits = "3..=3")]
    #[bitfield(name = "log_level", ty = "u32", bits = "4..=6")]
    #[bitfield(name = "reserved", ty = "u32", bits = "7..=7")]
    pub verbose_debug_optimize_cache_enabled_log_level_reserved: [u8; 1],
    pub c2rust_padding: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessState {
    pub flags: ConfigFlags,
    pub base_value: i32,
    pub multiplier: i32,
    pub operation: i8,
}

pub const BUFFER_SIZE: i32 = 256;

#[inline]
unsafe fn c_str_ptr_to_bytes(ptr: *const i8) -> &'static [i8] {
    if ptr.is_null() {
        &[]
    } else {
        // Preserve original C2Rust behavior (large upper bound).
        std::slice::from_raw_parts(ptr, 100000)
    }
}

#[inline]
unsafe fn c_atoi(s: *const i8) -> i32 {
    if s.is_null() {
        return 0;
    }
    let mut p = s;

    // Skip leading whitespace (C isspace subset).
    loop {
        let ch = *p as u8;
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == 0x0b || ch == 0x0c {
            p = p.add(1);
            continue;
        }
        break;
    }

    let mut sign: i32 = 1;
    if *p == b'-' as i8 {
        sign = -1;
        p = p.add(1);
    } else if *p == b'+' as i8 {
        p = p.add(1);
    }

    let mut acc: i32 = 0;
    loop {
        let ch = *p as u8;
        if !(b'0'..=b'9').contains(&ch) {
            break;
        }
        acc = acc.saturating_mul(10).saturating_add((ch - b'0') as i32);
        p = p.add(1);
    }

    acc.saturating_mul(sign)
}

pub(crate) unsafe fn parse_env_numeric(env_name: &[i8], default_val: i32) -> i32 {
    let env_ptr = getenv(env_name.as_ptr());
    let env_value = c_str_ptr_to_bytes(env_ptr);

    if env_value.is_empty() {
        return default_val;
    }

    if !strchr(env_value.as_ptr(), ',' as i32).is_null() {
        eprintln!(
            "Warning: Invalid character in {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(env_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return default_val;
    }

    if !strchr(env_value.as_ptr(), ';' as i32).is_null() {
        eprintln!(
            "Warning: Semicolon found in {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(env_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return default_val;
    }

    c_atoi(env_value.as_ptr())
}

pub(crate) unsafe fn init_config_from_env(mut flags: Option<&mut ConfigFlags>) {
    let flags = flags.as_deref_mut().unwrap();

    let verbose_env = c_str_ptr_to_bytes(getenv(b"PROG_VERBOSE\0" as *const u8 as *const i8));
    let debug_env = c_str_ptr_to_bytes(getenv(b"PROG_DEBUG\0" as *const u8 as *const i8));
    let optimize_env = getenv(b"PROG_OPTIMIZE\0" as *const u8 as *const i8);

    let verbose_on = !verbose_env.is_empty() && !strchr(verbose_env.as_ptr(), '1' as i32).is_null();
    let debug_on = !debug_env.is_empty() && !strchr(debug_env.as_ptr(), '1' as i32).is_null();
    let optimize_on = !optimize_env.is_null();

    flags.set_verbose(verbose_on as u32);
    flags.set_debug(debug_on as u32);
    flags.set_optimize(optimize_on as u32);
    flags.set_cache_enabled(1);
    flags.set_log_level(0o3);
    flags.set_reserved(0);
}

pub(crate) fn perform_operation(val1: i32, val2: i32, flags: Option<&mut ConfigFlags>) -> i32 {
    let flags = flags.as_deref().unwrap();
    let operation_mode: i32 = 0o755;

    let result = if flags.optimize() != 0 {
        val1 + val2
    } else {
        val1 * flags.log_level() as i32 + val2 / 2
    };

    if flags.debug() != 0 {
        println!("Debug: operation_mode = {0:o} (octal)", operation_mode as u32);
        println!("Debug: result before adjustment = {result}");
    }

    result
}

pub(crate) fn apply_bit_operations(mut value: i32, flags: Option<&mut ConfigFlags>) -> i32 {
    let flags = flags.as_deref().unwrap();

    if flags.verbose() != 0 {
        value <<= 1;
    }
    if flags.cache_enabled() != 0 {
        value |= 0xf;
    }
    value
}

#[no_mangle]
pub unsafe extern "C" fn envy(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut state: ProcessState = ProcessState {
        flags: ConfigFlags {
            verbose_debug_optimize_cache_enabled_log_level_reserved: [0; 1],
            c2rust_padding: [0; 3],
        },
        base_value: 0,
        multiplier: 0,
        operation: 0,
    };
    let mut state_backup: ProcessState = ProcessState {
        flags: ConfigFlags {
            verbose_debug_optimize_cache_enabled_log_level_reserved: [0; 1],
            c2rust_padding: [0; 3],
        },
        base_value: 0,
        multiplier: 0,
        operation: 0,
    };
    let mut buffer: [i8; 256] = [0; 256];

    init_config_from_env(Some(&mut state.flags));

    let base_offset: i32 = parse_env_numeric(bytemuck::cast_slice(b"PROG_BASE_OFFSET\0"), 0o100);
    let multiplier: i32 = parse_env_numeric(bytemuck::cast_slice(b"PROG_MULTIPLIER\0"), 0o12);

    if state.flags.verbose() != 0 {
        println!("Verbose mode enabled");
        println!("Base offset: {base_offset} (from octal 0100)");
        println!("Multiplier: {multiplier} (from octal 012)");
    }

    state.base_value = param1;
    state.multiplier = multiplier;
    state.operation = b'+' as i8;

    // Preserve original memcpy behavior.
    memcpy(
        &raw mut state_backup as *mut std::ffi::c_void,
        &raw const state as *const std::ffi::c_void,
        core::mem::size_of::<ProcessState>(),
    );

    if state.flags.debug() != 0 {
        println!("Debug: Created state backup using memcpy");
        println!("Debug: Backup base_value = {0}", state_backup.base_value);
    }

    let mut result: i32 = perform_operation(param1, param2, Some(&mut state.flags));

    if param3 != 0 {
        result += param3 * state.multiplier;
    }
    if param4 != 0 {
        result += param4 >> 2;
    }

    result = apply_bit_operations(result, Some(&mut state.flags));
    result += base_offset;

    snprintf(
        buffer.as_mut_ptr(),
        BUFFER_SIZE as usize,
        b"Result:%d:Complete\0" as *const u8 as *const i8,
        result,
    );

    let colon_ptr = strchr(buffer.as_ptr(), ':' as i32);
    let colon_pos: &[i8] = c_str_ptr_to_bytes(colon_ptr);

    if !colon_pos.is_empty() {
        if state.flags.verbose() != 0 {
            println!(
                "Found colon at position: {0}",
                colon_ptr.offset_from(buffer.as_ptr()) as i64
            );
        }

        if colon_pos.len() > 1 {
            let second_colon = strchr(colon_pos[1..].as_ptr(), ':' as i32);
            if !second_colon.is_null() && state.flags.debug() != 0 {
                println!("Debug: Result string format validated");
            }
        }
    }

    if result < 0 {
        memcpy(
            &raw mut state as *mut std::ffi::c_void,
            &raw const state_backup as *const std::ffi::c_void,
            core::mem::size_of::<ProcessState>(),
        );
        result = state.base_value;

        if state.flags.verbose() != 0 {
            println!("Restored state from backup");
        }
    }

    if state.flags.verbose() != 0 {
        println!("Final result: {result}");
        println!(
            "Configuration - Debug: {0}, Optimize: {1}, Log Level: {2}",
            state.flags.debug() as i32,
            state.flags.optimize() as i32,
            state.flags.log_level() as i32
        );
    }

    result
}
```

**Entity:** ConfigFlags (as used via Option<&mut ConfigFlags>/Option<&ConfigFlags>)

**States:** Absent (None), Present-but-uninitialized/unknown, Present-and-initialized (via init_config_from_env)

**Transitions:**
- Absent (None) -> panic via unwrap() in init_config_from_env/perform_operation/apply_bit_operations
- Present-but-uninitialized/unknown -> Present-and-initialized via init_config_from_env(Some(&mut ConfigFlags))

**Evidence:** init_config_from_env(mut flags: Option<&mut ConfigFlags>): `let flags = flags.as_deref_mut().unwrap();` (panics if None); perform_operation(..., flags: Option<&mut ConfigFlags>): `let flags = flags.as_deref().unwrap();` (panics if None); apply_bit_operations(..., flags: Option<&mut ConfigFlags>): `let flags = flags.as_deref().unwrap();` (panics if None); envy(): calls `init_config_from_env(Some(&mut state.flags));` before `perform_operation(..., Some(&mut state.flags))` and `apply_bit_operations(..., Some(&mut state.flags))` (ordering relied upon)

**Implementation:** Remove the `Option` and model initialization explicitly: `struct Flags<St>{ inner: ConfigFlags, _st: PhantomData<St> }` with `Uninit`/`Init` states. Provide `fn init(self) -> Flags<Init>` (or `fn init(&mut self) -> &mut Flags<Init>` via wrapper) and make `perform_operation/apply_bit_operations` accept `&Flags<Init>` only. If optionality is truly needed, use `Option<Flags<Init>>` at a higher level so `unwrap()` is not scattered.

---

## Precondition Invariants

### 1. ConfigFlags bitfield validity protocol (reserved bit must remain unset; log_level must be in-range)

**Location**: `/data/test_case/lib.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: ConfigFlags encodes multiple boolean/bitfield settings into a raw byte array. There is an implicit validity requirement that only the documented bits are used: the `reserved` bit should remain 0 (and potentially other bits outside the declared fields must remain 0), and `log_level` should be restricted to the meaningful subset of values for the system (even though 3 bits allow 0..=7). Because the underlying representation is a raw `[u8; 1]` with generated accessors, the type system cannot prevent constructing or mutating a ConfigFlags value into an invalid configuration (e.g., setting reserved=1 or using a nonsensical log_level).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ProcessState; 4 free function(s)


#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct ConfigFlags {
    #[bitfield(name = "verbose", ty = "u32", bits = "0..=0")]
    #[bitfield(name = "debug", ty = "u32", bits = "1..=1")]
    #[bitfield(name = "optimize", ty = "u32", bits = "2..=2")]
    #[bitfield(name = "cache_enabled", ty = "u32", bits = "3..=3")]
    #[bitfield(name = "log_level", ty = "u32", bits = "4..=6")]
    #[bitfield(name = "reserved", ty = "u32", bits = "7..=7")]
    pub verbose_debug_optimize_cache_enabled_log_level_reserved: [u8; 1],
    pub c2rust_padding: [u8; 3],
}

```

**Entity:** ConfigFlags

**States:** Valid, Invalid

**Transitions:**
- Valid -> Invalid via setting reserved/log_level bits on verbose_debug_optimize_cache_enabled_log_level_reserved

**Evidence:** field verbose_debug_optimize_cache_enabled_log_level_reserved: [u8; 1] stores all flags as raw bits; bitfield(name = "reserved", bits = "7..=7") indicates a reserved bit that is implicitly expected to be unused; bitfield(name = "log_level", bits = "4..=6") uses 3 bits (0..=7 possible) but typically only some levels are meaningful

**Implementation:** Introduce validated wrappers for the constrained fields, e.g. `struct LogLevel(u8);` with `TryFrom<u8>` enforcing allowed values, and `struct ConfigFlagsValid(ConfigFlags);` constructed only through safe constructors that (1) zero reserved bits and (2) accept a `LogLevel` instead of a raw integer; keep raw bit-twiddling/FFI access behind `unsafe` or `pub(crate)` APIs.

---

## Protocol Invariants

### 3. ProcessState snapshot/rollback protocol (Live ↔ BackedUp/Restored)

**Location**: `/data/test_case/lib.rs:1-292`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: `envy` maintains a manual backup of `ProcessState` using `memcpy` to allow rollback when `result < 0`. This implies a protocol: create a backup only after the state is fully set up, then either continue with the live state or restore from the backup on an error condition. The correctness relies on remembering to snapshot at the right time and to restore consistently; this is not captured in the type system and is easy to break if future edits move the `memcpy` calls or add new state fields that should participate in the snapshot.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ConfigFlags, 3 free function(s); struct ProcessState

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
    fn snprintf(__s: *mut i8, __maxlen: usize, __format: *const i8, ...) -> i32;
    fn getenv(__name: *const i8) -> *mut i8;
    fn memcpy(
        __dest: *mut core::ffi::c_void,
        __src: *const core::ffi::c_void,
        __n: usize,
    ) -> *mut core::ffi::c_void;
    fn strchr(__s: *const i8, __c: i32) -> *mut i8;
}

#[repr(C)]
#[derive(Copy, Clone, BitfieldStruct)]
pub struct ConfigFlags {
    #[bitfield(name = "verbose", ty = "u32", bits = "0..=0")]
    #[bitfield(name = "debug", ty = "u32", bits = "1..=1")]
    #[bitfield(name = "optimize", ty = "u32", bits = "2..=2")]
    #[bitfield(name = "cache_enabled", ty = "u32", bits = "3..=3")]
    #[bitfield(name = "log_level", ty = "u32", bits = "4..=6")]
    #[bitfield(name = "reserved", ty = "u32", bits = "7..=7")]
    pub verbose_debug_optimize_cache_enabled_log_level_reserved: [u8; 1],
    pub c2rust_padding: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessState {
    pub flags: ConfigFlags,
    pub base_value: i32,
    pub multiplier: i32,
    pub operation: i8,
}

pub const BUFFER_SIZE: i32 = 256;

#[inline]
unsafe fn c_str_ptr_to_bytes(ptr: *const i8) -> &'static [i8] {
    if ptr.is_null() {
        &[]
    } else {
        // Preserve original C2Rust behavior (large upper bound).
        std::slice::from_raw_parts(ptr, 100000)
    }
}

#[inline]
unsafe fn c_atoi(s: *const i8) -> i32 {
    if s.is_null() {
        return 0;
    }
    let mut p = s;

    // Skip leading whitespace (C isspace subset).
    loop {
        let ch = *p as u8;
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == 0x0b || ch == 0x0c {
            p = p.add(1);
            continue;
        }
        break;
    }

    let mut sign: i32 = 1;
    if *p == b'-' as i8 {
        sign = -1;
        p = p.add(1);
    } else if *p == b'+' as i8 {
        p = p.add(1);
    }

    let mut acc: i32 = 0;
    loop {
        let ch = *p as u8;
        if !(b'0'..=b'9').contains(&ch) {
            break;
        }
        acc = acc.saturating_mul(10).saturating_add((ch - b'0') as i32);
        p = p.add(1);
    }

    acc.saturating_mul(sign)
}

pub(crate) unsafe fn parse_env_numeric(env_name: &[i8], default_val: i32) -> i32 {
    let env_ptr = getenv(env_name.as_ptr());
    let env_value = c_str_ptr_to_bytes(env_ptr);

    if env_value.is_empty() {
        return default_val;
    }

    if !strchr(env_value.as_ptr(), ',' as i32).is_null() {
        eprintln!(
            "Warning: Invalid character in {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(env_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return default_val;
    }

    if !strchr(env_value.as_ptr(), ';' as i32).is_null() {
        eprintln!(
            "Warning: Semicolon found in {0}",
            std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(env_name))
                .unwrap()
                .to_str()
                .unwrap()
        );
        return default_val;
    }

    c_atoi(env_value.as_ptr())
}

pub(crate) unsafe fn init_config_from_env(mut flags: Option<&mut ConfigFlags>) {
    let flags = flags.as_deref_mut().unwrap();

    let verbose_env = c_str_ptr_to_bytes(getenv(b"PROG_VERBOSE\0" as *const u8 as *const i8));
    let debug_env = c_str_ptr_to_bytes(getenv(b"PROG_DEBUG\0" as *const u8 as *const i8));
    let optimize_env = getenv(b"PROG_OPTIMIZE\0" as *const u8 as *const i8);

    let verbose_on = !verbose_env.is_empty() && !strchr(verbose_env.as_ptr(), '1' as i32).is_null();
    let debug_on = !debug_env.is_empty() && !strchr(debug_env.as_ptr(), '1' as i32).is_null();
    let optimize_on = !optimize_env.is_null();

    flags.set_verbose(verbose_on as u32);
    flags.set_debug(debug_on as u32);
    flags.set_optimize(optimize_on as u32);
    flags.set_cache_enabled(1);
    flags.set_log_level(0o3);
    flags.set_reserved(0);
}

pub(crate) fn perform_operation(val1: i32, val2: i32, flags: Option<&mut ConfigFlags>) -> i32 {
    let flags = flags.as_deref().unwrap();
    let operation_mode: i32 = 0o755;

    let result = if flags.optimize() != 0 {
        val1 + val2
    } else {
        val1 * flags.log_level() as i32 + val2 / 2
    };

    if flags.debug() != 0 {
        println!("Debug: operation_mode = {0:o} (octal)", operation_mode as u32);
        println!("Debug: result before adjustment = {result}");
    }

    result
}

pub(crate) fn apply_bit_operations(mut value: i32, flags: Option<&mut ConfigFlags>) -> i32 {
    let flags = flags.as_deref().unwrap();

    if flags.verbose() != 0 {
        value <<= 1;
    }
    if flags.cache_enabled() != 0 {
        value |= 0xf;
    }
    value
}

#[no_mangle]
pub unsafe extern "C" fn envy(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut state: ProcessState = ProcessState {
        flags: ConfigFlags {
            verbose_debug_optimize_cache_enabled_log_level_reserved: [0; 1],
            c2rust_padding: [0; 3],
        },
        base_value: 0,
        multiplier: 0,
        operation: 0,
    };
    let mut state_backup: ProcessState = ProcessState {
        flags: ConfigFlags {
            verbose_debug_optimize_cache_enabled_log_level_reserved: [0; 1],
            c2rust_padding: [0; 3],
        },
        base_value: 0,
        multiplier: 0,
        operation: 0,
    };
    let mut buffer: [i8; 256] = [0; 256];

    init_config_from_env(Some(&mut state.flags));

    let base_offset: i32 = parse_env_numeric(bytemuck::cast_slice(b"PROG_BASE_OFFSET\0"), 0o100);
    let multiplier: i32 = parse_env_numeric(bytemuck::cast_slice(b"PROG_MULTIPLIER\0"), 0o12);

    if state.flags.verbose() != 0 {
        println!("Verbose mode enabled");
        println!("Base offset: {base_offset} (from octal 0100)");
        println!("Multiplier: {multiplier} (from octal 012)");
    }

    state.base_value = param1;
    state.multiplier = multiplier;
    state.operation = b'+' as i8;

    // Preserve original memcpy behavior.
    memcpy(
        &raw mut state_backup as *mut std::ffi::c_void,
        &raw const state as *const std::ffi::c_void,
        core::mem::size_of::<ProcessState>(),
    );

    if state.flags.debug() != 0 {
        println!("Debug: Created state backup using memcpy");
        println!("Debug: Backup base_value = {0}", state_backup.base_value);
    }

    let mut result: i32 = perform_operation(param1, param2, Some(&mut state.flags));

    if param3 != 0 {
        result += param3 * state.multiplier;
    }
    if param4 != 0 {
        result += param4 >> 2;
    }

    result = apply_bit_operations(result, Some(&mut state.flags));
    result += base_offset;

    snprintf(
        buffer.as_mut_ptr(),
        BUFFER_SIZE as usize,
        b"Result:%d:Complete\0" as *const u8 as *const i8,
        result,
    );

    let colon_ptr = strchr(buffer.as_ptr(), ':' as i32);
    let colon_pos: &[i8] = c_str_ptr_to_bytes(colon_ptr);

    if !colon_pos.is_empty() {
        if state.flags.verbose() != 0 {
            println!(
                "Found colon at position: {0}",
                colon_ptr.offset_from(buffer.as_ptr()) as i64
            );
        }

        if colon_pos.len() > 1 {
            let second_colon = strchr(colon_pos[1..].as_ptr(), ':' as i32);
            if !second_colon.is_null() && state.flags.debug() != 0 {
                println!("Debug: Result string format validated");
            }
        }
    }

    if result < 0 {
        memcpy(
            &raw mut state as *mut std::ffi::c_void,
            &raw const state_backup as *const std::ffi::c_void,
            core::mem::size_of::<ProcessState>(),
        );
        result = state.base_value;

        if state.flags.verbose() != 0 {
            println!("Restored state from backup");
        }
    }

    if state.flags.verbose() != 0 {
        println!("Final result: {result}");
        println!(
            "Configuration - Debug: {0}, Optimize: {1}, Log Level: {2}",
            state.flags.debug() as i32,
            state.flags.optimize() as i32,
            state.flags.log_level() as i32
        );
    }

    result
}
```

**Entity:** ProcessState

**States:** Live (current state), BackedUp (snapshot taken), Restored-from-backup (after rollback)

**Transitions:**
- Live -> BackedUp via memcpy(&mut state_backup, &state, size_of::<ProcessState>())
- Live -> Restored-from-backup via memcpy(&mut state, &state_backup, size_of::<ProcessState>()) when `result < 0`

**Evidence:** envy(): `let mut state: ProcessState = ...; let mut state_backup: ProcessState = ...;` (explicit separate backup storage); envy(): comment `// Preserve original memcpy behavior.` immediately before snapshot memcpy into `state_backup`; envy(): snapshot: `memcpy(&raw mut state_backup ..., &raw const state ..., size_of::<ProcessState>())`; envy(): rollback path: `if result < 0 { memcpy(&raw mut state ..., &raw const state_backup ..., size_of::<ProcessState>()); result = state.base_value; }`

**Implementation:** Introduce a safe snapshot guard that captures and can restore: `struct StateCheckpoint<'a>{ state: &'a mut ProcessState, backup: ProcessState }` with `fn new(state: &'a mut ProcessState) -> Self` (takes backup) and `fn restore(self)` or `Drop`-based conditional rollback (e.g., explicit `commit()` to prevent restore). This makes the snapshot timing explicit and reduces the chance of forgetting to snapshot/restore when modifying code.

---

