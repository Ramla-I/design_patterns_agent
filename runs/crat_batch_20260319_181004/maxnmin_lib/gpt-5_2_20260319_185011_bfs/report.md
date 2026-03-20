# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 2
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 2

## State Machine Invariants

### 1. Node registry initialization + capacity/length protocol

**Location**: `/data/test_case/lib.rs:1-182`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The module maintains a per-thread global registry of nodes in `node_storage` with the number of valid entries tracked separately by `node_count`. Valid operations implicitly require that (1) only indices `< node_count` are considered initialized/valid, and (2) `node_count` never exceeds `MAX_NODES`. This is enforced by runtime checks (`if count >= MAX_NODES`) and by slicing (`storage[..count]`). Callers also implicitly rely on resetting `node_count` to 0 to "reinitialize" the registry before repopulating (as done in `maxnmin`). None of this is captured in the type system: any code can mutate `node_count` independently of `node_storage`, and `add_node` returns `-1` on overflow rather than making overflow unrepresentable.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, 1 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub id: i32,
    pub parent_id: i32,
    pub name: [i8; 50],
    pub value: f64,
    pub active: i32,
}

pub const MAX_NODES: i32 = 100;
pub const MAX_NAME_LEN: i32 = 50;

thread_local! {
    static node_storage: std::cell::RefCell<[Node; 100]> = const {
        std::cell::RefCell::new([Node { id: 0, parent_id: 0, name: [0; 50], value: 0., active: 0 }; 100])
    };
}

thread_local! {
    static node_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

#[inline]
fn current_count() -> usize {
    node_count.get().max(0) as usize
}

pub(crate) fn add_node(id: i32, parent_id: i32, name: &[i8], value: f64) -> i32 {
    let count = node_count.get();
    if count >= MAX_NODES {
        return -1;
    }

    let mut new_node = Node {
        id,
        parent_id,
        name: [0; 50],
        value,
        active: 1,
    };

    // Copy up to MAX_NAME_LEN-1 bytes, then ensure NUL termination.
    let max_copy = (MAX_NAME_LEN as usize).saturating_sub(1);
    let copy_len = name.len().min(max_copy);
    new_node.name[..copy_len].copy_from_slice(&name[..copy_len]);
    new_node.name[max_copy] = 0;

    node_storage.with_borrow_mut(|storage| storage[count as usize] = new_node);
    node_count.set(count + 1);
    count
}

pub(crate) fn find_node_by_id(id: i32) -> *const Node {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        if let Some((idx, _)) = storage[..count]
            .iter()
            .enumerate()
            .find(|(_, n)| n.id == id && n.active != 0)
        {
            // Preserve original behavior: return pointer to the element via a subslice's as_ptr().
            storage[idx..].as_ptr()
        } else {
            std::ptr::null()
        }
    })
}

pub(crate) fn get_children_count(parent_id: i32) -> i32 {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == parent_id && n.active != 0)
            .count() as i32
    })
}

pub(crate) unsafe fn calculate_subtree_sum(node_id: i32) -> f64 {
    let node = find_node_by_id(node_id).as_ref();
    let Some(node) = node else { return 0.0 };

    let mut sum = node.value;

    let count = current_count();
    // Collect child ids first to avoid borrowing issues across recursion.
    let child_ids: Vec<i32> = node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == node_id && n.active != 0)
            .map(|n| n.id)
            .collect()
    });

    for child_id in child_ids {
        sum += calculate_subtree_sum(child_id);
    }

    sum
}

pub(crate) fn process_string(mut str: &[i8]) -> i32 {
    let mut result: i32 = 0;
    while let Some((&ch, rest)) = str.split_first() {
        if ch == 0 {
            break;
        }
        result += ch as i32;
        str = rest;
    }
    result
}

pub(crate) fn safe_double_to_int(d: f64) -> i32 {
    if d.is_nan() {
        return 0;
    }
    if d > INT_MAX as f64 {
        return INT_MAX;
    }
    if d < INT_MIN as f64 {
        return INT_MIN;
    }
    d as i32
}

