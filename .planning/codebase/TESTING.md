# Testing Patterns

**Analysis Date:** 2026-03-28

## Test Framework

**Runner:**
- Rust's built-in `cargo test` / libtest harness.
- `spoon/Cargo.toml` declares explicit integration targets for CLI and TUI flows under `spoon/tests/cli/*.rs` and `spoon/tests/tui/*.rs`.
- `spoon-backend/` uses the default cargo layout: inline unit tests in `spoon-backend/src/**` plus integration tests under `spoon-backend/tests/*.rs`.
- No `#[tokio::test]` usage was detected; async work is driven from plain `#[test]` functions via helper runtimes in `spoon/src/runtime.rs`, `spoon-backend/src/tests/mod.rs`, and `spoon-backend/tests/common.rs`.

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!`, and `unwrap_err()` macros.
- Thin assertion helpers live in `spoon/tests/common/assertions.rs` for repeated CLI/TUI expectations.

**Run Commands:**
```bash
cargo test                              # Run the workspace test suite
cargo test -p spoon --test json_flow    # Run one explicit spoon integration target
cargo test -- --ignored                 # Run the opt-in real-world network/install flows
```

## Test File Organization

**Location:**
- Inline unit tests stay next to the code they validate under `#[cfg(test)]`, for example `spoon/src/config/io.rs`, `spoon/src/packages/tool.rs`, `spoon/src/tui/render/render_shared.rs`, `spoon-backend/src/tests/task.rs`, and `xtask/src/main.rs`.
- App integration tests live under `spoon/tests/cli/`, `spoon/tests/tui/`, and shared helpers under `spoon/tests/common/`.
- Backend integration tests live under `spoon-backend/tests/`.

**Naming:**
- Flow-oriented integration tests use `*_flow.rs` file names such as `spoon/tests/cli/config_flow.rs`, `spoon/tests/cli/scoop_runtime_flow.rs`, `spoon/tests/tui/tui_output_modal_flow.rs`, and `spoon/tests/tui/tui_tool_detail_flow.rs`.
- Backend inline tests are grouped by feature folder with `tests/` submodules, for example `spoon-backend/src/scoop/tests/planner.rs` and `spoon-backend/src/msvc/tests/root.rs`.
- Test function names describe behavior in sentence-style `snake_case`, such as `status_json_prints_machine_readable_status_snapshot` in `spoon/tests/cli/json_flow.rs` and `clone_repo_respects_pre_cancelled_job` in `spoon-backend/tests/gitx.rs`.

**Structure:**
```text
spoon/
├── src/...                    # inline unit tests under #[cfg(test)]
└── tests/
    ├── cli/*.rs               # binary-driven CLI flows
    ├── tui/*.rs               # ratatui Harness-driven state flows
    └── common/*.rs            # shared assertions, setup, env guards, fixtures

spoon-backend/
├── src/.../tests/*.rs         # backend unit and focused integration-style tests
└── tests/*.rs                 # crate integration tests, including ignored network flows
```

## Test Structure

**Suite Organization:**
```rust
#[path = "../common/mod.rs"]
mod common;

use common::tui::open_tools;
use spoon::tui::test_support::Harness;

#[test]
fn esc_walks_back_through_the_shell() {
    let mut app = Harness::new();
    open_tools(&mut app);
    app.press(crossterm::event::KeyCode::Esc).unwrap();
    assert_eq!(app.screen_name(), "Configure");
}
```

**Patterns:**
- Tests usually arrange a temporary home or tool root, execute one behavior, and assert on user-visible output or serialized state. See `spoon/tests/cli/cli_flow.rs`, `spoon/tests/cli/json_flow.rs`, and `spoon-backend/tests/scoop_integration.rs`.
- Shared setup is explicit rather than hidden behind fixtures. Examples: `create_configured_home()` in `spoon/tests/common/setup.rs`, `Harness::with_install_root(...)` in `spoon/src/tui/test_support.rs`, and `temp_dir(...)` in `spoon-backend/tests/common.rs`.
- Cleanup is mostly manual when a test writes to temp directories, typically via `let _ = std::fs::remove_dir_all(...)`; see `spoon-backend/tests/gitx.rs`, `spoon-backend/src/msvc/tests/official.rs`, and many inline config tests in `spoon/src/config/io.rs`.
- Environment-sensitive tests serialize access with locks when necessary: `TEST_LOCK` in `spoon/src/tui/test_support.rs` and `env_lock()` in `spoon-backend/src/msvc/tests/root.rs`.

## Mocking

**Framework:** No mocking crate is used.

**Patterns:**
```rust
let temp_home = create_test_home();
let (ok, stdout, stderr) = run_in_home(&["--json", "status"], &temp_home, &[]);
assert_ok(ok, &stdout, &stderr);
```

```rust
let host = NoopScoopRuntimeHost;
let targets = expanded_shim_targets("python", &current_root, &source, &host);
assert!(targets.iter().any(|target| target.alias == "python"));
```

- Prefer fakes and controlled temp directories over dynamic mocks. `spoon/tests/common/setup.rs` writes real config files, `spoon/tests/common/fixtures.rs` seeds manifests and spins up a local TCP server, and `spoon-backend/src/scoop/runtime/execution.rs` exposes `NoopScoopRuntimeHost` for host-level substitution.
- CLI tests invoke the real built binary via `env!("CARGO_BIN_EXE_spoon")` in `spoon/tests/common/cli.rs`.
- TUI tests drive the real app state machine with `ratatui::backend::TestBackend` through `spoon/src/tui/test_support.rs`.

