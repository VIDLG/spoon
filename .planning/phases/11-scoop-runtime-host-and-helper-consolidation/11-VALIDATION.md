---
phase: 11
slug: scoop-runtime-host-and-helper-consolidation
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 11 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo check -p spoon-backend -p spoon` |
| **Full suite command** | `cargo check -p spoon-backend -p spoon && cargo test -p spoon-backend scoop_action_contract_uses_context -- --nocapture && cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture && cargo test -p spoon --test scoop_flow -- --nocapture && cargo test -p spoon --test status_backend_flow -- --nocapture` |
| **Estimated runtime** | ~20 seconds targeted, ~150 seconds full phase suite |

## Per-Task Verification Map

| Task ID | Plan | Requirement | Automated Command |
|---------|------|-------------|-------------------|
| 11-01-01 | 01 | SLEG-02 | `cargo check -p spoon-backend -p spoon` |
| 11-01-02 | 01 | SLEG-02 | `cargo test -p spoon-backend scoop_action_contract_uses_context -- --nocapture` |
| 11-02-01 | 02 | SLEG-02 | `cargo check -p spoon-backend` |
| 11-02-02 | 02 | SLEG-02 | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` |
| 11-03-01 | 03 | SLEG-02, BECT-05 | `cargo check -p spoon-backend -p spoon` |
| 11-03-02 | 03 | SLEG-02, BECT-05 | `cargo test -p spoon --test status_backend_flow -- --nocapture` |
| 11-04-01 | 04 | SLEG-02, BECT-05 | `cargo test -p spoon --test scoop_flow -- --nocapture` |
| 11-04-02 | 04 | SLEG-02, BECT-05 | `cargo check -p spoon-backend -p spoon` |

## Manual-Only Verifications

| Behavior | Why Manual |
|----------|------------|
| New Scoop names and directories read more clearly to a human reviewer | Requires architectural judgment |
| `host` is actually thinner rather than just renamed | Requires structural review |

## Validation Sign-Off

- [x] Every task has a targeted automated verify
- [x] Sampling continuity is preserved
- [x] Missing artifacts are owned by early tasks
- [x] `nyquist_compliant: true` set
