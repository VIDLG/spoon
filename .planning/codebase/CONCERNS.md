# Codebase Concerns

**Analysis Date:** 2026-03-28

## Tech Debt

**Oversized command and runtime modules:**
- Issue: Core orchestration is concentrated in a few very large files, which couples CLI parsing, rendering, backend dispatch, status shaping, filesystem work, and process execution into single edit surfaces.
- Files: `spoon/src/cli/run.rs`, `spoon/src/status/mod.rs`, `spoon-backend/src/msvc/mod.rs`, `spoon-backend/src/msvc/official.rs`, `spoon-backend/src/msvc/validation.rs`
- Impact: Small behavior changes have high regression risk, ownership boundaries are blurry, and test setup tends to mirror implementation detail instead of validating narrow units.
- Fix approach: Split by responsibility. Move command handlers out of `spoon/src/cli/run.rs`, move status rendering/modeling apart in `spoon/src/status/mod.rs`, and break MSVC flows into install/update/uninstall/validation submodules with operation-scoped helpers.

**Config parsing and serialization is permissive and brittle:**
- Issue: Config loaders silently fall back to defaults on TOML/JSON parse failure, and config writers patch serialized TOML with string replacement instead of relying only on structured document edits.
- Files: `spoon/src/config/io.rs`, `spoon/src/config/paths.rs`
- Impact: A malformed `~/.spoon/config.toml`, `~/.claude/settings.json`, or `~/.codex/config.toml` is treated like missing config, so user settings can appear to vanish without a clear error. Manual post-processing of serialized TOML increases the chance of format drift.
- Fix approach: Return typed parse errors to the caller, surface repair guidance in CLI/TUI, and replace the `"[policy ]"` string surgery with explicit `toml_edit` table handling plus atomic write paths.

**Shell-based Scoop runtime is still embedded in the backend:**
- Issue: Scoop lifecycle execution still depends on dynamically constructed PowerShell command strings and generated `.cmd` shim files.
- Files: `spoon-backend/src/scoop/runtime/hooks.rs`, `spoon-backend/src/scoop/runtime/surface.rs`, `spoon-backend/src/scoop/runtime/actions.rs`
- Impact: Hook behavior is hard to reason about, escaping rules are easy to get wrong, and backend behavior stays tied to shell semantics instead of typed Rust operations.
- Fix approach: Move common install/uninstall actions into structured Rust helpers, execute hook files via temporary scripts instead of long inline `-Command` strings, and keep shell use as a narrow compatibility layer.

**Workspace dependency versions are drifting:**
- Issue: The app and backend crates pin different major/minor versions of the same core libraries, especially `gix` and `msi`.
- Files: `spoon/Cargo.toml`, `spoon-backend/Cargo.toml`, `Cargo.lock`
- Impact: `cargo tree -d` shows duplicate dependency subgraphs, increasing compile time, binary size, and the risk of backend and app code behaving differently around Git or MSI handling.
- Fix approach: Centralize shared versions in workspace dependencies and align both crates on the same `gix`, `msi`, and related transport stacks before adding more Git/MSVC behavior.

## Known Bugs

**Scoop update/install can leave a package half-switched without rollback:**
- Symptoms: An update can remove the old `current` entry, sync persist state, and start writing the new version before the new install is fully validated. If a later hook, shim write, shortcut write, or layout check fails, the package can be left broken.
- Files: `spoon-backend/src/scoop/runtime/actions.rs`, `spoon-backend/src/scoop/extract.rs`, `spoon-backend/src/scoop/runtime/persist.rs`, `spoon-backend/src/scoop/runtime/surface.rs`
- Trigger: Any failure after `refresh_current_entry(...)` runs in `spoon-backend/src/scoop/runtime/actions.rs` and before the new installed state is fully written.
- Workaround: Re-run install for the same package or manually repair the package under the configured root. There is no automatic rollback path in the runtime.

**Malformed user config is silently treated as default state:**
- Symptoms: Root, proxy, Claude, or Codex settings can disappear from the UI/CLI after a hand-edited or partially written config file becomes invalid.
- Files: `spoon/src/config/io.rs`, `spoon/src/config/paths.rs`
- Trigger: Invalid TOML in `~/.spoon/config.toml` or `~/.codex/config.toml`, or invalid JSON in `~/.claude/settings.json` or `~/.codex/auth.json`.
- Workaround: Manually fix or remove the malformed file and let Spoon recreate it. Current loaders do not emit a strong user-facing parse failure.

