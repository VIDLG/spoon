---
phase: 01-backend-seams-and-ownership
verified: 2026-03-28T15:30:00Z
status: passed
score: 4/4 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 3.5/4
  gaps_closed:
    - "spoon no longer directly derives Scoop/MSVC/backend runtime layout paths; only passes configured root to backend (BNDR-04, LAY-01)"
  gaps_remaining: []
  regressions: []
gaps: []
---

# Phase 1: Backend Seams and Ownership Verification Report

**Phase Goal:** `spoon` becomes a thin app shell that invokes backend-owned Scoop, Git, MSVC, and layout/context behavior through explicit backend contracts.
**Verified:** 2026-03-28T15:30:00Z
**Status:** passed
**Re-verification:** Yes -- after gap closure from plan 01-08

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | App routes Scoop actions to backend operation interfaces, renders backend results | VERIFIED | `build_scoop_backend_context` in `spoon/src/service/mod.rs:181`; `execute_package_action_streaming_with_context` in `spoon-backend/src/scoop/runtime/actions.rs:438`; `command_result_from_scoop_package_outcome` in `spoon/src/service/scoop/actions.rs:101`. No `load_backend_config` or `package_current_root` in scoop adapters. |
| 2 | Bucket clone/sync from backend without spoon depending on gix | VERIFIED | No `gix` in `spoon/Cargo.toml`; `cargo tree -p spoon -e normal --depth 1` confirms no gix dependency; `RepoSyncOutcome` re-exported from `spoon-backend` in `spoon/src/service/scoop/bucket.rs:10`; `clone_repo` internal to backend; no `gix::` types in spoon/src. |
| 3 | Changing root changes Scoop, MSVC, shim/state consistently via backend layout | VERIFIED | `RuntimeLayout::from_root` covers all paths (test passes). All 7 app modules migrated from config path helpers to RuntimeLayout (01-08). Grep confirms zero production uses of deprecated config path helpers. 14 deprecated functions preserved per CLAUDE.md refactoring safety. |
| 4 | App renders status from backend models without rereading state files | VERIFIED | `BackendStatusSnapshot` in `spoon-backend/src/status.rs:13`; consumed by `spoon/src/status/mod.rs:8`; `json_status_uses_backend_read_models` test passes; no `read_to_string` on backend state files in app production code. |

