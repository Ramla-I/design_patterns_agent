# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 62
- **Temporal ordering**: 2
- **Resource lifecycle**: 11
- **State machine**: 24
- **Precondition**: 5
- **Protocol**: 20
- **Modules analyzed**: 20

## Temporal Ordering Invariants

### 19. Write-guard drop protocol (PoisonFinalizeThenUnlock; single-unlock ownership across mapping)

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-139`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Write guards have an implicit required ordering at drop: poison bookkeeping must be finalized before the lock is released, and unlocking must occur exactly once even if the guard has been mapped. This ordering is implemented manually in `Drop` (calling `done` then `write_unlock`) and for mapped guards via a separate poison flag (`poison_flag.done`) plus `inner_lock.write_unlock`. The type system does not encode the 'must call done before unlock' sequencing or the 'single unlock owner after mapping' requirement; it is maintained by `Drop` implementations and SAFETY comments.

**Evidence**:

```rust

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Display> fmt::Display for MappedRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe { self.data.as_ref() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &*self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &mut *self.lock.data.get() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Deref for MappedRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Deref for MappedRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> DerefMut for MappedRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_mut() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.poison.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe {
            self.lock.inner.write_unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.poison_flag.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe {
            self.inner_lock.write_unlock();
        }
    }
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard { data, inner_lock: &orig.inner_lock }
    }
```

**Entity:** RwLockWriteGuard<'a, T> / MappedRwLockWriteGuard<'a, U>

**States:** WriteLocked (guard alive; mutable access allowed), Dropping (must finalize poison state), Unlocked (write lock released; further access invalid)

**Transitions:**
- WriteLocked -> Unlocked via Drop for RwLockWriteGuard (poison.done then write_unlock)
- WriteLocked (original) -> WriteLocked (mapped) via map/filter_map (implied by SAFETY comments mentioning map/filter_map)
- WriteLocked (mapped) -> Unlocked via Drop for MappedRwLockWriteGuard (poison_flag.done then write_unlock)

**Evidence:** Drop for RwLockWriteGuard: `self.lock.poison.done(&self.poison);` occurs before `unsafe { self.lock.inner.write_unlock(); }` (explicit required order); Drop for MappedRwLockWriteGuard: `self.poison_flag.done(&self.poison);` occurs before `unsafe { self.inner_lock.write_unlock(); }`; Deref/DerefMut for RwLockWriteGuard uses `unsafe { &*self.lock.data.get() }` / `unsafe { &mut *self.lock.data.get() }`, which assumes the write lock is still held and unique; SAFETY comments in mapped write guard deref/drop: "conditions ... upheld throughout `map` and/or `filter_map`" indicates a multi-step protocol that isn't statically checked here

**Implementation:** Move an explicit 'unlock capability' (and separately a 'poison-finalize capability') into the guard value so that mapping consumes and transfers these capabilities, making it impossible to create two values that can both unlock/finalize. This can be modeled as an internal linear token moved from `RwLockWriteGuard` into `MappedRwLockWriteGuard` (or a generic `WriteGuard<Cap>`), ensuring at compile time that `done` is called exactly once and before the single `write_unlock` in `Drop`.

---

### 32. ListToken reservation protocol (Empty/Disconnected vs ReservedSlot)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-298`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: ListToken encodes whether an operation has successfully reserved a slot in the channel. start_send()/start_recv() are responsible for populating token.list.{block,offset}. write()/read() assume the token is in ReservedSlot state and use block+offset to access a slot with get_unchecked(). If block is null, write interprets it as 'disconnected' and returns Err(msg), while read interprets it as 'disconnected' and returns Err(()). The type system does not enforce that write() is only called after a successful start_send(), nor that read() is only called after a successful start_recv() that returned ready, nor that offset is within bounds for the referenced block.

**Evidence**:

```rust
                return;
            }
        }

        // No thread is using the block, now it is safe to destroy it.
        drop(unsafe { Box::from_raw(this) });
    }
}

/// A position in a channel.
#[derive(Debug)]
struct Position<T> {
    /// The index in the channel.
    index: Atomic<usize>,

    /// The block in the linked list.
    block: Atomic<*mut Block<T>>,
}

/// The token type for the list flavor.
#[derive(Debug)]
pub(crate) struct ListToken {
    /// The block of slots.
    block: *const u8,

    /// The offset into the block.
    offset: usize,
}

impl Default for ListToken {
    #[inline]
    fn default() -> Self {
        ListToken { block: ptr::null(), offset: 0 }
    }
}

/// Unbounded channel implemented as a linked list.
///
/// Each message sent into the channel is assigned a sequence number, i.e. an index. Indices are
/// represented as numbers of type `usize` and wrap on overflow.
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
```

**Entity:** ListToken (used via Token::list)

**States:** Null (no reservation / disconnected), ReservedSlot (block != null, offset valid)

**Transitions:**
- Null -> ReservedSlot via Channel::start_send() success path (sets token.list.block/offset)
- Null -> ReservedSlot via Channel::start_recv() success path (sets token.list.block/offset)
- Any -> Null via Channel::start_send() when tail & MARK_BIT != 0 (sets token.list.block = null)
- Any -> Null via Channel::start_recv() when disconnected (sets token.list.block = null)

**Evidence:** struct ListToken { block: *const u8, offset: usize } stores raw pointer + offset as implicit state; impl Default for ListToken: block initialized to ptr::null() meaning 'no slot'; Channel::start_send(): 'If tail & MARK_BIT != 0 { token.list.block = ptr::null(); return true; }' (null token used to signal disconnected); Channel::start_send(): on success sets token.list.block = block as *const u8; token.list.offset = offset; Channel::write(): 'if token.list.block.is_null() { return Err(msg); }' and then uses (*block).slots.get_unchecked(offset); Channel::start_recv(): on disconnected sets token.list.block = ptr::null(); return true; otherwise may return false without setting a usable token ("receive operation is not ready"); Channel::read(): 'if token.list.block.is_null() { return Err(()); }' and then uses (*block).slots.get_unchecked(offset)

**Implementation:** Replace the raw ListToken with a typed reservation handle produced by start_send/start_recv, e.g. SendReservation<'a, T> / RecvReservation<'a, T> containing NonNull<Block<T>> and a bounded Offset newtype. Expose write(self, msg) only on SendReservation, and read(self) only on RecvReservation. Encode 'disconnected' and 'not ready' as distinct return types (e.g., Result<Option<RecvReservation>, Disconnected>) to prevent calling read() after a non-ready start_recv().

---

## Resource Lifecycle Invariants

### 23. Sender intrusive refcount lifecycle (Alive -> Released/MaybeFreed)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/counter.rs:1-106`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Sender is an intrusive reference to a heap-allocated Counter<C> tracked by an atomic sender refcount inside Counter. The code relies on a protocol: acquire() must be paired with exactly one release() per Sender reference, and no methods (including deref/counter()) may be used after the final release that can free the Counter. This is not enforced by the type system because Sender stores a raw pointer (*mut Counter<C>) and release() is unsafe (caller must uphold the lifetime/use-after-release rules).

**Evidence**:

```rust

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

impl<C> PartialEq for Sender<C> {
    fn eq(&self, other: &Sender<C>) -> bool {
        self.counter == other.counter
    }
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

impl<C> PartialEq for Receiver<C> {
    fn eq(&self, other: &Receiver<C>) -> bool {
        self.counter == other.counter
    }
}
```

**Entity:** Sender<C>

**States:** Alive (holds a valid counter pointer), Released (must not be used; may free Counter when last ref), Freed (counter memory deallocated)

**Transitions:**
- Alive -> Alive via acquire() (increments senders refcount)
- Alive -> Released via unsafe release() (decrements senders refcount)
- Released -> Freed when senders.fetch_sub(...) == 1 and destroy.swap(true, ...) indicates the other side already requested destruction; then Box::from_raw(counter) is dropped

**Evidence:** field: Sender { counter: *mut Counter<C> } raw pointer requires external lifetime discipline; method: fn counter(&self) -> &Counter<C> { unsafe { &*self.counter } } dereferences raw pointer without validity checks; method: pub(crate) fn acquire(&self) -> Sender<C> increments self.counter().senders via fetch_add(1, Ordering::Relaxed); comment in acquire(): cloning + mem::forget can overflow the counter; aborts when count > isize::MAX as usize; method: pub(crate) unsafe fn release(...) uses senders.fetch_sub(1, Ordering::AcqRel) == 1 as 'last sender' condition; method: release() may deallocate: if self.counter().destroy.swap(true, Ordering::AcqRel) { drop(Box::from_raw(self.counter)) }; impl Deref for Sender returns &self.counter().chan, which becomes invalid if Counter is freed

**Implementation:** Make Sender own a non-null pointer (NonNull<Counter<C>>) plus implement Clone to call acquire() and Drop to call release() with a stored disconnect action or a Counter-provided vtable/callback. This removes the need for unsafe release() by ensuring exactly-once decrement in Drop, and prevents using a logically-released Sender because it is consumed/dropped. If disconnect requires a one-shot FnOnce, store disconnect logic inside Counter so Drop can call it without caller-provided closure.

---

### 7. MutexGuard thread-affinity + lock-held protocol

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-115`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: MutexGuard represents the state of a mutex being locked, and (by drop) transitions to the unlocked state. Additionally, on some platforms (pthreads) there is an implicit thread-affinity requirement: the guard must be dropped/unlocked on the same thread that acquired it. The API relies on an auto-trait negative impl (!Send) and documentation to prevent cross-thread drop; this is not expressed as an explicit 'acquired-on-thread X' capability/token, and the lock/unlock protocol is enforced primarily by RAII + trait bounds rather than a first-class type-level session/protocol that could also encode additional context (e.g., which mutex, which thread).

**Evidence**:

```rust
/// this manner. For instance, consider [`Rc`], a non-atomic reference counted smart pointer,
/// which is not `Send`. With `Rc`, we can have multiple copies pointing to the same heap
/// allocation with a non-atomic reference count. If we were to use `Mutex<Rc<_>>`, it would
/// only protect one instance of `Rc` from shared access, leaving other copies vulnerable
/// to potential data races.
///
/// Also note that it is not necessary for `T` to be `Sync` as `&T` is only made available
/// to one thread at a time if `T` is not `Sync`.
///
/// [`Rc`]: crate::rc::Rc
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`lock`]: Mutex::lock
/// [`try_lock`]: Mutex::try_lock
#[must_use = "if unused the Mutex will immediately unlock"]
#[must_not_suspend = "holding a MutexGuard across suspend \
                      points can cause deadlocks, delays, \
                      and cause Futures to not implement `Send`"]
#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "MutexGuard")]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    poison: poison::Guard,
}

/// A [`MutexGuard`] is not `Send` to maximize platform portablity.
///
/// On platforms that use POSIX threads (commonly referred to as pthreads) there is a requirement to
/// release mutex locks on the same thread they were acquired.
/// For this reason, [`MutexGuard`] must not implement `Send` to prevent it being dropped from
/// another thread.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

/// `T` must be `Sync` for a [`MutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
#[stable(feature = "mutexguard", since = "1.19.0")]
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a
/// subfield of the protected data. When this structure is dropped (falls out
/// of scope), the lock will be unlocked.
///
/// The main difference between `MappedMutexGuard` and [`MutexGuard`] is that the
/// former cannot be used with [`Condvar`], since that
/// could introduce soundness issues if the locked object is modified by another
/// thread while the `Mutex` is unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`map`] and [`filter_map`] methods on
/// [`MutexGuard`].
///
/// [`map`]: MutexGuard::map
/// [`filter_map`]: MutexGuard::filter_map
/// [`Condvar`]: crate::sync::Condvar
#[must_use = "if unused the Mutex will immediately unlock"]
#[must_not_suspend = "holding a MappedMutexGuard across suspend \
                      points can cause deadlocks, delays, \
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
```

**Entity:** MutexGuard<'a, T>

**States:** LockHeld(on acquiring thread), Dropped(UnlockPerformed)

**Transitions:**
- LockHeld(on acquiring thread) -> Dropped(UnlockPerformed) via Drop of MutexGuard

**Evidence:** field: `MutexGuard { lock: &'a Mutex<T>, poison: poison::Guard }` — guard exists only while lock is held; attribute: `#[must_use = "if unused the Mutex will immediately unlock"]` — implies a lock-held/then-unlocked lifecycle tied to dropping the guard; comment: "release mutex locks on the same thread they were acquired" — thread-affinity protocol is relied upon; impl: `impl<T: ?Sized> !Send for MutexGuard<'_, T> {}` — encoding the thread-affinity requirement indirectly via auto-traits; attribute: `#[must_not_suspend = "holding a MutexGuard across suspend points ..."]` — implies an additional temporal constraint about when the guard may be held

**Implementation:** Model acquisition as producing a non-transferable capability tied to the acquiring context, e.g., `lock(&self, ctx: ThreadToken) -> Guard<'_, 'ctx, T>` where `ThreadToken` is !Send and created per-thread, and `Guard` carries `PhantomData<&'ctx ThreadToken>`. This makes the 'must be dropped on same thread' rule an explicit capability relationship rather than only an auto-trait property of the guard itself.

---

### 24. Receiver intrusive refcount lifecycle (Alive -> Released/MaybeFreed)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/counter.rs:1-106`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Receiver mirrors Sender: it is an intrusive reference to Counter<C> with an atomic receiver refcount. The implicit protocol is that each acquired Receiver must be released exactly once, and Receiver must not be used after its final release since that release may free the shared Counter allocation (depending on the shared destroy flag). This is enforced only by unsafe and convention, not by the type system, because Receiver stores a raw pointer and exposes safe methods (like Deref) that assume the pointer is still valid.

**Evidence**:

```rust

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

impl<C> PartialEq for Sender<C> {
    fn eq(&self, other: &Sender<C>) -> bool {
        self.counter == other.counter
    }
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

impl<C> PartialEq for Receiver<C> {
    fn eq(&self, other: &Receiver<C>) -> bool {
        self.counter == other.counter
    }
}
```

**Entity:** Receiver<C>

**States:** Alive (holds a valid counter pointer), Released (must not be used; may free Counter when last ref), Freed (counter memory deallocated)

**Transitions:**
- Alive -> Alive via acquire() (increments receivers refcount)
- Alive -> Released via unsafe release() (decrements receivers refcount)
- Released -> Freed when receivers.fetch_sub(...) == 1 and destroy.swap(true, ...) is true; then Box::from_raw(counter) is dropped

**Evidence:** field: Receiver { counter: *mut Counter<C> } raw pointer; method: fn counter(&self) -> &Counter<C> { unsafe { &*self.counter } }; method: pub(crate) fn acquire(&self) -> Receiver<C> increments self.counter().receivers via fetch_add(1, Ordering::Relaxed); comment in acquire(): mem::forget overflow scenario; abort when count > isize::MAX as usize; method: pub(crate) unsafe fn release(...) uses receivers.fetch_sub(1, Ordering::AcqRel) == 1 as 'last receiver' condition; method: release() may deallocate: if self.counter().destroy.swap(true, Ordering::AcqRel) { drop(Box::from_raw(self.counter)) }; impl Deref for Receiver returns &self.counter().chan, which becomes invalid if Counter is freed

**Implementation:** As with Sender, implement Clone to call acquire() and Drop to call release() automatically, using NonNull<Counter<C>> instead of *mut. Move the disconnect behavior into Counter (or into a shared Arc-like inner) so Drop can safely perform it without an unsafe caller-provided closure, eliminating the unsafe release() API and reducing UAF risk.

---

### 34. Block slot lifecycle and destruction protocol (Allocated -> Writing -> Readable -> Read -> Destroying -> Freed)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-298`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Block memory is manually managed: pointers are stored in atomics, messages are written into MaybeUninit slots, and blocks are eventually freed via drop(Box::from_raw(ptr)). Correctness relies on a protocol: a sender must reserve a slot (token provides block+offset) before writing; a receiver must wait for WRITE before reading; destruction of a block must only happen when no thread is still accessing relevant slots, coordinated via slot.state bits (WRITE/READ/DESTROY) and special end-of-block handling. These constraints are enforced by runtime atomic flags, null checks, and unsafe pointer operations rather than type-level ownership/borrowing, so misuse (e.g., using a stale token after destruction) would be UB but is not prevented by signatures.

**Evidence**:

```rust
                return;
            }
        }

        // No thread is using the block, now it is safe to destroy it.
        drop(unsafe { Box::from_raw(this) });
    }
}

/// A position in a channel.
#[derive(Debug)]
struct Position<T> {
    /// The index in the channel.
    index: Atomic<usize>,

    /// The block in the linked list.
    block: Atomic<*mut Block<T>>,
}

/// The token type for the list flavor.
#[derive(Debug)]
pub(crate) struct ListToken {
    /// The block of slots.
    block: *const u8,

    /// The offset into the block.
    offset: usize,
}

impl Default for ListToken {
    #[inline]
    fn default() -> Self {
        ListToken { block: ptr::null(), offset: 0 }
    }
}

/// Unbounded channel implemented as a linked list.
///
/// Each message sent into the channel is assigned a sequence number, i.e. an index. Indices are
/// represented as numbers of type `usize` and wrap on overflow.
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
```

**Entity:** Block<T> (as used by Channel::read/write and Block::destroy)

**States:** Allocated/Alive, Slot reserved (index assigned), Message written (WRITE set), Message read (READ set), Destroy requested (DESTROY set), Destroyed/Freed

**Transitions:**
- Allocated/Alive -> Message written via Channel::write() (slot.msg.write + slot.state |= WRITE)
- Message written -> Message read via Channel::read() (slot.wait_write then msg.read().assume_init + slot.state |= READ)
- Any Alive -> Destroyed/Freed via Block::destroy(block, ...) which ultimately does drop(unsafe { Box::from_raw(this) }) when safe

**Evidence:** Channel::write(): writes into `slot.msg.get().write(MaybeUninit::new(msg))` and sets `slot.state.fetch_or(WRITE, Ordering::Release)`; Channel::read(): `slot.wait_write(); let msg = slot.msg.get().read().assume_init();` then conditionally calls Block::destroy(...) based on offset and `slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY`; Top snippet in Block::destroy path: comment 'No thread is using the block, now it is safe to destroy it.' followed by `drop(unsafe { Box::from_raw(this) });` (manual free behind protocol); Multiple uses of raw pointers to blocks: Position<T>::block: Atomic<*mut Block<T>>; ListToken::block used as *mut Block<T> in read/write

**Implementation:** Make the reservation token a linear capability tied to the block's lifetime (e.g., holds Arc<BlockInner> or an epoch-protected guard) so the block cannot be freed while capabilities exist. Alternatively, wrap Block pointers in a safe reclamation scheme handle (hazard pointer/epoch guard type) returned by start_send/start_recv, and require that guard to call read/write, preventing use-after-free at the type level.

---

### 51. MappedMutexGuard lock/poison lifecycle protocol (Locked -> Unlocked, poison completion on drop)

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-67`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: MappedMutexGuard represents an active, locked mutex region plus an in-progress poison tracking context. While the guard is alive, the underlying mutex must remain locked and the internal pointers/references (data, inner, poison_flag) must remain valid. On Drop, the guard must (1) finalize poisoning via poison_flag.done(&poison) and then (2) unlock the mutex via inner.unlock(). This protocol is encoded by raw/pointer-like fields and unsafe deref implementations, and relies on Drop order for correctness; the type system does not express that `data` is only valid while the lock is held, nor that `done()` must be called exactly once before `unlock()`.

**Evidence**:

```rust
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
impl<T: ?Sized> Deref for MappedMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> DerefMut for MappedMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.poison_flag.done(&self.poison);
            self.inner.unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for MappedMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Display> fmt::Display for MappedMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: ?Sized> MappedMutexGuard<'a, T> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedMutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(mut orig: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
```

**Entity:** MappedMutexGuard<'a, T>

**States:** Locked (guard alive, mutex held), Unlocked (guard dropped, mutex released)

**Transitions:**
- Locked -> Unlocked via Drop::drop()

**Evidence:** impl Deref for MappedMutexGuard: `unsafe { self.data.as_ref() }` (data validity depends on lock still being held); impl DerefMut for MappedMutexGuard: `unsafe { self.data.as_mut() }` (mutable access relies on exclusivity from the lock); impl Drop for MappedMutexGuard: `self.poison_flag.done(&self.poison); self.inner.unlock();` (required ordering and exactly-once protocol performed at runtime); construction snippet: `inner: &orig.lock.inner, poison_flag: &orig.lock.poison, poison: orig.poison.clone()` (guard carries borrowed lock internals; validity depends on lock lifetime discipline)

**Implementation:** Make the unlock/poison-finalization sequence unrepresentable to break: encapsulate `inner` and `poison_flag` in a single private RAII field (e.g., `struct HeldLock<'a>{ inner: &'a Inner, poison_flag: &'a PoisonFlag, poison: Poison }`) whose Drop performs `done()` then `unlock()`. Keep `data` tied to that RAII field so `Deref`/`DerefMut` cannot exist without the held-lock token. This strengthens the invariant that the data pointer is only usable while the lock-holding capability exists and that finalization happens exactly once.

---

### 9. MutexGuard lifecycle protocol (Locked -> Dropped/Unlocked, poison bookkeeping)

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-113`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: A MutexGuard value represents exclusive access to the mutex-protected data and implies that the underlying OS mutex is currently locked. While LiveGuardHoldingLock, deref/deref_mut may access the protected data via raw pointer dereference. On Drop, the guard must (1) record poison status via poison.done(...) and then (2) unlock the underlying mutex. This ordering and the requirement that unlock occurs exactly once are enforced by convention/Drop and unsafe code, not by a type-level state transition that prevents misuse in unsafe/internal APIs (e.g., constructing a guard without holding the lock, or calling unlock without matching poison bookkeeping).

**Evidence**:

```rust

#[stable(feature = "mutex_default", since = "1.10.0")]
impl<T: ?Sized + Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                d.field("data", &&*guard);
            }
            Err(TryLockError::Poisoned(err)) => {
                d.field("data", &&**err.get_ref());
            }
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            }
        }
        d.field("poisoned", &self.poison.get());
        d.finish_non_exhaustive()
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

