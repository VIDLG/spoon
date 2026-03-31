# Phase 4: Refactor Safety Net - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 04-refactor-safety-net
**Areas discussed:** testing weight, failure-path scope, test layering, repair/retry boundary, real integration scope

---

## Testing Weight

| Option | Description | Selected |
|--------|-------------|----------|
| Backend-heavy with thin app-shell coverage | Backend owns lifecycle risk tests; app focuses on translation/orchestration; real integration remains sparse | x |
| End-to-end heavy | App/CLI/TUI flows become the primary safety net | |
| Mostly unit-only | Minimize app coverage and rely almost entirely on low-level backend tests | |

**User's choice:** Backend-heavy with thin app-shell coverage.
**Notes:** This keeps backend correctness close to the code that owns lifecycle behavior and prevents the app shell from retaking backend ownership through tests.

---

## Failure Coverage Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Key failure boundaries only | Focus on the highest-risk lifecycle stop points rather than duplicating every stage with success/failure pairs | x |
| Exhaustive per-stage matrix | Test every stage in both success and failure modes | |
| Minimal smoke failures only | Cover only a few representative failure paths | |

**User's choice:** Key failure boundaries only.
**Notes:** Priority failures include hook failures, `persist_restoring`, `surface_applying`, `integrating`, pre-`state_committing` failures, uninstall fatal/warning boundaries, lock conflicts, and journal stop points.

---

## Test Layering

| Option | Description | Selected |
|--------|-------------|----------|
| Backend near-module + backend integration, thin app tests | Backend localizes correctness; app only checks translation/orchestration shells | x |
| Mostly backend integration | Prefer fewer unit tests and more end-to-end backend composition tests | |

**User's choice:** Backend near-module + backend integration, thin app tests.
**Notes:** App tests should not become the place that re-verifies backend lifecycle semantics.

---

## Repair / Retry Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Verify recoverable boundaries, no full repair engine | Use Phase 4 to prove failures are diagnosable and journal/doctor semantics are stable | x |
| Begin automatic retry/repair implementation | Expand Phase 4 into a repair subsystem | |
| Ignore repair and doctor semantics | Treat safety net as pure test expansion | |

**User's choice:** Verify recoverable boundaries, no full repair engine.
**Notes:** Phase 4 should validate diagnosability and recoverable-boundary contracts without opening a new architecture phase.

---

## Real Integration Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Sparse opt-in real integration smoke | Keep a small number of isolated real flows behind ignored/opt-in gates | x |
| Expand real integration substantially | Use many live environment tests as the main regression net | |
| Remove real integration almost entirely | Rely only on harness and backend integration coverage | |

**User's choice:** Sparse opt-in real integration smoke.
**Notes:** Real flows remain useful as smoke coverage, but not as the main safety strategy.

---

## the agent's Discretion

- Planning may choose the exact balance of backend unit, backend integration, CLI, TUI, and opt-in real integration checks as long as backend-weighted safety coverage remains the primary strategy.

## Deferred Ideas

- Full automatic retry/repair workflows.
- Large PTY-heavy end-to-end expansion.
