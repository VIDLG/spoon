---
phase: 02-canonical-scoop-state
plan: 2
subsystem: spoon-backend/src/scoop
tags: [state, runtime, canonical, bucket, architecture]
dependency_graph:
  requires: [02-01]
  provides: [02-03, 02-04]
  affects: [scoop-runtime-lifecycle, scoop-state-store]
tech_stack:
  added: []
  patterns: ["canonical-state-write-through", "RuntimeLayout-adoption-in-runtime"]
key_files:
  created:
    - spoon-backend/src/scoop/tests/runtime.rs (canonical-state regression tests)
  modified:
    - spoon-backend/src/scoop/runtime/actions.rs (write canonical state with bucket/architecture)
    - spoon-backend/src/scoop/runtime/integration.rs (read/write canonical state via RuntimeLayout)
    - spoon-backend/src/scoop/runtime/surface.rs (read/write canonical state via RuntimeLayout)
    - spoon-backend/src/scoop/runtime/mod.rs (deprecate old InstalledPackageState re-export)
    - spoon-backend/src/scoop/info.rs (allow deprecated warnings for old type usage)
    - spoon-backend/src/scoop/query.rs (allow deprecated warnings for old type usage)
key_decisions:
  - "Deprecated old runtime::InstalledPackageState re-export to maintain compile compatibility with query.rs and info.rs until 02-03/02-04"
  - "Runtime actions construct RuntimeLayout locally from tool_root rather than changing public API signatures"
  - "Uninstall hook context now reads bucket from canonical state instead of hardcoded None"
metrics:
  duration: "5m"
  completed_date: 2026-03-29
  tasks_completed: 2
  files_modified: 6
  files_created: 1
  commits:
    - "9994b2e: feat(02-02): migrate runtime write/read to canonical state store with bucket and architecture"
    - "bec5c2e: test(02-02): add runtime canonical-state regression tests"
---

# Phase 02 Plan 02: Runtime Canonical-State Write/Read Migration Summary

Runtime lifecycle (install/update/uninstall/reapply) now reads and writes the canonical `InstalledPackageState` from `scoop::state`, with `bucket` and `architecture` populated during install.

## What Changed

- **actions.rs**: Install/update writes canonical state with `bucket` (from `resolved_manifest.bucket.name`) and `architecture` (from `selected_architecture_key()`). Uninstall reads canonical state and populates `HookContext.bucket` from stored state.
- **integration.rs**: `reapply_package_integrations_streaming_with_host` reads/writes via canonical state store using `RuntimeLayout`.
- **surface.rs**: `reapply_package_command_surface_streaming_with_host` reads/writes via canonical state store using `RuntimeLayout`.
- **runtime/mod.rs**: Old `InstalledPackageState` re-export marked `#[deprecated]` with migration guidance; kept private `installed_state` module for backward compat until read-side migration (02-03/02-04).
- **query.rs/info.rs**: Added `#[allow(deprecated)]` to suppress warnings on old type usage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Deprecated re-export needed to avoid breaking query.rs and info.rs**
- **Found during:** Task 1
- **Issue:** Removing the `InstalledPackageState` re-export from `runtime/mod.rs` broke `query.rs` and `info.rs` which still reference the old type via `super::runtime::InstalledPackageState`.
- **Fix:** Re-added the re-export with `#[deprecated]` attribute and added `#[allow(deprecated)]` in `query.rs` and `info.rs`. The read-side migration to canonical `InstalledPackageState` is planned for 02-03/02-04.
- **Files modified:** `spoon-backend/src/scoop/runtime/mod.rs`, `spoon-backend/src/scoop/query.rs`, `spoon-backend/src/scoop/info.rs`
- **Commit:** 9994b2e

### Deferred Issues

- **Pre-existing integration test failure**: `scoop_package_info_integrates_manifest_and_installed_state` in `spoon-backend/tests/scoop_integration.rs` fails with `assertion failed: success.install.installed`. This test uses `package_info()` which reads via the old `read_installed_package_state` path (raw JSON) and `installed_package_states_filtered` (old `InstalledPackageState` struct). The failure predates this plan and will be resolved when the read-side consumers are migrated in plans 02-03/02-04.

## Known Stubs

None.

## Tests Added

| Test | File | What It Proves |
|------|------|----------------|
| `runtime_writes_canonical_scoop_state` | `spoon-backend/src/scoop/tests/runtime.rs` | Canonical write includes `bucket` and `architecture`; no absolute paths persisted |
| `reapply_inputs_come_from_canonical_state` | `spoon-backend/src/scoop/tests/runtime.rs` | All operational fields (bins, shortcuts, env, persist, hooks) roundtrip through canonical store for reapply/uninstall |

## Self-Check: PASSED

- All 75 `spoon-backend` lib tests pass
- Both new regression tests pass independently
- `actions.rs` contains `bucket:` and `architecture:` in write block
- `actions.rs` does not persist `current_root` in `InstalledPackageState`
- Both test names exist in `spoon-backend/src/scoop/tests/runtime.rs`
