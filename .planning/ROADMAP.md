# Roadmap: Spoon Backend Refactoring

## Overview

This roadmap turns `spoon` into a thin app shell over `spoon-backend` by fixing backend ownership first, then consolidating duplicated Scoop state, then splitting Scoop lifecycle behavior into backend-owned phases, and finally hardening the refactor with focused safety coverage.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Backend Seams and Ownership** - Move runtime ownership, layout/context derivation, and Git responsibilities behind backend contracts.
- [ ] **Phase 2: Canonical Scoop State** - Replace duplicated Scoop state with one backend-owned source of truth and shared read models.
- [ ] **Phase 3: Scoop Lifecycle Split and App Thinning** - Rebuild install/update/uninstall around explicit backend lifecycle phases and thin the app layer.
- [ ] **Phase 4: Refactor Safety Net** - Lock in the refactor with focused backend and app tests around the risky paths.

## Phase Details

### Phase 1: Backend Seams and Ownership
**Goal**: `spoon` becomes a thin app shell that invokes backend-owned Scoop, Git, MSVC, and layout/context behavior through explicit backend contracts.
**Depends on**: Nothing (first phase)
**Requirements**: BNDR-01, BNDR-02, BNDR-03, BNDR-04, BNDR-05, GIT-01, GIT-02, GIT-03, LAY-01, LAY-02, LAY-03
**Success Criteria** (what must be TRUE):
  1. A developer can run Spoon Scoop actions without `spoon` directly calling Scoop runtime details; the app routes requests to backend operation interfaces and renders backend results.
  2. A developer can run bucket clone and sync flows from Spoon without `spoon` depending on `gix`; backend-owned Git interfaces emit the progress and results the app displays.
  3. Changing Spoon's configured `root` changes Scoop, MSVC, and shared shim/state behavior consistently because backend layout derivation owns the runtime path model and the app only passes configuration/context.
  4. The app can render tool and runtime status from backend query/result models without rereading backend state files or reconstructing backend behavior locally.
**Plans**: 7 plans

Plans:
- [x] 01-01-PLAN.md - Define backend context, runtime layout, and split-port ownership contracts.
- [x] 01-02-PLAN.md - Move backend Scoop runtime and bucket contracts behind explicit backend context.
- [x] 01-03-PLAN.md - Replace MSVC global runtime config with explicit backend context requests.
- [x] 01-04-PLAN.md - Switch status, JSON, and TUI refresh surfaces to backend read models.
- [x] 01-05-PLAN.md - Thin Spoon Scoop runtime, package, and bucket adapters to backend request/response mapping.
- [x] 01-06-PLAN.md - Finish detail, prefix, and config surface cleanup around backend read and layout models.
- [x] 01-07-PLAN.md - Remove dead backend-path re-exports and drop the app-side `gix` dependency.

### Phase 2: Canonical Scoop State
**Goal**: `spoon-backend` owns one canonical Scoop installed-state model and one persisted source of truth for installed package facts.
**Depends on**: Phase 1
**Requirements**: SCST-01, SCST-02, SCST-03, SCST-04
**Success Criteria** (what must be TRUE):
  1. A developer can query package details, installed status, uninstall inputs, and reapply inputs from one backend-owned Scoop state model.
  2. State written by the backend contains only non-derivable Scoop facts, so layout-derived absolute paths are reconstructed from backend context instead of being duplicated in persisted state.
  3. Package list, status, and detail views stay consistent because every backend read model projects from the same canonical installed-state source.
  4. `spoon-backend/src/scoop/` no longer carries parallel Scoop state model definitions for the same installed-package facts.
**Plans**: TBD

### Phase 3: Scoop Lifecycle Split and App Thinning
**Goal**: `spoon-backend` owns a single explicit Scoop lifecycle for install, update, uninstall, reapply, persist, and hooks, while `spoon` only triggers operations and shows progress.
**Depends on**: Phase 2
**Requirements**: SCLF-01, SCLF-02, SCLF-03, SCLF-04, SCLF-05
**Success Criteria** (what must be TRUE):
  1. A developer can run Scoop install through one backend lifecycle entry point that executes named phases instead of a monolithic flow.
  2. A developer can run Scoop update through the same backend lifecycle model without app-side orchestration gaps or patch logic.
  3. A developer can run Scoop uninstall through the same backend lifecycle model without app-side orchestration gaps or patch logic.
  4. Reapply, persist restore/sync, and hook execution are coordinated from backend lifecycle entry points instead of ad hoc app or service calls.
  5. Lifecycle behavior is split into focused backend modules so failures and future changes localize to planner, acquire, surface, persist, and hook-style boundaries rather than one giant controller.
**Plans**: TBD

### Phase 4: Refactor Safety Net
**Goal**: The refactor is protected by focused backend and app tests so risky Scoop behavior can change safely.
**Depends on**: Phase 3
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. Users do not lose key install, update, or uninstall failure handling during the refactor because backend tests cover the critical failure paths for those operations.
  2. CLI and TUI regressions are caught at the app-shell level because `spoon` tests stay focused on routing and orchestration instead of re-owning backend internals.
  3. New or changed backend contracts introduced by the refactor have focused nearby tests, so failures surface at the backend responsibility that changed instead of only in broad end-to-end coverage.
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Backend Seams and Ownership | 0/7 | Not started | - |
| 2. Canonical Scoop State | 0/TBD | Not started | - |
| 3. Scoop Lifecycle Split and App Thinning | 0/TBD | Not started | - |
| 4. Refactor Safety Net | 0/TBD | Not started | - |