**MSVC runtime configuration is process-global, not operation-scoped:**
- Symptoms: Two MSVC operations in the same process can observe whichever root/proxy/arch was written last into the global runtime config.
- Files: `spoon-backend/src/msvc/mod.rs`, `spoon/src/service/msvc/mod.rs`
- Trigger: Concurrent or interleaved calls that both invoke `apply_runtime_config()` and then run backend MSVC operations.
- Workaround: Avoid concurrent MSVC operations with different roots or policies in the same process. The current backend API assumes effectively serialized access.

## Security Considerations

**Assistant auth tokens are stored in plain text on disk:**
- Risk: Claude and Codex credentials are written directly into user-home config files as normal JSON values.
- Files: `spoon/src/config/io.rs`, `spoon/src/config/paths.rs`
- Current mitigation: Files live under the user's home directory and Codex auth is split from the main Codex TOML file.
- Recommendations: Store tokens with Windows Credential Manager or DPAPI-backed encryption, keep secrets out of broader settings documents when possible, and ensure logs or CLI views never echo token values.

**Inline PowerShell hook execution expands the trust boundary:**
- Risk: Scoop lifecycle scripts are concatenated into a single PowerShell command string and executed with `powershell -Command`, while helper paths and context values are interpolated into that script body.
- Files: `spoon-backend/src/scoop/runtime/hooks.rs`, `spoon-backend/src/scoop/runtime/actions.rs`
- Current mitigation: Interpolated literals are single-quote escaped and hook scripts are sourced from package manifests rather than raw user terminal input.
- Recommendations: Prefer temporary script files or a typed hook DSL, validate allowed helper executables, and keep shell execution isolated from manifest parsing and install state transitions.

**User PATH and registry mutation is high-impact state:**
- Risk: Spoon writes to `HKCU\\Environment` and mutates process `PATH`, which means mistakes affect all future shells for the user, not just the current run.
- Files: `spoon/src/config/env.rs`, `spoon/src/env.rs`
- Current mitigation: Test mode avoids persistent writes, and helper functions de-duplicate PATH entries case-insensitively.
- Recommendations: Add pre-write validation for candidate paths, add a repair command that can reconcile/remove Spoon-owned entries, and keep persisted and process-only mutations clearly separated in all call sites.

## Performance Bottlenecks

**Status refresh does repeated process launches and directory walks:**
- Problem: Status collection probes each tool by launching its version command, computes managed install sizes by recursively walking directories, and then optionally resolves latest versions.
- Files: `spoon/src/status/discovery/probe.rs`, `spoon/src/status/mod.rs`, `spoon/src/status/update.rs`, `spoon/src/tui/background.rs`
- Cause: The background refresh path does eager version probing and eager size calculation for the full tool list every time `start_bg_status_check(...)` runs.
- Improvement path: Cache size/version data between refreshes, defer expensive size calculations until detail view is opened, and separate "is it present?" probing from "full metadata refresh" work.

**MSVC discovery and validation scan large trees repeatedly:**
- Problem: Managed MSVC probing and validation rely on `WalkDir` over large toolchain roots and can also compile validation samples as part of verification.
- Files: `spoon-backend/src/msvc/mod.rs`, `spoon-backend/src/msvc/validation.rs`, `spoon-backend/src/msvc/official.rs`
- Cause: Binary discovery prefers filesystem search over indexed metadata, and validation reconstructs fresh workspaces under cache roots.
- Improvement path: Cache resolved compiler/linker paths after install, store validated toolchain metadata in state, and narrow search roots to known layout anchors instead of broad recursive walks.

## Fragile Areas

**Scoop lifecycle runtime:**
- Files: `spoon-backend/src/scoop/runtime/actions.rs`, `spoon-backend/src/scoop/runtime/hooks.rs`, `spoon-backend/src/scoop/runtime/surface.rs`, `spoon-backend/src/scoop/extract.rs`
- Why fragile: Install/update logic spans download, extraction, persist sync, hook execution, shortcut/shim generation, and state writes. The flow is stateful and partially destructive before completion, so ordering bugs have user-visible fallout.
- Safe modification: Change one phase at a time, add failure-path tests for each mutation point, and add rollback coverage before refactoring state transitions.
- Test coverage: Gaps remain. `spoon-backend/src/scoop/tests/runtime.rs` only covers manifest parsing and shim target expansion, and `spoon-backend/tests/scoop_integration.rs` focuses on package info rather than lifecycle execution.

**Global mutable runtime/test state:**
- Files: `spoon-backend/src/msvc/mod.rs`, `spoon/src/config/state.rs`, `spoon/src/tui/test_support.rs`, `spoon/tests/common/env_guard.rs`
- Why fragile: Home overrides, test mode, environment variables, and MSVC backend settings are stored in process-global state. Some test paths serialize access with a mutex, but the helpers themselves are still global and easy to misuse.
- Safe modification: Prefer explicit context objects over global state, keep environment mutation behind scoped guards, and avoid introducing new process-wide toggles.
- Test coverage: TUI harness serialization exists in `spoon/src/tui/test_support.rs`, but there are no concurrency tests proving isolation for CLI/backend flows that touch env or runtime globals.

