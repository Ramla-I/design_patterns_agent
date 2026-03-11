# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 63
- **Temporal ordering**: 4
- **Resource lifecycle**: 1
- **State machine**: 29
- **Precondition**: 10
- **Protocol**: 19
- **Modules analyzed**: 21

## Temporal Ordering Invariants

### 37. Take limit protocol (set_limit before copy/reads)

**Location**: `/tmp/io_test_crate/src/io/copy/tests.rs:1-57`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The benchmark constructs a reader as `BufReader::with_capacity(..., dyn_in.take(0))`, which creates a `Take` adaptor with a 0-byte limit (effectively EOF). Before each iteration it must reset the limit to a nonzero value via `set_limit(BYTES)`; otherwise `io::copy` would copy 0 bytes. This is a temporal ordering requirement (configure limit, then perform I/O) encoded only by runtime state in the `Take` adaptor, not the type system.

**Evidence**:

```rust
    assert!(
        source.observed_buffer > DEFAULT_BUF_SIZE,
        "expected a large buffer to be provided to the reader, got {}",
        source.observed_buffer
    );
}

#[test]
fn copy_specializes_from_vecdeque() {
    let mut source = VecDeque::with_capacity(100 * 1024);
    for _ in 0..20 * 1024 {
        source.push_front(0);
    }
    for _ in 0..20 * 1024 {
        source.push_back(0);
    }
    let mut sink = WriteObserver { observed_buffer: 0 };
    assert_eq!(40 * 1024u64, io::copy(&mut source, &mut sink).unwrap());
    assert_eq!(20 * 1024, sink.observed_buffer);
}

#[test]
fn copy_specializes_from_slice() {
    let mut source = [1; 60 * 1024].as_slice();
    let mut sink = WriteObserver { observed_buffer: 0 };
    assert_eq!(60 * 1024u64, io::copy(&mut source, &mut sink).unwrap());
    assert_eq!(60 * 1024, sink.observed_buffer);
}

#[cfg(unix)]
mod io_benches {
    use test::Bencher;

    use crate::fs::{File, OpenOptions};
    use crate::io::BufReader;
    use crate::io::prelude::*;

    #[bench]
    #[cfg_attr(target_os = "emscripten", ignore)] // no /dev
    fn bench_copy_buf_reader(b: &mut Bencher) {
        let mut file_in = File::open("/dev/zero").expect("opening /dev/zero failed");
        // use dyn to avoid specializations unrelated to readbuf
        let dyn_in = &mut file_in as &mut dyn Read;
        let mut reader = BufReader::with_capacity(256 * 1024, dyn_in.take(0));
        let mut writer =
            OpenOptions::new().write(true).open("/dev/null").expect("opening /dev/null failed");

        const BYTES: u64 = 1024 * 1024;

        b.bytes = BYTES;

        b.iter(|| {
            reader.get_mut().set_limit(BYTES);
            crate::io::copy(&mut reader, &mut writer).unwrap()
        });
    }
}
```

**Entity:** Take<dyn Read> (created via dyn_in.take(0))

**States:** Limit=0 (EOF immediately), Limit>0 (Readable up to limit bytes)

**Transitions:**
- Limit=0 -> Limit>0 via set_limit(BYTES)
- Limit>0 -> Limit>0 via set_limit(...) (reconfiguration per iteration)

**Evidence:** bench_copy_buf_reader: `let mut reader = BufReader::with_capacity(256 * 1024, dyn_in.take(0));` (initial limit is 0); bench_copy_buf_reader: inside b.iter: `reader.get_mut().set_limit(BYTES);` (required reconfiguration step); bench_copy_buf_reader: `crate::io::copy(&mut reader, &mut writer).unwrap()` depends on the limit having been set

**Implementation:** Introduce a wrapper around `Take<R>` with state parameters, e.g. `Limited<R, Unset>` created by `take(0)`, and a method `set_limit(self, n) -> Limited<R, Set>`; only `Limited<R, Set>` implements/forwards `Read` (or is accepted by the benchmark/copy helper). Alternatively use a builder-like API that requires specifying the initial nonzero limit before yielding a readable handle.

---

### 15. Last-OS-error temporal ordering (platform error slot must be read immediately)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-165`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: `last_os_error()` relies on an external, thread-local/platform 'last error' slot (errno/GetLastError) being valid only immediately after a failing platform call. The code cannot enforce that no other OS/stdlib calls occur between the failure and `last_os_error()`, so callers must follow a temporal protocol described in docs. This invariant is entirely outside the Rust type system; misuse yields an `Error` that may represent an unrelated/overwritten OS error.

**Evidence**:

```rust
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
```

**Entity:** std::io::Error (last_os_error())

**States:** FreshErrno, IndeterminateErrno

**Transitions:**
- FreshErrno -> IndeterminateErrno via any intervening platform/stdlib call before last_os_error()

**Evidence:** doc on last_os_error(): 'This should be called immediately after a call to a platform function, otherwise the state of the error value is indeterminate... other standard library functions may call platform functions that may (or may not) reset the error value'; implementation last_os_error(): `Error::from_raw_os_error(sys::os::errno())` (reads ambient OS error slot)

**Implementation:** Have low-level syscalls return an error token/capability capturing the OS error code at the point of failure (e.g., `Result<T, OsErrorCode>` or `LastErrorToken(RawOsError)`), so conversion to `io::Error` is deterministic and cannot be delayed. Internally, `sys` functions would return `RawOsError` directly instead of requiring a later `errno()` read.

---

### 30. BufReader initialization protocol (Uninitialized -> Fully-initialized buffer after fill_buf)

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-86`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The test assumes an implicit invariant that calling fill_buf() fully initializes the BufReader's internal buffer memory, even if the underlying read returns only a single byte. This is a temporal ordering/initialization contract: before fill_buf(), initialized()==0; after fill_buf(), initialized()==capacity(). The state (how much of the internal buffer is initialized) is tracked at runtime, and correctness relies on the implementation ensuring initialization before exposing slices; the type system does not encode or enforce 'buffer fully initialized' as a distinct type/state.

**Evidence**:

```rust
    // would then try to buffer B and C, but because its capacity is 5,
    // it will only be able to buffer part of B. Because it's not possible
    // for it to buffer any complete lines, it should buffer as much of B as
    // possible
    assert_eq!(writer.write(content).unwrap(), 10);
    assert_eq!(writer.get_ref().buffer, *b"AAAAA");

    writer.flush().unwrap();
    assert_eq!(writer.get_ref().buffer, *b"AAAAABBBBB");
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordedEvent {
    Write(String),
    Flush,
}

#[derive(Debug, Clone, Default)]
struct WriteRecorder {
    pub events: Vec<RecordedEvent>,
}

impl Write for WriteRecorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use crate::str::from_utf8;

        self.events.push(RecordedEvent::Write(from_utf8(buf).unwrap().to_string()));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.push(RecordedEvent::Flush);
        Ok(())
    }
}

/// Test that a normal, formatted writeln only results in a single write
/// call to the underlying writer. A naive implementation of
/// LineWriter::write_all results in two writes: one of the buffered data,
/// and another of the final substring in the formatted set
#[test]
fn single_formatted_write() {
    let writer = WriteRecorder::default();
    let mut writer = LineWriter::new(writer);

    // Under a naive implementation of LineWriter, this will result in two
    // writes: "hello, world" and "!\n", because write() has to flush the
    // buffer before attempting to write the last "!\n". write_all shouldn't
    // have this limitation.
    writeln!(&mut writer, "{}, {}!", "hello", "world").unwrap();
    assert_eq!(writer.get_ref().events, [RecordedEvent::Write("hello, world!\n".to_string())]);
}

#[test]
fn bufreader_full_initialize() {
    struct OneByteReader;
    impl Read for OneByteReader {
        fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
            if buf.len() > 0 {
                buf[0] = 0;
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }
    let mut reader = BufReader::new(OneByteReader);
    // Nothing is initialized yet.
    assert_eq!(reader.initialized(), 0);

    let buf = reader.fill_buf().unwrap();
    // We read one byte...
    assert_eq!(buf.len(), 1);
    // But we initialized the whole buffer!
    assert_eq!(reader.initialized(), reader.capacity());
}

/// This is a regression test for https://github.com/rust-lang/rust/issues/127584.
#[test]
fn bufwriter_aliasing() {
    use crate::io::{BufWriter, Cursor};
    let mut v = vec![0; 1024];
    let c = Cursor::new(&mut v);
    let w = BufWriter::new(Box::new(c));
    let _ = w.into_parts();
}
```

**Entity:** BufReader<OneByteReader> (and BufReader<R> generally)

**States:** Uninitialized (internal buffer contents not initialized), Initialized (buffer memory initialized up to capacity)

**Transitions:**
- Uninitialized -> Initialized via fill_buf()

**Evidence:** bufreader_full_initialize(): "Nothing is initialized yet." followed by assert_eq!(reader.initialized(), 0); bufreader_full_initialize(): let buf = reader.fill_buf().unwrap(); then assert_eq!(reader.initialized(), reader.capacity()); OneByteReader::read(): returns Ok(1) after writing only buf[0] = 0, yet the test requires BufReader to report full initialization regardless

**Implementation:** Model initialization as a distinct type parameter/state, e.g. BufReader<R, S> where S is Uninit/Init. fill_buf(self: &mut BufReader<R, Uninit>) -> io::Result<&[u8]> could transition to Init internally (or return a BufReader<R, Init> in a consuming API). This would allow APIs that require initialized memory to only exist on the Init state, while keeping unsafe/initialization details encapsulated.

---

### 35. BufWriter buffered/dirty protocol (Buffered -> Flushed before Seek/Drop)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-72`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: BufWriter maintains an internal buffer that may contain unwritten data. Certain operations require the buffer to be flushed first: seek() explicitly flushes before delegating to the underlying writer, and drop() attempts to flush unless the thread is already panicking (tracked by panicked). These requirements are enforced by runtime calls (flush_buf() and the panicked flag) rather than the type system; callers can conceptually be in a 'dirty' state where direct operations on the underlying writer (e.g., via get_mut() elsewhere in the type) would be invalid unless preceded by a flush.

**Evidence**:

```rust
                    // sufficient room for any input <= the buffer size, which includes this input.
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    buf.len()
                }
            } else {
                return Ok(0);
            };
            debug_assert!(total_written != 0);
            for buf in iter {
                if buf.len() <= self.spare_capacity() {
                    // SAFETY: safe by above conditional.
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    // This cannot overflow `usize`. If we are here, we've written all of the bytes
                    // so far to our buffer, and we've ensured that we never exceed the buffer's
                    // capacity. Therefore, `total_written` <= `self.buf.capacity()` <= `usize::MAX`.
                    total_written += buf.len();
                } else {
                    break;
                }
            }
            Ok(total_written)
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.get_mut().flush())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> fmt::Debug for BufWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufWriter")
            .field("writer", &&self.inner)
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
            .finish()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write + Seek> Seek for BufWriter<W> {
    /// Seek to the offset, in bytes, in the underlying writer.
    ///
    /// Seeking always writes out the internal buffer before seeking.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.flush_buf()?;
        self.get_mut().seek(pos)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            // dtors should not panic, so we ignore a failed flush
            let _r = self.flush_buf();
        }
    }
}
```

**Entity:** BufWriter<W>

**States:** Buffered (dirty), Flushed (clean), Panicking (no flush on drop)

**Transitions:**
- Buffered (dirty) -> Flushed (clean) via flush() / flush_buf()
- Any -> Panicking (no flush on drop) via panicked flag being set (checked in Drop)

**Evidence:** seek(): comment "Seeking always writes out the internal buffer before seeking." and code `self.flush_buf()?; self.get_mut().seek(pos)`; flush(): `self.flush_buf().and_then(|()| self.get_mut().flush())` shows flush is a two-step protocol (buffer then inner); Drop::drop(): `if !self.panicked { let _r = self.flush_buf(); }` indicates an implicit 'panicking' state where flushing is skipped; Drop::drop(): comment "dtors should not panic, so we ignore a failed flush" indicates special handling/protocol during destruction

**Implementation:** Encode a dirty/clean typestate: `BufWriter<W, S>` with `S = Dirty | Clean`. Writing transitions to Dirty, `flush(self) -> BufWriter<W, Clean>`, and `seek(&mut self, ...)` only available for `BufWriter<W, Clean>` (or `seek(self, ...) -> (BufWriter<W, Clean>, u64)`), forcing callers to flush before seeking. A separate guard/capability could represent 'may flush on drop' vs 'in panic' if needed, though panicking is hard to fully encode statically.

---

## Resource Lifecycle Invariants

### 55. Stdio lock non-poisoning invariant across panics

**Location**: `/tmp/io_test_crate/src/io/stdio/tests.rs:1-166`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: The test relies on an implicit invariant about the stdio global locks: acquiring stdin/stdout/stderr locks and then panicking in the holding thread must not permanently poison or render those locks unusable for later acquisitions. This is verified by first panicking after taking locks in a spawned thread, then re-locking stdin/stdout/stderr successfully on the main thread. The type system does not express (or enforce) the guarantee that these locks are non-poisoning or recoverable after unwind; it's a behavioral contract validated at runtime.

**Evidence**:

```rust
use super::*;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::mpsc::sync_channel;
use crate::thread;

#[test]
fn stdout_unwind_safe() {
    assert_unwind_safe::<Stdout>();
}
#[test]
fn stdoutlock_unwind_safe() {
    assert_unwind_safe::<StdoutLock<'_>>();
    assert_unwind_safe::<StdoutLock<'static>>();
}
#[test]
fn stderr_unwind_safe() {
    assert_unwind_safe::<Stderr>();
}
#[test]
fn stderrlock_unwind_safe() {
    assert_unwind_safe::<StderrLock<'_>>();
    assert_unwind_safe::<StderrLock<'static>>();
}

fn assert_unwind_safe<T: UnwindSafe + RefUnwindSafe>() {}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn panic_doesnt_poison() {
    thread::spawn(|| {
        let _a = stdin();
        let _a = _a.lock();
        let _a = stdout();
        let _a = _a.lock();
        let _a = stderr();
        let _a = _a.lock();
        panic!();
    })
    .join()
    .unwrap_err();

    let _a = stdin();
    let _a = _a.lock();
    let _a = stdout();
    let _a = _a.lock();
    let _a = stderr();
    let _a = _a.lock();
}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stderr() {
    test_lock(stderr, || stderr().lock());
}
#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stdin() {
    test_lock(stdin, || stdin().lock());
}
#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stdout() {
    test_lock(stdout, || stdout().lock());
}

// Helper trait to make lock testing function generic.
trait Stdio<'a>: 'static
where
    Self::Lock: 'a,
{
    type Lock;
    fn lock(&'a self) -> Self::Lock;
}
impl<'a> Stdio<'a> for Stderr {
    type Lock = StderrLock<'a>;
    fn lock(&'a self) -> StderrLock<'a> {
        self.lock()
    }
}
impl<'a> Stdio<'a> for Stdin {
    type Lock = StdinLock<'a>;
    fn lock(&'a self) -> StdinLock<'a> {
        self.lock()
    }
}
impl<'a> Stdio<'a> for Stdout {
    type Lock = StdoutLock<'a>;
    fn lock(&'a self) -> StdoutLock<'a> {
        self.lock()
    }
}

// Helper trait to make lock testing function generic.
trait StdioOwnedLock: 'static {}
impl StdioOwnedLock for StderrLock<'static> {}
impl StdioOwnedLock for StdinLock<'static> {}
impl StdioOwnedLock for StdoutLock<'static> {}

// Tests locking on stdio handles by starting two threads and checking that
// they block each other appropriately.
fn test_lock<T, U>(get_handle: fn() -> T, get_locked: fn() -> U)
where
    T: for<'a> Stdio<'a>,
    U: StdioOwnedLock,
{
    // State enum to track different phases of the test, primarily when
    // each lock is acquired and released.
    #[derive(Debug, PartialEq)]
    enum State {
        Start1,
        Acquire1,
        Start2,
        Release1,
        Acquire2,
        Release2,
    }
    use State::*;
    // Logging vector to be checked to make sure lock acquisitions and
    // releases happened in the correct order.
    let log = Arc::new(Mutex::new(Vec::new()));
    let ((tx1, rx1), (tx2, rx2)) = (sync_channel(0), sync_channel(0));
    let th1 = {
        let (log, tx) = (Arc::clone(&log), tx1);
        thread::spawn(move || {
            log.lock().unwrap().push(Start1);
            let handle = get_handle();
            {
                let locked = handle.lock();
                log.lock().unwrap().push(Acquire1);
                tx.send(Acquire1).unwrap(); // notify of acquisition
                tx.send(Release1).unwrap(); // wait for release command
                log.lock().unwrap().push(Release1);
            }
            tx.send(Acquire1).unwrap(); // wait for th2 acquire
            {
                let locked = handle.lock();
                log.lock().unwrap().push(Acquire1);
            }
            log.lock().unwrap().push(Release1);
        })
    };
    let th2 = {
        let (log, tx) = (Arc::clone(&log), tx2);
        thread::spawn(move || {
            tx.send(Start2).unwrap(); // wait for start command
            let locked = get_locked();
            log.lock().unwrap().push(Acquire2);
            tx.send(Acquire2).unwrap(); // notify of acquisition
            tx.send(Release2).unwrap(); // wait for release command
            log.lock().unwrap().push(Release2);
        })
    };
    assert_eq!(rx1.recv().unwrap(), Acquire1); // wait for th1 acquire
    log.lock().unwrap().push(Start2);
    assert_eq!(rx2.recv().unwrap(), Start2); // block th2
    assert_eq!(rx1.recv().unwrap(), Release1); // release th1
    assert_eq!(rx2.recv().unwrap(), Acquire2); // wait for th2 acquire
    assert_eq!(rx1.recv().unwrap(), Acquire1); // block th1
    assert_eq!(rx2.recv().unwrap(), Release2); // release th2
    th2.join().unwrap();
    th1.join().unwrap();
    assert_eq!(
        *log.lock().unwrap(),
        [Start1, Acquire1, Start2, Release1, Acquire2, Release2, Acquire1, Release1]
    );
}
```

**Entity:** panic_doesnt_poison (stdio global-lock behavior)

**States:** Unlocked, LockedInThread, ThreadPanickedWhileLockHeld, UnlockedAfterPanic

**Transitions:**
- Unlocked -> LockedInThread via stdin().lock()/stdout().lock()/stderr().lock()
- LockedInThread -> ThreadPanickedWhileLockHeld via panic!()
- ThreadPanickedWhileLockHeld -> UnlockedAfterPanic via thread unwind + drop of lock guards
- UnlockedAfterPanic -> LockedInThread via subsequent stdin().lock()/stdout().lock()/stderr().lock() in main thread

**Evidence:** test name `panic_doesnt_poison` asserts the invariant in its intent; spawned thread acquires locks then panics: `let _a = stdin(); let _a = _a.lock(); ... panic!();`; join observes panic: `.join().unwrap_err();` (panic happened while locks had been taken); main thread then re-acquires all locks without handling poisoning errors: `let _a = stdin(); let _a = _a.lock(); ...`

**Implementation:** Expose a dedicated non-poisoning lock capability type for stdio (e.g., `NonPoisoningLock<'a, T>`) whose construction is only possible from stdio handles, and whose API does not surface poison-related failure modes. This makes the contract explicit in types and prevents accidentally swapping in a poisoning mutex/lock implementation behind the same API.

---

## State Machine Invariants

### 23. Line-buffered write protocol (Buffered tail / Completed-line pending / Flush-to-inner)

**Location**: `/tmp/io_test_crate/src/io/buffered/linewritershim.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The writer maintains an implicit line-buffering state machine over its internal `buffer`: when data contains newlines, all bytes up to the last newline should be written directly to the underlying writer (possibly after merging with buffered bytes), and only the trailing partial line (after the last newline) should remain buffered. Additionally, even when the incoming write contains no newline, the implementation must flush the existing buffer first *if the buffer already contains a completed line* (i.e., contains a newline), so that completed lines do not stay buffered. These invariants are enforced by runtime branching (`memrchr` on input, `buffered().is_empty()` checks, and `flush_if_completed_line()?`) rather than by the type system; callers cannot tell at compile time whether the object is in a state where a flush is required before buffering, nor can the compiler prevent forgetting to perform the flush/merge logic when adding new write entrypoints.

**Evidence**:

```rust
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
```

**Entity:** LineWriterShim (type owning `buffer` and delegating to `inner()`/`inner_mut()`)

**States:** BufferEmpty, BufferHasNoNewlineTail, BufferHasCompletedLine

**Transitions:**
- BufferEmpty -> BufferHasNoNewlineTail via write_all() when memrchr('\n', buf) == None and buffer.write_all(buf)
- BufferHasNoNewlineTail -> BufferHasNoNewlineTail via write_all() when memrchr('\n', buf) == None (may still flush if buffer overflows, per comment)
- BufferHasCompletedLine -> (flush) -> BufferEmpty via flush_if_completed_line()
- BufferHasNoNewlineTail -> (merge+flush lines) -> BufferHasNoNewlineTail via write_all() when memrchr('\n', buf) == Some(..) and !buffered().is_empty() (comment indicates merging buffered data with incoming `lines` before flushing)

**Evidence:** fn write_all(&mut self, buf: &[u8]) -> io::Result<()>: comment describes protocol: "data up to the last newline is sent directly ... and data after it is buffered"; write_all(): `match memchr::memrchr(b'\n', buf)` splits behavior into 'no newline' vs 'has newline' states; write_all(): in None arm: `self.flush_if_completed_line()?;` — explicit temporal requirement to flush prior completed lines before buffering more data; write_all(): `if self.buffered().is_empty() { self.inner_mut().write_all(lines)?; } else { ... add the incoming lines to that buffer before flushing ... }` — runtime check on buffer emptiness selecting different write/flush strategy; comment in else branch: "We can't really do this with `write`, since ... report a consistent state to the caller" — indicates state consistency concerns not represented in types

**Implementation:** Model the internal buffer condition as typestates, e.g. `LineWriterShim<S>` with `S = Empty | TailOnly | HasCompletedLine`. Provide internal transitions like `push_bytes(self, ..) -> LineWriterShim<...>` and `flush_completed(self) -> LineWriterShim<...>`. Expose `write_all` as a thin wrapper that drives these transitions, making it harder to introduce new methods that mutate `buffer` without first flushing `HasCompletedLine`.

---

### 39. Cursor position/seek protocol (implicit offset controls write location)

**Location**: `/tmp/io_test_crate/src/io/cursor/tests.rs:1-69`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Cursor maintains an implicit internal position that determines where subsequent write()/write_vectored() calls place bytes. Tests rely on the invariant that writes advance the position by the number of bytes written and that set_position() moves the position before any write occurs. This protocol is enforced only by runtime behavior and test assertions; the type system does not distinguish a 'positioned' cursor from a fresh cursor, nor does it statically prevent writing without first establishing the intended position.

**Evidence**:

```rust
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(writer, b);
}

#[test]
fn test_mem_writer() {
    let mut writer = Cursor::new(Vec::new());
    writer.set_position(10);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..10], &[0; 10]);
    assert_eq!(&writer.get_ref()[10..], b);
}

#[test]
fn test_mem_writer_preallocated() {
    let mut writer = Cursor::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 8, 9, 10]);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..], b);
}

#[test]
fn test_mem_mut_writer() {
    let mut vec = Vec::new();
    let mut writer = Cursor::new(&mut vec);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..], b);
}