#[no_mangle]
pub unsafe extern "C" fn maxnmin(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    node_count.set(0);
    add_node(1, -1, bytemuck::cast_slice(b"root\0"), 10.5f64);
    add_node(2, 1, bytemuck::cast_slice(b"child1\0"), 20.7f64);
    add_node(3, 1, bytemuck::cast_slice(b"child2\0"), 15.3f64);
    add_node(4, 2, bytemuck::cast_slice(b"grandchild1\0"), 5.9f64);
    add_node(5, 2, bytemuck::cast_slice(b"grandchild2\0"), 8.2f64);
    add_node(6, 3, bytemuck::cast_slice(b"grandchild3\0"), 12.4f64);

    let node_id: i32 = param1 % 6 + 1;
    if let Some(selected_node) = find_node_by_id(node_id).as_ref() {
        let name_ptr: &[i8] = &selected_node.name;
        if name_ptr[0] != 0 {
            result += process_string(name_ptr);
        }
        let subtree_sum: f64 = calculate_subtree_sum(node_id);
        result += safe_double_to_int(subtree_sum);
    }

    let second_node_id: i32 = param2 % 6 + 1;
    if let Some(second_node) = find_node_by_id(second_node_id).as_ref() {
        let value_multiplied: f64 = second_node.value * param3 as f64;
        result += safe_double_to_int(value_multiplied);
    }

    let parent_id: i32 = param4 % 3 + 1;
    let children: i32 = get_children_count(parent_id);
    result += children * 10;

    let mut calculation: f64 = (param1 + param2) as f64 / (param3 + 1) as f64;
    calculation *= param4 as f64;
    result += safe_double_to_int(calculation);

    result
}

