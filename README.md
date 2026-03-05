# Design Patterns Agent

An AI-powered Rust tool that analyzes Rust codebases to automatically discover and document invariants using LLM technology.

## What It Does

Analyzes Rust projects to identify:

- **State Machine Invariants**: Typestate patterns using `PhantomData` to enforce compile-time state transitions
- **Linear Type Invariants**: Ordering requirements and capability patterns that ensure operations happen in sequence
- **Ownership Invariants**: Lifetime and borrowing patterns that enforce memory safety

Optionally includes a **translation subcommand** (via the [`llm_translation`](../llm_translation) crate) for converting C2Rust or C code to idiomatic Rust. This feature is enabled by default and can be excluded at build time.

## Prerequisites

- Rust toolchain (1.70+)
- OpenAI-compatible API key

## Installation

```bash
git clone <repo-url>
cd design_patterns_agent
cargo build --release
```

To build **without** the translation feature (analysis only):

```bash
cargo build --release --no-default-features
```

## Usage

### Invariant Analysis

```bash
export OPENAI_API_KEY=sk-...

# Analyze a Rust codebase
cargo run --release -- analyze /path/to/rust/project

# Save output to a file
cargo run --release -- analyze /path/to/rust/project --output report.md

# JSON output
cargo run --release -- analyze /path/to/rust/project --format json --output report.json

# With a config file
cargo run --release -- analyze /path/to/rust/project --config config.toml

# Shorthand (path without subcommand defaults to analyze)
cargo run --release -- /path/to/rust/project
```

### Translation (requires `translation` feature)

The `translate` subcommand is a thin wrapper around the [`llm_translation`](../llm_translation) crate. For full translation documentation, see that repo's README.

```bash
# Translate programs from a test suite
cargo run --release -- translate /path/to/Public-Tests/B01_organic/bin2hex_lib

# With options
cargo run --release -- translate /path/to/Public-Tests/ \
  --max-retries 3 \
  --from-c \
  --skip-tests
```

### CLI Options

**Global options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--api-key <KEY>` | `$OPENAI_API_KEY` | OpenAI API key |
| `--model <MODEL>` | gpt-5.2 | LLM model name |
| `--format <FMT>` | markdown | Output format: `markdown` or `json` |
| `--output <PATH>` | stdout | Write output to this file |
| `--config <PATH>` | none | TOML config file |

**`analyze` options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--max-depth` | 10 | Maximum module exploration depth |
| `--max-items-per-module` | 50 | Max items to analyze per module |

**`translate` options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--max-retries` | 5 | Max LLM retry attempts per program |
| `--max-lines` | 2000 | Skip source files exceeding this line count |
| `--skip-tests` | false | Only verify build succeeds |
| `--from-c` | false | Translate from C source (`test_case/`) |
| `--analyze` | false | Run invariant analysis on successful translations |
| `--report <PATH>` | none | Write an extra copy of the report |

## Configuration

Create a `config.toml` file:

```toml
[llm]
provider = "openai"
api_key = "sk-..."  # Or use OPENAI_API_KEY env var
model = "gpt-4"

[exploration]
max_depth = 10
max_items_per_module = 50
context_window_tokens = 4000

[detection]
focus = ["state_machine", "linear_types", "ownership"]
```

## How It Works

1. **Parse**: Uses `syn` to parse Rust source files and extract type definitions, functions, and traits
2. **Navigate**: Performs top-down exploration of the module hierarchy
3. **Identify**: Finds "interesting" code patterns (PhantomData, Drop implementations, consuming methods)
4. **Analyze**: Uses LLM to analyze each pattern and identify invariants
5. **Report**: Generates a detailed report with code evidence and explanations

## Architecture

```
src/
  main.rs                    # Entry point
  cli/
    mod.rs                   # CLI parsing, subcommand dispatch
    config.rs                # TOML config loading
  parser/
    ast.rs                   # Rust AST extraction (syn)
    module_graph.rs          # Crate module hierarchy
  navigation/
    explorer.rs              # Top-down module traversal
    context.rs               # Identifies interesting code items
  detection/
    state_machine.rs         # Typestate pattern detector
    linear_types.rs          # Ordering invariant detector
    ownership.rs             # Lifetime pattern detector
    evidence.rs              # Code snippet extraction
  agent/
    mod.rs                   # Main analysis orchestrator
  llm/
    types.rs                 # LlmClient trait
    openai.rs                # OpenAI implementation
  report/
    markdown.rs              # Markdown output
    json.rs                  # JSON output
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `translation` | on | Enables `translate` subcommand via `llm_translation` crate |

The `llm_translation` dependency is declared as:
```toml
llm_translation = { path = "../llm_translation", optional = true }
```

## Development

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test parser
cargo test navigation
cargo test detection

# Build without translation
cargo build --no-default-features

# Check code
cargo check
```

For detailed development documentation, see [CLAUDE.md](CLAUDE.md).

## License

[MIT](LICENSE)

## Contributing

Contributions welcome! Please check existing issues or create a new one before submitting PRs.
