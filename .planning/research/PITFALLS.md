# Domain Pitfalls

**Domain:** Spoon frontend/backend refactor focused on `spoon-backend/src/scoop`
**Researched:** 2026-03-28
**Confidence:** HIGH for repo-specific findings, MEDIUM for `gix` ecosystem guidance

## Critical Pitfalls

Mistakes here are likely to force another rewrite or leave users with broken managed installs.

### Pitfall 1: Preserving the current frontend/backend leak instead of actually moving ownership
**What goes wrong:** The refactor moves files but not responsibility, so `spoon-backend` still depends on frontend-owned policy/config behavior through adapter seams.
**Why it happens:** The current Scoop runtime already crosses the boundary through `spoon/src/service/scoop/runtime.rs`, which implements `ScoopRuntimeHost` from `spoon-backend/src/scoop/runtime/execution.rs`. That host still owns test mode, home resolution, PATH mutation, supplemental shims, pip mirror display logic, and package integration application.
**Consequences:** `spoon-backend` remains a partial backend, `spoon` stays coupled to runtime internals, and every future Scoop change still requires coordinated edits in both crates.
**Prevention:** Treat this refactor as an ownership cut, not a wrapper shuffle. Backend should own Scoop runtime decisions and persisted state interpretation. Keep only a narrow OS/application port for side effects that truly must stay app-owned. If a method on `ScoopRuntimeHost` exists only to ask `spoon` how Scoop should behave, move that behavior into `spoon-backend`.
**Detection:** The refactor is going wrong if `spoon/src/service/scoop/runtime.rs` grows, if new backend traits mention app concepts like package policy or config scopes, or if `spoon/src/service/scoop/mod.rs` still computes runtime-derived Scoop state after the move.

### Pitfall 2: Reordering Scoop runtime state transitions without an explicit state machine
**What goes wrong:** Install/update/uninstall starts leaving half-applied packages behind because the current ordering of destructive and constructive steps gets changed casually during cleanup.
**Why it happens:** `spoon-backend/src/scoop/runtime/actions.rs` currently performs a long sequence with real side effects: dependency install, persist sync, shortcut removal, download, extract/materialize, pre-install hooks, installer hooks, persist restore, `current` refresh, target validation, shim creation, shortcut creation, post-install hooks, integration apply, then installed-state write. Uninstall similarly performs hooks, persist sync, shim removal, shortcut removal, package root deletion, then state deletion. There is no transaction wrapper or rollback journal.
**Consequences:** Users can end up with `current` updated but no state file, a state file pointing at files that do not exist, persisted data synced out and not restored, or shims removed while the package still exists.
**Prevention:** Before changing structure, write down the runtime phases as explicit backend states: `resolved -> staged -> activated -> integrated -> committed`, and keep every destructive transition behind a phase boundary. Add a temporary operation journal in `spoon-backend/src/scoop/state` or equivalent before removing old assets. Do not collapse install/update/uninstall into one generic “apply manifest” function unless the state machine is explicit.
**Detection:** After a failed action, check for disagreement between `spoon-backend/src/scoop/query.rs`, `spoon-backend/src/scoop/info.rs`, and the filesystem under `tool_root\\scoop\\apps`, `tool_root\\scoop\\persist`, and `tool_root\\scoop\\state\\packages`.

### Pitfall 3: Removing duplicate state models by name instead of by behavior
**What goes wrong:** Duplicate structs disappear, but important behavior encoded in one of them disappears too.
**Why it happens:** The repo has at least three different Scoop state layers:
- Legacy persisted package state in `spoon-backend/src/scoop/package_state.rs` with `name/version/bucket/architecture`
- Live installed runtime state in `spoon-backend/src/scoop/runtime/installed_state.rs` with bins, shortcuts, env, persist, integrations, and uninstall hooks
- Derived install state in both `spoon-backend/src/scoop/info.rs` and `spoon/src/service/scoop/actions.rs`

There is also a separate frontend tool-status model in `spoon/src/status/mod.rs`, and frontend path helpers in `spoon/src/config/paths.rs` duplicate backend path rules from `spoon-backend/src/scoop/paths.rs` and `spoon-backend/src/msvc/paths.rs`.
**Consequences:** The UI can lose ownership detection, installed version reporting, cache size reporting, integration display, uninstall hook replay, or path derivation even if “the same fields” seem to exist elsewhere.
**Prevention:** Make one table before deleting anything:
1. Canonical persisted Scoop package schema
2. Canonical derived runtime read model
3. Canonical frontend presentation model

Then map every producer and consumer to one of those three. Only delete `spoon-backend/src/scoop/package_state.rs` after proving no persisted files or report paths still depend on its semantics. Prefer backend-owned typed readers with `#[serde(default)]` for additive compatibility, and derive CLI/TUI models from backend outputs instead of recomputing install state in `spoon`.
**Detection:** Run `scoop status`, `scoop list`, `scoop info`, `scoop prefix`, tool detail rendering, and tool-page preselection against the same seeded package and verify they agree on installed/version/current/integration data.

