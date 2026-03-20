# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 3
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## Resource Lifecycle Invariants

### 1. C-heap ownership & validity protocol for DataEntry buffers (Null / Allocated / Freed)

**Location**: `/data/test_case/lib.rs:1-198`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The pointer returned by create_entries(count, base_id) is either null (allocation failed / count<=0) or a valid C-allocated buffer of exactly `count` contiguous DataEntry values. Callers must (1) check for null before forming a slice, (2) only use the buffer with the same `count` used for allocation, and (3) free it exactly once with `free()`. This protocol is enforced only by null checks and local control flow; the type system does not prevent use-after-free, double-free, or mismatched length when constructing slices.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataEntry, 3 free function(s)

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
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataEntry {
    pub id: i32,
    pub value: i32,
    pub name: [i8; 32],
}

pub const NAME_LENGTH: i32 = 32;

static lookup_table: [[i32; 3]; 4] = [
    [10, 20, 30],
    [40, 50, 60],
    [70, 80, 90],
    [100, 110, 120],
];

unsafe fn find_entry(entries: &mut [DataEntry], count: i32, target_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }
    let count = (count as usize).min(entries.len());
    match entries[..count].iter_mut().find(|e| e.id == target_id) {
        Some(e) => e as *mut DataEntry,
        None => core::ptr::null_mut(),
    }
}

unsafe fn process_name(dest: &mut [i8], src: &[i8], _: i32) -> i32 {
    if dest.is_empty() || dest[0] == 0 {
        return -1;
    }
    strcpy(dest.as_mut_ptr(), src.as_ptr());
    let len: i32 = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(dest))
        .unwrap()
        .count_bytes() as i32;
    len
}

fn calculate_lookup(row: i32, col: i32, result: Option<&mut i32>) -> i32 {
    let temp: i32 = lookup_table[row as usize][col as usize];
    if temp != 0 {
        *result.unwrap() = temp * 2;
        return 1;
    }
    0
}

unsafe fn create_entries(count: i32, base_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }

    // Keep C allocation semantics (caller frees with free()).
    let bytes = (count as usize).wrapping_mul(core::mem::size_of::<DataEntry>());
    let raw = malloc(bytes) as *mut DataEntry;
    if raw.is_null() {
        return core::ptr::null_mut();
    }

    let entries = core::slice::from_raw_parts_mut(raw, count as usize);

    let mut temp_name: [i8; 32] = [0; 32];
    for (i, entry) in entries.iter_mut().enumerate() {
        let i = i as i32;
        entry.id = base_id + i;
        entry.value = (base_id + i) * 10;
        sprintf(
            temp_name.as_mut_ptr(),
            b"Entry_%d\0" as *const u8 as *const i8,
            base_id + i,
        );
        strcpy(entry.name.as_mut_ptr(), temp_name.as_ptr());
    }

    raw
}

unsafe fn modify_entries(entries: *mut DataEntry, count: i32, multiplier: i32) -> i32 {
    if entries.is_null() {
        return -1;
    }
    if count <= 0 {
        return 0;
    }

    let entries = core::slice::from_raw_parts_mut(entries, count as usize);

    let mut total: i32 = 0;
    for entry in entries.iter_mut() {
        let temp_value = entry.value;
        if temp_value != 0 {
            entry.value = temp_value * multiplier;
            total += entry.value;
        }
    }
    total
}

