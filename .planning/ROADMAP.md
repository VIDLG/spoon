# Roadmap: Spoon Backend Refactoring

## Overview

This roadmap turns `spoon` into a thin app shell over `spoon-backend` by fixing backend ownership first, then consolidating duplicated Scoop state, then splitting Scoop lifecycle behavior into backend-owned phases, and finally hardening the refactor with focused safety coverage.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Backend Seams and Ownership** - Move runtime ownership, layout/context derivation, and Git responsibilities behind backend contracts.
- [x] **Phase 2: Canonical Scoop State** - Replace duplicated Scoop state with one backend-owned source of truth and shared read models.
- [x] **Phase 3: Scoop Lifecycle Split and App Thinning** - Rebuild install/update/uninstall around explicit backend lifecycle phases and thin the app layer.
- [x] **Phase 4: Refactor Safety Net** - Lock in the refactor with focused backend and app tests around the risky paths.
- [x] **Phase 5: Scoop Contract Alignment and Context Completion** - Close audit blockers by aligning stale Scoop regressions and remaining context seams with the SQLite-backed canonical contract.

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
**Plans**: 8 plans

Plans:
- [x] 01-01-PLAN.md - Define backend context, runtime layout, and split-port ownership contracts.
- [x] 01-02-PLAN.md - Move backend Scoop runtime and bucket contracts behind explicit backend context.
- [x] 01-03-PLAN.md - Replace MSVC global runtime config with explicit backend context requests.
- [x] 01-04-PLAN.md - Switch status, JSON, and TUI refresh surfaces to backend read models.
- [x] 01-05-PLAN.md - Thin Spoon Scoop runtime, package, and bucket adapters to backend request/response mapping.
- [x] 01-06-PLAN.md - Finish detail, prefix, and config surface cleanup around backend read and layout models.
- [x] 01-07-PLAN.md - Remove dead backend-path re-exports and drop the app-side `gix` dependency.
- [x] 01-08-PLAN.md - Migrate remaining app modules from config path helpers to RuntimeLayout (gap closure for BNDR-04/LAY-01).

### Phase 2: Canonical Scoop State
**Goal**: `spoon-backend` owns one canonical Scoop installed-state model and one persisted source of truth for installed package facts.
**Depends on**: Phase 1
**Requirements**: SCST-01, SCST-02, SCST-03, SCST-04
**Success Criteria** (what must be TRUE):
  1. A developer can query package details, installed status, uninstall inputs, and reapply inputs from one backend-owned Scoop state model.
  2. State written by the backend contains only non-derivable Scoop facts, so layout-derived absolute paths are reconstructed from backend context instead of being duplicated in persisted state.
  3. Package list, status, and detail views stay consistent because every backend read model projects from the same canonical installed-state source.
  4. `spoon-backend/src/scoop/` no longer carries parallel Scoop state model definitions for the same installed-package facts.
**Plans**: 5/5 plans executed

Plans:
- [x] 02-01-PLAN.md - Introduce `scoop/state/` and make `InstalledPackageState` the canonical persisted record.
- [x] 02-02-PLAN.md - Update runtime writes and reapply/uninstall inputs to use canonical state with `bucket` and `architecture`.
- [x] 02-03-PLAN.md - Move query and runtime-status surfaces onto canonical store enumeration and typed projections.
- [x] 02-04-PLAN.md - Rebuild package info and operation outcomes from typed canonical state projections.
- [x] 02-05-PLAN.md - Remove legacy `ScoopPackageState` APIs and report stale flat state explicitly.

### Phase 02.1: SQLite Control Plane and Sync-Async Boundary (INSERTED)

**Goal:** `spoon-backend` moves its control-plane metadata from JSON files to SQLite while keeping the filesystem as the runtime data plane and preserving a sync-core / async-edge architecture boundary.
**Requirements**: SQLCP-01, SQLCP-02, SQLCP-03, SQLCP-04, SQLCP-05
**Depends on:** Phase 2
**Success Criteria** (what must be TRUE):
  1. A developer can inspect installed package state, operation journal state, doctor/repair state, and bucket registry metadata from one SQLite-backed control plane instead of scattered JSON control files.
  2. Runtime paths, install roots, `current`, persist contents, shims, shortcuts, cache, and bucket repositories remain filesystem/layout-derived rather than being re-homed into the database.
  3. Backend business-rule modules remain sync-core and driver-free, while `rusqlite` plus repo-owned tokio bridging is contained inside store/repository edges.
  4. The project cuts over directly to SQLite control-plane state without long-lived JSON compatibility layers; stale old JSON state is surfaced explicitly for manual repair or cleanup.
