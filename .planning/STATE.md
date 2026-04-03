---
gsd_state_version: 1.0
milestone: v0.7.0
milestone_name: Scoop Legacy Cleanup and Domain Refinement
status: active
stopped_at: Phase 12.1 planned
last_updated: "2026-04-03T09:45:00.000Z"
last_activity: 2026-04-01
progress:
  total_phases: 5
  completed_phases: 3
  total_plans: 12
  completed_plans: 12
  percent: 75
---

# Project State

## Project Reference

See: .planning/PROJECT.md

**Core value:** Make `spoon-backend` the single trusted backend core and keep `spoon` as the thin app shell that orchestrates and presents it.  
**Current focus:** Execute the inserted control-plane simplification phase before the final safety-net phase.

## Current Position

Phase: 12.1. Control Plane Simplification and Migration Hardening  
Plan: Phase 12.1 planned  
Status: Ready to execute Phase 12.1  
Last activity: 2026-04-03 - Planned the inserted control-plane simplification phase

Progress: [########--] 75%

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

### Roadmap Evolution

- Phase 12.1 inserted after Phase 12: Control Plane Simplification and Migration Hardening (URGENT)

## Session Continuity

Last session: 2026-04-03T09:45:00.000Z  
Stopped at: Phase 12.1 planned; next step is execution  
Resume file: .planning/phases/12.1-control-plane-simplification-and-migration-hardening/12.1-01-PLAN.md