pub fn guard_lock<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a sys::Mutex {
    &guard.lock.inner
}

pub fn guard_poison<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a poison::Flag {
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
```

**Entity:** MutexGuard<'a, T>

**States:** LiveGuardHoldingLock, DroppedGuardLockReleased

**Transitions:**
- LiveGuardHoldingLock -> DroppedGuardLockReleased via Drop::drop

**Evidence:** MutexGuard::new(lock): unsafe constructor; relies on external precondition that the mutex is locked/guarded correctly: `unsafe fn new(lock: &'mutex Mutex<T>) -> LockResult<MutexGuard<'mutex, T>>`; Deref::deref uses raw access to protected data without re-checking lock state: `unsafe { &*self.lock.data.get() }`; DerefMut::deref_mut similarly: `unsafe { &mut *self.lock.data.get() }`; Drop::drop performs required cleanup and ordering: `self.lock.poison.done(&self.poison); self.lock.inner.unlock();`

**Implementation:** Make the unsafe construction path require a capability token that can only be obtained by actually locking `lock.inner` (e.g., `struct Locked(sys::MutexGuardLike)`), then define `MutexGuard<'a, T> { lock: &'a Mutex<T>, _cap: Locked, poison: poison::Guard }`. This ties the ability to create a MutexGuard (and thus access `data.get()`) to possession of an unforgeable lock capability, ensuring at compile time that deref is only possible when the lock is held and that unlock happens exactly once via Drop of the capability.

---

### 59. Thread-local checkout/return invariant (take -> use -> put back)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/context.rs:1-65`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: A Context instance is treated as a thread-local reusable resource stored in a cell (Option-like) and temporarily removed with take() for use. The intended lifecycle is: take the Context (CheckedOut), reset it, run a closure with a reference, then return it to the cell with set(Some(cx)) so it can be reused. This borrowing/return protocol is enforced manually by the closure structure and is not represented as a type-level guard; if the code evolves, it is easy to accidentally forget to return the context or to expose it beyond the intended scope.

**Evidence**:

```rust
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
```

**Entity:** Context (thread-local reuse via CONTEXT)

**States:** InCell(Available), CheckedOut(NotInCell)

**Transitions:**
- InCell(Available) -> CheckedOut(NotInCell) via cell.take() yielding Some(cx)
- CheckedOut(NotInCell) -> InCell(Available) via cell.set(Some(cx)) after use
- CheckedOut(NotInCell) -> InCell(Available) (fresh) via Context::new() path (when None)

**Evidence:** CONTEXT.try_with(|cell| match cell.take() { ... }) uses cell.take() to remove the stored Context; Some(cx) arm: cx.reset(); let res = f(&cx); cell.set(Some(cx)); res (manual return to the cell); None arm: None => f(&Context::new()) (fresh instance used when not available)

**Implementation:** Return an RAII guard from a `with_context`/`checkout` function: `struct ContextGuard { cx: Context, cell: &'a Cell<Option<Context>> }` with `Drop` that performs `reset()` and `cell.set(Some(cx))`. Expose `Deref<Target=Context>` for use within scope, making 'return to TLS' non-forgettable.

---

### 17. ReentrantLockGuard lock ownership & reentrancy protocol (Held with count > 0 -> Released at count == 0)

**Location**: `/tmp/sync_test_crate/src/sync/reentrant_lock.rs:1-41`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ReentrantLockGuard’s Drop relies on the implicit invariant that the guard 'owns the lock' and that `lock_count` accurately tracks the number of active reentrant acquisitions by the current owner thread. Dropping a guard decrements `lock_count`; only when it reaches 0 does it clear `owner` and unlock the underlying mutex. This is a runtime-encoded state machine (count and owner) that must stay consistent: guards must not be duplicated/dropped more times than acquired, and the `owner` must correspond to the thread performing the final release. The type system does not encode the reentrancy depth, nor does it encode that this guard corresponds to the current owner thread; correctness relies on the unsafe block + internal fields being maintained consistently by construction.

**Evidence**:

```rust
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

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: fmt::Debug + ?Sized> fmt::Debug for ReentrantLockGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[unstable(feature = "reentrant_lock", issue = "121440")]
impl<T: fmt::Display + ?Sized> fmt::Display for ReentrantLockGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
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

**Entity:** ReentrantLockGuard<'_, T>

**States:** Held(count>0), Released(count==0)

**Transitions:**
- Held(count=n>1) -> Held(count=n-1) via Drop::drop()
- Held(count=1) -> Released(count=0) via Drop::drop() (also clears owner and unlocks mutex)

**Evidence:** Drop::drop(): comment `// Safety: We own the lock.` indicates an implicit precondition not expressed in types; Drop::drop(): `*self.lock.lock_count.get() -= 1;` shows runtime reentrancy depth tracking; Drop::drop(): `if *self.lock.lock_count.get() == 0 { self.lock.owner.set(None); self.lock.mutex.unlock(); }` shows the release transition is conditional on reaching depth 0; Deref::deref(): returns `&self.lock.data` without additional checks, implying the guard must represent a valid held-lock state while it exists

**Implementation:** Model the lock as producing a guard type that carries a capability tying it to the owning thread (e.g., a non-Send token or thread-id capability) and/or split guard variants so only a 'final release' guard can transition the lock to unlocked. Alternatively, encapsulate the decrement-to-zero logic into a dedicated internal type that cannot be cloned/duplicated, ensuring the only way to reach unlock is via the unique Drop path.

---

### 27. Slot message initialization lifecycle (Empty -> Full -> Empty) encoded via stamp + MaybeUninit

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-64`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Each Slot stores `msg: UnsafeCell<MaybeUninit<T>>` and relies on `stamp: Atomic<usize>` to indicate whether `msg` is currently initialized (contains a message to be read) or uninitialized (empty). The comment states the message is 'Either read out in `read` or dropped through `discard_all_messages`', implying exactly-once ownership transfer/drop semantics gated by stamp changes. This empty/full lifecycle is enforced by runtime stamp protocols and unsafe memory discipline rather than by types, so incorrect stamp manipulation could cause double-drop or reading uninitialized memory.

**Evidence**:

```rust
use super::select::{Operation, Selected, Token};
use super::utils::{Backoff, CachePadded};
use super::waker::SyncWaker;
use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::ptr;
use crate::sync::atomic::{self, Atomic, AtomicUsize, Ordering};
use crate::time::Instant;

/// A slot in a channel.
struct Slot<T> {
    /// The current stamp.
    stamp: Atomic<usize>,

    /// The message in this slot. Either read out in `read` or dropped through
    /// `discard_all_messages`.
    msg: UnsafeCell<MaybeUninit<T>>,
}

/// The token type for the array flavor.
#[derive(Debug)]
pub(crate) struct ArrayToken {
    /// Slot to read from or write to.
    slot: *const u8,

    /// Stamp to store into the slot after reading or writing.
    stamp: usize,
}

impl Default for ArrayToken {
    #[inline]
    fn default() -> Self {
        ArrayToken { slot: ptr::null(), stamp: 0 }
    }
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
```

**Entity:** Slot<T>

**States:** Empty (no initialized T in msg), Full (msg contains initialized T)

**Transitions:**
- Empty -> Full via write/send that initializes `msg` and updates `stamp` (methods referenced by comment but not shown)
- Full -> Empty via `read` that takes/drops the message and updates `stamp`
- Full -> Empty via `discard_all_messages` dropping the message without reading

**Evidence:** field `Slot::msg: UnsafeCell<MaybeUninit<T>>` indicates manual initialization tracking outside the type system; field `Slot::stamp: Atomic<usize>` comment: 'The current stamp' used to coordinate slot state; comment on `Slot::msg`: 'Either read out in `read` or dropped through `discard_all_messages`' indicates an implicit ownership/lifecycle protocol

**Implementation:** Encapsulate `Slot` behind methods that return typed guards representing `Full` vs `Empty` states (e.g., `SlotRef<Full>` that can `take()` producing `T` and transitions to `Empty`). Internally keep stamp+MaybeUninit, but make the safe API require a proof token/capability that the slot is full before allowing read/drop, and full before allowing discard. Alternatively, use a private `enum SlotState<T> { Empty, Full(T) }` for non-lockfree variants, but for lockfree keep the representation and enforce state transitions via sealed types/guards.

---

### 63. Head block ownership/NULLing protocol during discard_all_messages vs Drop (OwnsHeadBlock / HeadBlockTaken)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-186`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: There is an implicit ownership transfer protocol for the heap-allocated block chain pointed to by `self.head.block`. discard_all_messages() atomically swaps `head.block` to null to take ownership of the allocation and then deallocates blocks while dropping messages. A critical invariant (stated in comments) is that any code path that attempts to deallocate `head.block` must also set it to NULL; otherwise, Drop will later see a non-null pointer and double-free. This is enforced by comments and careful use of `swap(ptr::null_mut())`, not by the type system; `Drop` and `discard_all_messages` both operate on raw pointers and rely on this temporal/ownership protocol.

**Evidence**:

```rust
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
        // will be deallocated by the sender in Drop.
        let mut block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);

        // If we're going to be dropping messages we need to synchronize with initialization
        if head >> SHIFT != tail >> SHIFT {
            // The block can be null here only if a sender is in the process of initializing the
            // channel while another sender managed to send a message by inserting it into the
            // semi-initialized channel and advanced the tail.
            // In that case, just wait until it gets initialized.
            while block.is_null() {
                backoff.spin_heavy();
                block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);
            }
        }
        // After this point `head.block` is not modified again and it will be deallocated if it's
        // non-null. The `Drop` code of the channel, which runs after this function, also attempts
        // to deallocate `head.block` if it's non-null. Therefore this function must maintain the
        // invariant that if a deallocation of head.block is attemped then it must also be set to
        // NULL. Failing to do so will lead to the Drop code attempting a double free. For this
        // reason both reads above do an atomic swap instead of a simple atomic load.

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head >> SHIFT != tail >> SHIFT {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    slot.wait_write();
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    (*block).wait_next();
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Acquire);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }

        head &= !MARK_BIT;
        self.head.index.store(head, Ordering::Release);
    }

    /// Returns `true` if the channel is disconnected.
    pub(crate) fn is_disconnected(&self) -> bool {
        self.tail.index.load(Ordering::SeqCst) & MARK_BIT != 0
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.index.load(Ordering::SeqCst);
        let tail = self.tail.index.load(Ordering::SeqCst);
        head >> SHIFT == tail >> SHIFT
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        false
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        let mut head = self.head.index.load(Ordering::Relaxed);
        let mut tail = self.tail.index.load(Ordering::Relaxed);
        let mut block = self.head.block.load(Ordering::Relaxed);

        // Erase the lower bits.
        head &= !((1 << SHIFT) - 1);
        tail &= !((1 << SHIFT) - 1);

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head != tail {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Relaxed);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }
    }
}
```

**Entity:** Channel<T>

**States:** OwnsHeadBlock (head.block may be non-null and must be freed by Drop), HeadBlockTaken (head.block swapped to null; discard_all_messages owns responsibility)

**Transitions:**
- OwnsHeadBlock -> HeadBlockTaken via `discard_all_messages(): self.head.block.swap(ptr::null_mut(), Ordering::AcqRel)`
- HeadBlockTaken -> (freed) via deallocating blocks with `drop(Box::from_raw(block))`

**Evidence:** discard_all_messages(): `let mut block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);` — taking ownership by nulling the atomic pointer; comment in discard_all_messages(): `this function must maintain the invariant that if a deallocation of head.block is attempted then it must also be set to NULL... otherwise ... double free` — explicit latent invariant; comment: `For this reason both reads above do an atomic swap instead of a simple atomic load.` — indicates required protocol not captured by types; Drop for Channel<T>: `let mut block = self.head.block.load(Ordering::Relaxed); ... drop(Box::from_raw(block));` — Drop assumes it owns/free's the chain when non-null, hence the need for the NULLing protocol

**Implementation:** Wrap `head.block` in an owned RAII handle that enforces single-owner semantics when extracting the pointer, e.g., store an `Option<NonNull<Block<T>>>` behind synchronization and provide a safe `take_head_block()` API returning a `BlockChainOwner` guard that deallocates on Drop. Ensure `Drop for Channel` only frees if it still holds the owner, making double-free structurally impossible.

---

### 30. Receiver reference-counted lifecycle + disconnect-on-last-drop protocol

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/mod.rs:1-63`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: Receiver<T> participates in a shared channel lifetime managed manually via acquire()/release() on the underlying channel implementation (Array/List/Zero). Cloning must increment the shared ownership count, and dropping must decrement it; when the last Receiver is dropped, the implementation disconnects receivers (or disconnects for zero-capacity). This is enforced by runtime/refcounting inside chan.acquire()/chan.release() and by unsafe code, rather than by a type-level ownership token that statically guarantees 'every acquire has a matching release' and that disconnect happens exactly once at the correct time.

**Evidence**:

```rust
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

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        unsafe {
            match &self.flavor {
                ReceiverFlavor::Array(chan) => chan.release(|c| c.disconnect_receivers()),
                ReceiverFlavor::List(chan) => chan.release(|c| c.disconnect_receivers()),
                ReceiverFlavor::Zero(chan) => chan.release(|c| c.disconnect()),
            }
        }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let flavor = match &self.flavor {
            ReceiverFlavor::Array(chan) => ReceiverFlavor::Array(chan.acquire()),
            ReceiverFlavor::List(chan) => ReceiverFlavor::List(chan.acquire()),
            ReceiverFlavor::Zero(chan) => ReceiverFlavor::Zero(chan.acquire()),
        };

        Receiver { flavor }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Receiver { .. }")
    }
}

#[cfg(test)]
mod tests;
```

**Entity:** Receiver<T> (and its internal ReceiverFlavor)

**States:** Live (connected, refcount >= 1), Dropping (about to release), Disconnected (receivers disconnected / channel shutdown initiated)

**Transitions:**
- Live -> Live via Clone::clone() (acquire another receiver handle)
- Live -> Dropping via Drop::drop() (release one receiver handle)
- Dropping -> Disconnected via chan.release(|c| c.disconnect_receivers()/disconnect()) when last receiver is released

**Evidence:** Drop for Receiver<T>: match &self.flavor { ... => chan.release(|c| c.disconnect_receivers()), ... Zero(chan) => chan.release(|c| c.disconnect()) }; Clone for Receiver<T>: ReceiverFlavor::{Array/List/Zero}(chan.acquire()); unsafe block in Drop::drop(): indicates correctness depends on hidden invariants of acquire/release/disconnect ordering

**Implementation:** Introduce an internal RAII handle (e.g., struct RxHandle { inner: NonNull<...> }) whose Drop performs the appropriate release+possible-disconnect, and make clone only possible via an explicit shared-ownership primitive (Arc-like) so that the 'acquire/release must balance' invariant is encoded structurally. Optionally, represent 'connected' vs 'disconnected' with a typestate/capability token if public APIs have different validity after disconnect.

---

## State Machine Invariants

### 4. SyncWaker emptiness cache protocol (is_empty must mirror inner queues)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/waker.rs:1-209`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: SyncWaker maintains a cached `is_empty` AtomicBool intended to reflect whether the inner Waker has any selectors/observers, and uses it as a fast-path to avoid locking. Correctness/performance depends on a protocol: every mutating operation on `inner` must update `is_empty` consistently, and notify() relies on a double-checked pattern (`if !is_empty` then lock then re-check) to tolerate races. This invariant is not enforced by types; future methods could mutate `inner` without updating the cache, or update it incorrectly, breaking the fast-path and the Drop assertion.

**Evidence**:

```rust
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

**States:** CacheAccurateEmpty, CacheAccurateNonEmpty, CacheStale (temporarily possible between operations)

**Transitions:**
- CacheAccurateEmpty -> CacheAccurateNonEmpty via register() (push into selectors)
- CacheAccurateNonEmpty -> CacheAccurateEmpty via unregister()/notify()/disconnect() when inner queues become empty
- CacheAccurateNonEmpty -> CacheAccurateNonEmpty via notify() when entries remain
- Any -> CacheStale if inner is mutated without updating is_empty (latent, prevented only by convention)

**Evidence:** field: `is_empty: Atomic<bool>` described as "true if the waker is empty" (cached state); register(): updates cache with `store(inner.selectors.is_empty() && inner.observers.is_empty(), Ordering::SeqCst)` after mutating inner; unregister(): same cache update pattern after removing an entry; notify(): uses `if !self.is_empty.load(...) { lock; if !self.is_empty.load(...) { ...; self.is_empty.store(...) } }` (double-check protocol depends on cache semantics); disconnect(): mutates inner then updates is_empty similarly; Drop for SyncWaker: `debug_assert!(self.is_empty.load(Ordering::SeqCst))` relies on cache being accurate at destruction

**Implementation:** Hide `inner: Mutex<Waker>` behind a private method that returns a guard/capability which updates `is_empty` on drop (e.g., `struct InnerGuard<'a> { guard: MutexGuard<'a, Waker>, is_empty: &'a AtomicBool }` with Drop recomputing emptiness). This makes it impossible to mutate `inner` without also refreshing the cache, turning the convention into a compile-time enforced access path.

---

### 1. Barrier rendezvous generation protocol (per-round Waiting -> Released)

**Location**: `/tmp/sync_test_crate/src/sync/barrier.rs:1-85`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Barrier encodes a cyclic rendezvous protocol: threads repeatedly enter a 'generation' by calling wait(), incrementing an internal count until the configured num_threads is reached, at which point all waiters are released and the barrier advances to the next generation (generation_id). This multi-round state machine is tracked entirely at runtime via BarrierState { count, generation_id } guarded by a Mutex/Condvar; the type system does not represent 'which generation' a thread is participating in, nor does it prevent misuse such as constructing a Barrier with num_threads == 0 (a configuration that cannot ever successfully rendezvous).

**Evidence**:

```rust
/// A barrier enables multiple threads to synchronize the beginning
/// of some computation.
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
pub struct Barrier {
    lock: Mutex<BarrierState>,
    cvar: Condvar,
    num_threads: usize,
}

// The inner state of a double barrier
struct BarrierState {
    count: usize,
    generation_id: usize,
}

/// A `BarrierWaitResult` is returned by [`Barrier::wait()`] when all threads
/// in the [`Barrier`] have rendezvoused.
///
/// # Examples
///
/// ```
/// use std::sync::Barrier;
///
/// let barrier = Barrier::new(1);
/// let barrier_wait_result = barrier.wait();
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
pub struct BarrierWaitResult(bool);

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Barrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Barrier").finish_non_exhaustive()
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
```

**Entity:** Barrier

**States:** GenerationOpen (collecting waiters), GenerationComplete (release in progress / next generation begins)

**Transitions:**
- GenerationOpen -> GenerationComplete when count reaches num_threads (via wait())
- GenerationComplete -> GenerationOpen by incrementing generation_id and resetting count for the next round (via wait())

**Evidence:** field `lock: Mutex<BarrierState>`: runtime-mutated shared state; struct `BarrierState { count: usize, generation_id: usize }`: explicit runtime state machine variables (arrival count + generation tracking); field `num_threads: usize`: the rendezvous threshold used to decide when a generation completes; doc comment on Barrier::new: "A barrier will block `n`-1 threads ... wake up all threads at once when the `n`th thread calls wait()" (describes threshold-based state transition); doc comment on wait(): "Blocks the current thread until all threads have rendezvoused here." (implies the multi-party rendezvous protocol, enforced via runtime synchronization primitives rather than types)

**Implementation:** Introduce a `NonZeroUsize` (or `BarrierSize(NonZeroUsize)`) parameter for `Barrier::new` to make `n > 0` a compile-time-checked API constraint (callers must construct a non-zero size). Full typestate for per-generation participation is not realistically expressible for arbitrary threads, but the configuration validity (non-zero participant count) is enforceable.

---

### 29. Receiver flavor protocol (Array / List / Zero) with capacity-dependent semantics

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/mod.rs:1-101`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Receiver<T> has an implicit runtime state encoded by `self.flavor` (Array/List/Zero). The semantics of inspection methods like `is_empty()`, `is_full()`, and `len()` depend on that flavor; in particular, zero-capacity channels have special invariants: they are always empty and always full. This state is only dispatched at runtime via `match &self.flavor` and is not reflected in the type system, so callers cannot express (or have the compiler enforce) that they are working with a zero-capacity vs buffered vs unbounded receiver, even though the meaning of these queries differs by state.

**Evidence**:

```rust
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
```

**Entity:** Receiver<T>

**States:** Array-backed channel, List-backed channel, Zero-capacity channel

**Evidence:** try_iter(): constructs `TryIter { rx: self }`, tying iterator behavior to the receiver's runtime flavor; Receiver::is_empty(): `match &self.flavor { ReceiverFlavor::Array(..) | List(..) | Zero(..) => ... }` shows runtime state dispatch; Doc comment on is_empty(): "Note: Zero-capacity channels are always empty." indicates flavor-specific invariant; Receiver::is_full(): `match &self.flavor { ... }` shows runtime state dispatch; Doc comment on is_full(): "Note: Zero-capacity channels are always full." indicates flavor-specific invariant; Receiver::len(): `match &self.flavor { ... }` shows runtime state dispatch and flavor-dependent meaning of length

**Implementation:** Make the channel kind a type parameter: `struct Receiver<T, K> { inner: K }` where `K` is one of `ArrayChan`, `ListChan`, `ZeroChan` (or `Receiver<Buffered<_>>`/`Receiver<Unbounded<_>>`/`Receiver<Zero<_>>`). Implement `is_empty/is_full/len` per kind; for `ZeroChan`, `is_empty()` and `is_full()` can be `const fn` returning `true` and `len()` returning `0`, eliminating the runtime match and making capacity-dependent semantics explicit in types.

---

### 11. Once initialization/poisoning state machine (Incomplete / Running / Complete / Poisoned)

