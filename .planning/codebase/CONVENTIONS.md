# Coding Conventions

**Analysis Date:** 2026-03-28

## Naming Patterns

**Files:**
- kebab-case for `.rs` files: `spoon/src/packages/mod.rs`
- snake_case for internal modules: `spoon/src/actions/model.rs`
- PascalCase for public enums: `spoon/src/actions/model.rs` - `ToolAction`

**Functions:**
- snake_case for both public and private functions
- clear action-oriented naming: `execute_tool_action`, `enable_test_mode`
- async function names are descriptive: `execute_tool_action_streaming`

**Variables:**
- snake_case for local variables
- descriptive names with scope prefixes: `path`, `config`, `child`
- short names for counters/indices: `i`, `x`, `y` in loops

**Types:**
- PascalCase for structs and enums
- public types derive common traits: `#[derive(Debug, Clone, PartialEq, Eq)]`
- Use serde serialize/deserialize where applicable

## Code Style

**Formatting:**
- Rustfmt is used for code formatting (based on Cargo.toml workspace)
- Standard Rust formatting rules apply

**Linting:**
- Clippy is used for linting
- Common Rust linting rules enforced
- No custom lint configuration detected

**Whitespace:**
- Consistent 4-space indentation
- Empty lines between logical blocks
- Trailing spaces removed

## Import Organization

**Order:**
1. Standard library imports
2. External crate imports
3. Local module imports
4. `super` and `self` imports

**Path Aliases:**
- No path aliases detected
- Relative imports used: `use super::background`
- Absolute paths for external crates: `use anyhow::Result`

## Error Handling

**Patterns:**
- `anyhow::Result<T>` for operation results
- Custom error types with `thiserror` in `spoon-backend`
- Error variants with context in `spoon-backend/src/error.rs`
- `?` operator used for error propagation

**Error Types:**
```rust
// In spoon-backend/src/error.rs
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("{message}: {source}")]
    Context { message: String, source: Box<BackendError> },
    #[error("filesystem {action} failed for {path}: {source}")]
    Fs { action: &'static str, path: PathBuf, source: std::io::Error },
}
```

**Error Conversion:**
- `#[from]` attribute for automatic conversion
- Custom methods for wrapping errors: `context()`, `fs()`, `network()`

## Logging

**Framework:** `log` crate used where applicable

**Patterns:**
- Logging calls are minimal in user-facing code
- Debug logging available via debug modal
- Log level configuration through standard Rust logging

## Comments

**When to Comment:**
- Complex business logic in CLI commands
- Platform-specific code with rationale
- TUI layout calculations
- Public API documentation

**JSDoc/TSDoc:**
- No JSDoc patterns (Rust uses rustdoc)
- Public functions and structs have minimal docs
- Comments explain "why" not "what"

## Function Design

**Size:**
- Functions are generally small and focused
- Average function size: 10-30 lines
- Maximum function size: ~100 lines for complex operations

**Parameters:**
- 1-3 parameters common
- Parameter objects for related data
- Optional parameters wrapped in `Option`

**Return Values:**
- `Result<T, E>` for fallible operations
- `Option<T>` for optional values
- Custom structs for structured results

## Module Design

**Exports:**
- Public APIs use `pub mod` and `pub use`
- Implementation details are private
- Re-exports in mod.rs for clean imports

**Barrel Files:**
- Common pattern in `spoon/src/config/mod.rs`
- Re-exports all submodules for clean imports

## Type Patterns

**Common Derives:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolAction {
    Install,
    Update,
    Uninstall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub editor: String,
    #[serde(default)]
    pub proxy: String,
}
```

**Serde Usage:**
- Default values with `#[serde(default)]`
- Custom defaults with functions: `#[serde(default = "default_msvc_arch")]`
- Structured configuration with nested types

## Async Patterns

**Naming:**
- Async functions clearly marked
- Streaming versions have `_streaming` suffix
- Tokio executor used throughout

**Usage:**
- `tokio::spawn` for background tasks
- `tokio::sync` channels for communication
- `async`/`await` pattern consistently used

## Test Support Patterns

**Test Mode:**
- Atomic flags for test mode: `static TEST_MODE: AtomicBool`
- Test mode overrides in multiple modules
- Test harnesses in `spoon/src/tui/test_support.rs`

**Test Utilities:**
- `Harness` struct for TUI testing
- Mock implementations for external dependencies
- Path isolation for test home directories

---

*Convention analysis: 2026-03-28*