pub const INT_MAX: i32 = __INT_MAX__;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const __INT_MAX__: i32 = 2147483647;
```

**Entity:** thread_local node_storage / node_count (global node registry)

**States:** Empty (count = 0), PartiallyFilled (0 < count < MAX_NODES), Full (count = MAX_NODES)

**Transitions:**
- Empty -> PartiallyFilled via add_node() (when node_count increases from 0)
- PartiallyFilled -> Full via repeated add_node() until node_count == MAX_NODES
- Any -> Empty via node_count.set(0) (reset/reinitialize in maxnmin)

**Evidence:** thread_local! static node_storage: RefCell<[Node; 100]> (fixed-capacity backing storage); thread_local! static node_count: Cell<i32> (separate runtime length/state field); fn current_count(): node_count.get().max(0) as usize (runtime sanitization of count); add_node(): `if count >= MAX_NODES { return -1; }` (capacity precondition enforced at runtime); find_node_by_id()/get_children_count()/calculate_subtree_sum(): `storage[..count]` (only first `count` entries are valid); maxnmin(): `node_count.set(0);` before a sequence of add_node() calls (implicit initialization/reset protocol)

**Implementation:** Encapsulate the registry in a non-global `NodeRegistry<S>` where `S` tracks whether it is `Empty`/`NonEmpty`, and store the length as `usize` tied to the array. Provide `fn new() -> NodeRegistry<Empty>`, `fn add_node(&mut self, ...) -> Result<NodeId, Full>`, and iteration/search only over the internal slice `&storage[..len]`. If a reset is needed, expose `fn clear(&mut self)` rather than direct access to `node_count`.

---

### 2. Node liveness + C-string name validity invariants (Active/Inactive, NUL-terminated)

**Location**: `/data/test_case/lib.rs:1-182`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Each `Node` has implicit validity constraints: (1) liveness is represented by `active: i32` and most queries treat nodes with `active == 0` as nonexistent, and (2) `name` is treated as a C string that must contain a terminating NUL (and be safe to scan until NUL). These invariants are maintained manually: `add_node` forces `active = 1` and writes `name[max_copy] = 0`, and readers gate on `n.active != 0` and stop processing at `ch == 0`. The type system does not distinguish active vs inactive nodes, nor does it guarantee that `name` is NUL-terminated (other code could write arbitrary bytes into `name`).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, 1 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub id: i32,
    pub parent_id: i32,
    pub name: [i8; 50],
    pub value: f64,
    pub active: i32,
}

pub const MAX_NODES: i32 = 100;
pub const MAX_NAME_LEN: i32 = 50;

thread_local! {
    static node_storage: std::cell::RefCell<[Node; 100]> = const {
        std::cell::RefCell::new([Node { id: 0, parent_id: 0, name: [0; 50], value: 0., active: 0 }; 100])
    };
}

thread_local! {
    static node_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

#[inline]
fn current_count() -> usize {
    node_count.get().max(0) as usize
}

pub(crate) fn add_node(id: i32, parent_id: i32, name: &[i8], value: f64) -> i32 {
    let count = node_count.get();
    if count >= MAX_NODES {
        return -1;
    }

    let mut new_node = Node {
        id,
        parent_id,
        name: [0; 50],
        value,
        active: 1,
    };

    // Copy up to MAX_NAME_LEN-1 bytes, then ensure NUL termination.
    let max_copy = (MAX_NAME_LEN as usize).saturating_sub(1);
    let copy_len = name.len().min(max_copy);
    new_node.name[..copy_len].copy_from_slice(&name[..copy_len]);
    new_node.name[max_copy] = 0;

    node_storage.with_borrow_mut(|storage| storage[count as usize] = new_node);
    node_count.set(count + 1);
    count
}

pub(crate) fn find_node_by_id(id: i32) -> *const Node {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        if let Some((idx, _)) = storage[..count]
            .iter()
            .enumerate()
            .find(|(_, n)| n.id == id && n.active != 0)
        {
            // Preserve original behavior: return pointer to the element via a subslice's as_ptr().
            storage[idx..].as_ptr()
        } else {
            std::ptr::null()
        }
    })
}

pub(crate) fn get_children_count(parent_id: i32) -> i32 {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == parent_id && n.active != 0)
            .count() as i32
    })
}

pub(crate) unsafe fn calculate_subtree_sum(node_id: i32) -> f64 {
    let node = find_node_by_id(node_id).as_ref();
    let Some(node) = node else { return 0.0 };

    let mut sum = node.value;

    let count = current_count();
    // Collect child ids first to avoid borrowing issues across recursion.
    let child_ids: Vec<i32> = node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == node_id && n.active != 0)
            .map(|n| n.id)
            .collect()
    });

    for child_id in child_ids {
        sum += calculate_subtree_sum(child_id);
    }

    sum
}

pub(crate) fn process_string(mut str: &[i8]) -> i32 {
    let mut result: i32 = 0;
    while let Some((&ch, rest)) = str.split_first() {
        if ch == 0 {
            break;
        }
        result += ch as i32;
        str = rest;
    }
    result
}

pub(crate) fn safe_double_to_int(d: f64) -> i32 {
    if d.is_nan() {
        return 0;
    }
    if d > INT_MAX as f64 {
        return INT_MAX;
    }
    if d < INT_MIN as f64 {
        return INT_MIN;
    }
    d as i32
}

#[no_mangle]
pub unsafe extern "C" fn maxnmin(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    node_count.set(0);
    add_node(1, -1, bytemuck::cast_slice(b"root\0"), 10.5f64);
    add_node(2, 1, bytemuck::cast_slice(b"child1\0"), 20.7f64);
    add_node(3, 1, bytemuck::cast_slice(b"child2\0"), 15.3f64);
    add_node(4, 2, bytemuck::cast_slice(b"grandchild1\0"), 5.9f64);
    add_node(5, 2, bytemuck::cast_slice(b"grandchild2\0"), 8.2f64);
    add_node(6, 3, bytemuck::cast_slice(b"grandchild3\0"), 12.4f64);

    let node_id: i32 = param1 % 6 + 1;
    if let Some(selected_node) = find_node_by_id(node_id).as_ref() {
        let name_ptr: &[i8] = &selected_node.name;
        if name_ptr[0] != 0 {
            result += process_string(name_ptr);
        }
        let subtree_sum: f64 = calculate_subtree_sum(node_id);
        result += safe_double_to_int(subtree_sum);
    }

    let second_node_id: i32 = param2 % 6 + 1;
    if let Some(second_node) = find_node_by_id(second_node_id).as_ref() {
        let value_multiplied: f64 = second_node.value * param3 as f64;
        result += safe_double_to_int(value_multiplied);
    }

    let parent_id: i32 = param4 % 3 + 1;
    let children: i32 = get_children_count(parent_id);
    result += children * 10;

    let mut calculation: f64 = (param1 + param2) as f64 / (param3 + 1) as f64;
    calculation *= param4 as f64;
    result += safe_double_to_int(calculation);

    result
}

pub const INT_MAX: i32 = __INT_MAX__;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const __INT_MAX__: i32 = 2147483647;
```

