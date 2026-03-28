# Phase 1: Backend Seams and Ownership - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 defines the permanent seam between `spoon` and `spoon-backend`.
`spoon` remains the CLI/TUI app shell and owner of Spoon-specific configuration domains.
`spoon-backend` becomes the single owner of backend runtime context, layout derivation, Git/Scoop/MSVC operational behavior, and backend read models.

This phase clarifies ownership and interface shape only. It does not introduce new end-user capabilities, and it does not attempt the deeper canonical Scoop state cleanup reserved for Phase 2.

</domain>

<decisions>
## Implementation Decisions

### Backend Context and Runtime Ownership
- **D-01:** Phase 1 uses forward design, not a compatibility-heavy transition layer.
- **D-02:** `spoon-backend` will introduce an explicit backend-owned runtime context contract, centered on a `BackendContext` plus backend-owned `RuntimeLayout`.
- **D-03:** Backend operations should consume explicit context instead of scattered `tool_root`, `proxy`, module-local config helpers, or implicit global runtime configuration.

### OS Integration Boundary
- **D-04:** The OS/runtime boundary is intentionally mixed.
- **D-05:** Generic runtime side effects move into backend ownership: PATH handling, runtime home/layout semantics, shim or command-surface behavior, and backend lifecycle orchestration.
- **D-06:** Spoon-specific configuration writes remain app-owned behind narrow ports only when they belong to Spoon's config domain, such as app-owned package integrations and other product configuration surfaces.

### Query and State Consumption
- **D-07:** `spoon` should stop directly reading backend state files or reconstructing backend status/detail semantics locally.
- **D-08:** Backend read/query models become the single source the app consumes for runtime status, package detail, and related backend-facing display data.

### Git and Bucket Interfaces
- **D-09:** The app should consume bucket and backend domain interfaces only. `gitx` remains an internal backend implementation detail and should not shape app contracts.

### Layout Ownership
- **D-10:** Backend layout derivation is single-owned by `spoon-backend`. The app knows the configured `root`, but backend derives `scoop`, `msvc`, `shims`, `state`, `cache`, and related runtime paths.
- **D-11:** App-side backend path helpers are legacy seams to remove, not a pattern to preserve.

### the agent's Discretion
No additional discretion was requested during discussion. Planning should treat the decisions above as locked.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Contracts
- `.planning/PROJECT.md` - Project-level refactor direction, core value, and non-negotiable backend ownership goals.
- `.planning/ROADMAP.md` - Phase ordering, dependencies, and official Phase 1 scope and success criteria.
- `.planning/REQUIREMENTS.md` - Requirement mapping for backend boundaries, Git ownership, and layout/context ownership.
- `AGENTS.md` - Repository-level constraints for Spoon direction, ownership, path rules, and testing strategy.

### Codebase Maps
- `.planning/codebase/STRUCTURE.md` - Current crate and module split, including where the app/service seam currently lives.
- `.planning/codebase/CONVENTIONS.md` - Existing module/interface conventions and error-boundary expectations.
- `.planning/codebase/STACK.md` - Current platform/runtime stack, including `gix`, `tokio`, and app/backend dependencies.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/scoop/query.rs`: Existing backend query/read-model surface that can become the app's canonical backend-facing status source.
- `spoon-backend/src/gitx.rs`: Existing backend-owned Git implementation that already keeps `gix` out of the UI layer in principle.
- `spoon-backend/src/scoop/runtime/actions.rs`: Existing backend lifecycle entry path that anchors Scoop orchestration ownership.

### Established Patterns
- `spoon/src/service/mod.rs`: Current adapter layer already translates backend errors and events, but it still owns runtime config and helper seams that should shrink.
- `spoon/src/service/scoop/runtime.rs`: Current `AppScoopRuntimeHost` shows where runtime ownership still leaks into the app.
- `spoon/src/service/scoop/actions.rs`: The app currently derives tool root, loads backend config, and helps assemble Scoop plans; these are phase-1 seam issues.
- `spoon/src/status/mod.rs`: The app still derives installed size and inspects managed Scoop state locally, which conflicts with the desired backend read-model ownership.

### Integration Points
- `spoon/src/actions/execute/mod.rs`: Main app entry where tool actions are partitioned and routed into backend-backed execution.
- `spoon/src/config/paths.rs`: Current app-owned path derivation module that should lose backend layout semantics once `RuntimeLayout` exists.
- `spoon/src/service/scoop/bucket.rs`: App-facing bucket adapter that should end up speaking only backend bucket/domain contracts.

</code_context>

<specifics>
## Specific Ideas

- Keep the crate split. The problem is not `spoon` versus `spoon-backend` as a concept; the problem is that the seam is still leaky.
- Do not preserve app-side backend knowledge just to ease migration. Treat app-owned backend path/state/orchestration code as debt to remove.
- Keep package integrations as a narrow app-owned port only where they truly belong to Spoon's own configuration domain.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within Phase 1 ownership and seam design.

</deferred>

---

*Phase: 01-backend-seams-and-ownership*
*Context gathered: 2026-03-28*