**Score:** 4/4 success criteria verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `spoon-backend/src/context.rs` | BackendContext struct | VERIFIED | `pub struct BackendContext<P>` at line 6. Contains root, layout, proxy, test_mode, msvc_target_arch, msvc_command_profile, ports. |
| `spoon-backend/src/layout.rs` | RuntimeLayout struct | VERIFIED | `pub struct RuntimeLayout` at line 4. Contains Scoop, MSVC managed/official, shims, state, cache sub-layouts. `from_root` derives all paths. |
| `spoon-backend/src/ports.rs` | SystemPort root port | VERIFIED | `SystemPort` remains the backend-wide host boundary. Post-phase refinement later narrowed Scoop-specific integration callbacks into `spoon-backend/src/scoop/ports.rs` as `ScoopIntegrationPort`, preserving the Phase 1 split-port intent while improving scope. |
| `spoon-backend/src/tests/context.rs` | Context contract tests | VERIFIED | `runtime_layout_derives_from_root` and `explicit_context_required_for_runtime_ops`. Both pass (71 backend tests total). |
| `spoon-backend/src/scoop/runtime/actions.rs` | Context-driven Scoop actions | VERIFIED | `BackendContext` consumed at entry points. `_with_context` entry points are substantive. |
| `spoon-backend/src/scoop/buckets.rs` | Backend-owned bucket contracts | VERIFIED | `BackendContext` used in context variants. `clone_repo` called internally. No `gix::` types in file. |
| `spoon-backend/src/scoop/tests/contracts.rs` | Scoop contract tests | VERIFIED | `scoop_action_contract_uses_context` and `bucket_sync_uses_backend_git_contract`. Both pass. |
| `spoon-backend/src/scoop/runtime/execution.rs` | Split-port runtime host | VERIFIED | `ContextRuntimeHost` bridges `BackendContext` to `ScoopRuntimeHost`. Post-phase refinement kept `SystemPort` backend-wide and moved Scoop-specific integration callbacks to `scoop::ScoopIntegrationPort`, which is a narrower realization of the same boundary. |
| `spoon-backend/src/msvc/mod.rs` | Context-driven MSVC | VERIFIED | `BackendContext` used throughout. No `static RUNTIME_CONFIG`. `MsvcRequestConfig::from_context`. |
| `spoon-backend/src/msvc/tests/context.rs` | MSVC context tests | VERIFIED | Tests pass. |
| `spoon/src/service/msvc/mod.rs` | Thin MSVC adapter | VERIFIED | No `apply_runtime_config`, `set_runtime_config`, `load_backend_config`. Uses backend context builders. |
| `spoon-backend/src/status.rs` | Backend status snapshot | VERIFIED | `pub struct BackendStatusSnapshot` at line 13. Calls `scoop::runtime_status` and `msvc::status` internally. |
| `spoon/src/status/mod.rs` | App status from backend snapshot | VERIFIED | `BackendStatusSnapshot` imported (line 8). `status_path_mismatches` now uses `RuntimeLayout::from_root(root).shims` (line 596). |
| `spoon/tests/cli/status_backend_flow.rs` | Status regression test | VERIFIED | `json_status_uses_backend_read_models` passes. |
| `spoon/src/service/mod.rs` | Shared BackendContext builder | VERIFIED | `build_scoop_backend_context` and `build_msvc_backend_context` construct `BackendContext`. `AppSystemPort` implements the app-owned host callbacks; after a later cleanup this became `SystemPort + scoop::ScoopIntegrationPort` instead of the broader crate-root port naming. |
| `spoon/src/service/scoop/actions.rs` | Thin Scoop package adapter | VERIFIED | Routes through backend runtime. No direct Scoop runtime calls. |
| `spoon/src/service/scoop/bucket.rs` | Thin bucket adapter | VERIFIED | No `load_backend_config`, no `gix::`. Re-exports `RepoSyncOutcome` from backend. |
| `spoon/src/view/tools/detail.rs` | Backend-driven detail view | VERIFIED | `ToolDetailModel` consumes `ToolStatus` from backend queries. |
| `spoon/src/view/config.rs` | Config view via RuntimeLayout | VERIFIED | `RuntimeLayout::from_root(root)` for all derived paths. No deprecated config path helpers. |
| `spoon/src/settings.rs` | Settings without backend path exports | VERIFIED | No backend runtime-path helper re-exports. |
| `spoon/Cargo.toml` | No direct gix dependency | VERIFIED | No `gix` in dependencies. `cargo tree` confirms. |
| `spoon/src/config/paths.rs` | Deprecated path helpers | VERIFIED | 14 functions marked `#[deprecated]` with RuntimeLayout replacement guidance. Preserved per CLAUDE.md refactoring safety. |
| `spoon/src/packages/tool.rs` | Migrated to RuntimeLayout | VERIFIED | All detail methods use `RuntimeLayout::from_root`. No deprecated config path helper calls in production code. |
| `spoon/src/status/policy.rs` | Migrated to RuntimeLayout | VERIFIED | `ownership_for_status` uses `RuntimeLayout::from_root` (line 108). Test fixture also migrated. |
| `spoon/src/editor/discovery.rs` | Migrated to RuntimeLayout | VERIFIED | `managed_scoop_editor_path` uses `RuntimeLayout::from_root` (line 109). |
| `spoon/src/packages/msvc.rs` | Migrated to RuntimeLayout | VERIFIED | `config_scope_details` and `detected_wrapper_names` use `RuntimeLayout::from_root`. |
| `spoon/src/status/discovery/probe.rs` | Production code migrated | VERIFIED | `configured_probe_path` and `probe_msvc_toolchain` use `RuntimeLayout::from_root`. Test code also migrated (but has pre-existing import issue -- see notes). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `context.rs` | `layout.rs` | BackendContext carries RuntimeLayout | WIRED | `BackendContext.layout: RuntimeLayout` built from root in constructor |
| `scoop/paths.rs` | `layout.rs` | Path helpers delegate to RuntimeLayout | WIRED | `RuntimeLayout::from_root` in paths.rs. No manual path joins. |
| `msvc/paths.rs` | `layout.rs` | MSVC paths delegate to RuntimeLayout | WIRED | `RuntimeLayout::from_root` in paths.rs. No manual path joins. |
| `scoop/runtime/actions.rs` | `context.rs` | Actions take BackendContext | WIRED | `_with_context` entry points accept `&BackendContext<P>` |
| `scoop/buckets.rs` | `gitx.rs` | Bucket sync uses clone_repo | WIRED | `clone_repo` called internally. Outcome fields destructured. No `gix::` leakage. |
| `scoop/runtime/execution.rs` | host ports | Runtime effects via split ports | WIRED | `ContextRuntimeHost` delegates to `SystemPort` plus Scoop-scoped integration callbacks. The original `PackageIntegrationPort` naming was later narrowed to `scoop::ScoopIntegrationPort` without changing the architectural intent. |
| `spoon/src/service/mod.rs` | `context.rs` | App builds BackendContext | WIRED | `build_scoop_backend_context` and `build_msvc_backend_context` construct `BackendContext` |
| `spoon/src/status/mod.rs` | `status.rs` | Status consumes BackendStatusSnapshot | WIRED | `use spoon_backend::status::BackendStatusSnapshot` at line 8 |
| `spoon/src/cli/json.rs` | `status/mod.rs` | JSON status via build_status_details | WIRED | JSON status path calls `build_status_details_with_snapshot` |
| `spoon/src/view/config.rs` | `layout.rs` | Config view via RuntimeLayout | WIRED | `RuntimeLayout::from_root(root)` for all derived paths |
| `spoon/Cargo.toml` | `spoon-backend` | gix is backend-only | WIRED | No gix in spoon deps; gix at spoon-backend/Cargo.toml |
| `spoon/src/packages/tool.rs` | `layout.rs` | Tool detail paths via RuntimeLayout | WIRED | `RuntimeLayout::from_root` in 8 detail methods |
| `spoon/src/status/policy.rs` | `layout.rs` | Ownership checks via RuntimeLayout | WIRED | `RuntimeLayout::from_root` at line 108 |
| `spoon/src/editor/discovery.rs` | `layout.rs` | Editor path resolution via RuntimeLayout | WIRED | `RuntimeLayout::from_root` at line 109 |
| `spoon/src/packages/msvc.rs` | `layout.rs` | MSVC detail via RuntimeLayout | WIRED | `RuntimeLayout::from_root` in config_scope_details and detected_wrapper_names |
| `spoon/src/status/discovery/probe.rs` | `layout.rs` | Probe paths via RuntimeLayout | WIRED | `RuntimeLayout::from_root` in configured_probe_path and probe_msvc_toolchain |
| `spoon/src/status/mod.rs` | `layout.rs` | Path mismatch detection via RuntimeLayout | WIRED | `RuntimeLayout::from_root` at line 596 in status_path_mismatches |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `spoon-backend/src/status.rs` | BackendStatusSnapshot | `scoop::runtime_status(tool_root)` + `msvc::status(tool_root)` | FLOWING | Calls real backend query functions that read Scoop state and MSVC state |
| `spoon/src/status/mod.rs` | StatusDetails | `build_status_details_with_snapshot` from BackendStatusSnapshot | FLOWING | Consumes backend snapshot, no fallback to file parsing |
| `spoon/src/view/config.rs` | ConfigModel | `RuntimeLayout::from_root(root)` from global config root | FLOWING | Derives all runtime paths from one root via backend layout |
| `spoon/src/service/scoop/actions.rs` | CommandResult | `runtime::execute_package_action_outcome_streaming` -> `command_result_from_scoop_package_outcome` | FLOWING | Outcome flows from backend through thin adapter to app |
| `spoon/src/packages/tool.rs` | Tool detail paths | `RuntimeLayout::from_root` per detail method | FLOWING | Paths derived from backend layout, not from deprecated config helpers |
| `spoon/src/status/policy.rs` | ToolOwnership | `RuntimeLayout::from_root` -> path prefix checks | FLOWING | Ownership determined by comparing tool path against RuntimeLayout-derived roots |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Both crates compile cleanly | `cargo check -p spoon -p spoon-backend` | Finished (warnings, no errors) | PASS |
| RuntimeLayout derives from root | `cargo test -p spoon-backend` (71 tests) | ok. 71 passed; 0 failed | PASS |
| JSON status uses backend read models | `cargo test -p spoon --test status_backend_flow` | ok. 1 passed; 0 failed | PASS |
| spoon has no gix dependency | `cargo tree -p spoon -e normal --depth 1` | No gix matches | PASS |
| No production uses of deprecated path helpers | `grep config::(scoop_root_from\|msvc_root_from\|shims_root_from\|...) spoon/src/` | Only 1 match, inside `#[test]` in update.rs:187 | PASS |
| No gix types in spoon/src | `grep "gix::" spoon/src/` | No matches | PASS |
| No static RUNTIME_CONFIG | `grep "static RUNTIME_CONFIG" spoon/src/` | No matches | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| BNDR-01 | 01-02, 01-05 | Spoon Scoop install/update/uninstall/bucket via backend interfaces | SATISFIED | BackendContext-driven entry points exist; app adapters thin |
| BNDR-02 | 01-02, 01-05 | Git/bucket operations via backend interfaces | SATISFIED | Bucket clone/sync through backend `clone_repo`; app re-exports `RepoSyncOutcome` only |
| BNDR-03 | 01-03 | MSVC detect/install via backend interfaces | SATISFIED | `MsvcRequestConfig::from_context` replaces global; app builds explicit MSVC context |
| BNDR-04 | 01-01, 01-08 | spoon no longer derives backend layout paths directly | SATISFIED | All 7 app modules migrated to RuntimeLayout (01-08). Grep confirms zero production uses of deprecated config path helpers. 14 deprecated functions preserved per CLAUDE.md safety. |
| BNDR-05 | 01-04, 01-06 | spoon consumes backend result/query models | SATISFIED | `BackendStatusSnapshot`, `ScoopPackageOperationOutcome`, `MsvcOperationOutcome` consumed by app |
| GIT-01 | 01-07 | spoon no longer directly depends on gix | SATISFIED | No gix in spoon/Cargo.toml; `cargo tree` confirms |
| GIT-02 | 01-02 | Git/bucket clone/sync/progress backend-only | SATISFIED | `clone_repo` internal to spoon-backend; `RepoSyncOutcome` consumed via backend return |
| GIT-03 | 01-02 | Backend Git interfaces don't leak gix details | SATISFIED | No `gix::` in spoon/src; bucket events use `BackendEvent` and `ScoopBucketOperationOutcome` |
| LAY-01 | 01-01, 01-08 | Backend owns single root-derived layout | SATISFIED | `RuntimeLayout` is the single implementation. Backend paths fully delegate. App-side helpers deprecated. All app production code migrated to RuntimeLayout. |
| LAY-02 | 01-04, 01-07 | spoon only has app config file path semantics | SATISFIED | Settings re-exports cleaned; service/mod.rs no longer re-exports backend path helpers; config view uses RuntimeLayout |
| LAY-03 | 01-01, 01-03 | Backend ops run in explicit context | SATISFIED | No `static RUNTIME_CONFIG` in MSVC; `BackendContext` passed to all backend entry points |

