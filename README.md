# Design Patterns Agent

An AI-powered Rust code analyzer that discovers invariants in Rust codebases using LLM technology.

## What It Does

This tool analyzes Rust projects to automatically identify and document:

- **State Machine Invariants**: Typestate patterns using `PhantomData` to enforce compile-time state transitions
- **Linear Type Invariants**: Ordering requirements and capability patterns that ensure operations happen in sequence
- **Ownership Invariants**: Lifetime and borrowing patterns that enforce memory safety

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- OpenAI API key

### Installation

```bash
git clone https://github.com/yourusername/design_patterns_agent.git
cd design_patterns_agent
cargo build --release
```

### Usage

```bash
# Set your OpenAI API key
export OPENAI_API_KEY=sk-...

# Analyze a Rust codebase
cargo run --release -- /path/to/rust/project

# Save output to a file
cargo run --release -- /path/to/rust/project --output report.md

# Get JSON output
cargo run --release -- /path/to/rust/project --format json --output report.json
```

### Example Output

```markdown
# Invariant Analysis Report

## Summary
- **Total invariants discovered**: 3
- **State machine invariants**: 2
- **Linear type invariants**: 1
- **Ownership invariants**: 0

## State Machine Invariants

### 1. FileHandle Typestate Pattern
**Location**: `src/file.rs:45-78`
**Description**: FileHandle uses typestate pattern to ensure files are opened before reading.

**Evidence**:
```rust
pub struct FileHandle<S> {
    path: PathBuf,
    _state: PhantomData<S>,
}

impl FileHandle<Closed> {
    pub fn open(self) -> FileHandle<Open> { ... }
}

impl FileHandle<Open> {
    pub fn read(&mut self) -> Result<Vec<u8>> { ... }
}
```

**Explanation**: The type system prevents calling `read()` on a closed file handle.
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
cargo run -- /path/to/project --config config.toml
```

## How It Works

1. **Parse**: Uses `syn` to parse Rust source files and extract type definitions, functions, and traits
2. **Navigate**: Performs top-down exploration of the module hierarchy
3. **Identify**: Finds "interesting" code patterns (PhantomData, Drop implementations, consuming methods)
4. **Analyze**: Uses LLM to analyze each pattern and identify invariants
5. **Report**: Generates a detailed report with code evidence and explanations

## Architecture

The tool is built with a modular architecture:

- **Parser**: Rust AST extraction using `syn`
- **Navigator**: Hierarchical codebase exploration
- **Detectors**: Specialized invariant detectors (state machine, linear types, ownership)
- **LLM Integration**: Async OpenAI API client
- **Report Generator**: Markdown and JSON output

## Development

```bash
# Run tests
cargo test

# Run integration tests
cargo test --test integration_test

# Check code
cargo check

# Run on the included test project
cargo run -- test_projects/typestate_example --output test_analysis.md
```

For detailed development documentation, see [CLAUDE.md](CLAUDE.md).

## Project Status

This is **Phase 1** of the Design Patterns Agent project. Current capabilities:

- ✅ Invariant discovery
- ✅ Markdown and JSON reports
- ✅ OpenAI LLM integration
- ⏳ Pattern suggestions (Phase 2 - planned)
- ⏳ Code refactoring assistance (Phase 2 - planned)

## License

[MIT](LICENSE)

## Contributing

Contributions welcome! Please check existing issues or create a new one before submitting PRs.