### Pitfall 4: Centralizing git responsibility but leaving version skew and API leakage in place
**What goes wrong:** Git logic is nominally “backend-owned” while `spoon` still carries git implementation dependencies or assumptions, making the split fragile.
**Why it happens:** Bucket cloning already goes through backend `clone_repo` in `spoon-backend/src/gitx.rs`, but `spoon/Cargo.toml` still depends on `gix = 0.80` while `spoon-backend/Cargo.toml` uses `gix = 0.70`. I did not find direct `gix::` usage in `spoon/src`, which means the dependency is already suspicious. Current primary source: docs.rs lists newer `gix` releases beyond both pinned versions, so leaking git types across the crate boundary will harden ongoing version skew.
**Consequences:** Two crates drift on git features and bug behavior, backend APIs start reflecting `gix` internals, and future git fixes require synchronized multi-crate surgery.
**Prevention:** Make `spoon-backend` the only crate allowed to depend on `gix`. Remove the unused `gix` dependency from `spoon/Cargo.toml` in the same milestone unless a concrete frontend use appears. Expose only backend-level repo operations and backend event types. Do not return `gix` types or branch/checkout policy knobs directly to `spoon`.
**Detection:** The migration failed if `cargo tree` still shows `spoon` directly depending on `gix`, if new public backend APIs mention gitoxide-specific types, or if bucket/git behavior requires frontend changes outside `spoon/src/service/*`.

### Pitfall 5: Accidentally breaking MSVC while cleaning up Scoop
**What goes wrong:** Scoop cleanup deletes or repurposes state that MSVC still shares.
**Why it happens:** Scoop and managed MSVC both use the shared `tool_root\\shims` location. Scoop derives it from `spoon-backend/src/scoop/paths.rs`; MSVC derives it from `spoon-backend/src/msvc/paths.rs` and writes wrappers from `spoon-backend/src/msvc/wrappers.rs`. Frontend path helpers in `spoon/src/config/paths.rs` duplicate the same topology. A broad “rebuild shims” or “clean stale state” pass can easily overreach.
**Consequences:** `spoon-cl.cmd` and other managed MSVC wrappers disappear, MSVC status becomes wrong, or Scoop assumes the whole shims root is Scoop-owned and prunes non-Scoop files.
**Prevention:** Any Scoop refactor that touches `tool_root\\shims` must operate by manifest-owned file list, never by directory wipe. Keep Scoop cleanup scoped to package-owned aliases and old `tool_root\\scoop\\shims` only. If path helpers move, add parity tests between Scoop and MSVC path derivation before changing behavior.
**Detection:** After Scoop install/update/uninstall, verify MSVC wrapper presence and `spoon msvc status` output still match pre-refactor behavior.

### Pitfall 6: No rollback story for bucket replacement and package activation
**What goes wrong:** A failed bucket update or package activation leaves the managed root in a worse state than before.
**Why it happens:** `spoon-backend/src/scoop/buckets.rs` clones to a temp path, then removes the current bucket and renames the temp directory into place. `spoon-backend/src/scoop/extract.rs` deletes `current` before recreating it. These are replace-style flows without an automatic restore path if the final rename/copy step fails.
**Consequences:** Missing buckets, empty `current` installs, broken shims, and inconsistent runtime status after interruption, filesystem errors, or locked files.
**Prevention:** Add explicit rollback candidates before destructive replacement:
- Bucket update: keep previous bucket dir until new dir is fully validated, then swap with a restorable backup name.
- Package activation: write `current.new`, validate targets, then atomically swap pointer/symlink where possible.
- Package state: write state to temp file and rename, never direct overwrite.
**Detection:** Kill the process between “remove old” and “finalize new” in a test harness and assert recovery or detectable repair state instead of silent corruption.

## Moderate Pitfalls

### Pitfall 1: Treating frontend status/policy code as harmless presentation logic
**What goes wrong:** Migration ignores frontend code because it “just renders UI”, but it actually embeds runtime ownership rules and action gating.
**Prevention:** Audit `spoon/src/status/mod.rs`, `spoon/src/status/discovery/probe.rs`, `spoon/src/status/policy.rs`, and `spoon/src/actions/execute/scoop.rs` as behavioral code, not view code. Move backend facts first, then re-derive frontend policy from backend facts.

### Pitfall 2: Keeping duplicate path derivation in both crates during the refactor
**What goes wrong:** `spoon` and `spoon-backend` derive different roots for the same managed assets.
**Prevention:** Consolidate Scoop and MSVC root derivation behind backend-owned path helpers or backend-returned runtime metadata. Do not maintain separate path math in `spoon/src/config/paths.rs` and backend path modules longer than necessary.

