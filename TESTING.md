# Testing Guide

This document describes how to test the Design Patterns Agent.

## Unit Tests

We have 47 unit tests covering all major components:

```bash
# Run all unit tests
cargo test

# Run tests for specific modules
cargo test parser
cargo test navigation
cargo test detection
cargo test report
cargo test llm
```

### Test Coverage

- **Parser Module** (10 tests): AST extraction, module graph building, PhantomData detection
- **Navigation Module** (11 tests): Explorer, context extraction, interesting item detection
- **Detection Module** (11 tests): Evidence extraction, state machine detection, response parsing
- **Report Module** (9 tests): Markdown generation, JSON serialization
- **LLM Module** (4 tests): Client creation, request building
- **CLI Module** (2 tests): Argument parsing

## Integration Tests

Integration tests verify the full pipeline works correctly:

```bash
# Run integration tests
cargo test --test integration_test -- --nocapture
```

### What Integration Tests Verify

1. **Parser**: Can parse the test Rust project
2. **Module Graph**: Builds correct module hierarchy
3. **Explorer**: Finds interesting code items
4. **Pattern Detection**: Identifies:
   - Typestate candidates (PhantomData usage)
   - Linear type candidates (Drop implementations)
   - State transitions (consuming methods)

### Test Project Results

When run on `test_projects/typestate_example`, the tool finds:

- **5 modules**: lib, file_handle, database, resource, builder
- **13 interesting items**:
  - 3 typestate candidates (FileHandle, Connection, Builder)
  - 2 linear type candidates (Resource, ResourceGuard)
  - 8 state transition patterns

## End-to-End Testing (Requires API Key)

To test the complete analysis including LLM integration:

```bash
# Set your OpenAI API key
export OPENAI_API_KEY=sk-...

# Run analysis on test project
cargo run --release -- test_projects/typestate_example --output test_report.md

# View the generated report
cat test_report.md
```

### Expected Output

The tool should identify and document:

**State Machine Invariants**:
- FileHandle typestate pattern (Closed ↔ Open states)
- Connection typestate pattern (Disconnected ↔ Connected states)
- Builder typestate pattern (Empty → WithName → Complete)
- Token capability ordering (Level 1 → 2 → 3)

**Linear Type Invariants**:
- Resource cleanup requirements (must call release())
- ResourceGuard RAII pattern (auto-cleanup)
- Transaction requirements (must commit or rollback)

## Test Project Structure

The `test_projects/typestate_example` directory contains:

```
test_projects/typestate_example/
├── Cargo.toml
├── README.md              # Detailed pattern documentation
└── src/
    ├── lib.rs            # Module declarations
    ├── file_handle.rs    # Classic typestate (Closed/Open)
    ├── database.rs       # Connection typestate + Transaction
    ├── resource.rs       # Linear types with Drop
    └── builder.rs        # Builder typestate + Token capabilities
```

Each module is extensively documented with:
- INVARIANT comments explaining what guarantees are enforced
- Doc comments on types and methods
- Test cases demonstrating correct usage

## Performance Testing

For larger codebases:

```bash
# Test on a real Rust project (example)
time cargo run --release -- /path/to/large/project --output report.md

# Monitor LLM API usage
# The tool prints progress including LLM calls
```

## Debugging

Enable verbose output:

```bash
# The tool already prints progress by default
cargo run -- test_projects/typestate_example

# Expected output:
# 🔍 Initializing analysis...
# 🤖 Connecting to LLM provider: openai
# 📂 Building module graph...
#    Found 5 modules
# 🧭 Exploring codebase...
#    Found 13 interesting code items
# 🔬 Analyzing items for invariants...
#    [1/13] Analyzing: file_handle
#      ✓ Found: FileHandle Typestate Pattern
#    [2/13] Analyzing: database
#      ✓ Found: Connection State Machine
# ...
# 📊 Analysis complete!
#    Total invariants discovered: 7
#    - State machine: 4
#    - Linear type: 2
#    - Ownership: 1
```

## Continuous Integration

For CI/CD pipelines:

```bash
# Run all tests (no API key needed)
cargo test --all

# Build release binary
cargo build --release

# Verify it compiles
cargo check --all-targets
```

## Known Limitations

1. **Line Numbers**: Currently set to placeholder values (line 1). Future improvement needed.
2. **LLM Costs**: Each interesting item makes 1-3 LLM calls depending on detection strategy.
3. **No Caching**: LLM responses are not cached yet. Same code analyzed twice makes duplicate calls.
4. **Module Depth**: Very deep module hierarchies may hit the max_depth limit.

## Test Data Quality

The test project includes:

✅ **Well-documented invariants** with comments
✅ **Multiple pattern variations** (typestate, linear types, capabilities)
✅ **Real-world patterns** commonly found in Rust codebases
✅ **Both compile-time and runtime** invariants
✅ **Edge cases** (error handling, RAII, Drop guards)

This ensures the LLM has clear context to identify and explain invariants.