**Tool catalog and tool-status policy logic:**
- Files: `spoon/src/packages/tool.rs`, `spoon/src/status/policy.rs`, `spoon/src/status/mod.rs`, `spoon/src/view/tools/detail.rs`
- Why fragile: Ownership rules, probe behavior, detail rendering, and user-facing policy all depend on the same hardcoded tool catalog. Adding or changing one tool tends to require edits across multiple unrelated surfaces.
- Safe modification: Treat tool metadata as a single source of truth and add targeted tests for ownership, detail rendering, and status state when modifying a tool definition.
- Test coverage: Tool-centric coverage exists, but most tests validate rendered outcomes instead of enforcing a smaller schema contract for metadata evolution.

## Scaling Limits

**Hardcoded tool inventory:**
- Current capacity: The current catalog is still manageable at the repository's present size, but it already drives a large metadata file and many branching status/detail rules.
- Limit: As more tools, editors, or toolchain modes are added, `spoon/src/packages/tool.rs` and the status/detail layers around it become increasingly expensive to modify safely.
- Scaling path: Move toward manifest-driven tool metadata or generate parts of the catalog from structured descriptors validated by tests.

**Single-process status refresh model:**
- Current capacity: A small, fixed set of tools can tolerate full refresh passes in the background.
- Limit: More tools or heavier per-tool probes will make the current "scan everything, size everything, then fetch updates" approach noticeably slower in the TUI.
- Scaling path: Introduce staged refreshes, persisted caches, and per-tool invalidation so the TUI does not recompute all metadata on every refresh cycle.

## Dependencies at Risk

**`gix`:**
- Risk: `spoon/Cargo.toml` uses `gix = 0.80` while `spoon-backend/Cargo.toml` uses `gix = 0.70`, producing two large Git dependency trees in one workspace.
- Impact: Build time and binary footprint increase, and future Git-related fixes may have to be applied twice against different APIs.
- Migration plan: Promote `gix` to a shared workspace dependency and migrate backend code to the newer API in one pass.

**`msi`:**
- Risk: `spoon/Cargo.toml` uses `msi = 0.10` while `spoon-backend/Cargo.toml` uses `msi = 0.8`.
- Impact: MSI handling behavior and bug fixes can diverge between crates, especially around managed MSVC work.
- Migration plan: Align both crates on one `msi` version and validate extraction/manifest behavior with backend integration tests before removing the older version.

## Missing Critical Features

**Transactional Scoop install/update rollback:**
- Problem: The runtime does not preserve a guaranteed-good package state that can be restored automatically after a failed update/install.
- Blocks: Reliable self-healing for broken package transitions and safer refactors in `spoon-backend/src/scoop/runtime/actions.rs`.

**Malformed-config diagnostics and repair flow:**
- Problem: Config corruption currently degrades into silent defaults instead of a surfaced repair path.
- Blocks: Safe user editing of `~/.spoon/config.toml`, `~/.claude/settings.json`, and `~/.codex/config.toml` without confusing state loss.

## Test Coverage Gaps

**Scoop lifecycle failure paths:**
- What's not tested: Direct execution coverage for `execute_package_action_streaming*`, hook-script failures, shim generation failures, and rollback behavior after `refresh_current_entry(...)`.
- Files: `spoon-backend/src/scoop/runtime/actions.rs`, `spoon-backend/src/scoop/runtime/hooks.rs`, `spoon-backend/src/scoop/runtime/surface.rs`, `spoon-backend/src/scoop/extract.rs`
- Risk: Package updates can regress into partially installed or broken states without being caught in CI.
- Priority: High

**Config corruption handling:**
- What's not tested: Behavior when global config, Claude settings, Codex config, or auth files contain invalid TOML/JSON.
- Files: `spoon/src/config/io.rs`, `spoon/src/config/paths.rs`
- Risk: The product silently falls back to defaults, and future refactors can make that behavior more confusing or destructive.
- Priority: High

**Concurrent/global-state safety:**
- What's not tested: Interleaving of MSVC operations with different runtime configs, and interaction between process-global env/test-mode overrides outside the TUI harness lock.
- Files: `spoon-backend/src/msvc/mod.rs`, `spoon/src/service/msvc/mod.rs`, `spoon/src/config/state.rs`, `spoon/tests/common/env_guard.rs`
- Risk: Races or cross-test contamination can stay latent until parallel test execution or background operations get expanded.
- Priority: Medium

---

*Concerns audit: 2026-03-28*
