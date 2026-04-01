# Phase 7: Canonical MSVC State and Lifecycle - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 7 takes the seam-first MSVC skeleton from Phase 6 and turns it into a real backend state machine. This is the phase where MSVC stops being just "well-structured code" and becomes a domain with canonical SQLite-backed state, a shared lifecycle contract, and backend-owned reconciliation between control-plane records and runtime evidence.

This phase should land:
- canonical MSVC state in the existing SQLite control plane
- one shared lifecycle contract across `managed` and `official`
- strategy-specific execution/validation behind that shared contract
- status/query/doctor alignment around canonical state plus detection evidence
- focused regressions close to the new state/lifecycle paths

This phase should **not** absorb:
- backend event redesign
- backend error redesign
- full repair system design
- broad shared archive/download/cache extraction
- a second Scoop architecture rewrite

</domain>

<decisions>
## Implementation Decisions

### Canonical State
- **D-01:** MSVC canonical state must live in the existing SQLite control plane, not in a separate persistence mechanism.
- **D-02:** Canonical state should use one shared envelope with `runtime_kind`, shared lifecycle facts, and strategy-specific detail sections.
- **D-03:** Canonical state is the backend authoritative record, but it must be supported by detect/validate evidence rather than written as imagined state.
- **D-04:** Detection is an evidence / refresh / reconcile source, not a replacement for canonical state.
- **D-05:** Apply a strict **derive-not-store** rule to new MSVC canonical state and schema. If a field can be recomputed from neighboring canonical facts, layout, or current evidence, it should not be persisted unless there is a strong justification.

### Lifecycle
- **D-06:** `managed` and `official` share one high-level lifecycle contract.
- **D-07:** Strategy-specific differences belong inside execute/validate branches, not in two independent lifecycle languages.
- **D-08:** `validate` is part of the lifecycle/operation story, not a side-channel.

### Query / Doctor / Repair
- **D-09:** Status and query surfaces should read from canonical state plus evidence-backed reconciliation, not from scattered module-local state assumptions.
- **D-10:** `doctor` should begin aligning around canonical state + detect/query evidence in this phase.
- **D-11:** A full repair system is out of scope; only the interfaces and residue facts needed by future work should be prepared.

### Shared Utility Boundaries
- **D-12:** Phase 7 should reuse current IO helpers as needed, but large shared extraction of `archive` / `download` / `cache` primitives belongs to Phase 8.

### Out of Scope
- **D-13:** Do not redesign the global backend event contract here.
- **D-14:** Do not redesign the entire backend error hierarchy here.
- **D-15:** Do not collapse `managed` into `official` or vice versa.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / Planning State
- `.planning/PROJECT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`

### Prior Phase Decisions
- `.planning/phases/06-msvc-seams-and-ownership-completion/06-CONTEXT.md`
- `.planning/phases/06-msvc-seams-and-ownership-completion/06-VERIFICATION.md`
- `.planning/milestones/v0.5.0-ROADMAP.md`

### Carry-Forward Follow-ups
- `.planning/todos/pending/2026-04-01-audit-derive-not-store-fields.md`
- `.planning/todos/pending/2026-03-31-tighten-backend-error-contract.md`
- `.planning/seeds/SEED-001-backend-event-contract-hardening.md`
- `.planning/todos/pending/2026-03-31-consolidate-remaining-fsx-helpers.md`
- `.planning/todos/pending/2026-03-31-remove-hardcoded-production-paths.md`

### Relevant Code
- `spoon-backend/src/control_plane/schema/0001_control_plane.sql`
- `spoon-backend/src/control_plane/sqlite.rs`
- `spoon-backend/src/msvc/plan.rs`
- `spoon-backend/src/msvc/detect.rs`
- `spoon-backend/src/msvc/execute.rs`
- `spoon-backend/src/msvc/official.rs`
- `spoon-backend/src/msvc/query.rs`
- `spoon-backend/src/msvc/status.rs`
- `spoon-backend/src/msvc/rules.rs`
- `spoon-backend/src/msvc/tests/context.rs`
- `spoon-backend/src/msvc/tests/root.rs`
- `spoon-backend/src/msvc/tests/official.rs`
- `spoon/src/service/msvc/mod.rs`
- `spoon/tests/cli/msvc_flow.rs`
- `spoon/tests/tui/tui_msvc_download_flow.rs`

</canonical_refs>

<code_context>
## Existing Code Insights

### Current State Reality
- Managed runtime still persists installed/runtime facts through ad hoc JSON under the runtime root.
- Official runtime also persists its own local state files, but they are still strategy-specific and not integrated into the SQLite control plane.
- The SQLite control plane currently has Scoop-focused tables only.

### Current Lifecycle Reality
- Managed execution is now isolated enough to become the first strategy-specific branch under a shared lifecycle.
- Official execution already behaves like an alternate strategy, but it is still external-installer-heavy and evidence-led.
- Validate paths exist already, which means Phase 7 can promote validation into the formal operation model instead of inventing it from scratch.

### Design Pressure
- If Phase 7 stores too many derivable fields in SQLite, the new canonical state will start with the same drift problem we already want to avoid.
- If Phase 7 refuses to store enough lifecycle facts, doctor/status/reconcile will remain weak and strategy-specific.

</code_context>

<specifics>
## Specific Ideas

- Prefer schema/store designs that capture backend-trusted facts, timestamps, lifecycle residue, and strategy-specific evidence summaries without persisting giant blobs of raw detection output.
- Favor one shared lifecycle journal language even if execution internals differ sharply between `managed` and `official`.

</specifics>

<deferred>
## Deferred Ideas

- Global event contract redesign
- Global error contract redesign
- Full repair automation
- Broad shared IO primitive extraction
- Read-model convenience cleanup that does not materially affect canonical state correctness

</deferred>

---

*Phase: 07-canonical-msvc-state-and-lifecycle*
*Context gathered: 2026-04-01*
