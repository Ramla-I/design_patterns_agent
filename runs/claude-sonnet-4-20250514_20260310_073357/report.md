# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 69
- **Temporal ordering**: 9
- **Resource lifecycle**: 10
- **State machine**: 10
- **Precondition**: 32
- **Protocol**: 8
- **Modules analyzed**: 1349

## Temporal Ordering Invariants

### 1. ConcatTreesHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-53`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: ConcatTreesHelper is in the building phase where TokenTrees can be pushed; must call build() or append_to() to finalize

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

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

// ... (other code) ...

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}


// ... (other code) ...


#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}


```

**Entity:** ConcatTreesHelper

**State:** Building

**State invariants:**
- trees Vec contains accumulated TokenTrees
- push() operations are valid
- build() or append_to() must be called to consume the helper

**Transitions:**
- Building -> Consumed via build() or append_to()

**Evidence:** new() creates empty Vec with capacity; push() mutably borrows and adds to trees Vec; build() and append_to() methods consume self (implied by usage pattern); Usage pattern: new() -> push()* -> build()/append_to() in FromIterator and Extend impls

**Implementation:** Split into ConcatTreesHelper<Building> and ConcatTreesHelper<Built>; new() returns Building state; push() only on Building; build()/append_to() consume Building and return Built/TokenStream

---

### 2. ConcatStreamsHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-47`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: ConcatStreamsHelper is accumulating TokenStreams via push() calls before being consumed by build() or append_to()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);

// ... (other code) ...

/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}


// ... (other code) ...


#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}


```

**Entity:** ConcatStreamsHelper

**State:** Building

**State invariants:**
- streams Vec contains accumulated bridge::client::TokenStream items
- push() operations are valid and extend the internal Vec
- build() or append_to() have not yet been called

**Transitions:**
- Building -> Consumed via build()
- Building -> Consumed via append_to()

**Evidence:** new() creates empty Vec with capacity; push() method accumulates streams into internal Vec; build() and append_to() methods consume the accumulated state; Usage pattern in FromIterator and Extend shows: new() -> push()* -> build()/append_to()

**Implementation:** Split into ConcatStreamsHelper<Building> and ConcatStreamsHelper<Built>; new() returns Building state; push() only available on Building; build()/append_to() consume Building and return Built state or final result

---

### 3. ConcatTreesHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-105`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ConcatTreesHelper is accumulating TokenTrees via push() calls before final build()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...


/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };

// ... (other code) ...

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),
            )
        }
    }
}

```

**Entity:** ConcatTreesHelper

**State:** Building

**State invariants:**
- Internal buffer is being populated with TokenTrees
- push() operations are valid and modify internal state
- build() has not yet been called

**Transitions:**
- Building -> Built via build()
- Building -> Appended via append_to()

**Evidence:** builder.push(tree) calls in FromIterator and Extend implementations; builder.build() called at end of FromIterator; builder.append_to(self) called at end of Extend; Separate methods for building vs appending suggest state transitions

**Implementation:** ConcatTreesHelper<Building> with push() method; build(self) -> TokenStream; append_to(self, &mut TokenStream); no methods available after consumption

---

### 4. ConcatStreamsHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-105`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ConcatStreamsHelper is accumulating TokenStreams via push() calls before final build()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...


/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };

// ... (other code) ...

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),
            )
        }
    }
}

```

**Entity:** ConcatStreamsHelper

**State:** Building

**State invariants:**
- Internal buffer is being populated with TokenStreams
- push() operations are valid and modify internal state
- build() has not yet been called

**Transitions:**
- Building -> Built via build()
- Building -> Appended via append_to()

**Evidence:** builder.push(stream) calls in FromIterator and Extend implementations; builder.build() called at end of FromIterator; builder.append_to(self) called at end of Extend; Parallel structure to ConcatTreesHelper suggests same state pattern

**Implementation:** ConcatStreamsHelper<Building> with push() method; build(self) -> TokenStream; append_to(self, &mut TokenStream); no methods available after consumption

---

### 9. ConcatTreesHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-461`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Helper is accumulating trees via push() calls before final build() or append_to()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]

// ... (other code) ...

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;


// ... (other code) ...

impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}


// ... (other code) ...

/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...


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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()

// ... (other code) ...

}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...


macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(

// ... (other code) ...

            )
        }
    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixe
// ... (truncated) ...
```

**Entity:** ConcatTreesHelper

**State:** Building

**State invariants:**
- trees Vec contains accumulated TokenTrees
- push() operations are valid
- build() or append_to() not yet called

**Transitions:**
- Building -> Consumed via build() or append_to()

**Evidence:** trees: Vec<bridge::TokenTree<...>> field accumulates state; push() method adds to trees Vec; build(self) and append_to(self) consume self - one-time use

**Implementation:** ConcatTreesHelper<Building> with push(); build(self) -> (TokenStream, ConcatTreesHelper<Consumed>); no methods on Consumed state

---

### 10. ConcatStreamsHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-461`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Helper is accumulating streams via push() calls before final build() or append_to()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]

// ... (other code) ...

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;


// ... (other code) ...

impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}


// ... (other code) ...

/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...


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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()

// ... (other code) ...

}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...


macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(

// ... (other code) ...

            )
        }
    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixe
// ... (truncated) ...
```

**Entity:** ConcatStreamsHelper

**State:** Building

**State invariants:**
- streams Vec contains accumulated TokenStreams
- push() operations are valid
- build() or append_to() not yet called

**Transitions:**
- Building -> Consumed via build() or append_to()

**Evidence:** streams: Vec<bridge::client::TokenStream> field accumulates state; push() method adds to streams Vec (if Some); build(mut self) and append_to(mut self) consume self - one-time use

**Implementation:** ConcatStreamsHelper<Building> with push(); build(self) -> (TokenStream, ConcatStreamsHelper<Consumed>); no methods on Consumed state

---

### 63. ConcatTreesHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Helper is accumulating trees via push() calls before final build()/append_to()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** ConcatTreesHelper

**State:** Building

**State invariants:**
- trees Vec can be mutated
- push() is valid operation
- build()/append_to() not yet called

**Transitions:**
- Building -> Consumed via build() or append_to()

**Evidence:** push(&mut self, tree: TokenTree) requires mutable access; build(self) and append_to(self, stream) consume self; Vec::with_capacity(capacity) in new() suggests accumulation pattern

**Implementation:** ConcatTreesHelper<Building> with push() method; build(self) -> TokenStream; append_to(self, &mut TokenStream); consumed state has no methods

---

### 64. ConcatStreamsHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Helper is accumulating streams via push() calls before final build()/append_to()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** ConcatStreamsHelper

**State:** Building

**State invariants:**
- streams Vec can be mutated
- push() is valid operation
- build()/append_to() not yet called

**Transitions:**
- Building -> Consumed via build() or append_to()

**Evidence:** push(&mut self, stream: TokenStream) requires mutable access; build(mut self) and append_to(mut self, stream) consume self; Vec::with_capacity(capacity) in new() suggests builder pattern

**Implementation:** ConcatStreamsHelper<Building> with push() method; build(self) -> TokenStream; append_to(self, &mut TokenStream); prevents reuse after consumption

---

### 67. FilteredTests::Building state

**Location**: `/data/rust/library/test/src/lib.rs:1-446`

**Confidence**: low

**Suggested Pattern**: builder

**Description**: FilteredTests is being constructed via add_test/add_bench calls before being consumed

**Evidence**:

```rust
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

**Entity:** FilteredTests

**State:** Building

**State invariants:**
- next_id increments with each add_test/add_bench call
- tests and benches vectors are being populated
- total_len() reflects current accumulated tests

**Transitions:**
- Building -> Consumed via field access in run_tests

**Evidence:** add_test() and add_bench() methods mutate internal state; next_id: usize field tracks insertion order; self.next_id += 1 in both add methods; total_len() method suggests query after building

**Implementation:** Builder pattern with FilteredTestsBuilder that has add_test/add_bench methods, then build() -> FilteredTests with only query methods

---

## Resource Lifecycle Invariants

### 11. panic_abort::Initialized state

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The panic_abort runtime is properly linked and initialized as the active panic runtime for the process

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

**Entity:** panic_abort runtime

**State:** Initialized

**State invariants:**
- __rust_start_panic is callable and will abort the process
- __rust_cleanup_panic exists but is unreachable
- Process will terminate via abort on any panic
- No unwinding or recovery is possible

**Transitions:**
- Initialized -> Aborted via __rust_start_panic()

**Evidence:** #![panic_runtime] attribute declares this as a panic runtime; unreachable!() in __rust_panic_cleanup indicates cleanup never happens; __rust_start_panic calls __rust_abort() which is marked -> !; Comment: 'Leak' the payload and shim to the relevant abort

**Implementation:** Use a PanicRuntime<Abort> capability token that can only be created once during process initialization; panic functions require this token to prove the runtime is properly configured

---

### 12. panic_abort::Aborted state

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The process has been terminated via abort; this is a terminal state with no recovery

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

**Entity:** panic_abort runtime

**State:** Aborted

**State invariants:**
- Process execution has terminated
- No further code execution is possible
- All cleanup functions are bypassed

**Evidence:** __rust_abort() is marked with -> ! (never returns); unreachable!() in cleanup function shows it's never called; Comment about 'leaking' the payload indicates no cleanup

**Implementation:** Terminal state in a typestate machine where Aborted has no methods and cannot transition to any other state

---

### 19. PanicPayload::Active state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: PanicPayload contains valid panic data that can be taken exactly once via take_box()

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** PanicPayload

**State:** Active

**State invariants:**
- payload contains valid panic data
- take_box() can be called exactly once
- payload is owned by the panic runtime

**Transitions:**
- Active -> Consumed via take_box()

**Evidence:** payload.take_box() in __rust_start_panic consumes the payload; Box::from_raw(payload.take_box()) assumes take_box() returns valid pointer; PanicPayload trait implies single-use semantics

**Implementation:** PanicPayload<Active> with take_box(self) -> (Box<dyn Any>, PanicPayload<Consumed>); only Active state allows take_box()

---

### 20. PanicPayload::Consumed state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: PanicPayload has been consumed via take_box() and no longer contains valid data

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** PanicPayload

**State:** Consumed

**State invariants:**
- take_box() has already been called
- no further operations are valid
- payload data has been transferred to Box

**Evidence:** Box::from_raw(payload.take_box()) transfers ownership; take_box() name implies single consumption; no subsequent payload operations after take_box() call

**Implementation:** PanicPayload<Consumed> has no methods - double consumption becomes compile error

---

### 21. cleanup payload::Raw state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Raw pointer from platform-specific panic implementation that must be cleaned up exactly once

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** panic cleanup payload

**State:** Raw

**State invariants:**
- payload pointer is valid and non-null
- payload was allocated by platform panic implementation
- cleanup() must be called exactly once

**Transitions:**
- Raw -> Cleaned via imp::cleanup()

**Evidence:** __rust_panic_cleanup takes *mut u8 payload parameter; imp::cleanup(payload) called exactly once; Box::into_raw() transfers ownership to caller

**Implementation:** CleanupPayload<Raw> newtype with Drop impl calling cleanup(); into_cleaned(self) -> CleanupPayload<Cleaned>

---

### 22. cleanup payload::Cleaned state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Payload has been processed by imp::cleanup() and converted to Box<dyn Any>

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** panic cleanup payload

**State:** Cleaned

**State invariants:**
- imp::cleanup() has been called on raw payload
- result is valid Box<dyn Any + Send + 'static>
- raw payload is no longer valid

