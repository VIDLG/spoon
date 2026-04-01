# Phase 7: Canonical MSVC State and Lifecycle - Research

**Researched:** 2026-04-01
**Domain:** Turning the new MSVC seam skeleton into a canonical SQLite-backed backend state machine
**Confidence:** HIGH

## Summary

Phase 6 established the shape of the MSVC domain without trying to fully rewrite its runtime semantics. That makes Phase 7 the right place to connect MSVC to the same backend architecture principles already proven on the Scoop side:

- one control plane
- one canonical state model
- one shared lifecycle contract
- app-shell translation only

The existing code strongly suggests a phased landing strategy:

1. extend the SQLite control plane to represent canonical MSVC state and lifecycle residue
2. keep one shared high-level lifecycle language (`planned / detecting / resolving / executing / validating / state_committing / completed`)
3. move `managed` and `official` behind strategy-specific execution/validation branches
4. update query/status/doctor to use canonical state plus evidence-backed reconciliation

The main design risk is storing the wrong things. The derive-not-store audit is especially important here:

- layout-derived absolute paths should not become persistent canonical facts
- raw detection output should not be dumped into state just because it is easy
- count-like and convenience fields should not be promoted into canonical schema without strong justification

The right target is a control-plane record that captures backend-trusted facts and lifecycle semantics, while leaving derivable or noisy data out.

The second risk is treating `official` as too special. Phase 7 should respect that official execution goes through an external installer, but it still should not remain architecturally outside the domain. Instead:

- canonical state remains authoritative
- official detection and validation become evidence used to establish, refresh, and reconcile that state

That produces a backend model that can support future doctor/repair work without making every query path do a full fresh machine probe.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| MSVC-02 | Canonical backend-owned MSVC state | Add SQLite-backed canonical state with one envelope and runtime-specific detail. |
| MSVC-03 | Explicit lifecycle model for MSVC operations | Land shared lifecycle contract and strategy-specific execution/validation branches. |

## Recommended Plan Order

1. Extend SQLite control-plane schema/store for MSVC canonical state and lifecycle facts
2. Formalize lifecycle contract execution for `managed`
3. Integrate `official` into the same high-level lifecycle and canonical-state reconciliation
4. Align query/status/doctor and lock with focused regressions

## Current Code Reality

- Phase 6 already separated `plan`, `detect`, `query`, and `execute`, which is the right staging point for a canonical-state/lifecycle phase.
- Managed runtime still writes local runtime/install JSON, so that legacy contract should be replaced or subordinated.
- Official runtime still depends on external detection and bootstrapper behavior, so evidence-backed reconciliation will matter more here than on the managed side.

## Validation Focus

- SQLite-backed canonical MSVC state exists and is read/write tested.
- Managed and official operations both use the shared lifecycle contract language.
- Status/doctor/query read from canonical state plus evidence-backed reconciliation rather than ad hoc local assumptions.
- The app shell remains thin while MSVC backend behavior deepens.

## Sources

- `.planning/phases/07-canonical-msvc-state-and-lifecycle/07-CONTEXT.md`
- `.planning/phases/06-msvc-seams-and-ownership-completion/06-VERIFICATION.md`
- `.planning/todos/pending/2026-04-01-audit-derive-not-store-fields.md`
- `spoon-backend/src/control_plane/schema/0001_control_plane.sql`
- `spoon-backend/src/control_plane/sqlite.rs`
- `spoon-backend/src/msvc/plan.rs`
- `spoon-backend/src/msvc/detect.rs`
- `spoon-backend/src/msvc/execute.rs`
- `spoon-backend/src/msvc/official.rs`
- `spoon-backend/src/msvc/query.rs`
- `spoon-backend/src/msvc/status.rs`
- `spoon-backend/src/msvc/rules.rs`
