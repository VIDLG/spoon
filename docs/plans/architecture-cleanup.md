# Plan: Architecture Cleanup

> Source PRD: docs/prd/architecture-cleanup.md

## Architectural decisions

Durable decisions that apply across all phases:

- **Module visibility**: New submodules use `pub(crate)` visibility. External access only through `mod.rs` re-exports. No submodules exposed as `pub mod`.
- **Event pattern**: Standardize on `EventSender` (tokio broadcast). Caller creates the event bus and decides capacity. Domain crates accept `Option<&EventSender>`.
- **Bridge layer**: `service/` becomes `bridge/` — shared glue between UI consumers (CLI, TUI) and domain crates. Contains types (CommandResult, StreamChunk), ScoopPorts impl, event formatting, and a shared `execute_scoop_action()` that returns `(Outcome, Receiver)`.
- **No public API changes**: All refactoring is internal. External consumers see identical types and function paths.
- **Execution order**: Strictly sequential 0→1→2→3→4. Each phase is one commit with `cargo test` passing.

---

## Phase 0: Cleanup — deduplicate, fix proxy bug, remove dead code

**User stories**: 0-5 (MsvcRequest builder, format_bytes, native_msvc_arch, dead deps, tool.rs, normalize_proxy_url)

### What to build

Fix the MsvcRequest proxy propagation bug by adding builder methods. Consolidate 3 implementations of `format_bytes` (2 duplicates + 1 canonical in main crates; xtask copy excluded) and 4 distinct implementations of `native_msvc_arch` to single implementations. Remove unused `gix` dependency from spoon-msvc. Delete the one-line `tool.rs` re-export and the duplicate `normalize_proxy_url`. All changes are localized to individual files with no structural changes.

### Steps

1. **MsvcRequest builder** (spoon-msvc/src/types.rs)
   - Add `pub fn proxy(mut self, proxy: impl Into<String>) -> Self`
   - Add `pub fn command_profile(mut self, profile: impl Into<String>) -> Self`
   - Add `pub fn test_mode(mut self, enabled: bool) -> Self`
   - Keep `for_tool_root()` as minimal constructor
   - Update 16 call sites in spoon/src/service/msvc/mod.rs to read config and pass proxy via builder

2. **Consolidate format_bytes** 
   - Copy the tested version from spoon/src/formatting.rs to spoon-core/src/ (new module or append to existing)
   - Add `pub use` in spoon-core/src/lib.rs
   - Replace uses in spoon-msvc/src/execute.rs and spoon-msvc/src/platform/msvc_runtime.rs with `spoon_core::format_bytes`
   - Update spoon/src/formatting.rs to re-export from spoon-core
   - Delete the local implementations

3. **Consolidate native_msvc_arch**
   - Keep `spoon-msvc/src/paths.rs` version as canonical (uses `std::env::consts::ARCH`, supports test overrides)
   - Delete 3 non-canonical implementations: `types.rs`, `platform/msvc_paths.rs`, `spoon/src/config/paths.rs`
   - Files that already delegate (`platform/msvc_runtime.rs`, `execute.rs`) just need import verification
   - Replace all internal uses with `crate::paths::native_msvc_arch()`
   - Update `spoon/src/config/paths.rs` to delegate to `spoon_msvc::paths::native_msvc_arch`

4. **Remove dead dependencies**
   - Remove `gix` from spoon-msvc/Cargo.toml
   - Check if sha1/sha2/walkdir in spoon/Cargo.toml are used directly or only through transitive deps — remove direct deps if unused

5. **Delete tool.rs re-export**
   - Delete spoon/src/tool.rs
   - Update all `use crate::tool::` imports to `use crate::packages::tool::`
   - Remove `mod tool` from spoon/src/lib.rs

6. **Unify normalize_proxy_url**
   - Delete local `normalize_proxy_url` from spoon/src/config/env.rs
   - Replace callers with `spoon_core::normalize_proxy_url`
   - Adjust for signature difference (core version returns `Result<Option<String>>`, app version returned plain `String`)

### Acceptance criteria

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] `MsvcRequest::for_tool_root(tool_root).proxy("http://...").command_profile("msvc")` compiles
- [ ] No duplicate `format_bytes` implementations remain (grep confirms)
- [ ] No duplicate `native_msvc_arch` implementations remain (grep confirms)
- [ ] `gix` removed from spoon-msvc/Cargo.toml
- [ ] `tool.rs` deleted, no `use crate::tool::` imports remain