**Evidence:** Box::into_raw(imp::cleanup(payload)) creates cleaned result; return type is *mut (dyn Any + Send + 'static); cleanup() consumes the raw payload

**Implementation:** CleanupPayload<Cleaned> contains Box<dyn Any>; no further cleanup needed

---

### 50. ConcatTreesHelper::Building state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-313`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: ConcatTreesHelper is accumulating TokenTrees via push() calls before being consumed by build()

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...


#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...

impl ConcatTreesHelper {
    fn new(capacity: usize) -> Self {
        ConcatTreesHelper { trees: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, tree: TokenTree) {
        self.trees.push(tree_to_bridge_tree(tree));
    }

    fn build(self) -> TokenStream {
        if self.trees.is_empty() {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...

        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }

// ... (other code) ...

    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

// ... (other code) ...

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),

// ... (other code) ...

}

/// A single token or a delimited sequence of token trees (e.g., `[1, (), ..]`).
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for TokenTree {}

impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

```

**Entity:** ConcatTreesHelper

**State:** Building

**State invariants:**
- trees Vec is owned and mutable
- push() operations are valid
- build() has not been called yet

**Transitions:**
- Building -> Consumed via build()

**Evidence:** build(self) consumes ownership with self parameter; push(&mut self, tree: TokenTree) requires mutable access; trees: Vec<_> field accumulates state

**Implementation:** Split into ConcatTreesHelper<Building> and ConcatTreesHelper<Built>; push() only on Building state; build(self) -> TokenStream consumes Building state

---

### 51. ConcatTreesHelper::Consumed state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-313`

**Confidence**: medium

**Suggested Pattern**: builder

**Description**: ConcatTreesHelper has been consumed by build() and is no longer usable

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...


#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...

impl ConcatTreesHelper {
    fn new(capacity: usize) -> Self {
        ConcatTreesHelper { trees: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, tree: TokenTree) {
        self.trees.push(tree_to_bridge_tree(tree));
    }

    fn build(self) -> TokenStream {
        if self.trees.is_empty() {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...

        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }

// ... (other code) ...

    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

// ... (other code) ...

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),

// ... (other code) ...

}

/// A single token or a delimited sequence of token trees (e.g., `[1, (), ..]`).
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for TokenTree {}

impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

```

**Entity:** ConcatTreesHelper

**State:** Consumed

**State invariants:**
- helper instance no longer exists
- trees Vec has been moved into TokenStream creation
- no further operations possible

**Evidence:** build(self) takes ownership and consumes the helper; if self.trees.is_empty() check suggests state examination before consumption

**Implementation:** ConcatTreesHelper<Built> would have no methods - compile-time prevention of use-after-build

---

### 65. RunningTest::Active state

**Location**: `/data/rust/library/test/src/lib.rs:1-446`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: RunningTest has a valid join_handle that represents a running thread

**Evidence**:

```rust
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

**Entity:** RunningTest

**State:** Active

**State invariants:**
- join_handle is Some(JoinHandle<()>)
- associated thread is still running
- join() has not been called yet

**Transitions:**
- Active -> Completed via join()

**Evidence:** join_handle: Option<thread::JoinHandle<()>> field; if let Some(join_handle) = self.join_handle check in join(); join() consumes self, preventing reuse

**Implementation:** Split into RunningTest<Active> and RunningTest<Completed>; join(self: RunningTest<Active>) -> RunningTest<Completed>; only Active state has the join_handle

---

### 66. RunningTest::Completed state

**Location**: `/data/rust/library/test/src/lib.rs:1-446`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: RunningTest after join() has been called - thread has been joined and handle consumed

**Evidence**:

```rust
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

**Entity:** RunningTest

**State:** Completed

**State invariants:**
- join_handle has been consumed
- thread has completed execution
- no further operations are valid

**Evidence:** join() consumes self via self.join_handle; Option<JoinHandle> becomes None after extraction; method signature fn join(self, ...) indicates state transition

**Implementation:** RunningTest<Completed> has no methods - prevents double-join at compile time

---

## State Machine Invariants

### 6. TokenStream::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-461`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: TokenStream contains no tokens, represented by None variant

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]

// ... (other code) ...

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;


// ... (other code) ...

impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}


// ... (other code) ...

/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...


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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()

// ... (other code) ...

}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...


macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(

// ... (other code) ...

            )
        }
    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixe
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** Empty

**State invariants:**
- self.0 == None
- is_empty() returns true
- Display prints nothing
- Iterator yields no items

**Transitions:**
- Empty -> NonEmpty via from_str(), from(), extend(), etc.

**Evidence:** TokenStream(Option<bridge::client::TokenStream>) field - None encodes empty; TokenStream::new() -> TokenStream(None); is_empty() checks self.0.as_ref().map(...).unwrap_or(true); Display match None => Ok(()) - empty case handled specially

**Implementation:** Split into TokenStream<Empty> and TokenStream<NonEmpty>; new() returns TokenStream<Empty>; operations like extend() transition to TokenStream<NonEmpty>; some operations only valid on non-empty streams

---

### 7. TokenStream::NonEmpty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-461`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: TokenStream contains actual tokens, represented by Some variant

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]

// ... (other code) ...

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;


// ... (other code) ...

impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}


// ... (other code) ...

/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...


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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()

// ... (other code) ...

}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...


macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(

// ... (other code) ...

            )
        }
    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixe
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** NonEmpty

**State invariants:**
- self.0 == Some(bridge::client::TokenStream)
- is_empty() returns false
- Display prints token representation
- Iterator yields actual TokenTree items

**Transitions:**
- NonEmpty -> Empty via consuming operations (rare)

**Evidence:** Some(bridge::client::TokenStream) variant holds actual tokens; is_empty() calls BridgeMethods::ts_is_empty(h) on Some case; Display match Some(ts) => write!(...) - actual formatting; expand_expr() requires Some variant: self.0.as_ref().ok_or(ExpandError)?

**Implementation:** TokenStream<NonEmpty> guarantees tokens exist; expand_expr() only available on non-empty streams; eliminates runtime None checks

---

### 30. TokenStream::Some state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-278`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TokenStream contains Some(ts) with actual token data that can be displayed and iterated

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}


// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }


// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}


// ... (other code) ...

        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}


// ... (other code) ...

        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}


// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** NonEmpty

**State invariants:**
- self.0 == Some(ts)
- fmt::Display will call BridgeMethods::ts_to_string(ts)
- iteration will yield actual TokenTree elements

**Transitions:**
- Empty -> NonEmpty via construction/parsing
- NonEmpty -> Empty via consumption

**Evidence:** match &self.0 { Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)) pattern; Option<T> field self.0 encodes presence/absence of token data; Different behavior in Display based on Some/None state

**Implementation:** TokenStream<Empty> and TokenStream<NonEmpty> states; parsing produces NonEmpty, empty constructor produces Empty; Display only implemented for NonEmpty

---

### 31. TokenStream::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-278`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TokenStream contains None, representing an empty token stream with no displayable content

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}


// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }


// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}


// ... (other code) ...

        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}


// ... (other code) ...

        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}


// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** Empty

**State invariants:**
- self.0 == None
- fmt::Display will write empty content
- iteration yields no elements

**Transitions:**
- Empty -> NonEmpty via token addition/parsing

**Evidence:** Option<T> field self.0 can be None; match &self.0 pattern suggests different behavior for None case; Empty token streams are a distinct concept from non-empty ones

**Implementation:** TokenStream<Empty> has no Display implementation or returns empty string; only TokenStream<NonEmpty> can be meaningfully displayed

---

