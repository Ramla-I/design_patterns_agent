# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 13
- **Temporal ordering**: 2
- **Resource lifecycle**: 2
- **State machine**: 2
- **Precondition**: 3
- **Protocol**: 4
- **Modules analyzed**: 11

## Temporal Ordering Invariants

### 4. Pre-bind-only socket option protocol (OnlyV6 must be set before bind)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-334`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Some socket options are only valid before binding a socket (here: only_v6). The current API still exposes set_only_v6/only_v6 on TcpListener (which is already bound, since it's constructed via bind()), relying on deprecation text and OS/runtime errors to enforce the ordering. The type system cannot prevent calling these methods after bind even though the protocol requires setting the option beforehand.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpStream, impl TcpStream (19 methods), impl Read for TcpStream (4 methods), impl Write for TcpStream (4 methods), impl Read for & TcpStream (4 methods), impl Write for & TcpStream (4 methods), impl AsInner < net_imp :: TcpStream > for TcpStream (1 methods), impl FromInner < net_imp :: TcpStream > for TcpStream (1 methods), impl IntoInner < net_imp :: TcpStream > for TcpStream (1 methods); struct Incoming, impl Iterator for Incoming < 'a > (1 methods), impl FusedIterator for Incoming < '_ > (0 methods), impl Iterator for IntoIncoming (1 methods), impl FusedIterator for IntoIncoming (0 methods); struct IntoIncoming

/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct TcpListener(net_imp::TcpListener);


// ... (other code) ...

    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`TcpListener::local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`:
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`. If that fails, create a
    /// TCP listener bound to `127.0.0.1:443`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, TcpListener};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 80)),
    ///     SocketAddr::from(([127, 0, 0, 1], 443)),
    /// ];
    /// let listener = TcpListener::bind(&addrs[..]).unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to a port assigned by the operating system
    /// at `127.0.0.1`.
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, net_imp::TcpListener::bind).map(TcpListener)
    }

    /// Returns the local socket address of this listener.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// assert_eq!(listener.local_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned [`TcpListener`] is a reference to the same socket that this
    /// object references. Both handles can be used to accept incoming
    /// connections and options set on one listener will affect the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// let listener_clone = listener.try_clone().unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<TcpListener> {
        self.0.duplicate().map(TcpListener)
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// match listener.accept() {
    ///     Ok((_socket, addr)) => println!("new client: {addr:?}"),
    ///     Err(e) => println!("couldn't get client: {e:?}"),
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        // On WASM, `TcpStream` is uninhabited (as it's unsupported) and so
        // the `a` variable here is technically unused.
        #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
        self.0.accept().map(|(a, b)| (TcpStream(a), b))
    }

    /// Returns an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`TcpListener::accept`] in a loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{TcpListener, TcpStream};
    ///
    /// fn handle_connection(stream: TcpStream) {
    ///    //...
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:80")?;
    ///
    ///     for stream in listener.incoming() {
    ///         match stream {
    ///             Ok(stream) => {
    ///                 handle_connection(stream);
    ///             }
    ///             Err(e) => { /* connection failed */ }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// Turn this into an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`TcpListener::accept`] in a loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcplistener_into_incoming)]
    /// use std::net::{TcpListener, TcpStream};
    ///
    /// fn listen_on(port: u16) -> impl Iterator<Item = TcpStream> {
    ///     let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    ///     listener.into_incoming()
    ///         .filter_map(Result::ok) /* Ignore failed connections */
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     for stream in listen_on(80) {
    ///         /* handle the connection here */
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
    pub fn into_incoming(self) -> IntoIncoming {
        IntoIncoming { listener: self }
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.set_ttl(100).expect("could not set TTL");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`TcpListener::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.set_ttl(100).expect("could not set TTL");
    /// assert_eq!(listener.ttl().unwrap_or(0), 100);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    #[stable(feature = "net2_mutators", since = "1.9.0")]
    #[deprecated(since = "1.16.0", note = "this option can only be set before the socket is bound")]
    #[allow(missing_docs)]
    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.0.set_only_v6(only_v6)
    }

    #[stable(feature = "net2_mutators", since = "1.9.0")]
    #[deprecated(since = "1.16.0", note = "this option can only be set before the socket is bound")]
    #[allow(missing_docs)]
    pub fn only_v6(&self) -> io::Result<bool> {
        self.0.only_v6()
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.take_error().expect("No error was expected");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in the `accept` operation becoming nonblocking,
    /// i.e., immediately returning from their calls. If the IO operation is
    /// successful, `Ok` is returned and no further action is required. If the
    /// IO operation could not be completed and needs to be retried, an error
    /// with kind [`io::ErrorKind::WouldBlock`] is returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    /// # Examples
    ///
    /// Bind a TCP listener to an address, listen for connections, and read
    /// bytes in nonblocking mode:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    /// listener.set_nonblocking(true).expect("Cannot set non-blocking");
    ///
    /// # fn wait_for_fd() { unimplemented!() }
    /// # fn handle_connection(stream: std::net::TcpStream) { unimplemented!() }
    /// for stream in listener.incoming() {
    ///     match stream {
    ///         Ok(s) => {
    ///             // do something with the TcpStream
    ///             handle_connection(s);
    ///         }
    ///         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    ///             // wait until network socket is ready, typically implemented
    ///             // via platform-specific APIs such as epoll or IOCP
    ///             wait_for_fd();
    ///             continue;
    ///         }
    ///         Err(e) => panic!("encountered IO error: {e}"),
    ///     }
    /// }
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

// ... (other code) ...

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl FusedIterator for IntoIncoming {}

impl AsInner<net_imp::TcpListener> for TcpListener {
    #[inline]
    fn as_inner(&self) -> &net_imp::TcpListener {
        &self.0
    }
}

impl FromInner<net_imp::TcpListener> for TcpListener {
    fn from_inner(inner: net_imp::TcpListener) -> TcpListener {
        TcpListener(inner)
    }
}

impl IntoInner<net_imp::TcpListener> for TcpListener {
    fn into_inner(self) -> net_imp::TcpListener {
        self.0
    }
}

```

**Entity:** TcpListener

**States:** PreBind (unbound socket), PostBind (bound/listening)

**Transitions:**
- PreBind -> PostBind via bind()

**Evidence:** set_only_v6(): #[deprecated(..., note = "this option can only be set before the socket is bound")]; only_v6(): #[deprecated(..., note = "this option can only be set before the socket is bound")]; bind(): constructs TcpListener and docs say "The returned listener is ready for accepting connections." (implies it's already bound/listening, i.e., PostBind state)

**Implementation:** Introduce a separate pre-bind builder/typestate, e.g. TcpSocket<Unbound> with set_only_v6 on Unbound only, and bind(self, addr) -> io::Result<TcpListener> or -> TcpSocket<Bound>. TcpListener would only represent the bound/listening state, so pre-bind-only APIs cannot be called on it.

---

### 10. LookupHost post-processing invariant (port must be applied to each resolved address)

**Location**: `/tmp/net_test_crate/src/net/socket_addr.rs:1-267`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: `LookupHost` yields addresses that must have the caller-specified port applied after resolution. This module encodes that as a required post-processing step (`lh.port()` + `set_port(p)` for each yielded address) before returning the iterator. The type system does not prevent accidentally returning raw resolved addresses without applying the port, nor does it make it explicit that the `LookupHost` iterator items are 'missing the final port' until this mapping occurs.

**Evidence**:

```rust
// Tests for this module
#[cfg(all(test, not(any(target_os = "emscripten", all(target_os = "wasi", target_env = "p1")))))]
mod tests;

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::sys::net::LookupHost;
use crate::{io, iter, option, slice, vec};

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// This trait is used for generic address resolution when constructing network
/// objects. By default it is implemented for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], [`SocketAddrV6`], <code>([IpAddr], [u16])</code>,
///    <code>([Ipv4Addr], [u16])</code>, <code>([Ipv6Addr], [u16])</code>:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * <code>(&[str], [u16])</code>: <code>&[str]</code> should be either a string representation
///    of an [`IpAddr`] address as expected by [`FromStr`] implementation or a host
///    name. [`u16`] is the port number.
///
///  * <code>&[str]</code>: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
///
/// This trait allows constructing network objects like [`TcpStream`] or
/// [`UdpSocket`] easily with values of various types for the bind/connection
/// address. It is needed because sometimes one type is more appropriate than
/// the other: for simple uses a string like `"localhost:12345"` is much nicer
/// than manual construction of the corresponding [`SocketAddr`], but sometimes
/// [`SocketAddr`] value is *the* main source of the address, and converting it to
/// some other type (e.g., a string) just for it to be converted back to
/// [`SocketAddr`] in constructor methods is pointless.
///
/// Addresses returned by the operating system that are not IP addresses are
/// silently ignored.
///
/// [`FromStr`]: crate::str::FromStr "std::str::FromStr"
/// [`TcpStream`]: crate::net::TcpStream "net::TcpStream"
/// [`to_socket_addrs`]: ToSocketAddrs::to_socket_addrs
/// [`UdpSocket`]: crate::net::UdpSocket "net::UdpSocket"
///
/// # Examples
///
/// Creating a [`SocketAddr`] iterator that yields one item:
///
/// ```
/// use std::net::{ToSocketAddrs, SocketAddr};
///
/// let addr = SocketAddr::from(([127, 0, 0, 1], 443));
/// let mut addrs_iter = addr.to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Creating a [`SocketAddr`] iterator from a hostname:
///
/// ```no_run
/// use std::net::{SocketAddr, ToSocketAddrs};
///
/// // assuming 'localhost' resolves to 127.0.0.1
/// let mut addrs_iter = "localhost:443".to_socket_addrs().unwrap();
/// assert_eq!(addrs_iter.next(), Some(SocketAddr::from(([127, 0, 0, 1], 443))));
/// assert!(addrs_iter.next().is_none());
///
/// // assuming 'foo' does not resolve
/// assert!("foo:443".to_socket_addrs().is_err());
/// ```
///
/// Creating a [`SocketAddr`] iterator that yields multiple items:
///
/// ```
/// use std::net::{SocketAddr, ToSocketAddrs};
///
/// let addr1 = SocketAddr::from(([0, 0, 0, 0], 80));
/// let addr2 = SocketAddr::from(([127, 0, 0, 1], 443));
/// let addrs = vec![addr1, addr2];
///
/// let mut addrs_iter = (&addrs[..]).to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr1), addrs_iter.next());
/// assert_eq!(Some(addr2), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Attempting to create a [`SocketAddr`] iterator from an improperly formatted
/// socket address `&str` (missing the port):
///
/// ```
/// use std::io;
/// use std::net::ToSocketAddrs;
///
/// let err = "127.0.0.1".to_socket_addrs().unwrap_err();
/// assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
/// ```
///
/// [`TcpStream::connect`] is an example of a function that utilizes
/// `ToSocketAddrs` as a trait bound on its parameter in order to accept
/// different types:
///
/// ```no_run
/// use std::net::{TcpStream, Ipv4Addr};
///
/// let stream = TcpStream::connect(("127.0.0.1", 443));
/// // or
/// let stream = TcpStream::connect("127.0.0.1:443");
/// // or
/// let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), 443));
/// ```
///
/// [`TcpStream::connect`]: crate::net::TcpStream::connect
#[stable(feature = "rust1", since = "1.0.0")]
pub trait ToSocketAddrs {
    /// Returned iterator over socket addresses which this type may correspond
    /// to.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Iter: Iterator<Item = SocketAddr>;

    /// Converts this object to an iterator of resolved [`SocketAddr`]s.
    ///
    /// The returned iterator might not actually yield any values depending on the
    /// outcome of any resolution performed.
    ///
    /// Note that this function may block the current thread while resolution is
    /// performed.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddrV6 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

fn resolve_socket_addr(lh: LookupHost) -> io::Result<vec::IntoIter<SocketAddr>> {
    let p = lh.port();
    let v: Vec<_> = lh
        .map(|mut a| {
            a.set_port(p);
            a
        })
        .collect();
    Ok(v.into_iter())
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (&str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            let addr = SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter());
        }
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            let addr = SocketAddrV6::new(addr, port, 0, 0);
            return Ok(vec![SocketAddr::V6(addr)].into_iter());
        }

        resolve_socket_addr((host, port).try_into()?)
    }
}

