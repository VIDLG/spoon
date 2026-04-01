---
phase: 07
slug: canonical-msvc-state-and-lifecycle
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 07 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture && cargo test -p spoon-backend --lib msvc::tests::root -- --nocapture && cargo test -p spoon-backend --lib msvc::tests::official -- --nocapture && cargo test -p spoon --test msvc_flow -- --nocapture` |
| **Estimated runtime** | ~25 seconds targeted, ~120 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 07-01-01 | 01 | MSVC-02 | `cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture` |
| 07-01-02 | 01 | MSVC-02 | `cargo check -p spoon-backend` |
| 07-02-01 | 02 | MSVC-03 | `cargo test -p spoon-backend --lib msvc::tests::root -- --nocapture` |
| 07-02-02 | 02 | MSVC-03 | `cargo test -p spoon --test tui_msvc_download_flow -- --nocapture` |
| 07-03-01 | 03 | MSVC-03 | `cargo test -p spoon-backend --lib msvc::tests::official -- --nocapture` |
| 07-03-02 | 03 | MSVC-02, MSVC-03 | `cargo test -p spoon --test msvc_flow -- --nocapture` |
| 07-04-01 | 04 | MSVC-02, MSVC-03 | `cargo test -p spoon-backend --lib msvc::tests::context -- --nocapture` |
| 07-04-02 | 04 | MSVC-02, MSVC-03 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| The new MSVC canonical state stores backend-trusted facts without bloating into derivable/noisy data | Requires architectural judgment |
| `official` still behaves like a runtime strategy inside one MSVC domain rather than re-splitting the architecture | Requires domain-level judgment |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
