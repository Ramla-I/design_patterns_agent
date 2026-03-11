# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 7
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 1
- **Precondition**: 5
- **Protocol**: 1
- **Modules analyzed**: 1349

## State Machine Invariants

### 6. ConcatTreesHelper::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-39`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Helper has collected no TokenTree items; build/append avoid invoking concatenation and either produce an empty TokenStream or no-op.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)


/// Non-generic helper for implementing `FromIterator<TokenTree>` and
/// `Extend<TokenTree>` with less monomorphization in calling crates.
struct ConcatTreesHelper {
    trees: Vec<
        bridge::TokenTree<
            bridge::client::TokenStream,
            bridge::client::Span,
            bridge::client::Symbol,
        >,
    >,
}

impl ConcatTreesHelper {
    fn new(capacity: usize) -> Self {
        ConcatTreesHelper { trees: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, tree: TokenTree) {
        self.trees.push(tree_to_bridge_tree(tree));
    }

    fn build(self) -> TokenStream {
        if self.trees.is_empty() {
            TokenStream(None)
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_trees(None, self.trees)))
        }
    }

    fn append_to(self, stream: &mut TokenStream) {
        if self.trees.is_empty() {
            return;
        }
        stream.0 = Some(BridgeMethods::ts_concat_trees(stream.0.take(), self.trees))
    }
}

```

**Entity:** ConcatTreesHelper

**State:** Empty

**State invariants:**
- trees.is_empty() == true
- build(self) constructs TokenStream(None) (an empty stream)
- append_to(self, stream) returns early without modifying stream

**Transitions:**
- Empty -> NonEmpty via push(tree)
- Empty -> Consumed via build(self)
- Empty -> Consumed via append_to(self, &mut TokenStream)

**Evidence:** line 6: struct ConcatTreesHelper { trees: Vec<...> } — emptiness tracked at runtime via Vec length; line 26: if self.trees.is_empty() — branch on runtime emptiness; line 27: TokenStream(None) — build returns empty stream when helper is empty; line 34: if self.trees.is_empty() { return; } — early return in append_to when empty

**Implementation:** Model as ConcatTreesHelper<S> with S ∈ {Empty, NonEmpty}. new() -> ConcatTreesHelper<Empty>; push(self, tree) -> ConcatTreesHelper<NonEmpty>; push on NonEmpty returns Self; provide build()/append_to() only for ConcatTreesHelper<NonEmpty> (and optionally a trivial build_empty() for Empty) to remove runtime emptiness checks.

---

## Precondition Invariants

### 11. __rust_panic_cleanup::Unreachable

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: This cleanup entry point must never be invoked in the panic=abort runtime; it immediately panics with unreachable! if called.

**Evidence**:

```rust
//! Implementation of Rust panics via process aborts
//!
//! When compared to the implementation via unwinding, this crate is *much*
//! simpler! That being said, it's not quite as versatile, but here goes!

#![no_std]
#![unstable(feature = "panic_abort", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![panic_runtime]
#![feature(panic_runtime)]
#![feature(std_internals)]
#![feature(staged_api)]
#![feature(rustc_attrs)]
#![allow(internal_features)]

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "zkvm")]
mod zkvm;

use core::any::Any;
use core::panic::PanicPayload;

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(_: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unreachable!()
}

// "Leak" the payload and shim to the relevant abort on the platform in question.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(_payload: &mut dyn PanicPayload) -> u32 {
    // Android has the ability to attach a message as part of the abort.
    #[cfg(target_os = "android")]
    unsafe {
        android::android_set_abort_message(_payload);
    }
    #[cfg(target_os = "zkvm")]
    unsafe {
        zkvm::zkvm_set_abort_message(_payload);
    }

    unsafe extern "Rust" {
        // This is defined in std::rt.
        #[rustc_std_internal_symbol]
        safe fn __rust_abort() -> !;
    }

    __rust_abort()
}

// This... is a bit of an oddity. The tl;dr; is that this is required to link
// correctly, the longer explanation is below.
//
// Right now the binaries of core/std that we ship are all compiled with
// `-C panic=unwind`. This is done to ensure that the binaries are maximally
// compatible with as many situations as possible. The compiler, however,
// requires a "personality function" for all functions compiled with `-C
// panic=unwind`. This personality function is hardcoded to the symbol
// `rust_eh_personality` and is defined by the `eh_personality` lang item.
//
// So... why not just define that lang item here? Good question! The way that
// panic runtimes are linked in is actually a little subtle in that they're
// "sort of" in the compiler's crate store, but only actually linked if another
// isn't actually linked. This ends up meaning that both this crate and the
// panic_unwind crate can appear in the compiler's crate store, and if both
// define the `eh_personality` lang item then that'll hit an error.
//
// To handle this the compiler only requires the `eh_personality` is defined if
// the panic runtime being linked in is the unwinding runtime, and otherwise
// it's not required to be defined (rightfully so). In this case, however, this
// library just defines this symbol so there's at least some personality
// somewhere.
//
// Essentially this symbol is just defined to get wired up to core/std
// binaries, but it should never be called as we don't link in an unwinding
// runtime at all.
pub mod personalities {
    // In the past this module used to contain stubs for the personality
    // functions of various platforms, but these where removed when personality
    // functions were moved to std.

    // This corresponds to the `eh_catch_typeinfo` lang item
    // that's only used on Emscripten currently.
    //
    // Since panics don't generate exceptions and foreign exceptions are
    // currently UB with -C panic=abort (although this may be subject to
    // change), any catch_unwind calls will never use this typeinfo.
    #[rustc_std_internal_symbol]
    #[allow(non_upper_case_globals)]
    #[cfg(target_os = "emscripten")]
    static rust_eh_catch_typeinfo: [usize; 2] = [0; 2];
}

```

**Entity:** __rust_panic_cleanup

**State:** Unreachable

**State invariants:**
- Function is not a valid code path under panic=abort
- Any call indicates a violated precondition

**Evidence:** lines 27-29: function body is just unreachable!(), asserting it should never be called

**Implementation:** Statically prevent this path by removing it from callable surfaces under panic=abort or by changing its signature to return ! to encode that no valid return is possible. Alternatively, gate any potential callers behind a type-level marker indicating unwinding support is disabled.

---

### 19. Punct::Valid punctuation character

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-78`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Punct can only be constructed with a language-permitted punctuation character; construction with any other char panics. This also implicitly requires the char to be ASCII so that casting to u8 is lossless.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// forms of `Spacing` returned.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub struct Punct(bridge::Punct<bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Punct {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Punct {}


