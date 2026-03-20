# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 1
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 2
- **Protocol**: 2
- **Modules analyzed**: 1

## Temporal Ordering Invariants

### 4. Analyzer initialization + valid ops protocol (Uninitialized -> Initialized(with complete ops) -> Analyzing)

**Location**: `/data/test_case/main.rs:1-593`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The analyzer relies on a global initialization step (analyzer_init) before analysis can run. This is enforced only by a runtime flag (INITIALIZED) and by panicking `expect("non-null function pointer")` when required tokenizer ops are missing. After initialization, analyze_text_internal assumes ANALYZER_OPS contains non-None function pointers for load_text/next_token/get_stats, and it mutates global aggregate counters (TOKEN_TYPE_COUNTS and COMMON_WORDS/COMMON_WORD_COUNTS). The type system does not prevent calling analyze_text_internal before analyzer_init, nor does it guarantee the passed tokenizer_ops_t is complete (non-None fields), so misuse yields runtime error prints or panics.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct token_t, 8 free function(s); struct tokenizer_ops_t, 3 free function(s); struct analysis_result_t, 2 free function(s)

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

use std::cell::{Cell, RefCell};
use std::ffi::CStr;
use std::io::{self, Read, Write};

pub type token_type_t = u32;

pub const TOKEN_EOF: token_type_t = 0;
pub const TOKEN_WORD: token_type_t = 1;
pub const TOKEN_NUMBER: token_type_t = 2;
pub const TOKEN_PUNCTUATION: token_type_t = 3;
pub const TOKEN_WHITESPACE: token_type_t = 4;
pub const TOKEN_NEWLINE: token_type_t = 5;
pub const TOKEN_IDENTIFIER: token_type_t = 6;
pub const TOKEN_KEYWORD: token_type_t = 7;
pub const TOKEN_OPERATOR: token_type_t = 8;
pub const TOKEN_STRING: token_type_t = 9;
pub const TOKEN_COMMENT: token_type_t = 10;
pub const TOKEN_ERROR: token_type_t = 11;

pub const MAX_TOKEN_LENGTH: usize = 256;
pub const MAX_BUFFER_SIZE: usize = 8192;
pub const MAX_INPUT_SIZE: usize = 4096;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct token_t {
    pub type_0: token_type_t,
    pub value: [i8; 256],
    pub length: usize,
    pub line: i32,
    pub column: i32,
}

pub type tokenizer_next_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_peek_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_reset_fn = Option<unsafe extern "C" fn() -> ()>;
pub type tokenizer_load_fn = Option<unsafe extern "C" fn(*const i8) -> i32>;
pub type tokenizer_get_stats_fn =
    Option<unsafe extern "C" fn(*mut usize, *mut usize, *mut usize) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tokenizer_ops_t {
    pub next_token: tokenizer_next_fn,
    pub peek_token: tokenizer_peek_fn,
    pub reset: tokenizer_reset_fn,
    pub load_text: tokenizer_load_fn,
    pub get_stats: tokenizer_get_stats_fn,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct analysis_result_t {
    pub word_count: usize,
    pub number_count: usize,
    pub keyword_count: usize,
    pub operator_count: usize,
    pub comment_count: usize,
    pub string_count: usize,
    pub line_count: usize,
    pub char_count: usize,
}

// ===================== tokenizer =====================

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static POS: Cell<usize> = const { Cell::new(0) };
    static LINE: Cell<i32> = const { Cell::new(1) };
    static COL: Cell<i32> = const { Cell::new(1) };
    static TOTAL_TOKENS: Cell<usize> = const { Cell::new(0) };
    static TOTAL_LINES: Cell<usize> = const { Cell::new(0) };
    static TOTAL_CHARS: Cell<usize> = const { Cell::new(0) };

    static LOOKAHEAD: Cell<token_t> = const {
        Cell::new(token_t { type_0: TOKEN_EOF, value: [0; 256], length: 0, line: 0, column: 0 })
    };
    static LOOKAHEAD_VALID: Cell<bool> = const { Cell::new(false) };

    // Match original C2Rust: num_keywords starts at 0 and is never set.
    static NUM_KEYWORDS: Cell<i32> = const { Cell::new(0) };
}

const KEYWORDS: [&str; 31] = [
    "if", "else", "while", "for", "return", "int", "char", "float", "double", "void", "struct",
    "typedef", "const", "static", "extern", "auto", "register", "sizeof", "break", "continue",
    "switch", "case", "default", "do", "goto", "enum", "union", "signed", "unsigned", "long",
    "short",
];

fn is_space_not_newline(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | 0x0b | 0x0c | b'\r')
}
fn is_alpha(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b)
}
fn is_digit(b: u8) -> bool {
    (b'0'..=b'9').contains(&b)
}
fn is_alnum(b: u8) -> bool {
    is_alpha(b) || is_digit(b)
}

fn peek_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            0
        } else {
            buf[pos]
        }
    })
}
fn peek_next_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos + 1 >= buf.len() {
            0
        } else {
            buf[pos + 1]
        }
    })
}

fn advance_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            return 0;
        }
        POS.set(pos + 1);
        let c = buf[pos];
        TOTAL_CHARS.set(TOTAL_CHARS.get() + 1);
        if c == b'\n' {
            LINE.set(LINE.get() + 1);
            COL.set(1);
            TOTAL_LINES.set(TOTAL_LINES.get() + 1);
        } else {
            COL.set(COL.get() + 1);
        }
        c
    })
}

fn skip_whitespace() {
    while peek_char() != 0 && is_space_not_newline(peek_char()) {
        advance_char();
    }
}

fn create_token(type_0: token_type_t, bytes: &[u8]) -> token_t {
    let mut token = token_t {
        type_0,
        value: [0; 256],
        length: 0,
        line: LINE.get(),
        column: 0,
    };

    let len = bytes.len().min(MAX_TOKEN_LENGTH - 1);
    token.length = len;
    for (i, &b) in bytes.iter().take(len).enumerate() {
        token.value[i] = b as i8;
    }
    token.value[len] = 0;

    token.column = (COL.get() as usize).saturating_sub(len) as i32;
    TOTAL_TOKENS.set(TOTAL_TOKENS.get() + 1);
    token
}

fn is_keyword(s: &str) -> bool {
    let n = NUM_KEYWORDS.get().max(0) as usize;
    KEYWORDS.iter().take(n).any(|&k| k == s)
}

fn scan_word() -> token_t {
    let mut buf = Vec::with_capacity(64);
    while peek_char() != 0
        && (is_alnum(peek_char()) || peek_char() == b'_')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        buf.push(advance_char());
    }
    let s = std::str::from_utf8(&buf).unwrap_or("");
    if is_keyword(s) {
        create_token(TOKEN_KEYWORD, &buf)
    } else {
        create_token(TOKEN_IDENTIFIER, &buf)
    }
}

fn scan_number() -> token_t {
    let mut buf = Vec::with_capacity(32);
    let mut has_decimal = false;
    while peek_char() != 0
        && (is_digit(peek_char()) || peek_char() == b'.')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        if peek_char() == b'.' {
            if has_decimal {
                break;
            }
            has_decimal = true;
        }
        buf.push(advance_char());
    }
    create_token(TOKEN_NUMBER, &buf)
}

fn scan_string() -> token_t {
    let mut buf = Vec::with_capacity(64);
    let quote = advance_char();
    buf.push(quote);

    while peek_char() != 0
        && peek_char() != quote
        && peek_char() != b'\n'
        && buf.len() < MAX_TOKEN_LENGTH - 2
    {
        if peek_char() == b'\\' {
            buf.push(advance_char());
            if peek_char() != 0 {
                buf.push(advance_char());
            }
        } else {
            buf.push(advance_char());
        }
    }
    if peek_char() == quote {
        buf.push(advance_char());
    }
    create_token(TOKEN_STRING, &buf)
}

