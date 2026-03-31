# Phase 3: Scoop Lifecycle Split and App Thinning - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 splits the current Scoop runtime monolith in `spoon-backend` into explicit lifecycle modules and reduces `spoon` to a thin app shell that only constructs requests, maps structured events, and renders outcomes.
This phase is about lifecycle structure, stage contract semantics, hook behavior preservation, install/update/uninstall/reapply ordering, and backend-owned progress semantics. It does not revisit canonical state design from Phase 2 or the SQLite control-plane direction from Phase 02.1, and it does not yet deliver the full repair/retry safety net reserved for Phase 4.

</domain>

<decisions>
## Implementation Decisions

### Lifecycle Structure
- **D-01:** Phase 3 should use thin orchestration entry points plus reusable lifecycle modules, not one mega-lifecycle and not a purely mechanical file split.
- **D-02:** Keep explicit backend entry points for `install`, `update`, `uninstall`, and `reapply`.
- **D-03:** The reusable lifecycle modules should be `planner -> acquire -> materialize -> persist -> surface -> integrate -> state`.
- **D-04:** `current` switching is not its own standalone lifecycle phase; it belongs inside `surface`.

### Reapply Semantics
- **D-05:** Keep `reapply` as its own lifecycle entry point; do not collapse it into `uninstall + install`.
- **D-06:** `reapply` means reapplying installed post-install effects without reacquiring payloads, rematerializing package contents, or changing version state.
- **D-07:** `reapply` should only run the back half of lifecycle behavior: `persist_restoring -> surface_applying -> integrating -> state_committing`.
- **D-08:** `reapply` should not run install/uninstall hooks.

### Stage Contract
- **D-09:** Phase 3 should define a stable lifecycle stage contract as a real backend contract, not merely internal implementation detail.
- **D-10:** Install/update stages should be:
  `planned -> acquiring -> materializing -> preparing_hooks -> persist_restoring -> surface_applying -> post_install_hooks -> integrating -> state_committing -> completed`
- **D-11:** Uninstall stages should be:
  `planned -> pre_uninstall_hooks -> uninstalling -> persist_syncing -> surface_removing -> state_removing -> post_uninstall_hooks -> completed`
- **D-12:** Reapply stages should be:
  `planned -> persist_restoring -> surface_applying -> integrating -> state_committing -> completed`

### Hook Boundary
- **D-13:** `hooks.rs` should stay as a shared execution module, not as a standalone lifecycle phase.
- **D-14:** `hooks.rs` owns how hooks are rendered and executed; lifecycle entry points own when hooks run.

### Hook Failure Policy
- **D-15:** `pre_install`, `installer_script`, and `post_install` failures are fatal and must stop install/update.
- **D-16:** `pre_uninstall` and `uninstaller_script` failures are fatal and must stop uninstall.
- **D-17:** `post_uninstall` failure is warning-only and must not invalidate the main uninstall result.

### Install / Update Ordering
- **D-18:** Install and update ordering should be:
  `planned, acquiring, materializing, preparing_hooks, persist_restoring, surface_applying, post_install_hooks, integrating, state_committing, completed`
- **D-19:** Before `surface_applying`, the new version must not be treated as live or ready.
- **D-20:** `persist_restoring` must happen before `surface_applying`, so the activated root is hydrated before it becomes live.
- **D-21:** `state_committing` must happen after integrations succeed, so backend state does not claim success before the live command surface is actually ready.

### Uninstall Ordering
- **D-22:** Uninstall ordering should be:
  `planned, pre_uninstall_hooks, uninstalling, persist_syncing, surface_removing, state_removing, post_uninstall_hooks, completed`
- **D-23:** `surface_removing` is the point where user-visible entry points may disappear; after this point the package is effectively non-usable.
- **D-24:** `state_removing` must happen before `post_uninstall_hooks`, so warning-only tail cleanup does not keep stale installed state alive.

### App / Backend Boundary
- **D-25:** `spoon/src/service/scoop/*` may only do request/context construction, event mapping, outcome mapping, cancellation wiring, and app-specific logging/telemetry glue.
- **D-26:** `spoon/src/service/scoop/*` must not orchestrate lifecycle steps, invent stage ordering, infer backend state gaps, or reconstruct lifecycle semantics.
- **D-27:** App can translate, but must not direct.

### Event Semantics
- **D-28:** Ordinary implementation logging should continue through `tracing`, not through backend events.
- **D-29:** `BackendEvent` should only carry structured product-semantic information such as stage changes, progress, warnings, blocking signals, and result summaries.
- **D-30:** Backend should emit authoritative structured lifecycle semantics; the app must consume and translate them, not invent its own parallel lifecycle model.

