# Plan 12-02 Summary

**Completed:** 2026-04-03
**Plan:** `12-02`
**Commit:** `39603da`

## Outcome

Low-value derived fields such as read-model counts stopped surviving by default.

## What Changed

- Removed count fields such as:
  - `bucket_count`
  - `installed_package_count`
  - `match_count`
  from the relevant Scoop read models in [`query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs).
- Updated app reporting to derive lengths directly from collections:
  - [`report.rs`](/d:/projects/spoon/spoon/src/service/scoop/report.rs)
- Updated backend-facing snapshot consumers to follow the leaner contract:
  - [`status.rs`](/d:/projects/spoon/spoon-backend/src/status.rs)

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon --test scoop_flow -- --nocapture`

## Notes

- This phase applies derive-not-store to read models, not just persisted state.
- The intent is quieter, more intentional output contracts, not merely smaller structs.