---

## Phase 1: Split execute.rs into submodules

**User stories**: 6-12 (pipeline, validate, discover, workflow modules; re-exports; path helpers co-located)

### What to build

Convert the 1871-line `execute.rs` file into an `execute/` directory with 4 focused submodules plus a re-export hub. No function signatures change. No external callers need updating.

### Steps

1. **Create execute/ directory structure**
   - Rename `spoon-msvc/src/execute.rs` → create `spoon-msvc/src/execute/mod.rs` (start with empty re-export shell)
   - Create `execute/pipeline.rs`, `execute/integrity.rs`, `execute/validate.rs`, `execute/discover.rs`, `execute/workflow.rs`

2. **Move pipeline functions** → `execute/pipeline.rs`
   - Move the 7 `ensure_*` functions (lines ~360-949)
   - Move 12 path helper functions (`payload_cache_dir`, `extracted_payload_cache_dir`, etc.) (lines ~27-117)
   - Move `copy_tree_into` helper
   - All moved functions keep their `pub` visibility as-is

2b. **Move integrity utilities** → `execute/integrity.rs`
   - Move (`decode_hex_sha256`, `file_sha256`, `read_cached_msi_cab_names`, `archive_kind_for_payload`, `payload_source_description`, `download_progress_target_label`, `download_or_copy_payload`) (lines ~119-358)
   - These are SHA verification, download, and payload description helpers — distinct from the pipeline steps

3. **Move validation** → `execute/validate.rs`
   - Move `validate_toolchain_async` (lines ~1509-1871)
   - Move any validation-only helpers it uses

4. **Move discovery** → `execute/discover.rs`
   - Move `find_preferred_msvc_binary` (lines ~194-226)
   - Move `managed_toolchain_flags_with_request` (lines ~228-337)

5. **Move workflow + state** → `execute/workflow.rs`
   - Move `ToolchainAction` enum and impl (lines ~1140-1160)
   - Move `run_toolchain_action_async` (lines ~1219-1403)
   - Move all public entry points: `install_toolchain_async`, `update_toolchain_async`, streaming variants, `uninstall_toolchain_async` (lines ~1409-1507)
   - Move `format_bytes` (already consolidated to spoon-core in Phase 0, so just import)
   - Move `user_facing_toolchain_label`, `dir_size_bytes`
   - Move state helpers: `write_installed_state`, `write_runtime_state`, `remove_autoenv_dir`, `write_managed_canonical_state`, `managed_toolchain_is_current`, `handle_manifest_refresh_failure`
   - Move materialization: `ensure_materialized_toolchain`, `cleanup_post_install_cache`

6. **Write execute/mod.rs** re-exports
   - `pub(crate) mod pipeline; pub(crate) mod integrity; pub(crate) mod validate; pub(crate) mod discover; pub(crate) mod workflow;`
   - Re-export all public items from submodules
   - Shared private functions that cross module boundaries live here (if any remain after the split)

7. **Verify all callers still compile**
   - `spoon_msvc::execute::install_toolchain_async` should resolve through re-exports
   - `spoon_msvc::official` functions that call into execute should resolve
   - Run `cargo check` and fix any visibility/import issues

### Acceptance criteria

- [ ] `cargo check` passes with no warnings
- [ ] `cargo test` passes
- [ ] No file in `execute/` exceeds 700 lines (pipeline.rs is expected to be ~680 lines; other modules well under)
- [ ] External callers (`service::msvc`, `spoon_msvc::official`) compile without changes
- [ ] `spoon_msvc::execute::install_toolchain_async` still resolves at call sites
- [ ] Original `execute.rs` single file no longer exists

---

## Phase 2: Flatten spoon-scoop directory structure

**User stories**: 13-18 (flat src/, no core/, no runtime/, explicit module declarations, preserved public API)

### What to build

Remove the `core/` and `runtime/` subdirectories from spoon-scoop. Move all `.rs` files directly into `src/`. Update `lib.rs` to declare modules explicitly. Public API remains flat at crate root.

### Steps

