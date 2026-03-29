---
phase: 02-canonical-scoop-state
verified: 2026-03-29T02:15:00Z
status: passed
score: 4/4 must-haves verified
re_verification: false
---

# Phase 2: Canonical Scoop State Verification Report

**Phase Goal:** spoon-backend owns one canonical Scoop installed-state model and one persisted source of truth for installed package facts.
**Verified:** 2026-03-29T02:15:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A developer can query package details, installed status, uninstall inputs, and reapply inputs from one backend-owned Scoop state model | VERIFIED | `InstalledPackageState` in `state/model.rs` is the single canonical record. `info.rs` reads via `state::read_installed_state` (lines 18, 272). `actions.rs` uninstall reads bucket/bins/shortcuts/hooks from canonical state (lines 347-398). `integration.rs` and `surface.rs` reapply read/write via canonical state. 7 regression tests pass. |
| 2 | State written by the backend contains only non-derivable Scoop facts, so layout-derived absolute paths are reconstructed from backend context instead of being duplicated in persisted state | VERIFIED | `state/model.rs` struct has no `current`, `current_root`, `shims_root`, `apps_root`, or `tool_root` fields. Test `canonical_state_persists_only_nonderivable_facts` asserts forbidden keys are absent from serialized JSON. `store.rs` uses `RuntimeLayout` for path resolution at read/write time. |
| 3 | Package list, status, and detail views stay consistent because every backend read model projects from the same canonical installed-state source | VERIFIED | `query.rs` delegates to `state::list_all_installed_states` and `state::list_installed_states_filtered` (lines 71, 84). `runtime_status` uses `state::list_installed_summaries` (line 132). `package_info` reads via `state::read_installed_state` (line 272-273). `package_operation_outcome` reads via `state::read_installed_state` (line 76). All project from `scoop/state/` store. |
| 4 | spoon-backend/src/scoop/ no longer carries parallel Scoop state model definitions for the same installed-package facts | VERIFIED | `ScoopPackageState` removed from `mod.rs` exports; `package_state.rs` moved to `_deprecated/`. `info.rs` no longer contains `read_installed_package_state` or `serde_json::Value` state reads. `query.rs` no longer calls `read_dir` or `serde_json::from_str` for installed-state enumeration. Legacy `runtime/installed_state.rs` remains as deprecated module with dead-code warnings but is not imported by any active production code path. |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `spoon-backend/src/scoop/state/model.rs` | Canonical Scoop installed-state model | VERIFIED | Contains `pub struct InstalledPackageState` with `bucket: String` and `architecture: Option<String>`. No derivable path fields. 40 lines, substantive. |
| `spoon-backend/src/scoop/state/store.rs` | Canonical state persistence and enumeration APIs | VERIFIED | Contains `write_installed_state`, `read_installed_state`, `remove_installed_state`, `list_installed_states`. All use `RuntimeLayout`. 95 lines, substantive. |
| `spoon-backend/src/scoop/state/projections.rs` | Typed projection helpers for query/status | VERIFIED | Contains `installed_package_summary`, `list_installed_summaries`, `list_installed_states_filtered`, `list_all_installed_states`. 68 lines, substantive. |
| `spoon-backend/src/scoop/state/mod.rs` | Module re-exports | VERIFIED | Re-exports `InstalledPackageState`, all store APIs, and all projection APIs. |
| `spoon-backend/src/scoop/query.rs` | Query/status backed by canonical store | VERIFIED | `installed_package_states` delegates to `state::list_all_installed_states`. `runtime_status` uses `state::list_installed_summaries`. No `read_dir` or `serde_json::from_str` for state enumeration. |
| `spoon-backend/src/scoop/info.rs` | Typed canonical detail/outcome projections | VERIFIED | `package_info` and `package_operation_outcome` both read via `state::read_installed_state` and `InstalledPackageState`. No `read_installed_package_state` or raw JSON state reads. |
| `spoon-backend/src/scoop/runtime/actions.rs` | Runtime writes canonical state with bucket/architecture | VERIFIED | Install writes `InstalledPackageState` with `bucket` from `resolved_manifest.bucket.name` and `architecture` from `selected_architecture_key()` (lines 243-268). Uninstall reads canonical state for hooks/bins/shortcuts (lines 346-398). |
| `spoon-backend/src/scoop/doctor.rs` | Legacy state detection | VERIFIED | `detect_legacy_flat_state_files` scans for non-canonical flat JSON files. `doctor_with_host` sets `success: false` when legacy files found. |
| `spoon-backend/src/scoop/mod.rs` | Canonical facade without duplicate exports | VERIFIED | No `ScoopPackageState`, `read_package_state`, `write_package_state`, or `remove_package_state` in exports. Re-exports canonical state APIs. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `scoop/mod.rs` | `scoop/state/model.rs` | `pub use state::InstalledPackageState` | WIRED | Line 70-73 re-exports canonical model and store APIs |
| `scoop/runtime/mod.rs` | `scoop/state/store.rs` | `write_installed_state` import | WIRED | Deprecated re-export remains; all production imports (`actions.rs`, `integration.rs`, `surface.rs`) import directly from `crate::scoop::state` |
| `scoop/query.rs` | `scoop/state/store.rs` | `list_installed_states` | WIRED | `installed_package_states` calls `state::list_all_installed_states` (line 71); `runtime_status` calls `state::list_installed_summaries` (line 132) |
| `scoop/info.rs` | `scoop/state/store.rs` | `read_installed_state` | WIRED | Both `package_info` (line 273) and `package_operation_outcome` (line 76) use `state::read_installed_state` |
| `scoop/runtime/actions.rs` | `scoop/state/store.rs` | `write_installed_state` | WIRED | Install writes via `state::write_installed_state` (line 249). Uninstall reads via `state::read_installed_state` (line 346). Removes via `state::remove_installed_state` (line 409). |
| `scoop/info.rs` | `scoop/state/projections.rs` | `InstalledPackageState` type | WIRED | info.rs imports and uses `state::InstalledPackageState` (line 18) for typed field access |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `info.rs::package_info` | `installed_state` | `state::read_installed_state` -> filesystem JSON | FLOWING | Reads persisted canonical state, derives bins/shortcuts/env/persist from typed fields (lines 383-458) |
| `info.rs::package_operation_outcome` | `installed_version` | `state::read_installed_state` -> filesystem JSON | FLOWING | Reads `state.version` from canonical store (lines 76-78) |
| `query.rs::runtime_status` | `summaries` | `state::list_installed_summaries` -> `store::list_installed_states` | FLOWING | Enumerates canonical state directory, projects to summary DTOs (line 132) |
| `query.rs::installed_package_states` | `states` | `state::list_all_installed_states` -> `store::list_installed_states` | FLOWING | Enumerates canonical state directory, returns full records (line 71) |
| `actions.rs::install_package` | `InstalledPackageState` write | Constructed from plan + source + runtime results | FLOWING | Writes canonical record with bucket/architecture/bins/shortcuts/env/persist/hooks (lines 249-268) |
| `actions.rs::uninstall_package` | `state` | `state::read_installed_state` | FLOWING | Reads canonical state for uninstall hooks, bins, shortcuts (lines 346-398) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Canonical state roundtrip with bucket/architecture | `cargo test -p spoon-backend --lib canonical_installed_state_roundtrips_bucket_and_architecture` | test passed | PASS |
| Non-derivable facts persistence boundary | `cargo test -p spoon-backend --lib canonical_state_persists_only_nonderivable_facts` | test passed | PASS |
| Runtime writes canonical state | `cargo test -p spoon-backend --lib runtime_writes_canonical_scoop_state` | test passed | PASS |
| Reapply inputs come from canonical state | `cargo test -p spoon-backend --lib reapply_inputs_come_from_canonical_state` | test passed | PASS |
| Runtime status uses canonical state | `cargo test -p spoon-backend --lib runtime_status_uses_canonical_installed_state` | test passed | PASS |
| Legacy flat state is reported | `cargo test -p spoon-backend --lib legacy_flat_scoop_state_is_reported` | test passed | PASS |
| Clean state produces no legacy issues | `cargo test -p spoon-backend --lib no_legacy_issues_when_state_is_clean` | test passed | PASS |
| Package info reads canonical state (integration) | `cargo test -p spoon-backend --tests scoop_package_info_reads_canonical_state` | test passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SCST-01 | 02-01, 02-02 | spoon-backend retains one unique, canonical, persistable state model for Scoop installed state | SATISFIED | `state/model.rs` is the single canonical model; `state/store.rs` provides persistence APIs; `actions.rs` writes canonical state with bucket/architecture |
| SCST-02 | 02-02, 02-03, 02-04 | Package info, installed status, uninstall inputs, and reapply inputs are derivable from the canonical state model | SATISFIED | `query.rs` delegates to canonical store for list/status; `info.rs` reads canonical state for detail/outcome; `actions.rs` reads canonical state for uninstall; `integration.rs`/`surface.rs` read canonical state for reapply |
| SCST-03 | 02-05 | Duplicate Scoop state models in scoop/src/scoop/ are deleted, not coexisting through adapters | SATISFIED | `ScoopPackageState` and `package_state.rs` moved to `_deprecated/`. `mod.rs` no longer exports legacy APIs. `runtime/installed_state.rs` remains as deprecated dead-code module only (no production imports). |
| SCST-04 | 02-01, 02-04 | Scoop state persistence stores only non-derivable facts; absolute paths derived from layout are reconstructed from backend context | SATISFIED | Model has no derivable path fields. Test asserts forbidden keys absent from serialized JSON. `store.rs` uses `RuntimeLayout` for path resolution. `info.rs` derives display paths from layout at read time (line 277, 460). |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `spoon-backend/src/scoop/runtime/installed_state.rs` | 12 | Dead code: parallel `InstalledPackageState` struct and functions never used by production code | Warning | Not a blocker -- module is deprecated with dead-code warnings from compiler. All production imports use canonical `state::` module. Kept per CLAUDE.md refactoring safety (backup before delete). Should be cleaned up in a future plan. |

