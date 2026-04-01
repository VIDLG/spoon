# Phase 11: Scoop Runtime Host and Helper Consolidation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `11-CONTEXT.md` - this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 11-scoop-runtime-host-and-helper-consolidation
**Areas discussed:** runtime naming, lifecycle purity, operation entry placement, package source model placement, read-model direction, external-library posture

---

## Runtime Naming

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `runtime` | Preserve current name and only trim behavior | |
| Rename to `host` | Use a name that matches the desired thin edge-layer role | X |
| Rename to `adapter`/`orchestration` | Also more explicit than runtime, but less direct than `host` | |

**User's choice:** Rename to `host`.
**Notes:** The user agreed that `runtime` no longer matches the intended responsibility and should be renamed if the name is wrong.

---

## Lifecycle Purity

| Option | Description | Selected |
|--------|-------------|----------|
| Keep mixed contents | Leave `planner/state` style modules under `lifecycle` | |
| Keep `lifecycle` but move non-stage modules out | Preserve the name while making the directory semantically pure | X |
| Rename `lifecycle` too | Rework both names at once | |

**User's choice:** Keep `lifecycle`, but move non-pure contents out.
**Notes:** The user agreed that `lifecycle` is the more accurate name, but only if the directory actually contains lifecycle stages.

---

## Operation Entry Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `actions` under host/runtime | Leave operation entry in the edge layer | |
| Promote `actions` to Scoop domain root | Make operation entry a first-class domain module | X |
| Fold `actions` into lifecycle | Treat operation entry as another lifecycle file | |

**User's choice:** Promote `actions` to the Scoop domain root.
**Notes:** This keeps host thin and makes the real operation entry obvious.

---

## Package Source Model Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `runtime/source.rs` | Leave domain model under the edge layer | |
| Move to root as `package_source.rs` | Make it an explicit Scoop-domain model file | X |
| Merge into `manifest.rs` or `planner.rs` | Combine with adjacent concerns | |

**User's choice:** Move to root as `package_source.rs`.
**Notes:** The user agreed this was the clearest name and placement.

---

## Read-Model Direction

| Option | Description | Selected |
|--------|-------------|----------|
| Keep query/info split, shrink or rename projection | Preserve useful entry separation while reducing catch-all helpers | X |
| Merge query/info/projection into one read-model module | Consolidate all read-model logic now | |
| Delay all read-model changes | Focus only on host/lifecycle naming | |

**User's choice:** Keep query/info split and stop `projection` from acting like a public bucket.
**Notes:** The larger data-structure redundancy pass stays in Phase 12.

---

## External Libraries

| Option | Description | Selected |
|--------|-------------|----------|
| Prefer in-house refactors only | Avoid new dependencies during the refactor | |
| Use mature crates when they materially simplify structure | Allow external libraries if they clearly improve readability/maintenance | X |
| Aggressively replace internal code with crates | Treat refactor as a dependency adoption opportunity | |

**User's choice:** Use mature crates when they materially simplify structure.
**Notes:** The user explicitly invited external crates where they help, but not as cargo-cult additions.

---

## the agent's Discretion

- Exact final split of `host/execution`, `host/integration`, and `host/hooks`
- Whether `projection` becomes internal-only in this phase or simply gets prepared for Phase 12

## Deferred Ideas

- Broader read-model/data-structure de-duplication stays Phase 12 work.
