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

### 3. ima_info borrowed-pointer validity (Uninitialized/Invalid -> Initialized with live backing buffer)

**Location**: `/data/test_case/lib.rs:1-202`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: ima_info contains `blocks: *const ima_block` plus metadata fields that are only meaningful after a successful ima_parse call, and `blocks` is only valid as long as the original `data` buffer passed to ima_parse remains alive and correctly aligned. The code writes `info.blocks = blocks_ptr` computed from `data`, but the type system does not connect the lifetime of `info` to the lifetime of `data`, nor does it prevent using ima_info when parsing failed (partial/old values). This is a latent lifecycle/borrowing invariant enforced only by convention and return codes.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ima_block; struct ima_info, 1 free function(s); struct caf_audio_description; struct caf_packet_table; struct caf_chunk; struct caf_header; struct caf_data

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type ima_u32_t = u32;
pub type ima_u64_t = u64;
pub type ima_f64_t = f64;
pub type ima_u8_t = u8;
pub type ima_u16_t = u16;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ima_block {
    pub preamble: ima_u16_t,
    pub data: [ima_u8_t; 32],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ima_info {
    pub blocks: *const ima_block,
    pub size: ima_u64_t,
    pub sample_rate: ima_f64_t,
    pub frame_count: ima_u64_t,
    pub channel_count: ima_u32_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub f: ima_f64_t,
    pub u: ima_u64_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_audio_description {
    pub sample_rate: ima_f64_t,
    pub format_id: ima_u32_t,
    pub format_flags: ima_u32_t,
    pub bytes_per_packet: ima_u32_t,
    pub frames_per_packet: ima_u32_t,
    pub channels_per_frame: ima_u32_t,
    pub bits_per_channel: ima_u32_t,
}

pub type ima_s64_t = i64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_packet_table {
    pub packet_count: ima_s64_t,
    pub frame_count: ima_s64_t,
    pub priming_frames: ima_s32_t,
    pub remainder_frames: ima_s32_t,
}

pub type ima_s32_t = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_chunk {
    pub type_0: ima_u32_t,
    pub size: ima_s64_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_header {
    pub type_0: ima_u32_t,
    pub version: ima_u16_t,
    pub flags: ima_u16_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_data {
    pub edit_count: ima_u32_t,
}

#[inline]
fn ima_bswap16(v: ima_u16_t) -> ima_u16_t {
    v.swap_bytes()
}
#[inline]
fn ima_bswap32(v: ima_u32_t) -> ima_u32_t {
    v.swap_bytes()
}
#[inline]
fn ima_bswap64(v: ima_u64_t) -> ima_u64_t {
    v.swap_bytes()
}
#[inline]
fn ima_btoh16(v: ima_u16_t) -> ima_u16_t {
    ima_bswap16(v)
}
#[inline]
fn ima_btoh32(v: ima_u32_t) -> ima_u32_t {
    ima_bswap32(v)
}
#[inline]
fn ima_btoh64(v: ima_u64_t) -> ima_u64_t {
    ima_bswap64(v)
}

#[inline]
const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

#[no_mangle]
pub unsafe extern "C" fn ima_parse(
    mut info: Option<&mut ima_info>,
    data: *const core::ffi::c_void,
) -> i32 {
    let Some(info) = info.as_deref_mut() else {
        // Preserve original behavior (would have panicked); return a generic failure.
        return -1;
    };
    if data.is_null() {
        return -1;
    }

    // CAF header is fixed-size; avoid arbitrary huge slices.
    let header = &*(data as *const caf_header);

    if ima_btoh32(header.type_0) != fourcc(b'c', b'a', b'f', b'f') {
        return -1;
    }
    if ima_btoh16(header.version) != 1 {
        return -2;
    }

    let mut desc: Option<&caf_audio_description> = None;
    let mut pakt: Option<&caf_packet_table> = None;
    let mut blocks_ptr: *const ima_block = core::ptr::null();
    let mut data_chunk_size: ima_s64_t = 0;

    // Chunks start immediately after the header.
    let mut p = (data as *const u8).add(core::mem::size_of::<caf_header>());

    loop {
        let chunk = &*(p as *const caf_chunk);
        let chunk_type = ima_btoh32(chunk.type_0);
        let chunk_size = ima_btoh64(chunk.size as u64) as i64;

        // Payload begins right after caf_chunk header.
        let payload = p.add(core::mem::size_of::<caf_chunk>());

        match chunk_type {
            t if t == fourcc(b'd', b'e', b's', b'c') => {
                desc = Some(&*(payload as *const caf_audio_description));
            }
            t if t == fourcc(b'p', b'a', b'k', b't') => {
                pakt = Some(&*(payload as *const caf_packet_table));
            }
            t if t == fourcc(b'd', b'a', b't', b'a') => {
                // caf_data then audio blocks.
                let after_caf_data = payload.add(core::mem::size_of::<caf_data>());
                blocks_ptr = after_caf_data as *const ima_block;
                data_chunk_size = chunk_size;
                break;
            }
            _ => {}
        }

        // Advance to next chunk: chunk header + payload size.
        p = p.add(core::mem::size_of::<caf_chunk>() + chunk_size as usize);
    }

    let desc = match desc {
        Some(v) => v,
        None => return -3,
    };
    let pakt = match pakt {
        Some(v) => v,
        None => return -3,
    };

    if ima_btoh32(desc.format_id) != fourcc(b'i', b'm', b'a', b'4') {
        return -3;
    }

    info.blocks = blocks_ptr;
    info.size = data_chunk_size as u64;
    info.frame_count = ima_btoh64(pakt.frame_count as u64);
    info.channel_count = ima_btoh32(desc.channels_per_frame);

    let mut conv64 = C2RustUnnamed { f: desc.sample_rate };
    conv64.u = ima_btoh64(conv64.u);
    info.sample_rate = conv64.f;

    0
}
```

**Entity:** ima_info

**States:** UninitializedOrStale, InitializedBorrowingData

**Transitions:**
- UninitializedOrStale -> InitializedBorrowingData via successful `ima_parse` returning 0

**Evidence:** struct ima_info: `pub blocks: *const ima_block` (raw pointer requires external validity/lifetime invariants); ima_parse: `blocks_ptr = after_caf_data as *const ima_block;` (pointer derived from input buffer); ima_parse: `info.blocks = blocks_ptr;` only on success path (implies 'initialized only if return == 0'); ima_parse signature: `data: *const core::ffi::c_void` with no length/lifetime information (cannot ensure backing storage outlives `info`)

**Implementation:** Replace `ima_info` with a lifetime-carrying view type: `struct ImaInfo<'a> { blocks: &'a [ima_block], ... }` (or `&'a ima_block` plus a length). Expose `fn parse(data: &'a [u8]) -> Result<ImaInfo<'a>, Error>`. If you must keep the C ABI, provide a safe Rust wrapper that returns `ImaInfo<'a>` and keep the raw `ima_info` only for FFI.

---

## Precondition Invariants

### 1. ima_info FFI validity invariant (non-null blocks pointer + size/count coherence)

**Location**: `/data/test_case/lib.rs:1-12`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: ima_info is an FFI-facing, Copy raw-data struct whose fields implicitly must satisfy memory-safety and coherence rules: `blocks` must either be null (representing no blocks) or point to a readable array of `ima_block` values whose length is given by `size` (or some other implicit unit). Additionally, fields like `channel_count`, `sample_rate`, and `frame_count` are likely expected to be non-zero / in-range and mutually consistent with the underlying buffer. None of these constraints are enforced by the type system: `blocks` is a raw pointer with no lifetime or length attached, and the scalar fields are plain numeric types that can hold nonsensical values. Because the struct is `Copy`, invalid states can be trivially duplicated and outlive the data `blocks` points to, making the implicit protocol ("only use while backing storage is alive") unenforced at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ima_block; struct caf_audio_description; struct caf_packet_table; struct caf_chunk; struct caf_header; struct caf_data; 7 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct ima_info {
    pub blocks: *const ima_block,
    pub size: ima_u64_t,
    pub sample_rate: ima_f64_t,
    pub frame_count: ima_u64_t,
    pub channel_count: ima_u32_t,
}

```

**Entity:** ima_info

**States:** Invalid/Untrusted (raw FFI struct), Valid (blocks points to size elements and numeric fields consistent)

**Transitions:**
- Invalid/Untrusted -> Valid via explicit validation/constructor (not present in snippet)

**Evidence:** line 7: `pub blocks: *const ima_block` is a raw pointer with no lifetime/ownership/length; line 8: `pub size: ima_u64_t` suggests an implicit length/count associated with `blocks`; line 4: `#[repr(C)]` indicates FFI layout and thus external initialization/unsafeness expectations; line 5: `#[derive(Copy, Clone)]` allows duplicating the pointer+metadata without any lifetime tracking

**Implementation:** Introduce a validated wrapper type, e.g. `struct ImaInfo<'a> { blocks: &'a [ima_block], sample_rate: NonZeroU32/FiniteF64, frame_count: u64, channel_count: NonZeroU32 }` (or keep f64 but validate `is_finite()`); provide `unsafe fn from_raw(raw: ima_info) -> Result<ImaInfo<'a>, Error>` that checks null/size coherence and range constraints, converting `blocks + size` into a slice via `slice::from_raw_parts`. Keep the raw `ima_info` for FFI boundaries only.

---

## Protocol Invariants

### 2. CAF parsing protocol producing a valid ima_info (Invalid -> ParsedCAF/IMA4)

**Location**: `/data/test_case/lib.rs:1-202`

**Confidence**: high

**Suggested Pattern**: builder

**Description**: ima_parse implements a multi-step parsing protocol over a raw pointer: it first requires non-null inputs, then validates the CAF header (magic + version), then scans chunks until it finds required 'desc' and 'pakt' and a terminating 'data' chunk, then validates the audio format_id is 'ima4', and only then writes output fields into ima_info. These states are encoded via runtime checks (-1/-2/-3), local Option<&...> variables (desc/pakt), and breaking out of the loop on the 'data' chunk. The type system does not prevent calling ima_parse with null/invalid pointers, does not tie the output ima_info lifetime to the backing data buffer, and does not enforce that all required chunks were found before populating info.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ima_block; struct ima_info, 1 free function(s); struct caf_audio_description; struct caf_packet_table; struct caf_chunk; struct caf_header; struct caf_data

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

pub type ima_u32_t = u32;
pub type ima_u64_t = u64;
pub type ima_f64_t = f64;
pub type ima_u8_t = u8;
pub type ima_u16_t = u16;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ima_block {
    pub preamble: ima_u16_t,
    pub data: [ima_u8_t; 32],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ima_info {
    pub blocks: *const ima_block,
    pub size: ima_u64_t,
    pub sample_rate: ima_f64_t,
    pub frame_count: ima_u64_t,
    pub channel_count: ima_u32_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union C2RustUnnamed {
    pub f: ima_f64_t,
    pub u: ima_u64_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_audio_description {
    pub sample_rate: ima_f64_t,
    pub format_id: ima_u32_t,
    pub format_flags: ima_u32_t,
    pub bytes_per_packet: ima_u32_t,
    pub frames_per_packet: ima_u32_t,
    pub channels_per_frame: ima_u32_t,
    pub bits_per_channel: ima_u32_t,
}

pub type ima_s64_t = i64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_packet_table {
    pub packet_count: ima_s64_t,
    pub frame_count: ima_s64_t,
    pub priming_frames: ima_s32_t,
    pub remainder_frames: ima_s32_t,
}

pub type ima_s32_t = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_chunk {
    pub type_0: ima_u32_t,
    pub size: ima_s64_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_header {
    pub type_0: ima_u32_t,
    pub version: ima_u16_t,
    pub flags: ima_u16_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct caf_data {
    pub edit_count: ima_u32_t,
}

#[inline]
fn ima_bswap16(v: ima_u16_t) -> ima_u16_t {
    v.swap_bytes()
}
#[inline]
fn ima_bswap32(v: ima_u32_t) -> ima_u32_t {
    v.swap_bytes()
}
#[inline]
fn ima_bswap64(v: ima_u64_t) -> ima_u64_t {
    v.swap_bytes()
}
#[inline]
fn ima_btoh16(v: ima_u16_t) -> ima_u16_t {
    ima_bswap16(v)
}
#[inline]
fn ima_btoh32(v: ima_u32_t) -> ima_u32_t {
    ima_bswap32(v)
}
#[inline]
fn ima_btoh64(v: ima_u64_t) -> ima_u64_t {
    ima_bswap64(v)
}

#[inline]
const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

#[no_mangle]
pub unsafe extern "C" fn ima_parse(
    mut info: Option<&mut ima_info>,
    data: *const core::ffi::c_void,
) -> i32 {
    let Some(info) = info.as_deref_mut() else {
        // Preserve original behavior (would have panicked); return a generic failure.
        return -1;
    };
    if data.is_null() {
        return -1;
    }

    // CAF header is fixed-size; avoid arbitrary huge slices.
    let header = &*(data as *const caf_header);

    if ima_btoh32(header.type_0) != fourcc(b'c', b'a', b'f', b'f') {
        return -1;
    }
    if ima_btoh16(header.version) != 1 {
        return -2;
    }

    let mut desc: Option<&caf_audio_description> = None;
    let mut pakt: Option<&caf_packet_table> = None;
    let mut blocks_ptr: *const ima_block = core::ptr::null();
    let mut data_chunk_size: ima_s64_t = 0;

    // Chunks start immediately after the header.
    let mut p = (data as *const u8).add(core::mem::size_of::<caf_header>());

    loop {
        let chunk = &*(p as *const caf_chunk);
        let chunk_type = ima_btoh32(chunk.type_0);
        let chunk_size = ima_btoh64(chunk.size as u64) as i64;

        // Payload begins right after caf_chunk header.
        let payload = p.add(core::mem::size_of::<caf_chunk>());

        match chunk_type {
            t if t == fourcc(b'd', b'e', b's', b'c') => {
                desc = Some(&*(payload as *const caf_audio_description));
            }
            t if t == fourcc(b'p', b'a', b'k', b't') => {
                pakt = Some(&*(payload as *const caf_packet_table));
            }
            t if t == fourcc(b'd', b'a', b't', b'a') => {
                // caf_data then audio blocks.
                let after_caf_data = payload.add(core::mem::size_of::<caf_data>());
                blocks_ptr = after_caf_data as *const ima_block;
                data_chunk_size = chunk_size;
                break;
            }
            _ => {}
        }

        // Advance to next chunk: chunk header + payload size.
        p = p.add(core::mem::size_of::<caf_chunk>() + chunk_size as usize);
    }

    let desc = match desc {
        Some(v) => v,
        None => return -3,
    };
    let pakt = match pakt {
        Some(v) => v,
        None => return -3,
    };

    if ima_btoh32(desc.format_id) != fourcc(b'i', b'm', b'a', b'4') {
        return -3;
    }

    info.blocks = blocks_ptr;
    info.size = data_chunk_size as u64;
    info.frame_count = ima_btoh64(pakt.frame_count as u64);
    info.channel_count = ima_btoh32(desc.channels_per_frame);

    let mut conv64 = C2RustUnnamed { f: desc.sample_rate };
    conv64.u = ima_btoh64(conv64.u);
    info.sample_rate = conv64.f;

    0
}
```

**Entity:** ima_parse (and output struct ima_info)

**States:** InvalidInput, HeaderValidated, ChunksScanning, ReadyToFillInfo, ParsedInfoReady

**Transitions:**
- InvalidInput -> HeaderValidated via non-null checks + caf_header magic/version checks
- HeaderValidated -> ChunksScanning via starting at data + size_of::<caf_header>()
- ChunksScanning -> ReadyToFillInfo via encountering 'data' chunk (break) and having desc/pakt Some(...)
- ReadyToFillInfo -> ParsedInfoReady via format_id == 'ima4' and writing to info.{blocks,size,frame_count,channel_count,sample_rate}

**Evidence:** ima_parse: `if data.is_null() { return -1; }` (non-null precondition on raw input); ima_parse: `let header = &*(data as *const caf_header);` (assumes `data` points to at least a caf_header and is properly aligned); ima_parse: `if ima_btoh32(header.type_0) != fourcc(b'c', b'a', b'f', b'f') { return -1; }` (CAF magic gating later steps); ima_parse: `if ima_btoh16(header.version) != 1 { return -2; }` (version-gated protocol); ima_parse: `let mut desc: Option<&caf_audio_description> = None; let mut pakt: Option<&caf_packet_table> = None;` (runtime-tracked required chunks); ima_parse: loop uses `match chunk_type { ... desc = Some(...); pakt = Some(...); ... break; }` (temporal ordering: must scan chunks, break only on 'data'); ima_parse: `let desc = match desc { Some(v) => v, None => return -3 };` and same for pakt (invariant: desc/pakt must exist before filling info); ima_parse: `if ima_btoh32(desc.format_id) != fourcc(b'i', b'm', b'a', b'4') { return -3; }` (format gating for output validity); ima_parse: `info.blocks = blocks_ptr; info.size = data_chunk_size as u64; ...` (only written after all checks)

**Implementation:** Introduce a safe parser API that consumes a byte slice: `fn parse(data: &[u8]) -> Result<ParsedCaf<'_>, Error>`. Internally use a small builder/accumulator struct (e.g., `struct CafParts { desc: Option<...>, pakt: Option<...>, data: Option<...> }`) and expose `ParsedCaf` only once all mandatory parts are present and validated. Then provide `impl ParsedCaf<'a> { fn into_ima_info(&self) -> ImaInfoRef<'a> }` (or return the final typed result directly). This removes the need for `Option<&mut ima_info>` and makes the required ordering explicit in types.

---