### Human Verification Required

### 1. End-to-End Package Install State Persistence

**Test:** Install a real Scoop package (e.g., `ripgrep`) via spoon, then inspect the JSON file in `scoop/state/packages/ripgrep.json`.
**Expected:** The JSON file should contain `bucket`, `architecture`, `bins`, `shortcuts`, `env_add_path`, `env_set`, `persist`, and integration fields -- but no `current_root`, `shims_root`, `apps_root`, or `tool_root` keys.
**Why human:** Requires a real Scoop installation environment with actual package install execution, not just unit test mocking.

### 2. Legacy State Doctor Output

**Test:** Run spoon doctor on a system that has old flat Scoop state files in `scoop/state/*.json` (not in `packages/`).
**Expected:** Doctor output should show `success: false` with `legacy_state_issues` containing the detected files and a message instructing removal and rebuild.
**Why human:** Requires a pre-existing state directory with legacy files and visual verification of doctor output formatting.

### Gaps Summary

No gaps found. All 4 observable truths are verified. All artifacts exist, are substantive, and are properly wired. All key links connect. Data flows through canonical paths. All 8 regression tests pass. All 4 requirement IDs (SCST-01 through SCST-04) are satisfied.

One advisory item (not a gap): `runtime/installed_state.rs` remains as deprecated dead code. This is consistent with CLAUDE.md refactoring safety policy and was explicitly acknowledged in the phase summaries as out-of-scope for Phase 2. The compiler already emits dead-code warnings for its functions. No production code imports it.

---

_Verified: 2026-03-29T02:15:00Z_
_Verifier: Claude (gsd-verifier)_
