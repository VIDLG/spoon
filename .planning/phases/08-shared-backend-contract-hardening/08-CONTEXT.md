# Phase 8: Shared Backend Contract Hardening - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 8 is the shared architecture hardening phase of milestone `v0.6.0`. Scoop and MSVC are now both real backend domains, which means the remaining weaknesses are no longer domain-local first; they are shared contract problems:

- weak event taxonomy
- broad and drifting error semantics
- duplicated download/archive/cache/fs primitives
- overlapping host/port boundaries
- legacy layout/path concepts that still reflect the old JSON control-plane world

This phase should tighten those shared contracts directly, without reopening major lifecycle/state work from previous phases and without trying to build over-ambitious new frameworks.

</domain>

<decisions>
## Implementation Decisions

### Event Contract
- **D-01:** Event contract work should use forward design, not backward compatibility preservation.
- **D-02:** The goal is a contract reset for backend/app/tests together, not a compatibility layer.
- **D-03:** Keep ordinary logs in `tracing`; events should carry product semantics only.
- **D-04:** Strengthen event typing, split overloaded event semantics, and improve result/notice representation without turning this into an oversized event platform rewrite.

### Error Contract
- **D-05:** Error contract work should also be forward-looking.
- **D-06:** Tighten the distinction between domain, infrastructure, and user-action-needed failures.
- **D-07:** Reduce high-value `Other(String)` drift, but do not attempt a maximal full-rewrite of every error site in one pass.

### Shared Utility Extraction
- **D-08:** `fsx` should remain filesystem-primitive-focused.
- **D-09:** Shared `archive` / `download` / maybe `cache` utilities should be extracted as separate primitive modules where duplication justifies it.
- **D-10:** Shared utility work should unify primitives, not force Scoop and MSVC into one high-level workflow.

### Port Boundaries
- **D-11:** `SystemPort` should become narrower and focus on genuine host-environment mutation.
- **D-12:** `home_dir()` should be removed from `SystemPort`.
- **D-13:** `ScoopRuntimeHost` should stop redundantly redefining generic system capabilities where it can compose clearer generic/domain boundaries instead.

### Layout / Path Legacy Cleanup
- **D-14:** Perform a targeted legacy sweep of path/layout concepts that still reflect the old JSON-era control plane.
- **D-15:** Focus on fields/helpers like `package_state_root`, `bucket_registry_path`, and related JSON-era layout assumptions rather than expanding to a repo-wide path cleanup.

### Out of Scope
- **D-16:** Do not redesign MSVC/Scoop lifecycle/state again here.
- **D-17:** Do not build a full repair system here.
- **D-18:** Do not expand Phase 8 into unrelated product-surface work.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone / Planning State
- `.planning/PROJECT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`

### Prior Phase Outputs
- `.planning/phases/06-msvc-seams-and-ownership-completion/06-VERIFICATION.md`
- `.planning/phases/07-canonical-msvc-state-and-lifecycle/07-VERIFICATION.md`
- `.planning/milestones/v0.5.0-ROADMAP.md`

### Carry-Forward Follow-ups
- `.planning/seeds/SEED-001-backend-event-contract-hardening.md`
- `.planning/todos/pending/2026-03-31-tighten-backend-error-contract.md`
- `.planning/todos/pending/2026-03-31-consolidate-remaining-fsx-helpers.md`
- `.planning/todos/pending/2026-03-31-remove-hardcoded-production-paths.md`
- `.planning/todos/pending/2026-04-01-audit-derive-not-store-fields.md`
- `.planning/todos/pending/2026-04-01-simplify-system-port-and-runtime-host-boundaries.md`

### Relevant Code
- `spoon-backend/src/event.rs`
- `spoon-backend/src/error.rs`
- `spoon-backend/src/fsx.rs`
- `spoon-backend/src/ports.rs`
- `spoon-backend/src/layout.rs`
- `spoon-backend/src/scoop/extract.rs`
- `spoon-backend/src/scoop/runtime/download.rs`
- `spoon-backend/src/scoop/runtime/execution.rs`
- `spoon-backend/src/msvc/mod.rs`
- `spoon-backend/src/msvc/execute.rs`
- `spoon-backend/src/msvc/official.rs`
- `spoon-backend/src/msvc/state.rs`
- `spoon/src/service/mod.rs`
- `spoon/src/service/msvc/mod.rs`
- `spoon/src/service/scoop/runtime.rs`
- `AGENTS.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Event / Error
- Event contract is still functional, but `ProgressEvent` remains overloaded and `FinishEvent` is still too thin for ideal product semantics.
- Error handling has a usable backbone, but broad fallback variants still carry too much recurring meaning.

### Shared Utility Pressure
- Scoop and MSVC both own real download/extract/cache primitives.
- The duplication is at the primitive level more than at the domain-flow level.

### Ports / Layout
- `SystemPort` currently mixes generic environment mutation with `home_dir()` as a path-source helper.
- `ScoopRuntimeHost` repeats some generic system capability signatures instead of cleanly composing generic and Scoop-specific host responsibilities.
- Layout still contains JSON-era concepts such as `package_state_root` and `bucket_registry_path` that no longer match the SQLite control-plane truth model.

</code_context>

<specifics>
## Specific Ideas

- Keep Phase 8 strongly contract-oriented: make backend/app/tests agree on cleaner shared contracts without trying to solve every adjacent problem.
- Prefer a few high-confidence shared primitive modules over one giant shared runtime abstraction.

</specifics>

<deferred>
## Deferred Ideas

- Full repair automation
- Broader reliability milestone beyond targeted safety-net work
- Large product-surface feature expansion unrelated to backend contracts

</deferred>

---

*Phase: 08-shared-backend-contract-hardening*
*Context gathered: 2026-04-01*
