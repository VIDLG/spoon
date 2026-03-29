---
phase: 02-canonical-scoop-state
plan: 5
subsystem: scoop-state
tags: [rust, scoop, state-cleanup, doctor, legacy-detection]

# Dependency graph
requires:
  - phase: 02-04
    provides: "Canonical state read-side migrated for info/outcome/uninstall/reapply consumers"
provides:
  - "Legacy ScoopPackageState API removed from backend public surface"
  - "Explicit stale legacy state detection in doctor module"
  - "Regression test for legacy flat state reporting"
affects: [03-lifecycle-thinning]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Forward-design stale-state signaling: legacy files detected and reported, not silently supported"

key-files:
  created:
    - spoon-backend/src/scoop/_deprecated/package_state.rs
    - spoon-backend/src/scoop/tests/state.rs (tests added)
  modified:
    - spoon-backend/src/scoop/mod.rs
    - spoon-backend/src/scoop/doctor.rs

key-decisions:
  - "Moved package_state.rs to _deprecated/ per CLAUDE.md refactoring safety instead of deleting"
  - "Legacy state detection scans scoop/state/*.json excluding buckets.json and packages/ directory"
  - "doctor_with_host sets success=false when legacy state files are detected"
  - "Added LegacyScoopStateIssue struct with kind, path, message fields for typed reporting"

patterns-established:
  - "Stale-state boundary pattern: detect and report legacy state explicitly instead of compatibility shims"

requirements-completed: [SCST-03, SCST-04]

# Metrics
duration: 2min
completed: 2026-03-29
---

# Phase 02 Plan 05: Legacy Removal and Stale-State Signaling Summary

**Removed duplicate ScoopPackageState API from backend surface and added explicit legacy flat state detection in doctor module**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-29T00:40:56Z
- **Completed:** 2026-03-29T00:43:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Removed `ScoopPackageState` and all legacy package_state exports from `scoop/mod.rs` public surface
- Moved `package_state.rs` to `_deprecated/` backup per refactoring safety policy
- Added `detect_legacy_flat_state_files()` to doctor module that scans for pre-canonical state files
- Added `legacy_state_issues` field to `ScoopDoctorDetails` with `LegacyScoopStateIssue` struct
- Doctor sets `success=false` when legacy flat state files are found
- Added `legacy_flat_scoop_state_is_reported` regression test and `no_legacy_issues_when_state_is_clean` negative test

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove legacy package_state.rs exports and duplicate public APIs** - `d807613` (refactor)
2. **Task 2: Report stale legacy flat state explicitly and pin it with tests** - `8a673a3` (feat)

## Files Created/Modified
- `spoon-backend/src/scoop/mod.rs` - Removed `mod package_state` declaration and `pub use package_state::{...}` export block
- `spoon-backend/src/scoop/_deprecated/package_state.rs` - Legacy file moved to backup location per CLAUDE.md refactoring safety
- `spoon-backend/src/scoop/doctor.rs` - Added `LegacyScoopStateIssue` struct, `detect_legacy_flat_state_files()`, `legacy_state_issues` field on doctor details, wired into `doctor_with_host`
- `spoon-backend/src/scoop/tests/state.rs` - Added `legacy_flat_scoop_state_is_reported` and `no_legacy_issues_when_state_is_clean` tests

## Decisions Made
- Moved `package_state.rs` to `_deprecated/` instead of deleting, per CLAUDE.md refactoring safety rule requiring backup before deletion
- Legacy detection scans `scoop/state/*.json` directly (old layout), excluding `buckets.json` (legitimate) and any directories (including canonical `packages/` subdirectory)
- Doctor reports `success: false` when legacy files detected, providing explicit operator signal

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing state_root directory creation in test**
- **Found during:** Task 2 (legacy_flat_scoop_state_is_reported test)
- **Issue:** Test tried to write a legacy file to `scoop/state/` but the directory didn't exist yet (only `packages/` was created by `write_installed_state`)
- **Fix:** Added `std::fs::create_dir_all(&layout.scoop.state_root)` before writing the legacy test file
- **Files modified:** `spoon-backend/src/scoop/tests/state.rs`
- **Verification:** Test passes after fix
- **Committed in:** `8a673a3` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor test fix, no scope change.

## Issues Encountered
None - plan executed smoothly.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 2 (canonical-scoop-state) is complete with one canonical model, no duplicate legacy API, and explicit stale-state reporting
- SCST-03 and SCST-04 fully satisfied
- Phase 3 (lifecycle-thinning) can proceed with confidence that only canonical state surfaces exist
- The 3 pre-existing dead-code warnings in `runtime/installed_state.rs` are out of scope (from prior plans) and could be cleaned up in a future plan

## Self-Check: PASSED

All files found, both commits verified, mod.rs clean of legacy exports, doctor.rs contains legacy detection, test function exists.

---
*Phase: 02-canonical-scoop-state*
*Completed: 2026-03-29*
