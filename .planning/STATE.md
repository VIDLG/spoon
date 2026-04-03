---
gsd_state_version: 1.0
milestone: v0.7.0
milestone_name: Scoop Legacy Cleanup and Domain Refinement
status: active
stopped_at: Phase 12 planned
last_updated: "2026-04-03T01:00:00.000Z"
last_activity: 2026-04-01
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 12
  completed_plans: 8
  percent: 67
---

# Project State

## Project Reference

See: .planning/PROJECT.md

**Core value:** Make `spoon-backend` the single trusted backend core and keep `spoon` as the thin app shell that orchestrates and presents it.  
**Current focus:** Execute Phase 12 and remove low-value Scoop read-model and DTO redundancy.

## Current Position

Phase: 12. Scoop Read Model and Shared Cleanup Refinement  
Plan: Phase 12 planned  
Status: Ready to execute Phase 12  
Last activity: 2026-04-03 - Captured and planned the Scoop read-model cleanup phase

Progress: [######----] 67%

## Carried Context

### Archived Milestones

- [`v0.5.0-MILESTONE-AUDIT.md`](/d:/projects/spoon/.planning/v0.5.0-MILESTONE-AUDIT.md)
- [`v0.5.0-ROADMAP.md`](/d:/projects/spoon/.planning/milestones/v0.5.0-ROADMAP.md)
- [`v0.5.0-REQUIREMENTS.md`](/d:/projects/spoon/.planning/milestones/v0.5.0-REQUIREMENTS.md)
- [`v0.6.0-MILESTONE-AUDIT.md`](/d:/projects/spoon/.planning/v0.6.0-MILESTONE-AUDIT.md)
- [`v0.6.0-ROADMAP.md`](/d:/projects/spoon/.planning/milestones/v0.6.0-ROADMAP.md)
- [`v0.6.0-REQUIREMENTS.md`](/d:/projects/spoon/.planning/milestones/v0.6.0-REQUIREMENTS.md)

### Pending Follow-ups

- Consolidate remaining reusable filesystem helpers into [`fsx.rs`](/d:/projects/spoon/spoon-backend/src/fsx.rs). See [`2026-03-31-consolidate-remaining-fsx-helpers.md`](/d:/projects/spoon/.planning/todos/pending/2026-03-31-consolidate-remaining-fsx-helpers.md).
- Tighten the backend error contract further. See [`2026-03-31-tighten-backend-error-contract.md`](/d:/projects/spoon/.planning/todos/pending/2026-03-31-tighten-backend-error-contract.md).
- Remove remaining hardcoded production paths from backend runtime execution. See [`2026-03-31-remove-hardcoded-production-paths.md`](/d:/projects/spoon/.planning/todos/pending/2026-03-31-remove-hardcoded-production-paths.md).
- Continue derive-not-store cleanup for lower-value read models. See [`2026-04-01-audit-derive-not-store-fields.md`](/d:/projects/spoon/.planning/todos/pending/2026-04-01-audit-derive-not-store-fields.md).
- Simplify `SystemPort` / `ScoopRuntimeHost` boundaries further. See [`2026-04-01-simplify-system-port-and-runtime-host-boundaries.md`](/d:/projects/spoon/.planning/todos/pending/2026-04-01-simplify-system-port-and-runtime-host-boundaries.md).
- Review backlog phase [`999.1-test-architecture-cleanup`](/d:/projects/spoon/.planning/phases/999.1-test-architecture-cleanup).

### Seeds

- [`SEED-001-backend-event-contract-hardening.md`](/d:/projects/spoon/.planning/seeds/SEED-001-backend-event-contract-hardening.md)
- [`SEED-002-async-zip-backend-evaluation.md`](/d:/projects/spoon/.planning/seeds/SEED-002-async-zip-backend-evaluation.md)

### Guardrails

- Keep the next milestone focused on Scoop cleanup rather than reopening a full cross-domain architecture wave.
- Prefer targeted cleanup of outdated or poorly shaped Scoop code over generalized churn.

## Session Continuity

Last session: 2026-04-03T01:00:00.000Z  
Stopped at: Phase 12 planned; next step is execution  
Resume file: .planning/phases/12-scoop-read-model-and-shared-cleanup-refinement/12-01-PLAN.md