#[stable(feature = "string_u16_to_socket_addrs", since = "1.46.0")]
impl ToSocketAddrs for (String, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&*self.0, self.1).to_socket_addrs()
    }
}

// accepts strings like 'localhost:12345'
#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        resolve_socket_addr(self.try_into()?)
    }
}

#[stable(feature = "slice_to_socket_addrs", since = "1.8.0")]
impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

#[stable(feature = "string_to_socket_addrs", since = "1.16.0")]
impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&**self).to_socket_addrs()
    }
}

```

**Entity:** resolve_socket_addr(lh: LookupHost)

**States:** LookupHostAddressesWithoutDesiredPort, SocketAddrWithPortApplied

**Transitions:**
- LookupHostAddressesWithoutDesiredPort -> SocketAddrWithPortApplied via `resolve_socket_addr`: `let p = lh.port();` then `map(|mut a| { a.set_port(p); a })`

**Evidence:** function `resolve_socket_addr(lh: LookupHost)`: `let p = lh.port();` extracts the port from the lookup handle; `lh.map(|mut a| { a.set_port(p); a }).collect();` mutates each resolved address to apply `p`

**Implementation:** Wrap `LookupHost` in a newtype that encodes whether the port has been applied, e.g. `LookupHost<PortUnset>` yielding `SocketAddr<PortUnset>`-like wrapper, and provide a consuming method `with_port_applied(self) -> impl Iterator<Item=SocketAddr>`; only the port-applied iterator would be exposed to callers.

---

## Resource Lifecycle Invariants

### 8. IntoIncoming iterator protocol (Owned listener consumption and accept capability)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-35`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: IntoIncoming (not shown, but used) is an iterator that owns a TcpListener (as implied by self.listener in next()) and repeatedly calls accept(). There is an implicit lifecycle/protocol that the listener resource is held for the entire iteration and that accept success/failure is handled at runtime. As with Incoming, next() never returns None, so there is no typed termination state; consumers must decide when to stop (e.g., after errors). The type system enforces ownership of the listener but does not enforce higher-level lifecycle states like 'still accepting' vs 'permanently failed/closed'.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpStream, impl TcpStream (19 methods), impl Read for TcpStream (4 methods), impl Write for TcpStream (4 methods), impl Read for & TcpStream (4 methods), impl Write for & TcpStream (4 methods), impl AsInner < net_imp :: TcpStream > for TcpStream (1 methods), impl FromInner < net_imp :: TcpStream > for TcpStream (1 methods), impl IntoInner < net_imp :: TcpStream > for TcpStream (1 methods); struct TcpListener, impl TcpListener (12 methods), impl AsInner < net_imp :: TcpListener > for TcpListener (1 methods), impl FromInner < net_imp :: TcpListener > for TcpListener (1 methods), impl IntoInner < net_imp :: TcpListener > for TcpListener (1 methods); struct IntoIncoming

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

// ... (other code) ...

// `AsRawSocket`/`IntoRawSocket`/`FromRawSocket` on Windows.

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

