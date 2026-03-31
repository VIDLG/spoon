# Architecture

**Analysis Date:** 2026-03-28

## Pattern Overview

**Overall:** Rust workspace with a thin app-orchestrator crate over a reusable backend capability crate.

**Key Characteristics:**
- `spoon/src/main.rs` selects between CLI mode and the default interactive TUI, but both modes delegate into the same app modules under `spoon/src/`.
- `spoon/src/service/` adapts `spoon-backend/src/` into app-facing results, progress streams, config injection, and host-specific integrations.
- `spoon/src/packages/` and `spoon/src/status/` provide a metadata-driven runtime model: tool definitions, config scopes, ownership detection, and action policy are data/trait driven rather than hard-coded in the UI.

## Layers

**Workspace / Crate Boundary:**
- Purpose: Separate the runnable application, reusable backend logic, and repo-local developer automation.
- Location: `Cargo.toml`, `spoon/Cargo.toml`, `spoon-backend/Cargo.toml`, `xtask/Cargo.toml`
- Contains: The workspace root plus three crates: `spoon`, `spoon-backend`, and `xtask`.
- Depends on: Cargo workspace membership and path dependency from `spoon` to `spoon-backend`.
- Used by: `cargo run -p spoon`, tests in both crates, and `cargo run -p xtask -- deploy`.

**Application Shell Layer:**
- Purpose: Parse commands, initialize process state, and choose CLI or TUI execution.
- Location: `spoon/src/main.rs`, `spoon/src/cli/`, `spoon/src/tui/`
- Contains: Clap argument definitions in `spoon/src/cli/args.rs`, CLI dispatch in `spoon/src/cli/run.rs`, and terminal loop/render/input handling in `spoon/src/tui/mod.rs`.
- Depends on: `spoon/src/config/`, `spoon/src/logger/`, `spoon/src/status/`, `spoon/src/service/`, `spoon/src/actions/`, `spoon/src/view/`
- Used by: End users invoking `spoon.exe` with or without subcommands.

**Application Orchestration Layer:**
- Purpose: Turn app intents into backend calls and normalized UI/CLI results.
- Location: `spoon/src/actions/`, `spoon/src/service/`, `spoon/src/status/`
- Contains: Tool action execution in `spoon/src/actions/execute/mod.rs`, status probing in `spoon/src/status/discovery/probe.rs`, and backend adapters in `spoon/src/service/mod.rs`, `spoon/src/service/scoop/mod.rs`, and `spoon/src/service/msvc/mod.rs`.
- Depends on: `spoon/src/packages/`, `spoon/src/config/`, `spoon-backend/src/`
- Used by: Both `spoon/src/cli/run.rs` and `spoon/src/tui/action_flow/tools.rs`.

**Configuration / Integration Layer:**
- Purpose: Persist Spoon-owned config and bridge managed tools into native user config and PATH.
- Location: `spoon/src/config/`, `spoon/src/packages/`, `spoon/src/editor/`, `spoon/src/platform/`, `spoon/src/launcher.rs`
- Contains: File path derivation in `spoon/src/config/paths.rs`, file IO in `spoon/src/config/io.rs`, package-specific config/integration behavior in `spoon/src/packages/*.rs`, editor discovery/launching in `spoon/src/editor/`, and Windows process launching in `spoon/src/platform/shell.rs`.
- Depends on: User home/config files and the backend host interfaces exposed by `spoon-backend/src/scoop/runtime/execution.rs` and `spoon-backend/src/msvc/mod.rs`.
- Used by: Config commands, TUI forms, and post-install command-surface/integration reapply flows.

**Presentation / View-Model Layer:**
- Purpose: Build stable display models from runtime/config data without putting rendering rules into the services.
- Location: `spoon/src/view/`, `spoon/src/tui/render/`
- Contains: Tool detail/status models in `spoon/src/view/tools/`, config models in `spoon/src/view/config.rs`, and Ratatui rendering code in `spoon/src/tui/render/render_pages/` and `spoon/src/tui/render/render_modals/`.
- Depends on: `spoon/src/status/`, `spoon/src/packages/`, `spoon/src/config/`
- Used by: CLI human-readable output and the TUI.