**Location**: `/tmp/sync_test_crate/src/sync/poison/once.rs:1-123`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Once has an implicit state machine governing initialization progress and poisoning. Some operations are only valid/meaningful in certain states: is_completed() queries completion; wait() blocks until completion but will panic if the Once is poisoned; wait_force() blocks ignoring poisoning. Internally, state()/set_state() rely on an additional temporal invariant: because they take &mut self, initialization cannot be running concurrently, so the state is restricted to a subset (Incomplete/Poisoned/Complete). None of these state distinctions are represented in the public type of Once, so callers can only discover invalid states via runtime behavior (panic) and boolean queries.

**Evidence**:

```rust
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for Once {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Once").finish_non_exhaustive()
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
```

**Entity:** Once

**States:** Incomplete, Running, Complete, Poisoned

**Transitions:**
- Incomplete -> Running via call_once(...) (implied by docs/examples and internal 'running' mention)
- Running -> Complete when initialization finishes successfully (implied by is_completed() semantics)
- Running -> Poisoned when initialization closure panics (evidenced by wait() panic docs and examples)
- Poisoned -> (remains Poisoned for wait(), but can be overridden/ignored by wait_force()/call_once_force(...)) (implied by wait_force() docs and call_once_force mention)
- Incomplete/Poisoned/Complete -> (set to other ExclusiveState) via set_state(...) (crate-private, constrained by &mut self invariant)

**Evidence:** method is_completed(&self) -> bool delegates to self.inner.is_completed(), indicating an internal completion state queried at runtime; wait(&self): `if !self.inner.is_completed() { self.inner.wait(false); }` shows a runtime branch on completion state and a blocking protocol until completion; wait() docs: "If this Once has been poisoned ... this method will also panic. Use wait_force if this behavior is not desired." — poisoning is a distinct state affecting legality of wait(); wait_force(&self): `self.inner.wait(true)` explicitly selects 'ignore poisoning', indicating a Poisoned vs non-Poisoned state distinction; state(&mut self) docs: "no initialization can currently be running, so the state must be either 'incomplete', 'poisoned' or 'complete'" — explicit latent invariant about allowed states under &mut; set_state(&mut self, new_state: ExclusiveState) shares the same comment/invariant and mutates the hidden state machine; examples in is_completed docs: after a panicking call_once in another thread, `assert_eq!(INIT.is_completed(), false);` indicates a post-panic Poisoned-but-not-Completed condition

**Implementation:** Expose (or internally use more pervasively) a typestate API separating a handle to an Incomplete Once from a Completed Once (and optionally a Poisoned Once). For example, Once::try_wait(self) -> Result<Once<Complete>, Once<PoisonedOrIncomplete>> or a pair of wrapper types returned from call_once/call_once_force/wait: wait(self) -> CompletedOnce token; wait_force(self) -> CompletedOnce token. Methods like wait() would only exist on Once<NotPoisoned>, while wait_force() would exist on Once<Any>. For internal state()/set_state(), take/return dedicated types representing the non-Running subset to encode the comment invariant (e.g., ExclusiveOnce<'_> that cannot be obtained while Running).

---

### 41. Packet message/ready state machine (Empty/HasMsg × NotReady/Ready; single handoff)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/zero.rs:1-105`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Packet coordinates a rendezvous handoff using two runtime state encodings: `ready` (an atomic flag controlling when another party may proceed) and `msg: UnsafeCell<Option<T>>` (whether a message has been placed). Callers must follow a protocol: populate/read `msg` only in the correct phase and synchronize using `ready` (wait_ready uses Acquire load). These phases are implicit and enforced by convention + runtime spinning; the type system does not distinguish 'ready to access msg' from 'not ready', nor 'contains message' from 'empty'. `on_stack` also encodes a lifetime/ownership expectation (stack allocation) that is not enforced by the type system once the packet is referenced indirectly.

**Evidence**:

```rust
//! Zero-capacity channel.
//!
//! This kind of channel is also known as *rendezvous* channel.

use super::context::Context;
use super::error::*;
use super::select::{Operation, Selected, Token};
use super::utils::Backoff;
use super::waker::Waker;
use crate::cell::UnsafeCell;
use crate::marker::PhantomData;
use crate::sync::Mutex;
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::time::Instant;
use crate::{fmt, ptr};

/// A pointer to a packet.
pub(crate) struct ZeroToken(*mut ());

impl Default for ZeroToken {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

impl fmt::Debug for ZeroToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&(self.0 as usize), f)
    }
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

/// Inner representation of a zero-capacity channel.
struct Inner {
    /// Senders waiting to pair up with a receive operation.
    senders: Waker,

    /// Receivers waiting to pair up with a send operation.
    receivers: Waker,

    /// Equals `true` when the channel is disconnected.
    is_disconnected: bool,
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
```

**Entity:** Packet<T>

**States:** NotReady (in transit), Ready (safe to read/write), EmptyMsg (msg=None), FullMsg (msg=Some(T)), StackAllocated (on_stack=true)

**Transitions:**
- NotReady -> Ready via some external store to `ready` (implied by `wait_ready()` spinning until true)
- EmptyMsg -> FullMsg via `message_on_stack()` / writing `msg` (constructor shows the intended transition)
- FullMsg -> EmptyMsg via consuming the message (implied by `Option<T>` slot semantics)

**Evidence:** Packet<T> fields: `ready: Atomic<bool>`, `msg: UnsafeCell<Option<T>>`, `on_stack: bool` (runtime state encodings); Packet::empty_on_stack(): initializes `msg` to None and `ready` to false; Packet::message_on_stack(msg): initializes `msg` to Some(msg) and `ready` to false; Packet::wait_ready(): spins until `self.ready.load(Ordering::Acquire)` becomes true (temporal ordering / synchronization requirement)

**Implementation:** Model packet phases as types, e.g. `Packet<NotReady, Empty>` / `Packet<NotReady, Full>` transitioning to `Packet<Ready, ...>` via methods that consume `self` and return a new typed state, and expose `msg` access only on `Ready` states. For pointer passing, use `NonNull<Packet<State...>>` so only correct-phase packets can be shared.

---

### 42. Channel connection state (Connected / Disconnected) affecting wait-queue behavior

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/zero.rs:1-105`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: Inner tracks whether the channel is disconnected via a boolean flag. This flag is intended to gate whether send/receive operations can be matched via the `senders`/`receivers` wakers, but the state is only represented at runtime (`is_disconnected: bool`). The type system does not prevent using the channel as if connected after disconnection; correctness depends on checks against this flag in operations (one such disconnection behavior is visible in Channel::write via a null token sentinel).

**Evidence**:

```rust
//! Zero-capacity channel.
//!
//! This kind of channel is also known as *rendezvous* channel.

use super::context::Context;
use super::error::*;
use super::select::{Operation, Selected, Token};
use super::utils::Backoff;
use super::waker::Waker;
use crate::cell::UnsafeCell;
use crate::marker::PhantomData;
use crate::sync::Mutex;
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::time::Instant;
use crate::{fmt, ptr};

/// A pointer to a packet.
pub(crate) struct ZeroToken(*mut ());

impl Default for ZeroToken {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

impl fmt::Debug for ZeroToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&(self.0 as usize), f)
    }
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

/// Inner representation of a zero-capacity channel.
struct Inner {
    /// Senders waiting to pair up with a receive operation.
    senders: Waker,

    /// Receivers waiting to pair up with a send operation.
    receivers: Waker,

    /// Equals `true` when the channel is disconnected.
    is_disconnected: bool,
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
```

**Entity:** Inner

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via setting `is_disconnected = true` (implied by field meaning and initialization)

**Evidence:** Inner field: `is_disconnected: bool` with comment `Equals true when the channel is disconnected.`; Channel::new(): initializes `is_disconnected: false` (establishes Connected initial state); Channel::write(): treats `token.zero.0.is_null()` as 'channel is disconnected' (disconnection semantics are encoded at runtime)

**Implementation:** Split channel handle types into `Channel<Connected, T>` and `Channel<Disconnected, T>` (or separate `Sender<T>`/`Receiver<T>` typestates) where operations requiring a live peer are only available on `Connected`. Disconnection transitions consume and return a disconnected handle, preventing further rendezvous operations at compile time.

---

### 14. RwLock poisoning protocol (Unpoisoned / Poisoned) gated by writer panics

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-156`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: RwLock carries an implicit poison state indicating whether a writer panicked while holding the lock. This state is stored in the runtime `poison: poison::Flag` and is consulted by operations (e.g., documented for a cloning accessor) to decide whether to return an error. The type system does not distinguish a poisoned lock from an unpoisoned one, so callers can only discover/handle poisoning at runtime via Result/error paths rather than by having different types/capabilities for the two states.

**Evidence**:

```rust
/// ```
///
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

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
///
/// [`read`]: RwLock::read
/// [`try_read`]: RwLock::try_read
#[must_use = "if unused the RwLock will immediately unlock"]
#[must_not_suspend = "holding a RwLockReadGuard across suspend \
                      points can cause deadlocks, delays, \
                      and cause Futures to not implement `Send`"]
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

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockReadGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
///
/// [`write`]: RwLock::write
/// [`try_write`]: RwLock::try_write
#[must_use = "if unused the RwLock will immediately unlock"]
#[must_not_suspend = "holding a RwLockWriteGuard across suspend \
                      points can cause deadlocks, delays, \
                      and cause Future's to not implement `Send`"]
#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "RwLockWriteGuard")]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
    poison: poison::Guard,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}

#[stable(feature = "rwlock_guard_sync", since = "1.23.0")]
unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`filter_map`] methods
/// on [`RwLockReadGuard`].
///
/// [`map`]: RwLockReadGuard::map
/// [`filter_map`]: RwLockReadGuard::filter_map
#[must_use = "if unused the RwLock will immediately unlock"]
#[must_not_suspend = "holding a MappedRwLockReadGuard across suspend \
                      points can cause deadlocks, delays, \
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

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> !Send for MappedRwLockReadGuard<'_, T> {}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockReadGuard<'_, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`filter_map`] methods
/// on [`RwLockWriteGuard`].
///
/// [`map`]: RwLockWriteGuard::map
/// [`filter_map`]: RwLockWriteGuard::filter_map
#[must_use = "if unused the RwLock will immediately unlock"]
#[must_not_suspend = "holding a MappedRwLockWriteGuard across suspend \
                      points can cause deadlocks, delays, \
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
```

**Entity:** RwLock<T>

**States:** Unpoisoned, Poisoned

**Transitions:**
- Unpoisoned -> Poisoned via writer panic while holding exclusive access (tracked by poison::Flag/poison::Guard)

**Evidence:** field `poison: poison::Flag` in `pub struct RwLock<T: ?Sized>` encodes poison state at runtime; field `poison: poison::Guard` in `pub struct RwLockWriteGuard<'a, T: ?Sized + 'a>` indicates write-guard participates in poison tracking; comment in `RwLock<T>` impl: "This function will return an error if the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer panics while holding an exclusive"

**Implementation:** Model poisoning as a type-level state: e.g. `RwLock<T, P>` where `P` is `Unpoisoned`/`Poisoned`, or return a capability token from write operations that can transition the lock into a `PoisonedRwLock<T>` wrapper on panic/unwind paths. Provide APIs that require `RwLock<T, Unpoisoned>` (or a `NotPoisoned` capability) for operations that currently error on poison; keep an explicit `recover(self) -> RwLock<T, Unpoisoned>` or `into_inner_ignore_poison`-style transition for opting out.

---

### 55. Channel connectivity state machine (Connected / Disconnected) shared by sender+receiver wait paths

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/zero.rs:1-63`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The channel has an implicit connectivity state tracked at runtime inside the locked `inner` (via `inner.is_disconnected`). Many operations (not fully shown here) must behave differently depending on whether the channel is connected: blocked operations can be woken and should return `Disconnected`, and disconnecting should be idempotent. The type system does not distinguish a connected channel handle from a disconnected one, so correctness relies on runtime checks and wakeups.

**Evidence**:

```rust
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

**Entity:** Channel (type owning `self.inner` in src::sync::mpmc::zero)

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via disconnect()
- Disconnected -> Disconnected via disconnect() (idempotent no-op)

**Evidence:** disconnect(): `if !inner.is_disconnected { inner.is_disconnected = true; ... } else { false }` encodes the runtime state machine; disconnect(): `inner.senders.disconnect(); inner.receivers.disconnect();` indicates that disconnection affects both sides and wakes blocked operations; match arm: `Selected::Disconnected => ... Err(RecvTimeoutError::Disconnected)` shows operations can complete specifically due to disconnect state

**Implementation:** Represent handles as `Channel<S>` where `S` is `Connected` or `Disconnected`. Make `disconnect(self or &Channel<Connected>) -> Channel<Disconnected>` (or return a `DisconnectedToken`). Restrict blocking send/recv APIs to `Channel<Connected>` so that after disconnection the only available operations are non-blocking queries/cleanup.

---

### 26. Channel stamp/mark state machine (Connected / Disconnected encoded in tail mark bit + lap/index encoding)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: Channel connectivity and queue position are encoded into packed 'stamp' integers in `head` and `tail`. In particular, the `tail` stamp's mark bit indicates the channel is disconnected, while `head`'s mark bit is stated to always be zero. The correctness of operations depends on maintaining this packed representation invariant (index/lap/mark layout, head mark always 0, tail mark toggled for disconnect) and on only performing certain transitions (e.g., setting the disconnect mark) in appropriate situations. These invariants are maintained by convention and atomic integer manipulation rather than being expressed as distinct types/states.

**Evidence**:

```rust
use super::select::{Operation, Selected, Token};
use super::utils::{Backoff, CachePadded};
use super::waker::SyncWaker;
use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::ptr;
use crate::sync::atomic::{self, Atomic, AtomicUsize, Ordering};
use crate::time::Instant;

/// A slot in a channel.
struct Slot<T> {
    /// The current stamp.
    stamp: Atomic<usize>,

    /// The message in this slot. Either read out in `read` or dropped through
    /// `discard_all_messages`.
    msg: UnsafeCell<MaybeUninit<T>>,
}

/// The token type for the array flavor.
#[derive(Debug)]
pub(crate) struct ArrayToken {
    /// Slot to read from or write to.
    slot: *const u8,

    /// Stamp to store into the slot after reading or writing.
    stamp: usize,
}

impl Default for ArrayToken {
    #[inline]
    fn default() -> Self {
        ArrayToken { slot: ptr::null(), stamp: 0 }
    }
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
```

**Entity:** Channel<T>

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via setting the mark bit in `tail` stamp (disconnect operation not shown in snippet)

**Evidence:** field `Channel::head: CachePadded<Atomic<usize>>` comment: stamp packs index/mark/lap; 'The mark bit in the head is always zero.'; field `Channel::tail: CachePadded<Atomic<usize>>` comment: stamp packs index/mark/lap; 'The mark bit indicates that the channel is disconnected.'; fields `Channel::cap` and `Channel::one_lap` plus comments indicate a specific packed-stamp arithmetic/layout protocol that callers must preserve

**Implementation:** Introduce a `Stamp` newtype encapsulating the `usize` with constructors/accessors enforcing the bit layout (index/lap/mark), plus separate `HeadStamp` and `TailStamp` types (or `Stamp<Role>`) that prevent setting a mark bit on head at compile time. Represent connectivity as `ConnectedTailStamp`/`DisconnectedTailStamp` (typestate) or provide methods that only allow `disconnect(self)` transitions for the tail.

---

### 53. LazyLock initialization state machine (Incomplete / Complete / Poisoned)

**Location**: `/tmp/sync_test_crate/src/sync/lazy_lock.rs:1-101`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: LazyLock has an implicit runtime state tracked inside `once` (via `Once::state()` / `ExclusiveState`). While `Incomplete`, `data` holds the initializer closure `f`; once initialized (`Complete`), `data` holds the produced `value`. If initialization panics, the cell becomes `Poisoned` and subsequent operations panic. This protocol (which variant of `Data` is currently valid, and which operations are allowed) is enforced by runtime state inspection and `unsafe` assumptions rather than the type system.

**Evidence**:

```rust
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
```

**Entity:** LazyLock<T, F>

**States:** Incomplete, Complete, Poisoned

**Transitions:**
- Incomplete -> Complete via initialization path (e.g., `force_mut` calling `really_init_mut`)
- Incomplete -> Poisoned if initialization panics (Drop of `PoisonOnPanic` sets state)
- Complete -> (terminal) value is available for reading/extraction
- Poisoned -> (terminal) operations panic

**Evidence:** `once: Once` field stores the runtime state machine; `data: UnsafeCell<Data<T, F>>` indicates interior mutability and data-representation depends on state; `into_inner`: `let state = this.once.state();` and `match state { ExclusiveState::Poisoned => panic_poisoned(), ... }` (state-dependent behavior); `into_inner`: on `ExclusiveState::Incomplete` returns `Err(... data.f ...)`, on `ExclusiveState::Complete` returns `Ok(... data.value ...)` (which `Data` field is valid depends on state); `force_mut`: inner function comment `/// # Safety May only be called when the state is Incomplete.` (implicit precondition); `force_mut`: `PoisonOnPanic` drop impl sets `self.0.once.set_state(ExclusiveState::Poisoned);` (panic -> poisoned transition)

**Implementation:** Represent the state at the type level (e.g., `LazyLock<T, F, S>` with `S = Uninit|Init|Poisoned`), storing either `F` or `T` in state-specific structs. `new` returns `LazyLock<_,_,Uninit>`, initialization/force returns a `LazyLock<_,_,Init>` (or yields `&T` from an `Init`-borrowed view), and operations that require initialization are only implemented for `Init`. Poisoning can be modeled as a separate type/state returned from a `try_force`-style API (or as an `InitResult` wrapper) rather than panicking based on runtime state.

---

### 28. Sender runtime channel-flavor state (Array/List/Zero) with flavor-specific semantics (capacity/emptiness/fullness)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/mod.rs:1-102`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: A Sender has an implicit runtime state encoded by `self.flavor` selecting one of three channel implementations. The meaning of `is_empty`, `is_full`, and `len` depends on this flavor (notably: comments state `Zero` channels are always empty and always full, which is a distinct semantic regime from bounded/unbounded buffering). This protocol is enforced by runtime dispatch (`match &self.flavor`) rather than at the type level, so users cannot express (or the compiler cannot exploit) that a given Sender is definitely bounded/unbounded/zero-capacity, nor can APIs be restricted to only the flavors where the query is meaningful.

**Evidence**:

```rust
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
```

**Entity:** Sender<T> (implied by impl block providing is_empty/is_full/len and matching on self.flavor: SenderFlavor::{Array,List,Zero})

**States:** Array (bounded buffer), List (unbounded buffer), Zero (zero-capacity rendezvous)

**Transitions:**
- Array/List/Zero chosen at construction (e.g., via channel()/sync_channel()) and then remains fixed for the Sender value (no type-level distinction exposed here)

**Evidence:** method `is_empty(&self)` matches on `&self.flavor` with `SenderFlavor::Array/List/Zero`; doc comment on `is_empty`: "Zero-capacity channels are always empty."; method `is_full(&self)` matches on `&self.flavor` with `SenderFlavor::Array/List/Zero`; doc comment on `is_full`: "Zero-capacity channels are always full."; method `len(&self)` matches on `&self.flavor` with `SenderFlavor::Array/List/Zero`

**Implementation:** Expose flavor at the type level, e.g. `struct Sender<T, F> { inner: F, ... }` where `F` is `ArrayChan`, `ListChan`, or `ZeroChan` (or marker types + enum hidden internally). Implement `is_full` only for bounded/zero flavors as appropriate, and encode zero-capacity semantics in the `Zero` sender type so callers can reason about it without runtime checks/dispatch.

---

### 20. ReentrantLock ownership + recursion protocol (Unlocked / Locked-by-current-thread / Locked-by-other-thread)

**Location**: `/tmp/sync_test_crate/src/sync/reentrant_lock.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ReentrantLock encodes a re-entrant mutex with an implicit ownership and recursion-depth state machine: when unlocked, any thread may acquire it; when locked by some thread, the same thread may re-enter (incrementing a recursion counter) while other threads must block/fail until the owner fully releases (recursion count returns to 0). This protocol is represented by runtime fields (owner thread id + lock_count + underlying sys::Mutex) rather than distinct types, so the type system cannot express/verify (a) that lock_count is nonzero iff the lock is owned, (b) that only the owning thread may increment/decrement recursion depth, and (c) that owner is cleared exactly when the final guard is dropped.

**Evidence**:

```rust
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

/// An RAII implementation of a "scoped lock" of a re-entrant lock. When this
/// structure is dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] implementation.
///
/// This structure is created by the [`lock`](ReentrantLock::lock) method on
/// [`ReentrantLock`].
///
/// # Mutability
///
/// Unlike [`MutexGuard`](super::MutexGuard), `ReentrantLockGuard` does not
/// implement [`DerefMut`](crate::ops::DerefMut), because implementation of
/// the trait would violate Rust’s reference aliasing rules. Use interior
/// mutability (usually [`RefCell`](crate::cell::RefCell)) in order to mutate
/// the guarded data.
#[must_use = "if unused the ReentrantLock will immediately unlock"]
#[unstable(feature = "reentrant_lock", issue = "121440")]
pub struct ReentrantLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a ReentrantLock<T>,
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
```

**Entity:** ReentrantLock<T>

**States:** Unlocked, LockedByCurrentThread(recursion_depth >= 1), LockedByOtherThread

**Transitions:**
- Unlocked -> LockedByCurrentThread via ReentrantLock::lock(...) (implied by guard creation comment)
- LockedByCurrentThread(d) -> LockedByCurrentThread(d+1) via ReentrantLock::lock(...) re-entry by same thread (implied by fields owner + lock_count)
- LockedByCurrentThread(d) -> LockedByCurrentThread(d-1) via Drop of ReentrantLockGuard (implied by RAII comment)
- LockedByCurrentThread(1) -> Unlocked via Drop of last ReentrantLockGuard (implied by RAII comment + lock_count field)
- LockedByOtherThread -> LockedByCurrentThread via release by owner then acquisition by this thread (implied by sys::Mutex + owner tracking)

**Evidence:** ReentrantLock::new: fields `mutex: sys::Mutex`, `owner: Tid::new()`, `lock_count: UnsafeCell::new(0)` show runtime tracking of ownership and recursion depth; Doc on ReentrantLockGuard: "When this structure is dropped ... the lock will be unlocked" indicates a release transition tied to guard Drop and recursion reaching zero; Doc: "This structure is created by the `lock` method on `ReentrantLock`" indicates acquisition is coupled to `ReentrantLock::lock` creating the guard

**Implementation:** Introduce a state parameter for the lock/guard relationship, e.g. `ReentrantLock<T>` plus an internal capability token representing 'this thread is the owner'. `lock()` would return a guard carrying an owner-capability (possibly via a non-Send token tied to the current thread) and a recursion counter newtype `RecursionDepth(NonZeroUsize)`. While true thread-identity cannot be fully modeled in stable Rust, you can still enforce some invariants at compile time: represent recursion depth as `NonZeroUsize` when locked; ensure only guards can decrement; and split APIs so operations requiring 'locked' state take `&ReentrantLockGuard`/capability rather than `&ReentrantLock`.

---

### 37. Channel endpoint liveness protocol (Open -> Disconnected/Closed)

**Location**: `/tmp/sync_test_crate/src/sync/mpsc.rs:1-172`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The channel has an implicit liveness state: while the sending half exists the receiver may block/receive values; once the sending half is dropped the receiver is in a disconnected/closed state where receives return None/RecvError/TryRecvError::Disconnected/RecvTimeoutError::Disconnected. This protocol is only reflected via runtime errors/Option results and error messages; the type system does not distinguish a Receiver that is definitely connected from one that is definitely disconnected, so callers must handle it dynamically.

**Evidence**:

```rust
        self.rx.recv().ok()
    }
}

