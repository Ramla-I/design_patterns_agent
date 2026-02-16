# Design Patterns Agent

An AI-powered Rust tool that discovers invariants in Rust codebases and translates C2Rust output to idiomatic Rust using LLM technology.

## What It Does

### Invariant Analysis

Analyzes Rust projects to automatically identify and document:

- **State Machine Invariants**: Typestate patterns using `PhantomData` to enforce compile-time state transitions
- **Linear Type Invariants**: Ordering requirements and capability patterns that ensure operations happen in sequence
- **Ownership Invariants**: Lifetime and borrowing patterns that enforce memory safety

### C2Rust Translation

Translates mechanically-generated C2Rust code into idiomatic, safe Rust:

- Iterative LLM-powered translation with build and test feedback loops
- Clippy-based idiomaticity scoring
- Automatic retry on build/test failures (up to configurable max)
- Test vector validation via the `cando2` harness
- Organized output in timestamped run directories (`runs/<model>_<timestamp>/`)

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- OpenAI-compatible API key

### Installation

```bash
git clone https://github.com/yourusername/design_patterns_agent.git
cd design_patterns_agent
cargo build --release
```

### Invariant Analysis

```bash
export OPENAI_API_KEY=sk-...

# Analyze a Rust codebase
cargo run --release -- analyze /path/to/rust/project

# Save output to a file
cargo run --release -- analyze /path/to/rust/project --output report.md

# Get JSON output
cargo run --release -- analyze /path/to/rust/project --format json --output report.json
```

### C2Rust Translation

```bash
export OPENAI_API_KEY=sk-...

# Translate all programs in a test suite directory
cargo run --release -- translate Public-Tests/

# Translate a single program
cargo run --release -- translate Public-Tests/B01_organic/bin2hex_lib

# With options
cargo run --release -- translate Public-Tests/ \
  --model gpt-4 \
  --max-retries 3 \
  --max-lines 500 \
  --report extra_copy.md

# Build-only mode (skip test vectors)
cargo run --release -- translate Public-Tests/ --skip-tests

# Include design pattern analysis on successful translations
cargo run --release -- translate Public-Tests/ --analyze
```

Translation outputs are saved to `runs/<model>_<YYYYMMDD>_<HHMMSS>/` with the structure:

```
runs/
  gpt-4_20260216_153000/
    bin2hex_lib/
      translated_rust_llm/
        lib.rs
        Cargo.toml
        results.json
    bitwriter_add_lib/
      translated_rust_llm/
        ...
    report.md
```

## Configuration

Create a `config.toml` file:

```toml
[llm]
provider = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50

[detection]
focus = ["state_machine", "linear_types", "ownership"]
```

Use it with:
```bash
cargo run -- analyze /path/to/project --config config.toml
```

## How It Works

### Analysis Pipeline

1. **Parse**: Uses `syn` to parse Rust source files and extract type definitions, functions, and traits
2. **Navigate**: Performs top-down exploration of the module hierarchy
3. **Identify**: Finds "interesting" code patterns (PhantomData, Drop implementations, consuming methods)
4. **Analyze**: Uses LLM to analyze each pattern and identify invariants
5. **Report**: Generates a detailed report with code evidence and explanations

### Translation Pipeline

1. **Discover**: Walks the directory tree to find programs with `runner/` and `test_vectors/`
2. **Collect**: Gathers source from `translated_rust/` (crat) or `dst/` (raw c2rust)
3. **Translate**: Sends source to LLM with a system prompt targeting idiomatic, safe Rust
4. **Build**: Compiles the translation with `cargo build --release`
5. **Test**: Runs test vectors via the `cando2` harness (symlink-swaps the candidate library)
6. **Feedback**: On failure, sends build errors or test diffs back to the LLM for another attempt
7. **Score**: Runs clippy analysis and computes an idiomaticity score
8. **Report**: Generates a per-run report with results for every program

## Architecture

- **CLI** (`src/cli/`): Argument parsing with `clap`, subcommands for `analyze` and `translate`
- **Parser** (`src/parser/`): Rust AST extraction using `syn`
- **Navigator** (`src/navigation/`): Hierarchical codebase exploration
- **Detectors** (`src/detection/`): Specialized invariant detectors (state machine, linear types, ownership)
- **Translation** (`src/translation/`): LLM translator, test runner, clippy analyzer, feedback formatter, report generation
- **LLM Integration** (`src/llm/`): Async OpenAI-compatible API client behind a trait
- **Report Generator** (`src/report/`): Markdown and JSON output for analysis reports
- **Tools** (`tools/`): `cando`/`cando2` test harnesses, helper scripts

## Development

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test translation
cargo test parser
cargo test detection

# Check code
cargo check

# Build for release
cargo build --release
```

For detailed development documentation, see [CLAUDE.md](CLAUDE.md).

## Project Status

- ✅ Invariant discovery (state machine, linear types, ownership)
- ✅ Markdown and JSON reports
- ✅ OpenAI-compatible LLM integration
- ✅ C2Rust to idiomatic Rust translation pipeline
- ✅ Iterative build/test feedback loop
- ✅ Clippy idiomaticity scoring
- ✅ Timestamped run directories for output organization

## License

[MIT](LICENSE)

## Contributing

Contributions welcome! Please check existing issues or create a new one before submitting PRs.