fn test_slice_writer<T>(writer: &mut Cursor<T>)
where
    T: AsRef<[u8]>,
    Cursor<T>: Write,
{
    assert_eq!(writer.position(), 0);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.position(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(writer.position(), 8);
    assert_eq!(writer.write(&[]).unwrap(), 0);
```

**Entity:** Cursor<T>

**States:** Position = 0 (start), Position = N (after set_position and/or writes)

**Transitions:**
- Position = 0 -> Position = 10 via set_position(10)
- Position = N -> Position = N + k via write(&[...]) returning k
- Position = N -> Position = N + k via write_vectored(&[IoSlice...]) returning k

**Evidence:** test_mem_writer: writer.set_position(10) before any write(); test_slice_writer: assert_eq!(writer.position(), 0) before writes; test_slice_writer: assert_eq!(writer.position(), 1) after write(&[0]) returns 1; test_slice_writer: assert_eq!(writer.position(), 8) after subsequent writes totaling 8 bytes; multiple tests: write_vectored(...).unwrap() return value used to imply advancement (e.g., returns 3 for [8,9] + [10])

**Implementation:** Encode the 'position established' concept in the type: Cursor<T, Pos> where Pos is a ZST representing whether an explicit position has been set (e.g., Unpositioned/Positioned). Provide set_position(self, u64) -> Cursor<T, Positioned>. Optionally wrap the numeric position in a newtype/capability token returned from set_position and required for certain write-at-offset operations (if the API supports them).

---

### 41. Cursor position validity protocol (In-bounds vs Out-of-bounds/clamped-for-read)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-64`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Cursor stores its current position as a raw u64 (`self.pos`) that can be set to any value via `set_position` with no validation against the underlying buffer length. Some operations (e.g., `split`) must treat out-of-bounds positions specially by clamping `pos` to `len` to avoid panics. This implies an implicit state distinction: when `pos` is within bounds, operations reflect the true position; when `pos` is out of bounds, operations must either clamp or handle the mismatch. The type system does not encode whether a Cursor's position is currently known to be in-bounds with respect to `inner`.

**Evidence**:

```rust
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
```

**Entity:** Cursor<T>

**States:** InBounds(pos <= inner.len()), OutOfBounds(pos > inner.len())

**Transitions:**
- InBounds -> OutOfBounds via set_position(pos) with pos > inner.as_ref().len()
- OutOfBounds -> InBounds via set_position(pos) with pos <= inner.as_ref().len()

**Evidence:** method `pub const fn position(&self) -> u64 { self.pos }` exposes raw `pos` with no bounds guarantee; method `pub const fn set_position(&mut self, pos: u64) { self.pos = pos; }` stores any u64 without validation; method `split(&self)`: `let pos = self.pos.min(slice.len() as u64);` clamps position to slice length before `split_at`; doc example for `split`: `buff.set_position(6); ...` on a 5-byte buffer yields full/empty slices, demonstrating out-of-bounds positions are allowed and handled by clamping

**Implementation:** Introduce a `CursorPos` newtype that is constructed/updated only with knowledge of the underlying slice length (e.g., `CursorPos::new(pos, len) -> CursorPos` clamps or validates). Store `pos: CursorPos` instead of `u64`, or provide an alternate API `set_position_checked(len, pos)`/`set_position_clamped(len, pos)` that returns a Cursor with an in-bounds position type parameter (typestate could be used if APIs want to distinguish `Cursor<InBounds>` from `Cursor<Unchecked>`).

---

### 48. Cursor-like consumption protocol for slice reads (Remaining -> EOF)

**Location**: `/tmp/io_test_crate/src/io/impls.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The `Read` impl for `&[u8]` uses the slice value itself as mutable cursor state: each `read()` advances the slice to the yet-unread tail. When all bytes are consumed, the slice becomes empty and subsequent reads indicate EOF. This protocol (that `&mut &[u8]` is a stateful cursor, not an immutable view) is only documented and relies on mutation of the reference; the type system does not distinguish a slice-with-remaining-data from an EOF/empty slice, nor does it make the 'cursor' nature explicit.

**Evidence**:

```rust
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
```

**Entity:** &[u8] (as Read)

**States:** HasRemaining, EOF (empty slice)

**Transitions:**
- HasRemaining -> HasRemaining via Read::read() (advances internal slice head)
- HasRemaining -> EOF (empty slice) via Read::read() when all bytes consumed

**Evidence:** comment: "Note that reading updates the slice to point to the yet unread part. The slice will be empty when EOF is reached."; method: `impl Read for &[u8] { fn read(&mut self, ...) ... }` uses `&mut self` (mutating the slice reference to track position); code: `let amt = cmp::min(buf.len(), self.len()); let (a, b) = self.split_at(amt);` indicates taking a prefix and a remaining tail (`b`) to advance the cursor

**Implementation:** Introduce an explicit cursor type (newtype) like `struct SliceCursor<'a> { rem: &'a [u8] }` implementing `Read`, so callers opt into stateful consumption explicitly. If desired, expose a method returning `Option<NonEmptySliceCursor>` to represent the HasRemaining vs EOF distinction at the type level for APIs that want to statically prevent reads on empty input.

---

### 51. Chain read progression state machine (ReadingFirst / ReadingSecond)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-68`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Chain has an implicit two-phase protocol: it must read from `first` until it is exhausted (or until a read makes no progress / delimiter rules say to switch), and only then read from `second`. This phase is tracked at runtime via `done_first: bool` and repeatedly checked to decide which inner reader to delegate to. The type system does not prevent calling methods "in the wrong phase" (it’s always allowed), so correctness relies on updating `done_first` consistently across all relevant I/O methods.

**Evidence**:

```rust
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

```

**Entity:** Chain<T, U>

**States:** ReadingFirst, ReadingSecond

**Transitions:**
- ReadingFirst -> ReadingSecond when `fill_buf()` sees `first.fill_buf()?` return an empty buffer
- ReadingFirst -> ReadingSecond when `read_buf()` observes no progress (`buf.written()` unchanged after `first.read_buf(...)`)
- ReadingFirst -> ReadingSecond when `read_until()` does not end with the delimiter (or reads 0), causing `self.done_first = true`

**Evidence:** `done_first` runtime flag: `if !self.done_first { ... } else { ... }` appears in `read_buf`, `fill_buf`, `consume`, and `read_until`; `read_buf`: `if !self.done_first { ... self.first.read_buf(...) ... else { self.done_first = true; } }` then delegates to `self.second.read_buf(buf)`; `fill_buf`: `if !self.done_first { match self.first.fill_buf()? { buf if buf.is_empty() => self.done_first = true, buf => return Ok(buf) } }` then `self.second.fill_buf()`; `consume`: `if !self.done_first { self.first.consume(amt) } else { self.second.consume(amt) }`; `read_until`: sets `self.done_first = true` in the `_ => self.done_first = true` arm, and returns early if the delimiter was found in `first` (`Some(b) if *b == byte && n != 0 => return Ok(read)`)

**Implementation:** Encode the phase in the type: `Chain<T, U, S>` with `S = First | Second` as ZST markers. Provide transition methods like `fn into_second(self) -> Chain<T, U, Second>` when `first` is known exhausted. Expose BufRead/Read impls only for a unified wrapper that internally holds an enum `Either<Chain<First>, Chain<Second>>` or provide a safe API that returns a `Chain<Second>` once the first part is done, so phase-dependent delegation is no longer controlled by a bare boolean.

---

### 63. State machine: partial success with error (data consumed even when returning Err)

**Location**: `/tmp/io_test_crate/src/io/tests.rs:1-60`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The `read_buf` implementation writes data into the provided cursor (`self.0.read_buf(buf).unwrap();`) and then returns `Err(...)`. The tests rely on the nuanced invariant that an error return does not imply 'no progress': bytes may have been appended to the buffer and the underlying slice advanced. When wrapped in `take(1)`, the first call both fills 1 byte and returns an error; the second call returns Ok and no additional bytes (take-limit reached), and the underlying reader has advanced (remaining `[5,6]`). This protocol (error-with-progress + take-limit reached behavior) is not representable in the type system; callers must remember to check how much was filled even on `Err` and that subsequent calls may transition to Ok/EOF-like behavior.

**Evidence**:

```rust

// Issue #120603
#[test]
#[should_panic]
fn read_buf_broken_read() {
    struct MalformedRead;

    impl Read for MalformedRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            // broken length calculation
            Ok(buf.len() + 1)
        }
    }

    let _ = BufReader::new(MalformedRead).fill_buf();
}

#[test]
fn read_buf_full_read() {
    struct FullRead;

    impl Read for FullRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
    }

    assert_eq!(BufReader::new(FullRead).fill_buf().unwrap().len(), DEFAULT_BUF_SIZE);
}

struct DataAndErrorReader(&'static [u8]);

impl Read for DataAndErrorReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        panic!("We want tests to use `read_buf`")
    }

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf).unwrap();
        Err(io::Error::other("error"))
    }
}

#[test]
fn read_buf_data_and_error_take() {
    let mut buf = [0; 64];
    let mut buf = io::BorrowedBuf::from(buf.as_mut_slice());

    let mut r = DataAndErrorReader(&[4, 5, 6]).take(1);
    assert!(r.read_buf(buf.unfilled()).is_err());
    assert_eq!(buf.filled(), &[4]);

    assert!(r.read_buf(buf.unfilled()).is_ok());
    assert_eq!(buf.filled(), &[4]);
    assert_eq!(r.get_ref().0, &[5, 6]);
}

#[test]
fn read_buf_data_and_error_buf() {
    let mut r = BufReader::new(DataAndErrorReader(&[4, 5, 6]));

```

**Entity:** DataAndErrorReader used with Take<...> (via `.take(1)`)

**States:** Ready, ErrorReturnedButProgressed, EOFWithinTakeLimit

**Transitions:**
- Ready -> ErrorReturnedButProgressed via DataAndErrorReader::read_buf writing into cursor then `Err(io::Error::other("error"))`
- ErrorReturnedButProgressed -> EOFWithinTakeLimit via `.take(1)` reaching its limit (subsequent `read_buf` returns Ok with no new filled bytes)

**Evidence:** `DataAndErrorReader::read_buf`: `self.0.read_buf(buf).unwrap();` followed by `Err(io::Error::other("error"))` shows 'progress then error'; test `read_buf_data_and_error_take`: `assert!(r.read_buf(...).is_err()); assert_eq!(buf.filled(), &[4]);` demonstrates progress despite error; same test: second call `assert!(r.read_buf(...).is_ok()); assert_eq!(buf.filled(), &[4]);` shows state change after take-limit reached; same test: `assert_eq!(r.get_ref().0, &[5, 6]);` shows underlying reader state advanced despite earlier Err

**Implementation:** Model read outcomes with a richer return type that encodes 'made progress' vs 'no progress', e.g. `enum ReadBufOutcome { Progress, NoProgress, ProgressWithError(io::Error), NoProgressWithError(io::Error) }` (or a `Result<Progress, (io::Error, Progress)>`-style type). For the `take` wrapper, expose a typestate/capability where after reaching the limit the wrapper transitions to an `Exhausted` state that only permits non-progressing reads (or a distinct `TakeExhausted` type returned by a consuming read).

---

### 56. Thread-local output capture enablement (Disabled/Enabled)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-34`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The thread-local OUTPUT_CAPTURE encodes whether output capturing is active for the current thread: None means disabled; Some(LocalStream) means enabled and points at the capture buffer. Code that writes to the capture buffer must first ensure capture is enabled (and likely coordinate with a global flag indicating whether the TLS should even be consulted). This enable/disable protocol is represented only by Option at runtime; the type system does not distinguish 'capturing' vs 'not capturing' contexts, so misuse must be prevented by conventions/branching.

**Evidence**:

```rust
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
```

**Entity:** OUTPUT_CAPTURE (thread_local static Cell<Option<LocalStream>>)

**States:** Disabled (None), Enabled (Some(LocalStream))

**Transitions:**
- Disabled -> Enabled by setting OUTPUT_CAPTURE to Some(LocalStream)
- Enabled -> Disabled by setting OUTPUT_CAPTURE back to None

**Evidence:** thread_local!: `static OUTPUT_CAPTURE: Cell<Option<LocalStream>> = ... Cell::new(None)` — Option state encodes disabled/enabled; doc comment: "Used by the test crate to capture the output of the print macros and panics." — indicates a conditional mode that must be turned on to take effect

**Implementation:** Introduce a `CaptureToken`/`OutputCaptureGuard` returned by an explicit `enable_capture()` API; printing/capture-writing code takes `Option<&CaptureToken>` or is routed through a `CaptureSink` passed explicitly. The guard's Drop disables capture, making the enabled period explicit and scoped.

---

### 21. LineWriter buffered-vs-direct write protocol (line-buffering shim)

**Location**: `/tmp/io_test_crate/src/io/buffered/linewriter.rs:1-70`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: LineWriter implicitly operates as a line-buffered writer: writes go through LineWriterShim which may buffer data internally and only forward it to the underlying writer at certain boundaries (e.g., newline) and/or when flushed. Whether data is currently pending in the buffer is an implicit runtime state inside `self.inner` (exposed only via debug). The type system does not distinguish when the LineWriter is in a 'has pending buffered data' state vs 'fully flushed' state, nor does it prevent consumers from assuming that a successful `write()` has reached the underlying writer without an explicit `flush()`.

**Evidence**:

```rust

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

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> fmt::Debug for LineWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("LineWriter")
            .field("writer", &self.get_ref())
            .field(
                "buffer",
                &format_args!("{}/{}", self.inner.buffer().len(), self.inner.capacity()),
            )
            .finish_non_exhaustive()
    }
}
```

**Entity:** LineWriter<W>

**States:** Buffering (pending bytes in inner buffer), Direct/Flushed (no pending bytes)

**Transitions:**
- Direct/Flushed -> Buffering via write()/write_all()/write_fmt()/write_vectored()/write_all_vectored() (through LineWriterShim)
- Buffering -> Direct/Flushed via flush()

**Evidence:** method write(&mut self, buf): `LineWriterShim::new(&mut self.inner).write(buf)` indicates writes are mediated by a shim (not necessarily forwarded immediately); method write_all(...): also routed through `LineWriterShim::new(&mut self.inner).write_all(buf)`; method write_fmt(...): also routed through `LineWriterShim::new(&mut self.inner).write_fmt(fmt)`; method flush(&mut self): `self.inner.flush()` is the explicit flushing operation; fmt::Debug impl prints `self.inner.buffer().len()` and `self.inner.capacity()`, showing an internal buffer whose length can vary at runtime

**Implementation:** Expose a typestate wrapper that distinguishes `LineWriter<Flushed, W>` vs `LineWriter<Buffered, W>` (or `LineWriter<W, S>`). Methods that can introduce buffered data (write/write_all/etc.) return `LineWriter<Buffered, W>`; `flush(self)` consumes and returns `LineWriter<Flushed, W>`. This would let APIs that require 'all bytes reached the underlying writer' accept only the Flushed state.

---

### 43. Cursor position/remaining-capacity protocol (pos within bounds; advances on write)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-97`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: These Write impls rely on Cursor's internal `pos` as a state variable that determines where bytes are written and how the cursor advances. Correctness requires `pos` to be within the current backing buffer bounds (slice length or vec length/capacity semantics) before writing; writing then advances `pos`. For slice-backed cursors, writes are bounded by the slice and will only write what fits; for vec-backed cursors, helpers may extend the vec. The type system does not encode whether `pos` is currently in-bounds for the chosen backing store, nor does it distinguish end-of-buffer from other positions—everything is mediated by runtime logic inside helper functions.

**Evidence**:

```rust
    Ok(buf_len)
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
    A: Allocator,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        vec_write_all(&mut self.pos, self.inner, buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        vec_write_all_vectored(&mut self.pos, self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        vec_write_all(&mut self.pos, self.inner, buf)?;
        Ok(())
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        vec_write_all_vectored(&mut self.pos, self.inner, bufs)?;
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<A> Write for Cursor<Vec<u8, A>>
where
    A: Allocator,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        vec_write_all(&mut self.pos, &mut self.inner, buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        vec_write_all_vectored(&mut self.pos, &mut self.inner, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        vec_write_all(&mut self.pos, &mut self.inner, buf)?;
        Ok(())
    }

    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        vec_write_all_vectored(&mut self.pos, &mut self.inner, bufs)?;
        Ok(())
    }

```

**Entity:** Cursor<T> (as used in impl Write for Cursor<&mut [u8]>, Cursor<&mut Vec<u8, A>>, Cursor<Vec<u8, A>>)

**States:** PositionValid (pos <= len), PositionAtEnd (pos == len), PositionOutOfBounds (pos > len)

**Transitions:**
- PositionValid -> PositionValid/PositionAtEnd via write()/write_all()/write_vectored()/write_all_vectored() (advances self.pos)
- PositionAtEnd -> PositionAtEnd (slice-backed) or PositionValid (vec-backed, if vec grows) via write*() (semantics depend on helper)
- PositionOutOfBounds -> (error/partial write behavior) via write*() (handled inside slice_write*/vec_write* helpers)

**Evidence:** write(&mut self, buf): calls slice_write(&mut self.pos, self.inner, buf) / vec_write_all(&mut self.pos, ... , buf) — `self.pos` is explicitly mutated and drives behavior; write_vectored(...): calls slice_write_vectored(&mut self.pos, ...) / vec_write_all_vectored(&mut self.pos, ...) — same state variable across operations; write_all(...): calls slice_write_all(&mut self.pos, ...) / vec_write_all(&mut self.pos, ...) — indicates an implicit contract that repeated writes advance position consistently; flush(&mut self) -> Ok(()) in all impls — no synchronization/commit phase; the only meaningful evolving state here is `pos` and (for vec) length

**Implementation:** Represent cursor position as a type-level witness tied to the backing buffer, e.g., `Cursor<B, P>` where `P` encodes an in-bounds offset (or uses a branded index type returned from `seek`/`set_position`). Writing APIs would require a `P: InBounds` witness and return a new cursor with an updated witness, preventing construction/use of out-of-bounds positions without an explicit checked conversion.

---

### 8. ShortReader consumption + configuration protocol (Configured -> Exhausted)

**Location**: `/tmp/io_test_crate/src/io/copy/tests.rs:1-62`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ShortReader encodes a consumable stream whose remaining bytes are tracked by the runtime field `cap`. Calls to `read()` implicitly transition the object toward exhaustion by decrementing `cap`. The behavior also depends on a configuration invariant: `read_size` is intended to be a fixed per-read upper bound, and `observed_buffer` is only meaningful after reads have occurred (it records the max buffer length ever provided). None of these states/roles are reflected in the type: callers can keep calling `read()` after exhaustion and will continue to get `Ok(0)` without any type-level indication, and tests rely on `observed_buffer` being updated as an observation side effect rather than a typed measurement.

**Evidence**:

```rust

    let mut r = repeat(0).take(1 << 17);
    assert_eq!(copy(&mut r as &mut dyn Read, &mut w as &mut dyn Write).unwrap(), 1 << 17);
}

struct ShortReader {
    cap: usize,
    read_size: usize,
    observed_buffer: usize,
}

impl Read for ShortReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = min(self.cap, self.read_size).min(buf.len());
        self.cap -= bytes;
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(bytes)
    }
}

struct WriteObserver {
    observed_buffer: usize,
}

impl Write for WriteObserver {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn copy_specializes_bufwriter() {
    let cap = 117 * 1024;
    let buf_sz = 16 * 1024;
    let mut r = ShortReader { cap, observed_buffer: 0, read_size: 1337 };
    let mut w = BufWriter::with_capacity(buf_sz, WriteObserver { observed_buffer: 0 });
    assert_eq!(
        copy(&mut r, &mut w).unwrap(),
        cap as u64,
        "expected the whole capacity to be copied"
    );
    assert_eq!(r.observed_buffer, buf_sz, "expected a large buffer to be provided to the reader");
    assert!(w.get_mut().observed_buffer > DEFAULT_BUF_SIZE, "expected coalesced writes");
}

#[test]
fn copy_specializes_bufreader() {
    let mut source = vec![0; 768 * 1024];
    source[1] = 42;
    let mut buffered = BufReader::with_capacity(256 * 1024, Cursor::new(&mut source));

    let mut sink = Vec::new();
    assert_eq!(crate::io::copy(&mut buffered, &mut sink).unwrap(), source.len() as u64);
    assert_eq!(source.as_slice(), sink.as_slice());

    let buf_sz = 71 * 1024;
    assert!(buf_sz > DEFAULT_BUF_SIZE, "test precondition");

```

**Entity:** ShortReader

**States:** Configured(cap > 0), Exhausted(cap == 0)

**Transitions:**
- Configured(cap > 0) -> Exhausted(cap == 0) via Read::read() (cap -= bytes)
- Configured/Exhausted -> (Observed) via Read::read() updating observed_buffer

**Evidence:** field: `cap: usize` represents remaining bytes and is mutated by `self.cap -= bytes` in `Read::read`; field: `read_size: usize` participates in `let bytes = min(self.cap, self.read_size).min(buf.len());` (per-call limit/configuration); field: `observed_buffer: usize` is updated as a side-channel metric: `self.observed_buffer = max(self.observed_buffer, buf.len());`; test `copy_specializes_bufwriter`: constructs `ShortReader { cap, observed_buffer: 0, read_size: 1337 }` and later asserts `copy(...)=cap` and `r.observed_buffer == buf_sz`, relying on these runtime-updated states

**Implementation:** Model the consumable nature as typestate: `ShortReader<State>` where `State` is `Remaining`/`Exhausted`, and `read(self, ...) -> (ShortReader<Remaining>|ShortReader<Exhausted>, usize)` (or expose an iterator-like API) so exhaustion is reflected in the returned type. Alternatively, separate the observation concern into a wrapper `ObservingReader<R>` so that a base reader type doesn't carry an implicit 'has observed anything yet' state.

---

### 25. Lines iterator single-pass consumption protocol

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-107`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: The iterator produced by reader.lines() has an implicit single-pass consumption state machine: successive next() calls yield lines until exhaustion, after which next() must return None forever. The test relies on this temporal ordering and on the fact that next() advances internal cursor state. The type system does not encode the consumed/exhausted state, so misuse (e.g., assuming it can be restarted or cloned) is only prevented by runtime behavior.

**Evidence**:

```rust
    assert_eq!(s, "");
}

#[test]
fn test_lines() {
    let in_buf: &[u8] = b"a\nb\nc";
    let reader = BufReader::with_capacity(2, in_buf);
    let mut it = reader.lines();
    assert_eq!(it.next().unwrap().unwrap(), "a".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "b".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "c".to_string());
    assert!(it.next().is_none());
}

#[test]
fn test_short_reads() {
    let inner = ShortReader { lengths: vec![0, 1, 2, 0, 1, 0] };
    let mut reader = BufReader::new(inner);
    let mut buf = [0, 0];
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
}

#[test]
#[should_panic]
fn dont_panic_in_drop_on_panicked_flush() {
    struct FailFlushWriter;

    impl Write for FailFlushWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::last_os_error())
        }
    }

    let writer = FailFlushWriter;
    let _writer = BufWriter::new(writer);

    // If writer panics *again* due to the flush error then the process will
    // abort.
    panic!();
}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn panic_in_write_doesnt_flush_in_drop() {
    static WRITES: AtomicUsize = AtomicUsize::new(0);

    struct PanicWriter;

    impl Write for PanicWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            WRITES.fetch_add(1, Ordering::SeqCst);
            panic!();
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    thread::spawn(|| {
        let mut writer = BufWriter::new(PanicWriter);
        let _ = writer.write(b"hello world");
        let _ = writer.flush();
    })
    .join()
    .unwrap_err();

    assert_eq!(WRITES.load(Ordering::SeqCst), 1);
}

#[bench]
fn bench_buffered_reader(b: &mut test::Bencher) {
    b.iter(|| BufReader::new(io::empty()));
}

#[bench]
fn bench_buffered_reader_small_reads(b: &mut test::Bencher) {
    let data = (0..u8::MAX).cycle().take(1024 * 4).collect::<Vec<_>>();
    b.iter(|| {
        let mut reader = BufReader::new(&data[..]);
        let mut buf = [0u8; 4];
        for _ in 0..1024 {
            reader.read_exact(&mut buf).unwrap();
            core::hint::black_box(&buf);
        }
    });
}

#[bench]
fn bench_buffered_writer(b: &mut test::Bencher) {
    b.iter(|| BufWriter::new(io::sink()));
}