#[stable(feature = "tcp_listener_incoming_fused_iterator", since = "1.64.0")]
impl FusedIterator for Incoming<'_> {}

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl Iterator for IntoIncoming {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl FusedIterator for IntoIncoming {}


```

**Entity:** IntoIncoming

**States:** Active (owns listener and can accept), Inactive (listener moved/closed/errored)

**Transitions:**
- Active -> Inactive via external event or internal close/error affecting listener, surfaced only as Err from accept()

**Evidence:** impl Iterator for IntoIncoming: next() { Some(self.listener.accept().map(|p| p.0)) } — repeated runtime accept(), no typed state change; impl FusedIterator for IntoIncoming {} — asserts fused semantics while next() never yields None, implying an implicit 'infinite stream unless you stop' contract

**Implementation:** Make the accept-loop represent terminal failure/closure explicitly (e.g., store an enum state inside and return None after a terminal condition), or encode an 'ActiveAccept' capability type produced from TcpListener that can be consumed/invalidated. If intended to be infinite, consider a dedicated infinite-iterator type that does not claim fused semantics unless it can actually become terminated.

---

### 3. TcpListener socket lifecycle (Bound/Listening -> Closed/Dropped) and shared-handle aliasing

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-334`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: A TcpListener represents ownership of an OS listener socket that is usable for accept()/incoming() only while the underlying OS resource is alive. The API also supports creating multiple independently-owned handles (try_clone) that alias the same underlying socket; socket options set via one handle affect the others. These are runtime/OS-level states (open vs closed; unique vs aliased) but the type system does not distinguish them. As a result, validity of accept()/incoming()/set_* calls depends on the underlying socket not being closed, and reasoning about option-setting has an implicit 'shared state' protocol across clones.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpStream, impl TcpStream (19 methods), impl Read for TcpStream (4 methods), impl Write for TcpStream (4 methods), impl Read for & TcpStream (4 methods), impl Write for & TcpStream (4 methods), impl AsInner < net_imp :: TcpStream > for TcpStream (1 methods), impl FromInner < net_imp :: TcpStream > for TcpStream (1 methods), impl IntoInner < net_imp :: TcpStream > for TcpStream (1 methods); struct Incoming, impl Iterator for Incoming < 'a > (1 methods), impl FusedIterator for Incoming < '_ > (0 methods), impl Iterator for IntoIncoming (1 methods), impl FusedIterator for IntoIncoming (0 methods); struct IntoIncoming

/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct TcpListener(net_imp::TcpListener);


// ... (other code) ...

    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`TcpListener::local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`:
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`. If that fails, create a
    /// TCP listener bound to `127.0.0.1:443`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, TcpListener};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 80)),
    ///     SocketAddr::from(([127, 0, 0, 1], 443)),
    /// ];
    /// let listener = TcpListener::bind(&addrs[..]).unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to a port assigned by the operating system
    /// at `127.0.0.1`.
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        super::each_addr(addr, net_imp::TcpListener::bind).map(TcpListener)
    }

    /// Returns the local socket address of this listener.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// assert_eq!(listener.local_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned [`TcpListener`] is a reference to the same socket that this
    /// object references. Both handles can be used to accept incoming
    /// connections and options set on one listener will affect the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// let listener_clone = listener.try_clone().unwrap();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<TcpListener> {
        self.0.duplicate().map(TcpListener)
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, the corresponding [`TcpStream`] and the
    /// remote peer's address will be returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    /// match listener.accept() {
    ///     Ok((_socket, addr)) => println!("new client: {addr:?}"),
    ///     Err(e) => println!("couldn't get client: {e:?}"),
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        // On WASM, `TcpStream` is uninhabited (as it's unsupported) and so
        // the `a` variable here is technically unused.
        #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
        self.0.accept().map(|(a, b)| (TcpStream(a), b))
    }

    /// Returns an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`TcpListener::accept`] in a loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{TcpListener, TcpStream};
    ///
    /// fn handle_connection(stream: TcpStream) {
    ///    //...
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:80")?;
    ///
    ///     for stream in listener.incoming() {
    ///         match stream {
    ///             Ok(stream) => {
    ///                 handle_connection(stream);
    ///             }
    ///             Err(e) => { /* connection failed */ }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }

    /// Turn this into an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`TcpListener::accept`] in a loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcplistener_into_incoming)]
    /// use std::net::{TcpListener, TcpStream};
    ///
    /// fn listen_on(port: u16) -> impl Iterator<Item = TcpStream> {
    ///     let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    ///     listener.into_incoming()
    ///         .filter_map(Result::ok) /* Ignore failed connections */
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     for stream in listen_on(80) {
    ///         /* handle the connection here */
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    #[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
    pub fn into_incoming(self) -> IntoIncoming {
        IntoIncoming { listener: self }
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.set_ttl(100).expect("could not set TTL");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`TcpListener::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.set_ttl(100).expect("could not set TTL");
    /// assert_eq!(listener.ttl().unwrap_or(0), 100);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    #[stable(feature = "net2_mutators", since = "1.9.0")]
    #[deprecated(since = "1.16.0", note = "this option can only be set before the socket is bound")]
    #[allow(missing_docs)]
    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.0.set_only_v6(only_v6)
    }

    #[stable(feature = "net2_mutators", since = "1.9.0")]
    #[deprecated(since = "1.16.0", note = "this option can only be set before the socket is bound")]
    #[allow(missing_docs)]
    pub fn only_v6(&self) -> io::Result<bool> {
        self.0.only_v6()
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// listener.take_error().expect("No error was expected");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in the `accept` operation becoming nonblocking,
    /// i.e., immediately returning from their calls. If the IO operation is
    /// successful, `Ok` is returned and no further action is required. If the
    /// IO operation could not be completed and needs to be retried, an error
    /// with kind [`io::ErrorKind::WouldBlock`] is returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    /// # Examples
    ///
    /// Bind a TCP listener to an address, listen for connections, and read
    /// bytes in nonblocking mode:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    /// listener.set_nonblocking(true).expect("Cannot set non-blocking");
    ///
    /// # fn wait_for_fd() { unimplemented!() }
    /// # fn handle_connection(stream: std::net::TcpStream) { unimplemented!() }
    /// for stream in listener.incoming() {
    ///     match stream {
    ///         Ok(s) => {
    ///             // do something with the TcpStream
    ///             handle_connection(s);
    ///         }
    ///         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    ///             // wait until network socket is ready, typically implemented
    ///             // via platform-specific APIs such as epoll or IOCP
    ///             wait_for_fd();
    ///             continue;
    ///         }
    ///         Err(e) => panic!("encountered IO error: {e}"),
    ///     }
    /// }
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

// ... (other code) ...

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl FusedIterator for IntoIncoming {}

impl AsInner<net_imp::TcpListener> for TcpListener {
    #[inline]
    fn as_inner(&self) -> &net_imp::TcpListener {
        &self.0
    }
}

impl FromInner<net_imp::TcpListener> for TcpListener {
    fn from_inner(inner: net_imp::TcpListener) -> TcpListener {
        TcpListener(inner)
    }
}

impl IntoInner<net_imp::TcpListener> for TcpListener {
    fn into_inner(self) -> net_imp::TcpListener {
        self.0
    }
}

```

**Entity:** TcpListener

**States:** Bound/Listening, Closed/Dropped, Aliased (cloned handle exists)

**Transitions:**
- Bound/Listening -> Closed/Dropped via Drop (implicit when TcpListener value is dropped)
- Bound/Listening -> Aliased via try_clone() (additional handle to same socket)
- Aliased -> Closed/Dropped when last handle is dropped (OS resource released when refcount reaches 0)

**Evidence:** pub struct TcpListener(net_imp::TcpListener); (wraps an OS-level listener resource); bind(): "The returned listener is ready for accepting connections." (implies post-bind 'listening' state); try_clone(): "Creates a new independently owned handle to the underlying socket" and "Both handles can be used ... and options set on one listener will affect the other." (explicit aliasing/shared-state protocol); accept(&self) -> io::Result<(TcpStream, SocketAddr)> (operation validity depends on the underlying socket still being open; errors otherwise are runtime/OS-enforced); into_incoming(self) consumes self: "`self` will be dropped if the result is not used" (consuming transition into an iterator owner; also highlights Drop-driven lifecycle)

**Implementation:** Model the OS socket as an internal shared capability (e.g., Arc<ListenerInner>) and make option-setting require an explicit capability token that denotes 'I can mutate listener options'. Alternatively, expose a separate owned type for the unique handle (e.g., UniqueTcpListener) that can be converted into a SharedTcpListener via try_clone(); only the unique form would allow certain mutating operations or would make the aliasing explicit in the type.

---

## State Machine Invariants

### 1. UdpSocket connectedness protocol (Unconnected / Connected peer)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-417`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: A UdpSocket can be in a 'connected to a specific peer' state or not. Some operations have different validity/semantics depending on whether the socket is connected (e.g., peer_addr() only succeeds when connected; other send/recv APIs in this module typically change behavior when connected vs unconnected). This connectedness is maintained by the OS and only reflected at runtime via io::Result errors; the type system does not distinguish a socket that has had connect() successfully called from one that has not.

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct UdpSocket(net_imp::UdpSocket);

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the socket. If none
    /// of the addresses succeed in creating a socket, the error returned from
    /// the last attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400`:
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");
    /// ```
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400`. If the socket cannot be
    /// bound to that address, create a UDP socket bound to `127.0.0.1:3401`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, UdpSocket};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 3400)),
    ///     SocketAddr::from(([127, 0, 0, 1], 3401)),
    /// ];
    /// let socket = UdpSocket::bind(&addrs[..]).expect("couldn't bind to address");
    /// ```
    ///
    /// Creates a UDP socket bound to a port assigned by the operating system
    /// at `127.0.0.1`.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    /// ```
    ///
    /// Note that `bind` declares the scope of your network connection.
    /// You can only receive datagrams from and send datagrams to
    /// participants in that view of the network.
    /// For instance, binding to a loopback address as in the example
    /// above will prevent you from sending datagrams to another device
    /// in your local network.
    ///
    /// In order to limit your view of the network the least, `bind` to
    /// [`Ipv4Addr::UNSPECIFIED`] or [`Ipv6Addr::UNSPECIFIED`].
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        super::each_addr(addr, net_imp::UdpSocket::bind).map(UdpSocket)
    }

    /// Receives a single datagram message on the socket. On success, returns the number
    /// of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
    ///                                         .expect("Didn't receive data");
    /// let filled_buf = &mut buf[..number_of_bytes];
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }

    /// Receives a single datagram message on the socket, without removing it from the
    /// queue. On success, returns the number of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying `recvfrom` system call.
    ///
    /// Do not use this function to implement busy waiting, instead use `libc::poll` to
    /// synchronize IO events on one or more sockets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = socket.peek_from(&mut buf)
    ///                                         .expect("Didn't receive data");
    /// let filled_buf = &mut buf[..number_of_bytes];
    /// ```
    #[stable(feature = "peek", since = "1.18.0")]
    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.peek_from(buf)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written. Note that the operating system may refuse
    /// buffers larger than 65507. However, partial writes are not possible
    /// until buffer sizes above `i32::MAX`.
    ///
    /// Address type can be any implementor of [`ToSocketAddrs`] trait. See its
    /// documentation for concrete examples.
    ///
    /// It is possible for `addr` to yield multiple addresses, but `send_to`
    /// will only send data to the first address yielded by `addr`.
    ///
    /// This will return an error when the IP version of the local socket
    /// does not match that returned from [`ToSocketAddrs`].
    ///
    /// See [Issue #34202] for more details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.send_to(&[0; 10], "127.0.0.1:4242").expect("couldn't send data");
    /// ```
    ///
    /// [Issue #34202]: https://github.com/rust-lang/rust/issues/34202
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        match addr.to_socket_addrs()?.next() {
            Some(addr) => self.0.send_to(buf, &addr),
            None => Err(io::const_error!(ErrorKind::InvalidInput, "no addresses to send data to")),
        }
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("192.168.0.1:41203").expect("couldn't connect to address");
    /// assert_eq!(socket.peer_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 41203)));
    /// ```
    ///
    /// If the socket isn't connected, it will return a [`NotConnected`] error.
    ///
    /// [`NotConnected`]: io::ErrorKind::NotConnected
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// assert_eq!(socket.peer_addr().unwrap_err().kind(),
    ///            std::io::ErrorKind::NotConnected);
    /// ```
    #[stable(feature = "udp_peer_addr", since = "1.40.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Returns the socket address that this socket was created from.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// assert_eq!(socket.local_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 34254)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `UdpSocket` is a reference to the same socket that this
    /// object references. Both handles will read and write the same port, and
    /// options set on one socket will be propagated to the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let socket_clone = socket.try_clone().expect("couldn't clone the socket");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<UdpSocket> {
        self.0.duplicate().map(UdpSocket)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`read`]: io::Read::read
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_read_timeout(None).expect("set_read_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    /// let result = socket.set_read_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`write`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`write`]: io::Write::write
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_write_timeout(None).expect("set_write_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    /// let result = socket.set_write_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`read`] calls will block indefinitely.
    ///
    /// [`read`]: io::Read::read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_read_timeout(None).expect("set_read_timeout call failed");
    /// assert_eq!(socket.read_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`write`] calls will block indefinitely.
    ///
    /// [`write`]: io::Write::write
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_write_timeout(None).expect("set_write_timeout call failed");
    /// assert_eq!(socket.write_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.0.set_broadcast(broadcast)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_broadcast`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// assert_eq!(socket.broadcast().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.broadcast()
    }

    /// Sets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// If enabled, multicast packets will be looped back to the local socket.
    /// Note that this might not have any effect on IPv6 sockets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> io::Result<()> {
        self.0.set_multicast_loop_v4(multicast_loop_v4)
    }

    /// Gets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_multicast_loop_v4`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
    /// assert_eq!(socket.multicast_loop_v4().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.0.multicast_loop_v4()
    }

    /// Sets the value of the `IP_MULTICAST_TTL` option for this socket.
    ///
    /// Indicates the time-to-live value of outgoing multicast packets for
    /// this socket. The default value is 1 which means that multicast packets
    /// don't leave the local network unless explicitly requested.
  
