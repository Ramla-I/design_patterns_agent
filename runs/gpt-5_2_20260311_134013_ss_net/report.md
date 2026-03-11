# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 24
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 8
- **Precondition**: 6
- **Protocol**: 9
- **Modules analyzed**: 10

## Resource Lifecycle Invariants

### 13. TcpListener bind/ownership protocol (Unbound -> Bound, unique endpoint binding)

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-418`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The tests assume an implicit lifecycle where binding to a socket address produces an exclusive bound listener, and attempting to bind to the same address again should fail. This exclusivity is only validated at runtime via OS error kinds; the type system does not encode 'this address is already bound' or enforce uniqueness of a bound endpoint within a process.

**Evidence**:

```rust
        }
    })
}

#[test]
fn double_bind() {
    each_ip(&mut |addr| {
        let listener1 = t!(TcpListener::bind(&addr));
        match TcpListener::bind(&addr) {
            Ok(listener2) => panic!(
                "This system (perhaps due to options set by TcpListener::bind) \
                 permits double binding: {:?} and {:?}",
                listener1, listener2
            ),
            Err(e) => {
                assert!(
                    e.kind() == ErrorKind::ConnectionRefused
                        || e.kind() == ErrorKind::Uncategorized
                        || e.kind() == ErrorKind::AddrInUse,
                    "unknown error: {} {:?}",
                    e,
                    e.kind()
                );
            }
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_smoke() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 0];
            assert_eq!(s.read(&mut buf).unwrap(), 1);
            assert_eq!(buf[0], 1);
            t!(s.write(&[2]));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            rx1.recv().unwrap();
            t!(s2.write(&[1]));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        assert_eq!(s1.read(&mut buf).unwrap(), 1);
        rx2.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_read() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            t!(s.write(&[1]));
            rx.recv().unwrap();
            t!(s.write(&[2]));
            rx.recv().unwrap();
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            let mut buf = [0, 0];
            t!(s2.read(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(s1.read(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_write() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 1];
            t!(s.read(&mut buf));
            t!(s.read(&mut buf));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            t!(s2.write(&[1]));
            done.send(()).unwrap();
        });
        t!(s1.write(&[2]));

        rx.recv().unwrap();
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn shutdown_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut c = t!(a.accept()).0;
            let mut b = [0];
            assert_eq!(c.read(&mut b).unwrap(), 0);
            t!(c.write(&[1]));
        });

        let mut s = t!(TcpStream::connect(&addr));
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[1]).is_err());
        let mut b = [0, 0];
        assert_eq!(t!(s.read(&mut b)), 1);
        assert_eq!(b[0], 1);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_readwrite_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let (tx, rx) = channel::<()>();
        let _t = thread::spawn(move || {
            let _s = t!(a.accept());
            let _ = rx.recv();
        });

        let mut b = [0];
        let mut s = t!(TcpStream::connect(&addr));
        let mut s2 = t!(s.try_clone());

        // closing should prevent reads/writes
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[0]).is_err());
        t!(s.shutdown(Shutdown::Read));
        assert_eq!(s.read(&mut b).unwrap(), 0);

        // closing should affect previous handles
        assert!(s2.write(&[0]).is_err());
        assert_eq!(s2.read(&mut b).unwrap(), 0);

        // closing should affect new handles
        let mut s3 = t!(s.try_clone());
        assert!(s3.write(&[0]).is_err());
        assert_eq!(s3.read(&mut b).unwrap(), 0);

        // make sure these don't die
        let _ = s2.shutdown(Shutdown::Read);
        let _ = s2.shutdown(Shutdown::Write);
        let _ = s3.shutdown(Shutdown::Read);
        let _ = s3.shutdown(Shutdown::Write);
        drop(tx);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
// On windows, shutdown will not wake up blocking I/O operations.
#[cfg_attr(windows, ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_read_wakes_up() {
    each_ip(&mut |addr| {
        let listener = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let (stream, _) = t!(listener.accept());
            stream
        });

        let mut stream = t!(TcpStream::connect(&addr));
        let stream2 = t!(stream.try_clone());

        let _t = thread::spawn(move || {
            let stream2 = stream2;

            // to make it more likely that `read` happens before `shutdown`
            thread::sleep(Duration::from_millis(1000));

            // this should wake up the reader up
            t!(stream2.shutdown(Shutdown::Read));
        });

        // this `read` should get interrupted by `shutdown`
        assert_eq!(t!(stream.read(&mut [0])), 0);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_while_reading() {
    each_ip(&mut |addr| {
        let accept = t!(TcpListener::bind(&addr));

        // Enqueue a thread to write to a socket
        let (tx, rx) = channel();
        let (txdone, rxdone) = channel();
        let txdone2 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp = t!(TcpStream::connect(&addr));
            rx.recv().unwrap();
            t!(tcp.write(&[0]));
            txdone2.send(()).unwrap();
        });

        // Spawn off a reading clone
        let tcp = t!(accept.accept()).0;
        let tcp2 = t!(tcp.try_clone());
        let txdone3 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp2 = tcp2;
            t!(tcp2.read(&mut [0]));
            txdone3.send(()).unwrap();
        });

        // Try to ensure that the reading clone is indeed reading
        for _ in 0..50 {
            thread::yield_now();
        }

        // clone the handle again while it's reading, then let it finish the
        // read.
        let _ = t!(tcp.try_clone());
        tx.send(()).unwrap();
        rxdone.recv().unwrap();
        rxdone.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        t!(a.accept());
        t!(a2.accept());
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_concurrent() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let (tx, rx) = channel();
        let tx2 = tx.clone();

        let _t = thread::spawn(move || {
            tx.send(t!(a.accept())).unwrap();
        });
        let _t = thread::spawn(move || {
            tx2.send(t!(a2.accept())).unwrap();
        });

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        rx.recv().unwrap();
        rx.recv().unwrap();
    })
}

#[test]
fn debug() {
    #[cfg(not(target_env = "sgx"))]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr
    }
    #[cfg(target_env = "sgx")]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr.to_string()
    }

    #[cfg(any(unix, target_os = "wasi"))]
    use crate::os::fd::AsRawFd;
    #[cfg(target_env = "sgx")]
    use crate::os::fortanix_sgx::io::AsRawFd;
    #[cfg(not(windows))]
    fn render_inner(addr: &dyn AsRawFd) -> impl fmt::Debug {
        addr.as_raw_fd()
    }
    #[cfg(windows)]
    fn render_inner(addr: &dyn crate::os::windows::io::AsRawSocket) -> impl fmt::Debug {
        addr.as_raw_socket()
    }

    let inner_name = if cfg!(windows) { "socket" } else { "fd" };
    let socket_addr = next_test_ip4();

    let listener = t!(TcpListener::bind(&socket_addr));
    let compare = format!(
        "TcpListener {{ addr: {:?}, {}: {:?} }}",
        render_socket_addr(&socket_addr),
        inner_name,
        render_inner(&listener)
    );
    assert_eq!(format!("{listener:?}"), compare);

    let stream = t!(TcpStream::connect(&("localhost", socket_addr.port())));
    let compare = format!(
        "TcpStream {{ addr: {:?}, peer: {:?}, {}: {:?} }}",
        render_socket_addr(&stream.local_addr().unwrap()),
        render_socket_addr(&stream.peer_addr().unwrap()),
        inner_name,
        render_inner(&stream)
    );
    assert_eq!(format!("{stream:?}"), compare);
}

// FIXME: re-enabled openbsd tests once their socket timeout code
//        no longer has rounding errors.
// VxWorks ignores SO_SNDTIMEO.
#[cfg_attr(
    any(target_os = "netbsd", target_os = "openbsd", target_os = "vxworks", target_os = "nto"),
    ignore
)]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
#[test]
fn timeouts() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));
    let dur = Duration::new(15410, 0);

    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_read_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.read_timeout()));

    assert_eq!(None, t!(stream.write_timeout()));

    t!(stream.set_write_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.write_timeout()));

    t!(stream.set_read_timeout(None));
    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_write_timeout(None));
    assert_eq!(None, t!(stream.write_timeout()));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];
    let start = Instant::now();
    let kind = stream.read_exact(&mut buf).err().expect("expected error").kind();
    assert!(
        kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
        "unexpected_error: {:?}",
        kind
    );
    assert!(start.elapsed() > Duration::from_millis(400));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_with_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

```

**Entity:** TcpListener

**States:** Unbound, Bound

**Transitions:**
- Unbound -> Bound via TcpListener::bind(addr)

**Evidence:** double_bind: let listener1 = t!(TcpListener::bind(&addr)); then match TcpListener::bind(&addr) { Ok(listener2) => panic!("permits double binding"), Err(e) => assert!(e.kind()==...||ErrorKind::AddrInUse) } expresses an exclusivity invariant using runtime errors; multiple tests rely on TcpListener::bind(&addr) producing a listener used for accept() (e.g., tcp_clone_smoke: let acceptor = t!(TcpListener::bind(&addr)); then acceptor.accept())

**Implementation:** Model 'bound endpoint' as a capability token returned by an address allocator used in tests (e.g., BoundAddr or PortLease) such that TcpListener::bind takes ownership of a BoundAddr rather than &SocketAddr. This would make double-binding within the test harness impossible at compile time and would encode the 'unique port lease' protocol explicitly (even though OS-level uniqueness cannot be fully proven by types across processes).

---

## State Machine Invariants

### 4. UdpSocket connection-dependent API (Unconnected / Connected)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-180`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: UdpSocket has an implicit runtime state: whether it has been connected to a remote peer. Calling connect() transitions the socket into a 'Connected' mode where send()/recv() use the connected peer and inbound packets are filtered by the OS. In 'Unconnected' mode, send() is documented to fail (and recv() semantics differ; typically recv_from is used, though not shown here). This connection-state is not represented in the type system: send() is available on &UdpSocket regardless of whether connect() has been called, and the invariant is only enforced by OS errors/runtime behavior.

**Evidence**:

```rust
    ///
    /// For more information about this option, see [`UdpSocket::set_multicast_loop_v6`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v6(false).expect("set_multicast_loop_v6 call failed");
    /// assert_eq!(socket.multicast_loop_v6().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.0.multicast_loop_v6()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// assert_eq!(socket.ttl().unwrap(), 42);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    /// Executes an operation of the `IP_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// address of the local interface with which the system should join the
    /// multicast group. If it's equal to [`UNSPECIFIED`](Ipv4Addr::UNSPECIFIED)
    /// then an appropriate interface is chosen by the system.
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.join_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// index of the interface to join/leave (or 0 to indicate any interface).
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.join_multicast_v6(multiaddr, interface)
    }

    /// Executes an operation of the `IP_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see [`UdpSocket::join_multicast_v4`].
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.leave_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see [`UdpSocket::join_multicast_v6`].
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.leave_multicast_v6(multiaddr, interface)
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
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// match socket.take_error() {
    ///     Ok(Some(error)) => println!("UdpSocket error: {error:?}"),
    ///     Ok(None) => println!("No error"),
    ///     Err(error) => println!("UdpSocket.take_error failed: {error:?}"),
    /// }
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` syscalls to be used to send data and also applies filters to only
    /// receive data from the specified address.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until the underlying OS function returns no
    /// error. Note that usually, a successful `connect` call does not specify
    /// that there is a remote server listening on the port, rather, such an
    /// error would only be detected after the first send. If the OS returns an
    /// error for each of the specified addresses, the error returned from the
    /// last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400` and connect the socket to
    /// `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// ```
    ///
    /// Unlike in the TCP case, passing an array of addresses to the `connect`
    /// function of a UDP socket is not a useful thing to do: The OS will be
    /// unable to determine whether something is listening on the remote
    /// address without the application sending data.
    ///
    /// If your first `connect` is to a loopback address, subsequent
    /// `connect`s to non-loopback addresses might fail, depending
    /// on the platform.
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        super::each_addr(addr, |addr| self.0.connect(addr))
    }

    /// Sends data on the socket to the remote address to which it is connected.
    /// On success, returns the number of bytes written. Note that the operating
    /// system may refuse buffers larger than 65507. However, partial writes are
    /// not possible until buffer sizes above `i32::MAX`.
    ///
    /// [`UdpSocket::connect`] will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// socket.send(&[0, 1, 2]).expect("couldn't send message");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    /// Receives a single datagram message on the socket from the remote address to
    /// which it is connected. On success, returns the number of bytes read.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
```

**Entity:** UdpSocket

**States:** Unconnected, Connected

**Transitions:**
- Unconnected -> Connected via connect()

**Evidence:** method doc for connect(): "Connects this UDP socket to a remote address, allowing the `send` and `recv` syscalls to be used... and also applies filters to only receive data from the specified address."; method doc for send(): "Sends data on the socket to the remote address to which it is connected."; method doc for send(): "This method will fail if the socket is not connected."; signature evidence: send(&self, ...) is callable without any connected-typed witness; connect(&self, ...) does not change the static type

**Implementation:** Introduce a typestate parameter: `struct UdpSocket<S> { inner: std::net::UdpSocket, _s: PhantomData<S> }` with `struct Unconnected; struct Connected;`. Provide `fn bind(...) -> UdpSocket<Unconnected>` and `fn connect(self, addr: ...) -> io::Result<UdpSocket<Connected>>`. Implement `send/recv` only for `UdpSocket<Connected>`; unconnected APIs (e.g., send_to/recv_from) remain on `UdpSocket<Unconnected>` or on both as appropriate.

---

### 21. UdpSocket I/O mode protocol (Blocking / Nonblocking)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-67`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: UdpSocket has an implicit runtime I/O mode that affects how recv_from (and other I/O) behaves. Calling set_nonblocking(true) transitions the socket into a mode where operations may return WouldBlock and the caller must use a readiness mechanism (epoll/IOCP/etc.) before retrying. The type system does not distinguish Blocking vs Nonblocking sockets, so APIs that require a particular mode (or require handling WouldBlock) are not enforced at compile time.

**Evidence**:

```rust
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:7878` and read bytes in
    /// nonblocking mode:
    ///
    /// ```no_run
    /// use std::io;
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:7878").unwrap();
    /// socket.set_nonblocking(true).unwrap();
    ///
    /// # fn wait_for_fd() { unimplemented!() }
    /// let mut buf = [0; 10];
    /// let (num_bytes_read, _) = loop {
    ///     match socket.recv_from(&mut buf) {
    ///         Ok(n) => break n,
    ///         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    ///             // wait until network socket is ready, typically implemented
    ///             // via platform-specific APIs such as epoll or IOCP
    ///             wait_for_fd();
    ///         }
    ///         Err(e) => panic!("encountered IO error: {e}"),
    ///     }
    /// };
    /// println!("bytes: {:?}", &buf[..num_bytes_read]);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