#[stable(feature = "receiver_into_iter", since = "1.1.0")]
impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { rx: self }
    }
}

#[stable(feature = "mpsc_debug", since = "1.8.0")]
impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendError").finish_non_exhaustive()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "sending on a closed channel".fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> error::Error for SendError<T> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "sending on a closed channel"
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => "Full(..)".fmt(f),
            TrySendError::Disconnected(..) => "Disconnected(..)".fmt(f),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => "sending on a full channel".fmt(f),
            TrySendError::Disconnected(..) => "sending on a closed channel".fmt(f),
        }
    }
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

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "receiving on a closed channel".fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl error::Error for RecvError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "receiving on a closed channel"
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => "receiving on an empty channel".fmt(f),
            TryRecvError::Disconnected => "receiving on a closed channel".fmt(f),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl error::Error for TryRecvError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match *self {
            TryRecvError::Empty => "receiving on an empty channel",
            TryRecvError::Disconnected => "receiving on a closed channel",
        }
    }
}

#[stable(feature = "mpsc_error_conversions", since = "1.24.0")]
impl From<RecvError> for TryRecvError {
    /// Converts a `RecvError` into a `TryRecvError`.
    ///
    /// This conversion always returns `TryRecvError::Disconnected`.
    ///
    /// No data is allocated on the heap.
    fn from(err: RecvError) -> TryRecvError {
        match err {
            RecvError => TryRecvError::Disconnected,
        }
    }
}

#[stable(feature = "mpsc_recv_timeout_error", since = "1.15.0")]
impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RecvTimeoutError::Timeout => "timed out waiting on channel".fmt(f),
            RecvTimeoutError::Disconnected => "channel is empty and sending half is closed".fmt(f),
        }
    }
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

**Entity:** Receiver<T> / channel endpoints

**States:** Open (connected), Disconnected/Closed (peer dropped)

**Transitions:**
- Open -> Disconnected/Closed when the sending half is dropped (not represented in types here)

**Evidence:** self.rx.recv().ok() converts a recv error into None, implying an error state must be handled at runtime; fmt::Display for SendError<T>: "sending on a closed channel" (closed/disconnected state exists); fmt::Display for RecvError: "receiving on a closed channel" (closed/disconnected state exists); TrySendError::Disconnected(..) and its Display: "sending on a closed channel"; TryRecvError::Disconnected and its Display: "receiving on a closed channel"; RecvTimeoutError::Disconnected and its Display: "channel is empty and sending half is closed"

**Implementation:** Expose a typestate for endpoint liveness where possible, e.g. Receiver<Connected> vs Receiver<Disconnected>. Transition could be produced by an explicit API that checks connectivity (or by consuming operations returning a new state on Disconnected). Operations like recv/try_recv could be limited to Connected, with a distinct type for the disconnected receiver to prevent repeated runtime-error handling in code that requires a live channel.

---

### 57. OnceLock initialization protocol (Uninitialized/Initialized with drop-only-after-init)

**Location**: `/tmp/sync_test_crate/src/sync/once_lock.rs:1-40`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: OnceLock has an implicit runtime state: either it contains an initialized T or it does not. Equality and Drop both rely on this state: eq() compares via get() (which implicitly yields None vs Some(&T)), and Drop conditionally drops the inner T only if the cell is initialized. The type system does not distinguish Initialized vs Uninitialized OnceLock, so operations must dynamically check state (and unsafe Drop relies on that check to uphold safety: 'initialized' implies it's sound to call assume_init_drop() and that no further access will occur).

**Evidence**:

```rust
    ///
    /// Two `OnceLock`s are equal if they either both contain values and their
    /// values are equal, or if neither contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// let five = OnceLock::new();
    /// five.set(5).unwrap();
    ///
    /// let also_five = OnceLock::new();
    /// also_five.set(5).unwrap();
    ///
    /// assert!(five == also_five);
    ///
    /// assert!(OnceLock::<u32>::new() == OnceLock::<u32>::new());
    /// ```
    #[inline]
    fn eq(&self, other: &OnceLock<T>) -> bool {
        self.get() == other.get()
    }
}

#[stable(feature = "once_cell", since = "1.70.0")]
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

**States:** Uninitialized, Initialized

**Transitions:**
- Uninitialized -> Initialized via initialization API (implied by use of get()/is_initialized() and OnceLock semantics)
- Initialized -> Dropped via Drop::drop() (drops inner T)
- Uninitialized -> Dropped via Drop::drop() (no-op)

**Evidence:** fn eq(&self, other: &OnceLock<T>) -> bool { self.get() == other.get() } uses get() which depends on whether a value is present; Drop::drop: if self.is_initialized() { unsafe { (&mut *self.value.get()).assume_init_drop() }; }; Drop SAFETY comment: "The cell is initialized and being dropped, so it can't be accessed again" and "We also don't touch the `T` other than dropping it"; Unsafe call to assume_init_drop() indicates an invariant that value memory is MaybeUninit<T> unless initialized, and must only be dropped when initialized

**Implementation:** Encode initialization in the type: OnceLock<T, S> where S is Uninit or Init. Only OnceLock<T, Init> exposes get_ref/eq-by-value and performs drop of T; OnceLock<T, Uninit> drop is a no-op. A set/init method would consume OnceLock<T, Uninit> and return OnceLock<T, Init> (or a guard/capability proving initialization).

---

### 36. Selected tagged-state encoding invariant (usize representation with reserved sentinel values)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/select.rs:1-67`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Selected is a logical tagged union, but it is also implicitly a compact integer encoding: 0,1,2 are reserved for Waiting/Aborted/Disconnected and any other usize denotes an Operation ID. Correctness relies on the invariant that Operation values never equal 0..=2 (enforced indirectly via Operation::hook's `assert!(val > 2)`). The type system does not couple Selected's encoding to Operation's construction, and `From<usize>` will happily interpret any `usize >= 3` as an Operation even if it was not created by `Operation::hook`, allowing invalid/forged Operation IDs to be constructed at the API boundary.

**Evidence**:

```rust
#[derive(Debug, Default)]
pub struct Token {
    pub(crate) array: super::array::ArrayToken,
    pub(crate) list: super::list::ListToken,
    #[allow(dead_code)]
    pub(crate) zero: super::zero::ZeroToken,
}

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

**Entity:** Selected

**States:** Waiting, Aborted, Disconnected, Operation(opaque-id)

**Transitions:**
- usize 0 -> Selected::Waiting via impl From<usize> for Selected
- usize 1 -> Selected::Aborted via impl From<usize> for Selected
- usize 2 -> Selected::Disconnected via impl From<usize> for Selected
- usize >= 3 -> Selected::Operation(Operation(val)) via impl From<usize> for Selected
- Selected::<variant> -> usize via impl Into<usize> for Selected

**Evidence:** impl From<usize> for Selected maps 0/1/2 to Waiting/Aborted/Disconnected and everything else to Operation(Operation(oper)); impl Into<usize> for Selected maps those variants back to 0/1/2 or the inner Operation(usize); Operation::hook asserts `val > 2` specifically to avoid collision with `Selected::{Waiting, Aborted, Disconnected}` representations

**Implementation:** Introduce a dedicated representation type and seal construction: e.g., `struct SelectedRepr(NonZeroUsize)` where 1..=3 encode the sentinels and 4.. encodes operations (or use an offset). Provide `TryFrom<usize>` instead of `From<usize>` to reject impossible/forged values, and make Operation's inner field private with a checked constructor so you can't build `Operation(1)` etc. outside the module.

---

### 39. Channel endpoint lifecycle protocol (Connected / Disconnected) with blocking vs nonblocking semantics

**Location**: `/tmp/sync_test_crate/src/sync/mpsc.rs:1-89`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The channel endpoints participate in an implicit lifecycle: operations succeed only while the opposite half is still connected. When the last Sender is dropped, Receiver operations transition to a permanently disconnected state (recv fails / try_recv reports Disconnected). When the Receiver is dropped, Sender/SyncSender operations transition to a permanently disconnected state (send fails with SendError; try_send reports Disconnected). Additionally, sync_channel introduces a capacity-dependent 'would block' condition (try_send -> Full) that is a distinct operational state of the channel at a moment in time. These states are described in docs/error types but are not represented in the endpoint types, so callers must handle them at runtime via Result/enum matching; the type system cannot express “this endpoint is definitely connected” or “this sender is for an unbounded channel so it will never be Full”.

**Evidence**:

```rust
unsafe impl<T: Send> Send for SyncSender<T> {}

/// An error returned from the [`Sender::send`] or [`SyncSender::send`]
/// function on **channel**s.
///
/// A **send** operation can only fail if the receiving end of a channel is
/// disconnected, implying that the data could never be received. The error
/// contains the data being sent as a payload so it can be recovered.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(#[stable(feature = "rust1", since = "1.0.0")] pub T);

/// An error returned from the [`recv`] function on a [`Receiver`].
///
/// The [`recv`] operation can only fail if the sending half of a
/// [`channel`] (or [`sync_channel`]) is disconnected, implying that no further
/// messages will ever be received.
///
/// [`recv`]: Receiver::recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RecvError;

/// This enumeration is the list of the possible reasons that [`try_recv`] could
/// not return data when called. This can occur with both a [`channel`] and
/// a [`sync_channel`].
///
/// [`try_recv`]: Receiver::try_recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub enum TryRecvError {
    /// This **channel** is currently empty, but the **Sender**(s) have not yet
    /// disconnected, so data may yet become available.
    #[stable(feature = "rust1", since = "1.0.0")]
    Empty,

    /// The **channel**'s sending half has become disconnected, and there will
    /// never be any more data received on it.
    #[stable(feature = "rust1", since = "1.0.0")]
    Disconnected,
}

/// This enumeration is the list of possible errors that made [`recv_timeout`]
/// unable to return data when called. This can occur with both a [`channel`] and
/// a [`sync_channel`].
///
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

