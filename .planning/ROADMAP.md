# Roadmap: Scoop Legacy Cleanup and Domain Refinement

## Overview

This roadmap continues phase numbering after `v0.6.0` and focuses on cleaning the remaining outdated, duplicated, or poorly shaped code in `spoon-backend/src/scoop/` while keeping the established backend architecture intact.

## Archived Milestones

- [x] [`v0.5.0` backend-refactor milestone](/d:/projects/spoon/.planning/milestones/v0.5.0-ROADMAP.md) - shipped on 2026-03-31 with 6 completed phases and 29 executed plans.
- [x] [`v0.6.0` backend-architecture-completion milestone](/d:/projects/spoon/.planning/milestones/v0.6.0-ROADMAP.md) - shipped on 2026-04-01 with 4 completed phases and 16 executed plans.

## Active Milestone: v0.7.0 Scoop Legacy Cleanup and Domain Refinement

- [x] **Phase 10: Scoop Legacy Path and State Cleanup** - Remove or isolate JSON-era and deprecated Scoop path/state concepts from active runtime paths.
- [x] **Phase 11: Scoop Runtime Host and Helper Consolidation** - Simplify stale helper layers and runtime-host seams inside the Scoop backend domain.
- [x] **Phase 12: Scoop Read Model and Shared Cleanup Refinement** - Remove low-value redundancy and finish adjacent shared cleanup required by the Scoop pass.
- [ ] **Phase 13: Scoop Cleanup Safety Net Refresh** - Refresh backend/app regressions so the Scoop legacy cleanup remains safe to evolve.

## Phase Details

### Phase 10: Scoop Legacy Path and State Cleanup
**Goal**: remove or isolate stale JSON-era path/state assumptions so active Scoop runtime behavior fully reflects the SQLite-backed control plane and current layout model.
**Depends on**: Phase 9
**Requirements**: SLEG-01, SLEG-04
**Success Criteria** (what must be TRUE):
  1. Active Scoop runtime logic no longer relies on stale JSON-era package-state or bucket-registry concepts except for explicit legacy detection/repair surfaces.
  2. Layout/path helpers inside the Scoop domain better reflect the current control-plane reality.
**Plans**: 4 plans

### Phase 11: Scoop Runtime Host and Helper Consolidation
**Goal**: simplify the Scoop backend domain by reducing stale helper layers, duplicated host seams, and ambiguous runtime responsibilities.
**Depends on**: Phase 10
**Requirements**: SLEG-02, BECT-05
**Success Criteria** (what must be TRUE):
  1. Scoop runtime helper layering is simpler and more explicit.
  2. Remaining host/runtime seam duplication inside the Scoop domain is materially reduced.
**Plans**: 4 plans

### Phase 12: Scoop Read Model and Shared Cleanup Refinement
**Goal**: remove low-value read-model redundancy and finish the shared cleanup that the Scoop pass directly depends on.
**Depends on**: Phase 11
**Requirements**: SLEG-03, BECT-06
**Success Criteria** (what must be TRUE):
  1. Scoop read models avoid low-value derivable redundancy when it adds no meaningful contract value.
  2. The adjacent shared cleanup touched by the Scoop pass stays aligned with backend-owned contract rules.
**Plans**: 4 plans

### Phase 12.1: Control Plane Simplification and Migration Hardening (INSERTED)

**Goal:** simplify the control-plane boundary so it is path-first rather than layout-first, and harden the migration story without overcomplicating the backend.
**Requirements**: BECT-07, BECT-08
**Depends on:** Phase 12
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 12.1 to break down)

### Phase 13: Scoop Cleanup Safety Net Refresh
**Goal**: protect the Scoop legacy cleanup with focused backend and app-shell regressions.
**Depends on**: Phase 12.1
**Requirements**: TEST-07, TEST-08
**Success Criteria** (what must be TRUE):
  1. Backend tests protect the Scoop legacy-cleanup breakpoints close to where the cleanup happened.
  2. App-shell Scoop tests remain thin but still catch user-visible regressions caused by the cleanup.
**Plans**: 0 plans

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 10. Scoop Legacy Path and State Cleanup | 4/4 | Complete | 2026-04-01 |
| 11. Scoop Runtime Host and Helper Consolidation | 4/4 | Complete | 2026-04-03 |
| 12. Scoop Read Model and Shared Cleanup Refinement | 4/4 | Complete | 2026-04-03 |
| 12.1. Control Plane Simplification and Migration Hardening | 0/0 | Inserted | - |
| 13. Scoop Cleanup Safety Net Refresh | 0/0 | Pending | - |

## Backlog

### Phase 999.1: Test Architecture Cleanup (BACKLOG)

**Goal:** Captured for future planning.  
**Requirements:** TBD  
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with `$gsd-review-backlog` when ready)