// In addition to the `impl`s here, `UdpSocket` also has `impl`s for
// `AsFd`/`From<OwnedFd>`/`Into<OwnedFd>` and
// `AsRawFd`/`IntoRawFd`/`FromRawFd`, on Unix and WASI, and
// `AsSocket`/`From<OwnedSocket>`/`Into<OwnedSocket>` and
// `AsRawSocket`/`IntoRawSocket`/`FromRawSocket` on Windows.

impl AsInner<net_imp::UdpSocket> for UdpSocket {
    #[inline]
    fn as_inner(&self) -> &net_imp::UdpSocket {
        &self.0
    }
}

impl FromInner<net_imp::UdpSocket> for UdpSocket {
    fn from_inner(inner: net_imp::UdpSocket) -> UdpSocket {
        UdpSocket(inner)
    }
}

impl IntoInner<net_imp::UdpSocket> for UdpSocket {
    fn into_inner(self) -> net_imp::UdpSocket {
        self.0
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
```

**Entity:** UdpSocket

**States:** Blocking, Nonblocking

**Transitions:**
- Blocking -> Nonblocking via set_nonblocking(true)
- Nonblocking -> Blocking via set_nonblocking(false)

**Evidence:** method `pub fn set_nonblocking(&self, nonblocking: bool)` toggles mode at runtime; doc example: `socket.set_nonblocking(true).unwrap();` followed by a loop that matches `Err(ref e) if e.kind() == io::ErrorKind::WouldBlock`; doc comment: in nonblocking mode callers should "wait until network socket is ready" via epoll/IOCP

**Implementation:** Represent the mode in the type: `struct UdpSocket<S> { inner: net_imp::UdpSocket, _s: PhantomData<S> }` with `Blocking`/`Nonblocking` marker types. Provide `fn set_nonblocking(self) -> io::Result<UdpSocket<Nonblocking>>` and `fn set_blocking(self) -> io::Result<UdpSocket<Blocking>>`. Optionally expose separate recv APIs where the nonblocking variant returns a `WouldBlock`-aware wrapper type or requires a readiness capability token.

---

### 7. TcpStream I/O mode protocol (Blocking / NonBlocking) affecting read/peek semantics

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-122`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: TcpStream has an implicit runtime I/O mode controlled by set_nonblocking(bool). In Blocking mode, operations like read()/peek() are expected to block until data is available. In NonBlocking mode, the same operations may fail with ErrorKind::WouldBlock instead of blocking. This mode is not represented in the type system, so code must remember which mode the stream is in and handle different error/behavior paths at runtime.

**Evidence**:

```rust

    let result = stream.set_write_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    let result = stream.set_read_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // linger not supported
fn linger() {
    let addr = next_test_ip4();
    let _listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    assert_eq!(None, t!(stream.linger()));
    t!(stream.set_linger(Some(Duration::from_secs(1))));
    assert_eq!(Some(Duration::from_secs(1)), t!(stream.linger()));
    t!(stream.set_linger(None));
    assert_eq!(None, t!(stream.linger()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn nodelay() {
    let addr = next_test_ip4();
    let _listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    assert_eq!(false, t!(stream.nodelay()));
    t!(stream.set_nodelay(true));
    assert_eq!(true, t!(stream.nodelay()));
    t!(stream.set_nodelay(false));
    assert_eq!(false, t!(stream.nodelay()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn ttl() {
    let ttl = 100;

    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    t!(listener.set_ttl(ttl));
    assert_eq!(ttl, t!(listener.ttl()));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    t!(stream.set_ttl(ttl));
    assert_eq!(ttl, t!(stream.ttl()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn set_nonblocking() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    t!(listener.set_nonblocking(true));
    t!(listener.set_nonblocking(false));

    let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));

    t!(stream.set_nonblocking(false));
    t!(stream.set_nonblocking(true));

    let mut buf = [0];
    match stream.read(&mut buf) {
        Ok(_) => panic!("expected error"),
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => panic!("unexpected error {e}"),
    }
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn peek() {
    each_ip(&mut |addr| {
        let (txdone, rxdone) = channel();

        let srv = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut cl = t!(srv.accept()).0;
            cl.write(&[1, 3, 3, 7]).unwrap();
            t!(rxdone.recv());
        });

        let mut c = t!(TcpStream::connect(&addr));
        let mut b = [0; 10];
        for _ in 1..3 {
            let len = c.peek(&mut b).unwrap();
            assert_eq!(len, 4);
        }
        let len = c.read(&mut b).unwrap();
        assert_eq!(len, 4);

        t!(c.set_nonblocking(true));
        match c.peek(&mut b) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {e}"),
        }
        t!(txdone.send(()));
    })
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
fn connect_timeout_valid() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
}
```

**Entity:** TcpStream

**States:** Blocking, NonBlocking

**Transitions:**
- Blocking -> NonBlocking via TcpStream::set_nonblocking(true)
- NonBlocking -> Blocking via TcpStream::set_nonblocking(false)

**Evidence:** set_nonblocking test: TcpStream::set_nonblocking(false) then TcpStream::set_nonblocking(true) on `stream`; set_nonblocking test: match stream.read(&mut buf) expects Err(kind == ErrorKind::WouldBlock) after enabling nonblocking; peek test: c.set_nonblocking(true); then c.peek(&mut b) is expected to return Err(ErrorKind::WouldBlock)

**Implementation:** Introduce typestate wrappers like `TcpStream<Blocking>` and `TcpStream<NonBlocking>`. Implement `set_nonblocking(self, true) -> TcpStream<NonBlocking>` and `set_nonblocking(self, false) -> TcpStream<Blocking>`. Provide `read_blocking()`/`peek_blocking()` only on `Blocking`, and `try_read()`/`try_peek()` returning `WouldBlock` only on `NonBlocking` to force correct handling at compile time.

---

### 2. TcpStream connection & half-shutdown state machine (Connected / ReadShutdown / WriteShutdown / BothShutdown)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-141`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: A TcpStream has an implicit runtime connection state and per-direction shutdown state. After shutdown(Shutdown::Read), read-related I/O is expected to complete immediately with an OS-appropriate result; similarly for Shutdown::Write. shutdown(Shutdown::Both) disables both halves. The type system does not distinguish these states, so methods like peer_addr/local_addr/try_clone/shutdown (and downstream read/write methods not shown) are callable regardless of half-shutdown state, leaving invalid operations to be handled by runtime behavior and OS errors. Additionally, repeated shutdown calls are noted to have platform-dependent behavior (Ok vs NotConnected), meaning the protocol 'shutdown at most once per direction' is relied upon but not enforced.

**Evidence**:

```rust
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
```

**Entity:** TcpStream

**States:** Connected, ReadShutdown, WriteShutdown, BothShutdown

**Transitions:**
- Connected -> ReadShutdown via shutdown(Shutdown::Read)
- Connected -> WriteShutdown via shutdown(Shutdown::Write)
- Connected -> BothShutdown via shutdown(Shutdown::Both)
- ReadShutdown -> BothShutdown via shutdown(Shutdown::Write|Both)
- WriteShutdown -> BothShutdown via shutdown(Shutdown::Read|Both)

**Evidence:** method shutdown(&self, how: Shutdown) -> io::Result<()>: explicit state transition API but no type-level state change; doc on shutdown: 'Shuts down the read, write, or both halves of this connection' and 'pending and future I/O ... return immediately'; doc on shutdown: 'Calling this function multiple times may result in different behavior... Linux ... Ok(()), macOS ... ErrorKind::NotConnected' indicates an implicit 'already shutdown' state checked by OS/runtime; methods peer_addr(), local_addr(), try_clone() are available on &self with no static restriction based on shutdown/connection state

**Implementation:** Model TcpStream as TcpStream<S> with S encoding directionality state (e.g., FullDuplex, ReadClosed, WriteClosed, Closed). Make shutdown(self, Shutdown::Read) -> TcpStream<ReadClosed>, shutdown(self, Shutdown::Write) -> TcpStream<WriteClosed>, etc. Provide Read/Write trait impls only for states that support them, and restrict repeated shutdown by lacking a shutdown method for an already-closed half (or by making it a no-op type transition explicitly).

---

### 11. TcpStream connection + half-shutdown state machine (Connected / ReadClosed / WriteClosed / FullyClosed)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The docs describe a TCP stream that is usable for reading and writing after it is created by connect()/accept(). The connection is closed on Drop, and each direction can be shut down independently via shutdown(). These are real protocol states (full-duplex open, half-closed in either direction, fully closed) that affect which operations are valid and what errors occur, but the type system (as presented here) does not expose separate types/capabilities for the read/write halves or for a shutdown direction; users can still attempt reads/writes after a shutdown and only discover invalidity via runtime behavior/errors.

**Evidence**:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(all(
    test,
    not(any(
        target_os = "emscripten",
        all(target_os = "wasi", target_env = "p1"),
        target_os = "xous",
        target_os = "trusty",
    ))
))]
mod tests;

use crate::fmt;
use crate::io::prelude::*;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::iter::FusedIterator;
use crate::net::{Shutdown, SocketAddr, ToSocketAddrs};
use crate::sys::net as net_imp;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::time::Duration;

/// A TCP stream between a local and a remote socket.
///
/// After creating a `TcpStream` by either [`connect`]ing to a remote host or
/// [`accept`]ing a connection on a [`TcpListener`], data can be transmitted
/// by [reading] and [writing] to it.
///
/// The connection will be closed when the value is dropped. The reading and writing
/// portions of the connection can also be shut down individually with the [`shutdown`]
/// method.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [`accept`]: TcpListener::accept
/// [`connect`]: TcpStream::connect
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [reading]: Read
/// [`shutdown`]: TcpStream::shutdown
/// [writing]: Write
///
/// # Examples
```

**Entity:** TcpStream

**States:** Connected, ReadClosed, WriteClosed, FullyClosed

**Transitions:**
- Connected -> ReadClosed via TcpStream::shutdown(Shutdown::Read)
- Connected -> WriteClosed via TcpStream::shutdown(Shutdown::Write)
- ReadClosed -> FullyClosed via TcpStream::shutdown(Shutdown::Write) or Drop
- WriteClosed -> FullyClosed via TcpStream::shutdown(Shutdown::Read) or Drop
- Connected -> FullyClosed via TcpStream::shutdown(Shutdown::Both) or Drop

**Evidence:** doc comment: "After creating a TcpStream by either connect()ing ... or accept()ing ... data can be transmitted by reading and writing" (implies Connected state after construction); doc comment: "The connection will be closed when the value is dropped." (implies FullyClosed transition via Drop); doc comment: "The reading and writing portions of the connection can also be shut down individually with the shutdown method." (implies independent ReadClosed/WriteClosed states); use crate::net::Shutdown; and doc links [`shutdown`]: TcpStream::shutdown (directional shutdown is part of the API contract)

**Implementation:** Represent directionality as capabilities: e.g., split(self) -> (TcpReadHalf, TcpWriteHalf) where each half implements Read/Write respectively; shutdown(self, Shutdown::Read) could consume and return a TcpStream<ReadClosed>, etc. Alternatively, make shutdown return a new state-parameterized type TcpStream<S> with S = Open, ReadShut, WriteShut, BothShut, so that write() is only available when not WriteShut and read() only when not ReadShut.

---

### 15. TcpStream I/O mode protocol (Blocking / Nonblocking with WouldBlock retry loop)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-271`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TcpStream can be switched between blocking and nonblocking modes via set_nonblocking(). In Nonblocking mode, I/O operations (read/write/recv/send) may return io::ErrorKind::WouldBlock and must be retried once the socket becomes ready (typically via epoll/IOCP). This is an implicit behavioral mode: the same Read/Write methods are always callable, but their correctness expectations and error-handling protocol depend on a runtime socket flag not represented in the type system.

**Evidence**:

```rust
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
    /// assert_eq!(stream.ttl().unwrap_or(0), 100);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
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
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.take_error().expect("No error was expected...");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` system operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind [`io::ErrorKind::WouldBlock`] is
    /// returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    /// # Examples
    ///
    /// Reading bytes from a TCP stream in non-blocking mode:
    ///
    /// ```no_run
    /// use std::io::{self, Read};
    /// use std::net::TcpStream;
    ///
    /// let mut stream = TcpStream::connect("127.0.0.1:7878")
    ///     .expect("Couldn't connect to the server...");
    /// stream.set_nonblocking(true).expect("set_nonblocking call failed");
    ///
    /// # fn wait_for_fd() { unimplemented!() }
    /// let mut buf = vec![];
    /// loop {
    ///     match stream.read_to_end(&mut buf) {
    ///         Ok(_) => break,
    ///         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    ///             // wait until network socket is ready, typically implemented
    ///             // via platform-specific APIs such as epoll or IOCP
    ///             wait_for_fd();
    ///         }
    ///         Err(e) => panic!("encountered IO error: {e}"),
    ///     };
    /// };
    /// println!("bytes: {buf:?}");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

// In addition to the `impl`s here, `TcpStream` also has `impl`s for
// `AsFd`/`From<OwnedFd>`/`Into<OwnedFd>` and
// `AsRawFd`/`IntoRawFd`/`FromRawFd`, on Unix and WASI, and
// `AsSocket`/`From<OwnedSocket>`/`Into<OwnedSocket>` and
// `AsRawSocket`/`IntoRawSocket`/`FromRawSocket` on Windows.

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsInner<net_imp::TcpStream> for TcpStream {
    #[inline]
    fn as_inner(&self) -> &net_imp::TcpStream {
        &self.0
    }
}

impl FromInner<net_imp::TcpStream> for TcpStream {
    fn from_inner(inner: net_imp::TcpStream) -> TcpStream {
        TcpStream(inner)
    }
}

impl IntoInner<net_imp::TcpStream> for TcpStream {
    fn into_inner(self) -> net_imp::TcpStream {
        self.0
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
```

**Entity:** TcpStream

**States:** Blocking, Nonblocking

**Transitions:**
- Blocking -> Nonblocking via set_nonblocking(true)
- Nonblocking -> Blocking via set_nonblocking(false)

**Evidence:** method: TcpStream::set_nonblocking(&self, nonblocking: bool) -> io::Result<()> toggles runtime mode; doc comment on set_nonblocking: "read, write, recv and send ... becoming nonblocking" and describes WouldBlock retry requirement; impl Read for TcpStream: read/read_buf/read_vectored delegate to self.0 without encoding blocking vs nonblocking in types; impl Write for TcpStream: write/write_vectored delegate to self.0 without encoding blocking vs nonblocking in types; example in set_nonblocking docs: loop matching Err(e) if e.kind()==WouldBlock then wait_for_fd() and retry

**Implementation:** Introduce a mode-typed wrapper: TcpStream<Blocking> and TcpStream<Nonblocking> (or BlockingTcpStream/NonblockingTcpStream newtypes). Provide set_nonblocking(self) -> io::Result<TcpStream<Nonblocking>> and set_blocking(self) -> io::Result<TcpStream<Blocking>>. Optionally define NonblockingRead/NonblockingWrite traits or return a custom result (e.g., Poll<usize>) to make the 'WouldBlock means retry after readiness' protocol explicit at call sites.

---

### 12. TcpStream half-close state machine (Open / ReadClosed / WriteClosed / FullyClosed)

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-418`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The tests rely on an implicit connection state machine driven by shutdown(Shutdown::{Read,Write}). After shutting down the write half, further writes must fail, while reads may still succeed. After shutting down the read half, reads must return EOF (0). After both halves are shut down, the stream behaves fully closed. This is enforced only by runtime I/O errors/EOF behavior; the type system does not prevent calling write() after Shutdown::Write or read() after Shutdown::Read, nor does it distinguish partially-closed from open streams.

**Evidence**:

```rust
        }
    })
}

#[test]
fn double_bind() {
    each_ip(&mut |addr| {
        let listener1 = t!(TcpListener::bind(&addr));
        match TcpListener::bind(&addr) {
            Ok(listener2) => panic!(
                "This system (perhaps due to options set by TcpListener::bind) \
                 permits double binding: {:?} and {:?}",
                listener1, listener2
            ),
            Err(e) => {
                assert!(
                    e.kind() == ErrorKind::ConnectionRefused
                        || e.kind() == ErrorKind::Uncategorized
                        || e.kind() == ErrorKind::AddrInUse,
                    "unknown error: {} {:?}",
                    e,
                    e.kind()
                );
            }
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_smoke() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 0];
            assert_eq!(s.read(&mut buf).unwrap(), 1);
            assert_eq!(buf[0], 1);
            t!(s.write(&[2]));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            rx1.recv().unwrap();
            t!(s2.write(&[1]));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        assert_eq!(s1.read(&mut buf).unwrap(), 1);
        rx2.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_read() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            t!(s.write(&[1]));
            rx.recv().unwrap();
            t!(s.write(&[2]));
            rx.recv().unwrap();
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            let mut buf = [0, 0];
            t!(s2.read(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(s1.read(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_write() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 1];
            t!(s.read(&mut buf));
            t!(s.read(&mut buf));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            t!(s2.write(&[1]));
            done.send(()).unwrap();
        });
        t!(s1.write(&[2]));

        rx.recv().unwrap();
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn shutdown_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut c = t!(a.accept()).0;
            let mut b = [0];
            assert_eq!(c.read(&mut b).unwrap(), 0);
            t!(c.write(&[1]));
        });

        let mut s = t!(TcpStream::connect(&addr));
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[1]).is_err());
        let mut b = [0, 0];
        assert_eq!(t!(s.read(&mut b)), 1);
        assert_eq!(b[0], 1);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_readwrite_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let (tx, rx) = channel::<()>();
        let _t = thread::spawn(move || {
            let _s = t!(a.accept());
            let _ = rx.recv();
        });

        let mut b = [0];
        let mut s = t!(TcpStream::connect(&addr));
        let mut s2 = t!(s.try_clone());

        // closing should prevent reads/writes
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[0]).is_err());
        t!(s.shutdown(Shutdown::Read));
        assert_eq!(s.read(&mut b).unwrap(), 0);

        // closing should affect previous handles
        assert!(s2.write(&[0]).is_err());
        assert_eq!(s2.read(&mut b).unwrap(), 0);

        // closing should affect new handles
        let mut s3 = t!(s.try_clone());
        assert!(s3.write(&[0]).is_err());
        assert_eq!(s3.read(&mut b).unwrap(), 0);

        // make sure these don't die
        let _ = s2.shutdown(Shutdown::Read);
        let _ = s2.shutdown(Shutdown::Write);
        let _ = s3.shutdown(Shutdown::Read);
        let _ = s3.shutdown(Shutdown::Write);
        drop(tx);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
// On windows, shutdown will not wake up blocking I/O operations.
#[cfg_attr(windows, ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_read_wakes_up() {
    each_ip(&mut |addr| {
        let listener = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let (stream, _) = t!(listener.accept());
            stream
        });

        let mut stream = t!(TcpStream::connect(&addr));
        let stream2 = t!(stream.try_clone());

        let _t = thread::spawn(move || {
            let stream2 = stream2;

            // to make it more likely that `read` happens before `shutdown`
            thread::sleep(Duration::from_millis(1000));

            // this should wake up the reader up
            t!(stream2.shutdown(Shutdown::Read));
        });

        // this `read` should get interrupted by `shutdown`
        assert_eq!(t!(stream.read(&mut [0])), 0);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_while_reading() {
    each_ip(&mut |addr| {
        let accept = t!(TcpListener::bind(&addr));

        // Enqueue a thread to write to a socket
        let (tx, rx) = channel();
        let (txdone, rxdone) = channel();
        let txdone2 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp = t!(TcpStream::connect(&addr));
            rx.recv().unwrap();
            t!(tcp.write(&[0]));
            txdone2.send(()).unwrap();
        });

        // Spawn off a reading clone
        let tcp = t!(accept.accept()).0;
        let tcp2 = t!(tcp.try_clone());
        let txdone3 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp2 = tcp2;
            t!(tcp2.read(&mut [0]));
            txdone3.send(()).unwrap();
        });

        // Try to ensure that the reading clone is indeed reading
        for _ in 0..50 {
            thread::yield_now();
        }

        // clone the handle again while it's reading, then let it finish the
        // read.
        let _ = t!(tcp.try_clone());
        tx.send(()).unwrap();
        rxdone.recv().unwrap();
        rxdone.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        t!(a.accept());
        t!(a2.accept());
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_concurrent() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let (tx, rx) = channel();
        let tx2 = tx.clone();

        let _t = thread::spawn(move || {
            tx.send(t!(a.accept())).unwrap();
        });
        let _t = thread::spawn(move || {
            tx2.send(t!(a2.accept())).unwrap();
        });

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        rx.recv().unwrap();
        rx.recv().unwrap();
    })
}

#[test]
fn debug() {
    #[cfg(not(target_env = "sgx"))]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr
    }
    #[cfg(target_env = "sgx")]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr.to_string()
    }

    #[cfg(any(unix, target_os = "wasi"))]
    use crate::os::fd::AsRawFd;
    #[cfg(target_env = "sgx")]
    use crate::os::fortanix_sgx::io::AsRawFd;
    #[cfg(not(windows))]
    fn render_inner(addr: &dyn AsRawFd) -> impl fmt::Debug {
        addr.as_raw_fd()
    }
    #[cfg(windows)]
    fn render_inner(addr: &dyn crate::os::windows::io::AsRawSocket) -> impl fmt::Debug {
        addr.as_raw_socket()
    }

    let inner_name = if cfg!(windows) { "socket" } else { "fd" };
    let socket_addr = next_test_ip4();

    let listener = t!(TcpListener::bind(&socket_addr));
    let compare = format!(
        "TcpListener {{ addr: {:?}, {}: {:?} }}",
        render_socket_addr(&socket_addr),
        inner_name,
        render_inner(&listener)
    );
    assert_eq!(format!("{listener:?}"), compare);

    let stream = t!(TcpStream::connect(&("localhost", socket_addr.port())));
    let compare = format!(
        "TcpStream {{ addr: {:?}, peer: {:?}, {}: {:?} }}",
        render_socket_addr(&stream.local_addr().unwrap()),
        render_socket_addr(&stream.peer_addr().unwrap()),
        inner_name,
        render_inner(&stream)
    );
    assert_eq!(format!("{stream:?}"), compare);
}

// FIXME: re-enabled openbsd tests once their socket timeout code
//        no longer has rounding errors.
// VxWorks ignores SO_SNDTIMEO.
#[cfg_attr(
    any(target_os = "netbsd", target_os = "openbsd", target_os = "vxworks", target_os = "nto"),
    ignore
)]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
#[test]
fn timeouts() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));
    let dur = Duration::new(15410, 0);

    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_read_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.read_timeout()));

    assert_eq!(None, t!(stream.write_timeout()));

    t!(stream.set_write_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.write_timeout()));

    t!(stream.set_read_timeout(None));
    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_write_timeout(None));
    assert_eq!(None, t!(stream.write_timeout()));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];
    let start = Instant::now();
    let kind = stream.read_exact(&mut buf).err().expect("expected error").kind();
    assert!(
        kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
        "unexpected_error: {:?}",
        kind
    );
    assert!(start.elapsed() > Duration::from_millis(400));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_with_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

```

**Entity:** TcpStream

**States:** Open, WriteClosed, ReadClosed, FullyClosed

**Transitions:**
- Open -> WriteClosed via TcpStream::shutdown(Shutdown::Write)
- Open -> ReadClosed via TcpStream::shutdown(Shutdown::Read)
- WriteClosed -> FullyClosed via TcpStream::shutdown(Shutdown::Read)
- ReadClosed -> FullyClosed via TcpStream::shutdown(Shutdown::Write)

**Evidence:** shutdown_smoke: t!(s.shutdown(Shutdown::Write)); assert!(s.write(&[1]).is_err()); then assert_eq!(t!(s.read(&mut b)), 1) demonstrates WriteClosed disallows write but allows read; close_readwrite_smoke: t!(s.shutdown(Shutdown::Write)); assert!(s.write(&[0]).is_err()); then t!(s.shutdown(Shutdown::Read)); assert_eq!(s.read(&mut b).unwrap(), 0) demonstrates FullyClosed/ReadClosed yields EOF; close_readwrite_smoke: assert!(s2.write(&[0]).is_err()); assert_eq!(s2.read(&mut b).unwrap(), 0) shows the shutdown state applies across clones/handles (shared underlying socket state); close_read_wakes_up: comment 'this `read` should get interrupted by `shutdown`' and assert_eq!(t!(stream.read(&mut [0])), 0) relies on shutdown(Read) affecting a concurrent blocking read

**Implementation:** Introduce a typestate wrapper around TcpStream such as Stream<S> where S is one of Open/ReadClosed/WriteClosed/FullyClosed, with shutdown_write(self)->Stream<WriteClosed>, shutdown_read(self)->Stream<ReadClosed>, and only implement Write for states that permit writing and Read for states that permit reading. For clone behavior, consider a shared-state wrapper (Arc) with capabilities/tokens for read/write halves, e.g., split(self)->(ReadHalf, WriteHalf) where shutdown consumes the corresponding capability.

---

### 19. UdpSocket I/O behavior depends on blocking mode & timeouts (Blocking / Nonblocking / TimeoutConfigured)

**Location**: `/tmp/net_test_crate/src/net/udp/tests.rs:1-380`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Several socket methods have behavior and expected error kinds that depend on runtime configuration: nonblocking changes recv() to return WouldBlock; read timeout changes recv_from() to eventually return WouldBlock/TimedOut (ignoring Interrupted); timeouts can be enabled/disabled and must be non-zero when set. These are runtime-configured modes, not tracked in the type system, so code must remember and coordinate configuration before calling I/O methods and must handle mode-dependent error results.

**Evidence**:

```rust
use crate::net::test::{compare_ignore_zoneid, next_test_ip4, next_test_ip6};
use crate::net::*;
use crate::sync::mpsc::channel;
use crate::thread;
use crate::time::{Duration, Instant};

fn each_ip(f: &mut dyn FnMut(SocketAddr, SocketAddr)) {
    f(next_test_ip4(), next_test_ip4());
    f(next_test_ip6(), next_test_ip6());
}

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    };
}

#[test]
fn bind_error() {
    match UdpSocket::bind("1.1.1.1:9999") {
        Ok(..) => panic!(),
        Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
    }
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn socket_smoke_test_ip4() {
    each_ip(&mut |server_ip, client_ip| {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let _t = thread::spawn(move || {
            let client = t!(UdpSocket::bind(&client_ip));
            rx1.recv().unwrap();
            t!(client.send_to(&[99], &server_ip));
            tx2.send(()).unwrap();
        });

        let server = t!(UdpSocket::bind(&server_ip));
        tx1.send(()).unwrap();
        let mut buf = [0];
        let (nread, src) = t!(server.recv_from(&mut buf));
        assert_eq!(nread, 1);
        assert_eq!(buf[0], 99);
        assert_eq!(compare_ignore_zoneid(&src, &client_ip), true);
        rx2.recv().unwrap();
    })
}

#[test]
fn socket_name() {
    each_ip(&mut |addr, _| {
        let server = t!(UdpSocket::bind(&addr));
        assert_eq!(addr, t!(server.local_addr()));
    })
}

#[test]
fn socket_peer() {
    each_ip(&mut |addr1, addr2| {
        let server = t!(UdpSocket::bind(&addr1));
        assert_eq!(server.peer_addr().unwrap_err().kind(), ErrorKind::NotConnected);
        t!(server.connect(&addr2));
        assert_eq!(addr2, t!(server.peer_addr()));
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn udp_clone_smoke() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));

        let _t = thread::spawn(move || {
            let mut buf = [0, 0];
            let res = sock2.recv_from(&mut buf).unwrap();
            assert_eq!(res.0, 1);
            assert_eq!(compare_ignore_zoneid(&res.1, &addr1), true);
            assert_eq!(buf[0], 1);
            t!(sock2.send_to(&[2], &addr1));
        });

        let sock3 = t!(sock1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            rx1.recv().unwrap();
            t!(sock3.send_to(&[1], &addr2));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        let res = sock1.recv_from(&mut buf).unwrap();
        assert_eq!(res.0, 1);
        assert_eq!(compare_ignore_zoneid(&res.1, &addr2), true);
        rx2.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn udp_clone_two_read() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move || {
            t!(sock2.send_to(&[1], &addr1));
            rx.recv().unwrap();
            t!(sock2.send_to(&[2], &addr1));
            rx.recv().unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut buf = [0, 0];
            t!(sock3.recv_from(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(sock1.recv_from(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn udp_clone_two_write() {
    each_ip(&mut |addr1, addr2| {
        let sock1 = t!(UdpSocket::bind(&addr1));
        let sock2 = t!(UdpSocket::bind(&addr2));

        let (tx, rx) = channel();
        let (serv_tx, serv_rx) = channel();

        let _t = thread::spawn(move || {
            let mut buf = [0, 1];
            rx.recv().unwrap();
            t!(sock2.recv_from(&mut buf));
            serv_tx.send(()).unwrap();
        });

        let sock3 = t!(sock1.try_clone());

        let (done, rx) = channel();
        let tx2 = tx.clone();
        let _t = thread::spawn(move || {
            if sock3.send_to(&[1], &addr2).is_ok() {
                let _ = tx2.send(());
            }
            done.send(()).unwrap();
        });
        if sock1.send_to(&[2], &addr2).is_ok() {
            let _ = tx.send(());
        }
        drop(tx);

        rx.recv().unwrap();
        serv_rx.recv().unwrap();
    })
}

#[test]
fn debug() {
    let name = if cfg!(windows) { "socket" } else { "fd" };
    let socket_addr = next_test_ip4();

    let udpsock = t!(UdpSocket::bind(&socket_addr));
    let udpsock_inner = udpsock.0.socket().as_raw();
    let compare = format!("UdpSocket {{ addr: {socket_addr:?}, {name}: {udpsock_inner:?} }}");
    assert_eq!(format!("{udpsock:?}"), compare);
}

// FIXME: re-enabled openbsd/netbsd tests once their socket timeout code
//        no longer has rounding errors.
// VxWorks ignores SO_SNDTIMEO.
#[cfg_attr(
    any(target_os = "netbsd", target_os = "openbsd", target_os = "vxworks", target_os = "nto"),
    ignore
)]
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
#[test]
fn timeouts() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
    let dur = Duration::new(15410, 0);

    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_read_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.read_timeout()));

    assert_eq!(None, t!(stream.write_timeout()));

    t!(stream.set_write_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.write_timeout()));

    t!(stream.set_read_timeout(None));
    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_write_timeout(None));
    assert_eq!(None, t!(stream.write_timeout()));
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];

    let start = Instant::now();
    loop {
        let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
        if kind != ErrorKind::Interrupted {
            assert!(
                kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
                "unexpected_error: {:?}",
                kind
            );
            break;
        }
    }
    assert!(start.elapsed() > Duration::from_millis(400));
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_with_timeout() {
    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    t!(stream.send_to(b"hello world", &addr));

    let mut buf = [0; 11];
    t!(stream.recv_from(&mut buf));
    assert_eq!(b"hello world", &buf[..]);

    let start = Instant::now();
    loop {
        let kind = stream.recv_from(&mut buf).err().expect("expected error").kind();
        if kind != ErrorKind::Interrupted {
            assert!(
                kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
                "unexpected_error: {:?}",
                kind
            );
            break;
        }
    }
    assert!(start.elapsed() > Duration::from_millis(400));
}

// Ensure the `set_read_timeout` and `set_write_timeout` calls return errors
// when passed zero Durations
#[test]
fn test_timeout_zero_duration() {
    let addr = next_test_ip4();

    let socket = t!(UdpSocket::bind(&addr));

    let result = socket.set_write_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    let result = socket.set_read_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
}

#[test]
fn connect_send_recv() {
    let addr = next_test_ip4();

    let socket = t!(UdpSocket::bind(&addr));
    t!(socket.connect(addr));

    t!(socket.send(b"hello world"));

    let mut buf = [0; 11];
    t!(socket.recv(&mut buf));
    assert_eq!(b"hello world", &buf[..]);
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // peek not supported
fn connect_send_peek_recv() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));
        t!(socket.connect(addr));

        t!(socket.send(b"hello world"));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let size = t!(socket.peek(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let size = t!(socket.recv(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // peek_from not supported
fn peek_from() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));
        t!(socket.send_to(b"hello world", &addr));

        for _ in 1..3 {
            let mut buf = [0; 11];
            let (size, _) = t!(socket.peek_from(&mut buf));
            assert_eq!(b"hello world", &buf[..]);
            assert_eq!(size, 11);
        }

        let mut buf = [0; 11];
        let (size, _) = t!(socket.recv_from(&mut buf));
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
    })
}

#[test]
fn ttl() {
    let ttl = 100;

    let addr = next_test_ip4();

    let stream = t!(UdpSocket::bind(&addr));

    t!(stream.set_ttl(ttl));
    assert_eq!(ttl, t!(stream.ttl()));
}

#[test]
fn set_nonblocking() {
    each_ip(&mut |addr, _| {
        let socket = t!(UdpSocket::bind(&addr));

        t!(socket.set_nonblocking(true));
        t!(socket.set_nonblocking(false));

        t!(socket.connect(addr));

        t!(socket.set_nonblocking(false));
        t!(socket.set_nonblocking(true));

        let mut buf = [0];
        match socket.recv(&mut buf) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {e}"),
        }
    })
}
```

**Entity:** UdpSocket

**States:** Blocking (default), Nonblocking, Blocking with read timeout, Blocking with write timeout

**Transitions:**
- Blocking -> Nonblocking via set_nonblocking(true)
- Nonblocking -> Blocking via set_nonblocking(false)
- Any -> ReadTimeoutConfigured via set_read_timeout(Some(dur))
- ReadTimeoutConfigured -> NoReadTimeout via set_read_timeout(None)
- Any -> WriteTimeoutConfigured via set_write_timeout(Some(dur))
- WriteTimeoutConfigured -> NoWriteTimeout via set_write_timeout(None)

**Evidence:** set_nonblocking: socket.set_nonblocking(true) then socket.recv(...) matches Err(kind == ErrorKind::WouldBlock); timeouts: set_read_timeout(Some(dur)) then read_timeout() returns Some(dur); later set_read_timeout(None) returns None; timeouts: set_write_timeout(Some(dur)) then write_timeout() returns Some(dur); later set_write_timeout(None) returns None; test_read_timeout: after set_read_timeout(Some(1000ms)), loop expects recv_from to error with WouldBlock or TimedOut (treating Interrupted specially); test_timeout_zero_duration: set_write_timeout(Some(Duration::new(0,0))) and set_read_timeout(Some(Duration::new(0,0))) must error with ErrorKind::InvalidInput

**Implementation:** Provide wrapper types that encode mode: e.g. `UdpSocket<Blocking>`, `UdpSocket<Nonblocking>`, and optional timeout markers. `set_nonblocking(self, true) -> Result<UdpSocket<Nonblocking>, _>` and `set_nonblocking(self, false) -> Result<UdpSocket<Blocking>, _>`. For timeouts, accept a `NonZeroDuration` newtype (constructed via `TryFrom<Duration>`) to make zero-duration unrepresentable, and optionally return a socket wrapper `UdpSocket<ReadTimed>` after setting read timeout.

---

## Precondition Invariants

### 22. UdpSocket configuration protocol (options require a valid, bound OS socket)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: These methods implicitly assume `self` refers to a valid OS UDP socket handle (typically obtained via `UdpSocket::bind`) and that the underlying platform supports the relevant socket option for the socket's address family (e.g., IPv4 vs IPv6). If the socket is invalid/closed or the option is unsupported, operations fail at runtime via `io::Result` rather than being prevented by the type system. The API surface exposes option setters/getters on `&self` with no type-level distinction between a socket that is 'configured for broadcast/multicast behavior' and one that is not, nor between address families that may or may not support particular options.

**Evidence**:

```rust
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
```

**Entity:** UdpSocket

**States:** ValidBoundSocket, InvalidSocketOrUnsupportedOption

**Transitions:**
- ValidBoundSocket (broadcast disabled) -> ValidBoundSocket (broadcast enabled) via set_broadcast(true)
- ValidBoundSocket (broadcast enabled) -> ValidBoundSocket (broadcast disabled) via set_broadcast(false)

**Evidence:** method `set_broadcast(&self, broadcast: bool) -> io::Result<()>` toggles runtime socket option `SO_BROADCAST`; method `broadcast(&self) -> io::Result<bool>` reads runtime socket option `SO_BROADCAST`; method `write_timeout(&self) -> io::Result<Option<Duration>>` reads runtime `SO_SNDTIMEO` configuration; relies on underlying OS socket state via `self.0.write_timeout()`; doc example shows required ordering: `let socket = UdpSocket::bind(...); socket.set_write_timeout(...); socket.write_timeout()` (bind before option access); comment in multicast loop docs: "Note that this might not have any effect on IPv6 sockets." indicates an address-family-dependent capability not reflected in types

**Implementation:** Expose a typed wrapper around `UdpSocket` that encodes (1) binding/valid-handle state and optionally (2) address family/capabilities. For example `struct UdpSocket<S> { inner: std::net::UdpSocket, _s: PhantomData<S> }` with `BoundV4`/`BoundV6` states returned from `bind_v4`/`bind_v6` (or inferred at runtime once, then fixed). Implement `set_broadcast/broadcast` only for `UdpSocket<BoundV4>` (or a `BroadcastCapable` trait). Keep a generic `UdpSocket<Bound>` for options valid across families. This makes "option may be unsupported/no-op" an unrepresentable state for methods that require support.

---

### 6. UdpSocket bind precondition (must bind to a local, well-formed address)

**Location**: `/tmp/net_test_crate/src/net/socket_addr/tests.rs:1-172`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The tests imply an implicit precondition on UdpSocket::bind: the input must be a well-formed socket address and must resolve (if name-like parsing occurs) to a *local* address. Invalid/ambiguous textual input (e.g., malformed IPv6) must not be accepted, and even if parsing/lookup yields some IP (like a DNS server returning its own address), binding to a non-local address must still fail. These requirements are enforced only via runtime parsing/OS errors (is_err), not at compile time via distinct types for 'validated local bind address' vs arbitrary string.

**Evidence**:

```rust
        "[8:9:a:b:c:d:e:f]:80"
    );

    // Shortest possible IPv6 length.
    assert_eq!(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0).to_string(), "[::]:0");

    // Longest possible IPv6 length.
    assert_eq!(
        SocketAddrV6::new(
            Ipv6Addr::new(0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888),
            u16::MAX,
            u32::MAX,
            u32::MAX,
        )
        .to_string(),
        "[1111:2222:3333:4444:5555:6666:7777:8888%4294967295]:65535"
    );

    // Test padding.
    assert_eq!(
        format!("{:22}", SocketAddrV6::new(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 9, 0, 0)),
        "[1:2:3:4:5:6:7:8]:9   "
    );
    assert_eq!(
        format!("{:>22}", SocketAddrV6::new(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 9, 0, 0)),
        "   [1:2:3:4:5:6:7:8]:9"
    );
}

#[test]
fn bind_udp_socket_bad() {
    // rust-lang/rust#53957: This is a regression test for a parsing problem
    // discovered as part of issue rust-lang/rust#23076, where we were
    // incorrectly parsing invalid input and then that would result in a
    // successful `UdpSocket` binding when we would expect failure.
    //
    // At one time, this test was written as a call to `tsa` with
    // INPUT_23076. However, that structure yields an unreliable test,
    // because it ends up passing junk input to the DNS server, and some DNS
    // servers will respond with `Ok` to such input, with the ip address of
    // the DNS server itself.
    //
    // This form of the test is more robust: even when the DNS server
    // returns its own address, it is still an error to bind a UDP socket to
    // a non-local address, and so we still get an error here in that case.

    const INPUT_23076: &str = "1200::AB00:1234::2552:7777:1313:34300";

    assert!(crate::net::UdpSocket::bind(INPUT_23076).is_err())
}

#[test]
fn set_ip() {
    fn ip4(low: u8) -> Ipv4Addr {
        Ipv4Addr::new(77, 88, 21, low)
    }
    fn ip6(low: u16) -> Ipv6Addr {
        Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, low)
    }

    let mut v4 = SocketAddrV4::new(ip4(11), 80);
    assert_eq!(v4.ip(), &ip4(11));
    v4.set_ip(ip4(12));
    assert_eq!(v4.ip(), &ip4(12));

    let mut addr = SocketAddr::V4(v4);
    assert_eq!(addr.ip(), IpAddr::V4(ip4(12)));
    addr.set_ip(IpAddr::V4(ip4(13)));
    assert_eq!(addr.ip(), IpAddr::V4(ip4(13)));
    addr.set_ip(IpAddr::V6(ip6(14)));
    assert_eq!(addr.ip(), IpAddr::V6(ip6(14)));

    let mut v6 = SocketAddrV6::new(ip6(1), 80, 0, 0);
    assert_eq!(v6.ip(), &ip6(1));
    v6.set_ip(ip6(2));
    assert_eq!(v6.ip(), &ip6(2));

    let mut addr = SocketAddr::V6(v6);
    assert_eq!(addr.ip(), IpAddr::V6(ip6(2)));
    addr.set_ip(IpAddr::V6(ip6(3)));
    assert_eq!(addr.ip(), IpAddr::V6(ip6(3)));
    addr.set_ip(IpAddr::V4(ip4(4)));
    assert_eq!(addr.ip(), IpAddr::V4(ip4(4)));
}

#[test]
fn set_port() {
    let mut v4 = SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80);
    assert_eq!(v4.port(), 80);
    v4.set_port(443);
    assert_eq!(v4.port(), 443);

    let mut addr = SocketAddr::V4(v4);
    assert_eq!(addr.port(), 443);
    addr.set_port(8080);
    assert_eq!(addr.port(), 8080);

    let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 0, 0);
    assert_eq!(v6.port(), 80);
    v6.set_port(443);
    assert_eq!(v6.port(), 443);

    let mut addr = SocketAddr::V6(v6);
    assert_eq!(addr.port(), 443);
    addr.set_port(8080);
    assert_eq!(addr.port(), 8080);
}

#[test]
fn set_flowinfo() {
    let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 10, 0);
    assert_eq!(v6.flowinfo(), 10);
    v6.set_flowinfo(20);
    assert_eq!(v6.flowinfo(), 20);
}

#[test]
fn set_scope_id() {
    let mut v6 = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 80, 0, 10);
    assert_eq!(v6.scope_id(), 10);
    v6.set_scope_id(20);
    assert_eq!(v6.scope_id(), 20);
}

#[test]
fn is_v4() {
    let v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80));
    assert!(v4.is_ipv4());
    assert!(!v4.is_ipv6());
}

#[test]
fn is_v6() {
    let v6 = SocketAddr::V6(SocketAddrV6::new(
        Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1),
        80,
        10,
        0,
    ));
    assert!(!v6.is_ipv4());
    assert!(v6.is_ipv6());
}

#[test]
fn socket_v4_to_str() {
    let socket = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 8080);

    assert_eq!(format!("{socket}"), "192.168.0.1:8080");
    assert_eq!(format!("{socket:<20}"), "192.168.0.1:8080    ");
    assert_eq!(format!("{socket:>20}"), "    192.168.0.1:8080");
    assert_eq!(format!("{socket:^20}"), "  192.168.0.1:8080  ");
    assert_eq!(format!("{socket:.10}"), "192.168.0.");
}

#[test]
fn socket_v6_to_str() {
    let mut socket = SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53, 0, 0);

    assert_eq!(format!("{socket}"), "[2a02:6b8:0:1::1]:53");
    assert_eq!(format!("{socket:<24}"), "[2a02:6b8:0:1::1]:53    ");
    assert_eq!(format!("{socket:>24}"), "    [2a02:6b8:0:1::1]:53");
    assert_eq!(format!("{socket:^24}"), "  [2a02:6b8:0:1::1]:53  ");
    assert_eq!(format!("{socket:.15}"), "[2a02:6b8:0:1::");

    socket.set_scope_id(5);

    assert_eq!(format!("{socket}"), "[2a02:6b8:0:1::1%5]:53");
    assert_eq!(format!("{socket:<24}"), "[2a02:6b8:0:1::1%5]:53  ");
    assert_eq!(format!("{socket:>24}"), "  [2a02:6b8:0:1::1%5]:53");
    assert_eq!(format!("{socket:^24}"), " [2a02:6b8:0:1::1%5]:53 ");
    assert_eq!(format!("{socket:.18}"), "[2a02:6b8:0:1::1%5");
}
```

**Entity:** crate::net::UdpSocket

**States:** Unbound, Bound

**Transitions:**
- Unbound -> Bound via UdpSocket::bind(valid_local_addr)

**Evidence:** fn bind_udp_socket_bad: const INPUT_23076: &str = "1200::AB00:1234::2552:7777:1313:34300" (malformed IPv6-like string); fn bind_udp_socket_bad: assert!(crate::net::UdpSocket::bind(INPUT_23076).is_err()); fn bind_udp_socket_bad comments: "incorrectly parsing invalid input"; "some DNS servers will respond with Ok"; "still an error to bind a UDP socket to a non-local address"

**Implementation:** Introduce a validated address type for binding (e.g., `struct LocalBindAddr(SocketAddr);`) constructed only via checks like `try_from(SocketAddr)` that ensures `is_unspecified()`/`is_loopback()`/interface-local as required. Provide `UdpSocket::bind(LocalBindAddr)` (and keep `bind(&str)` as a fallible convenience). This moves the 'local & valid' invariant into the type of the argument for the safe/primary API.

---

### 3. Non-zero timeout precondition (ValidDuration)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-141`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: connect_timeout requires a non-zero Duration. Passing a zero Duration is documented as an error and must be handled at runtime. This is a validity invariant on the timeout argument that could be represented as a non-zero duration newtype to prevent constructing invalid timeouts at compile time (or at least centralize validation).

**Evidence**:

```rust
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
```

**Entity:** TcpStream::connect_timeout

**States:** InvalidTimeout(Zero), ValidTimeout(NonZero)

**Transitions:**
- InvalidTimeout(Zero) -> ValidTimeout(NonZero) via validation/constructor (not present in code)

**Evidence:** doc on connect_timeout: 'It is an error to pass a zero Duration to this function.'; signature connect_timeout(addr: &SocketAddr, timeout: Duration) accepts any Duration, including zero

**Implementation:** Introduce a NonZeroDuration (or Timeout) newtype with a constructor like NonZeroDuration::new(Duration) -> Option<Self> / Result<Self, _>. Change connect_timeout to accept NonZeroDuration, ensuring the zero-duration case is unrepresentable at call sites that use the typed API.

---

### 8. Socket timeout validity invariant (NonZero timeout when Some)

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-122`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Socket timeout setters require that `Some(Duration)` is non-zero; `Some(Duration::new(0,0))` is rejected with ErrorKind::InvalidInput. This is a value-level precondition on the Duration argument that callers can currently violate and only discover at runtime via Result/error-kind checks.

**Evidence**:

```rust

    let result = stream.set_write_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    let result = stream.set_read_timeout(Some(Duration::new(0, 0)));
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);

    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // linger not supported
fn linger() {
    let addr = next_test_ip4();
    let _listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    assert_eq!(None, t!(stream.linger()));
    t!(stream.set_linger(Some(Duration::from_secs(1))));
    assert_eq!(Some(Duration::from_secs(1)), t!(stream.linger()));
    t!(stream.set_linger(None));
    assert_eq!(None, t!(stream.linger()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn nodelay() {
    let addr = next_test_ip4();
    let _listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    assert_eq!(false, t!(stream.nodelay()));
    t!(stream.set_nodelay(true));
    assert_eq!(true, t!(stream.nodelay()));
    t!(stream.set_nodelay(false));
    assert_eq!(false, t!(stream.nodelay()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn ttl() {
    let ttl = 100;

    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    t!(listener.set_ttl(ttl));
    assert_eq!(ttl, t!(listener.ttl()));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));

    t!(stream.set_ttl(ttl));
    assert_eq!(ttl, t!(stream.ttl()));
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)]
fn set_nonblocking() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    t!(listener.set_nonblocking(true));
    t!(listener.set_nonblocking(false));

    let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));

    t!(stream.set_nonblocking(false));
    t!(stream.set_nonblocking(true));

    let mut buf = [0];
    match stream.read(&mut buf) {
        Ok(_) => panic!("expected error"),
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => panic!("unexpected error {e}"),
    }
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn peek() {
    each_ip(&mut |addr| {
        let (txdone, rxdone) = channel();

        let srv = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut cl = t!(srv.accept()).0;
            cl.write(&[1, 3, 3, 7]).unwrap();
            t!(rxdone.recv());
        });

        let mut c = t!(TcpStream::connect(&addr));
        let mut b = [0; 10];
        for _ in 1..3 {
            let len = c.peek(&mut b).unwrap();
            assert_eq!(len, 4);
        }
        let len = c.read(&mut b).unwrap();
        assert_eq!(len, 4);

        t!(c.set_nonblocking(true));
        match c.peek(&mut b) {
            Ok(_) => panic!("expected error"),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => panic!("unexpected error {e}"),
        }
        t!(txdone.send(()));
    })
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
fn connect_timeout_valid() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
}
```

**Entity:** Duration

**States:** ValidTimeout(Some(nonzero)), Disabled(None), InvalidTimeout(Some(zero))

**Transitions:**
- Disabled(None) -> ValidTimeout(Some(nonzero)) via set_read_timeout(Some(d>0)) / set_write_timeout(Some(d>0))
- ValidTimeout(Some(nonzero)) -> Disabled(None) via set_read_timeout(None) / set_write_timeout(None)
- Any -> error via set_read_timeout(Some(Duration::new(0,0))) / set_write_timeout(Some(Duration::new(0,0)))

**Evidence:** code: stream.set_write_timeout(Some(Duration::new(0, 0))) then unwrap_err(); assert_eq!(err.kind(), ErrorKind::InvalidInput); code: stream.set_read_timeout(Some(Duration::new(0, 0))) then unwrap_err(); assert_eq!(err.kind(), ErrorKind::InvalidInput)

**Implementation:** Use a `NonZeroDuration` newtype (constructed via `TryFrom<Duration>` or `NonZeroU64`-backed representation) and change APIs to `set_read_timeout(Option<NonZeroDuration>)` / `set_write_timeout(Option<NonZeroDuration>)`, making zero durations unrepresentable for the `Some` case.

---

### 10. Result-must-be-Ok precondition (panic-on-Err) for network operations in tests

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-293`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: The test code assumes many I/O operations succeed and encodes this as a runtime precondition: any Err causes a panic with context. This is an implicit invariant that certain operations (bind/connect/accept/read/write/local_addr/peer_addr) 'must succeed' under the test setup. The type system still exposes these as fallible Result-returning APIs, so the invariant is only enforced by panics at runtime, not by construction of a context in which failures are unrepresentable.

**Evidence**:

```rust
use crate::io::prelude::*;
use crate::io::{BorrowedBuf, IoSlice, IoSliceMut};
use crate::mem::MaybeUninit;
use crate::net::test::{next_test_ip4, next_test_ip6};
use crate::net::*;
use crate::sync::mpsc::channel;
use crate::time::{Duration, Instant};
use crate::{fmt, thread};

fn each_ip(f: &mut dyn FnMut(SocketAddr)) {
    f(next_test_ip4());
    f(next_test_ip6());
}

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    };
}

#[test]
fn bind_error() {
    match TcpListener::bind("1.1.1.1:9999") {
        Ok(..) => panic!(),
        Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
    }
}

#[test]
fn connect_error() {
    match TcpStream::connect("0.0.0.0:1") {
        Ok(..) => panic!(),
        Err(e) => assert!(
            e.kind() == ErrorKind::ConnectionRefused
                || e.kind() == ErrorKind::InvalidInput
                || e.kind() == ErrorKind::AddrInUse
                || e.kind() == ErrorKind::AddrNotAvailable,
            "bad error: {} {:?}",
            e,
            e.kind()
        ),
    }
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
fn connect_timeout_error() {
    let socket_addr = next_test_ip4();
    let result = TcpStream::connect_timeout(&socket_addr, Duration::MAX);
    assert!(!matches!(result, Err(e) if e.kind() == ErrorKind::TimedOut));

    let _listener = TcpListener::bind(&socket_addr).unwrap();
    assert!(TcpStream::connect_timeout(&socket_addr, Duration::MAX).is_ok());
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn listen_localhost() {
    let socket_addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&socket_addr));

    let _t = thread::spawn(move || {
        let mut stream = t!(TcpStream::connect(&("localhost", socket_addr.port())));
        t!(stream.write(&[144]));
    });

    let mut stream = t!(listener.accept()).0;
    let mut buf = [0];
    t!(stream.read(&mut buf));
    assert!(buf[0] == 144);
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn connect_loopback() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let host = match addr {
                SocketAddr::V4(..) => "127.0.0.1",
                SocketAddr::V6(..) => "::1",
            };
            let mut stream = t!(TcpStream::connect(&(host, addr.port())));
            t!(stream.write(&[66]));
        });

        let mut stream = t!(acceptor.accept()).0;
        let mut buf = [0];
        t!(stream.read(&mut buf));
        assert!(buf[0] == 66);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn smoke_test() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let (tx, rx) = channel();
        let _t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            t!(stream.write(&[99]));
            tx.send(t!(stream.local_addr())).unwrap();
        });

        let (mut stream, addr) = t!(acceptor.accept());
        let mut buf = [0];
        t!(stream.read(&mut buf));
        assert!(buf[0] == 99);
        assert_eq!(addr, t!(rx.recv()));
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn read_eof() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let _stream = t!(TcpStream::connect(&addr));
            // Close
        });

        let mut stream = t!(acceptor.accept()).0;
        let mut buf = [0];
        let nread = t!(stream.read(&mut buf));
        assert_eq!(nread, 0);
        let nread = t!(stream.read(&mut buf));
        assert_eq!(nread, 0);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn write_close() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let (tx, rx) = channel();
        let _t = thread::spawn(move || {
            drop(t!(TcpStream::connect(&addr)));
            tx.send(()).unwrap();
        });

        let mut stream = t!(acceptor.accept()).0;
        rx.recv().unwrap();
        let buf = [0];
        match stream.write(&buf) {
            Ok(..) => {}
            Err(e) => {
                assert!(
                    e.kind() == ErrorKind::ConnectionReset
                        || e.kind() == ErrorKind::BrokenPipe
                        || e.kind() == ErrorKind::ConnectionAborted,
                    "unknown error: {e}"
                );
            }
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_serial() {
    each_ip(&mut |addr| {
        let max = 10;
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            for _ in 0..max {
                let mut stream = t!(TcpStream::connect(&addr));
                t!(stream.write(&[99]));
            }
        });

        for stream in acceptor.incoming().take(max) {
            let mut stream = t!(stream);
            let mut buf = [0];
            t!(stream.read(&mut buf));
            assert_eq!(buf[0], 99);
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_interleaved_greedy_schedule() {
    const MAX: usize = 10;
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let acceptor = acceptor;
            for (i, stream) in acceptor.incoming().enumerate().take(MAX) {
                // Start another thread to handle the connection
                let _t = thread::spawn(move || {
                    let mut stream = t!(stream);
                    let mut buf = [0];
                    t!(stream.read(&mut buf));
                    assert!(buf[0] == i as u8);
                });
            }
        });

        connect(0, addr);
    });

    fn connect(i: usize, addr: SocketAddr) {
        if i == MAX {
            return;
        }

        let t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            // Connect again before writing
            connect(i + 1, addr);
            t!(stream.write(&[i as u8]));
        });
        t.join().ok().expect("thread panicked");
    }
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_interleaved_lazy_schedule() {
    const MAX: usize = 10;
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            for stream in acceptor.incoming().take(MAX) {
                // Start another thread to handle the connection
                let _t = thread::spawn(move || {
                    let mut stream = t!(stream);
                    let mut buf = [0];
                    t!(stream.read(&mut buf));
                    assert!(buf[0] == 99);
                });
            }
        });

        connect(0, addr);
    });

    fn connect(i: usize, addr: SocketAddr) {
        if i == MAX {
            return;
        }

        let t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            connect(i + 1, addr);
            t!(stream.write(&[99]));
        });
        t.join().ok().expect("thread panicked");
    }
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn socket_and_peer_name() {
    each_ip(&mut |addr| {
        let listener = t!(TcpListener::bind(&addr));
        let so_name = t!(listener.local_addr());
        assert_eq!(addr, so_name);
        let _t = thread::spawn(move || {
            t!(listener.accept());
        });

        let stream = t!(TcpStream::connect(&addr));
        assert_eq!(addr, t!(stream.peer_addr()));
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn partial_read() {
    each_ip(&mut |addr| {
        let (tx, rx) = channel();
        let srv = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut cl = t!(srv.accept()).0;
            cl.write(&[10]).unwrap();
            let mut b = [0];
            t!(cl.read(&mut b));
            tx.send(()).unwrap();
        });
```

**Entity:** t! macro (test expectation wrapper)

**States:** OkValueAvailable, ErrValuePanics

**Transitions:**
- OkValueAvailable via matching Ok(t) => t in t!($e)
- ErrValuePanics via matching Err(e) => panic!(...) in t!($e)

**Evidence:** macro_rules! t: `match $e { Ok(t) => t, Err(e) => panic!("received error for `{}`: {}", ...) }`; Used throughout: `t!(TcpListener::bind(&addr))`, `t!(TcpStream::connect(&addr))`, `t!(listener.accept())`, `t!(stream.read(&mut buf))`, `t!(stream.write(&[99]))`, `t!(stream.local_addr())`, `t!(stream.peer_addr())`

**Implementation:** Introduce a test-only capability/context type that vends 'assumed-working' wrappers: e.g., `struct NetOk; impl NetOk { fn bind(&self, addr)->BoundListener; fn connect(&self, addr)->ConnectedStream; }` where wrappers store the underlying types but expose infallible methods (panicking internally once, at creation) so subsequent code cannot even express handling Err. This makes the invariant explicit at the type level within tests.

---

### 18. SocketAddr comparison mode protocol (NormalEq / V6ZoneIdIgnoredEq)

**Location**: `/tmp/net_test_crate/src/net/test.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The function implements an alternate equality relation for SocketAddr that is only applied for IPv6 addresses, intentionally ignoring the IPv6 scope_id/zone_id while still comparing segments, flowinfo, and port. The implicit protocol is that callers must choose the correct comparison mode for their context; this is not represented in types, so it is easy to accidentally use `==` where zone-id should be ignored (or to ignore it when it matters).

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

**Entity:** compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr)

**States:** Normal equality semantics, IPv6 zone-id-ignored equality semantics

**Transitions:**
- Normal equality semantics -> IPv6 zone-id-ignored equality semantics via calling compare_ignore_zoneid() instead of using `==`

**Evidence:** fn compare_ignore_zoneid(a: &SocketAddr, b: &SocketAddr) -> bool; match (a, b) { (SocketAddr::V6(a), SocketAddr::V6(b)) => { a.ip().segments() == b.ip().segments() && a.flowinfo() == b.flowinfo() && a.port() == b.port() } _ => a == b }; No use of `scope_id()`/zone id in the IPv6 branch (implied by only checking segments/flowinfo/port)

**Implementation:** Define wrapper types encoding comparison semantics, e.g. `struct IgnoreZoneId<'a>(&'a SocketAddr);` implementing `PartialEq`/`Eq` with the current logic, or a `SocketAddrCmp` enum/newtype parameterized by a `ZoneIdPolicy` marker type, so the desired equality relation is chosen by type rather than by remembering which helper to call.

---

## Protocol Invariants

### 14. Shared-handle concurrency protocol (coordination required around reads/writes and shutdown across clones)

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-418`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The tests rely on an implicit protocol that cloned TcpStream handles are interchangeable views of the same underlying socket and can be used concurrently across threads, but correctness depends on temporal coordination (via channels/sleeps/yields) to ensure operations happen in the intended order (e.g., writer writes only after reader is waiting; shutdown occurs after read blocks). The type system does not express 'this clone shares shutdown state', 'an I/O is in progress', or provide structured capabilities for coordinated read/write halves; instead, the code uses runtime synchronization and expects particular interleavings.

