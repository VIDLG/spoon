---
phase: 01-backend-seams-and-ownership
plan: 08
subsystem: backend-layout
tags: [rust, refactoring, runtime-layout, path-derivation]

# Dependency graph
requires:
  - phase: 01-backend-seams-and-ownership
    plan: 01-01
    provides: RuntimeLayout struct in spoon-backend
  - phase: 01-backend-seams-and-ownership
    plan: 01-04
    provides: BackendStatusSnapshot with runtime_roots
  - phase: 01-backend-seams-and-ownership
    plan: 01-06
    provides: view/config.rs migration pattern
provides:
  - Zero production uses of app-side config path helpers for backend layout derivation
  - 14 deprecated path helper functions preserved for backward compatibility
affects: [02-scoop-state-consolidation, 03-lifecycle-thinning]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "RuntimeLayout::from_root(root) as single entry point for all root-derived layout paths"

key-files:
  created: []
  modified:
    - spoon/src/packages/tool.rs
    - spoon/src/status/policy.rs
    - spoon/src/status/mod.rs
    - spoon/src/editor/discovery.rs
    - spoon/src/packages/msvc.rs
    - spoon/src/status/discovery/probe.rs
    - spoon/src/config/paths.rs

key-decisions:
  - "14 app-side path helper functions marked #[deprecated] with RuntimeLayout replacement guidance rather than deleted (per CLAUDE.md refactoring safety)"
  - "Test code in probe.rs and policy.rs migrated to RuntimeLayout for consistency despite not being production code"

patterns-established:
  - "RuntimeLayout::from_root(root) at function entry, then use layout.* fields throughout"

requirements-completed: [BNDR-04, LAY-01]

# Metrics
duration: 3min
completed: 2026-03-28
---

# Phase 01 Plan 08: Gap Closure Summary

**All 7 app modules migrated from app-side config path helpers to RuntimeLayout, with 14 deprecated functions preserved for backward compatibility.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-28T14:51:50Z
- **Completed:** 2026-03-28T14:54:52Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Eliminated all production uses of config::scoop_root_from, config::msvc_root_from, config::shims_root_from, config::msvc_cache_root_from, config::msvc_toolchain_root_from in spoon/src/
- Migrated 7 app modules to use RuntimeLayout::from_root(root) as single entry point for backend path derivation
- Deprecated 14 app-side path helper functions in config/paths.rs with RuntimeLayout replacement annotations

## Task Commits

Each task was committed atomically:

1. **Task 1: Migrate tool.rs, policy.rs, mod.rs, editor/discovery.rs to RuntimeLayout** - `222c011` (refactor)
2. **Task 2: Migrate msvc.rs, probe.rs and deprecate unused app-side path helpers** - `e26a16f` (refactor)

## Files Created/Modified
- `spoon/src/packages/tool.rs` - Replaced config::*_from imports with RuntimeLayout in 8 detail methods
- `spoon/src/status/policy.rs` - ownership_for_status and test fixture migrated to RuntimeLayout
- `spoon/src/status/mod.rs` - status_path_mismatches uses RuntimeLayout for shims path
- `spoon/src/editor/discovery.rs` - managed_scoop_editor_path uses RuntimeLayout for scoop/apps/shims paths
- `spoon/src/packages/msvc.rs` - config_scope_details and detected_wrapper_names use RuntimeLayout
- `spoon/src/status/discovery/probe.rs` - configured_probe_path, probe_msvc_toolchain, and test fixtures migrated
- `spoon/src/config/paths.rs` - 14 path helper functions marked #[deprecated] with RuntimeLayout replacement comments

## Decisions Made
- Preserved all deprecated functions with `#[deprecated]` attribute rather than deleting them, per CLAUDE.md refactoring safety guideline
- Migrated test code in probe.rs and policy.rs to RuntimeLayout for consistency, since the deprecated warnings in test code are acceptable noise

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- BNDR-04 fully satisfied: spoon app no longer derives backend layout paths in production code
- LAY-01 fully satisfied: RuntimeLayout is the single implementation; app-side helpers are deprecated
- All existing regression tests pass (json_status_uses_backend_read_models, runtime_layout_derives_from_root)
- cargo check -p spoon -p spoon-backend compiles with zero errors

---
*Phase: 01-backend-seams-and-ownership*
*Completed: 2026-03-28*

## Self-Check: PASSED
