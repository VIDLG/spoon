# Phase 5: Scoop Contract Alignment and Context Completion - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 5 is a gap-closure phase created from the v0.5.0 milestone audit. It exists to close the specific audit blockers that prevent milestone archive, not to open a new Scoop refactor track.
This phase should stay narrow: migrate stale regressions off the removed JSON control-plane contract, align the remaining partial Scoop context seam enough that the audit no longer reports ambiguity, and re-verify the milestone. It is not the place for additional lifecycle redesign, event redesign, warning cleanup, or broad new feature work.

</domain>

<decisions>
## Implementation Decisions

### Scope
- **D-01:** Phase 5 should be a minimal gap-closure phase, not a new Scoop architecture phase.
- **D-02:** The phase should only address the specific blockers surfaced by the milestone audit.

### Context Seam
- **D-03:** The Scoop `BackendContext` seam should be improved only far enough that the audit no longer reports `LAY-03` as partial.
- **D-04:** Phase 5 does not need to force Scoop into the exact same surface shape as MSVC if the remaining host/context split can be made explicit and non-ambiguous another way.

### Stale Regression Handling
- **D-05:** Stale tests must be migrated to the current SQLite/canonical contract rather than preserved as legacy checks.
- **D-06:** Existing regression intent should be kept where possible, but the world they assert must change to the current backend contract.

### Out of Scope
- **D-07:** Do not use Phase 5 to continue general Scoop cleanup, warning cleanup, event redesign, or backend error redesign.
- **D-08:** Real remote Scoop smoke remains best-effort and out of blocker scope unless a deterministic regression is found.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Audit / Milestone State
- `.planning/v0.5.0-MILESTONE-AUDIT.md` - The exact blocker gaps this phase must close.
- `.planning/ROADMAP.md` - Phase 5 goal, requirement mapping, and current milestone state.
- `.planning/REQUIREMENTS.md` - Requirement IDs `LAY-03`, `TEST-02`, `TEST-03` now assigned to Phase 5.
- `.planning/STATE.md` - Current project state after gap-phase insertion.

### Prior Phase Context
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-VERIFICATION.md` - SQLite control-plane truth that stale tests must align with.
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-CONTEXT.md` - Scoop lifecycle and app/backend boundary decisions that remain binding.
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-VERIFICATION.md` - Verified lifecycle split state that Phase 5 must not regress.
- `.planning/phases/04-refactor-safety-net/04-VERIFICATION.md` - Safety-net state and residual notes from the just-completed phase.

### Relevant Code / Tests
- `spoon/tests/cli/scoop_flow.rs` - Stale JSON-seeded CLI regressions identified by audit.
- `spoon-backend/src/scoop/tests/runtime.rs` - Stale backend runtime regression identified by audit.
- `spoon/src/service/scoop/runtime.rs` - Current app-side Scoop runtime adapter surface.
- `spoon/src/service/mod.rs` - Shared app/backend context builder and event translation helpers.
- `spoon-backend/src/scoop/state/store.rs` - SQLite-backed canonical state APIs.
- `spoon-backend/src/control_plane/sqlite.rs` - Control-plane DB path and facade.

</canonical_refs>

<code_context>
## Existing Code Insights

### Direct Audit Blockers
- `spoon/tests/cli/scoop_flow.rs` still seeds `scoop/state/packages/*.json` and fails against the SQLite-backed installed-state/query path.
- `spoon-backend/src/scoop/tests/runtime.rs` still has `runtime_writes_canonical_scoop_state` expecting JSON package-state persistence.
- The explicit Scoop `BackendContext` seam exists, but app-side live paths still mix host-based and context-based backend entry usage enough for the audit to classify `LAY-03` as partial.

### Established Patterns
- Canonical Scoop state lives in backend store APIs, not flat JSON package-state files.
- SQLite is the active control plane; filesystem is the data plane.
- App shell should translate backend semantics rather than recreate backend logic.

</code_context>

<specifics>
## Specific Ideas

- Prefer keeping test names/intents where they still represent real user-facing behavior, but rewrite setup and assertions around canonical store / control-plane reality.
- Treat the Scoop context-seam fix as a clarity task, not an excuse to reopen Phase 1 at large.

</specifics>

<deferred>
## Deferred Ideas

- Broader Scoop cleanup beyond the audited blockers.
- Backend event contract redesign.
- Backend error contract redesign.
- General warning cleanup.

</deferred>

---

*Phase: 05-scoop-contract-alignment-and-context-completion*
*Context gathered: 2026-03-31*