**Evidence**:

```rust
        }
    })
}

#[test]
fn double_bind() {
    each_ip(&mut |addr| {
        let listener1 = t!(TcpListener::bind(&addr));
        match TcpListener::bind(&addr) {
            Ok(listener2) => panic!(
                "This system (perhaps due to options set by TcpListener::bind) \
                 permits double binding: {:?} and {:?}",
                listener1, listener2
            ),
            Err(e) => {
                assert!(
                    e.kind() == ErrorKind::ConnectionRefused
                        || e.kind() == ErrorKind::Uncategorized
                        || e.kind() == ErrorKind::AddrInUse,
                    "unknown error: {} {:?}",
                    e,
                    e.kind()
                );
            }
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_smoke() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 0];
            assert_eq!(s.read(&mut buf).unwrap(), 1);
            assert_eq!(buf[0], 1);
            t!(s.write(&[2]));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            rx1.recv().unwrap();
            t!(s2.write(&[1]));
            tx2.send(()).unwrap();
        });
        tx1.send(()).unwrap();
        let mut buf = [0, 0];
        assert_eq!(s1.read(&mut buf).unwrap(), 1);
        rx2.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_read() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));
        let (tx1, rx) = channel();
        let tx2 = tx1.clone();

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            t!(s.write(&[1]));
            rx.recv().unwrap();
            t!(s.write(&[2]));
            rx.recv().unwrap();
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            let mut buf = [0, 0];
            t!(s2.read(&mut buf));
            tx2.send(()).unwrap();
            done.send(()).unwrap();
        });
        let mut buf = [0, 0];
        t!(s1.read(&mut buf));
        tx1.send(()).unwrap();

        rx.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn tcp_clone_two_write() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let mut s = t!(TcpStream::connect(&addr));
            let mut buf = [0, 1];
            t!(s.read(&mut buf));
            t!(s.read(&mut buf));
        });

        let mut s1 = t!(acceptor.accept()).0;
        let s2 = t!(s1.try_clone());

        let (done, rx) = channel();
        let _t = thread::spawn(move || {
            let mut s2 = s2;
            t!(s2.write(&[1]));
            done.send(()).unwrap();
        });
        t!(s1.write(&[2]));

        rx.recv().unwrap();
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn shutdown_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut c = t!(a.accept()).0;
            let mut b = [0];
            assert_eq!(c.read(&mut b).unwrap(), 0);
            t!(c.write(&[1]));
        });

        let mut s = t!(TcpStream::connect(&addr));
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[1]).is_err());
        let mut b = [0, 0];
        assert_eq!(t!(s.read(&mut b)), 1);
        assert_eq!(b[0], 1);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_readwrite_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let (tx, rx) = channel::<()>();
        let _t = thread::spawn(move || {
            let _s = t!(a.accept());
            let _ = rx.recv();
        });

        let mut b = [0];
        let mut s = t!(TcpStream::connect(&addr));
        let mut s2 = t!(s.try_clone());

        // closing should prevent reads/writes
        t!(s.shutdown(Shutdown::Write));
        assert!(s.write(&[0]).is_err());
        t!(s.shutdown(Shutdown::Read));
        assert_eq!(s.read(&mut b).unwrap(), 0);

        // closing should affect previous handles
        assert!(s2.write(&[0]).is_err());
        assert_eq!(s2.read(&mut b).unwrap(), 0);

        // closing should affect new handles
        let mut s3 = t!(s.try_clone());
        assert!(s3.write(&[0]).is_err());
        assert_eq!(s3.read(&mut b).unwrap(), 0);

        // make sure these don't die
        let _ = s2.shutdown(Shutdown::Read);
        let _ = s2.shutdown(Shutdown::Write);
        let _ = s3.shutdown(Shutdown::Read);
        let _ = s3.shutdown(Shutdown::Write);
        drop(tx);
    })
}

#[test]
// FIXME: https://github.com/fortanix/rust-sgx/issues/110
#[cfg_attr(target_env = "sgx", ignore)]
// On windows, shutdown will not wake up blocking I/O operations.
#[cfg_attr(windows, ignore)]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn close_read_wakes_up() {
    each_ip(&mut |addr| {
        let listener = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let (stream, _) = t!(listener.accept());
            stream
        });

        let mut stream = t!(TcpStream::connect(&addr));
        let stream2 = t!(stream.try_clone());

        let _t = thread::spawn(move || {
            let stream2 = stream2;

            // to make it more likely that `read` happens before `shutdown`
            thread::sleep(Duration::from_millis(1000));

            // this should wake up the reader up
            t!(stream2.shutdown(Shutdown::Read));
        });

        // this `read` should get interrupted by `shutdown`
        assert_eq!(t!(stream.read(&mut [0])), 0);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_while_reading() {
    each_ip(&mut |addr| {
        let accept = t!(TcpListener::bind(&addr));

        // Enqueue a thread to write to a socket
        let (tx, rx) = channel();
        let (txdone, rxdone) = channel();
        let txdone2 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp = t!(TcpStream::connect(&addr));
            rx.recv().unwrap();
            t!(tcp.write(&[0]));
            txdone2.send(()).unwrap();
        });

        // Spawn off a reading clone
        let tcp = t!(accept.accept()).0;
        let tcp2 = t!(tcp.try_clone());
        let txdone3 = txdone.clone();
        let _t = thread::spawn(move || {
            let mut tcp2 = tcp2;
            t!(tcp2.read(&mut [0]));
            txdone3.send(()).unwrap();
        });

        // Try to ensure that the reading clone is indeed reading
        for _ in 0..50 {
            thread::yield_now();
        }

        // clone the handle again while it's reading, then let it finish the
        // read.
        let _ = t!(tcp.try_clone());
        tx.send(()).unwrap();
        rxdone.recv().unwrap();
        rxdone.recv().unwrap();
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_smoke() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        t!(a.accept());
        t!(a2.accept());
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn clone_accept_concurrent() {
    each_ip(&mut |addr| {
        let a = t!(TcpListener::bind(&addr));
        let a2 = t!(a.try_clone());

        let (tx, rx) = channel();
        let tx2 = tx.clone();

        let _t = thread::spawn(move || {
            tx.send(t!(a.accept())).unwrap();
        });
        let _t = thread::spawn(move || {
            tx2.send(t!(a2.accept())).unwrap();
        });

        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });
        let _t = thread::spawn(move || {
            let _ = TcpStream::connect(&addr);
        });

        rx.recv().unwrap();
        rx.recv().unwrap();
    })
}

#[test]
fn debug() {
    #[cfg(not(target_env = "sgx"))]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr
    }
    #[cfg(target_env = "sgx")]
    fn render_socket_addr<'a>(addr: &'a SocketAddr) -> impl fmt::Debug + 'a {
        addr.to_string()
    }

    #[cfg(any(unix, target_os = "wasi"))]
    use crate::os::fd::AsRawFd;
    #[cfg(target_env = "sgx")]
    use crate::os::fortanix_sgx::io::AsRawFd;
    #[cfg(not(windows))]
    fn render_inner(addr: &dyn AsRawFd) -> impl fmt::Debug {
        addr.as_raw_fd()
    }
    #[cfg(windows)]
    fn render_inner(addr: &dyn crate::os::windows::io::AsRawSocket) -> impl fmt::Debug {
        addr.as_raw_socket()
    }

    let inner_name = if cfg!(windows) { "socket" } else { "fd" };
    let socket_addr = next_test_ip4();

    let listener = t!(TcpListener::bind(&socket_addr));
    let compare = format!(
        "TcpListener {{ addr: {:?}, {}: {:?} }}",
        render_socket_addr(&socket_addr),
        inner_name,
        render_inner(&listener)
    );
    assert_eq!(format!("{listener:?}"), compare);

    let stream = t!(TcpStream::connect(&("localhost", socket_addr.port())));
    let compare = format!(
        "TcpStream {{ addr: {:?}, peer: {:?}, {}: {:?} }}",
        render_socket_addr(&stream.local_addr().unwrap()),
        render_socket_addr(&stream.peer_addr().unwrap()),
        inner_name,
        render_inner(&stream)
    );
    assert_eq!(format!("{stream:?}"), compare);
}

// FIXME: re-enabled openbsd tests once their socket timeout code
//        no longer has rounding errors.
// VxWorks ignores SO_SNDTIMEO.
#[cfg_attr(
    any(target_os = "netbsd", target_os = "openbsd", target_os = "vxworks", target_os = "nto"),
    ignore
)]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
#[test]
fn timeouts() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let stream = t!(TcpStream::connect(&("localhost", addr.port())));
    let dur = Duration::new(15410, 0);

    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_read_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.read_timeout()));

    assert_eq!(None, t!(stream.write_timeout()));

    t!(stream.set_write_timeout(Some(dur)));
    assert_eq!(Some(dur), t!(stream.write_timeout()));

    t!(stream.set_read_timeout(None));
    assert_eq!(None, t!(stream.read_timeout()));

    t!(stream.set_write_timeout(None));
    assert_eq!(None, t!(stream.write_timeout()));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

    let mut stream = t!(TcpStream::connect(&("localhost", addr.port())));
    t!(stream.set_read_timeout(Some(Duration::from_millis(1000))));

    let mut buf = [0; 10];
    let start = Instant::now();
    let kind = stream.read_exact(&mut buf).err().expect("expected error").kind();
    assert!(
        kind == ErrorKind::WouldBlock || kind == ErrorKind::TimedOut,
        "unexpected_error: {:?}",
        kind
    );
    assert!(start.elapsed() > Duration::from_millis(400));
    drop(listener);
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
#[cfg_attr(target_os = "wasi", ignore)] // timeout not supported
fn test_read_with_timeout() {
    let addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&addr));

```

