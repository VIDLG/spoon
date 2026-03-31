# Phase 5: Scoop Contract Alignment and Context Completion - Research

**Researched:** 2026-03-31
**Domain:** Gap closure after milestone audit for stale Scoop regression contracts and partial context seam alignment
**Confidence:** HIGH

## Summary

Phase 5 is not another architecture phase. The milestone audit already narrowed the remaining blockers: some regressions still verify the removed JSON package-state world, and the Scoop app/runtime seam still reads as only partially context-owned. That means the right planning target is minimal contract alignment, not broader cleanup.

The first and clearest gap is stale verification. `spoon/tests/cli/scoop_flow.rs` still seeds `scoop/state/packages/*.json`, and `spoon-backend/src/scoop/tests/runtime.rs` still expects a JSON file for `runtime_writes_canonical_scoop_state`. Those expectations directly contradict Phase 02.1, which moved control-plane truth into SQLite, and they undermine Phase 4's claim that the safety net protects the current contract. The best repair is migration, not deletion: the user-facing intent of these tests still matters, but setup and assertions must shift to backend store/control-plane APIs.

The second gap is smaller but still real: Scoop's app-side runtime path still uses host-based backend entry points in places where MSVC already uses an explicit `BackendContext` seam. The audit correctly called this a `LAY-03 partial`, not a full failure. So Phase 5 should not try to make Scoop and MSVC perfectly symmetrical; it should make the seam unambiguous enough that the audit no longer reports mixed ownership.

This phase should therefore stay small and audit-driven:

1. align stale backend/runtime regressions with SQLite/canonical state
2. align stale app/CLI regressions with the same contract
3. clarify the remaining Scoop context seam enough to remove the `LAY-03 partial`
4. rerun the milestone audit

Anything broader would risk turning a gap-closure phase into an unnecessary second refactor wave.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| LAY-03 | Backend operations should run in explicit context rather than ambiguous mixed ownership. | Clarify or complete the Scoop context seam enough for audit closure. |
| TEST-02 | App tests should stay focused on routing/orchestration shell behavior. | Migrate stale app tests to the current contract without re-implementing backend logic. |
| TEST-03 | New or changed backend contracts should have nearby focused tests. | Update stale backend regressions to verify the SQLite/canonical contract. |

## Recommended Plan Order

1. Backend stale regression alignment
2. App stale regression alignment
3. Scoop context seam clarification and milestone re-audit

## Current Code Reality

- `spoon/tests/cli/scoop_flow.rs` still seeds legacy JSON package-state files.
- `spoon-backend/src/scoop/tests/runtime.rs` still has at least one stale JSON-oriented assertion.
- `spoon/src/service/scoop/runtime.rs` still routes through host-based runtime helpers instead of the more explicit context-driven seam already available elsewhere.

## Validation Focus

- Failing stale tests are migrated and pass against SQLite-backed canonical state.
- The remaining Scoop context seam is no longer ambiguous enough to keep `LAY-03` partial.
- Milestone audit can be re-run without the previously reproduced blocker failures.

## Sources

- `.planning/phases/05-scoop-contract-alignment-and-context-completion/05-CONTEXT.md`
- `.planning/v0.5.0-MILESTONE-AUDIT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `spoon/tests/cli/scoop_flow.rs`
- `spoon-backend/src/scoop/tests/runtime.rs`
- `spoon/src/service/scoop/runtime.rs`
- `spoon/src/service/mod.rs`
- `spoon-backend/src/scoop/state/store.rs`
- `spoon-backend/src/control_plane/sqlite.rs`