**Reusable Backend Layer:**
- Purpose: Own Scoop runtime execution, MSVC runtime execution, backend events, and reusable filesystem/network helpers.
- Location: `spoon-backend/src/scoop/`, `spoon-backend/src/msvc/`, `spoon-backend/src/event.rs`, `spoon-backend/src/task.rs`
- Contains: Scoop package planning and execution in `spoon-backend/src/scoop/planner.rs` and `spoon-backend/src/scoop/runtime/`, MSVC runtime/status/validation logic in `spoon-backend/src/msvc/mod.rs` and `spoon-backend/src/msvc/status.rs`, and progress/cancellation primitives in `spoon-backend/src/event.rs` and `spoon-backend/src/task.rs`.
- Depends on: Filesystem/network/process APIs and host callbacks supplied by the app for app-specific integration work.
- Used by: `spoon/src/service/` only; the app re-exports the backend as `spoon::backend` in `spoon/src/lib.rs`.

**Repo-local Developer Utility Layer:**
- Purpose: Build and replace the root executable.
- Location: `xtask/src/main.rs`
- Contains: The `deploy` command that builds `spoon` and replaces `spoon.exe` in the repo root and `~/.local/bin/`.
- Depends on: Cargo build output and Windows process/file replacement behavior.
- Used by: Maintainers, not end users at runtime.

## Data Flow

**CLI Command Flow:**

1. `spoon/src/main.rs` parses `Cli` from `spoon/src/cli/args.rs`, initializes logging, test-mode overrides, and resolves `root`.
2. `spoon/src/cli/run.rs` dispatches the selected `Commands` variant and resolves the effective configured root when a command needs Spoon-managed state.
3. Domain-specific service adapters in `spoon/src/service/scoop/` and `spoon/src/service/msvc/` call into `spoon-backend`, mapping backend results into `CommandResult` and `StreamChunk`.
4. `spoon/src/cli/output.rs`, `spoon/src/cli/json.rs`, and `spoon/src/cli/messages.rs` render either structured JSON or formatted text.

**TUI Flow:**

1. `spoon/src/tui/mod.rs` builds `App` from `spoon/src/tui/state.rs`.
2. `App::new` immediately loads a fast local status snapshot through `spoon/src/status/discovery/probe.rs` and starts a background worker through `spoon/src/runtime.rs`.
3. Keyboard/mouse handlers in `spoon/src/tui/keys/` and action coordinators in `spoon/src/tui/action_flow/` mutate `App`, open modals, and start long-running actions.
4. Background workers stream lines back over channels, `spoon/src/tui/background.rs` merges them into `OutputState`, and a follow-up status refresh repopulates the tools table.

**Tool Action Flow:**

1. The actionable tool set is derived from `ToolStatus` plus `ActionPolicy` in `spoon/src/status/policy.rs`.
2. `spoon/src/actions/execute/mod.rs` partitions selected tools by backend (`Scoop` versus `Native`).
3. Scoop actions route through `spoon/src/service/scoop/actions.rs`, which creates a `ScoopPackagePlan` via `spoon-backend/src/scoop/planner.rs`.
4. Runtime execution happens in `spoon-backend/src/scoop/runtime/actions.rs` or `spoon-backend/src/msvc/mod.rs`, with streamed backend events translated back into app `StreamChunk` updates.

**Configuration / Integration Flow:**

1. Config files are loaded and normalized by `spoon/src/config/io.rs` and `spoon/src/config/paths.rs`.
2. `spoon/src/packages/mod.rs` routes package-specific config and integration work through the `PackageSpec` trait implemented in files such as `spoon/src/packages/git.rs`, `spoon/src/packages/msvc.rs`, and `spoon/src/packages/codex.rs`.
3. Scoop runtime integration work crosses the crate boundary through `ScoopRuntimeHost` from `spoon-backend/src/scoop/runtime/execution.rs`, implemented by `AppScoopRuntimeHost` in `spoon/src/service/scoop/runtime.rs`.
4. Reapply flows update shims, PATH, and managed/native config files after policy changes or installs.

**State Management:**
- Process-level startup state lives in `spoon/src/main.rs` and `spoon/src/config/state.rs`.
- Interactive UI state is centralized in the `App` struct in `spoon/src/tui/state.rs`.
- Long-running action state is channel-driven via `BackgroundAction`, `BackgroundEvent`, and `tokio::sync::mpsc`.
- Persistent managed runtime state is file-based under the configured root, with path conventions defined in `spoon/src/config/paths.rs`, `spoon-backend/src/scoop/paths.rs`, and `spoon-backend/src/msvc/paths.rs`.

## Key Abstractions