// ... (other code) ...

    Alone,
}

impl Punct {
    /// Creates a new `Punct` from the given character and spacing.
    /// The `ch` argument must be a valid punctuation character permitted by the language,
    /// otherwise the function will panic.
    ///
    /// The returned `Punct` will have the default span of `Span::call_site()`
    /// which can be further configured with the `set_span` method below.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new(ch: char, spacing: Spacing) -> Punct {
        const LEGAL_CHARS: &[char] = &[
            '=', '<', '>', '!', '~', '+', '-', '*', '/', '%', '^', '&', '|', '@', '.', ',', ';',
            ':', '#', '$', '?', '\'',
        ];
        if !LEGAL_CHARS.contains(&ch) {
            panic!("unsupported character `{:?}`", ch);
        }
        Punct(bridge::Punct {
            ch: ch as u8,
            joint: spacing == Spacing::Joint,
            span: Span::call_site().0,
        })
    }

    /// Returns the value of this punctuation character as `char`.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn as_char(&self) -> char {
        self.0.ch as char
    }

    /// Returns the spacing of this punctuation character, indicating whether it can be potentially
    /// combined into a multi-character operator with the following token (`Joint`), or whether the
    /// operator has definitely ended (`Alone`).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn spacing(&self) -> Spacing {
        if self.0.joint { Spacing::Joint } else { Spacing::Alone }
    }

    /// Returns the span for this punctuation character.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        Span(self.0.span)
    }

    /// Configure the span for this punctuation character.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        self.0.span = span.0;
    }
}

// ... (other code) ...

}

#[stable(feature = "proc_macro_punct_eq", since = "1.50.0")]
impl PartialEq<char> for Punct {
    fn eq(&self, rhs: &char) -> bool {
        self.as_char() == *rhs
    }
}

```

**Entity:** Punct

**State:** ValidChar

**State invariants:**
- ch is a member of LEGAL_CHARS
- ch fits in u8 (ASCII) because Punct::new casts ch as u8
- new() initializes span to Span::call_site()

**Evidence:** line 21-22: doc comment states 'The `ch` argument must be a valid punctuation character ... otherwise the function will panic.'; line 28-34: LEGAL_CHARS whitelist and panic!("unsupported character ...") enforce the precondition at runtime; line 36: ch is cast to u8, relying on the whitelist to ensure ASCII; line 24-25: doc comment states default span is Span::call_site(); line 35-39: constructor sets span: Span::call_site().0

**Implementation:** Introduce a ValidPunctChar newtype with a checked constructor (e.g., TryFrom<char>) that validates membership in LEGAL_CHARS. Change Punct::new to accept ValidPunctChar instead of char, or provide TryFrom<ValidPunctChar> for Punct. This moves the 'valid punctuation' requirement into the type system so invalid chars cannot be passed to Punct::new at compile time (callers must first construct/obtain a ValidPunctChar).

---

### 24. Span::Same-file joinability

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-170`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: join() only succeeds when both spans originate from the same file.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// A region of source code, along with macro expansion information.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Copy, Clone)]
pub struct Span(bridge::client::Span);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Span {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Span {}


// ... (other code) ...

    };
}

impl Span {
    /// A span that resolves at the macro definition site.
    #[unstable(feature = "proc_macro_def_site", issue = "54724")]
    pub fn def_site() -> Span {
        Span(bridge::client::Span::def_site())
    }

    /// The span of the invocation of the current procedural macro.
    /// Identifiers created with this span will be resolved as if they were written
    /// directly at the macro call location (call-site hygiene) and other code
    /// at the macro call site will be able to refer to them as well.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn call_site() -> Span {
        Span(bridge::client::Span::call_site())
    }

    /// A span that represents `macro_rules` hygiene, and sometimes resolves at the macro
    /// definition site (local variables, labels, `$crate`) and sometimes at the macro
    /// call site (everything else).
    /// The span location is taken from the call-site.
    #[stable(feature = "proc_macro_mixed_site", since = "1.45.0")]
    pub fn mixed_site() -> Span {
        Span(bridge::client::Span::mixed_site())
    }

    /// The `Span` for the tokens in the previous macro expansion from which
    /// `self` was generated from, if any.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn parent(&self) -> Option<Span> {
        BridgeMethods::span_parent(self.0).map(Span)
    }

    /// The span for the origin source code that `self` was generated from. If
    /// this `Span` wasn't generated from other macro expansions then the return
    /// value is the same as `*self`.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn source(&self) -> Span {
        Span(BridgeMethods::span_source(self.0))
    }

    /// Returns the span's byte position range in the source file.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn byte_range(&self) -> Range<usize> {
        BridgeMethods::span_byte_range(self.0)
    }

    /// Creates an empty span pointing to directly before this span.
    #[stable(feature = "proc_macro_span_location", since = "1.88.0")]
    pub fn start(&self) -> Span {
        Span(BridgeMethods::span_start(self.0))
    }

    /// Creates an empty span pointing to directly after this span.
    #[stable(feature = "proc_macro_span_location", since = "1.88.0")]
    pub fn end(&self) -> Span {
        Span(BridgeMethods::span_end(self.0))
    }

    /// The one-indexed line of the source file where the span starts.
    ///
    /// To obtain the line of the span's end, use `span.end().line()`.
    #[stable(feature = "proc_macro_span_location", since = "1.88.0")]
    pub fn line(&self) -> usize {
        BridgeMethods::span_line(self.0)
    }

    /// The one-indexed column of the source file where the span starts.
    ///
    /// To obtain the column of the span's end, use `span.end().column()`.
    #[stable(feature = "proc_macro_span_location", since = "1.88.0")]
    pub fn column(&self) -> usize {
        BridgeMethods::span_column(self.0)
    }

    /// The path to the source file in which this span occurs, for display purposes.
    ///
    /// This might not correspond to a valid file system path.
    /// It might be remapped (e.g. `"/src/lib.rs"`) or an artificial path (e.g. `"<command line>"`).
    #[stable(feature = "proc_macro_span_file", since = "1.88.0")]
    pub fn file(&self) -> String {
        BridgeMethods::span_file(self.0)
    }

    /// The path to the source file in which this span occurs on the local file system.
    ///
    /// This is the actual path on disk. It is unaffected by path remapping.
    ///
    /// This path should not be embedded in the output of the macro; prefer `file()` instead.
    #[stable(feature = "proc_macro_span_file", since = "1.88.0")]
    pub fn local_file(&self) -> Option<PathBuf> {
        BridgeMethods::span_local_file(self.0).map(PathBuf::from)
    }

    /// Creates a new span encompassing `self` and `other`.
    ///
    /// Returns `None` if `self` and `other` are from different files.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn join(&self, other: Span) -> Option<Span> {
        BridgeMethods::span_join(self.0, other.0).map(Span)
    }

    /// Creates a new span with the same line/column information as `self` but
    /// that resolves symbols as though it were at `other`.
    #[stable(feature = "proc_macro_span_resolved_at", since = "1.45.0")]
    pub fn resolved_at(&self, other: Span) -> Span {
        Span(BridgeMethods::span_resolved_at(self.0, other.0))
    }

    /// Creates a new span with the same name resolution behavior as `self` but
    /// with the line/column information of `other`.
    #[stable(feature = "proc_macro_span_located_at", since = "1.45.0")]
    pub fn located_at(&self, other: Span) -> Span {
        other.resolved_at(*self)
    }

    /// Compares two spans to see if they're equal.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn eq(&self, other: &Span) -> bool {
        self.0 == other.0
    }

    /// Returns the source text behind a span. This preserves the original source
    /// code, including spaces and comments. It only returns a result if the span
    /// corresponds to real source code.
    ///
    /// Note: The observable result of a macro should only rely on the tokens and
    /// not on this source text. The result of this function is a best effort to
    /// be used for diagnostics only.
    #[stable(feature = "proc_macro_source_text", since = "1.66.0")]
    pub fn source_text(&self) -> Option<String> {
        BridgeMethods::span_source_text(self.0)
    }

    // Used by the implementation of `Span::quote`
    #[doc(hidden)]
    #[unstable(feature = "proc_macro_internals", issue = "27812")]
    pub fn save_span(&self) -> usize {
        BridgeMethods::span_save_span(self.0)
    }

    // Used by the implementation of `Span::quote`
    #[doc(hidden)]
    #[unstable(feature = "proc_macro_internals", issue = "27812")]
    pub fn recover_proc_macro_span(id: usize) -> Span {
        Span(BridgeMethods::span_recover_proc_macro_span(id))
    }

    diagnostic_method!(error, Level::Error);
    diagnostic_method!(warning, Level::Warning);
    diagnostic_method!(note, Level::Note);
    diagnostic_method!(help, Level::Help);
}

