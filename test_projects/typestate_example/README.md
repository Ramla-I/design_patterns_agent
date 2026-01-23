# Typestate Example Project

This is a test project for the Design Patterns Agent that demonstrates various Rust invariant patterns.

## Patterns Included

### 1. FileHandle - Classic Typestate Pattern (`src/file_handle.rs`)
- **Invariant**: Files must be opened before they can be read or written
- **Implementation**: Uses `PhantomData<S>` with `Closed` and `Open` states
- **Key Methods**:
  - `open(self) -> FileHandle<Open>` - State transition
  - `read(&self)` - Only available in `Open` state
  - `close(self) -> FileHandle<Closed>` - Reverse transition

### 2. Database Connection - Typestate with Error Handling (`src/database.rs`)
- **Invariant**: Queries require an active database connection
- **Implementation**: Uses `PhantomData<S>` with `Disconnected` and `Connected` states
- **Key Methods**:
  - `connect(self) -> Result<Connection<Connected>>` - Fallible state transition
  - `query(&self, sql)` - Only available when connected
  - `begin_transaction()` - Returns a linear type `Transaction`

- **Bonus**: `Transaction` is a linear type that must be committed or rolled back

### 3. Resource Management - Linear Types (`src/resource.rs`)
- **Invariant**: Resources must be explicitly released before being dropped
- **Implementation**: Uses `Drop` to ensure cleanup
- **Key Types**:
  - `Resource` - Must call `release()` before drop
  - `ResourceGuard` - RAII pattern that auto-releases on drop
- **Features**:
  - `#[must_use]` annotation
  - Drop implementation with emergency cleanup
  - Scoped resource guard

### 4. Builder Pattern - Typestate Builder (`src/builder.rs`)
- **Invariant**: Configuration can only be built when all required fields are set
- **Implementation**: Uses `PhantomData<S>` with `BuilderEmpty`, `BuilderWithName`, `BuilderComplete` states
- **Key Methods**:
  - `name()` - Transitions from `Empty` to `WithName`
  - `complete()` - Transitions to `BuilderComplete`
  - `build()` - Only available in `BuilderComplete` state

- **Bonus**: `Token` demonstrates capability-based access with ordering invariants
  - `level1() -> Token` - Create base token
  - `upgrade_to_level2(self) -> Token` - Must have level 1
  - `upgrade_to_level3(self) -> Token` - Must have level 2
  - `admin_operation(self)` - Requires level 3

## Running Tests

```bash
cargo test
```

## Expected Invariants to be Detected

When analyzed by the Design Patterns Agent, it should find:

**State Machine Invariants**:
1. FileHandle typestate pattern (Closed → Open)
2. Connection typestate pattern (Disconnected → Connected)
3. Builder typestate pattern (Empty → WithName → Complete)
4. Token capability ordering (Level 1 → 2 → 3)

**Linear Type Invariants**:
1. Resource must be released before drop
2. ResourceGuard RAII pattern
3. Transaction must be committed or rolled back

**State Transitions**:
1. Multiple methods that consume `self` and return different types
2. Ordering requirements in token upgrades

## Analyzing with Design Patterns Agent

From the parent directory:

```bash
cd ../..
cargo run --release -- test_projects/typestate_example --output test_analysis.md
```

This requires an OpenAI API key set via `OPENAI_API_KEY` environment variable.
