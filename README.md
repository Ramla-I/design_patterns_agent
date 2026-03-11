# Design Patterns Agent

An AI-powered Rust tool that analyzes Rust codebases to discover **latent invariants** — implicit protocols, temporal ordering requirements, and state dependencies that exist in code but aren't yet enforced by the type system.

## What It Does

The tool scans Rust modules for signals that suggest compile-time guarantees are missing — places where developers rely on runtime checks, comments, naming conventions, or error handling to enforce rules that *could* be compile-time guarantees.

**Invariant categories:**

| Category | What it catches | Example signal |
|----------|----------------|----------------|
| **Temporal Ordering** | "must call X before Y" | `// must call init() before use` |
| **Resource Lifecycle** | "must acquire then release" | open/close pairs, Drop guards |
| **State Machine** | boolean/enum flags encoding states | `if !self.is_open { return Err("closed") }` |
| **Precondition** | assertions revealing assumptions | `assert!(!closed)`, `expect("not initialized")` |
| **Protocol** | multi-step interaction sequences | function groups forming implicit workflows |

For each invariant found, the tool suggests a Rust design pattern to enforce it at compile time — **typestate**, **RAII**, **builder**, **newtype**, **session type**, or **capability passing** — with a confidence level and implementation sketch.

## Prerequisites

- Rust toolchain (1.70+)
- An LLM API key: **OpenAI** or **Anthropic**
- Docker (optional, for running the stdlib analysis)

## Installation

```bash
git clone <repo-url>
cd design_patterns_agent
cargo build --release
```

To build **without** the optional translation feature (invariant analysis only):

```bash
cargo build --release --no-default-features
```

## Quick Start

```bash
# Set your API key
export OPENAI_API_KEY=sk-...
# or
export ANTHROPIC_API_KEY=sk-ant-...

# Analyze a Rust project
cargo run --release -- analyze /path/to/rust/project

# Use Anthropic instead of OpenAI
cargo run --release -- analyze /path/to/rust/project --provider anthropic --model claude-sonnet-4-20250514

# Shorthand — a bare path defaults to the analyze subcommand
cargo run --release -- /path/to/rust/project
```

Results are always saved to a timestamped run directory:
```
runs/<model>_<YYYYMMDD>_<HHMMSS>_<mode>/report.md
```
The `<mode>` suffix is `_bfs` for exhaustive mode or `_ss` for semantic search. Previous runs are never overwritten.

## Running on the Rust Standard Library

The agent supports analyzing large multi-crate workspaces like the Rust standard library (`core`, `alloc`, `std`). A Docker setup and shell script handle everything.

### Targeted sync module test

```bash
# Analyze only std::sync (fast, ~200K tokens)
OPENAI_API_KEY=sk-... ./run_stdlib_sync_test.sh

# With a different provider
./run_stdlib_sync_test.sh --provider anthropic --model claude-sonnet-4-20250514
```

### Full stdlib run (Docker)

```bash
ANTHROPIC_API_KEY=sk-ant-... ./run_stdlib_test.sh
```

This builds the Docker image, clones the stdlib, and runs the analysis with sensible defaults (concurrency 5, 1M token budget, priority modules `sync,io,fs,net,cell,collections,thread,process`).

### Customizing the run

```bash
# Higher concurrency and budget
./run_stdlib_test.sh --concurrency 10 --token-budget 2000000

# Use a different model
./run_stdlib_test.sh --model claude-opus-4-6 --concurrency 3

# Use OpenAI
OPENAI_API_KEY=sk-... ./run_stdlib_test.sh --provider openai --model gpt-4o

# Resume a previous run (retries failed chunks, skips completed ones)
./run_stdlib_test.sh --resume /workspace/runs/<prev_run>/progress.jsonl
```

### Running directly with Docker

```bash
# Build the image
docker build -t dpa-stdlib .

# Run with defaults
docker run --rm \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -v ./runs:/workspace/runs \
  dpa-stdlib

# Override all flags
docker run --rm \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -v ./runs:/workspace/runs \
  dpa-stdlib \
  --multi-crate \
  --concurrency 10 \
  --priority-modules sync,io,fs \
  --provider anthropic \
  --model claude-opus-4-6 \
  --token-budget 2000000
```

