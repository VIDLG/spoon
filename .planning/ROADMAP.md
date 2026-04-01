# Roadmap: Backend Architecture Completion

## Overview

This roadmap continues phase numbering after `v0.5.0` and finishes the broader `spoon-backend` architectural cleanup by making `msvc` the primary refactor target, then hardening the backend contracts shared across domains, and finally locking the result with focused safety coverage.

## Archived Milestones

- [x] [`v0.5.0` backend-refactor milestone](/d:/projects/spoon/.planning/milestones/v0.5.0-ROADMAP.md) - shipped on 2026-03-31 with 6 completed phases and 29 executed plans.

## Active Milestone: v0.6.0 Backend Architecture Completion

- [x] **Phase 6: MSVC Seams and Ownership Completion** - Finish the app/backend seam for MSVC and split entry-point ownership cleanly into backend contracts.
- [ ] **Phase 7: Canonical MSVC State and Lifecycle** - Replace ad hoc MSVC models and flows with canonical backend state plus explicit lifecycle structure.
- [ ] **Phase 8: Shared Backend Contract Hardening** - Tighten backend event/error/fsx/path contracts that now affect both Scoop and MSVC.
- [ ] **Phase 9: MSVC and Shared Safety Net** - Add focused regression coverage so the MSVC cleanup and shared contract changes remain safe to evolve.

## Phase Details

### Phase 6: MSVC Seams and Ownership Completion
**Goal**: `spoon` becomes a thin app shell for MSVC in the same way it already is for Scoop: app code translates requests and results, while `spoon-backend` owns MSVC runtime behavior and context-sensitive logic.
**Depends on**: Phase 5
**Requirements**: MSVC-01, MSVC-04
**Success Criteria** (what must be TRUE):
  1. A developer can trigger MSVC operations and detail/status reads without app modules depending on MSVC module internals.
  2. Backend context and layout decisions used by live MSVC flows are explicit and auditable rather than partially reconstructed in the app.
  3. MSVC progress/result surfaces are emitted from backend-owned models and events that the app only translates.
**Plans**: 4 plans

### Phase 7: Canonical MSVC State and Lifecycle
**Goal**: `spoon-backend` owns one coherent MSVC state model and one explicit lifecycle model for install/update/remove/repair-style behavior.
**Depends on**: Phase 6
**Requirements**: MSVC-02, MSVC-03
**Success Criteria** (what must be TRUE):
  1. A developer can query MSVC state and inputs from one canonical backend model instead of scattered module-specific records.
  2. MSVC lifecycle behavior is split into focused backend stages instead of large mixed-responsibility flows.
  3. Side effects and persisted state transitions have clear ownership boundaries suitable for future diagnostics and repair.
**Plans**: 0 plans

### Phase 8: Shared Backend Contract Hardening
**Goal**: The backend contracts shared across runtime domains become stronger and more reusable, especially around events, errors, filesystem helpers, and runtime path handling.
**Depends on**: Phase 7
**Requirements**: BECT-01, BECT-02, BECT-03, BECT-04
**Success Criteria** (what must be TRUE):
  1. Event contracts no longer depend on fragile stringly conventions for core product semantics.
  2. Error handling expresses important backend failure classes without excessive fallback to generic buckets.
  3. Shared filesystem/path operations and production path resolution are centralized enough to reduce duplication and hardcoded runtime assumptions.
**Plans**: 0 plans

### Phase 9: MSVC and Shared Safety Net
**Goal**: The MSVC cleanup and shared backend contract hardening are protected by focused backend and app regressions.
**Depends on**: Phase 8
**Requirements**: TEST-04, TEST-05, TEST-06
**Success Criteria** (what must be TRUE):
  1. Risky MSVC lifecycle and state transitions are covered by backend-focused regression tests.
  2. App-side tests for MSVC/shared flows remain thin-shell tests rather than re-owning backend logic.
  3. A small amount of smoke coverage still exercises the most valuable integration seams without turning into coverage theater.
**Plans**: 0 plans

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 6. MSVC Seams and Ownership Completion | 4/4 | Complete | 2026-04-01 |
| 7. Canonical MSVC State and Lifecycle | 0/0 | Pending | - |
| 8. Shared Backend Contract Hardening | 0/0 | Pending | - |
| 9. MSVC and Shared Safety Net | 0/0 | Pending | - |

## Backlog

### Phase 999.1: Test Architecture Cleanup (BACKLOG)

**Goal:** Captured for future planning.  
**Requirements:** TBD  
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with `$gsd-review-backlog` when ready)
