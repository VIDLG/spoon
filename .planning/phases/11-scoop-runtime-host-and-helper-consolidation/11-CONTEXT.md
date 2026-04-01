# Phase 11: Scoop Runtime Host and Helper Consolidation - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase performs a structural Scoop-domain refactor to reduce duplicated helper layers, fix names that no longer match responsibilities, and make the Scoop backend easier for humans to read and maintain.

The intent is not another lifecycle redesign. The intent is to make the already-established Scoop architecture read as a coherent domain:

- clear operation entrypoints
- clear lifecycle stages
- clear host edge adapters
- clearer root-module exports

</domain>

<decisions>
## Implementation Decisions

### Runtime / Host Naming
- **D-01:** `runtime` is no longer an acceptable long-term name for the current edge layer because it is too broad and invites business logic to accumulate there.
- **D-02:** The current `runtime` layer should be renamed to **`host`** as part of this phase.
- **D-03:** After the rename, that layer should be intentionally thin and edge-oriented rather than acting like a second business core.

### Lifecycle Purity
- **D-04:** `lifecycle` remains the correct name for the stage-oriented business layer.
- **D-05:** `lifecycle` should contain only true lifecycle stage modules.
- **D-06:** Non-stage modules such as planning/state glue should be moved out of `lifecycle`.

### Domain Entry and Models
- **D-07:** `actions` should become a root Scoop-domain operation entry module rather than living under the host layer.
- **D-08:** `runtime/source.rs` should move to the Scoop domain root as a domain model file.
- **D-09:** The preferred name for that root model file is **`package_source.rs`**.

### Host Layer Scope
- **D-10:** The renamed `host/` layer should keep only thin edge-facing concerns such as execution entry, host seam wiring, integration bridging, and possibly hook glue.
- **D-11:** Files like `actions`, `source`, and other domain-heavy helpers should not remain under `host/`.

### Read-Model Direction
- **D-12:** `query` and `info` may remain distinct modules.
- **D-13:** `projection` should stop acting like a public catch-all bucket; either shrink it to an internal helper or rename it to something more explicit in a later cleanup wave.
- **D-14:** The bulk of read-model/data-structure redundancy cleanup belongs to Phase 12, but this phase may make naming/placement moves required to keep the structural refactor coherent.

### External Libraries
- **D-15:** Mature external libraries may be adopted when they materially simplify the Scoop structure or remove unnecessary in-house plumbing.
- **D-16:** External crates are a tool, not a goal; avoid adding dependencies that do not clearly improve readability, maintenance, or backend-owned structure.

### the agent's Discretion
- The exact final split between `host/execution`, `host/integration`, and `host/hooks` can be decided during implementation if the resulting boundary is clearer.
- Minor read-model naming adjustments may happen here only when needed to keep the structural refactor coherent; the larger redundancy pass remains Phase 12.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and Scope
- `.planning/PROJECT.md` - Current milestone framing and readability-first expectations for Scoop cleanup work.
- `.planning/ROADMAP.md` - Phase 11 goal, dependency chain, and milestone boundary.
- `.planning/REQUIREMENTS.md` - Phase 11 requirement mapping for `SLEG-02` and `BECT-05`.
- `.planning/STATE.md` - Current milestone state and guardrails.

### Prior Scoop Cleanup
- `.planning/phases/10-scoop-legacy-path-and-state-cleanup/10-CONTEXT.md` - Forward-only cleanup direction established in Phase 10.
- `.planning/phases/10-scoop-legacy-path-and-state-cleanup/10-VERIFICATION.md` - What was already cleaned and what remains.

### Prior Architecture Decisions
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-CONTEXT.md` - Lifecycle split and app/backend thinning direction that this phase must preserve.
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md` - Shared contract and naming/boundary cleanup direction already established in the repo.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/scoop/planner.rs` already exists at the domain root and is a natural destination for the single Scoop planning story.
- `spoon-backend/src/scoop/state/` already gives Scoop a dedicated canonical-state area and is a better home for state-related glue than `lifecycle/state.rs`.

### Established Patterns
- The repo now prefers layout-owned path truth, canonical state ownership, and thin app-shell glue.
- Earlier phases already accepted aggressive forward cleanup when names or abstractions no longer matched reality.

### Integration Points
- `spoon-backend/src/scoop/runtime/` is the main naming and responsibility mismatch hotspot.
- `spoon-backend/src/scoop/lifecycle/` still contains non-stage modules (`planner.rs`, `state.rs`) that muddy the directory meaning.
- `spoon-backend/src/scoop/mod.rs` is still a very wide facade and likely needs export slimming after the structural moves.
- `spoon-backend/src/scoop/query.rs`, `spoon-backend/src/scoop/info.rs`, and `spoon-backend/src/scoop/projection.rs` will need coordination so the structural refactor does not leave naming drift behind.

</code_context>

<specifics>
## Specific Ideas

- Treat this as a major Scoop-domain re-organization, not as a cosmetic rename pass.
- Prioritize names that tell a human reader what a module is for without having to open three other files first.
- Use external crates when they remove scaffolding or complexity, but do not add dependencies just because the project is refactoring.

</specifics>

<deferred>
## Deferred Ideas

- Full read-model/data-structure redundancy consolidation remains the primary Phase 12 follow-up.

</deferred>

---

*Phase: 11-scoop-runtime-host-and-helper-consolidation*
*Context gathered: 2026-04-01*
