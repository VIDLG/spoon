---
phase: 01-backend-seams-and-ownership
plan: 4
subsystem: backend-status
tags: [rust, backend, status-snapshot, read-models, spoon-backend]

# Dependency graph
requires:
  - phase: 01-01
    provides: "Backend module structure and lib.rs re-exports"
  - phase: 01-02
    provides: "Scoop query read models and runtime_status API"
  - phase: 01-03
    provides: "MSVC explicit context and status API"
provides:
  - "BackendStatusSnapshot aggregate read model for app consumption"
  - "Snapshot-accepting status collection variants in spoon"
  - "Backend-driven JSON status output path"
  - "Backend-driven TUI background refresh path"
affects: [01-05, 01-06, 01-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Aggregate read model pattern: BackendStatusSnapshot composes scoop::runtime_status and msvc::status"
    - "Snapshot-first collection: collect_statuses_with_snapshot replaces file-IO-based version lookup"

key-files:
  created:
    - spoon-backend/src/status.rs
    - spoon/tests/cli/status_backend_flow.rs
  modified:
    - spoon-backend/src/lib.rs
    - spoon/src/status/mod.rs
    - spoon/src/status/discovery/probe.rs
    - spoon/src/status/discovery/mod.rs
    - spoon/src/cli/json.rs
    - spoon/src/tui/background.rs
    - spoon/Cargo.toml

key-decisions:
  - "BackendStatusSnapshot aggregates scoop and MSVC queries into one serializable struct consumed by the app"
  - "probe.rs no longer reads Scoop state JSON files; version lookup goes through the snapshot only"
  - "build_status_details_with_snapshot accepts Optional snapshot for gradual migration"
  - "JSON and TUI background paths build their own snapshots before rendering"

patterns-established:
  - "Aggregate read model: BackendStatusSnapshot composes domain-specific backend queries into one app-facing type"
  - "Snapshot-driven collection: Status functions accept Option<&BackendStatusSnapshot> instead of opening state files"

requirements-completed: [BNDR-05, LAY-02]

# Metrics
duration: 10min
completed: 2026-03-28
---

# Phase 1 Plan 4: Backend Status Snapshot Summary

**BackendStatusSnapshot aggregate read model replacing app-side state file parsing for status, JSON, and TUI surfaces**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-28T13:56:49Z
- **Completed:** 2026-03-28T14:06:35Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Created `BackendStatusSnapshot` in `spoon-backend` that composes `scoop::runtime_status` and `msvc::status` into one serializable aggregate
- Eliminated direct state file reads from `spoon/src/status/discovery/probe.rs` (`scoop_state_root_from` and `read_to_string` both removed)
- JSON status output and TUI background refresh now build backend snapshots before rendering
- Regression test `json_status_uses_backend_read_models` validates the backend-driven path

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a backend-owned status snapshot surface for app consumers** - `14c3a79` (feat)
2. **Task 2: Rewire status, JSON, and TUI refresh code to backend read models only** - `dc741a2` (feat)

## Files Created/Modified
- `spoon-backend/src/status.rs` - BackendStatusSnapshot aggregate read model with Scoop, MSVC, and runtime roots
- `spoon-backend/src/lib.rs` - Re-exports `pub mod status`
- `spoon/src/status/mod.rs` - Added `build_status_details_with_snapshot` consuming backend snapshot for roots and state checks
- `spoon/src/status/discovery/probe.rs` - Removed `fs::read_to_string` and `scoop_state_root_from` usage; version lookup now goes through snapshot
- `spoon/src/status/discovery/mod.rs` - Re-exports snapshot-accepting collection functions
- `spoon/src/cli/json.rs` - `status_view` builds BackendStatusSnapshot before rendering
- `spoon/src/tui/background.rs` - `start_bg_status_check` builds snapshot for status collection
- `spoon/tests/cli/status_backend_flow.rs` - Regression test `json_status_uses_backend_read_models`
- `spoon/Cargo.toml` - Registered `status_backend_flow` test

## Decisions Made
- BackendStatusSnapshot derives all runtime paths from RuntimeLayout (D-10), not app-side config::paths helpers
- The snapshot aggregates scoop and MSVC status rather than exposing separate domain queries to the app
- Non-snapshot collection variants (`collect_statuses`, `collect_statuses_fast`) are preserved as thin wrappers that pass `None` to snapshot-accepting functions, ensuring backward compatibility for any code paths not yet migrated

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Pre-existing compilation error in `spoon/src/service/scoop/actions.rs` (partial move of `backend_outcome.action`) and 4 pre-existing test failures in `json_flow` (2 runtime-in-runtime panics, 2 assertion failures) -- all confirmed pre-existing by stashing changes and re-running tests. None caused by this plan's changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- BackendStatusSnapshot is ready for consumption by all app surfaces
- The status subsystem no longer reads backend state files directly (BNDR-05 satisfied)
- Layout ownership is consolidated in the backend (LAY-02 satisfied)
- Plans 01-05, 01-06, and 01-07 can now rely on backend-driven status as the single source of truth

---
*Phase: 01-backend-seams-and-ownership*
*Completed: 2026-03-28*
