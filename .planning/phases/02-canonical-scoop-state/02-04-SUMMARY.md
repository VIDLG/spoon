---
phase: 02-canonical-scoop-state
plan: 4
subsystem: backend
tags: [rust, serde, canonical-state, projections, scoop]

# Dependency graph
requires:
  - phase: 02-03
    provides: query/status projections on canonical installed state via list_installed_summaries
provides:
  - typed canonical state reads in info.rs replacing all raw JSON probing
  - canonical state regression test covering detail and outcome surfaces
affects: [02-05]

# Tech tracking
tech-stack:
  added: []
  patterns: [typed-state-projection, layout-derived-paths]

key-files:
  created: []
  modified:
    - spoon-backend/src/scoop/info.rs
    - spoon-backend/tests/scoop_integration.rs

key-decisions:
  - "info.rs reads canonical InstalledPackageState via state::read_installed_state instead of raw serde_json::Value"
  - "ShortcutEntry display strings computed from typed fields rather than parsed from JSON values"
  - "PersistEntry paths serialized to JSON only for output DTO; canonical state remains typed"

patterns-established:
  - "Typed state projection: read canonical InstalledPackageState, derive display strings from typed fields"
  - "Layout-derived paths at read time: state_path built from RuntimeLayout, not stored in state"

requirements-completed: [SCST-02, SCST-04]

# Metrics
duration: 5min
completed: 2026-03-29
---

# Phase 02 Plan 4: Unify Detail/Outcome Surfaces Around Canonical State Projections Summary

**Package detail and operation-outcome surfaces now project from typed InstalledPackageState instead of raw JSON rereads, with ShortcutEntry/PersistEntry display derived from canonical typed fields**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-29T00:34:52Z
- **Completed:** 2026-03-29T00:39:59Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Removed `read_installed_package_state()` raw JSON helper from info.rs, eliminating all `serde_json::Value`-based state reads in the package-info code path
- Both `package_info()` and `package_operation_outcome()` now consume canonical `InstalledPackageState` through `state::read_installed_state` via `RuntimeLayout`
- Added `scoop_package_info_reads_canonical_state` regression test validating bucket, architecture, bins, shortcuts, env, persist, and integrations are projected from typed canonical state

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace raw JSON state rereads in info.rs with typed canonical projections** - `b0d2667` (feat)
2. **Task 2: Pin canonical package-info behavior with backend and app-shell regressions** - `0d7c6ca` (test)

## Files Created/Modified
- `spoon-backend/src/scoop/info.rs` - Replaced raw JSON reads with typed canonical state projections; both package_info and package_operation_outcome now read through state::read_installed_state
- `spoon-backend/tests/scoop_integration.rs` - Added scoop_package_info_reads_canonical_state regression test covering detail and outcome surfaces with bucket, architecture, bins, shortcuts, env, persist, and integrations

## Decisions Made
- ShortcutEntry display strings are formatted from typed struct fields (name, target_path, args) rather than parsed from JSON values -- preserves the same output format without raw JSON dependency
- PersistEntry paths are serialized to serde_json::Value only for the output DTO (ScoopEnvironmentIntegration.persist); the canonical state itself remains fully typed
- Removed unused imports (value_to_display, package_state_path) that were only needed for the old JSON probing path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed shortcut manifest format in regression test**
- **Found during:** Task 2 (scoop_package_info_reads_canonical_state test)
- **Issue:** Test manifest used array-style shortcuts `[["ripgrep", "rg.exe"]]` which failed to deserialize as `Shortcut` enum (expects string or object, not array). Manifest parsed as None, causing all manifest-derived fields including latest_version to be None
- **Fix:** Changed to object-style `{ "name": "ripgrep", "target": "rg.exe" }` matching the `Shortcut::Detailed` variant
- **Files modified:** spoon-backend/tests/scoop_integration.rs
- **Verification:** Test passes with correct latest_version assertion
- **Committed in:** `0d7c6ca` (Task 2 commit)

**2. [Rule 3 - Blocking] Fixed async/await mismatch in bucket registration**
- **Found during:** Task 2 (scoop_package_info_reads_canonical_state test)
- **Issue:** `upsert_bucket_to_registry` is async but was called without `.await` in test code
- **Fix:** Wrapped call in `block_on()` to match the test runtime pattern
- **Files modified:** spoon-backend/tests/scoop_integration.rs
- **Verification:** Test compiles and passes
- **Committed in:** `0d7c6ca` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both were test infrastructure issues, not production code changes. No scope creep.

## Issues Encountered
None - all planned work completed without blocking issues.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- info.rs is now fully on canonical state projections, ready for plan 02-05 legacy removal
- All three verification boundaries (backend integration, app JSON info, app JSON status) remain green
- No remaining raw JSON state reads in info.rs; serde_json::Value usage in struct definitions is output DTO only

## Self-Check: PASSED

All files exist, all commits verified.

---
*Phase: 02-canonical-scoop-state*
*Completed: 2026-03-29*