fn scan_comment() -> token_t {
    let mut buf = Vec::with_capacity(128);
    buf.push(advance_char()); // '/'

    if peek_char() == b'/' {
        buf.push(advance_char());
        while peek_char() != 0 && peek_char() != b'\n' && buf.len() < MAX_TOKEN_LENGTH - 1 {
            buf.push(advance_char());
        }
    } else if peek_char() == b'*' {
        buf.push(advance_char());
        while peek_char() != 0 && buf.len() < MAX_TOKEN_LENGTH - 2 {
            if peek_char() == b'*' {
                buf.push(advance_char());
                if peek_char() == b'/' {
                    buf.push(advance_char());
                    break;
                }
            } else {
                buf.push(advance_char());
            }
        }
    }
    create_token(TOKEN_COMMENT, &buf)
}

fn scan_operator() -> token_t {
    let mut buf = Vec::with_capacity(2);
    let c = peek_char();
    buf.push(advance_char());
    let next = peek_char();

    let two = matches!(
        (c, next),
        (b'=', b'=')
            | (b'!', b'=')
            | (b'<', b'=')
            | (b'>', b'=')
            | (b'&', b'&')
            | (b'|', b'|')
            | (b'+', b'+')
            | (b'-', b'-')
            | (b'-', b'>')
            | (b'<', b'<')
            | (b'>', b'>')
    );
    if two {
        buf.push(advance_char());
    }
    create_token(TOKEN_OPERATOR, &buf)
}

fn is_operator_char(c: u8) -> bool {
    matches!(
        c,
        b'+' | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'='
            | b'<'
            | b'>'
            | b'!'
            | b'&'
            | b'|'
            | b'^'
            | b'~'
            | b'?'
            | b':'
    )
}
fn is_punct_char(c: u8) -> bool {
    matches!(c, b'(' | b')' | b'{' | b'}' | b'[' | b']' | b';' | b',' | b'.')
}

pub unsafe extern "C" fn tokenizer_next_token() -> token_t {
    if LOOKAHEAD_VALID.get() {
        LOOKAHEAD_VALID.set(false);
        return LOOKAHEAD.get();
    }

    skip_whitespace();

    if peek_char() == 0 {
        return create_token(TOKEN_EOF, &[]);
    }

    let c = peek_char();

    if c == b'\n' {
        let b = advance_char();
        return create_token(TOKEN_NEWLINE, &[b]);
    }

    if is_alpha(c) || c == b'_' {
        return scan_word();
    }

    if is_digit(c) {
        return scan_number();
    }

    if c == b'"' || c == b'\'' {
        return scan_string();
    }

    // Match original intent: detect comment by looking at next char.
    if c == b'/' && (peek_next_char() == b'/' || peek_next_char() == b'*') {
        return scan_comment();
    }

    if is_operator_char(c) {
        return scan_operator();
    }

    if is_punct_char(c) {
        let b = advance_char();
        return create_token(TOKEN_PUNCTUATION, &[b]);
    }

    let b = advance_char();
    create_token(TOKEN_ERROR, &[b])
}

pub unsafe extern "C" fn tokenizer_peek_token() -> token_t {
    if !LOOKAHEAD_VALID.get() {
        LOOKAHEAD.set(tokenizer_next_token());
        LOOKAHEAD_VALID.set(true);
    }
    LOOKAHEAD.get()
}

pub extern "C" fn tokenizer_reset() {
    POS.set(0);
    LINE.set(1);
    COL.set(1);
    LOOKAHEAD_VALID.set(false);
}

pub unsafe extern "C" fn tokenizer_load_text(text: *const i8) -> i32 {
    if text.is_null() {
        return -1;
    }
    let bytes = CStr::from_ptr(text).to_bytes();
    if bytes.len() >= MAX_BUFFER_SIZE {
        eprintln!("Error: Input text too large");
        return -1;
    }

    INPUT.with_borrow_mut(|buf| {
        buf.clear();
        buf.extend_from_slice(bytes);
    });

    tokenizer_reset();
    0
}

pub unsafe extern "C" fn tokenizer_get_stats(
    lines: *mut usize,
    tokens: *mut usize,
    chars: *mut usize,
) {
    if !lines.is_null() {
        *lines = TOTAL_LINES.get();
    }
    if !tokens.is_null() {
        *tokens = TOTAL_TOKENS.get();
    }
    if !chars.is_null() {
        *chars = TOTAL_CHARS.get();
    }
}

pub extern "C" fn get_tokenizer_ops() -> tokenizer_ops_t {
    tokenizer_ops_t {
        next_token: Some(tokenizer_next_token),
        peek_token: Some(tokenizer_peek_token),
        reset: Some(tokenizer_reset),
        load_text: Some(tokenizer_load_text),
        get_stats: Some(tokenizer_get_stats),
    }
}

// ===================== analyzer =====================

thread_local! {
    static ANALYZER_OPS: Cell<tokenizer_ops_t> = const {
        Cell::new(tokenizer_ops_t { next_token: None, peek_token: None, reset: None, load_text: None, get_stats: None })
    };
    static INITIALIZED: Cell<bool> = const { Cell::new(false) };
    static TOKEN_TYPE_COUNTS: RefCell<[i32; 20]> = const { RefCell::new([0; 20]) };

    static COMMON_WORDS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static COMMON_WORD_COUNTS: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

pub extern "C" fn analyzer_init(ops: tokenizer_ops_t) {
    ANALYZER_OPS.set(ops);
    INITIALIZED.set(true);
    TOKEN_TYPE_COUNTS.with_borrow_mut(|c| c.fill(0));
    COMMON_WORDS.with_borrow_mut(|w| w.clear());
    COMMON_WORD_COUNTS.with_borrow_mut(|c| c.clear());
}

fn cstr_from_i8_buf(buf: &[i8]) -> &CStr {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
}

fn track_word(word_c: &[i8]) {
    let word = cstr_from_i8_buf(word_c).to_string_lossy().into_owned();
    if word.is_empty() {
        return;
    }

    COMMON_WORDS.with_borrow_mut(|words| {
        COMMON_WORD_COUNTS.with_borrow_mut(|counts| {
            if let Some(idx) = words.iter().position(|w| w == &word) {
                counts[idx] += 1;
                return;
            }
            if words.len() < 100 {
                words.push(word);
                counts.push(1);
            }
        })
    });
}

pub(crate) unsafe fn analyze_text_internal(text: &mut [i8]) -> analysis_result_t {
    let mut result = analysis_result_t {
        word_count: 0,
        number_count: 0,
        keyword_count: 0,
        operator_count: 0,
        comment_count: 0,
        string_count: 0,
        line_count: 0,
        char_count: 0,
    };

    if !INITIALIZED.get() {
        eprintln!("Error: Analyzer not initialized");
        return result;
    }

    let ops = ANALYZER_OPS.get();
    if ops
        .load_text
        .expect("non-null function pointer")(text.as_mut_ptr())
        != 0
    {
        eprintln!("Error: Failed to load text");
        return result;
    }

    loop {
        let token = ops.next_token.expect("non-null function pointer")();
        if token.type_0 == TOKEN_EOF {
            break;
        }

        TOKEN_TYPE_COUNTS.with_borrow_mut(|counts| {
            let idx = token.type_0 as usize;
            if idx < counts.len() {
                counts[idx] += 1;
            }
        });

        match token.type_0 {
            TOKEN_WORD | TOKEN_IDENTIFIER => {
                result.word_count = result.word_count.wrapping_add(1);
                track_word(&token.value);
            }
            TOKEN_NUMBER => result.number_count = result.number_count.wrapping_add(1),
            TOKEN_KEYWORD => result.keyword_count = result.keyword_count.wrapping_add(1),
            TOKEN_OPERATOR => result.operator_count = result.operator_count.wrapping_add(1),
            TOKEN_COMMENT => result.comment_count = result.comment_count.wrapping_add(1),
            TOKEN_STRING => result.string_count = result.string_count.wrapping_add(1),
            TOKEN_NEWLINE => result.line_count = result.line_count.wrapping_add(1),
            _ => {}
        }
    }

    let mut lines: usize = 0;
    let mut tokens: usize = 0;
    let mut chars: usize = 0;
    ops.get_stats
        .expect("non-null function pointer")(&mut lines, &mut tokens, &mut chars);
    result.line_count = lines;
    result.char_count = chars;
    result
}

pub unsafe extern "C" fn print_token_distribution() {
    print!("\n=== Token Distribution ===\n");
    let token_names: [&str; 12] = [
        "EOF",
        "WORD",
        "NUMBER",
        "PUNCTUATION",
        "WHITESPACE",
        "NEWLINE",
        "IDENTIFIER",
        "KEYWORD",
        "OPERATOR",
        "STRING",
        "COMMENT",
        "ERROR",
    ];

    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        for i in 0..12 {
            if counts[i] > 0 {
                println!("{}: {}", token_names[i], counts[i]);
            }
        }
    });

    print!("\n=== Most Common Words ===\n");

    let mut pairs: Vec<(String, i32)> = Vec::new();
    COMMON_WORDS.with_borrow(|words| {
        COMMON_WORD_COUNTS.with_borrow(|counts| {
            for (w, &c) in words.iter().zip(counts.iter()) {
                pairs.push((w.clone(), c));
            }
        })
    });
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let limit = pairs.len().min(10);
    for (i, (w, c)) in pairs.into_iter().take(limit).enumerate() {
        println!("{}. {}: {} times", i + 1, w, c);
    }
}