/// This enumeration is the list of the possible error outcomes for the
/// [`try_send`] method.
///
/// [`try_send`]: SyncSender::try_send
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    /// The data could not be sent on the [`sync_channel`] because it would require that
    /// the callee block to send the data.
    ///
    /// If this is a buffered channel, then the buffer is full at this time. If
    /// this is not a buffered channel, then there is no [`Receiver`] available to
    /// acquire the data.
    #[stable(feature = "rust1", since = "1.0.0")]
    Full(#[stable(feature = "rust1", since = "1.0.0")] T),

    /// This [`sync_channel`]'s receiving half has disconnected, so the data could not be
    /// sent. The data is returned back to the callee in this case.
    #[stable(feature = "rust1", since = "1.0.0")]
    Disconnected(#[stable(feature = "rust1", since = "1.0.0")] T),
}

/// Creates a new asynchronous channel, returning the sender/receiver halves.
///
/// All data sent on the [`Sender`] will become available on the [`Receiver`] in
/// the same order as it was sent, and no [`send`] will block the calling thread
/// (this channel has an "infinite buffer", unlike [`sync_channel`], which will
/// block after its buffer limit is reached). [`recv`] will block until a message
/// is available while there is at least one [`Sender`] alive (including clones).
```

**Entity:** channel / sync_channel (Sender/Receiver halves as a pair)

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected when the opposite half is dropped (last Sender dropped disconnects Receiver; Receiver dropped disconnects Sender/SyncSender)
- Connected -> (temporarily) Full on sync_channel when buffer is full or no Receiver is ready (observable via try_send)

**Evidence:** comment (SendError): "A send operation can only fail if the receiving end of a channel is disconnected"; comment (RecvError): "recv operation can only fail if the sending half ... is disconnected"; TryRecvError::Disconnected doc: "sending half has become disconnected, and there will never be any more data"; TrySendError::Full doc: "could not be sent ... because it would require that the callee block" and "buffer is full" / "no Receiver available"; TrySendError::Disconnected doc: "receiving half has disconnected"; comment (channel creation): "recv will block until a message is available while there is at least one Sender alive (including clones)"

**Implementation:** Expose typestates for endpoint connectivity and channel kind, e.g. `Sender<T, Connected, Unbounded>` vs `SyncSender<T, Connected, Bounded<N>>`, where drop/close transitions yield `Disconnected` (or consume into a `DisconnectedSender<T>`). For sync channels, encode boundedness at the type level so `try_send` returning `Full` is only possible for bounded channels.

---

### 47. Mutex poisoning protocol (Healthy / Poisoned) affecting lock-derived operations

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-200`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Mutex has an implicit poisoning state: if any thread panics while holding the lock, the mutex becomes Poisoned. Subsequent operations that acquire the lock (lock/try_lock and the value accessors set/replace/get_cloned shown) return errors instead of succeeding normally. This is enforced dynamically via result types and a runtime poison flag; the type system does not distinguish a Healthy mutex from a Poisoned one, so callers must handle/propagate poisoning at each call site (or explicitly clear it elsewhere). Additionally, is_poisoned() is only a snapshot and may immediately become stale under concurrency, so using it as a pre-check for correctness is an implicit protocol that is not enforceable by the API as written.

**Evidence**:

```rust
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
```

**Entity:** Mutex<T>

**States:** Healthy, Poisoned

**Transitions:**
- Healthy -> Poisoned via panic while holding a MutexGuard (implied by docs and PoisonError paths)
- Poisoned -> (observed as error-returning acquisition) via lock()/try_lock() and derived accessors
- Poisoned -> Healthy via clear_poison() (implied by the presence of 'Clear the poisoned state from a mutex' doc comment, though implementation not shown)

**Evidence:** method lock(&self) -> LockResult<MutexGuard<'_, T>>: returning LockResult encodes 'may be poisoned' at runtime rather than as a distinct type; method try_lock(&self) -> TryLockResult<MutexGuard<'_, T>>: documentation distinguishes TryLockError::Poisoned vs WouldBlock, indicating a distinct runtime state; method is_poisoned(&self) -> bool returns self.poison.get(): explicit runtime poison flag governs behavior; doc comment on is_poisoned(): 'mutex can still become poisoned at any time' and 'should not trust a false value ... without additional synchronization' indicates a temporal/protocol constraint not captured by types; set(&self, value: T): match self.lock() { ... Err(_) => Err(PoisonError::new(value)) } shows poisoning gates mutation and changes error behavior; replace(&self, value: T): match self.lock() { ... Err(_) => Err(PoisonError::new(value)) } shows poisoning gates replacement; get_cloned(): match self.lock() { ... Err(_) => Err(PoisonError::new(())) } shows poisoning gates read access

**Implementation:** Introduce a (possibly opt-in) typestate wrapper around Mutex: e.g., `struct CheckedMutex<T, S>(Mutex<T>, PhantomData<S>)` with `Healthy`/`Poisoned` states. Acquisition on `CheckedMutex<T, Healthy>` returns guards that cannot observe poisoning (or returns `Poisoned` wrapper on panic detection), while `CheckedMutex<T, Poisoned>` forces an explicit recovery path (e.g., `into_inner_ignore_poison` / `clear_poison(self) -> CheckedMutex<T, Healthy>`). This makes 'I have handled poisoning' an explicit state transition rather than a repeated runtime check at each operation.

---

### 43. OnceLock initialization protocol (Uninitialized/Initializing -> Initialized)

**Location**: `/tmp/sync_test_crate/src/sync/once_lock.rs:1-69`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: OnceLock has an implicit state machine around one-time initialization. Before initialization completes, get()/get_mut() must not expose references and return None; after initialization, get()/get_mut()/wait() may return references into the stored T. The correctness of returning &T/&mut T relies on a runtime check (is_initialized()) and on the Once synchronization (wait_force()) to ensure initialization has completed before unsafe access. The type system does not distinguish these states, so callers must handle Option and the implementation must maintain the invariant that get_unchecked/get_unchecked_mut are only called after initialization is complete.

**Evidence**:

```rust
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
```

**Entity:** OnceLock<T>

**States:** Uninitialized, Initializing, Initialized

**Transitions:**
- Uninitialized -> Initializing via Once-driven initialization (implied by presence of once: Once and comments about 'being initialized')
- Initializing -> Initialized when Once completes (wait_force()/is_initialized() become true)
- Uninitialized -> Initialized via initialization API (implied by comment 'Initializes the contents of the cell to value')

**Evidence:** new(): fields once: Once, value: UnsafeCell<MaybeUninit<_>> indicate a one-time init protocol with deferred initialization; get(): comment 'Returns None if the cell is uninitialized, or being initialized' implies distinct Uninitialized vs Initializing states; get(): `if self.is_initialized() { ... unsafe { self.get_unchecked() } } else { None }` shows runtime state check gating unsafe access; get_mut(): same runtime gating via is_initialized() before `unsafe { self.get_unchecked_mut() }`; wait(): calls `self.once.wait_force();` then `unsafe { self.get_unchecked() }`—blocking transition to Initialized before unsafe read; safety comments: 'Safe b/c checked is_initialized' and 'Safe b/c checked is_initialized and we have a unique access' document the latent invariant

**Implementation:** Expose two (or three) types representing states, e.g. `OnceLock<Uninit, T>` and `OnceLock<Init, T>` (and optionally `InitInProgress` internal token). Provide an initializing method that consumes `OnceLock<Uninit, T>` and returns `OnceLock<Init, T>` (or a `Result`). Only implement `get_unchecked`-like accessors / `Deref<Target=T>` for the Initialized type; keep blocking `wait()` as a transition `OnceLock<Uninit, T> -> OnceLock<Init, T>` returning an initialized handle.

---

### 62. Channel disconnect protocol + post-disconnect message handling (Connected / SendersDisconnected / ReceiversDisconnected)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-186`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Channel disconnection is encoded in-band in the atomic tail index via MARK_BIT. disconnect_senders() and disconnect_receivers() both attempt to set MARK_BIT and only the first caller "wins" (returns true). After receivers are disconnected, the implementation relies on an additional protocol: it is now valid/required to eagerly discard all queued messages to free memory, and this discard routine must only run when no receivers exist. None of these states (connected vs disconnected, which side disconnected, and the exclusivity requirement for discard_all_messages()) are represented in the type system; they are enforced by atomic bit checks, return values, and comments.

**Evidence**:

```rust
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
        // will be deallocated by the sender in Drop.
        let mut block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);

        // If we're going to be dropping messages we need to synchronize with initialization
        if head >> SHIFT != tail >> SHIFT {
            // The block can be null here only if a sender is in the process of initializing the
            // channel while another sender managed to send a message by inserting it into the
            // semi-initialized channel and advanced the tail.
            // In that case, just wait until it gets initialized.
            while block.is_null() {
                backoff.spin_heavy();
                block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);
            }
        }
        // After this point `head.block` is not modified again and it will be deallocated if it's
        // non-null. The `Drop` code of the channel, which runs after this function, also attempts
        // to deallocate `head.block` if it's non-null. Therefore this function must maintain the
        // invariant that if a deallocation of head.block is attemped then it must also be set to
        // NULL. Failing to do so will lead to the Drop code attempting a double free. For this
        // reason both reads above do an atomic swap instead of a simple atomic load.

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head >> SHIFT != tail >> SHIFT {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    slot.wait_write();
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    (*block).wait_next();
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Acquire);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }

        head &= !MARK_BIT;
        self.head.index.store(head, Ordering::Release);
    }

    /// Returns `true` if the channel is disconnected.
    pub(crate) fn is_disconnected(&self) -> bool {
        self.tail.index.load(Ordering::SeqCst) & MARK_BIT != 0
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.index.load(Ordering::SeqCst);
        let tail = self.tail.index.load(Ordering::SeqCst);
        head >> SHIFT == tail >> SHIFT
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        false
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        let mut head = self.head.index.load(Ordering::Relaxed);
        let mut tail = self.tail.index.load(Ordering::Relaxed);
        let mut block = self.head.block.load(Ordering::Relaxed);

        // Erase the lower bits.
        head &= !((1 << SHIFT) - 1);
        tail &= !((1 << SHIFT) - 1);

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head != tail {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Relaxed);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }
    }
}
```

**Entity:** Channel<T>

**States:** Connected, SendersDisconnected, ReceiversDisconnected

**Transitions:**
- Connected -> SendersDisconnected via disconnect_senders() (sets MARK_BIT, wakes receivers)
- Connected -> ReceiversDisconnected via disconnect_receivers() (sets MARK_BIT, then discard_all_messages())
- SendersDisconnected -> SendersDisconnected (idempotent) via disconnect_senders()/disconnect_receivers() returning false
- ReceiversDisconnected -> ReceiversDisconnected (idempotent) via disconnect_senders()/disconnect_receivers() returning false

**Evidence:** disconnect_senders(): `let tail = self.tail.index.fetch_or(MARK_BIT, Ordering::SeqCst); if tail & MARK_BIT == 0 { ... }` — MARK_BIT encodes disconnection and first-wins transition; disconnect_receivers(): same fetch_or(MARK_BIT) guard; on first disconnect it calls `self.discard_all_messages()`; discard_all_messages() doc: `This method should only be called when all receivers are dropped.` — precondition not enforced by types; is_disconnected(): `self.tail.index.load(Ordering::SeqCst) & MARK_BIT != 0` — runtime check for state; discard_all_messages(): `head &= !MARK_BIT; self.head.index.store(head, Ordering::Release);` — relies on MARK_BIT protocol across indices

**Implementation:** Introduce a typestate parameter for Channel indicating connectivity, e.g. `Channel<T, S>` with states like `Connected`, `SendersDisconnected`, `ReceiversDisconnected`. Make `disconnect_senders(self: &Channel<T, Connected>) -> Channel<T, SendersDisconnected>` (or return a guard/token representing the disconnected state) and similarly for receivers. Gate `discard_all_messages()` behind a capability/token that can only be constructed when the last receiver is dropped (e.g., a `LastReceiver` drop-produced token), preventing calling it in other states.

---

### 33. Channel first-block initialization race (UninitializedBlocks -> InitializedBlocks)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-298`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Channel begins with head.block and tail.block as null pointers and lazily installs the first Block on the first send. There is an explicit intermediate 'half-initialized' state where tail.block has been set but head.block has not yet been stored (even intentionally under cfg(miri) with a yield). Receivers must detect and spin while head.block is null, assuming another thread is in the process of initialization. This cross-thread initialization protocol is encoded with null checks and ordering, but not represented in types, so code relies on runtime spinning and careful atomic orderings to avoid reading through null/uninitialized pointers.

**Evidence**:

```rust
                return;
            }
        }

        // No thread is using the block, now it is safe to destroy it.
        drop(unsafe { Box::from_raw(this) });
    }
}

/// A position in a channel.
#[derive(Debug)]
struct Position<T> {
    /// The index in the channel.
    index: Atomic<usize>,

    /// The block in the linked list.
    block: Atomic<*mut Block<T>>,
}

/// The token type for the list flavor.
#[derive(Debug)]
pub(crate) struct ListToken {
    /// The block of slots.
    block: *const u8,

    /// The offset into the block.
    offset: usize,
}

impl Default for ListToken {
    #[inline]
    fn default() -> Self {
        ListToken { block: ptr::null(), offset: 0 }
    }
}

/// Unbounded channel implemented as a linked list.
///
/// Each message sent into the channel is assigned a sequence number, i.e. an index. Indices are
/// represented as numbers of type `usize` and wrap on overflow.
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
```

**Entity:** Channel<T> (head/tail Position<T> initialization)

**States:** No blocks installed (head.block == null && tail.block == null), Half-initialized (tail.block != null && head.block == null), Initialized (head.block != null && tail.block != null)

**Transitions:**
- No blocks installed -> Half-initialized via Channel::start_send() CAS installing tail.block before head.block.store()
- Half-initialized -> Initialized via Channel::start_send() storing head.block
- No blocks installed/Half-initialized -> Initialized observed by Channel::start_recv() once head.block becomes non-null

**Evidence:** Channel::new(): head.block and tail.block initialized to ptr::null_mut() (Position { block: AtomicPtr::new(null_mut()) }); Channel::start_send(): 'if block.is_null() { ... tail.block.compare_exchange(...); ... self.head.block.store(new, Ordering::Release); }' installs first block lazily; Channel::start_send() comment: 'This yield point leaves the channel in a half-initialized state where the tail.block pointer is set but the head.block is not.'; Channel::start_recv() comment and logic: 'The block can be null here only if the first message is being sent... just wait until it gets initialized.' followed by `if block.is_null() { backoff.spin_heavy(); ... continue; }`

**Implementation:** Split Channel into an internal enum/typestate such as Channel<Uninit> and Channel<Init> where Init contains NonNull<Block<T>> for head/tail blocks. Have the first send perform an explicit initialization step returning Channel<Init> (or store an OnceLock<NonNull<Block<T>>> for the initial block) so recv paths don't need to treat null as a valid transient state.

---

### 48. Poison flag protocol (Healthy / Poisoned) with explicit clear() recovery

**Location**: `/tmp/sync_test_crate/src/sync/poison.rs:1-238`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Flag encodes whether an associated lock is poisoned. Callers are expected to (1) check poison state before proceeding (borrow()/guard()), (2) if holding a guard, call done(&Guard) at the end to potentially transition to Poisoned when a panic occurs while the lock is held, and (3) optionally clear() to manually recover. None of these sequencing requirements are enforced by the type system: Guard does not automatically 'commit' poisoning in Drop, and Flag's methods do not require proof that the lock was held (external synchronization is assumed by comments).

**Evidence**:

```rust
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
```

**Entity:** Flag

**States:** Healthy (not poisoned), Poisoned

**Transitions:**
- Healthy -> Poisoned via done(&Guard) when `!guard.panicking && thread::panicking()`
- Poisoned -> Healthy via clear()
- Healthy -> Healthy via borrow()/guard() returning Ok
- Poisoned -> Poisoned via borrow()/guard() returning Err(PoisonError)

**Evidence:** Field: Flag.failed: Atomic<bool> (cfg(panic = "unwind")) stores poison state; Method: Flag::borrow() returns Err(PoisonError::new(())) if self.get() is true; Method: Flag::guard() captures Guard { panicking: thread::panicking() } and returns Err(PoisonError::new(ret)) if self.get() is true; Method: Flag::done(&self, guard: &Guard) sets failed=true when panic begins after guard creation: `if !guard.panicking && thread::panicking()`; Method: Flag::clear() stores false into failed (manual recovery); Comment: ordering is Relaxed and 'actual location that this matters is when a mutex is locked'—implies an external protocol: these operations must be used only under lock acquisition/release synchronization

**Implementation:** Make the poisoning commit automatic by tying `Flag::done` to an RAII guard type owned by the lock guard: e.g., `struct PoisonGuard<'a> { flag: &'a Flag, started_panicking: bool }` with `Drop` calling the current done() logic. Return `PoisonGuard` from `Flag::guard()` (or integrate into Mutex/RwLock guards) so callers cannot forget to call done(). Optionally use a typestate/capability token to represent 'lock is held' and require that token for borrow/guard/clear operations.

---

### 22. Channel connection state encoded in mark_bit (Connected / Disconnected)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-222`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Channel connection/disconnection is encoded by setting a mark bit inside the packed head/tail counters. start_send checks (tail & mark_bit) to detect disconnection; start_recv checks the same bit on tail when empty to decide whether to report disconnected vs not-ready. This state is not represented in the type system (no Connected/Disconnected handle types), so APIs must repeatedly re-check at runtime and propagate the state via Token null-sentinel and Result/Err paths.

**Evidence**:

```rust
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
```

**Entity:** Channel<T> (head/tail mark_bit + stamp-based ring buffer)

**States:** Connected, Disconnected

**Transitions:**
- Connected -> Disconnected via (implicit) setting of tail's mark_bit (not shown in snippet)
- Disconnected -> (terminal) observed by Channel::start_send and Channel::start_recv

**Evidence:** start_send: 'Check if the channel is disconnected.' then `if tail & self.mark_bit != 0 { ... token.array.slot = ptr::null() ... }`; start_recv: when empty, it checks `if tail & self.mark_bit != 0 { ... token.array.slot = ptr::null() ... }` to indicate disconnected; Initialization comments: 'Head is initialized to { lap: 0, mark: 0, index: 0 }' and same for tail, indicating mark is a state flag embedded in the counter

**Implementation:** Introduce a connected capability/handle (e.g., Channel<Connected> or Sender/Receiver tokens carrying a Connected marker) and make disconnect transition consume or invalidate capabilities (e.g., close(self) -> Channel<Disconnected> or split Sender/Receiver where Drop of last endpoint transitions state). Even if the underlying atomic mark bit remains, the public API could ensure operations requiring connectivity are only callable when a connected capability exists.

---

### 49. RwLock poisoning protocol (Healthy / Poisoned) with fallible accessors

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-67`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: RwLock has an implicit global state of whether it is poisoned. The code relies on runtime detection of poisoning and propagates it via LockResult/PoisonError. Operations like replace() and read()/write() are valid in both states, but in the Poisoned state they return an error (often containing the acquired guard or the would-be-updated value) rather than succeeding. The type system does not distinguish Healthy vs Poisoned locks, so callers must handle poisoning dynamically each time (or unwrap and panic).

**Evidence**:

```rust
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
```

**Entity:** RwLock<T>

**States:** Healthy, Poisoned

**Transitions:**
- Healthy -> Poisoned when a writer panics while holding an exclusive lock (described in docs)

**Evidence:** replace(&self, value: T) -> LockResult<T>: matches on self.write(); Ok(..) vs Err(_) => Err(PoisonError::new(value)); Doc comment on replace(): "This function will return an error ... if the RwLock is poisoned" and defines poisoning: "whenever a writer panics while holding an exclusive lock."; Doc comment on read(): "This function will return an error if the RwLock is poisoned... The acquired lock guard will be contained in the returned error."

**Implementation:** Expose an API that can return a refined handle after checking poison once, e.g., `fn try_unpoison(&self) -> Result<HealthyRwLock<'_, T>, PoisonedRwLock<'_, T>>` where `HealthyRwLock` provides infallible `read/write/replace` (or returns guards that cannot carry poison errors), while `PoisonedRwLock` provides explicit recovery APIs. This pushes repeated poison-checking and associated branching out of normal call sites.

---

### 54. OnceLock initialization state machine (Uninitialized / Initializing / Initialized; reset via take)

**Location**: `/tmp/sync_test_crate/src/sync/once_lock.rs:1-227`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: OnceLock has an implicit runtime state machine governed by the embedded `Once` plus the `MaybeUninit<T>` slot. Many operations (notably the `unsafe` `get_unchecked*` helpers) require the cell to be Initialized, but this is only asserted/checked at runtime (`is_initialized()` / `debug_assert!`). Initialization is attempted via `initialize(f)` (called by `get_or_try_init` / `get_mut_or_try_init`), which may leave the cell Uninitialized if `f` returns `Err` or panics. There is also an in-progress (Initializing) phase while `Once::call_once_force` is executing; reentrant initialization from within `f` is explicitly forbidden and currently deadlocks. Finally, `take(&mut self)` transitions Initialized back to Uninitialized by resetting `self.once = Once::new()` and moving out the value; correctness relies on the `Once` state and the value slot staying in sync, which is not expressed in the type system.

**Evidence**:

```rust
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

// Why do we need `T: Send`?
// Thread A creates a `OnceLock` and shares it with
// scoped thread B, which fills the cell, which is
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

#[stable(feature = "once_cell", since = "1.70.0")]
impl<T> Default for OnceLock<T> {
    /// Creates a new uninitialized cell.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::OnceLock;
    ///
    /// fn main() {
    ///     assert_eq!(OnceLock::<()>::new(), OnceLock::default());
    /// }
    /// ```
```

**Entity:** OnceLock<T>

**States:** Uninitialized, Initializing (in-progress), Initialized

**Transitions:**
- Uninitialized -> Initializing via get_or_try_init() / get_mut_or_try_init() calling initialize(f)
- Initializing -> Initialized when f() returns Ok(value) and initialize() writes into self.value
- Initializing -> Uninitialized when f() returns Err(e) (and p.poison() is used to avoid marking completion)
- Initializing -> Uninitialized when f() panics (documented: panic propagated; cell remains uninitialized)
- Initialized -> Uninitialized via take(&mut self) (resets self.once and assume_init_read()s the slot)
- Uninitialized -> Uninitialized via take(&mut self) returning None

**Evidence:** method get_or_try_init(): fast-path `if let Some(value) = self.get() { return Ok(value); }` then `self.initialize(f)?;` then `debug_assert!(self.is_initialized());` then `unsafe { self.get_unchecked() }`; method get_mut_or_try_init(): `if self.get().is_none() { self.initialize(f)?; }` then `debug_assert!(self.is_initialized());` then `unsafe { self.get_unchecked_mut() }`; method is_initialized(): `self.once.is_completed()` encodes Initialized vs not; method initialize(): uses `self.once.call_once_force(|p| { ... })` and writes to `slot` only on `Ok(value)`; on `Err(e)` sets `res = Err(e)` and calls `p.poison()`; doc comment on get_or_try_init(): "If `f()` panics... the cell remains uninitialized"; doc comment on get_or_try_init(): "It is an error to reentrantly initialize the cell from `f`."; method take(&mut self): `if self.is_initialized() { self.once = Once::new(); ... assume_init_read() }` and comment: "self.once is reset ... prevents the value from being read twice"; unsafe fn get_unchecked()/get_unchecked_mut(): safety precondition stated in docs: "The cell must be initialized" and enforced only by `debug_assert!(self.is_initialized())`

**Implementation:** Represent the state at the type level: e.g., `OnceLock<T, S>` with `S = Uninit | Init`. Provide `fn try_init(self, f) -> Result<OnceLock<T, Init>, E>` (or a borrowed token/capability that proves initialization), and make `get_unchecked*` unnecessary by only exposing `&T`/`&mut T` on the `Init` state. For `take`, consume `OnceLock<T, Init>` and return `(T, OnceLock<T, Uninit>)` (or `OnceLock<T, Uninit>` + value) to encode the reset transition. Reentrancy can be prevented by passing an initialization capability that cannot be re-acquired inside `f` (or by making initialization take `&mut OnceLock<Uninit>` rather than `&OnceLock<_>`).

---

## Precondition Invariants

### 61. Tail snapshot / mark-bit masking invariant (Tail includes disconnection bit; masked tail used for indices and comparisons)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-148`

**Confidence**: medium

**Suggested Pattern**: newtype

**Description**: The `tail` atomic value is a tagged integer: it mixes position information with a disconnection marker (`mark_bit`). Many computations require masking out the mark bit (`tail & !self.mark_bit`) before comparing with `head` or using it as a logical cursor, while other operations intentionally preserve the bit to propagate disconnection. The correctness relies on consistently using raw vs masked tail in the right contexts (e.g., emptiness/fullness checks, discard loop termination). The type system does not distinguish a 'raw tagged tail' from a 'masked tail position', so passing the wrong form is a latent bug class.

**Evidence**:

```rust
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
    /// If a destructor panics, the remaining messages are leaked, matching the
    /// behavior of the unbounded channel.
    ///
    /// # Safety
    /// This method must only be called when dropping the last receiver. The
    /// destruction of all other receivers must have been observed with acquire
    /// ordering or stronger.
    unsafe fn discard_all_messages(&self, tail: usize) {
        debug_assert!(self.is_disconnected());

        // Only receivers modify `head`, so since we are the last one,
        // this value will not change and will not be observed (since
        // no new messages can be sent after disconnection).
        let mut head = self.head.load(Ordering::Relaxed);
        let tail = tail & !self.mark_bit;

        let backoff = Backoff::new();
        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may drop the message.
            if head + 1 == stamp {
                head = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                unsafe {
                    (*slot.msg.get()).assume_init_drop();
                }
            // If the tail equals the head, that means the channel is empty.
            } else if tail == head {
                return;
            // Otherwise, a sender is about to write into the slot, so we need
            // to wait for it to update the stamp.
            } else {
                backoff.spin_heavy();
            }
        }
    }

    /// Returns `true` if the channel is disconnected.
    pub(crate) fn is_disconnected(&self) -> bool {
        self.tail.load(Ordering::SeqCst) & self.mark_bit != 0
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);

        // Is the tail equal to the head?
        //
        // Note: If the head changes just before we load the tail, that means there was a moment
        // when the channel was not empty, so it is safe to just return `false`.
        (tail & !self.mark_bit) == head
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::SeqCst);
        let head = self.head.load(Ordering::SeqCst);

        // Is the head lagging one lap behind tail?
        //
        // Note: If the tail changes just before we load the head, that means there was a moment
        // when the channel was not full, so it is safe to just return `false`.
        head.wrapping_add(self.one_lap) == tail & !self.mark_bit
    }
}
```

**Entity:** Array channel internal struct (type owning head/tail/buffer; impl block shown)

**States:** TailRaw (may include mark_bit), TailMasked (mark_bit cleared; usable for head/tail arithmetic/indexing)

**Transitions:**
- TailRaw -> TailMasked via `tail & !self.mark_bit` (e.g., in is_empty/is_full/discard_all_messages)
- TailRaw(untagged) -> TailRaw(tagged/disconnected) via `fetch_or(self.mark_bit, ...)` (disconnect_* methods)

**Evidence:** disconnect_senders()/disconnect_receivers(): set disconnection bit via `self.tail.fetch_or(self.mark_bit, Ordering::SeqCst)`; is_empty(): compares `(tail & !self.mark_bit) == head` (explicit masking required for correctness); is_full(): compares `head.wrapping_add(self.one_lap) == tail & !self.mark_bit` (masked tail required); discard_all_messages(): `let tail = tail & !self.mark_bit;` followed by `} else if tail == head { return; }` relies on receiving a raw tail and then masking it exactly once

**Implementation:** Define `struct TaggedTail(usize); struct TailPos(usize);` with explicit constructors: `TaggedTail::load(&AtomicUsize)`, `TaggedTail::mark_disconnected(&AtomicUsize)`, and `impl TaggedTail { fn pos(self, mark_bit: usize) -> TailPos }`. Only `TailPos` exposes arithmetic/comparisons with `head`. This prevents accidental mixing of tagged and untagged values and documents intent in signatures (e.g., `discard_all_messages(&self, tail: TaggedTail)` or `tail_pos: TailPos`).

---

### 50. Non-reentrancy / thread-ownership precondition for locking

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-67`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: Lock acquisition has an implicit precondition that the lock is not already held by the current thread; violating it may panic. This is a runtime/behavioral constraint (and depends on OS/implementation details) rather than something represented in the type system, so the API allows calling read() even in situations where it will panic.

**Evidence**:

```rust
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
```

**Entity:** RwLock<T>

**States:** NotHeldByCurrentThread, HeldByCurrentThread (read or write)

**Transitions:**
- NotHeldByCurrentThread -> HeldByCurrentThread via read() (shared) / write() (exclusive) (implied by docs and usage in replace())
- HeldByCurrentThread -> NotHeldByCurrentThread when the returned RAII guard is dropped (described in docs)

**Evidence:** Doc comment on read(): "This function might panic when called if the lock is already held by the current thread."; Doc comment on read(): "Returns an RAII guard which will release this thread's shared access once it is dropped."; replace() calls self.write() and then mutates through the guard (`mem::replace(&mut *guard, value)`), indicating exclusive-hold is required for mutation

**Implementation:** Require an explicit non-reentrant capability/token to acquire the lock (or provide a separate API that proves non-reentrancy), e.g., a `ThreadLockToken` obtained from a scope that guarantees no nested acquisition, and `read(&self, token: &mut ThreadLockToken)`. While not always ergonomic/general-purpose, this can make reentrant acquisition impossible in APIs that need the guarantee (e.g., internal subsystems).

---

### 38. SyncSender thread-safety precondition (T: Send) for cross-thread transfer

**Location**: `/tmp/sync_test_crate/src/sync/mpsc.rs:1-89`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: SyncSender is only safe to move/send across threads when its payload type T is Send. This is enforced via an unsafe impl with a trait bound, meaning the safety relies on the invariant that the underlying synchronization/channel implementation does not allow non-Send T to be sent across threads. The state (safe vs not safe to send) is expressed as a trait-bound precondition rather than a dedicated type-level capability or wrapper indicating cross-thread use.

**Evidence**:

```rust
unsafe impl<T: Send> Send for SyncSender<T> {}

/// An error returned from the [`Sender::send`] or [`SyncSender::send`]
/// function on **channel**s.
///
/// A **send** operation can only fail if the receiving end of a channel is
/// disconnected, implying that the data could never be received. The error
/// contains the data being sent as a payload so it can be recovered.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(#[stable(feature = "rust1", since = "1.0.0")] pub T);

/// An error returned from the [`recv`] function on a [`Receiver`].
///
/// The [`recv`] operation can only fail if the sending half of a
/// [`channel`] (or [`sync_channel`]) is disconnected, implying that no further
/// messages will ever be received.
///
/// [`recv`]: Receiver::recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct RecvError;

/// This enumeration is the list of the possible reasons that [`try_recv`] could
/// not return data when called. This can occur with both a [`channel`] and
/// a [`sync_channel`].
///
/// [`try_recv`]: Receiver::try_recv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[stable(feature = "rust1", since = "1.0.0")]
pub enum TryRecvError {
    /// This **channel** is currently empty, but the **Sender**(s) have not yet
    /// disconnected, so data may yet become available.
    #[stable(feature = "rust1", since = "1.0.0")]
    Empty,

    /// The **channel**'s sending half has become disconnected, and there will
    /// never be any more data received on it.
    #[stable(feature = "rust1", since = "1.0.0")]
    Disconnected,
}

/// This enumeration is the list of possible errors that made [`recv_timeout`]
/// unable to return data when called. This can occur with both a [`channel`] and
/// a [`sync_channel`].
///
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

/// This enumeration is the list of the possible error outcomes for the
/// [`try_send`] method.
///
/// [`try_send`]: SyncSender::try_send
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    /// The data could not be sent on the [`sync_channel`] because it would require that
    /// the callee block to send the data.
    ///
    /// If this is a buffered channel, then the buffer is full at this time. If
    /// this is not a buffered channel, then there is no [`Receiver`] available to
    /// acquire the data.
    #[stable(feature = "rust1", since = "1.0.0")]
    Full(#[stable(feature = "rust1", since = "1.0.0")] T),

    /// This [`sync_channel`]'s receiving half has disconnected, so the data could not be
    /// sent. The data is returned back to the callee in this case.
    #[stable(feature = "rust1", since = "1.0.0")]
    Disconnected(#[stable(feature = "rust1", since = "1.0.0")] T),
}

/// Creates a new asynchronous channel, returning the sender/receiver halves.
///
/// All data sent on the [`Sender`] will become available on the [`Receiver`] in
/// the same order as it was sent, and no [`send`] will block the calling thread
/// (this channel has an "infinite buffer", unlike [`sync_channel`], which will
/// block after its buffer limit is reached). [`recv`] will block until a message
/// is available while there is at least one [`Sender`] alive (including clones).
```

**Entity:** SyncSender<T>

**States:** Not thread-transferable, Thread-transferable

**Transitions:**
- Not thread-transferable -> Thread-transferable by choosing T where T: Send (type-level constraint)

**Evidence:** code: `unsafe impl<T: Send> Send for SyncSender<T> {}` explicitly encodes the safety precondition and relies on unsafe implementation correctness

**Implementation:** Introduce a separate marker/capability type for cross-thread send (e.g., `struct CrossThread; struct Local; struct SyncSender<T, Mode>`), where `Mode=CrossThread` is only constructible when `T: Send`, and only `SyncSender<_, CrossThread>` implements `Send`.

---

### 40. ZeroToken validity protocol (Null = disconnected/unusable vs NonNull = active packet)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/zero.rs:1-105`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Channel::write() assumes token.zero contains a valid, live pointer to a packet slot. A null pointer is used as a sentinel meaning 'no packet' / 'channel disconnected', in which case write cannot proceed and returns Err(msg). This validity/disconnected state is encoded as a raw pointer value and checked at runtime; the type system does not prevent calling write() with an invalid token or one that is no longer associated with a live Packet.

**Evidence**:

```rust
//! Zero-capacity channel.
//!
//! This kind of channel is also known as *rendezvous* channel.

use super::context::Context;
use super::error::*;
use super::select::{Operation, Selected, Token};
use super::utils::Backoff;
use super::waker::Waker;
use crate::cell::UnsafeCell;
use crate::marker::PhantomData;
use crate::sync::Mutex;
use crate::sync::atomic::{Atomic, AtomicBool, Ordering};
use crate::time::Instant;
use crate::{fmt, ptr};

/// A pointer to a packet.
pub(crate) struct ZeroToken(*mut ());

impl Default for ZeroToken {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

impl fmt::Debug for ZeroToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&(self.0 as usize), f)
    }
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

/// Inner representation of a zero-capacity channel.
struct Inner {
    /// Senders waiting to pair up with a receive operation.
    senders: Waker,

    /// Receivers waiting to pair up with a send operation.
    receivers: Waker,

    /// Equals `true` when the channel is disconnected.
    is_disconnected: bool,
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
```

**Entity:** Token / ZeroToken

**States:** Null (no packet / disconnected), NonNull (points to a live Packet)

**Transitions:**
- NonNull -> Null via higher-level disconnect / token invalidation (implied by null-sentinel check in write())

**Evidence:** pub(crate) struct ZeroToken(*mut ()); (raw pointer encodes state); impl Default for ZeroToken { Self(ptr::null_mut()) } (default token is Null); Channel::write(): `if token.zero.0.is_null() { return Err(msg); }` (runtime guard + error meaning disconnected/unusable)

**Implementation:** Replace `ZeroToken(*mut ())` with an enum like `enum ZeroToken { Disconnected, Packet(NonNull<Packet<T>>), }` (or a generic `ZeroToken<T>`), so `write()` can accept only `ZeroToken::Packet(...)` (or a `NonNull` wrapper) and make the disconnected case unrepresentable at the call site.

---

### 46. Exclusive mutable access protocol for get_mut_or_init (requires unique ownership)

**Location**: `/tmp/sync_test_crate/src/sync/once_lock.rs:1-69`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The API distinguishes shared initialization (`get_or_init(&self, ...) -> &T`) from exclusive mutable initialization (`get_mut_or_init(&mut self, ...) -> &mut T`, referenced by docs). The latent invariant is that `get_mut_or_init` can only be called when the caller has unique access to the OnceLock (no concurrent callers), which is why it "never blocks" and returns `&mut T`. This exclusivity is enforced by requiring `&mut self` at the call site, but the 'never blocks' / 'no concurrent initialization' protocol is only documented and depends on callers not sharing the cell during the mutable borrow window.

**Evidence**:

```rust
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
```

**Entity:** OnceLock<T>

**States:** Uninitialized, Initialized

**Transitions:**
- Uninitialized -> Initialized via get_mut_or_init() running f under &mut self exclusivity

**Evidence:** doc comment: "Gets the mutable reference of the contents of the cell, initializing it to `f()` if the cell was uninitialized." (Uninitialized/Initialized split); doc comment: "This method never blocks." (relies on exclusivity / absence of contention); examples show `let mut cell = OnceLock::new(); let value = cell.get_mut_or_init(|| 92);` (implies the method requires unique access to the cell, not expressed as a distinct type/state beyond `&mut self` borrowing); comment under # Panics: "If `f()` panics ... the cell remains uninitialized." (same rollback invariant applies to this path)

**Implementation:** Model the "exclusive init" mode with an explicit capability token (e.g., `ExclusiveOnceLock<'a, T>` obtained from `&'a mut OnceLock<T>`) that alone exposes `get_mut_or_init`. This makes the non-blocking/exclusive protocol more explicit in the API surface and can help prevent mixing shared and exclusive initialization patterns in higher-level abstractions.

---

## Protocol Invariants

### 6. Condvar wait/notify predicate protocol (lock-then-check; update-under-lock; notify-after-update)

**Location**: `/tmp/sync_test_crate/src/sync/poison/condvar.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: The intended usage relies on an implicit protocol: a shared predicate (often a boolean) must be checked while holding the mutex; if false, the thread waits on the condvar; the notifying thread must acquire the same mutex, update the predicate, then notify. This ordering is described in documentation and demonstrated in the example, but the type system does not enforce that the predicate is checked under the lock, that the predicate is updated under the lock before notifying, or that the same predicate/mutex pair is used consistently. Misordering can lead to missed wakeups or logic bugs, and is currently prevented only by convention and documentation.

**Evidence**:

```rust
    /// let pair2 = Arc::clone(&pair);
    ///
    /// # let handle =
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///
    ///     // Let's wait 20 milliseconds before notifying the condvar.
    ///     thread::sleep(Duration::from_millis(20));
    ///
    ///     let mut started = lock.lock().unwrap();
    ///     // We update the boolean value.
    ///     *started = true;
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// loop {
    ///     // Let's put a timeout on the condvar's wait.
    ///     let result = cvar.wait_timeout(lock.lock().unwrap(), Duration::from_millis(10)).unwrap();
    ///     // 10 milliseconds have passed.
    ///     if result.1.timed_out() {
    ///         // timed out now and we can leave.
    ///         break
    ///     }
    /// }
    /// # // Prevent leaks for Miri.
    /// # let _ = handle.join();
    /// ```
    #[must_use]
    #[stable(feature = "wait_timeout", since = "1.5.0")]
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

/// A Condition Variable
///
/// Condition variables represent the ability to block a thread such that it
/// consumes no CPU time while waiting for an event to occur. Condition
/// variables are typically associated with a boolean predicate (a condition)
/// and a mutex. The predicate is always verified inside of the mutex before
/// determining that a thread must block.
///
/// Functions in this module will block the current **thread** of execution.
/// Note that any attempt to use multiple mutexes on the same condition
/// variable may result in a runtime panic.
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
/// // Inside of our lock, spawn a new thread, and then wait for it to start.
/// thread::spawn(move || {
///     let (lock, cvar) = &*pair2;
///     let mut started = lock.lock().unwrap();
///     *started = true;
///     // We notify the condvar that the value has changed.
///     cvar.notify_one();
```

**Entity:** Condvar (usage protocol with predicate under Mutex)

**States:** PredicateFalse (must wait), PredicateTrue (may proceed)

**Transitions:**
- PredicateFalse -> PredicateTrue via updater thread: lock.lock() then *started = true then cvar.notify_one()
- PredicateFalse (observer) -> PredicateFalse via cvar.wait_timeout(...) spurious wake/timeout; must re-check predicate under lock in a loop
- PredicateFalse (observer) -> PredicateTrue via cvar.wait()/wait_timeout() returning and predicate observed true under lock

**Evidence:** comment: "Condition variables are typically associated with a boolean predicate (a condition) and a mutex. The predicate is always verified inside of the mutex before determining that a thread must block."; example code: updater holds the lock when updating: `let mut started = lock.lock().unwrap(); *started = true; cvar.notify_one();`; example code: waiter loops and calls `cvar.wait_timeout(lock.lock().unwrap(), Duration::from_millis(10))` indicating the re-check-in-loop protocol around waits

**Implementation:** Model the pattern as a small API that couples (Mutex<T>, Condvar) with a predicate-checking closure, e.g. `struct Waiter<'a, T> { lock: &'a Mutex<T>, cvar: &'a Condvar }` and provide `wait_while(guard, |t| ...)` / `wait_timeout_while(...)` so waiting is only expressible together with predicate checking. Alternatively, introduce a newtype wrapper around the protected predicate that only allows mutation while a guard is held, and expose `notify_one_after_update(guard, update_fn)` to enforce update-before-notify sequencing in the API surface.

---

### 21. Two-phase reservation protocol for slot access (ReservedSend/ReservedRecv/Disconnected)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-222`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Token carries implicit state produced by start_send/start_recv and consumed by write/read. start_send prepares token.array.slot and token.array.stamp for a subsequent write(); start_recv prepares them for a subsequent read(). Disconnection is encoded by setting token.array.slot to null and stamp to 0, and write/read interpret a null slot as disconnected. None of this is enforced by the type system: write/read are unsafe and accept any &mut Token, so callers must uphold the temporal ordering (call start_* first, then the matching read/write exactly once, and not mix send/recv tokens).

**Evidence**:

```rust
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
```

**Entity:** Token (token.array.* used by Channel::{start_send,write,start_recv,read})

**States:** Fresh/Unreserved, ReservedForSend, ReservedForRecv, DisconnectedSentinel

**Transitions:**
- Fresh/Unreserved -> ReservedForSend via Channel::start_send (when it CASes tail successfully)
- Fresh/Unreserved -> ReservedForRecv via Channel::start_recv (when it CASes head successfully)
- Fresh/Unreserved -> DisconnectedSentinel via Channel::start_send when (tail & mark_bit) != 0
- Fresh/Unreserved -> DisconnectedSentinel via Channel::start_recv when empty && (tail & mark_bit) != 0
- ReservedForSend -> Fresh/Unreserved via Channel::write (conceptually completes the reservation by publishing stamp)
- ReservedForRecv -> Fresh/Unreserved via Channel::read (conceptually completes the reservation by freeing slot stamp)

**Evidence:** fn start_send(&self, token: &mut Token) -> bool: on success sets token.array.slot and token.array.stamp with comment 'Prepare the token for the follow-up call to `write`'; start_send: disconnect path sets token.array.slot = ptr::null(); token.array.stamp = 0; and returns true; pub(crate) unsafe fn write(&self, token: &mut Token, msg: T) -> Result<(), T>: checks token.array.slot.is_null() to decide 'channel is disconnected'; fn start_recv(&self, token: &mut Token) -> bool: on success sets token.array.slot and token.array.stamp with comment 'Prepare the token for the follow-up call to `read`'; start_recv: disconnect path sets token.array.slot = ptr::null(); token.array.stamp = 0; and returns true; pub(crate) unsafe fn read(&self, token: &mut Token) -> Result<T, ()>: checks token.array.slot.is_null() and returns Err(()) for disconnected

**Implementation:** Replace Token with a typed reservation object returned from start_send/start_recv, e.g. start_send(&self) -> Result<SendPermit<'_, T>, FullOrDisconnected>; start_recv(&self) -> Result<RecvPermit<'_, T>, EmptyOrDisconnected>. SendPermit::write(self, msg) and RecvPermit::read(self) consume the permit, preventing reuse/mixing and eliminating the null-sentinel encoding.

---

### 3. Waker registration lifecycle (Registered -> Unregistered/Notified) with packet ownership protocol

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/waker.rs:1-209`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: Waker maintains two internal queues (selectors/observers) of per-thread Entries. Callers implicitly rely on a lifecycle: threads register an Entry, then later must unregister it themselves (especially after disconnect), and the Waker must be empty at drop. Additionally, when registering with a packet pointer, the packet is expected to be stored into the Context exactly when selection succeeds; packet lifetime/ownership is not represented in types (raw *mut ()). None of these protocols are enforced by the type system: entries can be forgotten (violating drop assertions), unregister can be skipped after disconnect (by design but still a protocol), and packet validity/lifetime is unchecked.

**Evidence**:

```rust
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

**States:** Empty, HasSelectors, HasObservers, DisconnectedNotified (selectors retained)

**Transitions:**
- Empty -> HasSelectors via register()/register_with_packet()
- HasSelectors -> (HasSelectors or Empty) via unregister()
- HasSelectors -> (HasSelectors or Empty) via try_select() (removes one selected entry)
- HasObservers -> (HasObservers or Empty) via notify() (drain(..))
- HasSelectors/HasObservers -> DisconnectedNotified (selectors retained) via disconnect()

**Evidence:** field: Waker { selectors: Vec<Entry>, observers: Vec<Entry> } encodes runtime membership/state; method: register_with_packet() pushes Entry into selectors; packet is raw `*mut ()`; method: unregister() searches by `entry.oper == oper` and removes; returns Option<Entry>; method: try_select(): on successful `cx.try_select(...)` it calls `cx.store_packet(selector.packet)` then `cx.unpark()`, then removes the entry from selectors; method: notify(): `for entry in self.observers.drain(..)` drains all observers, selects and unparks; method: disconnect() comment: "Registered threads must unregister from the waker by themselves. They might also want to recover the packet value and destroy it" (explicit protocol); Drop for Waker: `debug_assert_eq!(self.selectors.len(), 0)` and same for observers (requires empty-at-drop invariant)

**Implementation:** Return an RAII registration token from register (e.g., `Registration<'a>` or `RegisteredOp`) that on Drop automatically unregisters from the Waker. Encode the packet as a typed owned payload/capability (e.g., `Packet<T>` or `NonNull<T>` with lifetime) rather than `*mut ()`, and make `register_with_packet` take that token so packet validity is tied to the registration lifetime. Optionally split Waker into `Waker<Connected>`/`Waker<Disconnected>` typestate if disconnect changes allowed operations (e.g., no new registrations) in the broader module.

---

### 13. OnceState validity/usage protocol (only meaningful inside call_once_force closure)

**Location**: `/tmp/sync_test_crate/src/sync/poison/once.rs:1-87`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: `OnceState` represents transient information about a particular `Once` execution (notably poisoning) and is intended to be used only within the dynamic extent of `Once::call_once_force()`'s closure. The type, as shown, is just a wrapper around `sys::OnceState` and does not encode any lifetime relationship to the `Once` instance or to the closure call, so the type system cannot prevent capturing/storing `OnceState` for later use where its information may be stale or meaningless relative to the `Once` it came from.

**Evidence**:

```rust
///
/// This type can only be constructed with [`Once::new()`].
///
/// # Examples
///
/// ```
/// use std::sync::Once;
///
/// static START: Once = Once::new();
///
/// START.call_once(|| {
///     // run initialization here
/// });
/// ```
///
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

/// State yielded to [`Once::call_once_force()`]’s closure parameter. The state
/// can be used to query the poison status of the [`Once`].
#[stable(feature = "once_poison", since = "1.51.0")]
pub struct OnceState {
    pub(crate) inner: sys::OnceState,
}

pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}

/// Initialization value for static [`Once`] values.
///
/// # Examples
///
/// ```
/// use std::sync::{Once, ONCE_INIT};
///
/// static START: Once = ONCE_INIT;
/// ```
#[stable(feature = "rust1", since = "1.0.0")]
#[deprecated(
    since = "1.38.0",
    note = "the `Once::new()` function is now preferred",
    suggestion = "Once::new()"
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
```

**Entity:** OnceState

**States:** EphemeralValid (during call_once_force closure), Invalid/Unavailable (outside that context)

**Transitions:**
- Invalid/Unavailable -> EphemeralValid when call_once_force supplies a OnceState to the closure (implied by docs reference)
- EphemeralValid -> Invalid/Unavailable when the closure returns (intended ephemeral nature not encoded)

**Evidence:** type `pub struct OnceState { pub(crate) inner: sys::OnceState }` — no lifetime tying it to a particular `Once` or call scope; comment: 'State yielded to Once::call_once_force()’s closure parameter' — indicates it is meant to be created/provided only for that call; comment: 'can be used to query the poison status of the Once' — context-dependent, tied to a particular Once execution

**Implementation:** Make `OnceState` carry a lifetime and be unnameable outside the closure, e.g., `pub struct OnceState<'a> { inner: sys::OnceState, _once: PhantomData<&'a Once> }` and have `call_once_force` pass `OnceState<'_>` so it cannot be stored beyond the call. Alternatively, make it a closure-only token type (private constructor + lifetime) to enforce scoped usage.

---

### 58. MPMC select protocol (Waiting -> Selected [+ optional Packet] -> Reset)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/context.rs:1-65`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Context encodes a multi-step selection protocol using atomics: callers are expected to (1) be in the Waiting state, (2) successfully claim selection via try_select(select), and then (3) optionally publish an associated packet pointer via store_packet(packet) if there is one. Later, the context is reused by calling reset(), which returns it to Waiting and clears the packet pointer. These ordering requirements are only documented and enforced by convention/atomic state, not by the type system: store_packet() is callable even if try_select() never succeeded, and nothing prevents double-select or using a stale packet if reset() is not performed before reuse.

**Evidence**:

```rust
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
```

**Entity:** Context

**States:** Waiting, SelectedNoPacket, SelectedWithPacket

**Transitions:**
- Waiting -> SelectedNoPacket via try_select(select) (Ok)
- Waiting -> SelectedWithPacket via try_select(select) (Ok) then store_packet(non-null)
- SelectedNoPacket -> SelectedWithPacket via store_packet(non-null)
- Selected* -> Waiting via reset()

**Evidence:** Inner.select: AtomicUsize initialized to Selected::Waiting in Context::new(); Context::reset(): self.inner.select.store(Selected::Waiting.into(), ...) and self.inner.packet.store(ptr::null_mut(), ...); Context::try_select(): compare_exchange(Selected::Waiting.into(), select.into(), ...) indicates a state machine with a single successful transition out of Waiting; Comment on store_packet(): "This method must be called after `try_select` succeeds and there is a packet to provide."; Context::store_packet(): writes to self.inner.packet with only a null check; no check that a selection has occurred

**Implementation:** Introduce typestate wrappers for the protocol steps, e.g., Context<Waiting>, Context<Selected>. Make try_select(self, select) -> Result<Context<Selected>, Selected> (or a guard token). Only implement store_packet() on Context<Selected> (or require a SelectionToken returned by try_select). Provide reset(self) -> Context<Waiting> to enable explicit, typed reuse.

---

### 64. Initialization/ready-to-drop protocol for block pointer during discard_all_messages (Uninitialized / Initializing / Initialized)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/list.rs:1-186`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: discard_all_messages() contains a synchronization protocol with channel initialization: the channel may be uninitialized, and head.block can be null even when head/tail indicate there are messages (a sender may be initializing while another sender advanced tail). The function busy-waits until `head.block` becomes non-null when it must drop messages. This reveals an implicit temporal ordering requirement between initialization of the first block and operations that traverse/drop messages; the protocol is enforced with spinning and null checks rather than a type-level 'initialized' state or a one-time init primitive.

**Evidence**:

```rust
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
        // will be deallocated by the sender in Drop.
        let mut block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);

        // If we're going to be dropping messages we need to synchronize with initialization
        if head >> SHIFT != tail >> SHIFT {
            // The block can be null here only if a sender is in the process of initializing the
            // channel while another sender managed to send a message by inserting it into the
            // semi-initialized channel and advanced the tail.
            // In that case, just wait until it gets initialized.
            while block.is_null() {
                backoff.spin_heavy();
                block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);
            }
        }
        // After this point `head.block` is not modified again and it will be deallocated if it's
        // non-null. The `Drop` code of the channel, which runs after this function, also attempts
        // to deallocate `head.block` if it's non-null. Therefore this function must maintain the
        // invariant that if a deallocation of head.block is attemped then it must also be set to
        // NULL. Failing to do so will lead to the Drop code attempting a double free. For this
        // reason both reads above do an atomic swap instead of a simple atomic load.

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head >> SHIFT != tail >> SHIFT {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    slot.wait_write();
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    (*block).wait_next();
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Acquire);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }

        head &= !MARK_BIT;
        self.head.index.store(head, Ordering::Release);
    }

    /// Returns `true` if the channel is disconnected.
    pub(crate) fn is_disconnected(&self) -> bool {
        self.tail.index.load(Ordering::SeqCst) & MARK_BIT != 0
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.index.load(Ordering::SeqCst);
        let tail = self.tail.index.load(Ordering::SeqCst);
        head >> SHIFT == tail >> SHIFT
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        false
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        let mut head = self.head.index.load(Ordering::Relaxed);
        let mut tail = self.tail.index.load(Ordering::Relaxed);
        let mut block = self.head.block.load(Ordering::Relaxed);

        // Erase the lower bits.
        head &= !((1 << SHIFT) - 1);
        tail &= !((1 << SHIFT) - 1);

        unsafe {
            // Drop all messages between head and tail and deallocate the heap-allocated blocks.
            while head != tail {
                let offset = (head >> SHIFT) % LAP;

                if offset < BLOCK_CAP {
                    // Drop the message in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    let p = &mut *slot.msg.get();
                    p.as_mut_ptr().drop_in_place();
                } else {
                    // Deallocate the block and move to the next one.
                    let next = (*block).next.load(Ordering::Relaxed);
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }
    }
}
```

**Entity:** Channel<T>

**States:** Uninitialized (head.block is null, no blocks yet), Initializing (senders may be allocating/setting head.block), Initialized (head.block non-null and usable for dropping messages)

**Transitions:**
- Uninitialized -> Initializing via sender-side allocation/initialization (implied by comments)
- Initializing -> Initialized when `head.block` becomes non-null (observed by spinning in discard_all_messages)
- Initialized -> (consumed) via `swap(ptr::null_mut())` in discard_all_messages taking ownership

**Evidence:** comment in discard_all_messages(): `The channel may be uninitialized... swap to avoid overwriting any sender's attempts to initialize the first block...` — explicit initialization protocol; discard_all_messages(): `let mut block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel);` followed by `while block.is_null() { backoff.spin_heavy(); block = self.head.block.swap(ptr::null_mut(), Ordering::AcqRel); }` — runtime waiting for initialization completion; comment: `The block can be null here only if a sender is in the process of initializing the channel... In that case, just wait until it gets initialized.` — states and ordering requirement described in prose

**Implementation:** Split the channel internals into an init state machine, e.g. `ChannelInner<Uninit>` that can only be used to send/receive after an initialization transition `init(self) -> ChannelInner<Init>`. Alternatively, store the first block in a `OnceLock<NonNull<Block<T>>>` (or equivalent) so that consumers can safely wait for initialization via a well-defined API instead of repeated null-swaps/spins.

---

### 52. Mapping protocol for guards (valid projection must stay within locked data)

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-67`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: MappedMutexGuard::map performs a one-way projection of a guard from `T` to a subcomponent `U`. The comment claims it 'cannot fail' because the mutex is already locked, but correctness additionally depends on the closure returning a reference that is actually derived from (and does not outlive) the locked `T`. This is a semantic precondition on `F: FnOnce(&mut T) -> &mut U` that the compiler cannot fully enforce (e.g., `U` must be a true projection into `T`, not a reference to unrelated memory obtained via interior tricks).

**Evidence**:

```rust
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
impl<T: ?Sized> Deref for MappedMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> DerefMut for MappedMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.poison_flag.done(&self.poison);
            self.inner.unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for MappedMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Display> fmt::Display for MappedMutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: ?Sized> MappedMutexGuard<'a, T> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedMutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(mut orig: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
```

**Entity:** MappedMutexGuard::map

**States:** Unmapped guard (points to T), Mapped/projection guard (points to U within T)

**Transitions:**
- Unmapped -> Mapped via `MappedMutexGuard::map(orig, f)`

**Evidence:** comment on map: `The Mutex is already locked, so this cannot fail.` (implies a protocol: mapping assumes an already-held lock and safe projection); signature: `pub fn map<U, F>(mut orig: Self, f: F) -> MappedMutexGuard<'a, U> where F: FnOnce(&mut T) -> &mut U` (projection relies on `f` obeying 'returns a sub-borrow of T' rule); Deref/DerefMut use `unsafe { self.data.as_ref()/as_mut() }`, so any invalid projection created by `map` would become UB when dereferenced

**Implementation:** Restrict mapping to safe, structured projections by making `map` take a capability that can only be used to derive sub-borrows from the original data (e.g., provide a `Projection<'a, T>` token that only exposes safe field/variant projection APIs), or model a limited set of projections via newtypes/closures that cannot fabricate unrelated `&mut U` (e.g., sealed trait `Project<T, U>` implemented only for known-safe projections).

---

### 5. Condvar single-associated-mutex protocol (bound-to-one Mutex / unbound)

**Location**: `/tmp/sync_test_crate/src/sync/poison/condvar.rs:1-64`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: A Condvar is implicitly expected to be used with a single associated mutex across all waits. The docs state that attempting to use multiple mutexes with the same condition variable may panic at runtime. This indicates an implicit binding/association that is not represented in the type system: after a Condvar has been used to wait with one Mutex, further waits are only valid with that same Mutex. The current API accepts any mutex guard, so misuse is only prevented by runtime checks/panics (or not prevented at all depending on implementation).

**Evidence**:

```rust
    /// let pair2 = Arc::clone(&pair);
    ///
    /// # let handle =
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///
    ///     // Let's wait 20 milliseconds before notifying the condvar.
    ///     thread::sleep(Duration::from_millis(20));
    ///
    ///     let mut started = lock.lock().unwrap();
    ///     // We update the boolean value.
    ///     *started = true;
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// loop {
    ///     // Let's put a timeout on the condvar's wait.
    ///     let result = cvar.wait_timeout(lock.lock().unwrap(), Duration::from_millis(10)).unwrap();
    ///     // 10 milliseconds have passed.
    ///     if result.1.timed_out() {
    ///         // timed out now and we can leave.
    ///         break
    ///     }
    /// }
    /// # // Prevent leaks for Miri.
    /// # let _ = handle.join();
    /// ```
    #[must_use]
    #[stable(feature = "wait_timeout", since = "1.5.0")]
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

/// A Condition Variable
///
/// Condition variables represent the ability to block a thread such that it
/// consumes no CPU time while waiting for an event to occur. Condition
/// variables are typically associated with a boolean predicate (a condition)
/// and a mutex. The predicate is always verified inside of the mutex before
/// determining that a thread must block.
///
/// Functions in this module will block the current **thread** of execution.
/// Note that any attempt to use multiple mutexes on the same condition
/// variable may result in a runtime panic.
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
/// // Inside of our lock, spawn a new thread, and then wait for it to start.
/// thread::spawn(move || {
///     let (lock, cvar) = &*pair2;
///     let mut started = lock.lock().unwrap();
///     *started = true;
///     // We notify the condvar that the value has changed.
///     cvar.notify_one();
```

**Entity:** Condvar

**States:** Unbound (no mutex observed yet), BoundToMutex(M), MisusedWithDifferentMutex (panic)

**Transitions:**
- Unbound -> BoundToMutex(M) via first wait()/wait_timeout() call with a guard from mutex M
- BoundToMutex(M) -> MisusedWithDifferentMutex via wait()/wait_timeout() called with guard from different mutex N (runtime panic per docs)

**Evidence:** comment: "Note that any attempt to use multiple mutexes on the same condition variable may result in a runtime panic."; example usage: "let (lock, cvar) = &*pair;" shows condvar paired with a particular Mutex in a tuple and used consistently with that lock; method name in example: cvar.wait_timeout(lock.lock().unwrap(), ...) demonstrates the API takes a mutex guard dynamically rather than being statically tied to a specific Mutex

**Implementation:** Introduce a mutex-bound condvar capability type, e.g. CondvarFor<'a, M> produced by pairing/binding: `let (lock, cvar) = ...; let cvar = cvar.bind(&lock);`. Then only `CondvarFor<'_, M>` exposes `wait`/`wait_timeout` that accept `MutexGuard<'_, M::Data>` from that specific mutex. This encodes the single-mutex association at compile time and makes using the same Condvar with a different Mutex a type error.

---

### 16. RwLockWriteGuard construction protocol (Locked-for-write -> Guard-instantiated)

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-63`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Creating an RwLockWriteGuard via unsafe new() is only valid if the current thread has already successfully acquired a write lock through lock.inner.write()/try_write(). This is an implicit temporal/ownership invariant enforced only by an unsafe contract (comment). The type system does not require proof that the write lock is held when constructing the guard; correctness depends on callers following the documented protocol.

**Evidence**:

```rust
}

#[stable(feature = "rw_lock_from", since = "1.24.0")]
impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}

impl<'rwlock, T: ?Sized> RwLockReadGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockReadGuard<T>` from a `RwLock<T>`.
    ///
    /// # Safety
    ///
    /// This function is safe if and only if the same thread has successfully and safely called
    /// `lock.inner.read()`, `lock.inner.try_read()`, or `lock.inner.downgrade()` before
    /// instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockReadGuard<'rwlock, T>> {
        poison::map_result(lock.poison.borrow(), |()| RwLockReadGuard {
            data: unsafe { NonNull::new_unchecked(lock.data.get()) },
            inner_lock: &lock.inner,
        })
    }
}

