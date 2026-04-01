---
created: 2026-04-01T00:00:00Z
title: Audit derive-not-store fields in backend state and read models
area: general
files:
  - spoon-backend/src/status.rs
  - spoon-backend/src/scoop/query.rs
  - spoon-backend/src/scoop/state/model.rs
  - spoon-backend/src/control_plane/schema/0001_control_plane.sql
  - spoon-backend/src/msvc/plan.rs
  - spoon-backend/src/msvc/status.rs
  - spoon-backend/src/msvc/official.rs
---

## Problem

Some backend structs still carry values that can be derived from neighboring fields or from layout/detection evidence. The risk is not equal everywhere:

- In persisted or canonical state, derivable fields create drift and double-write pressure.
- In read models and status snapshots, derivable fields are lower-risk but can still add noise and maintenance cost if they do not provide meaningful value.

Examples surfaced during review:

- `BackendScoopSummary.bucket_count` and `BackendScoopSummary.installed_package_count` are derivable from the adjacent vectors in `status.rs`.
- `ScoopRuntimeStatus.bucket_count`, `installed_package_count`, and `ScoopSearchResults.match_count` are read-model counts that could be recomputed.
- `InstalledPackageState.cache_size_bytes` and the matching `installed_packages.cache_size_bytes` SQLite column are more suspicious because they live in canonical persisted state despite being plausibly recomputable.
- The MSVC work entering Phase 7 must avoid repeating this pattern when defining canonical state and SQLite schema.

## Solution

Do a focused derive-not-store audit across backend state and projection types.

Separate findings into two classes:

1. **Dangerous persisted redundancies**
   Fields in canonical state / SQLite schema / durable runtime state that can be derived and therefore should usually be removed or justified explicitly.

2. **Low-risk projection redundancies**
   Fields in status/read-model/output structs that are redundant but may be retained if they materially simplify app/rendering contracts.

Use the results in two places:

- **Phase 7:** shape MSVC canonical state and SQLite schema so derive-not-store is enforced from the start.
- **Phase 8:** clean up remaining low-value projection redundancies and shared contract drift.