**Entity:** TcpStream (cloned handles)

**States:** SingleHandle, MultiHandle(Cloned), ConcurrentIOInProgress

**Transitions:**
- SingleHandle -> MultiHandle(Cloned) via TcpStream::try_clone()
- MultiHandle(Cloned) -> ConcurrentIOInProgress via spawning threads that read/write/shutdown on different clones
- ConcurrentIOInProgress -> MultiHandle(Cloned) after synchronization completes (channels/join-like behavior via recv)

**Evidence:** tcp_clone_smoke: let s2 = t!(s1.try_clone()); then a spawned thread waits on rx1.recv() before t!(s2.write(&[1])) while main thread does s1.read(...) shows required ordering via channels; tcp_clone_two_read: reads performed on s1 and s2 with tx1/tx2 signals to sequence two writes from the connector thread; tcp_clone_two_write: both s1 and s2 perform writes concurrently while peer performs two reads; clone_while_reading: comment 'Try to ensure that the reading clone is indeed reading' then for _ in 0..50 { thread::yield_now(); } and 'clone the handle again while it's reading' indicates reliance on timing/interleavings not expressible in types; close_read_wakes_up: thread::sleep(Duration::from_millis(1000)) used to increase likelihood that read blocks before shutdown(Read)

**Implementation:** Prefer an explicit split API in tests (or a wrapper) that yields independent typed capabilities, e.g., (ReadHalf, WriteHalf) that are Send + 'static, and make shutdown consume the corresponding half. For sequencing, encapsulate the handshake as a small session-type-like helper (e.g., a struct that owns the channels and exposes methods in the required order) so mis-ordering becomes a type error rather than a flaky runtime interleaving.

