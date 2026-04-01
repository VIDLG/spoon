# Plan 10-03 Summary

**Completed:** 2026-04-01
**Plan:** `10-03`
**Commit:** `9b9f948`

## Outcome

Adjacent app and test consumers now follow the layout-owned Scoop path model instead of reintroducing the deleted helper worldview.

## What Changed

- Replaced remaining `scoop_root_from(...).join(...)` style path derivations in representative app/test flows with `RuntimeLayout`.
- Updated focused CLI/TUI regression files so they no longer depend on the old Scoop root helper story:
  - [`cli_flow.rs`](/d:/projects/spoon/spoon/tests/cli/cli_flow.rs)
  - [`config_flow.rs`](/d:/projects/spoon/spoon/tests/cli/config_flow.rs)
  - [`json_flow.rs`](/d:/projects/spoon/spoon/tests/cli/json_flow.rs)
  - [`tui_form_flow.rs`](/d:/projects/spoon/spoon/tests/tui/tui_form_flow.rs)
  - [`tui_scoop_flow.rs`](/d:/projects/spoon/spoon/tests/tui/tui_scoop_flow.rs)
  - [`tui_table_render_flow.rs`](/d:/projects/spoon/spoon/tests/tui/tui_table_render_flow.rs)
  - [`tui_tool_detail_flow.rs`](/d:/projects/spoon/spoon/tests/tui/tui_tool_detail_flow.rs)
- Kept `shims_root_from` intact because it still expresses a valid non-Scoop-specific app helper story; this phase only removed the Scoop-specific legacy path worldview.

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon --test scoop_flow -- --nocapture`
- `cargo test -p spoon --test status_backend_flow -- --nocapture`

## Notes

- This plan intentionally focused on the direct spillover most coupled to the Scoop cleanup rather than broad repo-wide deprecated helper cleanup.
- Remaining MSVC-side deprecated helpers stay out of scope for this phase.
