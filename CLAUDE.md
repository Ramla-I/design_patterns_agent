# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Design Patterns Agent** is a Rust CLI tool that analyzes Rust codebases to discover **latent invariants** — implicit protocols, temporal ordering requirements, and state dependencies that exist in code but aren't yet enforced by the type system. It uses LLM-powered analysis to identify these invariants and suggest compile-time enforcement patterns (typestate, RAII, builder, newtype, session type, capability passing).

## Development Commands

### Build and Test
```bash
# Build without translation feature (invariant analysis only)
cargo build --no-default-features

# Build with all features
cargo build

# Run all tests (95 unit + 1 integration)
cargo test --no-default-features

# Run tests for a specific module
cargo test parser
cargo test navigation
cargo test detection
cargo test report
cargo test -- retry
cargo test -- progress
cargo test -- priority

# Check code
cargo check --no-default-features
```

### Running the Tool
```bash
# Basic usage
cargo run -- analyze /path/to/rust/codebase

# Multi-crate workspace (e.g., rust stdlib)
cargo run -- analyze /path/to/workspace --multi-crate --concurrency 5

# With token budget and priority modules
cargo run -- analyze /path/to/project --token-budget 500000 --priority-modules sync,io,fs

# Validation pass (second LLM call to verify invariants, doubles cost)
cargo run -- analyze /path/to/project --validate

# Use Anthropic
cargo run -- analyze /path/to/project --provider anthropic --model claude-sonnet-4-20250514

# Resume a previous run
cargo run -- analyze /path/to/project --resume runs/<prev_run>/progress.jsonl
```

### Environment Variables
```bash
export OPENAI_API_KEY=sk-...
# or
export ANTHROPIC_API_KEY=sk-ant-...
```

## Architecture

### 1. CLI (`src/cli/`)
- **Entry Point**: `src/main.rs` and `src/cli/mod.rs`
- **Configuration**: `src/cli/config.rs` handles CLI args, TOML config, `ExecutionConfig` (includes `--validate` flag)
- Subcommands: `analyze` (invariant discovery), `translate` (C2Rust translation, feature-gated)

### 2. Parser (`src/parser/`)
- **Core Module**: `src/parser/mod.rs` — tolerant multi-pass parsing:
  - Pass 1: Strip `#![feature(...)]` and `#![cfg_attr(...)]`
  - Pass 2: Strip `cfg_select! { ... }` blocks (brace-depth tracking)
  - Pass 3: Replace `unsafe extern` → `extern` (edition 2024)
  - Pass 4: Strip remaining inner attributes (except `#![doc]`, `#![allow]`)
  - Pass 5: Item-level fallback (parse each top-level item individually)
- **AST Extraction**: `src/parser/ast.rs` — uses `syn` with `proc-macro2` span-locations for real line numbers
- **Module Graph**: `src/parser/module_graph.rs` — crate module hierarchy, multi-crate workspaces

### 3. Navigator (`src/navigation/`)
- **Explorer**: `src/navigation/explorer.rs` — BFS module traversal
- **Context**: `src/navigation/context.rs` — always clusters by type affinity (struct + its impls + related functions), span-based source extraction, sibling summaries
- **Priority**: `src/navigation/priority.rs` — scoring: `--priority-modules` match (+1000), PhantomData (+50), Drop impl (+40), consuming self (+30), unsafe (+20), safety keywords (+10)

### 4. Detection (`src/detection/`)
- **Coordinator**: `src/detection/mod.rs` — routes chunks through the inference detector
- **Invariant Inference**: `src/detection/invariant_inference.rs` — single LLM prompt with 8 ranked signal categories + 2 worked examples, JSON/text response parsing, compile-time noise filter, citation verification with confidence calibration
- **Evidence**: `src/detection/evidence.rs` — formats chunks for LLM (prefers raw source, falls back to AST reconstruction)
- **Validation**: `src/detection/validation.rs` — optional second-pass LLM verification (`--validate` flag)

### 5. LLM Integration (`src/llm/`)
- **Client Trait**: `src/llm/types.rs` — `LlmClient` trait, `LlmRequest`, `LlmResponse`
- **OpenAI**: `src/llm/openai.rs` (async-openai)
- **Anthropic**: `src/llm/anthropic.rs` (reqwest)
- **Retry**: `src/llm/retry.rs` — exponential backoff with jitter, respects Retry-After
- **Tracking**: `src/llm/tracking.rs` — transparent token accumulation via shared AtomicU64

### 6. Agent Loop (`src/agent/`)
- **Orchestrator**: `src/agent/mod.rs` — retry client → token tracker → priority scoring → semaphore-based parallel execution → deduplication → optional validation → report
- **Progress**: `src/agent/progress.rs` — JSONL checkpointing, resume support
- Priority chunks bypass token budget enforcement
- Coverage stats tracked (priority vs. other modules)

### 7. Report Generation (`src/report/`)
- **Report Types**: `src/report/mod.rs` — Report, Invariant (with `entity` field), Evidence, deduplication (by entity+title+type, keeps highest confidence)
- **Markdown**: `src/report/markdown.rs`
- **JSON**: `src/report/json.rs`

## Key Data Flow

```
Rust codebase
  → Multi-pass tolerant parsing (syn + fallbacks for nightly syntax)
  → Module graph (single-crate or multi-crate)
  → Type-affinity clustering (always clusters, even small modules)
  → Priority scoring + budget partitioning
  → LLM invariant inference (per-chunk, concurrent)
  → Compile-time noise filtering
  → Citation verification + confidence calibration
  → Post-emission deduplication (entity+title+type key)
  → Optional validation pass (second LLM call)
  → Report (markdown/JSON)
  → runs/<model>_<YYYYMMDD>_<HHMMSS>/
```

## Quality Pipeline

Invariants pass through multiple quality gates:
1. **Prompt exclusions**: LLM instructed to skip cfg/compile-time/doc-policy invariants
2. **Compile-time noise filter**: Post-parse filter on keywords like `cfg_select`, `conditional compilation`, `enum exhaustiveness`
3. **Citation verification**: Evidence items with `line N:` citations checked against actual snippet; low citation rate downgrades confidence
4. **Deduplication**: Same entity+title+type → keep highest confidence, longest evidence
5. **Validation pass** (opt-in `--validate`): Second LLM call reviews each invariant for hallucination, misclassification

## Testing Strategy

Tests are organized by module (95 unit tests + 1 integration test):
- **Parser**: tolerant parsing (cfg_select, unsafe extern, item-level fallback), line numbers from spans
- **Navigation**: type-affinity clustering, priority scoring, always-clustering behavior
- **Detection**: JSON/text parsing, compile-time noise filter, citation verification, confidence calibration, validation response parsing
- **Report**: deduplication (same entity, different entities, confidence tiebreak)
- **LLM**: retry logic, client creation
- **Integration**: end-to-end parse of a typestate example project

## Configuration File Format

```toml
[llm]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000

[detection]
focus = ["temporal_ordering", "resource_lifecycle", "state_machine", "precondition", "protocol"]
min_confidence = "medium"

[execution]
concurrency = 5
token_budget = 1000000
multi_crate = true
priority_modules = ["sync", "io", "fs", "net", "cell"]
validate = false
```

## Dependencies

### Key External Crates
- **syn** + **proc-macro2** (span-locations): Rust parser with line number tracking
- **quote**: Code generation (formatting types)
- **async-openai**: OpenAI API client
- **clap**: CLI argument parsing
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **walkdir**: Recursive directory traversal
- **anyhow/thiserror**: Error handling
- **chrono**: Timestamped run directories
- **reqwest**: HTTP client (Anthropic API)
