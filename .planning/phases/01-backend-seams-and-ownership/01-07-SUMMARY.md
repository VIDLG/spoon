---
phase: 01-backend-seams-and-ownership
plan: 7
subsystem: backend-seams
tags: [rust, cleanup, gix-removal, dead-code, dependency-audit]

# Dependency graph
requires:
  - phase: 01-05
    provides: AppSystemPort, build_scoop_backend_context, thin package/bucket adapters
  - phase: 01-06
    provides: RuntimeLayout-based config and detail surfaces, backend query model consumption
provides:
  - App service module without dead backend-path re-exports
  - App settings module cleaned of backend runtime-path helpers
  - App manifest without direct gix dependency (GIT-01 satisfied)
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "No direct gix dependency in spoon app - Git remains backend-only"
    - "Dead re-exports removed from service layer and settings barrel"

key-files:
  created: []
  modified:
    - spoon/src/service/mod.rs
    - spoon/src/settings.rs
    - spoon/Cargo.toml

key-decisions:
  - "Kept app-owned config semantics (GlobalConfig, PolicyConfig, configured_tool_root, etc.) in settings.rs while removing backend runtime-path helpers"
  - "Removed gix from spoon/Cargo.toml entirely - no direct usage existed, and spoon-backend owns all Git implementation"

patterns-established: []

requirements-completed: [GIT-01, LAY-02]

# Metrics
duration: 3min
completed: 2026-03-28
---

# Phase 1 Plan 7: Terminal Cleanup After Contract Moves Summary

**Removed dead backend-path re-exports from service/settings modules and eliminated the direct app gix dependency, completing Phase 1 seam cleanup**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-28T14:22:34Z
- **Completed:** 2026-03-28T14:26:03Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Removed `shims_root_from`, `msvc_state_root_from`, `msvc_cache_root_from`, `msvc_toolchain_root_from` re-exports from `spoon/src/service/mod.rs`
- Cleaned `spoon/src/settings.rs` of all backend runtime-path helper re-exports (`scoop_root_from`, `msvc_root_from`, `official_msvc_root_from`, `shims_root_from`, `msvc_cache_root_from`, etc.) while retaining app-owned config semantics
- Removed direct `gix v0.80` dependency from `spoon/Cargo.toml`, satisfying GIT-01; Git remains backend-only via `spoon-backend`

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove dead backend-path re-exports from app service and settings modules** - `5a8695a` (refactor)
2. **Task 2: Remove the direct app gix dependency and verify backend-only Git ownership** - `5bca6ee` (feat)

## Files Created/Modified
- `spoon/src/service/mod.rs` - Removed 4 dead backend-path re-export functions (shims_root_from, msvc_state_root_from, msvc_cache_root_from, msvc_toolchain_root_from)
- `spoon/src/settings.rs` - Removed all backend runtime-path helper re-exports, kept app config semantics only
- `spoon/Cargo.toml` - Removed direct gix dependency

## Decisions Made
- Kept app-owned config semantics in `settings.rs` (GlobalConfig, PolicyConfig, load_global_config, configured_tool_root, normalize_proxy_url, etc.) since these are legitimate app config concerns, not backend runtime layout seams
- Removed gix entirely from spoon/Cargo.toml rather than keeping it as a transitive dependency, since no spoon source code imports gix directly

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 1 is now complete: all 7 plans executed successfully
- App shell is a thin orchestration layer with no direct backend path derivation or gix dependency
- Backend owns RuntimeLayout, BackendContext, Git implementation, and all read models
- Ready for Phase 2: canonical Scoop state cleanup

## Self-Check: PASSED

All files and commits verified present.

---
*Phase: 01-backend-seams-and-ownership*
*Completed: 2026-03-28*
