# Plan 10-01 Summary

**Completed:** 2026-04-01
**Plan:** `10-01`
**Commit:** `2a851c3`

## Outcome

Active Scoop path usage now converges on `RuntimeLayout` / `ScoopLayout` instead of the old free-function helper layer.

## What Changed

- Added layout-owned Scoop package path methods in [`layout.rs`](/d:/projects/spoon/spoon-backend/src/layout.rs).
- Deleted [`paths.rs`](/d:/projects/spoon/spoon-backend/src/scoop/paths.rs) from the Scoop domain.
- Removed public re-exports of legacy Scoop path helpers from [`mod.rs`](/d:/projects/spoon/spoon-backend/src/scoop/mod.rs).
- Updated representative Scoop runtime/read-model/backend code to use layout-owned paths directly:
  - [`actions.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/actions.rs)
  - [`query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs)
  - [`integration.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/integration.rs)
  - [`surface.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/surface.rs)
  - [`download.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/download.rs)
  - [`cache.rs`](/d:/projects/spoon/spoon-backend/src/scoop/cache.rs)
  - [`extract.rs`](/d:/projects/spoon/spoon-backend/src/scoop/extract.rs)
  - [`buckets.rs`](/d:/projects/spoon/spoon-backend/src/scoop/buckets.rs)
  - [`info.rs`](/d:/projects/spoon/spoon-backend/src/scoop/info.rs)
  - [`execution.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/execution.rs)

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture`

## Notes

- This plan intentionally favored explicit layout-owned semantics over preserving helper compatibility.
- The cleanup also forced one app-side cache-path consumer to move onto `RuntimeLayout` in [`cache.rs`](/d:/projects/spoon/spoon/src/service/cache.rs).
