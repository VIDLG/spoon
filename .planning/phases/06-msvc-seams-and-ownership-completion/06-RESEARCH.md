# Phase 6: MSVC Seams and Ownership Completion - Research

**Researched:** 2026-04-01
**Domain:** MSVC backend domain restructuring after the Scoop/backend-refactor milestone
**Confidence:** HIGH

## Summary

MSVC is now the largest remaining backend domain that has backend ownership in principle but not yet the same architectural clarity that Scoop gained in `v0.5.0`. The current code already hints at the right direction: app-side MSVC entry points build explicit backend context, backend status already projects both managed and official runtimes into one result, and `official.rs` already reads as an alternate strategy. But the overall shape is still uneven because `msvc/mod.rs` acts as a large managed-runtime controller, shared contracts are not yet explicit, and the app surface still leaks a split managed/official entry layout.

That means Phase 6 should not try to finish MSVC in one pass. Instead, it should establish the domain skeleton that later phases can safely deepen:

1. define MSVC as `detect / plan / execute / state-query`
2. treat `managed` and `official` as runtime strategies inside one MSVC domain
3. keep runtime preference visible at the request layer, but make backend orchestration the only owner of internal behavior
4. define canonical-state and lifecycle contracts strongly enough that Phase 7 can implement them without re-litigating boundaries

The current codebase supports that split:

- `spoon-backend/src/msvc/status.rs` already provides a unified result that includes both `managed` and `official`.
- `spoon-backend/src/msvc/official.rs` is already strategy-like.
- `spoon/src/service/msvc/mod.rs` already relies on context builders, so the app/backend seam is closer to completion than the internal backend shape is.

The risk is overscope. If Phase 6 tries to simultaneously:
- redesign the full lifecycle
- land canonical persisted state
- extract shared download/archive/cache utilities
- and thin every app-facing surface

then it will become a second major refactor wave instead of a seam-first phase. The better order is:

1. seam and contract definition
2. canonical state + lifecycle execution
3. shared-contract hardening
4. safety-net reinforcement

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| MSVC-01 | Backend owns explicit MSVC operation/query entry points. | Replace leaked app-facing internal seams with backend-owned requests/results. |
| MSVC-04 | App renders MSVC from backend models/events only. | Keep app-shell responsibilities limited to preference, request construction, and translation. |

## Recommended Plan Order

1. Define the MSVC domain surface and strategy model
2. Thin the app-side MSVC adapters onto backend-owned requests/results
3. Carve mixed status/detect/query logic into explicit backend modules and shared contracts
4. Lock the new seam with targeted regressions and execution guards

## Current Code Reality

- `msvc/mod.rs` mixes exports, managed execution flow, cache/download/extract helpers, wrapper integration, status helpers, and public entry points.
- `official.rs` is already distinct enough to be treated as a runtime strategy rather than a sibling product track.
- `status.rs` and app report code show that the system already thinks in terms of one MSVC domain with two runtime kinds.
- Shared primitive pressure is real across Scoop and MSVC, but the right abstraction target is IO primitives, not unified high-level workflows.

## Validation Focus

- App/runtime live paths no longer depend on MSVC module internals beyond backend-owned requests/results.
- The codebase reflects one MSVC domain with two strategies instead of a split surface leaking upward.
- Planning artifacts and tests make Phase 7's canonical-state/lifecycle work straightforward instead of reopening seam questions.

## Sources

- `.planning/phases/06-msvc-seams-and-ownership-completion/06-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/PROJECT.md`
- `spoon-backend/src/msvc/mod.rs`
- `spoon-backend/src/msvc/official.rs`
- `spoon-backend/src/msvc/status.rs`
- `spoon-backend/src/msvc/paths.rs`
- `spoon-backend/src/msvc/rules.rs`
- `spoon-backend/src/msvc/wrappers.rs`
- `spoon/src/service/msvc/mod.rs`
- `spoon/tests/cli/msvc_flow.rs`
- `spoon/tests/tui/tui_msvc_download_flow.rs`
