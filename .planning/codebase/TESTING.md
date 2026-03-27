# Testing Patterns

**Analysis Date:** 2026-03-28

## Test Framework

**Runner:**
- No unit test framework configuration detected in Cargo.toml
- Rust's built-in test runner used

**Assertion Library:**
- Standard `assert!` macro used
- `Option` matching and `Result` handling for assertions
- Custom assertion helpers in test support

**Run Commands:**
```bash
cargo test                    # Run all tests
cargo test --package spoon-backend  # Run specific package tests
cargo test --release         # Run tests in release mode
```

## Test File Organization

**Location:**
- Tests co-located in `tests/` directories alongside source
- Integration tests separate from unit tests
- Test fixtures in `test/fixtures/` directories

**Structure:**
```
spoon/
└── tests/
    ├── cli/
    │   ├── config_flow.rs    # Configuration flow tests
    │   ├── msvc_flow.rs     # MSVC installation tests
    │   └── scoop_runtime_flow.rs  # Scoop runtime tests
    └── mod.rs               # Test setup and common utilities

spoon-backend/
└── tests/
    ├── common.rs             # Shared test utilities
    ├── msvc_integration.rs   # MSVC integration tests
    ├── scoop_integration.rs  # Scoop integration tests
    └── mod.rs               # Test modules
```

## Test Structure

**Suite Organization:**
```rust
// Integration test example
#[tokio::test]
async fn test_scoop_install() -> Result<()> {
    // Setup
    let temp_dir = tempfile::tempdir()?;

    // Test execution
    let result = install_scoop_package("some-package", temp_dir.path()).await?;

    // Verification
    assert_eq!(result.status, CommandStatus::Success);
    assert!(path_exists(temp_dir.path().join("installed/path")));

    Ok(())
}
```

**Patterns:**
- `#[tokio::test]` for async tests
- `Result<T, E>` return type for error handling
- Temp directories for isolation
- State verification through file system checks

## Mocking

**Framework:** Custom mocking implementation

**Patterns:**
```rust
// Test mode in launcher.rs
static TEST_MODE: AtomicBool = AtomicBool::new(false);

pub fn enable_test_mode() {
    TEST_MODE.store(true, Ordering::Relaxed);
}

pub fn open_in_editor(command_line: &str) -> Result<LaunchResult> {
    if TEST_MODE.load(Ordering::Relaxed) {
        return Ok(LaunchResult { pid: Some(0) });
    }
    // Real implementation
}
```

**What to Mock:**
- External process spawning
- File system operations
- Network requests
- TUI interactions

**What NOT to Mock:**
- Business logic
- Configuration loading
- State transitions

## Fixtures and Factories

**Test Data:**
```rust
// Test fixture in contrib/Scoop/test/fixtures/
- Manifest files: wget.json, broken_schema.json
- Format fixtures: formatted/ and unformatted/ directories

// Test data in spoon-backend/tests/common.rs
- Shared test directories
- Mock runtime environments
- Sample package manifests
```

**Location:**
- Fixtures in `test/fixtures/`
- Shared utilities in `tests/common.rs`
- Test-specific data alongside test files

## Coverage

**Requirements:** No coverage requirements enforced

**View Coverage:**
```bash
cargo test -- --nocapture   # Run with output
cargo tarpaulin             # Generate coverage report (if installed)
```

## Test Types

**Unit Tests:**
- Limited unit tests detected
- Primarily in utility modules
- Focus on edge cases and error handling

**Integration Tests:**
- Full CLI flow testing
- TUI interaction testing via `test_support.rs`
- Environment integration tests

**E2E Tests:**
- CLI command testing
- Installation workflow testing
- Configuration validation

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn test_background_task() -> Result<()> {
    let harness = Harness::new();
    let mut harness = harness;

    // Start background action
    let task = start_background_task(&mut harness);

    // Wait for completion
    harness.wait_until(Duration::from_secs(10), |h| {
        h.background_action.is_none()
    });

    // Verify result
    assert_eq!(task.status, TaskStatus::Completed);

    Ok(())
}
```

**TUI Testing:**
```rust
// Using test harness
let mut harness = Harness::new();

// Simulate user input
harness.press(KeyCode::Down)?;
harness.press(KeyCode::Enter)?;

// Verify UI state
assert_eq!(harness.tools_selected_index(), Some(1));
assert_eq!(harness.modal_name(), Some("ToolDetail"));

// Mock output
harness.set_output_modal_for_test(
    "Test Output",
    vec!["line 1".to_string(), "line 2".to_string()],
    false,
);
```

**Error Testing:**
```rust
#[test]
fn test_error_handling() -> Result<()> {
    let result = potentially_failing_operation();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("expected error"));

    Ok(())
}
```

## Test Utilities

**Harness Pattern:**
```rust
// In spoon/src/tui/test_support.rs
pub struct Harness {
    app: App,
    _guard: MutexGuard<'static, ()>,
}

impl Harness {
    pub fn new() -> Self {
        // Setup test environment
        let test_home = temp_dir();
        config::set_home_override(test_home);
        Self { app: App::new(), _guard: guard }
    }

    pub fn press(&mut self, code: KeyCode) -> Result<bool> {
        keys::handle_key(&mut self.app, KeyEvent::new(code, KeyModifiers::NONE))?;
        self.settle();
        Ok(quit)
    }
}
```

**State Verification:**
- Public methods for common assertions
- Screen and modal state checking
- Output content validation
- Tool status verification

## Test Isolation

**Environment:**
- Test home directories created per test
- Atomic counters for unique test names
- Mutex guards for thread safety

**Cleanup:**
- `Drop` implementation for cleanup
- Temporary directories removed automatically
- Test mode state reset

---

*Testing analysis: 2026-03-28*