```

**Entity:** Span

**State:** SameFile

**State invariants:**
- self and other have the same underlying file identity
- join(self, other) returns Some(..) iff SameFile holds

**Evidence:** line 114: comment explicitly states 'Returns `None` if `self` and `other` are from different files.'; line 116: signature pub fn join(&self, other: Span) -> Option<Span>

**Implementation:** Brand spans by file: struct Span<F>(.., PhantomData<F>); join(&self, other: &Span<F>) -> Span<F>. Provide a fallible downcast fn try_into_file_branded(self) -> Result<Span<F>, _> when a file id is available.

---

### 43. Static-only input precondition

**Location**: `/data/rust/library/test/src/lib.rs:1-448`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: These entry points require all provided tests to be static; any dynamic test triggers a panic at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct FilteredTests, impl FilteredTests (3 methods)

//! Support code for rustc's built in unit-test and micro-benchmarking
//! framework.
//!
//! Almost all user code will only be interested in `Bencher` and
//! `black_box`. All other interactions (such as writing tests and
//! benchmarks themselves) should be done via the `#[test]` and
//! `#[bench]` attributes.
//!
//! See the [Testing Chapter](../book/ch11-00-testing.html) of the book for more
//! details.

// Currently, not much of this is meant for users. It is intended to
// support the simplest interface possible for representing and
// running tests while providing a base that other test frameworks may
// build off of.

#![unstable(feature = "test", issue = "50297")]
#![doc(test(attr(deny(warnings))))]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(file_buffered)]
#![feature(internal_output_capture)]
#![feature(io_const_error)]
#![feature(staged_api)]
#![feature(process_exitcode_internals)]
#![feature(panic_can_unwind)]
#![cfg_attr(test, feature(test))]
#![feature(thread_spawn_hook)]
#![allow(internal_features)]
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]

pub use cli::TestOpts;

pub use self::ColorConfig::*;
pub use self::bench::{Bencher, black_box};
pub use self::console::run_tests_console;
pub use self::options::{ColorConfig, Options, OutputFormat, RunIgnored, ShouldPanic};
pub use self::types::TestName::*;
pub use self::types::*;

// Module to be used by rustc to compile tests in libtest
pub mod test {
    pub use crate::bench::Bencher;
    pub use crate::cli::{TestOpts, parse_opts};
    pub use crate::helpers::metrics::{Metric, MetricMap};
    pub use crate::options::{Options, RunIgnored, RunStrategy, ShouldPanic};
    pub use crate::test_result::{TestResult, TrFailed, TrFailedMsg, TrIgnored, TrOk};
    pub use crate::time::{TestExecTime, TestTimeOptions};
    pub use crate::types::{
        DynTestFn, DynTestName, StaticBenchFn, StaticTestFn, StaticTestName, TestDesc,
        TestDescAndFn, TestId, TestName, TestType,
    };
    pub use crate::{assert_test_result, filter_tests, run_test, test_main, test_main_static};
}

use std::collections::VecDeque;
use std::io::prelude::Write;
use std::mem::ManuallyDrop;
use std::panic::{self, AssertUnwindSafe, PanicHookInfo, catch_unwind};
use std::process::{self, Command, Termination};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, io, thread};

pub mod bench;
mod cli;
mod console;
mod event;
mod formatters;
mod helpers;
mod options;
pub mod stats;
mod term;
mod test_result;
mod time;
mod types;

#[cfg(test)]
mod tests;

use core::any::Any;

use event::{CompletedTest, TestEvent};
use helpers::concurrency::get_concurrency;
use helpers::shuffle::{get_shuffle_seed, shuffle_tests};
use options::RunStrategy;
use test_result::*;
use time::TestExecTime;

