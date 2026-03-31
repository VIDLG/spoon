# Phase 2: Canonical Scoop State - Research

**Researched:** 2026-03-28
**Domain:** Scoop installed-state consolidation inside `spoon-backend`
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** End Phase 2 with exactly one canonical Scoop installed-package record in `spoon-backend`.
- **D-02:** Base the canonical record on `InstalledPackageState`, not on `ScoopPackageState` and not on a third model.
- **D-03:** Merge `bucket` and `architecture` into the canonical installed-package record.
- **D-04:** Persist only non-derivable Scoop facts; do not store absolute paths or `current` path data.
- **D-05:** Use forward design, not compatibility-preserving migration layers.
- **D-06:** Legacy Scoop state may become invalid across the Phase 2 boundary; handle it through explicit repair or rebuild paths rather than compatibility shims.
- **D-07:** Package list, package detail, uninstall inputs, reapply inputs, and status surfaces must all project from the same canonical backend state source.
- **D-08:** Switch these consumers in one pass during Phase 2; do not split write unification and read unification across phases.
- **D-09:** `spoon` stays a thin consumer of backend results; Phase 2 is backend-owned state work.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCST-01 | `spoon-backend` keeps one canonical, persisted Scoop installed-state model. | Replace duplicate `ScoopPackageState` / `InstalledPackageState` paths with one store under `spoon-backend/src/scoop/state/`. |
| SCST-02 | Package info, installed status, uninstall inputs, and reapply inputs derive from the same canonical model. | Move `query.rs`, `info.rs`, and lifecycle consumers onto typed projections from the canonical store. |
| SCST-03 | Duplicate Scoop state models in `spoon-backend/src/scoop/` are removed, not preserved behind adapters. | Delete `package_state.rs` and stop exporting its APIs after projections are moved. |
| SCST-04 | Persisted Scoop state contains only non-derivable facts, not absolute layout paths. | Keep layout/path derivation in `RuntimeLayout`; store only package/version/bucket/architecture and operation-relevant facts. |

</phase_requirements>

## Summary

Phase 2 is not a broad Scoop rewrite. It is a state-ownership refactor with one hard requirement: by the end of the phase there must be one installed-package truth source, and every major backend read surface must consume it. The live code already points to the right base model. `InstalledPackageState` is written by runtime lifecycle code, read by `query.rs`, and partially used by `info.rs`. `ScoopPackageState` is a thinner duplicate model exported from `scoop/mod.rs`, but no meaningful active call path depends on it. That makes `InstalledPackageState` the correct canonical base.

The most important technical cleanup is to remove state-reading ambiguity, not just duplicate struct definitions. Today `query.rs` enumerates package files directly, `info.rs` rereads installed state as raw `serde_json::Value`, and runtime writes omit `bucket` and `architecture`. If Phase 2 only patches the write schema, the codebase will still have split-brain reads. The right shape is a dedicated `scoop/state/` module containing the canonical record, persistence APIs, enumeration APIs, and typed projections that `query.rs`, `info.rs`, uninstall, and reapply all share.

The user's discuss decisions also sharpen rollout strategy. There should be no compatibility-preserving migration layer. Planning should therefore avoid dual-read or dual-write shims and instead execute a forward-design cut: canonical state module introduced, runtime writes updated, read surfaces moved in one pass, `package_state.rs` removed, and stale old flat-state files surfaced explicitly as repair-needed state rather than silently supported.

## Current Code Reality

### What already points to the canonical model
- `spoon-backend/src/scoop/runtime/installed_state.rs` defines `InstalledPackageState` and persists runtime facts the backend actually uses.
- `spoon-backend/src/scoop/runtime/actions.rs` writes `InstalledPackageState` during install/update and reads it during uninstall/update decisions.
- `spoon-backend/src/scoop/query.rs` already enumerates installed package files and deserializes them into `InstalledPackageState`.
- `spoon-backend/src/scoop/info.rs` already derives installed version from `InstalledPackageState` enumeration.

### What still causes split-brain behavior
- `spoon-backend/src/scoop/package_state.rs` defines `ScoopPackageState` as a second persisted model with `name`, `version`, `bucket`, and `architecture`.
- `spoon-backend/src/scoop/info.rs` still re-reads installed state as raw `serde_json::Value`.
- `InstalledPackageState` currently lacks `bucket` and `architecture`.
- `scoop/mod.rs` exports both the old and new state APIs, preserving ambiguity in the backend public surface.

### Recent seam cleanup to preserve
- `SystemPort` remains in backend crate-root ports for cross-domain OS/runtime effects.
- Scoop-specific host callbacks now live under `spoon-backend/src/scoop/ports.rs` as `ScoopIntegrationPort`, which is a cleaner scope for Phase 2 work than the older crate-root `PackageIntegrationPort` naming.
- Display-only pip mirror formatting is no longer part of backend lifecycle ports, so Phase 2 should keep projection/display helpers separate from lifecycle callbacks.

