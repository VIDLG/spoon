# Phase 3: Scoop Lifecycle Split and App Thinning - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 03-scoop-lifecycle-split-and-app-thinning
**Areas discussed:** lifecycle structure, stage contract, hook boundary, hook failure policy, install/uninstall ordering, app/backend orchestration boundary, event semantics, recovery boundary

---

## Lifecycle Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Thin orchestration + reusable lifecycle modules | Keep `install/update/uninstall/reapply` entry points and split reusable modules underneath | x |
| One generic mega-lifecycle | Replace entry points with one master execution flow | |
| Mechanical file split only | Mostly split `actions.rs` by file without clear reusable lifecycle slices | |

**User's choice:** Thin orchestration + reusable lifecycle modules.
**Notes:** The selected reusable module spine is `planner -> acquire -> materialize -> persist -> surface -> integrate -> state`, with `current` staying inside `surface`.

---

## Reapply Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `reapply` as a distinct lifecycle entry point | Reapply post-install effects without doing a full reinstall | x |
| Represent it as uninstall + install | Model reapply as a heavy reinstall flow | |
| Drop it entirely | Fold reapply into ad hoc repair logic later | |

**User's choice:** Keep `reapply` as a distinct lifecycle entry point.
**Notes:** `reapply` should only run `persist_restoring -> surface_applying -> integrating -> state_committing`, and it should not run install/uninstall hooks.

---

## Stage Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Stable formal stage contract | Define explicit stage names used by backend, journal, doctor, repair, and UI | x |
| Internal-only stages | Use stage names only inside implementation | |
| No stage contract yet | Split modules first and define stage vocabulary later | |

**User's choice:** Stable formal stage contract.
**Notes:** The agreed stage names are:

Install / update:
- `planned`
- `acquiring`
- `materializing`
- `preparing_hooks`
- `persist_restoring`
- `surface_applying`
- `post_install_hooks`
- `integrating`
- `state_committing`
- `completed`

Uninstall:
- `planned`
- `pre_uninstall_hooks`
- `uninstalling`
- `persist_syncing`
- `surface_removing`
- `state_removing`
- `post_uninstall_hooks`
- `completed`

Reapply:
- `planned`
- `persist_restoring`
- `surface_applying`
- `integrating`
- `state_committing`
- `completed`

---

## Hook Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Shared hook execution module | `hooks.rs` owns how hooks execute, but lifecycle entry points decide when they run | x |
| Hook-aware orchestrator module | `hooks.rs` also owns some sequencing/orchestration | |
| Split hooks into each lifecycle file | Install/uninstall/reapply each own their own hook execution logic | |

**User's choice:** Shared hook execution module.
**Notes:** `hooks.rs` remains centralized but is not a standalone lifecycle phase.

---

## Hook Failure Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Install/uninstall core hooks fatal, `post_uninstall` warning-only | Fatal for install/update pre+installer+post and uninstall pre+uninstaller; warning-only for `post_uninstall` | x |
| All hook failures fatal | Any hook failure aborts the operation | |
| Most hooks warning-only | Preserve main operation whenever possible | |

**User's choice:** Install/uninstall core hooks fatal, `post_uninstall` warning-only.
**Notes:** Reapply does not run hooks.

---

## Ordering And Intermediate State

| Option | Description | Selected |
|--------|-------------|----------|
| Persist before live surface, state last | Restore persist before `current`, expose user-visible surface before state commit, and remove state before warning-only uninstall tail cleanup | x |
| Surface before persist | Activate first, hydrate later | |
| State earlier in the flow | Record success before the live surface and integrations are fully ready | |

**User's choice:** Persist before live surface, state last.
**Notes:** The user also agreed to the uninstall side: `surface_removing` is where entry points can disappear; `state_removing` must happen before warning-only `post_uninstall_hooks`.

---

## App / Backend Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| App can translate, but cannot direct | App constructs requests, maps events/results, and renders; backend owns lifecycle ordering and state semantics | x |
| App keeps some orchestration | Leave some lifecycle sequencing in `spoon/src/service/scoop/*` | |
| Shared orchestration model | Backend and app both maintain lifecycle-stage knowledge | |

**User's choice:** App can translate, but cannot direct.
**Notes:** The app may not infer backend state gaps or invent its own lifecycle ordering.

---

## Event Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Structured product-semantic events only | Logs stay in `tracing`; `BackendEvent` carries structured stage/progress/warning/blocking/result semantics | x |
| Mixed logs and events | Let backend events also act as a general text log stream | |
| App reconstructs lifecycle semantics from low-level messages | Backend emits mostly text/progress and app invents stages | |

**User's choice:** Structured product-semantic events only.
**Notes:** Ordinary logs belong to `tracing`. Event channels should not become a generic log pipe.

---

## Recovery Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 3 defines recoverable boundaries, Phase 4 implements full repair system | Lock stage/journal semantics now, but defer full retry/repair workflows | x |
| Build the full retry/repair system in Phase 3 | Lifecycle split and safety net land together | |
| Ignore recovery for now | Leave repair semantics undefined until later | |

**User's choice:** Phase 3 defines recoverable boundaries, Phase 4 implements the full repair system.
**Notes:** Phase 3 should leave behind enough stage/journal semantics that Phase 4 can build repair/retry on a stable contract.

---

## the agent's Discretion

- Exact file names and internal module layout can still be chosen during planning as long as the sequencing, stage contract, event semantics, and app/backend boundary stay intact.

## Deferred Ideas

- None.
