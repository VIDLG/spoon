# Phase 1: Backend Seams and Ownership - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 1-backend-seams-and-ownership
**Areas discussed:** backend context contract, OS integration boundary, query and state consumption, Git and bucket interfaces, layout ownership

---

## Backend Context Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Unified backend context | Introduce backend-owned `BackendContext` and `RuntimeLayout`; app passes config/root and backend derives runtime semantics. | X |
| Light cleanup only | Keep scattered `tool_root` and config helpers, only tidy local seams. | |
| Partial shared context | Add a small shared context now and defer stronger consolidation. | |

**User's choice:** Unified backend-owned context, with a forward-designed end state rather than an incremental compromise shape.
**Notes:** User explicitly preferred the ideal final design and did not want an over-engineered transition plan.

---

## OS Integration Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Backend owns all runtime side effects | PATH, home, layout, shim/surface, and integrations all move behind backend ownership. | |
| Backend orchestration plus app host | Keep a broad runtime host owned by the app. | |
| Mixed boundary | Move generic runtime side effects into backend, but keep Spoon-config-domain integrations behind narrow app-owned ports. | X |

**User's choice:** Mixed boundary.
**Notes:** PATH, home/layout, shim or command-surface behavior, and runtime orchestration should move into backend. App-owned package integrations stay app-owned because they are tied to Spoon's config domain.

---

## Query and State Consumption

| Option | Description | Selected |
|--------|-------------|----------|
| Full backend read-model ownership | App stops reading backend state files and stops reconstructing backend-facing status/detail models. | X |
| Scoop-first partial adoption | Only Scoop query/detail surfaces move first; the rest follow later. | |
| App keeps local projection layer | Backend returns raw data while app continues building local state projections. | |

**User's choice:** Full backend read-model ownership.
**Notes:** The app should consume backend read models directly so Phase 2 can establish a real canonical Scoop state without app-side shadow models.

---

## Git and Bucket Interfaces

| Option | Description | Selected |
|--------|-------------|----------|
| Domain-only app contracts | App consumes bucket/domain operations only; `gitx` stays internal to backend. | X |
| Expose both domain and generic Git APIs | App may call general backend Git sync primitives directly. | |
| Keep limited Git awareness in app | App understands some repo-sync concepts even if it avoids `gix` directly. | |

**User's choice:** Domain-only app contracts.
**Notes:** The split should keep Git runtime details out of the app entirely.

---

## Layout Ownership

| Option | Description | Selected |
|--------|-------------|----------|
| Backend single-owner layout | Backend derives all backend runtime paths from the configured root. | X |
| Shared helper split | App keeps a few common layout helpers while backend owns the rest. | |
| Dual derivation during migration | Both sides can derive paths temporarily, with backend as the reference. | |

**User's choice:** Backend single-owner layout.
**Notes:** The user agreed this should not be a compromise area; duplicate layout semantics would undermine later state cleanup.

---

## the agent's Discretion

None.

## Deferred Ideas

None.