/// A simple `Write` target, designed to be wrapped by `LineWriter` /
/// `BufWriter` / etc, that can have its `write` & `flush` behavior
/// configured
#[derive(Default, Clone)]
struct ProgrammableSink {
    // Writes append to this slice
```

**Entity:** Lines<BufReader<R>> (iterator returned by BufRead::lines())

**States:** HasMoreLines, Exhausted

**Transitions:**
- HasMoreLines -> HasMoreLines via Lines::next() when a line is available
- HasMoreLines -> Exhausted via Lines::next() when end-of-input reached
- Exhausted -> Exhausted via Lines::next() returning None

**Evidence:** test_lines: let mut it = reader.lines(); then repeated it.next().unwrap().unwrap() yields "a", "b", "c" in order; test_lines: assert!(it.next().is_none()) demonstrates the Exhausted terminal state after consuming all lines

**Implementation:** Model the iterator as a typestate machine where the first call produces either (Item, Self) continuing state or an explicit Exhausted token/type (e.g., an enum carrying the continuation), making the terminal state explicit and preventing APIs that would pretend the iterator can be rewound/reused. In practice this could be expressed via a custom streaming API rather than std::iter::Iterator.

---

### 27. ProgrammableSink write/flush behavior state machine (writes remaining + injected error modes)

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-67`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: ProgrammableSink encodes multiple implicit runtime states that govern whether write()/flush() succeed, return Ok(0), or error. The active behavior is determined by configuration booleans and an internal write-budget counter (max_writes) that is decremented on each successful call to write(). When max_writes reaches 0, behavior splits: either return Ok(0) (EOF-like) or return an injected error depending on error_after_max_writes. Additionally, always_write_error and always_flush_error act like overriding modes. None of these states/transitions are represented in the type system; callers can only discover them by running write()/flush() and observing io::Result outcomes and specific error messages.

**Evidence**:

```rust
    // error; otherwise, it will return Ok(0).
    pub error_after_max_writes: bool,
}

impl Write for ProgrammableSink {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.always_write_error {
            return Err(io::Error::new(io::ErrorKind::Other, "test - always_write_error"));
        }

        match self.max_writes {
            Some(0) if self.error_after_max_writes => {
                return Err(io::Error::new(io::ErrorKind::Other, "test - max_writes"));
            }
            Some(0) => return Ok(0),
            Some(ref mut count) => *count -= 1,
            None => {}
        }

        let len = match self.accept_prefix {
            None => data.len(),
            Some(prefix) => data.len().min(prefix),
        };

        let data = &data[..len];
        self.buffer.extend_from_slice(data);

        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.always_flush_error {
            Err(io::Error::new(io::ErrorKind::Other, "test - always_flush_error"))
        } else {
            Ok(())
        }
    }
}

/// Previously the `LineWriter` could successfully write some bytes but
/// then fail to report that it has done so. Additionally, an erroneous
/// flush after a successful write was permanently ignored.
///
/// Test that a line writer correctly reports the number of written bytes,
/// and that it attempts to flush buffered lines from previous writes
/// before processing new data
///
/// Regression test for #37807
#[test]
fn erroneous_flush_retried() {
    let writer = ProgrammableSink {
        // Only write up to 4 bytes at a time
        accept_prefix: Some(4),

        // Accept the first two writes, then error the others
        max_writes: Some(2),
        error_after_max_writes: true,

        ..Default::default()
    };

    // This should write the first 4 bytes. The rest will be buffered, out
    // to the last newline.
    let mut writer = LineWriter::new(writer);
    assert_eq!(writer.write(b"a\nb\nc\nd\ne").unwrap(), 8);

    // This write should attempt to flush "c\nd\n", then buffer "e". No
```

**Entity:** ProgrammableSink

**States:** NormalWriting, WriteErrorAlways, WritesRemaining(n>0), MaxWritesReached(0), FlushErrorAlways

**Transitions:**
- WritesRemaining(n) -> WritesRemaining(n-1) via write() (when max_writes = Some(count) and count > 0)
- WritesRemaining(1) -> MaxWritesReached(0) via write() (decrement to 0)
- AnyWritableState -> WriteErrorAlways via always_write_error = true (configuration state)
- AnyFlushableState -> FlushErrorAlways via always_flush_error = true (configuration state)
- MaxWritesReached(0) -> (error) via write() when error_after_max_writes = true
- MaxWritesReached(0) -> Ok(0) via write() when error_after_max_writes = false

**Evidence:** field: always_write_error checked in Write::write(): `if self.always_write_error { return Err(..."test - always_write_error") }`; field: max_writes: `match self.max_writes { Some(0) ... Some(ref mut count) => *count -= 1, None => {} }` encodes a runtime write-budget that mutates over time; field: error_after_max_writes gates the terminal behavior: `Some(0) if self.error_after_max_writes => return Err(..."test - max_writes")` else `Some(0) => return Ok(0)`; field: always_flush_error checked in flush(): returns Err(..."test - always_flush_error") when set

**Implementation:** Model sink configuration as distinct types instead of booleans/options, e.g. `ProgrammableSink<WriteMode, FlushMode, Budget>`. Use typestates like `WriteMode = AlwaysErr | Normal`, `Budget = Unlimited | Remaining<const N: usize> | Exhausted<{Err|Zero}>` (or an enum). Provide constructors that yield the appropriate state, and implement `Write` only for states where writing is meaningful; budgeted states can return a new sink state on each write (or wrap in a helper that tracks remaining writes at the type level for tests).

---

### 3. BufWriter panic-safety mode (Normal / In-Panic passthrough)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-71`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: BufWriter has an implicit runtime mode tracked by a boolean (`panicked`) to protect internal invariants when delegating to the underlying writer and that writer may panic. Before calling `get_mut().write_all(buf)` (i.e., bypassing the buffer and writing directly), BufWriter sets `self.panicked = true` and clears it afterwards. Other operations (not shown here, but implied by the existence of `into_parts` error docs) must treat `panicked == true` as a special state where buffered data may be only partially written / recovery is required. This protocol is enforced by setting/clearing a flag at runtime rather than via a type-level state, so callers can end up with a BufWriter value in a 'poisoned/panicked previously' condition that only surfaces later (e.g., at `into_parts`).

**Evidence**:

```rust
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

#[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
/// Error returned for the buffered data from `BufWriter::into_parts`, when the underlying
/// writer has previously panicked.  Contains the (possibly partly written) buffered data.
///
/// # Example
///
/// ```
/// use std::io::{self, BufWriter, Write};
/// use std::panic::{catch_unwind, AssertUnwindSafe};
///
/// struct PanickingWriter;
/// impl Write for PanickingWriter {
///   fn write(&mut self, buf: &[u8]) -> io::Result<usize> { panic!() }
///   fn flush(&mut self) -> io::Result<()> { panic!() }
/// }
///
/// let mut stream = BufWriter::new(PanickingWriter);
/// write!(stream, "some data").unwrap();
/// let result = catch_unwind(AssertUnwindSafe(|| {
///     stream.flush().unwrap()
/// }));
/// assert!(result.is_err());
/// let (recovered_writer, buffered_data) = stream.into_parts();
```

**Entity:** BufWriter<W>

**States:** Normal, InPanicPassthrough

**Transitions:**
- Normal -> InPanicPassthrough via setting `self.panicked = true` before `self.get_mut().write_all(buf)`
- InPanicPassthrough -> Normal via `self.panicked = false` after the write completes normally

**Evidence:** `self.panicked = true;` set immediately before `self.get_mut().write_all(buf);`; `self.panicked = false;` cleared immediately after the `write_all` call; Doc comment on `BufWriter::into_parts`: "Error returned ... when the underlying writer has previously panicked" indicates a persisted/observable panic-related state; Doc comment: "Contains the (possibly partly written) buffered data" indicates the panic state affects buffered-data validity/meaning

**Implementation:** Represent the panic/poisoning mode in the type: `BufWriter<W, S>` where `S` is `Healthy` or `Poisoned`. Operations that can observe/recover buffered data (e.g., `into_parts`) could return `Result<(W, Vec<u8>), (W, BufWriterPanickedState)>` or require `BufWriter<W, Poisoned>` to call recovery APIs. Internally, the transition to `Poisoned` could be performed in a drop-guard that flips the state if unwinding occurs, making the invariant explicit and preventing use as-if-healthy without handling the state.

---

### 60. Take reader remaining-bytes protocol (Remaining(n) -> Exhausted)

**Location**: `/tmp/io_test_crate/src/io/util/tests.rs:1-56`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: A Take<R> value has an implicit runtime state: a remaining byte budget. While Remaining(n>0), reads/iteration yield bytes and decrement the budget; once Exhausted (n==0), subsequent reads yield EOF (0 bytes) and iteration stops. The type system does not distinguish these states, so code can keep calling read()/bytes()/next() without any static indication that the Take has been exhausted.

**Evidence**:

```rust
    assert_eq!(buf, vec![1, 2, 3]);

    let mut buf = String::new();
    assert_eq!(e.read_to_string(&mut buf).unwrap(), 0);
    assert_eq!(buf, "");
    let mut buf = "hello".to_owned();
    assert_eq!(e.read_to_string(&mut buf).unwrap(), 0);
    assert_eq!(buf, "hello");
}

#[test]
fn empty_seeks() {
    let mut e = empty();
    assert!(matches!(e.seek(SeekFrom::Start(0)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Start(1)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Start(u64::MAX)), Ok(0)));

    assert!(matches!(e.seek(SeekFrom::End(i64::MIN)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::End(-1)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::End(0)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::End(1)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::End(i64::MAX)), Ok(0)));

    assert!(matches!(e.seek(SeekFrom::Current(i64::MIN)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Current(-1)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Current(0)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Current(1)), Ok(0)));
    assert!(matches!(e.seek(SeekFrom::Current(i64::MAX)), Ok(0)));
}

#[test]
fn empty_sinks() {
    test_sinking(empty());
}

#[test]
fn repeat_repeats() {
    let mut r = repeat(4);
    let mut b = [0; 1024];
    assert_eq!(r.read(&mut b).unwrap(), 1024);
    assert!(b.iter().all(|b| *b == 4));
}

#[test]
fn take_some_bytes() {
    assert_eq!(repeat(4).take(100).bytes().count(), 100);
    assert_eq!(repeat(4).take(100).bytes().next().unwrap().unwrap(), 4);
    assert_eq!(repeat(1).take(10).chain(repeat(2).take(10)).bytes().count(), 20);
}

#[allow(dead_code)]
fn const_utils() {
    const _: Empty = empty();
    const _: Repeat = repeat(b'c');
    const _: Sink = sink();
}
```

**Entity:** std::io::Take<R> (via repeat(..).take(..))

**States:** Remaining(n > 0), Exhausted(n == 0)

**Transitions:**
- Remaining(n) -> Exhausted via reading/iterating until n reaches 0

**Evidence:** method call: repeat(4).take(100).bytes().count() == 100 asserts that the iterator stops after consuming the implicit byte budget; method call: repeat(4).take(100).bytes().next().unwrap().unwrap() == 4 demonstrates a valid operation only while budget remains; method call: repeat(1).take(10).chain(repeat(2).take(10)).bytes().count() == 20 relies on each Take enforcing its own remaining budget at runtime

**Implementation:** Encode the remaining budget in the type when it is known at compile time (e.g., a const-generic Take<R, const N: usize>), and expose an iterator type that carries the remaining count in its type-level state for certain APIs (or split into Take<Remaining> and Take<Exhausted> returned by a consuming read method).

---

### 13. io::Error representation protocol (Os / Custom / Simple / SimpleMessage)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-69`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: An io::Error value is backed by a runtime-tagged internal representation (via repr.data() -> ErrorData). Many operations (kind(), is_interrupted(), Debug, Display) must pattern-match and perform representation-specific decoding/formatting. This is an implicit state machine: behavior depends on which variant is currently stored, and correctness relies on consistently mapping variant-specific payloads (e.g., OS error codes) into higher-level meanings (ErrorKind, messages). The type system exposes only Error, not a typed distinction between OS-backed errors vs custom/kind-only/message errors, so APIs cannot statically require (or guarantee) that an Error has an OS code available, a stable message, etc.

**Evidence**:

```rust
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Error, ErrorKind};
    ///
    /// fn print_error(err: Error) {
    ///     println!("{:?}", err.kind());
    /// }
    ///
    /// fn main() {
    ///     // As no error has (visibly) occurred, this may print anything!
    ///     // It likely prints a placeholder for unidentified (non-)errors.
    ///     print_error(Error::last_os_error());
    ///     // Will print "AddrInUse".
    ///     print_error(Error::new(ErrorKind::AddrInUse, "oh no!"));
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        match self.repr.data() {
            ErrorData::Os(code) => sys::decode_error_kind(code),
            ErrorData::Custom(c) => c.kind,
            ErrorData::Simple(kind) => kind,
            ErrorData::SimpleMessage(m) => m.kind,
        }
    }

    #[inline]
    pub(crate) fn is_interrupted(&self) -> bool {
        match self.repr.data() {
            ErrorData::Os(code) => sys::is_interrupted(code),
            ErrorData::Custom(c) => c.kind == ErrorKind::Interrupted,
            ErrorData::Simple(kind) => kind == ErrorKind::Interrupted,
            ErrorData::SimpleMessage(m) => m.kind == ErrorKind::Interrupted,
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data() {
            ErrorData::Os(code) => fmt
                .debug_struct("Os")
                .field("code", &code)
                .field("kind", &sys::decode_error_kind(code))
                .field("message", &sys::os::error_string(code))
                .finish(),
            ErrorData::Custom(c) => fmt::Debug::fmt(&c, fmt),
            ErrorData::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
            ErrorData::SimpleMessage(msg) => fmt
                .debug_struct("Error")
                .field("kind", &msg.kind)
                .field("message", &msg.message)
                .finish(),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr.data() {
            ErrorData::Os(code) => {
                let detail = sys::os::error_string(code);
                write!(fmt, "{detail} (os error {code})")
            }
```

**Entity:** Error / Repr (std::io::Error internal representation)

**States:** Os, Custom, Simple, SimpleMessage

**Evidence:** kind(&self): match self.repr.data() { ErrorData::Os(code) => sys::decode_error_kind(code), ErrorData::Custom(c) => c.kind, ErrorData::Simple(kind) => kind, ErrorData::SimpleMessage(m) => m.kind }; is_interrupted(&self): same variant dispatch; Os(code) uses sys::is_interrupted(code) while others compare kind == ErrorKind::Interrupted; impl fmt::Debug for Repr: variant-specific formatting; Os(code) prints code/kind/message via sys::decode_error_kind and sys::os::error_string; impl fmt::Display for Error: Os(code) uses sys::os::error_string(code) and prints "(os error {code})" (representation-specific behavior)

**Implementation:** Expose (at least internally) typed wrappers like Error<OsRepr>, Error<CustomRepr>, etc., or provide capability-returning accessors (e.g., fn os_code(&self) -> Option<OsCode>) where OsCode is a newtype. This allows APIs that require OS codes/messages to take Error<Os> or an OsCode capability rather than a plain Error, eliminating ad-hoc pattern matching and making representation-specific requirements explicit.

---

### 42. Cursor position validity & seek/write protocol (in-bounds / past-end / invalid-negative / overflow)

**Location**: `/tmp/io_test_crate/src/io/cursor/tests.rs:1-85`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Cursor maintains an internal position and supports transitions via seek()/set_position(). The tests rely on implicit rules: (1) seek to before 0 is invalid and must error; (2) seeking past end is allowed and subsequent writes extend the underlying Vec, implicitly filling the gap with zeros; (3) positions that cannot be represented for Vec growth (e.g., > usize::MAX on 32-bit) cause writes to error. These are runtime-enforced by returning Err from seek/write methods; the type system does not distinguish a Cursor whose current position is valid for writing from one where the next operation must fail or will trigger implicit zero-fill growth.

**Evidence**:

```rust

    assert_eq!(writer.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(writer.position(), 0);
    assert_eq!(writer.write(&[3, 4]).unwrap(), 2);
    let b: &[_] = &[3, 4, 2, 3, 4, 5, 6, 7];
    assert_eq!(&writer.get_ref()[..], b);

    assert_eq!(writer.seek(SeekFrom::Current(1)).unwrap(), 3);
    assert_eq!(writer.write(&[0, 1]).unwrap(), 2);
    let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 7];
    assert_eq!(&writer.get_ref()[..], b);

    assert_eq!(writer.seek(SeekFrom::End(-1)).unwrap(), 7);
    assert_eq!(writer.write(&[1, 2]).unwrap(), 2);
    let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 1, 2];
    assert_eq!(&writer.get_ref()[..], b);

    assert_eq!(writer.seek(SeekFrom::End(1)).unwrap(), 10);
    assert_eq!(writer.write(&[1]).unwrap(), 1);
    let b: &[_] = &[3, 4, 2, 0, 1, 5, 6, 1, 2, 0, 1];
    assert_eq!(&writer.get_ref()[..], b);
}

#[test]
fn vec_seek_past_end() {
    let mut r = Cursor::new(Vec::new());
    assert_eq!(r.seek(SeekFrom::Start(10)).unwrap(), 10);
    assert_eq!(r.write(&[3]).unwrap(), 1);
}

#[test]
fn vec_seek_before_0() {
    let mut r = Cursor::new(Vec::new());
    assert!(r.seek(SeekFrom::End(-2)).is_err());
}

#[test]
#[cfg(target_pointer_width = "32")]
fn vec_seek_and_write_past_usize_max() {
    let mut c = Cursor::new(Vec::new());
    c.set_position(usize::MAX as u64 + 1);
    assert!(c.write_all(&[1, 2, 3]).is_err());
}

#[test]
fn test_partial_eq() {
    assert_eq!(Cursor::new(Vec::<u8>::new()), Cursor::new(Vec::<u8>::new()));
}

#[test]
fn test_eq() {
    struct AssertEq<T: Eq>(pub T);

    let _: AssertEq<Cursor<Vec<u8>>> = AssertEq(Cursor::new(Vec::new()));
}

#[allow(dead_code)]
fn const_cursor() {
    const CURSOR: Cursor<&[u8]> = Cursor::new(&[0]);
    const _: &&[u8] = CURSOR.get_ref();
    const _: u64 = CURSOR.position();
}

#[bench]
fn bench_write_vec(b: &mut test::Bencher) {
    let slice = &[1; 128];

    b.iter(|| {
        let mut buf = b"some random data to overwrite".to_vec();
        let mut cursor = Cursor::new(&mut buf);

        let _ = cursor.write_all(slice);
        test::black_box(&cursor);
    })
}

#[bench]
fn bench_write_vec_vectored(b: &mut test::Bencher) {
    let slices = [
        IoSlice::new(&[1; 128]),
        IoSlice::new(&[2; 256]),
        IoSlice::new(&[3; 512]),
        IoSlice::new(&[4; 1024]),
        IoSlice::new(&[5; 2048]),
        IoSlice::new(&[6; 4096]),
```

**Entity:** std::io::Cursor<T> (as used with Vec<u8>/&mut Vec<u8>/&[u8])

**States:** PositionInBounds, PositionPastEnd, InvalidPosition (negative / before 0), OverflowPosition (past usize::MAX for Vec-backed writes)

**Transitions:**
- PositionInBounds -> PositionPastEnd via seek(SeekFrom::End(positive)) / seek(SeekFrom::Start(large)) / set_position(large)
- PositionPastEnd -> PositionInBounds via seek(SeekFrom::Start(smaller)) / seek(SeekFrom::Current(negative))
- Any -> InvalidPosition via seek(SeekFrom::End(negative-too-large))
- Any -> OverflowPosition via set_position(usize::MAX as u64 + 1) (Vec-backed cursor on 32-bit)

**Evidence:** calls to seek(...) with unwrap(): `writer.seek(SeekFrom::Start(0)).unwrap()`, `writer.seek(SeekFrom::Current(1)).unwrap()`, `writer.seek(SeekFrom::End(-1)).unwrap()`, `writer.seek(SeekFrom::End(1)).unwrap()` imply a position state that must be valid for later write(); implicit gap-fill/extend behavior after seeking past end: `assert_eq!(writer.seek(SeekFrom::End(1)).unwrap(), 10);` then `writer.write(&[1]).unwrap()` and later `let b: &[_] = ... 0, 1]; assert_eq!(&writer.get_ref()[..], b);` (expects inserted 0 at index 9); `vec_seek_past_end`: `r.seek(SeekFrom::Start(10)).unwrap()` followed by `r.write(&[3]).unwrap()` demonstrates allowed PositionPastEnd state for write with Vec growth; `vec_seek_before_0`: `assert!(r.seek(SeekFrom::End(-2)).is_err());` demonstrates InvalidPosition state detected at runtime; `vec_seek_and_write_past_usize_max` (32-bit): `c.set_position(usize::MAX as u64 + 1); assert!(c.write_all(&[1,2,3]).is_err());` demonstrates OverflowPosition only failing on write at runtime

**Implementation:** Introduce state-indexed wrappers around Cursor for Vec-backed usage, e.g. `CursorVec<S>` where `S` encodes whether the position is known-valid-for-write. Provide fallible transitions like `try_seek(...) -> Result<CursorVec<WriteablePos>, CursorVec<Unknown/Invalid>>` and/or use newtypes for validated positions (`NonNegativeOffset`, `VecIndexUsize`) so APIs that require a representable Vec index cannot be called after `set_position(u64)` without re-validation.

---

### 7. Repr tagged-pointer encoding protocol (SimpleMessage / Custom / Os / Simple)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-79`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Repr encodes which error payload is stored by embedding a 2-bit tag in the low bits of a NonNull<()> pointer. Correctness relies on an implicit invariant that the stored pointer is aligned such that its two least-significant bits are zero before tagging, and that the tag value determines how the payload must later be interpreted/decoded. The type system only sees NonNull<()> and does not prevent constructing a Repr with an invalid tag, a misaligned pointer, or a payload/tag mismatch (e.g., tagging a Custom pointer as Os).

**Evidence**:

```rust
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
```

**Entity:** Repr

**States:** Tagged(SimpleMessage), Tagged(Custom), Tagged(Os), Tagged(Simple)

**Transitions:**
- ErrorData::Os(code) -> Tagged(Os) via Repr::new_os(code) (called from Repr::new)
- ErrorData::Simple(kind) -> Tagged(Simple) via Repr::new_simple(kind) (called from Repr::new)
- ErrorData::SimpleMessage(msg) -> Tagged(SimpleMessage) via Repr::new_simple_message(msg) (called from Repr::new)
- ErrorData::Custom(b) -> Tagged(Custom) via Repr::new_custom(b) (called from Repr::new)

**Evidence:** const TAG_MASK: usize = 0b11; and TAG_* constants: explicit 2-bit tag state machine; pub(super) struct Repr(NonNull<()>, PhantomData<ErrorData<Box<Custom>>>): stores erased payload in NonNull<()> (type does not encode which variant is present); impl Repr::new(dat: ErrorData<Box<Custom>>) matches on ErrorData::{Os, Simple, SimpleMessage, Custom} and dispatches to new_*: tag is chosen based on runtime variant; Repr::new_custom: debug_assert_eq!(p.addr() & TAG_MASK, 0): runtime-only check that pointer alignment leaves tag bits free; Repr::new_custom: let tagged = p.wrapping_add(TAG_CUSTOM).cast::<()>(); with comment 'TAG_CUSTOM + p is the same as TAG_CUSTOM | p, because p's alignment means it isn't allowed to have any of the ...': relies on alignment/bit-level invariant not enforced by types

**Implementation:** Represent the internal variants at the type level: e.g., enum ReprInner { SimpleMessage(SimpleMessage), Custom(NonNull<Custom>), Os(RawOsError), Simple(ErrorKind) } for safe mode; or keep bitpacking but introduce a private newtype TaggedPtr<Tag> where Tag is a zero-sized type (SimpleMessageTag/CustomTag/OsTag/SimpleTag) and only constructors for each tag can produce the corresponding TaggedPtr. Repr would then store a TaggedPtr<AnyTag> plus an explicit tag enum, or expose only safe constructors returning Repr<CustomTag>/Repr<OsTag> etc. so decoding methods are only available for the matching tag.

---

### 17. Stdout/Stderr validity protocol (FD open vs closed/invalid; EBADF treated as success)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-100`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: These Write impls implicitly rely on a runtime state of the underlying OS handle: writes/flushes may fail with EBADF (e.g., stdout/stderr closed or unavailable). Instead of surfacing an error, EBADF is converted to a successful no-op (or 'all bytes written'). This encodes an implicit state machine where, once the handle becomes invalid, operations should behave as if they succeed. The type system does not distinguish a 'disabled/closed' stdout/stderr from a valid one, so callers cannot know at compile time whether writing has real effects or is being silently ignored.

**Evidence**:

```rust
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
```

**Entity:** StdoutRaw / StderrRaw (Write impls)

**States:** Valid (underlying FD writable), Invalid/Closed (EBADF)

**Transitions:**
- Valid -> Invalid/Closed when underlying write/flush returns an EBADF error (detected by stdio::is_ebadf)

**Evidence:** fn handle_ebadf<T>(r: io::Result<T>, default: impl FnOnce() -> io::Result<T>) -> io::Result<T>: matches Err(e) if stdio::is_ebadf(e) => default(); impl Write for StdoutRaw: write() uses handle_ebadf(self.0.write(buf), || Ok(buf.len())) (EBADF becomes 'wrote everything'); impl Write for StdoutRaw: flush() uses handle_ebadf(self.0.flush(), || Ok(())) (EBADF becomes success); impl Write for StderrRaw mirrors the same EBADF-to-success behavior in write()/flush()/write_all()/write_fmt()

**Implementation:** Represent stdout/stderr as an enum/typestate like Stdout<Enabled> vs Stdout<Disabled> (or a newtype wrapper that is known-non-EBADF). Construction would decide the state once (or a fallible acquire step), and Write would only be implemented for Enabled. For the 'ignore EBADF' policy, expose an explicit wrapper like IgnoreEbadf<W: Write>(W) so the silent-success behavior is opt-in and visible in types.

---

### 5. Buffer internal cursor/filled window invariant (pos <= filled <= initialized <= capacity)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader/buffer.rs:1-125`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Buffer encodes a runtime state machine using the integer fields pos/filled/initialized, with an implicit invariant that the readable window is buf[pos..filled] and that this region is initialized. Correctness of buffer() relies on pos <= filled, and on filled/initialized tracking which bytes in the underlying MaybeUninit slice are actually initialized. Many methods assume and maintain these relationships, but the type system does not prevent constructing/transitioning into invalid combinations (e.g., filled > initialized, filled > capacity, pos > filled) except by convention and careful method implementation.

**Evidence**:

```rust
//! that user code which wants to do reads from a `BufReader` via `buffer` + `consume` can do so
//! without encountering any runtime bounds checks.

use crate::cmp;
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
```

**Entity:** Buffer

**States:** Empty (pos=0, filled=0), HasAvailable (pos < filled), Exhausted (pos == filled), HasBacklogForBackshift (pos > 0)

**Transitions:**
- Empty/Exhausted/HasAvailable -> Empty via discard_buffer() (sets pos=0, filled=0)
- HasAvailable -> HasAvailable/Exhausted via consume() (pos increases but clamped to filled)
- HasAvailable -> HasAvailable/Exhausted via consume_with() on success (pos += amt after bounds-checked slice claim)
- Any -> (more available) via read_more() (filled increases by bytes read; initialized adjusted)
- HasBacklogForBackshift -> Exhausted/HasAvailable via backshift() (moves buf[pos..] to front; filled -= pos; pos=0)
- HasAvailable/Exhausted -> HasAvailable (more data) via fill_buf() (not shown fully, but comment indicates it refills when at end)