/// Process exit code to be used to indicate test failures.
pub const ERROR_EXIT_CODE: i32 = 101;

const SECONDARY_TEST_INVOKER_VAR: &str = "__RUST_TEST_INVOKE";
const SECONDARY_TEST_BENCH_BENCHMARKS_VAR: &str = "__RUST_TEST_BENCH_BENCHMARKS";

// The default console test runner. It accepts the command line
// arguments and a vector of test_descs.
pub fn test_main(args: &[String], tests: Vec<TestDescAndFn>, options: Option<Options>) {
    test_main_with_exit_callback(args, tests, options, || {})
}

pub fn test_main_with_exit_callback<F: FnOnce()>(
    args: &[String],
    tests: Vec<TestDescAndFn>,
    options: Option<Options>,
    exit_callback: F,
) {
    let mut opts = match cli::parse_opts(args) {
        Some(Ok(o)) => o,
        Some(Err(msg)) => {
            eprintln!("error: {msg}");
            process::exit(ERROR_EXIT_CODE);
        }
        None => return,
    };
    if let Some(options) = options {
        opts.options = options;
    }
    if opts.list {
        if let Err(e) = console::list_tests_console(&opts, tests) {
            eprintln!("error: io error when listing tests: {e:?}");
            process::exit(ERROR_EXIT_CODE);
        }
    } else {
        if !opts.nocapture {
            // If we encounter a non-unwinding panic, flush any captured output from the current test,
            // and stop capturing output to ensure that the non-unwinding panic message is visible.
            // We also acquire the locks for both output streams to prevent output from other threads
            // from interleaving with the panic message or appearing after it.
            let builtin_panic_hook = panic::take_hook();
            let hook = Box::new({
                move |info: &'_ PanicHookInfo<'_>| {
                    if !info.can_unwind() {
                        std::mem::forget(std::io::stderr().lock());
                        let mut stdout = ManuallyDrop::new(std::io::stdout().lock());
                        if let Some(captured) = io::set_output_capture(None) {
                            if let Ok(data) = captured.lock() {
                                let _ = stdout.write_all(&data);
                                let _ = stdout.flush();
                            }
                        }
                    }
                    builtin_panic_hook(info);
                }
            });
            panic::set_hook(hook);
            // Use a thread spawning hook to make new threads inherit output capturing.
            std::thread::add_spawn_hook(|_| {
                // Get and clone the output capture of the current thread.
                let output_capture = io::set_output_capture(None);
                io::set_output_capture(output_capture.clone());
                // Set the output capture of the new thread.
                || {
                    io::set_output_capture(output_capture);
                }
            });
        }
        let res = console::run_tests_console(&opts, tests);
        // Prevent Valgrind from reporting reachable blocks in users' unit tests.
        drop(panic::take_hook());
        exit_callback();
        match res {
            Ok(true) => {}
            Ok(false) => process::exit(ERROR_EXIT_CODE),
            Err(e) => {
                eprintln!("error: io error when listing tests: {e:?}");
                process::exit(ERROR_EXIT_CODE);
            }
        }
    }
}

/// A variant optimized for invocation with a static test vector.
/// This will panic (intentionally) when fed any dynamic tests.
///
/// This is the entry point for the main function generated by `rustc --test`
/// when panic=unwind.
pub fn test_main_static(tests: &[&TestDescAndFn]) {
    let args = env::args().collect::<Vec<_>>();
    let owned_tests: Vec<_> = tests.iter().map(make_owned_test).collect();
    test_main(&args, owned_tests, None)
}

/// A variant optimized for invocation with a static test vector.
/// This will panic (intentionally) when fed any dynamic tests.
///
/// Runs tests in panic=abort mode, which involves spawning subprocesses for
/// tests.
///
/// This is the entry point for the main function generated by `rustc --test`
/// when panic=abort.
pub fn test_main_static_abort(tests: &[&TestDescAndFn]) {
    // If we're being run in SpawnedSecondary mode, run the test here. run_test
    // will then exit the process.
    if let Ok(name) = env::var(SECONDARY_TEST_INVOKER_VAR) {
        unsafe {
            env::remove_var(SECONDARY_TEST_INVOKER_VAR);
        }

        // Convert benchmarks to tests if we're not benchmarking.
        let mut tests = tests.iter().map(make_owned_test).collect::<Vec<_>>();
        if env::var(SECONDARY_TEST_BENCH_BENCHMARKS_VAR).is_ok() {
            unsafe {
                env::remove_var(SECONDARY_TEST_BENCH_BENCHMARKS_VAR);
            }
        } else {
            tests = convert_benchmarks_to_tests(tests);
        };

        let test = tests
            .into_iter()
            .find(|test| test.desc.name.as_slice() == name)
            .unwrap_or_else(|| panic!("couldn't find a test with the provided name '{name}'"));
        let TestDescAndFn { desc, testfn } = test;
        match testfn.into_runnable() {
            Runnable::Test(runnable_test) => {
                if runnable_test.is_dynamic() {
                    panic!("only static tests are supported");
                }
                run_test_in_spawned_subprocess(desc, runnable_test);
            }
            Runnable::Bench(_) => {
                panic!("benchmarks should not be executed into child processes")
            }
        }
    }

    let args = env::args().collect::<Vec<_>>();
    let owned_tests: Vec<_> = tests.iter().map(make_owned_test).collect();
    test_main(&args, owned_tests, Some(Options::new().panic_abort(true)))
}

/// Clones static values for putting into a dynamic vector, which test_main()
/// needs to hand out ownership of tests to parallel test runners.
///
/// This will panic when fed any dynamic tests, because they cannot be cloned.
fn make_owned_test(test: &&TestDescAndFn) -> TestDescAndFn {
    match test.testfn {
        StaticTestFn(f) => TestDescAndFn { testfn: StaticTestFn(f), desc: test.desc.clone() },
        StaticBenchFn(f) => TestDescAndFn { testfn: StaticBenchFn(f), desc: test.desc.clone() },
        _ => panic!("non-static tests passed to test::test_main_static"),
    }
}