### Pitfall 3: Leaving app-only test toggles attached to runtime semantics
**What goes wrong:** Tests still pass in fake mode while backend behavior regresses.
**Prevention:** Replace frontend-only switches like `set_real_backend_test_mode` in `spoon/src/service/scoop/mod.rs` and harness helpers in `spoon/src/tui/test_support.rs` with backend-oriented fixtures and seeded state tests. Keep fake mode for UI speed, but add direct backend tests for real state transitions.

### Pitfall 4: Preserving uninstall hook behavior accidentally or changing it accidentally
**What goes wrong:** Uninstall semantics change without anyone noticing.
**Why it matters:** `spoon-backend/src/scoop/runtime/hooks.rs` currently ignores uninstall hook failures but fails install hooks. That is a real behavior contract.
**Prevention:** Decide explicitly whether this asymmetry stays. Encode it in tests before moving hook execution code.

## Minor Pitfalls

### Pitfall 1: Breaking CLI/TUI output contracts while removing duplication
**What goes wrong:** The refactor changes titles, summary lines, or JSON shapes that tests and users already rely on.
**Prevention:** Snapshot key command outputs from `spoon/src/cli/run.rs`, `spoon/src/service/scoop/report.rs`, and TUI output modal flows before changing backend return types.

### Pitfall 2: Confusing dead exports with safe deletion
**What goes wrong:** `spoon-backend/src/scoop/mod.rs` still re-exports old state helpers, so deleting them can break compile-time consumers even if runtime usage is gone.
**Prevention:** Remove re-exports only after `rg` proves no remaining uses in either crate and tests are updated in the same patch.

### Pitfall 3: Letting bucket bootstrap rules drift during the refactor
**What goes wrong:** `main` bucket bootstrap behavior changes unintentionally.
**Prevention:** Preserve the current `ensure_main_bucket_ready` behavior in `spoon-backend/src/scoop/buckets.rs` unless there is a deliberate product decision to change first-run behavior.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Backend state-model consolidation | Deleting `ScoopPackageState` and keeping only the runtime state file without checking `info`, `status`, `prefix`, and UI detail consumers | Build a producer/consumer matrix first; migrate consumers to one backend read model before deleting old structs |
| Scoop runtime refactor | Reordering install/update steps and losing persist, shims, or uninstall metadata | Freeze current phase ordering in tests; introduce explicit transition phases before changing code structure |
| Git ownership move | Keeping `gix` in `spoon` “for later” while moving more git logic into backend | Remove the direct `gix` dependency from `spoon` as soon as backend-only ownership is established |
| Scoop cleanup | Deleting shared `tool_root\\shims` entries that belong to MSVC | Operate on per-package alias manifests only; add MSVC wrapper survival tests around Scoop actions |
| Boundary cleanup | Moving frontend config/policy logic into backend because it is convenient | Keep backend inputs data-oriented; keep app config serialization and UI policy in `spoon` |
| Test refresh | Replacing end-to-end assertions with only unit tests | Keep one ignored real Scoop flow and add backend failure-injection tests for partial installs, partial bucket updates, and cancellation |
| Rollback hardening | Assuming temp directories are enough rollback | Add backup-and-swap or journaled replacement for buckets, `current`, and state files |

## Sources

- Local code: `spoon-backend/src/scoop/runtime/actions.rs`
- Local code: `spoon-backend/src/scoop/runtime/installed_state.rs`
- Local code: `spoon-backend/src/scoop/package_state.rs`
- Local code: `spoon-backend/src/scoop/info.rs`
- Local code: `spoon-backend/src/scoop/buckets.rs`
- Local code: `spoon-backend/src/scoop/extract.rs`
- Local code: `spoon-backend/src/scoop/runtime/execution.rs`
- Local code: `spoon-backend/src/scoop/runtime/hooks.rs`
- Local code: `spoon-backend/src/gitx.rs`
- Local code: `spoon-backend/src/msvc/paths.rs`
- Local code: `spoon-backend/src/msvc/wrappers.rs`
- Local code: `spoon/src/service/scoop/runtime.rs`
- Local code: `spoon/src/service/scoop/actions.rs`
- Local code: `spoon/src/status/mod.rs`
- Local code: `spoon/src/status/discovery/probe.rs`
- Local code: `spoon/src/status/policy.rs`
- Local code: `spoon/src/config/paths.rs`
- Local code: `spoon/src/actions/execute/scoop.rs`
- Local code: `spoon/src/tui/test_support.rs`
- Local code: `spoon/tests/cli/scoop_flow.rs`
- Local code: `spoon/tests/cli/scoop_runtime_flow.rs`
- Local code: `spoon/tests/tui/tui_scoop_flow.rs`
- Local code: `spoon/Cargo.toml`
- Local code: `spoon-backend/Cargo.toml`
- Official docs: https://docs.rs/crate/gix/latest/features