### 35. Ident::RawIdentifier state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-272`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Ident is a raw identifier (r#ident) with stricter validation rules than regular identifiers

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {

// ... (other code) ...

    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...


        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {

// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

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

// ... (other code) ...

    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }


// ... (other code) ...

    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

// ... (other code) ...

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

// ... (other code) ...

        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {

// ... (other code) ...

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Ident

**State:** RawIdentifier

**State invariants:**
- is_raw == true
- string is valid identifier but not path segment keyword
- Display includes r# prefix

**Evidence:** is_raw: bool field in bridge::Ident struct; if self.0.is_raw { f.write_str("r#")?; } in Display impl; new_raw() sets is_raw: true, new() sets is_raw: false; Different validation rules mentioned in new_raw() docs

**Implementation:** Split into Ident<Regular> and Ident<Raw> with different constructors and Display impls; eliminates is_raw boolean field and conditional formatting

---

### 45. TokenStream::Some state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-419`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: TokenStream contains actual token data and can be displayed/processed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.

// ... (other code) ...

        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

        isize_unsuffixed => isize,
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_unsuffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_suffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f32"))
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_unsuffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f64` where the value
    /// specified is the preceding part of the token and `f64` is the suffix of
    /// the token. This token will always be inferred to be an `f64` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_suffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f64"))
    }

    /// String literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn string(string: &str) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.as_bytes(), escape);
        Literal::new(bridge::LitKind::Str, &repr, None)
    }

    /// Character literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn character(ch: char) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: false,
        };
        let repr = escape_bytes(ch.encode_utf8(&mut [0u8; 4]).as_bytes(), escape);
        Literal::new(bridge::LitKind::Char, &repr, None)
    }

    /// Byte character literal.
    #[stable(feature = "proc_macro_byte_character", since = "1.79.0")]
    pub fn byte_character(byte: u8) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: true,
        };
        let repr = escape_bytes(&[byte], escape);
        Literal::new(bridge::LitKind::Byte, &repr, None)
    }

    /// Byte string literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn byte_string(bytes: &[u8]) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: true,
        };
        let repr = escape_bytes(bytes, escape);
        Literal::new(bridge::LitKind::ByteStr, &repr, None)
    }

    /// C string literal.
    #[stable(feature = "proc_macro_c_str_literals", since = "1.79.0")]
    pub fn c_string(string: &CStr) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.to_bytes(), escape);
        Literal::new(bridge::LitKind::CStr, &repr, None)
    }

    /// Returns the span encompassing this literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {

// ... (other code) ...

    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the u
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** Some

**State invariants:**
- self.0 == Some(ts) where ts contains valid token data
- Display implementation will succeed
- Can be converted to string representation

**Evidence:** match &self.0 { Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)) } in Display impl; Option<T> field self.0 encodes presence/absence of token data; Pattern matching on Some/None in display logic

**Implementation:** Split into EmptyTokenStream and PopulatedTokenStream types; Display only implemented for PopulatedTokenStream; constructors return appropriate type

---

### 52. TokenStream::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-313`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: TokenStream contains None, representing an empty token stream

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...


#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...

impl ConcatTreesHelper {
    fn new(capacity: usize) -> Self {
        ConcatTreesHelper { trees: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, tree: TokenTree) {
        self.trees.push(tree_to_bridge_tree(tree));
    }

    fn build(self) -> TokenStream {
        if self.trees.is_empty() {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...

        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }

// ... (other code) ...

    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

// ... (other code) ...

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),

// ... (other code) ...

}

/// A single token or a delimited sequence of token trees (e.g., `[1, (), ..]`).
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for TokenTree {}

impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** Empty

**State invariants:**
- self.0 == None
- iteration produces no tokens
- display shows empty string

**Transitions:**
- Empty -> NonEmpty via extend() or from_iter()

**Evidence:** TokenStream(Option<bridge::client::TokenStream>) with None case; unwrap_or_default() in into_iter suggests None handling; if self.trees.is_empty() check in ConcatTreesHelper::build()

**Implementation:** TokenStream<Empty> and TokenStream<NonEmpty> types; certain operations only valid on non-empty streams

---

### 53. TokenStream::NonEmpty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-313`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: TokenStream contains Some(bridge_stream) with actual token data

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...


#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...

impl ConcatTreesHelper {
    fn new(capacity: usize) -> Self {
        ConcatTreesHelper { trees: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, tree: TokenTree) {
        self.trees.push(tree_to_bridge_tree(tree));
    }

    fn build(self) -> TokenStream {
        if self.trees.is_empty() {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...

        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }

// ... (other code) ...

    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

// ... (other code) ...

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),

// ... (other code) ...

}

/// A single token or a delimited sequence of token trees (e.g., `[1, (), ..]`).
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for TokenTree {}

impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** NonEmpty

**State invariants:**
- self.0 == Some(bridge_stream)
- iteration produces tokens
- display shows token representation

**Transitions:**
- NonEmpty -> Empty via consumption (theoretical)

**Evidence:** Some(ts) pattern matching in fmt::Display implementation; self.0.map() usage suggests Some/None distinction; BridgeMethods::ts_into_trees(v) called on Some variant

**Implementation:** Separate empty and non-empty TokenStream types to make emptiness explicit at compile time

---

### 59. TokenStream::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: TokenStream contains no tokens, represented by None variant

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** Empty

**State invariants:**
- self.0 == None
- is_empty() returns true
- Display outputs empty string
- expand_expr() returns Err(ExpandError)

**Transitions:**
- Empty -> NonEmpty via from_str(), from(), extend(), etc.

**Evidence:** TokenStream(Option<bridge::client::TokenStream>) - None encodes empty state; self.0.as_ref().ok_or(ExpandError)? in expand_expr() - None is invalid for expansion; unwrap_or(true) in is_empty() - None defaults to empty; match &self.0 { None => Ok(()) } in Display - special handling for None

**Implementation:** Split into TokenStream<Empty> and TokenStream<NonEmpty>; new() returns Empty; parsing/construction methods transition to NonEmpty; expand_expr() only available on NonEmpty

---

### 60. TokenStream::NonEmpty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: TokenStream contains actual tokens, represented by Some(bridge_stream)

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** NonEmpty

**State invariants:**
- self.0 == Some(bridge_stream)
- is_empty() delegates to bridge method
- expand_expr() can succeed or fail based on content
- Display outputs actual token representation

**Transitions:**
- NonEmpty -> Empty via clear operations (not shown in API)

**Evidence:** Some(BridgeMethods::ts_from_str(src)) in FromStr::from_str; Some(BridgeMethods::ts_from_token_tree(tree)) in From<TokenTree>; BridgeMethods::ts_expand_expr(stream) only called on Some variant; write!(f, "{}", BridgeMethods::ts_to_string(ts)) for Some case

**Implementation:** TokenStream<NonEmpty> has expand_expr() method; construction always produces NonEmpty; empty check becomes type-level

---

## Precondition Invariants

### 5. TokenStream::Bridge connection protocol

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-105`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: TokenStream operations require valid bridge connection to proc_macro runtime

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }

// ... (other code) ...


/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };

// ... (other code) ...

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(
                self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default().into_iter(),
            )
        }
    }
}

```

**Entity:** TokenStream

**State:** BridgeConnected

**State invariants:**
- self.0 contains Some(bridge_token_stream) for valid operations
- BridgeMethods::ts_into_trees() can be called safely
- Bridge client connection is active

**Evidence:** self.0.map(|v| BridgeMethods::ts_into_trees(v)).unwrap_or_default() pattern; unwrap_or_default() suggests self.0 can be None in some cases; Bridge-based implementation suggests dependency on external proc_macro runtime

**Implementation:** TokenStream<Connected> vs TokenStream<Disconnected>; operations only available on Connected variant; or capability token pattern

---

### 8. TokenStream::Expandable precondition

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-461`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TokenStream must be non-empty and contain expandable expressions for expand_expr() to succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]

// ... (other code) ...

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;


// ... (other code) ...

impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}


// ... (other code) ...

/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

// ... (other code) ...


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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {

// ... (other code) ...

    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()

// ... (other code) ...

}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }

// ... (other code) ...


macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

// ... (other code) ...

            self.0.count()
        }
    }

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl IntoIterator for TokenStream {
        type Item = TokenTree;
        type IntoIter = IntoIter;

        fn into_iter(self) -> IntoIter {
            IntoIter(

// ... (other code) ...

            )
        }
    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.

// ... (other code) ...

    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixe
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** Expandable

**State invariants:**
- self.0.is_some() - must have tokens
- tokens must represent expandable expressions
- currently only expressions expanding to literals succeed

**Evidence:** expand_expr() does self.0.as_ref().ok_or(ExpandError)? - requires Some; Comment: 'Currently only expressions expanding to literals will succeed'; BridgeMethods::ts_expand_expr(stream) can return Err(_) -> ExpandError

**Implementation:** ExpandableTokenStream newtype that validates expandability at construction; expand_expr() only accepts ExpandableTokenStream

---

### 13. PanicPayload::Valid precondition

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The PanicPayload parameter must be a valid, properly constructed panic payload before calling __rust_start_panic

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

**State:** Valid

**State invariants:**
- _payload points to valid PanicPayload implementation
- Payload contains panic information that can be extracted
- Platform-specific abort message can be set from payload

**Transitions:**
- Valid -> Consumed via __rust_start_panic()

**Evidence:** unsafe fn __rust_start_panic(_payload: &mut dyn PanicPayload) signature; Platform-specific code extracts message from _payload; android::android_set_abort_message(_payload) and zkvm::zkvm_set_abort_message(_payload) calls

**Implementation:** ValidatedPanicPayload newtype that can only be constructed through safe validation; __rust_start_panic takes ValidatedPanicPayload instead of raw dyn PanicPayload

---

### 15. TokenStream::Valid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-72`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TokenStream contains successfully parsed tokens with valid syntax and balanced delimiters

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]

// ... (other code) ...

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// but the literal token. Specifically, it must not contain whitespace or
/// comments in addition to the literal.
///
/// The resulting literal token will have a `Span::call_site()` span.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We
/// reserve the right to change these errors into `LexError`s later.
#[stable(feature = "proc_macro_literal_parse", since = "1.54.0")]
impl FromStr for Literal {
    type Err = LexError;

    fn from_str(src: &str) -> Result<Self, LexError> {
        match BridgeMethods::literal_from_str(src) {
            Ok(literal) => Ok(Literal(literal)),
            Err(msg) => Err(LexError(msg)),
        }
    }
}

/// Prints the literal as a string that should be losslessly convertible

```

**Entity:** TokenStream

**State:** Valid

**State invariants:**
- Internal token structure is well-formed
- Delimiters are balanced
- All characters exist in the Rust language
- Can be losslessly converted back to string representation

**Transitions:**
- Invalid -> Valid via successful FromStr::from_str()

**Evidence:** FromStr implementation can fail with LexError for "unbalanced delimiters"; FromStr implementation can fail for "characters not existing in the language"; Comment: "supposed to be losslessly convertible back into the same token stream"; NOTE comment reserves right to change panics into LexError - indicates current runtime failures

**Implementation:** ValidTokenStream newtype that can only be constructed through validated parsing; regular TokenStream operations only available on ValidTokenStream

---

### 16. TokenStream::Invalid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-72`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: String input that cannot be parsed into valid tokens due to syntax errors

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]

// ... (other code) ...

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// but the literal token. Specifically, it must not contain whitespace or
/// comments in addition to the literal.
///
/// The resulting literal token will have a `Span::call_site()` span.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We
/// reserve the right to change these errors into `LexError`s later.
#[stable(feature = "proc_macro_literal_parse", since = "1.54.0")]
impl FromStr for Literal {
    type Err = LexError;

    fn from_str(src: &str) -> Result<Self, LexError> {
        match BridgeMethods::literal_from_str(src) {
            Ok(literal) => Ok(Literal(literal)),
            Err(msg) => Err(LexError(msg)),
        }
    }
}

/// Prints the literal as a string that should be losslessly convertible

```

**Entity:** TokenStream

**State:** Invalid

**State invariants:**
- Contains unbalanced delimiters OR
- Contains characters not existing in Rust language OR
- Has other lexical/syntactic errors

**Evidence:** FromStr::from_str returns Result<TokenStream, LexError>; LexError wraps error message for invalid input; Comment mentions "unbalanced delimiters" and "characters not existing in the language" as failure cases

**Implementation:** Separate parsing phase that produces either ValidTokenStream or detailed parse errors; no invalid TokenStreams can exist at runtime

---

### 17. Literal::Valid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-72`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Literal contains valid token representation without whitespace or comments

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]

// ... (other code) ...

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// but the literal token. Specifically, it must not contain whitespace or
/// comments in addition to the literal.
///
/// The resulting literal token will have a `Span::call_site()` span.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We
/// reserve the right to change these errors into `LexError`s later.
#[stable(feature = "proc_macro_literal_parse", since = "1.54.0")]
impl FromStr for Literal {
    type Err = LexError;

    fn from_str(src: &str) -> Result<Self, LexError> {
        match BridgeMethods::literal_from_str(src) {
            Ok(literal) => Ok(Literal(literal)),
            Err(msg) => Err(LexError(msg)),
        }
    }
}

/// Prints the literal as a string that should be losslessly convertible

```

**Entity:** Literal

**State:** Valid

**State invariants:**
- Must not contain whitespace in addition to the literal
- Must not contain comments in addition to the literal
- Must be exactly one literal token
- Can be losslessly converted back to string

**Transitions:**
- Invalid -> Valid via successful FromStr::from_str()

**Evidence:** FromStr implementation returns Result<Self, LexError>; Comment: "it must not contain whitespace or comments in addition to the literal"; Comment: "should be losslessly convertible"; NOTE comment about panics vs LexError indicates runtime validation

**Implementation:** ValidLiteral newtype with private constructor; FromStr produces ValidLiteral or detailed error; literal operations only on ValidLiteral

---

### 23. platform impl::Unselected state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Platform-specific implementation module has not been selected via cfg_select

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** platform implementation

**State:** Unselected

**State invariants:**
- no imp module is available
- panic() and cleanup() functions are not callable
- compilation will fail if panic runtime is used

**Transitions:**
- Unselected -> Selected via cfg_select! macro

**Evidence:** cfg_select! macro chooses implementation based on target; imp::panic() and imp::cleanup() calls assume selected implementation; different #[path] attributes for different platforms

**Implementation:** PlatformPanic capability token provided only when valid implementation is selected; panic functions require token

---

### 24. platform impl::Selected state

**Location**: `/data/rust/library/panic_unwind/src/lib.rs:1-110`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Platform-specific implementation has been selected and provides panic()/cleanup() functions

**Evidence**:

```rust
//! Implementation of panics via stack unwinding
//!
//! This crate is an implementation of panics in Rust using "most native" stack
//! unwinding mechanism of the platform this is being compiled for. This
//! essentially gets categorized into three buckets currently:
//!
//! 1. MSVC targets use SEH in the `seh.rs` file.
//! 2. Emscripten uses C++ exceptions in the `emcc.rs` file.
//! 3. All other targets use libunwind/libgcc in the `gcc.rs` file.
//!
//! More documentation about each implementation can be found in the respective
//! module.

#![no_std]
#![unstable(feature = "panic_unwind", issue = "32837")]
#![doc(issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/")]
#![cfg_attr(all(target_os = "emscripten", not(emscripten_wasm_eh)), feature(lang_items))]
#![feature(cfg_emscripten_wasm_eh)]
#![feature(core_intrinsics)]
#![feature(panic_unwind)]
#![feature(staged_api)]
#![feature(std_internals)]
#![feature(rustc_attrs)]
#![panic_runtime]
#![feature(panic_runtime)]
#![allow(internal_features)]
#![allow(unused_features)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

use alloc::boxed::Box;
use core::any::Any;
use core::panic::PanicPayload;

cfg_select! {
    all(target_os = "emscripten", not(emscripten_wasm_eh)) => {
        #[path = "emcc.rs"]
        mod imp;
    }
    target_os = "hermit" => {
        #[path = "hermit.rs"]
        mod imp;
    }
    target_os = "l4re" => {
        // L4Re is unix family but does not yet support unwinding.
        #[path = "dummy.rs"]
        mod imp;
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(any(target_os = "espidf", target_os = "nuttx"))),
        all(target_vendor = "fortanix", target_env = "sgx"),
        target_family = "wasm",
    ) => {
        #[path = "gcc.rs"]
        mod imp;
    }
    miri => {
        // Use the Miri runtime on Windows as miri doesn't support funclet based unwinding,
        // only landingpad based unwinding. Also use the Miri runtime on unsupported platforms.
        #[path = "miri.rs"]
        mod imp;
    }
    all(target_env = "msvc", not(target_arch = "arm")) => {
        // LLVM does not support unwinding on 32 bit ARM msvc (thumbv7a-pc-windows-msvc)
        #[path = "seh.rs"]
        mod imp;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - nvptx64-nvidia-cuda
        // - arch=avr
        #[path = "dummy.rs"]
        mod imp;
    }
}

unsafe extern "C" {
    /// Handler in std called when a panic object is dropped outside of
    /// `catch_unwind`.
    #[rustc_std_internal_symbol]
    fn __rust_drop_panic() -> !;

    /// Handler in std called when a foreign exception is caught.
    #[rustc_std_internal_symbol]
    fn __rust_foreign_exception() -> !;
}

#[rustc_std_internal_symbol]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static) {
    unsafe { Box::into_raw(imp::cleanup(payload)) }
}

// Entry point for raising an exception, just delegates to the platform-specific
// implementation.
#[rustc_std_internal_symbol]
pub unsafe fn __rust_start_panic(payload: &mut dyn PanicPayload) -> u32 {
    unsafe {
        let payload = Box::from_raw(payload.take_box());

        imp::panic(payload)
    }
}

```

**Entity:** platform implementation

**State:** Selected

**State invariants:**
- imp module provides panic() and cleanup() functions
- functions match expected signatures
- implementation is compatible with target platform

**Evidence:** imp::panic(payload) call in __rust_start_panic; imp::cleanup(payload) call in __rust_panic_cleanup; cfg_select! ensures exactly one implementation is chosen

**Implementation:** Compile-time guarantee that required functions exist before allowing their use

---

### 25. Punct::ValidCharacter state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-455`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Punct contains only valid punctuation characters permitted by the language

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};

// ... (other code) ...

}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {

// ... (other code) ...

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),

// ... (other code) ...

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


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );


// ... (other code) ...

mod quote;

/// A region of source code, along with macro expansion information.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Copy, Clone)]
pub struct Span(bridge::client::Span);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Span {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Span {}

macro_rules! diagnostic_method {
    ($name:ident, $level:expr) => {
        /// Creates a new `Diagnostic` with the given `message` at the span
        /// `self`.

// ... (other code) ...

            Diagnostic::spanned(self, $level, message)
        }
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

// ... (other code) ...


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

// ... (other code) ...

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

/// Prints a span in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


// ... (other code) ...


impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),

// ... (other code) ...

    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),

// ... (other code) ...

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...


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

// ... (other code) ...


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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

}

/// An identifier (`ident`).
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Ident(bridge::Ident<bridge::client::Span, bridge::client::Symbol>);

impl Ident {
    /// Creates a new `Ident` with the given `string` as well as the specified
    /// `span`.
    /// The `string` argument must be a valid identifier permitted by the

// ... (other code) ...

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

// ... (other code) ...

