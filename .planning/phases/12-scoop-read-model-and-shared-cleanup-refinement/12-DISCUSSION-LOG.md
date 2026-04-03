# Phase 12: Scoop Read Model and Shared Cleanup Refinement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `12-CONTEXT.md` - this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 12-scoop-read-model-and-shared-cleanup-refinement
**Areas discussed:** DTO deletion, default reuse rule, derive-not-store strictness, projection handling, `schemars`

---

## DTO Cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Keep most DTOs | Only trim a few obvious duplicates | |
| Delete pure pass-through DTOs | Remove structs that only mirror domain/state data without adding contract value | X |
| Rebuild all DTOs from scratch | Larger read-model rewrite | |

**User's choice:** Delete pure pass-through DTOs.
**Notes:** The user explicitly chose an aggressive cleanup of redundant DTOs.

---

## Default Reuse Rule

| Option | Description | Selected |
|--------|-------------|----------|
| Default preserve | Keep separate read models unless proven redundant | |
| Default reuse | Reuse domain/state models by default and justify exceptions | X |
| Fully separate read-model world | Always project into dedicated outward structs | |

**User's choice:** Default reuse.
**Notes:** Independent read models survive only as explicit exceptions with real boundary value.

---

## Derive-Not-Store / Low-Value Fields

| Option | Description | Selected |
|--------|-------------|----------|
| Strict only for persisted state | Leave read models looser | |
| Strict for read models too | Remove low-value derived fields even from output DTOs unless they clearly help the contract | X |
| Only delete counts | Limit cleanup to the most obvious derived fields | |

**User's choice:** Strict for read models too.
**Notes:** Counts such as `bucket_count`, `installed_package_count`, and `match_count` are part of this cleanup target.

---

## Projection Layer

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `projection` as a public concept | Continue treating it like a first-class layer | |
| Make `projection` unimportant | Keep it only as an internal helper pool and shrink it over time | X |
| Split projection immediately | Turn it into multiple new helper modules in this phase | |

**User's choice:** Make `projection` unimportant.
**Notes:** The ideal end state is that `projection.rs` is no longer architecturally important.

---

## `schemars`

| Option | Description | Selected |
|--------|-------------|----------|
| Ignore schema tooling | Use `serde` only | |
| Consider `schemars` selectively | Apply schema generation to the true surviving outward-facing contracts | X |
| Apply schema widely | Derive schema for most Scoop structs | |

**User's choice:** Consider `schemars` selectively.
**Notes:** The user wants `serde` by default and `schemars` evaluated seriously, but not sprayed across internal models.

---

## the agent's Discretion

- Exact list of output structs that deserve to survive as formal contract-bearing DTOs
- Whether `schemars` is introduced in this phase or only prepared for as a near-term follow-up

## Deferred Ideas

- None beyond the already-deferred larger shared cleanup/backlog topics.