**Evidence:** struct fields: pos, filled, initialized are plain usize counters encoding the buffer state at runtime; field comment on pos: "must always be <= `filled`"; field comment on filled: "Each call to `fill_buf` sets `filled` to indicate how many bytes ... are initialized"; field comment on initialized: tracks "max number of bytes returned" to tell read_buf how many bytes are initialized; buffer(): unsafe get_unchecked(self.pos..self.filled).assume_init_ref() with SAFETY comment relying on invariants of this type; consume(): self.pos = cmp::min(self.pos + amt, self.filled) (runtime maintenance of pos<=filled); consume_with(): uses self.buffer().get(..amt) to check availability; then does self.pos += amt relying on that check; unconsume(): self.pos = self.pos.saturating_sub(amt) (allows moving pos backward, implying additional states like HasBacklogForBackshift); discard_buffer(): sets pos and filled to 0 (explicit state reset); backshift(): copy_within(self.pos.., 0); then self.filled -= self.pos; self.pos = 0 (state transition that assumes pos is a valid offset)

**Implementation:** Encode the cursor/availability states in the type: e.g., Buffer<S> where S is one of {Empty, Available, Exhausted}. Provide methods that transition between states (consume: Available->Available/Exhausted; fill/read_more: Exhausted/Empty->Available; discard: any->Empty). Additionally, represent the readable window as a dedicated newtype (e.g., struct Window { start: usize, end: usize }) that can only be constructed by methods maintaining start<=end<=capacity, so buffer() no longer needs unchecked slicing.

---

### 47. io::Error representation protocol (Custom / OS / Simple) gating downcast behavior

**Location**: `/tmp/io_test_crate/src/io/error/tests.rs:1-61`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: These tests rely on an implicit state machine inside std::io::Error: the error can carry a boxed custom error (created via Error::new), or an OS errno (created via Error::from_raw_os_error), or a simple/const-like representation (created via const_error!). Whether downcast() can succeed depends on being in the Custom state and having the requested concrete type; in other states downcast::<T>() must fail and return the original io::Error unchanged. None of these representation states are reflected in the type of Error, so callers can only discover the state by attempting downcast and handling Ok/Err at runtime.

**Evidence**:

```rust

    const CONST: Error = const_error!(NotFound, "definitely a constant!");
    check_simple_msg!(CONST, NotFound, "definitely a constant!");

    static STATIC: Error = const_error!(BrokenPipe, "a constant, sort of!");
    check_simple_msg!(STATIC, BrokenPipe, "a constant, sort of!");
}

#[derive(Debug, PartialEq)]
struct Bojji(bool);
impl error::Error for Bojji {}
impl fmt::Display for Bojji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ah! {:?}", self)
    }
}

#[test]
fn test_custom_error_packing() {
    use super::Custom;
    let test = Error::new(ErrorKind::Uncategorized, Bojji(true));
    assert_matches!(
        test.repr.data(),
        ErrorData::Custom(Custom {
            kind: ErrorKind::Uncategorized,
            error,
        }) if error.downcast_ref::<Bojji>().as_deref() == Some(&Bojji(true)),
    );
}

#[derive(Debug)]
struct E;

impl fmt::Display for E {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl error::Error for E {}

#[test]
fn test_std_io_error_downcast() {
    // Case 1: custom error, downcast succeeds
    let io_error = Error::new(ErrorKind::Other, Bojji(true));
    let e: Bojji = io_error.downcast().unwrap();
    assert!(e.0);

    // Case 2: custom error, downcast fails
    let io_error = Error::new(ErrorKind::Other, Bojji(true));
    let io_error = io_error.downcast::<E>().unwrap_err();

    //   ensures that the custom error is intact
    assert_eq!(ErrorKind::Other, io_error.kind());
    let e: Bojji = io_error.downcast().unwrap();
    assert!(e.0);

    // Case 3: os error
    let errno = 20;
    let io_error = Error::from_raw_os_error(errno);
    let io_error = io_error.downcast::<E>().unwrap_err();

```

**Entity:** std::io::Error (as used by tests in src::io::error::tests)

**States:** Custom(Box<dyn Error>), Os(i32 errno), Simple(kind-only/const-like)

**Transitions:**
- Custom(Box<dyn Error>) -> (Ok(T)) via Error::downcast::<T>() when the stored custom error is T
- Custom(Box<dyn Error>) -> Custom(Box<dyn Error>) via Error::downcast::<T>() returning Err(self) when T mismatches
- Os(i32 errno) -> Os(i32 errno) via Error::downcast::<T>() returning Err(self)
- Simple(kind-only/const-like) -> Simple(kind-only/const-like) via Error::downcast::<T>() returning Err(self)

**Evidence:** CONST/STATIC created via const_error!(NotFound/ BrokenPipe, "...") and checked with check_simple_msg!(...) implies a simple/const-like representation distinct from custom payloads; test_std_io_error_downcast Case 1: `let io_error = Error::new(ErrorKind::Other, Bojji(true)); let e: Bojji = io_error.downcast().unwrap();` shows a 'Custom' state where downcast can succeed; test_std_io_error_downcast Case 2: `let io_error = Error::new(..., Bojji(true)); let io_error = io_error.downcast::<E>().unwrap_err();` shows downcast failure returning the original Error; comment `//   ensures that the custom error is intact` plus `let e: Bojji = io_error.downcast().unwrap();` asserts that failed downcast preserves the underlying custom payload/state; test_std_io_error_downcast Case 3: `let io_error = Error::from_raw_os_error(errno); let io_error = io_error.downcast::<E>().unwrap_err();` shows an OS-error state where downcast must fail

**Implementation:** Model representation at the type level, e.g. `struct IoError<R> { kind: ErrorKind, repr: R }` with `Custom(Box<dyn Error>)`, `Os(i32)`, `Simple(&'static str/...)` as distinct R types; implement `downcast` only for `IoError<Custom>` (or return a typed `Result<T, IoError<Custom>>`), making it impossible to call a "must fail" downcast on OS/Simple errors without an explicit conversion.

---

### 11. Repr tagged-pointer encoding protocol (Os / Simple / SimpleMessage / Custom)

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-126`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Repr encodes an io::Error representation in the bits of a NonNull<()> using tag values (TAG_OS/TAG_SIMPLE/TAG_SIMPLE_MESSAGE/TAG_CUSTOM). Correctness relies on an implicit protocol: the internal pointer must always be produced by the module's constructors so that tag bits and payload layout match decode_repr's expectations. This is not enforced by the type system: decode_repr is `unsafe` and assumes its `ptr` argument is a valid Repr encoding; if the tag/payload are inconsistent (e.g., invalid ErrorKind discriminant, wrong tag for pointer), the code hits debug_assert/unreachable_unchecked or performs invalid pointer casts. Additionally, when in the Custom state, Repr implicitly owns the allocation and must be dropped exactly once or transferred via into_data; this ownership is enforced only by Drop/ManuallyDrop patterns, not by distinct types.

**Evidence**:

```rust
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
```

**Entity:** Repr

**States:** Os, Simple(ErrorKind), SimpleMessage(&'static SimpleMessage), Custom(Box<Custom> owned pointer)

**Transitions:**
- Custom(Box<Custom> owned pointer) -> (dropped/freed) via Drop::drop()
- Custom(Box<Custom> owned pointer) -> Custom(Box<Custom>) extracted via Repr::into_data(self) (prevents double-drop using ManuallyDrop)
- Any state -> ErrorData<&Custom> view via Repr::data(&self) (requires valid encoding)
- Any state -> ErrorData<&mut Custom> view via Repr::data_mut(&mut self) (requires valid encoding)

**Evidence:** Repr::data(): "Safety: We're a Repr, decode_repr is fine." then `unsafe { decode_repr(self.0, |c| &*c) }`; Repr::data_mut(): same pattern with `|c| &mut *c`; Repr::into_data(self): uses `ManuallyDrop::new(self)` and comment "safe because we prevent double-drop using `ManuallyDrop`"; then `decode_repr(this.0, |p| Box::from_raw(p))`; impl Drop for Repr: always calls `decode_repr(self.0, |p| Box::<Custom>::from_raw(p))` to free Custom payload when tagged as TAG_CUSTOM; decode_repr Safety contract comment: "ptr's bits should be encoded as described ... (it should `some_repr.0`)"; decode_repr TAG_SIMPLE branch: `kind_from_prim(kind_bits).unwrap_or_else(... debug_assert!(false, "Invalid io::error::Repr bits..."); unsafe { core::hint::unreachable_unchecked() })` indicates invalid-state UB if encoding is wrong; decode_repr TAG_SIMPLE_MESSAGE branch: `ErrorData::SimpleMessage(&*ptr.cast::<SimpleMessage>().as_ptr())` relies on tag guaranteeing correct pointee type/lifetime; decode_repr TAG_CUSTOM branch: `wrapping_byte_sub(TAG_CUSTOM).cast::<Custom>()` depends on pointer arithmetic protocol for custom encoding

**Implementation:** Make the internal representation a typed enum (or a generic `Repr<Tag>` with PhantomData) so decoding does not depend on raw tag bits. For example: `enum ReprInner { Os(RawOsError), Simple(ErrorKind), SimpleMessage(&'static SimpleMessage), Custom(NonNull<Custom>) }` (or `Box<Custom>` directly). If bitpacking is required, introduce a private `struct TaggedPtr(NonNull<()>);` plus a sealed `unsafe trait Tag` implemented only for the four tags, and expose constructors returning distinct wrapper types (`ReprOs`, `ReprSimple`, `ReprCustomOwned`). Only the Custom-owned wrapper implements Drop / into_box, preventing calling `Box::from_raw` unless the type proves ownership and correct tag.

---

### 49. BorrowedCursor capacity-bounded write protocol (HasCapacity / Full)

**Location**: `/tmp/io_test_crate/src/io/impls.rs:1-83`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Writing into a BorrowedCursor is capacity-bounded: writes may be partial depending on remaining capacity. Methods like write_all()/write_all_vectored() rely on a runtime check comparing bytes written to requested length and return WRITE_ALL_EOF when the cursor is full (or becomes full mid-operation). The type system does not distinguish a cursor with remaining capacity from a full cursor, so callers can only discover exhaustion at runtime.

**Evidence**:

```rust
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let len = bufs.iter().map(|b| b.len()).sum();
        self.reserve(len);
        for buf in bufs {
            self.extend(&**buf);
        }
        Ok(len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend(buf);
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

#[unstable(feature = "read_buf", issue = "78485")]
impl<'a> io::Write for core::io::BorrowedCursor<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.capacity());
        self.append(&buf[..amt]);
        Ok(amt)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut nwritten = 0;
        for buf in bufs {
            let n = self.write(buf)?;
            nwritten += n;
            if n < buf.len() {
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
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.write(buf)? < buf.len() { Err(io::Error::WRITE_ALL_EOF) } else { Ok(()) }
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
```

**Entity:** core::io::BorrowedCursor<'a>

**States:** HasCapacity, Full

**Transitions:**
- HasCapacity -> Full via write()/append() when self.capacity() reaches 0

**Evidence:** write(): `let amt = cmp::min(buf.len(), self.capacity());` followed by `self.append(&buf[..amt]);` encodes partial writes based on remaining capacity; write_vectored(): breaks early when `n < buf.len()` indicating capacity exhaustion mid-vector; write_all(): `if self.write(buf)? < buf.len() { Err(io::Error::WRITE_ALL_EOF) }` uses runtime check/error to enforce the invariant "must have enough capacity"; write_all_vectored(): returns `Err(io::Error::WRITE_ALL_EOF)` when any slice cannot be fully written

**Implementation:** Model remaining-capacity as a type-level state/capability: e.g., have `BorrowedCursor<HasCapacity>` and `BorrowedCursor<Full>` (or a `CapacityToken<N>` newtype) where `write_all` requires a proof/capability that sufficient capacity exists. A checked method like `try_reserve_exact(len) -> Result<BorrowedCursor<HasCapacity>, BorrowedCursor<Full>>` (or returning a token) can move the runtime check to a single transition point and make subsequent full writes infallible while the token is held.

---

### 31. Take limit protocol (Remaining > 0 / EOF-by-limit)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-66`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Take<T> carries an implicit runtime state in `self.limit`: when `limit == 0`, it must behave as EOF without calling into the inner reader (to avoid blocking), and methods must monotonically decrease `limit` based on bytes actually produced/consumed. This is enforced with runtime checks and arithmetic, not at the type level; callers can still hold a `Take<T>` value without any static indication whether reads may delegate to the inner reader or will immediately return EOF.

**Evidence**:

```rust
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

/// An iterator over `u8` values of a reader.
///
/// This struct is generally created by calling [`bytes`] on a reader.
/// Please see the documentation of [`bytes`] for more details.
///
/// [`bytes`]: Read::bytes
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Bytes<R> {
    inner: R,
}

#[stable(feature = "rust1", since = "1.0.0")]
```

**Entity:** Take<T>

**States:** Remaining(limit > 0), EOF(limit == 0)

**Transitions:**
- Remaining(limit > 0) -> EOF(limit == 0) via read/read_buf/fill_buf/consume as limit is decremented

**Evidence:** `if self.limit == 0 { return Ok(&[]); }` in `BufRead for Take<T>::fill_buf()` comment: "Don't call into inner reader at all at EOF because it may still block"; `self.limit -= filled as u64;` in `read_buf`-like logic (decrementing remaining limit based on bytes filled); `self.limit -= (buf.written() - written) as u64;` in the else-branch (decrement based on actual bytes written); `consume(&mut self, amt: usize)` clamps: `let amt = cmp::min(amt as u64, self.limit) as usize;` with comment: "Don't let callers reset the limit by passing an overlarge value" and then `self.limit -= amt as u64;`

**Implementation:** Represent the 'has remaining budget' vs 'at EOF-by-limit' distinction in the type: e.g., `Take<T, S>` with `S = Remaining`/`Eof`. Provide a method like `try_remaining(self) -> Result<Take<T, Remaining>, Take<T, Eof>>` or `split(self) -> (Take<T, Eof>, /*unused*/)` so code that must not call inner reader can be written against `Take<_, Eof>`. Alternatively, make reading APIs return an enum `TakeRead<'a, T> = Inner(&'a mut T) | Eof` capability to gate inner access when `limit==0`.

---

### 1. BufReader buffered/unbuffered view invariant (buffer may contain unread data)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-73`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: BufReader maintains an internal buffer that may contain unread bytes that have been pulled from the underlying reader. Calling into_inner(self) returns the underlying reader but silently drops any remaining buffered bytes, meaning subsequent reads from the returned R may observe a later position than expected (effective data loss from the perspective of the consumer). This is a latent state/protocol: users must ensure the buffer is empty (or that they intentionally abandon buffered bytes) before extracting the inner reader. The type system does not distinguish 'safe to unwrap' (empty buffer) from 'will drop data' (buffer non-empty).

**Evidence**:

```rust
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
    Self: Read,
{
```

**Entity:** BufReader<R>

**States:** Buffered (internal buffer contains unread/valid data), Unbuffered (no buffered data / buffer discarded)

**Transitions:**
- Buffered -> Unbuffered via discard_buffer(&mut self)
- Buffered -> (Dropped buffered bytes) via into_inner(self)

**Evidence:** into_inner(self) docs: "any leftover data in the internal buffer is lost" and "may lead to data loss"; method into_inner(self) returns self.inner without checking/handling buffered remainder; method discard_buffer(&mut self) "Invalidates all data in the internal buffer"; discard_buffer calls self.buf.discard_buffer() indicating explicit runtime buffer-validity state

**Implementation:** Encode buffer state at the type level, e.g., BufReader<R, S> with S = Buffered | Empty. Methods that can create buffered data (read/fill_buf) move to Buffered; discard_buffer(self) -> BufReader<R, Empty>; into_inner(self) only implemented for BufReader<R, Empty> (or provide into_inner_lossy(self) for Buffered).

---

### 33. Buffered read state machine (BufferEmpty / BufferHasData, and fast-path bypass when buffer exhausted)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: BufReader maintains an implicit runtime state describing whether its internal buffer currently contains unread bytes (pos < filled) or is exhausted (pos == filled). Many methods select different behavior based on this state: they either serve reads from the buffer, refill it, or discard it and bypass buffering to delegate directly to the inner reader for large vectored reads. This protocol is enforced by runtime checks (`pos()==filled()`, buffer consumption success) and by calling the right sequencing methods (`fill_buf` before reading from `rem`, `consume` after reading, `discard_buffer` before delegating). The type system does not prevent calling the 'buffer-using' path when the buffer is empty, forgetting to `consume`, or delegating without first discarding buffered bytes; correctness relies on internal discipline.

**Evidence**:

```rust
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
```

**Entity:** BufReader<R> (the surrounding buffered reader type implied by `self.buf`, `self.inner`, `fill_buf`, `consume`, `discard_buffer`)

**States:** BufferHasData, BufferEmptyOrExhausted, BypassBufferDirectRead (transient)

**Transitions:**
- BufferHasData -> BufferEmptyOrExhausted via consume(n) / internal consumption (e.g., consume_with)
- BufferEmptyOrExhausted -> BufferHasData via fill_buf()
- BufferEmptyOrExhausted -> BypassBufferDirectRead via discard_buffer() + inner.read_vectored(...) fast path
- BufferHasData -> BufferEmptyOrExhausted via discard_buffer() (e.g., before read_to_end delegation)

**Evidence:** read_exact/read_buf_exact: `self.buf.consume_with(...){ return Ok(()) }` chooses buffered consumption vs `default_read_exact(self, ...)` fallback; read_vectored: `if self.buf.pos() == self.buf.filled() && total_len >= self.capacity() { self.discard_buffer(); return self.inner.read_vectored(bufs); }` shows an 'exhausted buffer' check gating a bypass-to-inner fast path; read_vectored: `let mut rem = self.fill_buf()?; let nread = rem.read_vectored(bufs)?; self.consume(nread);` requires temporal ordering: fill_buf() -> read from returned slice -> consume(nread); read_to_end: drains `let inner_buf = self.buffer(); ... self.discard_buffer(); ... self.inner.read_to_end(buf)?` enforces 'drain/discard before delegating' behavior

**Implementation:** Split the internal buffer handling into state-typed helpers, e.g. `struct BufReader<R, S> { inner: R, buf: Buf, _s: PhantomData<S> }` with `HasData` and `Exhausted` states. `fill_buf(self) -> BufReader<R, HasData>` (or a guard type borrowing self) and `consume(self, n) -> BufReader<R, S2>` encode the required sequencing, while the bypass path is only available from `Exhausted` state and consumes/discards the buffer before handing out `&mut R` for direct reads.

---

### 44. Error payload accessibility protocol (HasInner / NoInner / Extracted)

**Location**: `/tmp/io_test_crate/src/io/error/tests.rs:1-69`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Error sometimes contains an inner error object (created via Error::new), and sometimes is just a kind/OS-code/simple message (e.g., from ErrorKind or from_raw_os_error). APIs like get_ref()/get_mut()/into_inner() return Option and are only meaningful when an inner payload exists. Additionally, into_inner() consumes the Error and transitions it into an 'Extracted' state (the Error value is gone and the inner payload is moved out). These states and valid operations are enforced via runtime Option checks/unwrapping in tests rather than being reflected in distinct types.

**Evidence**:

```rust
         }} \
         }}",
        code, kind, msg
    );
    assert_eq!(format!("{err:?}"), expected);
}

#[test]
fn test_downcasting() {
    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("asdf")
        }
    }

    impl error::Error for TestError {}

    // we have to call all of these UFCS style right now since method
    // resolution won't implicitly drop the Send+Sync bounds
    let mut err = Error::new(ErrorKind::Other, TestError);
    assert!(err.get_ref().unwrap().is::<TestError>());
    assert_eq!("asdf", err.get_ref().unwrap().to_string());
    assert!(err.get_mut().unwrap().is::<TestError>());
    let extracted = err.into_inner().unwrap();
    extracted.downcast::<TestError>().unwrap();
}

#[test]
fn test_const() {
    const E: Error = const_error!(ErrorKind::NotFound, "hello");

    assert_eq!(E.kind(), ErrorKind::NotFound);
    assert_eq!(E.to_string(), "hello");
    assert!(format!("{E:?}").contains("\"hello\""));
    assert!(format!("{E:?}").contains("NotFound"));
}

#[test]
fn test_os_packing() {
    for code in -20..20 {
        let e = Error::from_raw_os_error(code);
        assert_eq!(e.raw_os_error(), Some(code));
        assert_matches!(
            e.repr.data(),
            ErrorData::Os(c) if c == code,
        );
    }
}

#[test]
fn test_errorkind_packing() {
    assert_eq!(Error::from(ErrorKind::NotFound).kind(), ErrorKind::NotFound);
    assert_eq!(Error::from(ErrorKind::PermissionDenied).kind(), ErrorKind::PermissionDenied);
    assert_eq!(Error::from(ErrorKind::Uncategorized).kind(), ErrorKind::Uncategorized);
    // Check that the innards look like what we want.
    assert_matches!(
        Error::from(ErrorKind::OutOfMemory).repr.data(),
        ErrorData::Simple(ErrorKind::OutOfMemory),
    );
}

#[test]
fn test_simple_message_packing() {
    use super::ErrorKind::*;
    use super::SimpleMessage;
    macro_rules! check_simple_msg {
```

**Entity:** std::io::Error ("Error")

**States:** NoInner (kind-only), HasInner (boxed inner error), Extracted (inner moved out)

**Transitions:**
- NoInner -> HasInner via Error::new(ErrorKind, E)
- HasInner -> Extracted via Error::into_inner()

**Evidence:** test_downcasting: `let mut err = Error::new(ErrorKind::Other, TestError);` constructs an Error expected to have an inner payload; test_downcasting: `err.get_ref().unwrap()` and `err.get_mut().unwrap()` — unwrap implies a required 'HasInner' state for these calls to succeed; test_downcasting: `let extracted = err.into_inner().unwrap();` — into_inner() returns Option and consumes `err`, indicating a state-dependent operation and a terminal transition; test_errorkind_packing: `Error::from(ErrorKind::NotFound)` and friends create Errors without an inner payload (kind-only), contrasting with Error::new() usage; test_os_packing: `Error::from_raw_os_error(code)` yields an Error where only OS code is expected/checked (`raw_os_error() == Some(code)`), not an inner error object

**Implementation:** Introduce distinct wrapper types or a generic state parameter, e.g. `struct Error<S> { repr: Repr, _s: PhantomData<S> }` with `WithInner` and `NoInner`. Provide `get_ref/get_mut/into_inner` only on `Error<WithInner>`, and constructors like `Error::new` returning `Error<WithInner>` while `from(ErrorKind)`/`from_raw_os_error` return `Error<NoInner>`; optionally a conversion method to erase to a unified `Error` if needed.

---

### 14. Error representation protocol (OS error vs Custom vs Simple/SimpleMessage)

**Location**: `/tmp/io_test_crate/src/io/error.rs:1-165`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `Error` has multiple implicit runtime variants (backed by `repr`/`ErrorData`). Which APIs return `Some` vs `None` depends on how the `Error` was constructed: OS-constructed errors carry a raw OS code; custom-constructed errors may carry an inner `dyn Error`; simple/static-message errors carry only messages. This is enforced only by runtime matching on `self.repr.data()` and by documentation, not by the type system—callers must handle `Option` results and cannot statically know that an `Error` is definitely an OS error or definitely wraps an inner error.

**Evidence**:

```rust
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
```

**Entity:** std::io::Error

**States:** Os, Custom, Simple, SimpleMessage

**Transitions:**
- (construction) -> Os via last_os_error() / from_raw_os_error(code)
- (construction) -> Custom via other(E) / _new(kind, error)
- (construction) -> SimpleMessage via from_static_message(msg)

**Evidence:** method last_os_error(): constructs via Error::from_raw_os_error(sys::os::errno()); method from_raw_os_error(code): `Error { repr: Repr::new_os(code) }`; method other<E>(error): `Self::_new(ErrorKind::Other, error.into())`; method _new(kind, error): `Repr::new_custom(Box::new(Custom { kind, error }))`; method from_static_message(msg): `Repr::new_simple_message(msg)`; method raw_os_error(&self): `match self.repr.data() { ErrorData::Os(i) => Some(i), ErrorData::Custom(..) | ErrorData::Simple(..) | ErrorData::SimpleMessage(..) => None }`; doc on raw_os_error(): 'If this Error was constructed via last_os_error or from_raw_os_error ... Some, otherwise None'; doc on get_ref(): 'If this Error was constructed via new then ... Some, otherwise ... None' (implies construction-dependent availability of an inner error)

**Implementation:** Introduce typed wrappers for construction provenance, e.g. `struct OsError(Error)` (or `Error<Os>`), `struct CustomError(Error)` etc., with `OsError::raw_os_error() -> RawOsError` (non-Option) and `CustomError::get_ref() -> &(dyn Error + ...)` where applicable. Keep `Error` as the erased/sum type, but allow APIs to return the more specific wrapper when they can guarantee the variant (e.g., `last_os_error() -> OsError`).

---

### 45. Cursor position/content coupling (pos tracks consumption of split() remainder)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: These methods implicitly maintain an invariant that `self.pos` represents how many bytes have been consumed from the slice returned by `Cursor::split(self).1` (the remaining content). Successful reads advance `pos` by the amount read/written into the output buffer; some operations clamp `pos` to EOF on failure. This protocol is maintained manually by updating `self.pos` after delegating to `Read` impls on the split remainder, but the type system does not express that (a) reads must operate on the remainder produced by `split()` and (b) `pos` must be advanced/clamped consistently with the delegated operation’s effects.

**Evidence**:

```rust
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
```

**Entity:** Cursor<T>

**States:** PositionInBounds, PositionAtEOF

**Transitions:**
- PositionInBounds -> PositionInBounds via read()/read_buf()/read_vectored()/read_buf_exact()/read_to_end() (advance pos by bytes consumed)
- PositionInBounds -> PositionAtEOF via read_exact() on Err(_) (sets pos to inner.len())

**Evidence:** method read: `Read::read(&mut Cursor::split(self).1, buf)?; self.pos += n as u64;` (pos manually advanced to match delegated read); method read_buf: `prev_written = cursor.written(); ... Read::read_buf(...); self.pos += (cursor.written() - prev_written) as u64;` (pos advancement derived from BorrowedCursor side effect); method read_exact: `Ok(_) => self.pos += buf.len() as u64` and `Err(_) => self.pos = self.inner.as_ref().len() as u64` plus comment `// The only possible error condition is EOF, so place the cursor at "EOF"` (explicit EOF clamping protocol); method read_to_end: `let content = Cursor::split(self).1; let len = content.len(); ... self.pos += len as u64;` (pos becomes EOF after consuming remainder)

**Implementation:** Encode the coupling between "remainder" and position by returning a typed view representing the readable remainder (e.g., `struct Remainder<'a> { slice: &'a [u8], pos: &'a mut u64, len: u64 }`) and only allow read operations through that view, which updates `pos` internally. Alternatively use `Cursor<T, S>` with state markers like `InBounds`/`AtEof` and have `read_to_end(self) -> Cursor<T, AtEof>` / `read_exact` return a state transition on EOF, so callers/impls cannot forget to clamp/advance `pos`.

---

### 50. Take<T> read-budget protocol (Limited / Exhausted) with external mutation hazard

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-92`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Take<T> implicitly maintains a "remaining byte budget" in the `limit: u64` field. Reads are only meaningful while the budget is nonzero; once exhausted it should behave like EOF. `set_limit()` can reset the budget at any time (effectively re-entering the Limited state) regardless of how much has already been read. Additionally, the docs warn that exposing the underlying reader (via reference) allows external mutation of the underlying reader's I/O position/state, which can desynchronize/corrupt Take's internal notion of what should be read under the limit. None of these protocol constraints (budget exhaustion, resetting semantics, and 'don't mutate inner state while Take is active') are enforced by the type system.

**Evidence**:

```rust
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
```

**Entity:** Take<T>

**States:** Limited(limit > 0), Exhausted(limit == 0 / will return EOF)

**Transitions:**
- Limited -> Exhausted via reading until `limit` reaches 0 (implied by `limit` being 'bytes that can be read before EOF')
- Limited -> Limited via set_limit() (reset budget)
- Exhausted -> Limited via set_limit() (reset budget)

**Evidence:** field: `limit: u64` stores remaining bytes budget; doc on `limit()`: "Returns the number of bytes that can be read before this instance will return EOF." (defines Exhausted behavior when budget ends); doc on `set_limit(&mut self, limit: u64)`: "the amount of bytes read and the previous limit value don't matter" (reset semantics; permits Exhausted -> Limited); method: `into_inner(self) -> T` consumes Take, indicating a 'wrapper active' vs 'wrapper removed' phase boundary; doc snippet (get-ref section): "Care should be taken to avoid modifying the internal I/O state of the underlying reader as doing so may corrupt the internal limit of this" (implicit protocol forbidding certain uses of underlying reader while Take is in use)

**Implementation:** Encode budget state and inner-access capability at the type level: e.g., `Take<T, S>` where `S` is `Limited` or `Exhausted`. Provide `read` only for `Take<_, Limited>` and have it return either `Take<_, Limited>` or `Take<_, Exhausted>` depending on remaining budget (or expose a `try_read` that returns a state transition). For the mutation hazard, gate access to the underlying reader behind a capability token (or only allow `get_ref` but not `get_mut` while limited), or split APIs into `Take<T, Sealed>` vs `Take<T, Exposed>` where exposing `&mut T` transitions to a state where the limit accounting is considered invalid and must be re-established via an explicit `resync/set_limit` step.

---

## Precondition Invariants

### 52. BorrowedCursor capacity precondition for `read_buf` fast-path

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-68`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `read_buf` has a precondition-like branch on `buf.capacity() == 0` where it returns immediately. The method’s behavior depends on this property of `BorrowedCursor` at runtime; callers can supply a zero-capacity cursor and get a no-op. This is not represented at the type level (capacity is a runtime value), so the API relies on callers to pass a meaningful buffer when progress is expected.

**Evidence**:

```rust
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

```

**Entity:** Chain<T, U>

**States:** ZeroCapacity, NonZeroCapacity

**Transitions:**
- NonZeroCapacity -> (perform I/O) via `read_buf()`
- ZeroCapacity -> (no-op) via `read_buf()` early return

**Evidence:** `read_buf`: `if buf.capacity() == 0 { return Ok(()); }` early-return gate; `read_buf`: subsequent logic assumes capacity exists to observe progress via `buf.written()` comparisons

**Implementation:** Introduce a wrapper like `NonEmptyCursor<'a>(BorrowedCursor<'a>)` constructed only when `capacity() > 0`, and implement a `read_buf_nonempty(&mut self, buf: NonEmptyCursor<'_>)` internal method that omits the runtime check. (The public `read_buf` can keep the check and upgrade to `NonEmptyCursor` when possible.)

---

### 36. Spare-capacity precondition for unchecked buffer writes

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-72`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: BufWriter uses an unsafe fast path that writes into the internal buffer without bounds checks (`write_to_buffer_unchecked`). Correctness relies on a precondition that the input slice length is <= spare capacity (or otherwise within buffer size, depending on the earlier branch). This invariant is guarded by runtime conditionals and comments/SAFETY notes rather than a type-level guarantee; misuse would be UB if the unsafe write were reachable without the capacity check.

**Evidence**:

```rust
                    // sufficient room for any input <= the buffer size, which includes this input.
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    buf.len()
                }
            } else {
                return Ok(0);
            };
            debug_assert!(total_written != 0);
            for buf in iter {
                if buf.len() <= self.spare_capacity() {
                    // SAFETY: safe by above conditional.
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }

                    // This cannot overflow `usize`. If we are here, we've written all of the bytes
                    // so far to our buffer, and we've ensured that we never exceed the buffer's
                    // capacity. Therefore, `total_written` <= `self.buf.capacity()` <= `usize::MAX`.
                    total_written += buf.len();
                } else {
                    break;
                }
            }
            Ok(total_written)
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.get_mut().flush())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> fmt::Debug for BufWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufWriter")
            .field("writer", &&self.inner)
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
            .finish()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write + Seek> Seek for BufWriter<W> {
    /// Seek to the offset, in bytes, in the underlying writer.
    ///
    /// Seeking always writes out the internal buffer before seeking.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.flush_buf()?;
        self.get_mut().seek(pos)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<W: ?Sized + Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            // dtors should not panic, so we ignore a failed flush
            let _r = self.flush_buf();
        }
    }
}
```

**Entity:** BufWriter<W>

**States:** EnoughSpareCapacity, InsufficientSpareCapacity

**Transitions:**
- InsufficientSpareCapacity -> EnoughSpareCapacity via runtime conditional `if buf.len() <= self.spare_capacity()` (or earlier capacity logic)
- EnoughSpareCapacity -> EnoughSpareCapacity via repeated buffered writes while maintaining `total_written <= self.buf.capacity()`

**Evidence:** `if buf.len() <= self.spare_capacity() { unsafe { self.write_to_buffer_unchecked(buf); } }` shows the unchecked write is gated by a runtime capacity check; Comment preceding unsafe: "SAFETY: safe by above conditional." documents the precondition rather than encoding it; Earlier comment: "sufficient room for any input <= the buffer size" paired with `unsafe { self.write_to_buffer_unchecked(buf); }` indicates reliance on an implicit capacity invariant; Comment about `total_written` not overflowing: relies on invariant "we've ensured that we never exceed the buffer's capacity" and `total_written <= self.buf.capacity()`

**Implementation:** Introduce a helper API that produces a capability/newtype representing proven capacity, e.g. `struct FitsInSpare<'a>(&'a [u8]);` created only by `fn check_fits(&self, buf: &[u8]) -> Option<FitsInSpare<'_>>` (or returning a `BufWriteReservation`), and make `write_to_buffer_unchecked` accept that token instead of a raw `&[u8]`. This moves the proof obligation to construction and prevents calling the unsafe write without having established the precondition.

