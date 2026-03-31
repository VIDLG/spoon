---
phase: 02
slug: canonical-scoop-state
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-28
---

# Phase 02 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend canonical_installed_state_roundtrips_bucket_and_architecture -- --nocapture` |
| **Full suite command** | `cargo test -p spoon-backend --lib scoop && cargo test -p spoon --test json_flow scoop_info_json_prints_structured_package_view -- --nocapture && cargo test -p spoon --test json_flow scoop_status_json_prints_structured_runtime_view -- --nocapture && cargo test -p spoon --test status_backend_flow json_status_uses_backend_read_models -- --nocapture` |
| **Estimated runtime** | ~20 seconds for targeted verifies, ~75 seconds for full phase suite |

---

## Sampling Rate

- **After every task commit:** Run that task's `<automated>` command from its PLAN.
- **After every wave:** Run `cargo test -p spoon-backend --lib scoop`, then the phase-relevant app-shell regressions.
- **Before phase verification:** Full suite above must be green.
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | SCST-01, SCST-04 | backend unit | `cargo test -p spoon-backend canonical_installed_state_roundtrips_bucket_and_architecture -- --nocapture` | Created in task | pending |
| 02-01-02 | 01 | 1 | SCST-01, SCST-04 | backend unit | `cargo test -p spoon-backend canonical_state_persists_only_nonderivable_facts -- --nocapture` | Created in task | pending |
| 02-02-01 | 02 | 2 | SCST-01, SCST-02 | backend integration | `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture` | Created in task | pending |
| 02-02-02 | 02 | 2 | SCST-02 | backend integration | `cargo test -p spoon-backend reapply_inputs_come_from_canonical_state -- --nocapture` | Created in task | pending |
| 02-03-01 | 03 | 3 | SCST-01, SCST-02 | backend unit | `cargo test -p spoon-backend runtime_status_uses_canonical_installed_state -- --nocapture` | Created in task | pending |
| 02-03-02 | 03 | 3 | SCST-02 | app integration | `cargo test -p spoon --test json_flow scoop_status_json_prints_structured_runtime_view -- --nocapture` | Existing suite | pending |
| 02-04-01 | 04 | 4 | SCST-02, SCST-04 | backend integration | `cargo test -p spoon-backend scoop_package_info_reads_canonical_state -- --nocapture` | Created in task | pending |
| 02-04-02 | 04 | 4 | SCST-02 | app integration | `cargo test -p spoon --test json_flow scoop_info_json_prints_structured_package_view -- --nocapture` | Existing suite | pending |
| 02-05-01 | 05 | 5 | SCST-03 | build/audit | `rg -n "ScoopPackageState|read_package_state|write_package_state|remove_package_state" spoon-backend/src/scoop` | Existing command | pending |
| 02-05-02 | 05 | 5 | SCST-03, SCST-04 | backend unit | `cargo test -p spoon-backend legacy_flat_scoop_state_is_reported -- --nocapture` | Created in task | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Resolution

No separate bootstrap wave is required. Each missing test artifact is created by the first plan that needs it.

| Artifact | Owner | Resolution |
|----------|-------|------------|
| `spoon-backend/src/scoop/tests/state.rs` | 02-01 Task 2 | Introduced with canonical state module contract tests. |
| `spoon-backend/src/scoop/state/projections.rs` regressions | 02-03 / 02-04 | Added when query/info surfaces move to canonical projections. |
| legacy-state detection test | 02-05 Task 2 | Added alongside stale-state detection or doctor reporting. |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Existing real Scoop installs survive the Phase 2 refactor cleanly when rebuilt into the canonical state path | SCST-01, SCST-02 | Requires a real local tool root and installed package state | Run Spoon against a temp real root with at least one installed package, then verify `status`, `info`, and uninstall/reapply surfaces remain coherent |
| Operator-facing repair messaging is understandable when old flat state files remain | SCST-03 | Requires checking exact CLI/TUI messaging quality | Seed a legacy `scoop/state/<pkg>.json` file, run the relevant command, and confirm the surfaced repair guidance is actionable |

---

## Validation Sign-Off

- [x] All tasks have targeted automated verifies
- [x] Sampling continuity stays under 30 seconds
- [x] Missing test artifacts are owned by early plans
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-28