**Entity:** Node (as stored in node_storage)

**States:** Inactive (active == 0), Active (active != 0)

**Transitions:**
- Inactive -> Active via add_node() setting `active: 1` (initialization into storage)

**Evidence:** struct Node { ... name: [i8; 50], ... active: i32 } (runtime-encoded liveness + raw byte name buffer); add_node(): `active: 1` when constructing new_node (activation on insert); add_node(): comment `// Copy up to MAX_NAME_LEN-1 bytes, then ensure NUL termination.` and `new_node.name[max_copy] = 0;` (manual C-string invariant); find_node_by_id(): predicate `n.id == id && n.active != 0` (active gates visibility); get_children_count(): filter `n.parent_id == parent_id && n.active != 0` (active gates counting); calculate_subtree_sum(): child selection `n.parent_id == node_id && n.active != 0` (active gates recursion); process_string(): stops at `if ch == 0 { break; }` (assumes NUL-terminated input)

**Implementation:** Replace `active: i32` with an `enum ActiveState { Active, Inactive }` or split storage into `Option<NodeData>` to make absence explicit. Wrap `name` in a `CName([i8; 50])` newtype that can only be constructed through a function ensuring NUL termination (or use `std::ffi::CStr`/`CString` where possible). Expose only safe accessors returning `&CStr` (or a validated slice) rather than raw `[i8; 50]`.

---

## Precondition Invariants

### 4. Node field validity invariants (C-FFI layout: name string + active flag + ID relationships)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: `Node` is a `#[repr(C)]` POD-like struct intended for C interop. Several fields likely have implicit validity requirements that are not enforced by the Rust type system: (1) `name: [i8; 50]` is presumably a C string that should be NUL-terminated and contain valid bytes; (2) `active: i32` is likely used as a boolean/enum-like flag (e.g., 0/1) but any `i32` is accepted; (3) `id`/`parent_id` likely have relational constraints (e.g., parent_id is -1/0 for root or must refer to another node) but are unconstrained integers. Because the type is `Copy, Clone`, invalid instances are easy to duplicate and propagate without checks.

**Evidence**:

```rust
// Note: Other parts of this module contain: 7 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub id: i32,
    pub parent_id: i32,
    pub name: [i8; 50],
    pub value: f64,
    pub active: i32,
}

```

**Entity:** Node

**States:** Valid, Invalid

**Evidence:** line 5: `#[repr(C)]` suggests FFI / C layout expectations rather than Rust-enforced invariants; line 10: `pub name: [i8; 50]` indicates an inline C-string buffer; NUL-termination/valid encoding is not represented in the type; line 12: `pub active: i32` is a flag-like field encoded as an integer (latent boolean/enum domain); line 8-9: `pub id: i32`, `pub parent_id: i32` are raw identifiers with likely relational constraints not enforced by types; line 6: `#[derive(Copy, Clone)]` makes it easy to copy potentially-invalid states without validation

