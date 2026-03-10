# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 0
- **Precondition**: 3
- **Protocol**: 1
- **Modules analyzed**: 1349

## Precondition Invariants

### 23. Punct::Constructed (valid punctuation char)

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-411`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Punct::new requires that ch is one of the language-permitted punctuation characters; otherwise it panics. After construction, the character is guaranteed to be in the allowed set.

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
- ch ∈ LEGAL_CHARS (language-permitted punctuation)
- initial span is Span::call_site()
- spacing flag is encoded as joint == (spacing == Spacing::Joint)

**Evidence:** line 159: doc states 'The ch argument must be a valid punctuation character permitted by the language, otherwise the function will panic.'; line 166: const LEGAL_CHARS lists the only allowed characters; line 170: runtime check `if !LEGAL_CHARS.contains(&ch)` enforces the precondition; line 171: panic! with message "unsupported character ..." reveals the invariant; line 176: constructor sets span to Span::call_site().0

**Implementation:** Introduce a ValidPunctChar newtype (constructed via TryFrom<char>) that validates membership in LEGAL_CHARS. Change Punct::new to accept ValidPunctChar (or provide Punct::try_new returning Result), eliminating the runtime panic and making invalid construction a type/return-typed error.

---

### 51. Literal::Float constructors require finite input

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-392`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Float literal constructors panic if provided NaN or infinite values; only finite floats are accepted.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TokenStream, 1 free function(s), impl Send for TokenStream (0 methods), impl Sync for TokenStream (0 methods), impl TokenStream (3 methods), impl FromStr for TokenStream (1 methods), impl From < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenTree > for TokenStream (1 methods), impl FromIterator < TokenStream > for TokenStream (1 methods), impl Extend < TokenTree > for TokenStream (1 methods), impl Extend < TokenStream > for TokenStream (1 methods), impl IntoIterator for TokenStream (1 methods); struct LexError, impl error :: Error for LexError (0 methods), impl Send for LexError (0 methods), impl Sync for LexError (0 methods); struct ExpandError, impl error :: Error for ExpandError (0 methods), impl Send for ExpandError (0 methods), impl Sync for ExpandError (0 methods); struct ConcatTreesHelper, impl ConcatTreesHelper (4 methods); struct ConcatStreamsHelper, impl ConcatStreamsHelper (4 methods); struct IntoIter, impl Iterator for IntoIter (3 methods); struct Span, impl Send for Span (0 methods), impl Sync for Span (0 methods), impl Span (19 methods); struct Group, impl Send for Group (0 methods), impl Sync for Group (0 methods), impl Group (7 methods); struct Punct, impl Send for Punct (0 methods), impl Sync for Punct (0 methods), impl Punct (5 methods), impl PartialEq < char > for Punct (1 methods); struct Ident, impl Ident (4 methods); enum ConversionErrorKind; enum TokenTree, impl Send for TokenTree (0 methods), impl Sync for TokenTree (0 methods), impl TokenTree (2 methods), impl From < Group > for TokenTree (1 methods), impl From < Ident > for TokenTree (1 methods), impl From < Punct > for TokenTree (1 methods), impl From < Literal > for TokenTree (1 methods); enum Delimiter; enum Spacing; 3 free function(s), impl PartialEq < Punct > for char (1 methods)

/// Boolean literals like `true` and `false` do not belong here, they are `Ident`s.
#[derive(Clone)]
#[stable(feature = "proc_macro_lib2", since = "1.29.0")]
pub struct Literal(bridge::Literal<bridge::client::Span, bridge::client::Symbol>);


