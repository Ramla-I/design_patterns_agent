# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 1
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 1
- **Protocol**: 0
- **Modules analyzed**: 1

## Precondition Invariants

### 1. container_of pointer provenance protocol (field ref must originate from a live `test`)

**Location**: `/data/test_case/main.rs:1-58`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The `find_container_of_a`/`find_container_of_b` functions implement a C-style `container_of`: they compute a `*const test` by subtracting the field offset from a reference to an `i32`. This is only valid if the input reference actually points to the corresponding field (`test::a` or `test::b`) inside a properly-aligned, live `test` allocation. The type system does not encode that provenance relationship; any `&i32` can be passed, producing a pointer that may be invalid/unaligned/out-of-bounds. The unsafe dereference in `main` relies on this protocol being upheld.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct test, 2 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::env;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct test {
    pub a: i32,
    pub b: i32,
}

fn find_container_of_a(a_ref: &i32) -> *const test {
    // Equivalent to C's container_of(ptr_to_a, test, a)
    let a_ptr = std::ptr::from_ref(a_ref) as *const u8;
    let base = unsafe { a_ptr.sub(std::mem::offset_of!(test, a)) };
    base as *const test
}

fn find_container_of_b(b_ref: &i32) -> *const test {
    // Equivalent to C's container_of(ptr_to_b, test, b)
    let b_ptr = std::ptr::from_ref(b_ref) as *const u8;
    let base = unsafe { b_ptr.sub(std::mem::offset_of!(test, b)) };
    base as *const test
}

fn main() {
    let mut it = env::args();
    let _prog = it.next();

    let a: i32 = it
        .next()
        .expect("missing argv[1]")
        .parse()
        .expect("invalid integer argv[1]");
    let b: i32 = it
        .next()
        .expect("missing argv[2]")
        .parse()
        .expect("invalid integer argv[2]");

    // memset(&t, 0, sizeof(test)) is redundant in Rust; initialize to zero directly.
    let mut t = test { a: 0, b: 0 };
    t.a = a;
    t.b = b;

    let sum = unsafe { (*find_container_of_a(&t.a)).a + (*find_container_of_b(&t.b)).b };
    println!("{0}", sum);
}
```

**Entity:** test (and its fields a/b when passed to find_container_of_a/find_container_of_b)

**States:** ValidFieldRefFromLiveTest, InvalidOrForeignRef

**Transitions:**
- InvalidOrForeignRef -> ValidFieldRefFromLiveTest by constructing a `test` and borrowing `&t.a` / `&t.b`
- ValidFieldRefFromLiveTest -> InvalidOrForeignRef when the originating `test` is no longer live (dropped/moved) or if a ref not originating from `test` is passed

**Evidence:** fn find_container_of_a(a_ref: &i32) -> *const test: accepts any &i32 (no provenance tying it to `test::a`); let base = unsafe { a_ptr.sub(std::mem::offset_of!(test, a)) }; computes base pointer via raw pointer arithmetic; fn find_container_of_b(b_ref: &i32) -> *const test: same pattern for field `b`; main: `unsafe { (*find_container_of_a(&t.a)).a + (*find_container_of_b(&t.b)).b }` dereferences the computed pointers, assuming they are valid `test` pointers

**Implementation:** Replace `find_container_of_a(&i32)` with a safe API that cannot be called with arbitrary `&i32`, e.g. `fn container_from_a<'t>(a: &'t i32) -> &'t test` taking a branded/newtyped field reference like `struct AField<'t>(&'t test);` produced only by `impl test { fn a_field(&self) -> AField<'_> { ... } }`, or simply expose safe methods on `test` instead of reconstructing the container pointer.

---