---

### 34. Unsafe append_to_string precondition (String must be empty / append-only guarantee)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-63`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The unsafe fast path for `read_to_string` relies on a precondition about the destination `String`: it must be empty so that writes performed by the reader can only be appends and all bytes written are validated for UTF-8. If the string were non-empty, an untrusted `Read` implementation could mutate existing bytes, violating UTF-8 invariants because `append_to_string` only validates newly appended data. This is enforced by a runtime check (`if buf.is_empty()`) and an `unsafe` call, not by the type system.

**Evidence**:

```rust
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
```

**Entity:** BufReader<R>::read_to_string (interaction with `String` and `crate::io::append_to_string`)

**States:** StringEmpty, StringNonEmpty

**Transitions:**
- StringEmpty -> StringNonEmpty via `unsafe { crate::io::append_to_string(buf, |b| self.read_to_end(b)) }` (writes append bytes)

**Evidence:** read_to_string: `if buf.is_empty() { ... unsafe { crate::io::append_to_string(buf, |b| self.read_to_end(b)) } }`; comment: "`append_to_string`'s safety relies on the buffer only being appended to" and "If there were existing content in `buf` ... could ... modify existing bytes and render them invalid"; comment: "if `buf` is empty then ... `append_to_string` will validate all of the new bytes"

**Implementation:** Introduce a `struct EmptyString(String);` (or `struct AppendOnlyString<'a>(&'a mut String)` constructed only from an empty string) and change the unsafe helper to accept that wrapper: `append_to_string(EmptyString, ...)`. The wrapper's constructor performs the emptiness check once, making the unsafe call site require proof of the precondition at the type level.

---

### 4. Buffered write capacity precondition (Enough spare capacity before unchecked write)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufwriter.rs:1-71`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: `write_to_buffer_unchecked` relies on a strict precondition about spare capacity (`buf.len() <= self.buf.capacity() - self.buf.len()`). The caller code ensures this via conditional branching (either bypass to inner writer when `buf.len() >= capacity`, or ensure spare capacity by flushing before entering the block) and then calls the unsafe function. This is a latent invariant because correctness depends on maintaining the calling protocol; the type system does not enforce that `write_to_buffer_unchecked` is only ever called when capacity is sufficient, beyond a `debug_assert!`.

**Evidence**:

```rust
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

#[stable(feature = "bufwriter_into_parts", since = "1.56.0")]
/// Error returned for the buffered data from `BufWriter::into_parts`, when the underlying
/// writer has previously panicked.  Contains the (possibly partly written) buffered data.
///
/// # Example
///
/// ```
/// use std::io::{self, BufWriter, Write};
/// use std::panic::{catch_unwind, AssertUnwindSafe};
///
/// struct PanickingWriter;
/// impl Write for PanickingWriter {
///   fn write(&mut self, buf: &[u8]) -> io::Result<usize> { panic!() }
///   fn flush(&mut self) -> io::Result<()> { panic!() }
/// }
///
/// let mut stream = BufWriter::new(PanickingWriter);
/// write!(stream, "some data").unwrap();
/// let result = catch_unwind(AssertUnwindSafe(|| {
///     stream.flush().unwrap()
/// }));
/// assert!(result.is_err());
/// let (recovered_writer, buffered_data) = stream.into_parts();
```

**Entity:** BufWriter<W>

**States:** CapacitySufficient, CapacityInsufficient

**Transitions:**
- CapacityInsufficient -> CapacitySufficient via "flushed the buffer to ensure that there is" (as stated in the SAFETY comment), before calling `write_to_buffer_unchecked`
- CapacitySufficient -> CapacitySufficient via `write_to_buffer_unchecked` appending and updating `self.buf.set_len(old_len + buf_len)`

**Evidence:** SAFETY comment on `write_to_buffer_unchecked`: "Requires `buf.len() <= self.buf.capacity() - self.buf.len()`"; `debug_assert!(buf.len() <= self.spare_capacity());` in `write_to_buffer_unchecked`; `unsafe { self.write_to_buffer_unchecked(buf); }` call guarded by earlier size logic (`if buf.len() >= self.buf.capacity()` else branch); SAFETY comment in caller: "There was either enough spare capacity already, or ... we flushed the buffer to ensure that there is."; Implementation uses raw pointer copy + `self.buf.set_len(...)`, which is only sound if the capacity precondition holds

**Implementation:** Introduce a proof-carrying helper that encodes the precondition, e.g. `struct FitsInSpare<'a> { slice: &'a [u8] }` created only by a checked method like `fn fits_in_spare(&self, buf: &[u8]) -> Option<FitsInSpare<'_>>`. Then make `write_to_buffer_unchecked(&mut self, buf: FitsInSpare<'_>)` safe (no `unsafe` at the callsite) and keep the `unsafe` localized inside the constructor/checked method.

---

### 20. StdoutLock lifetime & global-static assumption (scoped lock vs effectively 'static' guard)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-193`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Stdout::lock() returns `StdoutLock<'static>` based on an implementation detail: the underlying lock is stored in a static. This encodes a latent invariant that the referenced lock outlives all uses of the guard; the API exposes a 'static guard even though the lock is conceptually tied to a particular Stdout handle. The type system does not express the dependency of the guard's lifetime on `&self`, so it cannot prevent patterns like storing the guard long-term (or leaking it), which the module itself later treats as a shutdown hazard (cleanup uses try_lock specifically because leaked guards can deadlock).

**Evidence**:

```rust
/// ```
///
/// Ensuring output is flushed immediately:
///
/// ```no_run
/// use std::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     let mut stdout = io::stdout();
///     stdout.write_all(b"hello, ")?;
///     stdout.flush()?;                // Manual flush
///     stdout.write_all(b"world!\n")?; // Automatically flushed
///     Ok(())
/// }
/// ```
///
/// [`flush`]: Write::flush
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "io_stdout")]
pub fn stdout() -> Stdout {
    Stdout {
        inner: STDOUT
            .get_or_init(|| ReentrantLock::new(RefCell::new(LineWriter::new(stdout_raw())))),
    }
}

// Flush the data and disable buffering during shutdown
// by replacing the line writer by one with zero
// buffering capacity.
pub fn cleanup() {
    let mut initialized = false;
    let stdout = STDOUT.get_or_init(|| {
        initialized = true;
        ReentrantLock::new(RefCell::new(LineWriter::with_capacity(0, stdout_raw())))
    });

    if !initialized {
        // The buffer was previously initialized, overwrite it here.
        // We use try_lock() instead of lock(), because someone
        // might have leaked a StdoutLock, which would
        // otherwise cause a deadlock here.
        if let Some(lock) = stdout.try_lock() {
            *lock.borrow_mut() = LineWriter::with_capacity(0, stdout_raw());
        }
    }
}

impl Stdout {
    /// Locks this handle to the standard output stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the `Write` trait for writing data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, Write};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stdout = io::stdout().lock();
    ///
    ///     stdout.write_all(b"hello world")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdoutLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `ReentrantMutex` is
        // static.
        StdoutLock { inner: self.inner.lock() }
    }
}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl UnwindSafe for Stdout {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for Stdout {}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stdout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdout").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Stdout {
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
impl Write for &Stdout {
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
impl UnwindSafe for StdoutLock<'_> {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for StdoutLock<'_> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for StdoutLock<'_> {
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for StdoutLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdoutLock").finish_non_exhaustive()
    }
}

/// A handle to the standard error stream of a process.
///
/// For more information, see the [`io::stderr`] method.
///
/// [`io::stderr`]: stderr
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
```

**Entity:** Stdout::lock() / StdoutLock<'static>

**States:** Unlocked, Locked(guard exists)

**Transitions:**
- Unlocked -> Locked via Stdout::lock()
- Locked -> Unlocked via Drop of StdoutLock

**Evidence:** Stdout::lock signature: `pub fn lock(&self) -> StdoutLock<'static>` returns a 'static guard rather than tying it to `&self`; Stdout::lock comment: "Locks this handle with 'static lifetime. This depends on the implementation detail that the underlying `ReentrantMutex` is static."; cleanup() comment + code: mentions leaked `StdoutLock` and uses `try_lock()` to avoid deadlock, implying leaking the guard violates an intended usage protocol

**Implementation:** Make the lock guard lifetime depend on `&self` (e.g., `fn lock(&self) -> StdoutLock<'_>`), and/or require a non-cloneable capability token to obtain a long-lived 'static lock if truly needed internally. This would prevent accidental long-term storage/leaks in user code and align cleanup's expectations with compile-time borrowing constraints.

---

### 61. Read::read contract: returned byte count must not exceed provided buffer length

