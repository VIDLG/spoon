---
phase: 06
slug: msvc-seams-and-ownership-completion
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 06 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture && cargo test -p spoon --test msvc_flow -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~90 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 06-01-01 | 01 | MSVC-01 | `cargo check -p spoon-backend` |
| 06-01-02 | 01 | MSVC-01, MSVC-04 | `cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture` |
| 06-02-01 | 02 | MSVC-01, MSVC-04 | `cargo test -p spoon --test msvc_flow -- --nocapture` |
| 06-02-02 | 02 | MSVC-04 | `cargo check -p spoon-backend -p spoon` |
| 06-03-01 | 03 | MSVC-01 | `cargo test -p spoon-backend --lib msvc::tests::root -- --nocapture` |
| 06-03-02 | 03 | MSVC-01, MSVC-04 | `cargo test -p spoon --test msvc_flow msvc_status_lists_managed_and_official_runtime_state -- --nocapture` |
| 06-04-01 | 04 | MSVC-01, MSVC-04 | `cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| 06-04-02 | 04 | MSVC-01, MSVC-04 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| The new MSVC module split reads as one domain with two runtime strategies rather than two separate products | Requires architecture-level judgment |
| Phase 6 stays seam-first and does not accidentally absorb Phase 7's deeper state/lifecycle work | Requires scope judgment across multiple modules |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
