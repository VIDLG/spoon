---
phase: 03-scoop-lifecycle-split-and-app-thinning
plan: 4
completed: 2026-03-29
requirements-completed: [SCLF-03, SCLF-04]
---

# Phase 03 Plan 4 Summary

Uninstall and reapply are now explicit lifecycle entry points instead of leftovers inside the install controller.

## Key Outcomes

- Added explicit lifecycle entry modules:
  - [`uninstall.rs`](/d:/projects/spoon/spoon-backend/src/scoop/lifecycle/uninstall.rs)
  - [`reapply.rs`](/d:/projects/spoon/spoon-backend/src/scoop/lifecycle/reapply.rs)
- Promoted `ScoopPackageAction::Reapply` into the planner and routed it through backend lifecycle execution.
- Preserved centralized hook execution and implemented the agreed warning-only `post_uninstall` behavior.
- Added and passed:
  - `uninstall_and_reapply_use_shared_lifecycle_contract`
  - `post_uninstall_hook_is_warning_only`

