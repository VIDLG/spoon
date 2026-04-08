## Problem Statement

After splitting `spoon-backend` into `spoon-core`, `spoon-scoop`, and `spoon-msvc`, the codebase carries leftover indirection. The `service/` layer is ~30% thin delegation wrappers (19/23 MSVC functions are one-liners). `execute.rs` is a 1871-line monolith with 6 distinct logical groupings. `spoon-scoop` has unnecessary directory nesting (`core/`, `runtime/`) that glob-re-exports flat to the crate root anyway. Two different event bridging patterns exist for the same job (closures vs broadcast channels).

This plan consolidates five phases of architecture improvements into a bottom-up execution sequence, preceded by a cleanup phase that fixes known bugs and removes dead code.

## Solution

0. **Phase 0: Cleanup** — fix MsvcRequest proxy bug, deduplicate utilities, remove dead dependencies, small cleanups
1. **Phase 1: Split `spoon-msvc/src/execute.rs`** into focused submodules under `execute/`
2. **Phase 2: Flatten `spoon-scoop`** by removing `core/` and `runtime/` directories, placing all modules directly in `src/`
3. **Phase 3: Unify the event bridge** by standardizing on the channel-based `EventSender` pattern across both domain crates
4. **Phase 4: Flatten and rename `service/` to `bridge/`**, having CLI commands call domain crates directly while keeping shared types and the execution bridge layer

## User Stories

### Phase 0: Cleanup

0. As a developer, I want `MsvcRequest` to support builder-pattern configuration (`.proxy()`, `.command_profile()`, `.test_mode()`), so that proxy settings from config propagate to MSVC operations
1. As a developer, I want `format_bytes` consolidated to one implementation in spoon-core, so that 3 duplicate copies are removed
2. As a developer, I want `native_msvc_arch` consolidated to one implementation in spoon-msvc/paths, so that 4 distinct implementations with divergent strategies (paths.rs, types.rs, platform/msvc_paths.rs, config/paths.rs) are unified
3. As a developer, I want the unused `gix` dependency removed from spoon-msvc/Cargo.toml, so that compile time is reduced
4. As a developer, I want `tool.rs` one-line re-export removed, so that imports use one canonical path
5. As a developer, I want `normalize_proxy_url` in spoon/config/env.rs replaced with the spoon-core version, so that edge cases are handled consistently

### Phase 1: Split execute.rs

1. As a developer, I want `execute.rs` split into modules under 400 lines each, so that I can navigate the codebase without scrolling through 1871 lines
2. As a developer, I want cache pipeline functions (`ensure_cached_payloads` through `ensure_install_image`) in their own module, so that the install pipeline is self-contained
3. As a developer, I want `validate_toolchain_async` (363 lines) in its own module, so that validation logic is isolated from install/update logic
4. As a developer, I want `find_preferred_msvc_binary` and `managed_toolchain_flags_with_request` in a discover module, so that toolchain discovery is reusable across workflows and validation
5. As a developer, I want the top-level entry points (`install_toolchain_async`, `update_toolchain_async`, etc.) in a workflow module, so that the orchestration layer is distinct from the pipeline steps
6. As a developer, I want `execute/mod.rs` to re-export all public items, so that existing callers don't change
7. As a developer, I want path helper functions (12 private functions) to live alongside the pipeline that uses them, so that related code is co-located

### Phase 2: Flatten spoon-scoop

8. As a developer, I want all `spoon-scoop` source files directly under `src/`, so that I don't navigate through unnecessary directory nesting
9. As a developer, I want `runtime/queries.rs` merged into the flat structure as `queries.rs`, so that the 3-line `runtime/mod.rs` delegation is eliminated
10. As a developer, I want `core/` directory removed and its contents moved to `src/`, so that there's one less level of indirection
11. As a developer, I want `lib.rs` to use `mod bucket; mod manifest; mod queries;` etc. instead of `mod core; pub mod runtime;`, so that module declarations are explicit
12. As a developer, I want the public API of `spoon-scoop` to remain unchanged (all types/functions accessible at crate root), so that consumers don't need updates
13. As a developer, I want `error.rs` and `tests.rs` to remain at their current `src/` level, so that crate-level files stay where they are

### Phase 3: Unify event bridge

14. As a developer, I want both domain crates to use the same event emission mechanism, so that I don't need to understand two different patterns
15. As a developer, I want `spoon-msvc` to accept `Option<&EventSender>` instead of `&mut dyn FnMut(SpoonEvent)`, so that it matches the `spoon-scoop` pattern
16. As a developer, I want the caller to create the event bus and decide capacity, so that domain crates don't control runtime concerns
17. As a developer, I want `forward_backend_event_to_stream` replaced with a channel-based adapter, so that there's one bridge mechanism
18. As a developer, I want the event bus to support multiple consumers, so that a future TUI can subscribe to events alongside the CLI formatter

### Phase 4: Flatten service → bridge