// ... (truncated) ...
```

**Entity:** UdpSocket

**States:** Bound+Unconnected, Bound+Connected

**Transitions:**
- Bound+Unconnected -> Bound+Connected via connect(...) (mentioned in docs/examples)

**Evidence:** struct UdpSocket(net_imp::UdpSocket): no type-level marker for connectedness; method peer_addr(&self) -> io::Result<SocketAddr>: doc says it returns ErrorKind::NotConnected if the socket isn't connected; peer_addr() docs/example call socket.connect("192.168.0.1:41203").expect(...) before peer_addr(); peer_addr() docs: "If the socket isn't connected, it will return a NotConnected error."

**Implementation:** Represent connectedness as a type parameter: UdpSocket<S> where S is Unconnected or Connected. bind(...) -> UdpSocket<Unconnected>; connect(self, addr) -> io::Result<UdpSocket<Connected>>; peer_addr(&self) only on UdpSocket<Connected>. (Keep current std API by offering an 'erased' wrapper or methods returning enum EitherConnected.)

---

### 5. TcpStream lifecycle/state machine (Connected/Open -> Half-shutdown -> Closed)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-439`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TcpStream methods implicitly assume the underlying socket is in a connected/open state. Calling shutdown() transitions the OS socket into a state where some or all future I/O is invalid or behaves differently. The API exposes shutdown(Shutdown::{Read,Write,Both}) but does not reflect the resulting half-closed/full-closed state in the type system, so operations like peek()/read/write/peer_addr/etc. remain callable and only fail at runtime (and even the meaning of 'shut down twice' varies by platform).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpListener, impl TcpListener (12 methods), impl AsInner < net_imp :: TcpListener > for TcpListener (1 methods), impl FromInner < net_imp :: TcpListener > for TcpListener (1 methods), impl IntoInner < net_imp :: TcpListener > for TcpListener (1 methods); struct Incoming, impl Iterator for Incoming < 'a > (1 methods), impl FusedIterator for Incoming < '_ > (0 methods), impl Iterator for IntoIncoming (1 methods), impl FusedIterator for IntoIncoming (0 methods); struct IntoIncoming

/// } // the stream is closed here
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct TcpStream(net_imp::TcpStream);


// ... (other code) ...

    listener: TcpListener,
}

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Open a TCP connection to `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    ///
    /// Open a TCP connection to `127.0.0.1:8080`. If the connection fails, open
    /// a TCP connection to `127.0.0.1:8081`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, TcpStream};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 8080)),
    ///     SocketAddr::from(([127, 0, 0, 1], 8081)),
    /// ];
    /// if let Ok(stream) = TcpStream::connect(&addrs[..]) {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        super::each_addr(addr, net_imp::TcpStream::connect).map(TcpStream)
    }

    /// Opens a TCP connection to a remote host with a timeout.
    ///
    /// Unlike `connect`, `connect_timeout` takes a single [`SocketAddr`] since
    /// timeout must be applied to individual addresses.
    ///
    /// It is an error to pass a zero `Duration` to this function.
    ///
    /// Unlike other methods on `TcpStream`, this does not correspond to a
    /// single system call. It instead calls `connect` in nonblocking mode and
    /// then uses an OS-specific mechanism to await the completion of the
    /// connection request.
    #[stable(feature = "tcpstream_connect_timeout", since = "1.21.0")]
    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        net_imp::TcpStream::connect_timeout(addr, timeout).map(TcpStream)
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// assert_eq!(stream.peer_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Returns the socket address of the local half of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{IpAddr, Ipv4Addr, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// assert_eq!(stream.local_addr().unwrap().ip(),
    ///            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value (see the
    /// documentation of [`Shutdown`]).
    ///
    /// # Platform-specific behavior
    ///
    /// Calling this function multiple times may result in different behavior,
    /// depending on the operating system. On Linux, the second call will
    /// return `Ok(())`, but on macOS, it will return `ErrorKind::NotConnected`.
    /// This may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Shutdown, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `TcpStream` is a reference to the same stream that this
    /// object references. Both handles will read and write the same stream of
    /// data, and options set on one stream will be propagated to the other
    /// stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// let stream_clone = stream.try_clone().expect("clone failed...");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<TcpStream> {
        self.0.duplicate().map(TcpStream)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`read`]: Read::read
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_read_timeout(None).expect("set_read_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    /// let result = stream.set_read_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`write`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`write`]: Write::write
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_write_timeout(None).expect("set_write_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    /// let result = stream.set_write_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`read`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`read`]: Read::read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_read_timeout(None).expect("set_read_timeout call failed");
    /// assert_eq!(stream.read_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`write`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`write`]: Write::write
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_write_timeout(None).expect("set_write_timeout call failed");
    /// assert_eq!(stream.write_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying `recv` system call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8000")
    ///                        .expect("Couldn't connect to the server...");
    /// let mut buf = [0; 10];
    /// let len = stream.peek(&mut buf).expect("peek failed");
    /// ```
    #[stable(feature = "peek", since = "1.18.0")]
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }

    /// Sets the value of the `SO_LINGER` option on this socket.
    ///
    /// This value controls how the socket is closed when data remains
    /// to be sent. If `SO_LINGER` is set, the socket will remain open
    /// for the specified duration as the system attempts to send pending data.
    /// Otherwise, the system may close the socket immediately, or wait for a
    /// default timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcp_linger)]
    ///
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_linger(Some(Duration::from_secs(0))).expect("set_linger call failed");
    /// ```
    #[unstable(feature = "tcp_linger", issue = "88494")]
    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        self.0.set_linger(linger)
    }

    /// Gets the value of the `SO_LINGER` option on this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_linger`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcp_linger)]
    ///
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_linger(Some(Duration::from_secs(0))).expect("set_linger call failed");
    /// assert_eq!(stream.linger().unwrap(), Some(Duration::from_secs(0)));
    /// ```
    #[unstable(feature = "tcp_linger", issue = "88494")]
    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.0.linger()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that
    /// segments are always sent as soon as possible, even if there is only a
    /// small amount of data. When not set, data is buffered until there is a
    /// sufficient amount to send out, thereby avoiding the frequent sending of
    /// small packets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.0.set_nodelay(nodelay)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_nodelay`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// assert_eq!(stream.nodelay().unwrap_or(false), true);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.nodelay()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// assert_eq!(stream.ttl().unwrap_or(0), 100
// ... (truncated) ...
```

**Entity:** TcpStream

**States:** Connected/Open, ReadShutdown, WriteShutdown, BothShutdown (Closed/NotConnected)

**Transitions:**
- Connected/Open -> ReadShutdown via shutdown(Shutdown::Read)
- Connected/Open -> WriteShutdown via shutdown(Shutdown::Write)
- Connected/Open -> BothShutdown (Closed/NotConnected) via shutdown(Shutdown::Both)
- ReadShutdown -> BothShutdown via shutdown(Shutdown::Write) or shutdown(Shutdown::Both)
- WriteShutdown -> BothShutdown via shutdown(Shutdown::Read) or shutdown(Shutdown::Both)

**Evidence:** pub struct TcpStream(net_imp::TcpStream); — a single type represents all runtime socket states (open, half-closed, closed); method shutdown(&self, how: Shutdown) -> io::Result<()> — explicit state transition API without a type-level state change; shutdown() docs: "Calling this function multiple times may result in different behavior" and mentions macOS returning ErrorKind::NotConnected on the second call — indicates an implicit 'already shut down / not connected' state tracked only by the OS/runtime; method peek(&self, buf: &mut [u8]) -> io::Result<usize> — only meaningful while the read half is open, but always callable; methods peer_addr()/local_addr() return io::Result — can fail at runtime if the socket becomes NotConnected/closed, but the type does not prevent calling them post-shutdown

**Implementation:** Introduce a generic TcpStream<S> with zero-sized states like Open, ReadClosed, WriteClosed, Closed. Make shutdown(self, Shutdown::Read) -> TcpStream<ReadClosed>, shutdown(self, Shutdown::Write) -> TcpStream<WriteClosed>, shutdown(self, Shutdown::Both) -> TcpStream<Closed>. Gate methods: peek/read only on states where read is open; write/flush only where write is open; peer_addr/local_addr on non-Closed. (In std this is likely too breaking, but it is the compile-time enforceable invariant.)

---

## Precondition Invariants

### 2. Timeout option validity (NonZero duration vs None)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-417`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Setting read/write timeouts has an implicit validity rule: Some(Duration::ZERO) is invalid and must be rejected at runtime, while None means 'block indefinitely'. This is enforced by returning InvalidInput errors, not by the type system; callers can still construct the invalid value (Duration::new(0,0)) and only find out at runtime.