pub extern "C" fn calculate_complexity_score() -> i32 {
    let mut score: i32 = 0;
    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        score += counts[TOKEN_KEYWORD as usize] * 2;
        score += counts[TOKEN_OPERATOR as usize];
        score += counts[TOKEN_PUNCTUATION as usize] / 10;
  
// ... (truncated) ...
```

**Entity:** analyzer (thread-local globals: ANALYZER_OPS/INITIALIZED/TOKEN_TYPE_COUNTS/COMMON_WORDS/COMMON_WORD_COUNTS)

**States:** Uninitialized, Initialized(OpsSet), Analyzing

**Transitions:**
- Uninitialized -> Initialized(OpsSet) via analyzer_init(ops) setting INITIALIZED=true and storing ANALYZER_OPS
- Initialized(OpsSet) -> Analyzing via analyze_text_internal() when it calls ops.load_text and then repeatedly ops.next_token
- Analyzing -> Initialized(OpsSet) on return (global counts remain mutated for later reporting via print_token_distribution/calculate_complexity_score)

**Evidence:** thread_local! INITIALIZED: Cell<bool> tracks runtime init state; analyzer_init(): `ANALYZER_OPS.set(ops); INITIALIZED.set(true);` and clears TOKEN_TYPE_COUNTS/COMMON_WORDS/COMMON_WORD_COUNTS; analyze_text_internal(): `if !INITIALIZED.get() { eprintln!("Error: Analyzer not initialized"); return result; }`; analyze_text_internal(): `ops.load_text.expect("non-null function pointer")(...)` and `ops.next_token.expect("non-null function pointer")()` and `ops.get_stats.expect("non-null function pointer")(...)` demonstrate required non-None ops fields enforced only by runtime panic; ANALYZER_OPS is stored as `Cell<tokenizer_ops_t>` with all fields initially `None`

**Implementation:** Introduce `struct Analyzer<S> { ops: CompleteTokenizerOps, counts: ..., common: ... }` where `Analyzer<Uninit>::init(ops: CompleteTokenizerOps) -> Analyzer<Init>`. Define `CompleteTokenizerOps` as a Rust struct of non-optional function pointers (or safe trait object) so missing callbacks are impossible. Expose `analyze(&mut self, text: &CStr) -> analysis_result_t` only on `Analyzer<Init>`.

---

## Precondition Invariants

### 2. token_t value/length validity invariant (length-bounded, type-dependent payload)

**Location**: `/data/test_case/main.rs:1-12`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: token_t encodes a token with a fixed-size byte buffer (value) plus a runtime length. Correctness implicitly requires that length never exceeds the capacity of value (256), and that only the prefix value[0..length] is considered meaningful. Additionally, the meaning/encoding of the bytes in value is likely dependent on type_0 (token_type_t), but this coupling is not enforced: any type_0 can be paired with any bytes/length at compile time. As written, the type system permits constructing token_t values where length is out of bounds or inconsistent with type_0, creating latent UB/panic risks in downstream code that slices/copies based on length or interprets bytes based on type.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct tokenizer_ops_t, 3 free function(s); struct analysis_result_t, 2 free function(s); 24 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct token_t {
    pub type_0: token_type_t,
    pub value: [i8; 256],
    pub length: usize,
    pub line: i32,
    pub column: i32,
}

```

**Entity:** token_t

**States:** ValidToken, InvalidToken

**Transitions:**
- InvalidToken -> ValidToken via construction/validation (not present in snippet)

**Evidence:** line 8: pub value: [i8; 256] is a fixed-capacity buffer; line 9: pub length: usize is an unchecked runtime length that must fit within the 256-byte buffer; line 7: pub type_0: token_type_t suggests payload interpretation depends on a token kind, but the struct does not encode this relationship

**Implementation:** Make fields private and expose constructors that enforce invariants, e.g., `struct TokenValue([u8; 256], u8);` where length is `u8` (0..=256) or `NonZeroU8` as needed. Alternatively store `value: [u8; 256]` plus `len: u8` and provide `fn as_bytes(&self) -> &[u8]` that returns `&self.value[..self.len as usize]`. If payload depends on token type, consider an enum representation `enum Token { Ident(SmallVec<u8>), Number(...), ... }` or a `Token<TKind>` typestate/newtype per kind to prevent mismatched (type_0, value) pairs.

---

### 5. token_t value/length C-string invariant (NUL-terminated, length <= MAX_TOKEN_LENGTH-1, UTF-8 assumptions downstream)

**Location**: `/data/test_case/main.rs:1-593`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Several functions assume that `token_t.value` is a valid NUL-terminated C string with a correct `length` field, and that it can be losslessly (or at least safely) interpreted as text. This is maintained when tokens are created through create_token(), but it is not enforced by the type system because token_t is a plain #[repr(C)] struct with public fields, and because downstream code converts `token.value` to `&CStr` using `CStr::from_ptr` (which requires a terminating NUL). If any token_t is constructed/modified elsewhere (FFI or future code) without the terminator or with inconsistent length, cstr_from_i8_buf/track_word invoke undefined behavior or produce incorrect results; additionally scan_word uses `from_utf8(&buf).unwrap_or("")`, implying an (unchecked) expectation that identifier bytes are UTF-8.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct token_t, 8 free function(s); struct tokenizer_ops_t, 3 free function(s); struct analysis_result_t, 2 free function(s)

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

use std::cell::{Cell, RefCell};
use std::ffi::CStr;
use std::io::{self, Read, Write};

pub type token_type_t = u32;

pub const TOKEN_EOF: token_type_t = 0;
pub const TOKEN_WORD: token_type_t = 1;
pub const TOKEN_NUMBER: token_type_t = 2;
pub const TOKEN_PUNCTUATION: token_type_t = 3;
pub const TOKEN_WHITESPACE: token_type_t = 4;
pub const TOKEN_NEWLINE: token_type_t = 5;
pub const TOKEN_IDENTIFIER: token_type_t = 6;
pub const TOKEN_KEYWORD: token_type_t = 7;
pub const TOKEN_OPERATOR: token_type_t = 8;
pub const TOKEN_STRING: token_type_t = 9;
pub const TOKEN_COMMENT: token_type_t = 10;
pub const TOKEN_ERROR: token_type_t = 11;

pub const MAX_TOKEN_LENGTH: usize = 256;
pub const MAX_BUFFER_SIZE: usize = 8192;
pub const MAX_INPUT_SIZE: usize = 4096;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct token_t {
    pub type_0: token_type_t,
    pub value: [i8; 256],
    pub length: usize,
    pub line: i32,
    pub column: i32,
}

pub type tokenizer_next_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_peek_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_reset_fn = Option<unsafe extern "C" fn() -> ()>;
pub type tokenizer_load_fn = Option<unsafe extern "C" fn(*const i8) -> i32>;
pub type tokenizer_get_stats_fn =
    Option<unsafe extern "C" fn(*mut usize, *mut usize, *mut usize) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tokenizer_ops_t {
    pub next_token: tokenizer_next_fn,
    pub peek_token: tokenizer_peek_fn,
    pub reset: tokenizer_reset_fn,
    pub load_text: tokenizer_load_fn,
    pub get_stats: tokenizer_get_stats_fn,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct analysis_result_t {
    pub word_count: usize,
    pub number_count: usize,
    pub keyword_count: usize,
    pub operator_count: usize,
    pub comment_count: usize,
    pub string_count: usize,
    pub line_count: usize,
    pub char_count: usize,
}

// ===================== tokenizer =====================

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static POS: Cell<usize> = const { Cell::new(0) };
    static LINE: Cell<i32> = const { Cell::new(1) };
    static COL: Cell<i32> = const { Cell::new(1) };
    static TOTAL_TOKENS: Cell<usize> = const { Cell::new(0) };
    static TOTAL_LINES: Cell<usize> = const { Cell::new(0) };
    static TOTAL_CHARS: Cell<usize> = const { Cell::new(0) };

    static LOOKAHEAD: Cell<token_t> = const {
        Cell::new(token_t { type_0: TOKEN_EOF, value: [0; 256], length: 0, line: 0, column: 0 })
    };
    static LOOKAHEAD_VALID: Cell<bool> = const { Cell::new(false) };

    // Match original C2Rust: num_keywords starts at 0 and is never set.
    static NUM_KEYWORDS: Cell<i32> = const { Cell::new(0) };
}

const KEYWORDS: [&str; 31] = [
    "if", "else", "while", "for", "return", "int", "char", "float", "double", "void", "struct",
    "typedef", "const", "static", "extern", "auto", "register", "sizeof", "break", "continue",
    "switch", "case", "default", "do", "goto", "enum", "union", "signed", "unsigned", "long",
    "short",
];

fn is_space_not_newline(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | 0x0b | 0x0c | b'\r')
}
fn is_alpha(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b)
}
fn is_digit(b: u8) -> bool {
    (b'0'..=b'9').contains(&b)
}
fn is_alnum(b: u8) -> bool {
    is_alpha(b) || is_digit(b)
}