**Tool Registry:**
- Purpose: Describe every managed or observed tool in one place.
- Examples: `spoon/src/packages/mod.rs`, `spoon/src/packages/tool.rs`, `spoon/src/packages/simple.rs`, `spoon/src/packages/msvc.rs`
- Pattern: Static metadata registry plus per-package `PackageSpec` implementations.

**Tool Status Model:**
- Purpose: Separate probing, ownership, readiness, and display details from UI code.
- Examples: `spoon/src/status/mod.rs`, `spoon/src/status/policy.rs`, `spoon/src/status/discovery/probe.rs`
- Pattern: Probe result -> `ToolStatus` -> action policy / view model.

**Backend Adapter Surface:**
- Purpose: Convert reusable backend APIs into app-facing results and inject app config/host behavior.
- Examples: `spoon/src/service/mod.rs`, `spoon/src/service/scoop/mod.rs`, `spoon/src/service/scoop/runtime.rs`, `spoon/src/service/msvc/mod.rs`
- Pattern: Thin adapter layer with result mapping and host callbacks.

**Streaming Event Bridge:**
- Purpose: Preserve progress reporting and cancellation across crate and UI boundaries.
- Examples: `spoon-backend/src/event.rs`, `spoon-backend/src/task.rs`, `spoon/src/service/mod.rs`, `spoon/src/tui/background.rs`
- Pattern: Backend emits typed progress/finish events, app converts them to append/replace stream chunks, UI consumes chunks incrementally.

**Managed Root Path Model:**
- Purpose: Keep all managed runtime state derivable from a single configured root.
- Examples: `spoon/src/config/paths.rs`, `spoon-backend/src/scoop/paths.rs`, `spoon-backend/src/msvc/paths.rs`
- Pattern: Deterministic path derivation; do not hardcode runtime directories outside these modules.

## Entry Points

**Application Binary:**
- Location: `spoon/src/main.rs`
- Triggers: Running `spoon.exe`
- Responsibilities: Parse CLI, initialize logging, set test-mode behavior, resolve root, and dispatch to CLI or TUI.

**CLI Dispatcher:**
- Location: `spoon/src/cli/run.rs`
- Triggers: Any non-empty subcommand from `Cli`
- Responsibilities: Command routing, JSON/text output selection, config subcommand mutation/reapply, and domain service invocation.

**TUI Runner:**
- Location: `spoon/src/tui/mod.rs`
- Triggers: Launching `spoon.exe` without a subcommand
- Responsibilities: Alternate-screen lifecycle, app loop, input polling, rendering, transitions, and background refresh orchestration.

**Backend Crate Surface:**
- Location: `spoon-backend/src/lib.rs`
- Triggers: Calls from `spoon/src/service/`
- Responsibilities: Re-export backend domains, event/cancellation primitives, and shared error/result types.

**Deploy Utility:**
- Location: `xtask/src/main.rs`
- Triggers: `cargo run -p xtask -- deploy`
- Responsibilities: Build `spoon`, replace `spoon.exe`, and clean transient deploy artifacts.

## Error Handling

**Strategy:** Application code uses `anyhow::Result` for orchestration, while backend code uses typed backend errors and converts them at the service boundary.

**Patterns:**
- `spoon/src/service/mod.rs` provides `backend_to_anyhow` so `spoon-backend::Result<T>` becomes `anyhow::Result<T>` only at the adapter boundary.
- Long-running operations return typed outcome structs such as `CommandResult`, `ScoopPackageOperationOutcome`, and `MsvcOperationOutcome` instead of relying on stdout parsing.
- CLI fallback errors are rendered in `spoon/src/main.rs`; JSON mode wraps errors through `spoon/src/cli/json.rs`.
- Cancellation is explicit through `CancellationToken` in `spoon-backend/src/task.rs` and TUI background actions in `spoon/src/tui/action_flow/tools.rs`.

## Cross-Cutting Concerns

**Logging:** `spoon/src/logger/mod.rs` configures tracing for file logs, buffered stdout logs, and the TUI logger widget.

**Validation:** `spoon/src/cli/args.rs` handles command-shape validation, `spoon/src/config/io.rs` normalizes persisted config, `spoon/src/status/policy.rs` controls legal actions, and runtime/toolchain validation lives in `spoon-backend/src/msvc/validation.rs`.

**Authentication:** No server-style auth layer is present. Tool credentials are treated as owned config artifacts through `spoon/src/config/io.rs` and exposed to external CLIs via their native config files such as `~/.claude/settings.json` and `~/.codex/auth.json`.

---

*Architecture analysis: 2026-03-28*
