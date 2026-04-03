# Plan 11-01 Summary

**Completed:** 2026-04-03
**Plan:** `11-01`
**Commit:** `ca05b27`

## Outcome

The old Scoop `runtime` layer was renamed and reshaped into a clearer `host` edge layer, while root-worthy domain modules were promoted to the Scoop root.

## What Changed

- Renamed the old Scoop `runtime/` topology into [`host/`](/d:/projects/spoon/spoon-backend/src/scoop/host).
- Promoted the real operation entry to [`actions.rs`](/d:/projects/spoon/spoon-backend/src/scoop/actions.rs).
- Promoted the package source model to [`package_source.rs`](/d:/projects/spoon/spoon-backend/src/scoop/package_source.rs).
- Updated root exports in [`mod.rs`](/d:/projects/spoon/spoon-backend/src/scoop/mod.rs) and app-side runtime glue in [`runtime.rs`](/d:/projects/spoon/spoon/src/service/scoop/runtime.rs).
- Updated representative backend tests to the new topology:
  - [`runtime.rs`](/d:/projects/spoon/spoon-backend/src/scoop/tests/runtime.rs)
  - [`state.rs`](/d:/projects/spoon/spoon-backend/src/scoop/tests/state.rs)

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon-backend scoop_action_contract_uses_context -- --nocapture`

## Notes

- This was a real topology change, not just a cosmetic rename.
- The old `runtime` directory no longer serves as the main Scoop edge-layer name.
