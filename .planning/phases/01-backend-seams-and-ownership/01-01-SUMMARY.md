---
phase: 01-backend-seams-and-ownership
plan: 01
subsystem: infra
tags: [rust, backend, context, layout, scoop, msvc]
requires: []
provides:
  - BackendContext for explicit runtime inputs
  - RuntimeLayout for root-derived backend paths
  - Split SystemPort and package-integration host contracts
  - Contract tests for layout derivation and explicit context usage
affects: [scoop-runtime, msvc-runtime, app-service-layer, status-models]
tech-stack:
  added: []
  patterns:
    - backend-owned runtime context
    - backend-owned runtime layout
    - split system and package integration ports
key-files:
  created:
    - spoon-backend/src/context.rs
    - spoon-backend/src/layout.rs
    - spoon-backend/src/ports.rs
    - spoon-backend/src/tests/context.rs
  modified:
    - spoon-backend/src/lib.rs
    - spoon-backend/src/scoop/paths.rs
    - spoon-backend/src/msvc/paths.rs
    - spoon-backend/src/tests/mod.rs
key-decisions:
  - "BackendContext owns explicit runtime inputs instead of scattered root/proxy helpers."
  - "RuntimeLayout is the single backend-owned derivation point for Scoop, MSVC, shims, cache, and state roots."
  - "SystemPort and package-integration callbacks are split at the contract layer instead of preserving the mixed host shape."
patterns-established:
  - "Backend path helpers delegate to RuntimeLayout::from_root() instead of joining paths locally."
  - "New backend seam types live at crate root and are re-exported for downstream phases."
requirements-completed: [BNDR-04, LAY-01, LAY-03]
duration: 25min
completed: 2026-03-28
---

# Phase 1 Plan 01: Backend Seams and Ownership Summary

**Backend runtime context and layout now exist as first-class seam contracts, with Scoop and MSVC path derivation funneled through one backend-owned model.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-03-28T19:40:00+08:00
- **Completed:** 2026-03-28T20:05:00+08:00
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Added `BackendContext` so backend runtime inputs can be expressed explicitly.
- Added `RuntimeLayout` with nested Scoop and MSVC layout models derived from `root`.
- Added split `SystemPort` and package-integration host contracts plus nearby seam tests.
- Rebased backend Scoop/MSVC path helpers onto `RuntimeLayout::from_root(...)`.

## Task Commits

Execution in this session has not been split into per-task git commits yet because the repository currently contains broader in-progress workspace changes. The implementation is recorded in the working tree and validated by the plan-level tests below.

## Files Created/Modified
- `spoon-backend/src/context.rs` - Defines `BackendContext`.
- `spoon-backend/src/layout.rs` - Defines `RuntimeLayout`, `ScoopLayout`, and MSVC sub-layouts.
- `spoon-backend/src/ports.rs` - Initially defined `SystemPort` plus Scoop integration callbacks at the crate seam.
- `spoon-backend/src/tests/context.rs` - Adds seam-focused contract tests.
- `spoon-backend/src/scoop/paths.rs` - Delegates Scoop path derivation to `RuntimeLayout`.
- `spoon-backend/src/msvc/paths.rs` - Delegates MSVC path derivation to `RuntimeLayout`.
- `spoon-backend/src/lib.rs` - Re-exports the new backend seam contracts.
- `spoon-backend/src/tests/mod.rs` - Wires the new context test module.

## Decisions Made
- Introduced the seam contracts without breaking existing path helper function names, so downstream phases can migrate call sites incrementally while the ownership model is already fixed.
- Kept the old persisted directory names untouched, so this plan stays in Phase 1 and does not drift into Phase 2 state migration.
- Follow-up refinement later narrowed the Scoop-specific integration callback trait into `spoon-backend/src/scoop/ports.rs` as `ScoopIntegrationPort` while leaving `SystemPort` at the backend root. That change preserved this plan's intent and improved scope clarity.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Backend seam primitives are ready for Scoop runtime and bucket contracts in `01-02`.
- MSVC and app service layers can now migrate toward explicit backend context instead of ad hoc path helpers.
- No blocker found for Wave 2.

## Self-Check: PASSED

---
*Phase: 01-backend-seams-and-ownership*
*Completed: 2026-03-28*
