---
created: 2026-03-31T00:00:00Z
title: Consolidate remaining fsx helpers
area: general
files:
  - spoon-backend/src/fsx.rs
  - spoon-backend/src/scoop/extract.rs
  - spoon-backend/src/scoop/buckets.rs
  - spoon-backend/src/scoop/runtime/persist.rs
---

## Problem

There are still reusable filesystem operations in `spoon-backend` that have not been consolidated into `fsx`. That leaves filesystem behavior split across multiple backend modules, makes reuse less obvious, and increases the chance that future cleanup work duplicates path-copy/remove/replace logic instead of reusing one backend utility surface.

This came up after the main Scoop/backend refactor milestone was completed, so it is not urgent enough for a new phase right now, but it is worth capturing as a concrete cleanup task.

## Solution

Audit remaining backend filesystem helpers and identify operations that are genuinely reusable across domains or lifecycle slices. Move those operations into `spoon-backend/src/fsx.rs` when they represent shared backend filesystem behavior, and keep domain-specific logic in place when it is tightly coupled to Scoop/MSVC semantics.

Prefer a narrow consolidation pass rather than dumping every file-related helper into `fsx`.