impl<'rwlock, T: ?Sized> RwLockWriteGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockWriteGuard<T>` from a `RwLock<T>`.
    // SAFETY: if and only if `lock.inner.write()` (or `lock.inner.try_write()`) has been
    // successfully called from the same thread before instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockWriteGuard<'rwlock, T>> {
        poison::map_result(lock.poison.guard(), |guard| RwLockWriteGuard { lock, poison: guard })
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

```

**Entity:** RwLockWriteGuard<'rwlock, T>

**States:** NoWriteLockHeld, WriteLockHeld(inside lock.inner), WriteGuardAlive

**Transitions:**
- NoWriteLockHeld -> WriteLockHeld(inside lock.inner) via lock.inner.write()/try_write() (mentioned in comment)
- WriteLockHeld(inside lock.inner) -> WriteGuardAlive via unsafe RwLockWriteGuard::new(lock)

**Evidence:** unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockWriteGuard<'rwlock, T>>; comment: "SAFETY: if and only if lock.inner.write() (or lock.inner.try_write()) has been successfully called from the same thread before instantiating this object."; body constructs guard without lock-token: `RwLockWriteGuard { lock, poison: guard }` after `lock.poison.guard()`

**Implementation:** As with read guards, thread the result/capability of `lock.inner.write()`/`try_write()` into guard construction (e.g., an internal `HeldWrite<'a>` token that cannot be forged). Then only code that actually acquired the write lock can construct `RwLockWriteGuard`, turning the unsafe precondition into a compile-time requirement.

