---
phase: 02-canonical-scoop-state
plan: 1
subsystem: state
tags: [rust, serde, canonical-state, scoop]

# Dependency graph
requires:
  - phase: 01-backend-seams-and-ownership
    provides: "RuntimeLayout for layout-derived paths, BackendContext seams"
provides:
  - "Canonical InstalledPackageState with bucket and architecture fields"
  - "State store APIs: read, write, remove, list_installed_states using RuntimeLayout"
  - "Contract tests pinning non-derivable persistence boundary"
affects: [02-canonical-scoop-state]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Canonical state module pattern: model.rs (struct) + store.rs (persistence) + mod.rs (re-exports)"

key-files:
  created:
    - spoon-backend/src/scoop/state/mod.rs
    - spoon-backend/src/scoop/state/model.rs
    - spoon-backend/src/scoop/state/store.rs
    - spoon-backend/src/scoop/tests/state.rs
  modified:
    - spoon-backend/src/scoop/mod.rs
    - spoon-backend/src/scoop/runtime/mod.rs
    - spoon-backend/src/scoop/runtime/surface.rs
    - spoon-backend/src/scoop/tests/mod.rs
    - spoon-backend/tests/scoop_integration.rs

key-decisions:
  - "Canonical InstalledPackageState lives in scoop/state/model.rs, not in runtime"
  - "Store APIs accept RuntimeLayout instead of raw Path, aligning with Phase 1 layout ownership"
  - "Old runtime::installed_state kept for internal use; will be migrated in plan 02-02"
  - "scoop::InstalledPackageState re-export now points to state module, not runtime"

patterns-established:
  - "Canonical state module: model.rs + store.rs + mod.rs structure"
  - "RuntimeLayout-first store API signature"
  - "skip_serializing_if for clean JSON output on empty/None fields"

requirements-completed: [SCST-01, SCST-04]

# Metrics
duration: 53min
completed: 2026-03-29
---

# Phase 2 Plan 1: Canonical State Module Summary

**Canonical InstalledPackageState with bucket/architecture fields and RuntimeLayout-backed store APIs**

## Performance

- **Duration:** 53 min
- **Started:** 2026-03-28T23:30:11Z
- **Completed:** 2026-03-29T00:22:45Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Created `scoop/state/` module as the single ownership point for canonical installed-package state
- Enriched `InstalledPackageState` with `bucket: String` and `architecture: Option<String>` fields
- Store APIs (`read`, `write`, `remove`, `list_installed_states`) use `RuntimeLayout` instead of raw `Path`
- Two contract tests verify round-trip fidelity and non-derivable persistence boundary

## Task Commits

Each task was committed atomically:

1. **Task 1: Move canonical installed-state ownership into scoop/state/ and enrich it with bucket and architecture** - `7851c03` (feat)
2. **Task 2: Add canonical state contract tests and wire them into Scoop test modules** - `3c33b3e` (test)

## Files Created/Modified
- `spoon-backend/src/scoop/state/mod.rs` - Module declaration and re-exports
- `spoon-backend/src/scoop/state/model.rs` - Canonical `InstalledPackageState` struct with bucket and architecture
- `spoon-backend/src/scoop/state/store.rs` - Persistence APIs using RuntimeLayout
- `spoon-backend/src/scoop/tests/state.rs` - Round-trip and non-derivable-facts contract tests
- `spoon-backend/src/scoop/mod.rs` - Re-exports canonical InstalledPackageState from state module
- `spoon-backend/src/scoop/runtime/mod.rs` - Removed store API re-exports, added transition comment
- `spoon-backend/src/scoop/runtime/surface.rs` - Fixed import to use installed_state module directly
- `spoon-backend/src/scoop/tests/mod.rs` - Wired `mod state;` into test module
- `spoon-backend/tests/scoop_integration.rs` - Updated for new state API signature and fields

## Decisions Made
- Store APIs accept `&RuntimeLayout` rather than `&Path`, consistent with Phase 1's layout ownership pattern
- Old `runtime::installed_state` module kept intact for internal runtime use; full migration deferred to plan 02-02
- `scoop::InstalledPackageState` re-export now resolves to the canonical `state::InstalledPackageState`, not the runtime one

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed surface.rs import after removing store re-exports from runtime/mod.rs**
- **Found during:** Task 1
- **Issue:** `surface.rs` imported `read_installed_state` and `write_installed_state` from `super::` (runtime module root), which no longer exported them after the canonical state migration
- **Fix:** Changed import to `super::installed_state::{read_installed_state, write_installed_state}` to use the old module directly
- **Files modified:** `spoon-backend/src/scoop/runtime/surface.rs`
- **Committed in:** `7851c03` (part of Task 1 commit)

**2. [Rule 1 - Bug] Fixed scoop_integration.rs for new state API and struct fields**
- **Found during:** Task 2
- **Issue:** Integration test used old `write_installed_state(&Path, ...)` signature and constructed `InstalledPackageState` without `bucket` and `architecture` fields
- **Fix:** Updated test to use `RuntimeLayout::from_root(&root)` and added `bucket: "main"` and `architecture: None` to struct construction
- **Files modified:** `spoon-backend/tests/scoop_integration.rs`
- **Committed in:** `3c33b3e` (part of Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bug fixes)
**Impact on plan:** Both fixes necessary for compilation correctness after canonical state introduction. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Canonical state module is ready for runtime write migration (plan 02-02)
- `list_installed_states` enumeration API is ready for query/status projection migration (plan 02-03)
- Old `runtime::installed_state` module still owns internal persistence; must be migrated before `package_state.rs` removal

---
*Phase: 02-canonical-scoop-state*
*Completed: 2026-03-29*

## Self-Check: PASSED

- All 4 created files verified: state/mod.rs, state/model.rs, state/store.rs, tests/state.rs
- Both commits verified: 7851c03, 3c33b3e
- SUMMARY.md verified at .planning/phases/02-canonical-scoop-state/02-01-SUMMARY.md