**Evidence**:

```rust
/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct UdpSocket(net_imp::UdpSocket);

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the socket. If none
    /// of the addresses succeed in creating a socket, the error returned from
    /// the last attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400`:
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");
    /// ```
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400`. If the socket cannot be
    /// bound to that address, create a UDP socket bound to `127.0.0.1:3401`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, UdpSocket};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 3400)),
    ///     SocketAddr::from(([127, 0, 0, 1], 3401)),
    /// ];
    /// let socket = UdpSocket::bind(&addrs[..]).expect("couldn't bind to address");
    /// ```
    ///
    /// Creates a UDP socket bound to a port assigned by the operating system
    /// at `127.0.0.1`.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    /// ```
    ///
    /// Note that `bind` declares the scope of your network connection.
    /// You can only receive datagrams from and send datagrams to
    /// participants in that view of the network.
    /// For instance, binding to a loopback address as in the example
    /// above will prevent you from sending datagrams to another device
    /// in your local network.
    ///
    /// In order to limit your view of the network the least, `bind` to
    /// [`Ipv4Addr::UNSPECIFIED`] or [`Ipv6Addr::UNSPECIFIED`].
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        super::each_addr(addr, net_imp::UdpSocket::bind).map(UdpSocket)
    }

    /// Receives a single datagram message on the socket. On success, returns the number
    /// of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
    ///                                         .expect("Didn't receive data");
    /// let filled_buf = &mut buf[..number_of_bytes];
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }

    /// Receives a single datagram message on the socket, without removing it from the
    /// queue. On success, returns the number of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying `recvfrom` system call.
    ///
    /// Do not use this function to implement busy waiting, instead use `libc::poll` to
    /// synchronize IO events on one or more sockets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = socket.peek_from(&mut buf)
    ///                                         .expect("Didn't receive data");
    /// let filled_buf = &mut buf[..number_of_bytes];
    /// ```
    #[stable(feature = "peek", since = "1.18.0")]
    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.peek_from(buf)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written. Note that the operating system may refuse
    /// buffers larger than 65507. However, partial writes are not possible
    /// until buffer sizes above `i32::MAX`.
    ///
    /// Address type can be any implementor of [`ToSocketAddrs`] trait. See its
    /// documentation for concrete examples.
    ///
    /// It is possible for `addr` to yield multiple addresses, but `send_to`
    /// will only send data to the first address yielded by `addr`.
    ///
    /// This will return an error when the IP version of the local socket
    /// does not match that returned from [`ToSocketAddrs`].
    ///
    /// See [Issue #34202] for more details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.send_to(&[0; 10], "127.0.0.1:4242").expect("couldn't send data");
    /// ```
    ///
    /// [Issue #34202]: https://github.com/rust-lang/rust/issues/34202
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        match addr.to_socket_addrs()?.next() {
            Some(addr) => self.0.send_to(buf, &addr),
            None => Err(io::const_error!(ErrorKind::InvalidInput, "no addresses to send data to")),
        }
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("192.168.0.1:41203").expect("couldn't connect to address");
    /// assert_eq!(socket.peer_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 41203)));
    /// ```
    ///
    /// If the socket isn't connected, it will return a [`NotConnected`] error.
    ///
    /// [`NotConnected`]: io::ErrorKind::NotConnected
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// assert_eq!(socket.peer_addr().unwrap_err().kind(),
    ///            std::io::ErrorKind::NotConnected);
    /// ```
    #[stable(feature = "udp_peer_addr", since = "1.40.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Returns the socket address that this socket was created from.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// assert_eq!(socket.local_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 34254)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `UdpSocket` is a reference to the same socket that this
    /// object references. Both handles will read and write the same port, and
    /// options set on one socket will be propagated to the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let socket_clone = socket.try_clone().expect("couldn't clone the socket");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<UdpSocket> {
        self.0.duplicate().map(UdpSocket)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`read`]: io::Read::read
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_read_timeout(None).expect("set_read_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    /// let result = socket.set_read_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`write`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`write`]: io::Write::write
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_write_timeout(None).expect("set_write_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    /// let result = socket.set_write_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`read`] calls will block indefinitely.
    ///
    /// [`read`]: io::Read::read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_read_timeout(None).expect("set_read_timeout call failed");
    /// assert_eq!(socket.read_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`write`] calls will block indefinitely.
    ///
    /// [`write`]: io::Write::write
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_write_timeout(None).expect("set_write_timeout call failed");
    /// assert_eq!(socket.write_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.0.set_broadcast(broadcast)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_broadcast`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// assert_eq!(socket.broadcast().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn broadcast(&self) -> io::Result<bool> {
        self.0.broadcast()
    }

    /// Sets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// If enabled, multicast packets will be looped back to the local socket.
    /// Note that this might not have any effect on IPv6 sockets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> io::Result<()> {
        self.0.set_multicast_loop_v4(multicast_loop_v4)
    }

    /// Gets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_multicast_loop_v4`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v4(false).expect("set_multicast_loop_v4 call failed");
    /// assert_eq!(socket.multicast_loop_v4().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.0.multicast_loop_v4()
    }

    /// Sets the value of the `IP_MULTICAST_TTL` option for this socket.
    ///
    /// Indicates the time-to-live value of outgoing multicast packets for
    /// this socket. The default value is 1 which means that multicast packets
    /// don't leave the local network unless explicitly requested.
  
// ... (truncated) ...
```

**Entity:** UdpSocket

**States:** Blocking (timeout=None), Timed (timeout=Some(nonzero))

**Transitions:**
- Any -> Blocking via set_read_timeout(None)
- Any -> Timed via set_read_timeout(Some(nonzero Duration))
- Any -> Blocking via set_write_timeout(None)
- Any -> Timed via set_write_timeout(Some(nonzero Duration))

**Evidence:** set_read_timeout(&self, dur: Option<Duration>) docs: "An Err is returned if the zero Duration is passed"; set_write_timeout(&self, dur: Option<Duration>) docs: "An Err is returned if the zero Duration is passed"; examples show Duration::new(0, 0) leading to io::ErrorKind::InvalidInput

**Implementation:** Introduce a NonZeroDuration (or Timeout) newtype that can only be constructed from nonzero durations (e.g., TryFrom<Duration>). Then accept Option<NonZeroDuration> in setters (or a Timeout enum { Blocking, After(NonZeroDuration) }). Keep fallible conversion at the boundary, eliminating the invalid-state value inside the API.

---

### 6. TcpStream duration validity precondition (NonZero timeout durations)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-439`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Several APIs require that a Duration is non-zero; passing a zero Duration is documented to be an error and is only rejected at runtime via io::Result. This is a latent value-level invariant that could be encoded in the type system to prevent constructing/using invalid timeout values.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpListener, impl TcpListener (12 methods), impl AsInner < net_imp :: TcpListener > for TcpListener (1 methods), impl FromInner < net_imp :: TcpListener > for TcpListener (1 methods), impl IntoInner < net_imp :: TcpListener > for TcpListener (1 methods); struct Incoming, impl Iterator for Incoming < 'a > (1 methods), impl FusedIterator for Incoming < '_ > (0 methods), impl Iterator for IntoIncoming (1 methods), impl FusedIterator for IntoIncoming (0 methods); struct IntoIncoming

/// } // the stream is closed here
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct TcpStream(net_imp::TcpStream);


