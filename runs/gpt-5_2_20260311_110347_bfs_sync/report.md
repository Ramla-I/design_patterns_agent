# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 80
- **Temporal ordering**: 1
- **Resource lifecycle**: 13
- **State machine**: 24
- **Precondition**: 7
- **Protocol**: 35
- **Modules analyzed**: 23

## Temporal Ordering Invariants

### 44. Slot message publication protocol (Empty -> Written)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-22`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Slot<T> implicitly encodes a publication protocol: a producer must write/initialize `msg` and then set a `WRITE` flag in `state`. Consumers must not read `msg` until `state` indicates the slot is written; `wait_write()` enforces this by spinning until the `WRITE` bit is observed. This protocol (and the safety requirement that `msg` is only read after being initialized) is not represented in the type system because `msg` is `UnsafeCell<MaybeUninit<T>>` and the write/read readiness is tracked by an atomic bitfield (`state`).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Block, impl Block < T > (3 methods); struct Position; struct ListToken; struct Channel, impl Channel < T > (17 methods), impl Drop for Channel < T > (1 methods)

const MARK_BIT: usize = 1;

/// A slot in a block.
struct Slot<T> {
    /// The message.
    msg: UnsafeCell<MaybeUninit<T>>,

    /// The state of the slot.
    state: Atomic<usize>,
}

impl<T> Slot<T> {
    /// Waits until a message is written into the slot.
    fn wait_write(&self) {
        let backoff = Backoff::new();
        while self.state.load(Ordering::Acquire) & WRITE == 0 {
            backoff.spin_heavy();
        }
    }
}

```

**Entity:** Slot<T>

**States:** Empty (no message published), Written (message available)

**Transitions:**
- Empty -> Written via producer setting `state` WRITE bit (implied by `wait_write()` checking `WRITE`)

**Evidence:** const MARK_BIT: usize = 1 and field `state: Atomic<usize>` indicate a bitflag-based runtime state encoding; field `msg: UnsafeCell<MaybeUninit<T>>` indicates `T` may be uninitialized until some protocol step completes; method `wait_write(&self)` spins on `while self.state.load(Ordering::Acquire) & WRITE == 0` implying a required ordering: observe WRITE before accessing `msg`; comment: `/// Waits until a message is written into the slot.` describes the required temporal ordering

**Implementation:** Represent readiness at the type level by splitting the slot view into states, e.g. `Slot<T>` containing storage plus `fn reserve(&self) -> WritePermit<'_, T>`; producer writes through `WritePermit::write(self, t)` which sets the WRITE bit and returns a `ReadPermit<'_, T>` (or makes a `&T` available). Consumers obtain a `ReadPermit` only after observing readiness, so reading `T` becomes impossible without the permit type. Alternatively, encapsulate `MaybeUninit` behind an API where `read()` requires a proof token produced after `wait_write()`.

---

## Resource Lifecycle Invariants

### 19. Receiver intrusive refcount + deallocation protocol (Alive / LastRelease / Freed)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/counter.rs:1-50`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Receiver<C> is a thin wrapper around a raw pointer to a shared Counter<C>. Correctness relies on an implicit lifecycle protocol: (1) receivers must be created via acquire() (or equivalent) to increment the receivers refcount; (2) each acquired receiver must be exactly once released via unsafe release(); (3) on the last release, disconnect() is invoked and the underlying Counter may be deallocated if the shared destroy flag indicates the other side already initiated destruction. None of this is enforced by the type system: Receiver is Copy-like (raw pointer) without Drop, deref() can be used even after the counter has been freed, and release() being unsafe indicates callers must uphold the protocol manually.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Counter; struct Sender, 1 free function(s), impl Sender < C > (3 methods), impl ops :: Deref for Sender < C > (1 methods)

}

/// The receiving side.
pub(crate) struct Receiver<C> {
    counter: *mut Counter<C>,
}

impl<C> Receiver<C> {
    /// Returns the internal `Counter`.
    fn counter(&self) -> &Counter<C> {
        unsafe { &*self.counter }
    }

    /// Acquires another receiver reference.
    pub(crate) fn acquire(&self) -> Receiver<C> {
        let count = self.counter().receivers.fetch_add(1, Ordering::Relaxed);

        // Cloning receivers and calling `mem::forget` on the clones could potentially overflow the
        // counter. It's very difficult to recover sensibly from such degenerate scenarios so we
        // just abort when the count becomes very large.
        if count > isize::MAX as usize {
            process::abort();
        }

        Receiver { counter: self.counter }
    }

    /// Releases the receiver reference.
    ///
    /// Function `disconnect` will be called if this is the last receiver reference.
    pub(crate) unsafe fn release<F: FnOnce(&C) -> bool>(&self, disconnect: F) {
        if self.counter().receivers.fetch_sub(1, Ordering::AcqRel) == 1 {
            disconnect(&self.counter().chan);

            if self.counter().destroy.swap(true, Ordering::AcqRel) {
                drop(unsafe { Box::from_raw(self.counter) });
            }
        }
    }
}

impl<C> ops::Deref for Receiver<C> {
    type Target = C;

    fn deref(&self) -> &C {
        &self.counter().chan
    }
}

```

**Entity:** Receiver<C>

**States:** Alive (counter valid), LastRelease (disconnect + possible destroy), Freed (counter deallocated; pointer invalid)

**Transitions:**
- Alive -> Alive via acquire() (increments receivers)
- Alive -> LastRelease via release() when receivers.fetch_sub(...) == 1
- LastRelease -> Freed via release() when destroy.swap(true, ...) returns true and Box::from_raw(self.counter) is dropped

**Evidence:** field `counter: *mut Counter<C>`: raw pointer with no lifetime tying it to an allocation; method `fn counter(&self) -> &Counter<C> { unsafe { &*self.counter } }`: dereferences raw pointer; requires it to remain valid (implicit Alive state); method `pub(crate) fn acquire(&self) -> Receiver<C>`: `receivers.fetch_add(1, ...)` implements runtime refcounting; comment in acquire(): warns about cloning + mem::forget overflowing the counter; aborts when `count > isize::MAX as usize` (runtime enforcement of protocol misuse); method `pub(crate) unsafe fn release<F: FnOnce(&C) -> bool>(&self, ...)`: unsafe indicates caller must ensure correct usage/ordering; release(): `if receivers.fetch_sub(1, ...) == 1 { disconnect(&chan); if destroy.swap(true, ...) { drop(Box::from_raw(self.counter)) } }`: last-release triggers disconnect and conditional deallocation

**Implementation:** Make Receiver own a non-null shared pointer (e.g., `NonNull<Counter<C>>`) and implement `Drop` to perform the fetch_sub/disconnect/deallocate logic automatically. Ensure cloning goes through `Clone` which calls fetch_add. Optionally split into a safe `Receiver` that always participates in RAII and an internal `ReceiverRef` for borrowed uses; use lifetimes or Arc-like inner to prevent deref after free.

---

### 63. MappedRwLockReadGuard validity/lifetime protocol (Lock-held pointer valid until Drop)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: MappedRwLockReadGuard stores a raw (NonNull) pointer to mapped data plus a reference to the underlying sys::RwLock. The safety protocol is that the pointer is only valid while the read lock is held, i.e., until the guard is dropped; after Drop releases the lock, the pointer must be treated as invalid. This is an implicit resource-lifecycle invariant: the type does not encode (beyond RAII Drop existing elsewhere) that `data` is derived from and protected by `inner_lock`, nor does it prevent constructing a guard with a mismatched/invalid `data` pointer relative to the referenced lock. The comment also indicates an aliasing/immutability constraint: the guard does not guarantee immutability for the whole lexical scope, only up to Drop, hence it cannot safely be represented as `&'a T`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLock, impl Send for RwLock < T > (0 methods), impl Sync for RwLock < T > (0 methods), impl Send for RwLockReadGuard < '_ , T > (0 methods), impl Sync for RwLockReadGuard < '_ , T > (0 methods), impl Send for RwLockWriteGuard < '_ , T > (0 methods), impl Sync for RwLockWriteGuard < '_ , T > (0 methods), impl Send for MappedRwLockReadGuard < '_ , T > (0 methods), impl Sync for MappedRwLockReadGuard < '_ , T > (0 methods), impl Send for MappedRwLockWriteGuard < '_ , T > (0 methods), impl Sync for MappedRwLockWriteGuard < '_ , T > (0 methods), impl RwLock < T > (4 methods), impl RwLock < T > (8 methods), impl From < T > for RwLock < T > (1 methods), impl RwLockReadGuard < 'rwlock , T > (1 methods), impl RwLockWriteGuard < 'rwlock , T > (1 methods), impl Deref for RwLockReadGuard < '_ , T > (1 methods), impl Deref for RwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for RwLockWriteGuard < '_ , T > (1 methods), impl Deref for MappedRwLockReadGuard < '_ , T > (1 methods), impl Deref for MappedRwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for MappedRwLockWriteGuard < '_ , T > (1 methods), impl Drop for RwLockReadGuard < '_ , T > (1 methods), impl Drop for RwLockWriteGuard < '_ , T > (1 methods), impl Drop for MappedRwLockReadGuard < '_ , T > (1 methods), impl Drop for MappedRwLockWriteGuard < '_ , T > (1 methods), impl RwLockReadGuard < 'a , T > (2 methods), impl MappedRwLockReadGuard < 'a , T > (2 methods), impl RwLockWriteGuard < 'a , T > (3 methods), impl MappedRwLockWriteGuard < 'a , T > (2 methods); struct RwLockReadGuard; struct RwLockWriteGuard; struct MappedRwLockWriteGuard

                      and cause Futures to not implement `Send`"]
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
#[clippy::has_significant_drop]
pub struct MappedRwLockReadGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `MappedRwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it drops.
    // `NonNull` is also covariant over `T`, just like we would have with `&T`. `NonNull`
    // is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
}

```

**Entity:** MappedRwLockReadGuard<'a, T>

**States:** Live (lock held; data pointer valid), Dropped (lock released; data pointer must not be used)

**Transitions:**
- Live -> Dropped via Drop (releasing the read lock)

**Evidence:** field `data: NonNull<T>`: raw pointer requires an external validity invariant (no inherent lifetime/aliasing guarantees); field `inner_lock: &'a sys::RwLock`: ties guard lifetime to a lock reference, but does not type-link `data` provenance to that lock; comment: "MappedRwLockReadGuard argument doesn't hold immutability for its whole scope, only until it drops" (temporal validity/aliasing protocol); comment: "we use a pointer instead of `&'a T` to avoid `noalias` violations" (indicates an implicit aliasing model not captured by the type)

**Implementation:** Introduce an internal, unforgeable capability/token type representing the held read-lock (e.g., `struct ReadCap<'a>(&'a sys::RwLock);` not constructible outside the module) and require `data` to be created only from that capability (e.g., `MappedRwLockReadGuard { data: NonNull<T>, cap: ReadCap<'a> }`). This makes it impossible (outside trusted code) to pair an arbitrary `NonNull<T>` with an unrelated lock reference, and more explicitly encodes that pointer validity is contingent on the lock-held capability until Drop.

---

### 36. Owned receiver drain protocol (OwnedReceiver -> Drained/Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: IntoIter owns a Receiver and repeatedly calls recv(). Like Iter, it maps any RecvError to None via ok(), terminating iteration when the channel is disconnected/exhausted. This encodes a lifecycle: an owned receiver is 'drained' until completion, after which it yields no more items. The type system does not represent the drained/disconnected terminal state; it is only observed at runtime when recv() returns Err and is converted into None.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[stable(feature = "receiver_try_iter", since = "1.15.0")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** IntoIter<T>

**States:** OwnedReceiver, DrainedOrDisconnected

**Transitions:**
- OwnedReceiver -> DrainedOrDisconnected via IntoIter::next() calling Receiver::recv() and observing Err (mapped to None)

**Evidence:** impl Iterator for IntoIter<T>: next(): self.rx.recv().ok() — draining an owned receiver until recv() errors; type name IntoIter<T> and field access self.rx (implied owned receiver inside IntoIter) indicate ownership-based iteration rather than borrowing

**Implementation:** Model the terminal state explicitly (e.g., IntoIter<Alive> -> IntoIter<Done>), or return a richer termination reason (e.g., Result<Option<T>, RecvError>) so callers can distinguish graceful exhaustion vs disconnection rather than only receiving None.

---

### 22. ReentrantLockGuard acquisition/holding protocol (Locked -> Unlocked on drop)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/reentrant_lock.rs:1-8`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: ReentrantLockGuard represents the 'Held' state of a ReentrantLock: while a guard value exists, the lock is considered locked (possibly reentrantly) and will be unlocked when the guard is dropped. This is an implicit protocol: callers must keep the guard alive for as long as they need the lock held, and dropping (including via forgetting/losing the guard) transitions the lock back toward 'NotHeld'. The type itself encodes 'Held' via possession of the guard, but the API still relies on a usage protocol (keep the value) that is only hinted at by attributes/messages rather than being structurally enforced (e.g., via a capability token that must be consumed to access protected operations).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ReentrantLock, impl Send for ReentrantLock < T > (0 methods), impl Sync for ReentrantLock < T > (0 methods), impl UnwindSafe for ReentrantLock < T > (0 methods), impl RefUnwindSafe for ReentrantLock < T > (0 methods), impl Send for ReentrantLockGuard < '_ , T > (0 methods), impl Sync for ReentrantLockGuard < '_ , T > (0 methods), impl ReentrantLock < T > (2 methods), impl ReentrantLock < T > (4 methods), impl From < T > for ReentrantLock < T > (1 methods), impl Deref for ReentrantLockGuard < '_ , T > (1 methods), impl Drop for ReentrantLockGuard < '_ , T > (1 methods)

/// the guarded data.
#[must_use = "if unused the ReentrantLock will immediately unlock"]
#[unstable(feature = "reentrant_lock", issue = "121440")]
pub struct ReentrantLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a ReentrantLock<T>,
}

```

**Entity:** ReentrantLockGuard<'a, T>

**States:** NotHeld, Held

**Transitions:**
- NotHeld -> Held via ReentrantLock::<T>::lock()/try_lock() (implied by guard type and module note)
- Held -> NotHeld via Drop for ReentrantLockGuard (implied by module note)

**Evidence:** struct field `lock: &'a ReentrantLock<T>` indicates the guard is tied to a lock instance and its lifetime; `#[must_use = "if unused the ReentrantLock will immediately unlock"]` explicitly documents a temporal/lifecycle requirement: the guard must be kept/used to keep the lock held; module note: `impl Drop for ReentrantLockGuard<'_, T> (1 methods)` indicates unlocking behavior is performed on drop

**Implementation:** Model access to the protected data as requiring a linear capability produced by locking (e.g., `fn lock(&self) -> LockToken<'_>`), and only expose operations that need mutual exclusion as methods taking `&LockToken`/`LockToken` rather than relying on 'keep the guard around' as an informal rule. (RAII Drop already handles unlock; capability-typed APIs can further prevent accidentally performing protected operations without an active guard.)

---

### 72. MappedRwLockWriteGuard validity protocol (Locked+Mapped+Poison-tracked -> Dropped)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-16`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: MappedRwLockWriteGuard is a scoped capability that is only valid while it is alive: it implies an underlying sys::RwLock write lock is held, a mapping has been applied so that `data: NonNull<T>` points into some subfield/region of the protected value, and poisoning is being tracked via `poison_flag`/`poison`. After Drop, the lock is released and the `data` pointer must no longer be dereferenced. The type encodes the lifetime 'a for the lock/flag references, but not the stronger invariant that `data` is derived from the locked value and remains valid exactly until Drop (it is just a raw NonNull). This relies on construction-time discipline and Drop-based protocol rather than a type-level witness tying `data` to the underlying protected allocation and mapping.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLock, impl Send for RwLock < T > (0 methods), impl Sync for RwLock < T > (0 methods), impl Send for RwLockReadGuard < '_ , T > (0 methods), impl Sync for RwLockReadGuard < '_ , T > (0 methods), impl Send for RwLockWriteGuard < '_ , T > (0 methods), impl Sync for RwLockWriteGuard < '_ , T > (0 methods), impl Send for MappedRwLockReadGuard < '_ , T > (0 methods), impl Sync for MappedRwLockReadGuard < '_ , T > (0 methods), impl Send for MappedRwLockWriteGuard < '_ , T > (0 methods), impl Sync for MappedRwLockWriteGuard < '_ , T > (0 methods), impl RwLock < T > (4 methods), impl RwLock < T > (8 methods), impl From < T > for RwLock < T > (1 methods), impl RwLockReadGuard < 'rwlock , T > (1 methods), impl RwLockWriteGuard < 'rwlock , T > (1 methods), impl Deref for RwLockReadGuard < '_ , T > (1 methods), impl Deref for RwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for RwLockWriteGuard < '_ , T > (1 methods), impl Deref for MappedRwLockReadGuard < '_ , T > (1 methods), impl Deref for MappedRwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for MappedRwLockWriteGuard < '_ , T > (1 methods), impl Drop for RwLockReadGuard < '_ , T > (1 methods), impl Drop for RwLockWriteGuard < '_ , T > (1 methods), impl Drop for MappedRwLockReadGuard < '_ , T > (1 methods), impl Drop for MappedRwLockWriteGuard < '_ , T > (1 methods), impl RwLockReadGuard < 'a , T > (2 methods), impl MappedRwLockReadGuard < 'a , T > (2 methods), impl RwLockWriteGuard < 'a , T > (3 methods), impl MappedRwLockWriteGuard < 'a , T > (2 methods); struct RwLockReadGuard; struct RwLockWriteGuard; struct MappedRwLockReadGuard

                      and cause Future's to not implement `Send`"]
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
#[clippy::has_significant_drop]
pub struct MappedRwLockWriteGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedRwLockWriteGuard` argument doesn't hold uniqueness for its whole scope, only until it drops.
    // `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
    poison_flag: &'a poison::Flag,
    poison: poison::Guard,
    _variance: PhantomData<&'a mut T>,
}

```

**Entity:** MappedRwLockWriteGuard<'a, T>

**States:** Alive (lock held; data pointer valid for mapped region), Dropped (lock released; data pointer must not be used)

**Transitions:**
- Alive (lock held; data pointer valid for mapped region) -> Dropped (lock released; data pointer must not be used) via Drop

**Evidence:** field `data: NonNull<T>` stores a raw pointer instead of `&'a mut T`, so validity/derivation from the locked data is not expressed in the type; comment: "we use a pointer instead of `&'a mut T` to avoid `noalias` violations" and "doesn't hold uniqueness for its whole scope, only until it drops" indicates a temporal uniqueness/validity requirement tied to Drop; field `inner_lock: &'a sys::RwLock` implies the guard’s meaning depends on holding/releasing an OS/impl lock during its lifetime; fields `poison_flag: &'a poison::Flag` and `poison: poison::Guard` show a runtime poisoning protocol is coupled to the guard’s lifecycle, not enforced as a separate type state; field `_variance: PhantomData<&'a mut T>` is used to correct variance/invariance, highlighting that aliasing/uniqueness constraints are being managed indirectly rather than as an explicit capability type

**Implementation:** Represent the mapped write access as a capability tied to the lock via a dedicated witness type (e.g., `struct WriteLockToken<'a> { _priv: (), lock: &'a sys::RwLock }`) that must be consumed to create a mapped guard, and store `data` as `NonNull<T>` plus a private provenance marker tying it to the token (e.g., `PhantomData<&'a mut WriteLockToken<'a>>`). This makes it impossible (outside the module) to fabricate a `MappedRwLockWriteGuard` with an arbitrary `NonNull<T>` not derived from the locked value, and clarifies that the pointer is only usable while the capability (guard) is alive.

---

### 60. RwLockReadGuard borrow-validity & lock-held protocol (Valid pointer / Lock held until drop)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: RwLockReadGuard implicitly represents the state of 'a read lock is currently held on inner_lock and data points to the protected T'. This is only upheld by convention and Drop semantics elsewhere in the module: while the guard is alive, the sys::RwLock must remain in a locked-for-read state and the NonNull<T> must remain valid and refer to the same protected allocation. After the guard is dropped, the lock is released and the pointer must no longer be dereferenced, but the type still contains a raw pointer (NonNull<T>) rather than an &'a T, so the compiler cannot enforce aliasing/immutability and relies on the guard's lifecycle to uphold safety. The comment explicitly indicates a non-standard aliasing/validity contract ('only until it drops'), which is a temporal invariant not represented in the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLock, impl Send for RwLock < T > (0 methods), impl Sync for RwLock < T > (0 methods), impl Send for RwLockReadGuard < '_ , T > (0 methods), impl Sync for RwLockReadGuard < '_ , T > (0 methods), impl Send for RwLockWriteGuard < '_ , T > (0 methods), impl Sync for RwLockWriteGuard < '_ , T > (0 methods), impl Send for MappedRwLockReadGuard < '_ , T > (0 methods), impl Sync for MappedRwLockReadGuard < '_ , T > (0 methods), impl Send for MappedRwLockWriteGuard < '_ , T > (0 methods), impl Sync for MappedRwLockWriteGuard < '_ , T > (0 methods), impl RwLock < T > (4 methods), impl RwLock < T > (8 methods), impl From < T > for RwLock < T > (1 methods), impl RwLockReadGuard < 'rwlock , T > (1 methods), impl RwLockWriteGuard < 'rwlock , T > (1 methods), impl Deref for RwLockReadGuard < '_ , T > (1 methods), impl Deref for RwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for RwLockWriteGuard < '_ , T > (1 methods), impl Deref for MappedRwLockReadGuard < '_ , T > (1 methods), impl Deref for MappedRwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for MappedRwLockWriteGuard < '_ , T > (1 methods), impl Drop for RwLockReadGuard < '_ , T > (1 methods), impl Drop for RwLockWriteGuard < '_ , T > (1 methods), impl Drop for MappedRwLockReadGuard < '_ , T > (1 methods), impl Drop for MappedRwLockWriteGuard < '_ , T > (1 methods), impl RwLockReadGuard < 'a , T > (2 methods), impl MappedRwLockReadGuard < 'a , T > (2 methods), impl RwLockWriteGuard < 'a , T > (3 methods), impl MappedRwLockWriteGuard < 'a , T > (2 methods); struct RwLockWriteGuard; struct MappedRwLockReadGuard; struct MappedRwLockWriteGuard

#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLockReadGuard")]
pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `RwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it drops.
    // `NonNull` is also covariant over `T`, just like we would have with `&T`. `NonNull`
    // is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
}

```

**Entity:** RwLockReadGuard<'a, T>

**States:** LockHeld+PointerValid, Dropped/Unlocked (pointer invalid)

**Transitions:**
- LockHeld+PointerValid -> Dropped/Unlocked (pointer invalid) via Drop (implied by guard type and comment about validity 'until it drops')

**Evidence:** field: data: NonNull<T> (raw pointer validity must be maintained externally; not an &'a T reference); field: inner_lock: &'a sys::RwLock (guard ties to a lock object; semantics depend on lock being held while guard lives); comment: "a `RwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it drops" (explicit temporal/aliasing protocol); comment: "we use a pointer instead of `&'a T` to avoid `noalias` violations" (relies on a discipline not expressible with a normal reference type)

**Implementation:** Model the right-to-deref as a non-cloneable capability tied to the guard's lifetime, e.g., keep NonNull<T> private and only expose safe accessors that require &RwLockReadGuard (capability token) to produce references, and consider encoding the 'lock is held' proof as an internal ZST token (e.g., struct ReadHeld<'a>(&'a sys::RwLock)) stored in the guard so all deref paths require the token. This makes it harder to accidentally leak/duplicate the raw pointer and more explicitly ties pointer usage to the lock-held capability.

---

### 59. RwLock write-lock + poison token protocol (HeldWriteLock / Released + PoisonTracking)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: RwLockWriteGuard represents the runtime state of holding an exclusive write lock on a particular RwLock<T> and carrying a poison::Guard token used to record/propagate poisoning. While the guard value exists, the lock is considered write-held; when the guard is dropped it releases the lock and updates poison state. This is primarily enforced via RAII, but there is an additional implicit protocol: the poison::Guard must correspond to the same lock instance and must be 'active' exactly while the write lock is held. The type system does not encode (1) that the poison token is tied to this exact lock instance, nor (2) that a guard is only constructible from a successful write-lock acquisition path (as opposed to being fabricated inside the module). Those invariants are maintained by module-level privacy and constructor discipline rather than by an explicit typestate/capability relationship between the lock and poison token.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLock, impl Send for RwLock < T > (0 methods), impl Sync for RwLock < T > (0 methods), impl Send for RwLockReadGuard < '_ , T > (0 methods), impl Sync for RwLockReadGuard < '_ , T > (0 methods), impl Send for RwLockWriteGuard < '_ , T > (0 methods), impl Sync for RwLockWriteGuard < '_ , T > (0 methods), impl Send for MappedRwLockReadGuard < '_ , T > (0 methods), impl Sync for MappedRwLockReadGuard < '_ , T > (0 methods), impl Send for MappedRwLockWriteGuard < '_ , T > (0 methods), impl Sync for MappedRwLockWriteGuard < '_ , T > (0 methods), impl RwLock < T > (4 methods), impl RwLock < T > (8 methods), impl From < T > for RwLock < T > (1 methods), impl RwLockReadGuard < 'rwlock , T > (1 methods), impl RwLockWriteGuard < 'rwlock , T > (1 methods), impl Deref for RwLockReadGuard < '_ , T > (1 methods), impl Deref for RwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for RwLockWriteGuard < '_ , T > (1 methods), impl Deref for MappedRwLockReadGuard < '_ , T > (1 methods), impl Deref for MappedRwLockWriteGuard < '_ , T > (1 methods), impl DerefMut for MappedRwLockWriteGuard < '_ , T > (1 methods), impl Drop for RwLockReadGuard < '_ , T > (1 methods), impl Drop for RwLockWriteGuard < '_ , T > (1 methods), impl Drop for MappedRwLockReadGuard < '_ , T > (1 methods), impl Drop for MappedRwLockWriteGuard < '_ , T > (1 methods), impl RwLockReadGuard < 'a , T > (2 methods), impl MappedRwLockReadGuard < 'a , T > (2 methods), impl RwLockWriteGuard < 'a , T > (3 methods), impl MappedRwLockWriteGuard < 'a , T > (2 methods); struct RwLockReadGuard; struct MappedRwLockReadGuard; struct MappedRwLockWriteGuard

#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLockWriteGuard")]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
    poison: poison::Guard,
}

```

**Entity:** RwLockWriteGuard<'a, T>

**States:** HeldWriteLock (active guard), Released (dropped/consumed guard)

**Transitions:**
- HeldWriteLock -> Released via Drop for RwLockWriteGuard (mentioned in module header comment)

**Evidence:** struct field `lock: &'a RwLock<T>` ties the guard to a specific lock by reference; struct field `poison: poison::Guard` indicates an additional runtime protocol/state tracked alongside the lock hold; module header comment lists `impl Drop for RwLockWriteGuard<'_, T> (1 methods)`, implying release/poison-update occurs on drop and is part of the lifecycle

**Implementation:** Make the poison token a capability parameterized by the specific lock borrow, e.g. `struct PoisonGuard<'a> { _lock: PhantomData<&'a ()>, ... }` (or tie it to `&'a RwLock<T>`), and require `RwLock::write()` to return `(WriteGuard<'a, T>, PoisonGuard<'a>)` or store a `PoisonGuard<'a>` inside the write guard. This makes it impossible (even within the crate) to mix poison tokens across locks or to construct a write guard without coming from the acquisition function that mints the correct capability.

---

### 8. Sender disconnect lifecycle (Connected -> Disconnected via Drop/release)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-337`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: Dropping the last Sender transitions the underlying channel into a disconnected state by calling `disconnect_senders()` (Array/List) or `disconnect()` (Zero). Sending operations thereafter fail with disconnection errors (e.g., `SendTimeoutError::Disconnected` mapped into `SendError`). This lifecycle (validity of sending depends on whether send-side is still connected) is enforced dynamically through Drop side effects and runtime error returns; the type system does not distinguish a Sender that is guaranteed connected from one that may already be disconnected due to other clones being dropped.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// assert_eq!(3, msg + msg2);
/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
pub struct Sender<T> {
    flavor: SenderFlavor<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Send for Sender<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Sync for Sender<T> {}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> UnwindSafe for Sender<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> RefUnwindSafe for Sender<T> {}

impl<T> Sender<T> {
    /// Attempts to send a message into the channel without blocking.
    ///
    /// This method will either send a message into the channel immediately or return an error if
    /// the channel is full or disconnected. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will send the message only if there
    /// happens to be a receive operation on the other side of the channel at the same time.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::{channel, Receiver, Sender};
    ///
    /// let (sender, _receiver): (Sender<i32>, Receiver<i32>) = channel();
    ///
    /// assert!(sender.try_send(1).is_ok());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.try_send(msg),
            SenderFlavor::List(chan) => chan.try_send(msg),
            SenderFlavor::Zero(chan) => chan.try_send(msg),
        }
    }

    /// Attempts to send a value on this channel, returning it back if it could
    /// not be sent.
    ///
    /// A successful send occurs when it is determined that the other end of
    /// the channel has not hung up already. An unsuccessful send would be one
    /// where the corresponding receiver has already been deallocated. Note
    /// that a return value of [`Err`] means that the data will never be
    /// received, but a return value of [`Ok`] does *not* mean that the data
    /// will be received. It is possible for the corresponding receiver to
    /// hang up immediately after this function returns [`Ok`]. However, if
    /// the channel is zero-capacity, it acts as a rendezvous channel and a
    /// return value of [`Ok`] means that the data has been received.
    ///
    /// If the channel is full and not disconnected, this call will block until
    /// the send operation can proceed. If the channel becomes disconnected,
    /// this call will wake up and return an error. The returned error contains
    /// the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive
    /// operation to appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    ///
    /// let (tx, rx) = channel();
    ///
    /// // This send is always successful
    /// tx.send(1).unwrap();
    ///
    /// // This send will fail because the receiver is gone
    /// drop(rx);
    /// assert!(tx.send(1).is_err());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.send(msg, None),
            SenderFlavor::List(chan) => chan.send(msg, None),
            SenderFlavor::Zero(chan) => chan.send(msg, None),
        }
        .map_err(|err| match err {
            SendTimeoutError::Disconnected(msg) => SendError(msg),
            SendTimeoutError::Timeout(_) => unreachable!(),
        })
    }
}

impl<T> Sender<T> {
    /// Waits for a message to be sent into the channel, but only for a limited time.
    ///
    /// If the channel is full and not disconnected, this call will block until the send operation
    /// can proceed or the operation times out. If the channel becomes disconnected, this call will
    /// wake up and return an error. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive operation to
    /// appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::time::Duration;
    ///
    /// let (tx, rx) = channel();
    ///
    /// tx.send_timeout(1, Duration::from_millis(400)).unwrap();
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send_timeout(&self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        match Instant::now().checked_add(timeout) {
            Some(deadline) => self.send_deadline(msg, deadline),
            // So far in the future that it's practically the same as waiting indefinitely.
            None => self.send(msg).map_err(SendTimeoutError::from),
        }
    }

    /// Waits for a message to be sent into the channel, but only until a given deadline.
    ///
    /// If the channel is full and not disconnected, this call will block until the send operation
    /// can proceed or the operation times out. If the channel becomes disconnected, this call will
    /// wake up and return an error. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive operation to
    /// appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::time::{Duration, Instant};
    ///
    /// let (tx, rx) = channel();
    ///
    /// let t = Instant::now() + Duration::from_millis(400);
    /// tx.send_deadline(1, t).unwrap();
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send_deadline(&self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.send(msg, Some(deadline)),
            SenderFlavor::List(chan) => chan.send(msg, Some(deadline)),
            SenderFlavor::Zero(chan) => chan.send(msg, Some(deadline)),
        }
    }

    /// Returns `true` if the channel is empty.
    ///
    /// Note: Zero-capacity channels are always empty.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::channel();
    ///
    /// let tx1 = send.clone();
    /// let tx2 = send.clone();
    ///
    /// assert!(tx1.is_empty());
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(!tx1.is_empty());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_empty(&self) -> bool {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.is_empty(),
            SenderFlavor::List(chan) => chan.is_empty(),
            SenderFlavor::Zero(chan) => chan.is_empty(),
        }
    }

    /// Returns `true` if the channel is full.
    ///
    /// Note: Zero-capacity channels are always full.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::sync_channel(1);
    ///
    /// let (tx1, tx2) = (send.clone(), send.clone());
    /// assert!(!tx1.is_full());
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(tx1.is_full());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_full(&self) -> bool {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.is_full(),
            SenderFlavor::List(chan) => chan.is_full(),
            SenderFlavor::Zero(chan) => chan.is_full(),
        }
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::channel();
    /// let (tx1, tx2) = (send.clone(), send.clone());
    ///
    /// assert_eq!(tx1.len(), 0);
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(tx1.len(), 1);
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn len(&self) -> usize {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.len(),
            SenderFlavor::List(chan) => chan.len(),
            SenderFlavor::Zero(chan) => chan.len(),
        }
    }

    /// If the channel is bounded, returns its capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::sync_channel(3);
    /// let (tx1, tx2) = (send.clone(), send.clone());
    ///
    /// assert_eq!(tx1.capacity(), Some(3));
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(tx1.capacity(), Some(3));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn capacity(&self) -> Option<usize> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.capacity(),
            SenderFlavor::List(chan) => chan.capacity(),
            SenderFlavor::Zero(chan) => chan.capacity(),
        }
    }

    /// Returns `true` if senders belong to the same channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    ///
    /// let (tx1, _) = mpmc::channel::<i32>();
    /// let (tx2, _) = mpmc::channel::<i32>();
    ///
    /// assert!(tx1.same_channel(&tx1));
    /// assert!(!tx1.same_channel(&tx2));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn same_channel(&self, other: &Sender<T>) -> bool {
        match (&self.flavor, &other.flavor) {
            (SenderFlavor::Array(a), SenderFlavor::Array(b)) => a == b,
            (SenderFlavor::List(a), SenderFlavor::List(b)) => a == b,
            (SenderFlavor::Zero(a), SenderFlavor::Zero(b)) => a == b,
            _ => false,
        }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        unsafe {
            match &self.flavor {
                SenderFlavor::Array(chan) => chan.release(|c| c.disconnect_senders()),
                SenderFlavor::List(chan) => chan.release(|c| c.disconnect_senders()),
                SenderFlavor::Zero(chan) => chan.release(|c| c.disconnect()),
            }
        }
    }
}

```

**Entity:** Sender<T>

**States:** Connected, Disconnected (send-side closed)

**Transitions:**
- Connected -> Disconnected (send-side closed) via `Drop for Sender<T>` calling `release(...disconnect...)`

**Evidence:** impl `Drop for Sender<T>`: `chan.release(|c| c.disconnect_senders())` for Array/List and `chan.release(|c| c.disconnect())` for Zero; method `send`: maps `SendTimeoutError::Disconnected(msg)` into `SendError(msg)` (explicit disconnected-state failure mode); docs on `send`: "send will fail because the receiver is gone" after `drop(rx)` (connection status affects validity)

**Implementation:** Introduce an explicit channel capability/token representing a live endpoint, e.g. `ConnectedSender<T, F>` created alongside a `ChannelHandle`/`Endpoints` object that owns connectivity. Operations requiring connectivity take `&ConnectedSender` (or a `SendPermit`) obtained from the handle; dropping the handle transitions to disconnected and makes it impossible to obtain permits. (This cannot eliminate all races in MPMC, but can make some 'definitely disconnected' states unrepresentable in single-threaded ownership paths.)

---

### 79. MutexGuard ownership protocol (Locked guard must be dropped on locking thread; drop performs ordered poison+unlock)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/mutex.rs:1-498`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: A `MutexGuard` represents the unique right to access the protected `T` while the underlying OS mutex is locked. Correctness relies on an implicit protocol: (1) `MutexGuard::new` is only called after `inner.lock()` / successful `try_lock()`; (2) the guard must eventually be dropped to unlock; (3) drop must run on the same thread to satisfy platform requirements; and (4) drop performs an ordered sequence (poison bookkeeping then unlock). The type system enforces some of this (RAII unlock, `!Send`), but the key precondition “guard can only be constructed when the mutex is actually locked” is only upheld by `unsafe` call discipline rather than being unrepresentable to misuse within the module.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct MutexGuard; struct MappedMutexGuard

///
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Mutex")]
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

// ... (other code) ...

///
/// [`into_inner`]: Mutex::into_inner
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}


// ... (other code) ...

///
/// [`Rc`]: crate::rc::Rc
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}


// ... (other code) ...

/// For this reason, [`MutexGuard`] must not implement `Send` to prevent it being dropped from
/// another thread.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

/// `T` must be `Sync` for a [`MutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
#[stable(feature = "mutexguard", since = "1.19.0")]
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedMutexGuard<'_, T> {}
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedMutexGuard<'_, T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> Mutex<T> {
        Mutex { inner: sys::Mutex::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.lock() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// mutex.set(11).unwrap();
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.lock() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11).unwrap(), 7);
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.lock() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    /// The exact behavior on locking a mutex in the thread which already holds
    /// the lock is left unspecified. However, this function will not return on
    /// the second call (it might panic or deadlock, for example).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error once the mutex is acquired. The acquired
    /// mutex guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock().unwrap() = 10;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        unsafe {
            self.inner.lock();
            MutexGuard::new(self)
        }
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`Err`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return the [`Poisoned`] error if the mutex would
    /// otherwise be acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     let mut lock = c_mutex.try_lock();
    ///     if let Ok(ref mut mutex) = lock {
    ///         **mutex = 10;
    ///     } else {
    ///         println!("try_lock failed");
    ///     }
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        unsafe {
            if self.inner.try_lock() {
                Ok(MutexGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the mutex is poisoned.
    ///
    /// If another thread is active, the mutex can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    /// assert_eq!(mutex.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a mutex.
    ///
    /// If the mutex is poisoned, it will remain poisoned until this function is called. This
    /// allows recovering from a poisoned state and marking that it has recovered. For example, if
    /// the value is overwritten by a known-good value, then the mutex can be marked as
    /// un-poisoned. Or possibly, the value could be inspected to determine if it is in a
    /// consistent state, and if so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    ///
    /// assert_eq!(mutex.is_poisoned(), true);
    /// let x = mutex.lock().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     mutex.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(mutex.is_poisoned(), false);
    /// assert_eq!(*x, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the underlying data
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner().unwrap(), 0);
    /// ```
    #[stable(feature = "mutex_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previous abandoned locks
    /// (e.g., via [`forget()`] on a [`MutexGuard`]).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing a mutable reference to the
    /// underlying data instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut().unwrap() = 10;
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    ///
    /// [`forget()`]: mem::forget
    #[stable(feature = "mutex_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

#[stable(feature = "mutex_from", since = "1.24.0")]
impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

// ... (other code) ...

    }
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    unsafe fn new(lock: &'mutex Mutex<T>) -> LockResult<MutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.guard(), |guard| MutexGuard { lock, poison: guard })
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock();
        }
    }
}

// ... (other code) ...

    &guard.lock.poison
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(orig: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedMutexGuard {
            data,
            inner: &orig.lock.inner,
            poison_flag: &orig.lock.poison,
            poison: orig.poison.clone(),
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::filter_map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn filter_map<U, F>(orig: Self, f: F) -> Result<MappedMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedMutexGuard {
                    data,
                    inner: &orig.lock.inner,
                    poison_flag: &orig.lock.poison,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            }
            None => Err(orig),
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl
// ... (truncated) ...
```

**Entity:** MutexGuard<'_, T>

**States:** HoldingLock, Released

**Transitions:**
- HoldingLock -> Released via Drop for MutexGuard (poison.done then inner.unlock)

**Evidence:** method: `pub fn lock(&self)` does `self.inner.lock(); MutexGuard::new(self)` (ordering requirement: lock before new); method: `pub fn try_lock(&self)` does `if self.inner.try_lock() { Ok(MutexGuard::new(self)?) }` (construction depends on successful try_lock); constructor: `unsafe fn new(lock: &Mutex<T>) -> LockResult<MutexGuard<...>>` (unsafe indicates a precondition not expressed in types); Drop impl: `self.lock.poison.done(&self.poison); self.lock.inner.unlock();` (ordered teardown protocol); comment: "MutexGuard must not implement Send to prevent it being dropped from another thread." and the impl `impl<T: ?Sized> !Send for MutexGuard<'_, T> {}`

**Implementation:** Within the module, model successful acquisition as an unforgeable capability token returned by `sys::Mutex::lock/try_lock` (e.g., `struct Held<'a>(&'a sys::Mutex);`). Then make `MutexGuard::new` take that token (`new(lock: &'a Mutex<T>, held: Held<'a>)`) so construction is only possible when the lock has been acquired, eliminating the latent “must call inner.lock() first” unsafe protocol.

---

### 11. Block linked-list lifecycle & safe reclamation protocol (Allocated/Linked/Destroyed)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-57`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Block<T> participates in an implicit lifecycle: it is allocated zero-initialized, then eventually linked into a list by storing a non-null `next` pointer, and finally reclaimed via a cooperative destruction protocol coordinated through per-slot atomic state bits (READ/DESTROY). Correctness relies on (a) `next` being null until linking, then written exactly once to a valid Block pointer, and (b) destroy() only freeing the block when no threads are still using any slot; otherwise another thread must continue destruction. None of these phases/permissions are represented in the type system: `wait_next()` can spin forever if `next` is never set; `destroy()` is `unsafe` and accepts a raw pointer plus an index, relying on external callers to uphold ordering and uniqueness (e.g., that `this` is a live Block and that destruction is initiated at the correct slot).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Position; struct ListToken; struct Channel, impl Channel < T > (17 methods), impl Drop for Channel < T > (1 methods)

/// A block in a linked list.
///
/// Each block in the list can hold up to `BLOCK_CAP` messages.
struct Block<T> {
    /// The next block in the linked list.
    next: Atomic<*mut Block<T>>,

    /// Slots for messages.
    slots: [Slot<T>; BLOCK_CAP],
}

impl<T> Block<T> {
    /// Creates an empty block.
    fn new() -> Box<Block<T>> {
        // SAFETY: This is safe because:
        //  [1] `Block::next` (Atomic<*mut _>) may be safely zero initialized.
        //  [2] `Block::slots` (Array) may be safely zero initialized because of [3, 4].
        //  [3] `Slot::msg` (UnsafeCell) may be safely zero initialized because it
        //       holds a MaybeUninit.
        //  [4] `Slot::state` (Atomic<usize>) may be safely zero initialized.
        unsafe { Box::new_zeroed().assume_init() }
    }

    /// Waits until the next pointer is set.
    fn wait_next(&self) -> *mut Block<T> {
        let backoff = Backoff::new();
        loop {
            let next = self.next.load(Ordering::Acquire);
            if !next.is_null() {
                return next;
            }
            backoff.spin_heavy();
        }
    }

    /// Sets the `DESTROY` bit in slots starting from `start` and destroys the block.
    unsafe fn destroy(this: *mut Block<T>, start: usize) {
        // It is not necessary to set the `DESTROY` bit in the last slot because that slot has
        // begun destruction of the block.
        for i in start..BLOCK_CAP - 1 {
            let slot = unsafe { (*this).slots.get_unchecked(i) };

            // Mark the `DESTROY` bit if a thread is still using the slot.
            if slot.state.load(Ordering::Acquire) & READ == 0
                && slot.state.fetch_or(DESTROY, Ordering::AcqRel) & READ == 0
            {
                // If a thread is still using the slot, it will continue destruction of the block.
                return;
            }
        }

        // No thread is using the block, now it is safe to destroy it.
        drop(unsafe { Box::from_raw(this) });
    }
}

```

**Entity:** Block<T>

**States:** Allocated (unlinked, next == null), Linked (next set to non-null), Destruction in progress (slot DESTROY propagation), Destroyed (freed)

**Transitions:**
- Allocated (unlinked) -> Linked via writing `next` (observed by wait_next())
- Linked -> Destruction in progress via unsafe destroy(this, start) setting DESTROY bits
- Destruction in progress -> Destroyed via drop(Box::from_raw(this)) when no slot is in READ use
- Destruction in progress -> (remains) Destruction in progress when a slot is still READ-used (early return to let another thread continue)

**Evidence:** field `next: Atomic<*mut Block<T>>` is a nullable raw pointer encoding 'linked vs unlinked'; Block::new(): uses `Box::new_zeroed().assume_init()` with SAFETY comment requiring invariants about zero-init layout; wait_next(): loops until `self.next.load(Ordering::Acquire)` is non-null; `backoff.spin_heavy()` implies an expected eventual transition; destroy(this, start): comment 'slot has begun destruction of the block' and loop setting `DESTROY` bit in slots; destroy(): checks `slot.state.load(...) & READ == 0` and `fetch_or(DESTROY, ...)` to coordinate with concurrent readers, returning early if a thread is still using the slot; destroy(): `drop(Box::from_raw(this))` frees memory based on raw pointer validity and uniqueness assumptions

**Implementation:** Represent lifecycle as distinct types: e.g., `Block<Unlinked, T>` with `next` absent, transitioning to `Block<Linked, T>` via a `link(self, next: NonNull<Block<Linked,T>>) -> Block<Linked,T>`. For reclamation, pass a linear/unique capability (token) to call `destroy`, e.g. `fn begin_destroy(self: NonNull<Block<Linked,T>>, start: StartIndexToken) -> DestroyingBlock<T>`, where `DestroyingBlock` owns the pointer and only allows final `free()` when a guard/capability proves no READ users remain. Also replace `*mut Block<T>` with `Option<NonNull<Block<T>>>` to encode non-null after linking.

---

### 71. MappedMutexGuard borrow/lock lifetime protocol (Locked & Mapped until Drop)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/mutex.rs:1-16`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: MappedMutexGuard represents an active mutex lock plus a derived/mapped view into the protected data. While the guard is Active, `data: NonNull<T>` must point into the mutex-protected allocation and be dereferenceable, and `inner: &'a sys::Mutex` must remain locked. After Drop, the lock is released and `data` must not be used. This is a lifecycle/state protocol: the validity of the raw pointer and the exclusivity window are tied to the guard's drop, but `NonNull<T>` itself does not encode 'only dereference while locked' at the type level—correctness relies on the guard not being leaked/forgotten and on internal invariants upheld by the implementation.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Mutex, 2 free function(s), impl Send for Mutex < T > (0 methods), impl Sync for Mutex < T > (0 methods), impl Send for MutexGuard < '_ , T > (0 methods), impl Sync for MutexGuard < '_ , T > (0 methods), impl Send for MappedMutexGuard < '_ , T > (0 methods), impl Sync for MappedMutexGuard < '_ , T > (0 methods), impl Mutex < T > (4 methods), impl Mutex < T > (6 methods), impl From < T > for Mutex < T > (1 methods), impl MutexGuard < 'mutex , T > (1 methods), impl Deref for MutexGuard < '_ , T > (1 methods), impl DerefMut for MutexGuard < '_ , T > (1 methods), impl Drop for MutexGuard < '_ , T > (1 methods), impl MutexGuard < 'a , T > (2 methods), impl Deref for MappedMutexGuard < '_ , T > (1 methods), impl DerefMut for MappedMutexGuard < '_ , T > (1 methods), impl Drop for MappedMutexGuard < '_ , T > (1 methods), impl MappedMutexGuard < 'a , T > (2 methods); struct MutexGuard

                      and cause Futures to not implement `Send`"]
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
#[clippy::has_significant_drop]
pub struct MappedMutexGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedMutexGuard` argument doesn't hold uniqueness for its whole scope, only until it drops.
    // `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner: &'a sys::Mutex,
    poison_flag: &'a poison::Flag,
    poison: poison::Guard,
    _variance: PhantomData<&'a mut T>,
}

```

**Entity:** MappedMutexGuard<'a, T>

**States:** Active (lock held, mapping valid), Dropped (lock released, mapping invalid)

**Transitions:**
- Active -> Dropped via Drop for MappedMutexGuard

**Evidence:** field `data: NonNull<T>`: raw pointer requires external validity invariant (points to the locked data) and cannot on its own enforce lifetime/lock coupling; field `inner: &'a sys::Mutex`: indicates the guard is tied to a specific mutex instance and (implicitly) its lock state while the guard lives; comment: "MappedMutexGuard argument doesn't hold uniqueness for its whole scope, only until it drops." (explicit temporal/lifecycle invariant tied to Drop); field `_variance: PhantomData<&'a mut T>` and comment about avoiding `noalias` violations: indicates reliance on subtle aliasing/uniqueness rules not fully represented by the raw-pointer field alone

**Implementation:** Represent the right-to-deref the mapped pointer as a non-cloneable capability tied to the lock state, e.g. store an internal `LockToken<'a>` (created only by successful lock acquisition) inside `MappedMutexGuard` and require that token to produce `&T/&mut T` from `NonNull<T>`. This keeps the 'may deref only while locked' permission explicit and non-forgeable, rather than relying on raw-pointer validity plus Drop timing.

---

### 14. Disconnect lifecycle & single-caller safety for disconnect_receivers (Connected -> Disconnected) plus post-disconnect cleanup

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-445`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Disconnection is encoded by setting `mark_bit` in `tail`. Multiple methods assume a precise lifecycle: once disconnected, send/recv reservations treat the channel as disconnected and must wake the opposing side. Additionally, `disconnect_receivers` has a strong single-caller/last-receiver precondition and performs message discarding based on a snapshot of `tail`; this is only documented and marked `unsafe`, not enforced by types. The correctness depends on: (1) `disconnect_receivers` called exactly once when the last receiver is dropped, (2) all other receivers' destruction observed with Acquire-or-stronger ordering, and (3) callers not racing this cleanup in ways that violate the intended shutdown protocol.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot; struct ArrayToken

}

/// Bounded channel based on a preallocated array.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit in the head is always zero.
    ///
    /// Messages are popped from the head of the channel.
    head: CachePadded<Atomic<usize>>,

    /// The tail of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit indicates that the channel is disconnected.
    ///
    /// Messages are pushed into the tail of the channel.
    tail: CachePadded<Atomic<usize>>,

    /// The buffer holding slots.
    buffer: Box<[Slot<T>]>,

    /// The channel capacity.
    cap: usize,

    /// A stamp with the value of `{ lap: 1, mark: 0, index: 0 }`.
    one_lap: usize,

    /// If this bit is set in the tail, that means the channel is disconnected.
    mark_bit: usize,

    /// Senders waiting while the channel is full.
    senders: SyncWaker,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,
}

impl<T> Channel<T> {
    /// Creates a bounded channel of capacity `cap`.
    pub(crate) fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be positive");

        // Compute constants `mark_bit` and `one_lap`.
        let mark_bit = (cap + 1).next_power_of_two();
        let one_lap = mark_bit * 2;

        // Head is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let head = 0;
        // Tail is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let tail = 0;

        // Allocate a buffer of `cap` slots initialized
        // with stamps.
        let buffer: Box<[Slot<T>]> = (0..cap)
            .map(|i| {
                // Set the stamp to `{ lap: 0, mark: 0, index: i }`.
                Slot { stamp: AtomicUsize::new(i), msg: UnsafeCell::new(MaybeUninit::uninit()) }
            })
            .collect();

        Channel {
            buffer,
            cap,
            one_lap,
            mark_bit,
            head: CachePadded::new(AtomicUsize::new(head)),
            tail: CachePadded::new(AtomicUsize::new(tail)),
            senders: SyncWaker::new(),
            receivers: SyncWaker::new(),
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            // Check if the channel is disconnected.
            if tail & self.mark_bit != 0 {
                token.array.slot = ptr::null();
                token.array.stamp = 0;
                return true;
            }

            // Deconstruct the tail.
            let index = tail & (self.mark_bit - 1);
            let lap = tail & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                let new_tail = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    tail + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail,
                    new_tail,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `write`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = tail + 1;
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        tail = self.tail.load(Ordering::Relaxed);
                    }
                }
            } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
                atomic::fence(Ordering::SeqCst);
                let head = self.head.load(Ordering::Relaxed);

                // If the head lags one lap behind the tail as well...
                if head.wrapping_add(self.one_lap) == tail {
                    // ...then the channel is full.
                    return false;
                }

                backoff.spin_light();
                tail = self.tail.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.array.slot.is_null() {
            return Err(msg);
        }

        // Write the message into the slot and update the stamp.
        unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.stamp.store(token.array.stamp, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `read`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = head.wrapping_add(self.one_lap);
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        head = self.head.load(Ordering::Relaxed);
                    }
                }
            } else if stamp == head {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if (tail & !self.mark_bit) == head {
                    // If the channel is disconnected...
                    if tail & self.mark_bit != 0 {
                        // ...then receive an error.
                        token.array.slot = ptr::null();
                        token.array.stamp = 0;
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                backoff.spin_light();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.array.slot.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message from the slot and update the stamp.
        let msg = unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);

            let msg = slot.msg.get().read().assume_init();
            slot.stamp.store(token.array.stamp, Ordering::Release);
            msg
        };

        // Wake a sleeping sender.
        self.senders.notify();
        Ok(msg)
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        let token = &mut Token::default();
        if self.start_send(token) {
            unsafe { self.write(token, msg).map_err(TrySendError::Disconnected) }
        } else {
            Err(TrySendError::Full(msg))
        }
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        loop {
            // Try sending a message.
            if self.start_send(token) {
                let res = unsafe { self.write(token, msg) };
                return res.map_err(SendTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(SendTimeoutError::Timeout(msg));
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a receiver wakes us up.
                let oper = Operation::hook(token);
                self.senders.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_full() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.senders.unregister(oper).unwrap();
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            // Try receiving a message.
            if self.start_recv(token) {
                let res = unsafe { self.read(token) };
                return res.map_err(|_| RecvTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a sender wakes us up.
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail, then load the head.
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            // If the tail didn't change, we've got consistent values to work with.
            if self.tail.load(Ordering::SeqCst) == tail {
                let hix = head & (self.mark_bit - 1);
                let tix = tail & (self.mark_bit - 1);

                return if hix < tix {
                    tix - hix
                } else if hix > tix {
                    self.cap - hix + tix
                } else if (tail & !self.mark_bit) == head {
                    0
                } else {
                    self.cap
                };
            }
        }
    }

    /// Returns the capacity of the channel.
    #[allow(clippy::unnecessary_wraps)] // This is intentional.
    pub(crate) fn capacity(&self) -> Option<usize> {
        Some(self.cap)
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);

        if tail & self.mark_bit == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers and wakes up all blocked senders.
    ///
    /// Returns `true` if this call disconnected the channel.
    ///
    /// # Safety
    /// May only be called once upon dropping the last receiver. The
    /// destruction of all other receivers must have been observed with acquire
    /// ordering or stronger.
    pub(crate) unsafe fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);
        let disconnected = if tail & self.mark_bit == 0 {
            self.senders.disconnect();
            true
        } else {
            false
        };

        unsafe { self.discard_all_messages(tail) };
        disconnected
    }

    /// Discards all messages.
    ///
    /// `tail` should be the current (and therefore last) value of `tail`.
    ///
    /// # Panicking
    /// If a de
// ... (truncated) ...
```

**Entity:** Channel<T>

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via disconnect_senders() (sets tail mark_bit, wakes receivers)
- Connected -> Disconnected via unsafe disconnect_receivers() (sets tail mark_bit, wakes senders, discards messages)
- Disconnected -> Disconnected via repeated disconnect_* calls (no-op indicated by return false)

**Evidence:** Field: `mark_bit: usize` and comment: `If this bit is set in the tail, that means the channel is disconnected.`; disconnect_senders: `let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);` and `if tail & self.mark_bit == 0 { self.receivers.disconnect(); true } else { false }` (idempotent disconnect transition); disconnect_receivers: marked `pub(crate) unsafe fn disconnect_receivers(&self) -> bool` with Safety comment: `May only be called once upon dropping the last receiver... observed with acquire ordering or stronger.`; disconnect_receivers: calls `unsafe { self.discard_all_messages(tail) };` relying on `tail` being the current/last value (comment: "tail should be the current (and therefore last) value of tail")

**Implementation:** Introduce linear/capability tokens representing the 'last sender' / 'last receiver' authority (e.g., a `ReceiverDropGuard` created only for the final receiver via refcounting type, or an internal `DisconnectToken` that only the owning endpoint can obtain). Make `disconnect_receivers` a safe method that requires this capability, preventing accidental multiple calls and encoding the 'last receiver' condition at the API boundary.

---

### 16. Sender reference-counted lifecycle (Acquired -> Released; disconnect-on-last; destroy-on-both-sides)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/counter.rs:1-50`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Sender<C> is a thin wrapper around a raw pointer to a shared Counter<C>. Correctness relies on an implicit lifecycle: each Sender must be paired with exactly one release(), and after release the Sender must not be used again. Releasing the last sender triggers a disconnect callback, and the Counter may be freed when an additional 'destroy' condition is met. None of this is tracked by the type system because Sender holds a *mut Counter<C> and release() is unsafe and takes &self, allowing double-release and use-after-release unless the caller follows the protocol.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Counter; struct Receiver, impl Receiver < C > (3 methods), impl ops :: Deref for Receiver < C > (1 methods)

}

/// The sending side.
pub(crate) struct Sender<C> {
    counter: *mut Counter<C>,
}

impl<C> Sender<C> {
    /// Returns the internal `Counter`.
    fn counter(&self) -> &Counter<C> {
        unsafe { &*self.counter }
    }

    /// Acquires another sender reference.
    pub(crate) fn acquire(&self) -> Sender<C> {
        let count = self.counter().senders.fetch_add(1, Ordering::Relaxed);

        // Cloning senders and calling `mem::forget` on the clones could potentially overflow the
        // counter. It's very difficult to recover sensibly from such degenerate scenarios so we
        // just abort when the count becomes very large.
        if count > isize::MAX as usize {
            process::abort();
        }

        Sender { counter: self.counter }
    }

    /// Releases the sender reference.
    ///
    /// Function `disconnect` will be called if this is the last sender reference.
    pub(crate) unsafe fn release<F: FnOnce(&C) -> bool>(&self, disconnect: F) {
        if self.counter().senders.fetch_sub(1, Ordering::AcqRel) == 1 {
            disconnect(&self.counter().chan);

            if self.counter().destroy.swap(true, Ordering::AcqRel) {
                drop(unsafe { Box::from_raw(self.counter) });
            }
        }
    }
}

impl<C> ops::Deref for Sender<C> {
    type Target = C;

    fn deref(&self) -> &C {
        &self.counter().chan
    }
}

```

**Entity:** Sender<C>

**States:** Live (refcounted), Released (must not be used), LastSenderReleased (disconnect executed), Destroyed (Counter freed)

**Transitions:**
- Live (refcounted) -> Live (refcounted) via acquire() (increments senders count)
- Live (refcounted) -> Released (must not be used) via unsafe release() (decrements senders count)
- Live (refcounted) -> LastSenderReleased (disconnect executed) via unsafe release() when senders.fetch_sub(...) == 1
- LastSenderReleased -> Destroyed (Counter freed) via unsafe release() when destroy.swap(true, ...) returns true (drop(Box::from_raw(self.counter)))

**Evidence:** field: Sender<C> { counter: *mut Counter<C> } raw pointer implies manual lifetime/state management; method acquire(): self.counter().senders.fetch_add(1, Ordering::Relaxed) indicates runtime refcounting state; comment in acquire(): "Cloning senders ... could potentially overflow the counter" + process::abort() indicates an implicit 'count must not overflow' invariant; method release() is declared unsafe and takes &self: pub(crate) unsafe fn release<...>(&self, ...) — allows double-release/use-after-release unless caller obeys protocol; release(): if self.counter().senders.fetch_sub(1, Ordering::AcqRel) == 1 { disconnect(&self.counter().chan); ... } shows last-sender transition and required ordering; release(): if self.counter().destroy.swap(true, Ordering::AcqRel) { drop(Box::from_raw(self.counter)) } shows conditional destruction/freeing of Counter behind the raw pointer; method counter(): unsafe { &*self.counter } dereferences raw pointer; validity depends on not having been destroyed

**Implementation:** Make Sender own a non-null pointer (NonNull<Counter<C>>) and implement Drop for Sender to perform the fetch_sub/disconnect/destroy logic automatically. Prevent double-release by removing release(&self) from the public surface; if a manual release is needed, make it take self (consuming) so the type system prevents use-after-release. Optionally wrap refcounting in an internal Arc-like type so cloning is safe and overflow is structurally prevented.

---

## State Machine Invariants

### 61. Once exclusive state machine (Incomplete / Poisoned / Complete)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/once.rs:1-10`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ExclusiveState encodes the runtime state of a Once-like initialization: it starts Incomplete, can become Complete when initialization finishes successfully, or Poisoned if initialization panics/aborts mid-flight. Callers elsewhere in the module are expected to branch on this enum to decide whether initialization may run, must be retried, or must fail/propagate poison. The type system does not prevent using an instance in the wrong state (e.g., treating Poisoned as Complete) or enforce valid transitions; correctness relies on runtime control flow and checking this value.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Once, impl UnwindSafe for Once (0 methods), impl RefUnwindSafe for Once (0 methods), impl Once (8 methods), impl OnceState (2 methods); struct OnceState

    pub(crate) inner: sys::OnceState,
}

pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}

```

**Entity:** ExclusiveState

**States:** Incomplete, Poisoned, Complete

**Transitions:**
- Incomplete -> Complete
- Incomplete -> Poisoned
- Poisoned -> Incomplete (if poison is cleared/ignored elsewhere in the module)
- Poisoned -> Complete (if initialization is rerun successfully elsewhere in the module)

**Evidence:** pub(crate) enum ExclusiveState { Incomplete, Poisoned, Complete }

**Implementation:** Model Once as Once<S> with zero-sized states (Incomplete/Poisoned/Complete). Expose only state-appropriate operations: e.g., begin_init() only on Once<Incomplete>, finish_ok(self)->Once<Complete>, finish_panic(self)->Once<Poisoned>. If the API must allow recovery, provide an explicit recover(self)->Once<Incomplete> capability to make poison-clearing a typed transition.

---

### 64. LazyLock initialization/poison state machine (Incomplete / Complete / Poisoned)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/lazy_lock.rs:1-268`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: LazyLock has an implicit runtime state machine driven by `once: Once` that determines which member of `data: UnsafeCell<Data<T, F>>` is currently initialized and may be accessed/dropped. In `Incomplete`, the initializer `f` must be present and `value` must not be read; in `Complete`, `value` must be present and immutable and `f` must not be used again; in `Poisoned`, neither should be accessed and operations typically panic/abort. These invariants are enforced via `Once` runtime state checks, `unsafe` blocks, and `ManuallyDrop` field juggling, not by the type system (i.e., there is no static guarantee that you only read `value` when complete, only take/drop `f` when incomplete, or that `into_inner` returns the correct variant without consulting `once.state()`).

**Evidence**:

```rust
// Note: Other parts of this module contain: 1 free function(s)

/// }
/// ```
#[stable(feature = "lazy_cell", since = "1.80.0")]
pub struct LazyLock<T, F = fn() -> T> {
    // FIXME(nonpoison_once): if possible, switch to nonpoison version once it is available
    once: Once,
    data: UnsafeCell<Data<T, F>>,
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Creates a new lazy value with the given initializing function.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::LazyLock;
    ///
    /// let hello = "Hello, World!".to_string();
    ///
    /// let lazy = LazyLock::new(|| hello.to_uppercase());
    ///
    /// assert_eq!(&*lazy, "HELLO, WORLD!");
    /// ```
    #[inline]
    #[stable(feature = "lazy_cell", since = "1.80.0")]
    #[rustc_const_stable(feature = "lazy_cell", since = "1.80.0")]
    pub const fn new(f: F) -> LazyLock<T, F> {
        LazyLock { once: Once::new(), data: UnsafeCell::new(Data { f: ManuallyDrop::new(f) }) }
    }

    /// Creates a new lazy value that is already initialized.
    #[inline]
    #[cfg(test)]
    pub(crate) fn preinit(value: T) -> LazyLock<T, F> {
        let once = Once::new();
        once.call_once(|| {});
        LazyLock { once, data: UnsafeCell::new(Data { value: ManuallyDrop::new(value) }) }
    }

    /// Consumes this `LazyLock` returning the stored value.
    ///
    /// Returns `Ok(value)` if `Lazy` is initialized and `Err(f)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_cell_into_inner)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let hello = "Hello, World!".to_string();
    ///
    /// let lazy = LazyLock::new(|| hello.to_uppercase());
    ///
    /// assert_eq!(&*lazy, "HELLO, WORLD!");
    /// assert_eq!(LazyLock::into_inner(lazy).ok(), Some("HELLO, WORLD!".to_string()));
    /// ```
    #[unstable(feature = "lazy_cell_into_inner", issue = "125623")]
    pub fn into_inner(mut this: Self) -> Result<T, F> {
        let state = this.once.state();
        match state {
            ExclusiveState::Poisoned => panic_poisoned(),
            state => {
                let this = ManuallyDrop::new(this);
                let data = unsafe { ptr::read(&this.data) }.into_inner();
                match state {
                    ExclusiveState::Incomplete => Err(ManuallyDrop::into_inner(unsafe { data.f })),
                    ExclusiveState::Complete => Ok(ManuallyDrop::into_inner(unsafe { data.value })),
                    ExclusiveState::Poisoned => unreachable!(),
                }
            }
        }
    }

    /// Forces the evaluation of this lazy value and returns a mutable reference to
    /// the result.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    /// use std::sync::LazyLock;
    ///
    /// let mut lazy = LazyLock::new(|| 92);
    ///
    /// let p = LazyLock::force_mut(&mut lazy);
    /// assert_eq!(*p, 92);
    /// *p = 44;
    /// assert_eq!(*lazy, 44);
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn force_mut(this: &mut LazyLock<T, F>) -> &mut T {
        #[cold]
        /// # Safety
        /// May only be called when the state is `Incomplete`.
        unsafe fn really_init_mut<T, F: FnOnce() -> T>(this: &mut LazyLock<T, F>) -> &mut T {
            struct PoisonOnPanic<'a, T, F>(&'a mut LazyLock<T, F>);
            impl<T, F> Drop for PoisonOnPanic<'_, T, F> {
                #[inline]
                fn drop(&mut self) {
                    self.0.once.set_state(ExclusiveState::Poisoned);
                }
            }

            // SAFETY: We always poison if the initializer panics (then we never check the data),
            // or set the data on success.
            let f = unsafe { ManuallyDrop::take(&mut this.data.get_mut().f) };
            // INVARIANT: Initiated from mutable reference, don't drop because we read it.
            let guard = PoisonOnPanic(this);
            let data = f();
            guard.0.data.get_mut().value = ManuallyDrop::new(data);
            guard.0.once.set_state(ExclusiveState::Complete);
            core::mem::forget(guard);
            // SAFETY: We put the value there above.
            unsafe { &mut this.data.get_mut().value }
        }

        let state = this.once.state();
        match state {
            ExclusiveState::Poisoned => panic_poisoned(),
            // SAFETY: The `Once` states we completed the initialization.
            ExclusiveState::Complete => unsafe { &mut this.data.get_mut().value },
            // SAFETY: The state is `Incomplete`.
            ExclusiveState::Incomplete => unsafe { really_init_mut(this) },
        }
    }

    /// Forces the evaluation of this lazy value and returns a reference to
    /// result. This is equivalent to the `Deref` impl, but is explicit.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::LazyLock;
    ///
    /// let lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::force(&lazy), &92);
    /// assert_eq!(&*lazy, &92);
    /// ```
    #[inline]
    #[stable(feature = "lazy_cell", since = "1.80.0")]
    pub fn force(this: &LazyLock<T, F>) -> &T {
        this.once.call_once(|| {
            // SAFETY: `call_once` only runs this closure once, ever.
            let data = unsafe { &mut *this.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });

        // SAFETY:
        // There are four possible scenarios:
        // * the closure was called and initialized `value`.
        // * the closure was called and panicked, so this point is never reached.
        // * the closure was not called, but a previous call initialized `value`.
        // * the closure was not called because the Once is poisoned, so this point
        //   is never reached.
        // So `value` has definitely been initialized and will not be modified again.
        unsafe { &*(*this.data.get()).value }
    }
}

impl<T, F> LazyLock<T, F> {
    /// Returns a mutable reference to the value if initialized, or `None` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let mut lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::get_mut(&mut lazy), None);
    /// let _ = LazyLock::force(&lazy);
    /// *LazyLock::get_mut(&mut lazy).unwrap() = 44;
    /// assert_eq!(*lazy, 44);
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn get_mut(this: &mut LazyLock<T, F>) -> Option<&mut T> {
        // `state()` does not perform an atomic load, so prefer it over `is_complete()`.
        let state = this.once.state();
        match state {
            // SAFETY:
            // The closure has been run successfully, so `value` has been initialized.
            ExclusiveState::Complete => Some(unsafe { &mut this.data.get_mut().value }),
            _ => None,
        }
    }

    /// Returns a reference to the value if initialized, or `None` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::get(&lazy), None);
    /// let _ = LazyLock::force(&lazy);
    /// assert_eq!(LazyLock::get(&lazy), Some(&92));
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn get(this: &LazyLock<T, F>) -> Option<&T> {
        if this.once.is_completed() {
            // SAFETY:
            // The closure has been run successfully, so `value` has been initialized
            // and will not be modified again.
            Some(unsafe { &(*this.data.get()).value })
        } else {
            None
        }
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T, F> Drop for LazyLock<T, F> {
    fn drop(&mut self) {
        match self.once.state() {
            ExclusiveState::Incomplete => unsafe { ManuallyDrop::drop(&mut self.data.get_mut().f) },
            ExclusiveState::Complete => unsafe {
                ManuallyDrop::drop(&mut self.data.get_mut().value)
            },
            ExclusiveState::Poisoned => {}
        }
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;

    /// Dereferences the value.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    #[inline]
    fn deref(&self) -> &T {
        LazyLock::force(self)
    }
}

// ... (other code) ...

// We never create a `&F` from a `&LazyLock<T, F>` so it is fine
// to not impl `Sync` for `F`.
#[stable(feature = "lazy_cell", since = "1.80.0")]
unsafe impl<T: Sync + Send, F: Send> Sync for LazyLock<T, F> {}
// auto-derived `Send` impl is OK.

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: RefUnwindSafe + UnwindSafe, F: UnwindSafe> RefUnwindSafe for LazyLock<T, F> {}
#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: UnwindSafe, F: UnwindSafe> UnwindSafe for LazyLock<T, F> {}

```

**Entity:** LazyLock<T, F>

**States:** Incomplete, Complete, Poisoned

**Transitions:**
- Incomplete -> Complete via force()/deref() (Once::call_once initializes value and stores it)
- Incomplete -> Complete via force_mut() (really_init_mut runs f(), writes value, sets Once state)
- Incomplete -> Poisoned via force_mut() if initializer panics (PoisonOnPanic Drop sets Once state)
- Incomplete -> (Err(F)) via into_inner() (returns initializer when not yet run)
- Complete -> (Ok(T)) via into_inner() (returns initialized value)
- Poisoned -> panic via into_inner()/force_mut() (panic_poisoned())

**Evidence:** struct fields: `once: Once` and `data: UnsafeCell<Data<T, F>>` encode state externally vs at type level; `into_inner`: `let state = this.once.state(); match state { ExclusiveState::Poisoned => panic_poisoned(), ... ExclusiveState::Incomplete => Err(...data.f...), ExclusiveState::Complete => Ok(...data.value...) }`; `force_mut`: `let state = this.once.state(); match state { Poisoned => panic_poisoned(), Complete => unsafe { &mut ...value }, Incomplete => unsafe { really_init_mut(this) } }`; `really_init_mut` safety contract comment: "May only be called when the state is `Incomplete`."; `really_init_mut`: `ManuallyDrop::take(&mut ...f)` then writes `...value = ManuallyDrop::new(data)` and `once.set_state(ExclusiveState::Complete)`; `PoisonOnPanic` Drop: `self.0.once.set_state(ExclusiveState::Poisoned);` establishes the poisoned transition on unwind; `force`: `this.once.call_once(|| { ... take f ... data.value = ... })` and then returns `unsafe { &*(*this.data.get()).value }` based on the runtime protocol described in the SAFETY comment; `Drop for LazyLock`: `match self.once.state() { Incomplete => drop f, Complete => drop value, Poisoned => {} }` shows the mutually-exclusive active field invariant

**Implementation:** Represent the state in the type: e.g., `LazyLock<T, F, S>` with `S = Uninit | Init | Poisoned` (ZSTs/PhantomData). Provide transitions `force(self: &LazyLock<..., Uninit>) -> &LazyLock<..., Init>` (or an internal guard that yields `&T` only after initialization), and have `into_inner(self)` be implemented separately for `Uninit` (returns `F`) and `Init` (returns `T`). Internally you may still need `Once`, but expose state-appropriate APIs so users (and internal code) cannot compile calls that assume the wrong active field without going through the transition.

---

### 25. SyncWaker emptiness cache coherence (is_empty <-> inner queues)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-211`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: SyncWaker maintains a cached emptiness flag (`is_empty`) that is intended to reflect whether `inner.selectors` and `inner.observers` are empty. Correctness/performance depends on a protocol: every mutation of the inner Waker must be followed by updating `is_empty`, and `notify()` relies on a double-checked pattern (check atomic, then lock, then re-check) to avoid unnecessary locking. This invariant (cache accurately mirrors inner state) is not enforced by types; it is maintained by convention across methods. Drop asserts `is_empty == true`, which is weaker than 'inner is empty' and relies on the cache being up-to-date.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Entry; struct Waker, impl Waker (7 methods), impl Drop for Waker (1 methods), impl SyncWaker (5 methods), impl Drop for SyncWaker (1 methods); struct SyncWaker

//! Waking mechanism for threads blocked on channel operations.

use super::context::Context;
use super::select::{Operation, Selected};
use crate::ptr;
use crate::sync::Mutex;
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};

/// Represents a thread blocked on a specific channel operation.
pub(crate) struct Entry {
    /// The operation.
    pub(crate) oper: Operation,

    /// Optional packet.
    pub(crate) packet: *mut (),

    /// Context associated with the thread owning this operation.
    pub(crate) cx: Context,
}

/// A queue of threads blocked on channel operations.
///
/// This data structure is used by threads to register blocking operations and get woken up once
/// an operation becomes ready.
pub(crate) struct Waker {
    /// A list of select operations.
    selectors: Vec<Entry>,

    /// A list of operations waiting to be ready.
    observers: Vec<Entry>,
}

impl Waker {
    /// Creates a new `Waker`.
    #[inline]
    pub(crate) fn new() -> Self {
        Waker { selectors: Vec::new(), observers: Vec::new() }
    }

    /// Registers a select operation.
    #[inline]
    pub(crate) fn register(&mut self, oper: Operation, cx: &Context) {
        self.register_with_packet(oper, ptr::null_mut(), cx);
    }

    /// Registers a select operation and a packet.
    #[inline]
    pub(crate) fn register_with_packet(&mut self, oper: Operation, packet: *mut (), cx: &Context) {
        self.selectors.push(Entry { oper, packet, cx: cx.clone() });
    }

    /// Unregisters a select operation.
    #[inline]
    pub(crate) fn unregister(&mut self, oper: Operation) -> Option<Entry> {
        if let Some((i, _)) =
            self.selectors.iter().enumerate().find(|&(_, entry)| entry.oper == oper)
        {
            let entry = self.selectors.remove(i);
            Some(entry)
        } else {
            None
        }
    }

    /// Attempts to find another thread's entry, select the operation, and wake it up.
    #[inline]
    pub(crate) fn try_select(&mut self) -> Option<Entry> {
        if self.selectors.is_empty() {
            None
        } else {
            let thread_id = current_thread_id();

            self.selectors
                .iter()
                .position(|selector| {
                    // Does the entry belong to a different thread?
                    selector.cx.thread_id() != thread_id
                        && selector // Try selecting this operation.
                            .cx
                            .try_select(Selected::Operation(selector.oper))
                            .is_ok()
                        && {
                            // Provide the packet.
                            selector.cx.store_packet(selector.packet);
                            // Wake the thread up.
                            selector.cx.unpark();
                            true
                        }
                })
                // Remove the entry from the queue to keep it clean and improve
                // performance.
                .map(|pos| self.selectors.remove(pos))
        }
    }

    /// Notifies all operations waiting to be ready.
    #[inline]
    pub(crate) fn notify(&mut self) {
        for entry in self.observers.drain(..) {
            if entry.cx.try_select(Selected::Operation(entry.oper)).is_ok() {
                entry.cx.unpark();
            }
        }
    }

    /// Notifies all registered operations that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&mut self) {
        for entry in self.selectors.iter() {
            if entry.cx.try_select(Selected::Disconnected).is_ok() {
                // Wake the thread up.
                //
                // Here we don't remove the entry from the queue. Registered threads must
                // unregister from the waker by themselves. They might also want to recover the
                // packet value and destroy it, if necessary.
                entry.cx.unpark();
            }
        }

        self.notify();
    }
}

impl Drop for Waker {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(self.selectors.len(), 0);
        debug_assert_eq!(self.observers.len(), 0);
    }
}

/// A waker that can be shared among threads without locking.
///
/// This is a simple wrapper around `Waker` that internally uses a mutex for synchronization.
pub(crate) struct SyncWaker {
    /// The inner `Waker`.
    inner: Mutex<Waker>,

    /// `true` if the waker is empty.
    is_empty: Atomic<bool>,
}

impl SyncWaker {
    /// Creates a new `SyncWaker`.
    #[inline]
    pub(crate) fn new() -> Self {
        SyncWaker { inner: Mutex::new(Waker::new()), is_empty: AtomicBool::new(true) }
    }

    /// Registers the current thread with an operation.
    #[inline]
    pub(crate) fn register(&self, oper: Operation, cx: &Context) {
        let mut inner = self.inner.lock().unwrap();
        inner.register(oper, cx);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }

    /// Unregisters an operation previously registered by the current thread.
    #[inline]
    pub(crate) fn unregister(&self, oper: Operation) -> Option<Entry> {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.unregister(oper);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
        entry
    }

    /// Attempts to find one thread (not the current one), select its operation, and wake it up.
    #[inline]
    pub(crate) fn notify(&self) {
        if !self.is_empty.load(Ordering::SeqCst) {
            let mut inner = self.inner.lock().unwrap();
            if !self.is_empty.load(Ordering::SeqCst) {
                inner.try_select();
                inner.notify();
                self.is_empty.store(
                    inner.selectors.is_empty() && inner.observers.is_empty(),
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Notifies all threads that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.disconnect();
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }
}

impl Drop for SyncWaker {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.is_empty.load(Ordering::SeqCst));
    }
}

/// Returns a unique id for the current thread.
#[inline]
pub fn current_thread_id() -> usize {
    // `u8` is not drop so this variable will be available during thread destruction,
    // whereas `thread::current()` would not be
    thread_local! { static DUMMY: u8 = const { 0 } }
    DUMMY.with(|x| (x as *const u8).addr())
}

```

**Entity:** SyncWaker

**States:** CacheTrueEmpty, CacheFalseNonEmpty, CacheStale

**Transitions:**
- CacheTrueEmpty -> CacheFalseNonEmpty via register() (after pushing into selectors)
- CacheFalseNonEmpty -> CacheTrueEmpty via unregister()/notify()/disconnect() when inner becomes empty
- CacheTrueEmpty/CacheFalseNonEmpty -> CacheStale if any future code mutates inner without updating is_empty (latent hazard)
- Any -> (debug-assert failure) on Drop if cache says non-empty

**Evidence:** struct SyncWaker { inner: Mutex<Waker>, is_empty: Atomic<bool> }; SyncWaker::new(): is_empty initialized to true while inner is Waker::new() (empty); SyncWaker::register()/unregister()/disconnect(): all recompute and store `inner.selectors.is_empty() && inner.observers.is_empty()` into is_empty; SyncWaker::notify(): uses `if !self.is_empty.load(...) { lock; if !self.is_empty.load(...) { ...; self.is_empty.store(inner.selectors.is_empty() && inner.observers.is_empty(), ...) } }` (double-checked protocol depends on cache coherence); impl Drop for SyncWaker: debug_assert!(self.is_empty.load(Ordering::SeqCst))

**Implementation:** Encapsulate all inner access behind a single helper that returns a guard which updates the cache on drop (RAII), e.g. `fn with_inner(&self) -> InnerGuard` where `InnerGuard` derefs to `Waker` and in `Drop` recomputes `is_empty`. This makes it impossible to mutate `inner` without updating the cache. Alternatively, remove `is_empty` and always lock (simpler but slower), or introduce a dedicated `struct NonEmpty` capability returned by register() that proves non-emptiness and is consumed by notify().

---

### 74. First-block initialization protocol (Uninitialized -> Initialized) with half-initialized intermediate

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-425`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The channel starts with null `head.block`/`tail.block` and lazily allocates/installs the first `Block<T>` on the first send. There is an explicit (and even tested under miri) intermediate state where `tail.block` is installed but `head.block` is still null (a 'half-initialized' channel). Receivers must spin-wait when observing `head.block` null, and senders must coordinate installation via CAS. This initialization protocol is enforced by runtime null-pointer checks and spinning, not by a type-level state that would make it impossible to call receive paths before initialization is complete (or would make the intermediate state unrepresentable to readers).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Block, impl Block < T > (3 methods); struct Position; struct ListToken

///
/// Consecutive messages are grouped into blocks in order to put less pressure on the allocator and
/// improve cache efficiency.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    head: CachePadded<Position<T>>,

    /// The tail of the channel.
    tail: CachePadded<Position<T>>,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,

    /// Indicates that dropping a `Channel<T>` may drop messages of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Creates a new unbounded channel.
    pub(crate) fn new() -> Self {
        Channel {
            head: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            tail: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            receivers: SyncWaker::new(),
            _marker: PhantomData,
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        let mut block = self.tail.block.load(Ordering::Acquire);
        let mut next_block = None;

        loop {
            // Check if the channel is disconnected.
            if tail & MARK_BIT != 0 {
                token.list.block = ptr::null();
                return true;
            }

            // Calculate the offset of the index into the block.
            let offset = (tail >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                tail = self.tail.index.load(Ordering::Acquire);
                block = self.tail.block.load(Ordering::Acquire);
                continue;
            }

            // If we're going to have to install the next block, allocate it in advance in order to
            // make the wait for other threads as short as possible.
            if offset + 1 == BLOCK_CAP && next_block.is_none() {
                next_block = Some(Block::<T>::new());
            }

            // If this is the first message to be sent into the channel, we need to allocate the
            // first block and install it.
            if block.is_null() {
                let new = Box::into_raw(Block::<T>::new());

                if self
                    .tail
                    .block
                    .compare_exchange(block, new, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    // This yield point leaves the channel in a half-initialized state where the
                    // tail.block pointer is set but the head.block is not. This is used to
                    // facilitate the test in src/tools/miri/tests/pass/issues/issue-139553.rs
                    #[cfg(miri)]
                    crate::thread::yield_now();
                    self.head.block.store(new, Ordering::Release);
                    block = new;
                } else {
                    next_block = unsafe { Some(Box::from_raw(new)) };
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                    continue;
                }
            }

            let new_tail = tail + (1 << SHIFT);

            // Try advancing the tail forward.
            match self.tail.index.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, install the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next_block = Box::into_raw(next_block.unwrap());
                        self.tail.block.store(next_block, Ordering::Release);
                        self.tail.index.fetch_add(1 << SHIFT, Ordering::Release);
                        (*block).next.store(next_block, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.list.block.is_null() {
            return Err(msg);
        }

        // Write the message into the slot.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.state.fetch_or(WRITE, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.index.load(Ordering::Acquire);
        let mut block = self.head.block.load(Ordering::Acquire);

        loop {
            // Calculate the offset of the index into the block.
            let offset = (head >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            let mut new_head = head + (1 << SHIFT);

            if new_head & MARK_BIT == 0 {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.index.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if head >> SHIFT == tail >> SHIFT {
                    // If the channel is disconnected...
                    if tail & MARK_BIT != 0 {
                        // ...then receive an error.
                        token.list.block = ptr::null();
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                // If head and tail are not in the same block, set `MARK_BIT` in head.
                if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                    new_head |= MARK_BIT;
                }
            }

            // The block can be null here only if the first message is being sent into the channel.
            // In that case, just wait until it gets initialized.
            if block.is_null() {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            // Try moving the head index forward.
            match self.head.index.compare_exchange_weak(
                head,
                new_head,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, move to the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next = (*block).wait_next();
                        let mut next_index = (new_head & !MARK_BIT).wrapping_add(1 << SHIFT);
                        if !(*next).next.load(Ordering::Relaxed).is_null() {
                            next_index |= MARK_BIT;
                        }

                        self.head.block.store(next, Ordering::Release);
                        self.head.index.store(next_index, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    head = self.head.index.load(Ordering::Acquire);
                    block = self.head.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.list.block.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.wait_write();
            let msg = slot.msg.get().read().assume_init();

            // Destroy the block if we've reached the end, or if another thread wanted to destroy but
            // couldn't because we were busy reading from the slot.
            if offset + 1 == BLOCK_CAP {
                Block::destroy(block, 0);
            } else if slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY != 0 {
                Block::destroy(block, offset + 1);
            }

            Ok(msg)
        }
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.send(msg, None).map_err(|err| match err {
            SendTimeoutError::Disconnected(msg) => TrySendError::Disconnected(msg),
            SendTimeoutError::Timeout(_) => unreachable!(),
        })
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        _deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        assert!(self.start_send(token));
        unsafe { self.write(token, msg).map_err(SendTimeoutError::Disconnected) }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            if self.start_recv(token) {
                unsafe {
                    return self.read(token).map_err(|_| RecvTimeoutError::Disconnected);
                }
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            // Prepare for blocking until a sender wakes us up.
            Context::with(|cx| {
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail index, then load the head index.
            let mut tail = self.tail.index.load(Ordering::SeqCst);
            let mut head = self.head.index.load(Ordering::SeqCst);

            // If the tail index didn't change, we've got consistent indices to work with.
            if self.tail.index.load(Ordering::SeqCst) == tail {
                // Erase the lower bits.
                tail &= !((1 << SHIFT) - 1);
                head &= !((1 << SHIFT) - 1);

                // Fix up indices if they fall onto block ends.
                if (tail >> SHIFT) & (LAP - 1) == LAP - 1 {
                    tail = tail.wrapping_add(1 << SHIFT);
                }
                if (head >> SHIFT) & (LAP - 1) == LAP - 1 {
                    head = head.wrapping_add(1 << SHIFT);
                }

                // Rotate indices so that head falls into the first block.
                let lap = (head >> SHIFT) / LAP;
                tail = tail.wrapping_sub((lap * LAP) << SHIFT);
                head = head.wrapping_sub((lap * LAP) << SHIFT);

                // Remove the lower bits.
                tail >>= SHIFT;
                head >>= SHIFT;

                // Return the difference minus the number of blocks between tail and head.
                return tail - head - tail / LAP;
            }
        }
    }

    /// Returns the capacity of the channel.
    pub(crate) fn capacity(&self) -> Option<usize> {
        None
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            // If receivers are dropped first, discard all messages to free
            // memory eagerly.
            self.discard_all_messages();
            true
        } else {
            false
        }
    }

    /// Discards all messages.
    ///
    /// This method should only be called when all receivers are dropped.
    fn discard_all_messages(&self) {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        loop {
            let offset = (tail >> SHIFT) % LAP;
            if offset != BLOCK_CAP {
                break;
            }

            // New updates to tail will be rejected by MARK_BIT and aborted unless it's
            // at boundary. We need to wait for the updates take affect otherwise there
            // can be memory leaks.
            backoff.spin_heavy();
            tail = self.tail.index.load(Ordering::Acquire);
        }

        let mut head = self.head.index.load(Ordering::Acquire);
        // The channel may be uninitialized, so we have to swap to avoid overwriting any sender's attempts
        // to initialize the first block before noticing that the receivers disconnected. Late allocations
        // will be deall
// ... (truncated) ...
```

**Entity:** Channel<T>

**States:** Uninitialized (head.block = null, tail.block = null), HalfInitialized (tail.block != null, head.block = null), Initialized (head.block != null, tail.block != null)

**Transitions:**
- Uninitialized -> HalfInitialized via start_send() successful `tail.block.compare_exchange(null, new, ...)` before `head.block.store(new, ...)`
- HalfInitialized -> Initialized via start_send() `self.head.block.store(new, Ordering::Release)`
- Uninitialized/HalfInitialized -> Initialized also possible via other sender racing and completing installation (start_send loop reloads and continues)

**Evidence:** new(): initializes `Position { block: AtomicPtr::new(ptr::null_mut()), ... }` for both head and tail; start_send(): `if block.is_null() { let new = Box::into_raw(Block::<T>::new()); ... tail.block.compare_exchange ...; self.head.block.store(new, Ordering::Release); }` is the lazy init transition; start_send(): comment: `This yield point leaves the channel in a half-initialized state where the tail.block pointer is set but the head.block is not.`; start_recv(): comment + logic: `The block can be null here only if the first message is being sent... In that case, just wait until it gets initialized.` followed by `if block.is_null() { backoff.spin_heavy(); ... continue; }`

**Implementation:** Make initialization explicit by splitting into `Channel<Uninit, T>` and `Channel<Init, T>` (or wrap the block pointer in a `OnceLock<NonNull<Block<T>>>`-like construct). `send()` would ensure transition to initialized before allowing any `recv` operations to proceed on an `Init` handle. If the half-initialized state must exist internally for concurrency, keep it private and expose only `Init` to receivers (e.g., store an `AtomicPtr` internally but gate public recv behind a one-time init barrier/Once).

---

### 20. Disconnect-on-last-receiver implicit state machine (Connected / ReceiversGone)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/counter.rs:1-50`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The channel has an implicit 'connectedness' state driven by the receiver refcount: when the last receiver is released, a user-provided `disconnect(&C) -> bool` callback must run to transition the channel into a disconnected state. This transition is encoded purely in runtime control flow inside unsafe release(); the type system does not express that operations through Receiver (via Deref to C) are only valid while Connected, nor that disconnect must run exactly once when transitioning to ReceiversGone.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Counter; struct Sender, 1 free function(s), impl Sender < C > (3 methods), impl ops :: Deref for Sender < C > (1 methods)

}

/// The receiving side.
pub(crate) struct Receiver<C> {
    counter: *mut Counter<C>,
}

impl<C> Receiver<C> {
    /// Returns the internal `Counter`.
    fn counter(&self) -> &Counter<C> {
        unsafe { &*self.counter }
    }

    /// Acquires another receiver reference.
    pub(crate) fn acquire(&self) -> Receiver<C> {
        let count = self.counter().receivers.fetch_add(1, Ordering::Relaxed);

        // Cloning receivers and calling `mem::forget` on the clones could potentially overflow the
        // counter. It's very difficult to recover sensibly from such degenerate scenarios so we
        // just abort when the count becomes very large.
        if count > isize::MAX as usize {
            process::abort();
        }

        Receiver { counter: self.counter }
    }

    /// Releases the receiver reference.
    ///
    /// Function `disconnect` will be called if this is the last receiver reference.
    pub(crate) unsafe fn release<F: FnOnce(&C) -> bool>(&self, disconnect: F) {
        if self.counter().receivers.fetch_sub(1, Ordering::AcqRel) == 1 {
            disconnect(&self.counter().chan);

            if self.counter().destroy.swap(true, Ordering::AcqRel) {
                drop(unsafe { Box::from_raw(self.counter) });
            }
        }
    }
}

impl<C> ops::Deref for Receiver<C> {
    type Target = C;

    fn deref(&self) -> &C {
        &self.counter().chan
    }
}

```

**Entity:** Receiver<C>

**States:** Connected (>=1 receiver ref), ReceiversGone (0 receiver refs; disconnect executed once)

**Transitions:**
- Connected -> Connected via acquire() (adds receiver ref, keeping channel connected)
- Connected -> ReceiversGone via release() when receivers.fetch_sub(...) == 1 (invokes disconnect(&chan))

**Evidence:** release() doc comment: "Function `disconnect` will be called if this is the last receiver reference."; release(): `if receivers.fetch_sub(1, ...) == 1 { disconnect(&self.counter().chan); ... }` encodes the Connected -> ReceiversGone transition; impl Deref for Receiver<C>: `fn deref(&self) -> &C { &self.counter().chan }` exposes channel access without any connectedness token/state

**Implementation:** Introduce typestates for the shared channel handle such as `Receiver<Connected, C>` and `Receiver<Disconnected, C>` (or a separate `ConnectedReceiver<C>`). Have `release(self)` consume the receiver and, when it is last, produce a disconnected marker/capability (or transition the channel handle type) so APIs requiring a connected receiver cannot be called once the last receiver is gone.

---

### 49. Backoff usage protocol (Fresh -> Spinning -> Yielding saturation)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/utils.rs:1-45`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Backoff encodes an implicit progression of retry behavior in the runtime field `step`: newly created backoffs start at step=0, successive calls increase the delay quadratically, and after `SPIN_LIMIT` the `spin_heavy()` strategy switches from spinning to yielding. Correctness/performance relies on callers using the same Backoff instance across retries (so step monotonically increases) and choosing the right strategy method for the right loop (lightweight for CAS-failure retry vs heavyweight for blocking loops). None of these temporal/protocol requirements (monotonic use, saturation behavior, or 'light vs heavy' intent) are enforced by the type system; any code can call either method in any order or create a new Backoff each iteration, silently breaking the intended behavior.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct CachePadded, impl CachePadded < T > (1 methods), impl Deref for CachePadded < T > (1 methods), impl DerefMut for CachePadded < T > (1 methods)

const SPIN_LIMIT: u32 = 6;

/// Performs quadratic backoff in spin loops.
pub struct Backoff {
    step: Cell<u32>,
}

impl Backoff {
    /// Creates a new `Backoff`.
    pub fn new() -> Self {
        Backoff { step: Cell::new(0) }
    }

    /// Backs off using lightweight spinning.
    ///
    /// This method should be used for retrying an operation because another thread made
    /// progress. i.e. on CAS failure.
    #[inline]
    pub fn spin_light(&self) {
        let step = self.step.get().min(SPIN_LIMIT);
        for _ in 0..step.pow(2) {
            crate::hint::spin_loop();
        }

        self.step.set(self.step.get() + 1);
    }

    /// Backs off using heavyweight spinning.
    ///
    /// This method should be used in blocking loops where parking the thread is not an option.
    #[inline]
    pub fn spin_heavy(&self) {
        if self.step.get() <= SPIN_LIMIT {
            for _ in 0..self.step.get().pow(2) {
                crate::hint::spin_loop()
            }
        } else {
            crate::thread::yield_now();
        }

        self.step.set(self.step.get() + 1);
    }
}

```

**Entity:** Backoff

**States:** Fresh(step=0), Spinning(step in 1..=SPIN_LIMIT), Yielding(step>SPIN_LIMIT)

**Transitions:**
- Fresh(step=0) -> Spinning via spin_light()/spin_heavy() (increments step)
- Spinning(step<=SPIN_LIMIT) -> Yielding via spin_heavy() once step exceeds SPIN_LIMIT (increments step past limit)
- Spinning -> Spinning via repeated spin_light()/spin_heavy() while step remains <= SPIN_LIMIT
- Yielding -> Yielding via repeated spin_heavy() (continues yielding; still increments step)

**Evidence:** const SPIN_LIMIT: u32 = 6; Backoff { step: Cell<u32> } runtime state field; Backoff::new() initializes step to 0; spin_light(): `let step = self.step.get().min(SPIN_LIMIT);` and then `self.step.set(self.step.get() + 1)` (monotonic progression, saturation for spinning); spin_heavy(): `if self.step.get() <= SPIN_LIMIT { ... spin_loop ... } else { crate::thread::yield_now(); }` (behavioral mode switch based on step); spin_light() doc: "should be used for retrying an operation ... on CAS failure" (comment-based protocol); spin_heavy() doc: "should be used in blocking loops where parking the thread is not an option" (comment-based protocol)

**Implementation:** Encode the mode in the type: e.g., `Backoff<S>` where `S` is `Light` or `Heavy`, constructed via `Backoff::light()` / `Backoff::heavy()`. Expose a single `snooze(&self)` per mode so callers can't mix strategies accidentally. Optionally split saturation into a distinct type/state (e.g., `Backoff<Heavy, Spinning>` transitioning to `Backoff<Heavy, Yielding>` when the counter crosses `SPIN_LIMIT`) to make the yield-switch explicit at compile time; the counter can remain runtime, but the strategy/protocol becomes statically enforced.

---

### 56. Packet allocation-location invariant (stack vs non-stack)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-35`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `on_stack: bool` records whether the packet is stack-allocated. In this snippet, both constructors hardcode `on_stack: true`, implying other code paths can create packets with `on_stack = false` and must treat ownership/lifetime/drop differently depending on this flag (e.g., whether the packet may be moved, referenced after scope, or requires deallocation). This is an implicit state carried as a boolean rather than as distinct types, so the compiler cannot prevent using a 'stack packet' in a context that requires heap/static storage (or vice versa).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Inner; struct Channel, impl Channel < T > (12 methods)

}

/// A slot for passing one message from a sender to a receiver.
struct Packet<T> {
    /// Equals `true` if the packet is allocated on the stack.
    on_stack: bool,

    /// Equals `true` once the packet is ready for reading or writing.
    ready: Atomic<bool>,

    /// The message.
    msg: UnsafeCell<Option<T>>,
}

impl<T> Packet<T> {
    /// Creates an empty packet on the stack.
    fn empty_on_stack() -> Packet<T> {
        Packet { on_stack: true, ready: AtomicBool::new(false), msg: UnsafeCell::new(None) }
    }

    /// Creates a packet on the stack, containing a message.
    fn message_on_stack(msg: T) -> Packet<T> {
        Packet { on_stack: true, ready: AtomicBool::new(false), msg: UnsafeCell::new(Some(msg)) }
    }

    /// Waits until the packet becomes ready for reading or writing.
    fn wait_ready(&self) {
        let backoff = Backoff::new();
        while !self.ready.load(Ordering::Acquire) {
            backoff.spin_heavy();
        }
    }
}

```

**Entity:** Packet<T>

**States:** OnStack, NotOnStack

**Transitions:**
- NotOnStack -> OnStack via `empty_on_stack()` / `message_on_stack()` (constructors produce OnStack packets)

**Evidence:** `on_stack: bool` field explicitly tracks allocation location state; `empty_on_stack()` sets `on_stack: true`; `message_on_stack(msg)` sets `on_stack: true`; Field doc comment: "Equals `true` if the packet is allocated on the stack." implies behavior depends on this state

**Implementation:** Split into two concrete types or a generic `Packet<T, Loc>` with `Loc = OnStack | OffStack` (PhantomData), or use separate constructors returning `StackPacket<T>` and `HeapPacket<T>`. Implement only the operations valid for each location and conversions that are actually safe, eliminating ad-hoc `if on_stack` branching elsewhere.

---

### 65. Channel connection state (Connected / Disconnected) and operation validity

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-248`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Channel has an implicit global connection state tracked at runtime by `Inner::is_disconnected`. In the Connected state, `send/recv/try_send/try_recv` may pair operations or block; in the Disconnected state, these operations must fail immediately with `Disconnected` errors and any blocked operations must be woken. This state is not represented in the type system; callers can invoke send/recv methods regardless of disconnect status and must handle runtime branching and error returns.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Packet, impl Packet < T > (3 methods); struct Inner

}

/// Zero-capacity channel.
pub(crate) struct Channel<T> {
    /// Inner representation of the channel.
    inner: Mutex<Inner>,

    /// Indicates that dropping a `Channel<T>` may drop values of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Constructs a new zero-capacity channel.
    pub(crate) fn new() -> Self {
        Channel {
            inner: Mutex::new(Inner {
                senders: Waker::new(),
                receivers: Waker::new(),
                is_disconnected: false,
            }),
            _marker: PhantomData,
        }
    }

    /// Writes a message into the packet.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no packet, the channel is disconnected.
        if token.zero.0.is_null() {
            return Err(msg);
        }

        unsafe {
            let packet = &*(token.zero.0 as *const Packet<T>);
            packet.msg.get().write(Some(msg));
            packet.ready.store(true, Ordering::Release);
        }
        Ok(())
    }

    /// Reads a message from the packet.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        // If there is no packet, the channel is disconnected.
        if token.zero.0.is_null() {
            return Err(());
        }

        let packet = unsafe { &*(token.zero.0 as *const Packet<T>) };

        if packet.on_stack {
            // The message has been in the packet from the beginning, so there is no need to wait
            // for it. However, after reading the message, we need to set `ready` to `true` in
            // order to signal that the packet can be destroyed.
            let msg = unsafe { packet.msg.get().replace(None) }.unwrap();
            packet.ready.store(true, Ordering::Release);
            Ok(msg)
        } else {
            // Wait until the message becomes available, then read it and destroy the
            // heap-allocated packet.
            packet.wait_ready();
            unsafe {
                let msg = packet.msg.get().replace(None).unwrap();
                drop(Box::from_raw(token.zero.0 as *mut Packet<T>));
                Ok(msg)
            }
        }
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting receiver, pair up with it.
        if let Some(operation) = inner.receivers.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                self.write(token, msg).ok().unwrap();
            }
            Ok(())
        } else if inner.is_disconnected {
            Err(TrySendError::Disconnected(msg))
        } else {
            Err(TrySendError::Full(msg))
        }
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting receiver, pair up with it.
        if let Some(operation) = inner.receivers.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                self.write(token, msg).ok().unwrap();
            }
            return Ok(());
        }

        if inner.is_disconnected {
            return Err(SendTimeoutError::Disconnected(msg));
        }

        Context::with(|cx| {
            // Prepare for blocking until a receiver wakes us up.
            let oper = Operation::hook(token);
            let mut packet = Packet::<T>::message_on_stack(msg);
            inner.senders.register_with_packet(oper, (&raw mut packet) as *mut (), cx);
            inner.receivers.notify();
            drop(inner);

            // Block the current thread.
            // SAFETY: the context belongs to the current thread.
            let sel = unsafe { cx.wait_until(deadline) };

            match sel {
                Selected::Waiting => unreachable!(),
                Selected::Aborted => {
                    self.inner.lock().unwrap().senders.unregister(oper).unwrap();
                    let msg = unsafe { packet.msg.get().replace(None).unwrap() };
                    Err(SendTimeoutError::Timeout(msg))
                }
                Selected::Disconnected => {
                    self.inner.lock().unwrap().senders.unregister(oper).unwrap();
                    let msg = unsafe { packet.msg.get().replace(None).unwrap() };
                    Err(SendTimeoutError::Disconnected(msg))
                }
                Selected::Operation(_) => {
                    // Wait until the message is read, then drop the packet.
                    packet.wait_ready();
                    Ok(())
                }
            }
        })
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting sender, pair up with it.
        if let Some(operation) = inner.senders.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else if inner.is_disconnected {
            Err(TryRecvError::Disconnected)
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting sender, pair up with it.
        if let Some(operation) = inner.senders.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                return self.read(token).map_err(|_| RecvTimeoutError::Disconnected);
            }
        }

        if inner.is_disconnected {
            return Err(RecvTimeoutError::Disconnected);
        }

        Context::with(|cx| {
            // Prepare for blocking until a sender wakes us up.
            let oper = Operation::hook(token);
            let mut packet = Packet::<T>::empty_on_stack();
            inner.receivers.register_with_packet(oper, (&raw mut packet) as *mut (), cx);
            inner.senders.notify();
            drop(inner);

            // Block the current thread.
            // SAFETY: the context belongs to the current thread.
            let sel = unsafe { cx.wait_until(deadline) };

            match sel {
                Selected::Waiting => unreachable!(),
                Selected::Aborted => {
                    self.inner.lock().unwrap().receivers.unregister(oper).unwrap();
                    Err(RecvTimeoutError::Timeout)
                }
                Selected::Disconnected => {
                    self.inner.lock().unwrap().receivers.unregister(oper).unwrap();
                    Err(RecvTimeoutError::Disconnected)
                }
                Selected::Operation(_) => {
                    // Wait until the message is provided, then read it.
                    packet.wait_ready();
                    unsafe { Ok(packet.msg.get().replace(None).unwrap()) }
                }
            }
        })
    }

    /// Disconnects the channel and wakes up all blocked senders and receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();

        if !inner.is_disconnected {
            inner.is_disconnected = true;
            inner.senders.disconnect();
            inner.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        0
    }

    /// Returns the capacity of the channel.
    #[allow(clippy::unnecessary_wraps)] // This is intentional.
    pub(crate) fn capacity(&self) -> Option<usize> {
        Some(0)
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        true
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        true
    }
}

```

**Entity:** Channel<T>

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via disconnect()
- Disconnected -> Disconnected via disconnect() (idempotent no-op)

**Evidence:** field: Channel::inner: Mutex<Inner> (shared mutable state where connection flag lives); constructor: Channel::new() initializes `is_disconnected: false` inside `Inner`; method: try_send(): `} else if inner.is_disconnected { Err(TrySendError::Disconnected(msg)) }`; method: send(): `if inner.is_disconnected { return Err(SendTimeoutError::Disconnected(msg)); }` and match arm `Selected::Disconnected => ... Err(SendTimeoutError::Disconnected(msg))`; method: try_recv(): `} else if inner.is_disconnected { Err(TryRecvError::Disconnected) }`; method: recv(): `if inner.is_disconnected { return Err(RecvTimeoutError::Disconnected); }` and match arm `Selected::Disconnected => ... Err(RecvTimeoutError::Disconnected)`; method: disconnect(): toggles `inner.is_disconnected = true;` and calls `inner.senders.disconnect(); inner.receivers.disconnect();`

**Implementation:** Split the API into `Channel<Connected, T>` and `Channel<Disconnected, T>` (or expose a `DisconnectHandle` capability). `disconnect(self or &self)` would transition to / produce a `Channel<Disconnected, T>` (or revoke a `Connected` capability token). Methods that cannot succeed when disconnected (e.g., blocking `send/recv`) would only be available on `Channel<Connected, T>`, eliminating runtime `is_disconnected` checks from the public surface.

---

### 2. SyncWaker empty-flag coherence protocol (is_empty must reflect inner queues)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-175`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: SyncWaker caches whether its inner Waker queues are empty using is_empty: AtomicBool. The notify fast-path relies on the invariant that is_empty is kept coherent with (inner.selectors.is_empty() && inner.observers.is_empty()) after any operation mutating the queues. This coherence is maintained manually by repeated stores after register/unregister/notify/disconnect, and by double-checking is_empty around locking in notify(). The type system cannot enforce that every future mutation of inner updates is_empty, so the correctness/performance of the fast-path depends on a latent protocol ('if you touch inner queues, you must update is_empty'). Drop asserts is_empty is true, implying (in debug) that all registrations were cleared before drop, but this is not statically enforced.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Entry; struct SyncWaker; 1 free function(s)

///
/// This data structure is used by threads to register blocking operations and get woken up once
/// an operation becomes ready.
pub(crate) struct Waker {
    /// A list of select operations.
    selectors: Vec<Entry>,

    /// A list of operations waiting to be ready.
    observers: Vec<Entry>,
}

impl Waker {
    /// Creates a new `Waker`.
    #[inline]
    pub(crate) fn new() -> Self {
        Waker { selectors: Vec::new(), observers: Vec::new() }
    }

    /// Registers a select operation.
    #[inline]
    pub(crate) fn register(&mut self, oper: Operation, cx: &Context) {
        self.register_with_packet(oper, ptr::null_mut(), cx);
    }

    /// Registers a select operation and a packet.
    #[inline]
    pub(crate) fn register_with_packet(&mut self, oper: Operation, packet: *mut (), cx: &Context) {
        self.selectors.push(Entry { oper, packet, cx: cx.clone() });
    }

    /// Unregisters a select operation.
    #[inline]
    pub(crate) fn unregister(&mut self, oper: Operation) -> Option<Entry> {
        if let Some((i, _)) =
            self.selectors.iter().enumerate().find(|&(_, entry)| entry.oper == oper)
        {
            let entry = self.selectors.remove(i);
            Some(entry)
        } else {
            None
        }
    }

    /// Attempts to find another thread's entry, select the operation, and wake it up.
    #[inline]
    pub(crate) fn try_select(&mut self) -> Option<Entry> {
        if self.selectors.is_empty() {
            None
        } else {
            let thread_id = current_thread_id();

            self.selectors
                .iter()
                .position(|selector| {
                    // Does the entry belong to a different thread?
                    selector.cx.thread_id() != thread_id
                        && selector // Try selecting this operation.
                            .cx
                            .try_select(Selected::Operation(selector.oper))
                            .is_ok()
                        && {
                            // Provide the packet.
                            selector.cx.store_packet(selector.packet);
                            // Wake the thread up.
                            selector.cx.unpark();
                            true
                        }
                })
                // Remove the entry from the queue to keep it clean and improve
                // performance.
                .map(|pos| self.selectors.remove(pos))
        }
    }

    /// Notifies all operations waiting to be ready.
    #[inline]
    pub(crate) fn notify(&mut self) {
        for entry in self.observers.drain(..) {
            if entry.cx.try_select(Selected::Operation(entry.oper)).is_ok() {
                entry.cx.unpark();
            }
        }
    }

    /// Notifies all registered operations that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&mut self) {
        for entry in self.selectors.iter() {
            if entry.cx.try_select(Selected::Disconnected).is_ok() {
                // Wake the thread up.
                //
                // Here we don't remove the entry from the queue. Registered threads must
                // unregister from the waker by themselves. They might also want to recover the
                // packet value and destroy it, if necessary.
                entry.cx.unpark();
            }
        }

        self.notify();
    }
}

impl Drop for Waker {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(self.selectors.len(), 0);
        debug_assert_eq!(self.observers.len(), 0);
    }
}

// ... (other code) ...

    is_empty: Atomic<bool>,
}

impl SyncWaker {
    /// Creates a new `SyncWaker`.
    #[inline]
    pub(crate) fn new() -> Self {
        SyncWaker { inner: Mutex::new(Waker::new()), is_empty: AtomicBool::new(true) }
    }

    /// Registers the current thread with an operation.
    #[inline]
    pub(crate) fn register(&self, oper: Operation, cx: &Context) {
        let mut inner = self.inner.lock().unwrap();
        inner.register(oper, cx);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }

    /// Unregisters an operation previously registered by the current thread.
    #[inline]
    pub(crate) fn unregister(&self, oper: Operation) -> Option<Entry> {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.unregister(oper);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
        entry
    }

    /// Attempts to find one thread (not the current one), select its operation, and wake it up.
    #[inline]
    pub(crate) fn notify(&self) {
        if !self.is_empty.load(Ordering::SeqCst) {
            let mut inner = self.inner.lock().unwrap();
            if !self.is_empty.load(Ordering::SeqCst) {
                inner.try_select();
                inner.notify();
                self.is_empty.store(
                    inner.selectors.is_empty() && inner.observers.is_empty(),
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Notifies all threads that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.disconnect();
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }
}

impl Drop for SyncWaker {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.is_empty.load(Ordering::SeqCst));
    }
}

```

**Entity:** SyncWaker

**States:** FlagTrueEmpty, FlagFalseNonEmpty, FlagStale

**Transitions:**
- FlagTrueEmpty -> FlagFalseNonEmpty via register() (after inner.register)
- FlagFalseNonEmpty -> FlagTrueEmpty via unregister()/notify()/disconnect() when inner queues become empty and store(true)
- Any -> FlagStale if inner queues change without a matching is_empty.store(...) (not prevented by types)

**Evidence:** field: is_empty: AtomicBool (runtime state cache); method: register() updates is_empty with inner.selectors.is_empty() && inner.observers.is_empty() after mutation; method: unregister() updates is_empty after removal; method: notify() uses if !self.is_empty.load(...) fast-path and then re-checks after locking; updates is_empty after calling inner.try_select() and inner.notify(); method: disconnect() updates is_empty after inner.disconnect(); impl Drop for SyncWaker: debug_assert!(self.is_empty.load(Ordering::SeqCst)) (expects empty at drop)

**Implementation:** Remove the separate AtomicBool by deriving emptiness under the mutex, or encapsulate inner+flag updates behind a single private helper that returns a guard type which on Drop recomputes/stores is_empty (ensuring coherence even if new methods are added). Alternatively, store the emptiness bit inside Waker and expose only methods that update it, eliminating the 'two sources of truth'.

---

### 33. Channel receive outcome states (Empty/Still-Connected vs Permanently-Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-44`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: RecvTimeoutError encodes two distinct runtime channel states: (1) the channel is currently empty but may produce values later because at least one Sender still exists (Timeout), and (2) the sending half is disconnected so no further values will ever arrive (Disconnected). This is an implicit state machine about the channel/endpoint lifecycle, but the type system does not provide a way to reflect or carry a proof of 'disconnected' forward to prevent repeated/pointless waiting calls or to make subsequent operations statically aware that the channel is terminal.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum TrySendError

/// [`recv_timeout`]: Receiver::recv_timeout
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[stable(feature = "mpsc_recv_timeout", since = "1.12.0")]
pub enum RecvTimeoutError {
    /// This **channel** is currently empty, but the **Sender**(s) have not yet
    /// disconnected, so data may yet become available.
    #[stable(feature = "mpsc_recv_timeout", since = "1.12.0")]
    Timeout,
    /// The **channel**'s sending half has become disconnected, and there will
    /// never be any more data received on it.
    #[stable(feature = "mpsc_recv_timeout", since = "1.12.0")]
    Disconnected,
}

// ... (other code) ...

}

#[stable(feature = "mpsc_recv_timeout_error", since = "1.15.0")]
impl error::Error for RecvTimeoutError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            RecvTimeoutError::Timeout => "timed out waiting on channel",
            RecvTimeoutError::Disconnected => "channel is empty and sending half is closed",
        }
    }
}

#[stable(feature = "mpsc_error_conversions", since = "1.24.0")]
impl From<RecvError> for RecvTimeoutError {
    /// Converts a `RecvError` into a `RecvTimeoutError`.
    ///
    /// This conversion always returns `RecvTimeoutError::Disconnected`.
    ///
    /// No data is allocated on the heap.
    fn from(err: RecvError) -> RecvTimeoutError {
        match err {
            RecvError => RecvTimeoutError::Disconnected,
        }
    }
}

```

**Entity:** RecvTimeoutError

**States:** WouldBlockOrWait (Timeout), TerminalDisconnect (Disconnected)

**Transitions:**
- WouldBlockOrWait (Timeout) -> TerminalDisconnect (Disconnected) when all Sender handles are dropped (channel disconnects)

**Evidence:** enum variants: RecvTimeoutError::Timeout and RecvTimeoutError::Disconnected; doc comment on Timeout: "Sender(s) have not yet disconnected, so data may yet become available" (implies nonterminal/continuing state); doc comment on Disconnected: "there will never be any more data received on it" (implies terminal state); Error::description() strings differentiate terminal vs nonterminal: "timed out waiting on channel" vs "channel is empty and sending half is closed"

**Implementation:** Introduce a typestate for Receiver like Receiver<T, S> with states Connected and Disconnected. Operations that can observe disconnection (e.g., recv/recv_timeout) could return a result that, on Disconnected, yields a Receiver<T, Disconnected> (or a token) where waiting/receiving APIs are removed or trivially return Disconnected, making the terminal state explicit and preventing misuse in code that should stop polling/waiting.

---

### 68. Receiver channel lifecycle (Connected/Open -> Disconnected/Drained)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-331`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Receiver operations implicitly depend on the channel’s lifecycle: while at least one Sender/SyncSender exists, recv* may block waiting for messages; once all senders are dropped, the receiver transitions to a disconnected state but may still yield buffered messages; after the buffer is drained, recv returns RecvError and iterators end (None). These states are tracked internally (in mpmc::Receiver) and surfaced only via runtime blocking behavior and Result/Option return values; the type system does not distinguish a connected receiver from a disconnected/drained one, so callers can only discover the state dynamically.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Receiver")]
pub struct Receiver<T> {
    inner: mpmc::Receiver<T>,
}

// ... (other code) ...

// The receiver port can be sent from place to place, so long as it
// is not used to receive non-sendable things.
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: Send> Send for Receiver<T> {}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> !Sync for Receiver<T> {}


// ... (other code) ...

// Receiver
////////////////////////////////////////////////////////////////////////////////

impl<T> Receiver<T> {
    /// Attempts to return a pending value on this receiver without blocking.
    ///
    /// This method will never block the caller in order to wait for data to
    /// become available. Instead, this will always return immediately with a
    /// possible option of pending data on the channel.
    ///
    /// This is useful for a flavor of "optimistic check" before deciding to
    /// block on a receiver.
    ///
    /// Compared with [`recv`], this function has two failure cases instead of one
    /// (one for disconnection, one for an empty buffer).
    ///
    /// [`recv`]: Self::recv
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::mpsc::{Receiver, channel};
    ///
    /// let (_, receiver): (_, Receiver<i32>) = channel();
    ///
    /// assert!(receiver.try_recv().is_err());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.inner.try_recv()
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent (at least one sender
    /// still exists). Once a message is sent to the corresponding [`Sender`]
    /// (or [`SyncSender`]), this receiver will wake up and return that
    /// message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::mpsc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpsc::channel();
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(Ok(1), recv.recv());
    /// ```
    ///
    /// Buffering behavior:
    ///
    /// ```
    /// use std::sync::mpsc;
    /// use std::thread;
    /// use std::sync::mpsc::RecvError;
    ///
    /// let (send, recv) = mpsc::channel();
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    ///     send.send(2).unwrap();
    ///     send.send(3).unwrap();
    ///     drop(send);
    /// });
    ///
    /// // wait for the thread to join so we ensure the sender is dropped
    /// handle.join().unwrap();
    ///
    /// assert_eq!(Ok(1), recv.recv());
    /// assert_eq!(Ok(2), recv.recv());
    /// assert_eq!(Ok(3), recv.recv());
    /// assert_eq!(Err(RecvError), recv.recv());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn recv(&self) -> Result<T, RecvError> {
        self.inner.recv()
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up, or if it waits more than `timeout`.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent (at least one sender
    /// still exists). Once a message is sent to the corresponding [`Sender`]
    /// (or [`SyncSender`]), this receiver will wake up and return that
    /// message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// Successfully receiving value before encountering timeout:
    ///
    /// ```no_run
    /// use std::thread;
    /// use std::time::Duration;
    /// use std::sync::mpsc;
    ///
    /// let (send, recv) = mpsc::channel();
    ///
    /// thread::spawn(move || {
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_timeout(Duration::from_millis(400)),
    ///     Ok('a')
    /// );
    /// ```
    ///
    /// Receiving an error upon reaching timeout:
    ///
    /// ```no_run
    /// use std::thread;
    /// use std::time::Duration;
    /// use std::sync::mpsc;
    ///
    /// let (send, recv) = mpsc::channel();
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(800));
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_timeout(Duration::from_millis(400)),
    ///     Err(mpsc::RecvTimeoutError::Timeout)
    /// );
    /// ```
    #[stable(feature = "mpsc_recv_timeout", since = "1.12.0")]
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.inner.recv_timeout(timeout)
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up, or if `deadline` is reached.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent. Once a message is
    /// sent to the corresponding [`Sender`] (or [`SyncSender`]), then this
    /// receiver will wake up and return that message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// Successfully receiving value before reaching deadline:
    ///
    /// ```no_run
    /// #![feature(deadline_api)]
    /// use std::thread;
    /// use std::time::{Duration, Instant};
    /// use std::sync::mpsc;
    ///
    /// let (send, recv) = mpsc::channel();
    ///
    /// thread::spawn(move || {
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_deadline(Instant::now() + Duration::from_millis(400)),
    ///     Ok('a')
    /// );
    /// ```
    ///
    /// Receiving an error upon reaching deadline:
    ///
    /// ```no_run
    /// #![feature(deadline_api)]
    /// use std::thread;
    /// use std::time::{Duration, Instant};
    /// use std::sync::mpsc;
    ///
    /// let (send, recv) = mpsc::channel();
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(800));
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_deadline(Instant::now() + Duration::from_millis(400)),
    ///     Err(mpsc::RecvTimeoutError::Timeout)
    /// );
    /// ```
    #[unstable(feature = "deadline_api", issue = "46316")]
    pub fn recv_deadline(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        self.inner.recv_deadline(deadline)
    }

    /// Returns an iterator that will block waiting for messages, but never
    /// [`panic!`]. It will return [`None`] when the channel has hung up.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::mpsc::channel;
    /// use std::thread;
    ///
    /// let (send, recv) = channel();
    ///
    /// thread::spawn(move || {
    ///     send.send(1).unwrap();
    ///     send.send(2).unwrap();
    ///     send.send(3).unwrap();
    /// });
    ///
    /// let mut iter = recv.iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { rx: self }
    }

    /// Returns an iterator that will attempt to yield all pending values.
    /// It will return `None` if there are no more pending values or if the
    /// channel has hung up. The iterator will never [`panic!`] or block the
    /// user by waiting for values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::mpsc::channel;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let (sender, receiver) = channel();
    ///
    /// // nothing is in the buffer yet
    /// assert!(receiver.try_iter().next().is_none());
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_secs(1));
    ///     sender.send(1).unwrap();
    ///     sender.send(2).unwrap();
    ///     sender.send(3).unwrap();
    /// });
    ///
    /// // nothing is in the buffer yet
    /// assert!(receiver.try_iter().next().is_none());
    ///
    /// // block for two seconds
    /// thread::sleep(Duration::from_secs(2));
    ///
    /// let mut iter = receiver.try_iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[stable(feature = "receiver_try_iter", since = "1.15.0")]
    pub fn try_iter(&self) -> TryIter<'_, T> {
        TryIter { rx: self }
    }
}

// ... (other code) ...

}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

// ... (other code) ...

}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { rx: self }
    }
}

```

**Entity:** Receiver<T>

**States:** Connected (senders exist; may receive), Disconnected (no senders; may still drain buffered messages), Drained/Terminated (disconnected and buffer empty; no further messages)

**Transitions:**
- Connected -> Disconnected via dropping all corresponding Sender/SyncSender handles (external event)
- Disconnected -> Drained/Terminated via receiving all buffered messages until empty (via recv/try_recv/iter/into_iter)

**Evidence:** struct Receiver<T> { inner: mpmc::Receiver<T> } — lifecycle state is held in an erased inner channel object; recv() docs: "will always block ... if ... possible for more data to be sent (at least one sender still exists)" and "If the corresponding Sender has disconnected ... return Err"; recv() docs: "messages sent before the disconnect will still be properly received" (implies Disconnected-but-buffered intermediate state); try_recv() docs: "two failure cases ... disconnection ... empty buffer" (runtime distinction of Empty vs Disconnected state via TryRecvError); iter() docs: "return None when the channel has hung up" (termination signal depends on disconnect/drain state); Buffering example in recv() docs: after drop(send), recv() yields Ok(1), Ok(2), Ok(3), then Err(RecvError)

**Implementation:** Expose (optionally, as an alternate API) distinct receiver types for lifecycle phases, e.g., Receiver<Connected> and Receiver<Disconnected>. A method like `fn disconnect(self) -> Receiver<Disconnected>` could be produced by an explicit capability/token indicating the last sender was dropped (or by a `JoinHandle`/scope API that statically proves no senders remain). In Disconnected, `recv` could be `fn drain(self) -> IntoIter<T>` and once drained transition to a terminal type where further recv methods are unavailable.

---

### 78. Mutex poisoning protocol (Healthy / Poisoned) + recovery

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/mutex.rs:1-498`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Mutex carries an implicit global state: whether it is poisoned due to a panic while held. This state is tracked at runtime via `poison: poison::Flag` and influences the results of operations that otherwise would succeed (e.g., `lock`, `try_lock`, `into_inner`, `get_mut`, and the convenience methods `get_cloned`/`set`/`replace`). The type system does not distinguish a poisoned mutex from a healthy one, so callers must remember to handle poisoning (or explicitly clear it) via runtime `Result` handling and calling `clear_poison()` after recovery.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct MutexGuard; struct MappedMutexGuard

///
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Mutex")]
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

// ... (other code) ...

///
/// [`into_inner`]: Mutex::into_inner
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}


// ... (other code) ...

///
/// [`Rc`]: crate::rc::Rc
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}


// ... (other code) ...

/// For this reason, [`MutexGuard`] must not implement `Send` to prevent it being dropped from
/// another thread.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

/// `T` must be `Sync` for a [`MutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
#[stable(feature = "mutexguard", since = "1.19.0")]
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedMutexGuard<'_, T> {}
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedMutexGuard<'_, T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> Mutex<T> {
        Mutex { inner: sys::Mutex::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.lock() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// mutex.set(11).unwrap();
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.lock() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11).unwrap(), 7);
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.lock() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    /// The exact behavior on locking a mutex in the thread which already holds
    /// the lock is left unspecified. However, this function will not return on
    /// the second call (it might panic or deadlock, for example).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error once the mutex is acquired. The acquired
    /// mutex guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock().unwrap() = 10;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        unsafe {
            self.inner.lock();
            MutexGuard::new(self)
        }
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`Err`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return the [`Poisoned`] error if the mutex would
    /// otherwise be acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     let mut lock = c_mutex.try_lock();
    ///     if let Ok(ref mut mutex) = lock {
    ///         **mutex = 10;
    ///     } else {
    ///         println!("try_lock failed");
    ///     }
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        unsafe {
            if self.inner.try_lock() {
                Ok(MutexGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the mutex is poisoned.
    ///
    /// If another thread is active, the mutex can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    /// assert_eq!(mutex.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a mutex.
    ///
    /// If the mutex is poisoned, it will remain poisoned until this function is called. This
    /// allows recovering from a poisoned state and marking that it has recovered. For example, if
    /// the value is overwritten by a known-good value, then the mutex can be marked as
    /// un-poisoned. Or possibly, the value could be inspected to determine if it is in a
    /// consistent state, and if so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    ///
    /// assert_eq!(mutex.is_poisoned(), true);
    /// let x = mutex.lock().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     mutex.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(mutex.is_poisoned(), false);
    /// assert_eq!(*x, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the underlying data
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner().unwrap(), 0);
    /// ```
    #[stable(feature = "mutex_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previous abandoned locks
    /// (e.g., via [`forget()`] on a [`MutexGuard`]).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing a mutable reference to the
    /// underlying data instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut().unwrap() = 10;
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    ///
    /// [`forget()`]: mem::forget
    #[stable(feature = "mutex_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

#[stable(feature = "mutex_from", since = "1.24.0")]
impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

// ... (other code) ...

    }
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    unsafe fn new(lock: &'mutex Mutex<T>) -> LockResult<MutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.guard(), |guard| MutexGuard { lock, poison: guard })
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock();
        }
    }
}

// ... (other code) ...

    &guard.lock.poison
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(orig: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedMutexGuard {
            data,
            inner: &orig.lock.inner,
            poison_flag: &orig.lock.poison,
            poison: orig.poison.clone(),
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::filter_map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn filter_map<U, F>(orig: Self, f: F) -> Result<MappedMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedMutexGuard {
                    data,
                    inner: &orig.lock.inner,
                    poison_flag: &orig.lock.poison,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            }
            None => Err(orig),
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl
// ... (truncated) ...
```

**Entity:** Mutex<T>

**States:** Healthy, Poisoned

**Transitions:**
- Healthy -> Poisoned via panic while holding a MutexGuard (poison flag set through guard lifecycle)
- Poisoned -> Healthy via Mutex::clear_poison()

**Evidence:** field: `poison: poison::Flag` in `pub struct Mutex<T>`; method: `pub fn is_poisoned(&self) -> bool { self.poison.get() }` documents runtime poison state; method: `pub fn clear_poison(&self) { self.poison.clear(); }` explicitly transitions back to unpoisoned; method: `pub fn into_inner(self) -> LockResult<T>` uses `poison::map_result(self.poison.borrow(), ...)` meaning return depends on poison state; method: `pub fn get_mut(&mut self) -> LockResult<&mut T>` uses `poison::map_result(self.poison.borrow(), ...)`; comment (lock docs): "If another user of this mutex panicked while holding the mutex, then this call will return an error once the mutex is acquired."; methods: `get_cloned`, `set`, `replace` all pattern-match on `self.lock()` and return `PoisonError` on `Err(_)`

**Implementation:** Introduce a wrapper typestate around `Mutex<T>` such as `struct Mutex<T, P> { ... }` with marker states `Healthy`/`Poisoned`. Methods like `lock/get_mut/into_inner` on `Mutex<T, Healthy>` could return non-poisoning results, while `Mutex<T, Poisoned>` would force recovery paths and/or require an explicit `recover(...) -> Mutex<T, Healthy>` (or `clear_poison(self) -> Mutex<T, Healthy>`). This would make “I have acknowledged/handled poisoning” a compile-time fact for APIs that want it.

---

### 32. ReceiverFlavor channel-kind state (Array / List / Zero) with variant-specific operational protocols

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ReceiverFlavor<T> is a runtime sum type selecting one of three fundamentally different channel implementations (bounded array, unbounded list, zero-capacity rendezvous). Many receiver operations in the module (e.g., recv/try_recv/iter semantics, capacity/boundedness behavior) implicitly depend on which flavor is active. The type system does not expose the channel kind as a type-level parameter, so APIs that are only meaningful for a subset of flavors (e.g., 'capacity'-like reasoning, rendezvous-specific guarantees) must be handled by runtime matching and convention rather than being statically prevented.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Sender, 2 free function(s), impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl UnwindSafe for Sender < T > (0 methods), impl RefUnwindSafe for Sender < T > (0 methods), impl Sender < T > (2 methods), impl Sender < T > (7 methods), impl Drop for Sender < T > (1 methods); struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor

}

/// Receiver flavors.
enum ReceiverFlavor<T> {
    /// Bounded channel based on a preallocated array.
    Array(counter::Receiver<array::Channel<T>>),

    /// Unbounded channel implemented as a linked list.
    List(counter::Receiver<list::Channel<T>>),

    /// Zero-capacity channel.
    Zero(counter::Receiver<zero::Channel<T>>),
}

```

**Entity:** ReceiverFlavor<T>

**States:** Array (bounded), List (unbounded), Zero (rendezvous/zero-capacity)

**Evidence:** enum ReceiverFlavor<T> with variants: Array(counter::Receiver<array::Channel<T>>), List(counter::Receiver<list::Channel<T>>), Zero(counter::Receiver<zero::Channel<T>>); variant doc comments encode semantic states: "Bounded channel", "Unbounded channel", "Zero-capacity channel"

**Implementation:** Make channel kind a type parameter: `struct Receiver<T, K> { inner: counter::Receiver<K> }` where `K` is `array::Channel<T> | list::Channel<T> | zero::Channel<T>`. Expose constructors returning `Receiver<T, ArrayKind>`, `Receiver<T, ListKind>`, `Receiver<T, ZeroKind>`. Provide only universally-valid methods on `Receiver<T, K>` and implement flavor-specific traits/methods for particular `K` (e.g., `BoundedReceiver` for array-backed). This removes the need for runtime matching when an operation requires a specific flavor.

---

### 4. RwLock poisoning protocol (Healthy / Poisoned) affecting accessors

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-506`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: RwLock has an implicit global state ('poisoned' vs 'not poisoned') tracked at runtime by `poison: poison::Flag`. A writer panic while holding an exclusive lock transitions the lock into the Poisoned state. In Poisoned, lock acquisition APIs still acquire the underlying OS lock but then return `Err(PoisonError<Guard>)`/`Err(PoisonError<T>)` rather than `Ok`, forcing callers to handle poisoning dynamically. `clear_poison()` transitions back to Healthy, but nothing in the type system distinguishes a proven-healthy lock/guard from a possibly-poisoned one, so the protocol remains runtime/error-driven.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLockReadGuard; struct RwLockWriteGuard; struct MappedRwLockReadGuard; struct MappedRwLockWriteGuard

/// [`Mutex`]: super::Mutex
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLock")]
pub struct RwLock<T: ?Sized> {
    inner: sys::RwLock,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockReadGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockReadGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockWriteGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockWriteGuard<'_, T> {}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(5);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> RwLock<T> {
        RwLock { inner: sys::RwLock::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.read() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// lock.set(11).unwrap();
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.write() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.replace(11).unwrap(), 7);
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.write() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. The failure will occur immediately after the lock has been
    /// acquired. The acquired lock guard will be contained in the returned
    /// error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read();
    ///     assert!(r.is_ok());
    /// }).join().unwrap();
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            self.inner.read();
            RwLockReadGuard::new(self)
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access.
    ///
    /// If the access could not be granted at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// match lock.try_read() {
    ///     Ok(n) => assert_eq!(*n, 1),
    ///     Err(_) => unreachable!(),
    /// };
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            if self.inner.try_read() {
                Ok(RwLockReadGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. An error will be returned when the lock is acquired. The acquired
    /// lock guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write().unwrap();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            self.inner.write();
            RwLockWriteGuard::new(self)
        }
    }

    /// Attempts to lock this `RwLock` with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// assert!(lock.try_write().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            if self.inner.try_write() {
                Ok(RwLockWriteGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the lock is poisoned.
    ///
    /// If another thread is active, the lock can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    /// assert_eq!(lock.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a lock.
    ///
    /// If the lock is poisoned, it will remain poisoned until this function is called. This allows
    /// recovering from a poisoned state and marking that it has recovered. For example, if the
    /// value is overwritten by a known-good value, then the lock can be marked as un-poisoned. Or
    /// possibly, the value could be inspected to determine if it is in a consistent state, and if
    /// so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    ///
    /// assert_eq!(lock.is_poisoned(), true);
    /// let guard = lock.write().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     lock.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(lock.is_poisoned(), false);
    /// assert_eq!(*guard, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the underlying data if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock. An error will only be returned
    /// if the lock would have otherwise been acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write().unwrap();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner().unwrap(), "modified");
    /// ```
    #[stable(feature = "rwlock_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previously abandoned locks
    /// (e.g., via [`forget()`] on a [`RwLockReadGuard`] or [`RwLockWriteGuard`]).
    ///
    /// # Errors
    ///
    /// This function will return an error containing a mutable reference to
    /// the underlying data if the `RwLock` is poisoned. An `RwLock` is
    /// poisoned whenever a writer panics while holding an exclusive lock.
    /// An error will only be returned if the lock would have otherwise been
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(0);
    /// *lock.get_mut().unwrap() = 10;
    /// assert_eq!(*lock.read().unwrap(), 10);
    /// ```
    #[stable(feature = "rwlock_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

// ... (other code) ...

}

#[stable(feature = "rw_lock_from", since = "1.24.0")]
impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}


// ... (truncated) ...
```

**Entity:** RwLock<T>

**States:** Healthy, Poisoned

**Transitions:**
- Healthy -> Poisoned via 'writer panics while holding an exclusive lock' (documented invariant)
- Poisoned -> Healthy via clear_poison()

**Evidence:** field: `poison: poison::Flag` stores poison state at runtime; doc on get_cloned/set/replace/read/write/try_read/try_write: 'An RwLock is poisoned whenever a writer panics while holding an exclusive lock.'; method: `pub fn is_poisoned(&self) -> bool { self.poison.get() }` exposes runtime poison bit; method: `pub fn clear_poison(&self) { self.poison.clear(); }` clears poison bit (state transition); method: `pub fn into_inner(self) -> LockResult<T> { ... poison::map_result(self.poison.borrow(), |()| data) }` returns Err when poisoned; method: `pub fn get_mut(&mut self) -> LockResult<&mut T> { ... poison::map_result(self.poison.borrow(), |()| data) }` returns Err when poisoned; methods: `get_cloned`/`set`/`replace` pattern-match on `self.read()`/`self.write()` and convert `Err(_)` into `PoisonError` (poison drives control flow)

**Implementation:** Introduce a wrapper distinguishing poison state at the type level for APIs that want to require/guarantee 'unpoisoned': e.g., `struct RwLock<T, P> { .. }` with marker types `Healthy`/`PoisonedOrUnknown`. Provide `fn assume_healthy(&self) -> &RwLock<T, Healthy>` (unsafe or checked) and `fn clear_poison(self) -> RwLock<T, Healthy>` (checked transition). Guards could similarly be `RwLockReadGuard<'a, T, P>` so code that has handled poisoning can carry a `Healthy` proof and avoid repeated `LockResult` plumbing.

---

### 30. Once state machine (Incomplete / Running / Poisoned / Complete) with poison-handling protocol

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/once.rs:1-323`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `Once` has an implicit state machine managed internally by `sys::Once`. Public methods behave differently depending on whether initialization has never started (Incomplete), is currently executing (Running), previously panicked (Poisoned), or finished successfully (Complete). This protocol is enforced via runtime checks and panics, not by the type system: `call_once` and `wait` propagate poisoning by panicking, while `call_once_force` and `wait_force` ignore poison and can transition a poisoned `Once` back to Complete after a successful forced run. The API also relies on a temporal/behavioral rule that recursive `call_once` on the same instance is invalid/unspecified (panic or deadlock). None of these states (or the allowed transitions / method availability) are represented at the type level.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct OnceState; enum ExclusiveState

/// [`OnceLock<T>`]: crate::sync::OnceLock
/// [`LazyLock<T, F>`]: crate::sync::LazyLock
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Once {
    inner: sys::Once,
}

#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl UnwindSafe for Once {}

#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl RefUnwindSafe for Once {}


// ... (other code) ...

)]
pub const ONCE_INIT: Once = Once::new();

impl Once {
    /// Creates a new `Once` value.
    #[inline]
    #[stable(feature = "once_new", since = "1.2.0")]
    #[rustc_const_stable(feature = "const_once_new", since = "1.32.0")]
    #[must_use]
    pub const fn new() -> Once {
        Once { inner: sys::Once::new() }
    }

    /// Performs an initialization routine once and only once. The given closure
    /// will be executed if this is the first time `call_once` has been called,
    /// and otherwise the routine will *not* be invoked.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// When this function returns, it is guaranteed that some initialization
    /// has run and completed (it might not be the closure specified). It is also
    /// guaranteed that any memory writes performed by the executed closure can
    /// be reliably observed by other threads at this point (there is a
    /// happens-before relation between the closure and code executing after the
    /// return).
    ///
    /// If the given closure recursively invokes `call_once` on the same [`Once`]
    /// instance, the exact behavior is not specified: allowed outcomes are
    /// a panic or a deadlock.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static mut VAL: usize = 0;
    /// static INIT: Once = Once::new();
    ///
    /// // Accessing a `static mut` is unsafe much of the time, but if we do so
    /// // in a synchronized fashion (e.g., write once or read all) then we're
    /// // good to go!
    /// //
    /// // This function will only call `expensive_computation` once, and will
    /// // otherwise always return the value returned from the first invocation.
    /// fn get_cached_val() -> usize {
    ///     unsafe {
    ///         INIT.call_once(|| {
    ///             VAL = expensive_computation();
    ///         });
    ///         VAL
    ///     }
    /// }
    ///
    /// fn expensive_computation() -> usize {
    ///     // ...
    /// # 2
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The closure `f` will only be executed once even if this is called
    /// concurrently amongst many threads. If that closure panics, however, then
    /// it will *poison* this [`Once`] instance, causing all future invocations of
    /// `call_once` to also panic.
    ///
    /// This is similar to [poisoning with mutexes][poison].
    ///
    /// [poison]: struct.Mutex.html#poisoning
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[track_caller]
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(false, &mut |_| f.take().unwrap()());
    }

    /// Performs the same function as [`call_once()`] except ignores poisoning.
    ///
    /// Unlike [`call_once()`], if this [`Once`] has been poisoned (i.e., a previous
    /// call to [`call_once()`] or [`call_once_force()`] caused a panic), calling
    /// [`call_once_force()`] will still invoke the closure `f` and will _not_
    /// result in an immediate panic. If `f` panics, the [`Once`] will remain
    /// in a poison state. If `f` does _not_ panic, the [`Once`] will no
    /// longer be in a poison state and all future calls to [`call_once()`] or
    /// [`call_once_force()`] will be no-ops.
    ///
    /// The closure `f` is yielded a [`OnceState`] structure which can be used
    /// to query the poison status of the [`Once`].
    ///
    /// [`call_once()`]: Once::call_once
    /// [`call_once_force()`]: Once::call_once_force
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// // poisoning propagates
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| {});
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// // call_once_force will still run and reset the poisoned state
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    ///
    /// // once any success happens, we stop propagating the poison
    /// INIT.call_once(|| {});
    /// ```
    #[inline]
    #[stable(feature = "once_poison", since = "1.51.0")]
    pub fn call_once_force<F>(&self, f: F)
    where
        F: FnOnce(&OnceState),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(true, &mut |p| f.take().unwrap()(p));
    }

    /// Returns `true` if some [`call_once()`] call has completed
    /// successfully. Specifically, `is_completed` will return false in
    /// the following situations:
    ///   * [`call_once()`] was not called at all,
    ///   * [`call_once()`] was called, but has not yet completed,
    ///   * the [`Once`] instance is poisoned
    ///
    /// This function returning `false` does not mean that [`Once`] has not been
    /// executed. For example, it may have been executed in the time between
    /// when `is_completed` starts executing and when it returns, in which case
    /// the `false` return value would be stale (but still permissible).
    ///
    /// [`call_once()`]: Once::call_once
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// INIT.call_once(|| {
    ///     assert_eq!(INIT.is_completed(), false);
    /// });
    /// assert_eq!(INIT.is_completed(), true);
    /// ```
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    /// assert_eq!(INIT.is_completed(), false);
    /// ```
    #[stable(feature = "once_is_completed", since = "1.43.0")]
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }

    /// Blocks the current thread until initialization has completed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static READY: Once = Once::new();
    ///
    /// let thread = thread::spawn(|| {
    ///     READY.wait();
    ///     println!("everything is ready");
    /// });
    ///
    /// READY.call_once(|| println!("performing setup"));
    /// ```
    ///
    /// # Panics
    ///
    /// If this [`Once`] has been poisoned because an initialization closure has
    /// panicked, this method will also panic. Use [`wait_force`](Self::wait_force)
    /// if this behavior is not desired.
    #[stable(feature = "once_wait", since = "1.86.0")]
    pub fn wait(&self) {
        if !self.inner.is_completed() {
            self.inner.wait(false);
        }
    }

    /// Blocks the current thread until initialization has completed, ignoring
    /// poisoning.
    #[stable(feature = "once_wait", since = "1.86.0")]
    pub fn wait_force(&self) {
        if !self.inner.is_completed() {
            self.inner.wait(true);
        }
    }

    /// Returns the current state of the `Once` instance.
    ///
    /// Since this takes a mutable reference, no initialization can currently
    /// be running, so the state must be either "incomplete", "poisoned" or
    /// "complete".
    #[inline]
    pub(crate) fn state(&mut self) -> ExclusiveState {
        self.inner.state()
    }

    /// Sets current state of the `Once` instance.
    ///
    /// Since this takes a mutable reference, no initialization can currently
    /// be running, so the state must be either "incomplete", "poisoned" or
    /// "complete".
    #[inline]
    pub(crate) fn set_state(&mut self, new_state: ExclusiveState) {
        self.inner.set_state(new_state);
    }
}

// ... (other code) ...

    }
}

impl OnceState {
    /// Returns `true` if the associated [`Once`] was poisoned prior to the
    /// invocation of the closure passed to [`Once::call_once_force()`].
    ///
    /// # Examples
    ///
    /// A poisoned [`Once`]:
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    /// ```
    ///
    /// An unpoisoned [`Once`]:
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(!state.is_poisoned());
    /// });
    #[stable(feature = "once_poison", since = "1.51.0")]
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Poison the associated [`Once`] without explicitly panicking.
    // NOTE: This is currently only exposed for `OnceLock`.
    #[inline]
    pub(crate) fn poison(&self) {
        self.inner.poison();
    }
}

```

**Entity:** Once

**States:** Incomplete, Running, Poisoned, Complete

**Transitions:**
- Incomplete -> Running via call_once()/call_once_force() (starts executing closure)
- Running -> Complete via successful closure completion (subsequent calls are no-ops via fast-path)
- Running -> Poisoned via closure panic in call_once() or call_once_force()
- Poisoned -> (panic) on call_once()/wait() (poison propagates)
- Poisoned -> Running via call_once_force() (forced execution despite poison)
- Poisoned -> Complete via call_once_force() when forced closure does not panic (poison cleared per docs)
- Any -> (blocked) via wait()/wait_force() until not Running (with wait() panicking if Poisoned)

**Evidence:** struct Once { inner: sys::Once } — state is stored in `sys::Once` rather than encoded in the type; call_once(): `if self.inner.is_completed() { return; }` fast-path indicates a 'Complete' terminal state; call_once(): docs under `# Panics` say a panic in the closure will "poison" the Once and future call_once will also panic; call_once_force(): docs explicitly define poison-ignoring behavior and that a non-panicking forced call clears poison and makes future calls no-ops; wait(): docs say it panics if poisoned; implementation calls `self.inner.wait(false)` when not completed; wait_force(): ignores poisoning; implementation calls `self.inner.wait(true)` when not completed; is_completed(): docs define that it returns false if never called, not yet completed, or poisoned; docs in call_once(): "If the given closure recursively invokes call_once on the same Once instance, ... allowed outcomes are a panic or a deadlock" (implicit precondition/protocol not enforced); state()/set_state(): comments: "Since this takes a mutable reference, no initialization can currently be running" and "state must be either incomplete, poisoned or complete" (implicit exclusion of Running, enforced by convention plus &mut requirement)

**Implementation:** Internally (not necessarily in the public API), model `Once` as `Once<S>` with ZST states like `Incomplete`, `Running`, `Poisoned`, `Complete`. Expose transitions as consuming methods on an owned guard representing the running initialization (e.g., `begin(self) -> Once<Running>` returning a guard that on drop sets Poisoned unless explicitly committed to Complete). Provide separate handles/capabilities for poison-propagating vs poison-ignoring operations (e.g., `PoisonPropagating` vs `PoisonIgnoring` token) so `wait()`/`call_once()` are only callable when using the propagating capability, while `*_force` requires an explicit ignoring capability. This would move parts of the runtime protocol (especially the poison-handling choice and the 'Running' guard behavior) into the type system.

---

### 38. BarrierWaitResult role flag protocol (Leader / Follower)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/barrier.rs:1-7`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: BarrierWaitResult encodes (as a private bool) which role the current thread had when released from Barrier::wait(): exactly one waiter is designated the 'leader' and all others are 'followers'. This is an implicit protocol encoded as a boolean rather than distinct types, so callers can only query it dynamically (via a method in the unseen impl), and cannot have the leader-only/follower-only control flow enforced by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Barrier, impl Barrier (2 methods), impl BarrierWaitResult (1 methods); struct BarrierState

/// let barrier_wait_result = barrier.wait();
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BarrierWaitResult(bool);


```

**Entity:** BarrierWaitResult

**States:** Follower, Leader

**Transitions:**
- (constructed by Barrier::wait()) -> Leader or Follower depending on internal barrier state

**Evidence:** pub struct BarrierWaitResult(bool); — a private boolean field encodes a latent two-state result; doc snippet: `let barrier_wait_result = barrier.wait();` implies Barrier::wait() produces this stateful result

**Implementation:** Replace the bool-flag wrapper with a sum type: `enum BarrierWaitResult { Leader, Follower }` (or two zero-sized newtypes). Provide `is_leader()` as a match, or return a `LeaderToken` capability to make leader-only actions require possession of the token.

---

### 23. LazyLock initialization/poisoning state machine (Incomplete / Complete / Poisoned) coupled to union field validity

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/lazy_lock.rs:1-354`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: LazyLock has an implicit 3-state protocol encoded in `once` (via `ExclusiveState`) that determines which `union Data<T,F>` field is valid to read/drop. In `Incomplete`, `data.f` must be present and may be taken exactly once to initialize. In `Complete`, `data.value` must be present and immutable thereafter. If initialization panics, the `Once` becomes `Poisoned` and callers must never read either union field (operations panic/short-circuit instead). These invariants are maintained via runtime state checks plus unsafe union field access, but the type system does not express which field is initialized, nor prevent calling methods that assume the wrong state (it relies on `once.state()`, `once.call_once`, `is_completed()`, and panics).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct LazyLock, impl LazyLock < T , F > (5 methods), impl LazyLock < T , F > (2 methods), impl Drop for LazyLock < T , F > (1 methods), impl Deref for LazyLock < T , F > (1 methods), impl Sync for LazyLock < T , F > (0 methods), impl RefUnwindSafe for LazyLock < T , F > (0 methods), impl UnwindSafe for LazyLock < T , F > (0 methods)

use super::poison::once::ExclusiveState;
use crate::cell::UnsafeCell;
use crate::mem::ManuallyDrop;
use crate::ops::Deref;
use crate::panic::{RefUnwindSafe, UnwindSafe};
use crate::sync::Once;
use crate::{fmt, ptr};

// We use the state of a Once as discriminant value. Upon creation, the state is
// "incomplete" and `f` contains the initialization closure. In the first call to
// `call_once`, `f` is taken and run. If it succeeds, `value` is set and the state
// is changed to "complete". If it panics, the Once is poisoned, so none of the
// two fields is initialized.
union Data<T, F> {
    value: ManuallyDrop<T>,
    f: ManuallyDrop<F>,
}

/// A value which is initialized on the first access.
///
/// This type is a thread-safe [`LazyCell`], and can be used in statics.
/// Since initialization may be called from multiple threads, any
/// dereferencing call will block the calling thread if another
/// initialization routine is currently running.
///
/// [`LazyCell`]: crate::cell::LazyCell
///
/// # Examples
///
/// Initialize static variables with `LazyLock`.
/// ```
/// use std::sync::LazyLock;
///
/// // Note: static items do not call [`Drop`] on program termination, so this won't be deallocated.
/// // this is fine, as the OS can deallocate the terminated program faster than we can free memory
/// // but tools like valgrind might report "memory leaks" as it isn't obvious this is intentional.
/// static DEEP_THOUGHT: LazyLock<String> = LazyLock::new(|| {
/// # mod another_crate {
/// #     pub fn great_question() -> String { "42".to_string() }
/// # }
///     // M3 Ultra takes about 16 million years in --release config
///     another_crate::great_question()
/// });
///
/// // The `String` is built, stored in the `LazyLock`, and returned as `&String`.
/// let _ = &*DEEP_THOUGHT;
/// ```
///
/// Initialize fields with `LazyLock`.
/// ```
/// use std::sync::LazyLock;
///
/// #[derive(Debug)]
/// struct UseCellLock {
///     number: LazyLock<u32>,
/// }
/// fn main() {
///     let lock: LazyLock<u32> = LazyLock::new(|| 0u32);
///
///     let data = UseCellLock { number: lock };
///     println!("{}", *data.number);
/// }
/// ```
#[stable(feature = "lazy_cell", since = "1.80.0")]
pub struct LazyLock<T, F = fn() -> T> {
    // FIXME(nonpoison_once): if possible, switch to nonpoison version once it is available
    once: Once,
    data: UnsafeCell<Data<T, F>>,
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Creates a new lazy value with the given initializing function.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::LazyLock;
    ///
    /// let hello = "Hello, World!".to_string();
    ///
    /// let lazy = LazyLock::new(|| hello.to_uppercase());
    ///
    /// assert_eq!(&*lazy, "HELLO, WORLD!");
    /// ```
    #[inline]
    #[stable(feature = "lazy_cell", since = "1.80.0")]
    #[rustc_const_stable(feature = "lazy_cell", since = "1.80.0")]
    pub const fn new(f: F) -> LazyLock<T, F> {
        LazyLock { once: Once::new(), data: UnsafeCell::new(Data { f: ManuallyDrop::new(f) }) }
    }

    /// Creates a new lazy value that is already initialized.
    #[inline]
    #[cfg(test)]
    pub(crate) fn preinit(value: T) -> LazyLock<T, F> {
        let once = Once::new();
        once.call_once(|| {});
        LazyLock { once, data: UnsafeCell::new(Data { value: ManuallyDrop::new(value) }) }
    }

    /// Consumes this `LazyLock` returning the stored value.
    ///
    /// Returns `Ok(value)` if `Lazy` is initialized and `Err(f)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_cell_into_inner)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let hello = "Hello, World!".to_string();
    ///
    /// let lazy = LazyLock::new(|| hello.to_uppercase());
    ///
    /// assert_eq!(&*lazy, "HELLO, WORLD!");
    /// assert_eq!(LazyLock::into_inner(lazy).ok(), Some("HELLO, WORLD!".to_string()));
    /// ```
    #[unstable(feature = "lazy_cell_into_inner", issue = "125623")]
    pub fn into_inner(mut this: Self) -> Result<T, F> {
        let state = this.once.state();
        match state {
            ExclusiveState::Poisoned => panic_poisoned(),
            state => {
                let this = ManuallyDrop::new(this);
                let data = unsafe { ptr::read(&this.data) }.into_inner();
                match state {
                    ExclusiveState::Incomplete => Err(ManuallyDrop::into_inner(unsafe { data.f })),
                    ExclusiveState::Complete => Ok(ManuallyDrop::into_inner(unsafe { data.value })),
                    ExclusiveState::Poisoned => unreachable!(),
                }
            }
        }
    }

    /// Forces the evaluation of this lazy value and returns a mutable reference to
    /// the result.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    /// use std::sync::LazyLock;
    ///
    /// let mut lazy = LazyLock::new(|| 92);
    ///
    /// let p = LazyLock::force_mut(&mut lazy);
    /// assert_eq!(*p, 92);
    /// *p = 44;
    /// assert_eq!(*lazy, 44);
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn force_mut(this: &mut LazyLock<T, F>) -> &mut T {
        #[cold]
        /// # Safety
        /// May only be called when the state is `Incomplete`.
        unsafe fn really_init_mut<T, F: FnOnce() -> T>(this: &mut LazyLock<T, F>) -> &mut T {
            struct PoisonOnPanic<'a, T, F>(&'a mut LazyLock<T, F>);
            impl<T, F> Drop for PoisonOnPanic<'_, T, F> {
                #[inline]
                fn drop(&mut self) {
                    self.0.once.set_state(ExclusiveState::Poisoned);
                }
            }

            // SAFETY: We always poison if the initializer panics (then we never check the data),
            // or set the data on success.
            let f = unsafe { ManuallyDrop::take(&mut this.data.get_mut().f) };
            // INVARIANT: Initiated from mutable reference, don't drop because we read it.
            let guard = PoisonOnPanic(this);
            let data = f();
            guard.0.data.get_mut().value = ManuallyDrop::new(data);
            guard.0.once.set_state(ExclusiveState::Complete);
            core::mem::forget(guard);
            // SAFETY: We put the value there above.
            unsafe { &mut this.data.get_mut().value }
        }

        let state = this.once.state();
        match state {
            ExclusiveState::Poisoned => panic_poisoned(),
            // SAFETY: The `Once` states we completed the initialization.
            ExclusiveState::Complete => unsafe { &mut this.data.get_mut().value },
            // SAFETY: The state is `Incomplete`.
            ExclusiveState::Incomplete => unsafe { really_init_mut(this) },
        }
    }

    /// Forces the evaluation of this lazy value and returns a reference to
    /// result. This is equivalent to the `Deref` impl, but is explicit.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::LazyLock;
    ///
    /// let lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::force(&lazy), &92);
    /// assert_eq!(&*lazy, &92);
    /// ```
    #[inline]
    #[stable(feature = "lazy_cell", since = "1.80.0")]
    pub fn force(this: &LazyLock<T, F>) -> &T {
        this.once.call_once(|| {
            // SAFETY: `call_once` only runs this closure once, ever.
            let data = unsafe { &mut *this.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });

        // SAFETY:
        // There are four possible scenarios:
        // * the closure was called and initialized `value`.
        // * the closure was called and panicked, so this point is never reached.
        // * the closure was not called, but a previous call initialized `value`.
        // * the closure was not called because the Once is poisoned, so this point
        //   is never reached.
        // So `value` has definitely been initialized and will not be modified again.
        unsafe { &*(*this.data.get()).value }
    }
}

impl<T, F> LazyLock<T, F> {
    /// Returns a mutable reference to the value if initialized, or `None` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let mut lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::get_mut(&mut lazy), None);
    /// let _ = LazyLock::force(&lazy);
    /// *LazyLock::get_mut(&mut lazy).unwrap() = 44;
    /// assert_eq!(*lazy, 44);
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn get_mut(this: &mut LazyLock<T, F>) -> Option<&mut T> {
        // `state()` does not perform an atomic load, so prefer it over `is_complete()`.
        let state = this.once.state();
        match state {
            // SAFETY:
            // The closure has been run successfully, so `value` has been initialized.
            ExclusiveState::Complete => Some(unsafe { &mut this.data.get_mut().value }),
            _ => None,
        }
    }

    /// Returns a reference to the value if initialized, or `None` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lazy_get)]
    ///
    /// use std::sync::LazyLock;
    ///
    /// let lazy = LazyLock::new(|| 92);
    ///
    /// assert_eq!(LazyLock::get(&lazy), None);
    /// let _ = LazyLock::force(&lazy);
    /// assert_eq!(LazyLock::get(&lazy), Some(&92));
    /// ```
    #[inline]
    #[unstable(feature = "lazy_get", issue = "129333")]
    pub fn get(this: &LazyLock<T, F>) -> Option<&T> {
        if this.once.is_completed() {
            // SAFETY:
            // The closure has been run successfully, so `value` has been initialized
            // and will not be modified again.
            Some(unsafe { &(*this.data.get()).value })
        } else {
            None
        }
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T, F> Drop for LazyLock<T, F> {
    fn drop(&mut self) {
        match self.once.state() {
            ExclusiveState::Incomplete => unsafe { ManuallyDrop::drop(&mut self.data.get_mut().f) },
            ExclusiveState::Complete => unsafe {
                ManuallyDrop::drop(&mut self.data.get_mut().value)
            },
            ExclusiveState::Poisoned => {}
        }
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;

    /// Dereferences the value.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    #[inline]
    fn deref(&self) -> &T {
        LazyLock::force(self)
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: Default> Default for LazyLock<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    #[inline]
    fn default() -> LazyLock<T> {
        LazyLock::new(T::default)
    }
}

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: fmt::Debug, F> fmt::Debug for LazyLock<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("LazyLock");
        match LazyLock::get(self) {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

#[cold]
#[inline(never)]
fn panic_poisoned() -> ! {
    panic!("LazyLock instance has previously been poisoned")
}

// We never create a `&F` from a `&LazyLock<T, F>` so it is fine
// to not impl `Sync` for `F`.
#[stable(feature = "lazy_cell", since = "1.80.0")]
unsafe impl<T: Sync + Send, F: Send> Sync for LazyLock<T, F> {}
// auto-derived `Send` impl is OK.

#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: RefUnwindSafe + UnwindSafe, F: UnwindSafe> RefUnwindSafe for LazyLock<T, F> {}
#[stable(feature = "lazy_cell", since = "1.80.0")]
impl<T: UnwindSafe, F: UnwindSafe> UnwindSafe for LazyLock<T, F> {}

```

**Entity:** LazyLock<T, F>

**States:** Incomplete (data.f valid), Complete (data.value valid), Poisoned (neither field may be assumed valid)

**Transitions:**
- Incomplete -> Complete via force()/Deref (once.call_once runs initializer and writes data.value)
- Incomplete -> Complete via force_mut() (really_init_mut runs initializer and writes data.value, then set_state(Complete))
- Incomplete -> Poisoned via panic during initialization (Once poisoning / PoisonOnPanic drop sets state to Poisoned)
- Complete -> (no further transitions; value must not be modified except through &mut access paths)
- Poisoned -> (terminal; operations that require a value panic)

**Evidence:** comment above `union Data<T, F>`: 'We use the state of a Once as discriminant value... If it panics, the Once is poisoned, so none of the two fields is initialized.'; field `once: Once` and use of `ExclusiveState` in into_inner()/force_mut()/get_mut()/Drop; field `data: UnsafeCell<Data<T, F>>` where `Data` is a `union` of `value` vs `f`; method `into_inner`: `let state = this.once.state(); match state { ExclusiveState::Incomplete => Err(... data.f ...), ExclusiveState::Complete => Ok(... data.value ...), ExclusiveState::Poisoned => panic_poisoned() }`; method `force_mut`: matches on `this.once.state()`; `ExclusiveState::Complete => unsafe { &mut this.data.get_mut().value }`, `ExclusiveState::Incomplete => unsafe { really_init_mut(this) }`, `ExclusiveState::Poisoned => panic_poisoned()`; nested `really_init_mut` safety comment: 'May only be called when the state is `Incomplete`.' and `PoisonOnPanic` Drop sets `self.0.once.set_state(ExclusiveState::Poisoned)`; method `force`: `this.once.call_once(|| { ... ManuallyDrop::take(&mut data.f) ... data.value = ... })` and later `unsafe { &*(*this.data.get()).value }` with a comment enumerating scenarios, excluding poisoned/failed init by panicking earlier; Drop impl: `match self.once.state() { Incomplete => drop(data.f), Complete => drop(data.value), Poisoned => {} }` relies on the state<->active-union-field invariant

**Implementation:** Represent the Once-discriminated union at the type level: e.g., `LazyLock<T, F, S>` with `S` in {Uninit, Init, Poisoned} (or split types `LazyLockUninit<T,F>` and `LazyLockInit<T>`). Make `new` return `LazyLock<_,_,Uninit>`, `force/force_mut` consume or return a guard that yields `&T`/`&mut T` while transitioning to `Init` internally, and make `into_inner` return `Result<T, F>` only for Uninit/Init with Poisoned represented as a distinct type or error. This would eliminate unsafe union reads based on runtime `ExclusiveState` checks and make invalid-field access unrepresentable.

---

### 70. Sender endpoint liveness protocol (Connected / Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-178`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: A Sender has an implicit liveness state tied to whether the corresponding Receiver still exists. send() only succeeds if, at the moment of the call, the receiver has not hung up; it returns Err(SendError<T>) once the receiver is deallocated. The type system does not represent this connected/disconnected state, so callers must handle it dynamically, and even an Ok(()) does not guarantee the message will ultimately be observed (receiver may disconnect immediately after).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// assert_eq!(3, msg + msg2);
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Sender<T> {
    inner: mpmc::Sender<T>,
}

// ... (other code) ...

// The send port can be sent from place to place, so long as it
// is not used to send non-sendable things.
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: Send> Send for Sender<T> {}

#[stable(feature = "mpsc_sender_sync", since = "1.72.0")]
unsafe impl<T: Send> Sync for Sender<T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: Send> Send for SyncSender<T> {}


// ... (other code) ...

// Sender
////////////////////////////////////////////////////////////////////////////////

impl<T> Sender<T> {
    /// Attempts to send a value on this channel, returning it back if it could
    /// not be sent.
    ///
    /// A successful send occurs when it is determined that the other end of
    /// the channel has not hung up already. An unsuccessful send would be one
    /// where the corresponding receiver has already been deallocated. Note
    /// that a return value of [`Err`] means that the data will never be
    /// received, but a return value of [`Ok`] does *not* mean that the data
    /// will be received. It is possible for the corresponding receiver to
    /// hang up immediately after this function returns [`Ok`].
    ///
    /// This method will never block the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::mpsc::channel;
    ///
    /// let (tx, rx) = channel();
    ///
    /// // This send is always successful
    /// tx.send(1).unwrap();
    ///
    /// // This send will fail because the receiver is gone
    /// drop(rx);
    /// assert_eq!(tx.send(1).unwrap_err().0, 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.inner.send(t)
    }
}

// ... (other code) ...

// SyncSender
////////////////////////////////////////////////////////////////////////////////

impl<T> SyncSender<T> {
    /// Sends a value on this synchronous channel.
    ///
    /// This function will *block* until space in the internal buffer becomes
    /// available or a receiver is available to hand off the message to.
    ///
    /// Note that a successful send does *not* guarantee that the receiver will
    /// ever see the data if there is a buffer on this channel. Items may be
    /// enqueued in the internal buffer for the receiver to receive at a later
    /// time. If the buffer size is 0, however, the channel becomes a rendezvous
    /// channel and it guarantees that the receiver has indeed received
    /// the data if this function returns success.
    ///
    /// This function will never panic, but it may return [`Err`] if the
    /// [`Receiver`] has disconnected and is no longer able to receive
    /// information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::mpsc::sync_channel;
    /// use std::thread;
    ///
    /// // Create a rendezvous sync_channel with buffer size 0
    /// let (sync_sender, receiver) = sync_channel(0);
    ///
    /// thread::spawn(move || {
    ///    println!("sending message...");
    ///    sync_sender.send(1).unwrap();
    ///    // Thread is now blocked until the message is received
    ///
    ///    println!("...message received!");
    /// });
    ///
    /// let msg = receiver.recv().unwrap();
    /// assert_eq!(1, msg);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.inner.send(t)
    }

    /// Attempts to send a value on this channel without blocking.
    ///
    /// This method differs from [`send`] by returning immediately if the
    /// channel's buffer is full or no receiver is waiting to acquire some
    /// data. Compared with [`send`], this function has two failure cases
    /// instead of one (one for disconnection, one for a full buffer).
    ///
    /// See [`send`] for notes about guarantees of whether the
    /// receiver has received the data or not if this function is successful.
    ///
    /// [`send`]: Self::send
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::mpsc::sync_channel;
    /// use std::thread;
    ///
    /// // Create a sync_channel with buffer size 1
    /// let (sync_sender, receiver) = sync_channel(1);
    /// let sync_sender2 = sync_sender.clone();
    ///
    /// // First thread owns sync_sender
    /// thread::spawn(move || {
    ///     sync_sender.send(1).unwrap();
    ///     sync_sender.send(2).unwrap();
    ///     // Thread blocked
    /// });
    ///
    /// // Second thread owns sync_sender2
    /// thread::spawn(move || {
    ///     // This will return an error and send
    ///     // no message if the buffer is full
    ///     let _ = sync_sender2.try_send(3);
    /// });
    ///
    /// let mut msg;
    /// msg = receiver.recv().unwrap();
    /// println!("message {msg} received");
    ///
    /// msg = receiver.recv().unwrap();
    /// println!("message {msg} received");
    ///
    /// // Third message may have never been sent
    /// match receiver.try_recv() {
    ///     Ok(msg) => println!("message {msg} received"),
    ///     Err(_) => println!("the third message was never sent"),
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_send(&self, t: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(t)
    }

    // Attempts to send for a value on this receiver, returning an error if the
    // corresponding channel has hung up, or if it waits more than `timeout`.
    //
    // This method is currently only used for tests.
    #[unstable(issue = "none", feature = "std_internals")]
    #[doc(hidden)]
    pub fn send_timeout(&self, t: T, timeout: Duration) -> Result<(), mpmc::SendTimeoutError<T>> {
        self.inner.send_timeout(t, timeout)
    }
}

```

**Entity:** Sender<T>

**States:** Connected (receiver still alive), Disconnected (receiver dropped)

**Transitions:**
- Connected -> Disconnected when the corresponding Receiver is dropped (external event)

**Evidence:** struct Sender<T> { inner: mpmc::Sender<T> } — liveness is managed by the shared inner channel state, not represented in Sender's type; Sender::send(&self, t: T) -> Result<(), SendError<T>> delegates to self.inner.send(t), indicating runtime failure on state; doc comment on Sender::send: "An unsuccessful send would be one where the corresponding receiver has already been deallocated"; doc comment on Sender::send: "a return value of Ok does not mean that the data will be received... receiver to hang up immediately after this function returns Ok"; example in Sender::send docs: drop(rx); assert_eq!(tx.send(1).unwrap_err().0, 1); demonstrates Disconnected state

**Implementation:** Model an explicit 'connection token' capability that is consumed/invalidated when the receiver is dropped (e.g., a ReceiverAlive guard handed out by Receiver and checked by Sender), so APIs that require a live receiver take &ReceiverAlive. This cannot fully prevent race-y disconnect-after-Ok, but can make "receiver already dropped" statically impossible in structured single-owner/channel-scoped code (e.g., when Sender is only usable while holding a Receiver borrow/guard).

---

### 39. BarrierState phase protocol (Counting -> Releasing via generation_id)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/barrier.rs:1-9`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: BarrierState encodes a barrier’s progress using raw integers: `count` tracks how many participants have arrived in the current generation, and `generation_id` distinguishes successive barrier cycles. Correctness relies on implicit invariants: `count` must be within an expected range for the configured barrier size, `generation_id` must monotonically increase on each barrier trip, and operations must interpret `count` relative to the current `generation_id` so that wakeups/arrivals from an old generation don’t satisfy the next one. None of these rules are enforced by the type system because both fields are plain `usize` without range/monotonicity guarantees or state-typed methods.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Barrier, impl Barrier (2 methods), impl BarrierWaitResult (1 methods); struct BarrierWaitResult

}

// The inner state of a double barrier
struct BarrierState {
    count: usize,
    generation_id: usize,
}

```

**Entity:** BarrierState

**States:** Counting (collecting arrivals), Releasing (trip/completion for a generation)

**Transitions:**
- Counting -> Releasing by incrementing `count` up to the barrier size and completing the generation (implied by `generation_id` field)
- Releasing -> Counting by resetting `count` for the next generation and incrementing `generation_id`

**Evidence:** `struct BarrierState { count: usize, generation_id: usize }` — two integer fields encode the runtime state machine; `count` field name implies a numeric arrival counter used to gate progress; `generation_id` field name implies a generational protocol to separate barrier cycles

**Implementation:** Model the barrier’s inner state as a typed state machine rather than two raw integers, e.g. `BarrierState<S>` with `S = Counting | Releasing`, and use newtypes for invariants: `struct GenerationId(NonZeroUsize)` (monotonic, non-wrapping semantics) and `struct ArrivalCount<const N: usize>(usize)` ensuring `<= N`. Transitions become consuming methods like `fn arrive(self, ...) -> BarrierState<...>` that only allow `count` changes in `Counting` and only allow `generation_id` increment/reset on the trip transition.

---

### 50. SyncWaker emptiness protocol (Empty / NonEmpty) mirrored between Atomic flag and inner Waker

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: SyncWaker encodes whether it currently holds any wake targets using a runtime atomic flag (is_empty) in addition to the actual contents of the inner Waker (behind a Mutex). This implies an invariant that `is_empty` must accurately reflect whether `inner` is logically empty, and that state transitions between Empty and NonEmpty must update both consistently and in the right order. The type system does not prevent clients (or internal methods) from reading a stale/incorrect `is_empty` value relative to `inner`, nor does it enforce that methods requiring a non-empty waker are only callable in the NonEmpty state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Entry; struct Waker, impl Waker (7 methods), impl Drop for Waker (1 methods), impl SyncWaker (5 methods), impl Drop for SyncWaker (1 methods); 1 free function(s)

/// A waker that can be shared among threads without locking.
///
/// This is a simple wrapper around `Waker` that internally uses a mutex for synchronization.
pub(crate) struct SyncWaker {
    /// The inner `Waker`.
    inner: Mutex<Waker>,

    /// `true` if the waker is empty.
    is_empty: Atomic<bool>,
}

```

**Entity:** SyncWaker

**States:** Empty, NonEmpty

**Transitions:**
- Empty -> NonEmpty via (some SyncWaker method that registers/adds a waker into inner)
- NonEmpty -> Empty via (some SyncWaker method that clears/drains/removes from inner)

**Evidence:** field: `is_empty: Atomic<bool>` — explicit runtime state flag; comment: "`true` if the waker is empty." — defines the implicit state meaning of `is_empty`; field: `inner: Mutex<Waker>` — actual state lives behind a mutex but is also mirrored by `is_empty`

**Implementation:** Split into `SyncWaker<Empty>` and `SyncWaker<NonEmpty>` (or `SyncWaker<State>` with PhantomData). Provide transitions like `fn add(self, ...) -> SyncWaker<NonEmpty>` and `fn clear(self) -> SyncWaker<Empty>`. Alternatively, eliminate the duplicated flag by making emptiness queries depend on the locked `inner`, or encapsulate atomic+mutex update ordering in a private helper that returns a capability token proving NonEmpty for methods that require it.

---

### 17. Poison flag protocol (Unpoisoned / Poisoned, with guarded detection)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison.rs:1-411`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Flag encodes a two-state poison marker in an AtomicBool (`failed`). Callers are expected to consult the flag before using protected data (`borrow()` or `guard()`), and (in the guarded case) to later call `done()` with the corresponding `Guard` so the flag can transition to Poisoned if the current scope panicked while holding the lock. None of this sequencing (guard() -> done()) is enforced by the type system; it relies on consumers to call `done()` at the right time with the right `Guard`, and to call `clear()` only when it is logically safe to forgive poisoning.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Flag, impl Flag (8 methods); struct Guard; struct PoisonError, impl Error for PoisonError < T > (1 methods), impl PoisonError < T > (5 methods); enum TryLockError, impl From < PoisonError < T > > for TryLockError < T > (1 methods), impl Error for TryLockError < T > (2 methods)

//! Synchronization objects that employ poisoning.
//!
//! # Poisoning
//!
//! All synchronization objects in this module implement a strategy called "poisoning"
//! where if a thread panics while holding the exclusive access granted by the primitive,
//! the state of the primitive is set to "poisoned".
//! This information is then propagated to all other threads
//! to signify that the data protected by this primitive is likely tainted
//! (some invariant is not being upheld).
//!
//! The specifics of how this "poisoned" state affects other threads
//! depend on the primitive. See [#Overview] bellow.
//!
//! For the alternative implementations that do not employ poisoning,
//! see `std::sys::nonpoisoning`.
//!
//! # Overview
//!
//! Below is a list of synchronization objects provided by this module
//! with a high-level overview for each object and a description
//! of how it employs "poisoning".
//!
//! - [`Condvar`]: Condition Variable, providing the ability to block
//!   a thread while waiting for an event to occur.
//!
//!   Condition variables are typically associated with
//!   a boolean predicate (a condition) and a mutex.
//!   This implementation is associated with [`poison::Mutex`](Mutex),
//!   which employs poisoning.
//!   For this reason, [`Condvar::wait()`] will return a [`LockResult`],
//!   just like [`poison::Mutex::lock()`](Mutex::lock) does.
//!
//! - [`Mutex`]: Mutual Exclusion mechanism, which ensures that at
//!   most one thread at a time is able to access some data.
//!
//!   [`Mutex::lock()`] returns a [`LockResult`],
//!   providing a way to deal with the poisoned state.
//!   See [`Mutex`'s documentation](Mutex#poisoning) for more.
//!
//! - [`Once`]: A thread-safe way to run a piece of code only once.
//!   Mostly useful for implementing one-time global initialization.
//!
//!   [`Once`] is poisoned if the piece of code passed to
//!   [`Once::call_once()`] or [`Once::call_once_force()`] panics.
//!   When in poisoned state, subsequent calls to [`Once::call_once()`] will panic too.
//!   [`Once::call_once_force()`] can be used to clear the poisoned state.
//!
//! - [`RwLock`]: Provides a mutual exclusion mechanism which allows
//!   multiple readers at the same time, while allowing only one
//!   writer at a time. In some cases, this can be more efficient than
//!   a mutex.
//!
//!   This implementation, like [`Mutex`], will become poisoned on a panic.
//!   Note, however, that an `RwLock` may only be poisoned if a panic occurs
//!   while it is locked exclusively (write mode). If a panic occurs in any reader,
//!   then the lock will not be poisoned.

// FIXME(sync_nonpoison) add links to sync::nonpoison to the doc comment above.

#[stable(feature = "rust1", since = "1.0.0")]
pub use self::condvar::{Condvar, WaitTimeoutResult};
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
pub use self::mutex::MappedMutexGuard;
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::mutex::{Mutex, MutexGuard};
#[stable(feature = "rust1", since = "1.0.0")]
#[expect(deprecated)]
pub use self::once::ONCE_INIT;
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::once::{Once, OnceState};
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
pub use self::rwlock::{MappedRwLockReadGuard, MappedRwLockWriteGuard};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::error::Error;
use crate::fmt;
#[cfg(panic = "unwind")]
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
#[cfg(panic = "unwind")]
use crate::thread;

mod condvar;
#[stable(feature = "rust1", since = "1.0.0")]
mod mutex;
pub(crate) mod once;
mod rwlock;

pub(crate) struct Flag {
    #[cfg(panic = "unwind")]
    failed: Atomic<bool>,
}

// Note that the Ordering uses to access the `failed` field of `Flag` below is
// always `Relaxed`, and that's because this isn't actually protecting any data,
// it's just a flag whether we've panicked or not.
//
// The actual location that this matters is when a mutex is **locked** which is
// where we have external synchronization ensuring that we see memory
// reads/writes to this flag.
//
// As a result, if it matters, we should see the correct value for `failed` in
// all cases.

impl Flag {
    #[inline]
    pub const fn new() -> Flag {
        Flag {
            #[cfg(panic = "unwind")]
            failed: AtomicBool::new(false),
        }
    }

    /// Checks the flag for an unguarded borrow, where we only care about existing poison.
    #[inline]
    pub fn borrow(&self) -> LockResult<()> {
        if self.get() { Err(PoisonError::new(())) } else { Ok(()) }
    }

    /// Checks the flag for a guarded borrow, where we may also set poison when `done`.
    #[inline]
    pub fn guard(&self) -> LockResult<Guard> {
        let ret = Guard {
            #[cfg(panic = "unwind")]
            panicking: thread::panicking(),
        };
        if self.get() { Err(PoisonError::new(ret)) } else { Ok(ret) }
    }

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn done(&self, guard: &Guard) {
        if !guard.panicking && thread::panicking() {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    #[inline]
    #[cfg(not(panic = "unwind"))]
    pub fn done(&self, _guard: &Guard) {}

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn get(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    #[inline(always)]
    #[cfg(not(panic = "unwind"))]
    pub fn get(&self) -> bool {
        false
    }

    #[inline]
    pub fn clear(&self) {
        #[cfg(panic = "unwind")]
        self.failed.store(false, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub(crate) struct Guard {
    #[cfg(panic = "unwind")]
    panicking: bool,
}

/// A type of error which can be returned whenever a lock is acquired.
///
/// Both [`Mutex`]es and [`RwLock`]s are poisoned whenever a thread fails while the lock
/// is held. The precise semantics for when a lock is poisoned is documented on
/// each lock. For a lock in the poisoned state, unless the state is cleared manually,
/// all future acquisitions will return this error.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let mutex = Arc::new(Mutex::new(1));
///
/// // poison the mutex
/// let c_mutex = Arc::clone(&mutex);
/// let _ = thread::spawn(move || {
///     let mut data = c_mutex.lock().unwrap();
///     *data = 2;
///     panic!();
/// }).join();
///
/// match mutex.lock() {
///     Ok(_) => unreachable!(),
///     Err(p_err) => {
///         let data = p_err.get_ref();
///         println!("recovered: {data}");
///     }
/// };
/// ```
/// [`Mutex`]: crate::sync::Mutex
/// [`RwLock`]: crate::sync::RwLock
#[stable(feature = "rust1", since = "1.0.0")]
pub struct PoisonError<T> {
    data: T,
    #[cfg(not(panic = "unwind"))]
    _never: !,
}

/// An enumeration of possible errors associated with a [`TryLockResult`] which
/// can occur while trying to acquire a lock, from the [`try_lock`] method on a
/// [`Mutex`] or the [`try_read`] and [`try_write`] methods on an [`RwLock`].
///
/// [`try_lock`]: crate::sync::Mutex::try_lock
/// [`try_read`]: crate::sync::RwLock::try_read
/// [`try_write`]: crate::sync::RwLock::try_write
/// [`Mutex`]: crate::sync::Mutex
/// [`RwLock`]: crate::sync::RwLock
#[stable(feature = "rust1", since = "1.0.0")]
pub enum TryLockError<T> {
    /// The lock could not be acquired because another thread failed while holding
    /// the lock.
    #[stable(feature = "rust1", since = "1.0.0")]
    Poisoned(#[stable(feature = "rust1", since = "1.0.0")] PoisonError<T>),
    /// The lock could not be acquired at this time because the operation would
    /// otherwise block.
    #[stable(feature = "rust1", since = "1.0.0")]
    WouldBlock,
}

/// A type alias for the result of a lock method which can be poisoned.
///
/// The [`Ok`] variant of this result indicates that the primitive was not
/// poisoned, and the operation result is contained within. The [`Err`] variant indicates
/// that the primitive was poisoned. Note that the [`Err`] variant *also* carries
/// an associated value assigned by the lock method, and it can be acquired through the
/// [`into_inner`] method. The semantics of the associated value depends on the corresponding
/// lock method.
///
/// [`into_inner`]: PoisonError::into_inner
#[stable(feature = "rust1", since = "1.0.0")]
pub type LockResult<T> = Result<T, PoisonError<T>>;

/// A type alias for the result of a nonblocking locking method.
///
/// For more information, see [`LockResult`]. A `TryLockResult` doesn't
/// necessarily hold the associated guard in the [`Err`] type as the lock might not
/// have been acquired for other reasons.
#[stable(feature = "rust1", since = "1.0.0")]
pub type TryLockResult<Guard> = Result<Guard, TryLockError<Guard>>;

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoisonError").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "poisoned lock: another task failed inside".fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Error for PoisonError<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "poisoned lock: another task failed inside"
    }
}

impl<T> PoisonError<T> {
    /// Creates a `PoisonError`.
    ///
    /// This is generally created by methods like [`Mutex::lock`](crate::sync::Mutex::lock)
    /// or [`RwLock::read`](crate::sync::RwLock::read).
    ///
    /// This method may panic if std was built with `panic="abort"`.
    #[cfg(panic = "unwind")]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn new(data: T) -> PoisonError<T> {
        PoisonError { data }
    }

    /// Creates a `PoisonError`.
    ///
    /// This is generally created by methods like [`Mutex::lock`](crate::sync::Mutex::lock)
    /// or [`RwLock::read`](crate::sync::RwLock::read).
    ///
    /// This method may panic if std was built with `panic="abort"`.
    #[cfg(not(panic = "unwind"))]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    #[track_caller]
    pub fn new(_data: T) -> PoisonError<T> {
        panic!("PoisonError created in a libstd built with panic=\"abort\"")
    }

    /// Consumes this error indicating that a lock is poisoned, returning the
    /// associated data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(HashSet::new()));
    ///
    /// // poison the mutex
    /// let c_mutex = Arc::clone(&mutex);
    /// let _ = thread::spawn(move || {
    ///     let mut data = c_mutex.lock().unwrap();
    ///     data.insert(10);
    ///     panic!();
    /// }).join();
    ///
    /// let p_err = mutex.lock().unwrap_err();
    /// let data = p_err.into_inner();
    /// println!("recovered {} items", data.len());
    /// ```
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// reference to the associated data.
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn get_ref(&self) -> &T {
        &self.data
    }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// mutable reference to the associated data.
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> From<PoisonError<T>> for TryLockError<T> {
    fn from(err: PoisonError<T>) -> TryLockError<T> {
        TryLockError::Poisoned(err)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(..) => "Poisoned(..)".fmt(f),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "WouldBlock".fmt(f),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(..) => "poisoned lock: another task failed inside",
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
        .fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Error for TryLockError<T> {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(ref p) => p.description(),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(ref p) => Some(p),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            _ => None,
        }
    }
}

pub(crate) fn map_result<T, U, F>(result: LockResult<T>, f: F) -> LockResult<U>
where
    F: FnOnce(T) -> U,
{
    match result {
        Ok(t) => Ok(f(t)),
        #[cfg(panic = "unwind")]
        Err(PoisonError { data }) => Err(PoisonError::new(f(data))),
    }
}

```

**Entity:** Flag

**States:** Unpoisoned, Poisoned

**Transitions:**
- Unpoisoned -> Poisoned via done() when `!guard.panicking && thread::panicking()`
- Poisoned -> Unpoisoned via clear()

**Evidence:** field `failed: Atomic<bool>` (cfg(panic = "unwind")) encodes poison state at runtime; method `borrow(&self) -> LockResult<()>` checks `self.get()` and returns `Err(PoisonError::new(()))` if poisoned; method `guard(&self) -> LockResult<Guard>` creates a `Guard { panicking: thread::panicking() }` and returns it (possibly wrapped in `PoisonError`) depending on `self.get()`; method `done(&self, guard: &Guard)` sets `failed` only when the thread starts panicking after guard creation: `if !guard.panicking && thread::panicking() { self.failed.store(true, ...) }`; method `clear(&self)` unconditionally resets the poison bit: `self.failed.store(false, ...)`

**Implementation:** Have `Flag::guard()` return an RAII object (e.g., `PoisonGuard<'a> { flag: &'a Flag, panicking_at_acquire: bool }`) that calls `flag.done(self)` in `Drop`. This makes the temporal requirement “must call done() exactly once for the guard you got from guard()” automatic and unskippable. Optionally, split guard return types into `UnpoisonedGuard` vs `PoisonedGuard` (typestate) to make the poisoned/unpoisoned branch explicit at the type level for internal callers.

---

### 35. Non-blocking receiver iteration protocol (BorrowedReceiver -> Empty | Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-42`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: TryIter performs non-blocking receives by calling try_recv() and mapping any error to None via ok(). This collapses at least two distinct runtime states—'no message available yet' (would typically be TryRecvError::Empty) and 'channel disconnected/closed' (TryRecvError::Disconnected)—into the same Option::None result. Users must implicitly know/remember that None does not necessarily mean the channel is finished. The type system does not enforce correct handling of Empty vs Disconnected; the iterator API erases that distinction.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[stable(feature = "receiver_try_iter", since = "1.15.0")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** TryIter<'a, T>

**States:** BorrowedReceiver, Empty, DisconnectedOrExhausted

**Transitions:**
- BorrowedReceiver -> Empty via TryIter::next() calling Receiver::try_recv() returning Err(Empty) (mapped to None)
- BorrowedReceiver -> DisconnectedOrExhausted via TryIter::next() calling Receiver::try_recv() returning Err(Disconnected) (mapped to None)

**Evidence:** impl Iterator for TryIter: next(): self.rx.try_recv().ok() — non-blocking receive; method name Receiver::try_recv() (invoked here) implies multiple error reasons (e.g., empty vs disconnected) that are erased by ok()

**Implementation:** Provide an iterator whose Item is a result-like discriminated union that preserves the state (e.g., enum TryNext<T> { Item(T), Empty, Disconnected }), or a new iterator trait/adaptor that yields Result<T, TryRecvError> so Empty/Disconnected are handled explicitly rather than erased into None.

---

### 77. ReentrantLock ownership/counter state machine (Unlocked / Locked-by-owner with recursion depth)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/reentrant_lock.rs:1-215`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ReentrantLock encodes a re-entrant locking protocol using runtime state: `owner: Tid` and `lock_count: UnsafeCell<u32>` must remain consistent with the underlying `mutex`. The correctness invariant is that only the owning thread may increment `lock_count` without taking `mutex`, and that `owner` is set/unset only while holding `mutex`. Additionally, `lock_count` must never underflow/overflow, and when it reaches 0 the lock must transition back to Unlocked by clearing `owner` and unlocking `mutex`. These relationships are maintained by `unsafe` code, debug assertions, and runtime checks, not by the type system.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ReentrantLockGuard

// we don't need to further synchronize the TID accesses, so they can be regular 64-bit
// non-atomic accesses.
#[unstable(feature = "reentrant_lock", issue = "121440")]
pub struct ReentrantLock<T: ?Sized> {
    mutex: sys::Mutex,
    owner: Tid,
    lock_count: UnsafeCell<u32>,
    data: T,
}

// ... (other code) ...

);

#[unstable(feature = "reentrant_lock", issue = "121440")]
unsafe impl<T: Send + ?Sized> Send for ReentrantLock<T> {}
#[unstable(feature = "reentrant_lock", issue = "121440")]
unsafe impl<T: Send + ?Sized> Sync for ReentrantLock<T> {}

// Because of the `UnsafeCell`, these traits are not implemented automatically
#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: UnwindSafe + ?Sized> UnwindSafe for ReentrantLock<T> {}
#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: RefUnwindSafe + ?Sized> RefUnwindSafe for ReentrantLock<T> {}


// ... (other code) ...

}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: ?Sized> !Send for ReentrantLockGuard<'_, T> {}

#[unstable(feature = "reentrant_lock", issue = "121440")]
unsafe impl<T: ?Sized + Sync> Sync for ReentrantLockGuard<'_, T> {}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T> ReentrantLock<T> {
    /// Creates a new re-entrant lock in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(reentrant_lock)]
    /// use std::sync::ReentrantLock;
    ///
    /// let lock = ReentrantLock::new(0);
    /// ```
    pub const fn new(t: T) -> ReentrantLock<T> {
        ReentrantLock {
            mutex: sys::Mutex::new(),
            owner: Tid::new(),
            lock_count: UnsafeCell::new(0),
            data: t,
        }
    }

    /// Consumes this lock, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(reentrant_lock)]
    ///
    /// use std::sync::ReentrantLock;
    ///
    /// let lock = ReentrantLock::new(0);
    /// assert_eq!(lock.into_inner(), 0);
    /// ```
    pub fn into_inner(self) -> T {
        self.data
    }
}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: ?Sized> ReentrantLock<T> {
    /// Acquires the lock, blocking the current thread until it is able to do
    /// so.
    ///
    /// This function will block the caller until it is available to acquire
    /// the lock. Upon returning, the thread is the only thread with the lock
    /// held. When the thread calling this method already holds the lock, the
    /// call succeeds without blocking.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(reentrant_lock)]
    /// use std::cell::Cell;
    /// use std::sync::{Arc, ReentrantLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(ReentrantLock::new(Cell::new(0)));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// thread::spawn(move || {
    ///     c_lock.lock().set(10);
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(lock.lock().get(), 10);
    /// ```
    pub fn lock(&self) -> ReentrantLockGuard<'_, T> {
        let this_thread = current_id();
        // Safety: We only touch lock_count when we own the inner mutex.
        // Additionally, we only call `self.owner.set()` while holding
        // the inner mutex, so no two threads can call it concurrently.
        unsafe {
            if self.owner.contains(this_thread) {
                self.increment_lock_count().expect("lock count overflow in reentrant mutex");
            } else {
                self.mutex.lock();
                self.owner.set(Some(this_thread));
                debug_assert_eq!(*self.lock_count.get(), 0);
                *self.lock_count.get() = 1;
            }
        }
        ReentrantLockGuard { lock: self }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `ReentrantLock` mutably, no actual locking
    /// needs to take place -- the mutable borrow statically guarantees no locks
    /// exist.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(reentrant_lock)]
    /// use std::sync::ReentrantLock;
    ///
    /// let mut lock = ReentrantLock::new(0);
    /// *lock.get_mut() = 10;
    /// assert_eq!(*lock.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned.
    ///
    /// This function does not block.
    // FIXME maybe make it a public part of the API?
    #[unstable(issue = "none", feature = "std_internals")]
    #[doc(hidden)]
    pub fn try_lock(&self) -> Option<ReentrantLockGuard<'_, T>> {
        let this_thread = current_id();
        // Safety: We only touch lock_count when we own the inner mutex.
        // Additionally, we only call `self.owner.set()` while holding
        // the inner mutex, so no two threads can call it concurrently.
        unsafe {
            if self.owner.contains(this_thread) {
                self.increment_lock_count()?;
                Some(ReentrantLockGuard { lock: self })
            } else if self.mutex.try_lock() {
                self.owner.set(Some(this_thread));
                debug_assert_eq!(*self.lock_count.get(), 0);
                *self.lock_count.get() = 1;
                Some(ReentrantLockGuard { lock: self })
            } else {
                None
            }
        }
    }

    unsafe fn increment_lock_count(&self) -> Option<()> {
        unsafe {
            *self.lock_count.get() = (*self.lock_count.get()).checked_add(1)?;
        }
        Some(())
    }
}

// ... (other code) ...

}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T> From<T> for ReentrantLock<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: ?Sized> Deref for ReentrantLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.lock.data
    }
}

// ... (other code) ...

}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: ?Sized> Drop for ReentrantLockGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Safety: We own the lock.
        unsafe {
            *self.lock.lock_count.get() -= 1;
            if *self.lock.lock_count.get() == 0 {
                self.lock.owner.set(None);
                self.lock.mutex.unlock();
            }
        }
    }
}

```

**Entity:** ReentrantLock<T>

**States:** Unlocked (owner=None, lock_count=0, mutex unlocked), LockedByOwner(depth>=1) (owner=Some(tid), lock_count=depth, mutex locked)

**Transitions:**
- Unlocked -> LockedByOwner(1) via lock() (slow path) [mutex.lock(); owner=Some(tid); lock_count=1]
- LockedByOwner(n) -> LockedByOwner(n+1) via lock()/try_lock() (re-entrant fast path) [increment_lock_count()]
- Unlocked -> LockedByOwner(1) via try_lock() (success path) [mutex.try_lock(); owner=Some(tid); lock_count=1]
- LockedByOwner(n) -> LockedByOwner(n-1) via ReentrantLockGuard::drop() [lock_count -= 1]
- LockedByOwner(1) -> Unlocked via ReentrantLockGuard::drop() [owner=None; mutex.unlock()]
- LockedByOwner(_) -> (no transition) via try_lock() returning None when mutex is held by another thread

**Evidence:** struct fields: `owner: Tid`, `lock_count: UnsafeCell<u32>`, `mutex: sys::Mutex` encode the state machine at runtime; lock(): `if self.owner.contains(this_thread) { self.increment_lock_count().expect("lock count overflow in reentrant mutex"); } else { self.mutex.lock(); self.owner.set(Some(this_thread)); debug_assert_eq!(*self.lock_count.get(), 0); *self.lock_count.get() = 1; }`; try_lock(): same split between re-entrant path and `mutex.try_lock()` path; returns `None` if it can't acquire; increment_lock_count(): uses `checked_add(1)?` showing an overflow precondition/state constraint; Drop for ReentrantLockGuard: `*self.lock.lock_count.get() -= 1; if *self.lock.lock_count.get() == 0 { self.lock.owner.set(None); self.lock.mutex.unlock(); }` encodes the release transition and requires no underflow; comments in lock()/try_lock(): `// Safety: We only touch lock_count when we own the inner mutex.` and `// ... only call self.owner.set() while holding the inner mutex` describe unenforced protocol requirements

**Implementation:** Model the internal state as a typestate (e.g., `ReentrantLock<Unlocked, T>` and `ReentrantLock<Locked, T>`), with the locked state carrying a non-Send guard/token that proves ownership. Re-entrancy depth could be tracked by a `NonZeroU32` newtype or by storing the count in the guard chain. While full thread-id tracking can’t be expressed purely in stable Rust types, you can still make illegal operations unrepresentable by moving the `lock_count` mutation behind an internal `OwnedLockToken` capability that is only constructible after `mutex.lock()` and is required to call `increment_lock_count()`/`owner.set()`.

---

### 57. Zero-capacity channel connection state (Connected / Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Inner encodes a runtime connection state via is_disconnected. Operations in the rest of the module (send/recv registration/wake-ups) must behave differently depending on whether the channel is still connected. The type system does not distinguish a connected Inner from a disconnected one, so correct behavior relies on runtime branching on is_disconnected and careful method ordering (e.g., once disconnected, only wake/cancel paths are valid and no new pairing should be attempted).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Packet, impl Packet < T > (3 methods); struct Channel, impl Channel < T > (12 methods)

}

/// Inner representation of a zero-capacity channel.
struct Inner {
    /// Senders waiting to pair up with a receive operation.
    senders: Waker,

    /// Receivers waiting to pair up with a send operation.
    receivers: Waker,

    /// Equals `true` when the channel is disconnected.
    is_disconnected: bool,
}

```

**Entity:** Inner

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via a disconnect/close/drop path in Channel/Packet (not shown in snippet)

**Evidence:** field: is_disconnected: bool — "Equals `true` when the channel is disconnected." (runtime state flag); field: senders: Waker — "Senders waiting to pair up with a receive operation." (behavior depends on whether pairing is still possible); field: receivers: Waker — "Receivers waiting to pair up with a send operation." (behavior depends on whether pairing is still possible)

**Implementation:** Represent connection as a type parameter: Inner<State> where State is Connected or Disconnected. Methods that register/pair senders/receivers are only implemented for Inner<Connected>. A disconnect(self) -> Inner<Disconnected> transition (or storing an enum Inner { Connected(InnerConnected), Disconnected(InnerDisconnected) }) makes post-disconnect operations explicit and prevents calling pairing logic after disconnection.

---

## Precondition Invariants

### 6. Self-deadlock / re-entrancy precondition (NonReentrant use) enforced only by runtime/panic

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-506`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `read()` and `write()` have an implicit precondition that the current thread does not already hold the lock in a conflicting way; otherwise they 'might panic'. This is a runtime/platform-dependent behavior (and/or deadlock avoidance in the underlying `sys::RwLock`) that is not modeled at the type level: the API allows calling `read()`/`write()` while already holding `RwLockReadGuard`/`RwLockWriteGuard` for the same lock, and the only enforcement is a potential panic at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLockReadGuard; struct RwLockWriteGuard; struct MappedRwLockReadGuard; struct MappedRwLockWriteGuard

/// [`Mutex`]: super::Mutex
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLock")]
pub struct RwLock<T: ?Sized> {
    inner: sys::RwLock,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockReadGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockReadGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockWriteGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockWriteGuard<'_, T> {}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(5);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> RwLock<T> {
        RwLock { inner: sys::RwLock::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.read() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// lock.set(11).unwrap();
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.write() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.replace(11).unwrap(), 7);
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.write() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. The failure will occur immediately after the lock has been
    /// acquired. The acquired lock guard will be contained in the returned
    /// error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read();
    ///     assert!(r.is_ok());
    /// }).join().unwrap();
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            self.inner.read();
            RwLockReadGuard::new(self)
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access.
    ///
    /// If the access could not be granted at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// match lock.try_read() {
    ///     Ok(n) => assert_eq!(*n, 1),
    ///     Err(_) => unreachable!(),
    /// };
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            if self.inner.try_read() {
                Ok(RwLockReadGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. An error will be returned when the lock is acquired. The acquired
    /// lock guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write().unwrap();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            self.inner.write();
            RwLockWriteGuard::new(self)
        }
    }

    /// Attempts to lock this `RwLock` with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// assert!(lock.try_write().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            if self.inner.try_write() {
                Ok(RwLockWriteGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the lock is poisoned.
    ///
    /// If another thread is active, the lock can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    /// assert_eq!(lock.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a lock.
    ///
    /// If the lock is poisoned, it will remain poisoned until this function is called. This allows
    /// recovering from a poisoned state and marking that it has recovered. For example, if the
    /// value is overwritten by a known-good value, then the lock can be marked as un-poisoned. Or
    /// possibly, the value could be inspected to determine if it is in a consistent state, and if
    /// so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    ///
    /// assert_eq!(lock.is_poisoned(), true);
    /// let guard = lock.write().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     lock.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(lock.is_poisoned(), false);
    /// assert_eq!(*guard, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the underlying data if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock. An error will only be returned
    /// if the lock would have otherwise been acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write().unwrap();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner().unwrap(), "modified");
    /// ```
    #[stable(feature = "rwlock_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previously abandoned locks
    /// (e.g., via [`forget()`] on a [`RwLockReadGuard`] or [`RwLockWriteGuard`]).
    ///
    /// # Errors
    ///
    /// This function will return an error containing a mutable reference to
    /// the underlying data if the `RwLock` is poisoned. An `RwLock` is
    /// poisoned whenever a writer panics while holding an exclusive lock.
    /// An error will only be returned if the lock would have otherwise been
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(0);
    /// *lock.get_mut().unwrap() = 10;
    /// assert_eq!(*lock.read().unwrap(), 10);
    /// ```
    #[stable(feature = "rwlock_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

// ... (other code) ...

}

#[stable(feature = "rw_lock_from", since = "1.24.0")]
impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}


// ... (truncated) ...
```

**Entity:** RwLock<T>

**States:** NotHeldByCurrentThread, HeldByCurrentThread

**Transitions:**
- NotHeldByCurrentThread -> HeldByCurrentThread via read()/write() returning a guard
- HeldByCurrentThread -> NotHeldByCurrentThread via dropping the corresponding guard

**Evidence:** doc on `read()`: '# Panics: This function might panic when called if the lock is already held by the current thread.'; doc on `write()`: '# Panics: This function might panic when called if the lock is already held by the current thread.'; methods: `read`/`write` unconditionally call `self.inner.read()` / `self.inner.write()` (no compile-time gating against re-entrancy)

**Implementation:** Expose an explicit non-reentrant session token tied to a borrow of the lock, e.g. `fn session(&self) -> RwLockSession<'_>`; only `RwLockSession` can call `read`/`write`, and creating a nested session would be prevented by borrowing rules (similar to `RefCell::borrow` style patterns). This can’t prevent all aliasing (e.g., via `Arc` clones), but can enforce non-reentrancy within a single borrow chain and make the precondition more explicit in types.

---

### 42. ArrayToken slot-pointer validity & provenance (bound to Channel/Slot lifetime)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: ArrayToken contains a raw pointer `slot: *const u8` that is expected to refer to a particular slot in the backing array used by the MPMC channel implementation. This implies a hidden validity/provenance invariant: the pointer must be non-null, properly aligned/castable to the actual slot type, and must not outlive the allocation of the channel’s slot array. None of this is enforced by the type system because the pointer is untyped (`*const u8`) and carries no lifetime tying it to the channel/slots it came from.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot; struct Channel, impl Channel < T > (17 methods)


/// The token type for the array flavor.
#[derive(Debug)]
pub(crate) struct ArrayToken {
    /// Slot to read from or write to.
    slot: *const u8,

    /// Stamp to store into the slot after reading or writing.
    stamp: usize,
}

```

**Entity:** ArrayToken

**States:** Invalid/Null/Dangling, Valid (points to a live Slot within a specific Channel)

**Transitions:**
- Invalid -> Valid when token is produced by Channel/Slot logic that sets `slot` to an in-bounds slot address
- Valid -> Invalid when the underlying Channel/slot storage is dropped while the token is still accessible

**Evidence:** field `slot: *const u8` (raw, untyped pointer with no lifetime/provenance guarantees); doc comment on `slot`: "Slot to read from or write to." (implies it must point at a real slot object)

**Implementation:** Make the token carry typed provenance and a lifetime, e.g. `struct ArrayToken<'a, T> { slot: NonNull<Slot<T>>, stamp: Stamp, _ch: PhantomData<&'a Channel<T>> }`, or store a slot index `usize` plus `PhantomData<&'a Channel<T>>` to ensure the token cannot outlive the channel and cannot refer to foreign memory.

---

### 18. PoisonError construction precondition (only valid under panic=unwind)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison.rs:1-411`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Creating a PoisonError is only meaningful/allowed when the standard library is built with unwinding panics. Under `panic="abort"`, `PoisonError::new` always panics at runtime. This is an implicit build-configuration precondition that is not represented in the type system: the same API exists but becomes a runtime trap in abort mode.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Flag, impl Flag (8 methods); struct Guard; struct PoisonError, impl Error for PoisonError < T > (1 methods), impl PoisonError < T > (5 methods); enum TryLockError, impl From < PoisonError < T > > for TryLockError < T > (1 methods), impl Error for TryLockError < T > (2 methods)

//! Synchronization objects that employ poisoning.
//!
//! # Poisoning
//!
//! All synchronization objects in this module implement a strategy called "poisoning"
//! where if a thread panics while holding the exclusive access granted by the primitive,
//! the state of the primitive is set to "poisoned".
//! This information is then propagated to all other threads
//! to signify that the data protected by this primitive is likely tainted
//! (some invariant is not being upheld).
//!
//! The specifics of how this "poisoned" state affects other threads
//! depend on the primitive. See [#Overview] bellow.
//!
//! For the alternative implementations that do not employ poisoning,
//! see `std::sys::nonpoisoning`.
//!
//! # Overview
//!
//! Below is a list of synchronization objects provided by this module
//! with a high-level overview for each object and a description
//! of how it employs "poisoning".
//!
//! - [`Condvar`]: Condition Variable, providing the ability to block
//!   a thread while waiting for an event to occur.
//!
//!   Condition variables are typically associated with
//!   a boolean predicate (a condition) and a mutex.
//!   This implementation is associated with [`poison::Mutex`](Mutex),
//!   which employs poisoning.
//!   For this reason, [`Condvar::wait()`] will return a [`LockResult`],
//!   just like [`poison::Mutex::lock()`](Mutex::lock) does.
//!
//! - [`Mutex`]: Mutual Exclusion mechanism, which ensures that at
//!   most one thread at a time is able to access some data.
//!
//!   [`Mutex::lock()`] returns a [`LockResult`],
//!   providing a way to deal with the poisoned state.
//!   See [`Mutex`'s documentation](Mutex#poisoning) for more.
//!
//! - [`Once`]: A thread-safe way to run a piece of code only once.
//!   Mostly useful for implementing one-time global initialization.
//!
//!   [`Once`] is poisoned if the piece of code passed to
//!   [`Once::call_once()`] or [`Once::call_once_force()`] panics.
//!   When in poisoned state, subsequent calls to [`Once::call_once()`] will panic too.
//!   [`Once::call_once_force()`] can be used to clear the poisoned state.
//!
//! - [`RwLock`]: Provides a mutual exclusion mechanism which allows
//!   multiple readers at the same time, while allowing only one
//!   writer at a time. In some cases, this can be more efficient than
//!   a mutex.
//!
//!   This implementation, like [`Mutex`], will become poisoned on a panic.
//!   Note, however, that an `RwLock` may only be poisoned if a panic occurs
//!   while it is locked exclusively (write mode). If a panic occurs in any reader,
//!   then the lock will not be poisoned.

// FIXME(sync_nonpoison) add links to sync::nonpoison to the doc comment above.

#[stable(feature = "rust1", since = "1.0.0")]
pub use self::condvar::{Condvar, WaitTimeoutResult};
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
pub use self::mutex::MappedMutexGuard;
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::mutex::{Mutex, MutexGuard};
#[stable(feature = "rust1", since = "1.0.0")]
#[expect(deprecated)]
pub use self::once::ONCE_INIT;
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::once::{Once, OnceState};
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
pub use self::rwlock::{MappedRwLockReadGuard, MappedRwLockWriteGuard};
#[stable(feature = "rust1", since = "1.0.0")]
pub use self::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::error::Error;
use crate::fmt;
#[cfg(panic = "unwind")]
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
#[cfg(panic = "unwind")]
use crate::thread;

mod condvar;
#[stable(feature = "rust1", since = "1.0.0")]
mod mutex;
pub(crate) mod once;
mod rwlock;

pub(crate) struct Flag {
    #[cfg(panic = "unwind")]
    failed: Atomic<bool>,
}

// Note that the Ordering uses to access the `failed` field of `Flag` below is
// always `Relaxed`, and that's because this isn't actually protecting any data,
// it's just a flag whether we've panicked or not.
//
// The actual location that this matters is when a mutex is **locked** which is
// where we have external synchronization ensuring that we see memory
// reads/writes to this flag.
//
// As a result, if it matters, we should see the correct value for `failed` in
// all cases.

impl Flag {
    #[inline]
    pub const fn new() -> Flag {
        Flag {
            #[cfg(panic = "unwind")]
            failed: AtomicBool::new(false),
        }
    }

    /// Checks the flag for an unguarded borrow, where we only care about existing poison.
    #[inline]
    pub fn borrow(&self) -> LockResult<()> {
        if self.get() { Err(PoisonError::new(())) } else { Ok(()) }
    }

    /// Checks the flag for a guarded borrow, where we may also set poison when `done`.
    #[inline]
    pub fn guard(&self) -> LockResult<Guard> {
        let ret = Guard {
            #[cfg(panic = "unwind")]
            panicking: thread::panicking(),
        };
        if self.get() { Err(PoisonError::new(ret)) } else { Ok(ret) }
    }

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn done(&self, guard: &Guard) {
        if !guard.panicking && thread::panicking() {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    #[inline]
    #[cfg(not(panic = "unwind"))]
    pub fn done(&self, _guard: &Guard) {}

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn get(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    #[inline(always)]
    #[cfg(not(panic = "unwind"))]
    pub fn get(&self) -> bool {
        false
    }

    #[inline]
    pub fn clear(&self) {
        #[cfg(panic = "unwind")]
        self.failed.store(false, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub(crate) struct Guard {
    #[cfg(panic = "unwind")]
    panicking: bool,
}

/// A type of error which can be returned whenever a lock is acquired.
///
/// Both [`Mutex`]es and [`RwLock`]s are poisoned whenever a thread fails while the lock
/// is held. The precise semantics for when a lock is poisoned is documented on
/// each lock. For a lock in the poisoned state, unless the state is cleared manually,
/// all future acquisitions will return this error.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let mutex = Arc::new(Mutex::new(1));
///
/// // poison the mutex
/// let c_mutex = Arc::clone(&mutex);
/// let _ = thread::spawn(move || {
///     let mut data = c_mutex.lock().unwrap();
///     *data = 2;
///     panic!();
/// }).join();
///
/// match mutex.lock() {
///     Ok(_) => unreachable!(),
///     Err(p_err) => {
///         let data = p_err.get_ref();
///         println!("recovered: {data}");
///     }
/// };
/// ```
/// [`Mutex`]: crate::sync::Mutex
/// [`RwLock`]: crate::sync::RwLock
#[stable(feature = "rust1", since = "1.0.0")]
pub struct PoisonError<T> {
    data: T,
    #[cfg(not(panic = "unwind"))]
    _never: !,
}

/// An enumeration of possible errors associated with a [`TryLockResult`] which
/// can occur while trying to acquire a lock, from the [`try_lock`] method on a
/// [`Mutex`] or the [`try_read`] and [`try_write`] methods on an [`RwLock`].
///
/// [`try_lock`]: crate::sync::Mutex::try_lock
/// [`try_read`]: crate::sync::RwLock::try_read
/// [`try_write`]: crate::sync::RwLock::try_write
/// [`Mutex`]: crate::sync::Mutex
/// [`RwLock`]: crate::sync::RwLock
#[stable(feature = "rust1", since = "1.0.0")]
pub enum TryLockError<T> {
    /// The lock could not be acquired because another thread failed while holding
    /// the lock.
    #[stable(feature = "rust1", since = "1.0.0")]
    Poisoned(#[stable(feature = "rust1", since = "1.0.0")] PoisonError<T>),
    /// The lock could not be acquired at this time because the operation would
    /// otherwise block.
    #[stable(feature = "rust1", since = "1.0.0")]
    WouldBlock,
}

/// A type alias for the result of a lock method which can be poisoned.
///
/// The [`Ok`] variant of this result indicates that the primitive was not
/// poisoned, and the operation result is contained within. The [`Err`] variant indicates
/// that the primitive was poisoned. Note that the [`Err`] variant *also* carries
/// an associated value assigned by the lock method, and it can be acquired through the
/// [`into_inner`] method. The semantics of the associated value depends on the corresponding
/// lock method.
///
/// [`into_inner`]: PoisonError::into_inner
#[stable(feature = "rust1", since = "1.0.0")]
pub type LockResult<T> = Result<T, PoisonError<T>>;

/// A type alias for the result of a nonblocking locking method.
///
/// For more information, see [`LockResult`]. A `TryLockResult` doesn't
/// necessarily hold the associated guard in the [`Err`] type as the lock might not
/// have been acquired for other reasons.
#[stable(feature = "rust1", since = "1.0.0")]
pub type TryLockResult<Guard> = Result<Guard, TryLockError<Guard>>;

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoisonError").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "poisoned lock: another task failed inside".fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Error for PoisonError<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "poisoned lock: another task failed inside"
    }
}

impl<T> PoisonError<T> {
    /// Creates a `PoisonError`.
    ///
    /// This is generally created by methods like [`Mutex::lock`](crate::sync::Mutex::lock)
    /// or [`RwLock::read`](crate::sync::RwLock::read).
    ///
    /// This method may panic if std was built with `panic="abort"`.
    #[cfg(panic = "unwind")]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn new(data: T) -> PoisonError<T> {
        PoisonError { data }
    }

    /// Creates a `PoisonError`.
    ///
    /// This is generally created by methods like [`Mutex::lock`](crate::sync::Mutex::lock)
    /// or [`RwLock::read`](crate::sync::RwLock::read).
    ///
    /// This method may panic if std was built with `panic="abort"`.
    #[cfg(not(panic = "unwind"))]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    #[track_caller]
    pub fn new(_data: T) -> PoisonError<T> {
        panic!("PoisonError created in a libstd built with panic=\"abort\"")
    }

    /// Consumes this error indicating that a lock is poisoned, returning the
    /// associated data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(HashSet::new()));
    ///
    /// // poison the mutex
    /// let c_mutex = Arc::clone(&mutex);
    /// let _ = thread::spawn(move || {
    ///     let mut data = c_mutex.lock().unwrap();
    ///     data.insert(10);
    ///     panic!();
    /// }).join();
    ///
    /// let p_err = mutex.lock().unwrap_err();
    /// let data = p_err.into_inner();
    /// println!("recovered {} items", data.len());
    /// ```
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// reference to the associated data.
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn get_ref(&self) -> &T {
        &self.data
    }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// mutable reference to the associated data.
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> From<PoisonError<T>> for TryLockError<T> {
    fn from(err: PoisonError<T>) -> TryLockError<T> {
        TryLockError::Poisoned(err)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(..) => "Poisoned(..)".fmt(f),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "WouldBlock".fmt(f),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(..) => "poisoned lock: another task failed inside",
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
        .fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Error for TryLockError<T> {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(ref p) => p.description(),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            #[cfg(panic = "unwind")]
            TryLockError::Poisoned(ref p) => Some(p),
            #[cfg(not(panic = "unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            _ => None,
        }
    }
}

pub(crate) fn map_result<T, U, F>(result: LockResult<T>, f: F) -> LockResult<U>
where
    F: FnOnce(T) -> U,
{
    match result {
        Ok(t) => Ok(f(t)),
        #[cfg(panic = "unwind")]
        Err(PoisonError { data }) => Err(PoisonError::new(f(data))),
    }
}

```

**Entity:** PoisonError<T>

**States:** Constructible (panic=unwind), Unconstructible (panic=abort)

**Transitions:**
- Constructible -> Unconstructible via compilation configuration `cfg(not(panic = "unwind"))` (build-time switch)

**Evidence:** method `PoisonError::new(data: T)` exists in two cfg variants; `#[cfg(not(panic = "unwind"))] pub fn new(_data: T) -> PoisonError<T> { panic!("PoisonError created in a libstd built with panic=\"abort\"") }` documents and enforces the precondition via runtime panic; doc comment: "This method may panic if std was built with `panic=\"abort\"`."

**Implementation:** Make PoisonError construction require an internal-only capability available only in unwind builds (e.g., `struct UnwindPoisoningEnabled(())` behind `cfg(panic="unwind")`; `PoisonError::new(_: UnwindPoisoningEnabled, data: T)`). Alternatively, expose `PoisonError::new` only under `cfg(panic="unwind")` (API split) so abort builds can’t call it at all, eliminating the runtime panic path.

---

### 75. Discard-all-messages requires exclusive 'no receivers' phase

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-425`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `discard_all_messages` has a strong precondition that it may only be called once all receivers are dropped, because it eagerly frees/discards queued messages and interacts with concurrent initialization and tail updates. This requirement is only documented in a comment and indirectly relied upon by being called from `disconnect_receivers`; the type system does not express the needed exclusivity (e.g., a capability proving there are no receivers) so misuse elsewhere would be unsound.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Block, impl Block < T > (3 methods); struct Position; struct ListToken

///
/// Consecutive messages are grouped into blocks in order to put less pressure on the allocator and
/// improve cache efficiency.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    head: CachePadded<Position<T>>,

    /// The tail of the channel.
    tail: CachePadded<Position<T>>,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,

    /// Indicates that dropping a `Channel<T>` may drop messages of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Creates a new unbounded channel.
    pub(crate) fn new() -> Self {
        Channel {
            head: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            tail: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            receivers: SyncWaker::new(),
            _marker: PhantomData,
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        let mut block = self.tail.block.load(Ordering::Acquire);
        let mut next_block = None;

        loop {
            // Check if the channel is disconnected.
            if tail & MARK_BIT != 0 {
                token.list.block = ptr::null();
                return true;
            }

            // Calculate the offset of the index into the block.
            let offset = (tail >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                tail = self.tail.index.load(Ordering::Acquire);
                block = self.tail.block.load(Ordering::Acquire);
                continue;
            }

            // If we're going to have to install the next block, allocate it in advance in order to
            // make the wait for other threads as short as possible.
            if offset + 1 == BLOCK_CAP && next_block.is_none() {
                next_block = Some(Block::<T>::new());
            }

            // If this is the first message to be sent into the channel, we need to allocate the
            // first block and install it.
            if block.is_null() {
                let new = Box::into_raw(Block::<T>::new());

                if self
                    .tail
                    .block
                    .compare_exchange(block, new, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    // This yield point leaves the channel in a half-initialized state where the
                    // tail.block pointer is set but the head.block is not. This is used to
                    // facilitate the test in src/tools/miri/tests/pass/issues/issue-139553.rs
                    #[cfg(miri)]
                    crate::thread::yield_now();
                    self.head.block.store(new, Ordering::Release);
                    block = new;
                } else {
                    next_block = unsafe { Some(Box::from_raw(new)) };
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                    continue;
                }
            }

            let new_tail = tail + (1 << SHIFT);

            // Try advancing the tail forward.
            match self.tail.index.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, install the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next_block = Box::into_raw(next_block.unwrap());
                        self.tail.block.store(next_block, Ordering::Release);
                        self.tail.index.fetch_add(1 << SHIFT, Ordering::Release);
                        (*block).next.store(next_block, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.list.block.is_null() {
            return Err(msg);
        }

        // Write the message into the slot.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.state.fetch_or(WRITE, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.index.load(Ordering::Acquire);
        let mut block = self.head.block.load(Ordering::Acquire);

        loop {
            // Calculate the offset of the index into the block.
            let offset = (head >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            let mut new_head = head + (1 << SHIFT);

            if new_head & MARK_BIT == 0 {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.index.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if head >> SHIFT == tail >> SHIFT {
                    // If the channel is disconnected...
                    if tail & MARK_BIT != 0 {
                        // ...then receive an error.
                        token.list.block = ptr::null();
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                // If head and tail are not in the same block, set `MARK_BIT` in head.
                if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                    new_head |= MARK_BIT;
                }
            }

            // The block can be null here only if the first message is being sent into the channel.
            // In that case, just wait until it gets initialized.
            if block.is_null() {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            // Try moving the head index forward.
            match self.head.index.compare_exchange_weak(
                head,
                new_head,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, move to the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next = (*block).wait_next();
                        let mut next_index = (new_head & !MARK_BIT).wrapping_add(1 << SHIFT);
                        if !(*next).next.load(Ordering::Relaxed).is_null() {
                            next_index |= MARK_BIT;
                        }

                        self.head.block.store(next, Ordering::Release);
                        self.head.index.store(next_index, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    head = self.head.index.load(Ordering::Acquire);
                    block = self.head.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.list.block.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.wait_write();
            let msg = slot.msg.get().read().assume_init();

            // Destroy the block if we've reached the end, or if another thread wanted to destroy but
            // couldn't because we were busy reading from the slot.
            if offset + 1 == BLOCK_CAP {
                Block::destroy(block, 0);
            } else if slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY != 0 {
                Block::destroy(block, offset + 1);
            }

            Ok(msg)
        }
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.send(msg, None).map_err(|err| match err {
            SendTimeoutError::Disconnected(msg) => TrySendError::Disconnected(msg),
            SendTimeoutError::Timeout(_) => unreachable!(),
        })
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        _deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        assert!(self.start_send(token));
        unsafe { self.write(token, msg).map_err(SendTimeoutError::Disconnected) }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            if self.start_recv(token) {
                unsafe {
                    return self.read(token).map_err(|_| RecvTimeoutError::Disconnected);
                }
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            // Prepare for blocking until a sender wakes us up.
            Context::with(|cx| {
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail index, then load the head index.
            let mut tail = self.tail.index.load(Ordering::SeqCst);
            let mut head = self.head.index.load(Ordering::SeqCst);

            // If the tail index didn't change, we've got consistent indices to work with.
            if self.tail.index.load(Ordering::SeqCst) == tail {
                // Erase the lower bits.
                tail &= !((1 << SHIFT) - 1);
                head &= !((1 << SHIFT) - 1);

                // Fix up indices if they fall onto block ends.
                if (tail >> SHIFT) & (LAP - 1) == LAP - 1 {
                    tail = tail.wrapping_add(1 << SHIFT);
                }
                if (head >> SHIFT) & (LAP - 1) == LAP - 1 {
                    head = head.wrapping_add(1 << SHIFT);
                }

                // Rotate indices so that head falls into the first block.
                let lap = (head >> SHIFT) / LAP;
                tail = tail.wrapping_sub((lap * LAP) << SHIFT);
                head = head.wrapping_sub((lap * LAP) << SHIFT);

                // Remove the lower bits.
                tail >>= SHIFT;
                head >>= SHIFT;

                // Return the difference minus the number of blocks between tail and head.
                return tail - head - tail / LAP;
            }
        }
    }

    /// Returns the capacity of the channel.
    pub(crate) fn capacity(&self) -> Option<usize> {
        None
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            // If receivers are dropped first, discard all messages to free
            // memory eagerly.
            self.discard_all_messages();
            true
        } else {
            false
        }
    }

    /// Discards all messages.
    ///
    /// This method should only be called when all receivers are dropped.
    fn discard_all_messages(&self) {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        loop {
            let offset = (tail >> SHIFT) % LAP;
            if offset != BLOCK_CAP {
                break;
            }

            // New updates to tail will be rejected by MARK_BIT and aborted unless it's
            // at boundary. We need to wait for the updates take affect otherwise there
            // can be memory leaks.
            backoff.spin_heavy();
            tail = self.tail.index.load(Ordering::Acquire);
        }

        let mut head = self.head.index.load(Ordering::Acquire);
        // The channel may be uninitialized, so we have to swap to avoid overwriting any sender's attempts
        // to initialize the first block before noticing that the receivers disconnected. Late allocations
        // will be deall
// ... (truncated) ...
```

**Entity:** Channel<T>

**States:** ReceiversAlive, NoReceivers (exclusive discard permitted)

**Transitions:**
- ReceiversAlive -> NoReceivers via external drop of all receiver handles (not represented in this snippet)
- NoReceivers -> (messages discarded) via discard_all_messages() (called from disconnect_receivers())

**Evidence:** discard_all_messages() comment: `This method should only be called when all receivers are dropped.`; disconnect_receivers(): comment and call-site: `If receivers are dropped first, discard all messages to free memory eagerly.` then `self.discard_all_messages();`; discard_all_messages(): comment indicates concurrency hazards: `The channel may be uninitialized, so we have to swap to avoid overwriting any sender's attempts to initialize the first block...` (truncated but clearly describing required coordination)

**Implementation:** Require a proof token/capability to call discard, e.g. `fn discard_all_messages(&self, _no_receivers: NoReceivers)` where `NoReceivers` is only constructible by the code that owns/joins receiver lifetimes (or by `disconnect_receivers(self: &Arc<Self>) -> DisconnectedChannel` returning a handle that cannot be used for receiving). Alternatively, structure the public API so only the 'receiver side dropped' path can reach the discard routine (sealed trait/private module + typed guard).

---

### 10. Thread-affinity invariant for wait_until (owner thread only)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/context.rs:1-140`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: wait_until() must only be invoked on the thread that owns the Context (the thread stored in Inner.thread / Inner.thread_id). This is enforced only by an unsafe API contract and comments; the type system does not prevent moving/sharing Context to other threads and calling wait_until() there. Misuse could lead to parking/unparking the wrong thread and logic bugs.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Inner


/// Thread-local context.
#[derive(Debug, Clone)]
pub struct Context {
    inner: Arc<Inner>,
}

// ... (other code) ...

    thread_id: usize,
}

impl Context {
    /// Creates a new context for the duration of the closure.
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Context) -> R,
    {
        thread_local! {
            /// Cached thread-local context.
            static CONTEXT: Cell<Option<Context>> = Cell::new(Some(Context::new()));
        }

        let mut f = Some(f);
        let mut f = |cx: &Context| -> R {
            let f = f.take().unwrap();
            f(cx)
        };

        CONTEXT
            .try_with(|cell| match cell.take() {
                None => f(&Context::new()),
                Some(cx) => {
                    cx.reset();
                    let res = f(&cx);
                    cell.set(Some(cx));
                    res
                }
            })
            .unwrap_or_else(|_| f(&Context::new()))
    }

    /// Creates a new `Context`.
    #[cold]
    fn new() -> Context {
        Context {
            inner: Arc::new(Inner {
                select: AtomicUsize::new(Selected::Waiting.into()),
                packet: AtomicPtr::new(ptr::null_mut()),
                thread: thread::current_or_unnamed(),
                thread_id: current_thread_id(),
            }),
        }
    }

    /// Resets `select` and `packet`.
    #[inline]
    fn reset(&self) {
        self.inner.select.store(Selected::Waiting.into(), Ordering::Release);
        self.inner.packet.store(ptr::null_mut(), Ordering::Release);
    }

    /// Attempts to select an operation.
    ///
    /// On failure, the previously selected operation is returned.
    #[inline]
    pub fn try_select(&self, select: Selected) -> Result<(), Selected> {
        self.inner
            .select
            .compare_exchange(
                Selected::Waiting.into(),
                select.into(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }

    /// Stores a packet.
    ///
    /// This method must be called after `try_select` succeeds and there is a packet to provide.
    #[inline]
    pub fn store_packet(&self, packet: *mut ()) {
        if !packet.is_null() {
            self.inner.packet.store(packet, Ordering::Release);
        }
    }

    /// Waits until an operation is selected and returns it.
    ///
    /// If the deadline is reached, `Selected::Aborted` will be selected.
    ///
    /// # Safety
    /// This may only be called from the thread this `Context` belongs to.
    #[inline]
    pub unsafe fn wait_until(&self, deadline: Option<Instant>) -> Selected {
        loop {
            // Check whether an operation has been selected.
            let sel = Selected::from(self.inner.select.load(Ordering::Acquire));
            if sel != Selected::Waiting {
                return sel;
            }

            // If there's a deadline, park the current thread until the deadline is reached.
            if let Some(end) = deadline {
                let now = Instant::now();

                if now < end {
                    // SAFETY: guaranteed by caller.
                    unsafe { self.inner.thread.park_timeout(end - now) };
                } else {
                    // The deadline has been reached. Try aborting select.
                    return match self.try_select(Selected::Aborted) {
                        Ok(()) => Selected::Aborted,
                        Err(s) => s,
                    };
                }
            } else {
                // SAFETY: guaranteed by caller.
                unsafe { self.inner.thread.park() };
            }
        }
    }

    /// Unparks the thread this context belongs to.
    #[inline]
    pub fn unpark(&self) {
        self.inner.thread.unpark();
    }

    /// Returns the id of the thread this context belongs to.
    #[inline]
    pub fn thread_id(&self) -> usize {
        self.inner.thread_id
    }
}

```

**Entity:** Context

**States:** OwnedByThread(T), CalledFromOtherThread(!T)

**Transitions:**
- OwnedByThread(T) -> CalledFromOtherThread(!T) via moving/cloning Context (it is Clone) and using it on a different thread

**Evidence:** Context derives Clone: `#[derive(Debug, Clone)] pub struct Context { inner: Arc<Inner> }` (can be shared across threads); Context::new(): stores `thread: thread::current_or_unnamed()` and `thread_id: current_thread_id()` into Inner; wait_until(): `# Safety This may only be called from the thread this Context belongs to.`; wait_until(): uses `unsafe { self.inner.thread.park() }` / `park_timeout(...)` with comment `// SAFETY: guaranteed by caller.`; Context::thread_id(): exposes the recorded owner thread id

**Implementation:** Return a non-Send/!Sync capability token tied to the creating thread (e.g., store `PhantomData<Rc<()>>` or a `std::thread::ThreadId`-bound guard) and require it to call wait_until(&self, &OwnerToken, ...). Alternatively, make a separate `LocalContext` that is `!Send` and provides wait_until safely, while `Context` remains shareable but cannot park.

---

### 51. ZeroToken raw-pointer validity/capability invariant (points to a live Packet/slot)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-7`

**Confidence**: low

**Suggested Pattern**: newtype

**Description**: ZeroToken is an untyped raw pointer wrapper (`*mut ()`) representing a pointer to some internal packet/slot. The type system does not enforce that the pointer is non-null, correctly aligned/typed, belongs to the right Channel/Inner instance, or that it is still live (not freed/reused). Any code that dereferences/casts this pointer relies on an implicit precondition that the token is valid and originates from the correct producer/consumer protocol elsewhere in the module.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Packet, impl Packet < T > (3 methods); struct Inner; struct Channel, impl Channel < T > (12 methods)

use crate::{fmt, ptr};

/// A pointer to a packet.
pub(crate) struct ZeroToken(*mut ());


```

**Entity:** ZeroToken

**States:** Invalid/Null or Dangling, Valid (points to a live packet/slot for this channel)

**Transitions:**
- Invalid -> Valid via creation/assignment of the inner pointer (not shown in snippet)
- Valid -> Invalid via channel/inner teardown or packet reuse/freeing (not shown in snippet)

**Evidence:** line 7: doc comment `/// A pointer to a packet.` indicates the pointer encodes a capability to access a packet; line 8: `pub(crate) struct ZeroToken(*mut ());` uses an untyped raw pointer, so null/dangling/wrong-provenance states are representable and unchecked here

**Implementation:** Replace `*mut ()` with `NonNull<Packet<T>>` (or `NonNull<PacketHeader>` if type-erased) to make null unrepresentable; optionally brand the token with a lifetime tied to the owning `Channel/Inner` (e.g., `struct ZeroToken<'a>(NonNull<PacketHeader>, PhantomData<&'a Inner>)`) so it cannot outlive the channel and cannot be mixed across channels.

---

### 46. ListToken validity/lifetime protocol (points to a live Block slot)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: ListToken is a raw pointer + offset pair that implicitly must refer to a currently-live block allocation and a valid slot position within that block. The type system does not tie the token to the lifetime of the underlying Block/Channel, does not prevent use-after-free when blocks are reclaimed, and does not enforce that `offset` is within the block's slot range or aligned to a slot boundary. Any code that constructs/consumes ListToken must therefore rely on external invariants (e.g., channel still alive, block not freed, offset computed correctly).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Block, impl Block < T > (3 methods); struct Position; struct Channel, impl Channel < T > (17 methods), impl Drop for Channel < T > (1 methods)


/// The token type for the list flavor.
#[derive(Debug)]
pub(crate) struct ListToken {
    /// The block of slots.
    block: *const u8,

    /// The offset into the block.
    offset: usize,
}

```

**Entity:** ListToken

**States:** Valid (points into a live Block at a valid slot offset), Invalid/Dangling (block null/garbage, freed/reused Block, or out-of-range/misaligned offset)

**Transitions:**
- Invalid/Dangling -> Valid via construction by channel/block code that sets `block` and `offset`
- Valid -> Invalid/Dangling when the referenced block is freed/recycled or the owning Channel is dropped

**Evidence:** field `block: *const u8` uses a raw pointer, implying unchecked validity and lifetime; field `offset: usize` encodes an index/byte-offset without a range-checked newtype; comment `/// The block of slots.` indicates `block` must correspond to a block allocation containing slots; comment `/// The offset into the block.` indicates `offset` is interpreted relative to `block` and therefore must be in-bounds for that block layout

**Implementation:** Replace `block: *const u8` with a typed pointer (e.g., `NonNull<Block<T>>` or `NonNull<u8>`) plus a `PhantomData<&'a Channel<T>>` or `&'a Block<T>`-tied lifetime so tokens cannot outlive the channel/block. Wrap `offset` in a newtype like `SlotIndex(u16/usize)` that is only constructible via checked constructors using the block's known slot count/alignment. If tokens must be sendable across threads without borrowing, use an epoch/hazard-pointer capability token or generation-count scheme embedded in the token type.

---

## Protocol Invariants

### 31. OnceState contextual validity (only meaningful during call_once_force callback)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/once.rs:1-323`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: `OnceState` is a contextual token whose methods (`is_poisoned`, and internal `poison`) are only intended to be used inside the closure passed to `Once::call_once_force`, reflecting the pre-existing poison status at the time the forced initialization runs. The type system does not express this temporal/contextual restriction directly; instead it relies on construction/visibility (not shown here) plus documentation that `OnceState` is "yielded" to the `call_once_force` closure and reports whether the `Once` "was poisoned prior to the invocation" of that closure.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct OnceState; enum ExclusiveState

/// [`OnceLock<T>`]: crate::sync::OnceLock
/// [`LazyLock<T, F>`]: crate::sync::LazyLock
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Once {
    inner: sys::Once,
}

#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl UnwindSafe for Once {}

#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl RefUnwindSafe for Once {}


// ... (other code) ...

)]
pub const ONCE_INIT: Once = Once::new();

impl Once {
    /// Creates a new `Once` value.
    #[inline]
    #[stable(feature = "once_new", since = "1.2.0")]
    #[rustc_const_stable(feature = "const_once_new", since = "1.32.0")]
    #[must_use]
    pub const fn new() -> Once {
        Once { inner: sys::Once::new() }
    }

    /// Performs an initialization routine once and only once. The given closure
    /// will be executed if this is the first time `call_once` has been called,
    /// and otherwise the routine will *not* be invoked.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// When this function returns, it is guaranteed that some initialization
    /// has run and completed (it might not be the closure specified). It is also
    /// guaranteed that any memory writes performed by the executed closure can
    /// be reliably observed by other threads at this point (there is a
    /// happens-before relation between the closure and code executing after the
    /// return).
    ///
    /// If the given closure recursively invokes `call_once` on the same [`Once`]
    /// instance, the exact behavior is not specified: allowed outcomes are
    /// a panic or a deadlock.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static mut VAL: usize = 0;
    /// static INIT: Once = Once::new();
    ///
    /// // Accessing a `static mut` is unsafe much of the time, but if we do so
    /// // in a synchronized fashion (e.g., write once or read all) then we're
    /// // good to go!
    /// //
    /// // This function will only call `expensive_computation` once, and will
    /// // otherwise always return the value returned from the first invocation.
    /// fn get_cached_val() -> usize {
    ///     unsafe {
    ///         INIT.call_once(|| {
    ///             VAL = expensive_computation();
    ///         });
    ///         VAL
    ///     }
    /// }
    ///
    /// fn expensive_computation() -> usize {
    ///     // ...
    /// # 2
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The closure `f` will only be executed once even if this is called
    /// concurrently amongst many threads. If that closure panics, however, then
    /// it will *poison* this [`Once`] instance, causing all future invocations of
    /// `call_once` to also panic.
    ///
    /// This is similar to [poisoning with mutexes][poison].
    ///
    /// [poison]: struct.Mutex.html#poisoning
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[track_caller]
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(false, &mut |_| f.take().unwrap()());
    }

    /// Performs the same function as [`call_once()`] except ignores poisoning.
    ///
    /// Unlike [`call_once()`], if this [`Once`] has been poisoned (i.e., a previous
    /// call to [`call_once()`] or [`call_once_force()`] caused a panic), calling
    /// [`call_once_force()`] will still invoke the closure `f` and will _not_
    /// result in an immediate panic. If `f` panics, the [`Once`] will remain
    /// in a poison state. If `f` does _not_ panic, the [`Once`] will no
    /// longer be in a poison state and all future calls to [`call_once()`] or
    /// [`call_once_force()`] will be no-ops.
    ///
    /// The closure `f` is yielded a [`OnceState`] structure which can be used
    /// to query the poison status of the [`Once`].
    ///
    /// [`call_once()`]: Once::call_once
    /// [`call_once_force()`]: Once::call_once_force
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// // poisoning propagates
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| {});
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// // call_once_force will still run and reset the poisoned state
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    ///
    /// // once any success happens, we stop propagating the poison
    /// INIT.call_once(|| {});
    /// ```
    #[inline]
    #[stable(feature = "once_poison", since = "1.51.0")]
    pub fn call_once_force<F>(&self, f: F)
    where
        F: FnOnce(&OnceState),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(true, &mut |p| f.take().unwrap()(p));
    }

    /// Returns `true` if some [`call_once()`] call has completed
    /// successfully. Specifically, `is_completed` will return false in
    /// the following situations:
    ///   * [`call_once()`] was not called at all,
    ///   * [`call_once()`] was called, but has not yet completed,
    ///   * the [`Once`] instance is poisoned
    ///
    /// This function returning `false` does not mean that [`Once`] has not been
    /// executed. For example, it may have been executed in the time between
    /// when `is_completed` starts executing and when it returns, in which case
    /// the `false` return value would be stale (but still permissible).
    ///
    /// [`call_once()`]: Once::call_once
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// INIT.call_once(|| {
    ///     assert_eq!(INIT.is_completed(), false);
    /// });
    /// assert_eq!(INIT.is_completed(), true);
    /// ```
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    /// assert_eq!(INIT.is_completed(), false);
    /// ```
    #[stable(feature = "once_is_completed", since = "1.43.0")]
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }

    /// Blocks the current thread until initialization has completed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static READY: Once = Once::new();
    ///
    /// let thread = thread::spawn(|| {
    ///     READY.wait();
    ///     println!("everything is ready");
    /// });
    ///
    /// READY.call_once(|| println!("performing setup"));
    /// ```
    ///
    /// # Panics
    ///
    /// If this [`Once`] has been poisoned because an initialization closure has
    /// panicked, this method will also panic. Use [`wait_force`](Self::wait_force)
    /// if this behavior is not desired.
    #[stable(feature = "once_wait", since = "1.86.0")]
    pub fn wait(&self) {
        if !self.inner.is_completed() {
            self.inner.wait(false);
        }
    }

    /// Blocks the current thread until initialization has completed, ignoring
    /// poisoning.
    #[stable(feature = "once_wait", since = "1.86.0")]
    pub fn wait_force(&self) {
        if !self.inner.is_completed() {
            self.inner.wait(true);
        }
    }

    /// Returns the current state of the `Once` instance.
    ///
    /// Since this takes a mutable reference, no initialization can currently
    /// be running, so the state must be either "incomplete", "poisoned" or
    /// "complete".
    #[inline]
    pub(crate) fn state(&mut self) -> ExclusiveState {
        self.inner.state()
    }

    /// Sets current state of the `Once` instance.
    ///
    /// Since this takes a mutable reference, no initialization can currently
    /// be running, so the state must be either "incomplete", "poisoned" or
    /// "complete".
    #[inline]
    pub(crate) fn set_state(&mut self, new_state: ExclusiveState) {
        self.inner.set_state(new_state);
    }
}

// ... (other code) ...

    }
}

impl OnceState {
    /// Returns `true` if the associated [`Once`] was poisoned prior to the
    /// invocation of the closure passed to [`Once::call_once_force()`].
    ///
    /// # Examples
    ///
    /// A poisoned [`Once`]:
    ///
    /// ```
    /// use std::sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    /// ```
    ///
    /// An unpoisoned [`Once`]:
    ///
    /// ```
    /// use std::sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(!state.is_poisoned());
    /// });
    #[stable(feature = "once_poison", since = "1.51.0")]
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Poison the associated [`Once`] without explicitly panicking.
    // NOTE: This is currently only exposed for `OnceLock`.
    #[inline]
    pub(crate) fn poison(&self) {
        self.inner.poison();
    }
}

```

**Entity:** OnceState

**States:** ValidInForceCallback, NotConstructible/NotMeaningfulOutsideCallback

**Transitions:**
- NotConstructible/NotMeaningfulOutsideCallback -> ValidInForceCallback via Once::call_once_force providing &OnceState to the closure
- ValidInForceCallback -> NotConstructible/NotMeaningfulOutsideCallback when the call_once_force closure returns (token should not escape conceptually)

**Evidence:** call_once_force(): signature `F: FnOnce(&OnceState)` and docs: "The closure `f` is yielded a `OnceState` structure"; OnceState::is_poisoned(): docs: "poisoned prior to the invocation of the closure passed to Once::call_once_force()" (context-dependent meaning); OnceState::poison(): comment: "only exposed for OnceLock" (indicates special-purpose token behavior)

**Implementation:** Make `OnceState` a linear/affine capability that cannot be stored/escaped: e.g., pass a fresh non-'static token type with an invariant lifetime tied to the call (already partially achieved by `&OnceState`), and gate side-effecting operations (like `poison`) behind a distinct capability type only constructible by the forcing path. If stronger guarantees are desired, use a private `OnceState<'a>` carrying a lifetime bound to the active call plus a private field so it cannot be fabricated, and ensure all meaningful operations require that lifetime (preventing any attempt to cache or reuse the state across calls).

---

### 21. Barrier rendezvous protocol (per-generation arrival counting + leader election)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/barrier.rs:1-118`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Barrier::wait() implements a cyclic rendezvous protocol across 'generations': threads increment a shared arrival count for the current generation; the last arriving thread (the 'leader') resets the count, advances generation_id, and wakes all waiters. Other threads must wait until generation_id changes. These protocol states and the requirement that exactly num_threads participants rendezvous per generation are enforced only by runtime shared state (Mutex<BarrierState>) and Condvar waiting, not by the type system. Misuse (e.g., fewer than num_threads threads ever calling wait, or mixing different groups of threads across generations) can lead to permanent blocking, but this is not representable at compile time with the current API.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct BarrierState; struct BarrierWaitResult

/// });
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Barrier {
    lock: Mutex<BarrierState>,
    cvar: Condvar,
    num_threads: usize,
}

// ... (other code) ...

    }
}

impl Barrier {
    /// Creates a new barrier that can block a given number of threads.
    ///
    /// A barrier will block `n`-1 threads which call [`wait()`] and then wake
    /// up all threads at once when the `n`th thread calls [`wait()`].
    ///
    /// [`wait()`]: Barrier::wait
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Barrier;
    ///
    /// let barrier = Barrier::new(10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_barrier", since = "1.78.0")]
    #[must_use]
    #[inline]
    pub const fn new(n: usize) -> Barrier {
        Barrier {
            lock: Mutex::new(BarrierState { count: 0, generation_id: 0 }),
            cvar: Condvar::new(),
            num_threads: n,
        }
    }

    /// Blocks the current thread until all threads have rendezvoused here.
    ///
    /// Barriers are re-usable after all threads have rendezvoused once, and can
    /// be used continuously.
    ///
    /// A single (arbitrary) thread will receive a [`BarrierWaitResult`] that
    /// returns `true` from [`BarrierWaitResult::is_leader()`] when returning
    /// from this function, and all other threads will receive a result that
    /// will return `false` from [`BarrierWaitResult::is_leader()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Barrier;
    /// use std::thread;
    ///
    /// let n = 10;
    /// let barrier = Barrier::new(n);
    /// thread::scope(|s| {
    ///     for _ in 0..n {
    ///         // The same messages will be printed together.
    ///         // You will NOT see any interleaving.
    ///         s.spawn(|| {
    ///             println!("before wait");
    ///             barrier.wait();
    ///             println!("after wait");
    ///         });
    ///     }
    /// });
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn wait(&self) -> BarrierWaitResult {
        let mut lock = self.lock.lock().unwrap();
        let local_gen = lock.generation_id;
        lock.count += 1;
        if lock.count < self.num_threads {
            let _guard =
                self.cvar.wait_while(lock, |state| local_gen == state.generation_id).unwrap();
            BarrierWaitResult(false)
        } else {
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1);
            self.cvar.notify_all();
            BarrierWaitResult(true)
        }
    }
}

// ... (other code) ...

    }
}

impl BarrierWaitResult {
    /// Returns `true` if this thread is the "leader thread" for the call to
    /// [`Barrier::wait()`].
    ///
    /// Only one thread will have `true` returned from their result, all other
    /// threads will have `false` returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Barrier;
    ///
    /// let barrier = Barrier::new(1);
    /// let barrier_wait_result = barrier.wait();
    /// println!("{:?}", barrier_wait_result.is_leader());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[must_use]
    pub fn is_leader(&self) -> bool {
        self.0
    }
}

```

**Entity:** Barrier

**States:** WaitingForArrivals(generation_id, count < num_threads), ReleasingGeneration(leader path; count == num_threads), NextGeneration(generation_id incremented, count reset)

**Transitions:**
- WaitingForArrivals -> WaitingForArrivals via Barrier::wait() when lock.count is incremented and remains < num_threads (thread blocks on cvar)
- WaitingForArrivals -> ReleasingGeneration via Barrier::wait() when lock.count reaches num_threads (leader branch)
- ReleasingGeneration -> NextGeneration via generation_id = generation_id.wrapping_add(1), count = 0, cvar.notify_all()

**Evidence:** field: Barrier { lock: Mutex<BarrierState>, cvar: Condvar, num_threads: usize } encodes protocol state in shared memory; Barrier::new(n): initializes BarrierState { count: 0, generation_id: 0 } and stores num_threads = n; Barrier::wait(): reads local_gen = lock.generation_id (per-generation token); Barrier::wait(): lock.count += 1; if lock.count < self.num_threads { ... wait_while(... local_gen == state.generation_id) ... } (blocks until generation_id changes); Barrier::wait(): else-branch resets lock.count = 0; advances lock.generation_id = ...wrapping_add(1); calls self.cvar.notify_all() (release step)

**Implementation:** Model 'participants for this generation' as a linear/affine token set: e.g., a Barrier could mint N Participant tokens (via a builder or split API) and only Participant::wait(&self) would be callable. Consuming all N tokens for a generation would be the only way to advance to the next generation, making 'exactly N arrivals' structurally enforced. (In practice, full enforcement is limited by threads/lifetimes, but a token-based API can prevent many accidental miscounts/missing participants.)

---

### 40. Slot stamp/message ownership protocol (Empty / Full / In-Transition)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-13`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Slot<T> encodes a concurrent single-slot protocol where the validity/initialization of `msg` is governed by the atomic `stamp`. The type system only sees `UnsafeCell<MaybeUninit<T>>`, so it cannot enforce that `msg` is only read when initialized, only written when empty, and dropped exactly once (either read out or discarded). Correctness relies on external code interpreting `stamp` values and performing the proper atomic transitions before accessing `msg` through `UnsafeCell`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ArrayToken; struct Channel, impl Channel < T > (17 methods)

use crate::time::Instant;

/// A slot in a channel.
struct Slot<T> {
    /// The current stamp.
    stamp: Atomic<usize>,

    /// The message in this slot. Either read out in `read` or dropped through
    /// `discard_all_messages`.
    msg: UnsafeCell<MaybeUninit<T>>,
}

```

**Entity:** Slot<T>

**States:** Empty (no initialized T in msg), Full (msg contains initialized T), In-Transition (being written/read/discarded; stamp used to arbitrate)

**Transitions:**
- Empty -> Full via producer write path (stamp change + initialize msg)
- Full -> Empty via consumer read path (stamp change + move-out msg)
- Full -> Empty via discard_all_messages (drop-in-place msg without read)

**Evidence:** field `stamp: Atomic<usize>`: "The current stamp." — runtime state machine token stored as an integer; field `msg: UnsafeCell<MaybeUninit<T>>`: message storage is explicitly maybe-uninitialized and interior-mutable; comment on `msg`: "Either read out in `read` or dropped through `discard_all_messages`." — implies an exclusive-or disposal protocol and exactly-once drop/read requirement not enforced by types

**Implementation:** Model the slot contents as a typestate or sum type tying initialization to the stamp protocol, e.g. `Slot<Empty, T>` vs `Slot<Full, T>` (or an internal `enum SlotState<T> { Empty, Full(T) }`) and expose only safe transitions; keep the low-level `UnsafeCell<MaybeUninit<T>>` + `stamp` in a private module and provide a safe API that returns a capability/token proving the slot is Full before allowing read/drop.

---

### 24. Waker registration lifecycle (Registered -> Unregistered/Notified/Disconnected-drained)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-211`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Waker is implicitly a registry/queue with a required cleanup protocol: threads register select operations (optionally with a packet) into `selectors`, and are expected to unregister themselves later; observers are drained by `notify()`. On `disconnect()`, selectors are notified but intentionally NOT removed (registered threads must still unregister to recover/destroy packets), while observers are drained. Drop asserts both queues are empty, meaning all registrations/observations must have been cleared before Waker is dropped. None of these obligations (e.g., 'must unregister before drop', 'disconnect not a terminal cleanup for selectors', 'packet pointer is valid until unregistered') are enforced by the type system; they rely on conventions, comments, and debug assertions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Entry; struct Waker, impl Waker (7 methods), impl Drop for Waker (1 methods), impl SyncWaker (5 methods), impl Drop for SyncWaker (1 methods); struct SyncWaker

//! Waking mechanism for threads blocked on channel operations.

use super::context::Context;
use super::select::{Operation, Selected};
use crate::ptr;
use crate::sync::Mutex;
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};

/// Represents a thread blocked on a specific channel operation.
pub(crate) struct Entry {
    /// The operation.
    pub(crate) oper: Operation,

    /// Optional packet.
    pub(crate) packet: *mut (),

    /// Context associated with the thread owning this operation.
    pub(crate) cx: Context,
}

/// A queue of threads blocked on channel operations.
///
/// This data structure is used by threads to register blocking operations and get woken up once
/// an operation becomes ready.
pub(crate) struct Waker {
    /// A list of select operations.
    selectors: Vec<Entry>,

    /// A list of operations waiting to be ready.
    observers: Vec<Entry>,
}

impl Waker {
    /// Creates a new `Waker`.
    #[inline]
    pub(crate) fn new() -> Self {
        Waker { selectors: Vec::new(), observers: Vec::new() }
    }

    /// Registers a select operation.
    #[inline]
    pub(crate) fn register(&mut self, oper: Operation, cx: &Context) {
        self.register_with_packet(oper, ptr::null_mut(), cx);
    }

    /// Registers a select operation and a packet.
    #[inline]
    pub(crate) fn register_with_packet(&mut self, oper: Operation, packet: *mut (), cx: &Context) {
        self.selectors.push(Entry { oper, packet, cx: cx.clone() });
    }

    /// Unregisters a select operation.
    #[inline]
    pub(crate) fn unregister(&mut self, oper: Operation) -> Option<Entry> {
        if let Some((i, _)) =
            self.selectors.iter().enumerate().find(|&(_, entry)| entry.oper == oper)
        {
            let entry = self.selectors.remove(i);
            Some(entry)
        } else {
            None
        }
    }

    /// Attempts to find another thread's entry, select the operation, and wake it up.
    #[inline]
    pub(crate) fn try_select(&mut self) -> Option<Entry> {
        if self.selectors.is_empty() {
            None
        } else {
            let thread_id = current_thread_id();

            self.selectors
                .iter()
                .position(|selector| {
                    // Does the entry belong to a different thread?
                    selector.cx.thread_id() != thread_id
                        && selector // Try selecting this operation.
                            .cx
                            .try_select(Selected::Operation(selector.oper))
                            .is_ok()
                        && {
                            // Provide the packet.
                            selector.cx.store_packet(selector.packet);
                            // Wake the thread up.
                            selector.cx.unpark();
                            true
                        }
                })
                // Remove the entry from the queue to keep it clean and improve
                // performance.
                .map(|pos| self.selectors.remove(pos))
        }
    }

    /// Notifies all operations waiting to be ready.
    #[inline]
    pub(crate) fn notify(&mut self) {
        for entry in self.observers.drain(..) {
            if entry.cx.try_select(Selected::Operation(entry.oper)).is_ok() {
                entry.cx.unpark();
            }
        }
    }

    /// Notifies all registered operations that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&mut self) {
        for entry in self.selectors.iter() {
            if entry.cx.try_select(Selected::Disconnected).is_ok() {
                // Wake the thread up.
                //
                // Here we don't remove the entry from the queue. Registered threads must
                // unregister from the waker by themselves. They might also want to recover the
                // packet value and destroy it, if necessary.
                entry.cx.unpark();
            }
        }

        self.notify();
    }
}

impl Drop for Waker {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(self.selectors.len(), 0);
        debug_assert_eq!(self.observers.len(), 0);
    }
}

/// A waker that can be shared among threads without locking.
///
/// This is a simple wrapper around `Waker` that internally uses a mutex for synchronization.
pub(crate) struct SyncWaker {
    /// The inner `Waker`.
    inner: Mutex<Waker>,

    /// `true` if the waker is empty.
    is_empty: Atomic<bool>,
}

impl SyncWaker {
    /// Creates a new `SyncWaker`.
    #[inline]
    pub(crate) fn new() -> Self {
        SyncWaker { inner: Mutex::new(Waker::new()), is_empty: AtomicBool::new(true) }
    }

    /// Registers the current thread with an operation.
    #[inline]
    pub(crate) fn register(&self, oper: Operation, cx: &Context) {
        let mut inner = self.inner.lock().unwrap();
        inner.register(oper, cx);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }

    /// Unregisters an operation previously registered by the current thread.
    #[inline]
    pub(crate) fn unregister(&self, oper: Operation) -> Option<Entry> {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.unregister(oper);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
        entry
    }

    /// Attempts to find one thread (not the current one), select its operation, and wake it up.
    #[inline]
    pub(crate) fn notify(&self) {
        if !self.is_empty.load(Ordering::SeqCst) {
            let mut inner = self.inner.lock().unwrap();
            if !self.is_empty.load(Ordering::SeqCst) {
                inner.try_select();
                inner.notify();
                self.is_empty.store(
                    inner.selectors.is_empty() && inner.observers.is_empty(),
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Notifies all threads that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.disconnect();
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }
}

impl Drop for SyncWaker {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.is_empty.load(Ordering::SeqCst));
    }
}

/// Returns a unique id for the current thread.
#[inline]
pub fn current_thread_id() -> usize {
    // `u8` is not drop so this variable will be available during thread destruction,
    // whereas `thread::current()` would not be
    thread_local! { static DUMMY: u8 = const { 0 } }
    DUMMY.with(|x| (x as *const u8).addr())
}

```

**Entity:** Waker

**States:** Empty, HasRegisteredSelectors, HasObservers, DisconnectedNotified

**Transitions:**
- Empty -> HasRegisteredSelectors via register()/register_with_packet()
- HasRegisteredSelectors -> HasRegisteredSelectors (minus one) via unregister()
- HasRegisteredSelectors -> HasRegisteredSelectors (minus one) via try_select() (removes selected entry)
- HasObservers -> Empty via notify() (drains observers)
- Any -> DisconnectedNotified via disconnect() (notifies selectors but does not remove; drains observers via notify())
- HasRegisteredSelectors/HasObservers -> (debug-assert failure) on Drop if not empty

**Evidence:** struct Waker { selectors: Vec<Entry>, observers: Vec<Entry> } encodes runtime state as queue emptiness/content; Waker::register_with_packet(): selectors.push(Entry { oper, packet, cx: cx.clone() }); Waker::unregister(): searches selectors by oper and removes it; otherwise returns None; Waker::try_select(): on successful selection, stores packet and unparks, then removes entry from selectors ("Remove the entry from the queue..."); Waker::notify(): for entry in self.observers.drain(..) { ... } drains observers (one-way transition); Waker::disconnect(): comment: "Here we don't remove the entry from the queue. Registered threads must unregister from the waker by themselves. They might also want to recover the packet value and destroy it"; impl Drop for Waker: debug_assert_eq!(self.selectors.len(), 0); debug_assert_eq!(self.observers.len(), 0)

**Implementation:** Return an RAII registration token from register/register_with_packet (e.g., `Registration<'a>` or `RegisteredOp`) that unregisters on Drop (and optionally yields/owns the packet). This makes 'must unregister before Waker drop' and packet cleanup automatic. If needed, split into typestates for `ConnectedWaker` vs `DisconnectedWaker` to make post-disconnect behavior explicit, or use a capability token for packet ownership to prevent dangling `*mut ()` usage.

---

### 45. Position pointer validity & ownership protocol (Uninitialized/Detached -> Attached to Block list)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Position<T> encodes a cursor into a linked-list-backed channel using an atomic index plus an atomic raw pointer to a Block<T>. The type system does not express whether `block: Atomic<*mut Block<T>>` is null/unset, points to a currently-live Block, or is properly associated with the current `index`. Any protocol that 'block must be a valid, live Block node corresponding to the current index' is implicit and must be upheld by the rest of the module via discipline and atomic ordering, but the struct exposes only a raw pointer state that can represent invalid/dangling values.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Block, impl Block < T > (3 methods); struct ListToken; struct Channel, impl Channel < T > (17 methods), impl Drop for Channel < T > (1 methods)


/// A position in a channel.
#[derive(Debug)]
struct Position<T> {
    /// The index in the channel.
    index: Atomic<usize>,

    /// The block in the linked list.
    block: Atomic<*mut Block<T>>,
}

```

**Entity:** Position<T>

**States:** DetachedOrUnknownBlock, AttachedToBlockList

**Transitions:**
- DetachedOrUnknownBlock -> AttachedToBlockList via storing a non-null valid `*mut Block<T>` into `block` (implicit; no typed transition)

**Evidence:** field `block: Atomic<*mut Block<T>>` uses a raw pointer with no lifetime/validity tracking; comment on `block`: "The block in the linked list." implies it should always reference an actual list node, but this is not encoded in the type; field `index: Atomic<usize>` together with `block` implies an implicit invariant that the pointer corresponds to the position denoted by `index` (relationship not enforced by types)

**Implementation:** Model the validity of the cursor as typestate: `Position<Detached, T>` vs `Position<Attached, T>`, where only `Position<Attached, T>` carries a `NonNull<Block<T>>` (or a wrapper like `BlockPtr<T>`), and transitions happen through constructors/methods that prove attachment. Alternatively, replace `*mut Block<T>` with `Option<NonNull<Block<T>>>` to at least rule out invalid null vs non-null states and make the 'detached' state explicit.

---

### 80. MappedMutexGuard protocol (derived-from-guard, non-null projection, must not outlive original lock/poison context)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/mutex.rs:1-498`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `MappedMutexGuard` is only valid as a projection derived from an existing `MutexGuard` while the mutex remains locked. Its correctness relies on an implicit protocol: (1) it must be created only from a live guard; (2) the mapping closure must return a reference into the guarded `T` (not an unrelated pointer); (3) the resulting pointer must be non-null and tied to the guard’s lifetime; (4) the original guard must not be dropped (to avoid unlocking) while the mapped guard exists—handled manually via `ManuallyDrop` and by moving the poison/inner references into the mapped guard. These constraints are enforced by unsafe blocks and comments rather than by a dedicated type-level representation of “this guard is a projection of that guard”.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct MutexGuard; struct MappedMutexGuard

///
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "Mutex")]
pub struct Mutex<T: ?Sized> {
    inner: sys::Mutex,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

// ... (other code) ...

///
/// [`into_inner`]: Mutex::into_inner
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}


// ... (other code) ...

///
/// [`Rc`]: crate::rc::Rc
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}


// ... (other code) ...

/// For this reason, [`MutexGuard`] must not implement `Send` to prevent it being dropped from
/// another thread.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

/// `T` must be `Sync` for a [`MutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
#[stable(feature = "mutexguard", since = "1.19.0")]
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedMutexGuard<'_, T> {}
#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedMutexGuard<'_, T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> Mutex<T> {
        Mutex { inner: sys::Mutex::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.lock() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// mutex.set(11).unwrap();
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.lock() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11).unwrap(), 7);
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.lock() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    /// The exact behavior on locking a mutex in the thread which already holds
    /// the lock is left unspecified. However, this function will not return on
    /// the second call (it might panic or deadlock, for example).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error once the mutex is acquired. The acquired
    /// mutex guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock().unwrap() = 10;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        unsafe {
            self.inner.lock();
            MutexGuard::new(self)
        }
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`Err`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return the [`Poisoned`] error if the mutex would
    /// otherwise be acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     let mut lock = c_mutex.try_lock();
    ///     if let Ok(ref mut mutex) = lock {
    ///         **mutex = 10;
    ///     } else {
    ///         println!("try_lock failed");
    ///     }
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        unsafe {
            if self.inner.try_lock() {
                Ok(MutexGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the mutex is poisoned.
    ///
    /// If another thread is active, the mutex can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    /// assert_eq!(mutex.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a mutex.
    ///
    /// If the mutex is poisoned, it will remain poisoned until this function is called. This
    /// allows recovering from a poisoned state and marking that it has recovered. For example, if
    /// the value is overwritten by a known-good value, then the mutex can be marked as
    /// un-poisoned. Or possibly, the value could be inspected to determine if it is in a
    /// consistent state, and if so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock().unwrap();
    ///     panic!(); // the mutex gets poisoned
    /// }).join();
    ///
    /// assert_eq!(mutex.is_poisoned(), true);
    /// let x = mutex.lock().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     mutex.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(mutex.is_poisoned(), false);
    /// assert_eq!(*x, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the underlying data
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner().unwrap(), 0);
    /// ```
    #[stable(feature = "mutex_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previous abandoned locks
    /// (e.g., via [`forget()`] on a [`MutexGuard`]).
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing a mutable reference to the
    /// underlying data instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut().unwrap() = 10;
    /// assert_eq!(*mutex.lock().unwrap(), 10);
    /// ```
    ///
    /// [`forget()`]: mem::forget
    #[stable(feature = "mutex_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

#[stable(feature = "mutex_from", since = "1.24.0")]
impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

// ... (other code) ...

    }
}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    unsafe fn new(lock: &'mutex Mutex<T>) -> LockResult<MutexGuard<'mutex, T>> {
        poison::map_result(lock.poison.guard(), |guard| MutexGuard { lock, poison: guard })
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock();
        }
    }
}

// ... (other code) ...

    &guard.lock.poison
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(orig: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedMutexGuard {
            data,
            inner: &orig.lock.inner,
            poison_flag: &orig.lock.poison,
            poison: orig.poison.clone(),
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::filter_map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn filter_map<U, F>(orig: Self, f: F) -> Result<MappedMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedMutexGuard {
                    data,
                    inner: &orig.lock.inner,
                    poison_flag: &orig.lock.poison,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            }
            None => Err(orig),
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl
// ... (truncated) ...
```

**Entity:** MappedMutexGuard<'_, U>

**States:** MappedHoldingLock, Released

**Transitions:**
- MappedHoldingLock -> Released via Drop of MappedMutexGuard (implied; constructed with inner/poison refs to perform unlock/poison bookkeeping when dropped)

**Evidence:** method: `pub fn map(orig: Self, f: F) -> MappedMutexGuard<'a, U>` uses `let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));` (requires f returns a valid reference into T; stored as raw NonNull); method: `pub fn map` uses `let orig = ManuallyDrop::new(orig);` (prevents original guard from dropping/unlocking); method: `pub fn filter_map` has the same pattern: `match f(unsafe { &mut *orig.lock.data.get() }) { Some(data) => { ... ManuallyDrop::new(orig) ... } None => Err(orig) }`; comments in both `map` and `filter_map`: "SAFETY: the conditions of `MutexGuard::new` were satisfied ..." and "The signature of the closure guarantees that it will not 'leak' the lifetime..." (explicitly describing a protocol requirement); type traits: `impl<T: ?Sized> !Send for MappedMutexGuard<'_, T> {}` indicates same-thread drop requirement carries over

**Implementation:** Represent the relationship “this is a projection of a held guard” explicitly by parameterizing `MappedMutexGuard` over a guard token/capability that cannot be duplicated. For example, have `MutexGuard` internally hold a private `Held<'a>` capability and `map(self, ...)` consume `MutexGuard` and produce `MappedMutexGuard` containing the same capability (not just `&sys::Mutex`), making it impossible (even internally) to construct a mapped guard without consuming an actual held-lock capability.

---

### 5. Leaked-guard / abandoned-lock hazard (Unlocked / Locked with outstanding guard) not reflected in &mut access

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/rwlock.rs:1-506`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The implementation relies on the protocol that lock guards are dropped to release the underlying lock. However, the docs explicitly note that `get_mut(&mut self)` does not clear previously abandoned locks (e.g., via `forget()` on guards). This implies an implicit state where the OS lock may remain locked even though the Rust borrow rules permit `&mut self` (because the guard was intentionally leaked). The type system cannot prevent `mem::forget(guard)` or model 'there exists an outstanding guard somewhere', so `get_mut` can be called in a logical state where the lock is still held, violating the usual expectation that `&mut self` implies exclusive access plus unlocked internal state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct RwLockReadGuard; struct RwLockWriteGuard; struct MappedRwLockReadGuard; struct MappedRwLockWriteGuard

/// [`Mutex`]: super::Mutex
#[stable(feature = "rust1", since = "1.0.0")]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLock")]
pub struct RwLock<T: ?Sized> {
    inner: sys::RwLock,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockReadGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockReadGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockReadGuard<'_, T> {}


// ... (other code) ...

}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockWriteGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockWriteGuard<'_, T> {}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(5);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[inline]
    pub const fn new(t: T) -> RwLock<T> {
        RwLock { inner: sys::RwLock::new(), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.read() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// lock.set(11).unwrap();
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.write() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                }
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(lock_value_accessors)]
    ///
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.replace(11).unwrap(), 7);
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    #[unstable(feature = "lock_value_accessors", issue = "133407")]
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.write() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. The failure will occur immediately after the lock has been
    /// acquired. The acquired lock guard will be contained in the returned
    /// error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read();
    ///     assert!(r.is_ok());
    /// }).join().unwrap();
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            self.inner.read();
            RwLockReadGuard::new(self)
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access.
    ///
    /// If the access could not be granted at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// match lock.try_read() {
    ///     Ok(n) => assert_eq!(*n, 1),
    ///     Err(_) => unreachable!(),
    /// };
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        unsafe {
            if self.inner.try_read() {
                Ok(RwLockReadGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. An error will be returned when the lock is acquired. The acquired
    /// lock guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write().unwrap();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            self.inner.write();
            RwLockWriteGuard::new(self)
        }
    }

    /// Attempts to lock this `RwLock` with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let n = lock.read().unwrap();
    /// assert_eq!(*n, 1);
    ///
    /// assert!(lock.try_write().is_err());
    /// ```
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        unsafe {
            if self.inner.try_write() {
                Ok(RwLockWriteGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Determines whether the lock is poisoned.
    ///
    /// If another thread is active, the lock can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    /// assert_eq!(lock.is_poisoned(), true);
    /// ```
    #[inline]
    #[stable(feature = "sync_poison", since = "1.2.0")]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a lock.
    ///
    /// If the lock is poisoned, it will remain poisoned until this function is called. This allows
    /// recovering from a poisoned state and marking that it has recovered. For example, if the
    /// value is overwritten by a known-good value, then the lock can be marked as un-poisoned. Or
    /// possibly, the value could be inspected to determine if it is in a consistent state, and if
    /// so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::thread;
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// }).join();
    ///
    /// assert_eq!(lock.is_poisoned(), true);
    /// let guard = lock.write().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     lock.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(lock.is_poisoned(), false);
    /// assert_eq!(*guard, 1);
    /// ```
    #[inline]
    #[stable(feature = "mutex_unpoison", since = "1.77.0")]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the underlying data if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock. An error will only be returned
    /// if the lock would have otherwise been acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write().unwrap();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner().unwrap(), "modified");
    /// ```
    #[stable(feature = "rwlock_into_inner", since = "1.6.0")]
    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no new locks can be acquired
    /// while this reference exists. Note that this method does not clear any previously abandoned locks
    /// (e.g., via [`forget()`] on a [`RwLockReadGuard`] or [`RwLockWriteGuard`]).
    ///
    /// # Errors
    ///
    /// This function will return an error containing a mutable reference to
    /// the underlying data if the `RwLock` is poisoned. An `RwLock` is
    /// poisoned whenever a writer panics while holding an exclusive lock.
    /// An error will only be returned if the lock would have otherwise been
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let mut lock = RwLock::new(0);
    /// *lock.get_mut().unwrap() = 10;
    /// assert_eq!(*lock.read().unwrap(), 10);
    /// ```
    #[stable(feature = "rwlock_get_mut", since = "1.6.0")]
    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

// ... (other code) ...

}

#[stable(feature = "rw_lock_from", since = "1.24.0")]
impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}


// ... (truncated) ...
```

**Entity:** RwLock<T>

**States:** NoOutstandingGuards, OutstandingGuardsOrAbandonedLock

**Transitions:**
- NoOutstandingGuards -> OutstandingGuardsOrAbandonedLock via `mem::forget()` / `forget()` on `RwLockReadGuard` or `RwLockWriteGuard` (documented)
- OutstandingGuardsOrAbandonedLock -> NoOutstandingGuards via 'guard dropped normally' (intended RAII protocol)

**Evidence:** doc on `get_mut(&mut self)`: 'Note that this method does not clear any previously abandoned locks (e.g., via [`forget()`] on a [`RwLockReadGuard`] or [`RwLockWriteGuard`]).'; methods `read`/`write`/`try_read`/`try_write`: return RAII guards ('will release ... once it is dropped') indicating the release protocol depends on Drop

**Implementation:** Make APIs that assume 'no outstanding OS-level lock state' require a capability token that can only be obtained when the lock is known-unlocked (e.g., returned from guard drop in a controlled API), or provide a separate `RwLock::into_mut(self)` consuming the lock (already safe) as the only way to get `&mut T` without depending on the OS lock state. Alternatively, wrap guards in a type that is harder to intentionally leak (not fully preventable in Rust, but can be discouraged by not exposing raw guards in certain higher-level APIs).

---

### 47. Channel-flavor token validity protocol (Array/List/Zero)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/select.rs:1-11`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Token aggregates per-flavor token data (array/list/zero). The implicit protocol is that, for any given select/operation, only the token for the actually-used channel flavor is meaningful; the other fields are logically inactive/garbage for that operation. This is not enforced by the type system because Token always contains all three sub-tokens simultaneously and does not encode which flavor (if any) is currently active. As a result, code elsewhere must rely on out-of-band knowledge (the channel flavor) to read/write the correct field and avoid mixing tokens across flavors.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Operation, impl Operation (1 methods); enum Selected, impl From < usize > for Selected (1 methods), impl Into < usize > for Selected (1 methods)

///
/// Each field contains data associated with a specific channel flavor.
#[derive(Debug, Default)]
pub struct Token {
    pub(crate) array: super::array::ArrayToken,
    pub(crate) list: super::list::ListToken,
    #[allow(dead_code)]
    pub(crate) zero: super::zero::ZeroToken,
}

```

**Entity:** Token

**States:** Array-active, List-active, Zero-active, Inactive/unused

**Transitions:**
- Inactive/unused -> {Array-active|List-active|Zero-active} via selection/registration code in other parts of the module (not shown)
- {Array-active|List-active|Zero-active} -> Inactive/unused via completion/cancellation code in other parts of the module (not shown)
- {Array-active|List-active|Zero-active} -> {Array-active|List-active|Zero-active} via reuse across operations (not shown)

**Evidence:** comment: "Each field contains data associated with a specific channel flavor." (implies mutual exclusivity / flavor-dependent validity); struct Token fields: array: super::array::ArrayToken, list: super::list::ListToken, zero: super::zero::ZeroToken (three parallel representations with no discriminator); #[derive(Default)] on Token (enables constructing a Token without selecting a flavor; implies an Inactive/unused state is possible)

**Implementation:** Replace the "all-fields" Token with a discriminated typestate: either `enum Token { Array(ArrayToken), List(ListToken), Zero(ZeroToken) }` (simple) or `struct Token<F> { inner: F::Token }` with marker types `ArrayFlavor/ListFlavor/ZeroFlavor` implementing a `Flavor` trait. Then APIs that operate on a given channel flavor accept/return `Token<ArrayFlavor>` etc., preventing accidental cross-flavor use at compile time.

---

### 62. OnceState wrapper depends on sys::OnceState protocol (initialized/poison/complete)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/once.rs:1-10`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: OnceState contains an inner sys::OnceState that likely encodes the low-level once/poison protocol (e.g., uninitialized vs running vs complete vs poisoned) for a particular platform. The wrapper exposes this state as a plain field, so the Rust type system cannot express which operations are legal in which inner state or ensure correct ordering; correctness depends on adhering to the implicit protocol of sys::OnceState at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Once, impl UnwindSafe for Once (0 methods), impl RefUnwindSafe for Once (0 methods), impl Once (8 methods), impl OnceState (2 methods); struct OnceState

    pub(crate) inner: sys::OnceState,
}

pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}

```

**Entity:** OnceState

**States:** (implicit) OS/arch-specific once states represented by sys::OnceState

**Transitions:**
- (implicit) transitions inside sys::OnceState as driven by Once/OnceState methods elsewhere in the module

**Evidence:** field: pub(crate) inner: sys::OnceState

**Implementation:** Hide `inner` behind a private field and represent the high-level state as a typed state parameter (OnceState<S>) that only allows calling the subset of sys operations valid for S. Alternatively, wrap sys::OnceState in a safe newtype with constructors that establish invariants and methods that consume/return newtypes representing the next state.

---

### 27. Receiver-driven iteration protocol (borrowed, blocking receive until end-of-stream)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Iter is an adapter over a borrowed Receiver that repeatedly calls recv() and maps any error to None. This encodes an implicit protocol: iteration may block while the channel is open/empty, and it terminates only when recv() returns an error (typically channel disconnect). The type system does not distinguish an iterator that can block from one that cannot, nor does it make the termination/disconnect condition explicit (it is erased by ok()).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Sender, 2 free function(s), impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl UnwindSafe for Sender < T > (0 methods), impl RefUnwindSafe for Sender < T > (0 methods), impl Sender < T > (2 methods), impl Sender < T > (7 methods), impl Drop for Sender < T > (1 methods); struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** Iter<'a, T>

**States:** Active (can block), Terminated (channel closed/recv error)

**Transitions:**
- Active -> Terminated via Iterator::next() when self.rx.recv() returns Err (mapped to None by ok())

**Evidence:** struct Iter { rx: &'a Receiver<T> } — iteration behavior is entirely delegated to Receiver; impl Iterator for Iter: next() { self.rx.recv().ok() } — blocking recv() + error-to-None conversion defines termination

**Implementation:** Introduce distinct iterator types (e.g., BlockingIter<'a, T> vs NonBlockingIter<'a, T>) or a typestate parameter indicating blocking behavior; alternatively return Option<Result<T, RecvError>> (or a custom enum) to avoid erasing the disconnect state into None.

---

### 53. Operation/Context pairing protocol (cx must belong to oper owner thread/task)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-15`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: Entry is described as representing a thread blocked on a specific channel operation. That implies an invariant that `cx` is the context associated with the thread/task that owns `oper`, and that this association remains valid while the Entry is stored/used for waking. The type system does not enforce that the `Context` is correctly paired with `oper`, that it originates from the correct thread/task, or that it remains valid for the necessary duration (depending on how Context is represented elsewhere).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Waker, impl Waker (7 methods), impl Drop for Waker (1 methods), impl SyncWaker (5 methods), impl Drop for SyncWaker (1 methods); struct SyncWaker; 1 free function(s)

use crate::sync::atomic::{Atomic, AtomicBool, Ordering};

/// Represents a thread blocked on a specific channel operation.
pub(crate) struct Entry {
    /// The operation.
    pub(crate) oper: Operation,

    /// Optional packet.
    pub(crate) packet: *mut (),

    /// Context associated with the thread owning this operation.
    pub(crate) cx: Context,
}

```

**Entity:** Entry

**States:** Unregistered (cx not yet associated with a real waiting thread/task), Registered (cx corresponds to the thread/task blocked on `oper`)

**Transitions:**
- Unregistered -> Registered by constructing/filling `Entry { oper, packet, cx }` for a blocking operation

**Evidence:** comment `/// Represents a thread blocked on a specific channel operation.` implies a registered-waiter protocol; field `oper: Operation` and field `cx: Context` co-exist with no type-level linkage ensuring they correspond to the same waiter/owner

**Implementation:** Introduce a waiter token/capability tying the operation and context together, created only by the code that registers a waiter: e.g., `struct Waiter { oper: Operation, cx: Context }` (private fields) returned from a safe constructor, and make `Entry` hold `waiter: Waiter` instead of separate `oper`/`cx`. This prevents creating mismatched pairs outside the module.

---

### 28. Receiver-driven iteration protocol (borrowed, non-blocking try_recv until end-of-stream)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: TryIter is an adapter over a borrowed Receiver that repeatedly calls try_recv() and maps any error to None. This implies a protocol: next() never blocks, and iteration stops when try_recv() returns an error. The type system does not express that this iterator is non-blocking, nor does it differentiate 'no message yet' vs 'disconnected' if those are represented as errors by try_recv(); ok() erases that information into None.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Sender, 2 free function(s), impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl UnwindSafe for Sender < T > (0 methods), impl RefUnwindSafe for Sender < T > (0 methods), impl Sender < T > (2 methods), impl Sender < T > (7 methods), impl Drop for Sender < T > (1 methods); struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** TryIter<'a, T>

**States:** Active (non-blocking polling), Terminated (channel closed/try_recv error)

**Transitions:**
- Active -> Terminated via Iterator::next() when self.rx.try_recv() returns Err (mapped to None by ok())

**Evidence:** impl Iterator for TryIter: next() { self.rx.try_recv().ok() } — non-blocking try_recv() + error-to-None conversion defines termination

**Implementation:** Use a newtype/alternative item type to preserve error/state (e.g., Iterator<Item = Result<T, TryRecvError>> or Item = Poll<T>), making 'empty' vs 'disconnected' explicit and preventing accidental misuse where None is assumed to mean permanent end.

---

### 67. IntoIter consumption protocol (Receiver-owned iteration)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-8`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: IntoIter<T> owns a Receiver<T> (field rx) and is intended to be used as a consuming iterator over the channel. There is an implicit protocol that once iteration begins, the Receiver is effectively dedicated to the iterator and will be driven until it is exhausted (channel closes / no more messages). The type system does not model the iterator's internal progress (active vs exhausted) nor does it prevent misuse patterns such as expecting the iterator to be restartable or to continue yielding after exhaustion; these conditions are handled at runtime by the Iterator implementation (not shown in this snippet).

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// ```
#[stable(feature = "receiver_into_iter", since = "1.1.0")]
#[derive(Debug)]
pub struct IntoIter<T> {
    rx: Receiver<T>,
}

```

**Entity:** IntoIter<T>

**States:** Active (can yield items), Exhausted (iteration finished; receiver drained/disconnected)

**Transitions:**
- Active -> Exhausted via Iterator::next() repeatedly returning None (implementation not shown here)

**Evidence:** pub struct IntoIter<T> { rx: Receiver<T>, } — owning Receiver implies a consuming-iteration protocol over channel state

**Implementation:** Model iterator progress with a typestate parameter, e.g., IntoIter<T, S> where S is Active/Exhausted; next(self) -> (Self in new state, Option<T>) or expose a separate 'drain' API that consumes IntoIter and returns a terminal marker. (In practice Rust's Iterator trait makes this awkward; a more realistic compile-time enforcement is to expose a dedicated drain type returned by Receiver::into_iter() that cannot be recreated without a new Receiver.)

---

### 37. Poison flag protocol (Clean / Poisoned) with guarded transition

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison.rs:1-70`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Flag encodes a runtime poison state (whether a protected operation previously panicked). Callers are expected to follow a specific protocol: before doing an operation they either call borrow() (read-only check) or guard() (check that also returns a Guard snapshot of panicking state). After completing the operation, callers must call done(&guard) to potentially mark the flag poisoned if a panic started during the guarded operation. None of these temporal ordering requirements are enforced by the type system: Guard does not carry a lifetime tying it to a particular Flag, and nothing forces done() to be called after guard(), or prevents calling done() with a Guard created for a different Flag.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Guard; struct PoisonError, impl Error for PoisonError < T > (1 methods), impl PoisonError < T > (5 methods); enum TryLockError, impl From < PoisonError < T > > for TryLockError < T > (1 methods), impl Error for TryLockError < T > (2 methods); 1 free function(s)

pub(crate) mod once;
mod rwlock;

pub(crate) struct Flag {
    #[cfg(panic = "unwind")]
    failed: Atomic<bool>,
}

// ... (other code) ...

// As a result, if it matters, we should see the correct value for `failed` in
// all cases.

impl Flag {
    #[inline]
    pub const fn new() -> Flag {
        Flag {
            #[cfg(panic = "unwind")]
            failed: AtomicBool::new(false),
        }
    }

    /// Checks the flag for an unguarded borrow, where we only care about existing poison.
    #[inline]
    pub fn borrow(&self) -> LockResult<()> {
        if self.get() { Err(PoisonError::new(())) } else { Ok(()) }
    }

    /// Checks the flag for a guarded borrow, where we may also set poison when `done`.
    #[inline]
    pub fn guard(&self) -> LockResult<Guard> {
        let ret = Guard {
            #[cfg(panic = "unwind")]
            panicking: thread::panicking(),
        };
        if self.get() { Err(PoisonError::new(ret)) } else { Ok(ret) }
    }

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn done(&self, guard: &Guard) {
        if !guard.panicking && thread::panicking() {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    #[inline]
    #[cfg(not(panic = "unwind"))]
    pub fn done(&self, _guard: &Guard) {}

    #[inline]
    #[cfg(panic = "unwind")]
    pub fn get(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    #[inline(always)]
    #[cfg(not(panic = "unwind"))]
    pub fn get(&self) -> bool {
        false
    }

    #[inline]
    pub fn clear(&self) {
        #[cfg(panic = "unwind")]
        self.failed.store(false, Ordering::Relaxed)
    }
}

```

**Entity:** Flag

**States:** Clean, Poisoned

**Transitions:**
- Clean -> Poisoned via done(&Guard) when guard.panicking == false && thread::panicking() == true
- Poisoned -> Clean via clear()

**Evidence:** field `failed: AtomicBool` (cfg(panic = "unwind")) stores runtime state for poisoned vs clean; method `borrow(&self) -> LockResult<()>`: `if self.get() { Err(PoisonError::new(())) } else { Ok(()) }` gates behavior on poison state; method `guard(&self) -> LockResult<Guard>` constructs `Guard { panicking: thread::panicking() }` and returns it inside `Ok(ret)` / `Err(PoisonError::new(ret))` depending on `self.get()`; method `done(&self, guard: &Guard)`: `if !guard.panicking && thread::panicking() { self.failed.store(true, ...) }` relies on Guard having been obtained before the critical section and on done() being called after; method `clear(&self)` resets `failed` to false, creating an explicit Poisoned -> Clean transition

**Implementation:** Make `Flag::guard()` return an RAII type tied to the `Flag`, e.g. `struct PoisonGuard<'a> { flag: &'a Flag, panicking_on_entry: bool }` that records `thread::panicking()` on creation and runs the current `done()` logic in `Drop`. This enforces the required ordering (guard acquisition before work; poison marking after) and prevents forgetting to call `done()`. Optionally, keep `borrow()` as a separate read-only check.

---

### 58. Sender/receiver rendezvous protocol via two wait-queues (must register then be woken/paired)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-15`

**Confidence**: low

**Suggested Pattern**: session_type

**Description**: Inner maintains two separate Waker sets for pending senders and pending receivers, implying a rendezvous protocol: a send can only complete when a receiver is present (and vice versa), otherwise the task must be registered in the appropriate wait-queue and later woken when a counterpart arrives. This multi-step interaction (register -> wait -> wake -> pair) is implicit and enforced by runtime coordination over the two Waker fields; the type system does not express which side is currently waiting or whether a pairing is in progress.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Packet, impl Packet < T > (3 methods); struct Channel, impl Channel < T > (12 methods)

}

/// Inner representation of a zero-capacity channel.
struct Inner {
    /// Senders waiting to pair up with a receive operation.
    senders: Waker,

    /// Receivers waiting to pair up with a send operation.
    receivers: Waker,

    /// Equals `true` when the channel is disconnected.
    is_disconnected: bool,
}

```

**Entity:** Inner

**States:** NoWaiters, SendersWaiting, ReceiversWaiting, PairedInProgress

**Transitions:**
- NoWaiters -> SendersWaiting via registering current task waker in senders
- NoWaiters -> ReceiversWaiting via registering current task waker in receivers
- SendersWaiting -> PairedInProgress via arrival of a receiver waking a sender
- ReceiversWaiting -> PairedInProgress via arrival of a sender waking a receiver
- PairedInProgress -> NoWaiters after handoff completes

**Evidence:** field: senders: Waker — "Senders waiting to pair up with a receive operation." (encodes 'send registered and pending'); field: receivers: Waker — "Receivers waiting to pair up with a send operation." (encodes 'recv registered and pending'); comment: "pair up" / "waiting" indicates a temporal multi-step rendezvous rather than a single atomic operation

**Implementation:** Model the rendezvous as explicit session states/capabilities: e.g., a ReceivePermit token produced when a receiver is registered, which must be consumed by a sender to complete the handoff (and symmetrically a SendPermit). This can be approximated with typestate/capability tokens (zero-sized markers) that make 'registered' vs 'completed/cancelled' phases explicit and reduce reliance on implicit Waker coordination.

---

### 3. Non-reentrant initialization protocol (initializer must not touch the same OnceLock)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/once_lock.rs:1-512`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The initializer closures passed to `get_or_init`/`get_or_try_init` must not reentrantly attempt to initialize the same cell (directly or indirectly). This is documented as an error with unspecified outcome (currently deadlock). The type system does not prevent calling `get_or_init` on the same `OnceLock` from within `f`, so correctness depends on a dynamic protocol ("do not reenter").

**Evidence**:

```rust
///
/// ```
#[stable(feature = "once_cell", since = "1.70.0")]
pub struct OnceLock<T> {
    // FIXME(nonpoison_once): switch to nonpoison version once it is available
    once: Once,
    // Whether or not the value is initialized is tracked by `once.is_completed()`.
    value: UnsafeCell<MaybeUninit<T>>,
    /// `PhantomData` to make sure dropck understands we're dropping T in our Drop impl.
    ///
    /// ```compile_fail,E0597
    /// use std::sync::OnceLock;
    ///
    /// struct A<'a>(&'a str);
    ///
    /// impl<'a> Drop for A<'a> {
    ///     fn drop(&mut self) {}
    /// }
    ///
    /// let cell = OnceLock::new();
    /// {
    ///     let s = String::new();
    ///     let _ = cell.set(A(&s));
    /// }
    /// ```
    _marker: PhantomData<T>,
}

impl<T> OnceLock<T> {
    /// Creates a new uninitialized cell.
    #[inline]
    #[must_use]
    #[stable(feature = "once_cell", since = "1.70.0")]
    #[rustc_const_stable(feature = "once_cell", since = "1.70.0")]
    pub const fn new() -> OnceLock<T> {
        OnceLock {
            once: Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is uninitialized, or being initialized.
    /// This method never blocks.
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is uninitialized, or being initialized.
    /// This method never blocks.
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized and we have a unique access
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    /// Blocks the current thread until the cell is initialized.
    ///
    /// # Example
    ///
    /// Waiting for a computation on another thread to finish:
    /// ```rust
    /// use std::thread;
    /// use std::sync::OnceLock;
    ///
    /// let value = OnceLock::new();
    ///
    /// thread::scope(|s| {
    ///     s.spawn(|| value.set(1 + 1));
    ///
    ///     let result = value.wait();
    ///     assert_eq!(result, &2);
    /// })
    /// ```
    #[inline]
    #[stable(feature = "once_wait", since = "1.86.0")]
    pub fn wait(&self) -> &T {
        self.once.wait_force();

        unsafe { self.get_unchecked() }
    }

    /// Initializes the contents of the cell to `value`.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when `set` returns, though not necessarily the one provided.
    ///
    /// Returns `Ok(())` if the cell was uninitialized and
    /// `Err(value)` if the cell was already initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.set(92), Ok(()));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.set(62), Err(62));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn set(&self, value: T) -> Result<(), T> {
        match self.try_insert(value) {
            Ok(_) => Ok(()),
            Err((_, value)) => Err(value),
        }
    }

    /// Initializes the contents of the cell to `value` if the cell was uninitialized,
    /// then returns a reference to it.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when `try_insert` returns, though not necessarily the
    /// one provided.
    ///
    /// Returns `Ok(&value)` if the cell was uninitialized and
    /// `Err((&current_value, value))` if it was already initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_try_insert)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.try_insert(92), Ok(&92));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.try_insert(62), Err((&92, 62)));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    #[unstable(feature = "once_cell_try_insert", issue = "116693")]
    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        let mut value = Some(value);
        let res = self.get_or_init(|| value.take().unwrap());
        match value {
            None => Ok(res),
            Some(value) => Err((res, value)),
        }
    }

    /// Gets the contents of the cell, initializing it to `f()` if the cell
    /// was uninitialized.
    ///
    /// Many threads may call `get_or_init` concurrently with different
    /// initializing functions, but it is guaranteed that only one function
    /// will be executed.
    ///
    /// # Panics
    ///
    /// If `f()` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. The
    /// exact outcome is unspecified. Current implementation deadlocks, but
    /// this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// let value = cell.get_or_init(|| 92);
    /// assert_eq!(value, &92);
    /// let value = cell.get_or_init(|| unreachable!());
    /// assert_eq!(value, &92);
    /// ```
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get_or_try_init(|| Ok::<T, !>(f())) {
            Ok(val) => val,
        }
    }

    /// Gets the mutable reference of the contents of the cell, initializing
    /// it to `f()` if the cell was uninitialized.
    ///
    /// This method never blocks.
    ///
    /// # Panics
    ///
    /// If `f()` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_get_mut)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// let mut cell = OnceLock::new();
    /// let value = cell.get_mut_or_init(|| 92);
    /// assert_eq!(*value, 92);
    ///
    /// *value += 2;
    /// assert_eq!(*value, 94);
    ///
    /// let value = cell.get_mut_or_init(|| unreachable!());
    /// assert_eq!(*value, 94);
    /// ```
    #[inline]
    #[unstable(feature = "once_cell_get_mut", issue = "121641")]
    pub fn get_mut_or_init<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match self.get_mut_or_try_init(|| Ok::<T, !>(f())) {
            Ok(val) => val,
        }
    }

    /// Gets the contents of the cell, initializing it to `f()` if
    /// the cell was uninitialized. If the cell was uninitialized
    /// and `f()` failed, an error is returned.
    ///
    /// # Panics
    ///
    /// If `f()` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`.
    /// The exact outcome is unspecified. Current implementation
    /// deadlocks, but this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_try)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// assert_eq!(cell.get_or_try_init(|| Err(())), Err(()));
    /// assert!(cell.get().is_none());
    /// let value = cell.get_or_try_init(|| -> Result<i32, ()> {
    ///     Ok(92)
    /// });
    /// assert_eq!(value, Ok(&92));
    /// assert_eq!(cell.get(), Some(&92))
    /// ```
    #[inline]
    #[unstable(feature = "once_cell_try", issue = "109737")]
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Fast path check
        // NOTE: We need to perform an acquire on the state in this method
        // in order to correctly synchronize `LazyLock::force`. This is
        // currently done by calling `self.get()`, which in turn calls
        // `self.is_initialized()`, which in turn performs the acquire.
        if let Some(value) = self.get() {
            return Ok(value);
        }
        self.initialize(f)?;

        debug_assert!(self.is_initialized());

        // SAFETY: The inner value has been initialized
        Ok(unsafe { self.get_unchecked() })
    }

    /// Gets the mutable reference of the contents of the cell, initializing
    /// it to `f()` if the cell was uninitialized. If the cell was uninitialized
    /// and `f()` failed, an error is returned.
    ///
    /// This method never blocks.
    ///
    /// # Panics
    ///
    /// If `f()` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(once_cell_get_mut)]
    ///
    /// use std::sync::OnceLock;
    ///
    /// let mut cell: OnceLock<u32> = OnceLock::new();
    ///
    /// // Failed attempts to initialize the cell do not change its contents
    /// assert!(cell.get_mut_or_try_init(|| "not a number!".parse()).is_err());
    /// assert!(cell.get().is_none());
    ///
    /// let value = cell.get_mut_or_try_init(|| "1234".parse());
    /// assert_eq!(value, Ok(&mut 1234));
    /// *value.unwrap() += 2;
    /// assert_eq!(cell.get(), Some(&1236))
    /// ```
    #[inline]
    #[unstable(feature = "once_cell_get_mut", issue = "121641")]
    pub fn get_mut_or_try_init<F, E>(&mut self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if self.get().is_none() {
            self.initialize(f)?;
        }
        debug_assert!(self.is_initialized());
        // SAFETY: The inner value has been initialized
        Ok(unsafe { self.get_unchecked_mut() })
    }

    /// Consumes the `OnceLock`, returning the wrapped value. Returns
    /// `None` if the cell was uninitialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.into_inner(), None);
    ///
    /// let cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.into_inner(), Some("hello".to_string()));
    /// ```
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    /// Takes the value out of this `OnceLock`, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the `OnceLock` was uninitialized.
    ///
    /// Safety is guaranteed by requiring a mutable reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let mut cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.take(), None);
    ///
    /// let mut cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.take(), Some("hello".to_string()));
    /// assert_eq!(cell.get(), None);
    /// ```
    #[inline]
    #[stable(feature = "once_cell", since = "1.70.0")]
    pub fn take(&mut self) -> Option<T> {
        if self.is_initialized() {
            self.once = Once::new();
            // SAFETY: `self.value` is initialized and contains a valid `T`.
            // `self.once` is reset, so `is_initialized()` will be false again
            // which prevents the value from being read twice.
            unsafe { Some((&mut *self.value.get()).assume_init_read()) }
        } else {
            None
        }
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    #[cold]
    #[optimize(size)]
    fn initialize<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut res: Result<(), E> = Ok(());
        let slot = &self.value;

        // Ignore poisoning from other threads
        // If another thread panics, then we'll be able to run our closure
        self.once.call_once_force(|p| {
            match f() {
                Ok(value) => {
                    unsafe { (&mut *slot.get()).write(value) };
                }
                Err(e) => {
                    res = Err(e);

                    // Treat the underlying `Once` as poisoned since we
                    // failed to initialize our value.
                    p.poison();
                }
            }
        });
        res
    }

    /// # Safety
    ///
    /// The cell must be initialized
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        unsafe { (&*self.value.get()).assume_init_ref() }
    }

    /// # Safety
    ///
    /// The cell must be initialized
    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        debug_assert!(self.is_initialized());
        unsafe { (&mut *self.value.get()).assume_init_mut() }
    }
}

// ... (other code) ...

// then destroyed by A. That is, destructor observes
// a sent value.
#[stable(feature = "once_cell", since = "1.70.0")]
unsafe impl<T: Sync + Send> Sync for OnceLock<T> {}
#[stable(feature = "once_cell", since = "1.70.0")]
unsafe impl<T: Send> Send for OnceLock<T> {}

#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceLock<T> {}
#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: UnwindSafe> UnwindSafe for OnceLock<T> {}


// ... (other code) ...

}

#[stable(feature = "once_cell", since = "1.70.0")]
impl<T> From<T> for OnceLock<T> {
    /// Creates a new cell with its contents set to `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// # fn main() -> Result<(), i32> {
    /// let a = OnceLock::from(3);
    /// let b = OnceLock::new();
    /// b.set(3)?;
    /// assert_eq!(a, b);
    /// Ok(())
    /// # }
    /// ```
    #[inline]
    fn from(value: T) -> Self {
        let cell = Self::new();
        match cell.set(value) {
            Ok(()) => cell,
            Err(_) => unreachable!(),
        }
    }
}

// ... (other code) ...

impl<T: Eq> Eq for OnceLock<T> {}

#[stable(feature = "once_cell", since = "1.70.0")]
unsafe impl<#[may_dangle] T> Drop for OnceLock<T> {
    #[inline]
    fn drop(&mut self) {
        if self.is_initialized() {
            // SAFETY: The cell is initialized and being dropped, so it can't
            // be accessed again. We also don't touch the `T` other than
            // dropping it, which validates our usage of #[may_dangle].
            unsafe { (&mut *self.value.get()).assume_init_drop() };
        }
    }
}

```

**Entity:** OnceLock<T>

**States:** OutsideInitialization, InsideInitializer

**Transitions:**
- OutsideInitialization -> InsideInitializer via get_or_init()/get_or_try_init() executing `f` inside `initialize()`/`Once::call_once_force`
- InsideInitializer -> OutsideInitialization when `f` returns (Ok/Err) or panics

**Evidence:** doc on `get_or_init`: "It is an error to reentrantly initialize the cell from `f`. The exact outcome is unspecified. Current implementation deadlocks"; doc on `get_or_try_init`: same reentrancy warning; implementation: initialization happens inside `self.once.call_once_force(|p| { match f() { ... } })` which will deadlock if `f` tries to initialize the same `Once` again

**Implementation:** Introduce an internal, non-Clone `InitializingToken` (or borrow-based guard) created by `get_or_try_init` and passed to the closure: `get_or_try_init(|token| ...)`, where the token does not allow calling `get_or_try_init` on the same cell (e.g., by requiring `&OnceLock` methods that initialize to take `&mut token` or by splitting the API so initialization requires a unique capability). While full prevention is hard for arbitrary call graphs, a token can at least make direct reentrancy impossible and document/enforce the protocol in the signature.

---

### 43. ArrayToken stamp protocol (must match slot state machine expectations)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-12`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ArrayToken carries `stamp: usize` described as "Stamp to store into the slot after reading or writing." This implies a protocol with the slot’s internal state machine: the stamp is not arbitrary, but must be derived from the slot’s current stamp and the intended operation (read vs write), and then applied in the correct temporal order (perform operation on slot payload, then store the new stamp). The type system does not distinguish a read-token from a write-token, does not encode the expected stamp relationship, and does not prevent using a token with an unrelated slot pointer.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot; struct Channel, impl Channel < T > (17 methods)


/// The token type for the array flavor.
#[derive(Debug)]
pub(crate) struct ArrayToken {
    /// Slot to read from or write to.
    slot: *const u8,

    /// Stamp to store into the slot after reading or writing.
    stamp: usize,
}

```

**Entity:** ArrayToken

**States:** Stamp mismatched (token not usable for intended operation), Stamp matched (token usable to perform read/write and then update stamp)

**Transitions:**
- Stamp matched -> Stamp matched(next) via successful read/write followed by storing `stamp` into the slot

**Evidence:** field `stamp: usize` (encodes slot state machine progress as a plain integer); doc comment on `stamp`: "Stamp to store into the slot after reading or writing." (implies ordering and correctness requirements relative to the operation)

**Implementation:** Split token types by operation/state, e.g. `ArrayToken<WriteReady>` vs `ArrayToken<ReadReady>`; make constructors private and only produced by Channel methods that compute the correct next-stamp. Represent stamp as a newtype (e.g. `struct Stamp(usize)`) and/or encode operation in the type so a `ReadToken` cannot be used in a write path.

---

### 73. Channel send/recv token protocol + disconnected/connected state

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/list.rs:1-425`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Channel operations rely on an implicit multi-step protocol mediated by a mutable `Token`: (1) reserve a slot (`start_send`/`start_recv`), which encodes success vs disconnection by writing `token.list.block` (null/non-null) and `token.list.offset`; then (2) perform the unsafe action (`write`/`read`) which assumes the token is in the correct reserved state and corresponds to the right operation. Separately, the channel has an implicit Connected/Disconnected state encoded in `tail.index`'s `MARK_BIT`; methods interpret `MARK_BIT` to reject new sends and to make receives return disconnection when empty. None of these state distinctions are represented in the type system: `Token` can be reused or passed to the wrong method; `write`/`read` are `unsafe` and only runtime-check `token.list.block.is_null()`; and callers use `assert!(self.start_send(token))` rather than a type that makes failure/branching explicit.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot, impl Slot < T > (1 methods); struct Block, impl Block < T > (3 methods); struct Position; struct ListToken

///
/// Consecutive messages are grouped into blocks in order to put less pressure on the allocator and
/// improve cache efficiency.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    head: CachePadded<Position<T>>,

    /// The tail of the channel.
    tail: CachePadded<Position<T>>,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,

    /// Indicates that dropping a `Channel<T>` may drop messages of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Creates a new unbounded channel.
    pub(crate) fn new() -> Self {
        Channel {
            head: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            tail: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicUsize::new(0),
            }),
            receivers: SyncWaker::new(),
            _marker: PhantomData,
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        let mut block = self.tail.block.load(Ordering::Acquire);
        let mut next_block = None;

        loop {
            // Check if the channel is disconnected.
            if tail & MARK_BIT != 0 {
                token.list.block = ptr::null();
                return true;
            }

            // Calculate the offset of the index into the block.
            let offset = (tail >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                tail = self.tail.index.load(Ordering::Acquire);
                block = self.tail.block.load(Ordering::Acquire);
                continue;
            }

            // If we're going to have to install the next block, allocate it in advance in order to
            // make the wait for other threads as short as possible.
            if offset + 1 == BLOCK_CAP && next_block.is_none() {
                next_block = Some(Block::<T>::new());
            }

            // If this is the first message to be sent into the channel, we need to allocate the
            // first block and install it.
            if block.is_null() {
                let new = Box::into_raw(Block::<T>::new());

                if self
                    .tail
                    .block
                    .compare_exchange(block, new, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    // This yield point leaves the channel in a half-initialized state where the
                    // tail.block pointer is set but the head.block is not. This is used to
                    // facilitate the test in src/tools/miri/tests/pass/issues/issue-139553.rs
                    #[cfg(miri)]
                    crate::thread::yield_now();
                    self.head.block.store(new, Ordering::Release);
                    block = new;
                } else {
                    next_block = unsafe { Some(Box::from_raw(new)) };
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                    continue;
                }
            }

            let new_tail = tail + (1 << SHIFT);

            // Try advancing the tail forward.
            match self.tail.index.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, install the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next_block = Box::into_raw(next_block.unwrap());
                        self.tail.block.store(next_block, Ordering::Release);
                        self.tail.index.fetch_add(1 << SHIFT, Ordering::Release);
                        (*block).next.store(next_block, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.list.block.is_null() {
            return Err(msg);
        }

        // Write the message into the slot.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.state.fetch_or(WRITE, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.index.load(Ordering::Acquire);
        let mut block = self.head.block.load(Ordering::Acquire);

        loop {
            // Calculate the offset of the index into the block.
            let offset = (head >> SHIFT) % LAP;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            let mut new_head = head + (1 << SHIFT);

            if new_head & MARK_BIT == 0 {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.index.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if head >> SHIFT == tail >> SHIFT {
                    // If the channel is disconnected...
                    if tail & MARK_BIT != 0 {
                        // ...then receive an error.
                        token.list.block = ptr::null();
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                // If head and tail are not in the same block, set `MARK_BIT` in head.
                if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                    new_head |= MARK_BIT;
                }
            }

            // The block can be null here only if the first message is being sent into the channel.
            // In that case, just wait until it gets initialized.
            if block.is_null() {
                backoff.spin_heavy();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            // Try moving the head index forward.
            match self.head.index.compare_exchange_weak(
                head,
                new_head,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, move to the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next = (*block).wait_next();
                        let mut next_index = (new_head & !MARK_BIT).wrapping_add(1 << SHIFT);
                        if !(*next).next.load(Ordering::Relaxed).is_null() {
                            next_index |= MARK_BIT;
                        }

                        self.head.block.store(next, Ordering::Release);
                        self.head.index.store(next_index, Ordering::Release);
                    }

                    token.list.block = block as *const u8;
                    token.list.offset = offset;
                    return true;
                },
                Err(_) => {
                    backoff.spin_light();
                    head = self.head.index.load(Ordering::Acquire);
                    block = self.head.block.load(Ordering::Acquire);
                }
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.list.block.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message.
        let block = token.list.block as *mut Block<T>;
        let offset = token.list.offset;
        unsafe {
            let slot = (*block).slots.get_unchecked(offset);
            slot.wait_write();
            let msg = slot.msg.get().read().assume_init();

            // Destroy the block if we've reached the end, or if another thread wanted to destroy but
            // couldn't because we were busy reading from the slot.
            if offset + 1 == BLOCK_CAP {
                Block::destroy(block, 0);
            } else if slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY != 0 {
                Block::destroy(block, offset + 1);
            }

            Ok(msg)
        }
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.send(msg, None).map_err(|err| match err {
            SendTimeoutError::Disconnected(msg) => TrySendError::Disconnected(msg),
            SendTimeoutError::Timeout(_) => unreachable!(),
        })
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        _deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        assert!(self.start_send(token));
        unsafe { self.write(token, msg).map_err(SendTimeoutError::Disconnected) }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            if self.start_recv(token) {
                unsafe {
                    return self.read(token).map_err(|_| RecvTimeoutError::Disconnected);
                }
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            // Prepare for blocking until a sender wakes us up.
            Context::with(|cx| {
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail index, then load the head index.
            let mut tail = self.tail.index.load(Ordering::SeqCst);
            let mut head = self.head.index.load(Ordering::SeqCst);

            // If the tail index didn't change, we've got consistent indices to work with.
            if self.tail.index.load(Ordering::SeqCst) == tail {
                // Erase the lower bits.
                tail &= !((1 << SHIFT) - 1);
                head &= !((1 << SHIFT) - 1);

                // Fix up indices if they fall onto block ends.
                if (tail >> SHIFT) & (LAP - 1) == LAP - 1 {
                    tail = tail.wrapping_add(1 << SHIFT);
                }
                if (head >> SHIFT) & (LAP - 1) == LAP - 1 {
                    head = head.wrapping_add(1 << SHIFT);
                }

                // Rotate indices so that head falls into the first block.
                let lap = (head >> SHIFT) / LAP;
                tail = tail.wrapping_sub((lap * LAP) << SHIFT);
                head = head.wrapping_sub((lap * LAP) << SHIFT);

                // Remove the lower bits.
                tail >>= SHIFT;
                head >>= SHIFT;

                // Return the difference minus the number of blocks between tail and head.
                return tail - head - tail / LAP;
            }
        }
    }

    /// Returns the capacity of the channel.
    pub(crate) fn capacity(&self) -> Option<usize> {
        None
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst);

        if tail & MARK_BIT == 0 {
            // If receivers are dropped first, discard all messages to free
            // memory eagerly.
            self.discard_all_messages();
            true
        } else {
            false
        }
    }

    /// Discards all messages.
    ///
    /// This method should only be called when all receivers are dropped.
    fn discard_all_messages(&self) {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        loop {
            let offset = (tail >> SHIFT) % LAP;
            if offset != BLOCK_CAP {
                break;
            }

            // New updates to tail will be rejected by MARK_BIT and aborted unless it's
            // at boundary. We need to wait for the updates take affect otherwise there
            // can be memory leaks.
            backoff.spin_heavy();
            tail = self.tail.index.load(Ordering::Acquire);
        }

        let mut head = self.head.index.load(Ordering::Acquire);
        // The channel may be uninitialized, so we have to swap to avoid overwriting any sender's attempts
        // to initialize the first block before noticing that the receivers disconnected. Late allocations
        // will be deall
// ... (truncated) ...
```

**Entity:** Channel<T>

**States:** Connected, Disconnected, SendSlotReserved(token valid for write), RecvSlotReserved(token valid for read)

**Transitions:**
- Connected -> Disconnected via disconnect_senders() or disconnect_receivers() (tail.index |= MARK_BIT)
- Connected -> SendSlotReserved via start_send(&mut Token) (sets token.list.{block,offset})
- SendSlotReserved -> Connected (slot committed) via unsafe write(&mut Token, msg)
- Connected -> RecvSlotReserved via start_recv(&mut Token) when message available
- Connected -> (not ready) via start_recv(&mut Token) returning false when empty and not disconnected
- RecvSlotReserved -> Connected (slot consumed) via unsafe read(&mut Token)

**Evidence:** start_send(): `if tail & MARK_BIT != 0 { token.list.block = ptr::null(); return true; }` uses null in token to encode disconnection; write(): `pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T>` and `if token.list.block.is_null() { return Err(msg); }` shows precondition is a prior successful start_send; send(): `assert!(self.start_send(token)); unsafe { self.write(token, msg) ... }` relies on ordering start_send() then write(); start_recv(): returns `false` for empty-but-connected (`return false;`) and sets `token.list.block = ptr::null()` for disconnected-empty case; read(): `pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()>` and `if token.list.block.is_null() { return Err(()); }` shows precondition is a prior successful start_recv; disconnect_senders()/disconnect_receivers(): both do `self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst)`; disconnection is encoded in a bit, not a type-state

**Implementation:** Replace the ad-hoc `Token` protocol with typed reservation objects: e.g. `fn reserve_send(&self) -> Result<SendPermit<'_, T>, Disconnected>` and `impl SendPermit { fn write(self, msg: T) }`; similarly `reserve_recv(&self) -> Result<RecvPermit<'_, T>, EmptyOrDisconnected>`, where `RecvPermit::read(self) -> T`. Encode disconnection in return types rather than `token.list.block = null`. Internally these permits can still store raw pointers/offsets, but the type system prevents calling `read` with a send permit (or an uninitialized token) and prevents skipping the reserve step.

---

### 15. Selected <-> usize encoding protocol (reserved tags + Operation range)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/select.rs:1-42`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Selected is encoded into a usize with reserved discriminants: 0=Waiting, 1=Aborted, 2=Disconnected, and any other value is interpreted as an Operation id. This creates an implicit protocol/invariant that Operation values used inside Selected::Operation must never be 0, 1, or 2, otherwise encoding/decoding would lose information (e.g., Selected::Operation(Operation(1)) would round-trip to Aborted). The type system does not prevent constructing Operation with reserved values (Operation wraps a usize), nor does it provide a fallible decoding path for invalid/reserved Operation ids.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Token; struct Operation, impl Operation (1 methods)


/// Current state of a blocking operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    /// Still waiting for an operation.
    Waiting,

    /// The attempt to block the current thread has been aborted.
    Aborted,

    /// An operation became ready because a channel is disconnected.
    Disconnected,

    /// An operation became ready because a message can be sent or received.
    Operation(Operation),
}

impl From<usize> for Selected {
    #[inline]
    fn from(val: usize) -> Selected {
        match val {
            0 => Selected::Waiting,
            1 => Selected::Aborted,
            2 => Selected::Disconnected,
            oper => Selected::Operation(Operation(oper)),
        }
    }
}

impl Into<usize> for Selected {
    #[inline]
    fn into(self) -> usize {
        match self {
            Selected::Waiting => 0,
            Selected::Aborted => 1,
            Selected::Disconnected => 2,
            Selected::Operation(Operation(val)) => val,
        }
    }
}

```

**Entity:** Selected (and its usize encoding via From<usize>/Into<usize>)

**States:** Waiting, Aborted, Disconnected, Operation

**Transitions:**
- Selected -> usize via Into<usize>
- usize -> Selected via From<usize>

**Evidence:** impl From<usize> for Selected: match val { 0 => Waiting, 1 => Aborted, 2 => Disconnected, oper => Operation(Operation(oper)) }; impl Into<usize> for Selected: Selected::Operation(Operation(val)) => val (no check that val > 2); enum Selected variants use Operation(Operation) but Operation is constructed from a raw usize in From<usize>

**Implementation:** Introduce a validated newtype for non-reserved operation ids, e.g. `struct OperationId(NonZeroUsize)` plus an offset (store val-2) or `struct EncodedSelected(usize)` that is the only way to serialize/deserialize. Make decoding fallible: `TryFrom<usize> for Selected` or `TryFrom<usize> for OperationId` to reject reserved/invalid values, and ensure `Selected::Operation` can only be constructed with `OperationId` (not a raw usize).

---

### 76. Receiver iteration consumption protocol (borrowed vs owning iteration) and termination-on-hangup

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-521`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: There is an implicit protocol around iteration: `Receiver::iter()`/`IntoIterator for &Receiver` create a borrowed iterator that blocks waiting for messages and ends with `None` when the channel hangs up; `Receiver::try_iter()` creates a non-blocking iterator that ends with `None` when empty or hung up; `IntoIterator for Receiver` consumes the receiver into `IntoIter { rx: self }`, after which the original `Receiver` is no longer usable. The end condition (hangup/disconnect) and the blocking vs non-blocking behavior are only described in docs and by which iterator type you chose; the type system does not encode the 'blocking' vs 'non-blocking' mode as a capability, nor does it distinguish 'still connected' vs 'hung up' beyond returning `None`/errors at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Sender, 2 free function(s), impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl UnwindSafe for Sender < T > (0 methods), impl RefUnwindSafe for Sender < T > (0 methods), impl Sender < T > (2 methods), impl Sender < T > (7 methods), impl Drop for Sender < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// rx_thread_2.join().unwrap();
/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
pub struct Receiver<T> {
    flavor: ReceiverFlavor<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { rx: self }
    }
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Send for Receiver<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Sync for Receiver<T> {}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> UnwindSafe for Receiver<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> RefUnwindSafe for Receiver<T> {}

impl<T> Receiver<T> {
    /// Attempts to receive a message from the channel without blocking.
    ///
    /// This method will never block the caller in order to wait for data to
    /// become available. Instead, this will always return immediately with a
    /// possible option of pending data on the channel.
    ///
    /// If called on a zero-capacity channel, this method will receive a message only if there
    /// happens to be a send operation on the other side of the channel at the same time.
    ///
    /// This is useful for a flavor of "optimistic check" before deciding to
    /// block on a receiver.
    ///
    /// Compared with [`recv`], this function has two failure cases instead of one
    /// (one for disconnection, one for an empty buffer).
    ///
    /// [`recv`]: Self::recv
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::{Receiver, channel};
    ///
    /// let (_, receiver): (_, Receiver<i32>) = channel();
    ///
    /// assert!(receiver.try_recv().is_err());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.try_recv(),
            ReceiverFlavor::List(chan) => chan.try_recv(),
            ReceiverFlavor::Zero(chan) => chan.try_recv(),
        }
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent (at least one sender
    /// still exists). Once a message is sent to the corresponding [`Sender`],
    /// this receiver will wake up and return that message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpmc::channel();
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(Ok(1), recv.recv());
    /// ```
    ///
    /// Buffering behavior:
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    /// use std::sync::mpmc::RecvError;
    ///
    /// let (send, recv) = mpmc::channel();
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    ///     send.send(2).unwrap();
    ///     send.send(3).unwrap();
    ///     drop(send);
    /// });
    ///
    /// // wait for the thread to join so we ensure the sender is dropped
    /// handle.join().unwrap();
    ///
    /// assert_eq!(Ok(1), recv.recv());
    /// assert_eq!(Ok(2), recv.recv());
    /// assert_eq!(Ok(3), recv.recv());
    /// assert_eq!(Err(RecvError), recv.recv());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn recv(&self) -> Result<T, RecvError> {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.recv(None),
            ReceiverFlavor::List(chan) => chan.recv(None),
            ReceiverFlavor::Zero(chan) => chan.recv(None),
        }
        .map_err(|_| RecvError)
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up, or if it waits more than `timeout`.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent (at least one sender
    /// still exists). Once a message is sent to the corresponding [`Sender`],
    /// this receiver will wake up and return that message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// Successfully receiving value before encountering timeout:
    ///
    /// ```no_run
    /// #![feature(mpmc_channel)]
    ///
    /// use std::thread;
    /// use std::time::Duration;
    /// use std::sync::mpmc;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// thread::spawn(move || {
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_timeout(Duration::from_millis(400)),
    ///     Ok('a')
    /// );
    /// ```
    ///
    /// Receiving an error upon reaching timeout:
    ///
    /// ```no_run
    /// #![feature(mpmc_channel)]
    ///
    /// use std::thread;
    /// use std::time::Duration;
    /// use std::sync::mpmc;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(800));
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_timeout(Duration::from_millis(400)),
    ///     Err(mpmc::RecvTimeoutError::Timeout)
    /// );
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        match Instant::now().checked_add(timeout) {
            Some(deadline) => self.recv_deadline(deadline),
            // So far in the future that it's practically the same as waiting indefinitely.
            None => self.recv().map_err(RecvTimeoutError::from),
        }
    }

    /// Attempts to wait for a value on this receiver, returning an error if the
    /// corresponding channel has hung up, or if `deadline` is reached.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent. Once a message is
    /// sent to the corresponding [`Sender`], then this receiver will wake up
    /// and return that message.
    ///
    /// If the corresponding [`Sender`] has disconnected, or it disconnects while
    /// this call is blocking, this call will wake up and return [`Err`] to
    /// indicate that no more messages can ever be received on this channel.
    /// However, since channels are buffered, messages sent before the disconnect
    /// will still be properly received.
    ///
    /// # Examples
    ///
    /// Successfully receiving value before reaching deadline:
    ///
    /// ```no_run
    /// #![feature(mpmc_channel)]
    ///
    /// use std::thread;
    /// use std::time::{Duration, Instant};
    /// use std::sync::mpmc;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// thread::spawn(move || {
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_deadline(Instant::now() + Duration::from_millis(400)),
    ///     Ok('a')
    /// );
    /// ```
    ///
    /// Receiving an error upon reaching deadline:
    ///
    /// ```no_run
    /// #![feature(mpmc_channel)]
    ///
    /// use std::thread;
    /// use std::time::{Duration, Instant};
    /// use std::sync::mpmc;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_millis(800));
    ///     send.send('a').unwrap();
    /// });
    ///
    /// assert_eq!(
    ///     recv.recv_deadline(Instant::now() + Duration::from_millis(400)),
    ///     Err(mpmc::RecvTimeoutError::Timeout)
    /// );
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn recv_deadline(&self, deadline: Instant) -> Result<T, RecvTimeoutError> {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.recv(Some(deadline)),
            ReceiverFlavor::List(chan) => chan.recv(Some(deadline)),
            ReceiverFlavor::Zero(chan) => chan.recv(Some(deadline)),
        }
    }

    /// Returns an iterator that will attempt to yield all pending values.
    /// It will return `None` if there are no more pending values or if the
    /// channel has hung up. The iterator will never [`panic!`] or block the
    /// user by waiting for values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let (sender, receiver) = channel();
    ///
    /// // nothing is in the buffer yet
    /// assert!(receiver.try_iter().next().is_none());
    ///
    /// thread::spawn(move || {
    ///     thread::sleep(Duration::from_secs(1));
    ///     sender.send(1).unwrap();
    ///     sender.send(2).unwrap();
    ///     sender.send(3).unwrap();
    /// });
    ///
    /// // nothing is in the buffer yet
    /// assert!(receiver.try_iter().next().is_none());
    ///
    /// // block for two seconds
    /// thread::sleep(Duration::from_secs(2));
    ///
    /// let mut iter = receiver.try_iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn try_iter(&self) -> TryIter<'_, T> {
        TryIter { rx: self }
    }
}

impl<T> Receiver<T> {
    /// Returns `true` if the channel is empty.
    ///
    /// Note: Zero-capacity channels are always empty.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// assert!(recv.is_empty());
    ///
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(!recv.is_empty());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_empty(&self) -> bool {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.is_empty(),
            ReceiverFlavor::List(chan) => chan.is_empty(),
            ReceiverFlavor::Zero(chan) => chan.is_empty(),
        }
    }

    /// Returns `true` if the channel is full.
    ///
    /// Note: Zero-capacity channels are always full.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpmc::sync_channel(1);
    ///
    /// assert!(!recv.is_full());
    ///
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(recv.is_full());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_full(&self) -> bool {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.is_full(),
            ReceiverFlavor::List(chan) => chan.is_full(),
            ReceiverFlavor::Zero(chan) => chan.is_full(),
        }
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpmc::channel();
    ///
    /// assert_eq!(recv.len(), 0);
    ///
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(recv.len(), 1);
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn len(&self) -> usize {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.len(),
            ReceiverFlavor::List(chan) => chan.len(),
            ReceiverFlavor::Zero(chan) => chan.len(),
        }
    }

    /// If the channel is bounded, returns its capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, recv) = mpmc::sync_channel(3);
    ///
    /// assert_eq!(recv.capacity(), Some(3));
    ///
    /// let handle = thread::spawn(move || {
    ///     send.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(recv.capacity(), Some(3));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn capacity(&self) -> Option<usize> {
        match &self.flavor {
            ReceiverFlavor::Array(chan) => chan.capacity(),
            ReceiverFlavor::List(chan) => chan.capacity(),
            ReceiverFlavor::Zero(chan) => chan.capacity(),
        }
    }

    /// Returns `true` if receivers belong to the same channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    ///
    /// let (_, rx1) = mpmc::channel::<i32>();
    /// let (_, rx2) = mpmc::channel::<i32>();
    ///
    /// assert!(rx1.same_channel(&rx1));
    /// assert!(!rx1.same_channel(&rx2));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn same_channel(&self, other: &Receiver<T>) -> bool {
        match (&self.flavor, &other.flavor) {
            (ReceiverFlavor::Array(a), ReceiverFlavor::Array(b)) => a == b,
            (ReceiverFlavor::List(a), ReceiverFlavor::List(b)) => a == b,
            (ReceiverFlavor::Zero(a), ReceiverFlavor::Zero(b)) => a == b,
            _ => false,
        }
    }

    /// Returns an iterator that will block waiting for messages, but never
    /// [`panic!`]. It will return [`None`] when the channel has hung up.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::thread;
    ///
    /// let (send, recv) = channel();
    ///
    /// thread::spawn(move || {
    ///     send.send(1).unwrap();
    ///     send.send(2).unwrap();
    ///     send.send(3).unwrap();
    /// });
    ///
    /// let mut iter = recv.iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { rx: self }
    }
}

#[unstable(feature = "mpmc_cha
// ... (truncated) ...
```

**Entity:** Iter<'a, T> / TryIter<'a, T> / IntoIter<T>

**States:** Borrowed iteration (non-consuming), Owned iteration (consuming), Terminated (channel hung up / no more values)

**Transitions:**
- Receiver (borrowed) -> Borrowed iteration via `Receiver::iter()` / `IntoIterator for &Receiver`
- Receiver (borrowed) -> Borrowed nonblocking iteration via `Receiver::try_iter()`
- Receiver (owned) -> Owned iteration via `IntoIterator for Receiver`
- Any iterator -> Terminated when channel hung up (or additionally for TryIter: when currently empty)

**Evidence:** impl: `IntoIterator for &'a Receiver<T>` returns `Iter<'a, T>` via `self.iter()`; impl: `IntoIterator for Receiver<T>` returns `IntoIter<T>` with `IntoIter { rx: self }` (receiver is moved/consumed); method: `iter(&self) -> Iter<'_, T> { Iter { rx: self } }` constructs borrowed iterator tied to `&self`; method: `try_iter(&self) -> TryIter<'_, T> { TryIter { rx: self } }` constructs non-blocking iterator; doc: `iter`: "will block waiting for messages ... will return None when the channel has hung up"; doc: `try_iter`: "will return None if there are no more pending values or if the channel has hung up" and "will never panic or block"

**Implementation:** Introduce distinct capability marker types for iteration modes, e.g. `Blocking` vs `NonBlocking`, and/or `Receiver<T, F>` as above plus `Iter<'a, T, Mode>`. `Receiver::iter()` returns `Iter<Blocking>` and `try_iter()` returns `Iter<NonBlocking>`. This makes it impossible to accidentally pass a blocking iterator into code expecting non-blocking semantics (or vice versa) without explicit conversion, and makes the mode visible in type signatures.

---

### 41. Inner rendezvous protocol for cross-thread packet handoff (Empty/HasPacket + Unselected/Selected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/context.rs:1-18`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: `Inner` encodes a two-dimensional concurrent protocol via atomics: (1) whether an operation has been selected (`select: Atomic<usize>`), and (2) whether a waiting thread has published a packet pointer for another thread to fill/consume (`packet: Atomic<*mut ()>`). This implies temporal/ordering requirements (publish packet before unpark; select exactly once; packet pointer must be valid while visible) that are not enforced by the type system. The raw pointer `*mut ()` also implies an implicit validity/lifetime invariant (non-null means 'a live Packet exists elsewhere' and must not dangle), but the type system cannot track that the pointed-to allocation outlives the period it is stored in `packet` or that it is only used by the intended peer.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Context, impl Context (8 methods)


/// Inner representation of `Context`.
#[derive(Debug)]
struct Inner {
    /// Selected operation.
    select: Atomic<usize>,

    /// A slot into which another thread may store a pointer to its `Packet`.
    packet: Atomic<*mut ()>,

    /// Thread handle.
    thread: Thread,

    /// Thread id.
    thread_id: usize,
}

```

**Entity:** Inner

**States:** Unselected + Empty, Unselected + HasPacket, Selected + Empty, Selected + HasPacket

**Transitions:**
- Unselected -> Selected via storing a value into `select` (operation chosen)
- Empty -> HasPacket via storing a non-null pointer into `packet` (publish Packet address)
- HasPacket -> Empty via clearing/swapping `packet` back to null (handoff complete)

**Evidence:** `select: Atomic<usize>` field comment: "Selected operation." (runtime state encoded as integer/atomic); `packet: Atomic<*mut ()>` field comment: "A slot into which another thread may store a pointer to its `Packet`." (cross-thread handoff protocol + raw pointer lifetime invariant); `packet` uses `*mut ()` (untyped raw pointer) indicating the invariant is maintained by convention/unsafe code rather than types

**Implementation:** Replace the raw integer/pointer state machine with typed states and a typed slot: e.g., `Inner<S, P>` where `S` is `Unselected/Selected` and `P` is `Empty/HasPacket`. For the packet slot, use a typed pointer wrapper carrying lifetime/capability, e.g. `PacketSlot<'a>(NonNull<Packet>)` or a token-based capability that can only be constructed when a `Packet` is guaranteed to live long enough. Expose only state-valid methods on the corresponding `Inner<...>` instantiations (e.g., `publish_packet(self, PacketToken<'a>) -> Inner<..., HasPacket<'a>>`, `select(self, Op) -> Inner<Selected, ...>`), keeping the atomics internal if needed for concurrency but preventing misuse at the API boundary.

---

### 34. Receiver borrow-iteration protocol (BorrowedReceiver -> Exhausted/Disconnected)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Iter is an iterator view over a borrowed Receiver. Each next() call performs a blocking recv() and maps any RecvError (e.g., channel disconnected) to None via ok(). This implicitly encodes a protocol/state machine where iteration continues while messages arrive, and terminates once the channel is exhausted/disconnected. The type system does not distinguish a receiver that is still able to yield items from one that is already disconnected/exhausted; termination is detected only at runtime via recv() returning Err.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct SendError, impl error :: Error for SendError < T > (1 methods), impl error :: Error for TrySendError < T > (1 methods), impl From < SendError < T > > for TrySendError < T > (1 methods); struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[stable(feature = "receiver_try_iter", since = "1.15.0")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** Iter<'a, T>

**States:** BorrowedReceiver, ExhaustedOrDisconnected

**Transitions:**
- BorrowedReceiver -> ExhaustedOrDisconnected via Iter::next() calling Receiver::recv() and observing Err (mapped to None)

**Evidence:** struct Iter<'a, T> { rx: &'a Receiver<T> } — iterator is tied to a borrowed Receiver; impl Iterator for Iter: next(): self.rx.recv().ok() — blocking receive; errors are converted to None, implicitly signaling terminal state

**Implementation:** Expose an explicit terminal-state iterator type or typestate on the iterator (e.g., Iter<Alive> -> Iter<Done>) where next() on Done is not available, or return a richer enum (e.g., Yield(T) | Disconnected) instead of Option to avoid collapsing 'temporarily no item' vs 'permanently done' (though recv() is blocking, the key latent invariant is permanent termination on disconnect).

---

### 48. Operation identifier protocol (unique, thread-local, long-lived, and non-sentinel)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/select.rs:1-22`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Operation is created by encoding a mutable reference's address as a usize. This relies on an implicit protocol: the referenced variable must be specific to the thread+operation, must remain alive for the entire duration of a blocking operation, and the resulting numeric value must not collide with reserved sentinel values used by `Selected::{Waiting, Aborted, Disconnected}` (apparently encoded as 0..=2). None of these properties are enforced by the type system: callers can pass a short-lived stack temporary (leading to reuse/ABA-like issues), share the same hook value across operations/threads, or (in principle) produce a value conflicting with sentinel encodings; the code only asserts the non-sentinel constraint at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Token; enum Selected, impl From < usize > for Selected (1 methods), impl Into < usize > for Selected (1 methods)


/// Identifier associated with an operation by a specific thread on a specific channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation(usize);

impl Operation {
    /// Creates an operation identifier from a mutable reference.
    ///
    /// This function essentially just turns the address of the reference into a number. The
    /// reference should point to a variable that is specific to the thread and the operation,
    /// and is alive for the entire duration of a blocking operation.
    #[inline]
    pub fn hook<T>(r: &mut T) -> Operation {
        let val = r as *mut T as usize;
        // Make sure that the pointer address doesn't equal the numerical representation of
        // `Selected::{Waiting, Aborted, Disconnected}`.
        assert!(val > 2);
        Operation(val)
    }
}

```

**Entity:** Operation

**States:** Invalid (violates requirements), Valid (usable as operation id)

**Transitions:**
- Invalid -> Valid via Operation::hook(&mut T) when lifetime/uniqueness/non-sentinel constraints are satisfied

**Evidence:** pub struct Operation(usize); — raw numeric id with no provenance/lifetime information; Operation::hook<T>(r: &mut T) -> Operation converts `r` to `*mut T as usize` (address-as-id encoding); comment in hook(): "reference should point to a variable that is specific to the thread and the operation, and is alive for the entire duration of a blocking operation" (implicit temporal/lifetime requirement); assert!(val > 2); and comment: "Make sure that the pointer address doesn't equal the numerical representation of `Selected::{Waiting, Aborted, Disconnected}`" (reserved sentinel protocol enforced only at runtime)

**Implementation:** Make Operation carry provenance to encode the 'hook must live long enough' requirement, e.g. `pub struct Operation<'a>(NonZeroUsize, PhantomData<&'a mut ()>);` with `fn hook<T>(r: &'a mut T) -> Operation<'a>` so the operation id cannot outlive the hooked storage. Also use `NonZeroUsize` + a dedicated `TryFrom<usize>`/constructor that rejects reserved sentinel values at construction time, preventing accidental creation from arbitrary usize.

---

### 54. MutexGuard lock-ownership + poison protocol (Held/Released; Clean/Poisoned)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/mutex.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: MutexGuard represents an acquired mutex lock plus an associated poisoning guard. While the Rust borrow checker enforces that the guard cannot outlive the borrowed Mutex (via `lock: &'a Mutex<T>`), it does not encode (in the type) the full protocol around poisoning: a guard participates in tracking whether a panic occurred while the lock was held, and on drop it updates poison state. The distinction between a 'clean' lock/unpoisoned mutex and a 'poisoned' mutex is tracked dynamically via `poison: poison::Guard` and other module logic (not shown here). As a result, callers must handle poisoning at runtime (typically via `LockResult`/`PoisonError` elsewhere), rather than being able to statically distinguish a guard that is known-clean vs one obtained from a previously-poisoned mutex.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Mutex, 2 free function(s), impl Send for Mutex < T > (0 methods), impl Sync for Mutex < T > (0 methods), impl Send for MutexGuard < '_ , T > (0 methods), impl Sync for MutexGuard < '_ , T > (0 methods), impl Send for MappedMutexGuard < '_ , T > (0 methods), impl Sync for MappedMutexGuard < '_ , T > (0 methods), impl Mutex < T > (4 methods), impl Mutex < T > (6 methods), impl From < T > for Mutex < T > (1 methods), impl MutexGuard < 'mutex , T > (1 methods), impl Deref for MutexGuard < '_ , T > (1 methods), impl DerefMut for MutexGuard < '_ , T > (1 methods), impl Drop for MutexGuard < '_ , T > (1 methods), impl MutexGuard < 'a , T > (2 methods), impl Deref for MappedMutexGuard < '_ , T > (1 methods), impl DerefMut for MappedMutexGuard < '_ , T > (1 methods), impl Drop for MappedMutexGuard < '_ , T > (1 methods), impl MappedMutexGuard < 'a , T > (2 methods); struct MappedMutexGuard

#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "MutexGuard")]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    poison: poison::Guard,
}

```

**Entity:** MutexGuard<'a, T>

**States:** LockHeld+PoisonTrackingActive, Released, Poisoned(associated-with-mutex)

**Transitions:**
- LockHeld+PoisonTrackingActive -> Released via Drop for MutexGuard (implied by `#[clippy::has_significant_drop]` and `poison: poison::Guard` field)
- LockHeld+PoisonTrackingActive -> Poisoned(associated-with-mutex) via panic/unwind while guard is held (tracked by `poison: poison::Guard` at runtime)

**Evidence:** struct MutexGuard<'a, T> { lock: &'a Mutex<T>, poison: poison::Guard } — presence of `poison: poison::Guard` indicates a runtime poison-tracking state coupled to the guard; #[clippy::has_significant_drop] attribute on MutexGuard — signals Drop has meaningful side effects (unlocking / poison bookkeeping), i.e., a lifecycle protocol; field `lock: &'a Mutex<T>` — indicates the guard is only valid while it holds a borrow to the Mutex, but does not encode poison-cleanliness in its type

**Implementation:** Introduce a state parameter for the poison condition, e.g. `MutexGuard<'a, T, P>` where `P` is `Clean` or `Poisoned`. `Mutex::lock()` could return `Result<MutexGuard<'a, T, Clean>, PoisonError<MutexGuard<'a, T, Poisoned>>>` (or similar) so downstream APIs can require `Clean` guards where appropriate. The Drop impl would remain, but the poison-related state would be reflected at the type level instead of only through runtime error paths.

---

### 1. Waker registration lifecycle (Registered -> Unregistered/Selected) with packet-ownership protocol

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-175`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Waker maintains two runtime queues of waiting operations (selectors and observers). Correctness relies on an implicit lifecycle: threads register an operation (optionally with an associated raw packet pointer), later either (a) the entry is removed when another thread successfully selects it (try_select removes from selectors), or (b) the owning thread unregisters it, especially after disconnect() where entries are intentionally not removed. Additionally, register_with_packet encodes a protocol that the packet pointer is either null (no packet) or a valid pointer that the waiting thread will later retrieve/destroy after being woken; this ownership/validity is not represented in the type system (uses *mut ()). The Drop impl asserts both queues are empty, implying a global invariant that all registrations must be cleared before Waker is dropped; this is only a debug_assert, not enforced by types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Entry; struct SyncWaker; 1 free function(s)

///
/// This data structure is used by threads to register blocking operations and get woken up once
/// an operation becomes ready.
pub(crate) struct Waker {
    /// A list of select operations.
    selectors: Vec<Entry>,

    /// A list of operations waiting to be ready.
    observers: Vec<Entry>,
}

impl Waker {
    /// Creates a new `Waker`.
    #[inline]
    pub(crate) fn new() -> Self {
        Waker { selectors: Vec::new(), observers: Vec::new() }
    }

    /// Registers a select operation.
    #[inline]
    pub(crate) fn register(&mut self, oper: Operation, cx: &Context) {
        self.register_with_packet(oper, ptr::null_mut(), cx);
    }

    /// Registers a select operation and a packet.
    #[inline]
    pub(crate) fn register_with_packet(&mut self, oper: Operation, packet: *mut (), cx: &Context) {
        self.selectors.push(Entry { oper, packet, cx: cx.clone() });
    }

    /// Unregisters a select operation.
    #[inline]
    pub(crate) fn unregister(&mut self, oper: Operation) -> Option<Entry> {
        if let Some((i, _)) =
            self.selectors.iter().enumerate().find(|&(_, entry)| entry.oper == oper)
        {
            let entry = self.selectors.remove(i);
            Some(entry)
        } else {
            None
        }
    }

    /// Attempts to find another thread's entry, select the operation, and wake it up.
    #[inline]
    pub(crate) fn try_select(&mut self) -> Option<Entry> {
        if self.selectors.is_empty() {
            None
        } else {
            let thread_id = current_thread_id();

            self.selectors
                .iter()
                .position(|selector| {
                    // Does the entry belong to a different thread?
                    selector.cx.thread_id() != thread_id
                        && selector // Try selecting this operation.
                            .cx
                            .try_select(Selected::Operation(selector.oper))
                            .is_ok()
                        && {
                            // Provide the packet.
                            selector.cx.store_packet(selector.packet);
                            // Wake the thread up.
                            selector.cx.unpark();
                            true
                        }
                })
                // Remove the entry from the queue to keep it clean and improve
                // performance.
                .map(|pos| self.selectors.remove(pos))
        }
    }

    /// Notifies all operations waiting to be ready.
    #[inline]
    pub(crate) fn notify(&mut self) {
        for entry in self.observers.drain(..) {
            if entry.cx.try_select(Selected::Operation(entry.oper)).is_ok() {
                entry.cx.unpark();
            }
        }
    }

    /// Notifies all registered operations that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&mut self) {
        for entry in self.selectors.iter() {
            if entry.cx.try_select(Selected::Disconnected).is_ok() {
                // Wake the thread up.
                //
                // Here we don't remove the entry from the queue. Registered threads must
                // unregister from the waker by themselves. They might also want to recover the
                // packet value and destroy it, if necessary.
                entry.cx.unpark();
            }
        }

        self.notify();
    }
}

impl Drop for Waker {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(self.selectors.len(), 0);
        debug_assert_eq!(self.observers.len(), 0);
    }
}

// ... (other code) ...

    is_empty: Atomic<bool>,
}

impl SyncWaker {
    /// Creates a new `SyncWaker`.
    #[inline]
    pub(crate) fn new() -> Self {
        SyncWaker { inner: Mutex::new(Waker::new()), is_empty: AtomicBool::new(true) }
    }

    /// Registers the current thread with an operation.
    #[inline]
    pub(crate) fn register(&self, oper: Operation, cx: &Context) {
        let mut inner = self.inner.lock().unwrap();
        inner.register(oper, cx);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }

    /// Unregisters an operation previously registered by the current thread.
    #[inline]
    pub(crate) fn unregister(&self, oper: Operation) -> Option<Entry> {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.unregister(oper);
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
        entry
    }

    /// Attempts to find one thread (not the current one), select its operation, and wake it up.
    #[inline]
    pub(crate) fn notify(&self) {
        if !self.is_empty.load(Ordering::SeqCst) {
            let mut inner = self.inner.lock().unwrap();
            if !self.is_empty.load(Ordering::SeqCst) {
                inner.try_select();
                inner.notify();
                self.is_empty.store(
                    inner.selectors.is_empty() && inner.observers.is_empty(),
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Notifies all threads that the channel is disconnected.
    #[inline]
    pub(crate) fn disconnect(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.disconnect();
        self.is_empty
            .store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst);
    }
}

impl Drop for SyncWaker {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.is_empty.load(Ordering::SeqCst));
    }
}

```

**Entity:** Waker

**States:** Empty, HasSelectors, HasObservers, DisconnectedNotified

**Transitions:**
- Empty -> HasSelectors via register()/register_with_packet() (pushes into selectors)
- HasSelectors -> HasSelectors (minus one) via try_select() (removes selected entry)
- HasSelectors -> HasSelectors (minus one) via unregister() (removes matching entry)
- HasObservers -> Empty via notify() (drain observers)
- Any -> DisconnectedNotified via disconnect() (selects Disconnected but does not remove selectors; requires later unregister by owners)

**Evidence:** field: selectors: Vec<Entry> and observers: Vec<Entry> encode runtime waiting sets; method: register_with_packet() pushes Entry { oper, packet, cx: cx.clone() } (registration step); method: unregister() searches selectors and removes entry (explicit unregistration step); method: try_select() removes an entry from selectors only after try_select(...).is_ok(), then calls store_packet(packet) and unpark() (selection/wake protocol + packet handoff); method: disconnect() comment: "Here we don't remove the entry from the queue. Registered threads must unregister... They might also want to recover the packet value and destroy it" (implicit post-disconnect responsibility + packet lifetime); impl Drop for Waker: debug_assert_eq!(self.selectors.len(), 0) and debug_assert_eq!(self.observers.len(), 0) (must be empty before drop)

**Implementation:** Introduce typed registration tokens: e.g., Waker::register(...) -> RegistrationHandle<'a, WithPacket<T>|NoPacket> that (1) unregisters on Drop (RAII) or is consumed by a take_unreg(self) method, and (2) stores a typed NonNull<T> instead of *mut (). Also consider splitting operations into stateful types so that after disconnect() callers must transition to a state where only unregister/recover_packet is possible.

---

### 29. Owned-receiver iteration protocol (consuming receiver, blocking receive until end-of-stream)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-42`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: IntoIter owns a Receiver and repeatedly calls recv(), mapping errors to None. This encodes an implicit lifecycle/protocol: consuming iteration holds the receiving capability for the duration of the iterator and may block; termination is tied to recv() error (disconnect), but that terminal cause is not represented in the type (erased by ok()). The type system does not make the disconnect/termination reason explicit, nor does it represent blocking behavior as a capability/state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Sender, 2 free function(s), impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl UnwindSafe for Sender < T > (0 methods), impl RefUnwindSafe for Sender < T > (0 methods), impl Sender < T > (2 methods), impl Sender < T > (7 methods), impl Drop for Sender < T > (1 methods); struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    rx: &'a Receiver<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<'a, T> Iterator for TryIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}

```

**Entity:** IntoIter<T>

**States:** Active (owns receiver; can block), Terminated (channel closed/recv error)

**Transitions:**
- Active -> Terminated via Iterator::next() when self.rx.recv() returns Err (mapped to None by ok())

**Evidence:** impl Iterator for IntoIter<T>: next() { self.rx.recv().ok() } — blocking recv() + error-to-None conversion; IntoIter holds a receiver as self.rx (used in next()) — owning the receive capability for iterator lifetime

**Implementation:** Provide distinct iterator types/capabilities for blocking vs non-blocking consumption, or expose termination reason via Item = Result<T, RecvError> to prevent conflating 'end-of-stream' with other recv failures.

---

### 69. Channel send-result protocol (Success vs Disconnected vs Full)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpsc.rs:1-48`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The API encodes (at runtime, via errors) an implicit channel state/protocol for sending: a send attempt either succeeds, fails because the channel is closed (disconnected), or (for non-blocking/bounded sends) fails because the channel is full. The type system does not let callers statically distinguish or prevent invalid operations like 'sending on a closed channel'; instead, callers discover the state via `SendError` / `TrySendError` at runtime. Additionally, `From<SendError<T>> for TrySendError<T>` encodes a one-way refinement: any `SendError<T>` is always the Disconnected case when viewed as a `TrySendError<T>`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, 2 free function(s), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl Receiver < T > (6 methods), impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; struct Sender, impl Send for Sender < T > (0 methods), impl Sync for Sender < T > (0 methods), impl Send for SyncSender < T > (0 methods), impl Sender < T > (1 methods), impl SyncSender < T > (3 methods); struct SyncSender; struct RecvError, impl error :: Error for RecvError (1 methods), impl error :: Error for TryRecvError (1 methods), impl From < RecvError > for TryRecvError (1 methods); enum TryRecvError; enum RecvTimeoutError, impl error :: Error for RecvTimeoutError (1 methods), impl From < RecvError > for RecvTimeoutError (1 methods); enum TrySendError

/// contains the data being sent as a payload so it can be recovered.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(#[stable(feature = "rust1", since = "1.0.0")] pub T);


// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> error::Error for SendError<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "sending on a closed channel"
    }
}

// ... (other code) ...

}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> error::Error for TrySendError<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            TrySendError::Full(..) => "sending on a full channel",
            TrySendError::Disconnected(..) => "sending on a closed channel",
        }
    }
}

#[stable(feature = "mpsc_error_conversions", since = "1.24.0")]
impl<T> From<SendError<T>> for TrySendError<T> {
    /// Converts a `SendError<T>` into a `TrySendError<T>`.
    ///
    /// This conversion always returns a `TrySendError::Disconnected` containing the data in the `SendError<T>`.
    ///
    /// No data is allocated on the heap.
    fn from(err: SendError<T>) -> TrySendError<T> {
        match err {
            SendError(t) => TrySendError::Disconnected(t),
        }
    }
}

```

**Entity:** SendError<T> / TrySendError<T>

**States:** SendSucceeded, SendFailedDisconnected, SendFailedFull

**Transitions:**
- SendSucceeded -> SendFailedDisconnected when receiver is dropped/closed (observed as SendError/ TrySendError::Disconnected)
- SendSucceeded -> SendFailedFull when bounded channel is at capacity (observed as TrySendError::Full)
- SendFailedDisconnected -> SendFailedDisconnected via From<SendError<T>> for TrySendError<T> (always maps to Disconnected)

**Evidence:** SendError<T>: error::Error::description() returns "sending on a closed channel" (runtime-only disconnected state); TrySendError<T>: description() matches TrySendError::Full(..) => "sending on a full channel" and TrySendError::Disconnected(..) => "sending on a closed channel"; impl From<SendError<T>> for TrySendError<T>: always returns TrySendError::Disconnected(t) (conversion hard-codes the invariant that SendError implies Disconnected)

**Implementation:** Introduce distinct sender capabilities tied to channel state, e.g. `Sender<Open>` vs `Sender<Closed>` (or a `ConnectedSender` token produced alongside `Receiver`), where only `Sender<Open>` exposes `send/try_send`. For bounded channels, optionally split `try_send` into a method requiring an additional `HasCapacity` capability token (issued by a reservation API) so that the 'Full' case is prevented when the token is held. This shifts some runtime errors (Disconnected/Full) into compile-time uncallable methods when the capability cannot be constructed.

---

### 55. Packet message/handshake protocol (Unready/Ready and Empty/Full)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-35`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Packet<T> encodes a single-slot rendezvous protocol using runtime state: `ready: Atomic<bool>` gates when it is legal for a producer/consumer to touch `msg`, and `msg: UnsafeCell<Option<T>>` encodes whether a message is present. Construction creates packets with `ready = false` (unready), while `msg` may start as None (empty) or Some(T) (pre-filled). `wait_ready()` busy-waits until `ready` becomes true, implying a temporal ordering requirement: some other actor must eventually set `ready` true (and must do so with a release store) after establishing the intended access rights to `msg`. None of these state transitions (who may write/read when, and whether `msg` is initialized) are represented in the type system; instead they rely on atomics, UnsafeCell, and Option to encode legality at runtime. Misuse can lead to reading/writing `msg` before readiness or concurrent aliasing violations hidden behind UnsafeCell.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Inner; struct Channel, impl Channel < T > (12 methods)

}

/// A slot for passing one message from a sender to a receiver.
struct Packet<T> {
    /// Equals `true` if the packet is allocated on the stack.
    on_stack: bool,

    /// Equals `true` once the packet is ready for reading or writing.
    ready: Atomic<bool>,

    /// The message.
    msg: UnsafeCell<Option<T>>,
}

impl<T> Packet<T> {
    /// Creates an empty packet on the stack.
    fn empty_on_stack() -> Packet<T> {
        Packet { on_stack: true, ready: AtomicBool::new(false), msg: UnsafeCell::new(None) }
    }

    /// Creates a packet on the stack, containing a message.
    fn message_on_stack(msg: T) -> Packet<T> {
        Packet { on_stack: true, ready: AtomicBool::new(false), msg: UnsafeCell::new(Some(msg)) }
    }

    /// Waits until the packet becomes ready for reading or writing.
    fn wait_ready(&self) {
        let backoff = Backoff::new();
        while !self.ready.load(Ordering::Acquire) {
            backoff.spin_heavy();
        }
    }
}

```

**Entity:** Packet<T>

**States:** Unready+Empty, Unready+Full, ReadyForWrite, ReadyForRead

**Transitions:**
- Unready+Empty -> ReadyForWrite when some other code sets `ready` to true after reserving packet for writer
- Unready+Full -> ReadyForRead when some other code sets `ready` to true after making message available
- ReadyForWrite -> ReadyForRead when writer stores `msg = Some(T)` then signals readiness for reader (not shown in snippet)
- ReadyForRead -> (consumed/empty) when reader takes `msg` and the packet is reused (not shown in snippet)

**Evidence:** `ready: Atomic<bool>` field tracks readiness state at runtime; `msg: UnsafeCell<Option<T>>` field encodes initialized/uninitialized message state and permits interior mutation beyond Rust aliasing rules; `empty_on_stack()` sets `ready: AtomicBool::new(false)` and `msg: None` (unready+empty); `message_on_stack(msg)` sets `ready: AtomicBool::new(false)` and `msg: Some(msg)` (unready+full); `wait_ready()` spins on `while !self.ready.load(Ordering::Acquire)` indicating a required happens-before protocol tied to `ready`

**Implementation:** Model the protocol as `Packet<T, S>` where `S` is a zero-sized state type (e.g., `UnreadyEmpty`, `UnreadyFull`, `WriteReady`, `ReadReady`). Constructors return `Packet<T, UnreadyEmpty>` / `Packet<T, UnreadyFull>`. Provide state-transition methods that consume `self` (or require a capability token) and return the next state, e.g. `fn wait_ready(self) -> Packet<T, ReadReady>` or split into `WriterPacket`/`ReaderPacket` newtypes so only the correct side can access `msg`. This moves “must wait until ready before touching msg” and “msg present/absent” into the type system.

---

### 13. Receive reservation protocol (start_recv -> read) with empty/disconnected outcomes

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-445`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Receiving is also a two-phase protocol: `start_recv()` reserves a slot to read from and writes reservation data into `token.array.{slot,stamp}`; then `read()` consumes that reservation by reading initialized data and updating the slot stamp to free it. `start_recv()` can also indicate 'would block/empty' by returning false, and 'disconnected and empty' by returning true with a null slot pointer. The type system does not prevent calling `read()` without a successful `start_recv()`, reusing a token, or mixing tokens across channels; safety is enforced by convention plus a null-pointer sentinel checked at runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot; struct ArrayToken

}

/// Bounded channel based on a preallocated array.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit in the head is always zero.
    ///
    /// Messages are popped from the head of the channel.
    head: CachePadded<Atomic<usize>>,

    /// The tail of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit indicates that the channel is disconnected.
    ///
    /// Messages are pushed into the tail of the channel.
    tail: CachePadded<Atomic<usize>>,

    /// The buffer holding slots.
    buffer: Box<[Slot<T>]>,

    /// The channel capacity.
    cap: usize,

    /// A stamp with the value of `{ lap: 1, mark: 0, index: 0 }`.
    one_lap: usize,

    /// If this bit is set in the tail, that means the channel is disconnected.
    mark_bit: usize,

    /// Senders waiting while the channel is full.
    senders: SyncWaker,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,
}

impl<T> Channel<T> {
    /// Creates a bounded channel of capacity `cap`.
    pub(crate) fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be positive");

        // Compute constants `mark_bit` and `one_lap`.
        let mark_bit = (cap + 1).next_power_of_two();
        let one_lap = mark_bit * 2;

        // Head is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let head = 0;
        // Tail is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let tail = 0;

        // Allocate a buffer of `cap` slots initialized
        // with stamps.
        let buffer: Box<[Slot<T>]> = (0..cap)
            .map(|i| {
                // Set the stamp to `{ lap: 0, mark: 0, index: i }`.
                Slot { stamp: AtomicUsize::new(i), msg: UnsafeCell::new(MaybeUninit::uninit()) }
            })
            .collect();

        Channel {
            buffer,
            cap,
            one_lap,
            mark_bit,
            head: CachePadded::new(AtomicUsize::new(head)),
            tail: CachePadded::new(AtomicUsize::new(tail)),
            senders: SyncWaker::new(),
            receivers: SyncWaker::new(),
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            // Check if the channel is disconnected.
            if tail & self.mark_bit != 0 {
                token.array.slot = ptr::null();
                token.array.stamp = 0;
                return true;
            }

            // Deconstruct the tail.
            let index = tail & (self.mark_bit - 1);
            let lap = tail & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                let new_tail = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    tail + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail,
                    new_tail,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `write`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = tail + 1;
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        tail = self.tail.load(Ordering::Relaxed);
                    }
                }
            } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
                atomic::fence(Ordering::SeqCst);
                let head = self.head.load(Ordering::Relaxed);

                // If the head lags one lap behind the tail as well...
                if head.wrapping_add(self.one_lap) == tail {
                    // ...then the channel is full.
                    return false;
                }

                backoff.spin_light();
                tail = self.tail.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.array.slot.is_null() {
            return Err(msg);
        }

        // Write the message into the slot and update the stamp.
        unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.stamp.store(token.array.stamp, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `read`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = head.wrapping_add(self.one_lap);
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        head = self.head.load(Ordering::Relaxed);
                    }
                }
            } else if stamp == head {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if (tail & !self.mark_bit) == head {
                    // If the channel is disconnected...
                    if tail & self.mark_bit != 0 {
                        // ...then receive an error.
                        token.array.slot = ptr::null();
                        token.array.stamp = 0;
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                backoff.spin_light();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.array.slot.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message from the slot and update the stamp.
        let msg = unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);

            let msg = slot.msg.get().read().assume_init();
            slot.stamp.store(token.array.stamp, Ordering::Release);
            msg
        };

        // Wake a sleeping sender.
        self.senders.notify();
        Ok(msg)
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        let token = &mut Token::default();
        if self.start_send(token) {
            unsafe { self.write(token, msg).map_err(TrySendError::Disconnected) }
        } else {
            Err(TrySendError::Full(msg))
        }
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        loop {
            // Try sending a message.
            if self.start_send(token) {
                let res = unsafe { self.write(token, msg) };
                return res.map_err(SendTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(SendTimeoutError::Timeout(msg));
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a receiver wakes us up.
                let oper = Operation::hook(token);
                self.senders.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_full() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.senders.unregister(oper).unwrap();
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            // Try receiving a message.
            if self.start_recv(token) {
                let res = unsafe { self.read(token) };
                return res.map_err(|_| RecvTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a sender wakes us up.
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail, then load the head.
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            // If the tail didn't change, we've got consistent values to work with.
            if self.tail.load(Ordering::SeqCst) == tail {
                let hix = head & (self.mark_bit - 1);
                let tix = tail & (self.mark_bit - 1);

                return if hix < tix {
                    tix - hix
                } else if hix > tix {
                    self.cap - hix + tix
                } else if (tail & !self.mark_bit) == head {
                    0
                } else {
                    self.cap
                };
            }
        }
    }

    /// Returns the capacity of the channel.
    #[allow(clippy::unnecessary_wraps)] // This is intentional.
    pub(crate) fn capacity(&self) -> Option<usize> {
        Some(self.cap)
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);

        if tail & self.mark_bit == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers and wakes up all blocked senders.
    ///
    /// Returns `true` if this call disconnected the channel.
    ///
    /// # Safety
    /// May only be called once upon dropping the last receiver. The
    /// destruction of all other receivers must have been observed with acquire
    /// ordering or stronger.
    pub(crate) unsafe fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);
        let disconnected = if tail & self.mark_bit == 0 {
            self.senders.disconnect();
            true
        } else {
            false
        };

        unsafe { self.discard_all_messages(tail) };
        disconnected
    }

    /// Discards all messages.
    ///
    /// `tail` should be the current (and therefore last) value of `tail`.
    ///
    /// # Panicking
    /// If a de
// ... (truncated) ...
```

**Entity:** Channel<T> (via Token/ArrayToken used by start_recv/read)

**States:** NoReservation, ReservedSlot, DisconnectedReservation, RecvWouldBlock(Empty)

**Transitions:**
- NoReservation -> ReservedSlot via start_recv() returning true and setting token.array.slot/stamp
- NoReservation -> DisconnectedReservation via start_recv() setting token.array.slot = null and returning true when tail is marked and empty
- NoReservation -> RecvWouldBlock(Empty) via start_recv() returning false when empty but not disconnected
- ReservedSlot -> NoReservation (logically consumed) via unsafe read() reading msg and updating stamp, waking senders
- DisconnectedReservation -> NoReservation via read() returning Err(())

**Evidence:** fn start_recv(&self, token: &mut Token) -> bool: returns false when empty+connected; otherwise sets token.array.slot/token.array.stamp for follow-up `read`; start_recv: on successful head CAS: `token.array.slot = slot as *const Slot<T> as *const u8; token.array.stamp = head.wrapping_add(self.one_lap);`; start_recv: empty case `if (tail & !self.mark_bit) == head { ... if tail & self.mark_bit != 0 { token.array.slot = ptr::null(); token.array.stamp = 0; return true; } else { return false; } }` distinguishes empty vs disconnected via return+sentinel; pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()>: `if token.array.slot.is_null() { return Err(()); }` runtime state check; try_recv/recv: required call ordering `if self.start_recv(token) { unsafe { self.read(token) } }`

**Implementation:** Introduce `RecvPermit<'_, T>` returned by `start_recv` (or a safe wrapper) that can only be constructed by the channel and must be consumed by `permit.read() -> Result<T, Disconnected>`. Encode Empty vs Disconnected as separate return types/variants, eliminating the `bool` + null-pointer token encoding and making `read` safe (no external `unsafe`).

---

### 7. Sender channel-flavor protocol (Array/List/Zero) with capability-dependent semantics

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/mod.rs:1-337`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Sender<T> is a single type whose runtime `flavor: SenderFlavor<T>` selects fundamentally different channel semantics (bounded vs unbounded vs zero-capacity rendezvous). Many methods have flavor-specific meaning: for example `try_send` can fail due to full/disconnected and `send` can block when bounded/full, while for zero-capacity channels `send`/`try_send` require a simultaneous receive and `is_empty`/`is_full` are documented as always true/always false depending on the query. None of these semantic differences are represented at the type level, so callers cannot express or rely on properties like 'this sender is rendezvous' or 'this sender is bounded with known capacity' without runtime branching or documentation assumptions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Receiver, impl IntoIterator for & 'a Receiver < T > (1 methods), impl IntoIterator for Receiver < T > (1 methods), impl Send for Receiver < T > (0 methods), impl Sync for Receiver < T > (0 methods), impl UnwindSafe for Receiver < T > (0 methods), impl RefUnwindSafe for Receiver < T > (0 methods), impl Receiver < T > (5 methods), impl Receiver < T > (6 methods), impl Drop for Receiver < T > (1 methods); struct Iter, impl Iterator for Iter < 'a , T > (1 methods), impl Iterator for TryIter < 'a , T > (1 methods), impl Iterator for IntoIter < T > (1 methods); struct TryIter; struct IntoIter; enum SenderFlavor; enum ReceiverFlavor

/// assert_eq!(3, msg + msg2);
/// ```
#[unstable(feature = "mpmc_channel", issue = "126840")]
pub struct Sender<T> {
    flavor: SenderFlavor<T>,
}

// ... (other code) ...

}

#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Send for Sender<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
unsafe impl<T: Send> Sync for Sender<T> {}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> UnwindSafe for Sender<T> {}
#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> RefUnwindSafe for Sender<T> {}

impl<T> Sender<T> {
    /// Attempts to send a message into the channel without blocking.
    ///
    /// This method will either send a message into the channel immediately or return an error if
    /// the channel is full or disconnected. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will send the message only if there
    /// happens to be a receive operation on the other side of the channel at the same time.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::{channel, Receiver, Sender};
    ///
    /// let (sender, _receiver): (Sender<i32>, Receiver<i32>) = channel();
    ///
    /// assert!(sender.try_send(1).is_ok());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.try_send(msg),
            SenderFlavor::List(chan) => chan.try_send(msg),
            SenderFlavor::Zero(chan) => chan.try_send(msg),
        }
    }

    /// Attempts to send a value on this channel, returning it back if it could
    /// not be sent.
    ///
    /// A successful send occurs when it is determined that the other end of
    /// the channel has not hung up already. An unsuccessful send would be one
    /// where the corresponding receiver has already been deallocated. Note
    /// that a return value of [`Err`] means that the data will never be
    /// received, but a return value of [`Ok`] does *not* mean that the data
    /// will be received. It is possible for the corresponding receiver to
    /// hang up immediately after this function returns [`Ok`]. However, if
    /// the channel is zero-capacity, it acts as a rendezvous channel and a
    /// return value of [`Ok`] means that the data has been received.
    ///
    /// If the channel is full and not disconnected, this call will block until
    /// the send operation can proceed. If the channel becomes disconnected,
    /// this call will wake up and return an error. The returned error contains
    /// the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive
    /// operation to appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    ///
    /// let (tx, rx) = channel();
    ///
    /// // This send is always successful
    /// tx.send(1).unwrap();
    ///
    /// // This send will fail because the receiver is gone
    /// drop(rx);
    /// assert!(tx.send(1).is_err());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.send(msg, None),
            SenderFlavor::List(chan) => chan.send(msg, None),
            SenderFlavor::Zero(chan) => chan.send(msg, None),
        }
        .map_err(|err| match err {
            SendTimeoutError::Disconnected(msg) => SendError(msg),
            SendTimeoutError::Timeout(_) => unreachable!(),
        })
    }
}

impl<T> Sender<T> {
    /// Waits for a message to be sent into the channel, but only for a limited time.
    ///
    /// If the channel is full and not disconnected, this call will block until the send operation
    /// can proceed or the operation times out. If the channel becomes disconnected, this call will
    /// wake up and return an error. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive operation to
    /// appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::time::Duration;
    ///
    /// let (tx, rx) = channel();
    ///
    /// tx.send_timeout(1, Duration::from_millis(400)).unwrap();
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send_timeout(&self, msg: T, timeout: Duration) -> Result<(), SendTimeoutError<T>> {
        match Instant::now().checked_add(timeout) {
            Some(deadline) => self.send_deadline(msg, deadline),
            // So far in the future that it's practically the same as waiting indefinitely.
            None => self.send(msg).map_err(SendTimeoutError::from),
        }
    }

    /// Waits for a message to be sent into the channel, but only until a given deadline.
    ///
    /// If the channel is full and not disconnected, this call will block until the send operation
    /// can proceed or the operation times out. If the channel becomes disconnected, this call will
    /// wake up and return an error. The returned error contains the original message.
    ///
    /// If called on a zero-capacity channel, this method will wait for a receive operation to
    /// appear on the other side of the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc::channel;
    /// use std::time::{Duration, Instant};
    ///
    /// let (tx, rx) = channel();
    ///
    /// let t = Instant::now() + Duration::from_millis(400);
    /// tx.send_deadline(1, t).unwrap();
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn send_deadline(&self, msg: T, deadline: Instant) -> Result<(), SendTimeoutError<T>> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.send(msg, Some(deadline)),
            SenderFlavor::List(chan) => chan.send(msg, Some(deadline)),
            SenderFlavor::Zero(chan) => chan.send(msg, Some(deadline)),
        }
    }

    /// Returns `true` if the channel is empty.
    ///
    /// Note: Zero-capacity channels are always empty.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::channel();
    ///
    /// let tx1 = send.clone();
    /// let tx2 = send.clone();
    ///
    /// assert!(tx1.is_empty());
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(!tx1.is_empty());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_empty(&self) -> bool {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.is_empty(),
            SenderFlavor::List(chan) => chan.is_empty(),
            SenderFlavor::Zero(chan) => chan.is_empty(),
        }
    }

    /// Returns `true` if the channel is full.
    ///
    /// Note: Zero-capacity channels are always full.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::sync_channel(1);
    ///
    /// let (tx1, tx2) = (send.clone(), send.clone());
    /// assert!(!tx1.is_full());
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert!(tx1.is_full());
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn is_full(&self) -> bool {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.is_full(),
            SenderFlavor::List(chan) => chan.is_full(),
            SenderFlavor::Zero(chan) => chan.is_full(),
        }
    }

    /// Returns the number of messages in the channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::channel();
    /// let (tx1, tx2) = (send.clone(), send.clone());
    ///
    /// assert_eq!(tx1.len(), 0);
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(tx1.len(), 1);
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn len(&self) -> usize {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.len(),
            SenderFlavor::List(chan) => chan.len(),
            SenderFlavor::Zero(chan) => chan.len(),
        }
    }

    /// If the channel is bounded, returns its capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    /// use std::thread;
    ///
    /// let (send, _recv) = mpmc::sync_channel(3);
    /// let (tx1, tx2) = (send.clone(), send.clone());
    ///
    /// assert_eq!(tx1.capacity(), Some(3));
    ///
    /// let handle = thread::spawn(move || {
    ///     tx2.send(1u8).unwrap();
    /// });
    ///
    /// handle.join().unwrap();
    ///
    /// assert_eq!(tx1.capacity(), Some(3));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn capacity(&self) -> Option<usize> {
        match &self.flavor {
            SenderFlavor::Array(chan) => chan.capacity(),
            SenderFlavor::List(chan) => chan.capacity(),
            SenderFlavor::Zero(chan) => chan.capacity(),
        }
    }

    /// Returns `true` if senders belong to the same channel.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(mpmc_channel)]
    ///
    /// use std::sync::mpmc;
    ///
    /// let (tx1, _) = mpmc::channel::<i32>();
    /// let (tx2, _) = mpmc::channel::<i32>();
    ///
    /// assert!(tx1.same_channel(&tx1));
    /// assert!(!tx1.same_channel(&tx2));
    /// ```
    #[unstable(feature = "mpmc_channel", issue = "126840")]
    pub fn same_channel(&self, other: &Sender<T>) -> bool {
        match (&self.flavor, &other.flavor) {
            (SenderFlavor::Array(a), SenderFlavor::Array(b)) => a == b,
            (SenderFlavor::List(a), SenderFlavor::List(b)) => a == b,
            (SenderFlavor::Zero(a), SenderFlavor::Zero(b)) => a == b,
            _ => false,
        }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        unsafe {
            match &self.flavor {
                SenderFlavor::Array(chan) => chan.release(|c| c.disconnect_senders()),
                SenderFlavor::List(chan) => chan.release(|c| c.disconnect_senders()),
                SenderFlavor::Zero(chan) => chan.release(|c| c.disconnect()),
            }
        }
    }
}

```

**Entity:** Sender<T>

**States:** Bounded(Array), Unbounded(List), Rendezvous(Zero)

**Evidence:** field `Sender<T>::flavor: SenderFlavor<T>` encodes the operational mode at runtime; method `try_send`: `match &self.flavor { SenderFlavor::Array(..) | List(..) | Zero(..) => ... }`; method `send`: same flavor dispatch; docs describe blocking behavior and special-case guarantee for zero-capacity rendezvous (`Ok` means received); method `is_empty` docs: "Zero-capacity channels are always empty." while still dispatching via `match &self.flavor`; method `is_full` docs: "Zero-capacity channels are always full." while still dispatching via `match &self.flavor`; method `capacity` returns `Option<usize>` (bounded vs unbounded/zero encoded at runtime rather than via type); method `same_channel` only compares when flavors match; otherwise returns false (`_ => false`), reflecting that cross-flavor senders are incomparable

**Implementation:** Parameterize Sender by a flavor/state marker: `Sender<T, F>` where `F` is `Bounded`, `Unbounded`, or `Rendezvous`. Constructors like `channel()`/`sync_channel()` would return `Sender<T, Unbounded>` or `Sender<T, Bounded<N>>` (or `Bounded` with runtime cap). Expose flavor-specific APIs: e.g., `capacity() -> usize` only on bounded, rendezvous-specific docs/guarantees attached to `Sender<T, Rendezvous>`, and potentially different error types for `try_send` on rendezvous vs buffered channels.

---

### 66. Zero token packet-pointer protocol (Null/NoPacket vs NonNull/HasPacket; stack vs heap packet ownership)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/zero.rs:1-248`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: `Channel::write` and `Channel::read` require a `Token` whose `zero.0` points to a valid `Packet<T>` produced by the channel pairing logic. A null pointer is treated as 'disconnected' and causes immediate failure. Additionally, `Packet::on_stack` selects a different lifecycle: stack packets must not be freed (only signal readiness), while heap packets must be freed exactly once by `read()` after consuming the message. These are enforced by runtime null checks, an `on_stack` boolean, and `unsafe` pointer casts; the type system does not prevent calling `write/read` with an unpaired token, a null pointer, a pointer of the wrong allocation kind, or a token reused after the packet is logically consumed/freed.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct ZeroToken; struct Packet, impl Packet < T > (3 methods); struct Inner

}

/// Zero-capacity channel.
pub(crate) struct Channel<T> {
    /// Inner representation of the channel.
    inner: Mutex<Inner>,

    /// Indicates that dropping a `Channel<T>` may drop values of type `T`.
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Constructs a new zero-capacity channel.
    pub(crate) fn new() -> Self {
        Channel {
            inner: Mutex::new(Inner {
                senders: Waker::new(),
                receivers: Waker::new(),
                is_disconnected: false,
            }),
            _marker: PhantomData,
        }
    }

    /// Writes a message into the packet.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no packet, the channel is disconnected.
        if token.zero.0.is_null() {
            return Err(msg);
        }

        unsafe {
            let packet = &*(token.zero.0 as *const Packet<T>);
            packet.msg.get().write(Some(msg));
            packet.ready.store(true, Ordering::Release);
        }
        Ok(())
    }

    /// Reads a message from the packet.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        // If there is no packet, the channel is disconnected.
        if token.zero.0.is_null() {
            return Err(());
        }

        let packet = unsafe { &*(token.zero.0 as *const Packet<T>) };

        if packet.on_stack {
            // The message has been in the packet from the beginning, so there is no need to wait
            // for it. However, after reading the message, we need to set `ready` to `true` in
            // order to signal that the packet can be destroyed.
            let msg = unsafe { packet.msg.get().replace(None) }.unwrap();
            packet.ready.store(true, Ordering::Release);
            Ok(msg)
        } else {
            // Wait until the message becomes available, then read it and destroy the
            // heap-allocated packet.
            packet.wait_ready();
            unsafe {
                let msg = packet.msg.get().replace(None).unwrap();
                drop(Box::from_raw(token.zero.0 as *mut Packet<T>));
                Ok(msg)
            }
        }
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting receiver, pair up with it.
        if let Some(operation) = inner.receivers.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                self.write(token, msg).ok().unwrap();
            }
            Ok(())
        } else if inner.is_disconnected {
            Err(TrySendError::Disconnected(msg))
        } else {
            Err(TrySendError::Full(msg))
        }
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting receiver, pair up with it.
        if let Some(operation) = inner.receivers.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                self.write(token, msg).ok().unwrap();
            }
            return Ok(());
        }

        if inner.is_disconnected {
            return Err(SendTimeoutError::Disconnected(msg));
        }

        Context::with(|cx| {
            // Prepare for blocking until a receiver wakes us up.
            let oper = Operation::hook(token);
            let mut packet = Packet::<T>::message_on_stack(msg);
            inner.senders.register_with_packet(oper, (&raw mut packet) as *mut (), cx);
            inner.receivers.notify();
            drop(inner);

            // Block the current thread.
            // SAFETY: the context belongs to the current thread.
            let sel = unsafe { cx.wait_until(deadline) };

            match sel {
                Selected::Waiting => unreachable!(),
                Selected::Aborted => {
                    self.inner.lock().unwrap().senders.unregister(oper).unwrap();
                    let msg = unsafe { packet.msg.get().replace(None).unwrap() };
                    Err(SendTimeoutError::Timeout(msg))
                }
                Selected::Disconnected => {
                    self.inner.lock().unwrap().senders.unregister(oper).unwrap();
                    let msg = unsafe { packet.msg.get().replace(None).unwrap() };
                    Err(SendTimeoutError::Disconnected(msg))
                }
                Selected::Operation(_) => {
                    // Wait until the message is read, then drop the packet.
                    packet.wait_ready();
                    Ok(())
                }
            }
        })
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting sender, pair up with it.
        if let Some(operation) = inner.senders.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else if inner.is_disconnected {
            Err(TryRecvError::Disconnected)
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        let mut inner = self.inner.lock().unwrap();

        // If there's a waiting sender, pair up with it.
        if let Some(operation) = inner.senders.try_select() {
            token.zero.0 = operation.packet;
            drop(inner);
            unsafe {
                return self.read(token).map_err(|_| RecvTimeoutError::Disconnected);
            }
        }

        if inner.is_disconnected {
            return Err(RecvTimeoutError::Disconnected);
        }

        Context::with(|cx| {
            // Prepare for blocking until a sender wakes us up.
            let oper = Operation::hook(token);
            let mut packet = Packet::<T>::empty_on_stack();
            inner.receivers.register_with_packet(oper, (&raw mut packet) as *mut (), cx);
            inner.senders.notify();
            drop(inner);

            // Block the current thread.
            // SAFETY: the context belongs to the current thread.
            let sel = unsafe { cx.wait_until(deadline) };

            match sel {
                Selected::Waiting => unreachable!(),
                Selected::Aborted => {
                    self.inner.lock().unwrap().receivers.unregister(oper).unwrap();
                    Err(RecvTimeoutError::Timeout)
                }
                Selected::Disconnected => {
                    self.inner.lock().unwrap().receivers.unregister(oper).unwrap();
                    Err(RecvTimeoutError::Disconnected)
                }
                Selected::Operation(_) => {
                    // Wait until the message is provided, then read it.
                    packet.wait_ready();
                    unsafe { Ok(packet.msg.get().replace(None).unwrap()) }
                }
            }
        })
    }

    /// Disconnects the channel and wakes up all blocked senders and receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();

        if !inner.is_disconnected {
            inner.is_disconnected = true;
            inner.senders.disconnect();
            inner.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        0
    }

    /// Returns the capacity of the channel.
    #[allow(clippy::unnecessary_wraps)] // This is intentional.
    pub(crate) fn capacity(&self) -> Option<usize> {
        Some(0)
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        true
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        true
    }
}

```

**Entity:** Token (used via `token.zero.0` in Channel::{write,read})

**States:** NoPacket(DisconnectedOrInvalid), HasPacket(StackPacket), HasPacket(HeapPacket)

**Transitions:**
- NoPacket -> HasPacket(StackPacket) via pairing and `register_with_packet(..., &raw mut packet as *mut (), ...)` in send()/recv()
- NoPacket -> HasPacket(HeapPacket) via pairing with an operation whose `operation.packet` refers to a heap packet (implied by `Box::from_raw` in read())
- HasPacket(StackPacket) -> NoPacket via read() (consumes message and signals `ready` so stack packet can be dropped by its owner)
- HasPacket(HeapPacket) -> NoPacket via read() (consumes message and `drop(Box::from_raw(...))` frees packet)

**Evidence:** method: write(): `if token.zero.0.is_null() { return Err(msg); }` (null token treated as disconnected/invalid); method: write(): `let packet = &*(token.zero.0 as *const Packet<T>);` (unchecked cast from raw pointer to typed packet); method: read(): `if token.zero.0.is_null() { return Err(()); }` (same null invariant on receive side); method: read(): branches on `if packet.on_stack { ... } else { ... drop(Box::from_raw(token.zero.0 as *mut Packet<T>)); }` (allocation-kind protocol + exactly-once free for heap packets); method: send(): creates `let mut packet = Packet::<T>::message_on_stack(msg);` and passes pointer via `register_with_packet(..., (&raw mut packet) as *mut (), ...)` (stack packet pointer escapes through untyped `*mut ()`); method: recv(): creates `let mut packet = Packet::<T>::empty_on_stack();` and similarly registers it with `register_with_packet`; method: try_send()/send()/try_recv()/recv(): assign `token.zero.0 = operation.packet;` (token is populated by pairing; correctness depends on prior selection/register protocol)

**Implementation:** Replace `Token`'s raw pointer with a typed capability such as `enum PacketRef<'a, T> { Stack(&'a Packet<T>), Heap(NonNull<Packet<T>>) }` carried in a `Token<Paired<T>>` typestate, where `write/read` require `Token<Paired<T>>` and cannot be called with `Token<Unpaired>`. Encode ownership in types so only the `Heap` variant runs `Box::from_raw`, and make the null/disconnected case unrepresentable (e.g., `Option<PacketRef<...>>` only at the pairing boundary).

---

### 12. Send reservation protocol (start_send -> write) with disconnected/full outcomes

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/array.rs:1-445`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Sending is a two-phase protocol: `start_send()` reserves a specific slot and encodes that reservation into `token.array.{slot,stamp}`; only then may `write()` be called to initialize the message and publish it by storing the stamp. The protocol also has a disconnected outcome encoded by a null slot pointer, and a 'would block/full' outcome encoded by `start_send()` returning false. None of these states are represented in the type system: `write()` is `unsafe` and relies on the caller to have previously called `start_send()` and to pass an unconsumed token belonging to the same channel and the current reservation. The null-pointer sentinel (`token.array.slot.is_null()`) is a runtime encoding of a distinct logical state.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Slot; struct ArrayToken

}

/// Bounded channel based on a preallocated array.
pub(crate) struct Channel<T> {
    /// The head of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit in the head is always zero.
    ///
    /// Messages are popped from the head of the channel.
    head: CachePadded<Atomic<usize>>,

    /// The tail of the channel.
    ///
    /// This value is a "stamp" consisting of an index into the buffer, a mark bit, and a lap, but
    /// packed into a single `usize`. The lower bits represent the index, while the upper bits
    /// represent the lap. The mark bit indicates that the channel is disconnected.
    ///
    /// Messages are pushed into the tail of the channel.
    tail: CachePadded<Atomic<usize>>,

    /// The buffer holding slots.
    buffer: Box<[Slot<T>]>,

    /// The channel capacity.
    cap: usize,

    /// A stamp with the value of `{ lap: 1, mark: 0, index: 0 }`.
    one_lap: usize,

    /// If this bit is set in the tail, that means the channel is disconnected.
    mark_bit: usize,

    /// Senders waiting while the channel is full.
    senders: SyncWaker,

    /// Receivers waiting while the channel is empty and not disconnected.
    receivers: SyncWaker,
}

impl<T> Channel<T> {
    /// Creates a bounded channel of capacity `cap`.
    pub(crate) fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be positive");

        // Compute constants `mark_bit` and `one_lap`.
        let mark_bit = (cap + 1).next_power_of_two();
        let one_lap = mark_bit * 2;

        // Head is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let head = 0;
        // Tail is initialized to `{ lap: 0, mark: 0, index: 0 }`.
        let tail = 0;

        // Allocate a buffer of `cap` slots initialized
        // with stamps.
        let buffer: Box<[Slot<T>]> = (0..cap)
            .map(|i| {
                // Set the stamp to `{ lap: 0, mark: 0, index: i }`.
                Slot { stamp: AtomicUsize::new(i), msg: UnsafeCell::new(MaybeUninit::uninit()) }
            })
            .collect();

        Channel {
            buffer,
            cap,
            one_lap,
            mark_bit,
            head: CachePadded::new(AtomicUsize::new(head)),
            tail: CachePadded::new(AtomicUsize::new(tail)),
            senders: SyncWaker::new(),
            receivers: SyncWaker::new(),
        }
    }

    /// Attempts to reserve a slot for sending a message.
    fn start_send(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            // Check if the channel is disconnected.
            if tail & self.mark_bit != 0 {
                token.array.slot = ptr::null();
                token.array.stamp = 0;
                return true;
            }

            // Deconstruct the tail.
            let index = tail & (self.mark_bit - 1);
            let lap = tail & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                let new_tail = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    tail + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail,
                    new_tail,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `write`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = tail + 1;
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        tail = self.tail.load(Ordering::Relaxed);
                    }
                }
            } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
                atomic::fence(Ordering::SeqCst);
                let head = self.head.load(Ordering::Relaxed);

                // If the head lags one lap behind the tail as well...
                if head.wrapping_add(self.one_lap) == tail {
                    // ...then the channel is full.
                    return false;
                }

                backoff.spin_light();
                tail = self.tail.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    /// Writes a message into the channel.
    pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T> {
        // If there is no slot, the channel is disconnected.
        if token.array.slot.is_null() {
            return Err(msg);
        }

        // Write the message into the slot and update the stamp.
        unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);
            slot.msg.get().write(MaybeUninit::new(msg));
            slot.stamp.store(token.array.stamp, Ordering::Release);
        }

        // Wake a sleeping receiver.
        self.receivers.notify();
        Ok(())
    }

    /// Attempts to reserve a slot for receiving a message.
    fn start_recv(&self, token: &mut Token) -> bool {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Prepare the token for the follow-up call to `read`.
                        token.array.slot = slot as *const Slot<T> as *const u8;
                        token.array.stamp = head.wrapping_add(self.one_lap);
                        return true;
                    }
                    Err(_) => {
                        backoff.spin_light();
                        head = self.head.load(Ordering::Relaxed);
                    }
                }
            } else if stamp == head {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if (tail & !self.mark_bit) == head {
                    // If the channel is disconnected...
                    if tail & self.mark_bit != 0 {
                        // ...then receive an error.
                        token.array.slot = ptr::null();
                        token.array.stamp = 0;
                        return true;
                    } else {
                        // Otherwise, the receive operation is not ready.
                        return false;
                    }
                }

                backoff.spin_light();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.spin_heavy();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    /// Reads a message from the channel.
    pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()> {
        if token.array.slot.is_null() {
            // The channel is disconnected.
            return Err(());
        }

        // Read the message from the slot and update the stamp.
        let msg = unsafe {
            let slot: &Slot<T> = &*(token.array.slot as *const Slot<T>);

            let msg = slot.msg.get().read().assume_init();
            slot.stamp.store(token.array.stamp, Ordering::Release);
            msg
        };

        // Wake a sleeping sender.
        self.senders.notify();
        Ok(msg)
    }

    /// Attempts to send a message into the channel.
    pub(crate) fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        let token = &mut Token::default();
        if self.start_send(token) {
            unsafe { self.write(token, msg).map_err(TrySendError::Disconnected) }
        } else {
            Err(TrySendError::Full(msg))
        }
    }

    /// Sends a message into the channel.
    pub(crate) fn send(
        &self,
        msg: T,
        deadline: Option<Instant>,
    ) -> Result<(), SendTimeoutError<T>> {
        let token = &mut Token::default();
        loop {
            // Try sending a message.
            if self.start_send(token) {
                let res = unsafe { self.write(token, msg) };
                return res.map_err(SendTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(SendTimeoutError::Timeout(msg));
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a receiver wakes us up.
                let oper = Operation::hook(token);
                self.senders.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_full() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.senders.unregister(oper).unwrap();
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Attempts to receive a message without blocking.
    pub(crate) fn try_recv(&self) -> Result<T, TryRecvError> {
        let token = &mut Token::default();

        if self.start_recv(token) {
            unsafe { self.read(token).map_err(|_| TryRecvError::Disconnected) }
        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Receives a message from the channel.
    pub(crate) fn recv(&self, deadline: Option<Instant>) -> Result<T, RecvTimeoutError> {
        let token = &mut Token::default();
        loop {
            // Try receiving a message.
            if self.start_recv(token) {
                let res = unsafe { self.read(token) };
                return res.map_err(|_| RecvTimeoutError::Disconnected);
            }

            if let Some(d) = deadline {
                if Instant::now() >= d {
                    return Err(RecvTimeoutError::Timeout);
                }
            }

            Context::with(|cx| {
                // Prepare for blocking until a sender wakes us up.
                let oper = Operation::hook(token);
                self.receivers.register(oper, cx);

                // Has the channel become ready just now?
                if !self.is_empty() || self.is_disconnected() {
                    let _ = cx.try_select(Selected::Aborted);
                }

                // Block the current thread.
                // SAFETY: the context belongs to the current thread.
                let sel = unsafe { cx.wait_until(deadline) };

                match sel {
                    Selected::Waiting => unreachable!(),
                    Selected::Aborted | Selected::Disconnected => {
                        self.receivers.unregister(oper).unwrap();
                        // If the channel was disconnected, we still have to check for remaining
                        // messages.
                    }
                    Selected::Operation(_) => {}
                }
            });
        }
    }

    /// Returns the current number of messages inside the channel.
    pub(crate) fn len(&self) -> usize {
        loop {
            // Load the tail, then load the head.
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            // If the tail didn't change, we've got consistent values to work with.
            if self.tail.load(Ordering::SeqCst) == tail {
                let hix = head & (self.mark_bit - 1);
                let tix = tail & (self.mark_bit - 1);

                return if hix < tix {
                    tix - hix
                } else if hix > tix {
                    self.cap - hix + tix
                } else if (tail & !self.mark_bit) == head {
                    0
                } else {
                    self.cap
                };
            }
        }
    }

    /// Returns the capacity of the channel.
    #[allow(clippy::unnecessary_wraps)] // This is intentional.
    pub(crate) fn capacity(&self) -> Option<usize> {
        Some(self.cap)
    }

    /// Disconnects senders and wakes up all blocked receivers.
    ///
    /// Returns `true` if this call disconnected the channel.
    pub(crate) fn disconnect_senders(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);

        if tail & self.mark_bit == 0 {
            self.receivers.disconnect();
            true
        } else {
            false
        }
    }

    /// Disconnects receivers and wakes up all blocked senders.
    ///
    /// Returns `true` if this call disconnected the channel.
    ///
    /// # Safety
    /// May only be called once upon dropping the last receiver. The
    /// destruction of all other receivers must have been observed with acquire
    /// ordering or stronger.
    pub(crate) unsafe fn disconnect_receivers(&self) -> bool {
        let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);
        let disconnected = if tail & self.mark_bit == 0 {
            self.senders.disconnect();
            true
        } else {
            false
        };

        unsafe { self.discard_all_messages(tail) };
        disconnected
    }

    /// Discards all messages.
    ///
    /// `tail` should be the current (and therefore last) value of `tail`.
    ///
    /// # Panicking
    /// If a de
// ... (truncated) ...
```

**Entity:** Channel<T> (via Token/ArrayToken used by start_send/write)

**States:** NoReservation, ReservedSlot, DisconnectedReservation, SendWouldBlock(Full)

**Transitions:**
- NoReservation -> ReservedSlot via start_send() returning true and setting token.array.slot/stamp
- NoReservation -> DisconnectedReservation via start_send() setting token.array.slot = null and returning true
- NoReservation -> SendWouldBlock(Full) via start_send() returning false
- ReservedSlot -> NoReservation (logically consumed) via unsafe write() publishing stamp and waking receivers
- DisconnectedReservation -> NoReservation via write() returning Err(msg)

**Evidence:** fn start_send(&self, token: &mut Token) -> bool: returns false when full; otherwise sets token.array.slot/token.array.stamp for follow-up `write`; start_send: `if tail & self.mark_bit != 0 { token.array.slot = ptr::null(); token.array.stamp = 0; return true; }` encodes 'disconnected' as null slot; start_send: on successful CAS: `token.array.slot = slot as *const Slot<T> as *const u8; token.array.stamp = tail + 1;` (reservation materialized in token); pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T>: `if token.array.slot.is_null() { return Err(msg); }` relies on sentinel state in token; try_send/send: call ordering `if self.start_send(token) { unsafe { self.write(token, msg) } }` demonstrates required sequence

**Implementation:** Replace the ad-hoc `Token` mutation with typed reservation tokens: `fn start_send(&self) -> Result<SendPermit<'_, T>, FullOrDisconnected>` where `SendPermit` contains `&Channel<T>` + slot pointer/index + stamp. Implement `impl SendPermit { fn write(self, msg: T) -> Result<(), T> }` consuming the permit so `write` cannot be called without a prior reservation and cannot be reused. Represent disconnected/full as distinct return variants instead of `bool + null ptr`.

---

### 26. Condvar mutex-affinity protocol (Unbound -> BoundToMutex(M) and must not switch)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/poison/condvar.rs:1-431`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: A Condvar is implicitly required to be used with the same mutex across time. The first wait operation effectively 'binds' the condvar to the mutex backing the provided MutexGuard; subsequent waits must use a guard from that same mutex. Violating this is documented to potentially panic at runtime. This affinity is not represented in the type system: Condvar is not parameterized by, or otherwise tied to, a particular Mutex instance, so nothing prevents calling wait/wait_timeout with guards from different mutexes over the lifetime of one Condvar value.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct WaitTimeoutResult, impl WaitTimeoutResult (1 methods)

/// }
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Condvar {
    inner: sys::Condvar,
}

impl Condvar {
    /// Creates a new condition variable which is ready to be waited on and
    /// notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Condvar;
    ///
    /// let condvar = Condvar::new();
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_stable(feature = "const_locks", since = "1.63.0")]
    #[must_use]
    #[inline]
    pub const fn new() -> Condvar {
        Condvar { inner: sys::Condvar::new() }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// Note that this function is susceptible to spurious wakeups. Condition
    /// variables normally have a boolean predicate associated with them, and
    /// the predicate must always be checked each time this function returns to
    /// protect against spurious wakeups.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [poisoning]: super::Mutex#poisoning
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock().unwrap();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock().unwrap();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// while !*started {
    ///     started = cvar.wait(started).unwrap();
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> LockResult<MutexGuard<'a, T>> {
        let poisoned = unsafe {
            let lock = mutex::guard_lock(&guard);
            self.inner.wait(lock);
            mutex::guard_poison(&guard).get()
        };
        if poisoned { Err(PoisonError::new(guard)) } else { Ok(guard) }
    }

    /// Blocks the current thread until the provided condition becomes false.
    ///
    /// `condition` is checked immediately; if not met (returns `true`), this
    /// will [`wait`] for the next notification then check again. This repeats
    /// until `condition` returns `false`, in which case this function returns.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    /// [`wait`]: Self::wait
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [poisoning]: super::Mutex#poisoning
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    ///
    /// let pair = Arc::new((Mutex::new(true), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut pending = lock.lock().unwrap();
    ///     *pending = false;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// // As long as the value inside the `Mutex<bool>` is `true`, we wait.
    /// let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| { *pending }).unwrap();
    /// ```
    #[stable(feature = "wait_until", since = "1.42.0")]
    pub fn wait_while<'a, T, F>(
        &self,
        mut guard: MutexGuard<'a, T>,
        mut condition: F,
    ) -> LockResult<MutexGuard<'a, T>>
    where
        F: FnMut(&mut T) -> bool,
    {
        while condition(&mut *guard) {
            guard = self.wait(guard)?;
        }
        Ok(guard)
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`]
    /// except that the thread will be blocked for roughly no longer
    /// than `ms` milliseconds. This method should not be used for
    /// precise timing due to anomalies such as preemption or platform
    /// differences that might not cause the maximum amount of time
    /// waited to be precisely `ms`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned boolean is `false` only if the timeout is known
    /// to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait`]: Self::wait
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock().unwrap();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock().unwrap();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// loop {
    ///     let result = cvar.wait_timeout_ms(started, 10).unwrap();
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     started = result.0;
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break
    ///     }
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[deprecated(since = "1.6.0", note = "replaced by `std::sync::Condvar::wait_timeout`")]
    pub fn wait_timeout_ms<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        ms: u32,
    ) -> LockResult<(MutexGuard<'a, T>, bool)> {
        let res = self.wait_timeout(guard, Duration::from_millis(ms as u64));
        poison::map_result(res, |(a, b)| (a, !b.timed_out()))
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`] except that
    /// the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time. This function is susceptible to spurious wakeups.
    /// Condition variables normally have a boolean predicate associated with
    /// them, and the predicate must always be checked each time this function
    /// returns to protect against spurious wakeups. Additionally, it is
    /// typically desirable for the timeout to not exceed some duration in
    /// spite of spurious wakes, thus the sleep-duration is decremented by the
    /// amount slept. Alternatively, use the `wait_timeout_while` method
    /// to wait with a timeout while a predicate is true.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout_while`]: Self::wait_timeout_while
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock().unwrap();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // wait for the thread to start up
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock().unwrap();
    /// // as long as the value inside the `Mutex<bool>` is `false`, we wait
    /// loop {
    ///     let result = cvar.wait_timeout(started, Duration::from_millis(10)).unwrap();
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     started = result.0;
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break
    ///     }
    /// }
    /// ```
    #[stable(feature = "wait_timeout", since = "1.5.0")]
    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        dur: Duration,
    ) -> LockResult<(MutexGuard<'a, T>, WaitTimeoutResult)> {
        let (poisoned, result) = unsafe {
            let lock = mutex::guard_lock(&guard);
            let success = self.inner.wait_timeout(lock, dur);
            (mutex::guard_poison(&guard).get(), WaitTimeoutResult(!success))
        };
        if poisoned { Err(PoisonError::new((guard, result))) } else { Ok((guard, result)) }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait_while`] except
    /// that the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed without the condition being met.
    ///
    /// Like [`wait_while`], the lock specified will be re-acquired when this
    /// function returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait_while`]: Self::wait_while
    /// [`wait_timeout`]: Self::wait_timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let pair = Arc::new((Mutex::new(true), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut pending = lock.lock().unwrap();
    ///     *pending = false;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // wait for the thread to start up
    /// let (lock, cvar) = &*pair;
    /// let result = cvar.wait_timeout_while(
    ///     lock.lock().unwrap(),
    ///     Duration::from_millis(100),
    ///     |&mut pending| pending,
    /// ).unwrap();
    /// if result.1.timed_out() {
    ///     // timed-out without the condition ever evaluating to false.
    /// }
    /// // access the locked mutex via result.0
    /// ```
    #[stable(feature = "wait_timeout_until", since = "1.42.0")]
    pub fn wait_timeout_while<'a, T, F>(
        &self,
        mut guard: MutexGuard<'a, T>,
        dur: Duration,
        mut condition: F,
    ) -> LockResult<(MutexGuard<'a, T>, WaitTimeoutResult)>
    where
        F: FnMut(&mut T) -> bool,
    {
        let start = Instant::now();
        loop {
            if !condition(&mut *guard) {
                return Ok((guard, WaitTimeoutResult(false)));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return Ok((guard, WaitTimeoutResult(true))),
            };
            guard = self.wait_timeout(guard, timeout)?.0;
        }
    }

    /// Wakes up one blocked thread on this condvar.
    ///
    /// If there is a blocked thread on this condition variable, then it will
    /// be woken up from its call to [`wait`] or [`wait_timeout`]. Calls to
    /// `notify_one` are not buffered in any way.
    ///
    /// To wake up all threads, see [`notify_all`].
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout`]: Self::wait_timeout
    /// [`notify_all`]: Self::notify_all
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock().unwrap();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock().unwrap();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// while !*started {
    ///     started = cvar.wait(started).unwrap();
    /// }
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn notify_one(&self) {
        self.inner.notify_one()
    }

    /// Wakes up all blocked threads on this condvar.
    ///
    /// This method will ensure that any current waiters on the condition
    /// variable are awoken. Calls to `notify_all()` are not buffered in any
    /// way.
    ///
    /// To wake up only one thread, see [`notify_one`].
    ///
    /// [`notify_one`]: Self::notify_one
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex, Condvar};
    /// use std::thread;
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spa
// ... (truncated) ...
```

**Entity:** Condvar

**States:** Unbound, BoundToMutex(MutexId)

**Transitions:**
- Unbound -> BoundToMutex(M) via wait()/wait_timeout()/wait_while()/wait_timeout_while() (first wait establishes mutex identity)

**Evidence:** struct Condvar { inner: sys::Condvar } — no type-level association to a specific Mutex; wait(&self, guard: MutexGuard<'a, T>) calls mutex::guard_lock(&guard) and self.inner.wait(lock) — uses the mutex behind the guard; wait_timeout(&self, guard: MutexGuard<'a, T>, dur: Duration) similarly extracts lock from guard and calls self.inner.wait_timeout(lock, dur); doc on wait(): "This function may panic! if it is used with more than one mutex over time." — explicit protocol requirement enforced at runtime/OS layer

**Implementation:** Introduce an opt-in API that binds a condvar to a particular mutex at construction time, e.g., `struct BoundCondvar<'m, T> { cvar: Condvar, _m: PhantomData<&'m Mutex<T>> }` created by `Mutex::condvar(&self) -> BoundCondvar<'_, T>`. Then expose `wait(&self, guard: MutexGuard<'m, T>)` only on `BoundCondvar<'m, T>`, preventing mixing different mutexes for the same condvar at compile time (while keeping the existing unbound Condvar API for backward compatibility).

---

### 9. Context selection/packet protocol (Waiting -> Selected + optional packet)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/context.rs:1-140`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Context encodes a multi-step protocol around selecting an operation and optionally attaching a packet pointer. The intended flow is: start in Waiting; a contender calls try_select(); if that succeeds and it has a packet, it must then call store_packet() to publish the packet; the owner thread waits via wait_until() until select != Waiting. The type system does not enforce (1) that store_packet() is only called after a successful try_select(), (2) that at most one selection happens per round (enforced via atomic CAS), or (3) that reset() is called to begin a new round (done implicitly in Context::with when reusing the TLS context). Because packet is a raw pointer and 'no packet' is represented by null, the coupling between selection state and packet availability is also only implicit/runtime.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Inner


/// Thread-local context.
#[derive(Debug, Clone)]
pub struct Context {
    inner: Arc<Inner>,
}

// ... (other code) ...

    thread_id: usize,
}

impl Context {
    /// Creates a new context for the duration of the closure.
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Context) -> R,
    {
        thread_local! {
            /// Cached thread-local context.
            static CONTEXT: Cell<Option<Context>> = Cell::new(Some(Context::new()));
        }

        let mut f = Some(f);
        let mut f = |cx: &Context| -> R {
            let f = f.take().unwrap();
            f(cx)
        };

        CONTEXT
            .try_with(|cell| match cell.take() {
                None => f(&Context::new()),
                Some(cx) => {
                    cx.reset();
                    let res = f(&cx);
                    cell.set(Some(cx));
                    res
                }
            })
            .unwrap_or_else(|_| f(&Context::new()))
    }

    /// Creates a new `Context`.
    #[cold]
    fn new() -> Context {
        Context {
            inner: Arc::new(Inner {
                select: AtomicUsize::new(Selected::Waiting.into()),
                packet: AtomicPtr::new(ptr::null_mut()),
                thread: thread::current_or_unnamed(),
                thread_id: current_thread_id(),
            }),
        }
    }

    /// Resets `select` and `packet`.
    #[inline]
    fn reset(&self) {
        self.inner.select.store(Selected::Waiting.into(), Ordering::Release);
        self.inner.packet.store(ptr::null_mut(), Ordering::Release);
    }

    /// Attempts to select an operation.
    ///
    /// On failure, the previously selected operation is returned.
    #[inline]
    pub fn try_select(&self, select: Selected) -> Result<(), Selected> {
        self.inner
            .select
            .compare_exchange(
                Selected::Waiting.into(),
                select.into(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }

    /// Stores a packet.
    ///
    /// This method must be called after `try_select` succeeds and there is a packet to provide.
    #[inline]
    pub fn store_packet(&self, packet: *mut ()) {
        if !packet.is_null() {
            self.inner.packet.store(packet, Ordering::Release);
        }
    }

    /// Waits until an operation is selected and returns it.
    ///
    /// If the deadline is reached, `Selected::Aborted` will be selected.
    ///
    /// # Safety
    /// This may only be called from the thread this `Context` belongs to.
    #[inline]
    pub unsafe fn wait_until(&self, deadline: Option<Instant>) -> Selected {
        loop {
            // Check whether an operation has been selected.
            let sel = Selected::from(self.inner.select.load(Ordering::Acquire));
            if sel != Selected::Waiting {
                return sel;
            }

            // If there's a deadline, park the current thread until the deadline is reached.
            if let Some(end) = deadline {
                let now = Instant::now();

                if now < end {
                    // SAFETY: guaranteed by caller.
                    unsafe { self.inner.thread.park_timeout(end - now) };
                } else {
                    // The deadline has been reached. Try aborting select.
                    return match self.try_select(Selected::Aborted) {
                        Ok(()) => Selected::Aborted,
                        Err(s) => s,
                    };
                }
            } else {
                // SAFETY: guaranteed by caller.
                unsafe { self.inner.thread.park() };
            }
        }
    }

    /// Unparks the thread this context belongs to.
    #[inline]
    pub fn unpark(&self) {
        self.inner.thread.unpark();
    }

    /// Returns the id of the thread this context belongs to.
    #[inline]
    pub fn thread_id(&self) -> usize {
        self.inner.thread_id
    }
}

```

**Entity:** Context

**States:** Idle(Waiting, no packet), Selected(op chosen, packet maybe stored), Aborted(selected Aborted), Reset(back to Waiting, packet cleared)

**Transitions:**
- Idle(Waiting) -> Selected via try_select(Selected::X) returning Ok(())
- Selected -> Selected(with packet published) via store_packet(non-null)
- Idle(Waiting) -> Aborted via wait_until(deadline reached) which calls try_select(Selected::Aborted)
- Selected/Aborted -> Reset via reset() (called in Context::with when reusing cached Context)

**Evidence:** Inner.select: AtomicUsize initialized to Selected::Waiting in Context::new(); Inner.packet: AtomicPtr initialized to ptr::null_mut() in Context::new(); Context::try_select(): compare_exchange from Selected::Waiting into select; returns previous selection on failure; Context::store_packet(): comment: "This method must be called after `try_select` succeeds and there is a packet to provide."; Context::reset(): "Resets `select` and `packet`"; stores Waiting and null; Context::with(): when reusing Some(cx), it calls cx.reset() before running the closure, then caches it back

**Implementation:** Model the round as typestate: e.g., Context<Waiting> where try_select(self, sel) -> Result<Context<Selected>, Context<Waiting>>; then store_packet(&Context<Selected>, NonNull<()>) is only available in Selected state. reset(self) -> Context<Waiting>. Represent 'has packet' separately (Context<SelectedWithPacket>) or require store_packet to take/return a token that proves selection succeeded.

---

### 52. Entry packet pointer validity protocol (Null / NonNull, typed-to-operation)

**Location**: `/var/folders/89/0yq0xxkn04gdz8f1b0v71k4w0000gn/T/tmp.IKPnZH2PLV/src/sync/mpmc/waker.rs:1-15`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Entry carries an optional packet as a raw `*mut ()`. The code implicitly relies on a protocol where `packet` is either null (no associated packet) or non-null (there is an associated packet), and where the pointed-to allocation is valid for the duration of the Entry being used by the waker/queue. Additionally, because the pointer is untyped (`*mut ()`), there is an implicit invariant that the actual pointee type/meaning matches `oper` (e.g., only certain operations expect a packet, and the packet layout depends on the operation). None of this is enforced by the type system: nullability, lifetimes/ownership, and the relation between `oper` and `packet` are all runtime/unsafe conventions.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Waker, impl Waker (7 methods), impl Drop for Waker (1 methods), impl SyncWaker (5 methods), impl Drop for SyncWaker (1 methods); struct SyncWaker; 1 free function(s)

use crate::sync::atomic::{Atomic, AtomicBool, Ordering};

/// Represents a thread blocked on a specific channel operation.
pub(crate) struct Entry {
    /// The operation.
    pub(crate) oper: Operation,

    /// Optional packet.
    pub(crate) packet: *mut (),

    /// Context associated with the thread owning this operation.
    pub(crate) cx: Context,
}

```

**Entity:** Entry

**States:** NoPacket (packet == null), HasPacket (packet != null)

**Transitions:**
- NoPacket -> HasPacket by setting Entry.packet to a non-null value
- HasPacket -> NoPacket by clearing Entry.packet to null (if supported elsewhere)
- HasPacket(valid) -> HasPacket(dangling/invalid) if the pointee is freed too early (preventable with typed ownership)

**Evidence:** field `packet: *mut ()` is a raw untyped pointer encoding optional presence via null and requiring external lifetime/ownership discipline; comment `/// Optional packet.` documents a nullability/option invariant not represented as `Option<...>`

**Implementation:** Model the presence/absence and operation-specific payload at the type level: e.g., `struct Entry<P> { oper: OperationKind<P>, packet: P, cx: Context }` where `P` is `()` for no packet and `NonNull<T>`/`Box<T>`/`Arc<T>` for packet-carrying operations. Alternatively use `enum Packet { None, Some(NonNull<T>) }` and make `Operation` an enum whose variants carry the correctly-typed packet.

---