**Implementation:** Introduce validated wrappers for the latent domains, e.g. `struct NodeId(i32); struct ParentId(Option<NodeId>); struct Active(bool)` or `#[repr(i32)] enum Active { Inactive=0, Active=1 }`. Replace `[i8; 50]` with a dedicated `NameBuf` newtype that enforces NUL-termination and provides safe accessors (and a separate `NodeFFI` repr(C) struct if needed for FFI boundaries). Provide `TryFrom<NodeFFI> for Node` to validate at the boundary.

---

## Protocol Invariants

### 3. Borrow-to-pointer escape protocol (pointer valid only while storage is not mutably borrowed/modified)

**Location**: `/data/test_case/lib.rs:1-182`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `find_node_by_id` returns a raw pointer into the thread-local `node_storage` backing array. This encodes a hidden protocol: the returned pointer is only valid as long as the underlying `node_storage` is not mutably borrowed (and, more generally, while the storage remains alive and not reentered in a conflicting way). The type system cannot express this because the pointer is not tied to a lifetime; callers must manually avoid holding the pointer across operations that could mutably borrow `node_storage`. This is partially mitigated in `calculate_subtree_sum` by collecting `child_ids` first "to avoid borrowing issues across recursion", which is evidence of a temporal/aliasing protocol being maintained by convention rather than types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, 1 free function(s)

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Node {
    pub id: i32,
    pub parent_id: i32,
    pub name: [i8; 50],
    pub value: f64,
    pub active: i32,
}

pub const MAX_NODES: i32 = 100;
pub const MAX_NAME_LEN: i32 = 50;

thread_local! {
    static node_storage: std::cell::RefCell<[Node; 100]> = const {
        std::cell::RefCell::new([Node { id: 0, parent_id: 0, name: [0; 50], value: 0., active: 0 }; 100])
    };
}

thread_local! {
    static node_count: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

#[inline]
fn current_count() -> usize {
    node_count.get().max(0) as usize
}

pub(crate) fn add_node(id: i32, parent_id: i32, name: &[i8], value: f64) -> i32 {
    let count = node_count.get();
    if count >= MAX_NODES {
        return -1;
    }

    let mut new_node = Node {
        id,
        parent_id,
        name: [0; 50],
        value,
        active: 1,
    };

    // Copy up to MAX_NAME_LEN-1 bytes, then ensure NUL termination.
    let max_copy = (MAX_NAME_LEN as usize).saturating_sub(1);
    let copy_len = name.len().min(max_copy);
    new_node.name[..copy_len].copy_from_slice(&name[..copy_len]);
    new_node.name[max_copy] = 0;

    node_storage.with_borrow_mut(|storage| storage[count as usize] = new_node);
    node_count.set(count + 1);
    count
}

pub(crate) fn find_node_by_id(id: i32) -> *const Node {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        if let Some((idx, _)) = storage[..count]
            .iter()
            .enumerate()
            .find(|(_, n)| n.id == id && n.active != 0)
        {
            // Preserve original behavior: return pointer to the element via a subslice's as_ptr().
            storage[idx..].as_ptr()
        } else {
            std::ptr::null()
        }
    })
}

pub(crate) fn get_children_count(parent_id: i32) -> i32 {
    let count = current_count();
    node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == parent_id && n.active != 0)
            .count() as i32
    })
}

pub(crate) unsafe fn calculate_subtree_sum(node_id: i32) -> f64 {
    let node = find_node_by_id(node_id).as_ref();
    let Some(node) = node else { return 0.0 };

    let mut sum = node.value;

    let count = current_count();
    // Collect child ids first to avoid borrowing issues across recursion.
    let child_ids: Vec<i32> = node_storage.with_borrow(|storage| {
        storage[..count]
            .iter()
            .filter(|n| n.parent_id == node_id && n.active != 0)
            .map(|n| n.id)
            .collect()
    });

    for child_id in child_ids {
        sum += calculate_subtree_sum(child_id);
    }

    sum
}