// ... (other code) ...

    listener: TcpListener,
}

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Open a TCP connection to `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    ///
    /// Open a TCP connection to `127.0.0.1:8080`. If the connection fails, open
    /// a TCP connection to `127.0.0.1:8081`:
    ///
    /// ```no_run
    /// use std::net::{SocketAddr, TcpStream};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 8080)),
    ///     SocketAddr::from(([127, 0, 0, 1], 8081)),
    /// ];
    /// if let Ok(stream) = TcpStream::connect(&addrs[..]) {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        super::each_addr(addr, net_imp::TcpStream::connect).map(TcpStream)
    }

    /// Opens a TCP connection to a remote host with a timeout.
    ///
    /// Unlike `connect`, `connect_timeout` takes a single [`SocketAddr`] since
    /// timeout must be applied to individual addresses.
    ///
    /// It is an error to pass a zero `Duration` to this function.
    ///
    /// Unlike other methods on `TcpStream`, this does not correspond to a
    /// single system call. It instead calls `connect` in nonblocking mode and
    /// then uses an OS-specific mechanism to await the completion of the
    /// connection request.
    #[stable(feature = "tcpstream_connect_timeout", since = "1.21.0")]
    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        net_imp::TcpStream::connect_timeout(addr, timeout).map(TcpStream)
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// assert_eq!(stream.peer_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Returns the socket address of the local half of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{IpAddr, Ipv4Addr, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// assert_eq!(stream.local_addr().unwrap().ip(),
    ///            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value (see the
    /// documentation of [`Shutdown`]).
    ///
    /// # Platform-specific behavior
    ///
    /// Calling this function multiple times may result in different behavior,
    /// depending on the operating system. On Linux, the second call will
    /// return `Ok(())`, but on macOS, it will return `ErrorKind::NotConnected`.
    /// This may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{Shutdown, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `TcpStream` is a reference to the same stream that this
    /// object references. Both handles will read and write the same stream of
    /// data, and options set on one stream will be propagated to the other
    /// stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// let stream_clone = stream.try_clone().expect("clone failed...");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_clone(&self) -> io::Result<TcpStream> {
        self.0.duplicate().map(TcpStream)
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`read`]: Read::read
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_read_timeout(None).expect("set_read_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    /// let result = stream.set_read_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    /// Sets the write timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`write`] calls will block
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a write times out
    /// as a result of setting this option. For example Unix typically returns
    /// an error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`write`]: Write::write
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_write_timeout(None).expect("set_write_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    /// let result = stream.set_write_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    /// Returns the read timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`read`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`read`]: Read::read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_read_timeout(None).expect("set_read_timeout call failed");
    /// assert_eq!(stream.read_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    /// Returns the write timeout of this socket.
    ///
    /// If the timeout is [`None`], then [`write`] calls will block indefinitely.
    ///
    /// # Platform-specific behavior
    ///
    /// Some platforms do not provide access to the current timeout.
    ///
    /// [`write`]: Write::write
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_write_timeout(None).expect("set_write_timeout call failed");
    /// assert_eq!(stream.write_timeout().unwrap(), None);
    /// ```
    #[stable(feature = "socket_timeout", since = "1.4.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    ///
    /// Successive calls return the same data. This is accomplished by passing
    /// `MSG_PEEK` as a flag to the underlying `recv` system call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8000")
    ///                        .expect("Couldn't connect to the server...");
    /// let mut buf = [0; 10];
    /// let len = stream.peek(&mut buf).expect("peek failed");
    /// ```
    #[stable(feature = "peek", since = "1.18.0")]
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }

    /// Sets the value of the `SO_LINGER` option on this socket.
    ///
    /// This value controls how the socket is closed when data remains
    /// to be sent. If `SO_LINGER` is set, the socket will remain open
    /// for the specified duration as the system attempts to send pending data.
    /// Otherwise, the system may close the socket immediately, or wait for a
    /// default timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcp_linger)]
    ///
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_linger(Some(Duration::from_secs(0))).expect("set_linger call failed");
    /// ```
    #[unstable(feature = "tcp_linger", issue = "88494")]
    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        self.0.set_linger(linger)
    }

    /// Gets the value of the `SO_LINGER` option on this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_linger`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(tcp_linger)]
    ///
    /// use std::net::TcpStream;
    /// use std::time::Duration;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_linger(Some(Duration::from_secs(0))).expect("set_linger call failed");
    /// assert_eq!(stream.linger().unwrap(), Some(Duration::from_secs(0)));
    /// ```
    #[unstable(feature = "tcp_linger", issue = "88494")]
    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.0.linger()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that
    /// segments are always sent as soon as possible, even if there is only a
    /// small amount of data. When not set, data is buffered until there is a
    /// sufficient amount to send out, thereby avoiding the frequent sending of
    /// small packets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.0.set_nodelay(nodelay)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_nodelay`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// assert_eq!(stream.nodelay().unwrap_or(false), true);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.nodelay()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// assert_eq!(stream.ttl().unwrap_or(0), 100
// ... (truncated) ...
```

**Entity:** TcpStream

**States:** ValidDuration (non-zero), InvalidDuration (zero)

**Transitions:**
- InvalidDuration -> (runtime error) via connect_timeout(addr, timeout)
- InvalidDuration -> (runtime error) via set_read_timeout(Some(dur))
- InvalidDuration -> (runtime error) via set_write_timeout(Some(dur))
- ValidDuration -> (success) via connect_timeout / set_*_timeout

**Evidence:** connect_timeout(addr: &SocketAddr, timeout: Duration) docs: "It is an error to pass a zero Duration to this function."; set_read_timeout(&self, dur: Option<Duration>) docs: "An Err is returned if the zero Duration is passed to this method." plus example uses Duration::new(0, 0) and asserts ErrorKind::InvalidInput; set_write_timeout(&self, dur: Option<Duration>) docs: same "An Err is returned if the zero Duration is passed" plus example

**Implementation:** Use a NonZeroDuration newtype (e.g., struct NonZeroDuration(Duration); impl TryFrom<Duration> for NonZeroDuration) and change APIs to accept Option<NonZeroDuration> (or a custom Timeout enum { Infinite, Finite(NonZeroDuration) }). This would make passing Duration::ZERO unrepresentable without an explicit fallible conversion.

---

### 13. Address-equivalence semantics protocol (Normal equality vs ZoneId-ignored equality)

**Location**: `/tmp/net_test_crate/src/net/test.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The code embeds an implicit notion of two different equality semantics for socket addresses: normal equality for non-V6 (and non-zone-sensitive comparisons), and a special V6 comparison that ignores the zone identifier but still requires segments/flowinfo/port equality. This semantic choice is not represented in types: both inputs are plain SocketAddr references, and callers must remember to choose this function when zone-id differences are expected/acceptable. The type system cannot distinguish 'zone-insensitive address' from a normal SocketAddr.

**Evidence**:

```rust
#![allow(warnings)] // not used on emscripten

use crate::env;
use crate::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use crate::sync::atomic::{AtomicUsize, Ordering};

static PORT: AtomicUsize = AtomicUsize::new(0);
const BASE_PORT: u16 = 19600;

pub fn next_test_ip4() -> SocketAddr {
    let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT;
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

pub fn next_test_ip6() -> SocketAddr {
    let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT;
    SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0))
}

pub fn sa4(a: Ipv4Addr, p: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(a, p))
}

pub fn sa6(a: Ipv6Addr, p: u16) -> SocketAddr {
    SocketAddr::V6(SocketAddrV6::new(a, p, 0, 0))
}

pub fn tsa<A: ToSocketAddrs>(a: A) -> Result<Vec<SocketAddr>, String> {
    match a.to_socket_addrs() {
        Ok(a) => Ok(a.collect()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr) -> bool {
    match (a, b) {
        (SocketAddr::V6(a), SocketAddr::V6(b)) => {
            a.ip().segments() == b.ip().segments()
                && a.flowinfo() == b.flowinfo()
                && a.port() == b.port()
        }
        _ => a == b,
    }
}

```

**Entity:** compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr) -> bool

**States:** NormalEqSemantics, ZoneIdIgnoredEqSemantics(V6-only)

**Transitions:**
- NormalEqSemantics -> ZoneIdIgnoredEqSemantics(V6-only) via compare_ignore_zoneid()

**Evidence:** fn compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr) -> bool (named to indicate special comparison semantics); match (a, b) { (SocketAddr::V6(a), SocketAddr::V6(b)) => { ... } _ => a == b } (V6 path uses custom equality; others fall back to ==); V6 comparison uses a.ip().segments() == b.ip().segments() and a.flowinfo() == b.flowinfo() and a.port() == b.port() (notably does not compare any zone-id-related field)

**Implementation:** Define a wrapper newtype that encodes the intended equivalence relation, e.g. struct ZoneIdIgnored<'a>(&'a SocketAddrV6) (or an owned variant) with Eq/Hash implemented by segments/flowinfo/port only. Expose constructors that only accept V6 addresses, making misuse (passing V4 or expecting full equality) a compile-time/type-level mismatch.

---

## Protocol Invariants

### 11. Address-resolution iteration protocol (Resolve -> Try each SocketAddr -> Final error)

**Location**: `/tmp/net_test_crate/src/net/mod.rs:1-91`

**Confidence**: medium

**Suggested Pattern**: session_type

**Description**: `each_addr` encodes a multi-step protocol: first resolve `A: ToSocketAddrs` into a list of `SocketAddr`s; if resolution fails, it still invokes `f` once with `Err(e)`. If resolution succeeds, it invokes `f` on each resolved address (as `Ok(&SocketAddr)`) until one attempt returns `Ok(T)`, otherwise it returns the last error from `f`, or a synthesized `InvalidInput` if the resolved list is empty. This relies on a convention that `f` must correctly handle being called with `Err(_)` as well as potentially multiple `Ok(_)` candidates, and that it should treat `Ok(&SocketAddr)` as a single attempt. None of these call-order / call-count expectations are expressible in the type system with the current `FnMut(io::Result<&SocketAddr>)` signature.

**Evidence**:

```rust
// Note: Other parts of this module contain: enum Shutdown

//! Networking primitives for TCP/UDP communication.
//!
//! This module provides networking functionality for the Transmission Control and User
//! Datagram Protocols, as well as types for IP and socket addresses.
//!
//! # Organization
//!
//! * [`TcpListener`] and [`TcpStream`] provide functionality for communication over TCP
//! * [`UdpSocket`] provides functionality for communication over UDP
//! * [`IpAddr`] represents IP addresses of either IPv4 or IPv6; [`Ipv4Addr`] and
//!   [`Ipv6Addr`] are respectively IPv4 and IPv6 addresses
//! * [`SocketAddr`] represents socket addresses of either IPv4 or IPv6; [`SocketAddrV4`]
//!   and [`SocketAddrV6`] are respectively IPv4 and IPv6 socket addresses
//! * [`ToSocketAddrs`] is a trait that is used for generic address resolution when interacting
//!   with networking objects like [`TcpListener`], [`TcpStream`] or [`UdpSocket`]
//! * Other types are return or parameter types for various methods in this module
//!
//! Rust disables inheritance of socket objects to child processes by default when possible.  For
//! example, through the use of the `CLOEXEC` flag in UNIX systems or the `HANDLE_FLAG_INHERIT`
//! flag on Windows.

#![stable(feature = "rust1", since = "1.0.0")]

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::net::AddrParseError;

#[stable(feature = "rust1", since = "1.0.0")]
pub use self::ip_addr::{IpAddr, Ipv4Addr, Ipv6Addr, Ipv6MulticastScope};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::socket_addr::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
pub use self::tcp::IntoIncoming;
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::tcp::{Incoming, TcpListener, TcpStream};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::udp::UdpSocket;
use crate::io::{self, ErrorKind};

mod ip_addr;
mod socket_addr;
mod tcp;
#[cfg(test)]
pub(crate) mod test;
mod udp;

/// Possible values which can be passed to the [`TcpStream::shutdown`] method.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub enum Shutdown {
    /// The reading portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [reads] will return <code>[Ok]\(0)</code>.
    ///
    /// [reads]: crate::io::Read "io::Read"
    #[stable(feature = "rust1", since = "1.0.0")]
    Read,
    /// The writing portion of the [`TcpStream`] should be shut down.
    ///
    /// All currently blocked and future [writes] will return an error.
    ///
    /// [writes]: crate::io::Write "io::Write"
    #[stable(feature = "rust1", since = "1.0.0")]
    Write,
    /// Both the reading and the writing portions of the [`TcpStream`] should be shut down.
    ///
    /// See [`Shutdown::Read`] and [`Shutdown::Write`] for more information.
    #[stable(feature = "rust1", since = "1.0.0")]
    Both,
}

fn each_addr<A: ToSocketAddrs, F, T>(addr: A, mut f: F) -> io::Result<T>
where
    F: FnMut(io::Result<&SocketAddr>) -> io::Result<T>,
{
    let addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => return f(Err(e)),
    };
    let mut last_err = None;
    for addr in addrs {
        match f(Ok(&addr)) {
            Ok(l) => return Ok(l),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        io::const_error!(ErrorKind::InvalidInput, "could not resolve to any addresses")
    }))
}

```

**Entity:** each_addr<A, F, T>(addr: A, f: F)

**States:** Unresolved(ToSocketAddrs), Resolved(iterable SocketAddr list), Attempting(candidate SocketAddr), Succeeded(T), Failed(io::Error)

**Transitions:**
- Unresolved(ToSocketAddrs) -> Failed(io::Error) via addr.to_socket_addrs() Err(e) then f(Err(e))
- Unresolved(ToSocketAddrs) -> Resolved(iterable SocketAddr list) via addr.to_socket_addrs() Ok(addrs)
- Resolved -> Attempting(candidate SocketAddr) via for addr in addrs { f(Ok(&addr)) }
- Attempting -> Succeeded(T) via f(Ok(&addr)) returning Ok(T)
- Attempting -> Failed(io::Error) via f(Ok(&addr)) returning Err(e) (recorded as last_err)
- Resolved(with zero addrs) -> Failed(io::Error) via synthesized InvalidInput("could not resolve to any addresses")
- Resolved(all attempts failed) -> Failed(io::Error) via returning last_err

**Evidence:** fn each_addr<A: ToSocketAddrs, F, T>(addr: A, mut f: F) where F: FnMut(io::Result<&SocketAddr>) -> io::Result<T>; let addrs = match addr.to_socket_addrs() { Ok(addrs) => addrs, Err(e) => return f(Err(e)), }; (calls f with Err on resolution failure); for addr in addrs { match f(Ok(&addr)) { Ok(l) => return Ok(l), Err(e) => last_err = Some(e), } } (calls f repeatedly with Ok for candidates until success); Err(last_err.unwrap_or_else(|| io::const_error!(ErrorKind::InvalidInput, "could not resolve to any addresses"))) (empty-resolution and all-failed behavior)

**Implementation:** Replace `F: FnMut(io::Result<&SocketAddr>) -> io::Result<T>` with a small protocol trait/state machine, e.g. `trait AddrAttempt { type Output; fn on_resolve_error(self, e: io::Error) -> io::Result<Self::Output>; fn try_addr(&mut self, addr: &SocketAddr) -> io::Result<Option<Self::Output>>; fn on_no_addrs(self) -> io::Result<Self::Output>; fn on_all_failed(self, last: io::Error) -> io::Result<Self::Output>; }`, so the type system enforces distinct handling for resolve-error vs per-address attempts vs empty-addresses. Alternatively split into two functions: `resolve_addrs(A) -> io::Result<impl Iterator<Item=SocketAddr>>` and `try_each_addr(iter, f: impl FnMut(&SocketAddr)->io::Result<T>)` to avoid the `io::Result<&SocketAddr>` conflation.

---

### 12. Global test-port allocation protocol (Fresh/Unused -> Allocated)

**Location**: `/tmp/net_test_crate/src/net/test.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The module maintains a global monotonically-increasing port counter and relies on the implicit protocol that each call to next_test_ip4()/next_test_ip6() yields a 'fresh' port intended to avoid collisions in tests. The uniqueness/freshness property is only probabilistic/temporal: it depends on all test code consistently using these helpers (and not reusing ports manually), and on the counter not wrapping when cast to u16. None of these constraints are enforced by the type system; callers just receive a plain SocketAddr indistinguishable from any other address.

**Evidence**:

```rust
#![allow(warnings)] // not used on emscripten

use crate::env;
use crate::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use crate::sync::atomic::{AtomicUsize, Ordering};

static PORT: AtomicUsize = AtomicUsize::new(0);
const BASE_PORT: u16 = 19600;

pub fn next_test_ip4() -> SocketAddr {
    let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT;
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

pub fn next_test_ip6() -> SocketAddr {
    let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT;
    SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0))
}

pub fn sa4(a: Ipv4Addr, p: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(a, p))
}

pub fn sa6(a: Ipv6Addr, p: u16) -> SocketAddr {
    SocketAddr::V6(SocketAddrV6::new(a, p, 0, 0))
}