### Recovery Boundary
- **D-31:** Phase 3 should define failure stop points, journal semantics, and recoverable boundary markers for lifecycle stages.
- **D-32:** Phase 3 should not implement the full retry/repair/safety-net system; that fuller hardening belongs to Phase 4.
- **D-33:** In practice, Phase 3 must leave behind enough stage/journal semantics that Phase 4 can build repair/retry logic on a stable contract instead of reverse-engineering lifecycle internals.

### the agent's Discretion
- Research and planning may choose exact file names and internal module layout, as long as the lifecycle structure, stage contract, hook policy, event semantics, and app/backend boundary above remain intact.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Direction
- `.planning/PROJECT.md` - Project-wide refactor direction and ownership rules.
- `.planning/ROADMAP.md` - Official Phase 3 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - Requirement IDs SCLF-01 through SCLF-05.
- `AGENTS.md` - Repo-specific ownership, testing, and artifact constraints.

### Prior Phase Context
- `.planning/STATE.md` - Current milestone position and accumulated Phase 1/2/2.1 decisions.
- `.planning/phases/01-backend-seams-and-ownership/01-CONTEXT.md` - Phase 1 seam and app/backend boundary decisions.
- `.planning/phases/02-canonical-scoop-state/02-CONTEXT.md` - Phase 2 canonical state decisions that must remain stable during lifecycle splitting.
- `.planning/phases/02-canonical-scoop-state/02-VERIFICATION.md` - Verified Phase 2 result showing canonical state is already the single read/write source.
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-CONTEXT.md` - SQLite control-plane and sync-core / async-edge decisions that now constrain lifecycle design.
- `.planning/phases/02.1-sqlite-control-plane-and-sync-async-boundary/02.1-VERIFICATION.md` - Verified SQLite control-plane direction that Phase 3 must build on.

### Research and Codebase Maps
- `.planning/research/SUMMARY.md` - Project-level sequencing advice and lifecycle split warnings.
- `.planning/codebase/STRUCTURE.md` - Current crate and module layout.
- `.planning/codebase/CONVENTIONS.md` - Existing module, error, and helper conventions.
- `.planning/codebase/CONCERNS.md` - Current Scoop lifecycle fragility, rollback concerns, and test gaps.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spoon-backend/src/scoop/runtime/actions.rs`: Current orchestration center that reveals the lifecycle boundaries to carve out.
- `spoon-backend/src/scoop/runtime/hooks.rs`: Shared hook execution machinery worth preserving as a centralized helper module.
- `spoon-backend/src/scoop/runtime/persist.rs`: Existing persist helpers already close to a dedicated lifecycle slice.
- `spoon-backend/src/scoop/runtime/surface.rs`: Existing shim/shortcut/current-entry behavior that naturally fits a `surface` module.
- `spoon-backend/src/scoop/state/*`: Canonical state/store/projection layer completed in Phase 2 and now ready to be consumed by lifecycle modules.
- `spoon-backend/src/control_plane/*`: SQLite control-plane and journal/lock scaffolding from Phase 02.1 that should now carry lifecycle stage semantics.

### Established Patterns
- Backend owns runtime context, layout, state, and control-plane metadata.
- Scoop-specific host callbacks are narrowed under `spoon-backend/src/scoop/ports.rs`.
- App/service code should stay translation-only and should not regain lifecycle ownership.
- Ordinary logs stay in `tracing`; product-level lifecycle semantics travel through structured backend events.

### Integration Points
- `spoon-backend/src/scoop/runtime/actions.rs`: Primary carve-up target for install/update/uninstall/reapply orchestration.
- `spoon-backend/src/control_plane/*`: Storage location for stage/journal/lock semantics that Phase 3 must align with.
- `spoon/src/service/scoop/*`: App shell layer that must lose remaining orchestration knowledge and become translation-only.
- `spoon/tests/cli/*` and `spoon/tests/tui/*`: App-facing regressions that should stay focused on routing/progress rendering rather than backend internals.

</code_context>

<specifics>
## Specific Ideas

- Preserve `reapply` as a distinct lifecycle entry point because it replays post-install effects without doing a full reinstall.
- Keep `hooks.rs` centralized, but treat it as a shared execution helper rather than a top-level lifecycle phase.
- Make backend-emitted lifecycle stages authoritative so journal, doctor, repair, and UI progress surfaces all share one semantic vocabulary.
- Let Phase 3 define recoverable stage boundaries and journal semantics, but leave the full retry/repair system to Phase 4.

</specifics>

<deferred>
## Deferred Ideas

None - this discussion stayed within Phase 3 lifecycle ownership, sequencing, event semantics, and recovery boundaries.

</deferred>

---

*Phase: 03-scoop-lifecycle-split-and-app-thinning*
*Context gathered: 2026-03-29*