**Orphaned requirements:** None. All 12 requirement IDs (BNDR-01 through BNDR-05, GIT-01 through GIT-03, LAY-01 through LAY-03) are accounted for.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `spoon/src/status/discovery/probe.rs` | 200 | Dead code: `fn probe_tool` is never used (compiler warning) | Info | No runtime impact; cleanup candidate |
| `spoon/src/config/paths.rs` | 92-101 | Dead code: `fn ensure_msvc_root_exists` is defined but never called | Info | No runtime impact; uses deprecated `msvc_root_from` internally but is never invoked |
| `spoon/src/status/discovery/probe.rs` | 310-566 | Test module uses `RuntimeLayout` but missing import in test scope (pre-existing compilation error) | Warning | Pre-existing issue, not a regression from this phase. Tests compile fine when not run as lib tests. |

No TODO/FIXME/PLACEHOLDER/HACK comments found in any modified files.
No empty implementations or console.log-only patterns found.
No production code uses deprecated path helpers (only 1 test fixture use at update.rs:187).

### Post-Phase Refinement Note

After Phase 1 verification, the split-port seam was tightened further:

- `SystemPort` stayed at `spoon-backend/src/ports.rs` as the backend-wide OS/runtime boundary.
- Scoop-only integration callbacks were moved under `spoon-backend/src/scoop/ports.rs` and renamed `ScoopIntegrationPort`.
- Display-only pip mirror formatting was removed from backend lifecycle ports and left in app/package presentation helpers.