fn peek_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            0
        } else {
            buf[pos]
        }
    })
}
fn peek_next_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos + 1 >= buf.len() {
            0
        } else {
            buf[pos + 1]
        }
    })
}

fn advance_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            return 0;
        }
        POS.set(pos + 1);
        let c = buf[pos];
        TOTAL_CHARS.set(TOTAL_CHARS.get() + 1);
        if c == b'\n' {
            LINE.set(LINE.get() + 1);
            COL.set(1);
            TOTAL_LINES.set(TOTAL_LINES.get() + 1);
        } else {
            COL.set(COL.get() + 1);
        }
        c
    })
}

fn skip_whitespace() {
    while peek_char() != 0 && is_space_not_newline(peek_char()) {
        advance_char();
    }
}

fn create_token(type_0: token_type_t, bytes: &[u8]) -> token_t {
    let mut token = token_t {
        type_0,
        value: [0; 256],
        length: 0,
        line: LINE.get(),
        column: 0,
    };

    let len = bytes.len().min(MAX_TOKEN_LENGTH - 1);
    token.length = len;
    for (i, &b) in bytes.iter().take(len).enumerate() {
        token.value[i] = b as i8;
    }
    token.value[len] = 0;

    token.column = (COL.get() as usize).saturating_sub(len) as i32;
    TOTAL_TOKENS.set(TOTAL_TOKENS.get() + 1);
    token
}

fn is_keyword(s: &str) -> bool {
    let n = NUM_KEYWORDS.get().max(0) as usize;
    KEYWORDS.iter().take(n).any(|&k| k == s)
}

fn scan_word() -> token_t {
    let mut buf = Vec::with_capacity(64);
    while peek_char() != 0
        && (is_alnum(peek_char()) || peek_char() == b'_')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        buf.push(advance_char());
    }
    let s = std::str::from_utf8(&buf).unwrap_or("");
    if is_keyword(s) {
        create_token(TOKEN_KEYWORD, &buf)
    } else {
        create_token(TOKEN_IDENTIFIER, &buf)
    }
}

fn scan_number() -> token_t {
    let mut buf = Vec::with_capacity(32);
    let mut has_decimal = false;
    while peek_char() != 0
        && (is_digit(peek_char()) || peek_char() == b'.')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        if peek_char() == b'.' {
            if has_decimal {
                break;
            }
            has_decimal = true;
        }
        buf.push(advance_char());
    }
    create_token(TOKEN_NUMBER, &buf)
}

fn scan_string() -> token_t {
    let mut buf = Vec::with_capacity(64);
    let quote = advance_char();
    buf.push(quote);

    while peek_char() != 0
        && peek_char() != quote
        && peek_char() != b'\n'
        && buf.len() < MAX_TOKEN_LENGTH - 2
    {
        if peek_char() == b'\\' {
            buf.push(advance_char());
            if peek_char() != 0 {
                buf.push(advance_char());
            }
        } else {
            buf.push(advance_char());
        }
    }
    if peek_char() == quote {
        buf.push(advance_char());
    }
    create_token(TOKEN_STRING, &buf)
}

fn scan_comment() -> token_t {
    let mut buf = Vec::with_capacity(128);
    buf.push(advance_char()); // '/'

    if peek_char() == b'/' {
        buf.push(advance_char());
        while peek_char() != 0 && peek_char() != b'\n' && buf.len() < MAX_TOKEN_LENGTH - 1 {
            buf.push(advance_char());
        }
    } else if peek_char() == b'*' {
        buf.push(advance_char());
        while peek_char() != 0 && buf.len() < MAX_TOKEN_LENGTH - 2 {
            if peek_char() == b'*' {
                buf.push(advance_char());
                if peek_char() == b'/' {
                    buf.push(advance_char());
                    break;
                }
            } else {
                buf.push(advance_char());
            }
        }
    }
    create_token(TOKEN_COMMENT, &buf)
}

fn scan_operator() -> token_t {
    let mut buf = Vec::with_capacity(2);
    let c = peek_char();
    buf.push(advance_char());
    let next = peek_char();

    let two = matches!(
        (c, next),
        (b'=', b'=')
            | (b'!', b'=')
            | (b'<', b'=')
            | (b'>', b'=')
            | (b'&', b'&')
            | (b'|', b'|')
            | (b'+', b'+')
            | (b'-', b'-')
            | (b'-', b'>')
            | (b'<', b'<')
            | (b'>', b'>')
    );
    if two {
        buf.push(advance_char());
    }
    create_token(TOKEN_OPERATOR, &buf)
}

fn is_operator_char(c: u8) -> bool {
    matches!(
        c,
        b'+' | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'='
            | b'<'
            | b'>'
            | b'!'
            | b'&'
            | b'|'
            | b'^'
            | b'~'
            | b'?'
            | b':'
    )
}
fn is_punct_char(c: u8) -> bool {
    matches!(c, b'(' | b')' | b'{' | b'}' | b'[' | b']' | b';' | b',' | b'.')
}

