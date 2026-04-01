# Phase 11: Scoop Runtime Host and Helper Consolidation - Research

**Researched:** 2026-04-01
**Domain:** Scoop structural refactor focused on host/helper naming, layer purity, and root-module clarity
**Confidence:** HIGH

## Summary

Phase 11 should be treated as a structural Scoop refactor, not a local cleanup. The current problem is not just "a few helpers are duplicated"; the problem is that the current directory and module names no longer tell a reader where the domain core really lives.

The clearest architectural target is:

- domain root:
  - `actions`
  - `planner`
  - `package_source`
  - `buckets`
  - `query`
  - `info`
  - `doctor`
  - `state/`
- `lifecycle/`:
  - only true lifecycle stage modules
- `host/`:
  - only edge-facing host/integration glue

This also suggests that `scoop/mod.rs` should stop acting like a maximal convenience facade. A slimmer facade will make the new shape legible instead of hiding all module distinctions behind root re-exports.

The repo scan also confirms that:

- `runtime/actions.rs` currently looks like the true operation entrypoint
- `runtime/source.rs` is clearly a domain model, not a host-edge file
- `lifecycle/planner.rs` and `lifecycle/state.rs` are semantically off relative to their directory
- `projection.rs` is a likely naming/placement cleanup target, but the broad data redundancy wave still fits better in Phase 12

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| SLEG-02 | Scoop runtime/helper boundaries are simpler and more explicit | Rename/restructure modules so responsibilities are obvious. |
| BECT-05 | Shared cleanup needed to finish the Scoop legacy pass is resolved without reopening the whole backend-hardening phase | Allow limited shared spillover only when directly needed to complete the Scoop structural refactor. |

## Recommended Plan Order

1. Establish the new Scoop topology (`runtime` -> `host`, `actions` and `package_source` at the root)
2. Purify `lifecycle/` by moving non-stage modules out
3. Thin the host layer and root facade to match the new topology
4. Re-run representative backend/app regressions and record the phase

## External-Library Posture

- External crates are acceptable when they remove structural boilerplate or simplify interfaces materially.
- Avoid dependencies that only replace small internal helpers without simplifying the Scoop domain model.
- This phase is about architecture readability first; library adoption is only justified when it serves that goal.

## Validation Focus

- The renamed `host/` layer is visibly thinner and more edge-oriented.
- The Scoop domain root now owns the real operation entry and source model.
- `lifecycle/` reads like a lifecycle directory rather than a mixed bag.
- Representative Scoop regressions still pass after the structural moves.

## Sources

- `.planning/phases/11-scoop-runtime-host-and-helper-consolidation/11-CONTEXT.md`
- `.planning/phases/10-scoop-legacy-path-and-state-cleanup/10-VERIFICATION.md`
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-CONTEXT.md`
- `spoon-backend/src/scoop/mod.rs`
- `spoon-backend/src/scoop/planner.rs`
- `spoon-backend/src/scoop/runtime/mod.rs`
- `spoon-backend/src/scoop/lifecycle/mod.rs`
- `spoon-backend/src/scoop/state/mod.rs`
