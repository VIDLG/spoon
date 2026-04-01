# Phase 9: MSVC and Shared Safety Net - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 9 is the safety-net phase for the `v0.6.0` milestone. By this point:

- MSVC seams exist
- canonical MSVC state exists
- shared backend contracts have been hardened

That means the main remaining work is not more architecture invention; it is making sure the highest-risk lifecycle and shared-contract regressions are caught close to where they matter.

The phase should therefore stay focused on:
- backend-heavy regression coverage
- critical MSVC failure stops
- shared contract regressions
- narrow, opt-in real smoke

It should not reopen major domain redesign, new feature work, or broad reliability platform work.

</domain>

<decisions>
## Implementation Decisions

### Testing Strategy
- **D-01:** Phase 9 should remain backend-heavy: backend tests own lifecycle/state/doctor/shared-contract regressions.
- **D-02:** App tests should stay translation/orchestration-shell focused rather than re-owning backend semantics.

### Failure Coverage
- **D-03:** Focus on key failure boundaries, not a full success/failure matrix for every stage.
- **D-04:** Prioritize managed download/extract/materialize failures, validation failures, canonical-state commit boundaries, official bootstrapper failures, official detect/reconcile mismatches, uninstall cleanup correctness, and doctor visibility.

### Shared Contract Coverage
- **D-05:** Shared event/error/archive/path/port regressions should be tested in a layered way:
  - near-module backend tests for local contract shape
  - backend integration where cross-cutting interaction matters
  - app-shell translation tests only where user-visible translation must stay stable

### Real Smoke
- **D-06:** Keep real smoke sparse, isolated, and opt-in.
- **D-07:** MSVC/official/managed real smoke can exist where valuable, but should never become the primary regression strategy.

### Out of Scope
- **D-08:** Do not expand into new architecture phases here.
- **D-09:** Do not build broad flaky environment-driven test suites.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / Planning State
- `.planning/PROJECT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`

### Prior Phase Verification
- `.planning/phases/06-msvc-seams-and-ownership-completion/06-VERIFICATION.md`
- `.planning/phases/07-canonical-msvc-state-and-lifecycle/07-VERIFICATION.md`
- `.planning/phases/08-shared-backend-contract-hardening/08-VERIFICATION.md`

### Relevant Code / Tests
- `spoon-backend/src/msvc/tests/context.rs`
- `spoon-backend/src/msvc/tests/root.rs`
- `spoon-backend/src/msvc/tests/official.rs`
- `spoon-backend/src/tests/event.rs`
- `spoon-backend/src/scoop/tests/contracts.rs`
- `spoon/tests/cli/msvc_flow.rs`
- `spoon/tests/cli/status_backend_flow.rs`
- `spoon/tests/cli/scoop_flow.rs`
- `spoon/tests/tui/tui_msvc_download_flow.rs`
- `spoon/tests/tui/tui_output_modal_flow.rs`
- `AGENTS.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### What is already protected
- Focused MSVC seam, canonical-state, and doctor regressions now exist.
- Shared event contract has been reset and tested.
- Shared utility extraction and path hardening now have representative flow coverage.

### What still needs stronger protection
- Managed failure paths that now write canonical state.
- Official external-installer failures and reconcile mismatches.
- Shared contract drift at the backend/app shell boundary after the event reset.
- Path/port cleanup regressions that might not show up until later if left unguarded.

</code_context>

<specifics>
## Specific Ideas

- Prefer adding the minimum number of high-value tests that lock the most dangerous breakpoints.
- Keep real smoke ignored/opt-in and clearly labeled.

</specifics>

<deferred>
## Deferred Ideas

- Broad environment matrix testing
- Larger repair/reliability milestone work
- Feature coverage theater

</deferred>

---

*Phase: 09-msvc-and-shared-safety-net*
*Context gathered: 2026-04-01*
