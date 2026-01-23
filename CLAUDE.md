# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Design Patterns Agent** is a Rust CLI tool that analyzes Rust codebases to discover invariants. It uses LLM-powered analysis to identify:
- State machine invariants (typestate patterns)
- Linear type invariants (ordering requirements, capabilities)
- Ownership and lifetime patterns

## Development Commands

### Build and Test
```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run tests for a specific module
cargo test parser
cargo test navigation
cargo test detection

# Check code without building
cargo check

# Run with optimizations
cargo build --release
```

### Running the Tool
```bash
# Basic usage
cargo run -- /path/to/rust/codebase

# With output file
cargo run -- /path/to/rust/codebase --output report.md

# JSON output format
cargo run -- /path/to/rust/codebase --format json --output report.json

# With custom configuration
cargo run -- /path/to/rust/codebase --config config.toml

# Specify API key and model
cargo run -- /path/to/rust/codebase --api-key sk-... --model gpt-4
```

### Environment Variables
```bash
# Set OpenAI API key
export OPENAI_API_KEY=sk-...
```

## Architecture

The codebase follows a modular architecture with seven main components:

### 1. CLI (`src/cli/`)
- **Entry Point**: `src/main.rs` and `src/cli/mod.rs`
- **Configuration**: `src/cli/config.rs` handles CLI args and config file loading
- Parses command-line arguments using `clap`
- Supports both CLI args and TOML configuration files

### 2. Parser (`src/parser/`)
- **Core Module**: `src/parser/mod.rs`
- **AST Extraction**: `src/parser/ast.rs` uses `syn` to extract Rust constructs
- **Module Graph**: `src/parser/module_graph.rs` builds the crate's module hierarchy
- Parses Rust source files and extracts:
  - Type definitions (structs, enums)
  - Function signatures and implementations
  - Trait definitions and implementations
  - PhantomData usage (typestate indicator)
  - Doc comments and inline comments

### 3. Navigator (`src/navigation/`)
- **Orchestration**: `src/navigation/mod.rs` coordinates exploration
- **Explorer**: `src/navigation/explorer.rs` performs top-down module traversal
- **Context**: `src/navigation/context.rs` identifies "interesting" code items
- Implements hierarchical exploration starting from crate root
- Identifies candidates for invariant analysis:
  - Types with PhantomData (typestate)
  - Types with Drop implementations (linear types)
  - Methods that consume `self` (state transitions)

### 4. Detection (`src/detection/`)
- **Coordinator**: `src/detection/mod.rs` routes items to specialized detectors
- **State Machine**: `src/detection/state_machine.rs` detects typestate patterns
- **Linear Types**: `src/detection/linear_types.rs` finds ordering invariants
- **Ownership**: `src/detection/ownership.rs` analyzes lifetime patterns
- **Evidence**: `src/detection/evidence.rs` extracts code snippets
- Each detector uses specialized LLM prompts for its domain
- Parses structured LLM responses to extract invariants

### 5. LLM Integration (`src/llm/`)
- **Client Trait**: `src/llm/types.rs` defines the LLM client interface
- **OpenAI Client**: `src/llm/openai.rs` implements OpenAI API calls
- Uses `async-openai` for API communication
- Abstracts LLM provider behind a trait for future extensibility
- Handles async API calls with error handling and retries

### 6. Agent Loop (`src/agent/`)
- **Main Orchestration**: `src/agent/mod.rs` contains `analyze_codebase()`
- Ties all components together:
  1. Creates LLM client
  2. Builds module graph via Navigator
  3. Explores codebase to find interesting items
  4. Runs detectors on each item
  5. Aggregates results into Report
- Provides progress output during analysis

### 7. Report Generation (`src/report/`)
- **Report Types**: `src/report/mod.rs` defines Report, Invariant, Evidence structures
- **Markdown**: `src/report/markdown.rs` generates formatted markdown output
- **JSON**: `src/report/json.rs` generates JSON output
- Groups invariants by type
- Includes code snippets with locations
- Summary statistics

## Key Data Flow

```
User Input (CLI)
  → Config
  → Navigator (builds module graph)
  → Explorer (finds interesting items)
  → Detector (analyzes with LLM)
  → Report (formats results)
  → Output (markdown/JSON)
```

## Important Implementation Details

### Typestate Detection
The parser specifically looks for `PhantomData<T>` in struct fields to identify potential typestate patterns. When found, the detector:
1. Extracts the struct and all related impl blocks
2. Looks for methods that consume `self` and return different types
3. Asks the LLM to confirm and explain the invariant

### Linear Type Detection
Identifies types with Drop implementations or explicit ordering requirements:
1. Finds types with `impl Drop`
2. Looks for methods that must be called in sequence
3. Analyzes capability-passing patterns

### Evidence Extraction
Code snippets are formatted to include:
- Struct definitions with fields
- Relevant impl blocks with method signatures
- Drop implementations where applicable

### LLM Prompts
Each detector uses specialized prompts that:
- Define the invariant type being searched for
- Provide examples of patterns to look for
- Request structured output for easy parsing

## Testing Strategy

Tests are organized by module:
- **Unit Tests**: Each module has its own `#[cfg(test)]` section
- **Integration Tests**: `src/agent/mod.rs` has end-to-end test structure
- **Test Data**: Uses `tempfile` crate to create temporary Rust projects
- **Mocking**: LLM tests verify structure without requiring API keys

## Configuration File Format

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

## Dependencies

### Key External Crates
- **syn**: Rust parser for AST extraction
- **quote**: Code generation (used for formatting types)
- **async-openai**: OpenAI API client
- **clap**: CLI argument parsing
- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **walkdir**: Recursive directory traversal
- **anyhow/thiserror**: Error handling

## Future Enhancements

The codebase is structured to support:
- Additional LLM providers (trait-based abstraction)
- More invariant types (pluggable detector architecture)
- Better line number tracking (currently set to 1 as placeholder)
- Caching of LLM responses
- Parallel analysis of multiple items
- Pattern suggestion phase (Phase 2 of the project)