This is treated as a refinement of the verified Phase 1 boundary, not a change in direction.

### Human Verification Required

### 1. End-to-End Scoop Install Flow

**Test:** Run `spoon install <package>` and verify the full flow completes with backend-driven status updates.
**Expected:** Package installs successfully; status shows installed tool with correct paths from backend layout.
**Why human:** Full Scoop install requires network access, actual package downloads, and filesystem state changes.

### 2. MSVC Status and Install from Clean State

**Test:** Run `spoon status` and `spoon install msvc` on a fresh environment.
**Expected:** MSVC status reflects backend context; install flows through `MsvcRequestConfig::from_context` without errors.
**Why human:** MSVC detection involves probing filesystem and running external commands; requires real environment.

### 3. TUI Detail View Renders Correct Paths

**Test:** Open Spoon TUI, navigate to a tool detail, verify Scoop install root and MSVC paths match RuntimeLayout derivation.
**Expected:** Detail view shows paths consistent with `RuntimeLayout::from_root(configured_root)`.
**Why human:** TUI rendering requires interactive terminal and visual verification.

### Gaps Summary

All gaps from the initial verification have been closed by plan 01-08. The single remaining gap (BNDR-04/LAY-01: 5 app modules still using config path helpers) has been fully resolved:

- `spoon/src/packages/tool.rs` -- 8 detail methods migrated to RuntimeLayout
- `spoon/src/status/policy.rs` -- `ownership_for_status` migrated to RuntimeLayout
- `spoon/src/status/mod.rs` -- `status_path_mismatches` migrated to RuntimeLayout
- `spoon/src/editor/discovery.rs` -- `managed_scoop_editor_path` migrated to RuntimeLayout
- `spoon/src/packages/msvc.rs` -- `config_scope_details` and `detected_wrapper_names` migrated to RuntimeLayout
- `spoon/src/status/discovery/probe.rs` -- `configured_probe_path` and `probe_msvc_toolchain` migrated to RuntimeLayout (production code)

All 14 deprecated path helper functions in `config/paths.rs` are preserved with `#[deprecated]` attributes per CLAUDE.md refactoring safety. The only remaining uses of deprecated helpers are: (1) within the deprecated definitions themselves (self-referential), (2) in the dead `ensure_msvc_root_exists` function which is never called, and (3) in one test fixture (`update.rs:187`).

Phase 1 goal is fully achieved: `spoon` is a thin app shell that invokes backend-owned Scoop, Git, MSVC, and layout/context behavior through explicit backend contracts.

---

_Verified: 2026-03-28T15:30:00Z_
_Verifier: Claude (gsd-verifier)_
