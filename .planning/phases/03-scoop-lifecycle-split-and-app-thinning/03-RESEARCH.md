# Phase 3: Scoop Lifecycle Split and App Thinning - Research

**Researched:** 2026-03-29
**Domain:** Scoop lifecycle decomposition and app-shell thinning after canonical state + SQLite control plane
**Confidence:** HIGH

## Summary

Phase 3 is no longer just a file-splitting exercise. After Phase 2 and Phase 02.1, the project already has the two pieces that make a proper lifecycle split feasible: one canonical installed-state model and one SQLite-backed control plane. That changes the right planning target. The phase should not merely carve `runtime/actions.rs` into smaller files; it should establish stable lifecycle semantics that backend orchestration, journaling, doctor/repair, and app progress rendering can all share.

The user's refreshed discuss decisions point to a clear structure: keep explicit `install`, `update`, `uninstall`, and `reapply` entry points, and split the reusable backend modules under them into `planner -> acquire -> materialize -> persist -> surface -> integrate -> state`, with `hooks.rs` remaining a shared execution helper rather than a top-level phase. That means the backend needs two kinds of contracts in this phase: module boundaries and stage vocabulary. Without the stage vocabulary, the SQLite journal added in Phase 02.1 would remain implementation-adjacent instead of becoming a product-level lifecycle contract.

The most important architectural consequence is that `BackendEvent` should now carry structured lifecycle semantics rather than generic log-like text. The user explicitly wants normal logs to stay in `tracing`. That means Phase 3 should add stable stage events and keep app-side service code as a pure translator. `spoon/src/service/scoop/*` should no longer maintain any shadow lifecycle model or infer execution order. The app should render what the backend says happened; it should not decide what is happening.

The right risk boundary is also now clearer. Phase 3 should define stop points, stage transitions, and recoverable boundaries, but it should not attempt to deliver the full retry/repair system. That fuller safety net belongs in Phase 4. In practice, this means Phase 3 must leave behind: stable stage names, journal writes at stage boundaries, explicit fatal vs warning-only hook behavior, and backend-owned ordering for persist/current/surface/state transitions. Phase 4 can then use that contract to implement repair/retry instead of reverse-engineering lifecycle behavior after the split.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| SCLF-01 | Install runs through one explicit backend lifecycle entry point with named phases. | Create install planner/orchestrator plus stable stage contract. |
| SCLF-02 | Update uses the same lifecycle model. | Share planner/acquire/materialize/persist/surface/integrate/state modules with install. |
| SCLF-03 | Uninstall uses the same lifecycle model. | Give uninstall its own orchestration entry point over shared modules and fatal/warning hook rules. |
| SCLF-04 | Reapply, persist restore/sync, and hooks are coordinated from backend lifecycle entry points. | Keep reapply as a first-class entry point and centralize hook execution in shared helpers. |
| SCLF-05 | Lifecycle behavior is split into focused modules rather than one giant controller. | Carve `runtime/actions.rs` by responsibility, not by arbitrary file size. |

## Recommended Plan Order

1. Stage/event/journal contract first
2. Install/update front half
3. Shared back half
4. Uninstall + reapply
5. App thinning

## Current Code Reality

- `spoon-backend/src/scoop/runtime/actions.rs` is still the orchestration monolith.
- `hooks.rs`, `persist.rs`, `surface.rs`, and `integration.rs` are already partial slices and should become formal lifecycle modules.
- `control_plane/*` now gives Phase 3 a place to record lifecycle stages and stop points.
- `spoon/src/service/scoop/*` still contains app translation glue that must not turn back into orchestration.

## Validation Focus

- Structured lifecycle stage events exist and are backend-authoritative.
- Install/update/uninstall/reapply all use the agreed stage ordering.
- Hooks remain centralized and obey the agreed fatal/warning policy.
- App-side service code no longer performs lifecycle orchestration.
- Existing CLI/TUI flows continue to render progress correctly from backend semantics.

## Sources

- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/phases/02-canonical-scoop-state/02-VERIFICATION.md`
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-VERIFICATION.md`
- `.planning/research/SUMMARY.md`
- `.planning/codebase/CONCERNS.md`
- `spoon-backend/src/event.rs`
- `spoon-backend/src/scoop/runtime/actions.rs`
- `spoon-backend/src/scoop/runtime/hooks.rs`
- `spoon-backend/src/scoop/runtime/persist.rs`
- `spoon-backend/src/scoop/runtime/surface.rs`
- `spoon-backend/src/scoop/runtime/integration.rs`
- `spoon-backend/src/control_plane/*`
- `spoon/src/service/scoop/actions.rs`
- `spoon/src/service/scoop/runtime.rs`