pub(crate) fn process_string(mut str: &[i8]) -> i32 {
    let mut result: i32 = 0;
    while let Some((&ch, rest)) = str.split_first() {
        if ch == 0 {
            break;
        }
        result += ch as i32;
        str = rest;
    }
    result
}

pub(crate) fn safe_double_to_int(d: f64) -> i32 {
    if d.is_nan() {
        return 0;
    }
    if d > INT_MAX as f64 {
        return INT_MAX;
    }
    if d < INT_MIN as f64 {
        return INT_MIN;
    }
    d as i32
}

#[no_mangle]
pub unsafe extern "C" fn maxnmin(param1: i32, param2: i32, param3: i32, param4: i32) -> i32 {
    let mut result: i32 = 0;

    node_count.set(0);
    add_node(1, -1, bytemuck::cast_slice(b"root\0"), 10.5f64);
    add_node(2, 1, bytemuck::cast_slice(b"child1\0"), 20.7f64);
    add_node(3, 1, bytemuck::cast_slice(b"child2\0"), 15.3f64);
    add_node(4, 2, bytemuck::cast_slice(b"grandchild1\0"), 5.9f64);
    add_node(5, 2, bytemuck::cast_slice(b"grandchild2\0"), 8.2f64);
    add_node(6, 3, bytemuck::cast_slice(b"grandchild3\0"), 12.4f64);

    let node_id: i32 = param1 % 6 + 1;
    if let Some(selected_node) = find_node_by_id(node_id).as_ref() {
        let name_ptr: &[i8] = &selected_node.name;
        if name_ptr[0] != 0 {
            result += process_string(name_ptr);
        }
        let subtree_sum: f64 = calculate_subtree_sum(node_id);
        result += safe_double_to_int(subtree_sum);
    }

    let second_node_id: i32 = param2 % 6 + 1;
    if let Some(second_node) = find_node_by_id(second_node_id).as_ref() {
        let value_multiplied: f64 = second_node.value * param3 as f64;
        result += safe_double_to_int(value_multiplied);
    }

    let parent_id: i32 = param4 % 3 + 1;
    let children: i32 = get_children_count(parent_id);
    result += children * 10;

    let mut calculation: f64 = (param1 + param2) as f64 / (param3 + 1) as f64;
    calculation *= param4 as f64;
    result += safe_double_to_int(calculation);

    result
}

pub const INT_MAX: i32 = __INT_MAX__;
pub const INT_MIN: i32 = -__INT_MAX__ - 1;
pub const __INT_MAX__: i32 = 2147483647;
```

**Entity:** find_node_by_id / calculate_subtree_sum pointer-based access

**States:** NoOutstandingNodePtr, OutstandingNodePtr (derived from thread-local storage)

**Transitions:**
- NoOutstandingNodePtr -> OutstandingNodePtr via find_node_by_id() returning `*const Node`
- OutstandingNodePtr -> NoOutstandingNodePtr when caller stops using the raw pointer (implicit)

**Evidence:** find_node_by_id(): return type `*const Node` (raw pointer escapes borrow checking); find_node_by_id(): comment `return pointer to the element via a subslice's as_ptr()` (explicit pointer-escape behavior); calculate_subtree_sum(): `let node = find_node_by_id(node_id).as_ref();` (dereference requires caller-side discipline/`unsafe` context); calculate_subtree_sum(): comment `// Collect child ids first to avoid borrowing issues across recursion.` (manual protocol to avoid aliasing across recursive calls); calculate_subtree_sum() is `unsafe fn` (signals untracked safety preconditions around pointer usage)

**Implementation:** Instead of returning `*const Node`, return an index/handle `NodeId(usize)` (capability) that is validated against the registry length, and provide accessor methods on a `NodeRegistry` borrow: `fn get(&self, id: NodeId) -> Option<&Node>`. This ties references to the borrow lifetime and prevents mutable reborrows while references exist, eliminating the need for raw pointers and the `unsafe` recursion pattern.

---

