# Phase 9: MSVC and Shared Safety Net - Research

**Researched:** 2026-04-01
**Domain:** Safety-net hardening for the MSVC and shared backend contract milestone
**Confidence:** HIGH

## Summary

By the time Phase 9 begins, the architecture work is no longer speculative. That means the safety-net phase should resist two temptations:

1. turning into another architecture phase
2. turning into a giant test-coverage exercise

The right safety-net shape here is backend-heavy and breakpoint-oriented. The most valuable failures to lock are the ones that could silently corrupt or desynchronize:

- managed lifecycle failures around download/extract/materialize/state write
- validation failures that should not leave misleading state
- official bootstrapper failures and detect/reconcile mismatches
- uninstall paths that should clear canonical state
- shared contract regressions where event/error/path cleanup could drift silently

The app layer should stay thin in tests just as in implementation. That means app tests are most useful when they verify:

- translation of backend events
- translation of backend outcomes
- output/shell continuity across backend contract changes

Real smoke remains valuable in narrow opt-in form, especially for MSVC, but should remain secondary to deterministic backend/app regressions.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| TEST-04 | Backend lifecycle/shared-contract regressions | Add focused backend tests around risky failure boundaries. |
| TEST-05 | App-shell translation/orchestration regressions | Keep app tests thin and translation-oriented. |
| TEST-06 | Narrow real smoke | Retain a small amount of opt-in real smoke where it adds confidence. |

## Recommended Plan Order

1. Backend failure-boundary regressions for managed + official lifecycle/state
2. Shared contract regression coverage near backend modules
3. App-shell translation regressions for MSVC/shared contracts
4. Real smoke curation and phase verification

## Current Code Reality

- MSVC now has enough structure that failure-boundary tests can target real seam/lifecycle/state behavior without giant harnesses.
- Shared contracts (event, error, path, ports, download/archive primitives) were materially changed in Phase 8 and now deserve explicit protection.
- The app shell remains a good place for translation regressions but not for backend correctness testing.

## Validation Focus

- Canonical-state/lifecycle failure boundaries are locked.
- Shared contract regressions are caught close to the code that changed.
- App shell still consumes backend contracts instead of reinventing them.
- Real smoke remains sparse and intentional.

## Sources

- `.planning/phases/09-msvc-and-shared-safety-net/09-CONTEXT.md`
- `.planning/phases/07-canonical-msvc-state-and-lifecycle/07-VERIFICATION.md`
- `.planning/phases/08-shared-backend-contract-hardening/08-VERIFICATION.md`
- `spoon-backend/src/msvc/tests/context.rs`
- `spoon-backend/src/msvc/tests/root.rs`
- `spoon-backend/src/msvc/tests/official.rs`
- `spoon-backend/src/tests/event.rs`
- `spoon/tests/cli/msvc_flow.rs`
- `spoon/tests/cli/status_backend_flow.rs`
- `spoon/tests/cli/scoop_flow.rs`
- `spoon/tests/tui/tui_msvc_download_flow.rs`
