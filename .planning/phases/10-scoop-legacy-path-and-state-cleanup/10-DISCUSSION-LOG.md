# Phase 10: Scoop Legacy Path and State Cleanup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `10-CONTEXT.md` - this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 10-scoop-legacy-path-and-state-cleanup
**Areas discussed:** legacy path/state API cleanup, Scoop path model, doctor legacy handling, cleanup priorities

---

## Legacy Path and State API Cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Remove from active runtime paths only | Keep legacy concepts around in limited compatibility-oriented surfaces | |
| Delete aggressively | Remove legacy path/state concepts as completely as possible instead of preserving compatibility helpers | X |
| Partial cleanup | Delete only the most obvious leftovers and defer the rest | |

**User's choice:** Delete aggressively.
**Notes:** The user explicitly wanted legacy path/state concepts deleted rather than preserved as a shadow subsystem.

---

## Scoop Path Model

| Option | Description | Selected |
|--------|-------------|----------|
| Thin `paths.rs` layer | Keep a small Scoop-specific path helper layer with only current-value helpers | |
| Layout-owned path model | Make `RuntimeLayout` / `ScoopLayout` the only formal path model and push package path semantics into layout-owned methods if needed | X |
| Keep `paths.rs` mostly intact | Delete legacy helpers but preserve the helper layer shape | |

**User's choice:** Layout-owned path model.
**Notes:** The user asked for the most ideal, forward-looking option with no backward-compatibility pressure.

---

## Doctor Legacy Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Thin legacy scan only | Keep a minimal residual scan for old JSON state while deleting the rest of the legacy API | |
| Delete legacy handling too | Do not retain doctor surfaces dedicated to old JSON-state residue | X |

**User's choice:** Delete legacy handling too.
**Notes:** The user emphasized that the product is still early enough to use this window to remove the old worldview completely.

---

## Cleanup Priority

| Option | Description | Selected |
|--------|-------------|----------|
| Model convergence + readability | Prioritize elegant, human-readable code that converges on the current backend model | X |
| Mechanical deletion first | Delete old code first and worry about readability later | |
| Local rewrites only | Improve selected files without forcing broader convergence | |

**User's choice:** Model convergence + readability.
**Notes:** The user explicitly wants Scoop code to be elegant and human-readable, not just technically cleaned.

---

## the agent's Discretion

- Whether any remaining package-level path helpers should become `ScoopLayout` methods or be inlined directly at call sites.

## Deferred Ideas

None.
