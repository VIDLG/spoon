---
created: 2026-03-31T00:00:00Z
title: Tighten backend error contract
area: general
files:
  - spoon-backend/src/error.rs
  - spoon-backend/src/scoop/runtime/actions.rs
  - spoon-backend/src/scoop/runtime/download.rs
  - spoon-backend/src/scoop/runtime/surface.rs
  - spoon-backend/src/msvc/mod.rs
---

## Problem

`spoon-backend` already has a usable `BackendError` enum, but the error contract is drifting toward broad fallback variants like `Other(String)` and `External { .. }`. That makes backend failure semantics less typed than they could be, and it weakens the separation between infrastructure failures, domain/lifecycle failures, and user-facing outcome text.

This is not severe enough to interrupt the completed backend refactor milestone, but it is worth capturing before more lifecycle, doctor, or event work adds additional string-based error paths.

## Solution

Audit `BackendError` usage and reduce cases where meaningful backend/domain failures are flattened into `Other(String)`. Prefer tighter typed variants for recurring failure shapes, and clarify the boundary between:

- infrastructure/transport failures
- domain/lifecycle invariant failures
- user-facing summaries carried through events/outcomes

Keep the scope narrow: this should be a contract-hardening cleanup, not a full redesign of every error path in one pass.
