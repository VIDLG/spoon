# Phase 8: Shared Backend Contract Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 08-shared-backend-contract-hardening
**Areas discussed:** event contract, error contract, shared utility extraction, port boundaries, layout/path legacy cleanup

---

## Event Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Forward-designed contract reset | No compatibility layer; backend/app/tests move together to a stronger event contract | x |
| Large event-platform redesign | |
| Minimal/no change | |

**User's choice:** Forward-designed contract reset.
**Notes:** Keep the scope to contract hardening, not an oversized event platform.

---

## Error Contract

| Option | Description | Selected |
|--------|-------------|----------|
| Forward contract hardening with controlled scope | Tighten domain/infrastructure/user-action-needed boundaries without rewriting everything | x |
| Maximal error-platform redesign | |
| Small opportunistic fixes only | |

**User's choice:** Forward contract hardening with controlled scope.
**Notes:** Unify the contract, but avoid churn-heavy empire building.

---

## Shared Utility Extraction

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `fsx` narrow and extract `archive` / `download` / maybe `cache` primitives | x |
| Fold everything into `fsx` | |
| Defer shared utility extraction entirely | |

**User's choice:** Keep `fsx` narrow and extract shared primitive modules.
**Notes:** Share primitives, not whole domain workflows.

---

## Port Boundaries

| Option | Description | Selected |
|--------|-------------|----------|
| Narrow `SystemPort`, remove `home_dir()`, and reduce `ScoopRuntimeHost` duplication | x |
| Minor naming cleanup only | |
| Collapse all host layers together | |

**User's choice:** Narrow `SystemPort`, remove `home_dir()`, and reduce host duplication.
**Notes:** Generic system mutation and domain-specific host behavior should be easier to reason about.

---

## Layout / Path Legacy Cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Targeted sweep of JSON-era layout/path leftovers | x |
| Opportunistic cleanup only | |
| Large repo-wide path cleanup | |

**User's choice:** Targeted JSON-era legacy sweep.
**Notes:** Focus on stale JSON-control-plane layout concepts, not a giant path refactor.

---

## the agent's Discretion

- Planning may choose the most execution-efficient ordering among event, error, shared utility, port, and layout cleanup as long as Phase 8 remains a shared-contract hardening phase rather than a disguised new architecture milestone.

## Deferred Ideas

- Full repair system
- Larger reliability platform work