/// Public API used by rustdoc to display the `total` and `compilation` times in the expected
/// format.
pub fn print_merged_doctests_times(args: &[String], total_time: f64, compilation_time: f64) {
    let opts = match cli::parse_opts(args) {
        Some(Ok(o)) => o,
        Some(Err(msg)) => {
            eprintln!("error: {msg}");
            process::exit(ERROR_EXIT_CODE);
        }
        None => return,
    };
    let mut formatter = console::get_formatter(&opts, 0);
    formatter.write_merged_doctests_times(total_time, compilation_time).unwrap();
}

/// Invoked when unit tests terminate. Returns `Result::Err` if the test is
/// considered a failure. By default, invokes `report()` and checks for a `0`
/// result.
pub fn assert_test_result<T: Termination>(result: T) -> Result<(), String> {
    let code = result.report().to_i32();
    if code == 0 {
        Ok(())
    } else {
        Err(format!(
            "the test returned a termination value with a non-zero status code \
             ({code}) which indicates a failure"
        ))
    }
}

struct FilteredTests {
    tests: Vec<(TestId, TestDescAndFn)>,
    benches: Vec<(TestId, TestDescAndFn)>,
    next_id: usize,
}

impl FilteredTests {
    fn add_bench(&mut self, desc: TestDesc, testfn: TestFn) {
        let test = TestDescAndFn { desc, testfn };
        self.benches.push((TestId(self.next_id), test));
        self.next_id += 1;
    }
    fn add_test(&mut self, desc: TestDesc, testfn: TestFn) {
        let test = TestDescAndFn { desc, testfn };
        self.tests.push((TestId(self.next_id), test));
        self.next_id += 1;
    }
    fn total_len(&self) -> usize {
        self.tests.len() + self.benches.len()
    }
}

pub fn run_tests<F>(
    opts: &TestOpts,
    tests: Vec<TestDescAndFn>,
    mut notify_about_test_event: F,
) -> io::Result<()>
where
    F: FnMut(TestEvent) -> io::Result<()>,
{
    use std::collections::HashMap;
    use std::hash::{BuildHasherDefault, DefaultHasher};
    use std::sync::mpsc::RecvTimeoutError;

    struct RunningTest {
        join_handle: Option<thread::JoinHandle<()>>,
    }

    impl RunningTest {
        fn join(self, completed_test: &mut CompletedTest) {
            if let Some(join_handle) = self.join_handle {
                if let Err(_) = join_handle.join() {
                    if let TrOk = completed_test.result {
                        completed_test.result =
                            TrFailedMsg("panicked after reporting success".to_string());
                    }
                }
            }
        }
    }

    // Use a deterministic hasher
    type TestMap = HashMap<TestId, RunningTest, BuildHasherDefault<DefaultHasher>>;

    struct TimeoutEntry {
        id: TestId,
        desc: TestDesc,
        timeout: Instant,
    }

    let tests_len = tests.len();

    let mut filtered = FilteredTests { tests: Vec::new(), benches: Vec::new(), next_id: 0 };

    let mut filtered_tests = filter_tests(opts, tests);
    if !opts.bench_benchmarks {
        filtered_tests = convert_benchmarks_to_tests(filtered_tests);
    }

    for test in filtered_tests {
        let mut desc = test.desc;
        desc.name = desc.name.with_padding(test.testfn.padding());

        match test.testfn {
            DynBenchFn(_) | StaticBenchFn(_) => {
                filtered.add_bench(desc, test.testfn);
            }
            testfn => {
                filtered.add_test(desc, testfn);
            }
        };
    }

    let filtered_out = tests_len - filtered.total_len();
    let event = TestEvent::TeFilteredOut(filtered_out);
    notify_about_test_event(event)?;

    let shuffle_seed = get_shuffle_seed(opts);

    let event = TestEvent::TeFiltered(filtered.total_len(), shuffle_seed);
    notify_about_test_event(event)?;

    let concurrency = opts.test_threads.unwrap_or_else(get_concurrency);

    let mut remaining = filtered.tests;
    if let Some(shuffle_seed) = shuffle_seed {
        shuffle_tests(shuffle_seed, &mut remaining);
    }
    // Store the tests in a VecDeque so we can efficiently remove the first element to run the
    // tests in the order they were passed (unless shuffled).
    let mut remaining = VecDeque::from(remaining);
    let mut pending = 0;

    let (tx, rx) = channel::<CompletedTest>();
    let run_strategy = if opts.options.panic_abort && !opts.force_run_in_process {
        RunStrategy::SpawnPrimary
    } else {
        RunStrategy::InProcess
    };

    let mut running_tests: TestMap = HashMap::default();
    let mut timeout_queue: VecDeque<TimeoutEntry> = VecDeque::new();

    fn get_timed_out_tests(
        running_tests: &TestMap,
        timeout_queue: &mut VecDeque<TimeoutEntry>,
    ) -> Vec<TestDesc> {
        let now = Instant::now();
        let mut timed_out = Vec::new();
        while let Some(timeout_entry) = timeout_queue.front() {
            if now < timeout_entry.timeout {
                break;
            }
            let timeout_entry = timeout_queue.pop_front().unwrap();
            if running_tests.contains_key(&timeout_entry.id) {
                timed_out.push(timeout_entry.desc);
            }
        }
        timed_out
    }

    fn calc_timeout(timeout_queue: &VecDeque<TimeoutEntry>) -> Option<Duration> {
        timeout_queue.front().map(|&TimeoutEntry { timeout: next_timeout, .. }| {
            let now = Instant::now();
            if next_timeout >= now { next_timeout - now } else { Duration::new(0, 0) }
        })
    }

    if concurrency == 1 {
        while !remaining.is_empty() {
            let (id, test) = remaining.pop_front().unwrap();
            let event = TestEvent::TeWait(test.desc.clone());
            notify_about_test_event(event)?;
            let join_handle = run_test(opts, !opts.run_tests, id, test, run_strategy, tx.clone());
            // Wait for the test to complete.
            let mut completed_test = rx.recv().unwrap();
            RunningTest { join_handle }.join(&mut completed_test);

            let fail_fast = match completed_test.result {
                TrIgnored | TrOk | TrBench(_) => false,
                TrFailed | TrFailedMsg(_) | TrTimedFail => opts.fail_fast,
            };

            let event = TestEvent::TeResult(completed_test);
            notify_about_test_event(event)?;

            if fail_fast {
                return Ok(());
            }
        }
    } else {
        while pending > 0 || !remaining.is_empty() {
            while pending < concurrency && !remaining.is_empty() {
                let (id, test) = remaining.pop_front().unwrap();
                let timeout = time::get_default_test_timeout();
                let desc = test.desc.clone();

                let event = TestEvent::TeWait(desc.clone());
 
// ... (truncated) ...
```

**Entity:** test_main_static + make_owned_test

**State:** StaticOnly

**State invariants:**
- All test functions provided must be StaticTestFn or StaticBenchFn
- Dynamic tests are rejected with a panic
- In spawned secondary abort mode, only static tests are allowed; benches are disallowed

**Evidence:** line 177-182: docstring explicitly states it will panic when fed any dynamic tests; line 241-246: make_owned_test panics with "non-static tests passed to test::test_main_static" on non-static variants; line 221-223: in abort secondary mode, panic!("only static tests are supported") if runnable_test.is_dynamic(); line 226-228: panic!("benchmarks should not be executed into child processes") when encountering Runnable::Bench in child

**Implementation:** Define a StaticOnly<T> wrapper that can only be constructed from StaticTestFn/StaticBenchFn (e.g., via TryFrom with a rejected error for dynamic). Change test_main_static/test_main_static_abort signatures to accept &[&StaticOnly<TestDescAndFn>] so non-static inputs are a compile error.

---

### 21. Punct::Construction precondition

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-411`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Punct can only be constructed from a restricted set of ASCII punctuation characters; otherwise construction panics. The constructor also fixes initial span and spacing-derived jointness.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}


// ... (other code) ...

    None,
}

impl Group {
    /// Creates a new `Group` with the given delimiter and token stream.
    ///
    /// This constructor will set the span for this group to
    /// `Span::call_site()`. To change the span you can use the `set_span`
    /// method below.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new(delimiter: Delimiter, stream: TokenStream) -> Group {
        Group(bridge::Group {
            delimiter,
            stream: stream.0,
            span: bridge::DelimSpan::from_single(Span::call_site().0),
        })
    }

    /// Returns the delimiter of this `Group`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn delimiter(&self) -> Delimiter {
        self.0.delimiter
    }

    /// Returns the `TokenStream` of tokens that are delimited in this `Group`.
    ///
    /// Note that the returned token stream does not include the delimiter
    /// returned above.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn stream(&self) -> TokenStream {
        TokenStream(self.0.stream.clone())
    }

    /// Returns the span for the delimiters of this token stream, spanning the
    /// entire `Group`.
    ///
    /// ```text
    /// pub fn span(&self) -> Span {
    ///            ^^^^^^^
    /// ```
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        Span(self.0.span.entire)
    }

    /// Returns the span pointing to the opening delimiter of this group.
    ///
    /// ```text
    /// pub fn span_open(&self) -> Span {
    ///                 ^
    /// ```
    #[stable(feature = "proc_macro_group_span", since = "1.55.0")]
    pub fn span_open(&self) -> Span {
        Span(self.0.span.open)
    }

    /// Returns the span pointing to the closing delimiter of this group.
    ///
    /// ```text
    /// pub fn span_close(&self) -> Span {
    ///                        ^
    /// ```
    #[stable(feature = "proc_macro_group_span", since = "1.55.0")]
    pub fn span_close(&self) -> Span {
        Span(self.0.span.close)
    }

    /// Configures the span for this `Group`'s delimiters, but not its internal
    /// tokens.
    ///
    /// This method will **not** set the span of all the internal tokens spanned
    /// by this group, but rather it will only set the span of the delimiter
    /// tokens at the level of the `Group`.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        self.0.span = bridge::DelimSpan::from_single(span.0);
    }
}

