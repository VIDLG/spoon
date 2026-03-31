# Phase 2: Canonical Scoop State - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2 makes `spoon-backend` the only owner of Scoop installed-package facts by replacing the current duplicate state models with one canonical persisted record plus backend-owned projections.
This phase is about canonical state shape, migration/repair behavior, and backend read-model wiring. It does not introduce new user-facing capabilities, and it does not yet split the full install/update/uninstall lifecycle into explicit backend phases reserved for Phase 3.

</domain>

<decisions>
## Implementation Decisions

### Canonical State Contract
- **D-01:** Phase 2 will end with exactly one canonical Scoop installed-package record in `spoon-backend`; do not preserve `ScoopPackageState` and `InstalledPackageState` as co-equal models behind adapters.
- **D-02:** The canonical record will be based on `InstalledPackageState`, not a new third model and not `ScoopPackageState` as the base.
- **D-03:** Merge `bucket` and `architecture` into the canonical installed-package record because they are installed facts that should not be re-guessed later.
- **D-04:** Persist only non-derivable Scoop facts. Absolute paths, `current` locations, and other layout-derived values must be reconstructed from backend context and `RuntimeLayout`, not stored in persisted state.

### Migration and Repair Policy
- **D-05:** Phase 2 should use forward design, not a compatibility-preserving migration layer. Do not keep `ScoopPackageState` alive through dual-read, dual-write, or adapter shims.
- **D-06:** Legacy Scoop state is allowed to become invalid across the Phase 2 boundary. If stale old state is encountered, the backend should use an explicit repair or rebuild path rather than silently pretending compatibility still exists.

### Projection and Read-Model Ownership
- **D-07:** Package list, package detail, uninstall inputs, reapply inputs, and status surfaces must all project from the same canonical backend state source. Do not let `query.rs`, `info.rs`, or app code each rediscover install facts independently.
- **D-08:** Phase 2 should switch these consumers in one pass. Do not end the phase with writes on the canonical model while major read surfaces still depend on legacy or duplicated state logic.

### App/Backend Boundary
- **D-09:** Phase 2 stays backend-centric. `spoon` remains a thin consumer of backend results and should not gain new ownership over Scoop state interpretation while the canonical model is consolidated.

### the agent's Discretion
No extra product-facing discretion was requested. Research and planning may choose the exact module split, repair command shape, and rollout mechanics as long as they preserve the decisions above.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Direction
- `.planning/PROJECT.md` - Project-level refactor direction, ownership rules, and evolution policy.
- `.planning/ROADMAP.md` - Phase ordering, official Phase 2 goal, and SCST success criteria.
- `.planning/REQUIREMENTS.md` - Requirement IDs SCST-01 through SCST-04 and traceability expectations.
- `AGENTS.md` - Repository-specific ownership, path, and testing constraints for Spoon.

### Prior Phase Context
- `.planning/STATE.md` - Current milestone position, completed Phase 1 decisions, and Phase 2 blocker note about migration/repair planning.
- `.planning/phases/01-backend-seams-and-ownership/01-CONTEXT.md` - Locked seam/layout/context decisions from Phase 1 that Phase 2 must preserve.
- `.planning/phases/01-backend-seams-and-ownership/01-VERIFICATION.md` - Verified Phase 1 contract surface and the completed gap closure that moved app path derivation behind `RuntimeLayout`.

### Research and Codebase Maps
- `.planning/research/SUMMARY.md` - Recommended sequencing and explicit warning to build a producer/consumer matrix before deleting duplicate state structs.
- `.planning/research/FEATURES.md` - Phase 2 feature guidance, including the requirement to choose one canonical Scoop state model and avoid compatibility adapters.
- `.planning/codebase/STRUCTURE.md` - Current crate/module split and where Scoop state ownership lives today.
- `.planning/codebase/CONVENTIONS.md` - Existing error, module, and helper conventions to follow while restructuring backend state code.
- `.planning/codebase/STACK.md` - Current workspace/runtime stack and backend dependency context.
- `.planning/codebase/CONCERNS.md` - Known Scoop lifecycle fragility and state corruption risks that planning must preserve while consolidating state.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/scoop/runtime/installed_state.rs`: Current richer persisted record with uninstall, integration, persist, and env facts already used by runtime flows.
- `spoon-backend/src/scoop/package_state.rs`: Minimal duplicate persisted model carrying package identity/source facts that need to be merged or eliminated.
- `spoon-backend/src/scoop/query.rs`: Existing package list and status queries that already consume `InstalledPackageState`.
- `spoon-backend/src/scoop/info.rs`: Package detail and operation-outcome code that currently mixes canonical-looking reads with raw JSON probing and should be unified.

### Established Patterns
- Phase 1 already established `BackendContext` plus `RuntimeLayout` as the only layout source. Phase 2 must keep deriving paths from layout instead of state.
- The repo prefers backend-focused tests close to behavior in `spoon-backend`, with app tests focused on orchestration and rendering.
- Forward design is preferred over compatibility-heavy adapters. Remove duplicate ownership rather than keeping both interfaces alive.
- `SystemPort` remains a backend-level host boundary, but Scoop-specific integration callbacks are now scoped under `spoon-backend/src/scoop/ports.rs` as `ScoopIntegrationPort` rather than living in backend crate-root ports.

### Integration Points
- `spoon-backend/src/scoop/runtime/actions.rs`: Writes installed-state facts today and will need to emit the canonical record.
- `spoon-backend/src/scoop/query.rs` and `spoon-backend/src/scoop/info.rs`: Main read-model consumers that must be moved onto one canonical state source.
- `spoon/src/service/scoop/*`, `spoon/src/status/*`, and `spoon/src/view/*`: Already thin enough after Phase 1 that they should only need backend projection updates, not new app-owned state logic.

</code_context>

<specifics>
## Specific Ideas

- Build a producer/consumer matrix before deleting either Scoop state model, so planning can see every write path, every read path, and every field that must survive the consolidation.
- Prefer a dedicated backend state/projection split over leaving canonical reads embedded across `query.rs`, `info.rs`, and runtime helpers.
- Treat repair/rebuild as part of the backend contract for this phase, but do not turn it into a long-lived compatibility bridge for the old state model.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within the canonical Scoop state scope.

</deferred>

---

*Phase: 02-canonical-scoop-state*
*Context gathered: 2026-03-28*
