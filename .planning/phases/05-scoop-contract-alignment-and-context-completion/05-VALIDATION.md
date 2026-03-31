---
phase: 05
slug: scoop-contract-alignment-and-context-completion
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-31
---

# Phase 05 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| **Full suite command** | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture && cargo test -p spoon --test scoop_flow scoop_status_lists_buckets_and_installed_packages -- --nocapture && cargo test -p spoon --test scoop_flow scoop_list_lists_installed_packages -- --nocapture && cargo test -p spoon --test scoop_flow -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~90 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 05-01-01 | 01 | TEST-03 | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| 05-01-02 | 01 | TEST-03 | `cargo test -p spoon-backend --lib scoop::tests::runtime -- --nocapture` |
| 05-02-01 | 02 | TEST-02 | `cargo test -p spoon --test scoop_flow scoop_status_lists_buckets_and_installed_packages -- --nocapture` |
| 05-02-02 | 02 | TEST-02 | `cargo test -p spoon --test scoop_flow scoop_list_lists_installed_packages -- --nocapture` |
| 05-02-03 | 02 | TEST-02 | `cargo test -p spoon --test scoop_flow -- --nocapture` |
| 05-03-01 | 03 | LAY-03 | `cargo check -p spoon-backend -p spoon` |
| 05-03-02 | 03 | LAY-03, TEST-02, TEST-03 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| The remaining Scoop app/backend seam is now clearly understandable rather than only technically passing audit | Requires architecture-level judgment |
| Re-run milestone audit and decide whether archive is now safe | Requires milestone-level review |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set