/// Prints the group as a string that should be losslessly convertible back
/// into the same group (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", TokenStream::from(TokenTree::from(self.clone())))
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Group")
            .field("delimiter", &self.delimiter())
            .field("stream", &self.stream())
            .field("span", &self.span())
            .finish()
    }
}

/// A `Punct` is a single punctuation character such as `+`, `-` or `#`.
///
/// Multi-character operators like `+=` are represented as two instances of `Punct` with different
/// forms of `Spacing` returned.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub struct Punct(bridge::Punct<bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Punct {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Punct {}

/// Indicates whether a `Punct` token can join with the following token
/// to form a multi-character operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Spacing {
    /// A `Punct` token can join with the following token to form a multi-character operator.
    ///
    /// In token streams constructed using proc macro interfaces, `Joint` punctuation tokens can be
    /// followed by any other tokens. However, in token streams parsed from source code, the
    /// compiler will only set spacing to `Joint` in the following cases.
    /// - When a `Punct` is immediately followed by another `Punct` without a whitespace. E.g. `+`
    ///   is `Joint` in `+=` and `++`.
    /// - When a single quote `'` is immediately followed by an identifier without a whitespace.
    ///   E.g. `'` is `Joint` in `'lifetime`.
    ///
    /// This list may be extended in the future to enable more token combinations.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Joint,
    /// A `Punct` token cannot join with the following token to form a multi-character operator.
    ///
    /// `Alone` punctuation tokens can be followed by any other tokens. In token streams parsed
    /// from source code, the compiler will set spacing to `Alone` in all cases not covered by the
    /// conditions for `Joint` above. E.g. `+` is `Alone` in `+ =`, `+ident` and `+()`. In
    /// particular, tokens not followed by anything will be marked as `Alone`.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Alone,
}

impl Punct {
    /// Creates a new `Punct` from the given character and spacing.
    /// The `ch` argument must be a valid punctuation character permitted by the language,
    /// otherwise the function will panic.
    ///
    /// The returned `Punct` will have the default span of `Span::call_site()`
    /// which can be further configured with the `set_span` method below.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new(ch: char, spacing: Spacing) -> Punct {
        const LEGAL_CHARS: &[char] = &[
            '=', '<', '>', '!', '~', '+', '-', '*', '/', '%', '^', '&', '|', '@', '.', ',', ';',
            ':', '#', '$', '?', '\'',
        ];
        if !LEGAL_CHARS.contains(&ch) {
            panic!("unsupported character `{:?}`", ch);
        }
        Punct(bridge::Punct {
            ch: ch as u8,
            joint: spacing == Spacing::Joint,
            span: Span::call_site().0,
        })
    }

    /// Returns the value of this punctuation character as `char`.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn as_char(&self) -> char {
        self.0.ch as char
    }

    /// Returns the spacing of this punctuation character, indicating whether it can be potentially
    /// combined into a multi-character operator with the following token (`Joint`), or whether the
    /// operator has definitely ended (`Alone`).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn spacing(&self) -> Spacing {
        if self.0.joint { Spacing::Joint } else { Spacing::Alone }
    }

    /// Returns the span for this punctuation character.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        Span(self.0.span)
    }

    /// Configure the span for this punctuation character.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        self.0.span = span.0;
    }
}