**Location**: `/tmp/io_test_crate/src/io/tests.rs:1-60`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The Read::read implementation must obey an implicit contract: it may only report having read up to buf.len() bytes. BufReader::fill_buf assumes this and will panic (or otherwise misbehave) if the contract is violated. The type system does not encode this postcondition on the returned usize, so violations are only detected at runtime (here, by a #[should_panic] test).

**Evidence**:

```rust

// Issue #120603
#[test]
#[should_panic]
fn read_buf_broken_read() {
    struct MalformedRead;

    impl Read for MalformedRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            // broken length calculation
            Ok(buf.len() + 1)
        }
    }

    let _ = BufReader::new(MalformedRead).fill_buf();
}

#[test]
fn read_buf_full_read() {
    struct FullRead;

    impl Read for FullRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
    }

    assert_eq!(BufReader::new(FullRead).fill_buf().unwrap().len(), DEFAULT_BUF_SIZE);
}

struct DataAndErrorReader(&'static [u8]);

impl Read for DataAndErrorReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        panic!("We want tests to use `read_buf`")
    }

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf).unwrap();
        Err(io::Error::other("error"))
    }
}

#[test]
fn read_buf_data_and_error_take() {
    let mut buf = [0; 64];
    let mut buf = io::BorrowedBuf::from(buf.as_mut_slice());

    let mut r = DataAndErrorReader(&[4, 5, 6]).take(1);
    assert!(r.read_buf(buf.unfilled()).is_err());
    assert_eq!(buf.filled(), &[4]);

    assert!(r.read_buf(buf.unfilled()).is_ok());
    assert_eq!(buf.filled(), &[4]);
    assert_eq!(r.get_ref().0, &[5, 6]);
}

#[test]
fn read_buf_data_and_error_buf() {
    let mut r = BufReader::new(DataAndErrorReader(&[4, 5, 6]));

```

**Entity:** MalformedRead / FullRead (Read impls used by BufReader::fill_buf)

**States:** ContractSatisfied, ContractViolated

**Transitions:**
- ContractSatisfied -> ContractViolated via Read::read returning buf.len() + 1

**Evidence:** fn read_buf_broken_read: comment `// broken length calculation` and `Ok(buf.len() + 1)` in `impl Read for MalformedRead`; `#[should_panic]` on `read_buf_broken_read` indicates runtime failure when used via `BufReader::new(MalformedRead).fill_buf()`; fn read_buf_full_read: `Ok(buf.len())` and `BufReader::new(FullRead).fill_buf().unwrap().len() == DEFAULT_BUF_SIZE` demonstrates the valid case

**Implementation:** Introduce a wrapper API for reads that returns a bounded length, e.g., `struct ReadCount(usize);` where construction requires `<= buf.len()` (enforced by a helper taking `buf_len`), and have internal consumers use that wrapper. Alternatively, expose a safe helper `fn read_at_most(&mut self, buf: &mut [u8]) -> io::Result<BoundedUsize<BUF_LEN>>` (const-generic or runtime-checked constructor) and keep unchecked `Read::read` behind an adapter.

---

### 53. TLS output-capture lifecycle (Accessible -> Destroying/Destroyed)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-120`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `set_output_capture` relies on an implicit lifecycle state of the thread-local storage (TLS) used for output capturing. The operation is only valid while the TLS key is still accessible; during or after TLS destruction (e.g., thread shutdown / TLS dtors running), attempting to access it is invalid and results in a panic. This precondition is enforced via a runtime `expect(...)` rather than the type system, so callers can invoke it in an invalid phase and crash.

**Evidence**:

```rust
        // static.
        StderrLock { inner: self.inner.lock() }
    }
}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl UnwindSafe for Stderr {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for Stderr {}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stderr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stderr").finish_non_exhaustive()
    }
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for StderrLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StderrLock").finish_non_exhaustive()
    }
}

/// Sets the thread-local output capture buffer and returns the old one.
#[unstable(
    feature = "internal_output_capture",
    reason = "this function is meant for use in the test crate \
        and may disappear in the future",
    issue = "none"
)]
#[doc(hidden)]
pub fn set_output_capture(sink: Option<LocalStream>) -> Option<LocalStream> {
    try_set_output_capture(sink).expect(
        "cannot access a Thread Local Storage value \
         during or after destruction",
    )
}

```

**Entity:** Thread-local output capture (set_output_capture / try_set_output_capture)

**States:** Accessible, DestroyingOrDestroyed

**Transitions:**
- Accessible -> DestroyingOrDestroyed via thread/TLS teardown (implicit, not represented in types)

**Evidence:** function `set_output_capture(sink: Option<LocalStream>) -> Option<LocalStream>` calls `try_set_output_capture(sink).expect(...)`; panic message in `expect`: "cannot access a Thread Local Storage value during or after destruction" encodes the required precondition/state

**Implementation:** Gate `set_output_capture` behind a non-`'static` capability/token that can only be obtained while TLS is known-live (e.g., `fn with_output_capture_tls<R>(f: impl FnOnce(OutputCaptureTls) -> R) -> R`). Expose `set_output_capture` as a method on that token (`impl OutputCaptureTls { fn set(self, sink: Option<LocalStream>) -> Option<LocalStream> }`), preventing calls from TLS destructors or after teardown without the token.

---

### 57. Terminal-dependent behavior protocol (TTY vs non-TTY)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-71`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: `is_terminal()` exposes a runtime property (whether the underlying handle is connected to a terminal). Callers are expected to branch on this to decide whether prompting/interactive behaviors are appropriate. The type system does not distinguish 'terminal-backed' vs 'pipe/file-backed' handles, so interactive-only operations/protocols (like prompting) cannot be made available only when in the Terminal state.

**Evidence**:

```rust
    ///
    ///     // Indicate that the user is prompted for input, if this is a terminal.
    ///     if stdin.is_terminal() {
    ///         print!("> ");
    ///         io::stdout().flush()?;
    ///     }
    ///
    ///     let mut name = String::new();
    ///     let _ = stdin.read_line(&mut name)?;
    ///
    ///     println!("Hello {}", name.trim_end());
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The example can be run in two ways:
    ///
    /// - If you run this example by piping some text to it, e.g. `echo "foo" | path/to/executable`
    ///   it will print: `Hello foo`.
    /// - If you instead run the example interactively by running `path/to/executable` directly, it will
    ///   prompt for input.
    ///
    /// [changes]: io#platform-specific-behavior
    /// [`Stdin`]: crate::io::Stdin
    #[doc(alias = "isatty")]
    #[stable(feature = "is_terminal", since = "1.70.0")]
    fn is_terminal(&self) -> bool;
}

macro_rules! impl_is_terminal {
    ($($t:ty),*$(,)?) => {$(
        #[unstable(feature = "sealed", issue = "none")]
        impl crate::sealed::Sealed for $t {}

        #[stable(feature = "is_terminal", since = "1.70.0")]
        impl IsTerminal for $t {
            #[inline]
            fn is_terminal(&self) -> bool {
                crate::sys::io::is_terminal(self)
            }
        }
    )*}
}

impl_is_terminal!(File, Stdin, StdinLock<'_>, Stdout, StdoutLock<'_>, Stderr, StderrLock<'_>);

#[unstable(
    feature = "print_internals",
    reason = "implementation detail which may disappear or be replaced at any time",
    issue = "none"
)]
#[doc(hidden)]
#[cfg(not(test))]
pub fn _print(args: fmt::Arguments<'_>) {
    print_to(args, stdout, "stdout");
}

#[unstable(
    feature = "print_internals",
    reason = "implementation detail which may disappear or be replaced at any time",
    issue = "none"
)]
#[doc(hidden)]
#[cfg(not(test))]
pub fn _eprint(args: fmt::Arguments<'_>) {
    print_to(args, stderr, "stderr");
}

#[cfg(test)]
pub use realstd::io::{_eprint, _print};
```

**Entity:** IsTerminal (impls for File/Stdin/Stdout/Stderr and their Lock types)

**States:** Terminal, NotTerminal

**Transitions:**
- NotTerminal <-> Terminal via external environment changes affecting `crate::sys::io::is_terminal(self)`

**Evidence:** doc comment: "Indicate that the user is prompted for input, if this is a terminal." followed by `if stdin.is_terminal() { print!("> "); io::stdout().flush()?; }`; trait method: `fn is_terminal(&self) -> bool;` returns a boolean runtime state; impl: `fn is_terminal(&self) -> bool { crate::sys::io::is_terminal(self) }` delegates to an OS query

**Implementation:** Introduce `Terminal<T>(T)` as a wrapper obtainable via `T::try_into_terminal()`/`T::as_terminal()` that performs the runtime check once. Put interactive-only helpers (e.g., `prompt()`/`interactive_flush_prompt()`) on `Terminal<Stdin>`/`Terminal<Stdout>` so code that assumes a TTY cannot compile without an explicit check/conversion.

---

### 38. Unix device-file precondition (/dev/zero readable, /dev/null writable)

**Location**: `/tmp/io_test_crate/src/io/copy/tests.rs:1-57`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: The benchmark assumes a Unix-like environment where `/dev/zero` can be opened for reading and `/dev/null` can be opened for writing. This environmental requirement is enforced only by runtime `expect(...)` calls and a conditional ignore for Emscripten, not by the type system.

**Evidence**:

```rust
    assert!(
        source.observed_buffer > DEFAULT_BUF_SIZE,
        "expected a large buffer to be provided to the reader, got {}",
        source.observed_buffer
    );
}

#[test]
fn copy_specializes_from_vecdeque() {
    let mut source = VecDeque::with_capacity(100 * 1024);
    for _ in 0..20 * 1024 {
        source.push_front(0);
    }
    for _ in 0..20 * 1024 {
        source.push_back(0);
    }
    let mut sink = WriteObserver { observed_buffer: 0 };
    assert_eq!(40 * 1024u64, io::copy(&mut source, &mut sink).unwrap());
    assert_eq!(20 * 1024, sink.observed_buffer);
}

#[test]
fn copy_specializes_from_slice() {
    let mut source = [1; 60 * 1024].as_slice();
    let mut sink = WriteObserver { observed_buffer: 0 };
    assert_eq!(60 * 1024u64, io::copy(&mut source, &mut sink).unwrap());
    assert_eq!(60 * 1024, sink.observed_buffer);
}

#[cfg(unix)]
mod io_benches {
    use test::Bencher;

    use crate::fs::{File, OpenOptions};
    use crate::io::BufReader;
    use crate::io::prelude::*;

    #[bench]
    #[cfg_attr(target_os = "emscripten", ignore)] // no /dev
    fn bench_copy_buf_reader(b: &mut Bencher) {
        let mut file_in = File::open("/dev/zero").expect("opening /dev/zero failed");
        // use dyn to avoid specializations unrelated to readbuf
        let dyn_in = &mut file_in as &mut dyn Read;
        let mut reader = BufReader::with_capacity(256 * 1024, dyn_in.take(0));
        let mut writer =
            OpenOptions::new().write(true).open("/dev/null").expect("opening /dev/null failed");

        const BYTES: u64 = 1024 * 1024;

        b.bytes = BYTES;

        b.iter(|| {
            reader.get_mut().set_limit(BYTES);
            crate::io::copy(&mut reader, &mut writer).unwrap()
        });
    }
}
```

**Entity:** File (/dev/zero) and File (/dev/null) in bench_copy_buf_reader

**States:** Valid device files present and accessible, Missing/inaccessible device files

**Transitions:**
- Missing/inaccessible -> Valid via running on a suitable Unix environment with permissions

**Evidence:** bench_copy_buf_reader: `File::open("/dev/zero").expect("opening /dev/zero failed")`; bench_copy_buf_reader: `OpenOptions::new().write(true).open("/dev/null").expect("opening /dev/null failed")`; bench_copy_buf_reader: `#[cfg_attr(target_os = "emscripten", ignore)] // no /dev` comment documents the environmental precondition

**Implementation:** Factor out a `DevZero`/`DevNull` capability type constructed only when `/dev/*` is available (behind `cfg(unix)` plus a fallible constructor returning a dedicated capability token). Bench functions would take these capability types instead of raw paths, making the dependency explicit and reducing ad-hoc runtime assumptions in the benchmark body.

---

### 10. Test precondition on buffer sizing (buf_sz > DEFAULT_BUF_SIZE)

**Location**: `/tmp/io_test_crate/src/io/copy/tests.rs:1-62`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The test encodes an invariant that a chosen buffer size must be greater than `DEFAULT_BUF_SIZE` for the intended specialization/behavior to be meaningfully exercised. This is enforced only by a runtime `assert!` labeled as a precondition; there is no type-level guarantee that a 'large enough buffer size' is provided to the code under test.

**Evidence**:

```rust

    let mut r = repeat(0).take(1 << 17);
    assert_eq!(copy(&mut r as &mut dyn Read, &mut w as &mut dyn Write).unwrap(), 1 << 17);
}

struct ShortReader {
    cap: usize,
    read_size: usize,
    observed_buffer: usize,
}

impl Read for ShortReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = min(self.cap, self.read_size).min(buf.len());
        self.cap -= bytes;
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(bytes)
    }
}

struct WriteObserver {
    observed_buffer: usize,
}

impl Write for WriteObserver {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn copy_specializes_bufwriter() {
    let cap = 117 * 1024;
    let buf_sz = 16 * 1024;
    let mut r = ShortReader { cap, observed_buffer: 0, read_size: 1337 };
    let mut w = BufWriter::with_capacity(buf_sz, WriteObserver { observed_buffer: 0 });
    assert_eq!(
        copy(&mut r, &mut w).unwrap(),
        cap as u64,
        "expected the whole capacity to be copied"
    );
    assert_eq!(r.observed_buffer, buf_sz, "expected a large buffer to be provided to the reader");
    assert!(w.get_mut().observed_buffer > DEFAULT_BUF_SIZE, "expected coalesced writes");
}

#[test]
fn copy_specializes_bufreader() {
    let mut source = vec![0; 768 * 1024];
    source[1] = 42;
    let mut buffered = BufReader::with_capacity(256 * 1024, Cursor::new(&mut source));

    let mut sink = Vec::new();
    assert_eq!(crate::io::copy(&mut buffered, &mut sink).unwrap(), source.len() as u64);
    assert_eq!(source.as_slice(), sink.as_slice());

    let buf_sz = 71 * 1024;
    assert!(buf_sz > DEFAULT_BUF_SIZE, "test precondition");

```

**Entity:** copy_specializes_bufreader (test case)

**States:** PreconditionNotMet(buf_sz <= DEFAULT_BUF_SIZE), PreconditionMet(buf_sz > DEFAULT_BUF_SIZE)

**Transitions:**
- PreconditionNotMet -> PreconditionMet by choosing a different constant/value for buf_sz

**Evidence:** in `copy_specializes_bufreader`: `let buf_sz = 71 * 1024; assert!(buf_sz > DEFAULT_BUF_SIZE, "test precondition");`

**Implementation:** Introduce a `struct LargerThanDefault(usize);` constructor `fn new(n: usize) -> Option<Self>` (or `Result`) that checks `n > DEFAULT_BUF_SIZE`, and use it in tests/APIs that require the property. Where possible, encode sizes as const generics (`struct Buf<const N: usize>;`) and enforce `N > DEFAULT_BUF_SIZE` via a compile-time assertion helper.

---

## Protocol Invariants

### 16. BufReader buffering exclusivity & post-into_inner data-loss protocol

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-31`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The docs define an implicit usage protocol around buffering: a BufReader maintains an internal buffer that may contain bytes already read from the underlying reader but not yet observed by the caller. This creates a latent exclusivity/aliasing invariant: you must not create multiple BufReader instances over the same underlying stream, and you must not read directly from the underlying reader after unwrapping via into_inner unless you have accounted for/discarded the buffered bytes. Violating this can cause logical data loss (skipped or duplicated bytes) even though the type system allows it (because the underlying reader can be shared/aliased outside the BufReader, and into_inner hands the reader back without coupling it to the buffered remainder).

**Evidence**:

```rust
mod buffer;

use buffer::Buffer;

use crate::fmt;
use crate::io::{
    self, BorrowedCursor, BufRead, DEFAULT_BUF_SIZE, IoSliceMut, Read, Seek, SeekFrom, SizeHint,
    SpecReadByte, uninlined_slow_read_byte,
};

/// The `BufReader<R>` struct adds buffering to any reader.
///
/// It can be excessively inefficient to work directly with a [`Read`] instance.
/// For example, every call to [`read`][`TcpStream::read`] on [`TcpStream`]
/// results in a system call. A `BufReader<R>` performs large, infrequent reads on
/// the underlying [`Read`] and maintains an in-memory buffer of the results.
///
/// `BufReader<R>` can improve the speed of programs that make *small* and
/// *repeated* read calls to the same file or network socket. It does not
/// help when reading very large amounts at once, or reading just one or a few
/// times. It also provides no advantage when reading from a source that is
/// already in memory, like a <code>[Vec]\<u8></code>.
///
/// When the `BufReader<R>` is dropped, the contents of its buffer will be
/// discarded. Creating multiple instances of a `BufReader<R>` on the same
/// stream can cause data loss. Reading from the underlying reader after
/// unwrapping the `BufReader<R>` with [`BufReader::into_inner`] can also cause
/// data loss.
///
/// [`TcpStream::read`]: crate::net::TcpStream::read
/// [`TcpStream`]: crate::net::TcpStream
```

**Entity:** BufReader<R>

**States:** Sole buffered owner of underlying reader, Aliased/multi-owner buffered access (data-loss risk), Unwrapped (into_inner) with unread buffered bytes

**Transitions:**
- Sole buffered owner of underlying reader -> Aliased/multi-owner buffered access (data-loss risk) via creating multiple BufReader instances on the same stream (external action)
- Sole buffered owner of underlying reader -> Unwrapped (into_inner) with unread buffered bytes via BufReader::into_inner()
- Sole buffered owner of underlying reader -> Unwrapped (into_inner) with unread buffered bytes via Drop (buffer discarded)

**Evidence:** doc comment: "When the `BufReader<R>` is dropped, the contents of its buffer will be discarded."; doc comment: "Creating multiple instances of a `BufReader<R>` on the same stream can cause data loss."; doc comment: "Reading from the underlying reader after unwrapping the `BufReader<R>` with [`BufReader::into_inner`] can also cause data loss."; use buffer::Buffer; (implies internal buffering state exists and can be out-of-sync with external reads)

**Implementation:** Introduce an unforgeable capability/token representing exclusive buffered ownership of the underlying reader (e.g., wrap R in a `Buffered<R>` newtype only constructible by BufReader, or require `BufReader::new` to take `OwnedRead<R>` that cannot be aliased). Alternatively provide `into_parts(self) -> (R, Buffer)` so callers must explicitly handle remaining buffered bytes when unwrapping, making the protocol explicit in types.

---

### 59. Write capability protocol (vectored-write + fmt-write expectations)

**Location**: `/tmp/io_test_crate/src/io/util/tests.rs:1-74`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The test assumes a specific capability profile of the passed-in Write implementation: it must support vectored writes (is_write_vectored() must be true) and must accept write_fmt() without evaluating the Display implementation (fmt arguments are expected to be ignored). These are runtime behavioral/capability requirements; the type system only knows W: Write, not whether it is vectored-capable nor whether its write_fmt path avoids calling Display::fmt. In this test, calling write_vectored()/write_all_vectored() and write_fmt() is only considered valid if those behavioral properties hold.

**Evidence**:

```rust
use crate::fmt;
use crate::io::prelude::*;
use crate::io::{
    BorrowedBuf, Empty, ErrorKind, IoSlice, IoSliceMut, Repeat, SeekFrom, Sink, empty, repeat, sink,
};
use crate::mem::MaybeUninit;

struct ErrorDisplay;

impl fmt::Display for ErrorDisplay {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}

struct PanicDisplay;

impl fmt::Display for PanicDisplay {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        panic!()
    }
}

#[track_caller]
fn test_sinking<W: Write>(mut w: W) {
    assert_eq!(w.write(&[]).unwrap(), 0);
    assert_eq!(w.write(&[0]).unwrap(), 1);
    assert_eq!(w.write(&[0; 1024]).unwrap(), 1024);
    w.write_all(&[]).unwrap();
    w.write_all(&[0]).unwrap();
    w.write_all(&[0; 1024]).unwrap();
    let mut bufs =
        [IoSlice::new(&[]), IoSlice::new(&[0]), IoSlice::new(&[0; 1024]), IoSlice::new(&[])];
    assert!(w.is_write_vectored());
    assert_eq!(w.write_vectored(&[]).unwrap(), 0);
    assert_eq!(w.write_vectored(&bufs).unwrap(), 1025);
    w.write_all_vectored(&mut []).unwrap();
    w.write_all_vectored(&mut bufs).unwrap();
    assert!(w.flush().is_ok());
    assert_eq!(w.by_ref().write(&[0; 1024]).unwrap(), 1024);
    // Ignores fmt arguments
    w.write_fmt(format_args!("{}", ErrorDisplay)).unwrap();
    w.write_fmt(format_args!("{}", PanicDisplay)).unwrap();
}

#[test]
fn sink_sinks() {
    test_sinking(sink());
}

#[test]
fn empty_reads() {
    let mut e = empty();
    assert_eq!(e.read(&mut []).unwrap(), 0);
    assert_eq!(e.read(&mut [0]).unwrap(), 0);
    assert_eq!(e.read(&mut [0; 1024]).unwrap(), 0);
    assert_eq!(Read::by_ref(&mut e).read(&mut [0; 1024]).unwrap(), 0);

    e.read_exact(&mut []).unwrap();
    assert_eq!(e.read_exact(&mut [0]).unwrap_err().kind(), ErrorKind::UnexpectedEof);
    assert_eq!(e.read_exact(&mut [0; 1024]).unwrap_err().kind(), ErrorKind::UnexpectedEof);

    assert!(!e.is_read_vectored());
    assert_eq!(e.read_vectored(&mut []).unwrap(), 0);
    let (mut buf1, mut buf1024) = ([0], [0; 1024]);
    let bufs = &mut [
        IoSliceMut::new(&mut []),
        IoSliceMut::new(&mut buf1),
        IoSliceMut::new(&mut buf1024),
        IoSliceMut::new(&mut []),
    ];
    assert_eq!(e.read_vectored(bufs).unwrap(), 0);

    let buf: &mut [MaybeUninit<_>] = &mut [];
```

**Entity:** W: Write (as exercised by test_sinking)

**States:** NonVectoredWrite, VectoredWrite

**Transitions:**
- NonVectoredWrite -> VectoredWrite asserted by is_write_vectored() == true (required before write_vectored/write_all_vectored in this test)

**Evidence:** fn test_sinking<W: Write>(mut w: W): generic only over Write, but test requires extra properties; assert!(w.is_write_vectored()); then w.write_vectored(&bufs).unwrap() and w.write_all_vectored(&mut bufs).unwrap() — runtime-gated capability; comment: "// Ignores fmt arguments" followed by w.write_fmt(format_args!("{}", ErrorDisplay)).unwrap() where ErrorDisplay::fmt returns Err(fmt::Error); w.write_fmt(format_args!("{}", PanicDisplay)).unwrap() where PanicDisplay::fmt panics — test relies on fmt not being invoked

**Implementation:** Introduce a separate capability trait (e.g., trait VectoredWrite: Write { ... } or a marker bound) and accept W: VectoredWrite in test_sinking when vectored behavior is required. For the fmt behavior, use a newtype/capability wrapper around Sink that provides a specialized write_fmt implementation guaranteeing it does not call Display::fmt (or expose a method like write_fmt_ignoring_args). Then the test can be typed against that wrapper/capability instead of plain Write.

---

### 2. Seek-relative fast-path validity (offset must stay within current buffer window)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader.rs:1-73`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: seek_relative uses a two-path protocol: if the requested offset stays within the already-buffered region, it updates internal buffer cursors (consume/unconsume) without seeking the underlying reader; otherwise it performs an actual seek on the underlying reader. This creates an implicit dependency on the current buffer position/filled window: correctness relies on the runtime checks against buf.pos() and buf.filled(), and on using consume/unconsume with the exact offset magnitude. The type system cannot express 'this offset is guaranteed to be within the current buffer window', so the fast-path is guarded by runtime arithmetic and bounds checks.

**Evidence**:

```rust
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
    Self: Read,
{
```

**Entity:** BufReader<R> (where R: Seek)

**States:** Offset within current buffer window (can adjust buffer pointers only), Offset outside buffer window (must perform underlying seek)

**Transitions:**
- Offset within buffer window -> (buffer cursor moved) via buf.consume()/buf.unconsume() inside seek_relative()
- Offset outside buffer window -> (underlying position changed) via self.seek(SeekFrom::Current(offset)) inside seek_relative()

**Evidence:** seek_relative(&mut self, offset: i64): computes pos = self.buf.pos() and compares against self.buf.filled() to decide path; seek_relative: negative case uses pos.checked_sub and then self.buf.unconsume((-offset) as usize); seek_relative: positive case uses pos.checked_add and checks new_pos <= self.buf.filled() then self.buf.consume(offset as usize); fallback path: self.seek(SeekFrom::Current(offset)).map(drop) when outside-buffer/overflow

**Implementation:** Introduce a capability/newtype representing an offset proven to be within the current buffer window, e.g., struct InBufferOffset(i64); provide a method try_in_buffer_offset(&self, offset: i64) -> Option<InBufferOffset> that performs the checks once; then a separate method seek_relative_in_buffer(&mut self, InBufferOffset) that cannot fall back to underlying seek.

---

### 18. Stdin locking protocol (Unlocked shared handle vs Locked exclusive BufRead access)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-100`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: The documentation describes an implicit protocol: Stdin is a shared handle to a global buffer; normal reads are "otherwise locked with respect to other reads", but to access full BufRead functionality (e.g., lines()), the handle must be lock()'d. It also warns that concurrent reads "must be executed with care", implying that correct usage depends on taking and holding the lock for multi-step read interactions. This protocol is only documented and enforced dynamically (by internal locking), not made explicit in the API surface for sequencing multi-call operations.

**Evidence**:

```rust
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
```

**Entity:** Stdin (doc-commented handle semantics)

**States:** Unlocked (shared, internally synchronized for reads), Locked (exclusive access via lock(), full BufRead methods)

**Transitions:**
- Unlocked -> Locked via lock()
- Locked -> Unlocked when the lock guard is dropped

**Evidence:** doc comment: "A handle can be `lock`'d to gain full access to [`BufRead`] methods (e.g., `.lines()`)."; doc comment: "Reads to this handle are otherwise locked with respect to other reads."; doc comment: "beware that concurrent reads of `Stdin` must be executed with care."

**Implementation:** Make the locked capability more explicit by ensuring all multi-step buffered operations require the lock guard type (e.g., StdinLock) and by steering users toward APIs that consume a lock guard for sequences (iterator constructors, read-line loops). If there are any non-locked methods that effectively assume exclusive access for correctness, move them onto the lock guard type so misuse is rejected at compile time.

---

### 46. BorrowedCursor written-count delta protocol (must snapshot written() before read_buf*)

**Location**: `/tmp/io_test_crate/src/io/cursor.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The implementation relies on a protocol that the number of bytes consumed from the underlying Cursor can be computed as `cursor.written() - prev_written`, where `prev_written` is captured before calling `Read::read_buf*` and the same `BorrowedCursor` is then inspected after the call. This implicitly requires the caller to snapshot written-count first, and requires that `read_buf*` only increases `written()`. None of this is expressed in the types; correctness depends on following this temporal ordering and on `written()` behaving monotonically.

**Evidence**:

```rust
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
```

**Entity:** BorrowedCursor<'_> (as used by Cursor<T>::read_buf/read_buf_exact)

**States:** BeforeReadBuf, AfterReadBuf

**Transitions:**
- BeforeReadBuf -> AfterReadBuf via Read::read_buf(..., cursor.reborrow())
- BeforeReadBuf -> AfterReadBuf via Read::read_buf_exact(..., cursor.reborrow())

**Evidence:** method read_buf: `let prev_written = cursor.written(); ... Read::read_buf(..., cursor.reborrow())?; self.pos += (cursor.written() - prev_written) as u64;` (explicit snapshot-then-delta protocol); method read_buf_exact: `let prev_written = cursor.written(); ... Read::read_buf_exact(..., cursor.reborrow()); self.pos += (cursor.written() - prev_written) as u64;` (same protocol repeated)

**Implementation:** Introduce a helper newtype that captures the initial `written()` at construction (e.g., `struct WrittenDelta<'a> { cur: BorrowedCursor<'a>, start: usize }`) and provides `fn finish(self) -> (BorrowedCursor<'a>, usize_delta)` or directly `fn delta_written(&self) -> usize`. This makes it impossible to forget the snapshot or to mix cursors when computing the delta, and centralizes the monotonicity assumption.

---

### 26. LineWriter write/flush buffering protocol (BufferedPartial / FlushedToNewline / IndeterminateAfterError)

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-73`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: LineWriter implicitly operates as a small state machine around its internal buffer: writes may either (a) be immediately forwarded, (b) be buffered as a partial line, or (c) be flushed up to a newline boundary. Additionally, if write_all returns an error, the tests document that the writer is left in an 'indeterminate state' where no further assertions/operations are meaningful. None of these states are represented in the type system: callers can continue using the same LineWriter value after an error, even though the API contract says its state is no longer well-defined.

**Evidence**:

```rust
    // This should write "Line 1\n" and buffer "Partial"
    assert_eq!(writer.write(b"Line 1\nPartial").unwrap(), 14);
    assert_eq!(&writer.get_ref().buffer, b"Line 1\n");

    // This will flush partial, which will succeed, but then return Ok(0)
    // when flushing " Line End\n"
    assert_eq!(writer.write(b" Line End\n").unwrap(), 0);
    assert_eq!(&writer.get_ref().buffer, b"Line 1\nPartial");
}

/// LineWriter has a custom `write_all`; make sure it works correctly
#[test]
fn line_write_all() {
    let writer = ProgrammableSink {
        // Only write 5 bytes at a time
        accept_prefix: Some(5),
        ..Default::default()
    };
    let mut writer = LineWriter::new(writer);

    writer.write_all(b"Line 1\nLine 2\nLine 3\nLine 4\nPartial").unwrap();
    assert_eq!(&writer.get_ref().buffer, b"Line 1\nLine 2\nLine 3\nLine 4\n");
    writer.write_all(b" Line 5\n").unwrap();
    assert_eq!(
        writer.get_ref().buffer.as_slice(),
        b"Line 1\nLine 2\nLine 3\nLine 4\nPartial Line 5\n".as_ref(),
    );
}

#[test]
fn line_write_all_error() {
    let writer = ProgrammableSink {
        // Only accept up to 3 writes of up to 5 bytes each
        accept_prefix: Some(5),
        max_writes: Some(3),
        ..Default::default()
    };

    let mut writer = LineWriter::new(writer);
    let res = writer.write_all(b"Line 1\nLine 2\nLine 3\nLine 4\nPartial");
    assert!(res.is_err());
    // An error from write_all leaves everything in an indeterminate state,
    // so there's nothing else to test here
}

/// Under certain circumstances, the old implementation of LineWriter
/// would try to buffer "to the last newline" but be forced to buffer
/// less than that, leading to inappropriate partial line writes.
/// Regression test for that issue.
#[test]
fn partial_multiline_buffering() {
    let writer = ProgrammableSink {
        // Write only up to 5 bytes at a time
        accept_prefix: Some(5),
        ..Default::default()
    };

    let mut writer = LineWriter::with_capacity(10, writer);

    let content = b"AAAAABBBBB\nCCCCDDDDDD\nEEE";

    // When content is written, LineWriter will try to write blocks A, B,
    // C, and D. Only block A will succeed. Under the old behavior, LineWriter
    // would then try to buffer B, C and D, but because its capacity is 10,
    // it will only be able to buffer B and C. We don't want to buffer
    // partial lines concurrent with whole lines, so the correct behavior
    // is to buffer only block B (out to the newline)
    assert_eq!(writer.write(content).unwrap(), 11);
    assert_eq!(writer.get_ref().buffer, *b"AAAAA");

    writer.flush().unwrap();
    assert_eq!(writer.get_ref().buffer, *b"AAAAABBBBB\n");
}