pub unsafe extern "C" fn tokenizer_next_token() -> token_t {
    if LOOKAHEAD_VALID.get() {
        LOOKAHEAD_VALID.set(false);
        return LOOKAHEAD.get();
    }

    skip_whitespace();

    if peek_char() == 0 {
        return create_token(TOKEN_EOF, &[]);
    }

    let c = peek_char();

    if c == b'\n' {
        let b = advance_char();
        return create_token(TOKEN_NEWLINE, &[b]);
    }

    if is_alpha(c) || c == b'_' {
        return scan_word();
    }

    if is_digit(c) {
        return scan_number();
    }

    if c == b'"' || c == b'\'' {
        return scan_string();
    }

    // Match original intent: detect comment by looking at next char.
    if c == b'/' && (peek_next_char() == b'/' || peek_next_char() == b'*') {
        return scan_comment();
    }

    if is_operator_char(c) {
        return scan_operator();
    }

    if is_punct_char(c) {
        let b = advance_char();
        return create_token(TOKEN_PUNCTUATION, &[b]);
    }

    let b = advance_char();
    create_token(TOKEN_ERROR, &[b])
}

pub unsafe extern "C" fn tokenizer_peek_token() -> token_t {
    if !LOOKAHEAD_VALID.get() {
        LOOKAHEAD.set(tokenizer_next_token());
        LOOKAHEAD_VALID.set(true);
    }
    LOOKAHEAD.get()
}

pub extern "C" fn tokenizer_reset() {
    POS.set(0);
    LINE.set(1);
    COL.set(1);
    LOOKAHEAD_VALID.set(false);
}

pub unsafe extern "C" fn tokenizer_load_text(text: *const i8) -> i32 {
    if text.is_null() {
        return -1;
    }
    let bytes = CStr::from_ptr(text).to_bytes();
    if bytes.len() >= MAX_BUFFER_SIZE {
        eprintln!("Error: Input text too large");
        return -1;
    }

    INPUT.with_borrow_mut(|buf| {
        buf.clear();
        buf.extend_from_slice(bytes);
    });

    tokenizer_reset();
    0
}

pub unsafe extern "C" fn tokenizer_get_stats(
    lines: *mut usize,
    tokens: *mut usize,
    chars: *mut usize,
) {
    if !lines.is_null() {
        *lines = TOTAL_LINES.get();
    }
    if !tokens.is_null() {
        *tokens = TOTAL_TOKENS.get();
    }
    if !chars.is_null() {
        *chars = TOTAL_CHARS.get();
    }
}

pub extern "C" fn get_tokenizer_ops() -> tokenizer_ops_t {
    tokenizer_ops_t {
        next_token: Some(tokenizer_next_token),
        peek_token: Some(tokenizer_peek_token),
        reset: Some(tokenizer_reset),
        load_text: Some(tokenizer_load_text),
        get_stats: Some(tokenizer_get_stats),
    }
}

// ===================== analyzer =====================

thread_local! {
    static ANALYZER_OPS: Cell<tokenizer_ops_t> = const {
        Cell::new(tokenizer_ops_t { next_token: None, peek_token: None, reset: None, load_text: None, get_stats: None })
    };
    static INITIALIZED: Cell<bool> = const { Cell::new(false) };
    static TOKEN_TYPE_COUNTS: RefCell<[i32; 20]> = const { RefCell::new([0; 20]) };

    static COMMON_WORDS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static COMMON_WORD_COUNTS: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

pub extern "C" fn analyzer_init(ops: tokenizer_ops_t) {
    ANALYZER_OPS.set(ops);
    INITIALIZED.set(true);
    TOKEN_TYPE_COUNTS.with_borrow_mut(|c| c.fill(0));
    COMMON_WORDS.with_borrow_mut(|w| w.clear());
    COMMON_WORD_COUNTS.with_borrow_mut(|c| c.clear());
}

fn cstr_from_i8_buf(buf: &[i8]) -> &CStr {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
}

fn track_word(word_c: &[i8]) {
    let word = cstr_from_i8_buf(word_c).to_string_lossy().into_owned();
    if word.is_empty() {
        return;
    }

    COMMON_WORDS.with_borrow_mut(|words| {
        COMMON_WORD_COUNTS.with_borrow_mut(|counts| {
            if let Some(idx) = words.iter().position(|w| w == &word) {
                counts[idx] += 1;
                return;
            }
            if words.len() < 100 {
                words.push(word);
                counts.push(1);
            }
        })
    });
}

pub(crate) unsafe fn analyze_text_internal(text: &mut [i8]) -> analysis_result_t {
    let mut result = analysis_result_t {
        word_count: 0,
        number_count: 0,
        keyword_count: 0,
        operator_count: 0,
        comment_count: 0,
        string_count: 0,
        line_count: 0,
        char_count: 0,
    };

    if !INITIALIZED.get() {
        eprintln!("Error: Analyzer not initialized");
        return result;
    }

    let ops = ANALYZER_OPS.get();
    if ops
        .load_text
        .expect("non-null function pointer")(text.as_mut_ptr())
        != 0
    {
        eprintln!("Error: Failed to load text");
        return result;
    }

    loop {
        let token = ops.next_token.expect("non-null function pointer")();
        if token.type_0 == TOKEN_EOF {
            break;
        }

        TOKEN_TYPE_COUNTS.with_borrow_mut(|counts| {
            let idx = token.type_0 as usize;
            if idx < counts.len() {
                counts[idx] += 1;
            }
        });

        match token.type_0 {
            TOKEN_WORD | TOKEN_IDENTIFIER => {
                result.word_count = result.word_count.wrapping_add(1);
                track_word(&token.value);
            }
            TOKEN_NUMBER => result.number_count = result.number_count.wrapping_add(1),
            TOKEN_KEYWORD => result.keyword_count = result.keyword_count.wrapping_add(1),
            TOKEN_OPERATOR => result.operator_count = result.operator_count.wrapping_add(1),
            TOKEN_COMMENT => result.comment_count = result.comment_count.wrapping_add(1),
            TOKEN_STRING => result.string_count = result.string_count.wrapping_add(1),
            TOKEN_NEWLINE => result.line_count = result.line_count.wrapping_add(1),
            _ => {}
        }
    }

    let mut lines: usize = 0;
    let mut tokens: usize = 0;
    let mut chars: usize = 0;
    ops.get_stats
        .expect("non-null function pointer")(&mut lines, &mut tokens, &mut chars);
    result.line_count = lines;
    result.char_count = chars;
    result
}

pub unsafe extern "C" fn print_token_distribution() {
    print!("\n=== Token Distribution ===\n");
    let token_names: [&str; 12] = [
        "EOF",
        "WORD",
        "NUMBER",
        "PUNCTUATION",
        "WHITESPACE",
        "NEWLINE",
        "IDENTIFIER",
        "KEYWORD",
        "OPERATOR",
        "STRING",
        "COMMENT",
        "ERROR",
    ];

    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        for i in 0..12 {
            if counts[i] > 0 {
                println!("{}: {}", token_names[i], counts[i]);
            }
        }
    });

    print!("\n=== Most Common Words ===\n");

    let mut pairs: Vec<(String, i32)> = Vec::new();
    COMMON_WORDS.with_borrow(|words| {
        COMMON_WORD_COUNTS.with_borrow(|counts| {
            for (w, &c) in words.iter().zip(counts.iter()) {
                pairs.push((w.clone(), c));
            }
        })
    });
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let limit = pairs.len().min(10);
    for (i, (w, c)) in pairs.into_iter().take(limit).enumerate() {
        println!("{}. {}: {} times", i + 1, w, c);
    }
}