#[no_mangle]
pub unsafe extern "C" fn dataentry(mode: i32, param1: i32, param2: i32, param3: i32) -> i32 {
    let mut entries: &mut [DataEntry] = &mut [];
    let found: Option<&mut DataEntry>;
    let mut result: i32 = 0;
    let count: i32;
    let mut lookup_result: i32 = 0;
    let mut buffer: [i8; 32] = [0; 32];
    buffer[0] = 'T' as i32 as i8;
    buffer[1] = '\0' as i32 as i8;

    match mode {
        1 => {
            count = if param1 > 0 { param1 } else { 5 };
            entries = {
                let _x = create_entries(count, 100);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() || count == 0 {
                result = -1;
            } else {
                found = find_entry(entries, count, 100i32 + param2).as_mut();
                if found.is_none() || found.as_deref().unwrap().id == 0 {
                    result = -2;
                } else {
                    result = found.as_deref().unwrap().value;
                    strcpy(buffer.as_mut_ptr(), found.as_deref().unwrap().name.as_ptr());
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        2 => {
            count = if param1 > 0 { param1 } else { 3 };
            entries = {
                let _x = create_entries(count, 200);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() {
                result = -1;
            } else {
                result = modify_entries(entries.as_mut_ptr(), count, param2);
                if result != 0 {
                    result += param3;
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        3 => {
            if (0..4).contains(&param1) && (0..3).contains(&param2) {
                result = calculate_lookup(param1, param2, Some(&mut lookup_result));
                if result != 0 {
                    result = lookup_result + param3;
                }
            }
        }
        _ => {
            strcpy(buffer.as_mut_ptr(), b"Default\0" as *const u8 as *const i8);
            result = process_name(
                &mut buffer,
                bytemuck::cast_slice(b"TestName\0"),
                NAME_LENGTH,
            );
            count = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
                .unwrap()
                .count_bytes() as i32;
            if count != 0 {
                result = count * param1;
            }
        }
    }
    result
}
```

**Entity:** DataEntry allocation returned by create_entries() (used in dataentry mode 1/2)

**States:** Null, Allocated, Freed

**Transitions:**
- Null -> Allocated via create_entries(count>0) returning non-null
- Allocated -> Freed via free(ptr)
- Allocated -> Null via allocation failure (create_entries returns null)

**Evidence:** create_entries(): `if count <= 0 { return core::ptr::null_mut(); }` and `let raw = malloc(bytes) as *mut DataEntry; if raw.is_null() { return core::ptr::null_mut(); }`; create_entries(): comment `// Keep C allocation semantics (caller frees with free()).`; dataentry mode 1: `let _x = create_entries(count, 100); if _x.is_null() { &mut [] } else { from_raw_parts_mut(_x, count as usize) }` then later `free(entries.as_mut_ptr() as *mut _)`; dataentry mode 2: same pattern and `free(entries.as_mut_ptr() as *mut _)`; modify_entries(): precondition checks `if entries.is_null() { return -1; }` then constructs `from_raw_parts_mut(entries, count as usize)` assuming pointer+count are valid

**Implementation:** Introduce an owning wrapper `struct Entries { ptr: NonNull<DataEntry>, len: usize }` with `impl Drop for Entries { free(self.ptr.as_ptr() as *mut _) }`. Make `create_entries` return `Option<Entries>` (or `Result<Entries, AllocError>`). Provide safe accessors `as_mut_slice(&mut self) -> &mut [DataEntry]` so `count` cannot be mismatched and null cannot occur.

---

## Precondition Invariants

### 2. Index-range + output-pointer protocol for calculate_lookup (InRange+Some / Otherwise)

**Location**: `/data/test_case/lib.rs:1-198`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: calculate_lookup assumes `row` and `col` are within the fixed bounds of `lookup_table` (4x3) and that `result` is `Some(&mut i32)`; otherwise it panics or can index out of bounds. The only protection is that dataentry mode 3 checks the index ranges before calling, and passes `Some(&mut lookup_result)`. The type system does not encode the valid index ranges or the fact that the function requires an output slot when it will write.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataEntry, 3 free function(s)

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
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataEntry {
    pub id: i32,
    pub value: i32,
    pub name: [i8; 32],
}

pub const NAME_LENGTH: i32 = 32;

static lookup_table: [[i32; 3]; 4] = [
    [10, 20, 30],
    [40, 50, 60],
    [70, 80, 90],
    [100, 110, 120],
];

unsafe fn find_entry(entries: &mut [DataEntry], count: i32, target_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }
    let count = (count as usize).min(entries.len());
    match entries[..count].iter_mut().find(|e| e.id == target_id) {
        Some(e) => e as *mut DataEntry,
        None => core::ptr::null_mut(),
    }
}

unsafe fn process_name(dest: &mut [i8], src: &[i8], _: i32) -> i32 {
    if dest.is_empty() || dest[0] == 0 {
        return -1;
    }
    strcpy(dest.as_mut_ptr(), src.as_ptr());
    let len: i32 = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(dest))
        .unwrap()
        .count_bytes() as i32;
    len
}

fn calculate_lookup(row: i32, col: i32, result: Option<&mut i32>) -> i32 {
    let temp: i32 = lookup_table[row as usize][col as usize];
    if temp != 0 {
        *result.unwrap() = temp * 2;
        return 1;
    }
    0
}

unsafe fn create_entries(count: i32, base_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }

    // Keep C allocation semantics (caller frees with free()).
    let bytes = (count as usize).wrapping_mul(core::mem::size_of::<DataEntry>());
    let raw = malloc(bytes) as *mut DataEntry;
    if raw.is_null() {
        return core::ptr::null_mut();
    }

    let entries = core::slice::from_raw_parts_mut(raw, count as usize);

    let mut temp_name: [i8; 32] = [0; 32];
    for (i, entry) in entries.iter_mut().enumerate() {
        let i = i as i32;
        entry.id = base_id + i;
        entry.value = (base_id + i) * 10;
        sprintf(
            temp_name.as_mut_ptr(),
            b"Entry_%d\0" as *const u8 as *const i8,
            base_id + i,
        );
        strcpy(entry.name.as_mut_ptr(), temp_name.as_ptr());
    }

    raw
}

unsafe fn modify_entries(entries: *mut DataEntry, count: i32, multiplier: i32) -> i32 {
    if entries.is_null() {
        return -1;
    }
    if count <= 0 {
        return 0;
    }

    let entries = core::slice::from_raw_parts_mut(entries, count as usize);

    let mut total: i32 = 0;
    for entry in entries.iter_mut() {
        let temp_value = entry.value;
        if temp_value != 0 {
            entry.value = temp_value * multiplier;
            total += entry.value;
        }
    }
    total
}

#[no_mangle]
pub unsafe extern "C" fn dataentry(mode: i32, param1: i32, param2: i32, param3: i32) -> i32 {
    let mut entries: &mut [DataEntry] = &mut [];
    let found: Option<&mut DataEntry>;
    let mut result: i32 = 0;
    let count: i32;
    let mut lookup_result: i32 = 0;
    let mut buffer: [i8; 32] = [0; 32];
    buffer[0] = 'T' as i32 as i8;
    buffer[1] = '\0' as i32 as i8;

    match mode {
        1 => {
            count = if param1 > 0 { param1 } else { 5 };
            entries = {
                let _x = create_entries(count, 100);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() || count == 0 {
                result = -1;
            } else {
                found = find_entry(entries, count, 100i32 + param2).as_mut();
                if found.is_none() || found.as_deref().unwrap().id == 0 {
                    result = -2;
                } else {
                    result = found.as_deref().unwrap().value;
                    strcpy(buffer.as_mut_ptr(), found.as_deref().unwrap().name.as_ptr());
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        2 => {
            count = if param1 > 0 { param1 } else { 3 };
            entries = {
                let _x = create_entries(count, 200);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() {
                result = -1;
            } else {
                result = modify_entries(entries.as_mut_ptr(), count, param2);
                if result != 0 {
                    result += param3;
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        3 => {
            if (0..4).contains(&param1) && (0..3).contains(&param2) {
                result = calculate_lookup(param1, param2, Some(&mut lookup_result));
                if result != 0 {
                    result = lookup_result + param3;
                }
            }
        }
        _ => {
            strcpy(buffer.as_mut_ptr(), b"Default\0" as *const u8 as *const i8);
            result = process_name(
                &mut buffer,
                bytemuck::cast_slice(b"TestName\0"),
                NAME_LENGTH,
            );
            count = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
                .unwrap()
                .count_bytes() as i32;
            if count != 0 {
                result = count * param1;
            }
        }
    }
    result
}
```

**Entity:** lookup_table indexing via calculate_lookup(row, col, result)

**States:** ValidIndicesAndOutput, InvalidIndicesOrMissingOutput

**Transitions:**
- InvalidIndicesOrMissingOutput -> ValidIndicesAndOutput via caller ensuring (0..4).contains(row) && (0..3).contains(col) and passing Some(&mut out)

**Evidence:** calculate_lookup(): `let temp: i32 = lookup_table[row as usize][col as usize];` (unchecked conversion and indexing); calculate_lookup(): `*result.unwrap() = temp * 2;` requires `result` to be Some when `temp != 0`; dataentry mode 3: guards `if (0..4).contains(&param1) && (0..3).contains(&param2)` and calls `calculate_lookup(param1, param2, Some(&mut lookup_result))`

**Implementation:** Define bounded index types, e.g. `struct Row(u8); struct Col(u8);` with `TryFrom<i32>` validating ranges. Change signature to `fn calculate_lookup(row: Row, col: Col) -> Option<i32>` (return computed value instead of taking `Option<&mut i32>`), eliminating `unwrap()` and making invalid indices unrepresentable once converted.

---

## Protocol Invariants

### 3. C-string buffer validity protocol (NonEmpty+NulTerminated / Invalid) for strcpy + CStr parsing

**Location**: `/data/test_case/lib.rs:1-198`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: process_name relies on C-string conventions: `dest` must be a writable buffer that is intended to hold a NUL-terminated string, and `src` must point to a NUL-terminated C string. It also relies on `strcpy` not overflowing `dest` and on `dest` ending up NUL-terminated so `CStr::from_bytes_until_nul(...).unwrap()` will not panic. The current checks (`dest.is_empty()` and `dest[0] == 0`) do not ensure sufficient capacity or that `src` fits, and the NUL-termination invariant is only assumed.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct DataEntry, 3 free function(s)

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
    fn sprintf(__s: *mut i8, __format: *const i8, ...) -> i32;
    fn malloc(__size: usize) -> *mut core::ffi::c_void;
    fn free(__ptr: *mut core::ffi::c_void);
    fn strcpy(__dest: *mut i8, __src: *const i8) -> *mut i8;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataEntry {
    pub id: i32,
    pub value: i32,
    pub name: [i8; 32],
}

pub const NAME_LENGTH: i32 = 32;

static lookup_table: [[i32; 3]; 4] = [
    [10, 20, 30],
    [40, 50, 60],
    [70, 80, 90],
    [100, 110, 120],
];

unsafe fn find_entry(entries: &mut [DataEntry], count: i32, target_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }
    let count = (count as usize).min(entries.len());
    match entries[..count].iter_mut().find(|e| e.id == target_id) {
        Some(e) => e as *mut DataEntry,
        None => core::ptr::null_mut(),
    }
}

unsafe fn process_name(dest: &mut [i8], src: &[i8], _: i32) -> i32 {
    if dest.is_empty() || dest[0] == 0 {
        return -1;
    }
    strcpy(dest.as_mut_ptr(), src.as_ptr());
    let len: i32 = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(dest))
        .unwrap()
        .count_bytes() as i32;
    len
}

fn calculate_lookup(row: i32, col: i32, result: Option<&mut i32>) -> i32 {
    let temp: i32 = lookup_table[row as usize][col as usize];
    if temp != 0 {
        *result.unwrap() = temp * 2;
        return 1;
    }
    0
}

unsafe fn create_entries(count: i32, base_id: i32) -> *mut DataEntry {
    if count <= 0 {
        return core::ptr::null_mut();
    }

    // Keep C allocation semantics (caller frees with free()).
    let bytes = (count as usize).wrapping_mul(core::mem::size_of::<DataEntry>());
    let raw = malloc(bytes) as *mut DataEntry;
    if raw.is_null() {
        return core::ptr::null_mut();
    }

    let entries = core::slice::from_raw_parts_mut(raw, count as usize);

    let mut temp_name: [i8; 32] = [0; 32];
    for (i, entry) in entries.iter_mut().enumerate() {
        let i = i as i32;
        entry.id = base_id + i;
        entry.value = (base_id + i) * 10;
        sprintf(
            temp_name.as_mut_ptr(),
            b"Entry_%d\0" as *const u8 as *const i8,
            base_id + i,
        );
        strcpy(entry.name.as_mut_ptr(), temp_name.as_ptr());
    }

    raw
}

unsafe fn modify_entries(entries: *mut DataEntry, count: i32, multiplier: i32) -> i32 {
    if entries.is_null() {
        return -1;
    }
    if count <= 0 {
        return 0;
    }

    let entries = core::slice::from_raw_parts_mut(entries, count as usize);

    let mut total: i32 = 0;
    for entry in entries.iter_mut() {
        let temp_value = entry.value;
        if temp_value != 0 {
            entry.value = temp_value * multiplier;
            total += entry.value;
        }
    }
    total
}

#[no_mangle]
pub unsafe extern "C" fn dataentry(mode: i32, param1: i32, param2: i32, param3: i32) -> i32 {
    let mut entries: &mut [DataEntry] = &mut [];
    let found: Option<&mut DataEntry>;
    let mut result: i32 = 0;
    let count: i32;
    let mut lookup_result: i32 = 0;
    let mut buffer: [i8; 32] = [0; 32];
    buffer[0] = 'T' as i32 as i8;
    buffer[1] = '\0' as i32 as i8;

    match mode {
        1 => {
            count = if param1 > 0 { param1 } else { 5 };
            entries = {
                let _x = create_entries(count, 100);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() || count == 0 {
                result = -1;
            } else {
                found = find_entry(entries, count, 100i32 + param2).as_mut();
                if found.is_none() || found.as_deref().unwrap().id == 0 {
                    result = -2;
                } else {
                    result = found.as_deref().unwrap().value;
                    strcpy(buffer.as_mut_ptr(), found.as_deref().unwrap().name.as_ptr());
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        2 => {
            count = if param1 > 0 { param1 } else { 3 };
            entries = {
                let _x = create_entries(count, 200);
                if _x.is_null() {
                    &mut []
                } else {
                    std::slice::from_raw_parts_mut(_x, count as usize)
                }
            };
            if entries.is_empty() {
                result = -1;
            } else {
                result = modify_entries(entries.as_mut_ptr(), count, param2);
                if result != 0 {
                    result += param3;
                }
                free(entries.as_mut_ptr() as *mut _);
            }
        }
        3 => {
            if (0..4).contains(&param1) && (0..3).contains(&param2) {
                result = calculate_lookup(param1, param2, Some(&mut lookup_result));
                if result != 0 {
                    result = lookup_result + param3;
                }
            }
        }
        _ => {
            strcpy(buffer.as_mut_ptr(), b"Default\0" as *const u8 as *const i8);
            result = process_name(
                &mut buffer,
                bytemuck::cast_slice(b"TestName\0"),
                NAME_LENGTH,
            );
            count = std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(&buffer))
                .unwrap()
                .count_bytes() as i32;
            if count != 0 {
                result = count * param1;
            }
        }
    }
    result
}
```

**Entity:** process_name(dest, src, NAME_LENGTH) / C string buffers

**States:** ValidDestBuffer, InvalidDestBuffer

**Transitions:**
- InvalidDestBuffer -> ValidDestBuffer via caller providing a properly sized buffer and a NUL-terminated, fitting `src`

**Evidence:** process_name(): guard `if dest.is_empty() || dest[0] == 0 { return -1; }` encodes a runtime precondition on dest; process_name(): calls `strcpy(dest.as_mut_ptr(), src.as_ptr());` (requires src NUL-terminated and dest large enough); process_name(): `CStr::from_bytes_until_nul(bytemuck::cast_slice(dest)).unwrap()` will panic if no NUL is present in dest after copy; dataentry default branch: passes `&mut buffer` where buffer is `[i8; 32]` and `bytemuck::cast_slice(b"TestName\0")` as src, relying on C-string rules

**Implementation:** Use safe string types: represent dest as `&mut [u8; 32]` (fixed capacity) and src as `&CStr` (or `&[u8]` validated to contain NUL). Replace `strcpy` with a bounded copy that ensures NUL-termination (e.g., copy up to dest.len()-1 and write trailing 0). Wrap validated buffers in newtypes like `struct CBuf32([u8;32]);` and `struct NulTerminated<'a>(&'a CStr)` so invalid inputs are rejected at construction.

---