---

### 9. TCP connection handshake protocol (Listener bound -> Accepting; Stream connected -> Read/Write; Close -> EOF/errors)

**Location**: `/tmp/net_test_crate/src/net/tcp/tests.rs:1-293`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The tests rely on an implicit multi-step TCP protocol: a TcpListener must be successfully bound before it can accept; accept() yields a connected TcpStream which may be read/written; when the peer drops its TcpStream, subsequent reads must return EOF (0) and writes may error (BrokenPipe/ConnectionReset/etc). None of these phases are represented in the types (TcpListener and TcpStream do not encode 'bound vs unbound' or 'connected vs closed' as distinct types), so the protocol is enforced only by runtime errors/return values and temporal ordering in the test code (spawn/connect/accept/read/write).

**Evidence**:

```rust
use crate::io::prelude::*;
use crate::io::{BorrowedBuf, IoSlice, IoSliceMut};
use crate::mem::MaybeUninit;
use crate::net::test::{next_test_ip4, next_test_ip6};
use crate::net::*;
use crate::sync::mpsc::channel;
use crate::time::{Duration, Instant};
use crate::{fmt, thread};

fn each_ip(f: &mut dyn FnMut(SocketAddr)) {
    f(next_test_ip4());
    f(next_test_ip6());
}

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
        }
    };
}

#[test]
fn bind_error() {
    match TcpListener::bind("1.1.1.1:9999") {
        Ok(..) => panic!(),
        Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
    }
}

#[test]
fn connect_error() {
    match TcpStream::connect("0.0.0.0:1") {
        Ok(..) => panic!(),
        Err(e) => assert!(
            e.kind() == ErrorKind::ConnectionRefused
                || e.kind() == ErrorKind::InvalidInput
                || e.kind() == ErrorKind::AddrInUse
                || e.kind() == ErrorKind::AddrNotAvailable,
            "bad error: {} {:?}",
            e,
            e.kind()
        ),
    }
}

#[test]
#[cfg_attr(target_env = "sgx", ignore)] // FIXME: https://github.com/fortanix/rust-sgx/issues/31
fn connect_timeout_error() {
    let socket_addr = next_test_ip4();
    let result = TcpStream::connect_timeout(&socket_addr, Duration::MAX);
    assert!(!matches!(result, Err(e) if e.kind() == ErrorKind::TimedOut));

    let _listener = TcpListener::bind(&socket_addr).unwrap();
    assert!(TcpStream::connect_timeout(&socket_addr, Duration::MAX).is_ok());
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn listen_localhost() {
    let socket_addr = next_test_ip4();
    let listener = t!(TcpListener::bind(&socket_addr));

    let _t = thread::spawn(move || {
        let mut stream = t!(TcpStream::connect(&("localhost", socket_addr.port())));
        t!(stream.write(&[144]));
    });

    let mut stream = t!(listener.accept()).0;
    let mut buf = [0];
    t!(stream.read(&mut buf));
    assert!(buf[0] == 144);
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn connect_loopback() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let host = match addr {
                SocketAddr::V4(..) => "127.0.0.1",
                SocketAddr::V6(..) => "::1",
            };
            let mut stream = t!(TcpStream::connect(&(host, addr.port())));
            t!(stream.write(&[66]));
        });

        let mut stream = t!(acceptor.accept()).0;
        let mut buf = [0];
        t!(stream.read(&mut buf));
        assert!(buf[0] == 66);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn smoke_test() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let (tx, rx) = channel();
        let _t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            t!(stream.write(&[99]));
            tx.send(t!(stream.local_addr())).unwrap();
        });

        let (mut stream, addr) = t!(acceptor.accept());
        let mut buf = [0];
        t!(stream.read(&mut buf));
        assert!(buf[0] == 99);
        assert_eq!(addr, t!(rx.recv()));
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn read_eof() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let _stream = t!(TcpStream::connect(&addr));
            // Close
        });

        let mut stream = t!(acceptor.accept()).0;
        let mut buf = [0];
        let nread = t!(stream.read(&mut buf));
        assert_eq!(nread, 0);
        let nread = t!(stream.read(&mut buf));
        assert_eq!(nread, 0);
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn write_close() {
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let (tx, rx) = channel();
        let _t = thread::spawn(move || {
            drop(t!(TcpStream::connect(&addr)));
            tx.send(()).unwrap();
        });

        let mut stream = t!(acceptor.accept()).0;
        rx.recv().unwrap();
        let buf = [0];
        match stream.write(&buf) {
            Ok(..) => {}
            Err(e) => {
                assert!(
                    e.kind() == ErrorKind::ConnectionReset
                        || e.kind() == ErrorKind::BrokenPipe
                        || e.kind() == ErrorKind::ConnectionAborted,
                    "unknown error: {e}"
                );
            }
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_serial() {
    each_ip(&mut |addr| {
        let max = 10;
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            for _ in 0..max {
                let mut stream = t!(TcpStream::connect(&addr));
                t!(stream.write(&[99]));
            }
        });

        for stream in acceptor.incoming().take(max) {
            let mut stream = t!(stream);
            let mut buf = [0];
            t!(stream.read(&mut buf));
            assert_eq!(buf[0], 99);
        }
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_interleaved_greedy_schedule() {
    const MAX: usize = 10;
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            let acceptor = acceptor;
            for (i, stream) in acceptor.incoming().enumerate().take(MAX) {
                // Start another thread to handle the connection
                let _t = thread::spawn(move || {
                    let mut stream = t!(stream);
                    let mut buf = [0];
                    t!(stream.read(&mut buf));
                    assert!(buf[0] == i as u8);
                });
            }
        });

        connect(0, addr);
    });

    fn connect(i: usize, addr: SocketAddr) {
        if i == MAX {
            return;
        }

        let t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            // Connect again before writing
            connect(i + 1, addr);
            t!(stream.write(&[i as u8]));
        });
        t.join().ok().expect("thread panicked");
    }
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn multiple_connect_interleaved_lazy_schedule() {
    const MAX: usize = 10;
    each_ip(&mut |addr| {
        let acceptor = t!(TcpListener::bind(&addr));

        let _t = thread::spawn(move || {
            for stream in acceptor.incoming().take(MAX) {
                // Start another thread to handle the connection
                let _t = thread::spawn(move || {
                    let mut stream = t!(stream);
                    let mut buf = [0];
                    t!(stream.read(&mut buf));
                    assert!(buf[0] == 99);
                });
            }
        });

        connect(0, addr);
    });

    fn connect(i: usize, addr: SocketAddr) {
        if i == MAX {
            return;
        }

        let t = thread::spawn(move || {
            let mut stream = t!(TcpStream::connect(&addr));
            connect(i + 1, addr);
            t!(stream.write(&[99]));
        });
        t.join().ok().expect("thread panicked");
    }
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn socket_and_peer_name() {
    each_ip(&mut |addr| {
        let listener = t!(TcpListener::bind(&addr));
        let so_name = t!(listener.local_addr());
        assert_eq!(addr, so_name);
        let _t = thread::spawn(move || {
            t!(listener.accept());
        });

        let stream = t!(TcpStream::connect(&addr));
        assert_eq!(addr, t!(stream.peer_addr()));
    })
}

#[test]
#[cfg_attr(target_os = "wasi", ignore)] // no threads
fn partial_read() {
    each_ip(&mut |addr| {
        let (tx, rx) = channel();
        let srv = t!(TcpListener::bind(&addr));
        let _t = thread::spawn(move || {
            let mut cl = t!(srv.accept()).0;
            cl.write(&[10]).unwrap();
            let mut b = [0];
            t!(cl.read(&mut b));
            tx.send(()).unwrap();
        });
```