pub fn tsa<A: ToSocketAddrs>(a: A) -> Result<Vec<SocketAddr>, String> {
    match a.to_socket_addrs() {
        Ok(a) => Ok(a.collect()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr) -> bool {
    match (a, b) {
        (SocketAddr::V6(a), SocketAddr::V6(b)) => {
            a.ip().segments() == b.ip().segments()
                && a.flowinfo() == b.flowinfo()
                && a.port() == b.port()
        }
        _ => a == b,
    }
}

```

**Entity:** PORT (static AtomicUsize) / next_test_ip4()/next_test_ip6()

**States:** Fresh/UnusedPort, AllocatedPort

**Transitions:**
- Fresh/UnusedPort -> AllocatedPort via next_test_ip4()
- Fresh/UnusedPort -> AllocatedPort via next_test_ip6()

**Evidence:** static PORT: AtomicUsize = AtomicUsize::new(0); (global mutable allocation state); const BASE_PORT: u16 = 19600; (implicit reserved base range for tests); next_test_ip4(): let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT; (allocates next port and narrows to u16); next_test_ip6(): let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT; (same global allocation for v6 too)

**Implementation:** Introduce a TestPortAllocator capability type (e.g., struct TestPortAllocator { counter: AtomicUsize }) that is constructed explicitly for a test scope and passed to helpers: allocator.next_ip4() -> TestAddr<TestAllocated>. Optionally return a newtype TestPort(u16) (validated non-wrapping / within range) or use a typestate marker to distinguish 'test-allocated' addresses from arbitrary SocketAddr.

---

### 7. Incoming iterator protocol (Listener must remain valid/open while iterating)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-35`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Incoming<'a> is an iterator that repeatedly calls TcpListener::accept() through a shared reference. There is an implicit protocol that the underlying TcpListener resource must remain in a valid state for accepting connections for the duration of iteration. The type system only ties lifetimes (&'a TcpListener) but does not encode the listener's operational state (e.g., 'open and accepting' vs 'closed/shutdown') or whether iteration should terminate after certain errors. next() always returns Some(Result<...>) and never returns None, so termination/stop conditions (if any) are left to runtime error handling and consumer behavior rather than being a typed state transition.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct TcpStream, impl TcpStream (19 methods), impl Read for TcpStream (4 methods), impl Write for TcpStream (4 methods), impl Read for & TcpStream (4 methods), impl Write for & TcpStream (4 methods), impl AsInner < net_imp :: TcpStream > for TcpStream (1 methods), impl FromInner < net_imp :: TcpStream > for TcpStream (1 methods), impl IntoInner < net_imp :: TcpStream > for TcpStream (1 methods); struct TcpListener, impl TcpListener (12 methods), impl AsInner < net_imp :: TcpListener > for TcpListener (1 methods), impl FromInner < net_imp :: TcpListener > for TcpListener (1 methods), impl IntoInner < net_imp :: TcpListener > for TcpListener (1 methods); struct IntoIncoming

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

// ... (other code) ...

// `AsRawSocket`/`IntoRawSocket`/`FromRawSocket` on Windows.

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

#[stable(feature = "tcp_listener_incoming_fused_iterator", since = "1.64.0")]
impl FusedIterator for Incoming<'_> {}

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl Iterator for IntoIncoming {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

#[unstable(feature = "tcplistener_into_incoming", issue = "88373")]
impl FusedIterator for IntoIncoming {}


```

**Entity:** Incoming<'a>

**States:** Usable (listener valid/open), Unusable (listener closed/dropped/errored)

**Transitions:**
- Usable -> Unusable via external event affecting listener (e.g., listener closed/dropped elsewhere or accept starts failing), observed only through io::Result from accept()

**Evidence:** struct Incoming<'a> { listener: &'a TcpListener } — holds only a shared reference; no typed 'open/closed' state; impl Iterator for Incoming<'a>: next() { Some(self.listener.accept().map(|p| p.0)) } — always yields Some(...) and delegates validity to accept() at runtime; impl FusedIterator for Incoming<'_> {} — promises fused behavior even though next() never returns None in this impl, implying an implicit 'never terminates' protocol and leaving 'stop on error' up to the user

**Implementation:** Introduce a dedicated accept-loop type parameterized by a listener state/capability, e.g., Incoming<'a, S> where S is a marker like Accepting. Construct Incoming only from a TcpListener in an 'Accepting' state (or via a method that yields an accept-capability token). If the design intends termination, model it by returning None after a terminal error and encode 'terminated' as a typestate (Incoming<Active> -> Incoming<Terminated>) or by not implementing FusedIterator unless termination is representable.

---

### 9. String address parsing/resolution protocol (Parsed literal -> DNS lookup fallback)

**Location**: `/tmp/net_test_crate/src/net/socket_addr.rs:1-267`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: String-based inputs follow an implicit multi-step protocol: first attempt to parse the input as a literal address (either full SocketAddr for `str` or bare IP for `(&str, u16)`), and only if parsing fails fall back to OS-backed host resolution (`LookupHost` via `try_into()` + `resolve_socket_addr`). The correctness of callers' expectations depends on using the right string format (e.g., `"host:port"` for `str`, or a host without port for `(&str,u16)`), but the type system cannot distinguish these formats; invalid formats only surface as runtime `io::Result` errors from parsing/`try_into()`.

**Evidence**:

```rust
// Tests for this module
#[cfg(all(test, not(any(target_os = "emscripten", all(target_os = "wasi", target_env = "p1")))))]
mod tests;

#[stable(feature = "rust1", since = "1.0.0")]
pub use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::sys::net::LookupHost;
use crate::{io, iter, option, slice, vec};

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// This trait is used for generic address resolution when constructing network
/// objects. By default it is implemented for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], [`SocketAddrV6`], <code>([IpAddr], [u16])</code>,
///    <code>([Ipv4Addr], [u16])</code>, <code>([Ipv6Addr], [u16])</code>:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * <code>(&[str], [u16])</code>: <code>&[str]</code> should be either a string representation
///    of an [`IpAddr`] address as expected by [`FromStr`] implementation or a host
///    name. [`u16`] is the port number.
///
///  * <code>&[str]</code>: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
///
/// This trait allows constructing network objects like [`TcpStream`] or
/// [`UdpSocket`] easily with values of various types for the bind/connection
/// address. It is needed because sometimes one type is more appropriate than
/// the other: for simple uses a string like `"localhost:12345"` is much nicer
/// than manual construction of the corresponding [`SocketAddr`], but sometimes
/// [`SocketAddr`] value is *the* main source of the address, and converting it to
/// some other type (e.g., a string) just for it to be converted back to
/// [`SocketAddr`] in constructor methods is pointless.
///
/// Addresses returned by the operating system that are not IP addresses are
/// silently ignored.
///
/// [`FromStr`]: crate::str::FromStr "std::str::FromStr"
/// [`TcpStream`]: crate::net::TcpStream "net::TcpStream"
/// [`to_socket_addrs`]: ToSocketAddrs::to_socket_addrs
/// [`UdpSocket`]: crate::net::UdpSocket "net::UdpSocket"
///
/// # Examples
///
/// Creating a [`SocketAddr`] iterator that yields one item:
///
/// ```
/// use std::net::{ToSocketAddrs, SocketAddr};
///
/// let addr = SocketAddr::from(([127, 0, 0, 1], 443));
/// let mut addrs_iter = addr.to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Creating a [`SocketAddr`] iterator from a hostname:
///
/// ```no_run
/// use std::net::{SocketAddr, ToSocketAddrs};
///
/// // assuming 'localhost' resolves to 127.0.0.1
/// let mut addrs_iter = "localhost:443".to_socket_addrs().unwrap();
/// assert_eq!(addrs_iter.next(), Some(SocketAddr::from(([127, 0, 0, 1], 443))));
/// assert!(addrs_iter.next().is_none());
///
/// // assuming 'foo' does not resolve
/// assert!("foo:443".to_socket_addrs().is_err());
/// ```
///
/// Creating a [`SocketAddr`] iterator that yields multiple items:
///
/// ```
/// use std::net::{SocketAddr, ToSocketAddrs};
///
/// let addr1 = SocketAddr::from(([0, 0, 0, 0], 80));
/// let addr2 = SocketAddr::from(([127, 0, 0, 1], 443));
/// let addrs = vec![addr1, addr2];
///
/// let mut addrs_iter = (&addrs[..]).to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr1), addrs_iter.next());
/// assert_eq!(Some(addr2), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Attempting to create a [`SocketAddr`] iterator from an improperly formatted
/// socket address `&str` (missing the port):
///
/// ```
/// use std::io;
/// use std::net::ToSocketAddrs;
///
/// let err = "127.0.0.1".to_socket_addrs().unwrap_err();
/// assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
/// ```
///
/// [`TcpStream::connect`] is an example of a function that utilizes
/// `ToSocketAddrs` as a trait bound on its parameter in order to accept
/// different types:
///
/// ```no_run
/// use std::net::{TcpStream, Ipv4Addr};
///
/// let stream = TcpStream::connect(("127.0.0.1", 443));
/// // or
/// let stream = TcpStream::connect("127.0.0.1:443");
/// // or
/// let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), 443));
/// ```
///
/// [`TcpStream::connect`]: crate::net::TcpStream::connect
#[stable(feature = "rust1", since = "1.0.0")]
pub trait ToSocketAddrs {
    /// Returned iterator over socket addresses which this type may correspond
    /// to.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Iter: Iterator<Item = SocketAddr>;

    /// Converts this object to an iterator of resolved [`SocketAddr`]s.
    ///
    /// The returned iterator might not actually yield any values depending on the
    /// outcome of any resolution performed.
    ///
    /// Note that this function may block the current thread while resolution is
    /// performed.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for SocketAddrV6 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

fn resolve_socket_addr(lh: LookupHost) -> io::Result<vec::IntoIter<SocketAddr>> {
    let p = lh.port();
    let v: Vec<_> = lh
        .map(|mut a| {
            a.set_port(p);
            a
        })
        .collect();
    Ok(v.into_iter())
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for (&str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            let addr = SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter());
        }
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            let addr = SocketAddrV6::new(addr, port, 0, 0);
            return Ok(vec![SocketAddr::V6(addr)].into_iter());
        }

        resolve_socket_addr((host, port).try_into()?)
    }
}

#[stable(feature = "string_u16_to_socket_addrs", since = "1.46.0")]
impl ToSocketAddrs for (String, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&*self.0, self.1).to_socket_addrs()
    }
}

// accepts strings like 'localhost:12345'
#[stable(feature = "rust1", since = "1.0.0")]
impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Ok(addr) = self.parse() {
            return Ok(vec![addr].into_iter());
        }

        resolve_socket_addr(self.try_into()?)
    }
}

#[stable(feature = "slice_to_socket_addrs", since = "1.8.0")]
impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

#[stable(feature = "string_to_socket_addrs", since = "1.16.0")]
impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&**self).to_socket_addrs()
    }
}

```

**Entity:** impl ToSocketAddrs for str / (&str, u16) (string-based address inputs)

**States:** UnparsedInput, ParsedAsLiteralSocketAddr, ParsedAsLiteralIpAddr(+port), NeedsResolution, ResolvedAddrs

**Transitions:**
- UnparsedInput -> ParsedAsLiteralSocketAddr via `str::to_socket_addrs` using `self.parse()`
- UnparsedInput -> ParsedAsLiteralIpAddr(+port) via `(&str,u16)::to_socket_addrs` using `host.parse::<Ipv4Addr/Ipv6Addr>()`
- UnparsedInput -> NeedsResolution via parse-failure branches
- NeedsResolution -> ResolvedAddrs via `self.try_into()?` / `(host,port).try_into()?` producing `LookupHost` then `resolve_socket_addr(...)`

**Evidence:** impl ToSocketAddrs for str: `if let Ok(addr) = self.parse() { return Ok(vec![addr].into_iter()); }` then `resolve_socket_addr(self.try_into()?)`; impl ToSocketAddrs for (&str, u16): parses `host.parse::<Ipv4Addr>()` / `host.parse::<Ipv6Addr>()` before `resolve_socket_addr((host, port).try_into()?)`; comment: `// accepts strings like 'localhost:12345'` on the `impl ToSocketAddrs for str` indicates a required input format not captured by types; use of fallible conversions: `self.try_into()?` and `(host, port).try_into()?` encode format/lookup preconditions as runtime errors

**Implementation:** Introduce newtypes for validated string formats, e.g. `HostPortStr<'a>(&'a str)` for `"host:port"` and `HostStr<'a>(&'a str)` for hostnames/IP-without-port, each constructed via `TryFrom<&str>` parsing/validation. Provide `ToSocketAddrs` impls for these newtypes so call sites must choose/validate the intended format before resolution.

---

