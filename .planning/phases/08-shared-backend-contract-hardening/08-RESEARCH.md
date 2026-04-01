# Phase 8: Shared Backend Contract Hardening - Research

**Researched:** 2026-04-01
**Domain:** Shared contract hardening after Scoop and MSVC both became real backend domains
**Confidence:** HIGH

## Summary

By the end of Phase 7, the architecture has crossed an important threshold: Scoop and MSVC are no longer the main structural problem. The remaining problems are shared-contract problems. That changes how the next phase should behave.

The biggest planning implication is that Phase 8 must stay cross-cutting and disciplined. If it drifts into domain-specific rewrites, it will simply recreate the last two phases under a different name. Instead, it should do five narrow but high-value contract passes:

1. reset the backend event contract using forward design
2. tighten the error contract without overreaching
3. extract shared IO primitives where duplication is now clearly justified
4. simplify host/port boundaries
5. sweep targeted JSON-era layout/path leftovers

These areas are strongly related:

- Event and error contracts shape how app-shell translation, diagnostics, and future repair semantics compose.
- Shared utility extraction and layout/port cleanup both reduce cross-domain duplication and semantic drift.
- The derive-not-store and path-legacy follow-ups already show that old Scoop-era assumptions still leak into shared layers even after the control plane moved to SQLite.

The safest strategy is to keep each subarea explicitly bounded:

- event: contract reset, not event-platform moonshot
- error: contract tightening, not total taxonomy rewrite
- shared utility: primitive extraction, not one unified runtime engine
- port/layout: targeted cleanup, not repo-wide path churn

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| BECT-01 | Stronger backend event contract | Replace weak stringly semantics and overloaded progress shapes. |
| BECT-02 | Stronger backend error contract | Reduce broad fallback cases and clarify contract layers. |
| BECT-03 | Shared filesystem/path primitives | Extract primitives into shared modules with `fsx` remaining narrow. |
| BECT-04 | Runtime path hardening | Remove stale JSON-era path concepts and avoid hardcoded production paths where abstractions exist. |

## Recommended Plan Order

1. Event contract reset
2. Error contract hardening
3. Shared primitive extraction
4. Port and layout legacy cleanup

## Current Code Reality

- `event.rs` still carries the known overloading and weak typing we previously captured as a seed.
- `error.rs` remains serviceable but broad fallback variants are still carrying too much meaning.
- Scoop and MSVC now both justify primitive extraction because the duplication is no longer hypothetical.
- `SystemPort` / `ScoopRuntimeHost` and layout JSON-era fields are now clearly technical debt, not open design questions.

## Validation Focus

- App/backend/tests move together on the new event contract without compatibility shims.
- Error semantics get clearer without destabilizing the new lifecycle/state work.
- Shared utility extraction reduces duplication without collapsing domain workflows together.
- Legacy path and host-boundary cleanup removes stale concepts without destabilizing runtime behavior.

## Sources

- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md`
- `.planning/phases/07-canonical-msvc-state-and-lifecycle/07-VERIFICATION.md`
- `.planning/seeds/SEED-001-backend-event-contract-hardening.md`
- `.planning/todos/pending/2026-03-31-tighten-backend-error-contract.md`
- `.planning/todos/pending/2026-03-31-consolidate-remaining-fsx-helpers.md`
- `.planning/todos/pending/2026-03-31-remove-hardcoded-production-paths.md`
- `.planning/todos/pending/2026-04-01-audit-derive-not-store-fields.md`
- `.planning/todos/pending/2026-04-01-simplify-system-port-and-runtime-host-boundaries.md`
- `spoon-backend/src/event.rs`
- `spoon-backend/src/error.rs`
- `spoon-backend/src/fsx.rs`
- `spoon-backend/src/ports.rs`
- `spoon-backend/src/layout.rs`