---

### 10. MappedMutexGuard mapping protocol (Derived view must not outlive original lock/poison context)

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-113`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: Mapping a MutexGuard into a MappedMutexGuard relies on an implicit protocol: the original mutex must remain locked, the mapped pointer must remain within the same allocation as the locked data, and the closure must not leak references beyond the guard lifetime. These constraints are upheld by consuming `orig`, using `ManuallyDrop` to suppress its Drop, and storing raw pointers (`NonNull`) plus references to the mutex internals. The safety relies on comments and closure signature assumptions rather than a fully explicit type-level relationship that the mapped guard is a reborrow/derivation of the original guard state.

**Evidence**:

```rust

#[stable(feature = "mutex_default", since = "1.10.0")]
impl<T: ?Sized + Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                d.field("data", &&*guard);
            }
            Err(TryLockError::Poisoned(err)) => {
                d.field("data", &&**err.get_ref());
            }
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            }
        }
        d.field("poisoned", &self.poison.get());
        d.finish_non_exhaustive()
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

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

pub fn guard_lock<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a sys::Mutex {
    &guard.lock.inner
}

pub fn guard_poison<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a poison::Flag {
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
```

**Entity:** MappedMutexGuard<'a, U> (constructed via MutexGuard::map)

**States:** DerivedGuardValid, DerivedGuardInvalidAfterDrop

**Transitions:**
- DerivedGuardValid -> DerivedGuardInvalidAfterDrop via dropping the MappedMutexGuard (implied unlock/poison completion in its Drop, not shown here)

**Evidence:** MutexGuard::map consumes the original guard: `pub fn map<U, F>(orig: Self, f: F) -> MappedMutexGuard<'a, U>`; Safety comment states protocol assumptions: `// SAFETY: the conditions of MutexGuard::new were satisfied ... upheld throughout map and/or filter_map ... closure guarantees that it will not "leak" the lifetime ... If the closure panics, the guard will be dropped.`; Creates a raw non-null pointer into the locked data: `let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));`; Suppresses original guard's Drop (and thus its unlock) to transfer responsibility: `let orig = ManuallyDrop::new(orig);`; Stores references to mutex internals rather than the original guard: `inner: &orig.lock.inner, poison_flag: &orig.lock.poison, poison: orig.poison.clone()`

**Implementation:** Model mapping as a typed reborrow tied to a live guard capability: `fn map<'g, U>(&'g mut self, f: impl FnOnce(&mut T)->&mut U) -> MappedMutexGuard<'g, U>`, so the mapped guard is statically a sub-borrow of an existing live guard (no `ManuallyDrop`, no raw transfer). Alternatively, introduce an internal `GuardToken<'a>` capability that is moved into either `MutexGuard` or `MappedMutexGuard` but cannot be duplicated, making the 'exactly one drop path unlocks' protocol explicit.

---

### 15. RwLockReadGuard construction protocol (Locked-for-read -> Guard-instantiated)

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-63`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: Creating an RwLockReadGuard via unsafe new() is only valid if the current thread has already successfully acquired (or downgraded to) a read lock through lock.inner.read()/try_read()/downgrade(). This ordering/ownership requirement is enforced only by an unsafe contract and comments; the type system does not tie possession of the underlying read-lock acquisition to the ability to construct the guard. Additionally, the guard stores a NonNull pointer to lock.data.get(), relying on the lock still being alive and properly locked for the guard’s lifetime; that relationship is partially expressed by the 'rwlock lifetime but the 'lock is held' fact is not represented as a capability/token.

**Evidence**:

```rust
}

#[stable(feature = "rw_lock_from", since = "1.24.0")]
impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}

impl<'rwlock, T: ?Sized> RwLockReadGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockReadGuard<T>` from a `RwLock<T>`.
    ///
    /// # Safety
    ///
    /// This function is safe if and only if the same thread has successfully and safely called
    /// `lock.inner.read()`, `lock.inner.try_read()`, or `lock.inner.downgrade()` before
    /// instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockReadGuard<'rwlock, T>> {
        poison::map_result(lock.poison.borrow(), |()| RwLockReadGuard {
            data: unsafe { NonNull::new_unchecked(lock.data.get()) },
            inner_lock: &lock.inner,
        })
    }
}

impl<'rwlock, T: ?Sized> RwLockWriteGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockWriteGuard<T>` from a `RwLock<T>`.
    // SAFETY: if and only if `lock.inner.write()` (or `lock.inner.try_write()`) has been
    // successfully called from the same thread before instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockWriteGuard<'rwlock, T>> {
        poison::map_result(lock.poison.guard(), |guard| RwLockWriteGuard { lock, poison: guard })
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_debug", since = "1.16.0")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "std_guard_impls", since = "1.20.0")]
impl<T: ?Sized + fmt::Display> fmt::Display for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

```

**Entity:** RwLockReadGuard<'rwlock, T>

**States:** NoReadLockHeld, ReadLockHeld(inside lock.inner), ReadGuardAlive

**Transitions:**
- NoReadLockHeld -> ReadLockHeld(inside lock.inner) via lock.inner.read()/try_read()/downgrade() (mentioned in safety docs)
- ReadLockHeld(inside lock.inner) -> ReadGuardAlive via unsafe RwLockReadGuard::new(lock)

**Evidence:** unsafe fn new(lock: &'rwlock RwLock<T>) -> LockResult<RwLockReadGuard<'rwlock, T>>; doc comment: "safe if and only if the same thread has successfully and safely called lock.inner.read(), lock.inner.try_read(), or lock.inner.downgrade() before instantiating this object"; construction uses raw pointer: `NonNull::new_unchecked(lock.data.get())` (requires data to be valid + protected by the lock); stores `inner_lock: &lock.inner` without any typed proof that a read-lock is currently held

**Implementation:** Introduce an internal, non-public capability token representing a held read lock (e.g., `struct HeldRead<'a>(&'a sys::RwLock)` returned by `lock.inner.read()`), and make `RwLockReadGuard::new` accept that token (`fn new(lock: &'a RwLock<T>, held: HeldRead<'a>) -> ...`). This would make it impossible to construct a guard without first acquiring the lock, without relying on `unsafe` preconditions.

---

### 56. Select/registration protocol (Registered -> (Unregistered | Completed)) around wait_until()

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/zero.rs:1-63`

**Confidence**: high

**Suggested Pattern**: raii

**Description**: The code follows an implicit multi-step protocol: an operation (`oper`) is registered (not shown in snippet), the current thread blocks via `cx.wait_until(deadline)`, and then the result must be handled. If the wait ends with `Aborted` or `Disconnected`, the operation must be unregistered before returning an error. If it ends with `Operation(_)`, the code must wait for readiness and then read the message. This is enforced by convention and runtime `unregister(oper).unwrap()` calls rather than by types; misuse (e.g., forgetting to unregister on some path, unregistering twice, or reading without readiness) is not prevented at compile time.

**Evidence**:

```rust
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

**Entity:** Receiver/operation wait path (the method containing `cx.wait_until(deadline)` and `unregister(oper)`)

**States:** Registered, Waiting, Aborted, Disconnected, OperationCompleted, Unregistered

**Transitions:**
- Registered -> Waiting via `cx.wait_until(deadline)`
- Waiting -> Unregistered via `Selected::Aborted` branch then `receivers.unregister(oper).unwrap()`
- Waiting -> Unregistered via `Selected::Disconnected` branch then `receivers.unregister(oper).unwrap()`
- Waiting -> OperationCompleted via `Selected::Operation(_)` then `packet.wait_ready()` and message read

**Evidence:** comment: `// Block the current thread.` followed by `let sel = unsafe { cx.wait_until(deadline) };` indicates a required temporal step in the protocol; comment: `// SAFETY: the context belongs to the current thread.` implies a thread-affinity precondition for using `cx` (not modeled in types); Selected::Aborted branch: `...receivers.unregister(oper).unwrap(); Err(RecvTimeoutError::Timeout)` shows mandatory cleanup on timeout/abort; Selected::Disconnected branch: `...receivers.unregister(oper).unwrap(); Err(RecvTimeoutError::Disconnected)` shows mandatory cleanup on disconnect; Selected::Operation(_) branch: `packet.wait_ready();` then `unsafe { Ok(packet.msg.get().replace(None).unwrap()) }` shows ordering requirements (wait_ready before reading) and single-consumption of the message (`replace(None).unwrap()`) relying on runtime/unsafe discipline

**Implementation:** Introduce an RAII `RegistrationGuard` returned by `receivers.register(...)` that automatically calls `unregister(oper)` in `Drop` unless explicitly `disarm()`ed on successful completion. Additionally, wrap the message slot in a typestate-like `Packet<Empty/Ready/Consumed>` or provide a safe `take_msg_after_ready(&self) -> T` that encodes the wait_ready-before-take ordering without `unsafe`/`unwrap()`.

---

### 45. OnceLock initialization protocol + reentrancy prohibition (Uninitialized / Initializing / Initialized)

**Location**: `/tmp/sync_test_crate/src/sync/once_lock.rs:1-69`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: OnceLock has an implicit multi-threaded initialization protocol: the cell starts Uninitialized, transitions to Initializing while one thread runs the initializer, and then becomes Initialized exactly once. Other threads may concurrently call get_or_init with different closures, but only one closure is executed and the rest observe the winning value. If the initializer panics, the cell must revert to Uninitialized (no value installed). Additionally, reentrant initialization (calling get_or_init/get_or_try_init again from inside the initializer f) is forbidden; the current implementation can deadlock, and the outcome is explicitly unspecified. None of these states (especially 'Initializing' and the reentrancy ban) are represented in the type system; they are enforced by runtime synchronization and documented preconditions.

**Evidence**:

```rust
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
```

**Entity:** OnceLock<T>

**States:** Uninitialized, Initializing (in-progress), Initialized

**Transitions:**
- Uninitialized -> Initializing via get_or_init()/get_or_try_init() starting evaluation of f
- Initializing -> Initialized via successful completion of f and installing T
- Initializing -> Uninitialized via panic in f (panic propagated; cell remains uninitialized)

**Evidence:** method `pub fn get_or_init<F>(&self, f: F) -> &T` returns &T even though initialization may occur inside the call (implicit Uninitialized/Initialized split); comment: "Gets the contents of the cell, initializing it to `f()` if the cell was uninitialized."; comment: "Many threads may call `get_or_init` concurrently ... but it is guaranteed that only one function will be executed." (implies an in-progress/Initialising state and a runtime arbitration protocol); comment under # Panics: "If `f()` panics ... the cell remains uninitialized." (Initializing -> Uninitialized rollback requirement); comment: "It is an error to reentrantly initialize the cell from `f`. The exact outcome is unspecified. Current implementation deadlocks..." (latent precondition/protocol not enforced by types); code: `match self.get_or_try_init(|| Ok::<T, !>(f())) { Ok(val) => val }` shows get_or_init delegates to a fallible initializer and assumes success variant only (relying on invariants about `!` and initialization behavior)

**Implementation:** Expose separate types representing phases, e.g. `OnceLock<Uninit, T>` and `OnceLock<Init, T>` (or an API that returns an `InitGuard` token while running the initializer). `get_or_init` would internally acquire an `Initializing` capability that cannot be re-entered (e.g., by making the initializer take an `&InitToken` that is !Clone/!Copy and not obtainable recursively), and only after successful completion produce/allow `&OnceLock<Init, T>`-like access. Panic rollback can be represented by only committing the transition when the guard is dropped successfully.

---

### 60. Channel disconnection & teardown protocol (Connected -> SendersDisconnected/ReceiversDisconnected -> Draining/Discarded)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-148`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: The channel encodes disconnection in-band in the `tail` atomic using `mark_bit`. Several operations require a specific temporal ordering: once disconnected, no new messages may be sent; when dropping the last receiver, the code must (1) disconnect receivers exactly once, (2) have observed destruction of all other receivers with Acquire-or-stronger ordering, and then (3) discard remaining messages by walking slots and dropping them. These requirements are enforced only by `unsafe` contracts, debug assertions, and the `mark_bit` runtime check; the type system does not prevent calling `disconnect_receivers`/`discard_all_messages` in the wrong state or more than once, nor does it encode the 'last receiver' capability.

**Evidence**:

