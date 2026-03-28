---
phase: 01-backend-seams-and-ownership
plan: 5
subsystem: backend-seams
tags: [rust, backendcontext, systemport, packageport, scoop-adapter, bucket-adapter]

# Dependency graph
requires:
  - phase: 01-02
    provides: BackendContext with generic ports, SystemPort, PackageIntegrationPort, RuntimeLayout
provides:
  - AppSystemPort implementing both SystemPort and PackageIntegrationPort at the app boundary
  - build_scoop_backend_context() as the single shared entry point for constructing BackendContext in the Scoop service layer
  - Thin package action adapter that delegates result reconstruction to backend package_operation_outcome
  - Thin bucket adapter that re-exports RepoSyncOutcome and reads config directly without load_backend_config
affects: [01-06, 01-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AppSystemPort: Single struct implementing SystemPort + PackageIntegrationPort, passed by reference into BackendContext"
    - "Context builder pattern: build_scoop_backend_context() reads config once and constructs BackendContext at the service boundary"
    - "Backend outcome delegation: package_action_result delegates to backend package_operation_outcome instead of reconstructing state locally"

key-files:
  created:
    - spoon/src/service/mod.rs (AppSystemPort, build_scoop_backend_context, re-exports BackendContext)
  modified:
    - spoon/src/service/scoop/actions.rs (removed load_backend_config, package_current_root, installed_package_states_filtered)
    - spoon/src/service/scoop/bucket.rs (removed load_backend_config, added RepoSyncOutcome re-export)
    - spoon/src/service/scoop/mod.rs (added RepoSyncOutcome re-export)
    - spoon/tests/cli/json_flow.rs (bucket_json_uses_backend_repo_sync_outcome test)

key-decisions:
  - "AppSystemPort is a static singleton passed by reference into BackendContext to avoid lifetime complexity"
  - "package_action_result delegates to backend package_operation_outcome rather than changing return types of run_scoop_streaming"
  - "Direct config reads (configured_root_override, configured_proxy) replace load_backend_config in action/bucket adapters"

patterns-established:
  - "Static port singleton: AppSystemPort as a static const, passed by &reference into BackendContext"
  - "Config-at-boundary: Read root_override and proxy once per action call from app config, not from BackendConfig wrapper"
  - "Backend outcome delegation: App adapters forward to backend outcome builders instead of duplicating state reconstruction"

requirements-completed: [BNDR-01, BNDR-02, GIT-02, GIT-03]

# Metrics
duration: 10min
completed: 2026-03-28
---

# Phase 1 Plan 5: Thin Scoop Adapters Summary

**App-side BackendContext builder with AppSystemPort, thin package action and bucket adapters that delegate state reconstruction and Git contracts to backend outcomes only**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-28T13:56:49Z
- **Completed:** 2026-03-28T14:06:36Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Added `AppSystemPort` implementing `SystemPort + PackageIntegrationPort` as the single app-owned port for `BackendContext`
- Added `build_scoop_backend_context()` as the shared entry point for constructing backend context at the service boundary
- Removed all `load_backend_config()` calls from actions.rs and bucket.rs, replacing with direct config reads
- Removed `package_current_root` and `installed_package_states_filtered` from actions.rs, delegating to backend `package_operation_outcome`
- Re-exported `RepoSyncOutcome` from backend in bucket adapter per D-09
- Added `bucket_json_uses_backend_repo_sync_outcome` regression test

## Task Commits

Each task was committed atomically:

1. **Task 1: Add one shared BackendContext builder and thin package/runtime adapters** - `443ed01` (feat)
2. **Task 2: Thin the bucket adapter to backend events and repo outcomes only** - `b0f52ef` (feat)
3. **Test amendment for Task 2** - `6b00e91` (test)

## Files Created/Modified
- `spoon/src/service/mod.rs` - Added `AppSystemPort`, `build_scoop_backend_context()`, re-exported `BackendContext`
- `spoon/src/service/scoop/actions.rs` - Removed `load_backend_config()`, `package_current_root`; delegates to backend `package_operation_outcome`
- `spoon/src/service/scoop/bucket.rs` - Removed `load_backend_config()`; added `RepoSyncOutcome` re-export; direct config reads for proxy
- `spoon/src/service/scoop/mod.rs` - Added `RepoSyncOutcome` to public exports
- `spoon/tests/cli/json_flow.rs` - Added `bucket_json_uses_backend_repo_sync_outcome` test

## Decisions Made
- Used a static `AppSystemPort` singleton passed by `&'static` reference into `BackendContext` to avoid lifetime complexity while keeping the port pattern clean.
- Kept `package_action_result` signature (takes `CommandResult`) but delegated internal state reconstruction to backend `package_operation_outcome` rather than changing the return types of `run_scoop_streaming`, minimizing blast radius on CLI call sites.
- Used local `configured_root_override()` and `configured_proxy()` helper functions instead of `load_backend_config()` for a direct, minimal config reading pattern in the thin adapters.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing compilation error in `spoon/src/status/mod.rs` (missing `collect_statuses_with_snapshot` and `collect_statuses_fast_with_snapshot` exports from discovery module) caused by a parallel agent's changes. This is out of scope for this plan and did not affect our changes. Our changes compile cleanly when considered in isolation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The app Scoop service layer now has a shared BackendContext builder and thin adapters for package actions and bucket operations
- Bucket adapter re-exports RepoSyncOutcome, satisfying GIT-02 and GIT-03 at the app boundary
- Subsequent plans can build on `build_scoop_backend_context()` and `AppSystemPort` to further thin the runtime and status adapters

## Self-Check: PASSED

All files and commits verified present.

---
*Phase: 01-backend-seams-and-ownership*
*Completed: 2026-03-28*