pub extern "C" fn calculate_complexity_score() -> i32 {
    let mut score: i32 = 0;
    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        score += counts[TOKEN_KEYWORD as usize] * 2;
        score += counts[TOKEN_OPERATOR as usize];
        score += counts[TOKEN_PUNCTUATION as usize] / 10;
  
// ... (truncated) ...
```

**Entity:** token_t

**States:** WellFormed(TokenBufferInvariantHolds), IllFormed(Length/Terminator/EncodingMismatch)

**Transitions:**
- IllFormed -> WellFormed via create_token(type, bytes) which writes NUL terminator and clamps length
- WellFormed -> IllFormed via any external construction/mutation of token_t fields (public fields, FFI) that violates NUL/length invariants

**Evidence:** token_t has public fields `value: [i8; 256]` and `length: usize` with no encapsulation; create_token(): `let len = bytes.len().min(MAX_TOKEN_LENGTH - 1); token.length = len; ... token.value[len] = 0;` establishes NUL-termination and bounded length; cstr_from_i8_buf(): `unsafe { CStr::from_ptr(buf.as_ptr()) }` requires buf to contain a NUL terminator at/after the start; track_word(): passes `&token.value` into cstr_from_i8_buf(), relying on token.value being a valid C string; scan_word(): `std::str::from_utf8(&buf).unwrap_or("")` reveals an implicit encoding expectation for keyword/identifier classification

**Implementation:** Make `token_t` fields private (or wrap in a Rust-only `Token` type) and represent the textual payload as a newtype that guarantees NUL-termination and bounded length, e.g. `struct TokenValue([u8; 256]);` with constructors that always write a terminator and track a `u8`/`NonZeroU8` length. For analysis, carry `&[u8]` slices (or `&CStr` where appropriate) instead of raw `[i8; 256]` to avoid `from_ptr` requirements.

---

## Protocol Invariants

### 1. Tokenizer ops vtable completeness + call-order protocol (load/reset before next/peek)

**Location**: `/data/test_case/main.rs:1-12`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: tokenizer_ops_t is an FFI-style function-pointer table (vtable). Correct use implicitly requires (1) all function pointers be non-null and point to the matching implementation set, and (2) callers follow an ordering protocol where tokenizer state is prepared (e.g., via load_text() and/or reset()) before calling next_token()/peek_token(), with get_stats() only meaningful after some activity. None of these requirements are enforced by the type system: the struct is Copy/Clone and all fields are plain function pointers, so an all-zero/partially-filled/mixed vtable can exist and be copied freely, and the API does not encode readiness/capabilities for particular operations.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct token_t, 8 free function(s); struct analysis_result_t, 2 free function(s); 24 free function(s)


#[repr(C)]
#[derive(Copy, Clone)]
pub struct tokenizer_ops_t {
    pub next_token: tokenizer_next_fn,
    pub peek_token: tokenizer_peek_fn,
    pub reset: tokenizer_reset_fn,
    pub load_text: tokenizer_load_fn,
    pub get_stats: tokenizer_get_stats_fn,
}

```

**Entity:** tokenizer_ops_t

**States:** Uninitialized/InvalidOps, ReadyOps

**Transitions:**
- Uninitialized/InvalidOps -> ReadyOps via constructing a fully-populated tokenizer_ops_t (all required function pointers set)
- ReadyOps (after load_text/reset) -> ReadyOps (token stream consumed) via next_token()
- ReadyOps (after load_text/reset) -> ReadyOps via peek_token()
- ReadyOps -> ReadyOps via reset() (rewinds/clears tokenizer state)
- ReadyOps -> ReadyOps via load_text() (loads new input; establishes precondition for next_token/peek_token)

**Evidence:** line 7: #[derive(Copy, Clone)] allows duplicating ops tables freely, even if partially initialized; line 9-13: fields are raw function pointers (next_token, peek_token, reset, load_text, get_stats) with no type-level guarantee they are valid/non-null or from the same implementation family; line 3: comment mentions other module items (token_t, analysis_result_t, many free functions), suggesting these ops are part of a larger tokenizer protocol that is not encoded here

**Implementation:** Make construction of ops tables safe by hiding fields and exposing constructors that return a validated/complete type (e.g., TokenizerOps<Ready>). If nullability is possible in the ABI, model it as Option<extern "C" fn(...)> and provide a try_new(...) -> Result<TokenizerOps<Ready>, MissingOp>. For call ordering, introduce a higher-level wrapper Tokenizer<Unloaded> / Tokenizer<Loaded> where load_text(self, ...) -> Tokenizer<Loaded>, reset(&mut Tokenizer<Loaded>), and next_token/peek_token/get_stats are only implemented for Tokenizer<Loaded>.

---

### 3. Tokenizer session protocol (Loaded/Reset -> Scanning with optional Lookahead)

**Location**: `/data/test_case/main.rs:1-593`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The tokenizer is an implicit state machine implemented via thread-local mutable globals rather than a value-level object. Correct use relies on calling tokenizer_load_text() (which also calls tokenizer_reset()) before calling tokenizer_next_token()/tokenizer_peek_token(); otherwise tokenization operates on whatever happens to be in thread-local INPUT/POS/LINE/COL (initially empty, or leftover from a prior run). Additionally, peek introduces a secondary cached-token state: tokenizer_peek_token() must set LOOKAHEAD/LOOKAHEAD_VALID, and tokenizer_next_token() must consume that cache first. None of these sequencing requirements are enforced by types because the API is a set of free functions with hidden global state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct token_t, 8 free function(s); struct tokenizer_ops_t, 3 free function(s); struct analysis_result_t, 2 free function(s)

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

use std::cell::{Cell, RefCell};
use std::ffi::CStr;
use std::io::{self, Read, Write};

pub type token_type_t = u32;

pub const TOKEN_EOF: token_type_t = 0;
pub const TOKEN_WORD: token_type_t = 1;
pub const TOKEN_NUMBER: token_type_t = 2;
pub const TOKEN_PUNCTUATION: token_type_t = 3;
pub const TOKEN_WHITESPACE: token_type_t = 4;
pub const TOKEN_NEWLINE: token_type_t = 5;
pub const TOKEN_IDENTIFIER: token_type_t = 6;
pub const TOKEN_KEYWORD: token_type_t = 7;
pub const TOKEN_OPERATOR: token_type_t = 8;
pub const TOKEN_STRING: token_type_t = 9;
pub const TOKEN_COMMENT: token_type_t = 10;
pub const TOKEN_ERROR: token_type_t = 11;

pub const MAX_TOKEN_LENGTH: usize = 256;
pub const MAX_BUFFER_SIZE: usize = 8192;
pub const MAX_INPUT_SIZE: usize = 4096;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct token_t {
    pub type_0: token_type_t,
    pub value: [i8; 256],
    pub length: usize,
    pub line: i32,
    pub column: i32,
}

pub type tokenizer_next_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_peek_fn = Option<unsafe extern "C" fn() -> token_t>;
pub type tokenizer_reset_fn = Option<unsafe extern "C" fn() -> ()>;
pub type tokenizer_load_fn = Option<unsafe extern "C" fn(*const i8) -> i32>;
pub type tokenizer_get_stats_fn =
    Option<unsafe extern "C" fn(*mut usize, *mut usize, *mut usize) -> ()>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tokenizer_ops_t {
    pub next_token: tokenizer_next_fn,
    pub peek_token: tokenizer_peek_fn,
    pub reset: tokenizer_reset_fn,
    pub load_text: tokenizer_load_fn,
    pub get_stats: tokenizer_get_stats_fn,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct analysis_result_t {
    pub word_count: usize,
    pub number_count: usize,
    pub keyword_count: usize,
    pub operator_count: usize,
    pub comment_count: usize,
    pub string_count: usize,
    pub line_count: usize,
    pub char_count: usize,
}

// ===================== tokenizer =====================

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static POS: Cell<usize> = const { Cell::new(0) };
    static LINE: Cell<i32> = const { Cell::new(1) };
    static COL: Cell<i32> = const { Cell::new(1) };
    static TOTAL_TOKENS: Cell<usize> = const { Cell::new(0) };
    static TOTAL_LINES: Cell<usize> = const { Cell::new(0) };
    static TOTAL_CHARS: Cell<usize> = const { Cell::new(0) };

    static LOOKAHEAD: Cell<token_t> = const {
        Cell::new(token_t { type_0: TOKEN_EOF, value: [0; 256], length: 0, line: 0, column: 0 })
    };
    static LOOKAHEAD_VALID: Cell<bool> = const { Cell::new(false) };

    // Match original C2Rust: num_keywords starts at 0 and is never set.
    static NUM_KEYWORDS: Cell<i32> = const { Cell::new(0) };
}

const KEYWORDS: [&str; 31] = [
    "if", "else", "while", "for", "return", "int", "char", "float", "double", "void", "struct",
    "typedef", "const", "static", "extern", "auto", "register", "sizeof", "break", "continue",
    "switch", "case", "default", "do", "goto", "enum", "union", "signed", "unsigned", "long",
    "short",
];

fn is_space_not_newline(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | 0x0b | 0x0c | b'\r')
}
fn is_alpha(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b) || (b'a'..=b'z').contains(&b)
}
fn is_digit(b: u8) -> bool {
    (b'0'..=b'9').contains(&b)
}
fn is_alnum(b: u8) -> bool {
    is_alpha(b) || is_digit(b)
}

