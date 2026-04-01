---
id: SEED-002
title: Evaluate async_zip for shared backend ZIP extraction
status: planted
planted: 2026-04-01
planted_during: v0.6.0
owner: codex
trigger_when:
  - Shared archive extraction is revisited beyond the current zip helper extraction.
  - The project wants to reduce `spawn_blocking` / sync ZIP extraction reliance in backend archive primitives.
  - Archive primitive consolidation becomes a higher-priority follow-up after Phase 8.
scope: medium
refreshed: 2026-04-01
---

# Backend ZIP Async Evaluation

## Why This Matters

The backend has already started extracting shared archive primitives, but ZIP extraction still relies on the synchronous `zip` crate pattern. `async_zip` looks like the most plausible forward-looking Rust candidate if the project later wants ZIP extraction to align more closely with the repo's async backend edges without depending on an external helper like the current 7z path.

This is not a current blocker, but it is worth capturing before archive work continues and the rationale is forgotten.

## When to Surface

Surface this seed when:

- shared archive primitives are revisited
- ZIP extraction becomes a meaningful async bottleneck or design inconsistency
- the team wants to evaluate whether ZIP extraction should move beyond synchronous helper style while keeping 7z on a separate path

## Scope Estimate

- evaluate `async_zip` maturity, ecosystem fit, and Windows behavior
- compare it to the current synchronous `zip` + blocking approach
- decide whether the switch is worth it for backend archive primitives

## Breadcrumbs

- `spoon-backend/src/archive.rs`
- `spoon-backend/src/scoop/extract.rs`
- `spoon-backend/src/msvc/mod.rs`
- `.planning/phases/08-shared-backend-contract-hardening/08-CONTEXT.md`
- `.planning/phases/08-shared-backend-contract-hardening/08-03-PLAN.md`

## Notes

- This seed is specifically about ZIP extraction. It does not imply the project has solved 7z extraction or found a unified async archive solution.
- If promoted later, keep the evaluation narrow and practical: async ZIP only, not a giant archive-platform rewrite.
