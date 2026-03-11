# BFS vs Semantic Search: Invariant Discovery Comparison

**Date:** 2026-03-11
**Model:** gpt-5.2
**Modules tested:** `std::sync`, `std::io`, `std::net`
**Validation:** disabled (neither mode used `--validate`)
**Token budget:** 200K (priority chunks bypass budget enforcement)

## Run Directories

| Module | BFS Run | Semantic Search Run |
|--------|---------|---------------------|
| sync | `runs/gpt-5_2_20260311_110347_bfs_sync/` | `runs/gpt-5_2_20260311_121135_ss_sync/` |
| io | `runs/gpt-5_2_20260311_133835_bfs_io/` | `runs/gpt-5_2_20260311_133856_ss_io/` |
| net | `runs/gpt-5_2_20260311_133853_bfs_net/` | `runs/gpt-5_2_20260311_134013_ss_net/` |

Each run directory contains:
- `invariants.jsonl` — one JSON object per discovered invariant
- `progress.jsonl` — per-chunk progress log
- `token_usage.json` — token breakdown (input/cached/output/total)
- `report.md` — human-readable markdown report

## Summary Table

| Run | Total | High | Medium | Low | H+M% | Tokens |
|-----|------:|-----:|-------:|----:|-----:|-------:|
| sync BFS | 80 | 22 | 50 | 8 | 90% | 288K |
| sync SS | 64 | 27 | 33 | 4 | 94% | 175K |
| io BFS | 51 | 14 | 36 | 1 | 98% | 217K |
| io SS | 63 | 19 | 40 | 4 | 94% | 196K |
| net BFS | 13 | 3 | 9 | 1 | 92% | 44K |
| net SS | 24 | 5 | 17 | 2 | 92% | 81K |

## High-Confidence Entity Overlap

### sync (deeply nested: mpmc/poison/waker/mutex/condvar/...)

- **BFS:** 22 high-confidence invariants across 16 unique entities
- **SS:** 27 high-confidence invariants across 22 unique entities
- **Shared (11):** Channel, Condvar, Context, Flag, LazyLock, Operation, Receiver, Selected, Sender, Token, Waker
- **BFS-only (5):** Block, MutexGuard, Packet, RwLock, TryIter
- **SS-only (11):** Array channel internal struct, ListToken, MappedMutexGuard, Mutex, OnceLock, Receiver/operation wait path, RwLockReadGuard, RwLockWriteGuard, Slot, SyncWaker, Token/ZeroToken

### io (moderate nesting: buffered/cursor/error/pipe/stdio/...)

- **BFS:** 14 high-confidence invariants across 11 unique entities
- **SS:** 19 high-confidence invariants across 14 unique entities
- **Shared (6):** Buffer, BufWriter, Chain, Cursor, LineWriter, Repr
- **BFS-only (5):** append_to_string, BorrowedBuf, Error, IoSlice::advance_slices, reserve_and_pad/vec_write_all_unchecked
- **SS-only (8):** BufReader, copy_specializes_bufreader, DataAndErrorReader, Global Stdout/Stdout, ProgrammableSink, std::io::Error, Tagged pointer payload types, Take

### net (flat: tcp/udp/ip_addr/socket_addr)

- **BFS:** 3 high-confidence invariants across 3 unique entities
- **SS:** 5 high-confidence invariants across 4 unique entities
- **Shared (2):** TcpStream, UdpSocket
- **BFS-only (1):** TcpListener
- **SS-only (2):** Duration, TcpStream::connect_timeout

## Analysis

### Semantic search finds more high-confidence invariants

Across all 3 modules, SS discovered more high-confidence invariants (27 vs 22, 19 vs 14, 5 vs 3). It particularly excels at finding invariants in guard types (MappedMutexGuard, RwLockReadGuard, RwLockWriteGuard), initialization protocols (OnceLock), and internal implementation types (Slot, SyncWaker, ListToken) that BFS tends to miss.

### Token efficiency depends on module size

- **sync (47 files):** SS uses 39% fewer tokens (175K vs 288K) — clear efficiency win
- **io (26 files):** SS uses 10% fewer tokens (196K vs 217K) — slight win
- **net (10 files):** SS uses 85% more tokens (81K vs 44K) — BFS is cheaper for small, flat modules

The crossover point appears to be around 15-20 files. Below that, BFS covers everything cheaply. Above that, semantic search's targeted queries avoid processing irrelevant chunks.

### The modes are complementary, not competing

Entity overlap is only ~50-60% across modules. Each mode consistently finds unique high-value invariants:
- **BFS strengths:** Complete coverage of the module tree; finds invariants in every type, including small helper types (Block, Packet, TryIter) that may not match semantic queries
- **SS strengths:** Finds invariants in types that match known patterns (guard protocols, initialization state machines, safety-critical code); discovers cross-cutting invariants (e.g., "copy_specializes_bufreader" in io) that span multiple types

### Quality is comparable

Both modes produce >90% high+medium confidence invariants across all modules, with SS slightly higher on sync (94% vs 90%) and BFS slightly higher on io (98% vs 94%). Neither mode has a clear quality advantage.

## Recommendations

1. **Hybrid mode** would capture the union of both approaches — semantic search first for targeted high-signal discovery, then BFS gap-fill for completeness
2. **Auto-select by module size:** Use BFS for small modules (<15 files), semantic for larger ones
3. **The two `"docs"` mode queries** ("must call before initialize first" and "SAFETY assumes precondition invariant caller must ensure") returned 0 matches in all 3 test modules. These may be more useful on codebases with extensive doc comments and safety annotations (e.g., `unsafe`-heavy code in `std::ptr`, `std::alloc`)
