# Phase 2: Canonical Scoop State - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-03-28T23:04:22+08:00
**Phase:** 02-canonical-scoop-state
**Areas discussed:** canonical record baseline, migration and repair policy, persistence boundary, projection boundary

---

## Canonical Record Baseline

| Option | Description | Selected |
|--------|-------------|----------|
| Base canonical state on `InstalledPackageState` | Keep the richer operational/runtime state path and absorb the missing installed facts into it | x |
| Keep both state structs behind adapters | Preserve both `ScoopPackageState` and `InstalledPackageState` with translation layers | |
| Create a third parallel state model immediately | Add a brand-new state struct first, then migrate later | |

**User's choice:** Base canonical state on `InstalledPackageState`.
**Notes:** Also add `bucket` and `architecture` into the canonical record. No third model was requested.

---

## Migration And Repair Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Forward design, no compatibility | Move to the new canonical state model directly and do not preserve old-state compatibility layers | x |
| One-shot backend migration with compatibility intent | Rewrite old state but still treat compatibility as a supported concern during rollout | |
| Long-lived dual-read and dual-write compatibility | Keep legacy and canonical state contracts active together for safety | |

**User's choice:** Forward design, no compatibility.
**Notes:** The user explicitly chose not to preserve compatibility with the old Scoop state. Old state may require explicit repair or rebuild instead of compatibility handling.

---

## Persistence Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Persist only non-derivable facts | Rebuild roots and absolute paths from backend layout/context at read time | x |
| Persist mixed facts and layout paths | Keep some absolute/runtime paths in state for convenience | |
| Preserve current payload shape unless forced | Minimize churn even if redundant fields remain | |

**User's choice:** Persist only non-derivable facts.
**Notes:** Agreed: keep install facts such as `package`, `version`, `bucket`, `architecture`, `bins`, `shortcuts`, env/persist/integration and uninstall facts, but do not store absolute paths or `current` path data.

---

## Projection Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Move all query/detail/status consumers onto canonical projections | Canonical state feeds package info, status, uninstall, and reapply surfaces consistently | x |
| Convert only runtime writes first | Leave query/info surfaces partially duplicated until later | |
| Let app layers bridge remaining gaps | Use app adapters to smooth over backend state duplication | |

**User's choice:** Move all query/detail/status consumers onto canonical projections.
**Notes:** The user wants this done in one pass during Phase 2 rather than splitting writes first and reads later.

---

## the agent's Discretion

- The exact module split for canonical state, migration helpers, and projections can be decided during research/planning.

## Deferred Ideas

- None.