```

**Entity:** LineWriter<W>

**States:** Buffered (may contain partial line), Flushed-to-newline (buffer ends at newline boundary), Indeterminate (after write_all error)

**Transitions:**
- Buffered -> Flushed-to-newline via flush() (or internally when a newline is encountered and flushing succeeds)
- Any -> Indeterminate via write_all(...) returning Err

**Evidence:** comment: "This should write \"Line 1\n\" and buffer \"Partial\"" (write() both writes-through and buffers remainder); asserts: writer.get_ref().buffer equals b"Line 1\n" after writer.write(b"Line 1\nPartial"); comment: "This will flush partial, which will succeed, but then return Ok(0) when flushing \" Line End\n\"" (write() has multi-phase behavior depending on buffer/newline state); test line_write_all(): "LineWriter has a custom `write_all`; make sure it works correctly" plus assertions that only complete lines are flushed and trailing partial data is buffered until later; test line_write_all_error(): comment "An error from write_all leaves everything in an indeterminate state" and the test intentionally performs no further checks after res.is_err()

**Implementation:** Encode post-error and buffering modes in the type: e.g., LineWriter<W, S> with states like Normal and Poisoned (after write_all error). Have write_all(self, ..) -> Result<LineWriter<W, Normal>, LineWriter<W, Poisoned>> (or return a Poisoned wrapper on error). Optionally split Normal into Buffered/AtLineBoundary to make certain operations (like assumptions about newline-terminated buffer) only available when the type guarantees it.

---

### 12. Tag-stealing alignment/thin-pointer protocol for payload pointers

**Location**: `/tmp/io_test_crate/src/io/error/repr_bitpacked.rs:1-69`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: The encoding protocol requires that pointers to `SimpleMessage` and `Custom` can be converted to and from a tagged form by manipulating low bits (and, per comment, by offsetting a pointer by `TAG_CUSTOM`). This relies on implicit alignment guarantees (low bits are zero) and on the pointer remaining within the same object and not wrapping the address space when adjusted. These are correctness/soundness requirements of the encoding, but they are not represented in the types of the pointers; they are enforced via `static_assert!`s and comments, and (elsewhere) presumably by careful use of `wrapping_add`/masking.

**Evidence**:

```rust
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
```

**Entity:** tagged pointer payload types (`SimpleMessage`, `Custom`) as stored in repr_bitpacked

**States:** UntaggedAlignedPtr, TaggedPtr

**Transitions:**
- UntaggedAlignedPtr -> TaggedPtr via applying TAG_MASK/TAG_* (bitpacking/encoding step)
- TaggedPtr -> UntaggedAlignedPtr via masking off TAG_MASK (decoding step)

**Evidence:** static_assert!((TAG_MASK + 1).is_power_of_two()): implies protocol uses low-bit masking for tags; static_assert!(align_of::<SimpleMessage>() >= TAG_MASK + 1) and static_assert!(align_of::<Custom>() >= TAG_MASK + 1): ensures low bits are available for tags; static_assert!(@usize_eq: TAG_MASK & TAG_SIMPLE_MESSAGE, TAG_SIMPLE_MESSAGE) (and similar for TAG_CUSTOM/TAG_OS/TAG_SIMPLE): tag constants must fit within TAG_MASK; comment: "in `Repr::new_custom` we offset a pointer by this value, and expect it to both be within the same object, and to not wrap around the address space"; comment: "at the moment we use `ptr::wrapping_add`, not `ptr::add` ... the assertion that we don't actually wrap around ... simplifies the safety reasoning"

**Implementation:** Define a private `#[repr(transparent)] struct TaggedPtr(NonNull<()>);` (or `usize`) with constructors that only accept `NonNull<SimpleMessage>`/`NonNull<Custom>` and perform the tagging internally. Expose only safe methods like `fn tag_simple_message(p: NonNull<SimpleMessage>) -> TaggedPtr` and `fn decode(&self) -> Decoded` where `Decoded` is an enum of the possible payloads. This concentrates and type-checks the tag protocol so callers cannot mix raw pointers/usize with tagged values accidentally.

---

### 24. BufWriter drop/flush protocol under panics (Normal / Panicking-Unwind)

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-107`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: BufWriter has an implicit protocol around flushing buffered data: in normal execution users may call flush() to push buffered bytes, and Drop will typically attempt to flush. However, when the thread is already panicking (unwinding), Drop must avoid doing fallible work that could panic again or otherwise escalate (double panic -> abort). This panic-aware behavior is relied on by tests, but the type system does not distinguish "safe to flush in Drop" from "must not flush because we're unwinding", nor does it force users into an explicit "finished/committed" state before drop.

**Evidence**:

```rust
    assert_eq!(s, "");
}

#[test]
fn test_lines() {
    let in_buf: &[u8] = b"a\nb\nc";
    let reader = BufReader::with_capacity(2, in_buf);
    let mut it = reader.lines();
    assert_eq!(it.next().unwrap().unwrap(), "a".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "b".to_string());
    assert_eq!(it.next().unwrap().unwrap(), "c".to_string());
    assert!(it.next().is_none());
}

#[test]
fn test_short_reads() {
    let inner = ShortReader { lengths: vec![0, 1, 2, 0, 1, 0] };
    let mut reader = BufReader::new(inner);
    let mut buf = [0, 0];
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read(&mut buf).unwrap(), 0);
}

#[test]
#[should_panic]
fn dont_panic_in_drop_on_panicked_flush() {
    struct FailFlushWriter;

    impl Write for FailFlushWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::last_os_error())
        }
    }

    let writer = FailFlushWriter;
    let _writer = BufWriter::new(writer);

    // If writer panics *again* due to the flush error then the process will
    // abort.
    panic!();
}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn panic_in_write_doesnt_flush_in_drop() {
    static WRITES: AtomicUsize = AtomicUsize::new(0);

    struct PanicWriter;

    impl Write for PanicWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            WRITES.fetch_add(1, Ordering::SeqCst);
            panic!();
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    thread::spawn(|| {
        let mut writer = BufWriter::new(PanicWriter);
        let _ = writer.write(b"hello world");
        let _ = writer.flush();
    })
    .join()
    .unwrap_err();

    assert_eq!(WRITES.load(Ordering::SeqCst), 1);
}

#[bench]
fn bench_buffered_reader(b: &mut test::Bencher) {
    b.iter(|| BufReader::new(io::empty()));
}

#[bench]
fn bench_buffered_reader_small_reads(b: &mut test::Bencher) {
    let data = (0..u8::MAX).cycle().take(1024 * 4).collect::<Vec<_>>();
    b.iter(|| {
        let mut reader = BufReader::new(&data[..]);
        let mut buf = [0u8; 4];
        for _ in 0..1024 {
            reader.read_exact(&mut buf).unwrap();
            core::hint::black_box(&buf);
        }
    });
}

#[bench]
fn bench_buffered_writer(b: &mut test::Bencher) {
    b.iter(|| BufWriter::new(io::sink()));
}

/// A simple `Write` target, designed to be wrapped by `LineWriter` /
/// `BufWriter` / etc, that can have its `write` & `flush` behavior
/// configured
#[derive(Default, Clone)]
struct ProgrammableSink {
    // Writes append to this slice
```

**Entity:** BufWriter<W>

**States:** Normal, Panicking (unwinding)

**Transitions:**
- Normal -> Panicking (unwinding) via panic!() while BufWriter is in scope
- Normal -> (explicitly flushed) via BufWriter::flush()
- Panicking (unwinding) -> drop without propagating flush error (must not panic again)

**Evidence:** test dont_panic_in_drop_on_panicked_flush: FailFlushWriter::flush() returns Err(io::Error::last_os_error()); comment: "If writer panics *again* due to the flush error then the process will abort." (explicitly states the invariant for Drop during unwinding); test panic_in_write_doesnt_flush_in_drop: PanicWriter::write() panics; thread::spawn(...).join().unwrap_err() asserts the panic happened; test panic_in_write_doesnt_flush_in_drop: WRITES AtomicUsize asserted == 1, implying drop-time flush must not trigger additional write attempts after a panic in write

**Implementation:** Expose an explicit "finish/commit" transition that users must call to guarantee flushing (e.g., BufWriter<Open>::into_inner(self) -> Result<W, IntoInnerError<BufWriter<Open>>> or close(self) -> Result<BufWriter<Flushed>, E>). Methods that rely on flushing semantics would be on the Flushed state, while Drop for the Open state would be best-effort and explicitly non-panicking. This makes "guaranteed flushed" a compile-time-distinguished state instead of an implicit runtime/panic-context behavior.

---

### 58. TestWriter configuration/behavior protocol (limited vectored write)

**Location**: `/tmp/io_test_crate/src/io/tests.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TestWriter’s behavior is governed by two runtime configuration parameters: it will only consider the first `n_bufs` IoSlices in a vectored write, and it will write at most `per_call` total bytes per call. It also accumulates all bytes written into `written` across calls. These constraints form an implicit protocol/contract relied on by the tests (e.g., 'only first buffer is read', 'at most N bytes per call'), but nothing in the type system distinguishes differently-configured writers (e.g., `n_bufs = 1` vs `n_bufs = 3`) or prevents constructing semantically invalid configurations (like `per_call = 0`) if that were undesired. The protocol is enforced only by runtime fields and method logic, not by types.

**Evidence**:

```rust

/// Creates a new writer that reads from at most `n_bufs` and reads
/// `per_call` bytes (in total) per call to write.
fn test_writer(n_bufs: usize, per_call: usize) -> TestWriter {
    TestWriter { n_bufs, per_call, written: Vec::new() }
}

struct TestWriter {
    n_bufs: usize,
    per_call: usize,
    written: Vec<u8>,
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_vectored(&[IoSlice::new(buf)])
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut left = self.per_call;
        let mut written = 0;
        for buf in bufs.iter().take(self.n_bufs) {
            let n = min(left, buf.len());
            self.written.extend_from_slice(&buf[0..n]);
            left -= n;
            written += n;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_writer_read_from_one_buf() {
    let mut writer = test_writer(1, 2);

    assert_eq!(writer.write(&[]).unwrap(), 0);
    assert_eq!(writer.write_vectored(&[]).unwrap(), 0);

    // Read at most 2 bytes.
    assert_eq!(writer.write(&[1, 1, 1]).unwrap(), 2);
    let bufs = &[IoSlice::new(&[2, 2, 2])];
    assert_eq!(writer.write_vectored(bufs).unwrap(), 2);

    // Only read from first buf.
    let bufs = &[IoSlice::new(&[3]), IoSlice::new(&[4, 4])];
    assert_eq!(writer.write_vectored(bufs).unwrap(), 1);

    assert_eq!(writer.written, &[1, 1, 2, 2, 3]);
}

#[test]
fn test_writer_read_from_multiple_bufs() {
    let mut writer = test_writer(3, 3);

    // Read at most 3 bytes from two buffers.
    let bufs = &[IoSlice::new(&[1]), IoSlice::new(&[2, 2, 2])];
    assert_eq!(writer.write_vectored(bufs).unwrap(), 3);

    // Read at most 3 bytes from three buffers.
```

**Entity:** TestWriter

**States:** Configured (n_bufs, per_call set), Accumulating (written growing over time)

**Transitions:**
- Configured -> Accumulating via write()/write_vectored() (appends into written)

**Evidence:** fn test_writer(n_bufs, per_call) -> TestWriter: constructs TestWriter with runtime parameters n_bufs/per_call; struct TestWriter { n_bufs: usize, per_call: usize, written: Vec<u8> }: fields encode configuration + evolving state; impl Write for TestWriter::write_vectored(): `for buf in bufs.iter().take(self.n_bufs)` enforces 'read from at most n_bufs' at runtime; impl Write for TestWriter::write_vectored(): `let mut left = self.per_call; let n = min(left, buf.len()); left -= n;` enforces 'at most per_call bytes per call' at runtime; impl Write for TestWriter::write_vectored(): `self.written.extend_from_slice(...)` accumulates bytes across calls (implicit state over time); test_writer_read_from_one_buf(): comments/assertions 'Read at most 2 bytes' and 'Only read from first buf' rely on this protocol; final `assert_eq!(writer.written, ...)` relies on accumulation across calls

**Implementation:** Introduce validated configuration types like `struct MaxBufs(NonZeroUsize)` and `struct PerCallLimit(NonZeroUsize)` (or domain-specific newtypes) and change `test_writer` to require them, preventing invalid/surprising configurations at compile time. Optionally encode common test modes as distinct types (e.g., `TestWriter<OneBuf>` vs `TestWriter<ManyBufs>`) using typestate/const generics (`TestWriter<const N_BUFS: usize, const PER_CALL: usize>`) so tests can’t accidentally use a writer with the wrong limits.

---

### 22. Line-buffered write protocol (buffer flush ordering + newline-aware splitting)

**Location**: `/tmp/io_test_crate/src/io/buffered/linewritershim.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The implementation relies on an implicit protocol for line buffering: (1) before buffering new bytes, if the existing internal buffer contains any newline(s), it should be flushed so completed lines are pushed to the inner writer; (2) when new input contains newline(s), bytes up to and including an appropriate newline boundary should be written directly (or prioritized) while the remainder is buffered; (3) buffering must not split a large remaining tail when the tail length exceeds the internal buffer capacity—those bytes should be deferred to a future write call to avoid short writes. These are enforced by ad-hoc branching on runtime values (newline positions, capacity, and how much has been flushed) and comments, but the type system does not distinguish ‘buffer currently contains a newline’ vs ‘does not’ nor ‘tail is safe to buffer’ vs ‘must defer’.

**Evidence**:

```rust
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
```

**Entity:** LineWriterShim (the type owning `self.buffer` in this module)

**States:** BufferHasNoNewline, BufferHasNewline, TailTooLargeToBuffer, TailFitsInBuffer

**Transitions:**
- BufferHasNoNewline -> BufferHasNewline via buffering data that includes '\n' (implied by comments about buffering after last newline)
- BufferHasNewline -> BufferHasNoNewline via flush() / self.buffer.flush() (completed lines drained)
- TailFitsInBuffer -> TailTooLargeToBuffer via tail.len() >= self.buffer.capacity() branch returning Ok(flushed)
- TailTooLargeToBuffer -> TailFitsInBuffer via subsequent write call with smaller tail (implied by comment: 'They can be written in full by the next call to write')

**Evidence:** comment: 'Because this function attempts to send completed lines to the underlying writer, it will also flush the existing buffer if it contains any newlines.'; comment: 'This means that, if any newlines are present in the data, the data up to and including ... the last newline is sent directly to the inner writer, and the data after it is buffered.'; code: `if tail.len() >= self.buffer.capacity() { return Ok(flushed); }` encodes the 'do not buffer tails >= capacity' rule; code: `else if newline_idx - flushed <= self.buffer.capacity() { &buf[flushed..newline_idx] }` encodes a 'fits in buffer up to newline' branch; code: `match memchr::memrchr(b'\n', scan_area) { ... }` shows runtime scanning for a newline boundary that fits within capacity; method: `fn flush(&mut self) -> io::Result<()> { self.buffer.flush() }` indicates explicit transition step to drain buffered data

**Implementation:** Split the shim/buffer wrapper into typestates that encode whether the internal buffer is known to contain a newline. For example: `LineWriterShim<S>` where `S` is `NoNl` or `HasNl`. Methods that may introduce a newline return `LineWriterShim<HasNl>`, while `flush(self) -> io::Result<LineWriterShim<NoNl>>`. Additionally, encapsulate the 'tail fits capacity' decision into a newtype/capability like `struct FitInBuf<'a>(&'a [u8]);` constructed only after checking `len < capacity`, so `write_to_buf` only accepts `FitInBuf` and cannot be called on oversized tails.

---

### 62. Protocol: must use read_buf, not read

**Location**: `/tmp/io_test_crate/src/io/tests.rs:1-60`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: This reader is only intended to be used through the `Read::read_buf` fast-path; calling `Read::read` is a protocol violation that triggers a panic. The intended usage is enforced only by a runtime panic message and test structure, not by types—any generic code that calls `read()` on a `Read` may panic when given this reader.

**Evidence**:

```rust

// Issue #120603
#[test]
#[should_panic]
fn read_buf_broken_read() {
    struct MalformedRead;

    impl Read for MalformedRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            // broken length calculation
            Ok(buf.len() + 1)
        }
    }

    let _ = BufReader::new(MalformedRead).fill_buf();
}

#[test]
fn read_buf_full_read() {
    struct FullRead;

    impl Read for FullRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
    }

    assert_eq!(BufReader::new(FullRead).fill_buf().unwrap().len(), DEFAULT_BUF_SIZE);
}

struct DataAndErrorReader(&'static [u8]);

impl Read for DataAndErrorReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        panic!("We want tests to use `read_buf`")
    }

    fn read_buf(&mut self, buf: io::BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf).unwrap();
        Err(io::Error::other("error"))
    }
}

#[test]
fn read_buf_data_and_error_take() {
    let mut buf = [0; 64];
    let mut buf = io::BorrowedBuf::from(buf.as_mut_slice());

    let mut r = DataAndErrorReader(&[4, 5, 6]).take(1);
    assert!(r.read_buf(buf.unfilled()).is_err());
    assert_eq!(buf.filled(), &[4]);

    assert!(r.read_buf(buf.unfilled()).is_ok());
    assert_eq!(buf.filled(), &[4]);
    assert_eq!(r.get_ref().0, &[5, 6]);
}

#[test]
fn read_buf_data_and_error_buf() {
    let mut r = BufReader::new(DataAndErrorReader(&[4, 5, 6]));

```

**Entity:** DataAndErrorReader (Read impl)

**States:** ReadBufPath, ReadPath

**Transitions:**
- ReadBufPath -> ReadPath via dispatch to `Read::read` (panic)

**Evidence:** `impl Read for DataAndErrorReader`: `fn read(...) { panic!("We want tests to use `read_buf`") }` encodes the protocol as a runtime check; `fn read_buf(&mut self, buf: io::BorrowedCursor<'_>)`: implemented and used by tests (`r.read_buf(...)`), indicating the intended path

**Implementation:** Define a separate trait/capability for types that support the `read_buf` path (e.g., `trait ReadBufOnly { fn read_buf(...); }`) and accept that in APIs/tests instead of `Read`. Alternatively, wrap `DataAndErrorReader` in a newtype that only exposes `read_buf` and does not implement `Read::read` (or implements `Read` only via an adapter that routes through `read_buf` without panicking).

---

### 40. Cursor-backed buffer growth/zero-fill invariant when writing past end

**Location**: `/tmp/io_test_crate/src/io/cursor/tests.rs:1-69`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: When the cursor position is moved beyond the current end of the underlying Vec (or an empty Vec), subsequent writes implicitly grow the buffer and (for the gap) fill intervening bytes with zeros. The tests depend on this behavior (gap is all zeros, then written bytes appear starting at the set position). This is a semantic protocol of the Cursor+Vec combination that is not reflected in the type system (e.g., 'sparse/zero-filling writer' vs 'in-bounds writer').

**Evidence**:

```rust
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(writer, b);
}

#[test]
fn test_mem_writer() {
    let mut writer = Cursor::new(Vec::new());
    writer.set_position(10);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..10], &[0; 10]);
    assert_eq!(&writer.get_ref()[10..], b);
}

#[test]
fn test_mem_writer_preallocated() {
    let mut writer = Cursor::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 8, 9, 10]);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..], b);
}

#[test]
fn test_mem_mut_writer() {
    let mut vec = Vec::new();
    let mut writer = Cursor::new(&mut vec);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(
        writer
            .write_vectored(&[IoSlice::new(&[]), IoSlice::new(&[8, 9]), IoSlice::new(&[10])],)
            .unwrap(),
        3
    );
    let b: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(&writer.get_ref()[..], b);
}

fn test_slice_writer<T>(writer: &mut Cursor<T>)
where
    T: AsRef<[u8]>,
    Cursor<T>: Write,
{
    assert_eq!(writer.position(), 0);
    assert_eq!(writer.write(&[0]).unwrap(), 1);
    assert_eq!(writer.position(), 1);
    assert_eq!(writer.write(&[1, 2, 3]).unwrap(), 3);
    assert_eq!(writer.write(&[4, 5, 6, 7]).unwrap(), 4);
    assert_eq!(writer.position(), 8);
    assert_eq!(writer.write(&[]).unwrap(), 0);
```

**Entity:** Cursor<Vec<u8>> (and Cursor<&mut Vec<u8>>)

**States:** Within current length, Past end (requires growth + fill)

**Transitions:**
- Within current length -> Past end via set_position(10) on an empty Vec
- Past end -> Within current length (after growth) via write(...) that extends the Vec

**Evidence:** test_mem_writer: let mut writer = Cursor::new(Vec::new()); writer.set_position(10); then writer.write(&[0]); test_mem_writer: assert_eq!(&writer.get_ref()[..10], &[0; 10]) asserts zero-filled gap before the first written byte; test_mem_writer: assert_eq!(&writer.get_ref()[10..], b) asserts bytes were written starting at offset 10 after gap

**Implementation:** Introduce a dedicated wrapper type for cursor writers that guarantee 'grow + zero-fill on seek-past-end' semantics (e.g., ZeroFillCursor<Vec<u8>>), making this behavior explicit in the type and docs. Alternatively, separate APIs: a checked in-bounds cursor (cannot seek past end) vs a growing cursor (can), with different types.

---

### 9. WriteObserver observation protocol (NoWritesYet -> ObservedWrites)

**Location**: `/tmp/io_test_crate/src/io/copy/tests.rs:1-62`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: WriteObserver is not just a writer; it is a measurement device that depends on the protocol 'call write() at least once' to produce a meaningful `observed_buffer`. Tests use it to infer properties of the copying algorithm (coalesced writes) via a runtime-updated field on the inner writer obtained through `BufWriter::get_mut()`. The type system does not distinguish between an observer that has seen writes and one that has not, nor does it prevent reading `observed_buffer` before any writes occurred.

**Evidence**:

```rust

    let mut r = repeat(0).take(1 << 17);
    assert_eq!(copy(&mut r as &mut dyn Read, &mut w as &mut dyn Write).unwrap(), 1 << 17);
}

struct ShortReader {
    cap: usize,
    read_size: usize,
    observed_buffer: usize,
}

impl Read for ShortReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes = min(self.cap, self.read_size).min(buf.len());
        self.cap -= bytes;
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(bytes)
    }
}

struct WriteObserver {
    observed_buffer: usize,
}

impl Write for WriteObserver {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.observed_buffer = max(self.observed_buffer, buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn copy_specializes_bufwriter() {
    let cap = 117 * 1024;
    let buf_sz = 16 * 1024;
    let mut r = ShortReader { cap, observed_buffer: 0, read_size: 1337 };
    let mut w = BufWriter::with_capacity(buf_sz, WriteObserver { observed_buffer: 0 });
    assert_eq!(
        copy(&mut r, &mut w).unwrap(),
        cap as u64,
        "expected the whole capacity to be copied"
    );
    assert_eq!(r.observed_buffer, buf_sz, "expected a large buffer to be provided to the reader");
    assert!(w.get_mut().observed_buffer > DEFAULT_BUF_SIZE, "expected coalesced writes");
}

#[test]
fn copy_specializes_bufreader() {
    let mut source = vec![0; 768 * 1024];
    source[1] = 42;
    let mut buffered = BufReader::with_capacity(256 * 1024, Cursor::new(&mut source));

    let mut sink = Vec::new();
    assert_eq!(crate::io::copy(&mut buffered, &mut sink).unwrap(), source.len() as u64);
    assert_eq!(source.as_slice(), sink.as_slice());

    let buf_sz = 71 * 1024;
    assert!(buf_sz > DEFAULT_BUF_SIZE, "test precondition");

```

**Entity:** WriteObserver

**States:** NoWritesYet(observed_buffer == 0), ObservedWrites(observed_buffer > 0)

**Transitions:**
- NoWritesYet -> ObservedWrites via Write::write() updating observed_buffer

**Evidence:** field: `observed_buffer: usize` is updated only in `Write::write`: `self.observed_buffer = max(self.observed_buffer, buf.len());`; test `copy_specializes_bufwriter`: asserts `w.get_mut().observed_buffer > DEFAULT_BUF_SIZE`, implicitly requiring that at least one write occurred and that observation is meaningful

**Implementation:** Split the observer into `WriteObserver<Unobserved>` and `WriteObserver<Observed>` and have `write()` transition to `Observed`, exposing an accessor like `observed_buffer()` only on `WriteObserver<Observed>`. If retaining the `Write` trait is necessary, wrap it: `ObservingWriter<W>` plus a separate `ObservationToken` capability returned after first write that permits reading the metric.

---

### 6. Initialization tracking protocol for MaybeUninit backing storage (initialized bytes must cover readable/filled bytes)

**Location**: `/tmp/io_test_crate/src/io/buffered/bufreader/buffer.rs:1-125`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Buffer uses Box<[MaybeUninit<u8>]> plus a separate initialized counter to describe which prefix of the allocation is actually initialized. read_more() relies on the protocol that before calling Read::read_buf it must correctly set BorrowedBuf's init length (old_init) to match the already-initialized tail (initialized - filled), and after the read it must update both filled and initialized consistently. This protocol is maintained manually with arithmetic and an unsafe set_init call; the type system does not prevent violating it (e.g., initialized < filled leading to assume_init_ref on uninitialized bytes, or initialized arithmetic underflow if invariants are broken).

**Evidence**:

```rust
//! that user code which wants to do reads from a `BufReader` via `buffer` + `consume` can do so
//! without encountering any runtime bounds checks.

use crate::cmp;
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
```

**Entity:** Buffer

**States:** NoInitializedData (initialized=0), PartiallyInitialized (0 < initialized < capacity), InitializedCoversFilled (initialized >= filled), InitializedOutOfSync (would be UB if buffer() exposes uninitialized)

**Transitions:**
- NoInitializedData/PartiallyInitialized -> PartiallyInitialized via read_more() (unsafe set_init + reader.read_buf + initialized update)
- Any -> InitializedCoversFilled via correct sequencing in read_more()/fill_buf() (ensuring initialized >= filled)
- Any -> NoInitializedData for purposes of 'available readable bytes' via discard_buffer() (filled=0; initialized retained, implying a distinct logical state)

**Evidence:** buf: Box<[MaybeUninit<u8>]> (explicitly allows uninitialized memory); field initialized comment: used "to tell `read_buf` how many bytes of buf are initialized" and that it "doesn't need to be" equal to filled; read_more(): let old_init = self.initialized - self.filled (requires invariant initialized >= filled to avoid underflow); read_more(): unsafe { buf.set_init(old_init); } (unsafe step depends on init-tracking correctness); read_more(): self.filled += buf.len(); self.initialized += buf.init_len() - old_init (manual arithmetic protocol to keep init tracking consistent); buffer(): assume_init_ref over self.pos..self.filled depends on the stronger invariant that bytes up to filled are actually initialized

**Implementation:** Split the buffer representation into a safe wrapper around initialization, e.g., a newtype that holds (storage, init_prefix_len) and only yields &mut [u8] for the uninitialized remainder through safe APIs. Alternatively, model a two-phase protocol: Buffer<InitTracked> where only that state can call buffer() (returning &[u8]) and transitions to/from a temporary BorrowedBuf-writing state (Buffer<Writing>) where the only operation is read_more/fill_buf and which must 'commit' the new init length before returning to InitTracked.

---

### 19. STDOUT initialization & shutdown protocol (Normal buffered -> Shutdown unbuffered)

**Location**: `/tmp/io_test_crate/src/io/stdio.rs:1-193`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: There is an implicit global protocol around the STDOUT singleton: normal operation uses a buffered LineWriter, while shutdown calls cleanup() to flush/disable buffering by swapping in a zero-capacity LineWriter. This relies on runtime initialization (get_or_init), a local 'initialized' flag to detect which branch occurred, and best-effort behavior during shutdown (try_lock to avoid deadlock if a lock was leaked). The type system does not encode whether STDOUT is in normal vs shutdown mode, nor whether cleanup() successfully performed the swap/flush (it may be skipped if try_lock fails).

**Evidence**:

```rust
/// ```
///
/// Ensuring output is flushed immediately:
///
/// ```no_run
/// use std::io::{self, Write};
///
/// fn main() -> io::Result<()> {
///     let mut stdout = io::stdout();
///     stdout.write_all(b"hello, ")?;
///     stdout.flush()?;                // Manual flush
///     stdout.write_all(b"world!\n")?; // Automatically flushed
///     Ok(())
/// }
/// ```
///
/// [`flush`]: Write::flush
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "io_stdout")]
pub fn stdout() -> Stdout {
    Stdout {
        inner: STDOUT
            .get_or_init(|| ReentrantLock::new(RefCell::new(LineWriter::new(stdout_raw())))),
    }
}

