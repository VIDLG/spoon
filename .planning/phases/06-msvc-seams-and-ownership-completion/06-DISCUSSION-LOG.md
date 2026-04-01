# Phase 6: MSVC Seams and Ownership Completion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 06-msvc-seams-and-ownership-completion
**Areas discussed:** domain/module shape, `managed` vs `official`, canonical state, lifecycle contract, app/backend seam, phase scope, shared utility extraction

---

## Module Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| `detect / plan / execute / state-query` | Structure MSVC by responsibility rather than by user action names | x |
| `install / update / remove / doctor / status` | Structure directly by top-level actions | |
| `detect / install / status / common` | Minimal, more conservative split | |

**User's choice:** `detect / plan / execute / state-query`.
**Notes:** Scoop remains lifecycle-first, but MSVC should use a parallel architecture philosophy with a shape that fits environment/toolchain provisioning.

---

## `managed` vs `official`

| Option | Description | Selected |
|--------|-------------|----------|
| Keep both as runtime strategies in one MSVC domain | Preserve both paths but stop modeling them as unrelated products | x |
| Remove `managed` now | Collapse onto official installer only | |
| Delay decision entirely | Avoid framing them together yet | |

**User's choice:** Keep both as runtime strategies in one domain.
**Notes:** Deleting `managed` now would be a product-direction change, not just architecture cleanup.

---

## Canonical State

| Option | Description | Selected |
|--------|-------------|----------|
| One canonical state with `runtime_kind` | Shared envelope + shared facts + strategy-specific detail | x |
| Separate persisted states for managed and official | Unified status only at read-model time | |
| No canonical persisted state yet | Read-model unification only | |

**User's choice:** One canonical state with `runtime_kind`.
**Notes:** This keeps MSVC from replaying the same multi-model drift that Scoop already had to clean up.

---

## Lifecycle Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Formal MSVC lifecycle contract with MSVC-specific stages | Use `planned / detecting / resolving / executing / validating / state_committing / completed` style stages | x |
| Internal stages only | No formal event/journal/test contract | |
| Delay lifecycle formalization until later | Focus only on seams and state for now | |

**User's choice:** Formal MSVC lifecycle contract with MSVC-specific stages.
**Notes:** The backend domain should become explicit enough that events, tests, and later diagnostics can share the same stage language.

---

## App / Backend Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| App expresses runtime preference only; backend owns orchestration | Thin-shell app boundary | x |
| App keeps separate managed/official flow façades | Thinner than today but still runtime-specific | |
| App hides runtime kind entirely | Backend auto-selects everything | |

**User's choice:** App expresses runtime preference only; backend owns orchestration.
**Notes:** Runtime choice may still be a product-visible preference, but runtime-specific internal behavior should not live in the app.

---

## Phase 6 Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 6 establishes seams, boundaries, and contracts; Phase 7 lands canonical state + lifecycle execution | Staged restructuring | x |
| Phase 6 does seams + canonical state; Phase 7 does lifecycle | |
| Phase 6 does seams + state + lifecycle all together | |

**User's choice:** Phase 6 establishes seams, boundaries, and contracts only.
**Notes:** The goal is to put the correct skeleton in place before the deeper rewrite.

---

## Shared Utility Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `fsx` narrow and extract shared `archive` / `download` / maybe `cache` later | Shared primitives, separate orchestration | x |
| Fold download/extract/cache into `fsx` | |
| Ignore shared utility overlap for now | |

**User's choice:** Keep `fsx` narrow; extract shared primitive modules later.
**Notes:** Scoop and MSVC share IO primitives, not one unified high-level workflow.

---

## the agent's Discretion

- Planning may decide the exact cut between Phase 6 and Phase 7 modules as long as Phase 6 remains seam-first and Phase 7 remains the place where canonical state and lifecycle execution land.

## Deferred Ideas

- Deleting `managed`
- Full canonical MSVC persistence
- Full lifecycle execution rewrite
- Broad Scoop follow-up work
