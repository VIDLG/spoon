# Phase 12: Scoop Read Model and Shared Cleanup Refinement - Research

**Researched:** 2026-04-03
**Domain:** DTO/read-model cleanup and contract clarification for the Scoop backend surface
**Confidence:** HIGH

## Summary

After Phase 11, the main remaining Scoop cleanup is no longer about directory topology. It is about data-shape discipline.

The current codebase shows three recurring issues:

1. read-model structs that simply mirror domain/state structs
2. low-value derived fields kept alongside the collections they derive from
3. an oversized `projection.rs` helper bucket that still looks more central than it should

The most effective cleanup approach is:

- delete pure pass-through DTOs
- reuse domain/state structures by default
- keep dedicated outward models only when they provide real contract value
- remove low-value derived fields, including counts, unless they materially improve the API boundary
- keep `query` and `info` separate
- demote `projection.rs` into an internal support file rather than a public concept

`schemars` fits this phase as a **contract litmus test**, not as a magic DTO-cleanup tool. If a read model is important enough to schema-derive and document, it probably deserves to exist as a separate outward struct. If not, it is a candidate for deletion or reuse.

## Phase Requirements

| ID | Description | Planning Implication |
|----|-------------|----------------------|
| SLEG-03 | Scoop read models and status/detail outputs avoid low-value derivable redundancies | Remove low-value counts and duplicate output-only wrappers. |
| BECT-06 | Shared helper debt touched by the Scoop cleanup stays aligned with backend-owned contract rules | Keep projection/internal helper cleanup aligned with the backend-owned contract model. |

## Recommended Plan Order

1. Audit and delete pure pass-through DTOs in `query/info/status`
2. Remove low-value derived fields and counts
3. Demote/shrink `projection.rs` and align any surviving outward contracts
4. Evaluate narrow `schemars` use for the surviving outward-facing read models and verify behavior

## Current Code Reality

- `query.rs` duplicates obvious domain-shaped structures such as bucket/package entries and retains count fields adjacent to collections.
- `status.rs` mirrors parts of the Scoop summary layer again for backend snapshot purposes.
- `info.rs` contains many output structs; some are justified contract surfaces, some likely over-split the model.
- `projection.rs` has already stopped being re-exported publicly, which makes it easier to demote further in this phase.

## `schemars` Posture

- `serde` remains the default and sufficient choice for most internal types.
- `schemars` is worth evaluating selectively for a small number of real outward read models.
- Avoid adding schema derivations to lifecycle, state internals, host types, or generic helper structs.

## Validation Focus

- Fewer structs survive in the Scoop outward model.
- Low-value count/derived fields are removed where they do not justify themselves.
- The app/backend boundary still reads clearly after DTO deletion.
- Representative backend/app regressions remain green.

## Sources

- `.planning/phases/12-scoop-read-model-and-shared-cleanup-refinement/12-CONTEXT.md`
- `.planning/phases/11-scoop-runtime-host-and-helper-consolidation/11-VERIFICATION.md`
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md`
- `spoon-backend/src/scoop/query.rs`
- `spoon-backend/src/scoop/info.rs`
- `spoon-backend/src/scoop/projection.rs`
- `spoon-backend/src/status.rs`
