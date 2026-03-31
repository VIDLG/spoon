# Coding Conventions

**Analysis Date:** 2026-03-28

## Naming Patterns

**Files:**
- Use `snake_case.rs` for source modules and keep test flow names descriptive by domain, for example `spoon/src/config/io.rs`, `spoon/src/status/discovery/probe.rs`, `spoon-backend/src/scoop/runtime/execution.rs`, `spoon/tests/cli/json_flow.rs`, and `spoon/tests/tui/tui_navigation_flow.rs`.
- Keep shared test helpers under `common/` modules such as `spoon/tests/common/cli.rs` and `spoon-backend/tests/common.rs`.

**Functions:**
- Use `snake_case` verbs for functions and methods, for example `save_global_config` in `spoon/src/config/io.rs`, `execute_tool_action_streaming` in `spoon/src/actions/execute/mod.rs`, `infer_owned_root_from_path` in `spoon/src/packages/tool.rs`, and `await_task_with_events` in `spoon-backend/src/task.rs`.
- Prefer names that encode the side effect or lookup target directly: `_path`, `_root`, `_config`, `_state`, `_output`, `_streaming`, `_for_test`, and `_with_host` suffixes are common in `spoon/src/config/io.rs`, `spoon/src/tui/test_support.rs`, and `spoon-backend/src/scoop/runtime/execution.rs`.

**Variables:**
- Use `snake_case` locals with explicit role names such as `tool_root`, `temp_home`, `state_root`, `manifest_root`, `output_lines`, `path_guard`, and `progress_events`; see `spoon/tests/cli/msvc_flow.rs`, `spoon/tests/tui/tui_scoop_action_flow.rs`, and `spoon-backend/tests/scoop_integration.rs`.
- Reserve `ALL_CAPS` for constants and static test guards, for example `CLI_SECTION_HEADERS` in `spoon/src/cli/output.rs`, `DEFAULT_WAIT` in `spoon/tests/common/constants.rs`, and `TEST_LOCK` in `spoon/src/tui/test_support.rs`.

**Types:**
- Use `PascalCase` for structs and enums, and derive traits aggressively when values cross module or serialization boundaries. Examples: `Cli` in `spoon/src/cli/args.rs`, `Tool` in `spoon/src/packages/tool.rs`, `BackendError` in `spoon-backend/src/error.rs`, and `ProgressEvent` in `spoon-backend/src/event.rs`.
- When a public enum is serialized, add serde casing at the type boundary instead of hand-formatting strings, for example `CommandStatus` in `spoon-backend/src/lib.rs` and `ConfigBadgeTone` in `spoon/src/packages/mod.rs`.

## Code Style

**Formatting:**
- Repository-wide text rules come from `.editorconfig`: UTF-8, LF, final newline, and trimmed trailing whitespace.
- No `rustfmt.toml`, `.rustfmt.toml`, `clippy.toml`, or `.clippy.toml` are present at the repo root. Follow standard `rustfmt` layout and keep formatting close to what already exists in `spoon/src/config/io.rs`, `spoon/src/packages/tool.rs`, `spoon-backend/src/gitx.rs`, and `xtask/src/main.rs`.
- Prefer multi-line builder and chain formatting when a call carries context or multiple fields. Examples: `save_codex_config` in `spoon/src/config/io.rs`, `tool_action_start` logging in `spoon/src/logger/events/app.rs`, and `ProgressEvent::new(...)` use in `spoon-backend/src/gitx.rs`.
- Prefer early returns and `let ... else` to flatten control flow. See `load_policy_config` in `spoon/src/config/io.rs`, `probe_msvc_toolchain` in `spoon/src/status/discovery/probe.rs`, and `emit_progress` in `spoon-backend/src/gitx.rs`.

**Linting:**
- No repo-specific lint policy is configured. There are no crate-level `deny(...)` or clippy policy files in the workspace manifests.
- The only recurring suppression is `#![allow(dead_code)]` in helper-heavy test modules such as `spoon/tests/common/assertions.rs`, `spoon/tests/common/cli.rs`, and `spoon-backend/tests/common.rs`. Keep suppressions narrow and test-scoped.

## Import Organization

**Order:**
1. Standard library imports first, for example `use std::fs;` and `use std::path::{Path, PathBuf};` in `spoon/src/config/io.rs` and `spoon-backend/src/gitx.rs`.
2. External crate imports second, for example `use anyhow::{Context, Result};`, `use serde_json::{Map, Value};`, and `use tokio::fs;`.
3. Internal imports last, grouped by `crate::...`, `super::...`, and then selective `pub use` re-exports, as in `spoon/src/service/mod.rs`, `spoon/src/actions/execute/mod.rs`, and `spoon-backend/src/lib.rs`.

**Path Aliases:**
- Not detected. Internal code uses relative module paths (`super::...`) and crate-root paths (`crate::...`) instead of alias maps.

## Error Handling