/// Prints the punctuation character as a string that should be losslessly convertible
/// back into the same character.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Punct")
            .field("ch", &self.as_char())
            .field("spacing", &self.spacing())
            .field("span", &self.span())
            .finish()
    }
}

#[stable(feature = "proc_macro_punct_eq", since = "1.50.0")]
impl PartialEq<char> for Punct {
    fn eq(&self, rhs: &char) -> bool {
        self.as_char() == *rhs
    }
}

#[stable(feature = "proc_macro_punct_eq_flipped", since = "1.52.0")]
impl PartialEq<Punct> for char {
    fn eq(&self, rhs: &Punct) -> bool {
        *self == rhs.as_char()
    }
}

/// An identifier (`ident`).
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Ident(bridge::Ident<bridge::client::Span, bridge::client::Symbol>);

impl Ident {
    /// Creates a new `Ident` with the given `string` as well as the specified
    /// `span`.
    /// The `string` argument must be a valid identifier permitted by the
    /// language (including keywords, e.g. `self` or `fn`). Otherwise, the function will panic.
    ///
    /// The constructed identifier will be NFC-normalized. See the [Reference] for more info.
    ///
    /// Note that `span`, currently in rustc, configures the hygiene information
    /// for this identifier.
    ///
    /// As of this time `Span::call_site()` explicitly opts-in to "call-site" hygiene
    /// meaning that identifiers created with this span will be resolved as if they were written
    /// directly at the location of the macro call, and other code at the macro call site will be
    /// able to refer to them as well.
    ///
    /// Later spans like `Span::def_site()` will allow to opt-in to "definition-site" hygiene
    /// meaning that identifiers created with this span will be resolved at the location of the
    /// macro definition and other code at the macro call site will not be able to refer to them.
    ///
    /// Due to the current importance of hygiene this constructor, unlike other
    /// tokens, requires a `Span` to be specified at construction.
    ///
    /// [Reference]: https://doc.rust-lang.org/nightly/reference/identifiers.html#r-ident.normalization
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new(string: &str, span: Span) -> Ident {
        Ident(bridge::Ident {
            sym: bridge::client::Symbol::new_ident(string, false),
            is_raw: false,
            span: span.0,
        })
    }

    /// Same as `Ident::new`, but creates a raw identifier (`r#ident`).
    /// The `string` argument be a valid identifier permitted by the language
    /// (including keywords, e.g. `fn`). Keywords which are usable in path segments
    /// (e.g. `self`, `super`) are not supported, and will cause a panic.
    #[stable(feature = "proc_macro_raw_ident", since = "1.47.0")]
    pub fn new_raw(string: &str, span: Span) -> Ident {
        Ident(bridge::Ident {
            sym: bridge::client::Symbol::new_ident(string, true),
            is_raw: true,
            span: span.0,
        })
    }

    /// Returns the span of this `Ident`, encompassing the entire string returned
    /// by [`to_string`](ToString::to_string).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        Span(self.0.span)
    }

    /// Configures the span of this `Ident`, possibly changing its hygiene context.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        self.0.span = span.0;
    }
}

/// Prints the identifier as a string that should be losslessly convertible back
/// into the same identifier.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_raw {
            f.write_str("r#")?;
        }
        fmt::Display::fmt(&self.0.sym, f)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ident")
            .field("ident", &self.to_string())
            .field("span", &self.span())
            .finish()
    }
}

/// A literal string (`"hello"`), byte string (`b"hello"`), C string (`c"hello"`),
/// character (`'a'`), byte character (`b'a'`), an integer or floating point number
/// with or without a suffix (`1`, `1u8`, `2.3`, `2.3f32`).
/// Boolean literals like `true` and `false` do not belong here, they are `Ident`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Literal(bridge::Literal<bridge::client::Span, bridge::client::Symbol>);

macro_rules! suffixed_int_literals {
    ($($name:ident => $kind:ident,)*) => ($(
        /// Creates a new suffixed integer literal with the specified value.
        ///
        /// This function will create an integer like `1u32` where the integer
        /// value specified is the first part of the token and the integral is
        /// also suffixed at the end.
        /// Literals created from negative numbers might not survive round-trips through
        /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
        ///
        /// Literals created through this method have the `Span::call_site()`
        /// span by default, which can be configured with the `set_span` method
        /// below.
        #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
        pub fn $name(n: $kind) -> Literal {
            Literal(bridge::Literal {
                kind: bridge::LitKind::Integer,
                symbol: bridge::client::Symbol::new(&n.to_string()),
                suffix: Some(bridge::client::Symbol::new(stringify!($kind))),
                span: Span::call_site().0,
            })
        }
    )*)
}

macro_rules! unsuffixed_int_literals {
    ($($name:ident => $kind:ident,)*) => ($(
        /// Creates a new unsuffixed integer literal with the specified value.
        ///
        /// This function will create an integer like `1` where the integer
        /// value specified is the first part of the token. No suffix is
        /// specified on this token, meaning that invocations like
        /// `Literal::i8_unsuffixed(1)` are equivalent to
        /// `Literal::u32_unsuffixed(1)`.
        /// Literals created from negative numbers might not survive rountrips through
        /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
        ///
        /// Literals created through this method have the `Span::call_site()`
        /// span by default, which can be configured with the `set_span` method
        /// below.
        #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
        pub fn $name(n: $kind) -> Literal {
            Literal(bridge::Literal {
                kind: bridge::LitKind::Integer,
                symbol: bridge::client::Symbol::new(&n.to_string()),
                suffix: None,
                span: Span::call_site().0,
            })
        }
    )*)
}

impl Literal {
    fn new(kind: bridge::LitKind, value: &str, suffix: Option<&str>) -> Self {
        Literal(bridge::Literal {
            kind,
            symbol: bridge::client::Symbol::new(value),
            suffix: suffix.map(bridge::client::Symbol::new),
            span: Span::call_site().0,
        })
    }

