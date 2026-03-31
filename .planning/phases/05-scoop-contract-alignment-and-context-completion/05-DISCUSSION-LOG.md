# Phase 5: Scoop Contract Alignment and Context Completion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-03-31
**Phase:** 05-scoop-contract-alignment-and-context-completion
**Areas discussed:** scope size, Scoop `BackendContext` seam depth, stale regression handling

---

## Scope Size

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal gap-closure phase | Only close milestone audit blockers; do not continue general Scoop cleanup | x |
| Medium follow-up cleanup | Fix blockers plus additional nearby Scoop cleanup | |
| Large continuation phase | Reopen Scoop architecture work broadly | |

**User's choice:** Minimal gap-closure phase.
**Notes:** This phase exists to unblock milestone archive, not to become a second Phase 3.

---

## Scoop `BackendContext` Seam

| Option | Description | Selected |
|--------|-------------|----------|
| Improve only until audit no longer reports `LAY-03` partial | Clarify/complete the seam enough for the blocker to disappear | x |
| Fully mirror MSVC style | Force Scoop into the same context-first surface shape everywhere | |
| Ignore seam and only fix tests | Leave the partial seam in place | |

**User's choice:** Improve only until audit no longer reports `LAY-03` partial.
**Notes:** The goal is audit closure and seam clarity, not symmetry for symmetry's sake.

---

## Stale Regression Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Migrate stale tests to the current contract | Keep user-intent regressions, but assert the SQLite/canonical reality | x |
| Delete and replace old tests wholesale | Rebuild the suite from scratch | |
| Preserve legacy tests as compatibility checks | Keep both worlds in the suite | |

**User's choice:** Migrate stale tests to the current contract.
**Notes:** The old regressions still describe useful user behavior; only their contract assumptions are stale.

---

## the agent's Discretion

- Planning may choose the exact split between backend test migration and app seam clarification, as long as it stays narrow and directly tied to the audit blockers.

## Deferred Ideas

- Broader Scoop cleanup beyond the milestone blockers.