    /// Same as `Ident::new`, but creates a raw identifier (`r#ident`).
    /// The `string` argument be a valid identifier permitted by the language
    /// (including keywords, e.g. `fn`). Keywords which are usable in path segments
    /// (e.g. `self`, `super`) are not supported, and will cause a panic.
    #[stable(feature = "pr
// ... (truncated) ...
```

**Entity:** Punct

**State:** ValidCharacter

**State invariants:**
- ch must be in LEGAL_CHARS array
- character is a valid punctuation token
- panic occurs if invalid character is provided

**Evidence:** const LEGAL_CHARS: &[char] = &['=', '<', '>', '!', ...] defines valid set; panic!("unsupported character `{:?}`", ch) in Punct::new(); runtime validation loop checking if ch is in LEGAL_CHARS

**Implementation:** Create ValidPunctChar newtype that can only be constructed with valid characters; Punct::new(ch: ValidPunctChar, spacing: Spacing) eliminates runtime check

---

### 26. Ident::ValidIdentifier state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-455`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Ident contains only valid identifier strings permitted by the language

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};

// ... (other code) ...

}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {

// ... (other code) ...

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),

// ... (other code) ...

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


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );


// ... (other code) ...

mod quote;

/// A region of source code, along with macro expansion information.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Copy, Clone)]
pub struct Span(bridge::client::Span);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Span {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Span {}

macro_rules! diagnostic_method {
    ($name:ident, $level:expr) => {
        /// Creates a new `Diagnostic` with the given `message` at the span
        /// `self`.

// ... (other code) ...

            Diagnostic::spanned(self, $level, message)
        }
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

// ... (other code) ...


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

// ... (other code) ...

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

/// Prints a span in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


// ... (other code) ...


impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),

// ... (other code) ...

    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),

// ... (other code) ...

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...


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

// ... (other code) ...


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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

}

/// An identifier (`ident`).
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Ident(bridge::Ident<bridge::client::Span, bridge::client::Symbol>);

impl Ident {
    /// Creates a new `Ident` with the given `string` as well as the specified
    /// `span`.
    /// The `string` argument must be a valid identifier permitted by the

// ... (other code) ...

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

// ... (other code) ...

    /// Same as `Ident::new`, but creates a raw identifier (`r#ident`).
    /// The `string` argument be a valid identifier permitted by the language
    /// (including keywords, e.g. `fn`). Keywords which are usable in path segments
    /// (e.g. `self`, `super`) are not supported, and will cause a panic.
    #[stable(feature = "pr
// ... (truncated) ...
```

**Entity:** Ident

**State:** ValidIdentifier

**State invariants:**
- string must be a valid identifier
- string must be NFC-normalized
- raw identifiers have additional restrictions (no path segments like 'self', 'super')

**Evidence:** comment: "The `string` argument must be a valid identifier permitted by the language"; comment: "Keywords which are usable in path segments (e.g. `self`, `super`) are not supported, and will cause a panic"; bridge::client::Symbol::new_ident(string, false) performs validation

**Implementation:** Create ValidIdentString and ValidRawIdentString newtypes with validation in constructor; Ident::new(string: ValidIdentString, span: Span) eliminates runtime validation

---

### 27. Span::SameFile state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-455`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Two spans must be from the same file for join operation to succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};

// ... (other code) ...

}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {

// ... (other code) ...

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),

// ... (other code) ...

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


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );


// ... (other code) ...

mod quote;

/// A region of source code, along with macro expansion information.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Copy, Clone)]
pub struct Span(bridge::client::Span);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Span {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Span {}

macro_rules! diagnostic_method {
    ($name:ident, $level:expr) => {
        /// Creates a new `Diagnostic` with the given `message` at the span
        /// `self`.

// ... (other code) ...

            Diagnostic::spanned(self, $level, message)
        }
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

// ... (other code) ...


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

// ... (other code) ...

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

/// Prints a span in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


// ... (other code) ...


impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),

// ... (other code) ...

    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),

// ... (other code) ...

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...


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

// ... (other code) ...


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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

}

/// An identifier (`ident`).
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Ident(bridge::Ident<bridge::client::Span, bridge::client::Symbol>);

impl Ident {
    /// Creates a new `Ident` with the given `string` as well as the specified
    /// `span`.
    /// The `string` argument must be a valid identifier permitted by the

// ... (other code) ...

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

// ... (other code) ...

    /// Same as `Ident::new`, but creates a raw identifier (`r#ident`).
    /// The `string` argument be a valid identifier permitted by the language
    /// (including keywords, e.g. `fn`). Keywords which are usable in path segments
    /// (e.g. `self`, `super`) are not supported, and will cause a panic.
    #[stable(feature = "pr
// ... (truncated) ...
```

**Entity:** Span

**State:** SameFile

**State invariants:**
- both spans reference the same source file
- join() returns Some(Span) only if spans are compatible
- join() returns None if spans are from different files

**Evidence:** comment: "Returns `None` if `self` and `other` are from different files" in join() method; BridgeMethods::span_join(self.0, other.0).map(Span) returns Option<Span>; runtime check in bridge layer determines file compatibility

**Implementation:** Create FileSpan<F> parameterized by file marker; join() only accepts spans with same file type parameter, eliminating Option return

---

### 28. TokenStream::ValidSyntax state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-455`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: TokenStream contains only syntactically valid tokens with balanced delimiters

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};

// ... (other code) ...

}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {

// ... (other code) ...

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),

// ... (other code) ...

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


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );


// ... (other code) ...

mod quote;

/// A region of source code, along with macro expansion information.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Copy, Clone)]
pub struct Span(bridge::client::Span);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Span {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Span {}

macro_rules! diagnostic_method {
    ($name:ident, $level:expr) => {
        /// Creates a new `Diagnostic` with the given `message` at the span
        /// `self`.

// ... (other code) ...

            Diagnostic::spanned(self, $level, message)
        }
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

// ... (other code) ...


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

// ... (other code) ...

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

/// Prints a span in a form convenient for debugging.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


// ... (other code) ...


impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),

// ... (other code) ...

    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),

// ... (other code) ...

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

// ... (other code) ...


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

// ... (other code) ...


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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

}

/// An identifier (`ident`).
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Ident(bridge::Ident<bridge::client::Span, bridge::client::Symbol>);

impl Ident {
    /// Creates a new `Ident` with the given `string` as well as the specified
    /// `span`.
    /// The `string` argument must be a valid identifier permitted by the

// ... (other code) ...

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

// ... (other code) ...

    /// Same as `Ident::new`, but creates a raw identifier (`r#ident`).
    /// The `string` argument be a valid identifier permitted by the language
    /// (including keywords, e.g. `fn`). Keywords which are usable in path segments
    /// (e.g. `self`, `super`) are not supported, and will cause a panic.
    #[stable(feature = "pr
// ... (truncated) ...
```

**Entity:** TokenStream

**State:** ValidSyntax

**State invariants:**
- all delimiters are properly balanced
- all characters exist in the language
- tokens form valid syntax tree
- parsing may fail with LexError

**Evidence:** comment: "May fail for a number of reasons, for example, if the string contains unbalanced delimiters or characters not existing in the language"; FromStr implementation returns Result<TokenStream, LexError>; comment: "NOTE: some errors may cause panics instead of returning `LexError`"

**Implementation:** Create ValidatedTokenStream newtype that can only be constructed from successfully parsed input; eliminates need for Result return in many contexts

---

### 29. Punct::ValidCharacter state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-278`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Punct contains a character that is a valid punctuation character permitted by the language

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}


// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }


// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}


// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.

// ... (other code) ...

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}


// ... (other code) ...

        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}