**Entity:** TcpListener/TcpStream (test harness usage)

**States:** ListenerUnbound, ListenerBound, AcceptPending, StreamDisconnected, StreamConnected, StreamClosed

**Transitions:**
- ListenerUnbound -> ListenerBound via TcpListener::bind(...)
- ListenerBound -> AcceptPending via TcpListener::accept() / incoming() iteration (blocks until a connection)
- StreamDisconnected -> StreamConnected via TcpStream::connect(...) / connect_timeout(...)
- StreamConnected -> StreamClosed via drop(stream) (peer closes) or scope end
- StreamClosed -> (EOF on read / error on write) via Read/Write calls after peer close

**Evidence:** TcpListener::bind(&socket_addr) is called before listener.accept()/incoming() in listen_localhost/connect_loopback/smoke_test/read_eof/write_close/multiple_connect_*; listen_localhost: thread connects and writes; main thread does listener.accept() then stream.read(...); read_eof: client thread creates TcpStream::connect and then comments `// Close` (drop); server stream.read(...) asserts nread == 0 twice (EOF protocol); write_close: client thread `drop(TcpStream::connect(&addr))`; server waits on rx then `stream.write(&buf)` expects Ok or specific error kinds (BrokenPipe/ConnectionReset/ConnectionAborted); multiple_connect_serial: server uses acceptor.incoming().take(max) and for each accepted stream performs read; relies on bound listener producing a sequence of connected streams