19. As a developer, I want CLI commands to call `spoon_msvc` and `spoon-scoop` directly, so that the thin service delegation layer is eliminated
20. As a developer, I want `CommandResult`, `StreamChunk`, `stream_chunk_from_event`, and `AppSystemPort` kept in `bridge/mod.rs`, so that CLI and TUI share the type bridge layer
21. As a developer, I want `bridge/mod.rs` to provide `execute_scoop_action() -> (Outcome, Receiver)`, so that both CLI and TUI can consume events differently
22. As a developer, I want `bridge/cache.rs` preserved as-is since it contains real business logic
23. As a developer, I want report formatting moved to `cli/report.rs`, so that display logic lives with CLI
24. As a developer, I want the scoop execution bridge (event bus + HTTP client orchestration) in `cli/scoop_runtime.rs`, so that CLI-specific event consumption is isolated
25. As a developer, I want `service/` renamed to `bridge/` as the final step, so that the module name reflects its actual role
26. As a developer, I want existing integration tests to continue passing after each phase

## Implementation Decisions

### Phase 0: Cleanup

- **MsvcRequest builder**: Add `.proxy(String)`, `.command_profile(String)`, `.test_mode(bool)` methods to `MsvcRequest`. Keep `for_tool_root()` as the minimal constructor (tests/simple cases). Update the 16 call sites in `service/msvc/mod.rs` to read config and pass values through the builder.
- **format_bytes**: Keep the version in `spoon/src/formatting.rs` (has tests). Move it to `spoon-core/src/formatting.rs` and re-export. Delete copies from `spoon-msvc/src/execute.rs` and `spoon-msvc/src/platform/msvc_runtime.rs`.
- **native_msvc_arch**: Consolidate to `spoon-msvc/src/paths.rs` using `std::env::consts::ARCH` (runtime, supports test overrides). 4 distinct implementations exist: `paths.rs` (canonical), `types.rs` (uses `cfg!()`), `platform/msvc_paths.rs`, and `spoon/src/config/paths.rs`. The other call sites (`platform/msvc_runtime.rs`, `execute.rs`, `service/mod.rs`) already delegate to other copies. Delete the 3 non-canonical implementations. Update `spoon/src/config/paths.rs` to delegate to `spoon_msvc::paths::native_msvc_arch`.
- **Dead dependencies**: Remove `gix` from `spoon-msvc/Cargo.toml`. Review and remove other unused direct deps from `spoon/Cargo.toml` (sha1, sha2, walkdir if only used through transitive deps).
- **tool.rs**: Delete `spoon/src/tool.rs`, update all `use crate::tool::` imports to `use crate::packages::tool::`.
- **normalize_proxy_url**: Delete `spoon/src/config/env.rs::normalize_proxy_url`, use `spoon_core::normalize_proxy_url` instead.

### Phase 1: Split execute.rs

- **Module structure**: Convert `execute.rs` into `execute/` directory with `pub(crate)` submodules:
  - `execute/mod.rs` — re-exports all public items + shared private functions (`format_bytes`, `user_facing_toolchain_label`, `write_installed_state`, `write_runtime_state`)
  - `execute/pipeline.rs` — the 7 `ensure_*` functions (~590 lines) + path helpers (~90 lines)
  - `execute/integrity.rs` — integrity utilities (SHA/download helpers, ~240 lines)
  - `execute/validate.rs` — `validate_toolchain_async` + validation-specific helpers
  - `execute/discover.rs` — `find_preferred_msvc_binary`, `managed_toolchain_flags_with_request`
  - `execute/workflow.rs` — `ToolchainAction` enum, `run_toolchain_action_async`, public entry points
- **Cross-module dependencies**: `validate.rs` calls `managed_toolchain_flags_with_request` from `discover.rs`. `workflow.rs` calls into `discover.rs` and `pipeline.rs`. Both are fine with `pub(crate)` visibility.
- **Submodule visibility**: All submodules are `pub(crate)`. External access only through `execute/mod.rs` re-exports. Submodule internals are implementation details.
- **No API changes**: All public function signatures remain identical.

### Phase 2: Flatten spoon-scoop

- **Directory removal**: Delete `src/core/` and `src/runtime/`. Move all `.rs` files to `src/`.
- **lib.rs**: Replace `mod core; pub mod runtime;` with explicit per-module declarations.
- **Re-export strategy**: Keep glob re-exports for flat public API at crate root.
- **No API changes**: Consumers continue to use `spoon_scoop::Thing`.

### Phase 3: Unify event bridge, eliminate non-streaming, remove lines accumulation