    suffixed_int_literals! {
        u8_suffixed => u8,
        u16_suffixed => u16,
        u32_suffixed => u32,
        u64_suffixed => u64,
        u128_suffixed => u128,
        usize_suffixed => usize,
        i8_suffixed => i8,
        i16_suffixed => i16,
        i32_suffixed => i32,
        i64_suffixed => i64,
        i128_suffixed => i128,
        is
// ... (truncated) ...
```

**Entity:** Punct

**State:** Constructed

**State invariants:**
- Input character ch must be one of LEGAL_CHARS
- If ch is not legal, Punct::new panics with "unsupported character ..."
- Stored byte ch is ASCII-compatible (cast via ch as u8)
- Initial span is Span::call_site()
- joint flag reflects spacing == Spacing::Joint

**Evidence:** line 159: doc comment: "The `ch` argument must be a valid punctuation character ... otherwise the function will panic."; line 166: LEGAL_CHARS declared as the whitelist of allowed punctuation; line 170: runtime check of LEGAL_CHARS.contains(&ch); line 171: panic!("unsupported character `{:?}`", ch) names the precondition; line 174: ch stored as u8, which relies on ASCII-only input ensured by the whitelist; line 176: span initialized with Span::call_site().0; line 175: joint set from spacing == Spacing::Joint

**Implementation:** Introduce a validated PunctChar newtype (e.g., struct PunctChar(u8)) with TryFrom<char> that rejects non-LEGAL_CHARS; change Punct::new to accept PunctChar instead of char so invalid inputs are unrepresentable at call sites.

---

## Protocol Invariants

### 10. __rust_start_panic::Aborting behavior

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Calling __rust_start_panic attaches a platform-specific abort message (if supported) and then unconditionally aborts the process; control flow never returns to the caller.

**Evidence**:

```rust
//! Implementation of Rust panics via process aborts
//!
//! When compared to the implementation via unwinding, this crate is *much*
//! simpler! That being said, it's not quite as versatile, but here goes!

#![no_std]
#![unstable(feature = "panic_abort", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![panic_runtime]
#![feature(panic_runtime)]
#![feature(std_internals)]
#![feature(staged_api)]
#![feature(rustc_attrs)]
#![allow(internal_features)]

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "zkvm")]
mod zkvm;

use core::any::Any;
use core::panic::PanicPayload;

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(_: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unreachable!()
}

// "Leak" the payload and shim to the relevant abort on the platform in question.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(_payload: &mut dyn PanicPayload) -> u32 {
    // Android has the ability to attach a message as part of the abort.
    #[cfg(target_os = "android")]
    unsafe {
        android::android_set_abort_message(_payload);
    }
    #[cfg(target_os = "zkvm")]
    unsafe {
        zkvm::zkvm_set_abort_message(_payload);
    }

    unsafe extern "Rust" {
        // This is defined in std::rt.
        #[rustc_std_internal_symbol]
        safe fn __rust_abort() -> !;
    }

    __rust_abort()
}

// This... is a bit of an oddity. The tl;dr; is that this is required to link
// correctly, the longer explanation is below.
//
// Right now the binaries of core/std that we ship are all compiled with
// `-C panic=unwind`. This is done to ensure that the binaries are maximally
// compatible with as many situations as possible. The compiler, however,
// requires a "personality function" for all functions compiled with `-C
// panic=unwind`. This personality function is hardcoded to the symbol
// `rust_eh_personality` and is defined by the `eh_personality` lang item.
//
// So... why not just define that lang item here? Good question! The way that
// panic runtimes are linked in is actually a little subtle in that they're
// "sort of" in the compiler's crate store, but only actually linked if another
// isn't actually linked. This ends up meaning that both this crate and the
// panic_unwind crate can appear in the compiler's crate store, and if both
// define the `eh_personality` lang item then that'll hit an error.
//
// To handle this the compiler only requires the `eh_personality` is defined if
// the panic runtime being linked in is the unwinding runtime, and otherwise
// it's not required to be defined (rightfully so). In this case, however, this
// library just defines this symbol so there's at least some personality
// somewhere.
//
// Essentially this symbol is just defined to get wired up to core/std
// binaries, but it should never be called as we don't link in an unwinding
// runtime at all.
pub mod personalities {
    // In the past this module used to contain stubs for the personality
    // functions of various platforms, but these where removed when personality
    // functions were moved to std.

    // This corresponds to the `eh_catch_typeinfo` lang item
    // that's only used on Emscripten currently.
    //
    // Since panics don't generate exceptions and foreign exceptions are
    // currently UB with -C panic=abort (although this may be subject to
    // change), any catch_unwind calls will never use this typeinfo.
    #[rustc_std_internal_symbol]
    #[allow(non_upper_case_globals)]
    #[cfg(target_os = "emscripten")]
    static rust_eh_catch_typeinfo: [usize; 2] = [0; 2];
}

```

**Entity:** __rust_start_panic

**State:** Aborting

**State invariants:**
- Function does not return; process termination is guaranteed by calling a divergent function
- Payload is intentionally not dropped ("leaked") so a message can be attached on supported platforms
- No unwinding occurs on this panic path

**Evidence:** line 31: comment states "Leak" the payload and shim to the relevant abort; lines 35-38: android::android_set_abort_message(_payload) uses the payload for abort message; lines 39-42: zkvm::zkvm_set_abort_message(_payload) uses the payload for abort message; lines 44-47: declares extern safe fn __rust_abort() -> ! (diverges); line 50: calls __rust_abort(), ensuring no return

**Implementation:** Encode the non-returning behavior in the type system by making __rust_start_panic return the never type (!) so call sites cannot assume it returns. More broadly, expose the panic runtime as a type-level state (e.g., PanicRuntime<Abort>) where APIs reachable only in the Abort state have divergent signatures.

---

## Skipped Files

10 file(s) could not be parsed:

- `/data/rust/library/core/src/array/equality.rs`: expected identifier or `_`
- `/data/rust/library/core/src/borrow.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/arith.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/index.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/bit.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/drop.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/function.rs`: expected identifier or `_`
- `/data/rust/library/core/src/iter/traits/iterator.rs`: expected identifier or `_`
- `/data/rust/library/core/src/default.rs`: expected identifier or `_`
- `/data/rust/library/core/src/cmp/bytewise.rs`: expected identifier or `_`

