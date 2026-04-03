---
phase: 12
slug: scoop-read-model-and-shared-cleanup-refinement
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-03
---

# Phase 12 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo check -p spoon-backend -p spoon` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon --test status_backend_flow -- --nocapture && cargo test -p spoon --test scoop_flow -- --nocapture && cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~150 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 12-01-01 | 01 | SLEG-03 | `cargo check -p spoon-backend -p spoon` |
| 12-01-02 | 01 | SLEG-03 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |
| 12-02-01 | 02 | SLEG-03 | `cargo check -p spoon-backend -p spoon` |
| 12-02-02 | 02 | SLEG-03 | `cargo test -p spoon --test scoop_flow -- --nocapture` |
| 12-03-01 | 03 | BECT-06 | `cargo check -p spoon-backend -p spoon` |
| 12-03-02 | 03 | BECT-06 | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| 12-04-01 | 04 | SLEG-03, BECT-06 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| Surviving read models are clearly justified boundary contracts | Requires architectural judgment |
| `projection.rs` has been demoted rather than simply renamed | Requires structural review |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
