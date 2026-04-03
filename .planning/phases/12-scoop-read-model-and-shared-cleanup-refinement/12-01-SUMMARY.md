# Plan 12-01 Summary

**Completed:** 2026-04-03
**Plan:** `12-01`
**Commit:** `39603da`

## Outcome

The most obvious pass-through read-model wrappers were removed from the Scoop query/status surface.

## What Changed

- Deleted the duplicated query-side bucket/package entry wrappers from [`query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs).
- Reused [`Bucket`](/d:/projects/spoon/spoon-backend/src/scoop/buckets.rs) directly for status/query bucket output.
- Replaced the old installed-package entry wrapper with a clearer state-owned summary type:
  - [`InstalledPackageSummary`](/d:/projects/spoon/spoon-backend/src/scoop/state/projections.rs)
- Simplified [`status.rs`](/d:/projects/spoon/spoon-backend/src/status.rs) to reuse Scoop-domain structures instead of re-wrapping them yet again.
- Updated app consumers accordingly:
  - [`mod.rs`](/d:/projects/spoon/spoon/src/service/scoop/mod.rs)
  - [`report.rs`](/d:/projects/spoon/spoon/src/service/scoop/report.rs)
  - [`run.rs`](/d:/projects/spoon/spoon/src/cli/run.rs)

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon --test status_backend_flow -- --nocapture`

## Notes

- This plan intentionally targeted the clearest pass-through wrappers first rather than trying to delete every outward struct at once.