**What to Mock:**
- Filesystem state via temp directories and real JSON/TOML files in `spoon/tests/common/setup.rs`, `spoon/tests/common/fixtures.rs`, and `spoon-backend/tests/scoop_integration.rs`.
- Environment variables and PATH state with guards from `spoon/tests/common/env_guard.rs` and `spoon/tests/common/windows_env.rs`.
- Slow or partial downloads with `spawn_slow_payload_server(...)` in `spoon/tests/common/fixtures.rs`.
- Host integrations through lightweight traits and no-op implementations such as `ScoopRuntimeHost` / `NoopScoopRuntimeHost` in `spoon-backend/src/scoop/runtime/execution.rs`.

**What NOT to Mock:**
- The CLI process boundary. Tests in `spoon/tests/cli/*.rs` intentionally run the real binary and assert on stdout/stderr and exit status.
- The TUI render/state loop. Tests in `spoon/tests/tui/*.rs` use `Harness` and `render_text(...)` instead of mocking screen transitions.
- Serialization formats. Tests write and read actual JSON/TOML/INI content in `spoon/src/config/io.rs`, `spoon/tests/cli/json_flow.rs`, and `spoon-backend/tests/scoop_integration.rs`.

## Fixtures and Factories

**Test Data:**
```rust
let env = create_configured_home();
let temp_home = env.home;
let tool_root = env.root;

std::fs::create_dir_all(tool_root.join("scoop").join("buckets").join("main").join("bucket")).unwrap();
std::fs::write(tool_root.join("scoop").join("buckets").join("main").join("bucket").join("jq.json"), r#"{ "version": "1.8.1" }"#).unwrap();
```

**Location:**
- App fixtures and guards live in `spoon/tests/common/fixtures.rs`, `spoon/tests/common/setup.rs`, `spoon/tests/common/env_guard.rs`, `spoon/tests/common/windows_env.rs`, and `spoon/tests/common/constants.rs`.
- TUI-only helper entry points live in `spoon/tests/common/tui.rs`.
- Backend fixtures live in `spoon-backend/tests/common.rs` and `spoon-backend/src/tests/mod.rs`.
- Test-only app harness APIs live directly in `spoon/src/tui/test_support.rs`.

## Coverage

**Requirements:** No explicit coverage threshold or coverage tool is configured. Coverage policy is behavior-focused rather than percentage-focused; `AGENTS.md` explicitly prefers regression and risky-flow tests over coverage theater.

**View Coverage:**
```bash
Not configured in this repository.
```

## Test Types

**Unit Tests:**
- Inline unit tests cover parsing, formatting, selection rules, and helper logic close to the implementation. Good examples are `spoon/src/config/env.rs`, `spoon/src/packages/tool.rs`, `spoon/src/tui/render/render_modals/utility.rs`, `spoon-backend/src/tests/task.rs`, and `xtask/src/main.rs`.

**Integration Tests:**
- `spoon/tests/cli/*.rs` exercise command flows end-to-end through the compiled binary.
- `spoon/tests/tui/*.rs` exercise the state machine, rendering, modal transitions, and action flows through `Harness`.
- `spoon-backend/tests/*.rs` validate backend behavior across multiple modules, especially git sync and Scoop/MSVC state composition.

**E2E Tests:**
- No separate browser/PTY/E2E framework is used.
- The closest E2E-style coverage is the ignored real-world flows in `spoon/tests/tui/tui_scoop_flow.rs`, `spoon/tests/cli/scoop_runtime_flow.rs`, `spoon/tests/cli/msvc_flow.rs`, and `spoon-backend/tests/gitx.rs`.

## Common Patterns

**Async Testing:**
```rust
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
```
- Use sync `#[test]` functions plus `block_on(...)` helpers from `spoon-backend/tests/common.rs` and `spoon-backend/src/tests/mod.rs`.
- App tests also call `spoon::runtime::test_block_on(...)` when they need backend async helpers from a sync integration test, as seen in `spoon/tests/cli/json_flow.rs` and `spoon/tests/tui/tui_table_render_flow.rs`.

**Error Testing:**
```rust
let json = parse_json(&stdout);
assert_eq!(json["kind"], "error");
assert!(json["data"]["chain"].is_array());
assert!(stderr.trim().is_empty());
```
- CLI error tests assert on both transport and presentation: success flag, stdout/stderr, and stable JSON envelopes. See `spoon/tests/cli/json_flow.rs` and `spoon/tests/common/assertions.rs`.
- Backend error tests often use `unwrap_err()` and string matching for high-value invariants, for example cancellation assertions in `spoon-backend/tests/gitx.rs`.
- TUI error and blocked-action tests assert on modal state and inline hints rather than internal error objects, as in `spoon/tests/tui/tui_tool_detail_flow.rs`.

## Opt-In Real Flows

**Ignored tests:**
- There are 8 ignored tests in the current tree.
- Real network or install/update/uninstall flows are kept behind `#[ignore]` in:
  - `spoon/tests/cli/msvc_flow.rs`
  - `spoon/tests/cli/scoop_runtime_flow.rs`
  - `spoon/tests/tui/tui_scoop_flow.rs`
  - `spoon-backend/tests/gitx.rs`
- Run them explicitly with `cargo test -- --ignored` and expect local machine prerequisites such as network access, proxy availability, or a prepared managed MSVC toolchain.

---

*Testing analysis: 2026-03-28*
