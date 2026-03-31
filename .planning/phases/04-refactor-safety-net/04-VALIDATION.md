---
phase: 04
slug: refactor-safety-net
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-30
---

# Phase 04 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend hook_failures_stop_before_state_commit -- --nocapture` |
| **Full suite command** | `cargo test -p spoon-backend hook_failures_stop_before_state_commit -- --nocapture && cargo test -p spoon-backend lock_conflict_and_journal_stop_points_are_diagnosable -- --nocapture && cargo test -p spoon --test status_backend_flow -- --nocapture && cargo test -p spoon --test scoop_runtime_flow -- --nocapture` |
| **Estimated runtime** | ~30 seconds targeted, ~120 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 04-01-01 | 01 | TEST-01, TEST-03 | `cargo test -p spoon-backend hook_failures_stop_before_state_commit -- --nocapture` |
| 04-01-02 | 01 | TEST-01, TEST-03 | `cargo test -p spoon-backend warning_only_uninstall_tail_preserves_main_result -- --nocapture` |
| 04-02-01 | 02 | TEST-01, TEST-03 | `cargo test -p spoon-backend lock_conflict_and_journal_stop_points_are_diagnosable -- --nocapture` |
| 04-02-02 | 02 | TEST-01 | `cargo test -p spoon-backend doctor_reports_failed_lifecycle_residue -- --nocapture` |
| 04-03-01 | 03 | TEST-02 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |
| 04-03-02 | 03 | TEST-02 | `cargo test -p spoon --test scoop_runtime_flow -- --nocapture` |
| 04-04-01 | 04 | TEST-01, TEST-02 | `cargo test -p spoon --test scoop_runtime_flow -- --ignored --nocapture` |
| 04-04-02 | 04 | TEST-03 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| Real ignored Scoop smoke flows still feel intentionally sparse rather than becoming the main regression strategy | Requires human judgment about suite balance |
| Failure/journal/doctor output remains operator-readable and consistent with lifecycle stage names | Requires human review of diagnostics semantics |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set