**Implementation:** Model states with typestate wrappers used in tests (or a small test-only API): Listener<Unbound>::bind(self, addr)->Listener<Bound>; Listener<Bound>::accept()->(Listener<Bound>, Stream<Connected>); Stream<Connected>::read/write; Stream<Connected>::close(self)->Stream<Closed> (or rely on Drop but make 'post-close' operations impossible by consuming). This would turn many runtime ordering assumptions in the tests into compile-time method availability.

---

### 1. Socket address parsing vs name resolution protocol (numeric-only vs DNS-dependent)

**Location**: `/tmp/net_test_crate/src/net/socket_addr/tests.rs:1-70`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: These tests rely on an implicit distinction between (1) purely numeric socket address strings that can be parsed locally (e.g., "77.88.21.11:24352", "[2a02:...]:53") and (2) hostname-based inputs (e.g., "localhost:23924") that require name resolution and therefore may be unavailable in some environments (notably SGX). This environmental dependency is handled via cfg-gating in the tests, but the API surface of tsa accepts both forms without a type-level marker indicating whether DNS resolution may be performed.

**Evidence**:

```rust
use crate::net::test::{sa4, sa6, tsa};
use crate::net::*;

#[test]
fn to_socket_addr_ipaddr_u16() {
    let a = Ipv4Addr::new(77, 88, 21, 11);
    let p = 12345;
    let e = SocketAddr::V4(SocketAddrV4::new(a, p));
    assert_eq!(Ok(vec![e]), tsa((a, p)));
}

#[test]
fn to_socket_addr_str_u16() {
    let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
    assert_eq!(Ok(vec![a]), tsa(("77.88.21.11", 24352)));

    let a = sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53);
    assert_eq!(Ok(vec![a]), tsa(("2a02:6b8:0:1::1", 53)));

    let a = sa4(Ipv4Addr::new(127, 0, 0, 1), 23924);
    #[cfg(not(target_env = "sgx"))]
    assert!(tsa(("localhost", 23924)).unwrap().contains(&a));
    #[cfg(target_env = "sgx")]
    let _ = a;
}

#[test]
fn to_socket_addr_str() {
    let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
    assert_eq!(Ok(vec![a]), tsa("77.88.21.11:24352"));

    let a = sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53);
    assert_eq!(Ok(vec![a]), tsa("[2a02:6b8:0:1::1]:53"));

    let a = sa4(Ipv4Addr::new(127, 0, 0, 1), 23924);
    #[cfg(not(target_env = "sgx"))]
    assert!(tsa("localhost:23924").unwrap().contains(&a));
    #[cfg(target_env = "sgx")]
    let _ = a;
}

#[test]
fn to_socket_addr_string() {
    let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 24352);
    assert_eq!(Ok(vec![a]), tsa(&*format!("{}:{}", "77.88.21.11", "24352")));
    assert_eq!(Ok(vec![a]), tsa(&format!("{}:{}", "77.88.21.11", "24352")));
    assert_eq!(Ok(vec![a]), tsa(format!("{}:{}", "77.88.21.11", "24352")));

    let s = format!("{}:{}", "77.88.21.11", "24352");
    assert_eq!(Ok(vec![a]), tsa(s));
    // s has been moved into the tsa call
}

#[test]
fn ipv4_socket_addr_to_string() {
    // Shortest possible IPv4 length.
    assert_eq!(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).to_string(), "0.0.0.0:0");

    // Longest possible IPv4 length.
    assert_eq!(
        SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), u16::MAX).to_string(),
        "255.255.255.255:65535"
    );

    // Test padding.
    assert_eq!(
        format!("{:16}", SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 53)),
        "1.1.1.1:53      "
    );
    assert_eq!(
```

**Entity:** Input to tsa (string forms used to parse/resolve SocketAddr)

**States:** NumericSocketAddrString, HostnameSocketAddrString (requires resolver/OS)

**Transitions:**
- NumericSocketAddrString -> Vec<SocketAddr> via tsa(...) (parse only)
- HostnameSocketAddrString -> Vec<SocketAddr> via tsa(...) (resolve + parse)

**Evidence:** to_socket_addr_str_u16: tsa(("77.88.21.11", 24352)) and tsa(("2a02:6b8:0:1::1", 53)) succeed as numeric address strings; to_socket_addr_str_u16: #[cfg(not(target_env = "sgx"))] assert!(tsa(("localhost", 23924)).unwrap().contains(&a)); indicates hostname resolution is expected only off-SGX; to_socket_addr_str: #[cfg(not(target_env = "sgx"))] assert!(tsa("localhost:23924").unwrap().contains(&a)); same DNS dependency for the single-string form

**Implementation:** Introduce distinct input wrapper types such as NumericAddrStr<'a>(...) and HostnameAddrStr<'a>(...) (or ParsedSocketAddr / UnresolvedName) and provide separate conversion functions/traits: e.g., parse_numeric(NumericAddrStr) -> SocketAddr(s) that is guaranteed DNS-free, and resolve(HostnameAddrStr, ResolverCapability) -> SocketAddr(s). This makes 'may perform DNS' explicit and can be capability-gated for restricted targets.

---

### 5. Multicast membership protocol (NotJoined / Joined for group+interface)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-180`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: Multicast group membership is an implicit protocol: join_multicast_v4/v6 conceptually adds membership for a (group, interface) pair, and leave_multicast_v4/v6 removes it. Correct usage requires pairing leave() with a prior join() for the same parameters; calling leave without join (or double-join/double-leave) is a logical error handled by OS/runtime errors. The type system does not track which multicast groups/interfaces have been joined for a given socket, nor does it provide a scoped RAII handle to ensure leave happens.

**Evidence**:

```rust
    ///
    /// For more information about this option, see [`UdpSocket::set_multicast_loop_v6`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_multicast_loop_v6(false).expect("set_multicast_loop_v6 call failed");
    /// assert_eq!(socket.multicast_loop_v6().unwrap(), false);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.0.multicast_loop_v6()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// assert_eq!(socket.ttl().unwrap(), 42);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    /// Executes an operation of the `IP_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// address of the local interface with which the system should join the
    /// multicast group. If it's equal to [`UNSPECIFIED`](Ipv4Addr::UNSPECIFIED)
    /// then an appropriate interface is chosen by the system.
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.join_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// index of the interface to join/leave (or 0 to indicate any interface).
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.join_multicast_v6(multiaddr, interface)
    }

    /// Executes an operation of the `IP_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see [`UdpSocket::join_multicast_v4`].
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.0.leave_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see [`UdpSocket::join_multicast_v6`].
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.0.leave_multicast_v6(multiaddr, interface)
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
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// match socket.take_error() {
    ///     Ok(Some(error)) => println!("UdpSocket error: {error:?}"),
    ///     Ok(None) => println!("No error"),
    ///     Err(error) => println!("UdpSocket.take_error failed: {error:?}"),
    /// }
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` syscalls to be used to send data and also applies filters to only
    /// receive data from the specified address.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until the underlying OS function returns no
    /// error. Note that usually, a successful `connect` call does not specify
    /// that there is a remote server listening on the port, rather, such an
    /// error would only be detected after the first send. If the OS returns an
    /// error for each of the specified addresses, the error returned from the
    /// last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400` and connect the socket to
    /// `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// ```
    ///
    /// Unlike in the TCP case, passing an array of addresses to the `connect`
    /// function of a UDP socket is not a useful thing to do: The OS will be
    /// unable to determine whether something is listening on the remote
    /// address without the application sending data.
    ///
    /// If your first `connect` is to a loopback address, subsequent
    /// `connect`s to non-loopback addresses might fail, depending
    /// on the platform.
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        super::each_addr(addr, |addr| self.0.connect(addr))
    }

    /// Sends data on the socket to the remote address to which it is connected.
    /// On success, returns the number of bytes written. Note that the operating
    /// system may refuse buffers larger than 65507. However, partial writes are
    /// not possible until buffer sizes above `i32::MAX`.
    ///
    /// [`UdpSocket::connect`] will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// socket.send(&[0, 1, 2]).expect("couldn't send message");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf)
    }

    /// Receives a single datagram message on the socket from the remote address to
    /// which it is connected. On success, returns the number of bytes read.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