### What the Docker image contains

- Multi-stage build: Rust builder compiles the release binary, runtime image is minimal Debian
- The Rust stdlib source is baked in via sparse git checkout (~250 MB, `.git` removed)
- Output is written to `/workspace/runs/` — mount a host volume to persist it

### Output files

Each run creates a timestamped directory under `runs/`:

```
runs/claude-sonnet-4-20250514_20260309_143201_bfs/
  progress.jsonl      # One line per analyzed chunk (status, tokens, timestamp)
  invariants.jsonl    # One serialized Invariant per line (incremental, survives crashes)
  report.md           # Final formatted report (generated at end)
  token_usage.json    # Token breakdown (input/cached/output/total for detector and validator)
```

## Usage

### Invariant Analysis

```bash
# Exhaustive mode (default) — parses all modules, 100% coverage
cargo run --release -- analyze /path/to/rust/project

# Multi-crate workspace (e.g., rust stdlib library/)
cargo run --release -- analyze /path/to/workspace --multi-crate

# Parallel LLM calls
cargo run --release -- analyze /path/to/project --concurrency 5

# With token budget limit
cargo run --release -- analyze /path/to/project --token-budget 500000

# Prioritize specific modules
cargo run --release -- analyze /path/to/project --multi-crate --priority-modules sync,io,fs,net

# Resume from a previous run
cargo run --release -- analyze /path/to/project --resume runs/<prev_run>/progress.jsonl

# Validate invariants with a second LLM pass (filters hallucinations, doubles cost)
cargo run --release -- analyze /path/to/project --validate

# Semantic search mode — uses octocode for targeted search, faster for large codebases
cargo run --release -- analyze /path/to/rust/project --search-mode semantic

# JSON output
cargo run --release -- analyze /path/to/rust/project --format json

# Use a TOML config file for all settings
cargo run --release -- analyze /path/to/rust/project --config config.toml
```

### Search Modes