- **Standardize on EventSender**: Both domain crates use `Option<&EventSender>` for all operations. No `FnMut(SpoonEvent)` patterns remain in domain crates. The service-layer `FnMut(StreamChunk)` pattern is also eliminated — all event consumption goes through EventSender receivers.
- **Caller owns event bus**: CLI/TUI creates event_bus with desired capacity, passes sender to domain functions. Non-streaming callers create internal event_bus and collect output from receiver.
- **Eliminate non-streaming variants**: All `_async` function variants deleted. They were just wrappers passing `None` to streaming versions. Only one canonical function per operation.
- **Drop `_streaming` suffix**: Since streaming is the only mode, rename `install_toolchain_streaming` → `install_toolchain`, etc. No suffix needed.
- **Remove `push_stream_line` and `lines` Vec**: All output goes through EventSender. No manual `lines: &mut Vec<String>` accumulation in internal functions. `emit_notice(sender, msg)` replaces `push_stream_line`. This affects ~35 call sites across `execute.rs` (~20) and `official.rs` (~15).
- **Remove `output` and `streamed` fields**: `CommandResult` becomes `{title, status}` only. No `output: Vec<String>` — all output goes through EventSender events. No `streamed: bool` — streaming is the only mode. Same treatment for ALL outcome types: `MsvcOperationOutcome`, `ScoopPackageOperationOutcome`, `ScoopBucketOperationOutcome`, `ScoopPackageActionOutcome`, `ScoopPackageManifestOutcome` — remove `output` and `streamed` fields from each. Callers who need collected output drain it from their own event receiver. This affects 25+ `.output`/`.streamed` consumers including `cli/json.rs`, `editor/manage.rs`, `logger/events/command.rs`, `actions/format.rs`, `service/cache.rs`.
- **Remove `forward_backend_event_to_stream`**: Replaced by channel-based pattern.
- **`official.rs` risk**: `official.rs` (1421 lines) is not split before Phase 3 but receives heavy modifications (~15 `push_stream_line` call sites, `FnMut(SpoonEvent)` signatures, `output`/`streamed` field removal). Accepted risk — the file is cohesive and bottom-up execution keeps changes localized.
- **JSON output breaking change**: Removing `output`/`streamed` from `CommandResult` changes the JSON output schema in `cli/json.rs`. Accepted — the JSON output format was never a stable API.
- **Bottom-up migration**: Change internal helpers first (`push_stream_line` → `emit_notice`), then `ensure_*` functions, then `run_toolchain_action_async`, then public entry points, then service-layer callers, then CLI consumers. Remove `output`/`streamed` fields last after all callers are updated.

### Phase 4: Flatten service → bridge

- **What stays as `bridge/mod.rs`**: `CommandResult { title, status }`, `StreamChunk`, `PackageRef`, `AppSystemPort`, `stream_chunk_from_event`, type conversion helpers, `execute_scoop_action() -> (Outcome, Receiver)`. This is the shared bridge between UI consumers (CLI, TUI) and domain crates. All output goes through EventSender — CommandResult carries no output.
- **What stays as `bridge/cache.rs`**: Real filesystem logic for prune/clear. Updated in Phase 3 to remove `.output`/`.streamed` references, but position unchanged in Phase 4.
- **What moves to `cli/report.rs`**: Report formatting from `service/msvc/report.rs` and `service/scoop/report.rs`.
- **What moves to `cli/scoop_runtime.rs`**: Scoop event bus + HTTP client orchestration (was `service/scoop/runtime.rs`). Calls `bridge::execute_scoop_action()` and consumes events for CLI display.
- **What gets removed**: `service/msvc/` directory (19 thin wrappers), `service/scoop/actions.rs`, `service/scoop/bucket.rs` (logic inlined into CLI handlers).
- **Final step**: Rename `service/` to `bridge/` — global mechanical rename of `crate::service` → `crate::bridge` across all subsystems: `cli/`, `actions/`, `editor/`, `packages/`, `status/`, `view/`, `logger/`, `tui/`.

## Testing Decisions

- **Good test definition**: Tests verify external behavior (given these inputs/files, does the function produce this output?) without asserting on internal details.
- **discover.rs boundary tests**: Test `find_preferred_msvc_binary` with constructed directory trees. Test `managed_toolchain_flags_with_request` with mock toolchain layouts. Prior art: existing `service::cache` tests use `tempfile::TempDir`.
- **pipeline.rs and validate.rs**: Deferred — these depend on network/MSI format/real compilers. Existing integration tests cover the workflows end-to-end.
- **Regression guard**: `cargo test` must pass after every phase.

## Out of Scope

- **TUI consumer**: The bridge layer enables TUI integration, but building the TUI is separate.
- **official.rs split**: The official installer module (1421 lines) is large but cohesive. Separate decision.
- **spoon-core changes**: No modifications to the shared infrastructure crate beyond adding `format_bytes`.
- **Error handling unification**: `anyhow` erasure at service boundary is a known issue but orthogonal to this plan. Revisit when retry/categorization is needed.
- **cli/run.rs split** (972 lines): Worth splitting but should be done after Phase 4 when the file has settled.
- **spoon-msvc test coverage**: 3291 lines with only 2 tests is a critical gap. Independent follow-up task — not blocking the refactors.

## Further Notes

- **Execution order**: Strictly sequential: 0→1→2→3→4. Phase 0 and 1 could theoretically parallel but sequential is safer.
- **Commit strategy**: Each phase is a separate commit with `cargo test` passing.
- **Risk**: Event bridge unification (Phase 3) is highest-risk — touches function signatures across two domain crates. Execute bottom-up within Phase 3: inner functions first, public API last.
- **Deferred items** (for future PRDs):
  - spoon-msvc test coverage improvement
  - cli/run.rs command dispatch split
  - Error handling unification (anyhow → structured errors)
  - config/io.rs per-target file split
