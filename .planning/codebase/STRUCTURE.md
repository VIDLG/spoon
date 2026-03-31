# Codebase Structure

**Analysis Date:** 2026-03-28

## Directory Layout

```text
[project-root]/
+-- spoon/                  # Runnable application crate: CLI, TUI, config, app-owned adapters
|   +-- src/
|   |   +-- cli/            # Clap types and CLI dispatch/output
|   |   +-- tui/            # Ratatui state machine, input handling, rendering, harness
|   |   +-- service/        # Active app-to-backend adapter layer
|   |   +-- packages/       # Tool registry and package-specific config/integration logic
|   |   +-- status/         # Tool probing, ownership, readiness, update summary
|   |   +-- config/         # Config models, file IO, path derivation, test-mode state
|   |   +-- actions/        # Tool action orchestration shared by CLI and TUI
|   |   +-- view/           # Display models for config and tool detail/status pages
|   |   +-- editor/         # Editor discovery, install/uninstall, launch helpers
|   |   +-- logger/         # Tracing setup and event helpers
|   |   `-- platform/       # OS-specific shell/process helpers
|   `-- tests/              # CLI and TUI flow tests plus shared test utilities
+-- spoon-backend/          # Reusable Scoop/MSVC backend crate
|   +-- src/
|   |   +-- scoop/          # Scoop query, bucket, manifest, planner, runtime logic
|   |   +-- msvc/           # Managed and official MSVC runtime logic
|   |   `-- tests/          # Unit tests for backend primitives
|   `-- tests/              # Opt-in integration-style backend tests
+-- xtask/                  # Maintenance-only build/deploy helper crate
+-- contrib/                # Vendored/reference upstream projects and supporting assets
+-- skills/                 # Local AI/bootstrap skills consumed by Spoon configuration flows
+-- scripts/                # Repo helper scripts not used as the primary product entrypoint
+-- .planning/codebase/     # Generated mapper documents
`-- spoon.exe               # Repository-root built artifact
```

## Directory Purposes

**`spoon/src/cli/`:**
- Purpose: Own the command-line interface.
- Contains: `args.rs`, `run.rs`, `json.rs`, `messages.rs`, `output.rs`, `response.rs`
- Key files: `spoon/src/cli/args.rs`, `spoon/src/cli/run.rs`

**`spoon/src/tui/`:**
- Purpose: Own the interactive Ratatui application.
- Contains: App state, modal/screen transitions, key handling, background workers, rendering, and the harness.
- Key files: `spoon/src/tui/mod.rs`, `spoon/src/tui/state.rs`, `spoon/src/tui/background.rs`, `spoon/src/tui/test_support.rs`

**`spoon/src/service/`:**
- Purpose: Serve as the live adapter boundary to `spoon-backend`.
- Contains: Cache helpers, Scoop adapters, MSVC adapters, backend result/event mapping.
- Key files: `spoon/src/service/mod.rs`, `spoon/src/service/scoop/mod.rs`, `spoon/src/service/scoop/runtime.rs`, `spoon/src/service/msvc/mod.rs`

**`spoon/src/services/`:**
- Purpose: Empty placeholder directory.
- Contains: No Rust source files.
- Key files: Not applicable

**`spoon/src/packages/`:**
- Purpose: Define the managed tool catalog plus per-package config/integration behavior.
- Contains: `PackageSpec` implementations such as `spoon/src/packages/git.rs`, `spoon/src/packages/msvc.rs`, `spoon/src/packages/claude.rs`, `spoon/src/packages/codex.rs`, `spoon/src/packages/python.rs`
- Key files: `spoon/src/packages/mod.rs`, `spoon/src/packages/tool.rs`

**`spoon/src/status/`:**
- Purpose: Probe runtime state and derive actionability.
- Contains: Discovery probes, ownership/readiness logic, update summary generation, and formatted status output.
- Key files: `spoon/src/status/mod.rs`, `spoon/src/status/discovery/probe.rs`, `spoon/src/status/policy.rs`

**`spoon/src/config/`:**
- Purpose: Centralize persisted config models, file IO, and managed root path derivation.
- Contains: Config structs, file readers/writers, path helpers, test-mode home override state.
- Key files: `spoon/src/config/model.rs`, `spoon/src/config/io.rs`, `spoon/src/config/paths.rs`, `spoon/src/config/state.rs`

**`spoon/src/actions/`:**
- Purpose: Convert a selected `ToolAction` into backend/native execution.
- Contains: Backend partitioning, result flattening/summarizing, action enums.
- Key files: `spoon/src/actions/mod.rs`, `spoon/src/actions/execute/mod.rs`

**`spoon/src/view/`:**
- Purpose: Build presentation models independent of the renderers.
- Contains: Tool detail/status rows and config view models.
- Key files: `spoon/src/view/config.rs`, `spoon/src/view/tools/detail.rs`, `spoon/src/view/tools/row.rs`

**`spoon/src/editor/`:**
- Purpose: Manage editor discovery and editor-specific launch/install behavior.
- Contains: Candidate discovery, apply/install/uninstall flows, default-editor resolution.
- Key files: `spoon/src/editor/discovery.rs`, `spoon/src/editor/manage.rs`, `spoon/src/editor/launch.rs`

**`spoon/src/logger/`:**
- Purpose: Centralize tracing setup and app/TUI event helpers.
- Contains: Logger settings plus event-specific helper modules.
- Key files: `spoon/src/logger/mod.rs`, `spoon/src/logger/events/`

**`spoon-backend/src/scoop/`:**
- Purpose: Own generic Scoop runtime logic that must stay reusable across frontends.
- Contains: Bucket registry operations, query/report data, manifest parsing, action planning, runtime execution helpers.
- Key files: `spoon-backend/src/scoop/mod.rs`, `spoon-backend/src/scoop/planner.rs`, `spoon-backend/src/scoop/query.rs`, `spoon-backend/src/scoop/runtime/`

**`spoon-backend/src/msvc/`:**
- Purpose: Own generic managed/official MSVC runtime logic.
- Contains: Runtime config, manifest handling, package rules, status reporting, wrapper generation, validation, and official installer support.
- Key files: `spoon-backend/src/msvc/mod.rs`, `spoon-backend/src/msvc/status.rs`, `spoon-backend/src/msvc/official.rs`, `spoon-backend/src/msvc/wrappers.rs`

**`spoon-backend/src/tests/`, `spoon-backend/src/scoop/tests/`, `spoon-backend/src/msvc/tests/`:**
- Purpose: Keep backend-focused unit tests close to the modules they verify.
- Contains: Internal module tests for events, tasks, buckets, runtime helpers, and MSVC logic.
- Key files: `spoon-backend/src/tests/task.rs`, `spoon-backend/src/scoop/tests/runtime.rs`, `spoon-backend/src/msvc/tests/root.rs`

**`spoon/tests/`:**
- Purpose: Validate app flows.
- Contains: CLI flow tests, TUI flow tests, and shared test support.
- Key files: `spoon/tests/common/`, `spoon/tests/cli/`, `spoon/tests/tui/`

**`xtask/src/`:**
- Purpose: Hold maintenance-only developer automation.
- Contains: A single deploy-oriented binary.
- Key files: `xtask/src/main.rs`

## Key File Locations

**Entry Points:**
- `spoon/src/main.rs`: Runtime application entrypoint.
- `spoon/src/lib.rs`: App crate module surface and backend re-export.
- `spoon-backend/src/lib.rs`: Backend crate public surface.
- `xtask/src/main.rs`: Deploy/build helper entrypoint.

**Configuration:**
- `spoon/src/config/io.rs`: File loading/saving for Spoon-owned config.
- `spoon/src/config/paths.rs`: Canonical managed-root path derivation.
- `spoon/src/packages/mod.rs`: Package config registry and config-target descriptors.

**Core Logic:**
- `spoon/src/cli/run.rs`: CLI command routing.
- `spoon/src/tui/state.rs`: Central TUI state model.
- `spoon/src/actions/execute/mod.rs`: Shared tool action executor.
- `spoon/src/service/scoop/actions.rs`: Scoop action adapter.
- `spoon/src/service/msvc/mod.rs`: MSVC adapter and runtime-config injection.
- `spoon-backend/src/scoop/runtime/actions.rs`: Scoop runtime execution pipeline.
- `spoon-backend/src/msvc/mod.rs`: Managed MSVC runtime pipeline.

**Testing:**
- `spoon/src/tui/test_support.rs`: TUI harness used by flow tests.
- `spoon/tests/tui/`: Ratatui flow coverage.
- `spoon/tests/cli/`: CLI coverage.
- `spoon-backend/tests/`: External integration-style backend coverage.

## Naming Conventions

**Files:**
- Use `snake_case.rs` for Rust modules, for example `spoon/src/status/discovery/probe.rs` and `spoon-backend/src/scoop/runtime/execution.rs`.
- Use `mod.rs` to define directory-backed modules such as `spoon/src/tui/mod.rs` and `spoon-backend/src/scoop/mod.rs`.
- Use domain-specific filenames for package specs, for example `spoon/src/packages/git.rs` and `spoon/src/packages/codex.rs`.

**Directories:**
- Use domain/layer names rather than feature tickets, for example `spoon/src/service/`, `spoon/src/status/`, `spoon/src/view/`, `spoon-backend/src/msvc/`.
- Keep backend reusable concerns under `spoon-backend/src/` and app-specific glue under `spoon/src/`.
- Prefer the singular live directory `spoon/src/service/`; do not place new code in the empty placeholder `spoon/src/services/`.

## Where to Add New Code

**New CLI Command Behavior:**
- Primary code: `spoon/src/cli/run.rs`
- Supporting types: `spoon/src/cli/args.rs`, `spoon/src/cli/json.rs`, `spoon/src/cli/messages.rs`
- Tests: `spoon/tests/cli/`

**New TUI Screen, Modal, or Interaction:**
- Primary code: `spoon/src/tui/state.rs`, `spoon/src/tui/action_flow/`, `spoon/src/tui/keys/`, `spoon/src/tui/render/`
- Display models: `spoon/src/view/`
- Tests: `spoon/tests/tui/` and, when reusable, `spoon/src/tui/test_support.rs`

**New Managed Tool or Package Policy:**
- Registry entry: `spoon/src/packages/mod.rs` and `spoon/src/packages/tool.rs`
- Package-specific behavior: a new or updated module in `spoon/src/packages/`
- Status integration: `spoon/src/status/` if ownership/probing rules change
- Tests: `spoon/tests/cli/`, `spoon/tests/tui/`, and targeted unit tests beside the affected package module

**New App-to-Backend Adapter:**
- Implementation: `spoon/src/service/`
- Use when: The logic depends on app config, UI progress mapping, PATH mutation, or package integrations.
- Do not add reusable backend logic here if it can live in `spoon-backend/src/`.

**New Reusable Scoop Logic:**
- Implementation: `spoon-backend/src/scoop/`
- Runtime execution helpers: `spoon-backend/src/scoop/runtime/`
- Query/report structures: `spoon-backend/src/scoop/query.rs`, `spoon-backend/src/scoop/info.rs`
- Tests: `spoon-backend/src/scoop/tests/` or `spoon-backend/tests/`

**New Reusable MSVC Logic:**
- Implementation: `spoon-backend/src/msvc/`
- Path rules: `spoon-backend/src/msvc/paths.rs`
- Wrapper/validation behavior: `spoon-backend/src/msvc/wrappers.rs`, `spoon-backend/src/msvc/validation.rs`
- Tests: `spoon-backend/src/msvc/tests/` or `spoon-backend/tests/`

**Utilities:**
- Shared app helpers: Place with the owning domain first, for example `spoon/src/config/`, `spoon/src/editor/`, or `spoon/src/status/` before creating a generic misc module.
- Shared backend helpers: Place in `spoon-backend/src/` alongside the owning backend domain.

## Special Directories

**`contrib/`:**
- Purpose: Reference or vendored upstream projects such as `contrib/Scoop/`, `contrib/hok/`, and `contrib/msvcup/`.
- Generated: No
- Committed: Yes

**`target/`:**
- Purpose: Cargo build output.
- Generated: Yes
- Committed: No

**`.planning/codebase/`:**
- Purpose: Generated architecture/stack/quality/concern mapper docs for later planning commands.
- Generated: Yes
- Committed: Usually yes within the GSD workflow

**`skills/`:**
- Purpose: Local AI/bootstrap skill artifacts referenced by Spoon-owned config flows.
- Generated: No
- Committed: Yes

**`tmp/`:**
- Purpose: Workspace scratch area containing transient or reference material such as `tmp/msvcup-rs/`.
- Generated: Mixed
- Committed: Yes

---

*Structure analysis: 2026-03-28*