// ... (other code) ...

        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {
    fn from(g: Punct) -> TokenTree {
        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}


// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Punct

**State:** ValidCharacter

**State invariants:**
- ch must be one of: '=', '<', '>', '!', '~', '+', '-', '*', '/', '%', '^', '&', '|', '@', '.', ',', ';', ':', '#', '$', '?', '''
- Punct::new() will not panic
- as_char() returns a valid punctuation character

**Evidence:** const LEGAL_CHARS: &[char] = &[...] defines valid characters; if !LEGAL_CHARS.contains(&ch) { panic!("unsupported character `{:?}`", ch); } runtime check; panic message reveals the invariant: character must be supported

**Implementation:** Create ValidPunctChar newtype that can only be constructed with valid characters; Punct::new(ch: ValidPunctChar, spacing: Spacing) eliminates runtime panic

---

### 32. TokenStream::Valid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-47`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TokenStream contains Some(stream) and can be used for macro expansion operations

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)


/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {

// ... (other code) ...

    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.

```

**Entity:** TokenStream

**State:** Valid

**State invariants:**
- self.0.is_some() == true
- expand_expr() can proceed with actual expansion
- underlying stream is accessible for bridge operations

**Evidence:** let stream = self.0.as_ref().ok_or(ExpandError)? in expand_expr(); TokenStream(Some(stream)) wrapping pattern; Option<T> field self.0 indicates valid/invalid states

**Implementation:** Split into TokenStream<Valid> and TokenStream<Invalid>; expand_expr() only available on TokenStream<Valid>; construction methods return appropriate typed variant

---

### 33. TokenStream::Invalid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-47`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TokenStream contains None and cannot be used for macro expansion operations

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)


/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {

// ... (other code) ...

    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.

```

**Entity:** TokenStream

**State:** Invalid

**State invariants:**
- self.0.is_none() == true
- expand_expr() immediately returns Err(ExpandError)
- no valid expansion operations possible

**Evidence:** self.0.as_ref().ok_or(ExpandError)? fails when None; Option<T> field self.0 encodes validity at runtime; ExpandError returned for invalid state

**Implementation:** TokenStream<Invalid> has no expand_expr() method — compile error instead of runtime ExpandError

---

### 34. Ident::ValidIdentifier state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-272`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Ident contains a valid Rust identifier string that has been validated and NFC-normalized

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {

// ... (other code) ...

    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...


        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {

// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

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

// ... (other code) ...

    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }


// ... (other code) ...

    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

// ... (other code) ...

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

// ... (other code) ...

        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {

// ... (other code) ...

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Ident

**State:** ValidIdentifier

**State invariants:**
- string argument is a valid identifier permitted by the language
- identifier is NFC-normalized
- for raw identifiers, keywords usable in path segments are rejected

**Evidence:** /// The `string` argument must be a valid identifier permitted by the language (including keywords, e.g. `self` or `fn`). Otherwise, the function will panic.; /// Keywords which are usable in path segments (e.g. `self`, `super`) are not supported, and will cause a panic.; bridge::client::Symbol::new_ident(string, false) - validation happens in bridge layer

**Implementation:** Create ValidatedIdentifierString newtype that performs validation at construction, then Ident::new takes ValidatedIdentifierString instead of &str, eliminating runtime panics

---

### 38. Literal::ByteCharacter state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Literal contains a byte character (bridge::LitKind::Char) and byte_character_value() will succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

// ... (other code) ...

        })
    }

    /// Returns the unescaped character value if the current literal is a byte character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a string or a string literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn str_value(&self) -> Result<String, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Str => {
                if symbol.contains('\\') {
                    let mut buf = String::with_capacity(symbol.len());
                    let mut error = None;

// ... (other code) ...

                        #[inline(always)]
                        |_, c| match c {
                            Ok(c) => buf.push(c),
                            Err(err) => {
                                if err.is_fatal() {
                                    error = Some(ConversionErrorKind::FailedToUnescape(err));
                                }
                            }
                        },
                    );
                    if let Some(error) = error { Err(error) } else { Ok(buf) }
                } else {
                    Ok(symbol.to_string())
                }
            }
            bridge::LitKind::StrRaw(_) => Ok(symbol.to_string()),
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a c-string or a c-string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn cstr_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::CStr => {
                let mut error = None;
                let mut buf = Vec::with_capacity(symbol.len());


// ... (other code) ...

                        buf.extend_from_slice(c.get().encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Ok(MixedUnit::HighByte(b)) => buf.push(b.get()),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error {
                    Err(error)

// ... (other code) ...

                // char.
                let mut buf = symbol.to_owned().into_bytes();
                buf.push(0);
                Ok(buf)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a byte string or a byte string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_str_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::ByteStr => {
                let mut buf = Vec::with_capacity(symbol.len());
                let mut error = None;

                unescape_byte_str(symbol, |_, res| match res {
                    Ok(b) => buf.push(b),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error { Err(error) } else { Ok(buf) }
            }
            bridge::LitKind::ByteStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>`.
                Ok(symbol.to_owned().into_bytes())
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }
}

/// Parse a single literal from its stringified representation.

```

**Entity:** Literal

**State:** ByteCharacter

**State invariants:**
- self.0.kind == bridge::LitKind::Char
- symbol represents a valid byte character
- byte_character_value() returns Ok(u8)

**Evidence:** match self.0.kind { bridge::LitKind::Char => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) }; Err(ConversionErrorKind::InvalidLiteralKind) for wrong literal kind; unescape_byte(symbol) called only when kind matches

**Implementation:** Split Literal into Literal<ByteChar>, Literal<Char>, Literal<Str>, etc. with PhantomData; byte_character_value() only available on Literal<ByteChar>

---

### 39. Literal::Character state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Literal contains a character (bridge::LitKind::Char) and character_value() will succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

// ... (other code) ...

        })
    }

    /// Returns the unescaped character value if the current literal is a byte character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a string or a string literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn str_value(&self) -> Result<String, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Str => {
                if symbol.contains('\\') {
                    let mut buf = String::with_capacity(symbol.len());
                    let mut error = None;

// ... (other code) ...

                        #[inline(always)]
                        |_, c| match c {
                            Ok(c) => buf.push(c),
                            Err(err) => {
                                if err.is_fatal() {
                                    error = Some(ConversionErrorKind::FailedToUnescape(err));
                                }
                            }
                        },
                    );
                    if let Some(error) = error { Err(error) } else { Ok(buf) }
                } else {
                    Ok(symbol.to_string())
                }
            }
            bridge::LitKind::StrRaw(_) => Ok(symbol.to_string()),
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a c-string or a c-string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn cstr_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::CStr => {
                let mut error = None;
                let mut buf = Vec::with_capacity(symbol.len());


// ... (other code) ...

                        buf.extend_from_slice(c.get().encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Ok(MixedUnit::HighByte(b)) => buf.push(b.get()),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error {
                    Err(error)

// ... (other code) ...

                // char.
                let mut buf = symbol.to_owned().into_bytes();
                buf.push(0);
                Ok(buf)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a byte string or a byte string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_str_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::ByteStr => {
                let mut buf = Vec::with_capacity(symbol.len());
                let mut error = None;

                unescape_byte_str(symbol, |_, res| match res {
                    Ok(b) => buf.push(b),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error { Err(error) } else { Ok(buf) }
            }
            bridge::LitKind::ByteStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>`.
                Ok(symbol.to_owned().into_bytes())
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }
}

/// Parse a single literal from its stringified representation.

```

**Entity:** Literal

**State:** Character

**State invariants:**
- self.0.kind == bridge::LitKind::Char
- symbol represents a valid character
- character_value() returns Ok(char)

**Evidence:** match self.0.kind { bridge::LitKind::Char => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) }; Err(ConversionErrorKind::InvalidLiteralKind) for wrong literal kind; unescape_char(symbol) called only when kind matches

**Implementation:** character_value() only callable on Literal<Char> — compile error instead of InvalidLiteralKind runtime error

---

### 40. Literal::String state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Literal contains a string (bridge::LitKind::Str or bridge::LitKind::StrRaw) and str_value() will succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

// ... (other code) ...

        })
    }

    /// Returns the unescaped character value if the current literal is a byte character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a string or a string literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn str_value(&self) -> Result<String, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Str => {
                if symbol.contains('\\') {
                    let mut buf = String::with_capacity(symbol.len());
                    let mut error = None;

// ... (other code) ...

                        #[inline(always)]
                        |_, c| match c {
                            Ok(c) => buf.push(c),
                            Err(err) => {
                                if err.is_fatal() {
                                    error = Some(ConversionErrorKind::FailedToUnescape(err));
                                }
                            }
                        },
                    );
                    if let Some(error) = error { Err(error) } else { Ok(buf) }
                } else {
                    Ok(symbol.to_string())
                }
            }
            bridge::LitKind::StrRaw(_) => Ok(symbol.to_string()),
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a c-string or a c-string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn cstr_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::CStr => {
                let mut error = None;
                let mut buf = Vec::with_capacity(symbol.len());


// ... (other code) ...

                        buf.extend_from_slice(c.get().encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Ok(MixedUnit::HighByte(b)) => buf.push(b.get()),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error {
                    Err(error)

// ... (other code) ...

                // char.
                let mut buf = symbol.to_owned().into_bytes();
                buf.push(0);
                Ok(buf)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a byte string or a byte string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_str_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::ByteStr => {
                let mut buf = Vec::with_capacity(symbol.len());
                let mut error = None;

                unescape_byte_str(symbol, |_, res| match res {
                    Ok(b) => buf.push(b),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error { Err(error) } else { Ok(buf) }
            }
            bridge::LitKind::ByteStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>`.
                Ok(symbol.to_owned().into_bytes())
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }
}

/// Parse a single literal from its stringified representation.

```

**Entity:** Literal

**State:** String

**State invariants:**
- self.0.kind == bridge::LitKind::Str || self.0.kind == bridge::LitKind::StrRaw(_)
- symbol represents a valid string literal
- str_value() returns Ok(String)

**Evidence:** match self.0.kind { bridge::LitKind::Str => ... bridge::LitKind::StrRaw(_) => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) }; Err(ConversionErrorKind::InvalidLiteralKind) for wrong literal kind; Different handling for Str vs StrRaw but both valid for str_value()

**Implementation:** str_value() only callable on Literal<Str> or Literal<StrRaw> — InvalidLiteralKind becomes compile error

---

### 41. Literal::CString state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Literal contains a C string (bridge::LitKind::CStr) and cstr_value() will succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

// ... (other code) ...

        })
    }

    /// Returns the unescaped character value if the current literal is a byte character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a string or a string literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn str_value(&self) -> Result<String, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Str => {
                if symbol.contains('\\') {
                    let mut buf = String::with_capacity(symbol.len());
                    let mut error = None;

// ... (other code) ...

                        #[inline(always)]
                        |_, c| match c {
                            Ok(c) => buf.push(c),
                            Err(err) => {
                                if err.is_fatal() {
                                    error = Some(ConversionErrorKind::FailedToUnescape(err));
                                }
                            }
                        },
                    );
                    if let Some(error) = error { Err(error) } else { Ok(buf) }
                } else {
                    Ok(symbol.to_string())
                }
            }
            bridge::LitKind::StrRaw(_) => Ok(symbol.to_string()),
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a c-string or a c-string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn cstr_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::CStr => {
                let mut error = None;
                let mut buf = Vec::with_capacity(symbol.len());


// ... (other code) ...

                        buf.extend_from_slice(c.get().encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Ok(MixedUnit::HighByte(b)) => buf.push(b.get()),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error {
                    Err(error)

// ... (other code) ...

                // char.
                let mut buf = symbol.to_owned().into_bytes();
                buf.push(0);
                Ok(buf)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a byte string or a byte string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_str_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::ByteStr => {
                let mut buf = Vec::with_capacity(symbol.len());
                let mut error = None;

                unescape_byte_str(symbol, |_, res| match res {
                    Ok(b) => buf.push(b),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error { Err(error) } else { Ok(buf) }
            }
            bridge::LitKind::ByteStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>`.
                Ok(symbol.to_owned().into_bytes())
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }
}

/// Parse a single literal from its stringified representation.

```

**Entity:** Literal

**State:** CString

**State invariants:**
- self.0.kind == bridge::LitKind::CStr
- symbol represents a valid C string literal
- cstr_value() returns Ok(Vec<u8>)

**Evidence:** match self.0.kind { bridge::LitKind::CStr => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) }; Err(ConversionErrorKind::InvalidLiteralKind) for wrong literal kind; Complex unescaping logic only executed when kind matches

**Implementation:** cstr_value() only callable on Literal<CStr> — type system prevents calling on wrong literal kinds

---

### 42. Literal::ByteString state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-137`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Literal contains a byte string (bridge::LitKind::ByteStr or bridge::LitKind::ByteStrRaw) and byte_str_value() will succeed

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

// ... (other code) ...

        })
    }

    /// Returns the unescaped character value if the current literal is a byte character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a string or a string literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn str_value(&self) -> Result<String, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Str => {
                if symbol.contains('\\') {
                    let mut buf = String::with_capacity(symbol.len());
                    let mut error = None;

// ... (other code) ...

                        #[inline(always)]
                        |_, c| match c {
                            Ok(c) => buf.push(c),
                            Err(err) => {
                                if err.is_fatal() {
                                    error = Some(ConversionErrorKind::FailedToUnescape(err));
                                }
                            }
                        },
                    );
                    if let Some(error) = error { Err(error) } else { Ok(buf) }
                } else {
                    Ok(symbol.to_string())
                }
            }
            bridge::LitKind::StrRaw(_) => Ok(symbol.to_string()),
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a c-string or a c-string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn cstr_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::CStr => {
                let mut error = None;
                let mut buf = Vec::with_capacity(symbol.len());


// ... (other code) ...

                        buf.extend_from_slice(c.get().encode_utf8(&mut [0; 4]).as_bytes())
                    }
                    Ok(MixedUnit::HighByte(b)) => buf.push(b.get()),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error {
                    Err(error)

// ... (other code) ...

                // char.
                let mut buf = symbol.to_owned().into_bytes();
                buf.push(0);
                Ok(buf)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped string value if the current literal is a byte string or a byte string
    /// literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn byte_str_value(&self) -> Result<Vec<u8>, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::ByteStr => {
                let mut buf = Vec::with_capacity(symbol.len());
                let mut error = None;

                unescape_byte_str(symbol, |_, res| match res {
                    Ok(b) => buf.push(b),
                    Err(err) => {
                        if err.is_fatal() {
                            error = Some(ConversionErrorKind::FailedToUnescape(err));
                        }
                    }
                });
                if let Some(error) = error { Err(error) } else { Ok(buf) }
            }
            bridge::LitKind::ByteStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>`.
                Ok(symbol.to_owned().into_bytes())
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }
}

/// Parse a single literal from its stringified representation.

```

**Entity:** Literal

**State:** ByteString

**State invariants:**
- self.0.kind == bridge::LitKind::ByteStr || self.0.kind == bridge::LitKind::ByteStrRaw(_)
- symbol represents a valid byte string literal
- byte_str_value() returns Ok(Vec<u8>)

**Evidence:** match self.0.kind { bridge::LitKind::ByteStr => ... bridge::LitKind::ByteStrRaw(_) => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) }; Err(ConversionErrorKind::InvalidLiteralKind) for wrong literal kind; Different handling for ByteStr vs ByteStrRaw but both valid for byte_str_value()

**Implementation:** byte_str_value() only callable on Literal<ByteStr> or Literal<ByteStrRaw> — prevents runtime InvalidLiteralKind errors

---

### 43. Literal::ValidFloat state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-419`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Float literals must contain finite values (not NaN or infinity) to be valid

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.

// ... (other code) ...

        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

        isize_unsuffixed => isize,
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_unsuffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_suffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f32"))
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_unsuffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f64` where the value
    /// specified is the preceding part of the token and `f64` is the suffix of
    /// the token. This token will always be inferred to be an `f64` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_suffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f64"))
    }

    /// String literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn string(string: &str) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.as_bytes(), escape);
        Literal::new(bridge::LitKind::Str, &repr, None)
    }

    /// Character literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn character(ch: char) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: false,
        };
        let repr = escape_bytes(ch.encode_utf8(&mut [0u8; 4]).as_bytes(), escape);
        Literal::new(bridge::LitKind::Char, &repr, None)
    }

    /// Byte character literal.
    #[stable(feature = "proc_macro_byte_character", since = "1.79.0")]
    pub fn byte_character(byte: u8) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: true,
        };
        let repr = escape_bytes(&[byte], escape);
        Literal::new(bridge::LitKind::Byte, &repr, None)
    }

    /// Byte string literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn byte_string(bytes: &[u8]) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: true,
        };
        let repr = escape_bytes(bytes, escape);
        Literal::new(bridge::LitKind::ByteStr, &repr, None)
    }

    /// C string literal.
    #[stable(feature = "proc_macro_c_str_literals", since = "1.79.0")]
    pub fn c_string(string: &CStr) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.to_bytes(), escape);
        Literal::new(bridge::LitKind::CStr, &repr, None)
    }

    /// Returns the span encompassing this literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {

// ... (other code) ...

    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the u
// ... (truncated) ...
```

**Entity:** Literal

**State:** ValidFloat

**State invariants:**
- n.is_finite() == true for all float values
- Float representation is well-formed
- Can be safely converted to string representation

**Evidence:** if !n.is_finite() { panic!("Invalid float literal {n}"); } in f32_unsuffixed, f32_suffixed, f64_unsuffixed, f64_suffixed; Runtime check prevents invalid float literals from being created; Panic message "Invalid float literal" names the invariant

**Implementation:** Create FiniteF32/FiniteF64 newtypes with TryFrom<f32/f64> that validates finiteness; Literal constructors take these validated types instead of raw floats

---

### 44. Literal::KindMatched state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-419`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Literal value extraction methods require the literal to have the matching kind

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),

// ... (other code) ...

) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()

// ... (other code) ...

    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Literal(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Literal),
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for TokenTree {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///

// ... (other code) ...

    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

/// Prints token tree in a form convenient for debugging.

// ... (other code) ...

        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

        TokenTree::Punct(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Literal> for TokenTree {
    fn from(g: Literal) -> TokenTree {
        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.

// ... (other code) ...

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

// ... (other code) ...

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

// ... (other code) ...

        isize_unsuffixed => isize,
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_unsuffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f32` where the value
    /// specified is the preceding part of the token and `f32` is the suffix of
    /// the token. This token will always be inferred to be an `f32` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f32_suffixed(n: f32) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f32"))
    }

    /// Creates a new unsuffixed floating-point literal.
    ///
    /// This constructor is similar to those like `Literal::i8_unsuffixed` where
    /// the float's value is emitted directly into the token but no suffix is
    /// used, so it may be inferred to be a `f64` later in the compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_unsuffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        let mut repr = n.to_string();
        if !repr.contains('.') {
            repr.push_str(".0");
        }
        Literal::new(bridge::LitKind::Float, &repr, None)
    }

    /// Creates a new suffixed floating-point literal.
    ///
    /// This constructor will create a literal like `1.0f64` where the value
    /// specified is the preceding part of the token and `f64` is the suffix of
    /// the token. This token will always be inferred to be an `f64` in the
    /// compiler.
    /// Literals created from negative numbers might not survive rountrips through
    /// `TokenStream` or strings and may be broken into two tokens (`-` and positive literal).
    ///
    /// # Panics
    ///
    /// This function requires that the specified float is finite, for
    /// example if it is infinity or NaN this function will panic.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn f64_suffixed(n: f64) -> Literal {
        if !n.is_finite() {
            panic!("Invalid float literal {n}");
        }
        Literal::new(bridge::LitKind::Float, &n.to_string(), Some("f64"))
    }

    /// String literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn string(string: &str) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.as_bytes(), escape);
        Literal::new(bridge::LitKind::Str, &repr, None)
    }

    /// Character literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn character(ch: char) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: false,
        };
        let repr = escape_bytes(ch.encode_utf8(&mut [0u8; 4]).as_bytes(), escape);
        Literal::new(bridge::LitKind::Char, &repr, None)
    }

    /// Byte character literal.
    #[stable(feature = "proc_macro_byte_character", since = "1.79.0")]
    pub fn byte_character(byte: u8) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: true,
            escape_double_quote: false,
            escape_nonascii: true,
        };
        let repr = escape_bytes(&[byte], escape);
        Literal::new(bridge::LitKind::Byte, &repr, None)
    }

    /// Byte string literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn byte_string(bytes: &[u8]) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: true,
        };
        let repr = escape_bytes(bytes, escape);
        Literal::new(bridge::LitKind::ByteStr, &repr, None)
    }

    /// C string literal.
    #[stable(feature = "proc_macro_c_str_literals", since = "1.79.0")]
    pub fn c_string(string: &CStr) -> Literal {
        let escape = EscapeOptions {
            escape_single_quote: false,
            escape_double_quote: true,
            escape_nonascii: false,
        };
        let repr = escape_bytes(string.to_bytes(), escape);
        Literal::new(bridge::LitKind::CStr, &repr, None)
    }

    /// Returns the span encompassing this literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {

// ... (other code) ...

    pub fn byte_character_value(&self) -> Result<u8, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_byte(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the unescaped character value if the current literal is a character literal.
    #[unstable(feature = "proc_macro_value", issue = "136652")]
    pub fn character_value(&self) -> Result<char, ConversionErrorKind> {
        self.0.symbol.with(|symbol| match self.0.kind {
            bridge::LitKind::Char => {
                unescape_char(symbol).map_err(ConversionErrorKind::FailedToUnescape)
            }
            _ => Err(ConversionErrorKind::InvalidLiteralKind),
        })
    }

    /// Returns the u
// ... (truncated) ...
```

**Entity:** Literal

**State:** KindMatched

**State invariants:**
- self.0.kind matches the expected LitKind for the extraction method
- symbol contains data compatible with the requested type
- Extraction will succeed without InvalidLiteralKind error

**Evidence:** match self.0.kind { bridge::LitKind::Char => ... _ => Err(ConversionErrorKind::InvalidLiteralKind) } in character_value and byte_character_value; ConversionErrorKind::InvalidLiteralKind error indicates type mismatch; Runtime kind checking in value extraction methods

**Implementation:** Parameterize Literal<K: LiteralKind> where K is a zero-sized type marker (CharLiteral, StringLiteral, etc.); value extraction methods only available on appropriate Literal<K> types

---

### 46. TokenStream::Valid state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-258`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TokenStream contains Some(bridge_stream) with valid token data that can be safely operated on

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...


fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

// ... (other code) ...

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }

// ... (other code) ...

impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {

// ... (other code) ...

        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...

/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {

// ... (other code) ...

    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
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

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** Valid

**State invariants:**
- inner Option<BridgeTokenStream> is Some(_)
- bridge token stream is well-formed
- iteration and conversion operations are valid

**Transitions:**
- Valid -> Valid via operations
- Invalid -> Valid via successful parsing

**Evidence:** TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)); TokenStream(self.0.stream.clone()) assumes inner stream exists; self.0.next().map() in Iterator impl assumes valid bridge iterator

**Implementation:** NonEmptyTokenStream newtype that guarantees the inner Option is Some; parsing returns Result<NonEmptyTokenStream, LexError>

---

### 47. TokenStream::Empty state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-258`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TokenStream contains None, representing an empty or invalid token stream

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...


fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

// ... (other code) ...

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }

// ... (other code) ...

impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {

// ... (other code) ...

        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...

/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {

// ... (other code) ...

    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
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

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** Empty

**State invariants:**
- inner Option<BridgeTokenStream> is None
- iteration yields no items
- display shows empty string

**Transitions:**
- Empty -> Valid via successful parsing

**Evidence:** TokenStream(Some(...)) pattern suggests None case exists; Option wrapper around bridge stream indicates empty/invalid states; map_err(LexError) suggests parsing can fail leaving empty state

**Implementation:** Separate EmptyTokenStream and ValidTokenStream types; operations only available on ValidTokenStream

---

### 48. Group::Consistent delimiter-stream state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-258`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: Group's delimiter and token stream are semantically consistent - the stream content matches what the delimiter promises

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...


fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

// ... (other code) ...

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }

// ... (other code) ...

impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {

// ... (other code) ...

        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...

/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {

// ... (other code) ...

    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
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

// ... (other code) ...

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

```

**Entity:** Group

**State:** Consistent

**State invariants:**
- delimiter type matches the actual bracketing in the stream
- DelimSpan open/close spans are properly paired
- internal TokenStream is valid for the delimiter type

**Transitions:**
- Consistent -> Consistent via span updates

**Evidence:** Group(bridge::Group { delimiter, stream: stream.0, span: bridge::DelimSpan::from_single(Span::call_site().0) }); set_span updates DelimSpan but doesn't validate consistency; Comment: 'except for possibly TokenTree::Group with Delimiter::None delimiters' suggests edge cases

**Implementation:** Group<D: DelimiterType> where D encodes the delimiter at type level; ParenGroup, BraceGroup, BracketGroup, NoneGroup with appropriate stream validation

---

### 49. TokenTree::Variant consistency state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-258`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: TokenTree variant matches its contained data type - no type confusion between Group/Ident/Punct/Literal

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...


fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...

    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

// ... (other code) ...

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub enum TokenTree {
    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    ///
    /// Note that if this token is a `Group` then this method will not configure
    /// the span of each of the internal tokens, this will simply delegate to
    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }

// ... (other code) ...

impl fmt::Debug for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Group> for TokenTree {
    fn from(g: Group) -> TokenTree {
        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {

// ... (other code) ...

        TokenTree::Literal(g)
    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as

// ... (other code) ...

/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Send for Group {}
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {

// ... (other code) ...

    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
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

// ... (other code) ...

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

```

**Entity:** TokenTree

**State:** VariantConsistent

**State invariants:**
- TokenTree::Group contains valid Group data
- TokenTree::Ident contains valid Ident data
- TokenTree::Punct contains valid Punct data
- TokenTree::Literal contains valid Literal data

**Transitions:**
- VariantConsistent -> VariantConsistent via operations

**Evidence:** match *self pattern in span(), set_span(), fmt() assumes variant consistency; bridge::TokenTree conversion assumes 1:1 mapping between variants; From<Group/Ident/Punct/Literal> for TokenTree creates specific variants

**Implementation:** Sealed trait TokenTreeVariant with GroupTree, IdentTree, PunctTree, LiteralTree newtypes; TokenTree<V: TokenTreeVariant>

---

### 54. Group::ValidDelimiterStream state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-107`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Group contains a valid TokenStream with matching delimiter semantics

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {
    /// `( ... )`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Parenthesis,
    /// `{ ... }`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    ///
    /// Note: rustc currently can ignore the grouping of tokens delimited by `None` in the output
    /// of a proc_macro. Only `None`-delimited groups created by a macro_rules macro in the input
    /// of a proc_macro macro are preserved, and only in very specific circumstances.
    /// Any `None`-delimited groups (re)created by a proc_macro will therefore not preserve
    /// operator priorities as indicated above. The other `Delimiter` variants should be used
    /// instead in this context. This is a rustc bug. For details, see
    /// [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).
    ///
    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Group

**State:** ValidDelimiterStream

**State invariants:**
- delimiter and stream contents are semantically consistent
- TokenStream is well-formed for the given Delimiter
- Delimiter::None groups preserve operator precedence only in specific rustc contexts

**Evidence:** Comment warns about rustc bug with Delimiter::None groups losing operator priorities; Display implementation converts through TokenTree::from(self.clone()) suggesting validation dependency; Constructor Group::new() accepts any Delimiter + TokenStream combination without validation

**Implementation:** Create ValidatedGroup<D: DelimiterType> where DelimiterType encodes semantic constraints; Group::new() validates delimiter-stream compatibility at construction

---

### 57. TokenStream::LosslesslyConvertible state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-107`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TokenStream/TokenTree can be converted to string and back without semantic loss

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {
    /// `( ... )`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Parenthesis,
    /// `{ ... }`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    ///
    /// Note: rustc currently can ignore the grouping of tokens delimited by `None` in the output
    /// of a proc_macro. Only `None`-delimited groups created by a macro_rules macro in the input
    /// of a proc_macro macro are preserved, and only in very specific circumstances.
    /// Any `None`-delimited groups (re)created by a proc_macro will therefore not preserve
    /// operator priorities as indicated above. The other `Delimiter` variants should be used
    /// instead in this context. This is a rustc bug. For details, see
    /// [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).
    ///
    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** TokenStream/TokenTree

**State:** LosslesslyConvertible

**State invariants:**
- to_string() output can be parsed back to equivalent TokenStream
- Spans may be lost but token structure preserved
- Delimiter::None groups may not round-trip correctly
- Negative numeric literals may not round-trip correctly

**Transitions:**
- LosslesslyConvertible -> PotentiallyLossy via to_string()

**Evidence:** Comment: 'supposed to be losslessly convertible back into the same token stream (modulo spans)'; Comment: 'except for possibly TokenTree::Group s with Delimiter::None delimiters and negative numeric literals'; Warning: 'you should *not* do any kind of simple substring matching on the output string'

**Implementation:** LosslessTokenStream vs GeneralTokenStream types; only LosslessTokenStream guarantees round-trip conversion; explicit conversion acknowledges potential loss

---

### 58. Punct::ValidCharacter state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-66`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Punct can only be constructed with characters from the predefined LEGAL_CHARS set

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

}

/// A `Punct` is a single punctuation character such as `+`, `-` or `#`.
///
/// Multi-character operators like `+=` are represented as two instances of `Punct` with different
/// forms of `Spacing` returned.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
#[derive(Clone)]
pub struct Punct(bridge::Punct<bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...


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

// ... (other code) ...

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

// ... (other code) ...


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

```

**Entity:** Punct

**State:** ValidCharacter

**State invariants:**
- ch must be one of: '=', '<', '>', '!', '~', '+', '-', '*', '/', '%', '^', '&', '|', '@', '.', ',', ';', ':', '#', '$', '?', '''
- construction with invalid characters causes panic
- once constructed, the character is guaranteed valid

**Evidence:** const LEGAL_CHARS: &[char] array defines valid characters; if !LEGAL_CHARS.contains(&ch) { panic!("unsupported character `{:?}`", ch); } runtime check; panic message explicitly names the invariant: "unsupported character"

**Implementation:** Create ValidPunctChar newtype that can only be constructed through a fallible constructor; Punct::new takes ValidPunctChar instead of char, eliminating runtime panic

---

### 61. proc_macro::Available context

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: proc_macro infrastructure is available - running inside a procedural macro

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** proc_macro module

**State:** Available

**State invariants:**
- is_available() returns true
- All proc_macro APIs work without panicking
- Bridge methods can be called safely

**Transitions:**
- Available -> Unavailable when macro execution ends

**Evidence:** is_available() function exists to check this condition; Documentation: "All the functions in this crate panic if invoked from outside of a procedural macro"; bridge::client::is_available() underlying check

**Implementation:** Require a ProcMacroContext capability token to access proc_macro APIs; is_available() returns Option<ProcMacroContext>; all APIs take &ProcMacroContext parameter

---

### 62. proc_macro::Unavailable context

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-436`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: proc_macro infrastructure is not available - running outside procedural macro context

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing

//! A support library for macro authors when defining new macros.
//!
//! This library, provided by the standard distribution, provides the types
//! consumed in the interfaces of procedurally defined macro definitions such as
//! function-like macros `#[proc_macro]`, macro attributes `#[proc_macro_attribute]` and
//! custom derive attributes `#[proc_macro_derive]`.
//!
//! See [the book] for more.
//!
//! [the book]: ../book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes

#![stable(feature = "proc_macro_lib", since = "1.15.0")]
#![deny(missing_docs)]
#![doc(
    html_playground_url = "https://play.rust-lang.org/",
    issue_tracker_base_url = "https://github.com/rust-lang/rust/issues/",
    test(no_crate_inject, attr(deny(warnings))),
    test(attr(allow(dead_code, deprecated, unused_variables, unused_mut)))
)]
#![doc(rust_logo)]
#![feature(rustdoc_internals)]
#![feature(staged_api)]
#![feature(allow_internal_unstable)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(panic_can_unwind)]
#![feature(restricted_std)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(mem_conjure_zst)]
#![recursion_limit = "256"]
#![allow(internal_features)]
#![deny(ffi_unwind_calls)]
#![allow(rustc::internal)] // Can't use FxHashMap when compiled as part of the standard library
#![warn(rustdoc::unescaped_backticks)]
#![warn(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]

