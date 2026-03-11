# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 51
- **Temporal ordering**: 1
- **Resource lifecycle**: 6
- **State machine**: 20
- **Precondition**: 6
- **Protocol**: 18
- **Modules analyzed**: 27

## Temporal Ordering Invariants

### 51. Immediate-after-syscall requirement for last_os_error()

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-473`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `Error::last_os_error()` relies on a thread-/platform-local 'last error' slot (errno/GetLastError). The docs state it must be called immediately after a platform function; otherwise intervening library/syscalls may overwrite/reset the last-error value, making the returned `Error` unrelated. This temporal coupling is purely documentary and cannot be checked by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct SimpleMessage; struct Custom; enum ErrorData; enum ErrorKind; 1 free function(s)

/// [`Write`]: crate::io::Write
/// [`Seek`]: crate::io::Seek
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Error {
    repr: Repr,
}

// ... (other code) ...


/// Common errors constants for use in std
#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    pub(crate) const READ_EXACT_EOF: Self =
        const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer");

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_error!(
        ErrorKind::NotFound,
        "the number of hardware threads is not known for the target platform",
    );

    pub(crate) const UNSUPPORTED_PLATFORM: Self =
        const_error!(ErrorKind::Unsupported, "operation not supported on this platform");

    pub(crate) const WRITE_ALL_EOF: Self =
        const_error!(ErrorKind::WriteZero, "failed to write whole buffer");

    pub(crate) const ZERO_TIMEOUT: Self =
        const_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout");
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<alloc::ffi::NulError> for Error {
    /// Converts a [`alloc::ffi::NulError`] into a [`Error`].
    fn from(_: alloc::ffi::NulError) -> Error {
        const_error!(ErrorKind::InvalidInput, "data provided contains a nul byte")
    }
}

#[stable(feature = "io_error_from_try_reserve", since = "1.78.0")]
impl From<alloc::collections::TryReserveError> for Error {
    /// Converts `TryReserveError` to an error with [`ErrorKind::OutOfMemory`].
    ///
    /// `TryReserveError` won't be available as the error `source()`,
    /// but this may change in the future.
    fn from(_: alloc::collections::TryReserveError) -> Error {
        // ErrorData::Custom allocates, which isn't great for handling OOM errors.
        ErrorKind::OutOfMemory.into()
    }
}

// ... (other code) ...

    Uncategorized,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            // tidy-alphabetical-start
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            FileTooLarge => "file too large",
            HostUnreachable => "host unreachable",
            InProgress => "in progress",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            QuotaExceeded => "quota exceeded",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            ResourceBusy => "resource busy",
            StaleNetworkFileHandle => "stale network file handle",
            StorageFull => "no storage space",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
            Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
            // tidy-alphabetical-end
        }
    }
}

// ... (other code) ...

/// Intended for use for errors not exposed to the user, where allocating onto
/// the heap (for normal construction via Error::new) is too costly.
#[stable(feature = "io_error_from_errorkind", since = "1.14.0")]
impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion creates a new error with a simple representation of error kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// let not_found = ErrorKind::NotFound;
    /// let error = Error::from(not_found);
    /// assert_eq!("entity not found", format!("{error}"));
    /// ```
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error { repr: Repr::new_simple(kind) }
    }
}

impl Error {
    /// Creates a new I/O error from a known kind of error as well as an
    /// arbitrary error payload.
    ///
    /// This function is used to generically create I/O errors which do not
    /// originate from the OS itself. The `error` argument is an arbitrary
    /// payload which will be contained in this [`Error`].
    ///
    /// Note that this function allocates memory on the heap.
    /// If no extra payload is required, use the `From` conversion from
    /// `ErrorKind`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// // errors can be created from strings
    /// let custom_error = Error::new(ErrorKind::Other, "oh no!");
    ///
    /// // errors can also be created from other errors
    /// let custom_error2 = Error::new(ErrorKind::Interrupted, custom_error);
    ///
    /// // creating an error without payload (and without memory allocation)
    /// let eof_error = Error::from(ErrorKind::UnexpectedEof);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline(never)]
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }

    /// Creates a new I/O error from an arbitrary error payload.
    ///
    /// This function is used to generically create I/O errors which do not
    /// originate from the OS itself. It is a shortcut for [`Error::new`]
    /// with [`ErrorKind::Other`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Error;
    ///
    /// // errors can be created from strings
    /// let custom_error = Error::other("oh no!");
    ///
    /// // errors can also be created from other errors
    /// let custom_error2 = Error::other(custom_error);
    /// ```
    #[stable(feature = "io_error_other", since = "1.74.0")]
    pub fn other<E>(error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(ErrorKind::Other, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Error {
        Error { repr: Repr::new_custom(Box::new(Custom { kind, error })) }
    }

    /// Creates a new I/O error from a known kind of error as well as a constant
    /// message.
    ///
    /// This function does not allocate.
    ///
    /// You should not use this directly, and instead use the `const_error!`
    /// macro: `io::const_error!(ErrorKind::Something, "some_message")`.
    ///
    /// This function should maybe change to `from_static_message<const MSG: &'static
    /// str>(kind: ErrorKind)` in the future, when const generics allow that.
    #[inline]
    #[doc(hidden)]
    #[unstable(feature = "io_const_error_internals", issue = "none")]
    pub const fn from_static_message(msg: &'static SimpleMessage) -> Error {
        Self { repr: Repr::new_simple_message(msg) }
    }

    /// Returns an error representing the last OS error which occurred.
    ///
    /// This function reads the value of `errno` for the target platform (e.g.
    /// `GetLastError` on Windows) and will return a corresponding instance of
    /// [`Error`] for the error code.
    ///
    /// This should be called immediately after a call to a platform function,
    /// otherwise the state of the error value is indeterminate. In particular,
    /// other standard library functions may call platform functions that may
    /// (or may not) reset the error value even if they succeed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Error;
    ///
    /// let os_error = Error::last_os_error();
    /// println!("last OS error: {os_error:?}");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[doc(alias = "GetLastError")]
    #[doc(alias = "errno")]
    #[must_use]
    #[inline]
    pub fn last_os_error() -> Error {
        Error::from_raw_os_error(sys::os::errno())
    }

    /// Creates a new instance of an [`Error`] from a particular OS error code.
    ///
    /// # Examples
    ///
    /// On Linux:
    ///
    /// ```
    /// # if cfg!(target_os = "linux") {
    /// use std::io;
    ///
    /// let error = io::Error::from_raw_os_error(22);
    /// assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    /// # }
    /// ```
    ///
    /// On Windows:
    ///
    /// ```
    /// # if cfg!(windows) {
    /// use std::io;
    ///
    /// let error = io::Error::from_raw_os_error(10022);
    /// assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    #[inline]
    pub fn from_raw_os_error(code: RawOsError) -> Error {
        Error { repr: Repr::new_os(code) }
    }

    /// Returns the OS error that this error represents (if any).
    ///
    /// If this [`Error`] was constructed via [`last_os_error`] or
    /// [`from_raw_os_error`], then this function will return [`Some`], otherwise
    /// it will return [`None`].
    ///
    /// [`last_os_error`]: Error::last_os_error
    /// [`from_raw_os_error`]: Error::from_raw_os_error
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_os_error(err: &Error) {
    ///     if let Some(raw_os_err) = err.raw_os_error() {
    ///         println!("raw OS error: {raw_os_err:?}");
    ///     } else {
    ///         println!("Not an OS error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "raw OS error: ...".
    ///     print_os_error(&Error::last_os_error());
    ///     // Will print "Not an OS error".
    ///     print_os_error(&Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    #[inline]
    pub fn raw_os_error(&self) -> Option<RawOsError> {
        match self.repr.data() {
            ErrorData::Os(i) => Some(i),
            ErrorData::Custom(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
        }
    }

    /// Returns a reference to the inner error wrapped by this error (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] then this function will
    /// return [`Some`], otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_error(err: &Error) {
    ///     if let Some(inner_err) = err.get_ref() {
    ///         println!("Inner error: {inner_err:?}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(&Error::last_os_error());
    ///     // Will print "Inner error: ...".
    ///     print_error(&Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use]
    #[inline]
    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(&*c.error),
        }
    }

    /// Returns a mutable reference to the inner error wrapped by this error
    /// (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] then this function will
    /// return [`Some`], otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    /// use std::{error, fmt};
    /// use std::fmt::Display;
    ///
    /// #[derive(Debug)]
    /// struct MyError {
    ///     v: String,
    /// }
    ///
    /// impl MyError {
    ///     fn new() -> MyError {
    ///         MyError {
    ///             v: "oh no!".to_string()
    ///         }
    ///     }
    ///
    ///     fn change_message(&mut self, new_message: &str) {
    ///         self.v = new_message.to_string();
    ///     }
    /// }
    ///
    /// impl error::Error for MyError {}
    ///
    /// impl Display for MyError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "MyError: {}", self.v)
    ///     }
    /// }
    ///
    /// fn change_error(mut err: Error) -> Error {
    ///     if let Some(inner_err) = err.get_mut() {
    ///         inner_err.downcast_mut::<MyError>().unwrap().change_message("I've been changed!");
    ///     }
    ///     err
    /// }
    ///
    /// fn print_error(err: &Error) {
    ///     if let Some(inner_err) = err.get_ref() {
    ///         println!("Inner error: {inner_err}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(&change_error(Error::last_os_error()));
    ///     // Will print "Inner error: ...".
    ///     print_error(&change_error(Error::new(ErrorKind::Other, MyError::new())));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data_mut() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(&mut *c.error),
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] or [`other`],
    /// then this function will return [`Some`],
    /// otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_error(err: Error) {
    ///     if let Some(inner_err) = err.into_inner() {
    ///         println!("Inner error: {inner_err}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(Error::last_os_error());
    ///     // Will print "Inner error: ...".
    ///     print_error(Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
        match self.repr.into_data() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(c.error),
   
// ... (truncated) ...
```

**Entity:** Error::last_os_error

**States:** ErrnoFresh, ErrnoClobbered/Indeterminate

**Transitions:**
- ErrnoFresh -> ErrnoClobbered/Indeterminate via any intervening platform call (including stdlib calls) before last_os_error()
- ErrnoFresh -> (captured) Os Error via last_os_error()

**Evidence:** doc comment on last_os_error(): 'This should be called immediately after a call to a platform function, otherwise the state of the error value is indeterminate.'; doc comment: 'other standard library functions may call platform functions that may (or may not) reset the error value even if they succeed.'; implementation: last_os_error() reads `sys::os::errno()` at call time

**Implementation:** Wrap FFI/platform calls to return a token/capability representing a captured last-error value (e.g., `struct LastOsErrorToken(RawOsError)` produced alongside the syscall result), or provide unsafe/ffi helper APIs that return `Result<T, OsError>` directly. Then construct `Error` from the captured code, eliminating the 'must call immediately' footgun.

---

## Resource Lifecycle Invariants

### 15. Repr ownership protocol for Custom payload (exactly-once Box::from_raw / drop)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-408`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: When the tag is `TAG_CUSTOM`, `Repr` logically owns a `Box<Custom>` whose raw pointer is stored in tagged form. That ownership must be consumed exactly once: either by `Drop` (reconstruct `Box::from_raw` and drop it) or by `into_data(self)` (reconstruct and return the `Box`). This is maintained by a runtime protocol (using `ManuallyDrop` to prevent double-drop) and by the implicit guarantee that no other code will call `Box::from_raw` on the same stored pointer. The type system does not express the 'owns custom' vs 'does not own custom' distinction; all variants share the same `Repr` type and `decode_repr` is generic over how the custom pointer is turned into a value.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Repr, impl Send for Repr (0 methods), impl Sync for Repr (0 methods), impl Repr (8 methods), impl Drop for Repr (1 methods)

//! This is a densely packed error representation which is used on targets with
//! 64-bit pointers.
//!
//! (Note that `bitpacked` vs `unpacked` here has no relationship to
//! `#[repr(packed)]`, it just refers to attempting to use any available bits in
//! a more clever manner than `rustc`'s default layout algorithm would).
//!
//! Conceptually, it stores the same data as the "unpacked" equivalent we use on
//! other targets. Specifically, you can imagine it as an optimized version of
//! the following enum (which is roughly equivalent to what's stored by
//! `repr_unpacked::Repr`, e.g. `super::ErrorData<Box<Custom>>`):
//!
//! ```ignore (exposition-only)
//! enum ErrorData {
//!    Os(i32),
//!    Simple(ErrorKind),
//!    SimpleMessage(&'static SimpleMessage),
//!    Custom(Box<Custom>),
//! }
//! ```
//!
//! However, it packs this data into a 64bit non-zero value.
//!
//! This optimization not only allows `io::Error` to occupy a single pointer,
//! but improves `io::Result` as well, especially for situations like
//! `io::Result<()>` (which is now 64 bits) or `io::Result<u64>` (which is now
//! 128 bits), which are quite common.
//!
//! # Layout
//! Tagged values are 64 bits, with the 2 least significant bits used for the
//! tag. This means there are 4 "variants":
//!
//! - **Tag 0b00**: The first variant is equivalent to
//!   `ErrorData::SimpleMessage`, and holds a `&'static SimpleMessage` directly.
//!
//!   `SimpleMessage` has an alignment >= 4 (which is requested with
//!   `#[repr(align)]` and checked statically at the bottom of this file), which
//!   means every `&'static SimpleMessage` should have the both tag bits as 0,
//!   meaning its tagged and untagged representation are equivalent.
//!
//!   This means we can skip tagging it, which is necessary as this variant can
//!   be constructed from a `const fn`, which probably cannot tag pointers (or
//!   at least it would be difficult).
//!
//! - **Tag 0b01**: The other pointer variant holds the data for
//!   `ErrorData::Custom` and the remaining 62 bits are used to store a
//!   `Box<Custom>`. `Custom` also has alignment >= 4, so the bottom two bits
//!   are free to use for the tag.
//!
//!   The only important thing to note is that `ptr::wrapping_add` and
//!   `ptr::wrapping_sub` are used to tag the pointer, rather than bitwise
//!   operations. This should preserve the pointer's provenance, which would
//!   otherwise be lost.
//!
//! - **Tag 0b10**: Holds the data for `ErrorData::Os(i32)`. We store the `i32`
//!   in the pointer's most significant 32 bits, and don't use the bits `2..32`
//!   for anything. Using the top 32 bits is just to let us easily recover the
//!   `i32` code with the correct sign.
//!
//! - **Tag 0b11**: Holds the data for `ErrorData::Simple(ErrorKind)`. This
//!   stores the `ErrorKind` in the top 32 bits as well, although it doesn't
//!   occupy nearly that many. Most of the bits are unused here, but it's not
//!   like we need them for anything else yet.
//!
//! # Use of `NonNull<()>`
//!
//! Everything is stored in a `NonNull<()>`, which is odd, but actually serves a
//! purpose.
//!
//! Conceptually you might think of this more like:
//!
//! ```ignore (exposition-only)
//! union Repr {
//!     // holds integer (Simple/Os) variants, and
//!     // provides access to the tag bits.
//!     bits: NonZero<u64>,
//!     // Tag is 0, so this is stored untagged.
//!     msg: &'static SimpleMessage,
//!     // Tagged (offset) `Box<Custom>` pointer.
//!     tagged_custom: NonNull<()>,
//! }
//! ```
//!
//! But there are a few problems with this:
//!
//! 1. Union access is equivalent to a transmute, so this representation would
//!    require we transmute between integers and pointers in at least one
//!    direction, which may be UB (and even if not, it is likely harder for a
//!    compiler to reason about than explicit ptr->int operations).
//!
//! 2. Even if all fields of a union have a niche, the union itself doesn't,
//!    although this may change in the future. This would make things like
//!    `io::Result<()>` and `io::Result<usize>` larger, which defeats part of
//!    the motivation of this bitpacking.
//!
//! Storing everything in a `NonZero<usize>` (or some other integer) would be a
//! bit more traditional for pointer tagging, but it would lose provenance
//! information, couldn't be constructed from a `const fn`, and would probably
//! run into other issues as well.
//!
//! The `NonNull<()>` seems like the only alternative, even if it's fairly odd
//! to use a pointer type to store something that may hold an integer, some of
//! the time.

use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

use super::{Custom, ErrorData, ErrorKind, RawOsError, SimpleMessage};

// The 2 least-significant bits are used as tag.
const TAG_MASK: usize = 0b11;
const TAG_SIMPLE_MESSAGE: usize = 0b00;
const TAG_CUSTOM: usize = 0b01;
const TAG_OS: usize = 0b10;
const TAG_SIMPLE: usize = 0b11;

/// The internal representation.
///
/// See the module docs for more, this is just a way to hack in a check that we
/// indeed are not unwind-safe.
///
/// ```compile_fail,E0277
/// fn is_unwind_safe<T: core::panic::UnwindSafe>() {}
/// is_unwind_safe::<std::io::Error>();
/// ```
#[repr(transparent)]
#[rustc_insignificant_dtor]
pub(super) struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>);

// All the types `Repr` stores internally are Send + Sync, and so is it.
unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}

impl Repr {
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        match dat {
            ErrorData::Os(code) => Self::new_os(code),
            ErrorData::Simple(kind) => Self::new_simple(kind),
            ErrorData::SimpleMessage(simple_message) => Self::new_simple_message(simple_message),
            ErrorData::Custom(b) => Self::new_custom(b),
        }
    }

    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        let p = Box::into_raw(b).cast::<u8>();
        // Should only be possible if an allocator handed out a pointer with
        // wrong alignment.
        debug_assert_eq!(p.addr() & TAG_MASK, 0);
        // Note: We know `TAG_CUSTOM <= size_of::<Custom>()` (static_assert at
        // end of file), and both the start and end of the expression must be
        // valid without address space wraparound due to `Box`'s semantics.
        //
        // This means it would be correct to implement this using `ptr::add`
        // (rather than `ptr::wrapping_add`), but it's unclear this would give
        // any benefit, so we just use `wrapping_add` instead.
        let tagged = p.wrapping_add(TAG_CUSTOM).cast::<()>();
        // Safety: `TAG_CUSTOM + p` is the same as `TAG_CUSTOM | p`,
        // because `p`'s alignment means it isn't allowed to have any of the
        // `TAG_BITS` set (you can verify that addition and bitwise-or are the
        // same when the operands have no bits in common using a truth table).
        //
        // Then, `TAG_CUSTOM | p` is not zero, as that would require
        // `TAG_CUSTOM` and `p` both be zero, and neither is (as `p` came from a
        // box, and `TAG_CUSTOM` just... isn't zero -- it's `0b01`). Therefore,
        // `TAG_CUSTOM + p` isn't zero and so `tagged` can't be, and the
        // `new_unchecked` is safe.
        let res = Self(unsafe { NonNull::new_unchecked(tagged) }, PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(matches!(res.data(), ErrorData::Custom(_)), "repr(custom) encoding failed");
        res
    }

    #[inline]
    pub(super) fn new_os(code: RawOsError) -> Self {
        let utagged = ((code as usize) << 32) | TAG_OS;
        // Safety: `TAG_OS` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Os(c) if c == code),
            "repr(os) encoding failed for {code}"
        );
        res
    }

    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        let utagged = ((kind as usize) << 32) | TAG_SIMPLE;
        // Safety: `TAG_SIMPLE` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Simple(k) if k == kind),
            "repr(simple) encoding failed {:?}",
            kind,
        );
        res
    }

    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        // Safety: References are never null.
        Self(unsafe { NonNull::new_unchecked(m as *const _ as *mut ()) }, PhantomData)
    }

    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &*c) }
    }

    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &mut *c) }
    }

    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        let this = core::mem::ManuallyDrop::new(self);
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we prevent double-drop using `ManuallyDrop`.
        unsafe { decode_repr(this.0, |p| Box::from_raw(p)) }
    }
}

impl Drop for Repr {
    #[inline]
    fn drop(&mut self) {
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we're being dropped.
        unsafe {
            let _ = decode_repr(self.0, |p| Box::<Custom>::from_raw(p));
        }
    }
}

// Shared helper to decode a `Repr`'s internal pointer into an ErrorData.
//
// Safety: `ptr`'s bits should be encoded as described in the document at the
// top (it should `some_repr.0`)
#[inline]
unsafe fn decode_repr<C, F>(ptr: NonNull<()>, make_custom: F) -> ErrorData<C>
where
    F: FnOnce(*mut Custom) -> C,
{
    let bits = ptr.as_ptr().addr();
    match bits & TAG_MASK {
        TAG_OS => {
            let code = ((bits as i64) >> 32) as RawOsError;
            ErrorData::Os(code)
        }
        TAG_SIMPLE => {
            let kind_bits = (bits >> 32) as u32;
            let kind = kind_from_prim(kind_bits).unwrap_or_else(|| {
                debug_assert!(false, "Invalid io::error::Repr bits: `Repr({:#018x})`", bits);
                // This means the `ptr` passed in was not valid, which violates
                // the unsafe contract of `decode_repr`.
                //
                // Using this rather than unwrap meaningfully improves the code
                // for callers which only care about one variant (usually
                // `Custom`)
                unsafe { core::hint::unreachable_unchecked() };
            });
            ErrorData::Simple(kind)
        }
        TAG_SIMPLE_MESSAGE => {
            // SAFETY: per tag
            unsafe { ErrorData::SimpleMessage(&*ptr.cast::<SimpleMessage>().as_ptr()) }
        }
        TAG_CUSTOM => {
            // It would be correct for us to use `ptr::byte_sub` here (see the
            // comment above the `wrapping_add` call in `new_custom` for why),
            // but it isn't clear that it makes a difference, so we don't.
            let custom = ptr.as_ptr().wrapping_byte_sub(TAG_CUSTOM).cast::<Custom>();
            ErrorData::Custom(make_custom(custom))
        }
        _ => {
            // Can't happen, and compiler can tell
            unreachable!();
        }
    }
}

// This compiles to the same code as the check+transmute, but doesn't require
// unsafe, or to hard-code max ErrorKind or its size in a way the compiler
// couldn't verify.
#[inline]
fn kind_from_prim(ek: u32) -> Option<ErrorKind> {
    macro_rules! from_prim {
        ($prim:expr => $Enum:ident { $($Variant:ident),* $(,)? }) => {{
            // Force a compile error if the list gets out of date.
            const _: fn(e: $Enum) = |e: $Enum| match e {
                $($Enum::$Variant => ()),*
            };
            match $prim {
                $(v if v == ($Enum::$Variant as _) => Some($Enum::$Variant),)*
                _ => None,
            }
        }}
    }
    from_prim!(ek => ErrorKind {
        NotFound,
        PermissionDenied,
        ConnectionRefused,
        ConnectionReset,
        HostUnreachable,
        NetworkUnreachable,
        ConnectionAborted,
        NotConnected,
        AddrInUse,
        AddrNotAvailable,
        NetworkDown,
        BrokenPipe,
        AlreadyExists,
        WouldBlock,
        NotADirectory,
        IsADirectory,
        DirectoryNotEmpty,
        ReadOnlyFilesystem,
        FilesystemLoop,
        StaleNetworkFileHandle,
        InvalidInput,
        InvalidData,
        TimedOut,
        WriteZero,
        StorageFull,
        NotSeekable,
        QuotaExceeded,
        FileTooLarge,
        ResourceBusy,
        ExecutableFileBusy,
        Deadlock,
        CrossesDevices,
        TooManyLinks,
        InvalidFilename,
        ArgumentListTooLong,
        Interrupted,
        Other,
        UnexpectedEof,
        Unsupported,
        OutOfMemory,
        InProgress,
        Uncategorized,
    })
}

// Some static checking to alert us if a change breaks any of the assumptions
// that our encoding relies on for correctness and soundness. (Some of these are
// a bit overly thorough/cautious, admittedly)
//
// If any of these are hit on a platform that std supports, we should likely
// just use `repr_unpacked.rs` there instead (unless the fix is easy).
macro_rules! static_assert {
    ($condition:expr) => {
        const _: () = assert!($condition);
    };
    (@usize_eq: $lhs:expr, $rhs:expr) => {
        const _: [(); $lhs] = [(); $rhs];
    };
}

// The bitpacking we use requires pointers be exactly 64 bits.
static_assert!(@usize_eq: size_of::<NonNull<()>>(), 8);

// We also require pointers and usize be the same size.
static_assert!(@usize_eq: size_of::<NonNull<()>>(), size_of::<usize>());

// `Custom` and `SimpleMessage` need to be thin pointers.
static_assert!(@usize_eq: size_of::<&'static SimpleMessage>(), 8);
static_assert!(@usize_eq: size_of::<Box<Custom>>(), 8);

static_assert!((TAG_MASK + 1).is_power_of_two());
// And they must have sufficient alignment.
static_assert!(align_of::<SimpleMessage>() >= TAG_MASK + 1);
static_assert!(align_of::<Custom>() >= TAG_MASK + 1);

static_assert!(@usize_eq: TAG_MASK & TAG_SIMPLE_MESSAGE, TAG_SIMPLE_MESSAGE);
static_assert!(@usize_eq: TAG_MASK & TAG_CUSTOM, TAG_CUSTOM);
static_assert!(@usize_eq: TAG_MASK & TAG_OS, TAG_OS);
static_assert!(@usize_eq: TAG_MASK & TAG_SIMPLE, TAG_SIMPLE);

// This is obviously true (`TAG_CUSTOM` is `0b01`), but in `Repr::new_custom` we
// offset a pointer by this value, and expect it to both be within the same
// object, and to not wrap around the address space. See the comment in that
// function for further details.
//
// Actually, at the moment we use `ptr::wrapping_add`, not `ptr::add`, so this
// check isn't needed for that one, although the assertion that we don't
// actually wrap around in that wrapping_add does simplify the safety reasoning
// elsewhere considerably.
static_assert!(size_of::<Custom>() >= TAG_CUSTOM);

// These two store a payload which is allowed to be zero, so they must be
// non-zero to
// ... (truncated) ...
```

**Entity:** Repr

**States:** Does not own Custom (SimpleMessage/Os/Simple), Owns Custom box (Custom), Custom ownership moved out (after into_data)

**Transitions:**
- Owns Custom box -> Custom ownership moved out via Repr::into_data(self) (uses ManuallyDrop + Box::from_raw)
- Owns Custom box -> dropped via Drop::drop (uses Box::from_raw)
- Does not own Custom -> dropped via Drop::drop (decode produces non-Custom and does not call Box::from_raw)

**Evidence:** Repr::into_data(self): `let this = core::mem::ManuallyDrop::new(self);` and comment 'prevent double-drop using ManuallyDrop'; Repr::into_data(self): `decode_repr(this.0, |p| Box::from_raw(p))` reconstructs ownership from raw pointer; impl Drop for Repr: `decode_repr(self.0, |p| Box::<Custom>::from_raw(p));` reconstructs and drops owned box when variant is Custom; decode_repr(): TAG_CUSTOM arm produces a `*mut Custom` derived from stored tagged pointer, which is only valid to pass to `Box::from_raw` if it originally came from `Box::into_raw` (as done in Repr::new_custom)

**Implementation:** Split representation into two internal types: `Repr` for non-owning variants and `ReprCustom` for the owning-custom variant (or `Repr<S>` with `S = OwnsCustom | NoCustom`). Only `Repr<OwnsCustom>` would implement `Drop` logic that calls `Box::from_raw`, and `into_data(self)` would only exist on `Repr<OwnsCustom>` returning the `Box`. Conversions from `ErrorData<Box<Custom>>` would yield the appropriate typestate based on the variant, eliminating the need for a generic `decode_repr` closure to control ownership.

---

### 49. Repr ownership/lifetime protocol for Custom (unique owner, exactly-once drop unless into_data)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-123`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: When Repr stores the Custom variant, it implicitly owns a heap allocation created by Box::into_raw and must free it exactly once. The code enforces this via Drop (reconstructing Box from raw) and via into_data() using ManuallyDrop to avoid double-free when transferring ownership out. This 'exactly-once deallocation unless moved out' protocol is not reflected in the type system as distinct states; correctness depends on callers following the intended move semantics and on decode_repr returning the correct variant/pointer type.

**Evidence**:

```rust
// Note: Other parts of this module contain: 2 free function(s)

/// ```
#[repr(transparent)]
#[rustc_insignificant_dtor]
pub(super) struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>);

// All the types `Repr` stores internally are Send + Sync, and so is it.
unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}

impl Repr {
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        match dat {
            ErrorData::Os(code) => Self::new_os(code),
            ErrorData::Simple(kind) => Self::new_simple(kind),
            ErrorData::SimpleMessage(simple_message) => Self::new_simple_message(simple_message),
            ErrorData::Custom(b) => Self::new_custom(b),
        }
    }

    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        let p = Box::into_raw(b).cast::<u8>();
        // Should only be possible if an allocator handed out a pointer with
        // wrong alignment.
        debug_assert_eq!(p.addr() & TAG_MASK, 0);
        // Note: We know `TAG_CUSTOM <= size_of::<Custom>()` (static_assert at
        // end of file), and both the start and end of the expression must be
        // valid without address space wraparound due to `Box`'s semantics.
        //
        // This means it would be correct to implement this using `ptr::add`
        // (rather than `ptr::wrapping_add`), but it's unclear this would give
        // any benefit, so we just use `wrapping_add` instead.
        let tagged = p.wrapping_add(TAG_CUSTOM).cast::<()>();
        // Safety: `TAG_CUSTOM + p` is the same as `TAG_CUSTOM | p`,
        // because `p`'s alignment means it isn't allowed to have any of the
        // `TAG_BITS` set (you can verify that addition and bitwise-or are the
        // same when the operands have no bits in common using a truth table).
        //
        // Then, `TAG_CUSTOM | p` is not zero, as that would require
        // `TAG_CUSTOM` and `p` both be zero, and neither is (as `p` came from a
        // box, and `TAG_CUSTOM` just... isn't zero -- it's `0b01`). Therefore,
        // `TAG_CUSTOM + p` isn't zero and so `tagged` can't be, and the
        // `new_unchecked` is safe.
        let res = Self(unsafe { NonNull::new_unchecked(tagged) }, PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(matches!(res.data(), ErrorData::Custom(_)), "repr(custom) encoding failed");
        res
    }

    #[inline]
    pub(super) fn new_os(code: RawOsError) -> Self {
        let utagged = ((code as usize) << 32) | TAG_OS;
        // Safety: `TAG_OS` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Os(c) if c == code),
            "repr(os) encoding failed for {code}"
        );
        res
    }

    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        let utagged = ((kind as usize) << 32) | TAG_SIMPLE;
        // Safety: `TAG_SIMPLE` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Simple(k) if k == kind),
            "repr(simple) encoding failed {:?}",
            kind,
        );
        res
    }

    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        // Safety: References are never null.
        Self(unsafe { NonNull::new_unchecked(m as *const _ as *mut ()) }, PhantomData)
    }

    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &*c) }
    }

    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &mut *c) }
    }

    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        let this = core::mem::ManuallyDrop::new(self);
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we prevent double-drop using `ManuallyDrop`.
        unsafe { decode_repr(this.0, |p| Box::from_raw(p)) }
    }
}

impl Drop for Repr {
    #[inline]
    fn drop(&mut self) {
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we're being dropped.
        unsafe {
            let _ = decode_repr(self.0, |p| Box::<Custom>::from_raw(p));
        }
    }
}

```

**Entity:** Repr

**States:** Owning(Custom allocated via Box), Borrowing view via data()/data_mut(), Moved-out ownership via into_data() (Repr must not drop)

**Transitions:**
- Owning(Custom) -> Moved-out ownership via into_data() (uses ManuallyDrop to suppress Drop)
- Owning(Custom) -> freed via Drop::drop()
- Owning(Custom) -> Borrowing view via data() / data_mut() (temporary, non-owning)

**Evidence:** new_custom(): Box::into_raw(b) (turns Box into raw pointer; ownership must be recovered exactly once); Drop::drop(): `decode_repr(self.0, |p| Box::<Custom>::from_raw(p));` with comment 'safe because we're being dropped' (exactly-once free on drop); into_data(self): `let this = ManuallyDrop::new(self);` and comment 'prevent double-drop' then `decode_repr(this.0, |p| Box::from_raw(p))` (ownership transfer out + suppress destructor); data()/data_mut(): decode_repr produces `&Custom` / `&mut Custom` (borrow-only access assumes allocation still owned by Repr and not already moved out)

**Implementation:** Split into distinct wrapper types for the owning vs moved-out phases, e.g. `struct ReprOwned(Repr);` with `fn into_data(self) -> ErrorData<Box<Custom>>` consuming ReprOwned, and prevent calling Drop path after move by making `Repr` itself non-`Drop` and keeping deallocation in `ReprOwned` only. Alternatively, encode the 'custom owns allocation' case as `ReprCustom(NonNull<Custom>)` newtype with its own Drop and conversion methods, and keep non-owning variants separate so `Box::from_raw` is only reachable from the owning type.

---

### 41. Stdin locking protocol (Unlocked handle -> Locked guard)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-233`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Stdin operations are implicitly a two-phase protocol: you either operate through a short-lived lock acquired from the global mutex, or you hold a lock guard (StdinLock) and then perform buffered reads. The code relies on repeatedly calling lock() inside each Read method (and in read_line/lines) to ensure mutual exclusion. This protocol (that reads should happen only while the mutex is held, and that holding the lock is the 'active' state) is not represented in Stdin's type; Stdin exposes Read directly and re-locks internally on every call, which also means multi-call sequences that expect a single consistent lock/buffer state are only achievable by manually using StdinLock.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Stdin")]
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

// ... (other code) ...

    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`]. In particular:
    /// * Previous content of the buffer will be preserved. To avoid appending
    ///   to the buffer, you need to [`clear`] it first.
    /// * The trailing newline character, if any, is included in the buffer.
    ///
    /// [`clear`]: String::clear
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     }
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly,
    ///   in which case it will wait for the Enter key to be pressed before
    ///   continuing
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_confusables("get_line")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "stdin_forwarders", since = "1.62.0")]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf_exact(cursor)
    }
}

#[stable(feature = "read_shared_stdin", since = "1.78.0")]
impl Read for &Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf_exact(cursor)
    }
}

// only used by platform-dependent io::copy specializations, i.e. unused on some platforms
#[cfg(any(target_os = "linux", target_os = "android"))]
impl StdinLock<'_> {
    pub(crate) fn as_mut_buf(&mut self) -> &mut BufReader<impl Read> {
        &mut self.inner
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.inner.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.inner.read_buf_exact(cursor)
    }
}

impl SpecReadByte for StdinLock<'_> {
    #[inline]
    fn spec_read_byte(&mut self) -> Option<io::Result<u8>> {
        BufReader::spec_read_byte(&mut *self.inner)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for StdinLock<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }
}

```

**Entity:** Stdin

**States:** Unlocked (not holding stdin mutex), Locked (holding stdin mutex via StdinLock)

**Transitions:**
- Unlocked -> Locked via Stdin::lock()
- Locked -> Unlocked when StdinLock is dropped (scope end)

**Evidence:** field `inner: &'static Mutex<BufReader<StdinRaw>>` encodes that all access is mediated by a global mutex; method `Stdin::lock(&self) -> StdinLock<'static>` is the explicit acquire step: `self.inner.lock().unwrap_or_else(|e| e.into_inner())`; comment in lock(): "The lock is released when the returned lock goes out of scope" and "depends on ... underlying `Mutex` is static"; method `Stdin::read_line(&self, ...)` immediately does `self.lock().read_line(buf)` (operation requires lock); method `Stdin::lines(self)` does `self.lock().lines()` (iterator tied to locked state); impl `Read for Stdin` methods all do `self.lock().read_*...` (re-acquire lock per operation)

**Implementation:** Make the ability to perform buffered stdin reads require an explicit capability/guard type (the lock), and avoid offering `Read` directly on `Stdin` (or make it only provide unbuffered/atomic operations). For example, expose reading APIs primarily on `StdinLock<'a>` and treat `Stdin` as a factory for acquiring the capability; callers who need consistent buffering across multiple calls are forced (at compile time) to hold `StdinLock`.

---

### 11. PipeWriter resource lifecycle (Open/Valid -> Closed/Invalid via drop/into_inner)

**Location**: `/tmp/io_test_crate/src/io/pipe.rs:1-108`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: PipeWriter is a thin owning wrapper around an OS pipe write-end (AnonPipe). It is only valid to perform io::Write operations while it still owns a live AnonPipe. The API permits consuming the writer (into_inner) or dropping it, which closes the underlying write end; after that, further writes are impossible (by move semantics) but the protocol-level effect (closing the write end, potentially signaling EOF to readers) is not represented as an explicit type/state transition. Additionally, because the inner handle can be extracted, the module relies on convention that the extracted AnonPipe is used consistently as the write end of the same pipe.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct PipeReader, 1 free function(s), impl FromInner < AnonPipe > for PipeReader (1 methods), impl IntoInner < AnonPipe > for PipeReader (1 methods), impl PipeReader (1 methods), impl io :: Read for & PipeReader (5 methods), impl io :: Read for PipeReader (5 methods)

/// Write end of an anonymous pipe.
#[stable(feature = "anonymous_pipe", since = "1.87.0")]
#[derive(Debug)]
pub struct PipeWriter(pub(crate) AnonPipe);


// ... (other code) ...

    }
}

impl FromInner<AnonPipe> for PipeWriter {
    fn from_inner(inner: AnonPipe) -> Self {
        Self(inner)
    }
}

impl IntoInner<AnonPipe> for PipeWriter {
    fn into_inner(self) -> AnonPipe {
        self.0
    }
}

// ... (other code) ...

    }
}

impl PipeWriter {
    /// Creates a new [`PipeWriter`] instance that shares the same underlying file description.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(miri)] fn main() {}
    /// # #[cfg(not(miri))]
    /// # fn main() -> std::io::Result<()> {
    /// use std::process::Command;
    /// use std::io::{pipe, Read};
    /// let (mut reader, writer) = pipe()?;
    ///
    /// // Spawn a process that writes to stdout and stderr.
    /// let mut peer = Command::new("bash")
    ///     .args([
    ///         "-c",
    ///         "echo -n foo\n\
    ///          echo -n bar >&2"
    ///     ])
    ///     .stdout(writer.try_clone()?)
    ///     .stderr(writer)
    ///     .spawn()?;
    ///
    /// // Read and check the result.
    /// let mut msg = String::new();
    /// reader.read_to_string(&mut msg)?;
    /// assert_eq!(&msg, "foobar");
    ///
    /// peer.wait()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "anonymous_pipe", since = "1.87.0")]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }
}

// ... (other code) ...

}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Write for &PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

```

**Entity:** PipeWriter

**States:** Open (owns a live AnonPipe write end), Closed/Invalid (write end has been moved out or dropped)

**Transitions:**
- Open -> Closed/Invalid via IntoInner<AnonPipe>::into_inner(self)
- Open -> Closed/Invalid via Drop of PipeWriter (implicit)

**Evidence:** field: `pub struct PipeWriter(pub(crate) AnonPipe);` indicates PipeWriter's validity is tied to owning an AnonPipe resource; method: `impl IntoInner<AnonPipe> for PipeWriter { fn into_inner(self) -> AnonPipe { self.0 } }` consumes PipeWriter and moves out the resource (state transition by consumption); impl: `impl io::Write for PipeWriter` and `impl io::Write for &PipeWriter` delegate writes to `self.0.write(...)`, implying operations require a live underlying handle

**Implementation:** Keep PipeWriter as the RAII owner but make the lifecycle transition explicit by providing a `close(self) -> io::Result<()>` (or `shutdown(self)`) that performs a defined close operation and returns a `ClosedPipeWriter` ZST/token. Alternatively, avoid exposing `IntoInner` publicly (or gate it behind a capability) so code cannot accidentally bypass the intended pipe-end protocol by extracting the raw handle.

---

### 16. StdoutLock lock-holding protocol (Locked -> Unlocked) + buffered writer flush expectations

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-8`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: StdoutLock represents a period during which the global stdout is locked via a re-entrant lock guard. While in the Locked state, writes go through a RefCell<LineWriter<StdoutRaw>> (interior mutability + buffering). When the StdoutLock value is dropped (or otherwise not kept), it immediately transitions to Unlocked, releasing the lock. The API relies on the user to hold onto the returned value for as long as mutual exclusion is desired; additionally, because the inner writer is a LineWriter, there is an implicit expectation that users may need to call Write::flush at certain boundaries to force buffered output. These usage constraints are conveyed via attributes/comments rather than being modeled as a distinct capability type separating 'locked stdout access' from 'unlocked'.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

/// [`flush`]: Write::flush
#[must_use = "if unused stdout will immediately unlock"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdoutLock<'a> {
    inner: ReentrantLockGuard<'a, RefCell<LineWriter<StdoutRaw>>>,
}

```

**Entity:** StdoutLock<'a>

**States:** Locked (guard held), Unlocked (guard dropped)

**Transitions:**
- Locked -> Unlocked via drop (scope end / value unused)

**Evidence:** attribute on StdoutLock: #[must_use = "if unused stdout will immediately unlock"] — indicates a runtime-relevant state transition if the value is not held; field inner: ReentrantLockGuard<'a, RefCell<LineWriter<StdoutRaw>>> — encodes 'lock is held' + interior mutability + buffering behind LineWriter; doc comment: "/// [`flush`]: Write::flush" — points at a behavioral requirement around flushing buffered output

**Implementation:** Return a distinct capability token representing 'stdout is locked' (e.g., StdoutLocked<'a>) that must be threaded to functions that require exclusive stdout access. Alternatively, make APIs that require the lock accept &mut StdoutLock<'_> (or a newtype over the guard) to force callers to keep the guard alive across the critical section. If flush-at-boundary is important, provide explicit methods (e.g., finish(self) -> Result<Unlocked, _>) that consume the lock and flush before unlocking.

---

### 37. Flush-before-extract lifecycle (Buffered -> Extracted inner)

**Location**: `/tmp/io_test_crate/src/io/buffered/linewriter.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: LineWriter has an internal buffer (via BufWriter) that must be flushed before the underlying writer can be safely extracted. The API enforces this at runtime: into_inner(self) attempts to flush and can fail, returning an IntoInnerError containing the original LineWriter so the caller can retry or recover. This is a state transition ('buffered writer' -> 'raw writer') that is not represented in the type system; callers must handle the error path to avoid losing buffered data.

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct LineWriter<W: ?Sized + Write> {
    inner: BufWriter<W>,
}

impl<W: Write> LineWriter<W> {
    /// Creates a new `LineWriter`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::new(file);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: W) -> LineWriter<W> {
        // Lines typically aren't that long, don't use a giant buffer
        LineWriter::with_capacity(1024, inner)
    }

    /// Creates a new `LineWriter` with at least the specified capacity for the
    /// internal buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::with_capacity(100, file);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: W) -> LineWriter<W> {
        LineWriter { inner: BufWriter::with_capacity(capacity, inner) }
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// Caution must be taken when calling methods on the mutable reference
    /// returned as extra writes could corrupt the output stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let mut file = LineWriter::new(file);
    ///
    ///     // we can use reference just like file
    ///     let reference = file.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    /// Unwraps this `LineWriter`, returning the underlying writer.
    ///
    /// The internal buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if an error occurs while flushing the buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///
    ///     let writer: LineWriter<File> = LineWriter::new(file);
    ///
    ///     let file: File = writer.into_inner()?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.inner.into_inner().map_err(|err| err.new_wrapped(|inner| LineWriter { inner }))
    }
}

impl<W: ?Sized + Write> LineWriter<W> {
    /// Gets a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::new(file);
    ///
    ///     let reference = file.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write_vectored(bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_fmt(fmt)
    }
}

```

**Entity:** LineWriter<W>

**States:** Buffered, ExtractingInnerSucceeded, ExtractingInnerFailedWithBufferedStateRetained

**Transitions:**
- Buffered -> ExtractingInnerSucceeded via into_inner() returning Ok(W)
- Buffered -> ExtractingInnerFailedWithBufferedStateRetained via into_inner() returning Err(IntoInnerError<LineWriter<W>>)

**Evidence:** method doc on into_inner(): "The internal buffer is written out before returning the writer."; method signature: pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> (error retains the LineWriter state); method body: self.inner.into_inner().map_err(|err| err.new_wrapped(|inner| LineWriter { inner })) (delegates to BufWriter flush/extract protocol and rewraps on error); doc on into_inner() errors: "Err will be returned if an error occurs while flushing the buffer."

**Implementation:** Model extraction as an explicit transition type, e.g., `LineWriter<Buffered, W>` with `try_into_inner(self) -> Result<W, LineWriter<Buffered, W>>` (or a dedicated `Extracting` state) so the 'still buffered on failure' path is explicit in the type rather than hidden inside IntoInnerError. This can also enable APIs that only allow `get_mut`/direct access in an 'EmptyBuffer' state if such a state can be tracked.

---

## State Machine Invariants

### 14. Repr tagged-encoding state machine (SimpleMessage / Custom / Os / Simple)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-408`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `Repr` stores four logically distinct variants inside a single `NonNull<()>` using pointer-tagging and integer stuffing. Correctness/soundness relies on the invariant that `self.0` is *always* encoded exactly per the module-level layout rules (bottom 2 bits are the tag; payload interpretation depends on tag). This is not enforced by the type system because the raw storage is a single untyped `NonNull<()>` and decoding is done via `unsafe fn decode_repr` with a contract expressed only in comments and debug assertions. Any construction of a `Repr` with invalid bits (or calling `decode_repr` on a non-`Repr` pointer) can lead to UB via `unreachable_unchecked`, invalid references, or `Box::from_raw` on a non-box pointer.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Repr, impl Send for Repr (0 methods), impl Sync for Repr (0 methods), impl Repr (8 methods), impl Drop for Repr (1 methods)

//! This is a densely packed error representation which is used on targets with
//! 64-bit pointers.
//!
//! (Note that `bitpacked` vs `unpacked` here has no relationship to
//! `#[repr(packed)]`, it just refers to attempting to use any available bits in
//! a more clever manner than `rustc`'s default layout algorithm would).
//!
//! Conceptually, it stores the same data as the "unpacked" equivalent we use on
//! other targets. Specifically, you can imagine it as an optimized version of
//! the following enum (which is roughly equivalent to what's stored by
//! `repr_unpacked::Repr`, e.g. `super::ErrorData<Box<Custom>>`):
//!
//! ```ignore (exposition-only)
//! enum ErrorData {
//!    Os(i32),
//!    Simple(ErrorKind),
//!    SimpleMessage(&'static SimpleMessage),
//!    Custom(Box<Custom>),
//! }
//! ```
//!
//! However, it packs this data into a 64bit non-zero value.
//!
//! This optimization not only allows `io::Error` to occupy a single pointer,
//! but improves `io::Result` as well, especially for situations like
//! `io::Result<()>` (which is now 64 bits) or `io::Result<u64>` (which is now
//! 128 bits), which are quite common.
//!
//! # Layout
//! Tagged values are 64 bits, with the 2 least significant bits used for the
//! tag. This means there are 4 "variants":
//!
//! - **Tag 0b00**: The first variant is equivalent to
//!   `ErrorData::SimpleMessage`, and holds a `&'static SimpleMessage` directly.
//!
//!   `SimpleMessage` has an alignment >= 4 (which is requested with
//!   `#[repr(align)]` and checked statically at the bottom of this file), which
//!   means every `&'static SimpleMessage` should have the both tag bits as 0,
//!   meaning its tagged and untagged representation are equivalent.
//!
//!   This means we can skip tagging it, which is necessary as this variant can
//!   be constructed from a `const fn`, which probably cannot tag pointers (or
//!   at least it would be difficult).
//!
//! - **Tag 0b01**: The other pointer variant holds the data for
//!   `ErrorData::Custom` and the remaining 62 bits are used to store a
//!   `Box<Custom>`. `Custom` also has alignment >= 4, so the bottom two bits
//!   are free to use for the tag.
//!
//!   The only important thing to note is that `ptr::wrapping_add` and
//!   `ptr::wrapping_sub` are used to tag the pointer, rather than bitwise
//!   operations. This should preserve the pointer's provenance, which would
//!   otherwise be lost.
//!
//! - **Tag 0b10**: Holds the data for `ErrorData::Os(i32)`. We store the `i32`
//!   in the pointer's most significant 32 bits, and don't use the bits `2..32`
//!   for anything. Using the top 32 bits is just to let us easily recover the
//!   `i32` code with the correct sign.
//!
//! - **Tag 0b11**: Holds the data for `ErrorData::Simple(ErrorKind)`. This
//!   stores the `ErrorKind` in the top 32 bits as well, although it doesn't
//!   occupy nearly that many. Most of the bits are unused here, but it's not
//!   like we need them for anything else yet.
//!
//! # Use of `NonNull<()>`
//!
//! Everything is stored in a `NonNull<()>`, which is odd, but actually serves a
//! purpose.
//!
//! Conceptually you might think of this more like:
//!
//! ```ignore (exposition-only)
//! union Repr {
//!     // holds integer (Simple/Os) variants, and
//!     // provides access to the tag bits.
//!     bits: NonZero<u64>,
//!     // Tag is 0, so this is stored untagged.
//!     msg: &'static SimpleMessage,
//!     // Tagged (offset) `Box<Custom>` pointer.
//!     tagged_custom: NonNull<()>,
//! }
//! ```
//!
//! But there are a few problems with this:
//!
//! 1. Union access is equivalent to a transmute, so this representation would
//!    require we transmute between integers and pointers in at least one
//!    direction, which may be UB (and even if not, it is likely harder for a
//!    compiler to reason about than explicit ptr->int operations).
//!
//! 2. Even if all fields of a union have a niche, the union itself doesn't,
//!    although this may change in the future. This would make things like
//!    `io::Result<()>` and `io::Result<usize>` larger, which defeats part of
//!    the motivation of this bitpacking.
//!
//! Storing everything in a `NonZero<usize>` (or some other integer) would be a
//! bit more traditional for pointer tagging, but it would lose provenance
//! information, couldn't be constructed from a `const fn`, and would probably
//! run into other issues as well.
//!
//! The `NonNull<()>` seems like the only alternative, even if it's fairly odd
//! to use a pointer type to store something that may hold an integer, some of
//! the time.

use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

use super::{Custom, ErrorData, ErrorKind, RawOsError, SimpleMessage};

// The 2 least-significant bits are used as tag.
const TAG_MASK: usize = 0b11;
const TAG_SIMPLE_MESSAGE: usize = 0b00;
const TAG_CUSTOM: usize = 0b01;
const TAG_OS: usize = 0b10;
const TAG_SIMPLE: usize = 0b11;

/// The internal representation.
///
/// See the module docs for more, this is just a way to hack in a check that we
/// indeed are not unwind-safe.
///
/// ```compile_fail,E0277
/// fn is_unwind_safe<T: core::panic::UnwindSafe>() {}
/// is_unwind_safe::<std::io::Error>();
/// ```
#[repr(transparent)]
#[rustc_insignificant_dtor]
pub(super) struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>);

// All the types `Repr` stores internally are Send + Sync, and so is it.
unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}

impl Repr {
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        match dat {
            ErrorData::Os(code) => Self::new_os(code),
            ErrorData::Simple(kind) => Self::new_simple(kind),
            ErrorData::SimpleMessage(simple_message) => Self::new_simple_message(simple_message),
            ErrorData::Custom(b) => Self::new_custom(b),
        }
    }

    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        let p = Box::into_raw(b).cast::<u8>();
        // Should only be possible if an allocator handed out a pointer with
        // wrong alignment.
        debug_assert_eq!(p.addr() & TAG_MASK, 0);
        // Note: We know `TAG_CUSTOM <= size_of::<Custom>()` (static_assert at
        // end of file), and both the start and end of the expression must be
        // valid without address space wraparound due to `Box`'s semantics.
        //
        // This means it would be correct to implement this using `ptr::add`
        // (rather than `ptr::wrapping_add`), but it's unclear this would give
        // any benefit, so we just use `wrapping_add` instead.
        let tagged = p.wrapping_add(TAG_CUSTOM).cast::<()>();
        // Safety: `TAG_CUSTOM + p` is the same as `TAG_CUSTOM | p`,
        // because `p`'s alignment means it isn't allowed to have any of the
        // `TAG_BITS` set (you can verify that addition and bitwise-or are the
        // same when the operands have no bits in common using a truth table).
        //
        // Then, `TAG_CUSTOM | p` is not zero, as that would require
        // `TAG_CUSTOM` and `p` both be zero, and neither is (as `p` came from a
        // box, and `TAG_CUSTOM` just... isn't zero -- it's `0b01`). Therefore,
        // `TAG_CUSTOM + p` isn't zero and so `tagged` can't be, and the
        // `new_unchecked` is safe.
        let res = Self(unsafe { NonNull::new_unchecked(tagged) }, PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(matches!(res.data(), ErrorData::Custom(_)), "repr(custom) encoding failed");
        res
    }

    #[inline]
    pub(super) fn new_os(code: RawOsError) -> Self {
        let utagged = ((code as usize) << 32) | TAG_OS;
        // Safety: `TAG_OS` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Os(c) if c == code),
            "repr(os) encoding failed for {code}"
        );
        res
    }

    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        let utagged = ((kind as usize) << 32) | TAG_SIMPLE;
        // Safety: `TAG_SIMPLE` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Simple(k) if k == kind),
            "repr(simple) encoding failed {:?}",
            kind,
        );
        res
    }

    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        // Safety: References are never null.
        Self(unsafe { NonNull::new_unchecked(m as *const _ as *mut ()) }, PhantomData)
    }

    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &*c) }
    }

    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &mut *c) }
    }

    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        let this = core::mem::ManuallyDrop::new(self);
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we prevent double-drop using `ManuallyDrop`.
        unsafe { decode_repr(this.0, |p| Box::from_raw(p)) }
    }
}

impl Drop for Repr {
    #[inline]
    fn drop(&mut self) {
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we're being dropped.
        unsafe {
            let _ = decode_repr(self.0, |p| Box::<Custom>::from_raw(p));
        }
    }
}

// Shared helper to decode a `Repr`'s internal pointer into an ErrorData.
//
// Safety: `ptr`'s bits should be encoded as described in the document at the
// top (it should `some_repr.0`)
#[inline]
unsafe fn decode_repr<C, F>(ptr: NonNull<()>, make_custom: F) -> ErrorData<C>
where
    F: FnOnce(*mut Custom) -> C,
{
    let bits = ptr.as_ptr().addr();
    match bits & TAG_MASK {
        TAG_OS => {
            let code = ((bits as i64) >> 32) as RawOsError;
            ErrorData::Os(code)
        }
        TAG_SIMPLE => {
            let kind_bits = (bits >> 32) as u32;
            let kind = kind_from_prim(kind_bits).unwrap_or_else(|| {
                debug_assert!(false, "Invalid io::error::Repr bits: `Repr({:#018x})`", bits);
                // This means the `ptr` passed in was not valid, which violates
                // the unsafe contract of `decode_repr`.
                //
                // Using this rather than unwrap meaningfully improves the code
                // for callers which only care about one variant (usually
                // `Custom`)
                unsafe { core::hint::unreachable_unchecked() };
            });
            ErrorData::Simple(kind)
        }
        TAG_SIMPLE_MESSAGE => {
            // SAFETY: per tag
            unsafe { ErrorData::SimpleMessage(&*ptr.cast::<SimpleMessage>().as_ptr()) }
        }
        TAG_CUSTOM => {
            // It would be correct for us to use `ptr::byte_sub` here (see the
            // comment above the `wrapping_add` call in `new_custom` for why),
            // but it isn't clear that it makes a difference, so we don't.
            let custom = ptr.as_ptr().wrapping_byte_sub(TAG_CUSTOM).cast::<Custom>();
            ErrorData::Custom(make_custom(custom))
        }
        _ => {
            // Can't happen, and compiler can tell
            unreachable!();
        }
    }
}

// This compiles to the same code as the check+transmute, but doesn't require
// unsafe, or to hard-code max ErrorKind or its size in a way the compiler
// couldn't verify.
#[inline]
fn kind_from_prim(ek: u32) -> Option<ErrorKind> {
    macro_rules! from_prim {
        ($prim:expr => $Enum:ident { $($Variant:ident),* $(,)? }) => {{
            // Force a compile error if the list gets out of date.
            const _: fn(e: $Enum) = |e: $Enum| match e {
                $($Enum::$Variant => ()),*
            };
            match $prim {
                $(v if v == ($Enum::$Variant as _) => Some($Enum::$Variant),)*
                _ => None,
            }
        }}
    }
    from_prim!(ek => ErrorKind {
        NotFound,
        PermissionDenied,
        ConnectionRefused,
        ConnectionReset,
        HostUnreachable,
        NetworkUnreachable,
        ConnectionAborted,
        NotConnected,
        AddrInUse,
        AddrNotAvailable,
        NetworkDown,
        BrokenPipe,
        AlreadyExists,
        WouldBlock,
        NotADirectory,
        IsADirectory,
        DirectoryNotEmpty,
        ReadOnlyFilesystem,
        FilesystemLoop,
        StaleNetworkFileHandle,
        InvalidInput,
        InvalidData,
        TimedOut,
        WriteZero,
        StorageFull,
        NotSeekable,
        QuotaExceeded,
        FileTooLarge,
        ResourceBusy,
        ExecutableFileBusy,
        Deadlock,
        CrossesDevices,
        TooManyLinks,
        InvalidFilename,
        ArgumentListTooLong,
        Interrupted,
        Other,
        UnexpectedEof,
        Unsupported,
        OutOfMemory,
        InProgress,
        Uncategorized,
    })
}

// Some static checking to alert us if a change breaks any of the assumptions
// that our encoding relies on for correctness and soundness. (Some of these are
// a bit overly thorough/cautious, admittedly)
//
// If any of these are hit on a platform that std supports, we should likely
// just use `repr_unpacked.rs` there instead (unless the fix is easy).
macro_rules! static_assert {
    ($condition:expr) => {
        const _: () = assert!($condition);
    };
    (@usize_eq: $lhs:expr, $rhs:expr) => {
        const _: [(); $lhs] = [(); $rhs];
    };
}

// The bitpacking we use requires pointers be exactly 64 bits.
static_assert!(@usize_eq: size_of::<NonNull<()>>(), 8);

// We also require pointers and usize be the same size.
static_assert!(@usize_eq: size_of::<NonNull<()>>(), size_of::<usize>());

// `Custom` and `SimpleMessage` need to be thin pointers.
static_assert!(@usize_eq: size_of::<&'static SimpleMessage>(), 8);
static_assert!(@usize_eq: size_of::<Box<Custom>>(), 8);

static_assert!((TAG_MASK + 1).is_power_of_two());
// And they must have sufficient alignment.
static_assert!(align_of::<SimpleMessage>() >= TAG_MASK + 1);
static_assert!(align_of::<Custom>() >= TAG_MASK + 1);

static_assert!(@usize_eq: TAG_MASK & TAG_SIMPLE_MESSAGE, TAG_SIMPLE_MESSAGE);
static_assert!(@usize_eq: TAG_MASK & TAG_CUSTOM, TAG_CUSTOM);
static_assert!(@usize_eq: TAG_MASK & TAG_OS, TAG_OS);
static_assert!(@usize_eq: TAG_MASK & TAG_SIMPLE, TAG_SIMPLE);

// This is obviously true (`TAG_CUSTOM` is `0b01`), but in `Repr::new_custom` we
// offset a pointer by this value, and expect it to both be within the same
// object, and to not wrap around the address space. See the comment in that
// function for further details.
//
// Actually, at the moment we use `ptr::wrapping_add`, not `ptr::add`, so this
// check isn't needed for that one, although the assertion that we don't
// actually wrap around in that wrapping_add does simplify the safety reasoning
// elsewhere considerably.
static_assert!(size_of::<Custom>() >= TAG_CUSTOM);

// These two store a payload which is allowed to be zero, so they must be
// non-zero to
// ... (truncated) ...
```

**Entity:** Repr

**States:** SimpleMessage (tag 0b00), Custom (tag 0b01), Os(i32) (tag 0b10), Simple(ErrorKind) (tag 0b11)

**Transitions:**
- ErrorData::Os -> Repr(NonNull<()> bits) via Repr::new_os()
- ErrorData::Simple -> Repr(NonNull<()> bits) via Repr::new_simple()
- ErrorData::SimpleMessage -> Repr(NonNull<()> ptr) via Repr::new_simple_message()
- ErrorData::Custom(Box<Custom>) -> Repr(tagged ptr) via Repr::new_custom()
- Repr -> ErrorData<&Custom> via Repr::data() (decode)
- Repr -> ErrorData<&mut Custom> via Repr::data_mut() (decode)
- Repr -> ErrorData<Box<Custom>> via Repr::into_data() (decode + take ownership)

**Evidence:** struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>) stores all variants in a single untyped pointer; module docs: 'Tagged values are 64 bits, with the 2 least significant bits used for the tag' and the 4 tag meanings; constants TAG_MASK/TAG_* define the runtime tag encoding; Repr::new_custom(): uses `wrapping_add(TAG_CUSTOM)` to tag a `Box` pointer; `debug_assert_eq!(p.addr() & TAG_MASK, 0)` assumes alignment leaves tag bits free; Repr::new_os()/new_simple(): constructs a `NonNull<()>` from `NonZeroUsize` bits via `NonNull::without_provenance(...)`; unsafe fn decode_repr(): comment 'Safety: `ptr`'s bits should be encoded as described ... (it should `some_repr.0`)' is the only contract enforcement; decode_repr(): `kind_from_prim(...).unwrap_or_else(... unreachable_unchecked())` assumes only valid `ErrorKind` encodings appear for TAG_SIMPLE; decode_repr(): TAG_SIMPLE_MESSAGE arm does `&*ptr.cast::<SimpleMessage>().as_ptr()` assuming tag==0 implies a valid `&'static SimpleMessage`; decode_repr(): TAG_CUSTOM arm subtracts tag offset and treats result as `*mut Custom` (later fed to `Box::from_raw` in Drop/into_data)

**Implementation:** Keep the packed representation internally, but make the *decoded* state explicit at the type/API boundary: introduce a private enum like `enum Decoded<'a> { SimpleMessage(&'static SimpleMessage), CustomPtr(NonNull<Custom>), Os(RawOsError), Simple(ErrorKind) }` and have a single `fn decode(&self) -> Decoded` that cannot produce invalid states (no `unreachable_unchecked`). Alternatively, use a typestate wrapper `struct TaggedPtr<T>(NonNull<()>, PhantomData<T>)` with constructors `from_simple_message`, `from_custom_ptr`, etc., so only valid tag/payload combinations are constructible, and only the Custom typestate exposes `into_box()/drop_box` paths.

---

### 23. Split iterator protocol (Active -> EOF / Error)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-28`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Split drives a BufRead source forward by repeatedly calling read_until(delim, &mut buf). It implicitly has an 'Active' state where next() may yield items, and terminal outcomes: EOF (next() returns None after read_until returns Ok(0)) and 'ErrorEncountered' (next() yields Some(Err(e)) when read_until errors). This protocol/state is not represented in the type system: after yielding an error, the iterator can still be polled and may yield further items or errors depending on the underlying BufRead, and nothing prevents callers from continuing to call next() after an error even if they intended error to be terminal.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

/// [`split`]: BufRead::split
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Split<B> {
    buf: B,
    delim: u8,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead> Iterator for Split<B> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.buf.read_until(self.delim, &mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf[buf.len() - 1] == self.delim {
                    buf.pop();
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

```

**Entity:** Split<B>

**States:** Active, EOF, ErrorEncountered

**Transitions:**
- Active -> EOF via next() when self.buf.read_until(...) returns Ok(0)
- Active -> ErrorEncountered via next() when self.buf.read_until(...) returns Err(e)
- ErrorEncountered -> Active via subsequent next() calls (behavior depends on underlying BufRead; not constrained here)

**Evidence:** struct Split<B> { buf: B, delim: u8 } stores an underlying BufRead and delimiter; Iterator::next(): match self.buf.read_until(self.delim, &mut buf); next(): Ok(0) => None encodes EOF terminal condition; next(): Err(e) => Some(Err(e)) exposes error as an item rather than making it a terminal state

**Implementation:** Model iterator states at the type level, e.g., Split<B, S> with states Active/Eof; have next(self: &mut Split<B, Active>) -> Option<Result<Vec<u8>>> and on Ok(0) transition to Split<B, Eof> (where next is a no-op returning None). If you want 'error is terminal', transition to an Error state that only yields None thereafter (or wrap Split in a helper that stops after first Err).

---

### 3. BufWriter panic-aware flushing protocol (Normal vs PanickedDuringInnerWrite)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-435`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: BufWriter tracks (at runtime) whether it is currently inside a call to the inner writer that might panic. While in that window, BufWriter must not later attempt to flush buffered data again (notably from Drop), because the inner writer may have partially written and then panicked, and re-flushing could duplicate bytes. This is implemented via the boolean field `panicked`, set true immediately before calling `inner.write(...)`/`inner.write_all(...)` and cleared after. The type system does not distinguish the 'in inner write'/'panicked' mode from the normal mode; correctness relies on disciplined setting/resetting of the flag around every inner write path (including flush_buf, write_cold, write_all_cold, and likely other truncated methods like write_vectored).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct WriterPanicked, impl WriterPanicked (1 methods), impl error :: Error for WriterPanicked (1 methods)

/// [`TcpStream`]: crate::net::TcpStream
/// [`flush`]: BufWriter::flush
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BufWriter<W: ?Sized + Write> {
    // The buffer. Avoid using this like a normal `Vec` in common code paths.
    // That is, don't use `buf.push`, `buf.extend_from_slice`, or any other
    // methods that require bounds checking or the like. This makes an enormous
    // difference to performance (we may want to stop using a `Vec` entirely).
    buf: Vec<u8>,
    // #30888: If the inner writer panics in a call to write, we don't want to
    // write the buffered data a second time in BufWriter's destructor. This
    // flag tells the Drop impl if it should skip the flush.
    panicked: bool,
    inner: W,
}

impl<W: Write> BufWriter<W> {
    /// Creates a new `BufWriter<W>` with a default buffer capacity. The default is currently 8 KiB,
    /// but may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub(crate) fn try_new_buffer() -> io::Result<Vec<u8>> {
        Vec::try_with_capacity(DEFAULT_BUF_SIZE).map_err(|_| {
            io::const_error!(ErrorKind::OutOfMemory, "failed to allocate write buffer")
        })
    }

    pub(crate) fn with_buffer(inner: W, buf: Vec<u8>) -> Self {
        Self { inner, buf, panicked: false }
    }

    /// Creates a new `BufWriter<W>` with at least the specified buffer capacity.
    ///
    /// # Examples
    ///
    /// Creating a buffer with a buffer of at least a hundred bytes.
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    /// let mut buffer = BufWriter::with_capacity(100, stream);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter { inner, buf: Vec::with_capacity(capacity), panicked: false }
    }

    /// Unwraps this `BufWriter<W>`, returning the underlying writer.
    ///
    /// The buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if an error occurs while flushing the buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // unwrap the TcpStream and flush the buffer
    /// let stream = buffer.into_inner().unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0),
        }
    }

    /// Disassembles this `BufWriter<W>`, returning the underlying writer, and any buffered but
    /// unwritten data.
    ///
    /// If the underlying writer panicked, it is not known what portion of the data was written.
    /// In this case, we return `WriterPanicked` for the buffered data (from which the buffer
    /// contents can still be recovered).
    ///
    /// `into_parts` makes no attempt to flush data and cannot fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{BufWriter, Write};
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut stream = BufWriter::new(buffer.as_mut());
    /// write!(stream, "too much data").unwrap();
    /// stream.flush().expect_err("it doesn't fit");
    /// let (recovered_writer, buffered_data) = stream.into_parts();
    /// assert_eq!(recovered_writer.len(), 0);
    /// assert_eq!(&buffered_data.unwrap(), b"ata");
    /// ```
    #[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
    pub fn into_parts(self) -> (W, Result<Vec<u8>, WriterPanicked>) {
        let mut this = ManuallyDrop::new(self);
        let buf = mem::take(&mut this.buf);
        let buf = if !this.panicked { Ok(buf) } else { Err(WriterPanicked { buf }) };

        // SAFETY: double-drops are prevented by putting `this` in a ManuallyDrop that is never dropped
        let inner = unsafe { ptr::read(&this.inner) };

        (inner, buf)
    }
}

impl<W: ?Sized + Write> BufWriter<W> {
    /// Send data in our local buffer into the inner writer, looping as
    /// necessary until either it's all been sent or an error occurs.
    ///
    /// Because all the data in the buffer has been reported to our owner as
    /// "successfully written" (by returning nonzero success values from
    /// `write`), any 0-length writes from `inner` must be reported as i/o
    /// errors from this method.
    pub(in crate::io) fn flush_buf(&mut self) -> io::Result<()> {
        /// Helper struct to ensure the buffer is updated after all the writes
        /// are complete. It tracks the number of written bytes and drains them
        /// all from the front of the buffer when dropped.
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }

        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }

            /// The unwritten part of the buffer
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }

            /// Flag some bytes as removed from the front of the buffer
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }

            /// true if all of the bytes have been written
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    self.buffer.drain(..self.written);
                }
            }
        }

        let mut guard = BufGuard::new(&mut self.buf);
        while !guard.done() {
            self.panicked = true;
            let r = self.inner.write(guard.remaining());
            self.panicked = false;

            match r {
                Ok(0) => {
                    return Err(io::const_error!(
                        ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.consume(n),
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Buffer some data without flushing it, regardless of the size of the
    /// data. Writes as much as possible without exceeding capacity. Returns
    /// the number of bytes written.
    pub(super) fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());

        // SAFETY: `amt_to_buffer` is <= buffer's spare capacity by construction.
        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }

        amt_to_buffer
    }

    /// Gets a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // we can use reference just like buffer
    /// let reference = buffer.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // we can use reference just like buffer
    /// let reference = buffer.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let buf_writer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // See how many bytes are currently buffered
    /// let bytes_buffered = buf_writer.buffer().len();
    /// ```
    #[stable(feature = "bufreader_buffer", since = "1.37.0")]
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns a mutable reference to the internal buffer.
    ///
    /// This can be used to write data directly into the buffer without triggering writers
    /// to the underlying writer.
    ///
    /// That the buffer is a `Vec` is an implementation detail.
    /// Callers should not modify the capacity as there currently is no public API to do so
    /// and thus any capacity changes would be unexpected by the user.
    pub(in crate::io) fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    /// Returns the number of bytes the internal buffer can hold without flushing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let buf_writer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // Check the capacity of the inner buffer
    /// let capacity = buf_writer.capacity();
    /// // Calculate how many bytes can be written without flushing
    /// let without_flush = capacity - buf_writer.buffer().len();
    /// ```
    #[stable(feature = "buffered_io_capacity", since = "1.46.0")]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    // Ensure this function does not get inlined into `write`, so that it
    // remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write`,
    // it's likely a sign that the client is using an improperly sized buffer
    // or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    fn write_cold(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }

        // Why not len > capacity? To avoid a needless trip through the buffer when the input
        // exactly fills it. We'd just need to flush it to the underlying writer anyway.
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer. In this case, we write to the buffer even if it fills it
            // exactly. Doing otherwise would mean flushing the buffer, then writing this
            // input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we
            // flushed the buffer to ensure that there is. In the latter case, we know that there
            // is because flushing ensured that our entire buffer is spare capacity, and we entered
            // this block because the input buffer length is less than that capacity. In either
            // case, it's safe to write the input buffer to our buffer.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        }
    }

    // Ensure this function does not get inlined into `write_all`, so that it
    // remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write_all`,
    // it's likely a sign that the client is using an improperly sized buffer
    // or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    fn write_all_cold(&mut self, buf: &[u8]) -> io::Result<()> {
        // Normally, `write_all` just calls `write` in a loop. We can do better
        // by calling `self.get_mut().write_all()` directly, which avoids
        // round trips through the buffer in the event of a series of partial
        // writes in some circumstances.

        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }

        // Why not len > capacity? To avoid a needless trip through the buffer when the input
        // exactly fills it. We'd just need to flush it to the underlying writer anyway.
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write_all(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer. In this case, we write to the buffer even if it fills it
            // exactly. Doing otherwise would mean flushing the buffer, then writing this
            // input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we
            // flushed the buffer to ensure that there is. In the latter case, we know that there
            // is because flushing ensured that our entire buffer is spare capacity, and we entered
            // this block because the input buffer length is less than that capacity. In either
            // case, it's safe to write the input buffer to our buffer.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        }
    }

    // SAFETY: Requires `buf.len() <= self.buf.capacity() - self.buf.len()`,
    // i.e., that input buffer length is less than or equal to spare capacity.
    #[inline]
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        debug_assert!(buf.len() <= self.spare_capacity());
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }

    #[inline]
    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Write for BufWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_cold` for details.
        if buf.len() < self.spare_capacity() {
            // SAFETY: safe by above conditional.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        } else {
            self.write_cold(buf)
        }
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_all_cold` for details.
        if buf.len() < self.spare_capacity() {
            // SAFETY: safe by above conditional.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        } else {
            self.write_all_cold(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<us
// ... (truncated) ...
```

**Entity:** BufWriter<W>

**States:** Normal, PanickedDuringInnerWrite

**Transitions:**
- Normal -> PanickedDuringInnerWrite by setting self.panicked = true before calling inner.write/inner.write_all
- PanickedDuringInnerWrite -> Normal by setting self.panicked = false after inner.write/inner.write_all returns normally
- PanickedDuringInnerWrite -> (observable) into_parts returns Err(WriterPanicked { buf }) for buffered data if a panic occurred and panicked was left true

**Evidence:** field: `panicked: bool` with comment: "If the inner writer panics ... don't want to write the buffered data a second time in BufWriter's destructor. This flag tells the Drop impl if it should skip the flush."; flush_buf(): `self.panicked = true; let r = self.inner.write(...); self.panicked = false;`; write_cold(): `self.panicked = true; let r = self.get_mut().write(buf); self.panicked = false;`; write_all_cold(): `self.panicked = true; let r = self.get_mut().write_all(buf); self.panicked = false;`; into_parts(): `let buf = if !this.panicked { Ok(buf) } else { Err(WriterPanicked { buf }) };` and comment: "If the underlying writer panicked, it is not known what portion of the data was written."

**Implementation:** Introduce an internal typestate/capability guard representing "in inner write" (e.g., `struct InWrite<'a>(&'a mut BufWriter<W>);` created by a method that sets `panicked=true` and whose Drop resets it to false). Route all inner write calls through this guard so it's impossible to forget to toggle the flag. Optionally make `panicked` an internal enum state (`Normal | Panicked`) that is only mutated by that guard.

---

### 13. Repr variant protocol (Os / Simple / SimpleMessage / Custom)

**Location**: `/tmp/io_test_crate/src/io/error/repr_unpacked.rs:1-48`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Repr is a newtype wrapper around `Inner = ErrorData<Box<Custom>>`, which is an internal sum type with four variants. Many operations on `Repr` depend on which variant it currently holds (e.g., whether a `Custom` payload exists to borrow mutably). This variant-dependent behavior is handled by runtime pattern matching in `data()`/`data_mut()`, and callers can only discover the 'state' by matching on the returned `ErrorData<...>`. The type system does not let code express or require 'this Repr is definitely Custom' at the call site, so any APIs that require custom-ness must either re-match or rely on convention.

**Evidence**:

```rust

type Inner = ErrorData<Box<Custom>>;

pub(super) struct Repr(Inner);

impl Repr {
    #[inline]
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        Self(dat)
    }
    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        Self(Inner::Custom(b))
    }
    #[inline]
    pub(super) fn new_os(code: RawOsError) -> Self {
        Self(Inner::Os(code))
    }
    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        Self(Inner::Simple(kind))
    }
    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        Self(Inner::SimpleMessage(m))
    }
    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        self.0
    }
    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        match &self.0 {
            Inner::Os(c) => ErrorData::Os(*c),
            Inner::Simple(k) => ErrorData::Simple(*k),
            Inner::SimpleMessage(m) => ErrorData::SimpleMessage(*m),
            Inner::Custom(m) => ErrorData::Custom(&*m),
        }
    }
    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        match &mut self.0 {
            Inner::Os(c) => ErrorData::Os(*c),
            Inner::Simple(k) => ErrorData::Simple(*k),
            Inner::SimpleMessage(m) => ErrorData::SimpleMessage(*m),
            Inner::Custom(m) => ErrorData::Custom(&mut *m),
        }
    }
}

```

**Entity:** Repr

**States:** Os, Simple, SimpleMessage, Custom

**Transitions:**
- ∅ -> Os via new_os()
- ∅ -> Simple via new_simple()
- ∅ -> SimpleMessage via new_simple_message()
- ∅ -> Custom via new_custom()
- Any -> (exposed as) ErrorData<&Custom> via data()
- Any -> (exposed as) ErrorData<&mut Custom> via data_mut()
- Any -> Inner via into_data()

**Evidence:** type Inner = ErrorData<Box<Custom>>; encodes multiple runtime variants inside Repr; pub(super) struct Repr(Inner); state is stored as the enum value in tuple field `0`; new_custom(b) constructs `Inner::Custom(b)`; new_os(code) constructs `Inner::Os(code)`; new_simple(kind) constructs `Inner::Simple(kind)`; new_simple_message(m) constructs `Inner::SimpleMessage(m)`; data() matches `&self.0` and returns different `ErrorData<&Custom>` variants depending on the stored variant; data_mut() matches `&mut self.0` and only yields `ErrorData::Custom(&mut *m)` in the `Inner::Custom` case

**Implementation:** Model `Repr` as `Repr<V>` where `V` is a zero-sized marker for the variant (e.g., `Repr<CustomV>`, `Repr<OsV>`, ...), or split into separate wrapper types `OsRepr`, `CustomRepr`, etc. Constructors return the specific type (`fn new_custom(...) -> Repr<CustomV>`), and only `Repr<CustomV>` exposes `custom_ref()` / `custom_mut()` without requiring re-matching. Provide an `enum AnyRepr { Os(Repr<OsV>), ... }` if dynamic dispatch is still needed.

---

### 7. Guard length-restore lifecycle (temporary set_len rollback)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-434`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `Guard` encodes a two-phase commit for changing a `Vec<u8>` length: initially it records the old length and will restore it on drop (rollback). If the operation succeeds, the owner updates `g.len` to the new length, effectively committing the new length on drop. This 'armed vs committed' state is represented only by a runtime field (`len`) and a convention (`g.len = g.buf.len()`), not by the type system; misuse (forgetting to commit, committing wrong len) changes behavior. The rollback-on-drop behavior is relied upon to maintain `String`'s UTF-8 invariant in `append_to_string`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom

//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! The `std::io` module contains a number of common things you'll need
//! when doing input and output. The most core part of this module is
//! the [`Read`] and [`Write`] traits, which provide the
//! most general interface for reading and writing input and output.
//!
//! ## Read and Write
//!
//! Because they are traits, [`Read`] and [`Write`] are implemented by a number
//! of other types, and you can implement them for your types too. As such,
//! you'll see a few different types of I/O throughout the documentation in
//! this module: [`File`]s, [`TcpStream`]s, and sometimes even [`Vec<T>`]s. For
//! example, [`Read`] adds a [`read`][`Read::read`] method, which we can use on
//! [`File`]s:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`Read`] and [`Write`] are so important, implementors of the two traits have a
//! nickname: readers and writers. So you'll sometimes see 'a reader' instead
//! of 'a type that implements the [`Read`] trait'. Much easier!
//!
//! ## Seek and BufRead
//!
//! Beyond that, there are two important traits that are provided: [`Seek`]
//! and [`BufRead`]. Both of these build on top of a reader to control
//! how the reading happens. [`Seek`] lets you control where the next byte is
//! coming from:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::SeekFrom;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // skip to the last 10 bytes of the file
//!     f.seek(SeekFrom::End(-10))?;
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`BufRead`] uses an internal buffer to provide a number of other ways to read, but
//! to show it off, we'll need to talk about buffers in general. Keep reading!
//!
//! ## BufReader and BufWriter
//!
//! Byte-based interfaces are unwieldy and can be inefficient, as we'd need to be
//! making near-constant calls to the operating system. To help with this,
//! `std::io` comes with two structs, [`BufReader`] and [`BufWriter`], which wrap
//! readers and writers. The wrapper uses a buffer, reducing the number of
//! calls and providing nicer methods for accessing exactly what you want.
//!
//! For example, [`BufReader`] works with the [`BufRead`] trait to add extra
//! methods to any reader:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let mut reader = BufReader::new(f);
//!     let mut buffer = String::new();
//!
//!     // read a line into buffer
//!     reader.read_line(&mut buffer)?;
//!
//!     println!("{buffer}");
//!     Ok(())
//! }
//! ```
//!
//! [`BufWriter`] doesn't add any new ways of writing; it just buffers every call
//! to [`write`][`Write::write`]:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufWriter;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::create("foo.txt")?;
//!     {
//!         let mut writer = BufWriter::new(f);
//!
//!         // write a byte to the buffer
//!         writer.write(&[42])?;
//!
//!     } // the buffer is flushed once writer goes out of scope
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Standard input and output
//!
//! A very common source of input is standard input:
//!
//! ```no_run
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!     Ok(())
//! }
//! ```
//!
//! Note that you cannot use the [`?` operator] in functions that do not return
//! a [`Result<T, E>`][`Result`]. Instead, you can call [`.unwrap()`]
//! or `match` on the return value to catch any possible errors:
//!
//! ```no_run
//! use std::io;
//!
//! let mut input = String::new();
//!
//! io::stdin().read_line(&mut input).unwrap();
//! ```
//!
//! And a very common source of output is standard output:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//!
//! fn main() -> io::Result<()> {
//!     io::stdout().write(&[42])?;
//!     Ok(())
//! }
//! ```
//!
//! Of course, using [`io::stdout`] directly is less common than something like
//! [`println!`].
//!
//! ## Iterator types
//!
//! A large number of the structures provided by `std::io` are for various
//! ways of iterating over I/O. For example, [`Lines`] is used to split over
//! lines:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let reader = BufReader::new(f);
//!
//!     for line in reader.lines() {
//!         println!("{}", line?);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Functions
//!
//! There are a number of [functions][functions-list] that offer access to various
//! features. For example, we can use three of these functions to copy everything
//! from standard input to standard output:
//!
//! ```no_run
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     io::copy(&mut io::stdin(), &mut io::stdout())?;
//!     Ok(())
//! }
//! ```
//!
//! [functions-list]: #functions-1
//!
//! ## io::Result
//!
//! Last, but certainly not least, is [`io::Result`]. This type is used
//! as the return type of many `std::io` functions that can cause an error, and
//! can be returned from your own functions as well. Many of the examples in this
//! module use the [`?` operator]:
//!
//! ```
//! use std::io;
//!
//! fn read_input() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!
//!     Ok(())
//! }
//! ```
//!
//! The return type of `read_input()`, [`io::Result<()>`][`io::Result`], is a very
//! common type for functions which don't have a 'real' return value, but do want to
//! return errors if they happen. In this case, the only purpose of this function is
//! to read the line and print it, so we use `()`.
//!
//! ## Platform-specific behavior
//!
//! Many I/O functions throughout the standard library are documented to indicate
//! what various library or syscalls they are delegated to. This is done to help
//! applications both understand what's happening under the hood as well as investigate
//! any possibly unclear semantics. Note, however, that this is informative, not a binding
//! contract. The implementation of many of these functions are subject to change over
//! time and may call fewer or more syscalls/library functions.
//!
//! ## I/O Safety
//!
//! Rust follows an I/O safety discipline that is comparable to its memory safety discipline. This
//! means that file descriptors can be *exclusively owned*. (Here, "file descriptor" is meant to
//! subsume similar concepts that exist across a wide range of operating systems even if they might
//! use a different name, such as "handle".) An exclusively owned file descriptor is one that no
//! other code is allowed to access in any way, but the owner is allowed to access and even close
//! it any time. A type that owns its file descriptor should usually close it in its `drop`
//! function. Types like [`File`] own their file descriptor. Similarly, file descriptors
//! can be *borrowed*, granting the temporary right to perform operations on this file descriptor.
//! This indicates that the file descriptor will not be closed for the lifetime of the borrow, but
//! it does *not* imply any right to close this file descriptor, since it will likely be owned by
//! someone else.
//!
//! The platform-specific parts of the Rust standard library expose types that reflect these
//! concepts, see [`os::unix`] and [`os::windows`].
//!
//! To uphold I/O safety, it is crucial that no code acts on file descriptors it does not own or
//! borrow, and no code closes file descriptors it does not own. In other words, a safe function
//! that takes a regular integer, treats it as a file descriptor, and acts on it, is *unsound*.
//!
//! Not upholding I/O safety and acting on a file descriptor without proof of ownership can lead to
//! misbehavior and even Undefined Behavior in code that relies on ownership of its file
//! descriptors: a closed file descriptor could be re-allocated, so the original owner of that file
//! descriptor is now working on the wrong file. Some code might even rely on fully encapsulating
//! its file descriptors with no operations being performed by any other part of the program.
//!
//! Note that exclusive ownership of a file descriptor does *not* imply exclusive ownership of the
//! underlying kernel object that the file descriptor references (also called "open file description" on
//! some operating systems). File descriptors basically work like [`Arc`]: when you receive an owned
//! file descriptor, you cannot know whether there are any other file descriptors that reference the
//! same kernel object. However, when you create a new kernel object, you know that you are holding
//! the only reference to it. Just be careful not to lend it to anyone, since they can obtain a
//! clone and then you can no longer know what the reference count is! In that sense, [`OwnedFd`] is
//! like `Arc` and [`BorrowedFd<'a>`] is like `&'a Arc` (and similar for the Windows types). In
//! particular, given a `BorrowedFd<'a>`, you are not allowed to close the file descriptor -- just
//! like how, given a `&'a Arc`, you are not allowed to decrement the reference count and
//! potentially free the underlying object. There is no equivalent to `Box` for file descriptors in
//! the standard library (that would be a type that guarantees that the reference count is `1`),
//! however, it would be possible for a crate to define a type with those semantics.
//!
//! [`File`]: crate::fs::File
//! [`TcpStream`]: crate::net::TcpStream
//! [`io::stdout`]: stdout
//! [`io::Result`]: self::Result
//! [`?` operator]: ../../book/appendix-02-operators.html
//! [`Result`]: crate::result::Result
//! [`.unwrap()`]: crate::result::Result::unwrap
//! [`os::unix`]: ../os/unix/io/index.html
//! [`os::windows`]: ../os/windows/io/index.html
//! [`OwnedFd`]: ../os/fd/struct.OwnedFd.html
//! [`BorrowedFd<'a>`]: ../os/fd/struct.BorrowedFd.html
//! [`Arc`]: crate::sync::Arc

#![stable(feature = "rust1", since = "1.0.0")]

#[cfg(test)]
mod tests;

#[unstable(feature = "read_buf", issue = "78485")]
pub use core::io::{BorrowedBuf, BorrowedCursor};
use core::slice::memchr;

#[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
pub use self::buffered::WriterPanicked;
#[unstable(feature = "raw_os_error_ty", issue = "107792")]
pub use self::error::RawOsError;
#[doc(hidden)]
#[unstable(feature = "io_const_error_internals", issue = "none")]
pub use self::error::SimpleMessage;
#[unstable(feature = "io_const_error", issue = "133448")]
pub use self::error::const_error;
#[stable(feature = "anonymous_pipe", since = "1.87.0")]
pub use self::pipe::{PipeReader, PipeWriter, pipe};
#[stable(feature = "is_terminal", since = "1.70.0")]
pub use self::stdio::IsTerminal;
pub(crate) use self::stdio::attempt_print_to_stderr;
#[unstable(feature = "print_internals", issue = "none")]
#[doc(hidden)]
pub use self::stdio::{_eprint, _print};
#[unstable(feature = "internal_output_capture", issue = "none")]
#[doc(no_inline, hidden)]
pub use self::stdio::{set_output_capture, try_set_output_capture};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::{
    buffered::{BufReader, BufWriter, IntoInnerError, LineWriter},
    copy::copy,
    cursor::Cursor,
    error::{Error, ErrorKind, Result},
    stdio::{Stderr, StderrLock, Stdin, StdinLock, Stdout, StdoutLock, stderr, stdin, stdout},
    util::{Empty, Repeat, Sink, empty, repeat, sink},
};
use crate::mem::take;
use crate::ops::{Deref, DerefMut};
use crate::{cmp, fmt, slice, str, sys};

mod buffered;
pub(crate) mod copy;
mod cursor;
mod error;
mod impls;
mod pipe;
pub mod prelude;
mod stdio;
mod util;

const DEFAULT_BUF_SIZE: usize = crate::sys::io::DEFAULT_BUF_SIZE;

pub(crate) use stdio::cleanup;

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

// Several `read_to_string` and `read_line` methods in the standard library will
// append data into a `String` buffer, but we need to be pretty careful when
// doing this. The implementation will just call `.as_mut_vec()` and then
// delegate to a byte-oriented reading method, but we must ensure that when
// returning we never leave `buf` in a state such that it contains invalid UTF-8
// in its bounds.
//
// To this end, we use an RAII guard (to protect against panics) which updates
// the length of the string when it is dropped. This guard initially truncates
// the string to the prior length and only after we've validated that the
// new contents are valid UTF-8 do we allow it to set a longer length.
//
// The unsafety in this function is twofold:
//
// 1. We're looking at the raw bytes of `buf`, so we take on the burden of UTF-8
//    checks.
// 2. We're passing a raw buffer to the function `f`, and it is expected that
//    the function only *appends* bytes to the buffer. We'll get undefined
//    behavior if existing bytes are overwritten to have non-UTF-8 data.
pub(crate) unsafe fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize>
where
    F: FnOnce(&mut Vec<u8>) -> Result<usize>,
{
    let mut g = Guard { len: buf.len(), buf: unsafe { buf.as_mut_vec() } };
    let ret = f(g.buf);

    // SAFETY: the caller promises to only append data to `buf`
    let appended = unsafe { g.buf.get_unchecked(g.len..) };
    if str::from_utf8(appended).is_err() {
        ret.and_then(|_| Err(Error::INVALID_UTF8))
    } else {
        g.len = g.buf.len();
        ret
    }
}

// Here we must serve many masters with conflicting goals:
//
// - avoid allocating unless necessary
// - avoid overallocating if we know the exact size (#89165)
// - avoid passing large buffers to readers that always initialize the free capacity if they perform short reads (#23815, #23820)
// - pass large buffers to readers that do not initialize the spare capacity. this can amortize per-call overheads
// - and finally pass not-too-small and not-too-large buffers to Windows read APIs because they manage to suffer from both problems
//   at the same time, i.e. small reads suffer from syscall overhead, all reads incur costs proportional to buffer size (#110650)
//
pub(crate) fn default_read_to_end<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut Vec<u8>,
    size_hint: Option<usize>,
) -> Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();
    // Optionally limit the maximum bytes read on each iteration.
    // This adds an arbitrary fiddle factor to allow for more data than we expect.
    let mut max_read_size = size_hint
        .and_then(|s| s.checked_add(1024)?.checked_next_multiple_of(DEFAULT_BUF_SIZE))
        .unwrap_or(DEFAULT_BUF_SIZE);

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration

    const PROBE_SIZE: usize = 32;

    fn small_probe_read<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize> {
        let mut probe = [0u8; PROBE_SIZE];

        loop {
            match r.read(&mut probe) {
                Ok
// ... (truncated) ...
```

**Entity:** Guard<'a>

**States:** Armed: holds original len and will restore it on Drop, Disarmed/Committed: len updated to new Vec length so Drop commits growth

**Transitions:**
- Armed -> Disarmed/Committed via assigning `g.len = g.buf.len()`
- Armed -> rollback on Drop via `Drop::drop` restoring old `set_len`
- Committed -> commit on Drop via `Drop::drop` setting length to already-updated `g.len`

**Evidence:** Guard fields: `buf: &'a mut Vec<u8>` and `len: usize` (tracks rollback length); Drop impl: `self.buf.set_len(self.len);` unconditionally applies stored len; append_to_string: initializes `Guard { len: buf.len(), ... }` (armed with old len); append_to_string: on success path sets `g.len = g.buf.len();` (commit step)

**Implementation:** Make `Guard` a typestate: `Guard<State>` where `State` is `Armed` or `Committed`. `Guard<Armed>::commit(self) -> Guard<Committed>` would consume and return the committed guard; only `Guard<Committed>` can be dropped without rollback, while `Guard<Armed>` always rolls back. This removes the implicit 'must set g.len to commit' convention.

---

### 48. Repr bitpacked tagged-union encoding (Os / Simple / SimpleMessage / Custom)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-123`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Repr is a tagged union encoded inside a single NonNull<()> using pointer/tag bitpacking. Correctness relies on an implicit state machine determined by tag bits: depending on the tag, the payload is either an immediate (OS error code or ErrorKind shifted into the upper bits), a raw pointer to a 'static SimpleMessage, or a heap pointer to Custom with an added tag offset. The type system does not express which variant is currently stored, nor does it enforce the representation invariants (non-null, tag bits disjoint from pointer alignment, correct tag ranges, and correct decoding) beyond runtime debug_asserts and safety comments.

**Evidence**:

```rust
// Note: Other parts of this module contain: 2 free function(s)

/// ```
#[repr(transparent)]
#[rustc_insignificant_dtor]
pub(super) struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>);

// All the types `Repr` stores internally are Send + Sync, and so is it.
unsafe impl Send for Repr {}
unsafe impl Sync for Repr {}

impl Repr {
    pub(super) fn new(dat: ErrorData<Box<Custom>>) -> Self {
        match dat {
            ErrorData::Os(code) => Self::new_os(code),
            ErrorData::Simple(kind) => Self::new_simple(kind),
            ErrorData::SimpleMessage(simple_message) => Self::new_simple_message(simple_message),
            ErrorData::Custom(b) => Self::new_custom(b),
        }
    }

    pub(super) fn new_custom(b: Box<Custom>) -> Self {
        let p = Box::into_raw(b).cast::<u8>();
        // Should only be possible if an allocator handed out a pointer with
        // wrong alignment.
        debug_assert_eq!(p.addr() & TAG_MASK, 0);
        // Note: We know `TAG_CUSTOM <= size_of::<Custom>()` (static_assert at
        // end of file), and both the start and end of the expression must be
        // valid without address space wraparound due to `Box`'s semantics.
        //
        // This means it would be correct to implement this using `ptr::add`
        // (rather than `ptr::wrapping_add`), but it's unclear this would give
        // any benefit, so we just use `wrapping_add` instead.
        let tagged = p.wrapping_add(TAG_CUSTOM).cast::<()>();
        // Safety: `TAG_CUSTOM + p` is the same as `TAG_CUSTOM | p`,
        // because `p`'s alignment means it isn't allowed to have any of the
        // `TAG_BITS` set (you can verify that addition and bitwise-or are the
        // same when the operands have no bits in common using a truth table).
        //
        // Then, `TAG_CUSTOM | p` is not zero, as that would require
        // `TAG_CUSTOM` and `p` both be zero, and neither is (as `p` came from a
        // box, and `TAG_CUSTOM` just... isn't zero -- it's `0b01`). Therefore,
        // `TAG_CUSTOM + p` isn't zero and so `tagged` can't be, and the
        // `new_unchecked` is safe.
        let res = Self(unsafe { NonNull::new_unchecked(tagged) }, PhantomData);
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(matches!(res.data(), ErrorData::Custom(_)), "repr(custom) encoding failed");
        res
    }

    #[inline]
    pub(super) fn new_os(code: RawOsError) -> Self {
        let utagged = ((code as usize) << 32) | TAG_OS;
        // Safety: `TAG_OS` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Os(c) if c == code),
            "repr(os) encoding failed for {code}"
        );
        res
    }

    #[inline]
    pub(super) fn new_simple(kind: ErrorKind) -> Self {
        let utagged = ((kind as usize) << 32) | TAG_SIMPLE;
        // Safety: `TAG_SIMPLE` is not zero, so the result of the `|` is not 0.
        let res = Self(
            NonNull::without_provenance(unsafe { NonZeroUsize::new_unchecked(utagged) }),
            PhantomData,
        );
        // quickly smoke-check we encoded the right thing (This generally will
        // only run in std's tests, unless the user uses -Zbuild-std)
        debug_assert!(
            matches!(res.data(), ErrorData::Simple(k) if k == kind),
            "repr(simple) encoding failed {:?}",
            kind,
        );
        res
    }

    #[inline]
    pub(super) const fn new_simple_message(m: &'static SimpleMessage) -> Self {
        // Safety: References are never null.
        Self(unsafe { NonNull::new_unchecked(m as *const _ as *mut ()) }, PhantomData)
    }

    #[inline]
    pub(super) fn data(&self) -> ErrorData<&Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &*c) }
    }

    #[inline]
    pub(super) fn data_mut(&mut self) -> ErrorData<&mut Custom> {
        // Safety: We're a Repr, decode_repr is fine.
        unsafe { decode_repr(self.0, |c| &mut *c) }
    }

    #[inline]
    pub(super) fn into_data(self) -> ErrorData<Box<Custom>> {
        let this = core::mem::ManuallyDrop::new(self);
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we prevent double-drop using `ManuallyDrop`.
        unsafe { decode_repr(this.0, |p| Box::from_raw(p)) }
    }
}

impl Drop for Repr {
    #[inline]
    fn drop(&mut self) {
        // Safety: We're a Repr, decode_repr is fine. The `Box::from_raw` is
        // safe because we're being dropped.
        unsafe {
            let _ = decode_repr(self.0, |p| Box::<Custom>::from_raw(p));
        }
    }
}

```

**Entity:** Repr

**States:** Os(tagged immediate), Simple(tagged immediate), SimpleMessage(raw pointer), Custom(tagged box pointer)

**Transitions:**
- (none/Unencoded) -> Os via new_os()
- (none/Unencoded) -> Simple via new_simple()
- (none/Unencoded) -> SimpleMessage via new_simple_message()
- (none/Unencoded) -> Custom via new_custom()
- Any state -> moved-out decoded ownership via into_data()
- Any state -> dropped (and if Custom, deallocated) via Drop::drop()

**Evidence:** struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>): stores erased pointer-sized payload + phantom ties it to ErrorData<Box<Custom>>; new(dat: ErrorData<Box<Custom>>) matches on ErrorData::{Os, Simple, SimpleMessage, Custom} and delegates to per-variant constructors; new_custom(): uses Box::into_raw(b) then p.wrapping_add(TAG_CUSTOM) and debug_assert_eq!(p.addr() & TAG_MASK, 0) (alignment/tag-bits invariant); new_custom() safety comment: relies on TAG_CUSTOM + p being non-zero and equivalent to TAG_CUSTOM | p due to disjoint bits; new_os()/new_simple(): build utagged = ((value as usize) << 32) | TAG_OS/TAG_SIMPLE and use NonZeroUsize::new_unchecked(utagged) (non-zero/tag invariant); new_simple_message(): stores m as *mut () with comment 'Safety: References are never null.' (non-null + correct provenance/lifetime invariant); data()/data_mut()/into_data()/Drop::drop(): all call unsafe decode_repr(self.0, ...) with comments 'We're a Repr, decode_repr is fine.' (requires tag/representation correctness for safety); into_data(): uses ManuallyDrop::new(self) and comment 'prevent double-drop' (ownership/state transition invariant)

**Implementation:** Represent the four variants as an explicit enum at the API boundary and confine bitpacking to a private, proven-correct layer. For stronger compile-time structure inside the module, introduce a typestate/tagged wrapper like `struct TaggedPtr<TAG>(NonNull<()>);` with zero-sized tag types `OsTag/SimpleTag/SimpleMessageTag/CustomTag` so constructors return `Repr<OsTag>` etc., and only allow decoding paths appropriate to that tag. Alternatively, use a private `enum ReprInner { Os(RawOsError), Simple(ErrorKind), SimpleMessage(&'static SimpleMessage), Custom(NonNull<Custom>) }` and provide a `From<ReprInner> for Repr` bitpack step, keeping unsafe decode isolated.

---

### 50. Error representation protocol (OS / SimpleKind / StaticMessage / CustomPayload)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-473`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `Error` has multiple implicit representation states determined by its private `repr: Repr` (backed by `ErrorData`). Which accessor methods are meaningful depends on how the `Error` was constructed: OS-backed errors expose `raw_os_error()`, custom errors expose `get_ref()/get_mut()/into_inner()`, and simple/static-message errors expose neither. These are enforced via runtime pattern matching that returns `Option`, not at the type level, so callers can only discover misuses at runtime and must branch or unwrap.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct SimpleMessage; struct Custom; enum ErrorData; enum ErrorKind; 1 free function(s)

/// [`Write`]: crate::io::Write
/// [`Seek`]: crate::io::Seek
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Error {
    repr: Repr,
}

// ... (other code) ...


/// Common errors constants for use in std
#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    pub(crate) const READ_EXACT_EOF: Self =
        const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer");

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_error!(
        ErrorKind::NotFound,
        "the number of hardware threads is not known for the target platform",
    );

    pub(crate) const UNSUPPORTED_PLATFORM: Self =
        const_error!(ErrorKind::Unsupported, "operation not supported on this platform");

    pub(crate) const WRITE_ALL_EOF: Self =
        const_error!(ErrorKind::WriteZero, "failed to write whole buffer");

    pub(crate) const ZERO_TIMEOUT: Self =
        const_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout");
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<alloc::ffi::NulError> for Error {
    /// Converts a [`alloc::ffi::NulError`] into a [`Error`].
    fn from(_: alloc::ffi::NulError) -> Error {
        const_error!(ErrorKind::InvalidInput, "data provided contains a nul byte")
    }
}

#[stable(feature = "io_error_from_try_reserve", since = "1.78.0")]
impl From<alloc::collections::TryReserveError> for Error {
    /// Converts `TryReserveError` to an error with [`ErrorKind::OutOfMemory`].
    ///
    /// `TryReserveError` won't be available as the error `source()`,
    /// but this may change in the future.
    fn from(_: alloc::collections::TryReserveError) -> Error {
        // ErrorData::Custom allocates, which isn't great for handling OOM errors.
        ErrorKind::OutOfMemory.into()
    }
}

// ... (other code) ...

    Uncategorized,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            // tidy-alphabetical-start
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            FileTooLarge => "file too large",
            HostUnreachable => "host unreachable",
            InProgress => "in progress",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            QuotaExceeded => "quota exceeded",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            ResourceBusy => "resource busy",
            StaleNetworkFileHandle => "stale network file handle",
            StorageFull => "no storage space",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
            Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
            // tidy-alphabetical-end
        }
    }
}

// ... (other code) ...

/// Intended for use for errors not exposed to the user, where allocating onto
/// the heap (for normal construction via Error::new) is too costly.
#[stable(feature = "io_error_from_errorkind", since = "1.14.0")]
impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion creates a new error with a simple representation of error kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// let not_found = ErrorKind::NotFound;
    /// let error = Error::from(not_found);
    /// assert_eq!("entity not found", format!("{error}"));
    /// ```
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error { repr: Repr::new_simple(kind) }
    }
}

impl Error {
    /// Creates a new I/O error from a known kind of error as well as an
    /// arbitrary error payload.
    ///
    /// This function is used to generically create I/O errors which do not
    /// originate from the OS itself. The `error` argument is an arbitrary
    /// payload which will be contained in this [`Error`].
    ///
    /// Note that this function allocates memory on the heap.
    /// If no extra payload is required, use the `From` conversion from
    /// `ErrorKind`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// // errors can be created from strings
    /// let custom_error = Error::new(ErrorKind::Other, "oh no!");
    ///
    /// // errors can also be created from other errors
    /// let custom_error2 = Error::new(ErrorKind::Interrupted, custom_error);
    ///
    /// // creating an error without payload (and without memory allocation)
    /// let eof_error = Error::from(ErrorKind::UnexpectedEof);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline(never)]
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }

    /// Creates a new I/O error from an arbitrary error payload.
    ///
    /// This function is used to generically create I/O errors which do not
    /// originate from the OS itself. It is a shortcut for [`Error::new`]
    /// with [`ErrorKind::Other`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Error;
    ///
    /// // errors can be created from strings
    /// let custom_error = Error::other("oh no!");
    ///
    /// // errors can also be created from other errors
    /// let custom_error2 = Error::other(custom_error);
    /// ```
    #[stable(feature = "io_error_other", since = "1.74.0")]
    pub fn other<E>(error: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Self::_new(ErrorKind::Other, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Error {
        Error { repr: Repr::new_custom(Box::new(Custom { kind, error })) }
    }

    /// Creates a new I/O error from a known kind of error as well as a constant
    /// message.
    ///
    /// This function does not allocate.
    ///
    /// You should not use this directly, and instead use the `const_error!`
    /// macro: `io::const_error!(ErrorKind::Something, "some_message")`.
    ///
    /// This function should maybe change to `from_static_message<const MSG: &'static
    /// str>(kind: ErrorKind)` in the future, when const generics allow that.
    #[inline]
    #[doc(hidden)]
    #[unstable(feature = "io_const_error_internals", issue = "none")]
    pub const fn from_static_message(msg: &'static SimpleMessage) -> Error {
        Self { repr: Repr::new_simple_message(msg) }
    }

    /// Returns an error representing the last OS error which occurred.
    ///
    /// This function reads the value of `errno` for the target platform (e.g.
    /// `GetLastError` on Windows) and will return a corresponding instance of
    /// [`Error`] for the error code.
    ///
    /// This should be called immediately after a call to a platform function,
    /// otherwise the state of the error value is indeterminate. In particular,
    /// other standard library functions may call platform functions that may
    /// (or may not) reset the error value even if they succeed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Error;
    ///
    /// let os_error = Error::last_os_error();
    /// println!("last OS error: {os_error:?}");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[doc(alias = "GetLastError")]
    #[doc(alias = "errno")]
    #[must_use]
    #[inline]
    pub fn last_os_error() -> Error {
        Error::from_raw_os_error(sys::os::errno())
    }

    /// Creates a new instance of an [`Error`] from a particular OS error code.
    ///
    /// # Examples
    ///
    /// On Linux:
    ///
    /// ```
    /// # if cfg!(target_os = "linux") {
    /// use std::io;
    ///
    /// let error = io::Error::from_raw_os_error(22);
    /// assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    /// # }
    /// ```
    ///
    /// On Windows:
    ///
    /// ```
    /// # if cfg!(windows) {
    /// use std::io;
    ///
    /// let error = io::Error::from_raw_os_error(10022);
    /// assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    /// # }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    #[inline]
    pub fn from_raw_os_error(code: RawOsError) -> Error {
        Error { repr: Repr::new_os(code) }
    }

    /// Returns the OS error that this error represents (if any).
    ///
    /// If this [`Error`] was constructed via [`last_os_error`] or
    /// [`from_raw_os_error`], then this function will return [`Some`], otherwise
    /// it will return [`None`].
    ///
    /// [`last_os_error`]: Error::last_os_error
    /// [`from_raw_os_error`]: Error::from_raw_os_error
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_os_error(err: &Error) {
    ///     if let Some(raw_os_err) = err.raw_os_error() {
    ///         println!("raw OS error: {raw_os_err:?}");
    ///     } else {
    ///         println!("Not an OS error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "raw OS error: ...".
    ///     print_os_error(&Error::last_os_error());
    ///     // Will print "Not an OS error".
    ///     print_os_error(&Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    #[inline]
    pub fn raw_os_error(&self) -> Option<RawOsError> {
        match self.repr.data() {
            ErrorData::Os(i) => Some(i),
            ErrorData::Custom(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
        }
    }

    /// Returns a reference to the inner error wrapped by this error (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] then this function will
    /// return [`Some`], otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_error(err: &Error) {
    ///     if let Some(inner_err) = err.get_ref() {
    ///         println!("Inner error: {inner_err:?}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(&Error::last_os_error());
    ///     // Will print "Inner error: ...".
    ///     print_error(&Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use]
    #[inline]
    pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(&*c.error),
        }
    }

    /// Returns a mutable reference to the inner error wrapped by this error
    /// (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] then this function will
    /// return [`Some`], otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    /// use std::{error, fmt};
    /// use std::fmt::Display;
    ///
    /// #[derive(Debug)]
    /// struct MyError {
    ///     v: String,
    /// }
    ///
    /// impl MyError {
    ///     fn new() -> MyError {
    ///         MyError {
    ///             v: "oh no!".to_string()
    ///         }
    ///     }
    ///
    ///     fn change_message(&mut self, new_message: &str) {
    ///         self.v = new_message.to_string();
    ///     }
    /// }
    ///
    /// impl error::Error for MyError {}
    ///
    /// impl Display for MyError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "MyError: {}", self.v)
    ///     }
    /// }
    ///
    /// fn change_error(mut err: Error) -> Error {
    ///     if let Some(inner_err) = err.get_mut() {
    ///         inner_err.downcast_mut::<MyError>().unwrap().change_message("I've been changed!");
    ///     }
    ///     err
    /// }
    ///
    /// fn print_error(err: &Error) {
    ///     if let Some(inner_err) = err.get_ref() {
    ///         println!("Inner error: {inner_err}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(&change_error(Error::last_os_error()));
    ///     // Will print "Inner error: ...".
    ///     print_error(&change_error(Error::new(ErrorKind::Other, MyError::new())));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
        match self.repr.data_mut() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(&mut *c.error),
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this [`Error`] was constructed via [`new`] or [`other`],
    /// then this function will return [`Some`],
    /// otherwise it will return [`None`].
    ///
    /// [`new`]: Error::new
    /// [`other`]: Error::other
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_error(err: Error) {
    ///     if let Some(inner_err) = err.into_inner() {
    ///         println!("Inner error: {inner_err}");
    ///     } else {
    ///         println!("No inner error");
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Will print "No inner error".
    ///     print_error(Error::last_os_error());
    ///     // Will print "Inner error: ...".
    ///     print_error(Error::new(ErrorKind::Other, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "io_error_inner", since = "1.3.0")]
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
        match self.repr.into_data() {
            ErrorData::Os(..) => None,
            ErrorData::Simple(..) => None,
            ErrorData::SimpleMessage(..) => None,
            ErrorData::Custom(c) => Some(c.error),
   
// ... (truncated) ...
```

**Entity:** Error

**States:** Os, Simple, SimpleMessage, Custom

**Transitions:**
- (none exposed directly; construction chooses a representation) Os via last_os_error()/from_raw_os_error()
- (none exposed directly; construction chooses a representation) Simple via From<ErrorKind>
- (none exposed directly; construction chooses a representation) SimpleMessage via from_static_message()/const_error!()
- (none exposed directly; construction chooses a representation) Custom via new()/other()

**Evidence:** struct Error { repr: Repr } — representation is encapsulated and can vary at runtime; Error::last_os_error() returns Error::from_raw_os_error(sys::os::errno()) (OS-backed representation); Error::from_raw_os_error(code) constructs `Repr::new_os(code)`; impl From<ErrorKind> for Error constructs `Repr::new_simple(kind)`; Error::from_static_message(msg) constructs `Repr::new_simple_message(msg)`; Error::new/_new and Error::other construct `Repr::new_custom(Box::new(Custom { kind, error }))`; raw_os_error(): matches `self.repr.data()` and returns Some only for `ErrorData::Os(i)`; otherwise None; get_ref()/get_mut()/into_inner(): return Some only for `ErrorData::Custom(c)`; otherwise None; docs on raw_os_error/get_ref/get_mut/into_inner: 'If constructed via X then Some, otherwise None'

**Implementation:** Introduce an internal typestate-like split (even if not exposed publicly) such as `OsError`, `KindError`, `StaticMsgError`, `CustomError` newtypes wrapping `Error`/`Repr`, with conversions into the erased `Error`. Expose state-specific APIs on the typed variants (e.g., `OsError::raw_os_error() -> RawOsError`, `CustomError::{get_ref,get_mut,into_inner} -> ...`) and only provide fallible downcasts (`TryFrom<Error> for OsError`, etc.) when converting from erased `Error`.

---

### 38. Cursor-like consumption protocol for byte slices (Remaining -> EOF)

**Location**: `/tmp/io_test_crate/src/io/impls.rs:1-589`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The implementations treat `&[u8]` as a moving read cursor: every successful read/consume advances `self` to the unread tail, and EOF is represented by `self.is_empty()`. Correct usage relies on the implicit state stored in the slice value itself (the remaining sub-slice). The type system does not distinguish a 'non-empty/readable' slice from an 'empty/EOF' slice, so APIs like `read_exact` must detect the state at runtime and error when insufficient bytes remain.

**Evidence**:

```rust
#[cfg(test)]
mod tests;

use crate::alloc::Allocator;
use crate::collections::VecDeque;
use crate::io::{self, BorrowedCursor, BufRead, IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write};
use crate::{cmp, fmt, mem, str};

// =============================================================================
// Forwarding implementations

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for &mut B {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for Box<B> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

// =============================================================================
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
#[stable(feature = "rust1", since = "1.0.0")]
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let amt = cmp::min(cursor.capacity(), self.len());
        let (a, b) = self.split_at(amt);

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            nread += self.read(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nread)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            // `read_exact` makes no promise about the content of `buf` if it
            // fails so don't bother about that.
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() > self.len() {
            // Append everything we can to the cursor.
            cursor.append(*self);
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(cursor.capacity());

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(*self);
        *self = &self[len..];
        Ok(len)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content = str::from_utf8(self).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = self.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        *self = &self[len..];
        Ok(len)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for &[u8] {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(*self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            nwritten += self.write(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nwritten)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? < data.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        for buf in bufs {
            if self.write(buf)? < buf.len() {
                return Err(io::Error::WRITE_ALL_EOF);
            }
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
#[stable(feature = "rust1", since = "1.0.0")]
impl<A: Allocator> Write for Vec<u8, A> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend_from_slice(buf);
        }
        Ok(len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        self.write_vectored(bufs)?;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
#[stable(feature = "vecdeque_read_write", since = "1.63.0")]
impl<A: Allocator> Read for VecDeque<u8, A> {
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let (front, back) = self.as_slices();

        // Use only the front buffer if it is big enough to fill `buf`, else use
        // the back buffer too.
        match buf.split_at_mut_checked(front.len()) {
            None => buf.copy_from_slice(&front[..buf.len()]),
            Some((buf_front, buf_back)) => match back.split_at_checked(buf_back.len()) {
                Some((back, _)) => {
                    buf_front.copy_from_slice(front);
                    buf_back.copy_from_slice(back);
                }
                None => {
                    self.clear();
                    return Err(io::Error::READ_EXACT_EOF);
                }
            },
        }

        self.drain(..buf.len());
        Ok(())
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let (ref mut front, _) = self.as_slices();
        let n = cmp::min(cursor.capacity(), front.len());
        Read::read_buf(front, cursor)?;
        self.drain(..n);
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let len = cursor.capacity();
        let (front, back) = self.as_slices();

        match front.split_at_checked(cursor.capacity()) {
            Some((front, _)) => cursor.append(front),
            None => {
                cursor.append(front);
                match back.split_at_checked(cursor.capacity()) {
                    Some((back, _)) => cursor.append(back),
                    None => {
                        cursor.append(back);
                        self.clear();
                        return Err(io::Error::READ_EXACT_EOF);
                    }
                }
            }
        }

        self.drain(..len);
        Ok(())
// ... (truncated) ...
```

**Entity:** &[u8] (as Read/BufRead)

**States:** Remaining(bytes_left > 0), EOF(bytes_left == 0)

**Transitions:**
- Remaining -> Remaining via read()/read_buf()/read_vectored()/read_to_end()/read_to_string()/BufRead::consume() (advances cursor)
- Remaining -> EOF via the same methods when the remaining length becomes 0
- Remaining -> EOF via read_exact()/read_buf_exact() on insufficient remaining bytes (also advances to empty before returning Err)

**Evidence:** comment: "reading updates the slice to point to the yet unread part. The slice will be empty when EOF is reached."; Read for &[u8]::read(): `let (a, b) = self.split_at(amt); ... *self = b;`; Read for &[u8]::read_exact(): `if buf.len() > self.len() { *self = &self[self.len()..]; return Err(io::Error::READ_EXACT_EOF); } ... *self = b;`; BufRead for &[u8]::consume(): `*self = &self[amt..];` (assumes caller obeys `amt <= self.len()` contract typical of BufRead)

**Implementation:** Introduce a dedicated cursor type (e.g., `struct SliceCursor<'a> { rem: &'a [u8] }`) with methods returning `Option<NonEmptySliceCursor>` or a `Result` that can encode 'EOF' as a distinct type/state; optionally use a `NonEmpty` newtype for non-empty slices when an operation requires at least 1 byte, and provide `advance(n)` that is bounds-checked.

---

### 39. Fixed-capacity sink protocol for mutable slices (SpaceLeft -> Full/WriteZero)

**Location**: `/tmp/io_test_crate/src/io/impls.rs:1-589`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `&mut [u8]` is used as a cursor-like fixed-capacity output buffer: each write advances `self` to the unwritten tail, and once empty it can only accept 0 bytes (short writes). The inability to write the requested amount is surfaced as a runtime condition: `write_all`/`write_all_vectored` return an error when the slice runs out of space. The type system does not prevent calling `write_all` with data longer than the remaining capacity, nor does it distinguish 'has space' vs 'full' at compile time.

**Evidence**:

```rust
#[cfg(test)]
mod tests;

use crate::alloc::Allocator;
use crate::collections::VecDeque;
use crate::io::{self, BorrowedCursor, BufRead, IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write};
use crate::{cmp, fmt, mem, str};

// =============================================================================
// Forwarding implementations

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for &mut B {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for Box<B> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

// =============================================================================
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
#[stable(feature = "rust1", since = "1.0.0")]
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let amt = cmp::min(cursor.capacity(), self.len());
        let (a, b) = self.split_at(amt);

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            nread += self.read(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nread)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            // `read_exact` makes no promise about the content of `buf` if it
            // fails so don't bother about that.
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() > self.len() {
            // Append everything we can to the cursor.
            cursor.append(*self);
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(cursor.capacity());

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(*self);
        *self = &self[len..];
        Ok(len)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content = str::from_utf8(self).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = self.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        *self = &self[len..];
        Ok(len)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for &[u8] {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(*self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            nwritten += self.write(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nwritten)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? < data.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        for buf in bufs {
            if self.write(buf)? < buf.len() {
                return Err(io::Error::WRITE_ALL_EOF);
            }
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
#[stable(feature = "rust1", since = "1.0.0")]
impl<A: Allocator> Write for Vec<u8, A> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend_from_slice(buf);
        }
        Ok(len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        self.write_vectored(bufs)?;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
#[stable(feature = "vecdeque_read_write", since = "1.63.0")]
impl<A: Allocator> Read for VecDeque<u8, A> {
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let (front, back) = self.as_slices();

        // Use only the front buffer if it is big enough to fill `buf`, else use
        // the back buffer too.
        match buf.split_at_mut_checked(front.len()) {
            None => buf.copy_from_slice(&front[..buf.len()]),
            Some((buf_front, buf_back)) => match back.split_at_checked(buf_back.len()) {
                Some((back, _)) => {
                    buf_front.copy_from_slice(front);
                    buf_back.copy_from_slice(back);
                }
                None => {
                    self.clear();
                    return Err(io::Error::READ_EXACT_EOF);
                }
            },
        }

        self.drain(..buf.len());
        Ok(())
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let (ref mut front, _) = self.as_slices();
        let n = cmp::min(cursor.capacity(), front.len());
        Read::read_buf(front, cursor)?;
        self.drain(..n);
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let len = cursor.capacity();
        let (front, back) = self.as_slices();

        match front.split_at_checked(cursor.capacity()) {
            Some((front, _)) => cursor.append(front),
            None => {
                cursor.append(front);
                match back.split_at_checked(cursor.capacity()) {
                    Some((back, _)) => cursor.append(back),
                    None => {
                        cursor.append(back);
                        self.clear();
                        return Err(io::Error::READ_EXACT_EOF);
                    }
                }
            }
        }

        self.drain(..len);
        Ok(())
// ... (truncated) ...
```

**Entity:** &mut [u8] (as Write)

**States:** SpaceLeft(remaining_capacity > 0), Full(remaining_capacity == 0)

**Transitions:**
- SpaceLeft -> SpaceLeft via write()/write_vectored() (advances cursor)
- SpaceLeft -> Full via write()/write_vectored() when remaining reaches 0
- SpaceLeft/Full -> (error) via write_all()/write_all_vectored() if a short write occurs

**Evidence:** comment: "writing updates the slice to point to the yet unwritten part. The slice will be empty when it has been completely overwritten."; Write for &mut [u8]::write(): `let (a, b) = mem::take(self).split_at_mut(amt); ... *self = b;`; comment: "If the number of bytes to be written exceeds the size of the slice, write operations will return short writes ... `write_all` returns an error ..."; Write for &mut [u8]::write_all(): `if self.write(data)? < data.len() { Err(io::Error::WRITE_ALL_EOF) }`; Write for &mut [u8]::write_all_vectored(): returns `Err(io::Error::WRITE_ALL_EOF)` when `self.write(buf)? < buf.len()`

**Implementation:** Wrap the slice in a cursor/sink type with states like `OutBuf<HasSpace>` / `OutBuf<Full>`; make `write_all` only available when compile-time capacity is known (e.g., const-generic `[u8; N]`) or accept a `&mut OutBuf<HasAtLeast<N>>` style capability token when length is statically known. Otherwise, provide only fallible `try_write_all` returning a remainder type to make partial writes explicit.

---

### 46. IoSlice cursor/consumption protocol (Unadvanced -> PartiallyAdvanced -> FullyAdvanced)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-155`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: IoSlice represents a view into an underlying byte slice plus an implicit 'cursor' that can be advanced. The API relies on runtime panics to enforce that advancing never exceeds the remaining length. After advancing, the logical contents shrink (the start pointer moves forward), and once fully advanced the slice becomes empty. These states (and the validity of 'advance by n') are not represented in the type system; any call site can attempt to over-advance and will panic at runtime. Additionally, IoSlice::new has a platform-specific size precondition (Windows: <= 4GB) that is not captured in the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

#[stable(feature = "iovec", since = "1.36.0")]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IoSlice<'a>(sys::io::IoSlice<'a>);

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Send for IoSlice<'a> {}

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Sync for IoSlice<'a> {}


// ... (other code) ...

    }
}

impl<'a> IoSlice<'a> {
    /// Creates a new `IoSlice` wrapping a byte slice.
    ///
    /// # Panics
    ///
    /// Panics on Windows if the slice is larger than 4GB.
    #[stable(feature = "iovec", since = "1.36.0")]
    #[must_use]
    #[inline]
    pub fn new(buf: &'a [u8]) -> IoSlice<'a> {
        IoSlice(sys::io::IoSlice::new(buf))
    }

    /// Advance the internal cursor of the slice.
    ///
    /// Also see [`IoSlice::advance_slices`] to advance the cursors of multiple
    /// buffers.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSlice;
    /// use std::ops::Deref;
    ///
    /// let data = [1; 8];
    /// let mut buf = IoSlice::new(&data);
    ///
    /// // Mark 3 bytes as read.
    /// buf.advance(3);
    /// assert_eq!(buf.deref(), [1; 5].as_ref());
    /// ```
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0.advance(n)
    }

    /// Advance a slice of slices.
    ///
    /// Shrinks the slice to remove any `IoSlice`s that are fully advanced over.
    /// If the cursor ends up in the middle of an `IoSlice`, it is modified
    /// to start at that cursor.
    ///
    /// For example, if we have a slice of two 8-byte `IoSlice`s, and we advance by 10 bytes,
    /// the result will only include the second `IoSlice`, advanced by 2 bytes.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSlice;
    /// use std::ops::Deref;
    ///
    /// let buf1 = [1; 8];
    /// let buf2 = [2; 16];
    /// let buf3 = [3; 8];
    /// let mut bufs = &mut [
    ///     IoSlice::new(&buf1),
    ///     IoSlice::new(&buf2),
    ///     IoSlice::new(&buf3),
    /// ][..];
    ///
    /// // Mark 10 bytes as written.
    /// IoSlice::advance_slices(&mut bufs, 10);
    /// assert_eq!(bufs[0].deref(), [2; 14].as_ref());
    /// assert_eq!(bufs[1].deref(), [3; 8].as_ref());
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance_slices(bufs: &mut &mut [IoSlice<'a>], n: usize) {
        // Number of buffers to remove.
        let mut remove = 0;
        // Remaining length before reaching n. This prevents overflow
        // that could happen if the length of slices in `bufs` were instead
        // accumulated. Those slice may be aliased and, if they are large
        // enough, their added length may overflow a `usize`.
        let mut left = n;
        for buf in bufs.iter() {
            if let Some(remainder) = left.checked_sub(buf.len()) {
                left = remainder;
                remove += 1;
            } else {
                break;
            }
        }

        *bufs = &mut take(bufs)[remove..];
        if bufs.is_empty() {
            assert!(left == 0, "advancing io slices beyond their length");
        } else {
            bufs[0].advance(left);
        }
    }

    /// Get the underlying bytes as a slice with the original lifetime.
    ///
    /// This doesn't borrow from `self`, so is less restrictive than calling
    /// `.deref()`, which does.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(io_slice_as_bytes)]
    /// use std::io::IoSlice;
    ///
    /// let data = b"abcdef";
    ///
    /// let mut io_slice = IoSlice::new(data);
    /// let tail = &io_slice.as_slice()[3..];
    ///
    /// // This works because `tail` doesn't borrow `io_slice`
    /// io_slice = IoSlice::new(tail);
    ///
    /// assert_eq!(io_slice.as_slice(), b"def");
    /// ```
    #[unstable(feature = "io_slice_as_bytes", issue = "132818")]
    pub const fn as_slice(self) -> &'a [u8] {
        self.0.as_slice()
    }
}

#[stable(feature = "iovec", since = "1.36.0")]
impl<'a> Deref for IoSlice<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

```

**Entity:** IoSlice<'a>

**States:** Unadvanced, PartiallyAdvanced, FullyAdvanced (empty)

**Transitions:**
- Unadvanced -> PartiallyAdvanced via IoSlice::advance(n) where 0 < n < len
- Unadvanced -> FullyAdvanced (empty) via IoSlice::advance(n) where n == len
- PartiallyAdvanced -> PartiallyAdvanced via IoSlice::advance(n) where n < remaining
- PartiallyAdvanced -> FullyAdvanced (empty) via IoSlice::advance(n) where n == remaining

**Evidence:** IoSlice::advance(&mut self, n): comment 'Advance the internal cursor of the slice.'; IoSlice::advance(&mut self, n): doc 'Panics when trying to advance beyond the end of the slice.'; IoSlice::new(buf): doc 'Panics on Windows if the slice is larger than 4GB.'; Deref for IoSlice: deref() returns self.0.as_slice(), implying the visible slice changes after advance()

**Implementation:** Introduce checked variants that encode the precondition in the return type instead of panicking, e.g. `fn try_new(buf: &'a [u8]) -> Result<IoSlice<'a>, TooLarge>` and `fn try_advance(&mut self, n: usize) -> Result<(), AdvancePastEnd>`. Optionally use a newtype for sizes like `struct AdvanceBy(usize);` constructed only if `n <= remaining`, so `advance(AdvanceBy)` cannot over-advance.

---

### 2. Take read-budget protocol (BudgetRemaining / EOFReached)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-252`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `Take<T>` encodes an implicit state machine driven by the runtime value of `limit`. While `limit > 0`, reads/delegation into `inner` are allowed but must not exceed the remaining budget, and successful reads decrement `limit`. Once `limit == 0`, `Take` must behave as EOF and must not call into the underlying reader (to avoid blocking). None of these states/transitions are represented in the type system; they are enforced by runtime branches, saturating logic, and an `assert!` on the underlying reader's behavior.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

/// [`take`]: Read::take
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Take<T> {
    inner: T,
    limit: u64,
}

impl<T> Take<T> {
    /// Returns the number of bytes that can be read before this instance will
    /// return EOF.
    ///
    /// # Note
    ///
    /// This instance may reach `EOF` after reading fewer bytes than indicated by
    /// this method if the underlying [`Read`] instance reaches EOF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let f = File::open("foo.txt")?;
    ///
    ///     // read at most five bytes
    ///     let handle = f.take(5);
    ///
    ///     println!("limit: {}", handle.limit());
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Sets the number of bytes that can be read before this instance will
    /// return EOF. This is the same as constructing a new `Take` instance, so
    /// the amount of bytes read and the previous limit value don't matter when
    /// calling this method.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let f = File::open("foo.txt")?;
    ///
    ///     // read at most five bytes
    ///     let mut handle = f.take(5);
    ///     handle.set_limit(10);
    ///
    ///     assert_eq!(handle.limit(), 10);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "take_set_limit", since = "1.27.0")]
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    /// Consumes the `Take`, returning the wrapped reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut file = File::open("foo.txt")?;
    ///
    ///     let mut buffer = [0; 5];
    ///     let mut handle = file.take(5);
    ///     handle.read(&mut buffer)?;
    ///
    ///     let file = handle.into_inner();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "io_take_into_inner", since = "1.15.0")]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying reader.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying reader as doing so may corrupt the internal limit of this
    /// `Take`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut file = File::open("foo.txt")?;
    ///
    ///     let mut buffer = [0; 5];
    ///     let mut handle = file.take(5);
    ///     handle.read(&mut buffer)?;
    ///
    ///     let file = handle.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "more_io_inner_methods", since = "1.20.0")]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying reader as doing so may corrupt the internal limit of this
    /// `Take`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut file = File::open("foo.txt")?;
    ///
    ///     let mut buffer = [0; 5];
    ///     let mut handle = file.take(5);
    ///     handle.read(&mut buffer)?;
    ///
    ///     let file = handle.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "more_io_inner_methods", since = "1.20.0")]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Read> Read for Take<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Don't call into inner reader at all at EOF because it may still block
        if self.limit == 0 {
            return Ok(0);
        }

        let max = cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        assert!(n as u64 <= self.limit, "number of read bytes exceeds limit");
        self.limit -= n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> Result<()> {
        // Don't call into inner reader at all at EOF because it may still block
        if self.limit == 0 {
            return Ok(());
        }

        if self.limit < buf.capacity() as u64 {
            // The condition above guarantees that `self.limit` fits in `usize`.
            let limit = self.limit as usize;

            let extra_init = cmp::min(limit, buf.init_ref().len());

            // SAFETY: no uninit data is written to ibuf
            let ibuf = unsafe { &mut buf.as_mut()[..limit] };

            let mut sliced_buf: BorrowedBuf<'_> = ibuf.into();

            // SAFETY: extra_init bytes of ibuf are known to be initialized
            unsafe {
                sliced_buf.set_init(extra_init);
            }

            let mut cursor = sliced_buf.unfilled();
            let result = self.inner.read_buf(cursor.reborrow());

            let new_init = cursor.init_ref().len();
            let filled = sliced_buf.len();

            // cursor / sliced_buf / ibuf must drop here

            unsafe {
                // SAFETY: filled bytes have been filled and therefore initialized
                buf.advance_unchecked(filled);
                // SAFETY: new_init bytes of buf's unfilled buffer have been initialized
                buf.set_init(new_init);
            }

            self.limit -= filled as u64;

            result
        } else {
            let written = buf.written();
            let result = self.inner.read_buf(buf.reborrow());
            self.limit -= (buf.written() - written) as u64;
            result
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: BufRead> BufRead for Take<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        // Don't call into inner reader at all at EOF because it may still block
        if self.limit == 0 {
            return Ok(&[]);
        }

        let buf = self.inner.fill_buf()?;
        let cap = cmp::min(buf.len() as u64, self.limit) as usize;
        Ok(&buf[..cap])
    }

    fn consume(&mut self, amt: usize) {
        // Don't let callers reset the limit by passing an overlarge value
        let amt = cmp::min(amt as u64, self.limit) as usize;
        self.limit -= amt as u64;
        self.inner.consume(amt);
    }
}

impl<T> SizeHint for Take<T> {
    #[inline]
    fn lower_bound(&self) -> usize {
        cmp::min(SizeHint::lower_bound(&self.inner) as u64, self.limit) as usize
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        match SizeHint::upper_bound(&self.inner) {
            Some(upper_bound) => Some(cmp::min(upper_bound as u64, self.limit) as usize),
            None => self.limit.try_into().ok(),
        }
    }
}

```

**Entity:** Take<T>

**States:** BudgetRemaining (limit > 0), EOFReached (limit == 0)

**Transitions:**
- BudgetRemaining -> BudgetRemaining via Read::read()/Read::read_buf()/BufRead::consume() when some bytes are read/consumed but limit stays > 0
- BudgetRemaining -> EOFReached via Read::read()/Read::read_buf()/BufRead::consume() when the operation exhausts the remaining limit
- EOFReached -> BudgetRemaining via Take::set_limit()

**Evidence:** field: `limit: u64` stores the remaining read budget and therefore the state; Read::read(): `if self.limit == 0 { return Ok(0); }` and comment `Don't call into inner reader at all at EOF because it may still block`; Read::read(): `assert!(n as u64 <= self.limit, "number of read bytes exceeds limit")` enforces the implicit precondition that the underlying reader must not over-read beyond `max`; Read::read(): `self.limit -= n as u64;` performs the state transition by decrementing the budget; Read::read_buf(): `if self.limit == 0 { return Ok(()); }` and then later `self.limit -= filled as u64;`; BufRead::fill_buf(): `if self.limit == 0 { return Ok(&[]); }` returns an empty slice (EOF view) without calling into `inner`; BufRead::consume(): comment `Don't let callers reset the limit by passing an overlarge value` and code `let amt = cmp::min(amt as u64, self.limit) as usize; self.limit -= amt as u64;`; Take::set_limit(): `self.limit = limit;` can re-open reading after EOF by setting a new budget

**Implementation:** Model the budget state at the type level: `struct Take<T, S> { inner: T, limit: u64, _s: PhantomData<S> }` with `struct Remaining; struct Exhausted;`. Provide constructors that yield `Take<T, Remaining>` when limit>0 and `Take<T, Exhausted>` when limit==0. Implement `Read/BufRead` only for `Remaining` (or implement for both but in `Exhausted` make methods trivially EOF without inner access). `set_limit` would transition `Take<T, Exhausted> -> Take<T, Remaining>` (and potentially `Remaining -> Remaining`), preventing accidental "EOF but still calls inner" variants and making the EOF/non-EOF behavior explicit in types.

---

### 22. Buffer cursor/length/initialization state machine (pos/filled/initialized invariants)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader/buffer.rs:1-143`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Buffer relies on a set of coupled numeric invariants to make unsafe slicing and partial-initialization tracking sound: (1) pos <= filled <= buf.len(); (2) bytes in range [0..filled) are initialized; (3) initialized tracks the maximum known-initialized prefix of buf used to seed BorrowedBuf::set_init, and must satisfy filled <= initialized <= buf.len(); (4) after fill_buf observes Exhausted (pos>=filled), it resets the cursor and updates filled/initialized from the read. These invariants are enforced by method discipline, debug_asserts, and unsafe comments rather than by the type system. Incorrect external mutation or future internal changes could violate them and make buffer()'s unchecked assume_init_ref unsound or make read_more's old_init computation underflow/lie to set_init.

**Evidence**:

```rust
use crate::io::{self, BorrowedBuf, ErrorKind, Read};
use crate::mem::MaybeUninit;

pub struct Buffer {
    // The buffer.
    buf: Box<[MaybeUninit<u8>]>,
    // The current seek offset into `buf`, must always be <= `filled`.
    pos: usize,
    // Each call to `fill_buf` sets `filled` to indicate how many bytes at the start of `buf` are
    // initialized with bytes from a read.
    filled: usize,
    // This is the max number of bytes returned across all `fill_buf` calls. We track this so that we
    // can accurately tell `read_buf` how many bytes of buf are initialized, to bypass as much of its
    // defensive initialization as possible. Note that while this often the same as `filled`, it
    // doesn't need to be. Calls to `fill_buf` are not required to actually fill the buffer, and
    // omitting this is a huge perf regression for `Read` impls that do not.
    initialized: usize,
}

impl Buffer {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let buf = Box::new_uninit_slice(capacity);
        Self { buf, pos: 0, filled: 0, initialized: 0 }
    }

    #[inline]
    pub fn try_with_capacity(capacity: usize) -> io::Result<Self> {
        match Box::try_new_uninit_slice(capacity) {
            Ok(buf) => Ok(Self { buf, pos: 0, filled: 0, initialized: 0 }),
            Err(_) => {
                Err(io::const_error!(ErrorKind::OutOfMemory, "failed to allocate read buffer"))
            }
        }
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        // SAFETY: self.pos and self.cap are valid, and self.cap => self.pos, and
        // that region is initialized because those are all invariants of this type.
        unsafe { self.buf.get_unchecked(self.pos..self.filled).assume_init_ref() }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn filled(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    // This is only used by a test which asserts that the initialization-tracking is correct.
    #[cfg(test)]
    pub fn initialized(&self) -> usize {
        self.initialized
    }

    #[inline]
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }

    #[inline]
    pub fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.filled);
    }

    /// If there are `amt` bytes available in the buffer, pass a slice containing those bytes to
    /// `visitor` and return true. If there are not enough bytes available, return false.
    #[inline]
    pub fn consume_with<V>(&mut self, amt: usize, mut visitor: V) -> bool
    where
        V: FnMut(&[u8]),
    {
        if let Some(claimed) = self.buffer().get(..amt) {
            visitor(claimed);
            // If the indexing into self.buffer() succeeds, amt must be a valid increment.
            self.pos += amt;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn unconsume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_sub(amt);
    }

    /// Read more bytes into the buffer without discarding any of its contents
    pub fn read_more(&mut self, mut reader: impl Read) -> io::Result<usize> {
        let mut buf = BorrowedBuf::from(&mut self.buf[self.filled..]);
        let old_init = self.initialized - self.filled;
        unsafe {
            buf.set_init(old_init);
        }
        reader.read_buf(buf.unfilled())?;
        self.filled += buf.len();
        self.initialized += buf.init_len() - old_init;
        Ok(buf.len())
    }

    /// Remove bytes that have already been read from the buffer.
    pub fn backshift(&mut self) {
        self.buf.copy_within(self.pos.., 0);
        self.filled -= self.pos;
        self.pos = 0;
    }

    #[inline]
    pub fn fill_buf(&mut self, mut reader: impl Read) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.filled {
            debug_assert!(self.pos == self.filled);

            let mut buf = BorrowedBuf::from(&mut *self.buf);
            // SAFETY: `self.filled` bytes will always have been initialized.
            unsafe {
                buf.set_init(self.initialized);
            }

            let result = reader.read_buf(buf.unfilled());

            self.pos = 0;
            self.filled = buf.len();
            self.initialized = buf.init_len();

            result?;
        }
        Ok(self.buffer())
    }
}

```

**Entity:** Buffer

**States:** Empty (pos==0, filled==0), Readable (pos < filled), Exhausted (pos==filled), Partially-initialized backing storage (initialized may be > filled)

**Transitions:**
- Empty -> Readable via fill_buf() when reader.read_buf reads >0 bytes (sets pos=0, filled=buf.len(), initialized=buf.init_len())
- Readable -> Readable via consume()/consume_with() (pos increases but must remain <= filled)
- Readable -> Exhausted via consume()/consume_with() when pos reaches filled
- Exhausted -> Readable via fill_buf() (pos>=filled branch) after new read
- Readable/Exhausted -> Empty via discard_buffer() (pos=0, filled=0)
- Any -> Readable (more data appended) via read_more() (filled increases; initialized adjusted)
- Readable/Exhausted -> Readable (compaction) via backshift() (moves unread tail to front; sets pos=0, decreases filled)

**Evidence:** fields: pos: usize comment: "must always be <= filled"; fields: filled: usize comment: "filled ... indicate how many bytes at the start of buf are initialized"; fields: initialized: usize comment: "max number of bytes returned across all fill_buf calls" and used to "tell read_buf how many bytes of buf are initialized"; method buffer(): unsafe { ...get_unchecked(self.pos..self.filled).assume_init_ref() } with SAFETY comment relying on invariants; method fill_buf(): guard `if self.pos >= self.filled { debug_assert!(self.pos == self.filled); ... unsafe { buf.set_init(self.initialized); } ... self.pos = 0; self.filled = buf.len(); self.initialized = buf.init_len(); }`; method read_more(): `let old_init = self.initialized - self.filled;` (requires initialized >= filled) and `unsafe { buf.set_init(old_init); }` (requires old_init accurately describes init in tail slice); method consume(): `self.pos = cmp::min(self.pos + amt, self.filled);` runtime maintenance of pos<=filled; method consume_with(): uses `self.buffer().get(..amt)` as a runtime check before doing `self.pos += amt`

**Implementation:** Encode the coupled invariants in types/newtypes and/or typestate: e.g., store indices as `struct Pos(usize)` and `struct Filled(usize)` created only via constructors that enforce `pos<=filled<=cap`; split Buffer into `Buffer<Exhausted>` and `Buffer<Readable>` where `buffer()` exists only on Readable and `fill_buf(self, reader)->Result<Buffer<Readable>>` performs the state transition; similarly represent initialized region with a `struct InitPrefixLen(usize)` that is guaranteed `>= filled` and `<= cap`, preventing `initialized - filled` underflow and making `set_init` calls structurally safe.

---

### 29. StdinRaw validity protocol (Valid FD / EBADF-as-EOF)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-53`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: StdinRaw wraps an underlying stdio::Stdin handle but relies on runtime error inspection to decide whether the underlying file descriptor/handle is valid. If an EBADF occurs (e.g., stdin is closed/invalid in the process), reads are coerced into 'EOF-like' success values (Ok(0)/Ok(())) for most read APIs, while read_exact/read_buf_exact translate EBADF into READ_EXACT_EOF. This is an implicit state machine where the handle can effectively behave as if it is at EOF or broken, but the type system exposes only a single StdinRaw state and cannot prevent calling read methods on an invalid/closed underlying handle or force callers to handle/acknowledge the EBADF-as-EOF behavior.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdin_raw` function.
struct StdinRaw(stdio::Stdin);


// ... (other code) ...

    StderrRaw(stdio::Stderr::new())
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        handle_ebadf(self.0.read(buf), || Ok(0))
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        handle_ebadf(self.0.read_buf(buf), || Ok(()))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        handle_ebadf(self.0.read_vectored(bufs), || Ok(0))
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        handle_ebadf(self.0.read_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }
        handle_ebadf(self.0.read_buf_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_end(buf), || Ok(0))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_string(buf), || Ok(0))
    }
}

```

**Entity:** StdinRaw

**States:** ValidUnderlyingStdin, InvalidOrClosedUnderlyingFd (EBADF treated as EOF)

**Transitions:**
- ValidUnderlyingStdin -> InvalidOrClosedUnderlyingFd (EBADF treated as EOF) via external close/invalidation of stdin (observed at next read* call)

**Evidence:** struct StdinRaw(stdio::Stdin); (newtype wrapper does not encode validity/closedness); Comment on StdinRaw: "This handle is not synchronized or buffered in any fashion" and "Constructed via ... stdin_raw" indicates a special raw-handle mode with expectations not represented in types; Read::read: handle_ebadf(self.0.read(buf), || Ok(0)) coerces EBADF into Ok(0); Read::read_buf: handle_ebadf(self.0.read_buf(buf), || Ok(())) coerces EBADF into Ok(()); Read::read_vectored/read_to_end/read_to_string: handle_ebadf(..., || Ok(0)) similarly treats EBADF as non-error EOF; Read::read_exact: handle_ebadf(self.0.read_exact(buf), || Err(io::Error::READ_EXACT_EOF)) maps EBADF to READ_EXACT_EOF, showing different behavior depending on method

**Implementation:** Encode the "EBADF becomes EOF" behavior explicitly in the type/state: e.g., StdinRaw<S> with states Live and EofOnBadFd (or a wrapper type EofOnEbadf<R: Read>) so callers opt into the semantic conversion. Alternatively, have stdin_raw() return a distinct capability type (e.g., RawStdinWithEbadfEofSemantics) whose Read impl documents/enforces the conversion, while keeping a separate type for strict error propagation.

---

### 9. Cursor position validity & clamping protocol (In-bounds / Out-of-bounds)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-556`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Cursor carries a logical stream position `pos: u64` that is not tied to the underlying buffer’s length/capacity. Many operations implicitly assume different validity regimes: for reading/splitting/slice-writes, `pos` is clamped to `len` (treating out-of-bounds as EOF). For Vec-backed resizing writes, `pos` must be convertible to `usize` (platform-dependent) or an InvalidInput error occurs. The type system does not distinguish these regimes, so callers can freely set `pos` to any `u64`, and the API silently changes behavior (clamp-to-EOF vs error) depending on the operation and backing storage.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Cursor, impl Cursor < T > (6 methods), impl Cursor < T > (1 methods), impl Cursor < T > (1 methods), impl io :: Seek for Cursor < T > (3 methods), impl Read for Cursor < T > (8 methods), impl BufRead for Cursor < T > (2 methods), impl Write for Cursor < & mut [u8] > (6 methods), impl Write for Cursor < & mut Vec < u8 , A > > (6 methods), impl Write for Cursor < Vec < u8 , A > > (6 methods), impl Write for Cursor < Box < [u8] , A > > (6 methods), impl Write for Cursor < [u8 ; N] > (6 methods)

#[cfg(test)]
mod tests;

use crate::alloc::Allocator;
use crate::cmp;
use crate::io::prelude::*;
use crate::io::{self, BorrowedCursor, ErrorKind, IoSlice, IoSliceMut, SeekFrom};

/// A `Cursor` wraps an in-memory buffer and provides it with a
/// [`Seek`] implementation.
///
/// `Cursor`s are used with in-memory buffers, anything implementing
/// <code>[AsRef]<\[u8]></code>, to allow them to implement [`Read`] and/or [`Write`],
/// allowing these buffers to be used anywhere you might use a reader or writer
/// that does actual I/O.
///
/// The standard library implements some I/O traits on various types which
/// are commonly used as a buffer, like <code>Cursor<[Vec]\<u8>></code> and
/// <code>Cursor<[&\[u8\]][bytes]></code>.
///
/// # Examples
///
/// We may want to write bytes to a [`File`] in our production
/// code, but use an in-memory buffer in our tests. We can do this with
/// `Cursor`:
///
/// [bytes]: crate::slice "slice"
/// [`File`]: crate::fs::File
///
/// ```no_run
/// use std::io::prelude::*;
/// use std::io::{self, SeekFrom};
/// use std::fs::File;
///
/// // a library function we've written
/// fn write_ten_bytes_at_end<W: Write + Seek>(mut writer: W) -> io::Result<()> {
///     writer.seek(SeekFrom::End(-10))?;
///
///     for i in 0..10 {
///         writer.write(&[i])?;
///     }
///
///     // all went well
///     Ok(())
/// }
///
/// # fn foo() -> io::Result<()> {
/// // Here's some code that uses this library function.
/// //
/// // We might want to use a BufReader here for efficiency, but let's
/// // keep this example focused.
/// let mut file = File::create("foo.txt")?;
/// // First, we need to allocate 10 bytes to be able to write into.
/// file.set_len(10)?;
///
/// write_ten_bytes_at_end(&mut file)?;
/// # Ok(())
/// # }
///
/// // now let's write a test
/// #[test]
/// fn test_writes_bytes() {
///     // setting up a real File is much slower than an in-memory buffer,
///     // let's use a cursor instead
///     use std::io::Cursor;
///     let mut buff = Cursor::new(vec![0; 15]);
///
///     write_ten_bytes_at_end(&mut buff).unwrap();
///
///     assert_eq!(&buff.get_ref()[5..15], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    ///
    /// Cursor initial position is `0` even if underlying buffer (e.g., [`Vec`])
    /// is not empty. So writing to cursor starts with overwriting [`Vec`]
    /// content, not with appending to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }

    /// Consumes this cursor, returning the underlying value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let vec = buff.into_inner();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying value in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying value in this cursor.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying value as it may corrupt this cursor's position.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the current position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::io::prelude::*;
    /// use std::io::SeekFrom;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.seek(SeekFrom::Current(2)).unwrap();
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(buff.position(), 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.set_position(4);
    /// assert_eq!(buff.position(), 4);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    /// Splits the underlying slice at the cursor position and returns them.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(cursor_split)]
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.split(), ([].as_slice(), [1, 2, 3, 4, 5].as_slice()));
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.split(), ([1, 2].as_slice(), [3, 4, 5].as_slice()));
    ///
    /// buff.set_position(6);
    /// assert_eq!(buff.split(), ([1, 2, 3, 4, 5].as_slice(), [].as_slice()));
    /// ```
    #[unstable(feature = "cursor_split", issue = "86369")]
    pub fn split(&self) -> (&[u8], &[u8]) {
        let slice = self.inner.as_ref();
        let pos = self.pos.min(slice.len() as u64);
        slice.split_at(pos as usize)
    }
}

impl<T> Cursor<T>
where
    T: AsMut<[u8]>,
{
    /// Splits the underlying slice at the cursor position and returns them
    /// mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(cursor_split)]
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.split_mut(), ([].as_mut_slice(), [1, 2, 3, 4, 5].as_mut_slice()));
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.split_mut(), ([1, 2].as_mut_slice(), [3, 4, 5].as_mut_slice()));
    ///
    /// buff.set_position(6);
    /// assert_eq!(buff.split_mut(), ([1, 2, 3, 4, 5].as_mut_slice(), [].as_mut_slice()));
    /// ```
    #[unstable(feature = "cursor_split", issue = "86369")]
    pub fn split_mut(&mut self) -> (&mut [u8], &mut [u8]) {
        let slice = self.inner.as_mut();
        let pos = self.pos.min(slice.len() as u64);
        slice.split_at_mut(pos as usize)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Clone for Cursor<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Cursor { inner: self.inner.clone(), pos: self.pos }
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
        self.pos = other.pos;
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> io::Seek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::const_error!(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.inner.as_ref().len() as u64)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Read for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut Cursor::split(self).1, buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let prev_written = cursor.written();

        Read::read_buf(&mut Cursor::split(self).1, cursor.reborrow())?;

        self.pos += (cursor.written() - prev_written) as u64;

        Ok(())
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            nread += n;
            if n < buf.len() {
                break;
            }
        }
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        true
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let result = Read::read_exact(&mut Cursor::split(self).1, buf);

        match result {
            Ok(_) => self.pos += buf.len() as u64,
            // The only possible error condition is EOF, so place the cursor at "EOF"
            Err(_) => self.pos = self.inner.as_ref().len() as u64,
        }

        result
    }

    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let prev_written = cursor.written();

        let result = Read::read_buf_exact(&mut Cursor::split(self).1, cursor.reborrow());
        self.pos += (cursor.written() - prev_written) as u64;

        result
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let content = Cursor::split(self).1;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(content);
        self.pos += len as u64;

        Ok(len)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content =
            crate::str::from_utf8(Cursor::split(self).1).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        self.pos += len as u64;

        Ok(len)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> BufRead for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(Cursor::split(self).1)
    }
    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
    }
}

// Non-resizing write implementation
#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<usize> {
    let pos = cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

#[inline]
fn slice_write_vectored(
    pos_mut: &mut u64,
    slice: &mut [u8],
    bufs: &[IoSlice<'_>],
) -> io::Result<usize> {
    let mut nwritten = 0;
    for buf in bufs {
        let n = slice_write(pos_mut, slice, buf)?;
        nwritten += n;
        if n < buf.len() {
            break;
        }
    }
    Ok(nwritten)
}

#[inline]
fn slice_write_all(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<()> {
    let n = slice_write(pos_mut, slice, buf)?;
    if n < buf.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
}

#[inline]
fn slice_write_all_vectored(
    pos_mut: &mut u64,
    slice: &mut [u8],
    bufs: &[IoSlice<'_>],
) -> io::Result<()> {
    for buf in bufs {
        let n = slice_write(pos_mut, slice, buf)?;
        if n < buf.len() {
            return Err(io::Error::WRITE_ALL_EOF);
        }
    }
    Ok(())
}

/// Reserves the required space, and pads the vec with 0s if necessary.
fn reserve_and_pad<A: Allocator>(
    pos_mut: &mut u64,
    vec: &mut Vec<u8, A>,
    buf_len: usize,
) -> io::Result<usize> {
    let pos: usize = (*pos_mut).try_into().map_err(|_| {
        io::const_error!(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length",
        )
    })?;

    // For safety reasons, we don't want these numbers to overflow
    // otherwise our allocation won't be enough
    let desired_cap = pos.saturating_add(buf_len);
    if desired_cap > vec.capacity() {
        // We want our vec's total capacity
        // to have room for (pos+buf_len) bytes. Reserve allocates
        // based on additional elements from the length, so we need to
        // reserve the difference
        vec.reserve(desired_cap - vec.len());
    }
    // Pad if pos is above the current len.
    if pos > vec.len() {
        let diff = pos - vec.len();
        // Unfortunately, `resize()` would suffice but the optimiser does not
        // realise the `reserve` it does can be eliminated. So we do it manually
        // to eliminate that extra branch
        let spare = vec.spare_capacity_mut();
        debug_assert!(spare.len() >= diff);
        // Safety: we have allocated enough capacity for this.
        // And we are only writing, not reading
        unsafe {
            spare.get_unchecked_mut(..diff).fill(core::mem::MaybeUninit::new(0));
            vec.set_len(pos);
        }
    }

    Ok(pos)
}

/// Writes the slice to the vec without allocating.
///
/// # Safety
///
/// `vec` must have `buf.len()` spare capacity.
unsafe fn vec_write_all_unchecked<A>(pos: usize, vec: &mut Vec<u8, A>, buf: &[u8]) -> usize
where
    A: Allocator,
{
    debug_assert!(vec.capacity() >= pos + buf.len());
    unsafe { vec.as_mut_ptr().add(pos).copy_from(buf.as_ptr(), buf.len()) };
    pos + buf.len()
}

/// Resizing `write_all` implementation for [`Cursor`].
///
/// Cursor is allowed to have a pre-allocated and initialised
/// vector body, but with a position of 0. This means the [`Write`]
/// will overwrite the contents of the vec.
///
/// This also allows for the vec body to be empty, but with a position of N.
/// This means that [`Write`] will pad the vec with 0 initially,
/// before writing anything from that point
fn vec_write_all<A>(pos_mut: &mut u64, vec: &mut Vec<u8, A>, buf: &[u8]) -> io::Result<usize>
where
    A: Allocator,
{
    let buf_len = buf.len();
    let mut pos = reserve_and_pad(pos_mut, vec, buf_len)?;

    // Write the buf then progress the vec forward if necessary
    // Safety: we have ensured that the capacity is available
    // and that all bytes get written up to pos
    unsafe {
        pos = vec_write_all_unchecked(pos, vec, buf);
        if pos > vec.len() {
            vec.set_len(pos);
        }
    };

    // Bump us forward
    *pos_mut += buf_len as u64;
    Ok(buf_len)
}

/// Resizing `write_all_vectored` implementation for [`Cursor`].
///
/// Cursor is allowed to have a pre-allocated and initialised

// ... (truncated) ...
```

**Entity:** Cursor<T>

**States:** InBounds(pos <= len), OutOfBounds(pos > len), Invalid(pos overflows usize for Vec-backed writes)

**Transitions:**
- InBounds -> OutOfBounds via set_position() / seek(SeekFrom::Start|Current|End) producing pos > len
- OutOfBounds -> InBounds via set_position()/seek() setting pos back <= len
- Any -> Invalid via set_position()/seek() setting pos > usize::MAX (relevant to Vec writes using reserve_and_pad)

**Evidence:** field `pos: u64` in `pub struct Cursor<T> { inner: T, pos: u64 }`; method `set_position(&mut self, pos: u64)` sets `self.pos = pos` with no bounds/validation; Seek impl: `SeekFrom::Start(n) => { self.pos = n; return Ok(n); }` sets arbitrary u64 without checking against buffer length; split(): `let pos = self.pos.min(slice.len() as u64);` clamps pos to len before `split_at`; split_mut(): `let pos = self.pos.min(slice.len() as u64);` clamps pos to len before `split_at_mut`; Read::read(): reads from `Cursor::split(self).1` (post-clamp) then `self.pos += n as u64`; Read::read_exact(): on error sets `self.pos = self.inner.as_ref().len() as u64` (forces an EOF-like state); slice_write(): `let pos = cmp::min(*pos_mut, slice.len() as u64);` clamps before writing to a fixed slice; reserve_and_pad(): converts position with `let pos: usize = (*pos_mut).try_into().map_err(|_| ... "cursor position exceeds maximum possible vector length")?;` making large pos an error only in Vec-resizing path

**Implementation:** Introduce position wrapper types to make the regime explicit: e.g. `struct Cursor<T, P> { inner: T, pos: P }` where `P` is `UncheckedPos(u64)` (allows any), `ClampedPos(u64)` (guaranteed <= len for a particular borrow), or `VecPos(usize)` (guaranteed convertible). Provide APIs like `cursor.clamp()` returning a temporary view with `ClampedPos`, and for Vec-backed cursors require/produce a `VecPos` via checked conversion before resizing writes.

---

### 30. StderrRaw validity protocol (Valid FD / Closed-or-invalid FD with EBADF fallback)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: StderrRaw wraps an underlying stdio::Stderr and assumes it refers to a valid, writable OS stderr. However, writes/flushes are guarded via handle_ebadf(...), implying an implicit runtime state where the underlying handle may become invalid/closed (EBADF). In that EBADF state, operations do not behave like ordinary I/O: they fall back to success (e.g., reporting that all bytes were written). This creates an implicit state machine where method results depend on an external resource state that is not reflected in the type system, and callers cannot distinguish 'real write succeeded' from 'stderr was invalid/closed and we pretended success' via types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stderr_raw` function.
struct StderrRaw(stdio::Stderr);


// ... (other code) ...

    }
}

impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

```

**Entity:** StderrRaw

**States:** ValidUnderlyingStderr, InvalidOrClosedUnderlyingStderr (EBADF)

**Transitions:**
- ValidUnderlyingStderr -> InvalidOrClosedUnderlyingStderr (EBADF) via external close/invalidating of the underlying stderr fd/handle
- InvalidOrClosedUnderlyingStderr (EBADF) -> InvalidOrClosedUnderlyingStderr (EBADF) on subsequent write/flush calls (fallback continues)

**Evidence:** struct StderrRaw(stdio::Stderr); — wrapper around an external OS resource whose validity can change outside Rust's type system; comment: "This handle is not synchronized or buffered in any fashion" and "Constructed via the std::io::stdio::stderr_raw function" — indicates this is a low-level raw handle with fewer guarantees; write(): handle_ebadf(self.0.write(buf), || Ok(buf.len())) — explicitly treats EBADF as a special state and returns Ok(buf.len()); write_vectored(): handle_ebadf(self.0.write_vectored(bufs), total) — same EBADF-dependent behavior; flush(): handle_ebadf(self.0.flush(), || Ok(())) — flush also treats EBADF as a tolerated state; write_all()/write_all_vectored()/write_fmt(): all routed through handle_ebadf(..., || Ok(())) — consistent implicit 'invalid but treated as success' mode

**Implementation:** Introduce a typestate split between a 'Checked' handle and an 'Unchecked/raw' handle, or encode the EBADF-tolerant behavior as a distinct capability. For example, StderrRaw<Unchecked> created by stderr_raw(), with a method try_validate(self) -> Result<StderrRaw<Valid>, _>. Implement Write for StderrRaw<Valid> without EBADF-fallback, and separately provide an explicit wrapper like EbadfTolerant<W: Write> that documents/encodes the fallback semantics at the type level.

---

### 18. BufWriter internal-buffer lifecycle during copy loop (Buffered -> FlushRequired -> Buffered)

**Location**: `/tmp/io_test_crate/src/io/copy.rs:1-297`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The specialized copy loop relies on a runtime state machine driven by `self.capacity()` and `buf.spare_capacity_mut()`: when there is enough spare capacity, it reads directly into the BufWriter's internal buffer and then manually increases its length; when spare capacity drops below DEFAULT_BUF_SIZE, it must flush to make progress, and it also updates `init` to reflect that buffered bytes are initialized before flushing. Correctness depends on respecting this ordering (read into spare -> commit length; else mark init and flush) and on not attempting the direct-read path when capacity is too small. This control flow is enforced only by `if read_buf.capacity() >= DEFAULT_BUF_SIZE { ... } else { ... self.flush_buf()?; }` and a separate `init` variable, not by the type system.

**Evidence**:

```rust
use super::{BorrowedBuf, BufReader, BufWriter, DEFAULT_BUF_SIZE, Read, Result, Write};
use crate::alloc::Allocator;
use crate::cmp;
use crate::collections::VecDeque;
use crate::io::IoSlice;
use crate::mem::MaybeUninit;

#[cfg(test)]
mod tests;

/// Copies the entire contents of a reader into a writer.
///
/// This function will continuously read data from `reader` and then
/// write it into `writer` in a streaming fashion until `reader`
/// returns EOF.
///
/// On success, the total number of bytes that were copied from
/// `reader` to `writer` is returned.
///
/// If you want to copy the contents of one file to another and you’re
/// working with filesystem paths, see the [`fs::copy`] function.
///
/// [`fs::copy`]: crate::fs::copy
///
/// # Errors
///
/// This function will return an error immediately if any call to [`read`] or
/// [`write`] returns an error. All instances of [`ErrorKind::Interrupted`] are
/// handled by this function and the underlying operation is retried.
///
/// [`read`]: Read::read
/// [`write`]: Write::write
/// [`ErrorKind::Interrupted`]: crate::io::ErrorKind::Interrupted
///
/// # Examples
///
/// ```
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut reader: &[u8] = b"hello";
///     let mut writer: Vec<u8> = vec![];
///
///     io::copy(&mut reader, &mut writer)?;
///
///     assert_eq!(&b"hello"[..], &writer[..]);
///     Ok(())
/// }
/// ```
///
/// # Platform-specific behavior
///
/// On Linux (including Android), this function uses `copy_file_range(2)`,
/// `sendfile(2)` or `splice(2)` syscalls to move data directly between file
/// descriptors if possible.
///
/// Note that platform-specific behavior [may change in the future][changes].
///
/// [changes]: crate::io#platform-specific-behavior
#[stable(feature = "rust1", since = "1.0.0")]
pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "android"))] {
            crate::sys::kernel_copy::copy_spec(reader, writer)
        } else {
            generic_copy(reader, writer)
        }
    }
}

/// The userspace read-write-loop implementation of `io::copy` that is used when
/// OS-specific specializations for copy offloading are not available or not applicable.
pub(crate) fn generic_copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    let read_buf = BufferedReaderSpec::buffer_size(reader);
    let write_buf = BufferedWriterSpec::buffer_size(writer);

    if read_buf >= DEFAULT_BUF_SIZE && read_buf >= write_buf {
        return BufferedReaderSpec::copy_to(reader, writer);
    }

    BufferedWriterSpec::copy_from(writer, reader)
}

/// Specialization of the read-write loop that reuses the internal
/// buffer of a BufReader. If there's no buffer then the writer side
/// should be used instead.
trait BufferedReaderSpec {
    fn buffer_size(&self) -> usize;

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64>;
}

impl<T> BufferedReaderSpec for T
where
    Self: Read,
    T: ?Sized,
{
    #[inline]
    default fn buffer_size(&self) -> usize {
        0
    }

    default fn copy_to(&mut self, _to: &mut (impl Write + ?Sized)) -> Result<u64> {
        unreachable!("only called from specializations")
    }
}

impl BufferedReaderSpec for &[u8] {
    fn buffer_size(&self) -> usize {
        // prefer this specialization since the source "buffer" is all we'll ever need,
        // even if it's small
        usize::MAX
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        to.write_all(self)?;
        *self = &self[len..];
        Ok(len as u64)
    }
}

impl<A: Allocator> BufferedReaderSpec for VecDeque<u8, A> {
    fn buffer_size(&self) -> usize {
        // prefer this specialization since the source "buffer" is all we'll ever need,
        // even if it's small
        usize::MAX
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        let (front, back) = self.as_slices();
        let bufs = &mut [IoSlice::new(front), IoSlice::new(back)];
        to.write_all_vectored(bufs)?;
        self.clear();
        Ok(len as u64)
    }
}

impl<I> BufferedReaderSpec for BufReader<I>
where
    Self: Read,
    I: ?Sized,
{
    fn buffer_size(&self) -> usize {
        self.capacity()
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let mut len = 0;

        loop {
            // Hack: this relies on `impl Read for BufReader` always calling fill_buf
            // if the buffer is empty, even for empty slices.
            // It can't be called directly here since specialization prevents us
            // from adding I: Read
            match self.read(&mut []) {
                Ok(_) => {}
                Err(e) if e.is_interrupted() => continue,
                Err(e) => return Err(e),
            }
            let buf = self.buffer();
            if self.buffer().len() == 0 {
                return Ok(len);
            }

            // In case the writer side is a BufWriter then its write_all
            // implements an optimization that passes through large
            // buffers to the underlying writer. That code path is #[cold]
            // but we're still avoiding redundant memcopies when doing
            // a copy between buffered inputs and outputs.
            to.write_all(buf)?;
            len += buf.len() as u64;
            self.discard_buffer();
        }
    }
}

/// Specialization of the read-write loop that either uses a stack buffer
/// or reuses the internal buffer of a BufWriter
trait BufferedWriterSpec: Write {
    fn buffer_size(&self) -> usize;

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64>;
}

impl<W: Write + ?Sized> BufferedWriterSpec for W {
    #[inline]
    default fn buffer_size(&self) -> usize {
        0
    }

    default fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        stack_buffer_copy(reader, self)
    }
}

impl<I: Write + ?Sized> BufferedWriterSpec for BufWriter<I> {
    fn buffer_size(&self) -> usize {
        self.capacity()
    }

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        if self.capacity() < DEFAULT_BUF_SIZE {
            return stack_buffer_copy(reader, self);
        }

        let mut len = 0;
        let mut init = 0;

        loop {
            let buf = self.buffer_mut();
            let mut read_buf: BorrowedBuf<'_> = buf.spare_capacity_mut().into();

            unsafe {
                // SAFETY: init is either 0 or the init_len from the previous iteration.
                read_buf.set_init(init);
            }

            if read_buf.capacity() >= DEFAULT_BUF_SIZE {
                let mut cursor = read_buf.unfilled();
                match reader.read_buf(cursor.reborrow()) {
                    Ok(()) => {
                        let bytes_read = cursor.written();

                        if bytes_read == 0 {
                            return Ok(len);
                        }

                        init = read_buf.init_len() - bytes_read;
                        len += bytes_read as u64;

                        // SAFETY: BorrowedBuf guarantees all of its filled bytes are init
                        unsafe { buf.set_len(buf.len() + bytes_read) };

                        // Read again if the buffer still has enough capacity, as BufWriter itself would do
                        // This will occur if the reader returns short reads
                    }
                    Err(ref e) if e.is_interrupted() => {}
                    Err(e) => return Err(e),
                }
            } else {
                // All the bytes that were already in the buffer are initialized,
                // treat them as such when the buffer is flushed.
                init += buf.len();

                self.flush_buf()?;
            }
        }
    }
}

impl BufferedWriterSpec for Vec<u8> {
    fn buffer_size(&self) -> usize {
        cmp::max(DEFAULT_BUF_SIZE, self.capacity() - self.len())
    }

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        reader.read_to_end(self).map(|bytes| u64::try_from(bytes).expect("usize overflowed u64"))
    }
}

pub fn stack_buffer_copy<R: Read + ?Sized, W: Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); DEFAULT_BUF_SIZE];
    let mut buf: BorrowedBuf<'_> = buf.into();

    let mut len = 0;

    loop {
        match reader.read_buf(buf.unfilled()) {
            Ok(()) => {}
            Err(e) if e.is_interrupted() => continue,
            Err(e) => return Err(e),
        };

        if buf.filled().is_empty() {
            break;
        }

        len += buf.filled().len() as u64;
        writer.write_all(buf.filled())?;
        buf.clear();
    }

    Ok(len)
}

```

**Entity:** BufWriter<I> specialization in BufferedWriterSpec::copy_from

**States:** Buffered(has spare capacity for direct reads), FlushRequired(insufficient spare capacity; must flush), EOF(done)

**Transitions:**
- Buffered -> Buffered via reader.read_buf(...) then unsafe buf.set_len(...) (when spare capacity remains >= DEFAULT_BUF_SIZE)
- Buffered -> FlushRequired when `read_buf.capacity() < DEFAULT_BUF_SIZE`
- FlushRequired -> Buffered via `self.flush_buf()?`
- Buffered -> EOF via `if bytes_read == 0 { return Ok(len); }`

**Evidence:** BufWriter<I>::copy_from: `if self.capacity() < DEFAULT_BUF_SIZE { return stack_buffer_copy(reader, self); }` (precondition gate for using this state machine at all); BufWriter<I>::copy_from loop: `let buf = self.buffer_mut(); let mut read_buf: BorrowedBuf<'_> = buf.spare_capacity_mut().into();` (operates on internal buffer state); BufWriter<I>::copy_from: `if read_buf.capacity() >= DEFAULT_BUF_SIZE { ... } else { init += buf.len(); self.flush_buf()?; }` (explicit state-based branch requiring flush before continuing); BufWriter<I>::copy_from: `unsafe { buf.set_len(buf.len() + bytes_read) };` (manual commit step dependent on having just read into spare capacity); BufWriter<I>::copy_from: `if bytes_read == 0 { return Ok(len); }` (EOF terminal transition)

**Implementation:** Factor the loop into two internal states represented by distinct helper types, e.g. `struct CanReadIntoSpare<'a>(&'a mut BufWriter<I>);` vs `struct MustFlush<'a>(&'a mut BufWriter<I>);` where only `CanReadIntoSpare` exposes `read_into_spare(reader) -> Result<CanReadIntoSpare|MustFlush|Eof>`, and only `MustFlush` exposes `flush() -> Result<CanReadIntoSpare>`. This makes the 'must flush before reading more' ordering impossible to violate in the specialized implementation and encapsulates `init` handling.

---

### 35. Chain read progression state machine (ReadingFirst / ReadingSecond)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-198`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Chain encodes a two-phase reader: it must read from `first` until `first` is considered exhausted, then it permanently switches to reading from `second`. This progression is tracked at runtime by `done_first: bool` and is updated based on observed read results (0 bytes / empty buffer conditions). The type system does not distinguish the two phases, so methods must branch on `done_first` and callers can obtain `&mut T`/`&mut U` and mutate underlying I/O state in ways that can desynchronize Chain's internal phase tracking.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

/// [`chain`]: Read::chain
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Chain<T, U> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T, U> Chain<T, U> {
    /// Consumes the `Chain`, returning the wrapped readers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut foo_file = File::open("foo.txt")?;
    ///     let mut bar_file = File::open("bar.txt")?;
    ///
    ///     let chain = foo_file.chain(bar_file);
    ///     let (foo_file, bar_file) = chain.into_inner();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "more_io_inner_methods", since = "1.20.0")]
    pub fn into_inner(self) -> (T, U) {
        (self.first, self.second)
    }

    /// Gets references to the underlying readers in this `Chain`.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying readers as doing so may corrupt the internal state of this
    /// `Chain`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut foo_file = File::open("foo.txt")?;
    ///     let mut bar_file = File::open("bar.txt")?;
    ///
    ///     let chain = foo_file.chain(bar_file);
    ///     let (foo_file, bar_file) = chain.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "more_io_inner_methods", since = "1.20.0")]
    pub fn get_ref(&self) -> (&T, &U) {
        (&self.first, &self.second)
    }

    /// Gets mutable references to the underlying readers in this `Chain`.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying readers as doing so may corrupt the internal state of this
    /// `Chain`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use std::io::prelude::*;
    /// use std::fs::File;
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut foo_file = File::open("foo.txt")?;
    ///     let mut bar_file = File::open("bar.txt")?;
    ///
    ///     let mut chain = foo_file.chain(bar_file);
    ///     let (foo_file, bar_file) = chain.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "more_io_inner_methods", since = "1.20.0")]
    pub fn get_mut(&mut self) -> (&mut T, &mut U) {
        (&mut self.first, &mut self.second)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Read, U: Read> Read for Chain<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.done_first {
            match self.first.read(buf)? {
                0 if !buf.is_empty() => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> {
        if !self.done_first {
            match self.first.read_vectored(bufs)? {
                0 if bufs.iter().any(|b| !b.is_empty()) => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.first.is_read_vectored() || self.second.is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        if !self.done_first {
            read += self.first.read_to_end(buf)?;
            self.done_first = true;
        }
        read += self.second.read_to_end(buf)?;
        Ok(read)
    }

    // We don't override `read_to_string` here because an UTF-8 sequence could
    // be split between the two parts of the chain

    fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }

        if !self.done_first {
            let old_len = buf.written();
            self.first.read_buf(buf.reborrow())?;

            if buf.written() != old_len {
                return Ok(());
            } else {
                self.done_first = true;
            }
        }
        self.second.read_buf(buf)
    }
}

#[stable(feature = "chain_bufread", since = "1.9.0")]
impl<T: BufRead, U: BufRead> BufRead for Chain<T, U> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if !self.done_first {
            match self.first.fill_buf()? {
                buf if buf.is_empty() => self.done_first = true,
                buf => return Ok(buf),
            }
        }
        self.second.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        if !self.done_first { self.first.consume(amt) } else { self.second.consume(amt) }
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize> {
        let mut read = 0;
        if !self.done_first {
            let n = self.first.read_until(byte, buf)?;
            read += n;

            match buf.last() {
                Some(b) if *b == byte && n != 0 => return Ok(read),
                _ => self.done_first = true,
            }
        }
        read += self.second.read_until(byte, buf)?;
        Ok(read)
    }

    // We don't override `read_line` here because an UTF-8 sequence could be
    // split between the two parts of the chain
}

impl<T, U> SizeHint for Chain<T, U> {
    #[inline]
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(&self.first) + SizeHint::lower_bound(&self.second)
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        match (SizeHint::upper_bound(&self.first), SizeHint::upper_bound(&self.second)) {
            (Some(first), Some(second)) => first.checked_add(second),
            _ => None,
        }
    }
}

```

**Entity:** Chain<T, U>

**States:** ReadingFirst, ReadingSecond

**Transitions:**
- ReadingFirst -> ReadingSecond via Read::read() when first.read(buf) returns 0 while buf is non-empty
- ReadingFirst -> ReadingSecond via Read::read_vectored() when first.read_vectored(bufs) returns 0 while any IoSliceMut is non-empty
- ReadingFirst -> ReadingSecond via Read::read_to_end() after delegating to first.read_to_end(buf) (unconditionally sets done_first = true)
- ReadingFirst -> ReadingSecond via Read::read_buf() when first.read_buf() writes 0 bytes (buf.written unchanged)
- ReadingFirst -> ReadingSecond via BufRead::fill_buf() when first.fill_buf() returns an empty slice
- ReadingFirst -> ReadingSecond via BufRead::read_until() when the delimiter was not found in first's output (buf.last() != byte or n == 0)

**Evidence:** field `done_first: bool` in `pub struct Chain<T, U>` tracks which underlying reader is active; `Read for Chain`: `if !self.done_first { ... self.done_first = true ... }` branches in read/read_vectored/read_to_end/read_buf; `BufRead for Chain`: `if !self.done_first { ... self.done_first = true ... }` branches in fill_buf/consume/read_until; read(): `0 if !buf.is_empty() => self.done_first = true` uses runtime result to transition phases; read_vectored(): `0 if bufs.iter().any(|b| !b.is_empty()) => self.done_first = true`; fill_buf(): `buf if buf.is_empty() => self.done_first = true`; comment on get_ref/get_mut: "Care should be taken to avoid modifying the internal I/O state ... may corrupt the internal state of this Chain." indicates a relied-on protocol not enforced by types; methods `get_ref()` and especially `get_mut()` expose `(&mut T, &mut U)` allowing external mutation that can violate the assumed phase/exhaustion relationship implied by `done_first`

**Implementation:** Represent the phase at the type level: `struct Chain<T,U,S>{ first:T, second:U, _s:PhantomData<S> }` with `ReadingFirst` and `ReadingSecond` marker types. Provide a transition method (e.g., `advance(self) -> Chain<T,U,ReadingSecond>`) once exhaustion is observed; expose reading methods only for the appropriate state or return an enum `EitherRead<Chain<...,ReadingFirst>, Chain<...,ReadingSecond>>` from read-like operations. Additionally, restrict/avoid `get_mut()` or only provide state-specific accessors (e.g., only `first_mut()` in `ReadingFirst`, only `second_mut()` in `ReadingSecond`) to prevent desynchronizing mutations.

---

### 33. IoSliceMut cursor/remaining-range protocol (Unadvanced -> PartiallyAdvanced -> FullyAdvanced)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-153`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: IoSliceMut has an implicit internal cursor over an underlying &'a mut [u8]. Methods like advance() and advance_slices() mutate this cursor by shrinking the visible range. Correctness relies on a runtime precondition that callers never advance past the remaining length; violating this panics. The type system does not encode how many bytes remain (or that an IoSliceMut is non-empty), so code can compile that may panic at runtime when advancing too far.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

/// Windows.
#[stable(feature = "iovec", since = "1.36.0")]
#[repr(transparent)]
pub struct IoSliceMut<'a>(sys::io::IoSliceMut<'a>);

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Send for IoSliceMut<'a> {}

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Sync for IoSliceMut<'a> {}


// ... (other code) ...

    }
}

impl<'a> IoSliceMut<'a> {
    /// Creates a new `IoSliceMut` wrapping a byte slice.
    ///
    /// # Panics
    ///
    /// Panics on Windows if the slice is larger than 4GB.
    #[stable(feature = "iovec", since = "1.36.0")]
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> IoSliceMut<'a> {
        IoSliceMut(sys::io::IoSliceMut::new(buf))
    }

    /// Advance the internal cursor of the slice.
    ///
    /// Also see [`IoSliceMut::advance_slices`] to advance the cursors of
    /// multiple buffers.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSliceMut;
    /// use std::ops::Deref;
    ///
    /// let mut data = [1; 8];
    /// let mut buf = IoSliceMut::new(&mut data);
    ///
    /// // Mark 3 bytes as read.
    /// buf.advance(3);
    /// assert_eq!(buf.deref(), [1; 5].as_ref());
    /// ```
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0.advance(n)
    }

    /// Advance a slice of slices.
    ///
    /// Shrinks the slice to remove any `IoSliceMut`s that are fully advanced over.
    /// If the cursor ends up in the middle of an `IoSliceMut`, it is modified
    /// to start at that cursor.
    ///
    /// For example, if we have a slice of two 8-byte `IoSliceMut`s, and we advance by 10 bytes,
    /// the result will only include the second `IoSliceMut`, advanced by 2 bytes.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSliceMut;
    /// use std::ops::Deref;
    ///
    /// let mut buf1 = [1; 8];
    /// let mut buf2 = [2; 16];
    /// let mut buf3 = [3; 8];
    /// let mut bufs = &mut [
    ///     IoSliceMut::new(&mut buf1),
    ///     IoSliceMut::new(&mut buf2),
    ///     IoSliceMut::new(&mut buf3),
    /// ][..];
    ///
    /// // Mark 10 bytes as read.
    /// IoSliceMut::advance_slices(&mut bufs, 10);
    /// assert_eq!(bufs[0].deref(), [2; 14].as_ref());
    /// assert_eq!(bufs[1].deref(), [3; 8].as_ref());
    /// ```
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance_slices(bufs: &mut &mut [IoSliceMut<'a>], n: usize) {
        // Number of buffers to remove.
        let mut remove = 0;
        // Remaining length before reaching n.
        let mut left = n;
        for buf in bufs.iter() {
            if let Some(remainder) = left.checked_sub(buf.len()) {
                left = remainder;
                remove += 1;
            } else {
                break;
            }
        }

        *bufs = &mut take(bufs)[remove..];
        if bufs.is_empty() {
            assert!(left == 0, "advancing io slices beyond their length");
        } else {
            bufs[0].advance(left);
        }
    }

    /// Get the underlying bytes as a mutable slice with the original lifetime.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(io_slice_as_bytes)]
    /// use std::io::IoSliceMut;
    ///
    /// let mut data = *b"abcdef";
    /// let io_slice = IoSliceMut::new(&mut data);
    /// io_slice.into_slice()[0] = b'A';
    ///
    /// assert_eq!(&data, b"Abcdef");
    /// ```
    #[unstable(feature = "io_slice_as_bytes", issue = "132818")]
    pub const fn into_slice(self) -> &'a mut [u8] {
        self.0.into_slice()
    }
}

#[stable(feature = "iovec", since = "1.36.0")]
impl<'a> Deref for IoSliceMut<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[stable(feature = "iovec", since = "1.36.0")]
impl<'a> DerefMut for IoSliceMut<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}

```

**Entity:** IoSliceMut<'a>

**States:** Unadvanced, PartiallyAdvanced, FullyAdvanced (len == 0)

**Transitions:**
- Unadvanced -> PartiallyAdvanced via IoSliceMut::advance(n) with 0 < n < len
- Unadvanced -> FullyAdvanced via IoSliceMut::advance(n) with n == len
- PartiallyAdvanced -> PartiallyAdvanced via IoSliceMut::advance(n) with n < remaining
- PartiallyAdvanced -> FullyAdvanced via IoSliceMut::advance(n) with n == remaining
- ([IoSliceMut], any cursor positions) -> (shorter slice + advanced first element) via IoSliceMut::advance_slices(bufs, n)

**Evidence:** method IoSliceMut::advance(&mut self, n: usize): doc says "Advance the internal cursor of the slice" and "Panics when trying to advance beyond the end of the slice"; forwards to self.0.advance(n); method IoSliceMut::advance_slices(bufs: &mut &mut [IoSliceMut<'a>], n: usize): doc says it "Shrinks the slice to remove any IoSliceMut's that are fully advanced over" and "Panics when trying to advance beyond the end of the slices"; advance_slices runtime enforcement: `assert!(left == 0, "advancing io slices beyond their length");` when bufs becomes empty after removing fully-consumed buffers; advance_slices state mutation of the 'cursor' is performed by `bufs[0].advance(left)` after slicing `*bufs = &mut take(bufs)[remove..];`

**Implementation:** Introduce a checked advance amount type, e.g. `struct AdvanceBy(usize);` constructed only via `AdvanceBy::new(n, remaining_len) -> Option<AdvanceBy>` (or a `TryFrom<(usize, usize)>`). Then change `advance(&mut self, by: AdvanceBy)` / `advance_slices(..., by: AdvanceBy)` (or provide parallel `try_advance` APIs) so "cannot advance beyond remaining" is enforced by construction rather than panicking at the call site.

---

### 19. Error representation protocol (OS / Simple kind / Static message / Custom boxed error)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-408`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The module encodes multiple distinct runtime representations of an I/O error in the private enum `ErrorData<C>`: an OS error code (`Os`), a bare `ErrorKind` (`Simple`), a `'static` pre-baked message (`SimpleMessage`), or an owned boxed error (`Custom`). The choice of representation determines important behavioral/semantic properties (e.g., whether allocation occurred, whether there is an underlying OS code available, whether a message is guaranteed to be `'static`, and whether an underlying error/source exists). These representation-dependent capabilities are not exposed in the type system; they are instead an internal protocol hidden behind `Error { repr: Repr }` and conversions/macros that must choose the correct variant at construction time (e.g., OOM paths must avoid allocating).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Error, impl Error (0 methods), impl From < alloc :: ffi :: NulError > for Error (1 methods), impl From < alloc :: collections :: TryReserveError > for Error (1 methods), impl ErrorKind (1 methods), impl From < ErrorKind > for Error (1 methods), impl Error (13 methods), impl error :: Error for Error (3 methods); struct SimpleMessage; struct Custom; enum ErrorData; enum ErrorKind

#[cfg(test)]
mod tests;

#[cfg(all(target_pointer_width = "64", not(target_os = "uefi")))]
mod repr_bitpacked;
#[cfg(all(target_pointer_width = "64", not(target_os = "uefi")))]
use repr_bitpacked::Repr;

#[cfg(any(not(target_pointer_width = "64"), target_os = "uefi"))]
mod repr_unpacked;
#[cfg(any(not(target_pointer_width = "64"), target_os = "uefi"))]
use repr_unpacked::Repr;

use crate::{error, fmt, result, sys};

/// A specialized [`Result`] type for I/O operations.
///
/// This type is broadly used across [`std::io`] for any operation which may
/// produce an error.
///
/// This typedef is generally used to avoid writing out [`io::Error`] directly and
/// is otherwise a direct mapping to [`Result`].
///
/// While usual Rust style is to import types directly, aliases of [`Result`]
/// often are not, to make it easier to distinguish between them. [`Result`] is
/// generally assumed to be [`std::result::Result`][`Result`], and so users of this alias
/// will generally use `io::Result` instead of shadowing the [prelude]'s import
/// of [`std::result::Result`][`Result`].
///
/// [`std::io`]: crate::io
/// [`io::Error`]: Error
/// [`Result`]: crate::result::Result
/// [prelude]: crate::prelude
///
/// # Examples
///
/// A convenience function that bubbles an `io::Result` to its caller:
///
/// ```
/// use std::io;
///
/// fn get_string() -> io::Result<String> {
///     let mut buffer = String::new();
///
///     io::stdin().read_line(&mut buffer)?;
///
///     Ok(buffer)
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(bootstrap), doc(search_unbox))]
pub type Result<T> = result::Result<T, Error>;

/// The error type for I/O operations of the [`Read`], [`Write`], [`Seek`], and
/// associated traits.
///
/// Errors mostly originate from the underlying OS, but custom instances of
/// `Error` can be created with crafted error messages and a particular value of
/// [`ErrorKind`].
///
/// [`Read`]: crate::io::Read
/// [`Write`]: crate::io::Write
/// [`Seek`]: crate::io::Seek
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Error {
    repr: Repr,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

/// Common errors constants for use in std
#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    pub(crate) const READ_EXACT_EOF: Self =
        const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer");

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_error!(
        ErrorKind::NotFound,
        "the number of hardware threads is not known for the target platform",
    );

    pub(crate) const UNSUPPORTED_PLATFORM: Self =
        const_error!(ErrorKind::Unsupported, "operation not supported on this platform");

    pub(crate) const WRITE_ALL_EOF: Self =
        const_error!(ErrorKind::WriteZero, "failed to write whole buffer");

    pub(crate) const ZERO_TIMEOUT: Self =
        const_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout");
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<alloc::ffi::NulError> for Error {
    /// Converts a [`alloc::ffi::NulError`] into a [`Error`].
    fn from(_: alloc::ffi::NulError) -> Error {
        const_error!(ErrorKind::InvalidInput, "data provided contains a nul byte")
    }
}

#[stable(feature = "io_error_from_try_reserve", since = "1.78.0")]
impl From<alloc::collections::TryReserveError> for Error {
    /// Converts `TryReserveError` to an error with [`ErrorKind::OutOfMemory`].
    ///
    /// `TryReserveError` won't be available as the error `source()`,
    /// but this may change in the future.
    fn from(_: alloc::collections::TryReserveError) -> Error {
        // ErrorData::Custom allocates, which isn't great for handling OOM errors.
        ErrorKind::OutOfMemory.into()
    }
}

// Only derive debug in tests, to make sure it
// doesn't accidentally get printed.
#[cfg_attr(test, derive(Debug))]
enum ErrorData<C> {
    Os(RawOsError),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
    Custom(C),
}

/// The type of raw OS error codes returned by [`Error::raw_os_error`].
///
/// This is an [`i32`] on all currently supported platforms, but platforms
/// added in the future (such as UEFI) may use a different primitive type like
/// [`usize`]. Use `as`or [`into`] conversions where applicable to ensure maximum
/// portability.
///
/// [`into`]: Into::into
#[unstable(feature = "raw_os_error_ty", issue = "107792")]
pub type RawOsError = sys::RawOsError;

// `#[repr(align(4))]` is probably redundant, it should have that value or
// higher already. We include it just because repr_bitpacked.rs's encoding
// requires an alignment >= 4 (note that `#[repr(align)]` will not reduce the
// alignment required by the struct, only increase it).
//
// If we add more variants to ErrorData, this can be increased to 8, but it
// should probably be behind `#[cfg_attr(target_pointer_width = "64", ...)]` or
// whatever cfg we're using to enable the `repr_bitpacked` code, since only the
// that version needs the alignment, and 8 is higher than the alignment we'll
// have on 32 bit platforms.
//
// (For the sake of being explicit: the alignment requirement here only matters
// if `error/repr_bitpacked.rs` is in use — for the unpacked repr it doesn't
// matter at all)
#[doc(hidden)]
#[unstable(feature = "io_const_error_internals", issue = "none")]
#[repr(align(4))]
#[derive(Debug)]
pub struct SimpleMessage {
    pub kind: ErrorKind,
    pub message: &'static str,
}

/// Creates a new I/O error from a known kind of error and a string literal.
///
/// Contrary to [`Error::new`], this macro does not allocate and can be used in
/// `const` contexts.
///
/// # Example
/// ```
/// #![feature(io_const_error)]
/// use std::io::{const_error, Error, ErrorKind};
///
/// const FAIL: Error = const_error!(ErrorKind::Unsupported, "tried something that never works");
///
/// fn not_here() -> Result<(), Error> {
///     Err(FAIL)
/// }
/// ```
#[rustc_macro_transparency = "semitransparent"]
#[unstable(feature = "io_const_error", issue = "133448")]
#[allow_internal_unstable(hint_must_use, io_const_error_internals)]
pub macro const_error($kind:expr, $message:expr $(,)?) {
    $crate::hint::must_use($crate::io::Error::from_static_message(
        const { &$crate::io::SimpleMessage { kind: $kind, message: $message } },
    ))
}

// As with `SimpleMessage`: `#[repr(align(4))]` here is just because
// repr_bitpacked's encoding requires it. In practice it almost certainly be
// already be this high or higher.
#[derive(Debug)]
#[repr(align(4))]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

/// A list specifying general categories of I/O error.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`io::Error`] type.
///
/// [`io::Error`]: Error
///
/// # Handling errors and matching on `ErrorKind`
///
/// In application code, use `match` for the `ErrorKind` values you are
/// expecting; use `_` to match "all other errors".
///
/// In comprehensive and thorough tests that want to verify that a test doesn't
/// return any known incorrect error kind, you may want to cut-and-paste the
/// current full list of errors from here into your test code, and then match
/// `_` as the correct case. This seems counterintuitive, but it will make your
/// tests more robust. In particular, if you want to verify that your code does
/// produce an unrecognized error kind, the robust solution is to check for all
/// the recognized error kinds and fail in those cases.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[stable(feature = "rust1", since = "1.0.0")]
#[allow(deprecated)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An entity was not found, often a file.
    #[stable(feature = "rust1", since = "1.0.0")]
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    #[stable(feature = "rust1", since = "1.0.0")]
    PermissionDenied,
    /// The connection was refused by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionRefused,
    /// The connection was reset by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionReset,
    /// The remote host is not reachable.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    HostUnreachable,
    /// The network containing the remote host is not reachable.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NetworkUnreachable,
    /// The connection was aborted (terminated) by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    #[stable(feature = "rust1", since = "1.0.0")]
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    #[stable(feature = "rust1", since = "1.0.0")]
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    #[stable(feature = "rust1", since = "1.0.0")]
    AddrNotAvailable,
    /// The system's networking is down.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NetworkDown,
    /// The operation failed because a pipe was closed.
    #[stable(feature = "rust1", since = "1.0.0")]
    BrokenPipe,
    /// An entity already exists, often a file.
    #[stable(feature = "rust1", since = "1.0.0")]
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    #[stable(feature = "rust1", since = "1.0.0")]
    WouldBlock,
    /// A filesystem object is, unexpectedly, not a directory.
    ///
    /// For example, a filesystem path was specified where one of the intermediate directory
    /// components was, in fact, a plain file.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NotADirectory,
    /// The filesystem object is, unexpectedly, a directory.
    ///
    /// A directory was specified when a non-directory was expected.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    IsADirectory,
    /// A non-empty directory was specified where an empty directory was expected.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    DirectoryNotEmpty,
    /// The filesystem or storage medium is read-only, but a write operation was attempted.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ReadOnlyFilesystem,
    /// Loop in the filesystem or IO subsystem; often, too many levels of symbolic links.
    ///
    /// There was a loop (or excessively long chain) resolving a filesystem object
    /// or file IO object.
    ///
    /// On Unix this is usually the result of a symbolic link loop; or, of exceeding the
    /// system-specific limit on the depth of symlink traversal.
    #[unstable(feature = "io_error_more", issue = "86442")]
    FilesystemLoop,
    /// Stale network file handle.
    ///
    /// With some network filesystems, notably NFS, an open file (or directory) can be invalidated
    /// by problems with the network or server.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    StaleNetworkFileHandle,
    /// A parameter was incorrect.
    #[stable(feature = "rust1", since = "1.0.0")]
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    #[stable(feature = "io_invalid_data", since = "1.2.0")]
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    #[stable(feature = "rust1", since = "1.0.0")]
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    #[stable(feature = "rust1", since = "1.0.0")]
    WriteZero,
    /// The underlying storage (typically, a filesystem) is full.
    ///
    /// This does not include out of quota errors.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    StorageFull,
    /// Seek on unseekable file.
    ///
    /// Seeking was attempted on an open file handle which is not suitable for seeking - for
    /// example, on Unix, a named pipe opened with `File::open`.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NotSeekable,
    /// Filesystem quota or some other kind of quota was exceeded.
    #[stable(feature = "io_error_quota_exceeded", since = "1.85.0")]
    QuotaExceeded,
    /// File larger than allowed or supported.
    ///
    /// This might arise from a hard limit of the underlying filesystem or file access API, or from
    /// an administratively imposed resource limitation.  Simple disk full, and out of quota, have
    /// their own errors.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    FileTooLarge,
    /// Resource is busy.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ResourceBusy,
    /// Executable file is busy.
    ///
    /// An attempt was made to write to a file which is also in use as a running program.  (Not all
    /// operating systems detect this situation.)
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ExecutableFileBusy,
    /// Deadlock (avoided).
    ///
    /// A file locking operation would result in deadlock.  This situation is typically detected, if
    /// at all, on a best-effort basis.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    Deadlock,
    /// Cross-device or cross-filesystem (hard) link or rename.
    #[stable(feature = "io_error_crosses_devices", since = "1.85.0")]
    CrossesDevices,
    /// Too many (hard) links to the same filesystem object.
    ///
    /// The filesystem does not support making so many hardlinks to the same file.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    TooManyLinks,
    /// A filename was invalid.
    ///
    /// This error can also occur if a length limit for a name was exceeded.
    #[stable(feature = "io_error_invalid_filename", since = "1.87.0")]
    InvalidFilename,
    /// Program argument list too long.
    ///
    /// When trying to run an external program, a system or process limit on the size of the
    /// arguments would have been exceeded.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ArgumentListTooLong,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    #[stable(feature = "rust1", since = "1.0.0")]
    Interrupted,

    /// This operation is unsupported on this platform.
    ///
    /// This means that the operation can never succeed.
    #[stable(feature = "unsupported_error", since = "1.53.0")]
    Unsupported,

    // ErrorKinds which are primarily categorisations for OS error
    // codes should be added above.
    //
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particul
// ... (truncated) ...
```

**Entity:** ErrorData<C>

**States:** Os, Simple, SimpleMessage, Custom

**Transitions:**
- External error -> (likely) Simple via `ErrorKind::OutOfMemory.into()` (avoids Custom allocation)
- const_error!(kind, message) -> SimpleMessage via `Error::from_static_message(...)`
- alloc::ffi::NulError -> const_error!(InvalidInput, ...) (likely SimpleMessage)
- TryReserveError -> ErrorKind::OutOfMemory.into() (forces non-allocating representation)

**Evidence:** enum ErrorData<C> { Os(RawOsError), Simple(ErrorKind), SimpleMessage(&'static SimpleMessage), Custom(C) } encodes multiple runtime states/representations; impl From<alloc::collections::TryReserveError> for Error: comment + code `// ErrorData::Custom allocates ... handling OOM errors. ErrorKind::OutOfMemory.into()` indicates a protocol choice based on allocation behavior; macro const_error!: constructs `&SimpleMessage { kind, message }` and calls `Error::from_static_message(...)`, implying a distinct 'static-message representation; struct Custom { kind: ErrorKind, error: Box<dyn error::Error + Send + Sync> } shows the allocating/boxed-error representation is distinct from Simple/SimpleMessage/Os

**Implementation:** Introduce internal (or public, if desired) typed wrappers representing the representation class, e.g. `struct IoError<R> { repr: R }` with `OsRepr { code: RawOsError }`, `KindRepr { kind: ErrorKind }`, `StaticMsgRepr { msg: &'static SimpleMessage }`, `CustomRepr { kind: ErrorKind, source: Box<dyn Error + Send + Sync> }`. Conversions like `From<TryReserveError>` would be constrained to produce `IoError<KindRepr>` (non-allocating), while `Error::new`-like APIs would produce `IoError<CustomRepr>`. Provide `impl From<IoError<_>> for Error` to erase to the existing `Error` when needed.

---

### 28. StdoutRaw validity protocol (Valid handle / Invalid-or-closed handle with EBADF fallback)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: StdoutRaw is a thin wrapper over an underlying stdio::Stdout whose operations can fail with EBADF (invalid file descriptor / closed stdio). The implementation relies on a runtime EBADF-recovery/fallback path via handle_ebadf(...) that turns certain write/flush failures into successful no-ops (or 'pretend success' by returning buf.len()). This implies an implicit state machine where the handle may become invalid, yet Write methods are still callable and will be treated specially. The type system does not represent or prevent the InvalidOrClosed state, nor does it force callers to acknowledge that writes may be dropped/treated as successful when EBADF occurs.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdout_raw` function.
struct StdoutRaw(stdio::Stdout);


// ... (other code) ...

    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

```

**Entity:** StdoutRaw

**States:** Valid, InvalidOrClosed

**Transitions:**
- Valid -> InvalidOrClosed via underlying stdio::Stdout becoming EBADF (external event, observed during write/flush calls)

**Evidence:** comment on StdoutRaw: "This handle is not synchronized or buffered in any fashion. Constructed via the std::io::stdio::stdout_raw function." (indicates special semantics/constraints beyond the type); method Write::write: handle_ebadf(self.0.write(buf), || Ok(buf.len())) (explicit EBADF handling + 'pretend all bytes written' fallback); method Write::write_vectored: handle_ebadf(self.0.write_vectored(bufs), total) (same EBADF-recovery path); method Write::flush: handle_ebadf(self.0.flush(), || Ok(())) (treats flush failure as success under EBADF); methods write_all / write_all_vectored / write_fmt all route through handle_ebadf(..., || Ok(())) (systematically encoding the same invalid-handle behavior)

**Implementation:** Introduce a typestate/capability split such as StdoutRaw<Valid> for the normal case and StdoutRaw<PossiblyInvalid> (or a dedicated 'best-effort' writer newtype) for the EBADF-swallowing behavior. Alternatively, expose two constructors: one returning a StrictStdoutRaw that propagates EBADF, and one returning BestEffortStdoutRaw that documents/delimits the 'pretend success' semantics at the type level; implement Write only for the appropriate wrapper so callers must opt into the semantics.

---

## Precondition Invariants

### 20. Static const-error message validity (must be 'static and alignment-compatible with repr_bitpacked)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-408`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `SimpleMessage` is intended to be used only as a `'static` reference (for `const_error!`) and must satisfy a layout/alignment precondition when the bitpacked representation is enabled. These requirements are currently enforced by comments, macro shape, and attributes rather than by a dedicated type-level capability. Misuse (e.g., attempting to smuggle a non-'static message into APIs expecting a static message, or changing alignment assumptions relied upon by `repr_bitpacked`) is prevented socially/structurally rather than by a strong type distinction.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Error, impl Error (0 methods), impl From < alloc :: ffi :: NulError > for Error (1 methods), impl From < alloc :: collections :: TryReserveError > for Error (1 methods), impl ErrorKind (1 methods), impl From < ErrorKind > for Error (1 methods), impl Error (13 methods), impl error :: Error for Error (3 methods); struct SimpleMessage; struct Custom; enum ErrorData; enum ErrorKind

#[cfg(test)]
mod tests;

#[cfg(all(target_pointer_width = "64", not(target_os = "uefi")))]
mod repr_bitpacked;
#[cfg(all(target_pointer_width = "64", not(target_os = "uefi")))]
use repr_bitpacked::Repr;

#[cfg(any(not(target_pointer_width = "64"), target_os = "uefi"))]
mod repr_unpacked;
#[cfg(any(not(target_pointer_width = "64"), target_os = "uefi"))]
use repr_unpacked::Repr;

use crate::{error, fmt, result, sys};

/// A specialized [`Result`] type for I/O operations.
///
/// This type is broadly used across [`std::io`] for any operation which may
/// produce an error.
///
/// This typedef is generally used to avoid writing out [`io::Error`] directly and
/// is otherwise a direct mapping to [`Result`].
///
/// While usual Rust style is to import types directly, aliases of [`Result`]
/// often are not, to make it easier to distinguish between them. [`Result`] is
/// generally assumed to be [`std::result::Result`][`Result`], and so users of this alias
/// will generally use `io::Result` instead of shadowing the [prelude]'s import
/// of [`std::result::Result`][`Result`].
///
/// [`std::io`]: crate::io
/// [`io::Error`]: Error
/// [`Result`]: crate::result::Result
/// [prelude]: crate::prelude
///
/// # Examples
///
/// A convenience function that bubbles an `io::Result` to its caller:
///
/// ```
/// use std::io;
///
/// fn get_string() -> io::Result<String> {
///     let mut buffer = String::new();
///
///     io::stdin().read_line(&mut buffer)?;
///
///     Ok(buffer)
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(bootstrap), doc(search_unbox))]
pub type Result<T> = result::Result<T, Error>;

/// The error type for I/O operations of the [`Read`], [`Write`], [`Seek`], and
/// associated traits.
///
/// Errors mostly originate from the underlying OS, but custom instances of
/// `Error` can be created with crafted error messages and a particular value of
/// [`ErrorKind`].
///
/// [`Read`]: crate::io::Read
/// [`Write`]: crate::io::Write
/// [`Seek`]: crate::io::Seek
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Error {
    repr: Repr,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

/// Common errors constants for use in std
#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    pub(crate) const READ_EXACT_EOF: Self =
        const_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer");

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_error!(
        ErrorKind::NotFound,
        "the number of hardware threads is not known for the target platform",
    );

    pub(crate) const UNSUPPORTED_PLATFORM: Self =
        const_error!(ErrorKind::Unsupported, "operation not supported on this platform");

    pub(crate) const WRITE_ALL_EOF: Self =
        const_error!(ErrorKind::WriteZero, "failed to write whole buffer");

    pub(crate) const ZERO_TIMEOUT: Self =
        const_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout");
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<alloc::ffi::NulError> for Error {
    /// Converts a [`alloc::ffi::NulError`] into a [`Error`].
    fn from(_: alloc::ffi::NulError) -> Error {
        const_error!(ErrorKind::InvalidInput, "data provided contains a nul byte")
    }
}

#[stable(feature = "io_error_from_try_reserve", since = "1.78.0")]
impl From<alloc::collections::TryReserveError> for Error {
    /// Converts `TryReserveError` to an error with [`ErrorKind::OutOfMemory`].
    ///
    /// `TryReserveError` won't be available as the error `source()`,
    /// but this may change in the future.
    fn from(_: alloc::collections::TryReserveError) -> Error {
        // ErrorData::Custom allocates, which isn't great for handling OOM errors.
        ErrorKind::OutOfMemory.into()
    }
}

// Only derive debug in tests, to make sure it
// doesn't accidentally get printed.
#[cfg_attr(test, derive(Debug))]
enum ErrorData<C> {
    Os(RawOsError),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
    Custom(C),
}

/// The type of raw OS error codes returned by [`Error::raw_os_error`].
///
/// This is an [`i32`] on all currently supported platforms, but platforms
/// added in the future (such as UEFI) may use a different primitive type like
/// [`usize`]. Use `as`or [`into`] conversions where applicable to ensure maximum
/// portability.
///
/// [`into`]: Into::into
#[unstable(feature = "raw_os_error_ty", issue = "107792")]
pub type RawOsError = sys::RawOsError;

// `#[repr(align(4))]` is probably redundant, it should have that value or
// higher already. We include it just because repr_bitpacked.rs's encoding
// requires an alignment >= 4 (note that `#[repr(align)]` will not reduce the
// alignment required by the struct, only increase it).
//
// If we add more variants to ErrorData, this can be increased to 8, but it
// should probably be behind `#[cfg_attr(target_pointer_width = "64", ...)]` or
// whatever cfg we're using to enable the `repr_bitpacked` code, since only the
// that version needs the alignment, and 8 is higher than the alignment we'll
// have on 32 bit platforms.
//
// (For the sake of being explicit: the alignment requirement here only matters
// if `error/repr_bitpacked.rs` is in use — for the unpacked repr it doesn't
// matter at all)
#[doc(hidden)]
#[unstable(feature = "io_const_error_internals", issue = "none")]
#[repr(align(4))]
#[derive(Debug)]
pub struct SimpleMessage {
    pub kind: ErrorKind,
    pub message: &'static str,
}

/// Creates a new I/O error from a known kind of error and a string literal.
///
/// Contrary to [`Error::new`], this macro does not allocate and can be used in
/// `const` contexts.
///
/// # Example
/// ```
/// #![feature(io_const_error)]
/// use std::io::{const_error, Error, ErrorKind};
///
/// const FAIL: Error = const_error!(ErrorKind::Unsupported, "tried something that never works");
///
/// fn not_here() -> Result<(), Error> {
///     Err(FAIL)
/// }
/// ```
#[rustc_macro_transparency = "semitransparent"]
#[unstable(feature = "io_const_error", issue = "133448")]
#[allow_internal_unstable(hint_must_use, io_const_error_internals)]
pub macro const_error($kind:expr, $message:expr $(,)?) {
    $crate::hint::must_use($crate::io::Error::from_static_message(
        const { &$crate::io::SimpleMessage { kind: $kind, message: $message } },
    ))
}

// As with `SimpleMessage`: `#[repr(align(4))]` here is just because
// repr_bitpacked's encoding requires it. In practice it almost certainly be
// already be this high or higher.
#[derive(Debug)]
#[repr(align(4))]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

/// A list specifying general categories of I/O error.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`io::Error`] type.
///
/// [`io::Error`]: Error
///
/// # Handling errors and matching on `ErrorKind`
///
/// In application code, use `match` for the `ErrorKind` values you are
/// expecting; use `_` to match "all other errors".
///
/// In comprehensive and thorough tests that want to verify that a test doesn't
/// return any known incorrect error kind, you may want to cut-and-paste the
/// current full list of errors from here into your test code, and then match
/// `_` as the correct case. This seems counterintuitive, but it will make your
/// tests more robust. In particular, if you want to verify that your code does
/// produce an unrecognized error kind, the robust solution is to check for all
/// the recognized error kinds and fail in those cases.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[stable(feature = "rust1", since = "1.0.0")]
#[allow(deprecated)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An entity was not found, often a file.
    #[stable(feature = "rust1", since = "1.0.0")]
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    #[stable(feature = "rust1", since = "1.0.0")]
    PermissionDenied,
    /// The connection was refused by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionRefused,
    /// The connection was reset by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionReset,
    /// The remote host is not reachable.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    HostUnreachable,
    /// The network containing the remote host is not reachable.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NetworkUnreachable,
    /// The connection was aborted (terminated) by the remote server.
    #[stable(feature = "rust1", since = "1.0.0")]
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    #[stable(feature = "rust1", since = "1.0.0")]
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    #[stable(feature = "rust1", since = "1.0.0")]
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    #[stable(feature = "rust1", since = "1.0.0")]
    AddrNotAvailable,
    /// The system's networking is down.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NetworkDown,
    /// The operation failed because a pipe was closed.
    #[stable(feature = "rust1", since = "1.0.0")]
    BrokenPipe,
    /// An entity already exists, often a file.
    #[stable(feature = "rust1", since = "1.0.0")]
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    #[stable(feature = "rust1", since = "1.0.0")]
    WouldBlock,
    /// A filesystem object is, unexpectedly, not a directory.
    ///
    /// For example, a filesystem path was specified where one of the intermediate directory
    /// components was, in fact, a plain file.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NotADirectory,
    /// The filesystem object is, unexpectedly, a directory.
    ///
    /// A directory was specified when a non-directory was expected.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    IsADirectory,
    /// A non-empty directory was specified where an empty directory was expected.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    DirectoryNotEmpty,
    /// The filesystem or storage medium is read-only, but a write operation was attempted.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ReadOnlyFilesystem,
    /// Loop in the filesystem or IO subsystem; often, too many levels of symbolic links.
    ///
    /// There was a loop (or excessively long chain) resolving a filesystem object
    /// or file IO object.
    ///
    /// On Unix this is usually the result of a symbolic link loop; or, of exceeding the
    /// system-specific limit on the depth of symlink traversal.
    #[unstable(feature = "io_error_more", issue = "86442")]
    FilesystemLoop,
    /// Stale network file handle.
    ///
    /// With some network filesystems, notably NFS, an open file (or directory) can be invalidated
    /// by problems with the network or server.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    StaleNetworkFileHandle,
    /// A parameter was incorrect.
    #[stable(feature = "rust1", since = "1.0.0")]
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    #[stable(feature = "io_invalid_data", since = "1.2.0")]
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    #[stable(feature = "rust1", since = "1.0.0")]
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    #[stable(feature = "rust1", since = "1.0.0")]
    WriteZero,
    /// The underlying storage (typically, a filesystem) is full.
    ///
    /// This does not include out of quota errors.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    StorageFull,
    /// Seek on unseekable file.
    ///
    /// Seeking was attempted on an open file handle which is not suitable for seeking - for
    /// example, on Unix, a named pipe opened with `File::open`.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    NotSeekable,
    /// Filesystem quota or some other kind of quota was exceeded.
    #[stable(feature = "io_error_quota_exceeded", since = "1.85.0")]
    QuotaExceeded,
    /// File larger than allowed or supported.
    ///
    /// This might arise from a hard limit of the underlying filesystem or file access API, or from
    /// an administratively imposed resource limitation.  Simple disk full, and out of quota, have
    /// their own errors.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    FileTooLarge,
    /// Resource is busy.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ResourceBusy,
    /// Executable file is busy.
    ///
    /// An attempt was made to write to a file which is also in use as a running program.  (Not all
    /// operating systems detect this situation.)
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ExecutableFileBusy,
    /// Deadlock (avoided).
    ///
    /// A file locking operation would result in deadlock.  This situation is typically detected, if
    /// at all, on a best-effort basis.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    Deadlock,
    /// Cross-device or cross-filesystem (hard) link or rename.
    #[stable(feature = "io_error_crosses_devices", since = "1.85.0")]
    CrossesDevices,
    /// Too many (hard) links to the same filesystem object.
    ///
    /// The filesystem does not support making so many hardlinks to the same file.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    TooManyLinks,
    /// A filename was invalid.
    ///
    /// This error can also occur if a length limit for a name was exceeded.
    #[stable(feature = "io_error_invalid_filename", since = "1.87.0")]
    InvalidFilename,
    /// Program argument list too long.
    ///
    /// When trying to run an external program, a system or process limit on the size of the
    /// arguments would have been exceeded.
    #[stable(feature = "io_error_a_bit_more", since = "1.83.0")]
    ArgumentListTooLong,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    #[stable(feature = "rust1", since = "1.0.0")]
    Interrupted,

    /// This operation is unsupported on this platform.
    ///
    /// This means that the operation can never succeed.
    #[stable(feature = "unsupported_error", since = "1.53.0")]
    Unsupported,

    // ErrorKinds which are primarily categorisations for OS error
    // codes should be added above.
    //
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particul
// ... (truncated) ...
```

**Entity:** SimpleMessage

**States:** ValidStaticMessage, InvalidMessageLayoutOrLifetime

**Transitions:**
- Literal kind+message -> ValidStaticMessage via `const_error!` (builds `&'static SimpleMessage` in a const context)

**Evidence:** variant `ErrorData::SimpleMessage(&'static SimpleMessage)` requires `'static`; pub struct SimpleMessage { pub kind: ErrorKind, pub message: &'static str } encodes `'static` message storage; comment on `SimpleMessage`: "repr_bitpacked.rs's encoding requires an alignment >= 4" plus `#[repr(align(4))]` indicates a non-local layout invariant relied upon elsewhere; macro const_error!: `const { &$crate::io::SimpleMessage { ... } }` constructs a `'static` reference, implying a protocol that these messages are only created via this route for const/non-allocating errors

**Implementation:** Make construction of `SimpleMessage` impossible outside the macro by making fields private and providing only an internal `const fn new(kind, msg: &'static str) -> &'static SimpleMessage` (or a newtype `StaticIoMessage(&'static SimpleMessage)` token). APIs that require the 'static-message representation accept the capability type rather than `&'static SimpleMessage` directly, making the intended protocol explicit and limiting future layout-sensitive changes to one construction point.

---

### 25. Two independent stdin buffering domains (Raw unbuffered vs Global buffered) must not be mixed

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-471`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: There is an implicit exclusivity/consistency requirement: using `StdinRaw` concurrently with `Stdin` violates assumptions about where bytes are buffered/consumed. The docs state that `StdinRaw` does not interact with handles from `stdin()` and that data buffered in the global `Stdin` is not available to raw handles. This is a protocol constraint ("choose one domain") that is only documented, not enforced by types, so code can accidentally interleave reads from both and observe surprising behavior (lost bytes, reordering relative to buffering).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock

#![cfg_attr(test, allow(unused))]

#[cfg(test)]
mod tests;

use crate::cell::{Cell, RefCell};
use crate::fmt;
use crate::fs::File;
use crate::io::prelude::*;
use crate::io::{
    self, BorrowedCursor, BufReader, IoSlice, IoSliceMut, LineWriter, Lines, SpecReadByte,
};
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::sync::{Arc, Mutex, MutexGuard, OnceLock, ReentrantLock, ReentrantLockGuard};
use crate::sys::stdio;
use crate::thread::AccessError;

type LocalStream = Arc<Mutex<Vec<u8>>>;

thread_local! {
    /// Used by the test crate to capture the output of the print macros and panics.
    static OUTPUT_CAPTURE: Cell<Option<LocalStream>> = const {
        Cell::new(None)
    }
}

/// Flag to indicate OUTPUT_CAPTURE is used.
///
/// If it is None and was never set on any thread, this flag is set to false,
/// and OUTPUT_CAPTURE can be safely ignored on all threads, saving some time
/// and memory registering an unused thread local.
///
/// Note about memory ordering: This contains information about whether a
/// thread local variable might be in use. Although this is a global flag, the
/// memory ordering between threads does not matter: we only want this flag to
/// have a consistent order between set_output_capture and print_to *within
/// the same thread*. Within the same thread, things always have a perfectly
/// consistent order. So Ordering::Relaxed is fine.
static OUTPUT_CAPTURE_USED: Atomic<bool> = AtomicBool::new(false);

/// A handle to a raw instance of the standard input stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdin_raw` function.
struct StdinRaw(stdio::Stdin);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdout_raw` function.
struct StdoutRaw(stdio::Stdout);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stderr_raw` function.
struct StderrRaw(stdio::Stderr);

/// Constructs a new raw handle to the standard input of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdin`. Data buffered by the `std::io::stdin`
/// handles is **not** available to raw handles returned from this function.
///
/// The returned handle has no external synchronization or buffering.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdin_raw() -> StdinRaw {
    StdinRaw(stdio::Stdin::new())
}

/// Constructs a new raw handle to the standard output stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdout`. Note that data is buffered by the
/// `std::io::stdout` handles so writes which happen via this raw handle may
/// appear before previous writes.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdout_raw() -> StdoutRaw {
    StdoutRaw(stdio::Stdout::new())
}

/// Constructs a new raw handle to the standard error stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stderr`.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stderr_raw() -> StderrRaw {
    StderrRaw(stdio::Stderr::new())
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        handle_ebadf(self.0.read(buf), || Ok(0))
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        handle_ebadf(self.0.read_buf(buf), || Ok(()))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        handle_ebadf(self.0.read_vectored(bufs), || Ok(0))
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        handle_ebadf(self.0.read_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }
        handle_ebadf(self.0.read_buf_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_end(buf), || Ok(0))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_string(buf), || Ok(0))
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

fn handle_ebadf<T>(r: io::Result<T>, default: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    match r {
        Err(ref e) if stdio::is_ebadf(e) => default(),
        r => r,
    }
}

/// A handle to the standard input stream of a process.
///
/// Each handle is a shared reference to a global buffer of input data to this
/// process. A handle can be `lock`'d to gain full access to [`BufRead`] methods
/// (e.g., `.lines()`). Reads to this handle are otherwise locked with respect
/// to other reads.
///
/// This handle implements the `Read` trait, but beware that concurrent reads
/// of `Stdin` must be executed with care.
///
/// Created by the [`io::stdin`] method.
///
/// [`io::stdin`]: stdin
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     stdin.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Stdin")]
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A locked reference to the [`Stdin`] handle.
///
/// This handle implements both the [`Read`] and [`BufRead`] traits, and
/// is constructed via the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     {
///         let mut handle = stdin.lock(); // We get `StdinLock` here.
///         handle.read_line(&mut buffer)?;
///     } // `StdinLock` is dropped here.
///     Ok(())
/// }
/// ```
#[must_use = "if unused stdin will immediately unlock"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<StdinRaw>>,
}

/// Constructs a new handle to the standard input of the current process.
///
/// Each handle returned is a reference to a shared global buffer whose access
/// is synchronized via a mutex. If you need more explicit control over
/// locking, see the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     io::stdin().read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin();
///     let mut handle = stdin.lock();
///
///     handle.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn stdin() -> Stdin {
    static INSTANCE: OnceLock<Mutex<BufReader<StdinRaw>>> = OnceLock::new();
    Stdin {
        inner: INSTANCE.get_or_init(|| {
            Mutex::new(BufReader::with_capacity(stdio::STDIN_BUF_SIZE, stdin_raw()))
        }),
    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`]. In particular:
    /// * Previous content of the buffer will be preserved. To avoid appending
    ///   to the buffer, you need to [`clear`] it first.
    /// * The trailing newline character, if any, is included in the buffer.
    ///
    /// [`clear`]: String::clear
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     }
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly,
    ///   in which case it will wait for the Enter key to be pressed before
    ///   continuing
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_confusables("get_line")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "stdin_forwarders", since = "1.62.0")]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self,
// ... (truncated) ...
```

**Entity:** stdin_raw / StdinRaw vs stdin() / Stdin

**States:** Using global buffered stdin (Stdin), Using independent raw stdin handle (StdinRaw)

**Transitions:**
- Choose buffered domain via stdin() (initializes/uses OnceLock global buffer)
- Choose raw domain via stdin_raw() (creates independent handle)

**Evidence:** fn stdin_raw() -> StdinRaw and doc: "does not interact with any other handles created nor handles returned by std::io::stdin"; doc on stdin_raw(): "Data buffered by the std::io::stdin handles is **not** available to raw handles"; type: Stdin uses OnceLock<Mutex<BufReader<StdinRaw>>> (global buffered singleton), while StdinRaw wraps stdio::Stdin directly

**Implementation:** Model a process-level choice as a singleton typestate (e.g., `StdinMode<Buffered>` vs `StdinMode<Raw>`) obtained from a single constructor, where acquiring one mode consumes/locks out the other at the type level (or via a capability token passed to constructors) to prevent mixed-domain usage in the same program component.

---

### 4. BufWriter raw-buffer write precondition (spare capacity required for unchecked copy)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-435`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Writing into the internal `Vec<u8>` uses an unsafe fast-path (`write_to_buffer_unchecked`) that assumes the caller has ensured enough spare capacity (`buf.len() <= self.spare_capacity()`). Callers (write/write_all fast path, write_to_buf, write_cold, write_all_cold) enforce this by conditional checks and/or flushing first. The correctness of memory writes depends on this protocol being followed everywhere `write_to_buffer_unchecked` is used; the type system does not encode the capacity proof, so misuse would be UB.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct WriterPanicked, impl WriterPanicked (1 methods), impl error :: Error for WriterPanicked (1 methods)

/// [`TcpStream`]: crate::net::TcpStream
/// [`flush`]: BufWriter::flush
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BufWriter<W: ?Sized + Write> {
    // The buffer. Avoid using this like a normal `Vec` in common code paths.
    // That is, don't use `buf.push`, `buf.extend_from_slice`, or any other
    // methods that require bounds checking or the like. This makes an enormous
    // difference to performance (we may want to stop using a `Vec` entirely).
    buf: Vec<u8>,
    // #30888: If the inner writer panics in a call to write, we don't want to
    // write the buffered data a second time in BufWriter's destructor. This
    // flag tells the Drop impl if it should skip the flush.
    panicked: bool,
    inner: W,
}

impl<W: Write> BufWriter<W> {
    /// Creates a new `BufWriter<W>` with a default buffer capacity. The default is currently 8 KiB,
    /// but may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub(crate) fn try_new_buffer() -> io::Result<Vec<u8>> {
        Vec::try_with_capacity(DEFAULT_BUF_SIZE).map_err(|_| {
            io::const_error!(ErrorKind::OutOfMemory, "failed to allocate write buffer")
        })
    }

    pub(crate) fn with_buffer(inner: W, buf: Vec<u8>) -> Self {
        Self { inner, buf, panicked: false }
    }

    /// Creates a new `BufWriter<W>` with at least the specified buffer capacity.
    ///
    /// # Examples
    ///
    /// Creating a buffer with a buffer of at least a hundred bytes.
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    /// let mut buffer = BufWriter::with_capacity(100, stream);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter { inner, buf: Vec::with_capacity(capacity), panicked: false }
    }

    /// Unwraps this `BufWriter<W>`, returning the underlying writer.
    ///
    /// The buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if an error occurs while flushing the buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // unwrap the TcpStream and flush the buffer
    /// let stream = buffer.into_inner().unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0),
        }
    }

    /// Disassembles this `BufWriter<W>`, returning the underlying writer, and any buffered but
    /// unwritten data.
    ///
    /// If the underlying writer panicked, it is not known what portion of the data was written.
    /// In this case, we return `WriterPanicked` for the buffered data (from which the buffer
    /// contents can still be recovered).
    ///
    /// `into_parts` makes no attempt to flush data and cannot fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{BufWriter, Write};
    ///
    /// let mut buffer = [0u8; 10];
    /// let mut stream = BufWriter::new(buffer.as_mut());
    /// write!(stream, "too much data").unwrap();
    /// stream.flush().expect_err("it doesn't fit");
    /// let (recovered_writer, buffered_data) = stream.into_parts();
    /// assert_eq!(recovered_writer.len(), 0);
    /// assert_eq!(&buffered_data.unwrap(), b"ata");
    /// ```
    #[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
    pub fn into_parts(self) -> (W, Result<Vec<u8>, WriterPanicked>) {
        let mut this = ManuallyDrop::new(self);
        let buf = mem::take(&mut this.buf);
        let buf = if !this.panicked { Ok(buf) } else { Err(WriterPanicked { buf }) };

        // SAFETY: double-drops are prevented by putting `this` in a ManuallyDrop that is never dropped
        let inner = unsafe { ptr::read(&this.inner) };

        (inner, buf)
    }
}

impl<W: ?Sized + Write> BufWriter<W> {
    /// Send data in our local buffer into the inner writer, looping as
    /// necessary until either it's all been sent or an error occurs.
    ///
    /// Because all the data in the buffer has been reported to our owner as
    /// "successfully written" (by returning nonzero success values from
    /// `write`), any 0-length writes from `inner` must be reported as i/o
    /// errors from this method.
    pub(in crate::io) fn flush_buf(&mut self) -> io::Result<()> {
        /// Helper struct to ensure the buffer is updated after all the writes
        /// are complete. It tracks the number of written bytes and drains them
        /// all from the front of the buffer when dropped.
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }

        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }

            /// The unwritten part of the buffer
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }

            /// Flag some bytes as removed from the front of the buffer
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }

            /// true if all of the bytes have been written
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }

        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    self.buffer.drain(..self.written);
                }
            }
        }

        let mut guard = BufGuard::new(&mut self.buf);
        while !guard.done() {
            self.panicked = true;
            let r = self.inner.write(guard.remaining());
            self.panicked = false;

            match r {
                Ok(0) => {
                    return Err(io::const_error!(
                        ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.consume(n),
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Buffer some data without flushing it, regardless of the size of the
    /// data. Writes as much as possible without exceeding capacity. Returns
    /// the number of bytes written.
    pub(super) fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());

        // SAFETY: `amt_to_buffer` is <= buffer's spare capacity by construction.
        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }

        amt_to_buffer
    }

    /// Gets a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // we can use reference just like buffer
    /// let reference = buffer.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let mut buffer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // we can use reference just like buffer
    /// let reference = buffer.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let buf_writer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // See how many bytes are currently buffered
    /// let bytes_buffered = buf_writer.buffer().len();
    /// ```
    #[stable(feature = "bufreader_buffer", since = "1.37.0")]
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns a mutable reference to the internal buffer.
    ///
    /// This can be used to write data directly into the buffer without triggering writers
    /// to the underlying writer.
    ///
    /// That the buffer is a `Vec` is an implementation detail.
    /// Callers should not modify the capacity as there currently is no public API to do so
    /// and thus any capacity changes would be unexpected by the user.
    pub(in crate::io) fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    /// Returns the number of bytes the internal buffer can hold without flushing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufWriter;
    /// use std::net::TcpStream;
    ///
    /// let buf_writer = BufWriter::new(TcpStream::connect("127.0.0.1:34254").unwrap());
    ///
    /// // Check the capacity of the inner buffer
    /// let capacity = buf_writer.capacity();
    /// // Calculate how many bytes can be written without flushing
    /// let without_flush = capacity - buf_writer.buffer().len();
    /// ```
    #[stable(feature = "buffered_io_capacity", since = "1.46.0")]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    // Ensure this function does not get inlined into `write`, so that it
    // remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write`,
    // it's likely a sign that the client is using an improperly sized buffer
    // or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    fn write_cold(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }

        // Why not len > capacity? To avoid a needless trip through the buffer when the input
        // exactly fills it. We'd just need to flush it to the underlying writer anyway.
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer. In this case, we write to the buffer even if it fills it
            // exactly. Doing otherwise would mean flushing the buffer, then writing this
            // input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we
            // flushed the buffer to ensure that there is. In the latter case, we know that there
            // is because flushing ensured that our entire buffer is spare capacity, and we entered
            // this block because the input buffer length is less than that capacity. In either
            // case, it's safe to write the input buffer to our buffer.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        }
    }

    // Ensure this function does not get inlined into `write_all`, so that it
    // remains inlineable and its common path remains as short as possible.
    // If this function ends up being called frequently relative to `write_all`,
    // it's likely a sign that the client is using an improperly sized buffer
    // or their write patterns are somewhat pathological.
    #[cold]
    #[inline(never)]
    fn write_all_cold(&mut self, buf: &[u8]) -> io::Result<()> {
        // Normally, `write_all` just calls `write` in a loop. We can do better
        // by calling `self.get_mut().write_all()` directly, which avoids
        // round trips through the buffer in the event of a series of partial
        // writes in some circumstances.

        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }

        // Why not len > capacity? To avoid a needless trip through the buffer when the input
        // exactly fills it. We'd just need to flush it to the underlying writer anyway.
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write_all(buf);
            self.panicked = false;
            r
        } else {
            // Write to the buffer. In this case, we write to the buffer even if it fills it
            // exactly. Doing otherwise would mean flushing the buffer, then writing this
            // input to the inner writer, which in many cases would be a worse strategy.

            // SAFETY: There was either enough spare capacity already, or there wasn't and we
            // flushed the buffer to ensure that there is. In the latter case, we know that there
            // is because flushing ensured that our entire buffer is spare capacity, and we entered
            // this block because the input buffer length is less than that capacity. In either
            // case, it's safe to write the input buffer to our buffer.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        }
    }

    // SAFETY: Requires `buf.len() <= self.buf.capacity() - self.buf.len()`,
    // i.e., that input buffer length is less than or equal to spare capacity.
    #[inline]
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        debug_assert!(buf.len() <= self.spare_capacity());
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }

    #[inline]
    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Write for BufWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_cold` for details.
        if buf.len() < self.spare_capacity() {
            // SAFETY: safe by above conditional.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(buf.len())
        } else {
            self.write_cold(buf)
        }
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // Use < instead of <= to avoid a needless trip through the buffer in some cases.
        // See `write_all_cold` for details.
        if buf.len() < self.spare_capacity() {
            // SAFETY: safe by above conditional.
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }

            Ok(())
        } else {
            self.write_all_cold(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<us
// ... (truncated) ...
```

**Entity:** BufWriter<W>

**States:** CapacitySufficientForAppend, CapacityInsufficientForAppend

**Transitions:**
- CapacityInsufficientForAppend -> CapacitySufficientForAppend via `flush_buf()` (empties/drains buffer, making spare capacity large)
- CapacitySufficientForAppend -> (append) via unsafe `write_to_buffer_unchecked()` which increases `self.buf.len()`

**Evidence:** unsafe fn write_to_buffer_unchecked(): comment: "SAFETY: Requires `buf.len() <= self.buf.capacity() - self.buf.len()`"; write(): guard `if buf.len() < self.spare_capacity() { unsafe { self.write_to_buffer_unchecked(buf); } }`; write_all(): guard `if buf.len() < self.spare_capacity() { unsafe { self.write_to_buffer_unchecked(buf); } }`; write_to_buf(): computes `amt_to_buffer = self.spare_capacity().min(buf.len())` then calls `unsafe { self.write_to_buffer_unchecked(&buf[..amt_to_buffer]); }`; write_cold()/write_all_cold(): `if buf.len() > self.spare_capacity() { self.flush_buf()?; }` followed by unsafe call with a SAFETY comment relying on the flush+length checks

**Implementation:** Create an internal helper type that represents a proven amount that fits, e.g. `struct FitsSpare<'a> { bw: &'a mut BufWriter<W>, amt: usize }` constructed only by checking `amt <= spare_capacity()`. Expose an API like `fn reserve_and_get_slot(&mut self, amt) -> FitsSpare` (possibly flushing) and then make `write_to_buffer_unchecked` accept `FitsSpare` (or use a closure-taking method) so the proof travels in the type rather than comments/discipline.

---

### 31. Empty read protocol (always-EOF; read_exact only valid for empty request)

**Location**: `/tmp/io_test_crate/src/io/util.rs:1-189`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `Empty` is a reader that is permanently at EOF: all read operations return 0 / empty. The only operation that can fail is `read_exact`/`read_buf_exact`, which requires (as a precondition) that the requested amount/capacity is zero; otherwise it returns `READ_EXACT_EOF`. This invariant is enforced by runtime checks on the input buffer/cursor rather than by the type system (which cannot express 'zero-length buffer' for slices/cursors).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Repeat, 1 free function(s), impl Read for Repeat (8 methods), impl SizeHint for Repeat (2 methods); struct Sink, 1 free function(s), impl Write for Sink (7 methods), impl Write for & Sink (7 methods)

#[stable(feature = "rust1", since = "1.0.0")]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default)]
pub struct Empty;


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Empty {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_buf(&mut self, _cursor: BorrowedCursor<'_>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        // Do not force `Chain<Empty, T>` or `Chain<T, Empty>` to use vectored
        // reads, unless the other reader is vectored.
        false
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if !buf.is_empty() { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() != 0 { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }
    }

    #[inline]
    fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_to_string(&mut self, _buf: &mut String) -> io::Result<usize> {
        Ok(0)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for Empty {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&[])
    }

    #[inline]
    fn consume(&mut self, _n: usize) {}

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        Ok(false)
    }

    #[inline]
    fn read_until(&mut self, _byte: u8, _buf: &mut Vec<u8>) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn skip_until(&mut self, _byte: u8) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_line(&mut self, _buf: &mut String) -> io::Result<usize> {
        Ok(0)
    }
}

#[stable(feature = "empty_seek", since = "1.51.0")]
impl Seek for Empty {
    #[inline]
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Ok(0)
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(0)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(0)
    }
}

impl SizeHint for Empty {
    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        Some(0)
    }
}

#[stable(feature = "empty_write", since = "1.73.0")]
impl Write for Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum();
        Ok(total_len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, _bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, _args: fmt::Arguments<'_>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[stable(feature = "empty_write", since = "1.73.0")]
impl Write for &Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum();
        Ok(total_len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, _bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, _args: fmt::Arguments<'_>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

```

**Entity:** Empty

**States:** EOF/NoData

**Evidence:** struct Empty has no fields (unit-like state), implying no internal state transitions; impl Read for Empty: read() returns Ok(0); impl Read for Empty: read_vectored() returns Ok(0); impl BufRead for Empty: fill_buf() returns Ok(&[]) and has_data_left() returns Ok(false); Read::read_exact(): `if !buf.is_empty() { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }`; Read::read_buf_exact(): `if cursor.capacity() != 0 { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }`

**Implementation:** Introduce specialized APIs that encode the precondition in the argument type, e.g. `fn read_exact_empty(&mut self, _: &mut [u8; 0]) -> io::Result<()>` (or a `ZeroLenBuf` newtype) and/or a dedicated trait like `EofRead` for readers that are statically known to be at EOF, avoiding `read_exact` misuse on this type.

---

### 40. Discontiguous-buffer read protocol (FrontOnly vs Front+Back) with exact-read failure clearing

**Location**: `/tmp/io_test_crate/src/io/impls.rs:1-589`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `read_exact`/`read_buf_exact` depend on a precondition that the deque contains at least the requested number of bytes across its two internal slices (`front` and `back`). When this precondition is violated, the methods perform observable state changes (they `clear()` the deque) before returning `READ_EXACT_EOF`. This 'failure consumes available data and empties the buffer' behavior is a latent protocol not expressed in the type system: callers cannot tell from types that an exact-read failure will drop buffered bytes.

**Evidence**:

```rust
#[cfg(test)]
mod tests;

use crate::alloc::Allocator;
use crate::collections::VecDeque;
use crate::io::{self, BorrowedCursor, BufRead, IoSlice, IoSliceMut, Read, Seek, SeekFrom, Write};
use crate::{cmp, fmt, mem, str};

// =============================================================================
// Forwarding implementations

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for &mut B {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        (**self).is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (**self).write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        (**self).stream_len()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        (**self).seek_relative(offset)
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead + ?Sized> BufRead for Box<B> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn has_data_left(&mut self) -> io::Result<bool> {
        (**self).has_data_left()
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn skip_until(&mut self, byte: u8) -> io::Result<usize> {
        (**self).skip_until(byte)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

// =============================================================================
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
#[stable(feature = "rust1", since = "1.0.0")]
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let amt = cmp::min(cursor.capacity(), self.len());
        let (a, b) = self.split_at(amt);

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            nread += self.read(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nread)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            // `read_exact` makes no promise about the content of `buf` if it
            // fails so don't bother about that.
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() > self.len() {
            // Append everything we can to the cursor.
            cursor.append(*self);
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(cursor.capacity());

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(*self);
        *self = &self[len..];
        Ok(len)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content = str::from_utf8(self).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = self.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        *self = &self[len..];
        Ok(len)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for &[u8] {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(*self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            nwritten += self.write(buf)?;
            if self.is_empty() {
                break;
            }
        }

        Ok(nwritten)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? < data.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        for buf in bufs {
            if self.write(buf)? < buf.len() {
                return Err(io::Error::WRITE_ALL_EOF);
            }
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
#[stable(feature = "rust1", since = "1.0.0")]
impl<A: Allocator> Write for Vec<u8, A> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend_from_slice(buf);
        }
        Ok(len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        self.write_vectored(bufs)?;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
#[stable(feature = "vecdeque_read_write", since = "1.63.0")]
impl<A: Allocator> Read for VecDeque<u8, A> {
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let (front, back) = self.as_slices();

        // Use only the front buffer if it is big enough to fill `buf`, else use
        // the back buffer too.
        match buf.split_at_mut_checked(front.len()) {
            None => buf.copy_from_slice(&front[..buf.len()]),
            Some((buf_front, buf_back)) => match back.split_at_checked(buf_back.len()) {
                Some((back, _)) => {
                    buf_front.copy_from_slice(front);
                    buf_back.copy_from_slice(back);
                }
                None => {
                    self.clear();
                    return Err(io::Error::READ_EXACT_EOF);
                }
            },
        }

        self.drain(..buf.len());
        Ok(())
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let (ref mut front, _) = self.as_slices();
        let n = cmp::min(cursor.capacity(), front.len());
        Read::read_buf(front, cursor)?;
        self.drain(..n);
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let len = cursor.capacity();
        let (front, back) = self.as_slices();

        match front.split_at_checked(cursor.capacity()) {
            Some((front, _)) => cursor.append(front),
            None => {
                cursor.append(front);
                match back.split_at_checked(cursor.capacity()) {
                    Some((back, _)) => cursor.append(back),
                    None => {
                        cursor.append(back);
                        self.clear();
                        return Err(io::Error::READ_EXACT_EOF);
                    }
                }
            }
        }

        self.drain(..len);
        Ok(())
// ... (truncated) ...
```

**Entity:** VecDeque<u8, A> (as Read)

**States:** EnoughBytes(total_len >= requested), NotEnoughBytes(total_len < requested)

**Transitions:**
- EnoughBytes -> EnoughBytes (with fewer bytes) via read()/read_exact()/read_buf()/read_buf_exact() (drain(..n) or drain(..len))
- NotEnoughBytes -> Empty via read_exact()/read_buf_exact() failure path calling clear() then returning Err(READ_EXACT_EOF)

**Evidence:** VecDeque<u8, A> Read::read(): reads from `front` then `self.drain(..n);`; VecDeque<u8, A> Read::read_exact(): on insufficient bytes in `back` path: `self.clear(); return Err(io::Error::READ_EXACT_EOF);`; VecDeque<u8, A> Read::read_buf_exact(): on insufficient bytes across `front`+`back`: `self.clear(); return Err(io::Error::READ_EXACT_EOF);`; use of `let (front, back) = self.as_slices();` and conditional split logic shows the implicit 'may require two segments' protocol

**Implementation:** Introduce a checked capability like `struct Available<'a> { dq: &'a mut VecDeque<u8, A>, len: usize }` produced by a `try_reserve_read(len) -> Result<Available, NeedMore>`; only `Available` exposes `read_exact(len)` without clearing-on-error. Alternatively return a remainder type on failure (e.g., `Err((READ_EXACT_EOF, bytes_consumed))`) to make the consumption protocol explicit.

---

### 45. peek()/consume coupling and bounds precondition (Peeked slice validity)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-462`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: BufReader::peek exposes a slice into the internal buffer and documents a usage protocol: `n` must be <= capacity, and after peeking the caller 'may call consume with a value <= n' to advance over returned bytes. The implementation enforces the bound with a runtime `assert!`, and relies on the temporal coupling between peeked data and subsequent consume/reads to keep the caller's interpretation correct. While Rust's borrow checker prevents many misuses of the returned slice within the same borrow, the protocol details (n <= capacity; and that advancing should be limited to the peeked region for the intended semantics) are not represented as types/newtypes and are partly enforced by panic.

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BufReader<R: ?Sized> {
    buf: Buffer,
    inner: R,
}

impl<R: Read> BufReader<R> {
    /// Creates a new `BufReader<R>` with a default buffer capacity. The default is currently 8 KiB,
    /// but may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let reader = BufReader::new(f);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub(crate) fn try_new_buffer() -> io::Result<Buffer> {
        Buffer::try_with_capacity(DEFAULT_BUF_SIZE)
    }

    pub(crate) fn with_buffer(inner: R, buf: Buffer) -> Self {
        Self { inner, buf }
    }

    /// Creates a new `BufReader<R>` with the specified buffer capacity.
    ///
    /// # Examples
    ///
    /// Creating a buffer with ten bytes of capacity:
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let reader = BufReader::with_capacity(10, f);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        BufReader { inner, buf: Buffer::with_capacity(capacity) }
    }
}

impl<R: Read + ?Sized> BufReader<R> {
    /// Attempt to look ahead `n` bytes.
    ///
    /// `n` must be less than or equal to `capacity`.
    ///
    /// The returned slice may be less than `n` bytes long if
    /// end of file is reached.
    ///
    /// After calling this method, you may call [`consume`](BufRead::consume)
    /// with a value less than or equal to `n` to advance over some or all of
    /// the returned bytes.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// #![feature(bufreader_peek)]
    /// use std::io::{Read, BufReader};
    ///
    /// let mut bytes = &b"oh, hello there"[..];
    /// let mut rdr = BufReader::with_capacity(6, &mut bytes);
    /// assert_eq!(rdr.peek(2).unwrap(), b"oh");
    /// let mut buf = [0; 4];
    /// rdr.read(&mut buf[..]).unwrap();
    /// assert_eq!(&buf, b"oh, ");
    /// assert_eq!(rdr.peek(5).unwrap(), b"hello");
    /// let mut s = String::new();
    /// rdr.read_to_string(&mut s).unwrap();
    /// assert_eq!(&s, "hello there");
    /// assert_eq!(rdr.peek(1).unwrap().len(), 0);
    /// ```
    #[unstable(feature = "bufreader_peek", issue = "128405")]
    pub fn peek(&mut self, n: usize) -> io::Result<&[u8]> {
        assert!(n <= self.capacity());
        while n > self.buf.buffer().len() {
            if self.buf.pos() > 0 {
                self.buf.backshift();
            }
            let new = self.buf.read_more(&mut self.inner)?;
            if new == 0 {
                // end of file, no more bytes to read
                return Ok(&self.buf.buffer()[..]);
            }
            debug_assert_eq!(self.buf.pos(), 0);
        }
        Ok(&self.buf.buffer()[..n])
    }
}

impl<R: ?Sized> BufReader<R> {
    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike [`fill_buf`], this will not attempt to fill the buffer if it is empty.
    ///
    /// [`fill_buf`]: BufRead::fill_buf
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{BufReader, BufRead};
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f);
    ///     assert!(reader.buffer().is_empty());
    ///
    ///     if reader.fill_buf()?.len() > 0 {
    ///         assert!(!reader.buffer().is_empty());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "bufreader_buffer", since = "1.37.0")]
    pub fn buffer(&self) -> &[u8] {
        self.buf.buffer()
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{BufReader, BufRead};
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f);
    ///
    ///     let capacity = reader.capacity();
    ///     let buffer = reader.fill_buf()?;
    ///     assert!(buffer.len() <= capacity);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "buffered_io_capacity", since = "1.46.0")]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Unwraps this `BufReader<R>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost. Therefore,
    /// a following read from the underlying reader may lead to data loss.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.into_inner();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> R
    where
        R: Sized,
    {
        self.inner
    }

    /// Invalidates all data in the internal buffer.
    #[inline]
    pub(in crate::io) fn discard_buffer(&mut self) {
        self.buf.discard_buffer()
    }
}

// This is only used by a test which asserts that the initialization-tracking is correct.
#[cfg(test)]
impl<R: ?Sized> BufReader<R> {
    #[allow(missing_docs)]
    pub fn initialized(&self) -> usize {
        self.buf.initialized()
    }
}

impl<R: ?Sized + Seek> BufReader<R> {
    /// Seeks relative to the current position. If the new position lies within the buffer,
    /// the buffer will not be flushed, allowing for more efficient seeks.
    /// This method does not return the location of the underlying reader, so the caller
    /// must track this information themselves if it is required.
    #[stable(feature = "bufreader_seek_relative", since = "1.53.0")]
    pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        let pos = self.buf.pos() as u64;
        if offset < 0 {
            if let Some(_) = pos.checked_sub((-offset) as u64) {
                self.buf.unconsume((-offset) as usize);
                return Ok(());
            }
        } else if let Some(new_pos) = pos.checked_add(offset as u64) {
            if new_pos <= self.buf.filled() as u64 {
                self.buf.consume(offset as usize);
                return Ok(());
            }
        }

        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}

impl<R> SpecReadByte for BufReader<R>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.buf.pos() == self.buf.filled() && buf.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let mut rem = self.fill_buf()?;
        let nread = rem.read(buf)?;
        self.consume(nread);
        Ok(nread)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.buf.pos() == self.buf.filled() && cursor.capacity() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_buf(cursor);
        }

        let prev = cursor.written();

        let mut rem = self.fill_buf()?;
        rem.read_buf(cursor.reborrow())?; // actually never fails

        self.consume(cursor.written() - prev); //slice impl of read_buf known to never unfill buf

        Ok(())
    }

    // Small read_exacts from a BufReader are extremely common when used with a deserializer.
    // The default implementation calls read in a loop, which results in surprisingly poor code
    // generation for the common path where the buffer has enough bytes to fill the passed-in
    // buffer.
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if self.buf.consume_with(buf.len(), |claimed| buf.copy_from_slice(claimed)) {
            return Ok(());
        }

        crate::io::default_read_exact(self, buf)
    }

    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if self.buf.consume_with(cursor.capacity(), |claimed| cursor.append(claimed)) {
            return Ok(());
        }

        crate::io::default_read_buf_exact(self, cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.buf.pos() == self.buf.filled() && total_len >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }
        let mut rem = self.fill_buf()?;
        let nread = rem.read_vectored(bufs)?;

        self.consume(nread);
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    // The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    // delegate to the inner implementation.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let inner_buf = self.buffer();
        buf.try_reserve(inner_buf.len())?;
        buf.extend_from_slice(inner_buf);
        let nread = inner_buf.len();
        self.discard_buffer();
        Ok(nread + self.inner.read_to_end(buf)?)
    }

    // The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    // delegate to the inner implementation.
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        // In the general `else` case below we must read bytes into a side buffer, check
        // that they are valid UTF-8, and then append them to `buf`. This requires a
        // potentially large memcpy.
        //
        // If `buf` is empty--the most common case--we can leverage `append_to_string`
        // to read directly into `buf`'s internal byte buffer, saving an allocation and
        // a memcpy.
        if buf.is_empty() {
            // `append_to_string`'s safety relies on the buffer only being appended to since
            // it only checks the UTF-8 validity of new data. If there were existing content in
            // `buf` then an untrustworthy reader (i.e. `self.inner`) could not only append
            // bytes but also modify existing bytes and render them invalid. On the other hand,
            // if `buf` is empty then by definition any writes must be appends and
            // `append_to_string` will validate all of the new bytes.
            unsafe { crate::io::append_to_string(buf, |b| self.read_to_end(b)) }
        } else {
            // We cannot append our byte buffer directly onto the `buf` String as there could
            // be an incomplete UTF-8 sequence that has only been partially read. We must read
            // everything into a side buffer first and then call `from_utf8` on the complete
            // buffer.
            let mut bytes = Vec::new();
            self.read_to_end(&mut bytes)?;
            let string = crate::str::from_utf8(&bytes).map_err(|_| io::Error::INVALID_UTF8)?;
            *buf += string;
            Ok(string.len())
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.buf.fill_buf(&mut self.inner)
    }

    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt)
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Seek> Seek for BufReader<R> {
    /// Seek to an offset, in bytes, in the underlying reader.
    ///
    /// The position used for seeking with <code>[SeekFrom::Current]\(_)</code> is the
    /// position the underlying reader would be at if the `BufReader<R>` had no
    /// internal buffer.
    ///
    /// Seeking always discards the internal buffer, even if the seek position
    /// would otherwise fall within it. This guarantees that calling
    /// [`BufReader::into_inner()`] immediately after a seek yields the underlying reader
    /// at the same position.
    ///
    /// To seek without discarding the internal buffer, use [`BufReader::seek_relative`].
    ///
    /// See [`std::io::Seek`] for more details.
    ///
    /// Note: In the edge case where you're seeking with <code>[SeekFrom::Current]\(n)</code>
    /// where `n` minus the internal buffer length overflows an `i64`, two
    /// seeks will be performed instead of one. If the second seek returns
    /// [`Err`], the underlying reader will be left at the same position it would
    /// have if you called `seek` with <code>[SeekFrom::Current]\(0)</code>.
    ///
    /// [`std::io::Seek`]: Seek
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.buf.filled() - self.buf.pos()) as i64;
            // it should be safe to assume that remainder fits within an i64 as the alternative
            // means we managed to allocate 8 exbibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to
            // support seeking by i64::MIN so we need to handle underflow when subtracting
            // remainder.
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                // seek backwards by our remainder, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.discard_buffer();
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length.
            result = self.inner.seek(pos)?;
        }
        self.discard_buffer();
        Ok(result)
    }

    /// Returns the current seek position from the start of the stream.
    ///

// ... (truncated) ...
```

**Entity:** BufReader<R>

**States:** NotPeeked(no outstanding peek view), Peeked(view returned; caller expected to consume <= n before other advances)

**Transitions:**
- NotPeeked -> Peeked via peek(n) returning `&[u8]` tied to `&mut self` borrow
- Peeked -> NotPeeked by ending the borrow (dropping the slice) and optionally calling consume(k) with k <= n

**Evidence:** peek() doc: '`n` must be less than or equal to `capacity`' and 'After calling this method, you may call consume ... <= n'; peek() implementation: `assert!(n <= self.capacity());` (runtime-enforced precondition); peek() returns `Ok(&self.buf.buffer()[..n])` / `[..]` (slice validity depends on buffer management and the caller not holding it across incompatible mutations)

**Implementation:** Introduce a bounded request type like `struct PeekLen(usize);` constructed by `BufReader::peek_len(n) -> Option<PeekLen>` (or `Result`) that validates `n <= capacity` without panicking, and/or return a guard `Peeked<'a>` that carries the maximum consumable length and offers `fn consume(self, reader: &mut BufReader<_>, amt: usize)` which statically ties 'consume <= n' to the peek result (still checked, but centralized and harder to misuse).

---

## Protocol Invariants

### 12. PipeWriter endpoint capability protocol (must only be used as the pipe's write end)

**Location**: `/tmp/io_test_crate/src/io/pipe.rs:1-108`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: PipeWriter semantically represents the write end of an anonymous pipe. Correctness relies on treating this handle as a 'write capability' that can be transferred to child processes or duplicated, and on not confusing it with a read end. The type enforces 'write vs read' at the Rust level (PipeWriter implements io::Write), but does not enforce higher-level protocol constraints such as ensuring at least one writer remains open while a peer expects to read, or ensuring cloned writers are accounted for before expecting EOF on the reader. These are temporal/protocol expectations around cloning/duplicating the write capability that remain implicit.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct PipeReader, 1 free function(s), impl FromInner < AnonPipe > for PipeReader (1 methods), impl IntoInner < AnonPipe > for PipeReader (1 methods), impl PipeReader (1 methods), impl io :: Read for & PipeReader (5 methods), impl io :: Read for PipeReader (5 methods)

/// Write end of an anonymous pipe.
#[stable(feature = "anonymous_pipe", since = "1.87.0")]
#[derive(Debug)]
pub struct PipeWriter(pub(crate) AnonPipe);


// ... (other code) ...

    }
}

impl FromInner<AnonPipe> for PipeWriter {
    fn from_inner(inner: AnonPipe) -> Self {
        Self(inner)
    }
}

impl IntoInner<AnonPipe> for PipeWriter {
    fn into_inner(self) -> AnonPipe {
        self.0
    }
}

// ... (other code) ...

    }
}

impl PipeWriter {
    /// Creates a new [`PipeWriter`] instance that shares the same underlying file description.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(miri)] fn main() {}
    /// # #[cfg(not(miri))]
    /// # fn main() -> std::io::Result<()> {
    /// use std::process::Command;
    /// use std::io::{pipe, Read};
    /// let (mut reader, writer) = pipe()?;
    ///
    /// // Spawn a process that writes to stdout and stderr.
    /// let mut peer = Command::new("bash")
    ///     .args([
    ///         "-c",
    ///         "echo -n foo\n\
    ///          echo -n bar >&2"
    ///     ])
    ///     .stdout(writer.try_clone()?)
    ///     .stderr(writer)
    ///     .spawn()?;
    ///
    /// // Read and check the result.
    /// let mut msg = String::new();
    /// reader.read_to_string(&mut msg)?;
    /// assert_eq!(&msg, "foobar");
    ///
    /// peer.wait()?;
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "anonymous_pipe", since = "1.87.0")]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }
}

// ... (other code) ...

}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Write for &PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

```

**Entity:** PipeWriter

**States:** Write-end capability held, Write-end capability transferred/duplicated (via try_clone)

**Transitions:**
- Write-end capability held -> Write-end capability transferred/duplicated via PipeWriter::try_clone(&self) -> io::Result<PipeWriter>

**Evidence:** comment: `/// Write end of an anonymous pipe.` defines the capability role; method: `pub fn try_clone(&self) -> io::Result<Self> { self.0.try_clone().map(Self) }` duplicates the underlying file description, implicitly affecting EOF/liveness semantics for the pipe; example: `.stdout(writer.try_clone()?).stderr(writer)` demonstrates intended protocol of cloning and then transferring writers to multiple destinations

**Implementation:** Model 'unique writer' vs 'shared writer' explicitly: e.g., `PipeWriter<Unique>` returned by `pipe()`, `try_clone(&self) -> PipeWriter<Shared>` (or `PipeWriter::share(&self)`), and require an explicit `into_shared()` before multi-target distribution. This makes protocol points (duplication affects EOF) more visible and restricts APIs that assume uniqueness.

---

### 42. MutexGuard poisoning recovery protocol (Poisoned -> Recovered) hidden inside lock()

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-233`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Acquiring a StdinLock implicitly includes a poisoning-handling protocol: if the underlying mutex is poisoned, the implementation discards the poison state by taking the inner guard anyway (`into_inner`). This creates an implicit invariant that downstream code will proceed even after a panic occurred while holding the lock, and that all operations on StdinLock assume this recovery is acceptable. The choice (propagate poison vs recover) is encoded only as runtime behavior in Stdin::lock(), not at the type level, so callers cannot opt into a 'poison-checked' vs 'poison-ignored' locked state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock; trait IsTerminal, 9 free function(s)

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Stdin")]
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

// ... (other code) ...

    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`]. In particular:
    /// * Previous content of the buffer will be preserved. To avoid appending
    ///   to the buffer, you need to [`clear`] it first.
    /// * The trailing newline character, if any, is included in the buffer.
    ///
    /// [`clear`]: String::clear
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     }
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly,
    ///   in which case it will wait for the Enter key to be pressed before
    ///   continuing
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_confusables("get_line")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "stdin_forwarders", since = "1.62.0")]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf_exact(cursor)
    }
}

#[stable(feature = "read_shared_stdin", since = "1.78.0")]
impl Read for &Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf_exact(cursor)
    }
}

// only used by platform-dependent io::copy specializations, i.e. unused on some platforms
#[cfg(any(target_os = "linux", target_os = "android"))]
impl StdinLock<'_> {
    pub(crate) fn as_mut_buf(&mut self) -> &mut BufReader<impl Read> {
        &mut self.inner
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.inner.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.inner.read_buf_exact(cursor)
    }
}

impl SpecReadByte for StdinLock<'_> {
    #[inline]
    fn spec_read_byte(&mut self) -> Option<io::Result<u8>> {
        BufReader::spec_read_byte(&mut *self.inner)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl BufRead for StdinLock<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, n: usize) {
        self.inner.consume(n)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }
}

```

**Entity:** StdinLock<'_>

**States:** Mutex not poisoned (normal lock acquisition), Mutex poisoned (lock acquisition returns PoisonError, recovered via into_inner)

**Transitions:**
- Mutex poisoned -> Recovered Locked guard via `unwrap_or_else(|e| e.into_inner())` in Stdin::lock()
- Mutex not poisoned -> Locked guard via `Mutex::lock()` success path

**Evidence:** in `Stdin::lock()`: `self.inner.lock().unwrap_or_else(|e| e.into_inner())` explicitly recovers from poisoning rather than propagating it; return type `StdinLock<'static>` gives no indication whether the lock was acquired from a poisoned mutex or not

**Implementation:** Return different guard types (or a Result) to reflect poison handling at compile time/API level, e.g. `fn try_lock(&self) -> Result<StdinLock<'_>, PoisonedStdinLock<'_>>` or `fn lock(&self) -> Result<StdinLock<'_>, PoisonError<StdinLock<'_>>>`, and provide an explicit `recover()` transition to obtain a usable guard if desired.

---

### 43. Vec length restoration protocol (temporarily modified len must be restored on scope exit)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-17`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Guard encodes an implicit protocol around temporarily manipulating a Vec<u8>'s length: some code is expected to change the vector's length (often via unsafe set_len or by writing into spare capacity) and then rely on Guard::drop to restore it back to the saved `len`. The correctness of this protocol depends on external code ensuring that `len` is a valid length for `buf` at drop time and that restoring to `len` upholds Vec's safety invariants (initialized elements, capacity bounds). None of this is enforced by the type system; it is enforced only by RAII + an unsafe `set_len` in Drop and by the (implicit) expectation that users create Guard at the right time with the right `len`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)


pub(crate) use stdio::cleanup;

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

```

**Entity:** Guard<'a>

**States:** Armed (will restore len on Drop), Dropped (len restored)

**Transitions:**
- Armed -> Dropped via Drop::drop()

**Evidence:** struct Guard<'a> { buf: &'a mut Vec<u8>, len: usize } — stores a saved length alongside a mutable Vec reference; impl Drop for Guard<'_> { unsafe { self.buf.set_len(self.len); } } — Drop unconditionally restores the Vec length using unsafe set_len

**Implementation:** Replace raw `(buf: &mut Vec<u8>, len: usize)` with a dedicated capability/newtype that can only be constructed from a Vec in a way that proves `len` is valid (e.g., `struct SavedLen(usize); impl SavedLen { fn capture(v: &Vec<u8>) -> Self { SavedLen(v.len()) } }`). Alternatively, make the guard constructor take `&mut Vec<u8>` and internally capture `len` (no external `usize`), preventing mismatched/forged lengths. If the real intent is 'temporarily set_len to capacity', expose a safe API like `with_spare_capacity(|spare: &mut [MaybeUninit<u8>]| ...)` and commit an initialized length explicitly rather than restoring a caller-provided length.

---

### 47. Multi-buffer consumption protocol (RemainingBytes within total length; prefix removal + head advance)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-155`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: advance_slices consumes a requested byte count across a mutable slice of IoSlice, removing fully-consumed leading buffers and advancing into the first partially-consumed buffer. Correctness requires that `n` not exceed the total remaining bytes across all buffers; otherwise it panics. The function also depends on a specific aliasing/overflow avoidance protocol (using `checked_sub` per-buffer rather than summing lengths), and performs an in-place state transition on the caller-provided `&mut &mut [IoSlice<'a>]` by re-slicing it and mutating its first element. None of these constraints (especially `n <= total_len`) are encoded in types; misuse yields a runtime assertion panic.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

#[stable(feature = "iovec", since = "1.36.0")]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IoSlice<'a>(sys::io::IoSlice<'a>);

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Send for IoSlice<'a> {}

#[stable(feature = "iovec_send_sync", since = "1.44.0")]
unsafe impl<'a> Sync for IoSlice<'a> {}


// ... (other code) ...

    }
}

impl<'a> IoSlice<'a> {
    /// Creates a new `IoSlice` wrapping a byte slice.
    ///
    /// # Panics
    ///
    /// Panics on Windows if the slice is larger than 4GB.
    #[stable(feature = "iovec", since = "1.36.0")]
    #[must_use]
    #[inline]
    pub fn new(buf: &'a [u8]) -> IoSlice<'a> {
        IoSlice(sys::io::IoSlice::new(buf))
    }

    /// Advance the internal cursor of the slice.
    ///
    /// Also see [`IoSlice::advance_slices`] to advance the cursors of multiple
    /// buffers.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSlice;
    /// use std::ops::Deref;
    ///
    /// let data = [1; 8];
    /// let mut buf = IoSlice::new(&data);
    ///
    /// // Mark 3 bytes as read.
    /// buf.advance(3);
    /// assert_eq!(buf.deref(), [1; 5].as_ref());
    /// ```
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0.advance(n)
    }

    /// Advance a slice of slices.
    ///
    /// Shrinks the slice to remove any `IoSlice`s that are fully advanced over.
    /// If the cursor ends up in the middle of an `IoSlice`, it is modified
    /// to start at that cursor.
    ///
    /// For example, if we have a slice of two 8-byte `IoSlice`s, and we advance by 10 bytes,
    /// the result will only include the second `IoSlice`, advanced by 2 bytes.
    ///
    /// # Panics
    ///
    /// Panics when trying to advance beyond the end of the slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::IoSlice;
    /// use std::ops::Deref;
    ///
    /// let buf1 = [1; 8];
    /// let buf2 = [2; 16];
    /// let buf3 = [3; 8];
    /// let mut bufs = &mut [
    ///     IoSlice::new(&buf1),
    ///     IoSlice::new(&buf2),
    ///     IoSlice::new(&buf3),
    /// ][..];
    ///
    /// // Mark 10 bytes as written.
    /// IoSlice::advance_slices(&mut bufs, 10);
    /// assert_eq!(bufs[0].deref(), [2; 14].as_ref());
    /// assert_eq!(bufs[1].deref(), [3; 8].as_ref());
    #[stable(feature = "io_slice_advance", since = "1.81.0")]
    #[inline]
    pub fn advance_slices(bufs: &mut &mut [IoSlice<'a>], n: usize) {
        // Number of buffers to remove.
        let mut remove = 0;
        // Remaining length before reaching n. This prevents overflow
        // that could happen if the length of slices in `bufs` were instead
        // accumulated. Those slice may be aliased and, if they are large
        // enough, their added length may overflow a `usize`.
        let mut left = n;
        for buf in bufs.iter() {
            if let Some(remainder) = left.checked_sub(buf.len()) {
                left = remainder;
                remove += 1;
            } else {
                break;
            }
        }

        *bufs = &mut take(bufs)[remove..];
        if bufs.is_empty() {
            assert!(left == 0, "advancing io slices beyond their length");
        } else {
            bufs[0].advance(left);
        }
    }

    /// Get the underlying bytes as a slice with the original lifetime.
    ///
    /// This doesn't borrow from `self`, so is less restrictive than calling
    /// `.deref()`, which does.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(io_slice_as_bytes)]
    /// use std::io::IoSlice;
    ///
    /// let data = b"abcdef";
    ///
    /// let mut io_slice = IoSlice::new(data);
    /// let tail = &io_slice.as_slice()[3..];
    ///
    /// // This works because `tail` doesn't borrow `io_slice`
    /// io_slice = IoSlice::new(tail);
    ///
    /// assert_eq!(io_slice.as_slice(), b"def");
    /// ```
    #[unstable(feature = "io_slice_as_bytes", issue = "132818")]
    pub const fn as_slice(self) -> &'a [u8] {
        self.0.as_slice()
    }
}

#[stable(feature = "iovec", since = "1.36.0")]
impl<'a> Deref for IoSlice<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

```

**Entity:** IoSlice::advance_slices

**States:** Nonempty buffer list with remaining bytes, Empty buffer list (all fully advanced)

**Transitions:**
- Nonempty buffer list with remaining bytes -> Nonempty buffer list (prefix removed, head advanced) via IoSlice::advance_slices(bufs, n) when 0 < n < total_len and not aligned to buffer boundary
- Nonempty buffer list with remaining bytes -> Empty buffer list via IoSlice::advance_slices(bufs, n) when n == total_len
- Nonempty buffer list with remaining bytes -> (panic) via IoSlice::advance_slices(bufs, n) when n > total_len

**Evidence:** advance_slices(bufs, n): doc 'Shrinks the slice to remove any `IoSlice`s that are fully advanced over.'; advance_slices(bufs, n): doc 'Panics when trying to advance beyond the end of the slices.'; advance_slices: `if bufs.is_empty() { assert!(left == 0, "advancing io slices beyond their length"); }` runtime check encodes the key invariant; advance_slices: `*bufs = &mut take(bufs)[remove..];` demonstrates the protocol step of removing fully-consumed prefixes; advance_slices: `bufs[0].advance(left);` shows the final step depends on `left <= bufs[0].len()` (otherwise advance() would panic)

**Implementation:** Provide a checked API that returns a typed remainder rather than panicking, e.g. `fn try_advance_slices<'a>(bufs: &mut &mut [IoSlice<'a>], n: usize) -> Result<(), AdvancePastEnd>`. Alternatively, introduce a `struct TotalLen(usize)` computed once (or a `Remaining` capability token) and require callers to obtain a `struct WithinTotal(usize)` via a checked constructor before calling the advancing function.

---

### 26. Two independent stdout/stderr write domains (Raw unbuffered vs Global buffered) and ordering expectations

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-471`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The module documents an implicit protocol that raw output handles (`StdoutRaw`, `StderrRaw`) do not synchronize or coordinate with the buffered/synchronized global handles (returned by `std::io::stdout`/`stderr`). Mixing them can break ordering assumptions: writes through the raw handle may appear before previous buffered writes. This is a cross-handle ordering/state dependency that is only documented and handled by convention; the type system cannot prevent code from constructing both and assuming consistent ordering.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock

#![cfg_attr(test, allow(unused))]

#[cfg(test)]
mod tests;

use crate::cell::{Cell, RefCell};
use crate::fmt;
use crate::fs::File;
use crate::io::prelude::*;
use crate::io::{
    self, BorrowedCursor, BufReader, IoSlice, IoSliceMut, LineWriter, Lines, SpecReadByte,
};
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::sync::{Arc, Mutex, MutexGuard, OnceLock, ReentrantLock, ReentrantLockGuard};
use crate::sys::stdio;
use crate::thread::AccessError;

type LocalStream = Arc<Mutex<Vec<u8>>>;

thread_local! {
    /// Used by the test crate to capture the output of the print macros and panics.
    static OUTPUT_CAPTURE: Cell<Option<LocalStream>> = const {
        Cell::new(None)
    }
}

/// Flag to indicate OUTPUT_CAPTURE is used.
///
/// If it is None and was never set on any thread, this flag is set to false,
/// and OUTPUT_CAPTURE can be safely ignored on all threads, saving some time
/// and memory registering an unused thread local.
///
/// Note about memory ordering: This contains information about whether a
/// thread local variable might be in use. Although this is a global flag, the
/// memory ordering between threads does not matter: we only want this flag to
/// have a consistent order between set_output_capture and print_to *within
/// the same thread*. Within the same thread, things always have a perfectly
/// consistent order. So Ordering::Relaxed is fine.
static OUTPUT_CAPTURE_USED: Atomic<bool> = AtomicBool::new(false);

/// A handle to a raw instance of the standard input stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdin_raw` function.
struct StdinRaw(stdio::Stdin);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdout_raw` function.
struct StdoutRaw(stdio::Stdout);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stderr_raw` function.
struct StderrRaw(stdio::Stderr);

/// Constructs a new raw handle to the standard input of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdin`. Data buffered by the `std::io::stdin`
/// handles is **not** available to raw handles returned from this function.
///
/// The returned handle has no external synchronization or buffering.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdin_raw() -> StdinRaw {
    StdinRaw(stdio::Stdin::new())
}

/// Constructs a new raw handle to the standard output stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdout`. Note that data is buffered by the
/// `std::io::stdout` handles so writes which happen via this raw handle may
/// appear before previous writes.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdout_raw() -> StdoutRaw {
    StdoutRaw(stdio::Stdout::new())
}

/// Constructs a new raw handle to the standard error stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stderr`.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stderr_raw() -> StderrRaw {
    StderrRaw(stdio::Stderr::new())
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        handle_ebadf(self.0.read(buf), || Ok(0))
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        handle_ebadf(self.0.read_buf(buf), || Ok(()))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        handle_ebadf(self.0.read_vectored(bufs), || Ok(0))
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        handle_ebadf(self.0.read_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }
        handle_ebadf(self.0.read_buf_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_end(buf), || Ok(0))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_string(buf), || Ok(0))
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

fn handle_ebadf<T>(r: io::Result<T>, default: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    match r {
        Err(ref e) if stdio::is_ebadf(e) => default(),
        r => r,
    }
}

/// A handle to the standard input stream of a process.
///
/// Each handle is a shared reference to a global buffer of input data to this
/// process. A handle can be `lock`'d to gain full access to [`BufRead`] methods
/// (e.g., `.lines()`). Reads to this handle are otherwise locked with respect
/// to other reads.
///
/// This handle implements the `Read` trait, but beware that concurrent reads
/// of `Stdin` must be executed with care.
///
/// Created by the [`io::stdin`] method.
///
/// [`io::stdin`]: stdin
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     stdin.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Stdin")]
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A locked reference to the [`Stdin`] handle.
///
/// This handle implements both the [`Read`] and [`BufRead`] traits, and
/// is constructed via the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     {
///         let mut handle = stdin.lock(); // We get `StdinLock` here.
///         handle.read_line(&mut buffer)?;
///     } // `StdinLock` is dropped here.
///     Ok(())
/// }
/// ```
#[must_use = "if unused stdin will immediately unlock"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<StdinRaw>>,
}

/// Constructs a new handle to the standard input of the current process.
///
/// Each handle returned is a reference to a shared global buffer whose access
/// is synchronized via a mutex. If you need more explicit control over
/// locking, see the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     io::stdin().read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin();
///     let mut handle = stdin.lock();
///
///     handle.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn stdin() -> Stdin {
    static INSTANCE: OnceLock<Mutex<BufReader<StdinRaw>>> = OnceLock::new();
    Stdin {
        inner: INSTANCE.get_or_init(|| {
            Mutex::new(BufReader::with_capacity(stdio::STDIN_BUF_SIZE, stdin_raw()))
        }),
    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`]. In particular:
    /// * Previous content of the buffer will be preserved. To avoid appending
    ///   to the buffer, you need to [`clear`] it first.
    /// * The trailing newline character, if any, is included in the buffer.
    ///
    /// [`clear`]: String::clear
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     }
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly,
    ///   in which case it will wait for the Enter key to be pressed before
    ///   continuing
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_confusables("get_line")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "stdin_forwarders", since = "1.62.0")]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self,
// ... (truncated) ...
```

**Entity:** stdout_raw / StdoutRaw vs stdout (mentioned in docs) and stderr_raw / StderrRaw vs stderr (mentioned in docs)

**States:** Using global buffered stdout/stderr handles, Using independent raw stdout/stderr handles

**Transitions:**
- Choose buffered domain via std::io::stdout()/stderr() (buffered/synchronized)
- Choose raw domain via stdout_raw()/stderr_raw() (independent, unbuffered)

**Evidence:** fn stdout_raw() -> StdoutRaw doc: "does not interact with ... handles returned by std::io::stdout"; fn stdout_raw() doc: "data is buffered by the std::io::stdout handles so writes which happen via this raw handle may appear before previous writes"; fn stderr_raw() -> StderrRaw doc: "does not interact with ... handles returned by std::io::stderr"; types: StdoutRaw(stdio::Stdout) and StderrRaw(stdio::Stderr) are separate from the (documented) buffered stdout/stderr handles

**Implementation:** Require an explicit 'output domain' capability to create writers (e.g., `OutputDomain::Buffered` vs `OutputDomain::Raw`), and thread that capability through constructors so code must commit to a domain for a given subsystem; alternatively provide a single writer enum with variants but only constructible through one factory that makes ordering semantics explicit.

---

### 27. Lines iterator protocol (Readable -> EOF | Error)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-30`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Lines<B> encodes an implicit iteration protocol over an underlying BufRead: repeated calls to next() attempt to read a line. When read_line() returns Ok(0), the iterator transitions to an EOF state and thereafter should yield None. When read_line() returns Err(e), next() yields Some(Err(e)); many consumers treat this as terminal, but the type system does not enforce whether the iterator is allowed/expected to continue producing items after an error (it depends on the underlying BufRead and caller discipline). This is a latent state machine (Reading/EOF/Error) represented only by runtime results of read_line() and consumer convention, not by distinct types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); enum SeekFrom; trait Read, trait Write, trait Seek, trait BufRead, trait SpecReadByte, trait SizeHint, 12 free function(s), impl SpecReadByte for R (1 methods), impl SizeHint for T (2 methods), impl SizeHint for & mut T (2 methods), impl SizeHint for Box < T > (2 methods), impl SizeHint for & [u8] (2 methods)

#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
#[cfg_attr(not(test), rustc_diagnostic_item = "IoLines")]
pub struct Lines<B> {
    buf: B,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

```

**Entity:** Lines<B>

**States:** Reading, EOF, Error(terminal?)

**Transitions:**
- Reading -> EOF via next() when self.buf.read_line(&mut buf) returns Ok(0)
- Reading -> Reading via next() when self.buf.read_line(&mut buf) returns Ok(_n>0)
- Reading -> Error(terminal?) via next() when self.buf.read_line(&mut buf) returns Err(e)

**Evidence:** struct Lines<B> { buf: B } stores underlying reader state only in `buf`; Iterator::next(): `match self.buf.read_line(&mut buf)` drives control flow; Ok(0) => None indicates EOF state transition; Err(e) => Some(Err(e)) indicates an error-producing state that is not tracked in the type

**Implementation:** Model the iteration as a typestate-driven cursor: e.g., `struct Lines<B, S> { buf: B, _s: PhantomData<S> }` with states `Reading`, `Eof`; `next()` on `Lines<_, Reading>` returns either `(Lines<_, Reading>, Option<Result<String>>)` or transitions to `Lines<_, Eof>` when Ok(0) is observed. If error should be terminal, include an `Errored` state and return `Lines<_, Errored>` on Err(e).

---

### 34. LineWriterShim protocol: buffer must be empty before unbuffered inner writes; newline-completion drives flushing

**Location**: `/tmp/io_test_crate/src/io/buffered/linewritershim.rs:1-285`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: LineWriterShim relies on an implicit protocol between its line-buffering logic and the underlying BufWriter: (1) direct (unbuffered) writes to the inner writer via inner_mut().write()/write_vectored()/write_all() are only performed when the BufWriter buffer is known to be empty, and the code explicitly flushes the BufWriter first to achieve this; (2) if the existing buffer ends with a newline, it must be flushed before accepting additional buffered-only writes, to avoid leaving a completed line buffered across calls (this is treated as a 'retry flushing completed line' condition). These states are inferred from runtime inspection of the last buffered byte and from ordering constraints (flush_buf before inner_mut writes). The type system does not distinguish or enforce these states; correctness depends on calling flush_if_completed_line()/flush_buf() in the right places and not calling inner_mut() in contexts that would bypass or corrupt the intended line-buffering semantics.

**Evidence**:

```rust
/// `BufWriters` to be temporarily given line-buffering logic; this is what
/// enables Stdout to be alternately in line-buffered or block-buffered mode.
#[derive(Debug)]
pub struct LineWriterShim<'a, W: ?Sized + Write> {
    buffer: &'a mut BufWriter<W>,
}

impl<'a, W: ?Sized + Write> LineWriterShim<'a, W> {
    pub fn new(buffer: &'a mut BufWriter<W>) -> Self {
        Self { buffer }
    }

    /// Gets a reference to the inner writer (that is, the writer
    /// wrapped by the BufWriter).
    fn inner(&self) -> &W {
        self.buffer.get_ref()
    }

    /// Gets a mutable reference to the inner writer (that is, the writer
    /// wrapped by the BufWriter). Be careful with this writer, as writes to
    /// it will bypass the buffer.
    fn inner_mut(&mut self) -> &mut W {
        self.buffer.get_mut()
    }

    /// Gets the content currently buffered in self.buffer
    fn buffered(&self) -> &[u8] {
        self.buffer.buffer()
    }

    /// Flushes the buffer iff the last byte is a newline (indicating that an
    /// earlier write only succeeded partially, and we want to retry flushing
    /// the buffered line before continuing with a subsequent write).
    fn flush_if_completed_line(&mut self) -> io::Result<()> {
        match self.buffered().last().copied() {
            Some(b'\n') => self.buffer.flush_buf(),
            _ => Ok(()),
        }
    }
}

impl<'a, W: ?Sized + Write> Write for LineWriterShim<'a, W> {
    /// Writes some data into this BufReader with line buffering.
    ///
    /// This means that, if any newlines are present in the data, the data up to
    /// the last newline is sent directly to the underlying writer, and data
    /// after it is buffered. Returns the number of bytes written.
    ///
    /// This function operates on a "best effort basis"; in keeping with the
    /// convention of `Write::write`, it makes at most one attempt to write
    /// new data to the underlying writer. If that write only reports a partial
    /// success, the remaining data will be buffered.
    ///
    /// Because this function attempts to send completed lines to the underlying
    /// writer, it will also flush the existing buffer if it ends with a
    /// newline, even if the incoming data does not contain any newlines.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let newline_idx = match memchr::memrchr(b'\n', buf) {
            // If there are no new newlines (that is, if this write is less than
            // one line), just do a regular buffered write (which may flush if
            // we exceed the inner buffer's size)
            None => {
                self.flush_if_completed_line()?;
                return self.buffer.write(buf);
            }
            // Otherwise, arrange for the lines to be written directly to the
            // inner writer.
            Some(newline_idx) => newline_idx + 1,
        };

        // Flush existing content to prepare for our write. We have to do this
        // before attempting to write `buf` in order to maintain consistency;
        // if we add `buf` to the buffer then try to flush it all at once,
        // we're obligated to return Ok(), which would mean suppressing any
        // errors that occur during flush.
        self.buffer.flush_buf()?;

        // This is what we're going to try to write directly to the inner
        // writer. The rest will be buffered, if nothing goes wrong.
        let lines = &buf[..newline_idx];

        // Write `lines` directly to the inner writer. In keeping with the
        // `write` convention, make at most one attempt to add new (unbuffered)
        // data. Because this write doesn't touch the BufWriter state directly,
        // and the buffer is known to be empty, we don't need to worry about
        // self.buffer.panicked here.
        let flushed = self.inner_mut().write(lines)?;

        // If buffer returns Ok(0), propagate that to the caller without
        // doing additional buffering; otherwise we're just guaranteeing
        // an "ErrorKind::WriteZero" later.
        if flushed == 0 {
            return Ok(0);
        }

        // Now that the write has succeeded, buffer the rest (or as much of
        // the rest as possible). If there were any unwritten newlines, we
        // only buffer out to the last unwritten newline that fits in the
        // buffer; this helps prevent flushing partial lines on subsequent
        // calls to LineWriterShim::write.

        // Handle the cases in order of most-common to least-common, under
        // the presumption that most writes succeed in totality, and that most
        // writes are smaller than the buffer.
        // - Is this a partial line (ie, no newlines left in the unwritten tail)
        // - If not, does the data out to the last unwritten newline fit in
        //   the buffer?
        // - If not, scan for the last newline that *does* fit in the buffer
        let tail = if flushed >= newline_idx {
            let tail = &buf[flushed..];
            // Avoid unnecessary short writes by not splitting the remaining
            // bytes if they're larger than the buffer.
            // They can be written in full by the next call to write.
            if tail.len() >= self.buffer.capacity() {
                return Ok(flushed);
            }
            tail
        } else if newline_idx - flushed <= self.buffer.capacity() {
            &buf[flushed..newline_idx]
        } else {
            let scan_area = &buf[flushed..];
            let scan_area = &scan_area[..self.buffer.capacity()];
            match memchr::memrchr(b'\n', scan_area) {
                Some(newline_idx) => &scan_area[..newline_idx + 1],
                None => scan_area,
            }
        };

        let buffered = self.buffer.write_to_buf(tail);
        Ok(flushed + buffered)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }

    /// Writes some vectored data into this BufReader with line buffering.
    ///
    /// This means that, if any newlines are present in the data, the data up to
    /// and including the buffer containing the last newline is sent directly to
    /// the inner writer, and the data after it is buffered. Returns the number
    /// of bytes written.
    ///
    /// This function operates on a "best effort basis"; in keeping with the
    /// convention of `Write::write`, it makes at most one attempt to write
    /// new data to the underlying writer.
    ///
    /// Because this function attempts to send completed lines to the underlying
    /// writer, it will also flush the existing buffer if it contains any
    /// newlines.
    ///
    /// Because sorting through an array of `IoSlice` can be a bit convoluted,
    /// This method differs from write in the following ways:
    ///
    /// - It attempts to write the full content of all the buffers up to and
    ///   including the one containing the last newline. This means that it
    ///   may attempt to write a partial line, that buffer has data past the
    ///   newline.
    /// - If the write only reports partial success, it does not attempt to
    ///   find the precise location of the written bytes and buffer the rest.
    ///
    /// If the underlying vector doesn't support vectored writing, we instead
    /// simply write the first non-empty buffer with `write`. This way, we
    /// get the benefits of more granular partial-line handling without losing
    /// anything in efficiency
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        // If there's no specialized behavior for write_vectored, just use
        // write. This has the benefit of more granular partial-line handling.
        if !self.is_write_vectored() {
            return match bufs.iter().find(|buf| !buf.is_empty()) {
                Some(buf) => self.write(buf),
                None => Ok(0),
            };
        }

        // Find the buffer containing the last newline
        // FIXME: This is overly slow if there are very many bufs and none contain
        // newlines. e.g. writev() on Linux only writes up to 1024 slices, so
        // scanning the rest is wasted effort. This makes write_all_vectored()
        // quadratic.
        let last_newline_buf_idx = bufs
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, buf)| memchr::memchr(b'\n', buf).map(|_| i));

        // If there are no new newlines (that is, if this write is less than
        // one line), just do a regular buffered write
        let last_newline_buf_idx = match last_newline_buf_idx {
            // No newlines; just do a normal buffered write
            None => {
                self.flush_if_completed_line()?;
                return self.buffer.write_vectored(bufs);
            }
            Some(i) => i,
        };

        // Flush existing content to prepare for our write
        self.buffer.flush_buf()?;

        // This is what we're going to try to write directly to the inner
        // writer. The rest will be buffered, if nothing goes wrong.
        let (lines, tail) = bufs.split_at(last_newline_buf_idx + 1);

        // Write `lines` directly to the inner writer. In keeping with the
        // `write` convention, make at most one attempt to add new (unbuffered)
        // data. Because this write doesn't touch the BufWriter state directly,
        // and the buffer is known to be empty, we don't need to worry about
        // self.panicked here.
        let flushed = self.inner_mut().write_vectored(lines)?;

        // If inner returns Ok(0), propagate that to the caller without
        // doing additional buffering; otherwise we're just guaranteeing
        // an "ErrorKind::WriteZero" later.
        if flushed == 0 {
            return Ok(0);
        }

        // Don't try to reconstruct the exact amount written; just bail
        // in the event of a partial write
        let mut lines_len: usize = 0;
        for buf in lines {
            // With overlapping/duplicate slices the total length may in theory
            // exceed usize::MAX
            lines_len = lines_len.saturating_add(buf.len());
            if flushed < lines_len {
                return Ok(flushed);
            }
        }

        // Now that the write has succeeded, buffer the rest (or as much of the
        // rest as possible)
        let buffered: usize = tail
            .iter()
            .filter(|buf| !buf.is_empty())
            .map(|buf| self.buffer.write_to_buf(buf))
            .take_while(|&n| n > 0)
            .sum();

        Ok(flushed + buffered)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner().is_write_vectored()
    }

    /// Writes some data into this BufReader with line buffering.
    ///
    /// This means that, if any newlines are present in the data, the data up to
    /// the last newline is sent directly to the underlying writer, and data
    /// after it is buffered.
    ///
    /// Because this function attempts to send completed lines to the underlying
    /// writer, it will also flush the existing buffer if it contains any
    /// newlines, even if the incoming data does not contain any newlines.
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match memchr::memrchr(b'\n', buf) {
            // If there are no new newlines (that is, if this write is less than
            // one line), just do a regular buffered write (which may flush if
            // we exceed the inner buffer's size)
            None => {
                self.flush_if_completed_line()?;
                self.buffer.write_all(buf)
            }
            Some(newline_idx) => {
                let (lines, tail) = buf.split_at(newline_idx + 1);

                if self.buffered().is_empty() {
                    self.inner_mut().write_all(lines)?;
                } else {
                    // If there is any buffered data, we add the incoming lines
                    // to that buffer before flushing, which saves us at least
                    // one write call. We can't really do this with `write`,
                    // since we can't do this *and* not suppress errors *and*
                    // report a consistent state to the caller in a return
                    // value, but here in write_all it's fine.
                    self.buffer.write_all(lines)?;
                    self.buffer.flush_buf()?;
                }

                self.buffer.write_all(tail)
            }
        }
    }
}

```

**Entity:** LineWriterShim<'a, W>

**States:** BufferedTailIncomplete (BufWriter contains a partial line / last byte != '\n' or empty), BufferedLineComplete (BufWriter buffer ends with '\n'), PreparedForDirectWrite (BufWriter buffer known-empty before calling inner_mut().write*)

**Transitions:**
- BufferedTailIncomplete -> BufferedLineComplete when buffered().last() becomes '\n' (implicit; observed via flush_if_completed_line())
- BufferedLineComplete -> PreparedForDirectWrite via flush_if_completed_line() or explicit buffer.flush_buf()
- BufferedTailIncomplete -> PreparedForDirectWrite via explicit buffer.flush_buf() (done before inner_mut().write* when newlines present in input)
- PreparedForDirectWrite -> BufferedTailIncomplete via buffer.write_to_buf(tail) / buffer.write_all(tail) when tail exists after last newline

**Evidence:** field `buffer: &'a mut BufWriter<W>`: shim correctness depends on BufWriter's internal buffer contents/state; method `flush_if_completed_line`: inspects `self.buffered().last()` and flushes iff it is `Some(b'\n')`; write(): in the no-newline path, calls `self.flush_if_completed_line()?;` before `self.buffer.write(buf)`; write(): comment + ordering: "Flush existing content to prepare for our write. We have to do this before attempting to write `buf` in order to maintain consistency" followed by `self.buffer.flush_buf()?;` before `self.inner_mut().write(lines)?;`; write(): comment: "Because this write doesn't touch the BufWriter state directly, and the buffer is known to be empty" (relies on the prior flush_buf call to establish the state); write_vectored(): no-newline path calls `self.flush_if_completed_line()?;` before `self.buffer.write_vectored(bufs)`; write_vectored(): performs `self.buffer.flush_buf()?;` before `self.inner_mut().write_vectored(lines)?;` with the same "buffer is known to be empty" rationale; write_all(): if newlines exist, branches on `if self.buffered().is_empty()` to decide whether it is safe/beneficial to call `self.inner_mut().write_all(lines)?;` directly vs buffering then flushing

**Implementation:** Encode the BufWriter-buffer condition as a typestate carried by the shim (e.g., `LineWriterShim<'a, W, S>` with `S = Unknown|Empty|EndsWithNewline|PartialLine`). Provide internal transition methods like `ensure_empty(self) -> LineWriterShim<Empty>` (does flush_buf) and `flush_completed_line(self) -> LineWriterShim<PartialLineOrEmpty>` (does the last-byte check + flush). Restrict direct inner writes (`inner_mut().write*`) to the `Empty` state wrapper so the ordering (flush before bypassing the buffer) is enforced structurally rather than by convention/comments.

---

### 10. Capacity-reservation then unchecked write protocol (Reserved -> UncheckedWriteSafe)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-556`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The Vec-backed write implementation relies on a multi-step safety protocol: (1) ensure cursor position fits in `usize`; (2) reserve enough capacity for `pos + buf_len`; (3) pad zeroes up to `pos` by writing into spare capacity and updating len; only then (4) perform an unchecked raw-pointer copy into the Vec. This protocol is enforced by call ordering and `unsafe` comments/debug_asserts, not by types. Mis-ordering or calling the unchecked write helper without the reservation preconditions would be UB, but the type system does not encode the 'capacity reserved and padded' state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Cursor, impl Cursor < T > (6 methods), impl Cursor < T > (1 methods), impl Cursor < T > (1 methods), impl io :: Seek for Cursor < T > (3 methods), impl Read for Cursor < T > (8 methods), impl BufRead for Cursor < T > (2 methods), impl Write for Cursor < & mut [u8] > (6 methods), impl Write for Cursor < & mut Vec < u8 , A > > (6 methods), impl Write for Cursor < Vec < u8 , A > > (6 methods), impl Write for Cursor < Box < [u8] , A > > (6 methods), impl Write for Cursor < [u8 ; N] > (6 methods)

#[cfg(test)]
mod tests;

use crate::alloc::Allocator;
use crate::cmp;
use crate::io::prelude::*;
use crate::io::{self, BorrowedCursor, ErrorKind, IoSlice, IoSliceMut, SeekFrom};

/// A `Cursor` wraps an in-memory buffer and provides it with a
/// [`Seek`] implementation.
///
/// `Cursor`s are used with in-memory buffers, anything implementing
/// <code>[AsRef]<\[u8]></code>, to allow them to implement [`Read`] and/or [`Write`],
/// allowing these buffers to be used anywhere you might use a reader or writer
/// that does actual I/O.
///
/// The standard library implements some I/O traits on various types which
/// are commonly used as a buffer, like <code>Cursor<[Vec]\<u8>></code> and
/// <code>Cursor<[&\[u8\]][bytes]></code>.
///
/// # Examples
///
/// We may want to write bytes to a [`File`] in our production
/// code, but use an in-memory buffer in our tests. We can do this with
/// `Cursor`:
///
/// [bytes]: crate::slice "slice"
/// [`File`]: crate::fs::File
///
/// ```no_run
/// use std::io::prelude::*;
/// use std::io::{self, SeekFrom};
/// use std::fs::File;
///
/// // a library function we've written
/// fn write_ten_bytes_at_end<W: Write + Seek>(mut writer: W) -> io::Result<()> {
///     writer.seek(SeekFrom::End(-10))?;
///
///     for i in 0..10 {
///         writer.write(&[i])?;
///     }
///
///     // all went well
///     Ok(())
/// }
///
/// # fn foo() -> io::Result<()> {
/// // Here's some code that uses this library function.
/// //
/// // We might want to use a BufReader here for efficiency, but let's
/// // keep this example focused.
/// let mut file = File::create("foo.txt")?;
/// // First, we need to allocate 10 bytes to be able to write into.
/// file.set_len(10)?;
///
/// write_ten_bytes_at_end(&mut file)?;
/// # Ok(())
/// # }
///
/// // now let's write a test
/// #[test]
/// fn test_writes_bytes() {
///     // setting up a real File is much slower than an in-memory buffer,
///     // let's use a cursor instead
///     use std::io::Cursor;
///     let mut buff = Cursor::new(vec![0; 15]);
///
///     write_ten_bytes_at_end(&mut buff).unwrap();
///
///     assert_eq!(&buff.get_ref()[5..15], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    ///
    /// Cursor initial position is `0` even if underlying buffer (e.g., [`Vec`])
    /// is not empty. So writing to cursor starts with overwriting [`Vec`]
    /// content, not with appending to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }

    /// Consumes this cursor, returning the underlying value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let vec = buff.into_inner();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying value in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying value in this cursor.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying value as it may corrupt this cursor's position.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the current position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::io::prelude::*;
    /// use std::io::SeekFrom;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.seek(SeekFrom::Current(2)).unwrap();
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(buff.position(), 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.set_position(4);
    /// assert_eq!(buff.position(), 4);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    /// Splits the underlying slice at the cursor position and returns them.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(cursor_split)]
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.split(), ([].as_slice(), [1, 2, 3, 4, 5].as_slice()));
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.split(), ([1, 2].as_slice(), [3, 4, 5].as_slice()));
    ///
    /// buff.set_position(6);
    /// assert_eq!(buff.split(), ([1, 2, 3, 4, 5].as_slice(), [].as_slice()));
    /// ```
    #[unstable(feature = "cursor_split", issue = "86369")]
    pub fn split(&self) -> (&[u8], &[u8]) {
        let slice = self.inner.as_ref();
        let pos = self.pos.min(slice.len() as u64);
        slice.split_at(pos as usize)
    }
}

impl<T> Cursor<T>
where
    T: AsMut<[u8]>,
{
    /// Splits the underlying slice at the cursor position and returns them
    /// mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(cursor_split)]
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.split_mut(), ([].as_mut_slice(), [1, 2, 3, 4, 5].as_mut_slice()));
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.split_mut(), ([1, 2].as_mut_slice(), [3, 4, 5].as_mut_slice()));
    ///
    /// buff.set_position(6);
    /// assert_eq!(buff.split_mut(), ([1, 2, 3, 4, 5].as_mut_slice(), [].as_mut_slice()));
    /// ```
    #[unstable(feature = "cursor_split", issue = "86369")]
    pub fn split_mut(&mut self) -> (&mut [u8], &mut [u8]) {
        let slice = self.inner.as_mut();
        let pos = self.pos.min(slice.len() as u64);
        slice.split_at_mut(pos as usize)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Clone for Cursor<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Cursor { inner: self.inner.clone(), pos: self.pos }
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
        self.pos = other.pos;
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> io::Seek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::const_error!(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.inner.as_ref().len() as u64)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Read for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = Read::read(&mut Cursor::split(self).1, buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let prev_written = cursor.written();

        Read::read_buf(&mut Cursor::split(self).1, cursor.reborrow())?;

        self.pos += (cursor.written() - prev_written) as u64;

        Ok(())
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            nread += n;
            if n < buf.len() {
                break;
            }
        }
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        true
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let result = Read::read_exact(&mut Cursor::split(self).1, buf);

        match result {
            Ok(_) => self.pos += buf.len() as u64,
            // The only possible error condition is EOF, so place the cursor at "EOF"
            Err(_) => self.pos = self.inner.as_ref().len() as u64,
        }

        result
    }

    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let prev_written = cursor.written();

        let result = Read::read_buf_exact(&mut Cursor::split(self).1, cursor.reborrow());
        self.pos += (cursor.written() - prev_written) as u64;

        result
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let content = Cursor::split(self).1;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(content);
        self.pos += len as u64;

        Ok(len)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content =
            crate::str::from_utf8(Cursor::split(self).1).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        self.pos += len as u64;

        Ok(len)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> BufRead for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(Cursor::split(self).1)
    }
    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
    }
}

// Non-resizing write implementation
#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<usize> {
    let pos = cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

#[inline]
fn slice_write_vectored(
    pos_mut: &mut u64,
    slice: &mut [u8],
    bufs: &[IoSlice<'_>],
) -> io::Result<usize> {
    let mut nwritten = 0;
    for buf in bufs {
        let n = slice_write(pos_mut, slice, buf)?;
        nwritten += n;
        if n < buf.len() {
            break;
        }
    }
    Ok(nwritten)
}

#[inline]
fn slice_write_all(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<()> {
    let n = slice_write(pos_mut, slice, buf)?;
    if n < buf.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
}

#[inline]
fn slice_write_all_vectored(
    pos_mut: &mut u64,
    slice: &mut [u8],
    bufs: &[IoSlice<'_>],
) -> io::Result<()> {
    for buf in bufs {
        let n = slice_write(pos_mut, slice, buf)?;
        if n < buf.len() {
            return Err(io::Error::WRITE_ALL_EOF);
        }
    }
    Ok(())
}

/// Reserves the required space, and pads the vec with 0s if necessary.
fn reserve_and_pad<A: Allocator>(
    pos_mut: &mut u64,
    vec: &mut Vec<u8, A>,
    buf_len: usize,
) -> io::Result<usize> {
    let pos: usize = (*pos_mut).try_into().map_err(|_| {
        io::const_error!(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length",
        )
    })?;

    // For safety reasons, we don't want these numbers to overflow
    // otherwise our allocation won't be enough
    let desired_cap = pos.saturating_add(buf_len);
    if desired_cap > vec.capacity() {
        // We want our vec's total capacity
        // to have room for (pos+buf_len) bytes. Reserve allocates
        // based on additional elements from the length, so we need to
        // reserve the difference
        vec.reserve(desired_cap - vec.len());
    }
    // Pad if pos is above the current len.
    if pos > vec.len() {
        let diff = pos - vec.len();
        // Unfortunately, `resize()` would suffice but the optimiser does not
        // realise the `reserve` it does can be eliminated. So we do it manually
        // to eliminate that extra branch
        let spare = vec.spare_capacity_mut();
        debug_assert!(spare.len() >= diff);
        // Safety: we have allocated enough capacity for this.
        // And we are only writing, not reading
        unsafe {
            spare.get_unchecked_mut(..diff).fill(core::mem::MaybeUninit::new(0));
            vec.set_len(pos);
        }
    }

    Ok(pos)
}

/// Writes the slice to the vec without allocating.
///
/// # Safety
///
/// `vec` must have `buf.len()` spare capacity.
unsafe fn vec_write_all_unchecked<A>(pos: usize, vec: &mut Vec<u8, A>, buf: &[u8]) -> usize
where
    A: Allocator,
{
    debug_assert!(vec.capacity() >= pos + buf.len());
    unsafe { vec.as_mut_ptr().add(pos).copy_from(buf.as_ptr(), buf.len()) };
    pos + buf.len()
}

/// Resizing `write_all` implementation for [`Cursor`].
///
/// Cursor is allowed to have a pre-allocated and initialised
/// vector body, but with a position of 0. This means the [`Write`]
/// will overwrite the contents of the vec.
///
/// This also allows for the vec body to be empty, but with a position of N.
/// This means that [`Write`] will pad the vec with 0 initially,
/// before writing anything from that point
fn vec_write_all<A>(pos_mut: &mut u64, vec: &mut Vec<u8, A>, buf: &[u8]) -> io::Result<usize>
where
    A: Allocator,
{
    let buf_len = buf.len();
    let mut pos = reserve_and_pad(pos_mut, vec, buf_len)?;

    // Write the buf then progress the vec forward if necessary
    // Safety: we have ensured that the capacity is available
    // and that all bytes get written up to pos
    unsafe {
        pos = vec_write_all_unchecked(pos, vec, buf);
        if pos > vec.len() {
            vec.set_len(pos);
        }
    };

    // Bump us forward
    *pos_mut += buf_len as u64;
    Ok(buf_len)
}

/// Resizing `write_all_vectored` implementation for [`Cursor`].
///
/// Cursor is allowed to have a pre-allocated and initialised

// ... (truncated) ...
```

**Entity:** reserve_and_pad / vec_write_all_unchecked (Vec-backed Cursor write path)

**States:** NotReserved, ReservedAndPadded, UncheckedWriteSafe

**Transitions:**
- NotReserved -> ReservedAndPadded via `reserve_and_pad(pos_mut, vec, buf_len)`
- ReservedAndPadded -> UncheckedWriteSafe via `vec_write_all_unchecked(pos, vec, buf)` (requires spare capacity)
- UncheckedWriteSafe -> ReservedAndPadded via `vec.set_len(pos)` updates after write (maintains initialized length invariant)

**Evidence:** reserve_and_pad() comment: "Reserves the required space, and pads the vec with 0s if necessary."; reserve_and_pad(): computes `desired_cap = pos.saturating_add(buf_len)` and calls `vec.reserve(...)` to ensure capacity; reserve_and_pad(): pads by writing to `vec.spare_capacity_mut()` and then `unsafe { ... vec.set_len(pos); }` with comment "Safety: we have allocated enough capacity for this. And we are only writing, not reading"; vec_write_all_unchecked() is `unsafe fn` with doc: "`vec` must have `buf.len()` spare capacity."; vec_write_all_unchecked(): `debug_assert!(vec.capacity() >= pos + buf.len());` then raw pointer copy `vec.as_mut_ptr().add(pos).copy_from(...)`; vec_write_all(): calls `reserve_and_pad(...)` then `unsafe { pos = vec_write_all_unchecked(pos, vec, buf); if pos > vec.len() { vec.set_len(pos); } }` with comment "Safety: we have ensured that the capacity is available and that all bytes get written up to pos"

**Implementation:** Wrap Vec write preparation in a helper that returns a typestate token/view proving preconditions, e.g. `struct PreparedVecWrite<'a, A> { vec: &'a mut Vec<u8, A>, pos: usize }`. `reserve_and_pad` would return `io::Result<PreparedVecWrite>`. Only `PreparedVecWrite` exposes an unsafe-free `write_all(buf)` that performs the raw copy internally, preventing calling `vec_write_all_unchecked` without prior reservation/padding.

---

### 24. Stdin locking protocol (Unlocked handle -> Locked guard for BufRead)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-471`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The API relies on an implicit locking protocol: the global stdin buffer must be locked to safely/efficiently use full BufRead functionality (e.g. lines(), read_line()) and to synchronize reads across threads. This is mostly achieved via Stdin::lock() returning StdinLock, but Stdin itself also implements Read by internally calling lock() per operation, meaning callers can accidentally mix 'implicit per-call locking' with an existing long-lived lock, causing unintended interleavings/latency. The type system does not distinguish 'I am holding the stdin lock' from 'I am not', and does not prevent using the implicitly-locking Read impl in contexts where a locked session is required for protocol/atomicity.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct Stderr, 1 free function(s), impl Stderr (1 methods), impl UnwindSafe for Stderr (0 methods), impl RefUnwindSafe for Stderr (0 methods), impl Write for Stderr (7 methods), impl Write for & Stderr (7 methods), impl UnwindSafe for StderrLock < '_ > (0 methods), impl RefUnwindSafe for StderrLock < '_ > (0 methods), impl Write for StderrLock < '_ > (6 methods); struct StderrLock

#![cfg_attr(test, allow(unused))]

#[cfg(test)]
mod tests;

use crate::cell::{Cell, RefCell};
use crate::fmt;
use crate::fs::File;
use crate::io::prelude::*;
use crate::io::{
    self, BorrowedCursor, BufReader, IoSlice, IoSliceMut, LineWriter, Lines, SpecReadByte,
};
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::sync::{Arc, Mutex, MutexGuard, OnceLock, ReentrantLock, ReentrantLockGuard};
use crate::sys::stdio;
use crate::thread::AccessError;

type LocalStream = Arc<Mutex<Vec<u8>>>;

thread_local! {
    /// Used by the test crate to capture the output of the print macros and panics.
    static OUTPUT_CAPTURE: Cell<Option<LocalStream>> = const {
        Cell::new(None)
    }
}

/// Flag to indicate OUTPUT_CAPTURE is used.
///
/// If it is None and was never set on any thread, this flag is set to false,
/// and OUTPUT_CAPTURE can be safely ignored on all threads, saving some time
/// and memory registering an unused thread local.
///
/// Note about memory ordering: This contains information about whether a
/// thread local variable might be in use. Although this is a global flag, the
/// memory ordering between threads does not matter: we only want this flag to
/// have a consistent order between set_output_capture and print_to *within
/// the same thread*. Within the same thread, things always have a perfectly
/// consistent order. So Ordering::Relaxed is fine.
static OUTPUT_CAPTURE_USED: Atomic<bool> = AtomicBool::new(false);

/// A handle to a raw instance of the standard input stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdin_raw` function.
struct StdinRaw(stdio::Stdin);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stdout_raw` function.
struct StdoutRaw(stdio::Stdout);

/// A handle to a raw instance of the standard output stream of this process.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `std::io::stdio::stderr_raw` function.
struct StderrRaw(stdio::Stderr);

/// Constructs a new raw handle to the standard input of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdin`. Data buffered by the `std::io::stdin`
/// handles is **not** available to raw handles returned from this function.
///
/// The returned handle has no external synchronization or buffering.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdin_raw() -> StdinRaw {
    StdinRaw(stdio::Stdin::new())
}

/// Constructs a new raw handle to the standard output stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stdout`. Note that data is buffered by the
/// `std::io::stdout` handles so writes which happen via this raw handle may
/// appear before previous writes.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stdout_raw() -> StdoutRaw {
    StdoutRaw(stdio::Stdout::new())
}

/// Constructs a new raw handle to the standard error stream of this process.
///
/// The returned handle does not interact with any other handles created nor
/// handles returned by `std::io::stderr`.
///
/// The returned handle has no external synchronization or buffering layered on
/// top.
#[unstable(feature = "libstd_sys_internals", issue = "none")]
const fn stderr_raw() -> StderrRaw {
    StderrRaw(stdio::Stderr::new())
}

impl Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        handle_ebadf(self.0.read(buf), || Ok(0))
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        handle_ebadf(self.0.read_buf(buf), || Ok(()))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        handle_ebadf(self.0.read_vectored(bufs), || Ok(0))
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }
        handle_ebadf(self.0.read_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        if buf.capacity() == 0 {
            return Ok(());
        }
        handle_ebadf(self.0.read_buf_exact(buf), || Err(io::Error::READ_EXACT_EOF))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_end(buf), || Ok(0))
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        handle_ebadf(self.0.read_to_string(buf), || Ok(0))
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

impl Write for StderrRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        handle_ebadf(self.0.write(buf), || Ok(buf.len()))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let total = || Ok(bufs.iter().map(|b| b.len()).sum());
        handle_ebadf(self.0.write_vectored(bufs), total)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        handle_ebadf(self.0.flush(), || Ok(()))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        handle_ebadf(self.0.write_all(buf), || Ok(()))
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        handle_ebadf(self.0.write_all_vectored(bufs), || Ok(()))
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        handle_ebadf(self.0.write_fmt(fmt), || Ok(()))
    }
}

fn handle_ebadf<T>(r: io::Result<T>, default: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    match r {
        Err(ref e) if stdio::is_ebadf(e) => default(),
        r => r,
    }
}

/// A handle to the standard input stream of a process.
///
/// Each handle is a shared reference to a global buffer of input data to this
/// process. A handle can be `lock`'d to gain full access to [`BufRead`] methods
/// (e.g., `.lines()`). Reads to this handle are otherwise locked with respect
/// to other reads.
///
/// This handle implements the `Read` trait, but beware that concurrent reads
/// of `Stdin` must be executed with care.
///
/// Created by the [`io::stdin`] method.
///
/// [`io::stdin`]: stdin
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     stdin.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Stdin")]
pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
}

/// A locked reference to the [`Stdin`] handle.
///
/// This handle implements both the [`Read`] and [`BufRead`] traits, and
/// is constructed via the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin(); // We get `Stdin` here.
///     {
///         let mut handle = stdin.lock(); // We get `StdinLock` here.
///         handle.read_line(&mut buffer)?;
///     } // `StdinLock` is dropped here.
///     Ok(())
/// }
/// ```
#[must_use = "if unused stdin will immediately unlock"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<StdinRaw>>,
}

/// Constructs a new handle to the standard input of the current process.
///
/// Each handle returned is a reference to a shared global buffer whose access
/// is synchronized via a mutex. If you need more explicit control over
/// locking, see the [`Stdin::lock`] method.
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to read bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
///
/// # Examples
///
/// Using implicit synchronization:
///
/// ```no_run
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     io::stdin().read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
///
/// Using explicit synchronization:
///
/// ```no_run
/// use std::io::{self, BufRead};
///
/// fn main() -> io::Result<()> {
///     let mut buffer = String::new();
///     let stdin = io::stdin();
///     let mut handle = stdin.lock();
///
///     handle.read_line(&mut buffer)?;
///     Ok(())
/// }
/// ```
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn stdin() -> Stdin {
    static INSTANCE: OnceLock<Mutex<BufReader<StdinRaw>>> = OnceLock::new();
    Stdin {
        inner: INSTANCE.get_or_init(|| {
            Mutex::new(BufReader::with_capacity(stdio::STDIN_BUF_SIZE, stdin_raw()))
        }),
    }
}

impl Stdin {
    /// Locks this handle to the standard input stream, returning a readable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Read`] and [`BufRead`] traits for
    /// accessing the underlying data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, BufRead};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut buffer = String::new();
    ///     let stdin = io::stdin();
    ///     let mut handle = stdin.lock();
    ///
    ///     handle.read_line(&mut buffer)?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdinLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `Mutex` is static.
        StdinLock { inner: self.inner.lock().unwrap_or_else(|e| e.into_inner()) }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::read_line`]. In particular:
    /// * Previous content of the buffer will be preserved. To avoid appending
    ///   to the buffer, you need to [`clear`] it first.
    /// * The trailing newline character, if any, is included in the buffer.
    ///
    /// [`clear`]: String::clear
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let mut input = String::new();
    /// match io::stdin().read_line(&mut input) {
    ///     Ok(n) => {
    ///         println!("{n} bytes read");
    ///         println!("{input}");
    ///     }
    ///     Err(error) => println!("error: {error}"),
    /// }
    /// ```
    ///
    /// You can run the example one of two ways:
    ///
    /// - Pipe some text to it, e.g., `printf foo | path/to/executable`
    /// - Give it text interactively by running the executable directly,
    ///   in which case it will wait for the Enter key to be pressed before
    ///   continuing
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_confusables("get_line")]
    pub fn read_line(&self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_line(buf)
    }

    /// Consumes this handle and returns an iterator over input lines.
    ///
    /// For detailed semantics of this method, see the documentation on
    /// [`BufRead::lines`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    ///
    /// let lines = io::stdin().lines();
    /// for line in lines {
    ///     println!("got a line: {}", line.unwrap());
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "stdin_forwarders", since = "1.62.0")]
    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdin").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lock().read(buf)
    }
    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.lock().read_buf(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.lock().read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.lock().is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.lock().read_to_end(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.lock().read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.lock().read_exact(buf)
    }
    fn read_buf_exact(&mut self,
// ... (truncated) ...
```

**Entity:** Stdin (and StdinLock<'a>)

**States:** Unlocked (shared handle), Locked (exclusive access via StdinLock)

**Transitions:**
- Unlocked -> Locked via Stdin::lock()
- Locked -> Unlocked via drop(StdinLock)

**Evidence:** field: Stdin { inner: &'static Mutex<BufReader<StdinRaw>> } — global mutex-backed state; struct: StdinLock<'a> { inner: MutexGuard<'a, BufReader<StdinRaw>> } — represents the locked state; comment on Stdin: "A handle can be `lock`'d to gain full access to BufRead methods" and "Reads to this handle are otherwise locked"; method: Stdin::lock(&self) -> StdinLock<'static> — explicit state transition into locked guard; impl Read for Stdin: read/read_exact/etc call self.lock().<op>() — implicit per-call locking hides the protocol boundary

**Implementation:** Expose a distinct capability/token representing an active stdin session (e.g., `StdinSession<'a>(StdinLock<'a>)`) and move higher-level buffered operations (lines/read_line) onto that session type only; optionally avoid implementing `Read for Stdin` (or gate it behind a wrapper) so callers must explicitly choose between `stdin().lock()` (session) and a separate 'one-shot' reader type.

---

### 32. Stderr global-handle + locking protocol (Unlocked -> Locked guard)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-135`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Stderr is a thin handle to a global ('static) stderr instance that must be locked to perform actual writes safely/atomically. The API relies on a runtime locking protocol: Stderr::lock() produces a StderrLock guard that grants mutable access to the underlying StderrRaw via interior mutability. Separately, the implementation assumes (but does not encode in the type system) that the underlying lock is 'static; lock() returns StderrLock<'static> based on an implementation detail comment rather than a type-level guarantee tying the lifetime to the actual storage duration.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct StdinRaw, 1 free function(s), impl Read for StdinRaw (8 methods); struct StdoutRaw, 1 free function(s), impl Write for StdoutRaw (7 methods); struct StderrRaw, 1 free function(s), impl Write for StderrRaw (7 methods); struct Stdin, 1 free function(s), impl Stdin (3 methods), impl Read for Stdin (8 methods), impl Read for & Stdin (8 methods), impl StdinLock < '_ > (1 methods), impl Read for StdinLock < '_ > (8 methods), impl SpecReadByte for StdinLock < '_ > (1 methods), impl BufRead for StdinLock < '_ > (4 methods); struct StdinLock; struct Stdout, 1 free function(s), impl Stdout (1 methods), impl UnwindSafe for Stdout (0 methods), impl RefUnwindSafe for Stdout (0 methods), impl Write for Stdout (7 methods), impl Write for & Stdout (7 methods), impl UnwindSafe for StdoutLock < '_ > (0 methods), impl RefUnwindSafe for StdoutLock < '_ > (0 methods), impl Write for StdoutLock < '_ > (6 methods); struct StdoutLock; struct StderrLock; trait IsTerminal, 9 free function(s)

/// `Write` will do nothing and silently succeed. All other I/O operations, via the
/// standard library or via raw Windows API calls, will fail.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Stderr {
    inner: &'static ReentrantLock<RefCell<StderrRaw>>,
}

// ... (other code) ...

    Stderr { inner: &INSTANCE }
}

impl Stderr {
    /// Locks this handle to the standard error stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the [`Write`] trait for writing data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{self, Write};
    ///
    /// fn foo() -> io::Result<()> {
    ///     let stderr = io::stderr();
    ///     let mut handle = stderr.lock();
    ///
    ///     handle.write_all(b"hello world")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StderrLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `ReentrantMutex` is
        // static.
        StderrLock { inner: self.inner.lock() }
    }
}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl UnwindSafe for Stderr {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for Stderr {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self).write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (&*self).write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        io::Write::is_write_vectored(&&*self)
    }
    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (&*self).write_all(buf)
    }
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        (&*self).write_all_vectored(bufs)
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        (&*self).write_fmt(args)
    }
}

#[stable(feature = "write_mt", since = "1.48.0")]
impl Write for &Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.lock().write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.lock().write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.lock().is_write_vectored()
    }
    fn flush(&mut self) -> io::Result<()> {
        self.lock().flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.lock().write_all(buf)
    }
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        self.lock().write_all_vectored(bufs)
    }
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        self.lock().write_fmt(args)
    }
}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl UnwindSafe for StderrLock<'_> {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for StderrLock<'_> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for StderrLock<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.inner.borrow_mut().write_vectored(bufs)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.inner.borrow_mut().is_write_vectored()
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.borrow_mut().write_all(buf)
    }
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        self.inner.borrow_mut().write_all_vectored(bufs)
    }
}

```

**Entity:** Stderr

**States:** Unlocked (handle only), Locked (has StderrLock guard)

**Transitions:**
- Unlocked -> Locked via Stderr::lock()
- Locked -> Unlocked via Drop of StderrLock (end of scope)

**Evidence:** field: Stderr { inner: &'static ReentrantLock<RefCell<StderrRaw>> } encodes that the handle points at a global lock-protected resource; method: pub fn lock(&self) -> StderrLock<'static> returns a guard, establishing a lock acquisition protocol; comment in lock(): "Locks this handle with 'static lifetime. This depends on the implementation detail that the underlying `ReentrantMutex` is static." — lifetime/state assumption not enforced by types; impl Write for &Stderr: each write/flush method does `self.lock().write_*` indicating writes are intended to occur under a lock guard

**Implementation:** Encode the 'this is the global instance' assumption as a capability/token type, e.g. `struct GlobalStderr(&'static ReentrantLock<...>);` produced only by `stderr()`, and have `lock(&GlobalStderr) -> StderrLock<'static>`; alternatively make `Stderr` itself a zero-sized global token with no borrowable lifetime, and keep the actual `'static` reference private so callers cannot construct other instances. This moves the "underlying lock is static" precondition out of comments and into construction/visibility invariants.

---

### 1. BufWriter panic-recovery payload protocol (Normal buffered data -> Panicked/unknown write state)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-29`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: WriterPanicked represents a special post-panic recovery state for BufWriter where buffered bytes exist but the write-status of those bytes is unknown (some bytes may already have been written by the panicking underlying writer). The API relies on documentation to enforce that callers must not treat the returned Vec<u8> as simply 'unwritten data' and re-write it, because that could duplicate bytes. This is an implicit protocol/state distinction (normal buffered bytes vs. panic-recovered bytes) that is not enforced by the type system beyond the existence of this distinct error type; once you have the Vec<u8>, it is indistinguishable from ordinary buffered bytes and can be mistakenly re-used.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct BufWriter, impl BufWriter < W > (6 methods), impl BufWriter < W > (11 methods), impl Write for BufWriter < W > (5 methods), impl Seek for BufWriter < W > (1 methods), impl Drop for BufWriter < W > (1 methods)

/// assert!(matches!(recovered_writer, PanickingWriter));
/// assert_eq!(buffered_data.unwrap_err().into_inner(), b"some data");
/// ```
pub struct WriterPanicked {
    buf: Vec<u8>,
}

impl WriterPanicked {
    /// Returns the perhaps-unwritten data.  Some of this data may have been written by the
    /// panicking call(s) to the underlying writer, so simply writing it again is not a good idea.
    #[must_use = "`self` will be dropped if the result is not used"]
    #[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    const DESCRIPTION: &'static str =
        "BufWriter inner writer panicked, what data remains unwritten is not known";
}

#[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
impl error::Error for WriterPanicked {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

```

**Entity:** WriterPanicked

**States:** Normal buffered data (safe to flush/write), Panicked/unknown (may be partially written; unsafe to re-write blindly)

**Transitions:**
- Normal buffered data (safe) -> Panicked/unknown via underlying writer panicking (recovered as WriterPanicked carrying buf)
- Panicked/unknown -> raw bytes via WriterPanicked::into_inner(self)

**Evidence:** struct WriterPanicked { buf: Vec<u8> } stores buffered bytes in a special error type rather than returning them as ordinary Vec<u8>; WriterPanicked::into_inner(self) doc: "Returns the perhaps-unwritten data. Some of this data may have been written ... so simply writing it again is not a good idea."; WriterPanicked::DESCRIPTION: "BufWriter inner writer panicked, what data remains unwritten is not known" encodes the 'unknown write state' invariant; WriterPanicked::into_inner has #[must_use], indicating callers are expected to handle this state explicitly

**Implementation:** Keep the bytes wrapped in a distinct type that carries the 'unknown/possibly written' taint, e.g. `struct PossiblyWritten(Vec<u8>);` and have `into_inner(self) -> PossiblyWritten` (or expose only restricted operations on it). Alternatively provide an API that forces an explicit choice: `enum Recovery { Discard, Inspect(PossiblyWritten) }`, making accidental re-write less likely at compile time.

---

### 5. Cursor position/buffer-consistency protocol (pos must track underlying buffer)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-268`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Cursor<T> maintains an internal I/O position (`pos`) that is assumed to correspond to offsets within the underlying buffer (`inner`). The API exposes `get_mut()` which allows callers to mutate `inner` arbitrarily, but `pos` is not automatically updated to remain valid/meaningful relative to the mutated `inner` (e.g., shrinking/truncating a Vec, swapping buffers, etc.). The code/doc relies on a human protocol: if you mutate the underlying buffer in ways that change its effective length/contents layout, you must also manage/repair the cursor's `pos` (typically via `set_position`) or avoid such mutations. This state dependency is not enforced by the type system; `get_mut()` is always available and returns `&mut T` without any capability that would require re-validating `pos` before subsequent Read/Write/Seek operations.

**Evidence**:

```rust
// Note: Other parts of this module contain: 8 free function(s)

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    ///
    /// Cursor initial position is `0` even if underlying buffer (e.g., [`Vec`])
    /// is not empty. So writing to cursor starts with overwriting [`Vec`]
    /// content, not with appending to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }

    /// Consumes this cursor, returning the underlying value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let vec = buff.into_inner();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the underlying value in this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_ref();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying value in this cursor.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying value as it may corrupt this cursor's position.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(Vec::new());
    /// # fn force_inference(_: &Cursor<Vec<u8>>) {}
    /// # force_inference(&buff);
    ///
    /// let reference = buff.get_mut();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the current position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use std::io::prelude::*;
    /// use std::io::SeekFrom;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.seek(SeekFrom::Current(2)).unwrap();
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.seek(SeekFrom::Current(-1)).unwrap();
    /// assert_eq!(buff.position(), 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_io_structs", since = "1.79.0")]
    pub const fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// let mut buff = Cursor::new(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(buff.position(), 0);
    ///
    /// buff.set_position(2);
    /// assert_eq!(buff.position(), 2);
    ///
    /// buff.set_position(4);
    /// assert_eq!(buff.position(), 4);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_mut_cursor", since = "1.86.0")]
    pub const fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T> Cursor<T>
where

// ... (other code) ...

    }
}

impl<T> Cursor<T>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> io::Seek for Cursor<T>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Read for Cursor<T>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> BufRead for Cursor<T>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Cursor<&mut [u8]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, self.inner, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        slice_write_vectored(&mut self.pos, self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        slice_write_all(&mut self.pos, self.inner, buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        slice_write_all_vectored(&mut self.pos, self.inner, bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[stable(feature = "cursor_mut_vec", since = "1.25.0")]
impl<A> Write for Cursor<&mut Vec<u8, A>>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<A> Write for Cursor<Vec<u8, A>>
where

// ... (other code) ...

}

#[stable(feature = "cursor_box_slice", since = "1.5.0")]
impl<A> Write for Cursor<Box<[u8], A>>
where

// ... (other code) ...

}

#[stable(feature = "cursor_array", since = "1.61.0")]
impl<const N: usize> Write for Cursor<[u8; N]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        slice_write_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        slice_write_all(&mut self.pos, &mut self.inner, buf)
    }

    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        slice_write_all_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

```

**Entity:** Cursor<T>

**States:** PositionConsistentWithInner, PositionPossiblyInvalidAfterInnerMutation

**Transitions:**
- PositionConsistentWithInner -> PositionPossiblyInvalidAfterInnerMutation via get_mut() (followed by arbitrary &mut T modifications)
- PositionPossiblyInvalidAfterInnerMutation -> PositionConsistentWithInner via set_position() (caller re-establishes a meaningful position)

**Evidence:** field: `pos: u64` stores cursor position separately from `inner: T`; method: `pub const fn get_mut(&mut self) -> &mut T` exposes direct mutable access to `inner`; doc on get_mut(): "Care should be taken to avoid modifying the internal I/O state of the underlying value as it may corrupt this cursor's position."; method: `pub const fn set_position(&mut self, pos: u64)` allows manual repair/override of `pos` independent of `inner`

**Implementation:** Restrict direct `&mut T` access behind a capability that temporarily suspends I/O invariants. For example, expose `fn with_inner_mut<R>(&mut self, f: impl FnOnce(&mut T, &mut u64) -> R) -> R` so any mutation that can affect length/offsets has access to also update `pos`. Alternatively provide a separate wrapper type `CursorInnerMut<'a, T>` (capability) that, on Drop, forces revalidation/clamping of `pos` (or requires the closure to return a validated position).

---

### 8. PipeReader file-descriptor sharing protocol (Original / Cloned handles)

**Location**: `/tmp/io_test_crate/src/io/pipe.rs:1-132`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: PipeReader is a wrapper around an underlying OS pipe handle (AnonPipe). Calling try_clone() creates another PipeReader that refers to the same underlying file description/pipe endpoint. After cloning, multiple PipeReader values share read-side state (e.g., reads from one affect what the others can read; dropping one may or may not close the underlying OS resource depending on refcounting). This 'shared underlying description' protocol is documented but not represented in the type system; PipeReader does not distinguish 'unique' from 'shared' ownership or provide capabilities to prevent unintended aliasing patterns at compile time.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct PipeWriter, impl FromInner < AnonPipe > for PipeWriter (1 methods), impl IntoInner < AnonPipe > for PipeWriter (1 methods), impl PipeWriter (1 methods), impl io :: Write for & PipeWriter (4 methods), impl io :: Write for PipeWriter (4 methods)

/// Read end of an anonymous pipe.
#[stable(feature = "anonymous_pipe", since = "1.87.0")]
#[derive(Debug)]
pub struct PipeReader(pub(crate) AnonPipe);


// ... (other code) ...

#[derive(Debug)]
pub struct PipeWriter(pub(crate) AnonPipe);

impl FromInner<AnonPipe> for PipeReader {
    fn from_inner(inner: AnonPipe) -> Self {
        Self(inner)
    }
}

impl IntoInner<AnonPipe> for PipeReader {
    fn into_inner(self) -> AnonPipe {
        self.0
    }
}

// ... (other code) ...

    }
}

impl PipeReader {
    /// Creates a new [`PipeReader`] instance that shares the same underlying file description.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(miri)] fn main() {}
    /// # #[cfg(not(miri))]
    /// # fn main() -> std::io::Result<()> {
    /// use std::fs;
    /// use std::io::{pipe, Write};
    /// use std::process::Command;
    /// const NUM_SLOT: u8 = 2;
    /// const NUM_PROC: u8 = 5;
    /// const OUTPUT: &str = "work.txt";
    ///
    /// let mut jobs = vec![];
    /// let (reader, mut writer) = pipe()?;
    ///
    /// // Write NUM_SLOT characters the pipe.
    /// writer.write_all(&[b'|'; NUM_SLOT as usize])?;
    ///
    /// // Spawn several processes that read a character from the pipe, do some work, then
    /// // write back to the pipe. When the pipe is empty, the processes block, so only
    /// // NUM_SLOT processes can be working at any given time.
    /// for _ in 0..NUM_PROC {
    ///     jobs.push(
    ///         Command::new("bash")
    ///             .args(["-c",
    ///                 &format!(
    ///                      "read -n 1\n\
    ///                       echo -n 'x' >> '{OUTPUT}'\n\
    ///                       echo -n '|'",
    ///                 ),
    ///             ])
    ///             .stdin(reader.try_clone()?)
    ///             .stdout(writer.try_clone()?)
    ///             .spawn()?,
    ///     );
    /// }
    ///
    /// // Wait for all jobs to finish.
    /// for mut job in jobs {
    ///     job.wait()?;
    /// }
    ///
    /// // Check our work and clean up.
    /// let xs = fs::read_to_string(OUTPUT)?;
    /// fs::remove_file(OUTPUT)?;
    /// assert_eq!(xs, "x".repeat(NUM_PROC.into()));
    /// # Ok(())
    /// # }
    /// ```
    #[stable(feature = "anonymous_pipe", since = "1.87.0")]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }
}

// ... (other code) ...

}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Read for &PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }
    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }
}

#[stable(feature = "anonymous_pipe", since = "1.87.0")]
impl io::Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }
    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }
}

```

**Entity:** PipeReader

**States:** Unique handle, Shared handle(s) (after try_clone)

**Transitions:**
- Unique handle -> Shared handle(s) via PipeReader::try_clone()

**Evidence:** comment on PipeReader::try_clone: "shares the same underlying file description"; PipeReader::try_clone(&self) -> io::Result<Self> creates a new PipeReader from the same underlying AnonPipe (self.0.try_clone().map(Self)); struct PipeReader(pub(crate) AnonPipe) shows it's just a thin wrapper and does not encode uniqueness/sharing in its type

**Implementation:** Introduce a state parameter indicating sharing, e.g., PipeReader<Unique> produced by pipe(); try_clone(&PipeReader<Unique>) -> PipeReader<Shared> and also transitions the original to Shared (or require cloning via an explicit Shared wrapper such as Arc-like handle). This makes it explicit when multiple handles may interact through shared underlying pipe state.

---

### 36. LineWriter buffer ownership/consistency protocol (Buffered-only vs External writes interleaved)

**Location**: `/tmp/io_test_crate/src/io/buffered/linewriter.rs:1-155`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: LineWriter relies on the invariant that all writes to the underlying writer flow through the LineWriter/BufWriter buffering logic (and its newline-triggered flushing via LineWriterShim). Calling get_mut() hands out &mut W, enabling arbitrary direct writes that can interleave with buffered data already held in BufWriter. This can violate the implicit protocol 'do not write directly to the underlying writer while LineWriter has buffered data', potentially causing stream corruption/out-of-order output. The type system does not prevent obtaining &mut W nor does it provide a capability that restricts what can be done while a buffer may be non-empty.

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct LineWriter<W: ?Sized + Write> {
    inner: BufWriter<W>,
}

impl<W: Write> LineWriter<W> {
    /// Creates a new `LineWriter`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::new(file);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: W) -> LineWriter<W> {
        // Lines typically aren't that long, don't use a giant buffer
        LineWriter::with_capacity(1024, inner)
    }

    /// Creates a new `LineWriter` with at least the specified capacity for the
    /// internal buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::with_capacity(100, file);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: W) -> LineWriter<W> {
        LineWriter { inner: BufWriter::with_capacity(capacity, inner) }
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// Caution must be taken when calling methods on the mutable reference
    /// returned as extra writes could corrupt the output stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let mut file = LineWriter::new(file);
    ///
    ///     // we can use reference just like file
    ///     let reference = file.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    /// Unwraps this `LineWriter`, returning the underlying writer.
    ///
    /// The internal buffer is written out before returning the writer.
    ///
    /// # Errors
    ///
    /// An [`Err`] will be returned if an error occurs while flushing the buffer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///
    ///     let writer: LineWriter<File> = LineWriter::new(file);
    ///
    ///     let file: File = writer.into_inner()?;
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> Result<W, IntoInnerError<LineWriter<W>>> {
        self.inner.into_inner().map_err(|err| err.new_wrapped(|inner| LineWriter { inner }))
    }
}

impl<W: ?Sized + Write> LineWriter<W> {
    /// Gets a reference to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use std::io::LineWriter;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let file = File::create("poem.txt")?;
    ///     let file = LineWriter::new(file);
    ///
    ///     let reference = file.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        LineWriterShim::new(&mut self.inner).write_vectored(bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all(buf)
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_all_vectored(bufs)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        LineWriterShim::new(&mut self.inner).write_fmt(fmt)
    }
}

```

**Entity:** LineWriter<W>

**States:** ConsistentBufferedAccess, PotentiallyCorruptedByExternalWrites

**Transitions:**
- ConsistentBufferedAccess -> PotentiallyCorruptedByExternalWrites via get_mut() (then writing through returned &mut W)
- PotentiallyCorruptedByExternalWrites -> ConsistentBufferedAccess only by convention (e.g., user avoids direct writes / flushes appropriately), not enforced

**Evidence:** field: inner: BufWriter<W> (buffering implies internal queued writes can exist); method: pub fn get_mut(&mut self) -> &mut W { self.inner.get_mut() } (exposes underlying writer mutably); doc comment on get_mut(): "Caution must be taken ... extra writes could corrupt the output stream." (explicitly states the latent invariant); Write impl routes through LineWriterShim::new(&mut self.inner).write* (indicates intended exclusive write path for correctness)

**Implementation:** Replace/augment get_mut() with an API that requires an explicit capability/token representing 'buffer is empty' (e.g., a method `into_unbuffered(self) -> (W, ...)` or `with_underlying_mut(|w| ...)` that flushes first and only exposes a scoped wrapper permitting writes while buffer is known empty). Alternatively, provide `get_mut_unchecked` as unsafe and a safe `get_mut_flushed(&mut self) -> io::Result<&mut W>` that flushes before handing out &mut W.

---

### 17. BorrowedBuf initialization/filledness protocol (Uninit -> PartiallyInit -> Filled -> Cleared)

**Location**: `/tmp/io_test_crate/src/io/copy.rs:1-297`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The code relies on a multi-step protocol for safe I/O with partially-uninitialized buffers: create a BorrowedBuf over MaybeUninit storage, optionally set how many bytes are already initialized (set_init), pass only the unfilled region to read_buf, then treat only the filled region as initialized for writing, and finally clear before reuse. This protocol is enforced partly by BorrowedBuf APIs but still has a latent invariant carried in a separate runtime variable (`init`) and an unsafe `set_len` that assumes the protocol was followed exactly. The type system does not encode that `init` corresponds to the current buffer instance/iteration, nor that `set_len` is only called with bytes that were actually written/initialized in the immediately preceding read.

**Evidence**:

```rust
use super::{BorrowedBuf, BufReader, BufWriter, DEFAULT_BUF_SIZE, Read, Result, Write};
use crate::alloc::Allocator;
use crate::cmp;
use crate::collections::VecDeque;
use crate::io::IoSlice;
use crate::mem::MaybeUninit;

#[cfg(test)]
mod tests;

/// Copies the entire contents of a reader into a writer.
///
/// This function will continuously read data from `reader` and then
/// write it into `writer` in a streaming fashion until `reader`
/// returns EOF.
///
/// On success, the total number of bytes that were copied from
/// `reader` to `writer` is returned.
///
/// If you want to copy the contents of one file to another and you’re
/// working with filesystem paths, see the [`fs::copy`] function.
///
/// [`fs::copy`]: crate::fs::copy
///
/// # Errors
///
/// This function will return an error immediately if any call to [`read`] or
/// [`write`] returns an error. All instances of [`ErrorKind::Interrupted`] are
/// handled by this function and the underlying operation is retried.
///
/// [`read`]: Read::read
/// [`write`]: Write::write
/// [`ErrorKind::Interrupted`]: crate::io::ErrorKind::Interrupted
///
/// # Examples
///
/// ```
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut reader: &[u8] = b"hello";
///     let mut writer: Vec<u8> = vec![];
///
///     io::copy(&mut reader, &mut writer)?;
///
///     assert_eq!(&b"hello"[..], &writer[..]);
///     Ok(())
/// }
/// ```
///
/// # Platform-specific behavior
///
/// On Linux (including Android), this function uses `copy_file_range(2)`,
/// `sendfile(2)` or `splice(2)` syscalls to move data directly between file
/// descriptors if possible.
///
/// Note that platform-specific behavior [may change in the future][changes].
///
/// [changes]: crate::io#platform-specific-behavior
#[stable(feature = "rust1", since = "1.0.0")]
pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "android"))] {
            crate::sys::kernel_copy::copy_spec(reader, writer)
        } else {
            generic_copy(reader, writer)
        }
    }
}

/// The userspace read-write-loop implementation of `io::copy` that is used when
/// OS-specific specializations for copy offloading are not available or not applicable.
pub(crate) fn generic_copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    let read_buf = BufferedReaderSpec::buffer_size(reader);
    let write_buf = BufferedWriterSpec::buffer_size(writer);

    if read_buf >= DEFAULT_BUF_SIZE && read_buf >= write_buf {
        return BufferedReaderSpec::copy_to(reader, writer);
    }

    BufferedWriterSpec::copy_from(writer, reader)
}

/// Specialization of the read-write loop that reuses the internal
/// buffer of a BufReader. If there's no buffer then the writer side
/// should be used instead.
trait BufferedReaderSpec {
    fn buffer_size(&self) -> usize;

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64>;
}

impl<T> BufferedReaderSpec for T
where
    Self: Read,
    T: ?Sized,
{
    #[inline]
    default fn buffer_size(&self) -> usize {
        0
    }

    default fn copy_to(&mut self, _to: &mut (impl Write + ?Sized)) -> Result<u64> {
        unreachable!("only called from specializations")
    }
}

impl BufferedReaderSpec for &[u8] {
    fn buffer_size(&self) -> usize {
        // prefer this specialization since the source "buffer" is all we'll ever need,
        // even if it's small
        usize::MAX
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        to.write_all(self)?;
        *self = &self[len..];
        Ok(len as u64)
    }
}

impl<A: Allocator> BufferedReaderSpec for VecDeque<u8, A> {
    fn buffer_size(&self) -> usize {
        // prefer this specialization since the source "buffer" is all we'll ever need,
        // even if it's small
        usize::MAX
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        let (front, back) = self.as_slices();
        let bufs = &mut [IoSlice::new(front), IoSlice::new(back)];
        to.write_all_vectored(bufs)?;
        self.clear();
        Ok(len as u64)
    }
}

impl<I> BufferedReaderSpec for BufReader<I>
where
    Self: Read,
    I: ?Sized,
{
    fn buffer_size(&self) -> usize {
        self.capacity()
    }

    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let mut len = 0;

        loop {
            // Hack: this relies on `impl Read for BufReader` always calling fill_buf
            // if the buffer is empty, even for empty slices.
            // It can't be called directly here since specialization prevents us
            // from adding I: Read
            match self.read(&mut []) {
                Ok(_) => {}
                Err(e) if e.is_interrupted() => continue,
                Err(e) => return Err(e),
            }
            let buf = self.buffer();
            if self.buffer().len() == 0 {
                return Ok(len);
            }

            // In case the writer side is a BufWriter then its write_all
            // implements an optimization that passes through large
            // buffers to the underlying writer. That code path is #[cold]
            // but we're still avoiding redundant memcopies when doing
            // a copy between buffered inputs and outputs.
            to.write_all(buf)?;
            len += buf.len() as u64;
            self.discard_buffer();
        }
    }
}

/// Specialization of the read-write loop that either uses a stack buffer
/// or reuses the internal buffer of a BufWriter
trait BufferedWriterSpec: Write {
    fn buffer_size(&self) -> usize;

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64>;
}

impl<W: Write + ?Sized> BufferedWriterSpec for W {
    #[inline]
    default fn buffer_size(&self) -> usize {
        0
    }

    default fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        stack_buffer_copy(reader, self)
    }
}

impl<I: Write + ?Sized> BufferedWriterSpec for BufWriter<I> {
    fn buffer_size(&self) -> usize {
        self.capacity()
    }

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        if self.capacity() < DEFAULT_BUF_SIZE {
            return stack_buffer_copy(reader, self);
        }

        let mut len = 0;
        let mut init = 0;

        loop {
            let buf = self.buffer_mut();
            let mut read_buf: BorrowedBuf<'_> = buf.spare_capacity_mut().into();

            unsafe {
                // SAFETY: init is either 0 or the init_len from the previous iteration.
                read_buf.set_init(init);
            }

            if read_buf.capacity() >= DEFAULT_BUF_SIZE {
                let mut cursor = read_buf.unfilled();
                match reader.read_buf(cursor.reborrow()) {
                    Ok(()) => {
                        let bytes_read = cursor.written();

                        if bytes_read == 0 {
                            return Ok(len);
                        }

                        init = read_buf.init_len() - bytes_read;
                        len += bytes_read as u64;

                        // SAFETY: BorrowedBuf guarantees all of its filled bytes are init
                        unsafe { buf.set_len(buf.len() + bytes_read) };

                        // Read again if the buffer still has enough capacity, as BufWriter itself would do
                        // This will occur if the reader returns short reads
                    }
                    Err(ref e) if e.is_interrupted() => {}
                    Err(e) => return Err(e),
                }
            } else {
                // All the bytes that were already in the buffer are initialized,
                // treat them as such when the buffer is flushed.
                init += buf.len();

                self.flush_buf()?;
            }
        }
    }
}

impl BufferedWriterSpec for Vec<u8> {
    fn buffer_size(&self) -> usize {
        cmp::max(DEFAULT_BUF_SIZE, self.capacity() - self.len())
    }

    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        reader.read_to_end(self).map(|bytes| u64::try_from(bytes).expect("usize overflowed u64"))
    }
}

pub fn stack_buffer_copy<R: Read + ?Sized, W: Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); DEFAULT_BUF_SIZE];
    let mut buf: BorrowedBuf<'_> = buf.into();

    let mut len = 0;

    loop {
        match reader.read_buf(buf.unfilled()) {
            Ok(()) => {}
            Err(e) if e.is_interrupted() => continue,
            Err(e) => return Err(e),
        };

        if buf.filled().is_empty() {
            break;
        }

        len += buf.filled().len() as u64;
        writer.write_all(buf.filled())?;
        buf.clear();
    }

    Ok(len)
}

```

**Entity:** BorrowedBuf<'_> (as used in stack_buffer_copy and BufWriter<I>::copy_from)

**States:** Empty(Unfilled, init_len tracked), HasUnfilledCapacity, Filled(has initialized bytes), Cleared(reset for reuse)

**Transitions:**
- Empty/HasUnfilledCapacity -> Filled via reader.read_buf(buf.unfilled()) / reader.read_buf(cursor.reborrow())
- Filled -> Cleared via buf.clear() (stack_buffer_copy)
- Filled -> (BufWriter internal length advanced) via unsafe buf.set_len(buf.len() + bytes_read) (BufWriter<I>::copy_from)
- HasUnfilledCapacity -> Empty (flush/rotate) via self.flush_buf()? (BufWriter<I>::copy_from) when capacity is low

**Evidence:** stack_buffer_copy: `let buf: &mut [_] = &mut [MaybeUninit::uninit(); DEFAULT_BUF_SIZE]; let mut buf: BorrowedBuf<'_> = buf.into();` (BorrowedBuf over uninitialized memory); stack_buffer_copy: `reader.read_buf(buf.unfilled())` followed by `writer.write_all(buf.filled())` (must only write initialized/filled bytes); stack_buffer_copy: `buf.clear();` (explicit reset step required before next read); BufWriter<I>::copy_from: `let mut init = 0;` carried across loop iterations as separate state; BufWriter<I>::copy_from: `unsafe { read_buf.set_init(init); }` with comment `// SAFETY: init is either 0 or the init_len from the previous iteration.` (latent invariant about init tracking); BufWriter<I>::copy_from: `let bytes_read = cursor.written(); ... init = read_buf.init_len() - bytes_read;` (manual bookkeeping tying init to previous iteration); BufWriter<I>::copy_from: `unsafe { buf.set_len(buf.len() + bytes_read) };` with comment `// SAFETY: BorrowedBuf guarantees all of its filled bytes are init` (assumes protocol correctness)

**Implementation:** Hide the `init` bookkeeping and the unsafe `set_len` behind a small internal wrapper type that encodes states like `Spare<'a, UninitKnown>` -> `Spare<'a, ReadComplete>` and only exposes `advance_len(bytes_read)` when it has proof (by construction) that those bytes were written by the immediately-associated `read_buf` call. Concretely: make a helper `struct ReadIntoSpare<'a> { buf: BorrowedBuf<'a>, prior_init: usize }` with a method `read_from(&mut self, reader) -> Result<ReadResult>` returning an object that carries `bytes_read` and provides a safe `commit_to_vec(&mut Vec<u8>)` / `commit_to_bufwriter(&mut BufWriter<_>)` consuming that token.

---

### 44. BufReader buffer/inner coherence protocol (Buffered / Bypassed / Discarded)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-462`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: BufReader maintains an implicit coherence invariant between (a) the logical stream position the caller experiences and (b) the underlying reader's actual position, mediated by the internal buffer. Several methods rely on a protocol: buffered bytes must be consumed via BufRead::consume (or internal helpers) and certain operations (large reads, seeking, draining into a Vec/String) must first invalidate/discard the buffer before delegating to the inner reader to avoid duplicating/skipping bytes. This protocol is enforced by runtime checks/ordering in method bodies and documentation warnings (e.g., 'inadvisable to directly read from the underlying reader'), but the type system does not prevent users from violating it (e.g., calling get_mut() and reading/seek on the inner reader while BufReader still holds buffered bytes).

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BufReader<R: ?Sized> {
    buf: Buffer,
    inner: R,
}

impl<R: Read> BufReader<R> {
    /// Creates a new `BufReader<R>` with a default buffer capacity. The default is currently 8 KiB,
    /// but may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let reader = BufReader::new(f);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub(crate) fn try_new_buffer() -> io::Result<Buffer> {
        Buffer::try_with_capacity(DEFAULT_BUF_SIZE)
    }

    pub(crate) fn with_buffer(inner: R, buf: Buffer) -> Self {
        Self { inner, buf }
    }

    /// Creates a new `BufReader<R>` with the specified buffer capacity.
    ///
    /// # Examples
    ///
    /// Creating a buffer with ten bytes of capacity:
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let reader = BufReader::with_capacity(10, f);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        BufReader { inner, buf: Buffer::with_capacity(capacity) }
    }
}

impl<R: Read + ?Sized> BufReader<R> {
    /// Attempt to look ahead `n` bytes.
    ///
    /// `n` must be less than or equal to `capacity`.
    ///
    /// The returned slice may be less than `n` bytes long if
    /// end of file is reached.
    ///
    /// After calling this method, you may call [`consume`](BufRead::consume)
    /// with a value less than or equal to `n` to advance over some or all of
    /// the returned bytes.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// #![feature(bufreader_peek)]
    /// use std::io::{Read, BufReader};
    ///
    /// let mut bytes = &b"oh, hello there"[..];
    /// let mut rdr = BufReader::with_capacity(6, &mut bytes);
    /// assert_eq!(rdr.peek(2).unwrap(), b"oh");
    /// let mut buf = [0; 4];
    /// rdr.read(&mut buf[..]).unwrap();
    /// assert_eq!(&buf, b"oh, ");
    /// assert_eq!(rdr.peek(5).unwrap(), b"hello");
    /// let mut s = String::new();
    /// rdr.read_to_string(&mut s).unwrap();
    /// assert_eq!(&s, "hello there");
    /// assert_eq!(rdr.peek(1).unwrap().len(), 0);
    /// ```
    #[unstable(feature = "bufreader_peek", issue = "128405")]
    pub fn peek(&mut self, n: usize) -> io::Result<&[u8]> {
        assert!(n <= self.capacity());
        while n > self.buf.buffer().len() {
            if self.buf.pos() > 0 {
                self.buf.backshift();
            }
            let new = self.buf.read_more(&mut self.inner)?;
            if new == 0 {
                // end of file, no more bytes to read
                return Ok(&self.buf.buffer()[..]);
            }
            debug_assert_eq!(self.buf.pos(), 0);
        }
        Ok(&self.buf.buffer()[..n])
    }
}

impl<R: ?Sized> BufReader<R> {
    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.get_ref();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.get_mut();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike [`fill_buf`], this will not attempt to fill the buffer if it is empty.
    ///
    /// [`fill_buf`]: BufRead::fill_buf
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{BufReader, BufRead};
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f);
    ///     assert!(reader.buffer().is_empty());
    ///
    ///     if reader.fill_buf()?.len() > 0 {
    ///         assert!(!reader.buffer().is_empty());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "bufreader_buffer", since = "1.37.0")]
    pub fn buffer(&self) -> &[u8] {
        self.buf.buffer()
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{BufReader, BufRead};
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::open("log.txt")?;
    ///     let mut reader = BufReader::new(f);
    ///
    ///     let capacity = reader.capacity();
    ///     let buffer = reader.fill_buf()?;
    ///     assert!(buffer.len() <= capacity);
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "buffered_io_capacity", since = "1.46.0")]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Unwraps this `BufReader<R>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost. Therefore,
    /// a following read from the underlying reader may lead to data loss.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::BufReader;
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f1 = File::open("log.txt")?;
    ///     let reader = BufReader::new(f1);
    ///
    ///     let f2 = reader.into_inner();
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_inner(self) -> R
    where
        R: Sized,
    {
        self.inner
    }

    /// Invalidates all data in the internal buffer.
    #[inline]
    pub(in crate::io) fn discard_buffer(&mut self) {
        self.buf.discard_buffer()
    }
}

// This is only used by a test which asserts that the initialization-tracking is correct.
#[cfg(test)]
impl<R: ?Sized> BufReader<R> {
    #[allow(missing_docs)]
    pub fn initialized(&self) -> usize {
        self.buf.initialized()
    }
}

impl<R: ?Sized + Seek> BufReader<R> {
    /// Seeks relative to the current position. If the new position lies within the buffer,
    /// the buffer will not be flushed, allowing for more efficient seeks.
    /// This method does not return the location of the underlying reader, so the caller
    /// must track this information themselves if it is required.
    #[stable(feature = "bufreader_seek_relative", since = "1.53.0")]
    pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        let pos = self.buf.pos() as u64;
        if offset < 0 {
            if let Some(_) = pos.checked_sub((-offset) as u64) {
                self.buf.unconsume((-offset) as usize);
                return Ok(());
            }
        } else if let Some(new_pos) = pos.checked_add(offset as u64) {
            if new_pos <= self.buf.filled() as u64 {
                self.buf.consume(offset as usize);
                return Ok(());
            }
        }

        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}

impl<R> SpecReadByte for BufReader<R>
where

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.buf.pos() == self.buf.filled() && buf.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let mut rem = self.fill_buf()?;
        let nread = rem.read(buf)?;
        self.consume(nread);
        Ok(nread)
    }

    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.buf.pos() == self.buf.filled() && cursor.capacity() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_buf(cursor);
        }

        let prev = cursor.written();

        let mut rem = self.fill_buf()?;
        rem.read_buf(cursor.reborrow())?; // actually never fails

        self.consume(cursor.written() - prev); //slice impl of read_buf known to never unfill buf

        Ok(())
    }

    // Small read_exacts from a BufReader are extremely common when used with a deserializer.
    // The default implementation calls read in a loop, which results in surprisingly poor code
    // generation for the common path where the buffer has enough bytes to fill the passed-in
    // buffer.
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if self.buf.consume_with(buf.len(), |claimed| buf.copy_from_slice(claimed)) {
            return Ok(());
        }

        crate::io::default_read_exact(self, buf)
    }

    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if self.buf.consume_with(cursor.capacity(), |claimed| cursor.append(claimed)) {
            return Ok(());
        }

        crate::io::default_read_buf_exact(self, cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.buf.pos() == self.buf.filled() && total_len >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }
        let mut rem = self.fill_buf()?;
        let nread = rem.read_vectored(bufs)?;

        self.consume(nread);
        Ok(nread)
    }

    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    // The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    // delegate to the inner implementation.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let inner_buf = self.buffer();
        buf.try_reserve(inner_buf.len())?;
        buf.extend_from_slice(inner_buf);
        let nread = inner_buf.len();
        self.discard_buffer();
        Ok(nread + self.inner.read_to_end(buf)?)
    }

    // The inner reader might have an optimized `read_to_end`. Drain our buffer and then
    // delegate to the inner implementation.
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        // In the general `else` case below we must read bytes into a side buffer, check
        // that they are valid UTF-8, and then append them to `buf`. This requires a
        // potentially large memcpy.
        //
        // If `buf` is empty--the most common case--we can leverage `append_to_string`
        // to read directly into `buf`'s internal byte buffer, saving an allocation and
        // a memcpy.
        if buf.is_empty() {
            // `append_to_string`'s safety relies on the buffer only being appended to since
            // it only checks the UTF-8 validity of new data. If there were existing content in
            // `buf` then an untrustworthy reader (i.e. `self.inner`) could not only append
            // bytes but also modify existing bytes and render them invalid. On the other hand,
            // if `buf` is empty then by definition any writes must be appends and
            // `append_to_string` will validate all of the new bytes.
            unsafe { crate::io::append_to_string(buf, |b| self.read_to_end(b)) }
        } else {
            // We cannot append our byte buffer directly onto the `buf` String as there could
            // be an incomplete UTF-8 sequence that has only been partially read. We must read
            // everything into a side buffer first and then call `from_utf8` on the complete
            // buffer.
            let mut bytes = Vec::new();
            self.read_to_end(&mut bytes)?;
            let string = crate::str::from_utf8(&bytes).map_err(|_| io::Error::INVALID_UTF8)?;
            *buf += string;
            Ok(string.len())
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.buf.fill_buf(&mut self.inner)
    }

    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt)
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<R: ?Sized + Seek> Seek for BufReader<R> {
    /// Seek to an offset, in bytes, in the underlying reader.
    ///
    /// The position used for seeking with <code>[SeekFrom::Current]\(_)</code> is the
    /// position the underlying reader would be at if the `BufReader<R>` had no
    /// internal buffer.
    ///
    /// Seeking always discards the internal buffer, even if the seek position
    /// would otherwise fall within it. This guarantees that calling
    /// [`BufReader::into_inner()`] immediately after a seek yields the underlying reader
    /// at the same position.
    ///
    /// To seek without discarding the internal buffer, use [`BufReader::seek_relative`].
    ///
    /// See [`std::io::Seek`] for more details.
    ///
    /// Note: In the edge case where you're seeking with <code>[SeekFrom::Current]\(n)</code>
    /// where `n` minus the internal buffer length overflows an `i64`, two
    /// seeks will be performed instead of one. If the second seek returns
    /// [`Err`], the underlying reader will be left at the same position it would
    /// have if you called `seek` with <code>[SeekFrom::Current]\(0)</code>.
    ///
    /// [`std::io::Seek`]: Seek
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.buf.filled() - self.buf.pos()) as i64;
            // it should be safe to assume that remainder fits within an i64 as the alternative
            // means we managed to allocate 8 exbibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to
            // support seeking by i64::MIN so we need to handle underflow when subtracting
            // remainder.
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                // seek backwards by our remainder, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.discard_buffer();
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length.
            result = self.inner.seek(pos)?;
        }
        self.discard_buffer();
        Ok(result)
    }

    /// Returns the current seek position from the start of the stream.
    ///

// ... (truncated) ...
```

**Entity:** BufReader<R>

**States:** Buffered(data present, buf.pos < buf.filled), Empty(no buffered data, buf.pos == buf.filled), Bypassed(next read delegated directly to inner after discard_buffer), Invalidated(discarded; buffer must be refilled before use)

**Transitions:**
- Empty -> Buffered via fill_buf()/peek() (ultimately Buffer::fill_buf/read_more)
- Buffered -> Buffered via consume(n) / seek_relative within-buffer adjustments (consume/unconsume)
- Buffered/Empty -> Invalidated via discard_buffer() (also used before delegating to inner for bypass/seek/drain)
- Invalidated -> Empty/Buffered via subsequent fill_buf()/read()/peek() refilling
- Empty -> Bypassed via read/read_buf/read_vectored fast-path (discard_buffer then inner.read*)
- Buffered -> Invalidated via Seek::seek() (always discards after performing seek)

**Evidence:** struct fields: `buf: Buffer`, `inner: R` (two sources of truth that must remain coherent); peek(): `assert!(n <= self.capacity());` and loop that may `backshift()` then `read_more(&mut self.inner)`; returns slices of `self.buf.buffer()` (relies on buffer state being valid); Read::read(): fast-path `if self.buf.pos() == self.buf.filled() && buf.len() >= self.capacity() { self.discard_buffer(); return self.inner.read(buf); }` (ordering requirement: discard before delegating); Read::read_buf(): same bypass protocol `discard_buffer(); return self.inner.read_buf(cursor);`; Read::read_vectored(): same bypass protocol based on total_len and `discard_buffer()`; Read::read_to_end(): drains `let inner_buf = self.buffer(); ... self.discard_buffer(); Ok(nread + self.inner.read_to_end(buf)?)` (requires draining then discarding before delegating); Seek::seek() docs: 'Seeking always discards the internal buffer ... guarantees that calling BufReader::into_inner() immediately after a seek yields the underlying reader at the same position.' and code `self.discard_buffer();` on all paths; get_ref()/get_mut() docs: 'It is inadvisable to directly read from the underlying reader.' (implied protocol not enforced by types)

**Implementation:** Hide `get_mut()`-style unrestricted access behind a capability/token that can only be obtained when the buffer is provably empty/discarded. For example, provide `fn into_parts(self) -> (R, Buffer)` (already partially present via into_inner/with_buffer internally) or `fn with_inner_mut<F>(&mut self, f: F)` where F receives a wrapper that can only perform operations that preserve coherence (or automatically calls discard_buffer() first). A stronger typestate variant would be `BufReader<R, S>` where `S` encodes `Empty` vs `Buffered`, and only `InnerAccess` is available in `Empty` state; transitions occur via fill/consume/discard.

---

### 21. Repeat infinite-source protocol (bounded reads OK / unbounded reads unsupported)

**Location**: `/tmp/io_test_crate/src/io/util.rs:1-77`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Repeat behaves as an infinite byte source: bounded read operations (read, read_exact, read_buf, vectored reads) always succeed by filling the provided buffer with the repeated byte, and there is no terminal EOF state. Because there is no end-of-stream, unbounded aggregation helpers (read_to_end, read_to_string) are intentionally unsupported and return an OutOfMemory error. This is an implicit protocol: callers must choose bounded reads and must not call read_to_end/read_to_string if they expect progress toward completion; the type system does not distinguish 'finite' vs 'infinite' readers or prevent use of unbounded aggregation APIs on infinite sources.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Empty, 1 free function(s), impl Read for Empty (8 methods), impl BufRead for Empty (6 methods), impl Seek for Empty (3 methods), impl SizeHint for Empty (1 methods), impl Write for Empty (7 methods), impl Write for & Empty (7 methods); struct Sink, 1 free function(s), impl Write for Sink (7 methods), impl Write for & Sink (7 methods)

/// This struct is generally created by calling [`repeat()`]. Please
/// see the documentation of [`repeat()`] for more details.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Repeat {
    byte: u8,
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for Repeat {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        buf.fill(self.byte);
        Ok(buf.len())
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        buf.fill(self.byte);
        Ok(())
    }

    #[inline]
    fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> io::Result<()> {
        // SAFETY: No uninit bytes are being written.
        unsafe { buf.as_mut() }.write_filled(self.byte);
        // SAFETY: the entire unfilled portion of buf has been initialized.
        unsafe { buf.advance_unchecked(buf.capacity()) };
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.read_buf(buf)
    }

    /// This function is not supported by `io::Repeat`, because there's no end of its data
    fn read_to_end(&mut self, _: &mut Vec<u8>) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::OutOfMemory))
    }

    /// This function is not supported by `io::Repeat`, because there's no end of its data
    fn read_to_string(&mut self, _: &mut String) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::OutOfMemory))
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            nwritten += self.read(buf)?;
        }
        Ok(nwritten)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        true
    }
}

impl SizeHint for Repeat {
    #[inline]
    fn lower_bound(&self) -> usize {
        usize::MAX
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        None
    }
}

```

**Entity:** Repeat

**States:** InfiniteStream (no EOF), UnsupportedUnboundedAggregation (read_to_end/read_to_string)

**Transitions:**
- InfiniteStream -> UnsupportedUnboundedAggregation via Read::read_to_end()
- InfiniteStream -> UnsupportedUnboundedAggregation via Read::read_to_string()

**Evidence:** struct Repeat { byte: u8 } (no length/position/end state tracked); Read::read(): buf.fill(self.byte); Ok(buf.len()) (always fills entire provided buffer; no EOF behavior); Read::read_exact(): buf.fill(self.byte); Ok(()) (always succeeds for any buf length); comment on read_to_end(): "not supported ... because there's no end of its data"; read_to_end(): Err(io::Error::from(io::ErrorKind::OutOfMemory)); comment on read_to_string(): "not supported ... because there's no end of its data"; read_to_string(): Err(io::Error::from(io::ErrorKind::OutOfMemory)); impl SizeHint for Repeat: lower_bound() -> usize::MAX; upper_bound() -> None (signals 'unbounded/infinite')

**Implementation:** Introduce marker traits/types to distinguish infinite vs finite readers, e.g., `trait FiniteRead: Read {}` implemented by finite sources, and provide `read_to_end`/`read_to_string` only for `FiniteRead` (or provide separate free functions `read_to_end_finite<R: FiniteRead>(...)`). Alternatively wrap `Repeat` in `struct Infinite<R>(R)` and avoid exposing unbounded aggregation methods in that API surface.

---

### 6. append_to_string UTF-8 + append-only buffer protocol

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-434`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: `append_to_string` temporarily breaks the normal `String` invariant by exposing `buf.as_mut_vec()` to a caller-provided closure `f`. The safety contract requires that `f` only appends bytes (does not overwrite existing initialized bytes) and that on return the newly appended region must be valid UTF-8 to be committed. A `Guard` RAII drop restores the original length on unwind or early return so invalid UTF-8 is never left within the `String`'s length. None of the 'append-only' behavior or 'do not overwrite existing bytes' contract is enforced by the type system; it relies on `unsafe fn`, comments, and post-validation via `str::from_utf8`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard, impl Drop for Guard < '_ > (1 methods); struct IoSliceMut, 1 free function(s), impl Send for IoSliceMut < 'a > (0 methods), impl Sync for IoSliceMut < 'a > (0 methods), impl IoSliceMut < 'a > (4 methods), impl Deref for IoSliceMut < 'a > (1 methods), impl DerefMut for IoSliceMut < 'a > (1 methods); struct IoSlice, 1 free function(s), impl Send for IoSlice < 'a > (0 methods), impl Sync for IoSlice < 'a > (0 methods), impl IoSlice < 'a > (4 methods), impl Deref for IoSlice < 'a > (1 methods); struct Chain, impl Chain < T , U > (3 methods), impl Read for Chain < T , U > (5 methods), impl BufRead for Chain < T , U > (3 methods), impl SizeHint for Chain < T , U > (2 methods); struct Take, impl Take < T > (5 methods), impl Read for Take < T > (2 methods), impl BufRead for Take < T > (2 methods), impl SizeHint for Take < T > (2 methods); struct Bytes, impl Iterator for Bytes < R > (2 methods); struct Split, impl Iterator for Split < B > (1 methods); struct Lines, impl Iterator for Lines < B > (1 methods); enum SeekFrom

//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! The `std::io` module contains a number of common things you'll need
//! when doing input and output. The most core part of this module is
//! the [`Read`] and [`Write`] traits, which provide the
//! most general interface for reading and writing input and output.
//!
//! ## Read and Write
//!
//! Because they are traits, [`Read`] and [`Write`] are implemented by a number
//! of other types, and you can implement them for your types too. As such,
//! you'll see a few different types of I/O throughout the documentation in
//! this module: [`File`]s, [`TcpStream`]s, and sometimes even [`Vec<T>`]s. For
//! example, [`Read`] adds a [`read`][`Read::read`] method, which we can use on
//! [`File`]s:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`Read`] and [`Write`] are so important, implementors of the two traits have a
//! nickname: readers and writers. So you'll sometimes see 'a reader' instead
//! of 'a type that implements the [`Read`] trait'. Much easier!
//!
//! ## Seek and BufRead
//!
//! Beyond that, there are two important traits that are provided: [`Seek`]
//! and [`BufRead`]. Both of these build on top of a reader to control
//! how the reading happens. [`Seek`] lets you control where the next byte is
//! coming from:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::SeekFrom;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let mut f = File::open("foo.txt")?;
//!     let mut buffer = [0; 10];
//!
//!     // skip to the last 10 bytes of the file
//!     f.seek(SeekFrom::End(-10))?;
//!
//!     // read up to 10 bytes
//!     let n = f.read(&mut buffer)?;
//!
//!     println!("The bytes: {:?}", &buffer[..n]);
//!     Ok(())
//! }
//! ```
//!
//! [`BufRead`] uses an internal buffer to provide a number of other ways to read, but
//! to show it off, we'll need to talk about buffers in general. Keep reading!
//!
//! ## BufReader and BufWriter
//!
//! Byte-based interfaces are unwieldy and can be inefficient, as we'd need to be
//! making near-constant calls to the operating system. To help with this,
//! `std::io` comes with two structs, [`BufReader`] and [`BufWriter`], which wrap
//! readers and writers. The wrapper uses a buffer, reducing the number of
//! calls and providing nicer methods for accessing exactly what you want.
//!
//! For example, [`BufReader`] works with the [`BufRead`] trait to add extra
//! methods to any reader:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let mut reader = BufReader::new(f);
//!     let mut buffer = String::new();
//!
//!     // read a line into buffer
//!     reader.read_line(&mut buffer)?;
//!
//!     println!("{buffer}");
//!     Ok(())
//! }
//! ```
//!
//! [`BufWriter`] doesn't add any new ways of writing; it just buffers every call
//! to [`write`][`Write::write`]:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufWriter;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::create("foo.txt")?;
//!     {
//!         let mut writer = BufWriter::new(f);
//!
//!         // write a byte to the buffer
//!         writer.write(&[42])?;
//!
//!     } // the buffer is flushed once writer goes out of scope
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Standard input and output
//!
//! A very common source of input is standard input:
//!
//! ```no_run
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!     Ok(())
//! }
//! ```
//!
//! Note that you cannot use the [`?` operator] in functions that do not return
//! a [`Result<T, E>`][`Result`]. Instead, you can call [`.unwrap()`]
//! or `match` on the return value to catch any possible errors:
//!
//! ```no_run
//! use std::io;
//!
//! let mut input = String::new();
//!
//! io::stdin().read_line(&mut input).unwrap();
//! ```
//!
//! And a very common source of output is standard output:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//!
//! fn main() -> io::Result<()> {
//!     io::stdout().write(&[42])?;
//!     Ok(())
//! }
//! ```
//!
//! Of course, using [`io::stdout`] directly is less common than something like
//! [`println!`].
//!
//! ## Iterator types
//!
//! A large number of the structures provided by `std::io` are for various
//! ways of iterating over I/O. For example, [`Lines`] is used to split over
//! lines:
//!
//! ```no_run
//! use std::io;
//! use std::io::prelude::*;
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! fn main() -> io::Result<()> {
//!     let f = File::open("foo.txt")?;
//!     let reader = BufReader::new(f);
//!
//!     for line in reader.lines() {
//!         println!("{}", line?);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Functions
//!
//! There are a number of [functions][functions-list] that offer access to various
//! features. For example, we can use three of these functions to copy everything
//! from standard input to standard output:
//!
//! ```no_run
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     io::copy(&mut io::stdin(), &mut io::stdout())?;
//!     Ok(())
//! }
//! ```
//!
//! [functions-list]: #functions-1
//!
//! ## io::Result
//!
//! Last, but certainly not least, is [`io::Result`]. This type is used
//! as the return type of many `std::io` functions that can cause an error, and
//! can be returned from your own functions as well. Many of the examples in this
//! module use the [`?` operator]:
//!
//! ```
//! use std::io;
//!
//! fn read_input() -> io::Result<()> {
//!     let mut input = String::new();
//!
//!     io::stdin().read_line(&mut input)?;
//!
//!     println!("You typed: {}", input.trim());
//!
//!     Ok(())
//! }
//! ```
//!
//! The return type of `read_input()`, [`io::Result<()>`][`io::Result`], is a very
//! common type for functions which don't have a 'real' return value, but do want to
//! return errors if they happen. In this case, the only purpose of this function is
//! to read the line and print it, so we use `()`.
//!
//! ## Platform-specific behavior
//!
//! Many I/O functions throughout the standard library are documented to indicate
//! what various library or syscalls they are delegated to. This is done to help
//! applications both understand what's happening under the hood as well as investigate
//! any possibly unclear semantics. Note, however, that this is informative, not a binding
//! contract. The implementation of many of these functions are subject to change over
//! time and may call fewer or more syscalls/library functions.
//!
//! ## I/O Safety
//!
//! Rust follows an I/O safety discipline that is comparable to its memory safety discipline. This
//! means that file descriptors can be *exclusively owned*. (Here, "file descriptor" is meant to
//! subsume similar concepts that exist across a wide range of operating systems even if they might
//! use a different name, such as "handle".) An exclusively owned file descriptor is one that no
//! other code is allowed to access in any way, but the owner is allowed to access and even close
//! it any time. A type that owns its file descriptor should usually close it in its `drop`
//! function. Types like [`File`] own their file descriptor. Similarly, file descriptors
//! can be *borrowed*, granting the temporary right to perform operations on this file descriptor.
//! This indicates that the file descriptor will not be closed for the lifetime of the borrow, but
//! it does *not* imply any right to close this file descriptor, since it will likely be owned by
//! someone else.
//!
//! The platform-specific parts of the Rust standard library expose types that reflect these
//! concepts, see [`os::unix`] and [`os::windows`].
//!
//! To uphold I/O safety, it is crucial that no code acts on file descriptors it does not own or
//! borrow, and no code closes file descriptors it does not own. In other words, a safe function
//! that takes a regular integer, treats it as a file descriptor, and acts on it, is *unsound*.
//!
//! Not upholding I/O safety and acting on a file descriptor without proof of ownership can lead to
//! misbehavior and even Undefined Behavior in code that relies on ownership of its file
//! descriptors: a closed file descriptor could be re-allocated, so the original owner of that file
//! descriptor is now working on the wrong file. Some code might even rely on fully encapsulating
//! its file descriptors with no operations being performed by any other part of the program.
//!
//! Note that exclusive ownership of a file descriptor does *not* imply exclusive ownership of the
//! underlying kernel object that the file descriptor references (also called "open file description" on
//! some operating systems). File descriptors basically work like [`Arc`]: when you receive an owned
//! file descriptor, you cannot know whether there are any other file descriptors that reference the
//! same kernel object. However, when you create a new kernel object, you know that you are holding
//! the only reference to it. Just be careful not to lend it to anyone, since they can obtain a
//! clone and then you can no longer know what the reference count is! In that sense, [`OwnedFd`] is
//! like `Arc` and [`BorrowedFd<'a>`] is like `&'a Arc` (and similar for the Windows types). In
//! particular, given a `BorrowedFd<'a>`, you are not allowed to close the file descriptor -- just
//! like how, given a `&'a Arc`, you are not allowed to decrement the reference count and
//! potentially free the underlying object. There is no equivalent to `Box` for file descriptors in
//! the standard library (that would be a type that guarantees that the reference count is `1`),
//! however, it would be possible for a crate to define a type with those semantics.
//!
//! [`File`]: crate::fs::File
//! [`TcpStream`]: crate::net::TcpStream
//! [`io::stdout`]: stdout
//! [`io::Result`]: self::Result
//! [`?` operator]: ../../book/appendix-02-operators.html
//! [`Result`]: crate::result::Result
//! [`.unwrap()`]: crate::result::Result::unwrap
//! [`os::unix`]: ../os/unix/io/index.html
//! [`os::windows`]: ../os/windows/io/index.html
//! [`OwnedFd`]: ../os/fd/struct.OwnedFd.html
//! [`BorrowedFd<'a>`]: ../os/fd/struct.BorrowedFd.html
//! [`Arc`]: crate::sync::Arc

#![stable(feature = "rust1", since = "1.0.0")]

#[cfg(test)]
mod tests;

#[unstable(feature = "read_buf", issue = "78485")]
pub use core::io::{BorrowedBuf, BorrowedCursor};
use core::slice::memchr;

#[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
pub use self::buffered::WriterPanicked;
#[unstable(feature = "raw_os_error_ty", issue = "107792")]
pub use self::error::RawOsError;
#[doc(hidden)]
#[unstable(feature = "io_const_error_internals", issue = "none")]
pub use self::error::SimpleMessage;
#[unstable(feature = "io_const_error", issue = "133448")]
pub use self::error::const_error;
#[stable(feature = "anonymous_pipe", since = "1.87.0")]
pub use self::pipe::{PipeReader, PipeWriter, pipe};
#[stable(feature = "is_terminal", since = "1.70.0")]
pub use self::stdio::IsTerminal;
pub(crate) use self::stdio::attempt_print_to_stderr;
#[unstable(feature = "print_internals", issue = "none")]
#[doc(hidden)]
pub use self::stdio::{_eprint, _print};
#[unstable(feature = "internal_output_capture", issue = "none")]
#[doc(no_inline, hidden)]
pub use self::stdio::{set_output_capture, try_set_output_capture};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::{
    buffered::{BufReader, BufWriter, IntoInnerError, LineWriter},
    copy::copy,
    cursor::Cursor,
    error::{Error, ErrorKind, Result},
    stdio::{Stderr, StderrLock, Stdin, StdinLock, Stdout, StdoutLock, stderr, stdin, stdout},
    util::{Empty, Repeat, Sink, empty, repeat, sink},
};
use crate::mem::take;
use crate::ops::{Deref, DerefMut};
use crate::{cmp, fmt, slice, str, sys};

mod buffered;
pub(crate) mod copy;
mod cursor;
mod error;
mod impls;
mod pipe;
pub mod prelude;
mod stdio;
mod util;

const DEFAULT_BUF_SIZE: usize = crate::sys::io::DEFAULT_BUF_SIZE;

pub(crate) use stdio::cleanup;

struct Guard<'a> {
    buf: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buf.set_len(self.len);
        }
    }
}

// Several `read_to_string` and `read_line` methods in the standard library will
// append data into a `String` buffer, but we need to be pretty careful when
// doing this. The implementation will just call `.as_mut_vec()` and then
// delegate to a byte-oriented reading method, but we must ensure that when
// returning we never leave `buf` in a state such that it contains invalid UTF-8
// in its bounds.
//
// To this end, we use an RAII guard (to protect against panics) which updates
// the length of the string when it is dropped. This guard initially truncates
// the string to the prior length and only after we've validated that the
// new contents are valid UTF-8 do we allow it to set a longer length.
//
// The unsafety in this function is twofold:
//
// 1. We're looking at the raw bytes of `buf`, so we take on the burden of UTF-8
//    checks.
// 2. We're passing a raw buffer to the function `f`, and it is expected that
//    the function only *appends* bytes to the buffer. We'll get undefined
//    behavior if existing bytes are overwritten to have non-UTF-8 data.
pub(crate) unsafe fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize>
where
    F: FnOnce(&mut Vec<u8>) -> Result<usize>,
{
    let mut g = Guard { len: buf.len(), buf: unsafe { buf.as_mut_vec() } };
    let ret = f(g.buf);

    // SAFETY: the caller promises to only append data to `buf`
    let appended = unsafe { g.buf.get_unchecked(g.len..) };
    if str::from_utf8(appended).is_err() {
        ret.and_then(|_| Err(Error::INVALID_UTF8))
    } else {
        g.len = g.buf.len();
        ret
    }
}

// Here we must serve many masters with conflicting goals:
//
// - avoid allocating unless necessary
// - avoid overallocating if we know the exact size (#89165)
// - avoid passing large buffers to readers that always initialize the free capacity if they perform short reads (#23815, #23820)
// - pass large buffers to readers that do not initialize the spare capacity. this can amortize per-call overheads
// - and finally pass not-too-small and not-too-large buffers to Windows read APIs because they manage to suffer from both problems
//   at the same time, i.e. small reads suffer from syscall overhead, all reads incur costs proportional to buffer size (#110650)
//
pub(crate) fn default_read_to_end<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut Vec<u8>,
    size_hint: Option<usize>,
) -> Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();
    // Optionally limit the maximum bytes read on each iteration.
    // This adds an arbitrary fiddle factor to allow for more data than we expect.
    let mut max_read_size = size_hint
        .and_then(|s| s.checked_add(1024)?.checked_next_multiple_of(DEFAULT_BUF_SIZE))
        .unwrap_or(DEFAULT_BUF_SIZE);

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration

    const PROBE_SIZE: usize = 32;

    fn small_probe_read<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize> {
        let mut probe = [0u8; PROBE_SIZE];

        loop {
            match r.read(&mut probe) {
                Ok
// ... (truncated) ...
```

**Entity:** append_to_string (unsafe fn)

**States:** Pre-call: buf is valid UTF-8 (String invariant holds), In-call: raw Vec<u8> view exposed; temporary invalid UTF-8 permitted beyond original len, Post-call success: appended bytes validated UTF-8 and length committed, Post-call failure/panic: length restored to original; String invariant preserved

**Transitions:**
- Pre-call -> In-call via `buf.as_mut_vec()` and calling `f(g.buf)`
- In-call -> Post-call success via `str::from_utf8(appended)` OK then `g.len = g.buf.len()`
- In-call -> Post-call failure via `str::from_utf8(appended)` Err then return `Error::INVALID_UTF8` (Guard drops, restoring old len)
- In-call -> Post-call panic via unwind (Guard drops, restoring old len)

**Evidence:** struct Guard<'a> { buf: &'a mut Vec<u8>, len: usize } tracks prior length; impl Drop for Guard: `self.buf.set_len(self.len);` (restores length on drop); comment above append_to_string: "must ensure ... never leave buf ... invalid UTF-8" and "function only *appends* bytes" and "UB if existing bytes are overwritten"; append_to_string: `let mut g = Guard { len: buf.len(), buf: unsafe { buf.as_mut_vec() } };` exposes raw bytes; append_to_string: `let appended = unsafe { g.buf.get_unchecked(g.len..) };` relies on the 'append-only' promise; append_to_string: `if str::from_utf8(appended).is_err() { ... Err(Error::INVALID_UTF8) } else { g.len = g.buf.len(); }` is the runtime UTF-8 gate

**Implementation:** Replace `F: FnOnce(&mut Vec<u8>)` with a restricted capability type like `struct AppendOnly<'a>(&'a mut Vec<u8>, start_len: usize);` that only exposes safe `extend_from_slice`/`push`/`reserve`-style APIs and does not allow indexing/writing into the prefix `[0..start_len)`. Then `append_to_string` can pass `AppendOnly` to `f` (safe fn), validate UTF-8 for the appended region, and commit length; the type prevents overwriting existing bytes at compile time.

---