// Flush the data and disable buffering during shutdown
// by replacing the line writer by one with zero
// buffering capacity.
pub fn cleanup() {
    let mut initialized = false;
    let stdout = STDOUT.get_or_init(|| {
        initialized = true;
        ReentrantLock::new(RefCell::new(LineWriter::with_capacity(0, stdout_raw())))
    });

    if !initialized {
        // The buffer was previously initialized, overwrite it here.
        // We use try_lock() instead of lock(), because someone
        // might have leaked a StdoutLock, which would
        // otherwise cause a deadlock here.
        if let Some(lock) = stdout.try_lock() {
            *lock.borrow_mut() = LineWriter::with_capacity(0, stdout_raw());
        }
    }
}

impl Stdout {
    /// Locks this handle to the standard output stream, returning a writable
    /// guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The
    /// returned guard also implements the `Write` trait for writing data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::{self, Write};
    ///
    /// fn main() -> io::Result<()> {
    ///     let mut stdout = io::stdout().lock();
    ///
    ///     stdout.write_all(b"hello world")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> StdoutLock<'static> {
        // Locks this handle with 'static lifetime. This depends on the
        // implementation detail that the underlying `ReentrantMutex` is
        // static.
        StdoutLock { inner: self.inner.lock() }
    }
}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl UnwindSafe for Stdout {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for Stdout {}

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Stdout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stdout").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for Stdout {
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
impl Write for &Stdout {
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
impl UnwindSafe for StdoutLock<'_> {}

#[stable(feature = "catch_unwind", since = "1.9.0")]
impl RefUnwindSafe for StdoutLock<'_> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl Write for StdoutLock<'_> {
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for StdoutLock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdoutLock").finish_non_exhaustive()
    }
}

/// A handle to the standard error stream of a process.
///
/// For more information, see the [`io::stderr`] method.
///
/// [`io::stderr`]: stderr
///
/// ### Note: Windows Portability Considerations
///
/// When operating in a console, the Windows implementation of this stream does not support
/// non-UTF-8 byte sequences. Attempting to write bytes that are not valid UTF-8 will return
/// an error.
///
/// In a process with a detached console, such as one using
/// `#![windows_subsystem = "windows"]`, or in a child process spawned from such a process,
/// the contained handle will be null. In such cases, the standard library's `Read` and
```

**Entity:** global STDOUT / Stdout (and cleanup())

**States:** Uninitialized, InitializedNormal(LineWriter::new), ShutdownUnbuffered(LineWriter::with_capacity(0)), LockedLeaked(try_lock fails)

**Transitions:**
- Uninitialized -> InitializedNormal via stdout() (STDOUT.get_or_init(|| ... LineWriter::new(stdout_raw())))
- Uninitialized -> ShutdownUnbuffered via cleanup() (STDOUT.get_or_init(|| ... LineWriter::with_capacity(0, ...)))
- InitializedNormal -> ShutdownUnbuffered via cleanup() when try_lock() succeeds and overwrites the LineWriter
- InitializedNormal -> LockedLeaked via leaked StdoutLock preventing try_lock() during cleanup()

**Evidence:** stdout(): `STDOUT.get_or_init(|| ReentrantLock::new(RefCell::new(LineWriter::new(stdout_raw()))))` selects the 'normal buffered' writer; cleanup() comment: "Flush the data and disable buffering during shutdown" and "replacing the line writer by one with zero buffering capacity"; cleanup(): `let mut initialized = false;` plus `initialized = true;` inside the get_or_init closure to detect whether STDOUT was newly initialized; cleanup(): `if !initialized { ... if let Some(lock) = stdout.try_lock() { *lock.borrow_mut() = LineWriter::with_capacity(0, stdout_raw()); } }` shows a conditional overwrite only if already initialized and lock is obtainable; cleanup() comment: "someone might have leaked a StdoutLock, which would otherwise cause a deadlock here" plus use of `try_lock()` indicates an implicit 'no leaked lock at shutdown' requirement

**Implementation:** Represent global stdout as a handle parameterized by a state, e.g. `StdoutHandle<Normal>` and `StdoutHandle<Shutdown>`, where `shutdown(self) -> StdoutHandle<Shutdown>` performs the swap. Plumb this through a higher-level runtime/termination API so cleanup is not a free function that can be called at any time; alternatively use a capability token required to call cleanup (only held by the runtime) to prevent arbitrary transitions from user code.

---

### 28. LineWriter protocol: buffered-line flushing must be attempted before accepting new data

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-67`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The test relies on an implicit protocol in LineWriter: after a write that buffers data up to a newline, subsequent write() calls must first attempt to flush previously buffered complete lines before processing new incoming data, and must accurately report how many bytes were accepted vs buffered. This is a temporal ordering requirement (flush buffered lines before handling new data) that is only verified by runtime behavior and comments/assertions in the test, not encoded in types. The underlying sink is also configured to start failing after a limited number of writes, making the 'pending buffered line' vs 'new data' distinction observable and important.

**Evidence**:

```rust
    // error; otherwise, it will return Ok(0).
    pub error_after_max_writes: bool,
}

impl Write for ProgrammableSink {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.always_write_error {
            return Err(io::Error::new(io::ErrorKind::Other, "test - always_write_error"));
        }

        match self.max_writes {
            Some(0) if self.error_after_max_writes => {
                return Err(io::Error::new(io::ErrorKind::Other, "test - max_writes"));
            }
            Some(0) => return Ok(0),
            Some(ref mut count) => *count -= 1,
            None => {}
        }

        let len = match self.accept_prefix {
            None => data.len(),
            Some(prefix) => data.len().min(prefix),
        };

        let data = &data[..len];
        self.buffer.extend_from_slice(data);

        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.always_flush_error {
            Err(io::Error::new(io::ErrorKind::Other, "test - always_flush_error"))
        } else {
            Ok(())
        }
    }
}

/// Previously the `LineWriter` could successfully write some bytes but
/// then fail to report that it has done so. Additionally, an erroneous
/// flush after a successful write was permanently ignored.
///
/// Test that a line writer correctly reports the number of written bytes,
/// and that it attempts to flush buffered lines from previous writes
/// before processing new data
///
/// Regression test for #37807
#[test]
fn erroneous_flush_retried() {
    let writer = ProgrammableSink {
        // Only write up to 4 bytes at a time
        accept_prefix: Some(4),

        // Accept the first two writes, then error the others
        max_writes: Some(2),
        error_after_max_writes: true,

        ..Default::default()
    };

    // This should write the first 4 bytes. The rest will be buffered, out
    // to the last newline.
    let mut writer = LineWriter::new(writer);
    assert_eq!(writer.write(b"a\nb\nc\nd\ne").unwrap(), 8);

    // This write should attempt to flush "c\nd\n", then buffer "e". No
```

**Entity:** LineWriter<ProgrammableSink> (as exercised by test `erroneous_flush_retried`)

**States:** NoPendingBufferedLine, HasPendingBufferedLineNeedingFlush, UnderlyingSinkWriteFailureEncountered

**Transitions:**
- NoPendingBufferedLine -> HasPendingBufferedLineNeedingFlush via LineWriter::write() when input contains complete line(s) but underlying sink accepts only a prefix (accept_prefix)
- HasPendingBufferedLineNeedingFlush -> NoPendingBufferedLine via LineWriter::write()/flush path that successfully flushes buffered complete lines before handling new data
- HasPendingBufferedLineNeedingFlush -> UnderlyingSinkWriteFailureEncountered via LineWriter::write() when flushing buffered lines triggers sink error (max_writes/error_after_max_writes)

**Evidence:** comment in test: "Test that a line writer correctly reports the number of written bytes, and that it attempts to flush buffered lines from previous writes before processing new data"; test setup relies on partial writes: ProgrammableSink configured with `accept_prefix: Some(4)` ("Only write up to 4 bytes at a time"); test setup relies on eventual sink failure: `max_writes: Some(2), error_after_max_writes: true` ("Accept the first two writes, then error the others"); comment describing ordering: "This write should attempt to flush \"c\nd\n\", then buffer \"e\"."; assertion about reported bytes: `assert_eq!(writer.write(b"a\nb\nc\nd\ne").unwrap(), 8);` depends on correct accounting despite buffering/flush behavior

**Implementation:** Expose a typed API for the buffering protocol in tests, e.g. a `LineWriter` wrapper that returns a state token after `write()` indicating whether a flush is pending (`PendingFlush` vs `NoPending`). Only allow subsequent `write()` without an explicit `flush_pending()` call when in `NoPending`. Alternatively, provide a test-only adapter around `LineWriter` that splits operations into `write_new_data()` and `flush_buffered_lines()` with distinct types to force the intended ordering.

---

### 54. Two-thread lock-step handshake protocol (Start/Acquire/Release phases)

**Location**: `/tmp/io_test_crate/src/io/stdio/tests.rs:1-166`

**Confidence**: medium

**Suggested Pattern**: session_type

**Description**: The test encodes an implicit multi-step protocol between the main thread and two worker threads to ensure locks block/unblock in a specific temporal order. Correctness depends on sending/receiving a specific sequence of `State` messages on the correct channels (rx1/rx2) and in the correct order; violating the sequence can deadlock or make assertions meaningless. This protocol is enforced only by runtime sequencing of `send`/`recv` and by final log equality, not by the type system (channels carry an unrefined `State` enum that permits any message at any time).

**Evidence**:

```rust
use super::*;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::mpsc::sync_channel;
use crate::thread;

#[test]
fn stdout_unwind_safe() {
    assert_unwind_safe::<Stdout>();
}
#[test]
fn stdoutlock_unwind_safe() {
    assert_unwind_safe::<StdoutLock<'_>>();
    assert_unwind_safe::<StdoutLock<'static>>();
}
#[test]
fn stderr_unwind_safe() {
    assert_unwind_safe::<Stderr>();
}
#[test]
fn stderrlock_unwind_safe() {
    assert_unwind_safe::<StderrLock<'_>>();
    assert_unwind_safe::<StderrLock<'static>>();
}

fn assert_unwind_safe<T: UnwindSafe + RefUnwindSafe>() {}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn panic_doesnt_poison() {
    thread::spawn(|| {
        let _a = stdin();
        let _a = _a.lock();
        let _a = stdout();
        let _a = _a.lock();
        let _a = stderr();
        let _a = _a.lock();
        panic!();
    })
    .join()
    .unwrap_err();

    let _a = stdin();
    let _a = _a.lock();
    let _a = stdout();
    let _a = _a.lock();
    let _a = stderr();
    let _a = _a.lock();
}

#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stderr() {
    test_lock(stderr, || stderr().lock());
}
#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stdin() {
    test_lock(stdin, || stdin().lock());
}
#[test]
#[cfg_attr(any(target_os = "emscripten", target_os = "wasi"), ignore)] // no threads
fn test_lock_stdout() {
    test_lock(stdout, || stdout().lock());
}

// Helper trait to make lock testing function generic.
trait Stdio<'a>: 'static
where
    Self::Lock: 'a,
{
    type Lock;
    fn lock(&'a self) -> Self::Lock;
}
impl<'a> Stdio<'a> for Stderr {
    type Lock = StderrLock<'a>;
    fn lock(&'a self) -> StderrLock<'a> {
        self.lock()
    }
}
impl<'a> Stdio<'a> for Stdin {
    type Lock = StdinLock<'a>;
    fn lock(&'a self) -> StdinLock<'a> {
        self.lock()
    }
}
impl<'a> Stdio<'a> for Stdout {
    type Lock = StdoutLock<'a>;
    fn lock(&'a self) -> StdoutLock<'a> {
        self.lock()
    }
}

// Helper trait to make lock testing function generic.
trait StdioOwnedLock: 'static {}
impl StdioOwnedLock for StderrLock<'static> {}
impl StdioOwnedLock for StdinLock<'static> {}
impl StdioOwnedLock for StdoutLock<'static> {}

// Tests locking on stdio handles by starting two threads and checking that
// they block each other appropriately.
fn test_lock<T, U>(get_handle: fn() -> T, get_locked: fn() -> U)
where
    T: for<'a> Stdio<'a>,
    U: StdioOwnedLock,
{
    // State enum to track different phases of the test, primarily when
    // each lock is acquired and released.
    #[derive(Debug, PartialEq)]
    enum State {
        Start1,
        Acquire1,
        Start2,
        Release1,
        Acquire2,
        Release2,
    }
    use State::*;
    // Logging vector to be checked to make sure lock acquisitions and
    // releases happened in the correct order.
    let log = Arc::new(Mutex::new(Vec::new()));
    let ((tx1, rx1), (tx2, rx2)) = (sync_channel(0), sync_channel(0));
    let th1 = {
        let (log, tx) = (Arc::clone(&log), tx1);
        thread::spawn(move || {
            log.lock().unwrap().push(Start1);
            let handle = get_handle();
            {
                let locked = handle.lock();
                log.lock().unwrap().push(Acquire1);
                tx.send(Acquire1).unwrap(); // notify of acquisition
                tx.send(Release1).unwrap(); // wait for release command
                log.lock().unwrap().push(Release1);
            }
            tx.send(Acquire1).unwrap(); // wait for th2 acquire
            {
                let locked = handle.lock();
                log.lock().unwrap().push(Acquire1);
            }
            log.lock().unwrap().push(Release1);
        })
    };
    let th2 = {
        let (log, tx) = (Arc::clone(&log), tx2);
        thread::spawn(move || {
            tx.send(Start2).unwrap(); // wait for start command
            let locked = get_locked();
            log.lock().unwrap().push(Acquire2);
            tx.send(Acquire2).unwrap(); // notify of acquisition
            tx.send(Release2).unwrap(); // wait for release command
            log.lock().unwrap().push(Release2);
        })
    };
    assert_eq!(rx1.recv().unwrap(), Acquire1); // wait for th1 acquire
    log.lock().unwrap().push(Start2);
    assert_eq!(rx2.recv().unwrap(), Start2); // block th2
    assert_eq!(rx1.recv().unwrap(), Release1); // release th1
    assert_eq!(rx2.recv().unwrap(), Acquire2); // wait for th2 acquire
    assert_eq!(rx1.recv().unwrap(), Acquire1); // block th1
    assert_eq!(rx2.recv().unwrap(), Release2); // release th2
    th2.join().unwrap();
    th1.join().unwrap();
    assert_eq!(
        *log.lock().unwrap(),
        [Start1, Acquire1, Start2, Release1, Acquire2, Release2, Acquire1, Release1]
    );
}
```

**Entity:** test_lock (thread coordination protocol via channels)

**States:** Start1, Acquire1, Start2, Release1, Acquire2, Release2

**Transitions:**
- Main waits for th1: recv Acquire1 on rx1
- Main signals th2 start: log Start2 then th2 recv Start2 on rx2
- Main releases th1: recv Release1 on rx1 (th1 only proceeds after receiving Release1 command via tx1/tx2 pairing)
- Main waits for th2 acquire: recv Acquire2 on rx2
- Main blocks th1 until th2 releases: recv Acquire1 on rx1 then recv Release2 on rx2
- Threads complete and join

**Evidence:** comment: "State enum to track different phases of the test" and "releases happened in the correct order"; enum State { Start1, Acquire1, Start2, Release1, Acquire2, Release2 } used as protocol tokens; sync_channel(0) used twice: `let ((tx1, rx1), (tx2, rx2)) = (sync_channel(0), sync_channel(0));` (zero-capacity rendezvous enforces step ordering at runtime); th1: `tx.send(Acquire1)` then `tx.send(Release1)` (used as notifications / wait points); th2: `tx.send(Start2)` then `tx.send(Acquire2)` then `tx.send(Release2)` (used as wait points); main thread assertions hard-code order: `assert_eq!(rx1.recv().unwrap(), Acquire1)`, then `assert_eq!(rx2.recv().unwrap(), Start2)`, etc.; final invariant checked only at runtime: `assert_eq!(*log.lock().unwrap(), [Start1, Acquire1, Start2, Release1, Acquire2, Release2, Acquire1, Release1])`

**Implementation:** Model each channel interaction as a typed session: e.g., define phantom-typed endpoints `Tx<S>`/`Rx<S>` where `S` encodes the next expected message(s). Each `send/recv` consumes an endpoint and returns the endpoint advanced to the next state, making it impossible to `recv` `Release2` before `Acquire2` or use `rx1` when the protocol expects `rx2`.

---

### 29. LineWriter buffering/flush protocol (Buffered -> Flushed-on-newline)

**Location**: `/tmp/io_test_crate/src/io/buffered/tests.rs:1-86`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The tests rely on an implicit protocol that LineWriter aggregates formatted output so that a single formatted writeln! results in exactly one underlying write call (not multiple writes caused by intermediate buffer flushes). Correct behavior depends on LineWriter deferring/combining writes until it can emit a complete line (newline-terminated) or until flush() is explicitly requested. This batching/flush behavior is a runtime policy of LineWriter; the type system does not distinguish 'currently holding partial line' vs 'contains newline and should flush', nor can it express 'this write will be forwarded as a single underlying write'.

**Evidence**:

```rust
    // would then try to buffer B and C, but because its capacity is 5,
    // it will only be able to buffer part of B. Because it's not possible
    // for it to buffer any complete lines, it should buffer as much of B as
    // possible
    assert_eq!(writer.write(content).unwrap(), 10);
    assert_eq!(writer.get_ref().buffer, *b"AAAAA");

    writer.flush().unwrap();
    assert_eq!(writer.get_ref().buffer, *b"AAAAABBBBB");
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordedEvent {
    Write(String),
    Flush,
}

#[derive(Debug, Clone, Default)]
struct WriteRecorder {
    pub events: Vec<RecordedEvent>,
}

impl Write for WriteRecorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use crate::str::from_utf8;

        self.events.push(RecordedEvent::Write(from_utf8(buf).unwrap().to_string()));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.events.push(RecordedEvent::Flush);
        Ok(())
    }
}

/// Test that a normal, formatted writeln only results in a single write
/// call to the underlying writer. A naive implementation of
/// LineWriter::write_all results in two writes: one of the buffered data,
/// and another of the final substring in the formatted set
#[test]
fn single_formatted_write() {
    let writer = WriteRecorder::default();
    let mut writer = LineWriter::new(writer);

    // Under a naive implementation of LineWriter, this will result in two
    // writes: "hello, world" and "!\n", because write() has to flush the
    // buffer before attempting to write the last "!\n". write_all shouldn't
    // have this limitation.
    writeln!(&mut writer, "{}, {}!", "hello", "world").unwrap();
    assert_eq!(writer.get_ref().events, [RecordedEvent::Write("hello, world!\n".to_string())]);
}

#[test]
fn bufreader_full_initialize() {
    struct OneByteReader;
    impl Read for OneByteReader {
        fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize> {
            if buf.len() > 0 {
                buf[0] = 0;
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }
    let mut reader = BufReader::new(OneByteReader);
    // Nothing is initialized yet.
    assert_eq!(reader.initialized(), 0);

    let buf = reader.fill_buf().unwrap();
    // We read one byte...
    assert_eq!(buf.len(), 1);
    // But we initialized the whole buffer!
    assert_eq!(reader.initialized(), reader.capacity());
}

/// This is a regression test for https://github.com/rust-lang/rust/issues/127584.
#[test]
fn bufwriter_aliasing() {
    use crate::io::{BufWriter, Cursor};
    let mut v = vec![0; 1024];
    let c = Cursor::new(&mut v);
    let w = BufWriter::new(Box::new(c));
    let _ = w.into_parts();
}
```

**Entity:** LineWriter<WriteRecorder> (and LineWriter<W> generally)

**States:** Buffering (no newline seen / partial line), Ready-to-flush (newline present or explicit flush), Flushed (buffer drained to inner writer)

**Transitions:**
- Buffering -> Ready-to-flush via writeln!/write_all producing a newline-terminated chunk
- Ready-to-flush -> Flushed via internal flush-to-inner (triggered by write/write_all) or via flush()
- Buffering -> Flushed via flush() (forces emission even without newline)

**Evidence:** comment above single_formatted_write(): "Test that a normal, formatted writeln only results in a single write call to the underlying writer."; comment: "A naive implementation of LineWriter::write_all results in two writes" and "write_all shouldn't have this limitation."; single_formatted_write(): writeln!(&mut writer, "{}, {}!", ...) followed by assert_eq!(writer.get_ref().events, [RecordedEvent::Write("hello, world!\n".to_string())]) which encodes the 'exactly one underlying write' protocol; WriteRecorder::write(): records each call as RecordedEvent::Write(...), making the underlying-call-count/aggregation a tested behavioral invariant

**Implementation:** Expose an explicit stateful API that separates 'building a line' from 'committing' it, e.g. a LineWriter with a method line() -> LineBuilder<'_, W, Open> where only LineBuilder<Open> allows push/formatting and only LineBuilder<Complete> (newline present) allows commit() that performs exactly one inner write. Alternatively, provide a dedicated write_line(&str) API returning a capability token that guarantees a single underlying write for a complete line.

---

### 32. BufMut initialization protocol when using read_buf (unfilled initialized bytes tracking)

**Location**: `/tmp/io_test_crate/src/io/mod.rs:1-66`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The `read_buf` implementation relies on a strict protocol with the `BufMut`-like `buf`: after bytes are filled, the code must advance the cursor and then mark the correct number of newly-initialized bytes with `set_init(new_init)`. This is maintained via an unsafe sequence (`advance_unchecked` + `set_init`) and a SAFETY comment, meaning the correctness depends on the temporal ordering and matching of `filled`/`new_init` at runtime, not on the type system.

**Evidence**:

```rust
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

/// An iterator over `u8` values of a reader.
///
/// This struct is generally created by calling [`bytes`] on a reader.
/// Please see the documentation of [`bytes`] for more details.
///
/// [`bytes`]: Read::bytes
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Bytes<R> {
    inner: R,
}

#[stable(feature = "rust1", since = "1.0.0")]
```

**Entity:** Take<T> (read_buf / BufMut interaction)

**States:** Unfilled buffer not initialized, Unfilled buffer initialized (set_init advanced)

**Transitions:**
- Unfilled buffer not initialized -> Unfilled buffer initialized via `buf.advance_unchecked(filled); buf.set_init(new_init);` after a successful fill

**Evidence:** `buf.advance_unchecked(filled);` uses unchecked advancement (requires protocol correctness); SAFETY comment: "new_init bytes of buf's unfilled buffer have been initialized" immediately before `buf.set_init(new_init);`; `buf.set_init(new_init);` explicitly records initialized byte count, implying a required ordering/consistency with the actual writes

**Implementation:** Use typestates/capabilities for buffer initialization: split the buffer API into a stage that yields an `UninitSlice`/`Unfilled<'a>` token and only allows returning a `Filled<'a>` token after writing, where `Filled`'s constructor requires the exact initialized length. Then `advance`/`set_init` become safe operations on `Filled` and cannot be called out of order.

---