## Architecture Patterns

### Recommended Module Split

```text
spoon-backend/src/scoop/
  mod.rs
  query.rs
  info.rs
  runtime/
  state/
    mod.rs
    model.rs
    store.rs
    projections.rs
```

### Pattern 1: State Store plus Projections
One store module handles persistence and enumeration; one projections module converts canonical state into status/detail/outcome DTO inputs.

### Pattern 2: Runtime writes canonical state, never view DTOs
Lifecycle code writes one installed-package record containing only non-derivable facts.

### Pattern 3: Forward-design stale-state handling
When old flat state files from `package_state.rs` still exist, surface a typed legacy-state issue or doctor finding instead of keeping compatibility code alive.

## Concrete Findings

### Duplicate state surface
`ScoopPackageState` appears only in:
- `spoon-backend/src/scoop/package_state.rs`
- `spoon-backend/src/scoop/mod.rs` re-exports

No meaningful active backend or app call path was found depending on `read_package_state(...)` or `write_package_state(...)`, which makes removal low-risk once the export surface is cleaned up.

### Typed state already drives the real flows
`InstalledPackageState` is consumed by:
- runtime lifecycle writes in `spoon-backend/src/scoop/runtime/actions.rs`
- list/status enumeration in `spoon-backend/src/scoop/query.rs`
- package info/version checks in `spoon-backend/src/scoop/info.rs`
- backend integration tests in `spoon-backend/tests/scoop_integration.rs`

### Biggest Phase 2 cleanup target
`spoon-backend/src/scoop/info.rs` is the most important read-side cleanup because it still mixes manifest projection, raw JSON probing, and installed-state logic in one file.

## Recommended Plan Order

1. Canonical state module
2. Runtime write/read migration
3. Query/status projections
4. Info/outcome/uninstall/reapply projections
5. Legacy removal and stale-state detection

## Anti-Patterns to Avoid

- Dual-read compatibility shims
- Persisting derived roots or `current` paths
- Projection logic inside app code
- Deleting old state without explicit stale-state detection

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` |
| Config file | none - standard Cargo test discovery |
| Quick run command | `cargo test -p spoon-backend canonical_installed_state_roundtrips_bucket_and_architecture -- --nocapture` |
| Full suite command | `cargo test -p spoon-backend --lib scoop && cargo test -p spoon --test json_flow scoop_info_json_prints_structured_package_view -- --nocapture && cargo test -p spoon --test status_backend_flow json_status_uses_backend_read_models -- --nocapture` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCST-01 | One canonical installed-state model exists and round-trips bucket/architecture. | backend unit | `cargo test -p spoon-backend canonical_installed_state_roundtrips_bucket_and_architecture -- --nocapture` | Created in Plan 02-01 |
| SCST-02 | Query/detail/uninstall/reapply consumers read the same canonical state. | backend integration | `cargo test -p spoon-backend scoop_package_info_reads_canonical_state -- --nocapture` | Created/updated in Plans 02-03 / 02-04 |
| SCST-03 | Legacy `ScoopPackageState` APIs are removed from `scoop/mod.rs`. | build/audit | `rg -n "ScoopPackageState|read_package_state|write_package_state" spoon-backend/src/scoop` | Existing command |
| SCST-04 | Persisted state stores no absolute paths or `current` roots. | backend unit | `cargo test -p spoon-backend canonical_state_persists_only_nonderivable_facts -- --nocapture` | Created in Plan 02-01 |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/02-canonical-scoop-state/02-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/research/SUMMARY.md`
- `.planning/research/FEATURES.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/codebase/STRUCTURE.md`
- `.planning/codebase/CONVENTIONS.md`
- `.planning/codebase/STACK.md`
- `.planning/codebase/CONCERNS.md`
- `spoon-backend/src/scoop/package_state.rs`
- `spoon-backend/src/scoop/runtime/installed_state.rs`
- `spoon-backend/src/scoop/runtime/actions.rs`
- `spoon-backend/src/scoop/query.rs`
- `spoon-backend/src/scoop/info.rs`
- `spoon-backend/src/scoop/mod.rs`
- `spoon-backend/tests/scoop_integration.rs`

### Secondary (MEDIUM confidence)
- `spoon/tests/cli/json_flow.rs`
- `spoon/tests/cli/status_backend_flow.rs`

## Metadata

**Confidence breakdown:**
- State-model direction: HIGH
- Projection cleanup target selection: HIGH
- Legacy-state handling without compatibility: MEDIUM-HIGH

**Research date:** 2026-03-28
**Valid until:** 2026-04-27
