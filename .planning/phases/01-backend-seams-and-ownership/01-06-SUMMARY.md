---
phase: "01"
plan: "06"
subsystem: spoon-app-shell
tags: [refactor, backend-seams, layout-ownership, detail-surface, config-surface]
dependency_graph:
  requires: [01-04, 01-05]
  provides: [01-07]
  affects: [spoon/src/view, spoon/src/cli, spoon/src/service]
tech_stack:
  added: []
  patterns: [RuntimeLayout, backend-query-models]
key_files:
  created: []
  modified:
    - spoon/src/service/scoop/report.rs
    - spoon/src/service/scoop/mod.rs
    - spoon/src/cli/run.rs
    - spoon/src/view/config.rs
    - spoon/tests/cli/json_flow.rs
    - spoon/tests/tui/tui_tool_detail_flow.rs
decisions: []
metrics:
  duration_minutes: 12
  completed_date: "2026-03-28"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 6
---

# Phase 01 Plan 06: Detail, Prefix, and Config Surface Conversion Summary

Converted the remaining detail, prefix, and config surfaces in the app shell from app-owned backend path helpers to backend layout contracts, completing the Phase 1 surface conversion for BNDR-05 and LAY-02.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Convert prefix, detail, and report surfaces to backend query and outcome models | `0c7a93a` | report.rs, mod.rs, run.rs, json_flow.rs, tui_tool_detail_flow.rs |
| 2 | Convert config and runtime-path surfaces to backend layout semantics only | `f4d84e4` | config.rs |

## Deviations from Plan

None - plan executed exactly as written.

## Pre-existing Issues Deferred

The following tests were already failing before this plan's changes (confirmed by reverting and re-running):

- `scoop_prefix_json_prints_structured_prefix_view` - tokio runtime nesting issue
- `install_json_prints_structured_package_action_results` - tokio runtime nesting issue
- `scoop_bucket_remove_json_prints_structured_action_result` - tokio runtime nesting issue
- `status_refresh_json_embeds_structured_bucket_update_result` - tokio runtime nesting issue
- `external_tool_detail_rejects_install_action` - tokio runtime context issue
- `tool_detail_prioritizes_summary_ops_versions_and_config` - tokio runtime context issue

## Key Decisions

- `RuntimeLayout::from_root` replaces `package_current_root` for prefix path derivation in report.rs and run.rs
- `runtime_status` backend query replaces `installed_package_states_filtered` for installed package lookup in prefix reports
- `RuntimeLayout` struct fields used directly in `ConfigModel::from_global` for all derived runtime paths (scoop root, MSVC roots, toolchain root)

## Self-Check: PASSED

- `0c7a93a`: FOUND (Task 1 commit)
- `f4d84e4`: FOUND (Task 2 commit)
- `spoon/src/service/scoop/report.rs`: no `package_current_root`, no `installed_package_states_filtered`
- `spoon/src/view/config.rs`: contains `RuntimeLayout`, no `config::scoop_root_from`, no `config::msvc_root_from`, no `config::official_msvc_root_from`
- `spoon/tests/tui/tui_tool_detail_flow.rs`: contains `tool_detail_uses_backend_models`
- `spoon/tests/cli/json_flow.rs`: contains `package_report_uses_backend_models`