```

**Entity:** UdpSocket

**States:** NotJoined(group, interface), Joined(group, interface)

**Transitions:**
- NotJoined(group, interface) -> Joined(group, interface) via join_multicast_v4() / join_multicast_v6()
- Joined(group, interface) -> NotJoined(group, interface) via leave_multicast_v4() / leave_multicast_v6()

**Evidence:** join_multicast_v4 doc: "specifies a new multicast group for this socket to join"; join_multicast_v6 doc: "specifies a new multicast group for this socket to join"; leave_multicast_v4 doc: "IP_DROP_MEMBERSHIP" and "see join_multicast_v4" (implies inverse operation and pairing); leave_multicast_v6 doc: "IPV6_DROP_MEMBERSHIP" and "see join_multicast_v6" (implies inverse operation and pairing); API shape: join_* and leave_* are independent &self methods; no token/handle returned from join to enforce later leave with matching (multiaddr, interface)

**Implementation:** Have `join_multicast_v4/v6` return a guard that records `(socket, group, interface)` and calls `leave_multicast_*` in Drop: `struct MulticastMembership<'a> { sock: &'a UdpSocket, group: ..., iface: ... }`. Optionally make the guard non-clonable and tie it to the socket lifetime, preventing forgetting to leave and preventing leaving with mismatched parameters without having constructed the guard.

---

### 16. TcpStream SO_ERROR consumption semantics (ErrorPresent / ErrorCleared)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-271`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: take_error() both retrieves and clears the underlying socket error (SO_ERROR). This creates an implicit stateful protocol: calling take_error() changes future observations by clearing the error field, but the type system does not reflect that the call is consuming/clearing state. Users can accidentally 'eat' an error by probing at the wrong time, or assume take_error() is a pure getter when it is not.

**Evidence**:

```rust
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
    /// assert_eq!(stream.ttl().unwrap_or(0), 100);
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
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
    /// use std::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.take_error().expect("No error was expected...");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` system operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind [`io::ErrorKind::WouldBlock`] is
    /// returned.
    ///
    /// On Unix platforms, calling this method corresponds to calling `fcntl`
    /// `FIONBIO`. On Windows calling this method corresponds to calling
    /// `ioctlsocket` `FIONBIO`.
    ///
    /// # Examples
    ///
    /// Reading bytes from a TCP stream in non-blocking mode:
    ///
    /// ```no_run
    /// use std::io::{self, Read};
    /// use std::net::TcpStream;
    ///
    /// let mut stream = TcpStream::connect("127.0.0.1:7878")
    ///     .expect("Couldn't connect to the server...");
    /// stream.set_nonblocking(true).expect("set_nonblocking call failed");
    ///
    /// # fn wait_for_fd() { unimplemented!() }
    /// let mut buf = vec![];
    /// loop {
    ///     match stream.read_to_end(&mut buf) {
    ///         Ok(_) => break,
    ///         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    ///             // wait until network socket is ready, typically implemented
    ///             // via platform-specific APIs such as epoll or IOCP
    ///             wait_for_fd();
    ///         }
    ///         Err(e) => panic!("encountered IO error: {e}"),
    ///     };
    /// };
    /// println!("bytes: {buf:?}");
    /// ```
    #[stable(feature = "net2_mutators", since = "1.9.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

// In addition to the `impl`s here, `TcpStream` also has `impl`s for
// `AsFd`/`From<OwnedFd>`/`Into<OwnedFd>` and
// `AsRawFd`/`IntoRawFd`/`FromRawFd`, on Unix and WASI, and
// `AsSocket`/`From<OwnedSocket>`/`Into<OwnedSocket>` and
// `AsRawSocket`/`IntoRawSocket`/`FromRawSocket` on Windows.

#[stable(feature = "rust1", since = "1.0.0")]
impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsInner<net_imp::TcpStream> for TcpStream {
    #[inline]
    fn as_inner(&self) -> &net_imp::TcpStream {
        &self.0
    }
}

impl FromInner<net_imp::TcpStream> for TcpStream {
    fn from_inner(inner: net_imp::TcpStream) -> TcpStream {
        TcpStream(inner)
    }
}

impl IntoInner<net_imp::TcpStream> for TcpStream {
    fn into_inner(self) -> net_imp::TcpStream {
        self.0
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
```

**Entity:** TcpStream

**States:** ErrorPresent, ErrorCleared

**Transitions:**
- ErrorPresent -> ErrorCleared via take_error()
- ErrorCleared -> ErrorCleared via take_error() (returns Ok(None))

**Evidence:** method: TcpStream::take_error(&self) -> io::Result<Option<io::Error>>; doc comment on take_error: "retrieve the stored error ... clearing the field in the process"

**Implementation:** Expose a distinct 'error-drain' capability or explicit API naming/type to mark consumption, e.g. fn drain_error(&self) -> io::Result<Option<io::Error>> returning a newtype DrainError that is intentionally consumptive; or provide two APIs: peek_error(&self) (non-consuming, where supported) vs take/drain (consuming). While the kernel SO_ERROR is inherently runtime, a capability/newtype can make the side-effect explicit in signatures and discourage accidental misuse.

---

### 23. Address-resolution iteration protocol (Resolve -> Try each addr -> Return first success / last error)

**Location**: `/tmp/net_test_crate/src/net/mod.rs:1-81`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: each_addr encodes a multi-step protocol: (1) resolve a generic input via ToSocketAddrs, (2) if resolution fails, call the callback once with Err(e) and return its result, (3) otherwise iterate resolved SocketAddr values and call the callback with Ok(&addr) for each until the first Ok(T) is returned, (4) if all attempts fail, return the last callback error, or a synthetic InvalidInput error if there were no addresses. This protocol (especially 'callback must handle Err from resolution', and 'callback may be called multiple times with Ok') is only enforced by convention and runtime control flow, not by types.

**Evidence**:

```rust
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

**Entity:** each_addr<A: ToSocketAddrs, F, T>

**States:** UnresolvedInput, ResolvedIteratorAvailable, TryingAddresses, Succeeded, ExhaustedWithError, ResolveFailedImmediately

**Transitions:**
- UnresolvedInput -> ResolveFailedImmediately via addr.to_socket_addrs() returning Err(e), then f(Err(e))
- UnresolvedInput -> ResolvedIteratorAvailable via addr.to_socket_addrs() returning Ok(addrs)
- ResolvedIteratorAvailable -> TryingAddresses via for addr in addrs
- TryingAddresses -> Succeeded via f(Ok(&addr)) returning Ok(T)
- TryingAddresses -> ExhaustedWithError via all f(Ok(&addr)) returning Err(e) and last_err being Some(e)
- ResolvedIteratorAvailable -> ExhaustedWithError via empty addrs leading to synthetic InvalidInput error

**Evidence:** fn each_addr<A: ToSocketAddrs, F, T>(addr: A, mut f: F) -> io::Result<T>; let addrs = match addr.to_socket_addrs() { Ok(addrs) => addrs, Err(e) => return f(Err(e)), }; (callback is invoked on resolution failure); F: FnMut(io::Result<&SocketAddr>) -> io::Result<T> (callback must accept both Ok(&SocketAddr) and Err(io::Error)); for addr in addrs { match f(Ok(&addr)) { Ok(l) => return Ok(l), Err(e) => last_err = Some(e), } } (callback may be invoked multiple times; first Ok wins); Err(last_err.unwrap_or_else(|| io::const_error!(ErrorKind::InvalidInput, "could not resolve to any addresses"))) (empty iterator is mapped to a specific error message)

**Implementation:** Split the API into two typed phases: a resolver that returns a non-empty address list (e.g., NonEmptyVec<SocketAddr> newtype) or an error, and a second function that takes an iterator of concrete SocketAddr (no longer io::Result<&SocketAddr>) so the callback only handles the 'trying concrete addr' case. Alternatively, model callback inputs as an enum like ResolveEvent::{ResolveFailed(io::Error), Addr(SocketAddr)} to make the protocol explicit, and enforce non-emptiness with a newtype to eliminate the synthetic "could not resolve" branch.

---

### 24. Blocking-mode protocol for accept()/incoming() (Blocking vs Nonblocking + WouldBlock handling)

**Location**: `/tmp/net_test_crate/src/net/tcp.rs:1-237`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: TcpListener has an implicit mode toggled by set_nonblocking(). In nonblocking mode, accept()/Incoming iteration may yield io::ErrorKind::WouldBlock and the caller is expected to integrate with a readiness mechanism (epoll/IOCP/etc.) before retrying. This is described in docs and examples but not represented in types: Incoming/IntoIncoming always yield io::Result<TcpStream> and do not distinguish the 'needs retry' condition at the type level, so misuse (e.g., busy-looping) is easy.

**Evidence**:

```rust
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

// In addition to the `impl`s here, `TcpListener` also has `impl`s for
// `AsFd`/`From<OwnedFd>`/`Into<OwnedFd>` and
// `AsRawFd`/`IntoRawFd`/`FromRawFd`, on Unix and WASI, and
// `AsSocket`/`From<OwnedSocket>`/`Into<OwnedSocket>` and
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

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
```

**Entity:** TcpListener (with Incoming<'_> / IntoIncoming)

**States:** Blocking, Nonblocking

**Transitions:**
- Blocking -> Nonblocking via set_nonblocking(true)
- Nonblocking -> Blocking via set_nonblocking(false)

**Evidence:** method TcpListener::set_nonblocking(&self, nonblocking: bool); docs for set_nonblocking: "If the IO operation could not be completed and needs to be retried, an error with kind io::ErrorKind::WouldBlock is returned."; Incoming::next and IntoIncoming::next both call self.listener.accept() and always return Some(...), so iteration is an endless protocol around accept() and WouldBlock handling

**Implementation:** Split listener into TcpListener<Blocking> and TcpListener<Nonblocking>, with set_nonblocking(self) -> TcpListener<Nonblocking> (and reverse). Provide Incoming<Blocking> that conceptually blocks (no WouldBlock) and Incoming<Nonblocking> that returns a richer enum like AcceptOutcome::{Ready(TcpStream), WouldBlock, Err(io::Error)} (or a dedicated error type) to force callers to handle readiness explicitly.

---

### 17. Global test-port allocation protocol (FreshPort / WrappedOrColliding)

**Location**: `/tmp/net_test_crate/src/net/test.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The module relies on a single global counter (PORT) to generate distinct port numbers by adding BASE_PORT. Correctness implicitly assumes the counter will not overflow the u16 conversion and that generated ports remain unique (or at least non-colliding) across the test run and across both next_test_ip4() and next_test_ip6(). This uniqueness/valid-range requirement is not enforced by the type system: the code truncates the AtomicUsize to u16 (`as u16`) and uses `Ordering::Relaxed`, so after enough calls ports can wrap and silently collide; additionally, uniqueness is a global property that callers must not subvert by manual port selection.

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

**Entity:** PORT (static AtomicUsize) / next_test_ip4()/next_test_ip6() allocator

**States:** FreshPort (unique within expected test range), WrappedOrColliding (potential reuse/collision)

**Transitions:**
- FreshPort -> WrappedOrColliding via repeated calls to next_test_ip4()/next_test_ip6() causing `(PORT as u16)` wraparound

**Evidence:** static PORT: AtomicUsize = AtomicUsize::new(0);; const BASE_PORT: u16 = 19600;; next_test_ip4(): `let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + BASE_PORT;` (u16 truncation + global counter); next_test_ip6(): same `fetch_add(..) as u16 + BASE_PORT`; use of shared PORT in both next_test_ip4() and next_test_ip6() couples allocation across IPv4/IPv6

**Implementation:** Introduce a `TestPort(u16)` newtype returned by a `TestPortAllocator` (instead of a global atomic) that uses checked arithmetic (e.g., `try_next() -> Option<TestPort>` or `Result<TestPort, Exhausted>`). Then make `next_test_ip4/6` accept `TestPort` (or return `(TestPort, SocketAddr)`), preventing unchecked `as u16` truncation and making exhaustion/collision explicit.

---

### 20. UdpSocket usage protocol (Bound -> Optionally Connected)

**Location**: `/tmp/net_test_crate/src/net/udp.rs:1-41`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The docs describe an implicit protocol: a UdpSocket is first created by binding to a local address (Bound). After that it can communicate in a connectionless way using send_to/recv_from with explicit addresses. Optionally, the socket can be 'connected' to a specific remote peer; once connected, send/recv operate with an implicit peer address. This creates two meaningful runtime modes (Bound vs Connected) with different valid method sets, but the type system (as presented here) does not distinguish them, relying on method choice and runtime OS state instead.

**Evidence**:

```rust
#[cfg(all(
    test,
    not(any(
        target_os = "emscripten",
        all(target_os = "wasi", target_env = "p1"),
        target_env = "sgx",
        target_os = "xous",
        target_os = "trusty",
    ))
))]
mod tests;

use crate::fmt;
use crate::io::{self, ErrorKind};
use crate::net::{Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use crate::sys::net as net_imp;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::time::Duration;

/// A UDP socket.
///
/// After creating a `UdpSocket` by [`bind`]ing it to a socket address, data can be
/// [sent to] and [received from] any other socket address.
///
/// Although UDP is a connectionless protocol, this implementation provides an interface
/// to set an address where data should be sent and received from. After setting a remote
/// address with [`connect`], data can be sent to and received from that address with
/// [`send`] and [`recv`].
///
/// As stated in the User Datagram Protocol's specification in [IETF RFC 768], UDP is
/// an unordered, unreliable protocol; refer to [`TcpListener`] and [`TcpStream`] for TCP
/// primitives.
///
/// [`bind`]: UdpSocket::bind
/// [`connect`]: UdpSocket::connect
/// [IETF RFC 768]: https://tools.ietf.org/html/rfc768
/// [`recv`]: UdpSocket::recv
/// [received from]: UdpSocket::recv_from
/// [`send`]: UdpSocket::send
/// [sent to]: UdpSocket::send_to
/// [`TcpListener`]: crate::net::TcpListener
```

**Entity:** UdpSocket

**States:** Unbound (not constructible via this API), Bound, Connected(to a fixed peer)

**Transitions:**
- Bound -> Connected(to a fixed peer) via UdpSocket::connect(...)

**Evidence:** doc comment: "After creating a `UdpSocket` by [`bind`]ing it to a socket address..." (bind is a required first step described as creation); doc comment: "After setting a remote address with [`connect`], data can be sent to and received from that address with [`send`] and [`recv`]." (connect enables send/recv mode); doc links/method mentions: UdpSocket::bind, UdpSocket::connect, UdpSocket::send, UdpSocket::recv, UdpSocket::send_to, UdpSocket::recv_from (distinct method families imply distinct modes)

**Implementation:** Represent the peer-selection mode at the type level: `struct UdpSocket<S> { inner: net_imp::UdpSocket, _state: PhantomData<S> }` with states like `Bound` and `Connected`. Provide `connect(self, addr) -> UdpSocket<Connected>`; implement `send/recv` only for `UdpSocket<Connected>` and `send_to/recv_from` for `UdpSocket<Bound>` (or for both if desired). Keep `bind(...) -> UdpSocket<Bound>`.

---

