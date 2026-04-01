---
phase: 09
slug: msvc-and-shared-safety-net
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 09 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo check -p spoon-backend -p spoon` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture && cargo test -p spoon-backend --lib msvc::tests::root -- --nocapture && cargo test -p spoon-backend --lib msvc::tests::official -- --nocapture && cargo test -p spoon-backend --lib event -- --nocapture && cargo test -p spoon --test msvc_flow -- --nocapture && cargo test -p spoon --test status_backend_flow -- --nocapture && cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| **Estimated runtime** | ~30 seconds targeted, ~180 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 09-01-01 | 01 | TEST-04 | `cargo test -p spoon-backend --lib msvc::tests::root -- --nocapture` |
| 09-01-02 | 01 | TEST-04 | `cargo test -p spoon-backend --lib msvc::tests::official -- --nocapture` |
| 09-02-01 | 02 | TEST-04, TEST-05 | `cargo test -p spoon-backend --lib event -- --nocapture` |
| 09-02-02 | 02 | TEST-04, TEST-05 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |
| 09-03-01 | 03 | TEST-05 | `cargo test -p spoon --test msvc_flow -- --nocapture` |
| 09-03-02 | 03 | TEST-05 | `cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| 09-04-01 | 04 | TEST-06 | `cargo test -p spoon --test msvc_flow -- --ignored --nocapture` |
| 09-04-02 | 04 | TEST-04, TEST-05, TEST-06 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| Safety-net focus stayed on high-value breakpoints instead of drifting into coverage theater | Requires judgment |
| Real smoke remains sparse and intentional | Requires milestone-level review |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
