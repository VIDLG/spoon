# Phase 4: Refactor Safety Net - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 does not introduce a new backend architecture direction. It hardens the Phase 1/2/2.1/3 refactor by adding focused safety coverage around the highest-risk Scoop lifecycle and app-shell seams.
This phase is about test weighting, failure-boundary coverage, recovery-boundary verification, and a small amount of real integration smoke coverage. It is not the phase for a full automatic repair engine, a large retry system, or broad PTY/end-to-end expansion.

</domain>

<decisions>
## Implementation Decisions

### Testing Weight
- **D-01:** Phase 4 should put most safety coverage in backend tests, not in app-side end-to-end flows.
- **D-02:** App tests should stay focused on translation, routing, and orchestration-shell behavior.
- **D-03:** Real integration tests should remain sparse and opt-in, not become the primary safety strategy.

### Failure Coverage Scope
- **D-04:** Phase 4 should focus on key failure boundaries rather than exhaustive success/failure duplication for every single stage.
- **D-05:** Priority backend failure coverage should include install/update hook failures, `persist_restoring` failure, `surface_applying` failure, `integrating` failure, and failures before `state_committing`.
- **D-06:** Priority backend failure coverage should include uninstall `pre_uninstall` / `uninstaller_script` failures, warning-only `post_uninstall`, operation-lock conflicts, and correct journal stop points.

### Test Layering
- **D-07:** Backend near-module tests should lock local lifecycle contracts such as stage ordering, fail-hard versus warning-only behavior, and outcome/projection rules.
- **D-08:** Backend integration tests should verify lifecycle + state/journal/lock/doctor composition semantics.
- **D-09:** App tests must not re-validate backend lifecycle correctness; they only validate app-shell translation/orchestration consumption of backend contracts.

### Repair / Retry Scope
- **D-10:** Phase 4 should not implement a full automatic repair or retry system.
- **D-11:** Phase 4 should instead verify that failures stop at explainable journal stages and remain diagnosable through doctor/reporting surfaces.
- **D-12:** The safety net should confirm recoverable-boundary semantics without expanding the project scope into a new repair subsystem.

### Real Integration Coverage
- **D-13:** Keep a small number of isolated, opt-in real integration smoke tests for the highest-value Scoop bucket and package flows.
- **D-14:** Do not expand real integration coverage into broad PTY-driven or network-heavy suites.

### Acceptance Shape
- **D-15:** Phase 4 should be judged by whether key user-facing regressions are prevented in risky lifecycle paths, not by coverage percentage.
- **D-16:** Safety-net planning may choose the exact mix of backend unit, backend integration, app CLI, and app TUI tests as long as the weighting above remains intact.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Direction
- `.planning/PROJECT.md` - Project-wide direction and ownership goals.
- `.planning/ROADMAP.md` - Official Phase 4 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - Requirement IDs TEST-01 through TEST-03.
- `AGENTS.md` - Repo-specific testing strategy and ownership constraints.

### Prior Phase Context
- `.planning/STATE.md` - Current milestone position and accumulated decisions.
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-VERIFICATION.md` - Control-plane verification that Phase 4 must preserve.
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-CONTEXT.md` - Phase 3 lifecycle/stage/event semantics that the safety net must now protect.
- `.planning/phases/03-scoop-lifecycle-split-and-app-thinning/03-VERIFICATION.md` - Verified lifecycle split outcomes and current regression anchors.

### Research and Codebase Maps
- `.planning/research/SUMMARY.md` - Project-level sequencing rationale and risk notes.
- `.planning/codebase/TESTING.md` - Existing testing strategy and gaps.
- `.planning/codebase/CONCERNS.md` - Current fragility and regression hotspots.
- `.planning/codebase/STRUCTURE.md` - Where backend and app tests live now.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/scoop/tests/runtime.rs`: Existing lifecycle regression tests added in Phase 3.
- `spoon-backend/src/tests/control_plane.rs`: Current control-plane verification surface.
- `spoon/tests/cli/scoop_runtime_flow.rs`: App-facing Scoop runtime flow regression suite.
- `spoon/tests/cli/status_backend_flow.rs`: App-side structured event/read-model regression coverage.
- `spoon/tests/tui/*`: Harness-oriented app-shell tests that should remain focused on orchestration/rendering rather than backend internals.

### Established Patterns
- Backend owns lifecycle correctness, state semantics, and control-plane truth.
- App shell translates backend events/results and should not regain backend behavior ownership.
- Structured lifecycle stages now exist and can be used as stable verification anchors.
- SQLite journal/lock/doctor surfaces already exist and can be asserted in focused backend tests.

### Integration Points
- `spoon-backend/src/scoop/lifecycle/*`: Main location for focused lifecycle risk coverage.
- `spoon-backend/src/control_plane/*`: Journal/lock/doctor assertions for failure-boundary safety.
- `spoon/src/service/scoop/*`: Translation-only app seam that needs lightweight regression protection.
- `spoon/tests/cli/*` and `spoon/tests/tui/*`: App-shell coverage layer for routing and presentation contracts.

</code_context>

<specifics>
## Specific Ideas

- Add focused backend tests around stage stop points and state/journal consistency rather than trying to exhaustively test every success path again.
- Preserve the current strategy of sparse opt-in real Scoop flows, especially for bucket and package smoke scenarios.
- Use lifecycle stage semantics as the main safety anchor for failure assertions.

</specifics>

<deferred>
## Deferred Ideas

- Full automatic retry/repair workflows remain out of scope for Phase 4.
- Large-scale PTY or network-heavy end-to-end suites remain out of scope.

</deferred>

---

*Phase: 04-refactor-safety-net*
*Context gathered: 2026-03-30*
