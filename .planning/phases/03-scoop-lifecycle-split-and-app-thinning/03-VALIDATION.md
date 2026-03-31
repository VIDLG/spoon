---
phase: 03
slug: scoop-lifecycle-split-and-app-thinning
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-29
---

# Phase 03 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend install_lifecycle_emits_stage_contract -- --nocapture` |
| **Full suite command** | `cargo test -p spoon-backend --lib scoop && cargo test -p spoon --test scoop_runtime_flow -- --nocapture && cargo test -p spoon --test status_backend_flow json_status_uses_backend_read_models -- --nocapture` |
| **Estimated runtime** | ~25 seconds targeted, ~90 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 03-01-01 | 01 | SCLF-01, SCLF-04 | `cargo test -p spoon-backend install_lifecycle_emits_stage_contract -- --nocapture` |
| 03-01-02 | 01 | SCLF-04 | `cargo test -p spoon-backend uninstall_lifecycle_emits_stage_contract -- --nocapture` |
| 03-02-01 | 02 | SCLF-01, SCLF-02, SCLF-05 | `cargo test -p spoon-backend install_update_share_front_half_lifecycle_modules -- --nocapture` |
| 03-02-02 | 02 | SCLF-02 | `cargo test -p spoon --test scoop_runtime_flow spoon_scoop_package_cli_handles_install_update_uninstall_with_local_bucket_source -- --nocapture` |
| 03-03-01 | 03 | SCLF-01, SCLF-04, SCLF-05 | `cargo test -p spoon-backend install_lifecycle_orders_persist_surface_integrate_state -- --nocapture` |
| 03-03-02 | 03 | SCLF-04 | `cargo test -p spoon-backend reapply_runs_without_hooks_and_reuses_back_half_modules -- --nocapture` |
| 03-04-01 | 04 | SCLF-03, SCLF-04, SCLF-05 | `cargo test -p spoon-backend uninstall_and_reapply_use_shared_lifecycle_contract -- --nocapture` |
| 03-04-02 | 04 | SCLF-03 | `cargo test -p spoon-backend post_uninstall_hook_is_warning_only -- --nocapture` |
| 03-05-01 | 05 | SCLF-01, SCLF-02, SCLF-03, SCLF-04 | `cargo test -p spoon --test scoop_runtime_flow -- --nocapture` |
| 03-05-02 | 05 | SCLF-04 | `cargo test -p spoon --test status_backend_flow json_status_uses_backend_read_models -- --nocapture` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| CLI/TUI progress copy reads naturally from backend stage events without app-side lifecycle invention | Requires human review of UX wording |
| Failure stop points and journal stage markers make sense to an operator reading doctor/repair output | Requires human review of operator-facing semantics |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
