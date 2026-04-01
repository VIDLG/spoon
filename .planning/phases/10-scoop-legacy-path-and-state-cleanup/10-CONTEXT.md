# Phase 10: Scoop Legacy Path and State Cleanup - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase removes stale JSON-era Scoop path/state concepts from the active backend code so `spoon-backend/src/scoop/` fully reflects the current SQLite-backed control plane and `RuntimeLayout` model.

This is a forward-looking cleanup phase, not a third Scoop architecture redesign. The goal is to make the Scoop code more readable, more obviously backend-owned, and more aligned with the current canonical model.

</domain>

<decisions>
## Implementation Decisions

### Legacy Path and State Removal
- **D-01:** Delete legacy JSON-era Scoop path/state concepts as aggressively as possible instead of preserving compatibility-oriented helper layers.
- **D-02:** Active Scoop code should no longer expose or depend on path APIs shaped around `packages/*.json`, `buckets.json`, or other pre-SQLite control-plane concepts.
- **D-03:** This phase should prefer deleting old concepts outright rather than downgrading them to long-lived deprecated helpers.

### Path Model
- **D-04:** `RuntimeLayout` / `ScoopLayout` becomes the single authoritative path model for Scoop.
- **D-05:** `spoon-backend/src/scoop/paths.rs` should not remain a long-term public path abstraction layer.
- **D-06:** Package-level path semantics may survive only if they become layout-owned methods with clear domain value; otherwise code should use `layout.scoop.*` directly.

### Doctor and Legacy Residue
- **D-07:** Do not keep dedicated legacy JSON-state diagnostics as a formal subsystem just to ease migration from old Scoop layouts.
- **D-08:** `doctor` should not preserve a compatibility worldview around old flat JSON state; this milestone should use the early-stage product window to delete that worldview rather than institutionalize it.

### Cleanup Priorities
- **D-09:** Prefer model convergence and readability over purely mechanical deletion.
- **D-10:** The resulting Scoop backend code should read like maintainable human-owned backend code, with fewer indirection layers and less historical residue.

### the agent's Discretion
- The exact split between direct `layout.scoop.*` usage and a very small number of retained layout-owned package path helpers is left to implementation judgment.
- Incidental shared cleanup is allowed only when directly needed to finish the Scoop cleanup cleanly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and Scope
- `.planning/PROJECT.md` - Current milestone framing and the readability-first expectation for backend cleanup work.
- `.planning/ROADMAP.md` - Phase 10 goal, dependency chain, and milestone boundary.
- `.planning/REQUIREMENTS.md` - Phase 10 requirement mapping for `SLEG-01` and `SLEG-04`.
- `.planning/STATE.md` - Active milestone state and guardrails.

### Prior Scoop Architecture
- `.planning/phases/02-canonical-scoop-state/02-CONTEXT.md` - Canonical Scoop state decisions that replaced the old flat JSON model.
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-CONTEXT.md` - SQLite control-plane and sync/async boundary decisions that now define authoritative Scoop state.

### Shared Contract Context
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md` - Shared contract hardening decisions, especially path/layout cleanup direction and backend-owned contract rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/layout.rs` already contains the authoritative `RuntimeLayout` / `ScoopLayout` model this phase wants to converge on.
- `spoon-backend/src/scoop/state/` already expresses the SQLite-backed canonical Scoop state; this phase should lean on it rather than recreate migration surfaces.

### Established Patterns
- Current backend direction prefers canonical control-plane state, layout-owned path truth, and thin app-shell glue.
- The repo has already accepted aggressive forward cleanup in earlier phases when old abstractions were clearly wrong.

### Integration Points
- `spoon-backend/src/scoop/paths.rs` is the main obvious legacy-path concentration point.
- `spoon-backend/src/scoop/doctor.rs` still carries explicit legacy JSON-state scanning logic.
- `spoon-backend/src/scoop/query.rs` and adjacent read-model code still reflect some older path/helper assumptions.
- `spoon/src/config/paths.rs` contains deprecated Scoop path helpers that may need to be reduced or isolated as spillover from the backend cleanup.

</code_context>

<specifics>
## Specific Ideas

- Treat this as a readability and elegance pass, not just a deletion pass.
- Prefer code that is explicit and obviously grounded in `RuntimeLayout` over helper-heavy indirection.
- Use the project's early-stage state as justification to delete the old worldview rather than preserving migration comfort.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 10-scoop-legacy-path-and-state-cleanup*
*Context gathered: 2026-04-01*
