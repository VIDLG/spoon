---
phase: 01
slug: backend-seams-and-ownership
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-28
---

# Phase 01 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via `cargo test` |
| **Config file** | none - standard Cargo test discovery |
| **Quick run command** | `cargo test -p spoon-backend runtime_layout_derives_from_root -- --nocapture` |
| **Full suite command** | `cargo test -p spoon-backend --lib && cargo test -p spoon --test json_flow && cargo test -p spoon --test status_backend_flow && cargo test -p spoon --test tui_tool_detail_flow && cargo test -p spoon --test msvc_flow` |
| **Estimated runtime** | ~25 seconds for targeted task verifies, ~90 seconds for full wave suite |

---

## Sampling Rate

- **After every task commit:** Run that task's `<automated>` command from its PLAN. These targeted commands are the Nyquist samples and must stay under 30 seconds.
- **After every plan wave:** Run `cargo test -p spoon-backend --lib`, then add the relevant app-shell suites for that wave: `cargo test -p spoon --test json_flow`, `cargo test -p spoon --test status_backend_flow`, `cargo test -p spoon --test tui_tool_detail_flow`, and `cargo test -p spoon --test msvc_flow` when their owning plans have landed.
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | BNDR-04, LAY-01, LAY-03 | backend unit | `cargo test -p spoon-backend runtime_layout_derives_from_root -- --nocapture` | Created in task | pending |
| 01-01-02 | 01 | 1 | BNDR-04, LAY-01, LAY-03 | backend unit | `cargo test -p spoon-backend explicit_context_required_for_runtime_ops -- --nocapture` | Created in task | pending |
| 01-02-01 | 02 | 2 | BNDR-01, BNDR-02, GIT-02, GIT-03 | backend integration | `cargo test -p spoon-backend scoop_action_contract_uses_context -- --nocapture` | Created in task 1 | pending |
| 01-02-02 | 02 | 2 | BNDR-02, GIT-02, GIT-03 | backend integration | `cargo test -p spoon-backend bucket_sync_uses_backend_git_contract -- --nocapture` | Created in task | pending |
| 01-03-01 | 03 | 2 | BNDR-03, LAY-03 | backend unit/integration | `cargo test -p spoon-backend msvc_context_drives_status_and_install -- --nocapture` | Created in task | pending |
| 01-03-02 | 03 | 2 | BNDR-03, LAY-03 | app integration | `cargo test -p spoon --test msvc_flow -- --nocapture` | Existing suite | pending |
| 01-04-01 | 04 | 3 | BNDR-05, LAY-02 | backend unit | `cargo test -p spoon-backend --lib status -- --nocapture` | Existing suite | pending |
| 01-04-02 | 04 | 3 | BNDR-05, LAY-02 | app integration | `cargo test -p spoon --test status_backend_flow json_status_uses_backend_read_models -- --nocapture` | Created in task | pending |
| 01-05-01 | 05 | 3 | BNDR-01, GIT-02 | app integration | `cargo test -p spoon --test json_flow install_json_prints_structured_package_action_results -- --nocapture` | Existing suite | pending |
| 01-05-02 | 05 | 3 | BNDR-02, GIT-03 | app integration | `cargo test -p spoon --test json_flow bucket_json_uses_backend_repo_sync_outcome -- --nocapture` | Existing suite | pending |
| 01-06-01 | 06 | 4 | BNDR-05 | app integration | `cargo test -p spoon --test tui_tool_detail_flow tool_detail_uses_backend_models -- --nocapture` | Existing suite | pending |
| 01-06-02 | 06 | 4 | LAY-02 | app integration | `cargo test -p spoon --test json_flow config_json_prints_structured_view_model -- --nocapture` | Existing suite | pending |
| 01-07-01 | 07 | 5 | LAY-02 | app integration | `cargo test -p spoon --test json_flow config_json_prints_structured_view_model -- --nocapture` | Existing suite | pending |
| 01-07-02 | 07 | 5 | GIT-01 | build/audit | `powershell -NoProfile -Command "if ((cargo tree -p spoon -e normal --depth 1 | Select-String ' gix v').Count -eq 0) { Write-Output 'PASS' } else { Write-Error 'spoon still directly depends on gix'; exit 1 }"` | No test file needed | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Resolution

No separate pre-execution Wave 0 is required. Every missing verification artifact is created by the earliest plan that needs it, so execution never depends on an unowned test file.

| Artifact | Owner | Resolution |
|----------|-------|------------|
| `spoon-backend/src/tests/context.rs` | 01-01 Task 1 | Created alongside `BackendContext` and `RuntimeLayout`, before later backend context verifies run. |
| `spoon-backend/src/scoop/tests/contracts.rs` | 01-02 Task 1 | Created in the backend Scoop runtime contract task, then extended by the bucket-contract task before the second backend verify runs. |
| `spoon-backend/src/msvc/tests/context.rs` | 01-03 Task 1 | Created in the MSVC context migration task before the MSVC contract verify runs. |
| `spoon/tests/cli/status_backend_flow.rs` | 01-04 Task 2 | Created in the app status migration task that introduces backend-driven status JSON verification. |
| `spoon/tests/cli/json_flow.rs` / `spoon/tests/tui/tui_tool_detail_flow.rs` additions | 01-05 through 01-07 | Existing suites gain the new regression names during their owning plans, so no separate bootstrap step is missing. |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI tool and runtime pages still render coherent status after backend read-model and detail-surface switch | BNDR-05, LAY-02 | Ratatui presentation quality and operator sanity are easiest to confirm interactively after large adapter changes | Run `cargo test -p spoon --test tui_tool_detail_flow`, then launch Spoon TUI against a temp root and verify tools/runtime pages show backend-derived paths and detail data without local reconstruction |
| PATH mutation remains correct across current-process and persisted-user scopes after host split | BNDR-01, LAY-03 | Windows PATH side effects are environment-sensitive and should be spot-checked in a real shell | Execute one managed Scoop install in a temp root, verify current shell PATH update, then reopen shell and confirm persisted PATH behavior remains correct |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-28
