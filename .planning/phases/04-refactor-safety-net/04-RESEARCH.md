# Phase 4: Refactor Safety Net - Research

**Researched:** 2026-03-30
**Domain:** Lifecycle-risk regression coverage after backend seam consolidation, canonical state, SQLite control plane, and lifecycle split
**Confidence:** HIGH

## Summary

Phase 4 should not be planned like a generic “add more tests” phase. The codebase already has enough structure from Phases 2, 02.1, and 3 to place safety coverage very deliberately. Canonical installed state is already centralized, lifecycle stages are now explicit, SQLite control-plane semantics already exist, and the app shell has been narrowed to translation-oriented behavior. That means the most valuable safety net is no longer broad end-to-end duplication; it is targeted protection around the exact failure boundaries that can still leave users in bad states.

The codebase concerns and testing map both point to the same highest-value gap: Scoop lifecycle failure paths remain more dangerous than ordinary success paths. The project already has success and contract regressions in `spoon-backend/src/scoop/tests/runtime.rs`, `spoon/tests/cli/scoop_runtime_flow.rs`, and `spoon/tests/cli/status_backend_flow.rs`, but the most important remaining risk is what happens when a lifecycle stage fails after prior state mutation has already begun. Phase 4 should therefore anchor on failure-boundary tests around hook failures, persist restoration failures, surface failures, integration failures, pre-commit failures, uninstall warning-only tails, and lock/journal stop points.

The best layering is backend-heavy. The repository instructions explicitly prefer backend behavior tests in `spoon-backend` and app tests focused on shell flows. That fits the architecture decisions from earlier phases: backend owns lifecycle correctness, control-plane truth, and recoverable boundaries; the app shell only translates backend events/results. Therefore, app tests should not become a second backend oracle. They should only assert that backend lifecycle events, outcomes, and warnings are rendered or routed correctly at the CLI/TUI boundary.

Phase 4 should also stay disciplined about repair/retry scope. The lifecycle stage contract from Phase 3 already created the substrate for future repair logic, and Phase 02.1 already provided the SQLite journal/lock/doctor plane. The correct Phase 4 move is to verify diagnosability, stop-point correctness, and contract stability, not to build a full automatic repair system. That keeps the phase aligned with the roadmap goal of hardening the refactor instead of opening a new architecture branch.

Finally, real integration tests still have value, but only in the same narrow role described in `AGENTS.md`: isolated, temporary, opt-in smoke coverage. They are best used to retain one or two high-confidence real-world Scoop bucket/package flows behind ignored gates, not to become the main regression strategy. The main safety net should remain focused backend tests plus a thin layer of app-shell translation regressions.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| TEST-01 | Protect critical install/update/uninstall failure handling with backend tests. | Prioritize lifecycle failure-boundary and stop-point coverage in `spoon-backend`. |
| TEST-02 | Keep `spoon` tests focused on routing and orchestration shell behavior. | Add or update CLI/TUI translation tests without re-encoding backend correctness there. |
| TEST-03 | New backend contracts get nearby focused tests. | Add tests near lifecycle, control-plane, and event translation code rather than relying only on broad flows. |

## Recommended Plan Order

1. Backend near-module failure contracts
2. Backend integration coverage for journal/lock/doctor and recoverable boundaries
3. App-shell CLI/TUI translation regressions
4. Sparse opt-in real smoke and verification consolidation

## Current Code Reality

- `spoon-backend/src/scoop/tests/runtime.rs` already captures the lifecycle contract and is the natural home for new failure-boundary regressions.
- `spoon-backend/src/tests/control_plane.rs` and control-plane modules provide anchors for journal/lock/doctor verification.
- `spoon/tests/cli/scoop_runtime_flow.rs` and `spoon/tests/cli/status_backend_flow.rs` already prove the app shell can consume backend outcomes and stage events.
- Existing ignored real flows in `spoon/tests/cli/scoop_runtime_flow.rs` and related tests are appropriate candidates for limited smoke retention.

## Validation Focus

- Fatal lifecycle boundaries stop at the correct stage and do not silently over-commit state.
- Warning-only uninstall tails remain warning-only.
- Lock conflicts and journal stop points remain observable and diagnosable.
- App-shell regressions confirm translation, not backend re-implementation.
- Ignored real flows remain sparse and intentional.

## Sources

- `.planning/phases/04-refactor-safety-net/04-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/codebase/TESTING.md`
- `.planning/codebase/CONCERNS.md`
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-VERIFICATION.md`
- `AGENTS.md`
- `spoon-backend/src/scoop/tests/runtime.rs`
- `spoon-backend/src/tests/control_plane.rs`
- `spoon/tests/cli/scoop_runtime_flow.rs`
- `spoon/tests/cli/status_backend_flow.rs`

