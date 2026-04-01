# Phase 6: MSVC Seams and Ownership Completion - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 6 is the first execution phase of milestone `v0.6.0`. Its job is to establish the correct MSVC domain shape before deeper runtime rewrites begin. The phase should make MSVC look like a first-class backend domain with explicit seams, module boundaries, strategy modeling, and thin app-shell ownership, but it should not yet attempt to fully land canonical MSVC persisted state or the final lifecycle execution rewrite. Those belong to Phase 7.

The phase must therefore focus on structure and contracts:
- define the MSVC domain around `detect / plan / execute / state-query`
- stop treating `managed` and `official` as two unrelated product tracks
- keep the app layer translation-only, with runtime preference expression but no runtime-specific orchestration
- define the contracts that later phases will execute against

</domain>

<decisions>
## Implementation Decisions

### Domain Shape
- **D-01:** MSVC should be modeled as `detect / plan / execute / state-query`, not as direct copies of Scoop's `acquire/materialize/surface` structure.
- **D-02:** The backend should become the only place that owns MSVC detect/plan/execute semantics; app code should only build requests and translate results.

### Runtime Strategies
- **D-03:** Do not preemptively delete `managed`.
- **D-04:** `managed` and `official` should be treated as two runtime strategies inside one MSVC domain rather than two unrelated parallel products.

### State and Lifecycle
- **D-05:** MSVC should move toward one canonical state model with `runtime_kind` as a dimension rather than separate domain models forever.
- **D-06:** MSVC should get a formal lifecycle contract, but with MSVC-specific stages such as `planned`, `detecting`, `resolving`, `executing`, `validating`, `state_committing`, and `completed`.
- **D-07:** Phase 6 may define these contracts and shapes, but it should not be forced to land the full canonical-state persistence and lifecycle execution rewrite; Phase 7 owns that.

### App / Backend Boundary
- **D-08:** The app may express runtime preference, but it must not orchestrate runtime-specific internal steps.
- **D-09:** Managed/official adapters in the app should shrink toward request/result translation only.

### Shared Utility Direction
- **D-10:** Shared IO primitives such as archive/download/cache work should not all be stuffed into `fsx`.
- **D-11:** `fsx` should stay filesystem-primitive-focused; cross-domain archive/download/cache primitives are better handled as separate shared backend modules later.
- **D-12:** Shared utility extraction is acknowledged here for future compatibility, but the main extraction work belongs to Phase 8.

### Out of Scope
- **D-13:** Do not reopen a second large Scoop refactor in this phase.
- **D-14:** Do not decide product-direction questions like deleting the managed runtime before the unified MSVC domain model exists.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / Planning State
- `.planning/PROJECT.md` - Current milestone framing and carried-forward constraints.
- `.planning/ROADMAP.md` - Phase 6 goal, milestone structure, and phase dependencies.
- `.planning/REQUIREMENTS.md` - Requirement IDs `MSVC-01`, `MSVC-04` directly bound to this phase.
- `.planning/STATE.md` - Current project state and follow-up inventory.

### Prior Backend Architecture Decisions
- `.planning/milestones/v0.5.0-ROADMAP.md` - The just-shipped backend-refactor milestone and the level of Scoop/backend maturity already reached.
- `.planning/milestones/v0.5.0-REQUIREMENTS.md` - Previously completed seam/state/lifecycle/test requirements.
- `.planning/v0.5.0-MILESTONE-AUDIT.md` - What was accepted as tech debt and intentionally deferred.
- `.planning/seeds/SEED-001-backend-event-contract-hardening.md` - Event-contract follow-up that Phase 6 must not accidentally over-scope into.
- `.planning/todos/pending/2026-03-31-tighten-backend-error-contract.md` - Error-contract follow-up relevant to later shared-contract hardening.
- `.planning/todos/pending/2026-03-31-consolidate-remaining-fsx-helpers.md` - Filesystem-helper follow-up relevant to later shared-utility extraction.
- `.planning/todos/pending/2026-03-31-remove-hardcoded-production-paths.md` - Runtime path hardcoding follow-up relevant to later shared-contract hardening.

### Relevant Code
- `spoon-backend/src/msvc/mod.rs` - Current monolithic managed-path entry surface and exported contracts.
- `spoon-backend/src/msvc/official.rs` - Official-runtime strategy implementation.
- `spoon-backend/src/msvc/status.rs` - Current unified status projection over managed + official facts.
- `spoon-backend/src/msvc/paths.rs` - Runtime-layout-derived MSVC path model.
- `spoon-backend/src/msvc/rules.rs` - Current managed installed-state rule helpers and persisted-state assumptions.
- `spoon-backend/src/msvc/wrappers.rs` - Managed command-surface integration logic.
- `spoon/src/service/msvc/mod.rs` - Current app-side MSVC adapter surface that must thin further.
- `spoon-backend/src/event.rs` - Shared backend event contract that MSVC must fit without reintroducing ad hoc event shapes.
- `spoon-backend/src/fsx.rs` - Shared filesystem helper boundary that should stay narrow.
- `AGENTS.md` - Repo-level architecture/testing ownership rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Current Domain Shape
- `spoon-backend/src/msvc/mod.rs` currently behaves like a large managed-runtime controller plus export surface.
- `official.rs` already behaves like an alternative strategy, but the surrounding domain model still reads as two parallel worlds rather than one domain with two strategies.
- `status.rs` already projects both `managed` and `official` into a single backend result, which is a useful signal for the future canonical shape.

### Current App Boundary
- `spoon/src/service/msvc/mod.rs` already builds explicit backend contexts, which is good, but still exposes a visibly split managed/official surface.
- This means the seam is improved compared with older Scoop, but the runtime strategy model is still leaking into the app entry layout.

### Shared Utility Pressure
- Scoop and MSVC both own download/extract/cache primitives today.
- The duplication is real, but the safe abstraction target is shared IO primitives, not a forced shared package/toolchain workflow.

</code_context>

<specifics>
## Specific Ideas

- Prefer Phase 6 plans that establish domain boundaries and contracts with minimal irreversible runtime churn.
- Make it easy for Phase 7 to land canonical state/lifecycle execution without reopening the same seam questions.

</specifics>

<deferred>
## Deferred Ideas

- Full canonical MSVC persisted-state implementation.
- Full MSVC lifecycle execution rewrite.
- Event/error/fsx/download/archive shared-contract hardening beyond what Phase 6 needs for clean seams.
- Broad Scoop cleanup unrelated to MSVC or shared backend contracts.

</deferred>

---

*Phase: 06-msvc-seams-and-ownership-completion*
*Context gathered: 2026-04-01*