// ... (other code) ...

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
        isize_suffixed => isize,
    }

    unsuffixed_int_literals! {
        u8_unsuffixed => u8,
        u16_unsuffixed => u16,
        u32_unsuffixed => u32,
        u64_unsuffixed => u64,
        u128_unsuffixed => u128,
        usize_unsuffixed => usize,
        i8_unsuffixed => i8,
        i16_unsuffixed => i16,
        i32_unsuffixed => i32,
        i64_unsuffixed => i64,
        i128_unsuffixed => i128,
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
        Span(self.0.span)
    }

    /// Configures the span associated for this literal.
    #[stable(feature = "proc_macro_lib2", since = "1.29.0")]
    pub fn set_span(&mut self, span: Span) {
        self.0.span = span.0;
    }

    /// Returns a `Span` that is a subset of `self.span()` containing only the
    /// source bytes in range `range`. Returns `None` if the would-be trimmed
    /// span is outside the bounds of `self`.
    // FIXME(SergioBenitez): check that the byte range starts and ends at a
    // UTF-8 boundary of the source. otherwise, it's likely that a panic will
    // occur elsewhere when the source text is printed.
    // FIXME(SergioBenitez): there is no way for the user to know what
    // `self.span()` actually maps to, so this method can currently only be
    // called blindly. For example, `to_string()` for the character 'c' returns
    // "'\u{63}'"; there is no way for the user to know whether the source text
    // was 'c' or whether it was '\u{63}'.
    #[unstable(feature = "proc_macro_span", issue = "54725")]
    pub fn subspan<R: RangeBounds<usize>>(&self, range: R) -> Option<Span> {
        BridgeMethods::span_subspan(
            self.0.span,
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        )
        .map(Span)
    }

    fn with_symbol_and_suffix<R>(&self, f: impl FnOnce(&str, &str) -> R) -> R {
        self.0.symbol.with(|symbol| match self.0.suffix {
            Some(suffix) => suffix.with(|suffix| f(symbol, suffix)),
            None => f(symbol, ""),
        })
    }

    /// Invokes the callback with a `&[&str]` consisting of each part of the
    /// literal's representation. This is done to allow the `ToString` and
    /// `Display` implementations to borrow references to symbol values, and
    /// both be optimized to reduce overhead.
    fn with_stringify_parts<R>(&self, f: impl FnOnce(&[&str]) -> R) -> R {
        /// Returns a string containing exactly `num` '#' characters.
        /// Uses a 256-character source string literal which is always safe to
        /// index with a `u8` index.
        fn get_hashes_str(num: u8) -> &'static str {
            const HASHES: &str = "\
            ################################################################\
            ################################################################\
            ################################################################\
            ################################################################\
            ";
            const _: () = assert!(HASHES.len() == 256);
            &HASHES[..num as usize]
        }

        self.with_symbol_and_suffix(|symbol, suffix| match self.0.kind {
            bridge::LitKind::Byte => f(&["b'", symbol, "'", suffix]),
            bridge::LitKind::Char => f(&["'", symbol, "'", suffix]),
            bridge::LitKind::Str => f(&["\"", symbol, "\"", suffix]),
            bridge::LitKind::StrRaw(n) => {
                let hashes = get_hashes_str(n);
                f(&["r", hashes, "\"", symbol, "\"", hashes, suffix])
            }
            bridge::LitKind::ByteStr => f(&["b\"", symbol, "\"", suffix]),
            bridge::LitKind::ByteStrRaw(n) => {
                let hashes = get_hashes_str(n);
                f(&["br", hashes, "\"", symbol, "\"", hashes, suffix])
            }
            bridge::LitKind::CStr => f(&["c\"", symbol, "\"", suffix]),
            bridge::LitKind::CStrRaw(n) => {
                let hashes = get_hashes_str(n);
                f(&["cr", hashes, "\"", symbol, "\"", hashes, suffix])
            }

            bridge::LitKind::Integer | bridge::LitKind::Float | bridge::LitKind::ErrWithGuar => {
                f(&[symbol, suffix])
            }
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
                    // Force-inlining here is aggressive but the closure is
                    // called on every char in the string, so it can be hot in
                    // programs with many long strings containing escapes.
                    unescape_str(
                        symbol,
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

                unescape_c_str(symbol, |_span, res| match res {
                    Ok(MixedUnit::Char(c)) => {
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
                } else {
                    buf.push(0);
                    Ok(buf)
                }
            }
            bridge::LitKind::CStrRaw(_) => {
                // Raw strings have no escapes so we can convert the symbol
                // directly to a `Lrc<u8>` after appending the terminating NUL
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

                unesc
// ... (truncated) ...
```

**Entity:** Literal

**State:** FloatConstructionPrecondition

**State invariants:**
- Inputs to f32_* and f64_* constructors must satisfy is_finite()
- On violation, constructors panic at runtime

**Evidence:** line 68-70: f32_unsuffixed panics if !n.is_finite(); line 93-95: f32_suffixed panics if !n.is_finite(); line 113-115: f64_unsuffixed panics if !n.is_finite(); line 138-140: f64_suffixed panics if !n.is_finite()

**Implementation:** Introduce FiniteF32/FiniteF64 newtypes (like NotNaN) constructible via TryFrom<f32/f64>; change constructors to accept FiniteF32/FiniteF64, eliminating panics and making non-finite inputs a type error at call sites when using typed wrappers.

---

### 20. Punct::Construction precondition (allowed character set)

**Location**: `/data/rust/library/proc_macro/src/lib.rs:1-78`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Punct::new only accepts punctuation characters from a fixed allowlist; otherwise it panics. This guarantees the stored byte fits the ASCII punctuation range and the internal u8 cast is sound.

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

**State:** AllowedChar

**State invariants:**
- input ch is in LEGAL_CHARS
- ch as u8 is lossless (ASCII punctuation only)
- constructed Punct always represents a language-permitted punctuation

**Transitions:**
- AllowedChar -> Punct constructed via Punct::new(ch, spacing)
- DisallowedChar -> panic! in Punct::new

**Evidence:** line 28: LEGAL_CHARS contains the allowlist for punctuation; line 32: if !LEGAL_CHARS.contains(&ch) { ... } runtime guard; line 33: panic!("unsupported character `{:?}`", ch) names the precondition; line 36: ch is downcast to u8, relying on the allowlist for correctness

**Implementation:** Introduce an AllowedPunctChar newtype (TryFrom<char>) that validates membership in LEGAL_CHARS. Change Punct::new to accept AllowedPunctChar or implement TryFrom<char> for Punct (returning Result) to eliminate the panic and make the precondition explicit in the type.

---

## Protocol Invariants

### 6. __rust_panic_cleanup::Unreachable stub

**Location**: `/data/rust/library/panic_abort/src/lib.rs:1-94`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: In the panic=abort runtime, cleanup for unwinding must never be invoked because unwinding is not linked; this symbol exists only to satisfy linkage expectations.

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
- Function must never be called in panic=abort
- If called, execution hits unreachable!()

**Evidence:** line 27: extern symbol __rust_panic_cleanup is provided for ABI compatibility; line 28: unreachable!() asserts the function is never reached

**Implementation:** If ABI allowed, give the stub a diverging return type (!) to encode that no valid return exists. Otherwise, keep an internal diverging shim and make the exported symbol uncallable within the crate.

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