| Mode | Flag | How it works | Trade-offs |
|------|------|-------------|------------|
| **Exhaustive** (default) | `--search-mode exhaustive` | Parses every `.rs` file with `syn`, builds module-level analysis chunks, sends all to LLM | 100% coverage. More expensive for large codebases. |
| **Semantic** | `--search-mode semantic` | Runs 12 targeted semantic queries via [octocode](https://github.com/Muvon/octocode) for invariant signals. Only sends high-relevance code to LLM. | Faster and cheaper on large codebases. May miss signals not covered by the predefined queries. |

The two modes are **complementary** — semantic search finds more unique high-confidence entities on larger modules, while BFS provides complete coverage on smaller ones. See [reports/bfs_vs_semantic_search_comparison.md](reports/bfs_vs_semantic_search_comparison.md) for a detailed comparison across `std::sync`, `std::io`, and `std::net`.

**Semantic mode prerequisites:**

```bash
# Install octocode (one-time)
curl -fsSL https://raw.githubusercontent.com/Muvon/octocode/master/install.sh | sh

# Configure embedding model (one-time)
octocode config \
  --code-embedding-model "openai:text-embedding-3-small" \
  --text-embedding-model "openai:text-embedding-3-small"
```

The tool automatically initializes a git repo (if needed) and runs `octocode index` before searching, so no manual indexing step is required.

### Translation (requires `translation` feature)

The `translate` subcommand wraps the [`llm_translation`](../llm_translation) crate for converting C2Rust or C code to idiomatic Rust. See that crate's README for full documentation.

```bash
# Translate programs from a test suite
cargo run --release -- translate /path/to/Public-Tests/B01_organic/bin2hex_lib

# With options
cargo run --release -- translate /path/to/Public-Tests/ \
  --max-retries 3 \
  --from-c \
  --skip-tests
```

### CLI Reference

**Global options** (apply to all subcommands):

| Flag | Default | Description |
|------|---------|-------------|
| `--api-key <KEY>` | `$OPENAI_API_KEY` / `$ANTHROPIC_API_KEY` | API key |
| `--provider <NAME>` | `openai` | LLM provider: `openai` or `anthropic` |
| `--model <MODEL>` | `gpt-5.2` | LLM model name |
| `--format <FMT>` | `markdown` | Output format: `markdown` or `json` |
| `--output <PATH>` | — | Write an additional copy of the report |
| `--config <PATH>` | — | TOML config file |

**`analyze` options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--max-depth <N>` | `10` | Maximum module exploration depth |
| `--search-mode <MODE>` | `exhaustive` | `exhaustive` or `semantic` |
| `--similarity-threshold <F>` | `0.1` | Min similarity for semantic search (0.0-1.0) |
| `--multi-crate` | `false` | Discover and analyze multiple crates under one directory |
| `--concurrency <N>` | `1` | Number of concurrent LLM calls |
| `--token-budget <N>` | `0` (unlimited) | Stop after using this many tokens |
| `--resume <PATH>` | — | Path to `progress.jsonl` from a previous run to resume |
| `--priority-modules <LIST>` | — | Comma-separated module prefixes to analyze first |
| `--max-retries <N>` | `5` | Max retries per LLM call on transient errors |
| `--retry-base-delay <SECS>` | `2` | Base delay for exponential backoff |
| `--validate` | `false` | Run a validation pass on discovered invariants (doubles LLM cost) |

**`translate` options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--max-retries <N>` | `5` | Max LLM retry attempts per program |
| `--max-lines <N>` | `2000` | Skip source files exceeding this line count |
| `--skip-tests` | `false` | Only verify build succeeds |
| `--from-c` | `false` | Translate from C source (`test_case/`) |
| `--analyze` | `false` | Run invariant analysis on successful translations |
| `--report <PATH>` | — | Write an extra copy of the report |

## Robustness and Fault Tolerance

The agent is designed for long-running analysis of large codebases:

**Retry with exponential backoff** — Rate limits (429), server errors (500/502/503), Anthropic overload (529), timeouts, and connection errors are retried automatically with exponential backoff and jitter. Server-provided `Retry-After` headers are respected. Consecutive rate limits across all concurrent tasks trigger progressively longer delays.

**Incremental progress** — Every completed chunk is recorded to `progress.jsonl` and every discovered invariant to `invariants.jsonl` as they happen. If the process crashes, no work is lost.

**Resume** — Pass `--resume <path>/progress.jsonl` to skip already-completed chunks. Failed chunks (e.g., rate-limit exhaustion) are automatically retried on resume.

**Token budget** — Set `--token-budget` to cap spending. Once exceeded, remaining chunks are skipped and a partial report is generated.

**Graceful shutdown** — Ctrl+C lets in-flight LLM calls finish, then generates a partial report from results collected so far.

**Tolerant parsing** — Files that fail `syn` parsing go through a 5-pass recovery pipeline: strip `#![feature(...)]`/`#![cfg_attr(...)]`, strip `cfg_select! { ... }` blocks, replace `unsafe extern` → `extern` (edition 2024), strip remaining inner attributes, and finally item-level fallback parsing (parse each top-level item individually, skip failures). Unparseable files are logged and listed in the report's "Skipped Files" section.

## Configuration

All CLI flags can alternatively be set in a `config.toml`:

```toml
[llm]
provider = "openai"       # "openai" or "anthropic"
api_key = "sk-..."        # Or use OPENAI_API_KEY / ANTHROPIC_API_KEY env var
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000    # Max tokens per analysis chunk sent to LLM

[detection]
focus = ["temporal_ordering", "resource_lifecycle", "state_machine", "precondition", "protocol"]
min_confidence = "medium"       # Filter: "high", "medium", or "low"

[search]
mode = "exhaustive"             # "exhaustive" or "semantic"
similarity_threshold = 0.1      # Semantic mode: minimum cosine similarity (0.0-1.0)
max_results_per_query = 20      # Semantic mode: max results per search query
context_lines = 30              # Lines of context above/below each match

[execution]
concurrency = 5
token_budget = 1000000
multi_crate = true
max_retries = 5
retry_base_delay = 2
priority_modules = ["sync", "io", "fs", "net", "cell"]
validate = false                    # Second-pass LLM validation (doubles cost)
```

## How It Works

### Pipeline

```
Rust codebase
  |
  |-- [Multi-crate] Discover crates in subdirectories -> merge module graphs
  |-- [Exhaustive]  Parse all .rs files with syn -> module graph -> BFS traversal
  |-- [Semantic]    Run 12 semantic queries via octocode -> group by file -> merge regions
  |
  v
Analysis Chunks (raw source preserving comments + structured AST data)
  |
  |- Priority scoring: PhantomData (+50), Drop impl (+40), consuming self (+30),
  |  unsafe (+20), safety keywords (+10), --priority-modules match (+1000)
  |- Always clustered by type affinity (struct + its impls + related functions)
  |- Span-based source extraction using real line numbers from proc-macro2
  |- Token estimation (4 chars/token); function bodies stripped if still over budget
  |- Sibling summaries give LLM awareness of other parts when module is split
  |
  v
LLM Invariant Detection (one prompt per chunk, temperature 0.3)
  |
  |- Retry client: exponential backoff for 429/5xx/timeouts (up to --max-retries)
  |- Token tracking: accumulates usage across all calls
  |- System prompt with 8 ranked signal categories + 2 worked examples + exclusion rules
  |- Requests ONE entry per entity covering all states and transitions
  |- JSON response parsing with text-based fallback
  |- Priority chunks bypass token budget enforcement
  |
  v
Quality Pipeline
  |- Compile-time noise filter (cfg_select, conditional compilation, enum exhaustiveness)
  |- Enum variant noise filter (error types describing variants, single-variant listings)
  |- Citation verification: evidence with 'line N:' checked against source (±5 line window
  |  + content fallback); <30% verified → confidence downgrade
  |- Post-emission deduplication:
  |    Phase 1: exact key (entity+title+type), keeps highest confidence
  |    Phase 2: fuzzy dedup (same entity+type, >60% title word overlap)
  |- Optional validation pass (--validate): second LLM call reviews each invariant
  |
  v
Incremental Output
  |- progress.jsonl  (one entry per chunk: completed/failed, tokens, timestamp)
  |- invariants.jsonl (one Invariant per line, written immediately on discovery)
  |
  v
Report (Markdown or JSON, grouped by invariant type, with skipped-files section)
  -> runs/<model>_<YYYYMMDD>_<HHMMSS>_<bfs|ss>/report.{md,json}
```

### What the LLM Looks For (ranked by signal reliability)

1. **Runtime state checks** — `if !self.initialized`, `assert!(connected)`, `.is_null()`, `unwrap()` with precondition messages
2. **Boolean/enum state fields** — `is_open: bool`, `state: AtomicU8`, `inner: Option<T>` tracking state at runtime
3. **Comment-based protocols** — `// must call X before Y`, `// SAFETY: assumes ...`, `// Invariant:`
4. **Error messages revealing invariants** — `Err("not initialized")`, `panic!("connection closed")`
5. **Self-consuming methods** — `fn close(self)` that destroy state (transitions that should produce a new type)
6. **Method availability patterns** — methods checking state before acting (`read()` checks `is_open`)
7. **Option/UnsafeCell patterns** — `inner: UnsafeCell<Option<T>>`, "written at most once" invariants
8. **Atomic state machines** — `state: AtomicU8` with named constants (`INCOMPLETE=0, RUNNING=1, COMPLETE=2`)

### Suggested Patterns

For each invariant, the tool recommends one of:

| Pattern | When to use |
|---------|-------------|
| **Typestate** | Distinct states with different valid operations (`PhantomData<State>`) |
| **Builder** | Complex initialization sequences with required steps |
| **RAII** | Resources that must be released (Drop-based cleanup) |
| **Newtype** | Validity invariants on values (`NonEmptyVec`, `ValidatedEmail`) |
| **Session type** | Multi-step protocol enforcement via type-level state machines |
| **Capability** | Authorization via token/capability passing |

## Architecture

```
src/
  main.rs                    # Entry point (tokio async)
  lib.rs                     # Module exports

  cli/
    mod.rs                   # CLI parsing (clap), subcommand dispatch
    config.rs                # TOML config, SearchMode, ExecutionConfig, InvariantType

  parser/
    ast.rs                   # syn-based AST extraction: structs, enums, functions (with bodies),
                             #   traits, impl blocks, PhantomData detection, attributes, unsafe flag.
                             #   Uses proc-macro2 span-locations for real line numbers.
    module_graph.rs          # Crate module hierarchy via walkdir + mod declaration parsing.
                             #   Supports single crate (from_crate_root) and multi-crate workspace
                             #   (from_workspace) with crate-prefixed module names. Tracks parse failures.
                             #   Multi-pass tolerant parsing (5 fallback strategies for nightly syntax).

  navigation/
    explorer.rs              # BFS module traversal, reads raw source per module
    context.rs               # AnalysisChunk + ItemCluster: always clusters by type affinity,
                             #   span-based source extraction, token estimation, body stripping,
                             #   sibling summaries
    priority.rs              # Chunk priority scoring for --priority-modules and heuristic signals

  search/
    octocode.rs              # Async MCP client (JSON-RPC 2.0 over subprocess stdin/stdout),
                             #   auto-indexes codebase on startup
    queries.rs               # 12 predefined semantic queries targeting invariant signals
    mod.rs                   # Search orchestrator: run queries, deduplicate, merge regions, build chunks

  detection/
    invariant_inference.rs   # System prompt (8 signal categories, 2 worked examples, exclusion rules),
                             #   LLM request (one entry per entity), JSON/text response parsing,
                             #   compile-time noise filter, enum variant noise filter,
                             #   citation verification with confidence calibration
    evidence.rs              # Formats chunks for LLM: prefers raw source, falls back to AST reconstruction
    validation.rs            # Optional second-pass LLM verification of invariants (--validate flag)

  agent/
    mod.rs                   # Top-level orchestrator: auto git-init -> retry client -> token tracker ->
                             #   priority partitioning -> semaphore-based parallel execution ->
                             #   deduplication -> optional validation -> progress tracking ->
                             #   graceful shutdown -> report (run dir suffixed with _bfs or _ss)
    progress.rs              # JSONL-based progress and invariant checkpointing. Resume support
                             #   (completed chunks skipped, failed chunks retried).

  llm/
    types.rs                 # LlmClient trait, LlmRequest, LlmResponse (with cached/reasoning tokens)
    openai.rs                # OpenAI implementation (async-openai)
    anthropic.rs             # Anthropic implementation (reqwest)
    retry.rs                 # RetryClient: exponential backoff with jitter for rate limits and
                             #   transient errors. Respects Retry-After. Adaptive pressure from
                             #   consecutive rate-limit hits across concurrent tasks.
    tracking.rs              # TokenTrackingClient: transparent token accumulation via shared AtomicU64

  report/
    mod.rs                   # Report, Summary, Invariant (with entity field), Evidence, InvariantType,
                             #   Confidence, two-phase deduplication (exact key + fuzzy title overlap),
                             #   parse_failures list
    markdown.rs              # Markdown report generation (grouped by invariant type, skipped files section)
    json.rs                  # JSON report generation (serde)

runs/                        # Timestamped output directories (never overwritten)
reports/                     # Analysis and comparison reports
Dockerfile                   # Multi-stage build for stdlib analysis
run_stdlib_test.sh           # One-command script to build and run on the full Rust stdlib (Docker)
run_stdlib_sync_test.sh      # Targeted test on std::sync only (no Docker, local build)
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `translation` | on | Enables `translate` subcommand via `llm_translation` crate |

```toml
# In Cargo.toml
llm_translation = { path = "../llm_translation", optional = true }
```

## Development

```bash
# Run all tests (102 unit + 1 integration)
cargo test --no-default-features

# Run tests for a specific module
cargo test parser
cargo test navigation
cargo test detection
cargo test -- retry
cargo test -- progress
cargo test -- tracking
cargo test -- priority

# Build without translation feature
cargo build --no-default-features

# Check code
cargo check --no-default-features
```

For detailed development documentation, see [CLAUDE.md](CLAUDE.md).

## License

[MIT](LICENSE)

## Contributing

Contributions welcome! Please check existing issues or create a new one before submitting PRs.