**Plans:** 4/4 plans executed

Plans:
- [x] 02.1-01-PLAN.md - Add SQLite control-plane infrastructure, migrations, and async store façades.
- [x] 02.1-02-PLAN.md - Move installed state and bucket registry metadata into SQLite while keeping filesystem runtime artifacts.
- [x] 02.1-03-PLAN.md - Add operation journal, lock state, and doctor/repair metadata stores with sync-core / async-edge boundaries.
- [x] 02.1-04-PLAN.md - Cut Scoop read/write paths directly to SQLite control-plane state and report legacy JSON for manual repair.

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
**Plans**: 5/5 plans executed

Plans:
- [x] 03-01-PLAN.md - Define the formal lifecycle stage/event/journal contract.
- [x] 03-02-PLAN.md - Extract planner/acquire/materialize for install/update.
- [x] 03-03-PLAN.md - Extract persist/surface/integrate/state for the shared back half.
- [x] 03-04-PLAN.md - Extract uninstall/reapply over the shared lifecycle contract and centralized hooks.
- [x] 03-05-PLAN.md - Thin the app shell to request/event/outcome translation only.

### Phase 4: Refactor Safety Net
**Goal**: The refactor is protected by focused backend and app tests so risky Scoop behavior can change safely.
**Depends on**: Phase 3
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. Users do not lose key install, update, or uninstall failure handling during the refactor because backend tests cover the critical failure paths for those operations.
  2. CLI and TUI regressions are caught at the app-shell level because `spoon` tests stay focused on routing and orchestration instead of re-owning backend internals.
  3. New or changed backend contracts introduced by the refactor have focused nearby tests, so failures surface at the backend responsibility that changed instead of only in broad end-to-end coverage.
**Plans**: 4 plans

### Phase 5: Scoop Contract Alignment and Context Completion
**Goal**: Close milestone audit blockers by aligning stale Scoop tests and remaining app/backend context seams with the SQLite-backed canonical contract that the backend now owns.
**Depends on**: Phase 4
**Requirements**: LAY-03, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. CLI and backend Scoop regressions no longer seed or expect legacy `scoop/state/packages/*.json` state files; they verify the SQLite-backed canonical/control-plane contract instead.
  2. Backend runtime write regressions validate control-plane persistence rather than removed JSON package-state files.
  3. The remaining partial Scoop `BackendContext` seam is either completed end-to-end or explicitly removed so app/backend ownership is no longer ambiguous.
  4. Milestone audit blockers are cleared and the milestone can be safely re-audited for archive.
**Plans**: 3/3 plans executed

Plans:
- [x] 05-01-PLAN.md - Align stale backend Scoop regressions with SQLite/canonical state.
- [x] 05-02-PLAN.md - Align stale CLI Scoop regressions with backend canonical/store APIs.
- [x] 05-03-PLAN.md - Clarify the remaining Scoop context seam and prepare the milestone for re-audit.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Backend Seams and Ownership | 8/8 | Complete | 2026-03-28 |
| 2. Canonical Scoop State | 5/5 | Complete | 2026-03-29 |
| 2.1. SQLite Control Plane and Sync-Async Boundary | 4/4 | Complete | 2026-03-29 |
| 3. Scoop Lifecycle Split and App Thinning | 5/5 | Complete | 2026-03-29 |
| 4. Refactor Safety Net | 4/4 | Complete | 2026-03-31 |
| 5. Scoop Contract Alignment and Context Completion | 3/3 | Complete | 2026-03-31 |

## Backlog

### Phase 999.1: Test Architecture Cleanup (BACKLOG)

**Goal:** Captured for future planning.
**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with `$gsd-review-backlog` when ready)
