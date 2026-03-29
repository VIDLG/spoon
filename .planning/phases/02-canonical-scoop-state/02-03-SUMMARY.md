---
phase: 02-canonical-scoop-state
plan: 3
subsystem: state, query
tags: [rust, projections, canonical-state, serde]

# Dependency graph
requires:
  - phase: 02-02
    provides: "Canonical state store with write/read/enum APIs in scoop/state/store.rs"
provides:
  - "Typed projections module (scoop/state/projections.rs) for query-safe summaries"
  - "query.rs backed by canonical store enumeration instead of ad hoc read_dir + JSON parsing"
  - "runtime_status uses canonical store for installed package count and summaries"
  - "Backend regression test proving runtime_status reads canonical state"
affects: [02-04, 02-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Projection layer: canonical model -> lightweight DTOs for query/status surfaces"
    - "RuntimeLayout constructed at query boundary to bridge tool_root API with canonical store"

key-files:
  created:
    - spoon-backend/src/scoop/state/projections.rs
  modified:
    - spoon-backend/src/scoop/state/mod.rs
    - spoon-backend/src/scoop/query.rs
    - spoon-backend/src/scoop/info.rs
    - spoon-backend/src/scoop/tests/state.rs
    - spoon/tests/cli/json_flow.rs

key-decisions:
  - "Query functions keep tool_root signature but construct RuntimeLayout internally to call canonical store"
  - "info.rs closures updated to use canonical state::InstalledPackageState type alongside query.rs migration"
  - "Projection helpers return ScoopInstalledPackageEntry DTOs, not full canonical model, for status surfaces"

patterns-established:
  - "Projection pattern: state model -> lightweight summary DTOs for query/status surfaces"
  - "Boundary bridge: public API takes &Path, internally constructs RuntimeLayout for store access"

requirements-completed: [SCST-01, SCST-02]

# Metrics
duration: 4min
completed: 2026-03-29
---

# Phase 02 Plan 03: Query/Status Projections Summary

**Typed projection layer and canonical-store-backed query/status surfaces replacing ad hoc read_dir + serde_json::from_str in query.rs**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-29T00:29:53Z
- **Completed:** 2026-03-29T00:34:04Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Created `scoop/state/projections.rs` with typed helpers for projecting `InstalledPackageState` into query-safe summaries
- Rewired `query.rs` to delegate installed-state enumeration through canonical `list_installed_states` from `state/store.rs`
- Eliminated `read_dir` + `serde_json::from_str` from installed-state enumeration path in `query.rs`
- Added backend regression test `runtime_status_uses_canonical_installed_state` proving end-to-end canonical store consumption
- Updated app-shell test to seed canonical schema and assert concrete package name/version/count

## Task Commits

Each task was committed atomically:

1. **Task 1: Add canonical installed-state projections and rewire query.rs to use them** - `ba2c395` (feat)
2. **Task 2: Add status/list regressions at backend and app-shell boundaries** - `32997cb` (test)

## Files Created/Modified
- `spoon-backend/src/scoop/state/projections.rs` - Typed projection helpers: installed_package_summary, list_installed_summaries, list_all_installed_states, list_installed_states_filtered
- `spoon-backend/src/scoop/state/mod.rs` - Added projections module and re-exports
- `spoon-backend/src/scoop/query.rs` - Rewired installed_package_states/filtered and runtime_status to use canonical store + projections
- `spoon-backend/src/scoop/info.rs` - Updated filter closures to use canonical state::InstalledPackageState type
- `spoon-backend/src/scoop/tests/state.rs` - Added runtime_status_uses_canonical_installed_state regression test
- `spoon/tests/cli/json_flow.rs` - Updated status and prefix tests to seed canonical schema

## Decisions Made
- Query functions keep their existing `tool_root: &Path` public signature; they construct `RuntimeLayout` internally at the boundary. This avoids changing the public API while routing through the canonical store.
- `info.rs` filter closures were updated from `super::runtime::InstalledPackageState` to the canonical `state::InstalledPackageState` as part of this task since the return type of `installed_package_states_filtered` changed. This is a minimal compat fix, not a full info.rs migration (planned for 02-04).
- `runtime_status` now uses `list_installed_summaries` (projection layer) instead of collecting full `InstalledPackageState` records and mapping inline.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed scoop_prefix_json_prints_structured_prefix_view test to use canonical schema**
- **Found during:** Task 2 (Add status/list regressions)
- **Issue:** The test seeded installed-state JSON without the required `bucket` field, which the canonical `InstalledPackageState` model requires. The file silently failed to deserialize, causing `package_prefix_report` to report the package as not installed.
- **Fix:** Added `"bucket": "main"` to the JSON seed in the test.
- **Files modified:** `spoon/tests/cli/json_flow.rs`
- **Verification:** `cargo test -p spoon --test json_flow scoop_prefix_json_prints_structured_prefix_view` passes
- **Committed in:** `32997cb` (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix was required for correctness -- the test broke because the canonical model enforces `bucket` as a required field. No scope creep.

## Issues Encountered
- 3 pre-existing test failures in json_flow.rs (scoop_bucket_remove, install_json_prints, status_refresh_json) confirmed unrelated to this plan's changes via git stash comparison. These failures existed before this plan.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Query/status surfaces are fully on canonical store. Ready for 02-04 (info/outcome/uninstall/reapply projections).
- `info.rs` still uses raw `serde_json::Value` reading via `read_installed_package_state` (local function). This is the target of plan 02-04.
- The deprecated `runtime::InstalledPackageState` re-export can be removed once info.rs is fully migrated in 02-04.

---
*Phase: 02-canonical-scoop-state*
*Completed: 2026-03-29*