```rust
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
    /// If a destructor panics, the remaining messages are leaked, matching the
    /// behavior of the unbounded channel.
    ///
    /// # Safety
    /// This method must only be called when dropping the last receiver. The
    /// destruction of all other receivers must have been observed with acquire
    /// ordering or stronger.
    unsafe fn discard_all_messages(&self, tail: usize) {
        debug_assert!(self.is_disconnected());

        // Only receivers modify `head`, so since we are the last one,
        // this value will not change and will not be observed (since
        // no new messages can be sent after disconnection).
        let mut head = self.head.load(Ordering::Relaxed);
        let tail = tail & !self.mark_bit;

        let backoff = Backoff::new();
        loop {
            // Deconstruct the head.
            let index = head & (self.mark_bit - 1);
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may drop the message.
            if head + 1 == stamp {
                head = if index + 1 < self.cap {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, mark: 0, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), mark: 0, index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                unsafe {
                    (*slot.msg.get()).assume_init_drop();
                }
            // If the tail equals the head, that means the channel is empty.
            } else if tail == head {
                return;
            // Otherwise, a sender is about to write into the slot, so we need
            // to wait for it to update the stamp.
            } else {
                backoff.spin_heavy();
            }
        }
    }

    /// Returns `true` if the channel is disconnected.
    pub(crate) fn is_disconnected(&self) -> bool {
        self.tail.load(Ordering::SeqCst) & self.mark_bit != 0
    }

    /// Returns `true` if the channel is empty.
    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);

        // Is the tail equal to the head?
        //
        // Note: If the head changes just before we load the tail, that means there was a moment
        // when the channel was not empty, so it is safe to just return `false`.
        (tail & !self.mark_bit) == head
    }

    /// Returns `true` if the channel is full.
    pub(crate) fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::SeqCst);
        let head = self.head.load(Ordering::SeqCst);

        // Is the head lagging one lap behind tail?
        //
        // Note: If the tail changes just before we load the head, that means there was a moment
        // when the channel was not full, so it is safe to just return `false`.
        head.wrapping_add(self.one_lap) == tail & !self.mark_bit
    }
}
```

**Entity:** Array channel internal struct (type owning head/tail/buffer; impl block shown)

**States:** Connected, SendersDisconnected, ReceiversDisconnected, TeardownInProgress (last receiver dropping; discard_all_messages allowed), TeardownComplete

**Transitions:**
- Connected -> SendersDisconnected via disconnect_senders() (sets mark_bit in tail; wakes receivers)
- Connected -> ReceiversDisconnected via unsafe disconnect_receivers() (sets mark_bit in tail; wakes senders)
- ReceiversDisconnected -> TeardownInProgress via unsafe disconnect_receivers() calling discard_all_messages(tail)
- TeardownInProgress -> TeardownComplete via discard_all_messages() returning when tail == head

**Evidence:** disconnect_senders(): `let tail = self.tail.fetch_or(self.mark_bit, Ordering::SeqCst);` and `if tail & self.mark_bit == 0 { ... }` uses mark_bit as a runtime state flag; disconnect_receivers(): marked `unsafe` with contract comment: "May only be called once upon dropping the last receiver" and requires observing other receivers' destruction with Acquire ordering or stronger; disconnect_receivers(): unconditionally calls `unsafe { self.discard_all_messages(tail) };` after attempting to set mark_bit, relying on the caller being 'last receiver'; discard_all_messages(): `debug_assert!(self.is_disconnected());` indicates a precondition/state requirement not enforced by types; discard_all_messages() safety comment repeats: "must only be called when dropping the last receiver" + acquire observation requirement; is_disconnected(): `self.tail.load(Ordering::SeqCst) & self.mark_bit != 0` defines the runtime-disconnected state

**Implementation:** Introduce a linear/unique capability representing "LastReceiver" (or "TeardownToken") that is created only by the receiver-drop path when refcount reaches 1 (or equivalent). Make `disconnect_receivers(self, token: LastReceiverToken) -> ...` and `discard_all_messages(&self, token: LastReceiverToken, tail: TailSnapshot)` only callable with that token. This removes the need for `unsafe` on these methods and prevents multiple calls/incorrect callers at compile time. The token can also carry proof of an Acquire fence (e.g., created only after an acquire load/CAS succeeds).

---

### 18. Mapped read-guard borrowing protocol (OriginalGuardAlive -> MappedGuardValid)

**Location**: `/tmp/sync_test_crate/src/sync/poison/rwlock.rs:1-139`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Mapping a read guard creates a new guard that points at a subcomponent (U) while still relying on the original guard's lock ownership and lifetime. This protocol is upheld via `ManuallyDrop` and raw pointer storage (`NonNull`), plus SAFETY comments asserting that the original `RwLockReadGuard::new` conditions continue to hold through `map`/`filter_map`. The type system does not fully encode that (a) the mapped guard's `data: NonNull<U>` must remain derived from the locked T, (b) the original guard must not be dropped normally (to avoid double-unlock), and (c) unlocking must happen exactly once when the mapped guard drops.

**Evidence**:

```rust

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized + fmt::Display> fmt::Display for MappedRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe { self.data.as_ref() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &*self.lock.data.get() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &mut *self.lock.data.get() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Deref for MappedRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Deref for MappedRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_ref() }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> DerefMut for MappedRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe { self.data.as_mut() }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.poison.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe {
            self.lock.inner.write_unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

#[unstable(feature = "mapped_lock_guards", issue = "117108")]
impl<T: ?Sized> Drop for MappedRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.poison_flag.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        unsafe {
            self.inner_lock.write_unlock();
        }
    }
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    #[unstable(feature = "mapped_lock_guards", issue = "117108")]
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `filter_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard { data, inner_lock: &orig.inner_lock }
    }
```

**Entity:** RwLockReadGuard<'a, T> / MappedRwLockReadGuard<'a, U>

**States:** OriginalGuardAlive (owns read lock + points at T), MappedGuardValid (owns read lock + points at sub-U), Unlocked/Invalid (lock released; data pointer must not be used)

**Transitions:**
- OriginalGuardAlive -> MappedGuardValid via RwLockReadGuard::map(orig, f)
- MappedGuardValid -> Unlocked/Invalid via Drop for MappedRwLockReadGuard (read_unlock)
- OriginalGuardAlive -> Unlocked/Invalid via Drop for RwLockReadGuard (read_unlock)

**Evidence:** RwLockReadGuard::map(orig: Self, ...) consumes `orig` and then uses `let orig = ManuallyDrop::new(orig);` to suppress `Drop` on the original guard (double-unlock would otherwise occur); `let data = NonNull::from(f(unsafe { orig.data.as_ref() }));` stores a raw non-null pointer derived from the guarded data; validity depends on the lock still being held; Drop for MappedRwLockReadGuard: `unsafe { self.inner_lock.read_unlock(); }` shows the mapped guard is responsible for unlocking; Drop for RwLockReadGuard: `unsafe { self.inner_lock.read_unlock(); }` indicates the original guard normally unlocks on drop; mapping must prevent that; SAFETY comments in Deref/Drop impls: "conditions of `RwLockReadGuard::new` were satisfied ... and have been upheld throughout `map` and/or `filter_map`" indicates an implicit, not-fully-typed protocol

**Implementation:** Encode a distinct "mapped" state at the type level to prevent accidental double-drop/unlock patterns and to tie the mapped pointer more directly to a guard lifetime. For example, represent mapping as `fn map(self) -> RwLockReadGuard<'a, U, MappedFrom<T>>` (or a separate guard type that carries an internal token proving it is the sole unlock owner) rather than using `ManuallyDrop` + raw pointer. Alternatively, use an internal capability/token that is moved from `RwLockReadGuard` into `MappedRwLockReadGuard` to make 'exactly one unlock owner' explicit without relying on `ManuallyDrop`.

---

### 31. Iterator depends on receiver liveness / disconnection to terminate

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/mod.rs:1-63`

**Confidence**: low

**Suggested Pattern**: capability

**Description**: The iteration protocol implicitly relies on the receiver and channel state: iteration yields values until the channel is disconnected, at which point it yields None. The example asserts eventual None, implying an implicit ordering: all senders must be dropped/disconnected for iteration to terminate. This termination condition is not expressed in the type system; Iter merely holds a borrow of Receiver, but does not encode whether the channel can still be kept alive by other Sender clones, so 'next() eventually returns None' is a runtime/liveness property.

**Evidence**:

```rust
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

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        unsafe {
            match &self.flavor {
                ReceiverFlavor::Array(chan) => chan.release(|c| c.disconnect_receivers()),
                ReceiverFlavor::List(chan) => chan.release(|c| c.disconnect_receivers()),
                ReceiverFlavor::Zero(chan) => chan.release(|c| c.disconnect()),
            }
        }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let flavor = match &self.flavor {
            ReceiverFlavor::Array(chan) => ReceiverFlavor::Array(chan.acquire()),
            ReceiverFlavor::List(chan) => ReceiverFlavor::List(chan.acquire()),
            ReceiverFlavor::Zero(chan) => ReceiverFlavor::Zero(chan.acquire()),
        };

        Receiver { flavor }
    }
}

#[unstable(feature = "mpmc_channel", issue = "126840")]
impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("Receiver { .. }")
    }
}

#[cfg(test)]
mod tests;
```

**Entity:** Iter<'a, T> (created by Receiver::iter)

**States:** Iterating (receiver still alive / may receive), Terminated (channel disconnected => next() returns None)

**Transitions:**
- Iterating -> Terminated when channel becomes disconnected (e.g., all senders dropped), observed via Iterator::next() returning None

**Evidence:** Receiver::iter(&self) -> Iter<'_, T> constructs Iter { rx: self }; Doc example: after sending 1,2,3, it asserts iter.next() == None (implies protocol: iteration ends on disconnect)

**Implementation:** If the API wanted to make termination guarantees explicit, separate 'finite stream' receivers from 'possibly-infinite' receivers via a capability/token returned when the last Sender is dropped (or a Close/Disconnect capability). Iter could then require that capability to promise eventual termination; otherwise iteration remains potentially non-terminating by type.

---

### 44. Condvar waiting protocol (must hold matching MutexGuard; releases+reacquires; must re-check predicate)

**Location**: `/tmp/sync_test_crate/src/sync/poison/condvar.rs:1-130`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Condvar::wait / wait_while implement a multi-step protocol that is only partially reflected in types. A caller must enter wait() while holding the mutex (via a MutexGuard), the wait operation temporarily unlocks the mutex and blocks (WaitingUnlocked), then re-locks before returning (ReacquiredMutexGuard). Additionally, due to spurious wakeups and missed notifications, callers must use a predicate loop (as shown and as implemented by wait_while) to re-check the condition after each wake. Finally, after reacquisition, the mutex may be poisoned; this is reported via LockResult, creating a distinct 'PoisonedOnReacquire' outcome. The type system enforces ownership of a MutexGuard parameter, but does not encode (1) that the guard must correspond to the same mutex associated with the condition variable's underlying wait, (2) the temporal unlock->block->relock sequence as an explicit state machine/capability, nor (3) the requirement that condition checks happen in a loop rather than a single wait.

**Evidence**:

```rust
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
```

**Entity:** Condvar (src::sync::poison::Condvar)

**States:** HoldingMutexGuard, WaitingUnlocked, ReacquiredMutexGuard, PoisonedOnReacquire

**Transitions:**
- HoldingMutexGuard -> WaitingUnlocked -> ReacquiredMutexGuard via Condvar::wait()
- ReacquiredMutexGuard -> PoisonedOnReacquire via Condvar::wait() returning Err(PoisonError)
- HoldingMutexGuard -> (WaitingUnlocked -> ReacquiredMutexGuard)* -> ReacquiredMutexGuard via Condvar::wait_while() loop

**Evidence:** fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> LockResult<MutexGuard<'a, T>>: taking/returning a MutexGuard encodes 'must hold lock' but not which lock/condvar pairing; wait(): `let lock = mutex::guard_lock(&guard); self.inner.wait(lock);` shows it extracts the underlying lock handle from the guard and passes it to the condvar inner wait (implicit coupling between this Condvar and that mutex); wait() doc comment: "This function will atomically unlock the mutex specified (represented by guard) and block... When this function call returns, the lock specified will have been re-acquired." (explicit temporal protocol); wait_while(): `while condition(&mut *guard) { guard = self.wait(guard)?; }` encodes the 'must re-check predicate after wakeup' rule as runtime control flow rather than a required typed API usage; wait(): `mutex::guard_poison(&guard).get()` and `if poisoned { Err(PoisonError::new(guard)) }` indicates an additional poisoned outcome/state after the wait+reacquire sequence

**Implementation:** Introduce a capability tying a Condvar to a specific Mutex at construction, e.g. `struct Condvar<'m, T> { inner: ..., _m: PhantomData<&'m Mutex<T>> }` or a token returned by `Mutex::new_condvar()`; then `wait(&self, guard: MutexGuard<'m, T>) -> ...` ensures at compile time that the guard belongs to the mutex associated with that condvar. Optionally, provide only predicate-based waiting APIs (or a distinct `Waiter` typestate) to bias callers toward the loop protocol.

---

### 8. MappedMutexGuard subfield-borrow protocol (cannot be used with Condvar / unlock+relock)

**Location**: `/tmp/sync_test_crate/src/sync/poison/mutex.rs:1-115`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: MappedMutexGuard represents a lock-held state like MutexGuard, but with an additional implicit protocol restriction: it must not participate in condition-variable waiting/unlocking patterns (unlock + allow other thread to mutate + relock) because it may only point to a subfield of the protected data and could become invalid/soundness-breaking if the larger object is modified while unlocked. This restriction is documented but not enforced at the type level; the type system does not distinguish 'full-object guard usable with Condvar' vs 'mapped/subfield guard not usable with Condvar' except by using a different guard type and relying on downstream APIs to accept only MutexGuard.

**Evidence**:

```rust
/// this manner. For instance, consider [`Rc`], a non-atomic reference counted smart pointer,
/// which is not `Send`. With `Rc`, we can have multiple copies pointing to the same heap
/// allocation with a non-atomic reference count. If we were to use `Mutex<Rc<_>>`, it would
/// only protect one instance of `Rc` from shared access, leaving other copies vulnerable
/// to potential data races.
///
/// Also note that it is not necessary for `T` to be `Sync` as `&T` is only made available
/// to one thread at a time if `T` is not `Sync`.
///
/// [`Rc`]: crate::rc::Rc
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`lock`]: Mutex::lock
/// [`try_lock`]: Mutex::try_lock
#[must_use = "if unused the Mutex will immediately unlock"]
#[must_not_suspend = "holding a MutexGuard across suspend \
                      points can cause deadlocks, delays, \
                      and cause Futures to not implement `Send`"]
#[stable(feature = "rust1", since = "1.0.0")]
#[clippy::has_significant_drop]
#[cfg_attr(not(test), rustc_diagnostic_item = "MutexGuard")]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    poison: poison::Guard,
}

/// A [`MutexGuard`] is not `Send` to maximize platform portablity.
///
/// On platforms that use POSIX threads (commonly referred to as pthreads) there is a requirement to
/// release mutex locks on the same thread they were acquired.
/// For this reason, [`MutexGuard`] must not implement `Send` to prevent it being dropped from
/// another thread.
#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}

/// `T` must be `Sync` for a [`MutexGuard<T>`] to be `Sync`
/// because it is possible to get a `&T` from `&MutexGuard` (via `Deref`).
#[stable(feature = "mutexguard", since = "1.19.0")]
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

/// An RAII mutex guard returned by `MutexGuard::map`, which can point to a
/// subfield of the protected data. When this structure is dropped (falls out
/// of scope), the lock will be unlocked.
///
/// The main difference between `MappedMutexGuard` and [`MutexGuard`] is that the
/// former cannot be used with [`Condvar`], since that
/// could introduce soundness issues if the locked object is modified by another
/// thread while the `Mutex` is unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`map`] and [`filter_map`] methods on
/// [`MutexGuard`].
///
/// [`map`]: MutexGuard::map
/// [`filter_map`]: MutexGuard::filter_map
/// [`Condvar`]: crate::sync::Condvar
#[must_use = "if unused the Mutex will immediately unlock"]
#[must_not_suspend = "holding a MappedMutexGuard across suspend \
                      points can cause deadlocks, delays, \
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
```

**Entity:** MappedMutexGuard<'a, T>

**States:** MappedLockHeld(pointing to subfield), Dropped(UnlockPerformed)

**Transitions:**
- MappedLockHeld(pointing to subfield) -> Dropped(UnlockPerformed) via Drop of MappedMutexGuard

**Evidence:** struct: `pub struct MappedMutexGuard<'a, T> { data: NonNull<T>, inner: &'a sys::Mutex, ... }` — holds a raw pointer to potentially interior data, indicating a different validity regime than `&'a mut T`; comment: "former cannot be used with Condvar, since that could introduce soundness issues if the locked object is modified by another thread while the Mutex is unlocked" — explicit protocol restriction; comment: "we use a pointer instead of `&'a mut T` to avoid `noalias` violations ... only until it drops" — indicates an implicit uniqueness/aliasing protocol not expressible directly with a normal borrow; impl: `impl<T: ?Sized> !Send for MappedMutexGuard<'_, T> {}` — same thread-affinity constraint as MutexGuard; attribute: `#[must_use = "if unused the Mutex will immediately unlock"]` — same lock lifecycle via drop

**Implementation:** Introduce a guard kind parameter to make the 'condvar-wait-capable' property explicit, e.g. `struct Guard<'a, T, K> { ... }` with marker types `Full` and `Mapped`. Implement `Condvar::wait` only for `Guard<_, _, Full>`. `Mutex::lock` returns `Guard<_, _, Full>`, while `Guard::map` returns `Guard<_, U, Mapped>`. This makes the protocol restriction a first-class type-level state rather than a documented caveat.

---

### 35. Operation hook pointer-identity protocol (valid pointer-derived ID with lifetime/thread uniqueness)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/select.rs:1-67`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Operation identifiers are created by converting the address of a mutable reference into a usize. This relies on a protocol that the referenced variable is (1) thread/operation-specific (not reused concurrently for another operation), and (2) kept alive for the entire duration of a blocking operation. It also relies on a numeric precondition that the pointer-as-usize is > 2 so it cannot collide with the sentinel encodings used by Selected::{Waiting, Aborted, Disconnected}. None of these constraints (lifetime of the referenced value, uniqueness, and non-collision with sentinel values) are enforced by the type system; they are enforced by comments and a runtime assert.

**Evidence**:

```rust
#[derive(Debug, Default)]
pub struct Token {
    pub(crate) array: super::array::ArrayToken,
    pub(crate) list: super::list::ListToken,
    #[allow(dead_code)]
    pub(crate) zero: super::zero::ZeroToken,
}

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

**Entity:** Operation

**States:** Unhooked (no Operation), Hooked (Operation created from &mut T address)

**Transitions:**
- Unhooked -> Hooked via Operation::hook(&mut T)

**Evidence:** Operation::hook<T>(r: &mut T) converts `r as *mut T as usize` into an ID; comment in Operation::hook: "reference should point to a variable that is specific to the thread and the operation, and is alive for the entire duration of a blocking operation"; Operation::hook: `assert!(val > 2);` with comment about avoiding numerical representations of `Selected::{Waiting, Aborted, Disconnected}`

**Implementation:** Make Operation carry a nonzero, non-sentinel representation and (optionally) a lifetime tying it to the hooked storage: e.g., `struct OperationId(NonZeroUsize);` plus a constructor that encodes/offsets the pointer (or uses `NonZeroUsize` + checked range) so `0..=2` are unrepresentable. If you want to enforce 'alive for duration', use `struct Operation<'a>(NonZeroUsize, PhantomData<&'a mut ()>)` returned from `hook(&'a mut T)` so the borrow must be held while blocking.

---

### 25. ArrayToken validity protocol (Uninitialized/null -> Bound to a Slot)

**Location**: `/tmp/sync_test_crate/src/sync/mpmc/array.rs:1-64`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: ArrayToken carries runtime state indicating whether it is usable for an operation: Default creates a token with slot = null and stamp = 0, implying an 'unbound' token that must be populated (by a selection/operation step) before it can be used to read/write a slot. This validity is represented only by a raw pointer sentinel (null) and is not enforced by the type system; misuse would require downstream null checks or unsafe dereference discipline.

**Evidence**:

```rust
use super::select::{Operation, Selected, Token};
use super::utils::{Backoff, CachePadded};
use super::waker::SyncWaker;
use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::ptr;
use crate::sync::atomic::{self, Atomic, AtomicUsize, Ordering};
use crate::time::Instant;

/// A slot in a channel.
struct Slot<T> {
    /// The current stamp.
    stamp: Atomic<usize>,

    /// The message in this slot. Either read out in `read` or dropped through
    /// `discard_all_messages`.
    msg: UnsafeCell<MaybeUninit<T>>,
}

/// The token type for the array flavor.
#[derive(Debug)]
pub(crate) struct ArrayToken {
    /// Slot to read from or write to.
    slot: *const u8,

    /// Stamp to store into the slot after reading or writing.
    stamp: usize,
}

impl Default for ArrayToken {
    #[inline]
    fn default() -> Self {
        ArrayToken { slot: ptr::null(), stamp: 0 }
    }
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
```

**Entity:** ArrayToken

**States:** Unbound (null slot), Bound (points to a Slot)

**Transitions:**
- Unbound -> Bound via selection/operation that sets ArrayToken.slot and ArrayToken.stamp (not shown in snippet)

**Evidence:** field `ArrayToken::slot: *const u8` is a raw pointer used as a handle to a slot; impl Default for ArrayToken: `slot: ptr::null()` constructs an explicit invalid/unbound sentinel state; field `ArrayToken::stamp: usize` is described as 'Stamp to store into the slot after reading or writing', implying it is meaningful only once bound to a slot

**Implementation:** Represent the token as `ArrayToken<S>` with `Unbound`/`Bound` states (PhantomData). `Default` yields `ArrayToken<Unbound>`. Only selection APIs can produce `ArrayToken<Bound>` containing a non-null `NonNull<u8>` (or better, `NonNull<Slot<T>>`). Slot read/write APIs accept only `ArrayToken<Bound>` so an uninitialized token cannot be used.

---