fn peek_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            0
        } else {
            buf[pos]
        }
    })
}
fn peek_next_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos + 1 >= buf.len() {
            0
        } else {
            buf[pos + 1]
        }
    })
}

fn advance_char() -> u8 {
    INPUT.with_borrow(|buf| {
        let pos = POS.get();
        if pos >= buf.len() {
            return 0;
        }
        POS.set(pos + 1);
        let c = buf[pos];
        TOTAL_CHARS.set(TOTAL_CHARS.get() + 1);
        if c == b'\n' {
            LINE.set(LINE.get() + 1);
            COL.set(1);
            TOTAL_LINES.set(TOTAL_LINES.get() + 1);
        } else {
            COL.set(COL.get() + 1);
        }
        c
    })
}

fn skip_whitespace() {
    while peek_char() != 0 && is_space_not_newline(peek_char()) {
        advance_char();
    }
}

fn create_token(type_0: token_type_t, bytes: &[u8]) -> token_t {
    let mut token = token_t {
        type_0,
        value: [0; 256],
        length: 0,
        line: LINE.get(),
        column: 0,
    };

    let len = bytes.len().min(MAX_TOKEN_LENGTH - 1);
    token.length = len;
    for (i, &b) in bytes.iter().take(len).enumerate() {
        token.value[i] = b as i8;
    }
    token.value[len] = 0;

    token.column = (COL.get() as usize).saturating_sub(len) as i32;
    TOTAL_TOKENS.set(TOTAL_TOKENS.get() + 1);
    token
}

fn is_keyword(s: &str) -> bool {
    let n = NUM_KEYWORDS.get().max(0) as usize;
    KEYWORDS.iter().take(n).any(|&k| k == s)
}

fn scan_word() -> token_t {
    let mut buf = Vec::with_capacity(64);
    while peek_char() != 0
        && (is_alnum(peek_char()) || peek_char() == b'_')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        buf.push(advance_char());
    }
    let s = std::str::from_utf8(&buf).unwrap_or("");
    if is_keyword(s) {
        create_token(TOKEN_KEYWORD, &buf)
    } else {
        create_token(TOKEN_IDENTIFIER, &buf)
    }
}

fn scan_number() -> token_t {
    let mut buf = Vec::with_capacity(32);
    let mut has_decimal = false;
    while peek_char() != 0
        && (is_digit(peek_char()) || peek_char() == b'.')
        && buf.len() < MAX_TOKEN_LENGTH - 1
    {
        if peek_char() == b'.' {
            if has_decimal {
                break;
            }
            has_decimal = true;
        }
        buf.push(advance_char());
    }
    create_token(TOKEN_NUMBER, &buf)
}

fn scan_string() -> token_t {
    let mut buf = Vec::with_capacity(64);
    let quote = advance_char();
    buf.push(quote);

    while peek_char() != 0
        && peek_char() != quote
        && peek_char() != b'\n'
        && buf.len() < MAX_TOKEN_LENGTH - 2
    {
        if peek_char() == b'\\' {
            buf.push(advance_char());
            if peek_char() != 0 {
                buf.push(advance_char());
            }
        } else {
            buf.push(advance_char());
        }
    }
    if peek_char() == quote {
        buf.push(advance_char());
    }
    create_token(TOKEN_STRING, &buf)
}

fn scan_comment() -> token_t {
    let mut buf = Vec::with_capacity(128);
    buf.push(advance_char()); // '/'

    if peek_char() == b'/' {
        buf.push(advance_char());
        while peek_char() != 0 && peek_char() != b'\n' && buf.len() < MAX_TOKEN_LENGTH - 1 {
            buf.push(advance_char());
        }
    } else if peek_char() == b'*' {
        buf.push(advance_char());
        while peek_char() != 0 && buf.len() < MAX_TOKEN_LENGTH - 2 {
            if peek_char() == b'*' {
                buf.push(advance_char());
                if peek_char() == b'/' {
                    buf.push(advance_char());
                    break;
                }
            } else {
                buf.push(advance_char());
            }
        }
    }
    create_token(TOKEN_COMMENT, &buf)
}

fn scan_operator() -> token_t {
    let mut buf = Vec::with_capacity(2);
    let c = peek_char();
    buf.push(advance_char());
    let next = peek_char();

    let two = matches!(
        (c, next),
        (b'=', b'=')
            | (b'!', b'=')
            | (b'<', b'=')
            | (b'>', b'=')
            | (b'&', b'&')
            | (b'|', b'|')
            | (b'+', b'+')
            | (b'-', b'-')
            | (b'-', b'>')
            | (b'<', b'<')
            | (b'>', b'>')
    );
    if two {
        buf.push(advance_char());
    }
    create_token(TOKEN_OPERATOR, &buf)
}

fn is_operator_char(c: u8) -> bool {
    matches!(
        c,
        b'+' | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'='
            | b'<'
            | b'>'
            | b'!'
            | b'&'
            | b'|'
            | b'^'
            | b'~'
            | b'?'
            | b':'
    )
}
fn is_punct_char(c: u8) -> bool {
    matches!(c, b'(' | b')' | b'{' | b'}' | b'[' | b']' | b';' | b',' | b'.')
}

pub unsafe extern "C" fn tokenizer_next_token() -> token_t {
    if LOOKAHEAD_VALID.get() {
        LOOKAHEAD_VALID.set(false);
        return LOOKAHEAD.get();
    }

    skip_whitespace();

    if peek_char() == 0 {
        return create_token(TOKEN_EOF, &[]);
    }

    let c = peek_char();

    if c == b'\n' {
        let b = advance_char();
        return create_token(TOKEN_NEWLINE, &[b]);
    }

    if is_alpha(c) || c == b'_' {
        return scan_word();
    }

    if is_digit(c) {
        return scan_number();
    }

    if c == b'"' || c == b'\'' {
        return scan_string();
    }

    // Match original intent: detect comment by looking at next char.
    if c == b'/' && (peek_next_char() == b'/' || peek_next_char() == b'*') {
        return scan_comment();
    }

    if is_operator_char(c) {
        return scan_operator();
    }

    if is_punct_char(c) {
        let b = advance_char();
        return create_token(TOKEN_PUNCTUATION, &[b]);
    }

    let b = advance_char();
    create_token(TOKEN_ERROR, &[b])
}

pub unsafe extern "C" fn tokenizer_peek_token() -> token_t {
    if !LOOKAHEAD_VALID.get() {
        LOOKAHEAD.set(tokenizer_next_token());
        LOOKAHEAD_VALID.set(true);
    }
    LOOKAHEAD.get()
}

pub extern "C" fn tokenizer_reset() {
    POS.set(0);
    LINE.set(1);
    COL.set(1);
    LOOKAHEAD_VALID.set(false);
}

pub unsafe extern "C" fn tokenizer_load_text(text: *const i8) -> i32 {
    if text.is_null() {
        return -1;
    }
    let bytes = CStr::from_ptr(text).to_bytes();
    if bytes.len() >= MAX_BUFFER_SIZE {
        eprintln!("Error: Input text too large");
        return -1;
    }

    INPUT.with_borrow_mut(|buf| {
        buf.clear();
        buf.extend_from_slice(bytes);
    });

    tokenizer_reset();
    0
}

