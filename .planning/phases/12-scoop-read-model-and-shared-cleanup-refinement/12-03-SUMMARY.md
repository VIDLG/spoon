# Plan 12-03 Summary

**Completed:** 2026-04-03
**Plan:** `12-03`
**Commit:** `c5d0768`

## Outcome

`projection.rs` is now more clearly an internal helper file, and a narrow `schemars` trial has been applied to the surviving outward-facing read models.

## What Changed

- Removed a set of now-unused projection helpers from [`projection.rs`](/d:/projects/spoon/spoon-backend/src/scoop/projection.rs):
  - `url_lines`
  - `bin_lines`
  - `shortcut_lines`
  - `notes_lines`
  - `value_field`
  - `license_field`
- Added a narrow `schemars` dependency in [`Cargo.toml`](/d:/projects/spoon/spoon-backend/Cargo.toml).
- Applied `JsonSchema` only to the surviving, clearly outward-facing read-model structures:
  - [`Bucket`](/d:/projects/spoon/spoon-backend/src/scoop/buckets.rs)
  - [`InstalledPackageSummary`](/d:/projects/spoon/spoon-backend/src/scoop/state/model.rs)
  - query/status structs in [`query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs)
  - snapshot structs in [`status.rs`](/d:/projects/spoon/spoon-backend/src/status.rs)
- Continued the cleanup by deleting the thin state summary projection layer entirely:
  - deleted [`projections.rs`](/d:/projects/spoon/spoon-backend/src/scoop/state/projections.rs)
  - moved [`InstalledPackageSummary`](/d:/projects/spoon/spoon-backend/src/scoop/state/model.rs) into the canonical state model file
  - inlined summary/filter/sort logic into the real use sites in [`query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs)
- Removed two additional thin state forwarding wrappers from [`mod.rs`](/d:/projects/spoon/spoon-backend/src/scoop/state/mod.rs).
- Standardized stored-row decoding toward `TryFrom` in:
  - [`store.rs`](/d:/projects/spoon/spoon-backend/src/scoop/state/store.rs)
  - [`state.rs`](/d:/projects/spoon/spoon-backend/src/msvc/state.rs)

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture`

## Notes

- `schemars` was adopted narrowly, as discussed: only for the outward read models that survived the cleanup.
- It was not applied to lifecycle internals, host types, or generic/internal helper models.
- The later thin-projection cleanup stayed inside Phase 12 scope because it was the same class of read-model/adapter simplification rather than a new phase.