1. **Move core/ files to src/**
   - Move `src/core/bucket.rs` → `src/bucket.rs`
   - Move `src/core/bucket_ops.rs` → `src/bucket_ops.rs`
   - Move `src/core/manifest.rs` → `src/manifest.rs`
   - Move `src/core/ports.rs` → `src/ports.rs`
   - Move `src/core/state.rs` → `src/state.rs`
   - Move `src/core/source.rs` → `src/source.rs`
   - Move `src/core/response.rs` → `src/response.rs`
   - Move `src/core/helpers.rs` → `src/helpers.rs`
   - Move `src/core/workflow.rs` → `src/workflow.rs`
   - Delete `src/core/mod.rs` and `src/core/` directory

2. **Move runtime/ files to src/**
   - Move `src/runtime/queries.rs` → `src/queries.rs`
   - Delete `src/runtime/mod.rs` and `src/runtime/` directory

3. **Rewrite lib.rs**
   - Replace `mod core; pub mod runtime;` with:
     ```
     mod bucket_ops;
     mod helpers;
     mod queries;
     
     pub mod bucket;
     pub mod manifest;
     pub mod ports;
     pub mod state;
     pub mod source;
     pub mod response;
     pub mod workflow;
     ```
   - Keep `pub use` glob re-exports for flat public API
   - Keep `pub use error::{ScoopError, Result};`

4. **Fix internal imports**
   - Files that used `use super::` to reference core siblings should still work (they're now actual siblings)
   - Files that used `crate::core::` need updating to `crate::`
   - Run `cargo check` and fix any broken paths

### Acceptance criteria

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No `core/` or `runtime/` directory exists in spoon-scoop/src/
- [ ] `spoon_scoop::ScoopManifest`, `spoon_scoop::install_package`, `spoon_scoop::search_results` still resolve at call sites
- [ ] `src/lib.rs` has explicit `mod bucket; mod manifest;` etc. instead of `mod core;`

---

## Phase 3: Unify event bridge, eliminate non-streaming, remove lines accumulation

**User stories**: 19-23 (EventSender everywhere, caller-owned event bus, channel-based adapter, multi-consumer support)

### What to build

Unify both domain crates on `Option<&EventSender>` pattern. Eliminate all non-streaming function variants and the service-layer `FnMut(StreamChunk)` pattern. Remove `push_stream_line` and manual `lines: Vec<String>` accumulation — all output goes through EventSender. Rename `_streaming` functions to drop the suffix. Remove `output: Vec<String>` and `streamed: bool` from ALL outcome types (`CommandResult`, `MsvcOperationOutcome`, `ScoopPackageOperationOutcome`, `ScoopBucketOperationOutcome`, `ScoopPackageActionOutcome`, `ScoopPackageManifestOutcome`). Callers who need collected output drain it from their own event receiver.

**Scope note**: This phase touches 25+ `.output`/`.streamed` consumers across the entire spoon crate, and ~35 `push_stream_line` call sites across `execute.rs` (~20) and `official.rs` (~15). The `official.rs` file (1421 lines) is not pre-split but receives heavy modifications — accepted risk due to its internal cohesiveness. Removing `output`/`streamed` from `CommandResult` changes the JSON output schema in `cli/json.rs` — accepted as the JSON format was never a stable API.

### Steps

1. **Replace push_stream_line with emit_notice** (spoon-msvc/src/common.rs)
   - Delete `push_stream_line` function
   - Add `emit_notice(emit: Option<&EventSender>, message: &str)` — sends `SpoonEvent::Notice` if sender present, always does `tracing::info!`
   - All callers of `push_stream_line` updated to use `emit_notice` (no lines accumulation)

2. **Eliminate lines Vec accumulation** (spoon-msvc/src/execute/*, official.rs)
   - Remove `lines: &mut Vec<String>` parameter from all internal functions
   - Remove `let mut lines = vec![];` from all workflow functions
   - All output messaging goes through `emit_notice(sender, msg)` only
   - Outcome types no longer carry output — see step 3 for field removal

3. **Remove output/streamed from ALL outcome types**
   - Remove `output: Vec<String>` and `streamed: bool` from `CommandResult` — becomes `{title, status}` only
   - Remove `output: Vec<String>` and `streamed: bool` from `MsvcOperationOutcome` — becomes `{title, status, ...domain_fields}` only
   - Remove `output: Vec<String>` and `streamed: bool` from `ScoopPackageOperationOutcome`
   - Remove `output: Vec<String>` and `streamed: bool` from `ScoopBucketOperationOutcome`
   - Remove `output: Vec<String>` and `streamed: bool` from `ScoopPackageActionOutcome`
   - Remove `streamed: bool` from `ScoopPackageManifestOutcome` (no `output` field on this type)
   - Callers who need collected output drain it from their own event receiver (not from the result struct)
   - Update `command_result()` helper in service/scoop/mod.rs — no longer accepts `output`/`streamed` params

4. **Delete non-streaming functions, rename streaming** (spoon-msvc)
   - Delete from execute/workflow.rs: `install_toolchain_async`, `update_toolchain_async` (the non-streaming wrappers)
   - Delete from official.rs: `install_toolchain_async`, `update_toolchain_async`, `uninstall_toolchain_async`
   - Rename `install_toolchain_streaming` → `install_toolchain`
   - Rename `update_toolchain_streaming` → `update_toolchain`
   - Rename `uninstall_toolchain_streaming` → `uninstall_toolchain`
   - Same renames in official.rs

5. **Update spoon/ callers** — all `.output`/`.streamed` consumers and `FnMut(StreamChunk)` patterns
   - **Domain API renames**: Remove `install_toolchain_async`, `update_toolchain_async` wrappers from service/msvc/mod.rs. Remove `install_tools`/`update_tools`/`uninstall_tools` (non-streaming) from service/scoop/actions.rs. Rename all `_streaming` functions to drop suffix.
   - **Remove `forward_backend_event_to_stream`** from service/mod.rs
   - **service/msvc/mod.rs**: Each function creates `event_bus(64)`, passes `Some(&tx)`, drains receiver for display. Remove `FnMut(StreamChunk)` from `reapply_managed_command_surface_streaming`.
   - **actions/execute/native.rs**: Create event_bus, call domain function, collect events from receiver instead of reading `.output`
   - **actions/execute/scoop.rs**: Same pattern — replace `.output` reads with event receiver drain
   - **actions/format.rs**: Remove `.output` reads from result display formatting
   - **cli/run.rs**: Update function names (remove `_streaming` suffix). Replace `print_lines(&result.output)` with event-driven output
   - **cli/json.rs**: Remove `output` and `streamed` from JSON serialization — JSON output becomes `{ title, status }` only
   - **editor/manage.rs**: Stop constructing `CommandResult` with `output`/`streamed`. Use event_bus for output instead
   - **logger/events/command.rs**: Remove `.output` iteration for logging — log from event receiver instead
   - **service/cache.rs**: Remove `.output.clone()` reference — cache results no longer carry output
   - **service/scoop/actions.rs**: Remove all `.output` forwarding, `FnMut(StreamChunk)` closures, and `command_result_from_scoop_package_outcome`
   - **service/scoop/bucket.rs**: Remove `.output`/`.streamed` from outcome construction, remove `FnMut(StreamChunk)` from `bucket_update_streaming`
   - **service/scoop/mod.rs**: Update `command_result()` helper to take only `title` and `status`. Remove `command_result_from_scoop_package_outcome` — construct CommandResult directly from `{title, status}`

6. **Verify**
   - `cargo check` passes
   - `cargo test` passes
   - No `FnMut(SpoonEvent)` pattern remains anywhere
   - No `FnMut(StreamChunk)` pattern remains in the spoon crate
   - No `push_stream_line` function exists
   - No `_streaming` suffix on function names (all are the primary name)
   - No `_async` suffix variants coexisting with streaming versions
   - No `.output` or `.streamed` field reads on CommandResult or outcome types

### Acceptance criteria

- [ ] `cargo check` passes with no warnings
- [ ] `cargo test` passes
- [ ] No `FnMut(SpoonEvent)` pattern remains in spoon-msvc or spoon-scoop public API
- [ ] No `FnMut(StreamChunk)` pattern remains in the spoon crate
- [ ] No `push_stream_line` function exists
- [ ] No `lines: &mut Vec<String>` parameter in internal functions
- [ ] No `_streaming` or `_async` suffix variants — each operation has one canonical function
- [ ] `CommandResult` has only `title` and `status` fields (no `output`, no `streamed`)
- [ ] All outcome types (`MsvcOperationOutcome`, `ScoopPackageOperationOutcome`, `ScoopBucketOperationOutcome`, `ScoopPackageActionOutcome`, `ScoopPackageManifestOutcome`) have no `output` or `streamed` fields
- [ ] CLI creates event_bus and passes sender to domain functions
- [ ] `cli/json.rs` JSON output is `{ title, status }` only
- [ ] `editor/manage.rs` uses event_bus instead of constructing CommandResult with output
- [ ] `logger/events/command.rs` logs from event receiver instead of `.output`
- [ ] `service/cache.rs` no longer references `.output` on CommandResult

---

## Phase 4: Flatten service → bridge

**User stories**: 24-30 (direct domain calls, bridge/mod.rs as shared layer, cache preserved, reports to cli/, scoop runtime to cli/, service→bridge rename)

### What to build

Remove the thin service delegation layer. CLI commands call domain crates directly. `service/mod.rs` keeps shared types (`CommandResult { title, status }`, StreamChunk, AppSystemPort, event formatting) and gains `execute_scoop_action()`. `service/cache.rs` keeps its real filesystem logic (`.output`/`.streamed` references already removed in Phase 3). Reports move to `cli/report.rs`. Scoop runtime bridge moves to `cli/scoop_runtime.rs`. Finally rename `service/` to `bridge/`, updating imports across all subsystems (cli/, actions/, editor/, status/, view/, logger/, tui/).

### Steps

1. **Move reports to cli/report.rs**
   - Create `spoon/src/cli/report.rs`
   - Move formatting functions from `service/msvc/report.rs` (85 lines) and `service/scoop/report.rs` (299 lines)
   - Organize as `pub mod msvc { ... }` and `pub mod scoop { ... }` or flat functions with prefixes
   - Update callers in cli/run.rs to use `crate::cli::report::*`

2. **Move scoop runtime to cli/scoop_runtime.rs**
   - Create `spoon/src/cli/scoop_runtime.rs`
   - Move `execute_package_action_outcome_streaming` and the reapply functions from `service/scoop/runtime.rs`
   - This file creates event_bus, builds HTTP client, calls spoon-scoop workflows, drains receiver

3. **Add execute_scoop_action to service/mod.rs**
   - Extract the shared pattern: create event_bus + call spoon-scoop + return (Outcome, Receiver)
   - This is the future TUI entry point
   - cli/scoop_runtime.rs calls this and adds CLI-specific event consumption

4. **Remove thin wrappers**
   - Delete `service/msvc/` directory entirely (19 wrappers + report.rs already moved)
   - Delete `service/scoop/actions.rs` (7 RunMode dispatchers — logic inlined into CLI handlers)
   - Delete `service/scoop/bucket.rs` (bucket operations inlined into CLI handlers)
   - Delete `service/scoop/mod.rs` (re-exports no longer needed)
   - Update cli/run.rs to call domain crates directly:
     - MSVC: `let req = MsvcRequest::for_tool_root(&root).proxy(proxy); spoon_msvc::execute::install_toolchain(&req).await`
     - Scoop: `cli/scoop_runtime::execute_package_action(...)` or `spoon_scoop::install_package(...)`

5. **Rename service/ to bridge/**
   - Rename directory `spoon/src/service/` → `spoon/src/bridge/`
   - Rename `mod service` → `mod bridge` in spoon/src/lib.rs
   - Global find-replace `crate::service::` → `crate::bridge::` across ALL subsystems:
     - `cli/` (run.rs, json.rs, output.rs)
     - `actions/` (format.rs, execute/mod.rs, execute/native.rs, execute/scoop.rs)
     - `editor/` (manage.rs)
     - `packages/` (python.rs — uses `service::scoop::runtime::resolved_pip_mirror_url_for_display`)
     - `status/` (mod.rs, update.rs, discovery/probe.rs)
     - `view/` (tools/detail.rs)
     - `logger/` (events/command.rs)
     - `tui/` (state.rs, test_support.rs, action_flow/*.rs)

6. **Verify**
   - `cargo check` passes
   - `cargo test` passes
   - No `crate::service::` references remain

### Acceptance criteria

- [ ] `cargo check` passes with no warnings
- [ ] `cargo test` passes
- [ ] `service/` directory no longer exists
- [ ] `bridge/mod.rs` contains: `CommandResult { title, status }`, StreamChunk, AppSystemPort, stream_chunk_from_event, execute_scoop_action
- [ ] `bridge/cache.rs` exists (`.output`/`.streamed` references already removed in Phase 3, position unchanged)
- [ ] `cli/report.rs` contains all report formatting functions
- [ ] `cli/scoop_runtime.rs` contains the event bus + HTTP client bridge
- [ ] cli/run.rs calls spoon_msvc and spoon_scoop directly (no intermediate delegation)
- [ ] No `crate::service::` references remain in codebase (grep confirms)