pub unsafe extern "C" fn tokenizer_get_stats(
    lines: *mut usize,
    tokens: *mut usize,
    chars: *mut usize,
) {
    if !lines.is_null() {
        *lines = TOTAL_LINES.get();
    }
    if !tokens.is_null() {
        *tokens = TOTAL_TOKENS.get();
    }
    if !chars.is_null() {
        *chars = TOTAL_CHARS.get();
    }
}

pub extern "C" fn get_tokenizer_ops() -> tokenizer_ops_t {
    tokenizer_ops_t {
        next_token: Some(tokenizer_next_token),
        peek_token: Some(tokenizer_peek_token),
        reset: Some(tokenizer_reset),
        load_text: Some(tokenizer_load_text),
        get_stats: Some(tokenizer_get_stats),
    }
}

// ===================== analyzer =====================

thread_local! {
    static ANALYZER_OPS: Cell<tokenizer_ops_t> = const {
        Cell::new(tokenizer_ops_t { next_token: None, peek_token: None, reset: None, load_text: None, get_stats: None })
    };
    static INITIALIZED: Cell<bool> = const { Cell::new(false) };
    static TOKEN_TYPE_COUNTS: RefCell<[i32; 20]> = const { RefCell::new([0; 20]) };

    static COMMON_WORDS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static COMMON_WORD_COUNTS: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

pub extern "C" fn analyzer_init(ops: tokenizer_ops_t) {
    ANALYZER_OPS.set(ops);
    INITIALIZED.set(true);
    TOKEN_TYPE_COUNTS.with_borrow_mut(|c| c.fill(0));
    COMMON_WORDS.with_borrow_mut(|w| w.clear());
    COMMON_WORD_COUNTS.with_borrow_mut(|c| c.clear());
}

fn cstr_from_i8_buf(buf: &[i8]) -> &CStr {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
}

fn track_word(word_c: &[i8]) {
    let word = cstr_from_i8_buf(word_c).to_string_lossy().into_owned();
    if word.is_empty() {
        return;
    }

    COMMON_WORDS.with_borrow_mut(|words| {
        COMMON_WORD_COUNTS.with_borrow_mut(|counts| {
            if let Some(idx) = words.iter().position(|w| w == &word) {
                counts[idx] += 1;
                return;
            }
            if words.len() < 100 {
                words.push(word);
                counts.push(1);
            }
        })
    });
}

pub(crate) unsafe fn analyze_text_internal(text: &mut [i8]) -> analysis_result_t {
    let mut result = analysis_result_t {
        word_count: 0,
        number_count: 0,
        keyword_count: 0,
        operator_count: 0,
        comment_count: 0,
        string_count: 0,
        line_count: 0,
        char_count: 0,
    };

    if !INITIALIZED.get() {
        eprintln!("Error: Analyzer not initialized");
        return result;
    }

    let ops = ANALYZER_OPS.get();
    if ops
        .load_text
        .expect("non-null function pointer")(text.as_mut_ptr())
        != 0
    {
        eprintln!("Error: Failed to load text");
        return result;
    }

    loop {
        let token = ops.next_token.expect("non-null function pointer")();
        if token.type_0 == TOKEN_EOF {
            break;
        }

        TOKEN_TYPE_COUNTS.with_borrow_mut(|counts| {
            let idx = token.type_0 as usize;
            if idx < counts.len() {
                counts[idx] += 1;
            }
        });

        match token.type_0 {
            TOKEN_WORD | TOKEN_IDENTIFIER => {
                result.word_count = result.word_count.wrapping_add(1);
                track_word(&token.value);
            }
            TOKEN_NUMBER => result.number_count = result.number_count.wrapping_add(1),
            TOKEN_KEYWORD => result.keyword_count = result.keyword_count.wrapping_add(1),
            TOKEN_OPERATOR => result.operator_count = result.operator_count.wrapping_add(1),
            TOKEN_COMMENT => result.comment_count = result.comment_count.wrapping_add(1),
            TOKEN_STRING => result.string_count = result.string_count.wrapping_add(1),
            TOKEN_NEWLINE => result.line_count = result.line_count.wrapping_add(1),
            _ => {}
        }
    }

    let mut lines: usize = 0;
    let mut tokens: usize = 0;
    let mut chars: usize = 0;
    ops.get_stats
        .expect("non-null function pointer")(&mut lines, &mut tokens, &mut chars);
    result.line_count = lines;
    result.char_count = chars;
    result
}

pub unsafe extern "C" fn print_token_distribution() {
    print!("\n=== Token Distribution ===\n");
    let token_names: [&str; 12] = [
        "EOF",
        "WORD",
        "NUMBER",
        "PUNCTUATION",
        "WHITESPACE",
        "NEWLINE",
        "IDENTIFIER",
        "KEYWORD",
        "OPERATOR",
        "STRING",
        "COMMENT",
        "ERROR",
    ];

    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        for i in 0..12 {
            if counts[i] > 0 {
                println!("{}: {}", token_names[i], counts[i]);
            }
        }
    });

    print!("\n=== Most Common Words ===\n");

    let mut pairs: Vec<(String, i32)> = Vec::new();
    COMMON_WORDS.with_borrow(|words| {
        COMMON_WORD_COUNTS.with_borrow(|counts| {
            for (w, &c) in words.iter().zip(counts.iter()) {
                pairs.push((w.clone(), c));
            }
        })
    });
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let limit = pairs.len().min(10);
    for (i, (w, c)) in pairs.into_iter().take(limit).enumerate() {
        println!("{}. {}: {} times", i + 1, w, c);
    }
}

pub extern "C" fn calculate_complexity_score() -> i32 {
    let mut score: i32 = 0;
    TOKEN_TYPE_COUNTS.with_borrow(|counts| {
        score += counts[TOKEN_KEYWORD as usize] * 2;
        score += counts[TOKEN_OPERATOR as usize];
        score += counts[TOKEN_PUNCTUATION as usize] / 10;
  
// ... (truncated) ...
```

**Entity:** tokenizer (thread-local globals: INPUT/POS/LINE/COL/LOOKAHEAD/LOOKAHEAD_VALID/TOTAL_*)

**States:** NoInputOrStaleState, LoadedAndReset, Scanning(NoLookahead), Scanning(HasLookahead)

**Transitions:**
- NoInputOrStaleState -> LoadedAndReset via tokenizer_load_text() (calls tokenizer_reset())
- LoadedAndReset -> Scanning(NoLookahead) via tokenizer_next_token()
- Scanning(NoLookahead) -> Scanning(HasLookahead) via tokenizer_peek_token() setting LOOKAHEAD_VALID=true
- Scanning(HasLookahead) -> Scanning(NoLookahead) via tokenizer_next_token() consuming LOOKAHEAD when LOOKAHEAD_VALID is true
- Scanning(*) -> LoadedAndReset via tokenizer_reset()

**Evidence:** thread_local! INPUT: RefCell<Vec<u8>>, POS/LINE/COL Cells hold mutable scanning state globally; thread_local! LOOKAHEAD: Cell<token_t> and LOOKAHEAD_VALID: Cell<bool> implement cached lookahead state; tokenizer_next_token(): `if LOOKAHEAD_VALID.get() { LOOKAHEAD_VALID.set(false); return LOOKAHEAD.get(); }` consumes cached token first; tokenizer_peek_token(): `if !LOOKAHEAD_VALID.get() { LOOKAHEAD.set(tokenizer_next_token()); LOOKAHEAD_VALID.set(true); }` creates cache; tokenizer_load_text(): checks `text.is_null()` and size; then fills INPUT and calls `tokenizer_reset()`; tokenizer_reset(): sets POS/LINE/COL and `LOOKAHEAD_VALID.set(false)`

**Implementation:** Replace thread-local state + free functions with a `struct Tokenizer<S>` that owns the input buffer and position fields. Provide `Tokenizer<Unloaded>::load(self, &CStr) -> Result<Tokenizer<Loaded>, ...>` and methods `next_token(&mut self)`/`peek_token(&mut self)` only on `Tokenizer<Loaded>`. Model lookahead as an internal `Option<token_t>` field instead of a separate global boolean, eliminating cross-call hidden state.

---