#[unstable(feature = "proc_macro_internals", issue = "27812")]
#[doc(hidden)]
pub mod bridge;

mod diagnostic;
mod escape;
mod to_tokens;

use core::ops::BitOr;
use std::ffi::CStr;
use std::ops::{Range, RangeBounds};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

#[unstable(feature = "proc_macro_diagnostic", issue = "54140")]
pub use diagnostic::{Diagnostic, Level, MultiSpan};
#[unstable(feature = "proc_macro_value", issue = "136652")]
pub use rustc_literal_escaper::EscapeError;
use rustc_literal_escaper::{
    MixedUnit, unescape_byte, unescape_byte_str, unescape_c_str, unescape_char, unescape_str,
};
#[unstable(feature = "proc_macro_totokens", issue = "130977")]
pub use to_tokens::ToTokens;

use crate::bridge::client::Methods as BridgeMethods;
use crate::escape::{EscapeOptions, escape_bytes};

/// Errors returned when trying to retrieve a literal unescaped value.
#[unstable(feature = "proc_macro_value", issue = "136652")]
#[derive(Debug, PartialEq, Eq)]
pub enum ConversionErrorKind {
    /// The literal failed to be escaped, take a look at [`EscapeError`] for more information.
    FailedToUnescape(EscapeError),
    /// Trying to convert a literal with the wrong type.
    InvalidLiteralKind,
}

