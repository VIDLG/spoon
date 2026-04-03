# Phase 12: Scoop Read Model and Shared Cleanup Refinement - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase refines the Scoop read-model and adjacent shared cleanup after the structural refactor of Phase 11.

The goal is to remove low-value DTO duplication, eliminate low-value derivable fields, and keep only the read models that have real boundary value. This phase does **not** reopen the topology refactor from Phase 11.

</domain>

<decisions>
## Implementation Decisions

### DTO Cleanup
- **D-01:** Pure pass-through DTOs should be deleted.
- **D-02:** If a struct is just a field-for-field copy of an existing domain/state model and provides no extra contract value, default to reusing the original structure.
- **D-03:** Independent read-model structs should remain only when they provide real boundary value through combination, reshaping, naming, or output-contract stabilization.

### Default Rule
- **D-04:** Default to reusing domain/state structures; preserve independent read models only as explicit exceptions.
- **D-05:** This phase should treat duplicated read-model structs as guilty until proven useful.

### Derive-Not-Store / Low-Value Fields
- **D-06:** Low-value derivable fields should be removed from read models unless they clearly improve the external contract.
- **D-07:** This includes counts and similarly trivial derived values such as `bucket_count`, `installed_package_count`, and `match_count` when adjacent collections already exist.
- **D-08:** The derive-not-store principle applies here even though these are read models rather than canonical persisted state.

### Projection Layer
- **D-09:** `projection.rs` should no longer be treated as a public layer.
- **D-10:** The ideal direction is to make `projection.rs` unimportant: an internal helper pool rather than a first-class architectural concept.
- **D-11:** This phase should delete unused helpers and shrink the module, but it does not need to perform a large helper split if that would distract from DTO/read-model cleanup.

### `query` / `info` Split
- **D-12:** Keep `query.rs` and `info.rs` as separate modules.
- **D-13:** Do not collapse all read models into one mega module.

### `schemars`
- **D-14:** `serde` remains the default serialization mechanism.
- **D-15:** `schemars` is worth considering selectively for the output structs that survive this cleanup and represent true JSON/read-model contracts.
- **D-16:** Do not schema-derive every internal helper or state structure; use `schemars` only where it helps identify and stabilize the real outward-facing contract.

### External Libraries
- **D-17:** External crates may be adopted when they clearly reduce boilerplate or sharpen the contract boundary.
- **D-18:** Do not introduce DTO mapper-style abstractions just to automate conversions; they would obscure the cleanup rather than clarify it.

### the agent's Discretion
- The exact set of surviving outward-facing read-model structs can be determined during execution as long as the default rule stays "reuse unless a distinct contract is justified."
- `schemars` may be introduced in a narrow, targeted way if a small set of JSON-facing structs clearly benefits from schema-backed contract hardening.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone and Scope
- `.planning/PROJECT.md` - Current milestone framing and backend readability goals.
- `.planning/ROADMAP.md` - Phase 12 goal and milestone boundary.
- `.planning/REQUIREMENTS.md` - Phase 12 requirement mapping for `SLEG-03` and `BECT-06`.
- `.planning/STATE.md` - Current milestone position and guardrails.

### Prior Scoop Cleanup
- `.planning/phases/10-scoop-legacy-path-and-state-cleanup/10-VERIFICATION.md` - Confirms the old path/state worldview is already gone.
- `.planning/phases/11-scoop-runtime-host-and-helper-consolidation/11-VERIFICATION.md` - Confirms structural cleanup is complete and Phase 12 can focus on read-model/data cleanup.

### Shared Contract Context
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md` - Prior derive-not-store and contract-hardening decisions relevant to DTO cleanup.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scoop/state/` already provides canonical state models and projection helpers that many read surfaces should reuse instead of duplicating.
- `status.rs` already aggregates backend snapshots and is a good place to verify which read models genuinely need a stable app-facing shape.

### Established Patterns
- The repo now prefers layout-owned paths, canonical state ownership, and explicit edge-layer boundaries.
- Previous phases have already established that forward cleanup is preferred over preserving weak compatibility shapes.

### Integration Points
- `spoon-backend/src/scoop/query.rs` contains obvious low-value derived fields and DTOs close to domain-shaped data.
- `spoon-backend/src/scoop/info.rs` contains many output structs, some of which are likely legitimate contract surfaces and some of which may be over-sliced.
- `spoon-backend/src/status.rs` duplicates parts of Scoop summary data again for app/backend snapshot purposes.
- `spoon-backend/src/scoop/projection.rs` still acts as a large helper bucket and should be demoted further.

</code_context>

<specifics>
## Specific Ideas

- Treat this phase as "contract clarification through deletion."
- `schemars` should be evaluated as a way to mark the output structs that truly deserve to survive as formal contracts.
- The ideal result is fewer structs, fewer counts, and a clearer sense of which outputs are actually stable contracts.

</specifics>

<deferred>
## Deferred Ideas

- Broader repo-wide cleanup of all deprecated helper APIs remains outside this phase.
- Any major archive/runtime abstraction expansion beyond what Scoop directly touches remains deferred to later backlog/seed review.

</deferred>

---

*Phase: 12-scoop-read-model-and-shared-cleanup-refinement*
*Context gathered: 2026-04-03*
