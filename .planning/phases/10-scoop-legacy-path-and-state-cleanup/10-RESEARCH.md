# Phase 10: Scoop Legacy Path and State Cleanup - Research

**Researched:** 2026-04-01
**Domain:** Forward cleanup of legacy Scoop path/state concepts after the SQLite control-plane cutover
**Confidence:** HIGH

## Summary

Phase 10 should not behave like a migration-support phase. The current backend architecture already made two core decisions:

1. SQLite is the authoritative Scoop control plane
2. `RuntimeLayout` is the authoritative runtime path model

The remaining Scoop legacy debt is therefore not "missing compatibility work" - it is residue from an older worldview that now makes the code harder to read and easier to misuse.

The most valuable cleanup move is to converge active Scoop code onto:

- `RuntimeLayout` / `ScoopLayout` for path truth
- `scoop/state/` plus control-plane reads for state truth
- explicit package/runtime semantics instead of helper-heavy JSON-era path functions

The repo scan shows the clearest cleanup targets are:

- `spoon-backend/src/scoop/paths.rs`
- `spoon-backend/src/scoop/doctor.rs`
- read-model callers still shaped around old path helpers
- app-side deprecated Scoop path helpers in `spoon/src/config/paths.rs`

Because the user explicitly chose a forward-looking approach with no backward-compatibility preservation, planning should bias toward deleting legacy APIs rather than wrapping or downgrading them.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| SLEG-01 | Active Scoop runtime paths no longer depend on JSON-era package-state or bucket-registry concepts | Remove JSON-era path helpers and callers instead of fencing them off. |
| SLEG-04 | Deprecated/legacy-only Scoop path helpers are removed, downgraded, or isolated from active runtime behavior | Prefer removal and layout convergence; isolate only if deletion would create churn without value. |

## Recommended Plan Order

1. Collapse active Scoop path usage onto `RuntimeLayout` / `ScoopLayout`
2. Remove JSON-era legacy path APIs and doctor logic
3. Clean app/backend spillover helpers that still encode the old Scoop path worldview
4. Re-run focused Scoop path/state regressions and phase verification

## Current Code Reality

- `spoon-backend/src/scoop/paths.rs` still exports helpers for `buckets.json` and `packages/*.json`, even though those are no longer active control-plane truths.
- `spoon-backend/src/scoop/doctor.rs` still carries legacy JSON detection logic, effectively preserving the old worldview in a dedicated subsystem.
- `spoon-backend/src/scoop/query.rs` still exposes low-value read-model counts and path assembly patterns that can be tightened once path truth converges.
- `spoon/src/config/paths.rs` still contains deprecated Scoop path helpers that will drift if the backend path model becomes cleaner while the app layer keeps old aliases alive.

## Validation Focus

- Active Scoop runtime code uses `RuntimeLayout` / `ScoopLayout` as the path source of truth.
- Legacy JSON-era path/state helper APIs disappear from active code.
- No dedicated legacy JSON-state diagnostic subsystem remains in Scoop doctoring.
- Focused backend/app regressions still pass after path/state cleanup.

## Sources

- `.planning/phases/10-scoop-legacy-path-and-state-cleanup/10-CONTEXT.md`
- `.planning/phases/02-canonical-scoop-state/02-CONTEXT.md`
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-CONTEXT.md`
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md`
- `spoon-backend/src/layout.rs`
- `spoon-backend/src/scoop/paths.rs`
- `spoon-backend/src/scoop/doctor.rs`
- `spoon-backend/src/scoop/query.rs`
- `spoon/src/config/paths.rs`