/// Determines whether proc_macro has been made accessible to the currently
/// running program.
///
/// The proc_macro crate is only intended for use inside the implementation of
/// procedural macros. All the functions in this crate panic if invoked from
/// outside of a procedural macro, such as from a build script or unit test or
/// ordinary Rust binary.
///
/// With consideration for Rust libraries that are designed to support both
/// macro and non-macro use cases, `proc_macro::is_available()` provides a
/// non-panicking way to detect whether the infrastructure required to use the
/// API of proc_macro is presently available. Returns true if invoked from
/// inside of a procedural macro, false if invoked from any other binary.
#[stable(feature = "proc_macro_is_available", since = "1.57.0")]
pub fn is_available() -> bool {
    bridge::client::is_available()
}

/// The main type provided by this crate, representing an abstract stream of
/// tokens, or, more specifically, a sequence of token trees.
/// The type provides interfaces for iterating over those token trees and, conversely,
/// collecting a number of token trees into one stream.
///
/// This is both the input and output of `#[proc_macro]`, `#[proc_macro_attribute]`
/// and `#[proc_macro_derive]` definitions.
#[cfg_attr(feature = "rustc-dep-of-std", rustc_diagnostic_item = "TokenStream")]
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[derive(Clone)]
pub struct TokenStream(Option<bridge::client::TokenStream>);

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for TokenStream {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for TokenStream {}

/// Error returned from `TokenStream::from_str`.
///
/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]
pub struct ExpandError;

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("macro expansion failed")
    }
}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl error::Error for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Send for ExpandError {}

#[unstable(feature = "proc_macro_expand", issue = "90765")]
impl !Sync for ExpandError {}

impl TokenStream {
    /// Returns an empty `TokenStream` containing no token trees.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn new() -> TokenStream {
        TokenStream(None)
    }

    /// Checks if this `TokenStream` is empty.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map(|h| BridgeMethods::ts_is_empty(h)).unwrap_or(true)
    }

    /// Parses this `TokenStream` as an expression and attempts to expand any
    /// macros within it. Returns the expanded `TokenStream`.
    ///
    /// Currently only expressions expanding to literals will succeed, although
    /// this may be relaxed in the future.
    ///
    /// NOTE: In error conditions, `expand_expr` may leave macros unexpanded,
    /// report an error, failing compilation, and/or return an `Err(..)`. The
    /// specific behavior for any error condition, and what conditions are
    /// considered errors, is unspecified and may change in the future.
    #[unstable(feature = "proc_macro_expand", issue = "90765")]
    pub fn expand_expr(&self) -> Result<TokenStream, ExpandError> {
        let stream = self.0.as_ref().ok_or(ExpandError)?;
        match BridgeMethods::ts_expand_expr(stream) {
            Ok(stream) => Ok(TokenStream(Some(stream))),
            Err(_) => Err(ExpandError),
        }
    }
}

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(ts) => write!(f, "{}", BridgeMethods::ts_to_string(ts)),
            None => Ok(()),
        }
    }
}

/// Prints tokens in a form convenient for debugging.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Debug for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenStream ")?;
        f.debug_list().entries(self.clone()).finish()
    }
}

#[stable(feature = "proc_macro_token_stream_default", since = "1.45.0")]
impl Default for TokenStream {
    fn default() -> Self {
        TokenStream::new()
    }
}

#[unstable(feature = "proc_macro_quote", issue = "54722")]
pub use quote::{HasIterator, RepInterp, ThereIsNoIteratorInRepetition, ext, quote, quote_span};

fn tree_to_bridge_tree(
    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream(Some(BridgeMethods::ts_from_token_tree(tree_to_bridge_tree(tree))))
    }
}

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

/// Non-generic helper for implementing `FromIterator<TokenStream>` and
/// `Extend<TokenStream>` with less monomorphization in calling crates.
struct ConcatStreamsHelper {
    streams: Vec<bridge::client::TokenStream>,
}

impl ConcatStreamsHelper {
    fn new(capacity: usize) -> Self {
        ConcatStreamsHelper { streams: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, stream: TokenStream) {
        if let Some(stream) = stream.0 {
            self.streams.push(stream);
        }
    }

    fn build(mut self) -> TokenStream {
        if self.streams.len() <= 1 {
            TokenStream(self.streams.pop())
        } else {
            TokenStream(Some(BridgeMethods::ts_concat_streams(None, self.streams)))
        }
    }

    fn append_to(mut self, stream: &mut TokenStream) {
        if self.streams.is_empty() {
            return;
        }
        let base = stream.0.take();
        if base.is_none() && self.streams.len() == 1 {
            stream.0 = self.streams.pop();
        } else {
            stream.0 = Some(BridgeMethods::ts_concat_streams(base, self.streams));
        }
    }
}

/// Collects a number of token trees into a single stream.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(trees: I) -> Self {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.build()
    }
}

/// A "flattening" operation on token streams, collects token trees
/// from multiple token streams into a single stream.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.build()
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, trees: I) {
        let iter = trees.into_iter();
        let mut builder = ConcatTreesHelper::new(iter.size_hint().0);
        iter.for_each(|tree| builder.push(tree));
        builder.append_to(self);
    }
}

#[stable(feature = "token_stream_extend", since = "1.30.0")]
impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        let iter = streams.into_iter();
        let mut builder = ConcatStreamsHelper::new(iter.size_hint().0);
        iter.for_each(|stream| builder.push(stream));
        builder.append_to(self);
    }
}

macro_rules! extend_items {
    ($($item:ident)*) => {
        $(
            #[stable(feature = "token_stream_extend_ts_items", since = "1.92.0")]
            impl Extend<$item> for TokenStream {
                fn extend<T: IntoIterator<Item = $item>>(&mut self, iter: T) {
                    self.extend(iter.into_iter().map(TokenTree::$item));
                }
            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub struct IntoIter(
        std::vec::IntoIter<
            bridge::TokenTree<
                bridge::client::TokenStream,
                bridge::client::Span,
                bridge::client::Symbol,
            >,
        >,
    );

    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self
// ... (truncated) ...
```

**Entity:** proc_macro module

**State:** Unavailable

**State invariants:**
- is_available() returns false
- All proc_macro APIs will panic if called
- Bridge is not initialized

**Transitions:**
- Unavailable -> Available when entering macro execution

**Evidence:** Documentation warns about panics outside proc macro context; is_available() provides non-panicking detection; bridge::client::is_available() returns false

**Implementation:** Without ProcMacroContext token, proc_macro APIs are not accessible at compile time rather than panicking at runtime

---

## Protocol Invariants

### 14. personalities::Unreachable state

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Personality functions and exception handling symbols exist for linking compatibility but should never be executed in abort mode

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

**Entity:** personalities module

**State:** Unreachable

**State invariants:**
- rust_eh_catch_typeinfo exists but is never accessed
- No unwinding personality functions are called
- Exception handling is completely bypassed

**Evidence:** Comment: 'it should never be called as we don't link in an unwinding runtime at all'; Comment: 'any catch_unwind calls will never use this typeinfo'; Static symbols defined only for linking compatibility

**Implementation:** Use compile-time feature flags or phantom types to ensure personality functions are only available when unwinding is enabled; abort mode should not expose these symbols at all

---

### 18. LexError::Unstable state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-72`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Error message content is explicitly unstable and may change between versions

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// The contained error message is explicitly not guaranteed to be stable in any way,
/// and may change between Rust versions or across compilations.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
#[non_exhaustive]
#[derive(Debug)]
pub struct LexError(String);

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[stable(feature = "proc_macro_lexerror_impls", since = "1.44.0")]
impl error::Error for LexError {}

#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Send for LexError {}
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl !Sync for LexError {}

/// Error returned from `TokenStream::expand_expr`.
#[unstable(feature = "proc_macro_expand", issue = "90765")]
#[non_exhaustive]
#[derive(Debug)]

// ... (other code) ...

/// Attempts to break the string into tokens and parse those tokens into a token stream.
/// May fail for a number of reasons, for example, if the string contains unbalanced delimiters
/// or characters not existing in the language.
/// All tokens in the parsed stream get `Span::call_site()` spans.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We reserve the right to
/// change these errors into `LexError`s later.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(src: &str) -> Result<TokenStream, LexError> {
        Ok(TokenStream(Some(BridgeMethods::ts_from_str(src).map_err(LexError)?)))
    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s

// ... (other code) ...

/// but the literal token. Specifically, it must not contain whitespace or
/// comments in addition to the literal.
///
/// The resulting literal token will have a `Span::call_site()` span.
///
/// NOTE: some errors may cause panics instead of returning `LexError`. We
/// reserve the right to change these errors into `LexError`s later.
#[stable(feature = "proc_macro_literal_parse", since = "1.54.0")]
impl FromStr for Literal {
    type Err = LexError;

    fn from_str(src: &str) -> Result<Self, LexError> {
        match BridgeMethods::literal_from_str(src) {
            Ok(literal) => Ok(Literal(literal)),
            Err(msg) => Err(LexError(msg)),
        }
    }
}

/// Prints the literal as a string that should be losslessly convertible

```

**Entity:** LexError

**State:** Unstable

**State invariants:**
- Error message is not guaranteed stable across Rust versions
- Error message may change across compilations
- Only Display trait should be used, not direct message inspection

**Evidence:** Comment: "The contained error message is explicitly not guaranteed to be stable in any way"; Comment: "may change between Rust versions or across compilations"; String field is private, only accessible via Display

**Implementation:** Structured error enum with stable variants instead of String; Display impl can still change but pattern matching is stable

---

### 36. TokenStream::StringRepresentationInstability state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-272`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: TokenStream's string representation is unstable and should not be used for substring matching in proc macros

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {

// ... (other code) ...

    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...


        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {

// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

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

// ... (other code) ...

    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }


// ... (other code) ...

    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

// ... (other code) ...

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

// ... (other code) ...

        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {

// ... (other code) ...

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** TokenStream

**State:** StringRepresentationInstability

**State invariants:**
- to_string() output format may change between compiler versions
- whitespace between tokens is not guaranteed
- substring matching on string output is unreliable

**Evidence:** /// Note: the exact form of the output is subject to change, e.g. there might be changes in the whitespace used between tokens.; /// Therefore, you should *not* do any kind of simple substring matching on the output string; /// Instead, you should work at the `TokenTree` level

**Implementation:** Provide StableStringRepr capability token that must be explicitly requested, or remove Display impl entirely and force TokenTree-level processing

---

### 37. TokenTree::StructuralProcessing state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-272`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: TokenTree should be processed structurally via pattern matching rather than string operations

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib", since = "1.15.0")]
impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {

// ... (other code) ...

    tree: TokenTree,
) -> bridge::TokenTree<bridge::client::TokenStream, bridge::client::Span, bridge::client::Symbol> {
    match tree {
        TokenTree::Group(tt) => bridge::TokenTree::Group(tt.0),
        TokenTree::Punct(tt) => bridge::TokenTree::Punct(tt.0),
        TokenTree::Ident(tt) => bridge::TokenTree::Ident(tt.0),
        TokenTree::Literal(tt) => bridge::TokenTree::Literal(tt.0),
    }
}

/// Creates a token stream containing a single token tree.

// ... (other code) ...

            }
        )*
    };
}

extend_items!(Group Literal Punct Ident);

/// Public implementation details for the `TokenStream` type, such as iterators.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub mod token_stream {
    use crate::{BridgeMethods, Group, Ident, Literal, Punct, TokenStream, TokenTree, bridge};

    /// An iterator over `TokenStream`'s `TokenTree`s.
    /// The iteration is "shallow", e.g., the iterator doesn't recurse into delimited groups,
    /// and returns whole groups as token trees.
    #[derive(Clone)]

// ... (other code) ...


        fn next(&mut self) -> Option<TokenTree> {
            self.0.next().map(|tree| match tree {
                bridge::TokenTree::Group(tt) => TokenTree::Group(Group(tt)),
                bridge::TokenTree::Punct(tt) => TokenTree::Punct(Punct(tt)),
                bridge::TokenTree::Ident(tt) => TokenTree::Ident(Ident(tt)),
                bridge::TokenTree::Literal(tt) => TokenTree::Literal(Literal(tt)),
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {

// ... (other code) ...

    }
}

/// `quote!(..)` accepts arbitrary tokens and expands into a `TokenStream` describing the input.
/// For example, `quote!(a + b)` will produce an expression, that, when evaluated, constructs
/// the `TokenStream` `[Ident("a"), Punct('+', Alone), Ident("b")]`.
///
/// Unquoting is done with `$`, and works by taking the single next ident as the unquoted term.
/// To quote `$` itself, use `$$`.
#[unstable(feature = "proc_macro_quote", issue = "54722")]
#[allow_internal_unstable(proc_macro_def_site, proc_macro_internals, proc_macro_totokens)]

// ... (other code) ...

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

// ... (other code) ...

    /// A token stream surrounded by bracket delimiters.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Group(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Group),
    /// An identifier.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Ident(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Ident),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Punct(#[stable(feature = "proc_macro_lib2", since = "1.29.0")] Punct),
    /// A literal character (`'a'`), string (`"hello"`), number (`2.3`), etc.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    /// the contained token or a delimited stream.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn span(&self) -> Span {
        match *self {
            TokenTree::Group(ref t) => t.span(),
            TokenTree::Ident(ref t) => t.span(),
            TokenTree::Punct(ref t) => t.span(),
            TokenTree::Literal(ref t) => t.span(),
        }
    }


// ... (other code) ...

    /// the `set_span` method of each variant.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        match *self {
            TokenTree::Group(ref mut t) => t.set_span(span),
            TokenTree::Ident(ref mut t) => t.set_span(span),
            TokenTree::Punct(ref mut t) => t.set_span(span),
            TokenTree::Literal(ref mut t) => t.set_span(span),
        }
    }
}