**Patterns:**
- Use `anyhow::Result` with `.context(...)` or `.with_context(...)` at application edges in `spoon/`, especially around filesystem and process orchestration. See `spoon/src/config/io.rs`, `spoon/src/main.rs`, and `spoon/src/config/env.rs`.
- Use the typed backend error model in `spoon-backend/`. `spoon-backend/src/error.rs` centralizes variants like `Fs`, `Network`, `HttpClient`, `Git`, `Task`, and `Cancelled`; backend modules construct these with helpers such as `BackendError::fs(...)`, `BackendError::git(...)`, and `BackendError::task(...)`.
- Translate backend failures to app-facing `anyhow` only at the boundary. The bridge lives in `spoon/src/service/mod.rs` via `backend_to_anyhow(...)`.
- Prefer user-facing, contextual error text over bare propagation. Examples: `"failed to write {}"` in `spoon/src/config/io.rs`, `"failed to update user PATH: {err}"` in `spoon/src/service/mod.rs`, and `"failed to remove old shim root {}: {err}"` in `spoon-backend/src/scoop/runtime/execution.rs`.
- Reserve `unwrap()` and `expect()` for tests, one-time initialization, and invariant-only code paths. Production code overwhelmingly returns `Result` or `Option`; test code uses `unwrap()` heavily in `spoon/tests/*`, `spoon-backend/src/*/tests/*.rs`, and `xtask/src/main.rs` tests.

## Logging

**Framework:** `tracing`

**Patterns:**
- Emit structured event names with fields, not free-form paragraphs, in dedicated logger modules such as `spoon/src/logger/events/app.rs`, `spoon/src/logger/events/config.rs`, `spoon/src/logger/events/editor.rs`, and `spoon/src/logger/events/tui.rs`.
- Use terse dotted event names like `session.start`, `app.start`, `tool.action.start`, and `config.root.set`.
- Mirror streamed backend lines into logs when the stream itself matters. Examples: `emit_backend_event` in `spoon/src/service/scoop/runtime.rs`, line-level logging in `spoon-backend/src/msvc/common.rs`, and progress/status logging in `spoon-backend/src/scoop/runtime/actions.rs`.
- Keep log configuration centralized in `spoon/src/logger/settings.rs`: verbose mode enables stdout info logs, while file and TUI logging remain more detailed.

## Comments

**When to Comment:**
- Keep comments sparse and targeted at non-obvious behavior. Examples include the backend error type docs in `spoon-backend/src/error.rs`, the gix progress adapter explanation in `spoon-backend/src/gitx.rs`, and test module headers in `spoon-backend/src/tests/mod.rs`.
- Do not narrate straightforward assignments or control flow. Most business logic files such as `spoon/src/packages/tool.rs` and `spoon/src/actions/execute/mod.rs` rely on naming instead of inline commentary.

**JSDoc/TSDoc:**
- Not applicable. Rust doc comments are used selectively on public backend APIs and enums in `spoon-backend/src/error.rs`, `spoon-backend/src/gitx.rs`, and `spoon-backend/src/event.rs`.

## Function Design

**Size:** Keep orchestration functions readable by moving parsing, formatting, and state transitions into helpers. `spoon/src/actions/execute/mod.rs` delegates backend-specific execution; `spoon/src/service/mod.rs` contains conversion helpers; `spoon-backend/src/gitx.rs` splits progress handling into `GixProgressState`, `GixProgress`, and small utility functions.

**Parameters:** Prefer borrowed inputs and typed enums over loosely structured maps. Common signatures use `&Path`, `Option<&Path>`, `&str`, slices, or callback references such as `&mut dyn FnMut(...)`; see `spoon/src/service/scoop/runtime.rs`, `spoon/src/status/discovery/probe.rs`, `spoon-backend/src/task.rs`, and `spoon-backend/src/scoop/runtime/execution.rs`.

**Return Values:** Return domain structs, `Option<T>`, or `Result<T>` rather than sentinel strings. Examples include `ProbeResult` in `spoon/src/status/discovery/probe.rs`, `CommandResult` in `spoon/src/service/mod.rs`, `RepoSyncOutcome` in `spoon-backend/src/gitx.rs`, and `Result<Vec<String>>` for path/setup operations in `spoon-backend/src/scoop/runtime/execution.rs`.

## Module Design

**Exports:** Use `mod.rs` or crate root files to define the surface area explicitly. `spoon/src/lib.rs`, `spoon-backend/src/lib.rs`, and `spoon/src/service/mod.rs` re-export selected types instead of exposing every submodule detail.

**Barrel Files:** Moderate usage. Barrel-style `mod.rs` files are the standard module boundary pattern in both crates, but they stay selective:
- `spoon/src/service/mod.rs` re-exports backend-facing service types and helper functions.
- `spoon-backend/src/lib.rs` re-exports backend event, error, git, proxy, and task APIs.
- Feature areas still keep their implementation split across nested modules such as `spoon/src/tui/render/...` and `spoon-backend/src/scoop/runtime/...`.

---

*Convention analysis: 2026-03-28*
