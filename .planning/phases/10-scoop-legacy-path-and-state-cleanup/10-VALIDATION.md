---
phase: 10
slug: scoop-legacy-path-and-state-cleanup
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 10 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo check -p spoon-backend -p spoon` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend --lib scoop::tests::state -- --nocapture && cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture && cargo test -p spoon --test scoop_flow -- --nocapture && cargo test -p spoon --test status_backend_flow -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~120 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 10-01-01 | 01 | SLEG-01, SLEG-04 | `cargo check -p spoon-backend -p spoon` |
| 10-01-02 | 01 | SLEG-01 | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| 10-02-01 | 02 | SLEG-01, SLEG-04 | `cargo test -p spoon-backend --lib scoop::tests::state -- --nocapture` |
| 10-02-02 | 02 | SLEG-01 | `cargo check -p spoon-backend` |
| 10-03-01 | 03 | SLEG-04 | `cargo check -p spoon-backend -p spoon` |
| 10-03-02 | 03 | SLEG-04 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |
| 10-04-01 | 04 | SLEG-01, SLEG-04 | `cargo test -p spoon --test scoop_flow -- --nocapture` |
| 10-04-02 | 04 | SLEG-01, SLEG-04 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| Scoop path model now reads as layout-owned rather than helper-driven | Requires architectural judgment |
| Legacy worldview is actually removed rather than just renamed | Requires code review judgment |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
