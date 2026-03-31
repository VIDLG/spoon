---
phase: 05-scoop-contract-alignment-and-context-completion
plan: 2
completed: 2026-03-31
requirements-completed: [TEST-02]
---

# Phase 05 Plan 2 Summary

The stale app-side Scoop regressions have been migrated to the current SQLite/canonical backend contract.

## Key Outcomes

- Updated [`scoop_flow.rs`](/d:/projects/spoon/spoon/tests/cli/scoop_flow.rs) so the stale CLI status/list/info/prefix setup now seeds installed state through backend canonical store APIs instead of writing `scoop/state/packages/*.json`.
- Kept the user-facing regression intent intact while aligning assertions to the current output contract where the old JSON-era expectations were no longer accurate.
- Cleaned the remaining nearby stale assumptions in the same file so the full `scoop_flow` suite now passes against the shipped SQLite/canonical design.
- Follow-on stale app regressions uncovered during re-audit were also aligned:
  - [`config_flow.rs`](/d:/projects/spoon/spoon/tests/cli/config_flow.rs) policy integration / command-profile cases now seed canonical store state and assert the current info rendering contract.
  - [`json_flow.rs`](/d:/projects/spoon/spoon/tests/cli/json_flow.rs) prefix JSON setup now uses canonical store state instead of flat JSON files.
  - [`tui_table_render_flow.rs`](/d:/projects/spoon/spoon/tests/tui/tui_table_render_flow.rs) no longer depends on stale Scoop state seeding or unstable background refresh assumptions for its latest-version rendering contract.

## Verification

- `cargo test -p spoon --test scoop_flow scoop_status_lists_buckets_and_installed_packages -- --nocapture`
- `cargo test -p spoon --test scoop_flow scoop_list_lists_installed_packages -- --nocapture`
- `cargo test -p spoon --test scoop_flow -- --nocapture`
- `cargo test -p spoon --test config_flow scoop_info_shows_applied_policy_integrations -- --nocapture`
- `cargo test -p spoon --test json_flow scoop_prefix_json_prints_structured_prefix_view -- --nocapture`
- `cargo test -p spoon --test tui_table_render_flow tools_table_hides_latest_when_same_as_current -- --nocapture`