// ... (other code) ...

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Each of these has the name in the struct type in the derived debug,
        // so don't bother with an extra layer of indirection
        match *self {
            TokenTree::Group(ref tt) => tt.fmt(f),
            TokenTree::Ident(ref tt) => tt.fmt(f),
            TokenTree::Punct(ref tt) => tt.fmt(f),
            TokenTree::Literal(ref tt) => tt.fmt(f),
        }
    }
}

// ... (other code) ...

        TokenTree::Group(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Ident> for TokenTree {
    fn from(g: Ident) -> TokenTree {
        TokenTree::Ident(g)
    }
}

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl From<Punct> for TokenTree {

// ... (other code) ...

/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching
/// might stop working if such changes happen. Instead, you should work at the
/// `TokenTree` level, e.g. matching against `TokenTree::Ident`,
/// `TokenTree::Punct`, or `TokenTree::Literal`.
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Group(t) => write!(f, "{t}"),
            TokenTree::Ident(t) => write!(f, "{t}"),
            TokenTree::Punct(t) => write!(f, "{t}"),
            TokenTree::Literal(t) => write!(f, "{t}"),
        }
    }
}

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** TokenTree

**State:** StructuralProcessing

**State invariants:**
- matching should be done against TokenTree::Ident, TokenTree::Punct, etc.
- string representation is for display only, not parsing
- structural access preserves semantic meaning

**Evidence:** /// you should work at the `TokenTree` level, e.g. matching against `TokenTree::Ident`, `TokenTree::Punct`, or `TokenTree::Literal`; Repeated warnings in both TokenStream and TokenTree Display impl docs; Rich enum structure with specific variant types

**Implementation:** Remove or restrict Display impl, provide StructuralAccess capability that forces pattern matching on enum variants instead of string operations

---

### 55. Group::PreservedPrecedence state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-107`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Group with Delimiter::None that preserves operator precedence in rustc compilation

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {
    /// `( ... )`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Parenthesis,
    /// `{ ... }`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    ///
    /// Note: rustc currently can ignore the grouping of tokens delimited by `None` in the output
    /// of a proc_macro. Only `None`-delimited groups created by a macro_rules macro in the input
    /// of a proc_macro macro are preserved, and only in very specific circumstances.
    /// Any `None`-delimited groups (re)created by a proc_macro will therefore not preserve
    /// operator priorities as indicated above. The other `Delimiter` variants should be used
    /// instead in this context. This is a rustc bug. For details, see
    /// [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).
    ///
    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Group

**State:** PreservedPrecedence

**State invariants:**
- Delimiter is None
- Group was created by macro_rules in proc_macro input
- Group exists in very specific rustc circumstances
- Operator priorities are preserved during compilation

**Transitions:**
- PreservedPrecedence -> LostPrecedence via proc_macro recreation

**Evidence:** Comment: 'Only None-delimited groups created by a macro_rules macro in the input of a proc_macro macro are preserved'; Comment: 'Any None-delimited groups (re)created by a proc_macro will therefore not preserve operator priorities'; Explicit warning about rustc bug rust-lang/rust#67062

**Implementation:** MacroRulesGroup vs ProcMacroGroup newtypes; only MacroRulesGroup preserves precedence; conversion from MacroRulesGroup to ProcMacroGroup is explicit and lossy

---

### 56. Group::LostPrecedence state

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-107`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Group with Delimiter::None that has lost operator precedence guarantees

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); struct Literal, impl Literal (20 methods), impl FromStr for Literal (1 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

    }
}

/// Prints the token stream as a string that is supposed to be losslessly convertible back
/// into the same token stream (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// Prints the token tree as a string that is supposed to be losslessly convertible back
/// into the same token tree (modulo spans), except for possibly `TokenTree::Group`s
/// with `Delimiter::None` delimiters and negative numeric literals.
///
/// Note: the exact form of the output is subject to change, e.g. there might
/// be changes in the whitespace used between tokens. Therefore, you should
/// *not* do any kind of simple substring matching on the output string (as
/// produced by `to_string`) to implement a proc macro, because that matching

// ... (other code) ...

    }
}

/// A delimited token stream.
///
/// A `Group` internally contains a `TokenStream` which is surrounded by `Delimiter`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Group(bridge::Group<bridge::client::TokenStream, bridge::client::Span>);

#[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

impl !Sync for Group {}

/// Describes how a sequence of token trees is delimited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub enum Delimiter {
    /// `( ... )`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    Parenthesis,
    /// `{ ... }`
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

    ///
    /// Note: rustc currently can ignore the grouping of tokens delimited by `None` in the output
    /// of a proc_macro. Only `None`-delimited groups created by a macro_rules macro in the input
    /// of a proc_macro macro are preserved, and only in very specific circumstances.
    /// Any `None`-delimited groups (re)created by a proc_macro will therefore not preserve
    /// operator priorities as indicated above. The other `Delimiter` variants should be used
    /// instead in this context. This is a rustc bug. For details, see
    /// [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).
    ///
    /// </div>
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]

// ... (other code) ...

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

// ... (other code) ...

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

```

**Entity:** Group

**State:** LostPrecedence

**State invariants:**
- Delimiter is None
- Group was (re)created by proc_macro
- Operator priorities are not preserved
- Should use other Delimiter variants for precedence

**Evidence:** Comment: 'Any None-delimited groups (re)created by a proc_macro will therefore not preserve operator priorities'; Comment: 'The other Delimiter variants should be used instead in this context'; Warning about rustc bug affecting precedence

**Implementation:** ProcMacroGroup type that cannot be used where precedence matters; compiler error instead of runtime precedence loss

---

### 68. Secondary process execution mode

**Location**: `/data/rust/library/test/src/lib.rs:1-446`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Process is running as a spawned secondary test process with specific environment variables set

**Evidence**:

```rust
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

**Entity:** test framework

**State:** SecondaryProcess

**State invariants:**
- SECONDARY_TEST_INVOKER_VAR environment variable is set
- process should run exactly one test and exit
- only static tests are supported in this mode

**Transitions:**
- SecondaryProcess -> ProcessExit via run_test_in_spawned_subprocess

**Evidence:** if let Ok(name) = env::var(SECONDARY_TEST_INVOKER_VAR) check; unsafe { env::remove_var(SECONDARY_TEST_INVOKER_VAR) } cleanup; panic!("only static tests are supported") for dynamic tests; panic!("benchmarks should not be executed into child processes")

**Implementation:** Separate ProcessMode<Primary> and ProcessMode<Secondary> types; test_main_static_abort only callable on Primary mode; secondary mode has restricted API

---

### 69. Primary process execution mode

**Location**: `/data/rust/library/test/src/lib.rs:1-446`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Process is running as the main test coordinator that can spawn secondary processes

**Evidence**:

```rust
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

**Entity:** test framework

**State:** PrimaryProcess

**State invariants:**
- SECONDARY_TEST_INVOKER_VAR is not set
- can spawn subprocesses for panic=abort tests
- handles full test suite coordination

**Transitions:**
- PrimaryProcess -> SecondaryProcess via subprocess spawn

**Evidence:** absence of SECONDARY_TEST_INVOKER_VAR check determines primary mode; RunStrategy::SpawnPrimary vs RunStrategy::InProcess distinction; opts.options.panic_abort && !opts.force_run_in_process condition

**Implementation:** ProcessMode<Primary> has spawn capabilities; ProcessMode<Secondary> has restricted single-test execution

---

## Skipped Files

28 file(s) could not be parsed:

- `/data/rust/library/alloc/src/vec/mod.rs`: expected identifier or `_`
- `/data/rust/library/alloc/src/raw_vec/mod.rs`: expected identifier or `_`
- `/data/rust/library/core/src/pin.rs`: expected identifier or `_`
- `/data/rust/library/core/src/slice/index.rs`: expected identifier or `_`
- `/data/rust/library/core/src/slice/cmp.rs`: expected identifier or `_`
- `/data/rust/library/core/src/slice/mod.rs`: expected identifier or `_`
- `/data/rust/library/core/src/pat.rs`: expected identifier or `_`
- `/data/rust/library/core/src/convert/mod.rs`: expected identifier or `_`
- `/data/rust/library/core/src/alloc/mod.rs`: expected identifier or `_`
- `/data/rust/library/core/src/array/equality.rs`: expected identifier or `_`
- `/data/rust/library/core/src/marker.rs`: expected identifier or `_`
- `/data/rust/library/core/src/borrow.rs`: expected identifier or `_`
- `/data/rust/library/core/src/cmp.rs`: expected identifier or `_`
- `/data/rust/library/core/src/str/traits.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/arith.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/try_trait.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/range.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/index.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/bit.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/drop.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/deref.rs`: expected identifier or `_`
- `/data/rust/library/core/src/ops/function.rs`: expected identifier or `_`
- `/data/rust/library/core/src/iter/traits/iterator.rs`: expected identifier or `_`
- `/data/rust/library/core/src/iter/traits/collect.rs`: expected identifier or `_`
- `/data/rust/library/core/src/default.rs`: expected identifier or `_`
- `/data/rust/library/core/src/cmp/bytewise.rs`: expected identifier or `_`
- `/data/rust/library/core/src/intrinsics/fallback.rs`: expected identifier or `_`
- `/data/rust/library/core/src/clone.rs`: expected identifier or `_`

