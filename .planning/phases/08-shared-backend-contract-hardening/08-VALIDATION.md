---
phase: 08
slug: shared-backend-contract-hardening
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 08 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo check -p spoon-backend` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend --lib -- --nocapture && cargo test -p spoon --test msvc_flow -- --nocapture && cargo test -p spoon --test scoop_flow -- --nocapture && cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~150 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 08-01-01 | 01 | BECT-01 | `cargo check -p spoon-backend -p spoon` |
| 08-01-02 | 01 | BECT-01 | `cargo test -p spoon --test msvc_flow -- --nocapture` |
| 08-02-01 | 02 | BECT-02 | `cargo check -p spoon-backend` |
| 08-02-02 | 02 | BECT-02 | `cargo test -p spoon-backend --lib -- --nocapture` |
| 08-03-01 | 03 | BECT-03 | `cargo check -p spoon-backend` |
| 08-03-02 | 03 | BECT-03 | `cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| 08-04-01 | 04 | BECT-03, BECT-04 | `cargo check -p spoon-backend -p spoon` |
| 08-04-02 | 04 | BECT-04 | `cargo test -p spoon --test scoop_flow -- --nocapture` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| Event and error contracts are meaningfully clearer without becoming over-designed frameworks | Requires architecture judgment |
| Layout/port cleanup removes stale JSON-era concepts without obscuring runtime behavior | Requires design judgment beyond pure test signals |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
