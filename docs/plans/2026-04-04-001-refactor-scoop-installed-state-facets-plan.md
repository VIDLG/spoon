---
date: 2026-04-04
type: refactor
title: "refactor: Scoop installed state facets"
status: completed
origin: docs/brainstorms/2026-04-04-scoop-installed-state-facets-requirements.md
---

# refactor: Scoop installed state facets

## Problem Frame

Before this refactor, `spoon-backend/src/scoop/state.rs` persisted Scoop installed state as one flat `InstalledPackageState` model backed by one wide `installed_packages` table. That shape still stored canonical facts, but it mixed identity, command surface, integrations, and uninstall lifecycle data in one record.

The rest of the Scoop domain has already been decomposed into narrower modules. State is now the largest remaining place where unrelated concerns are still packed together. This refactor will redesign the Scoop installed-state contract in a forward-only way so the Rust model, SQLite schema, and read/write API all reflect the same semantic facets.

Source of truth: `docs/brainstorms/2026-04-04-scoop-installed-state-facets-requirements.md`

## Requirements Trace

| Requirement | Planned treatment |
|---|---|
| R1-R3 | Replace the flat state shape with grouped facet structs while continuing to store only non-derivable facts |
| R4-R8 | Replace the wide `installed_packages` row with a primary identity table plus facet tables; store integrations one-per-row |
| R9-R11 | Rework the Scoop state helpers around the facet-based model while keeping stable high-level read/write/list/remove entrypoints |
| R12-R13 | Update install, uninstall, reapply, query, and info readers/writers so behavior remains equivalent and integrations stay structured |

## Context & Research

### Existing repo patterns

- `spoon-backend/src/scoop/state.rs` was the canonical installed-state model and store helper before decomposition.
- `spoon-backend/src/control_plane/schema/0001_control_plane.sql` originally defined one wide `installed_packages` table with multiple JSON-serialized fields.
- `spoon-backend/src/scoop/actions/install.rs` wrote all Scoop state in one `write_installed_state` call after install/update.
- `spoon-backend/src/scoop/actions/uninstall.rs` consumed uninstall fields from one loaded state object before removing the row.
- `spoon-backend/src/scoop/host/integration.rs` and `spoon-backend/src/scoop/host/surface/reapply.rs` updated only one slice of the state but still needed canonical store writes.
- `spoon-backend/src/scoop/info/package.rs` and `spoon-backend/src/scoop/query.rs` consumed the installed-state contract directly.
- `spoon-backend/src/scoop/tests/state.rs` and `spoon-backend/src/tests/control_plane.rs` already provided focused coverage around state roundtrips and control-plane bootstrap before the current refactor was completed.

### Planning decisions from origin

- Forward-only redesign; no compatibility bridge required (see origin)
- Top-level state must group identity, command surface, integrations, and uninstall state (see origin)
- The grouped Scoop state should live in a Scoop-owned shared model layer that both store and higher-level readers/writers depend on (see origin)
- Integrations must become one row per integration (see origin)
- Command-surface data should stay grouped as one facet rather than exploding into many micro-tables (see origin)

### External research decision

Skipped. The repo already has strong local patterns for SQLite-backed state, narrow backend facades, and recent Scoop decomposition work. This plan is best driven by repo-grounded restructuring rather than generic external guidance.

## Resolved During Planning

- **Top-level Rust name:** keep the public top-level name `InstalledPackageState`, but change its internal shape to grouped facet structs. This preserves the domain name while still giving the codebase explicit semantic structure.
- **Model/store layering:** split Scoop state into a Scoop-owned shared model layer plus a store layer, rather than keeping model definitions and SQLite persistence fused in one file.
- **Primary table strategy:** keep `installed_packages` as the package identity table instead of inventing a second top-level name. Its columns should shrink to package-level identity fields only.
- **Facet table set:** introduce:
  - `installed_package_command_surface`
  - `installed_package_integrations`
  - `installed_package_uninstall`
- **Integration storage:** use one row per integration with a uniqueness constraint on `(package, integration_key)` or equivalent foreign-key pair.
- **Read posture:** keep one canonical composed full-state read/write path first, while making the schema facet-friendly enough that later work can add facet-specific reads without redesigning storage again.
- **Migration posture:** use a new control-plane migration to create facet tables, backfill data from the previous wide shape, rebuild `installed_packages` as the identity table, and then switch all Scoop readers/writers to the new layout in one forward migration step.

## High-Level Technical Design

### State shape

The top-level Scoop state becomes a composition of facet structs that live in a shared Scoop-owned model layer:

| Facet | Contents | Persistence shape |
|---|---|---|
| `identity` | package, version, bucket, architecture, cache_size_bytes | `installed_packages` |
| `command_surface` | bins, shortcuts, env_add_path, env_set, persist | `installed_package_command_surface` |
| `integrations` | applied integration key/value rows | `installed_package_integrations` |
| `uninstall` | pre_uninstall, uninstaller_script, post_uninstall | `installed_package_uninstall` |

Recommended module shape:

- `spoon-backend/src/scoop/state/model.rs`
  - grouped state structs and lightweight helper methods
- `spoon-backend/src/scoop/state/store.rs`
  - SQLite row types plus read/write/list/remove helpers
- `spoon-backend/src/scoop/state/mod.rs`
  - stable re-export surface

This keeps the state model reusable inside Scoop without introducing a backend-global abstraction that MSVC or unrelated domains must depend on.

### Storage direction

`installed_packages` remains the root package record. The facet tables attach by package identity and are loaded or written together by the canonical store helpers.

This keeps:

- one obvious root row for package-level facts
- one grouped table for command surface state
- one normalized table for integrations
- one grouped table for uninstall lifecycle state

### Read/write composition

`spoon-backend/src/scoop/state/store.rs` remains the canonical state store module, while `spoon-backend/src/scoop/state/model.rs` becomes the shared semantic contract. The store stops serializing one giant row. Instead it:

1. reads the package identity row
2. reads zero-or-one command-surface row
3. reads zero-or-many integration rows
4. reads zero-or-one uninstall row
5. composes the grouped `InstalledPackageState`

Writes follow the inverse path:

1. upsert identity
2. upsert command-surface facet
3. replace integration rows for that package
4. upsert uninstall facet

This keeps the backend contract simple while allowing future facet-specific reads to emerge naturally.

## Open Questions

### Deferred to Implementation

- What exact foreign-key and delete-cascade shape is cleanest in SQLite for the new facet tables?
- Should the model layer stop at `state/model.rs`, or should the facet structs eventually split again into `state/model/identity.rs`, `state/model/command_surface.rs`, `state/model/integrations.rs`, and `state/model/uninstall.rs` if the shared model grows noisy?
- Whether package summary/list queries should continue to reuse the composed read path initially, or add a dedicated identity-only query immediately.

## Implementation Units

### Unit 1: Introduce the new Scoop state schema and shared model layer

**Goal:** Replace the flat installed-state shape with grouped facet structs, establish a Scoop-owned shared model layer, and align schema to the new facets.

**Files:**
- `spoon-backend/src/control_plane/schema/0001_control_plane.sql`
- `spoon-backend/src/scoop/state/model.rs`
- `spoon-backend/src/scoop/state/store.rs`
- `spoon-backend/src/scoop/state/mod.rs`
- `spoon-backend/src/db.rs` (only if migration/bootstrap wiring needs adjustment)

**Approach:**
- Add grouped facet structs under the public `InstalledPackageState` root in `state/model.rs`.
- Shrink `installed_packages` to identity fields only.
- Add `installed_package_command_surface`, `installed_package_integrations`, and `installed_package_uninstall`.
- Keep `state/mod.rs` as the stable Scoop-owned export surface so higher layers depend on model semantics rather than store implementation details.
- Use one new migration or one rebuilt bootstrap schema path so the SQLite layout matches the grouped model exactly.

**Pattern references:**
- `spoon-backend/src/scoop/state.rs` (pre-refactor fused shape that was replaced)
- `spoon-backend/src/control_plane/schema/0001_control_plane.sql`

**Test files:**
- `spoon-backend/src/scoop/tests/state.rs`
- `spoon-backend/src/tests/control_plane.rs`

**Test scenarios:**
- Roundtrip a grouped installed state with every facet populated.
- Roundtrip a minimal installed state where optional facets are absent.
- Verify integrations persist as multiple rows rather than one serialized JSON blob.
- Verify the SQLite bootstrap/migration creates all required facet tables.

### Unit 2: Rebuild canonical state store helpers around facet composition

**Goal:** Keep `read_installed_state`, `write_installed_state`, `remove_installed_state`, and `list_installed_states` as stable backend entrypoints while moving their persistence logic into a store layer that depends on the shared state model.

**Files:**
- `spoon-backend/src/scoop/state/store.rs`
- `spoon-backend/src/scoop/state/model.rs`
- `spoon-backend/src/scoop/state/mod.rs`

**Approach:**
- Replace the old wide-row store helper shape with row types aligned to identity and facet tables.
- Implement canonical composed reads from identity + command surface + integrations + uninstall tables.
- Implement writes as coordinated upserts/replacements across the facet tables.
- Ensure removal deletes all facet rows for a package.
- Keep model-level helper methods light and semantic; do not move SQL, row decoding, or persistence policy back into the model layer.

**Pattern references:**
- `spoon-backend/src/db.rs`
- `spoon-backend/src/scoop/state.rs` (pre-refactor fused shape that was replaced)

**Test files:**
- `spoon-backend/src/scoop/tests/state.rs`

**Test scenarios:**
- `read_installed_state` composes all facets correctly.
- `write_installed_state` updates changed facets without dropping unrelated ones.
- `remove_installed_state` removes every facet row for the package.
- `list_installed_states` returns complete grouped states for multiple packages.

### Unit 3: Update Scoop write paths to emit facet-based state

**Goal:** Keep install, reapply, and uninstall behavior equivalent while making all writers populate the new grouped state model.

**Files:**
- `spoon-backend/src/scoop/actions/install.rs`
- `spoon-backend/src/scoop/actions/uninstall.rs`
- `spoon-backend/src/scoop/host/integration.rs`
- `spoon-backend/src/scoop/host/surface/reapply.rs`

**Approach:**
- Update install/update writes to construct grouped facets explicitly.
- Update reapply command-surface logic to rewrite only the command-surface facet through the canonical store helper.
- Update reapply integrations logic to rewrite only the integrations facet through the canonical store helper.
- Update uninstall flows to read uninstall and command-surface facets from the grouped state.

**Pattern references:**
- `spoon-backend/src/scoop/actions/install.rs`
- `spoon-backend/src/scoop/host/integration.rs`
- `spoon-backend/src/scoop/host/surface/reapply.rs`

**Test files:**
- `spoon-backend/src/scoop/tests/runtime.rs`
- `spoon-backend/src/scoop/tests/state.rs`

**Test scenarios:**
- Install/update writes populate every intended facet.
- Integration reapply updates integrations without mutating unrelated state facts.
- Command-surface reapply updates bins/env surface without mutating uninstall or identity facets.
- Uninstall still consumes the grouped uninstall facet correctly.

### Unit 4: Update Scoop read/query/info consumers to use the grouped contract

**Goal:** Make the rest of Scoop read the new grouped state shape without re-flattening it as the internal source of truth.

**Files:**
- `spoon-backend/src/scoop/query.rs`
- `spoon-backend/src/scoop/info/package.rs`
- `spoon/src/service/scoop/report.rs`
- `spoon/src/cli/run.rs`

**Approach:**
- Update query and info consumers to read from grouped facets.
- Preserve existing user-visible output semantics while mapping through the new grouped state.
- Avoid introducing a shadow flat DTO as the canonical internal representation.

**Pattern references:**
- `spoon-backend/src/scoop/query.rs`
- `spoon-backend/src/scoop/info/package.rs`
- `spoon/src/service/scoop/report.rs`

**Test files:**
- `spoon-backend/src/scoop/tests/state.rs`
- `spoon-backend/tests/scoop_integration.rs`
- `spoon/src/service/scoop/report.rs` (if there are direct report-level tests nearby, extend them; otherwise keep focused backend tests)

**Test scenarios:**
- Runtime status still lists installed packages correctly from the new identity facet.
- Package info still resolves command, environment, persist, and integration sections correctly.
- CLI/service package listing continues to report names and versions without depending on the old flat row shape.

### Unit 5: Refresh the state-focused safety net around the new facet boundaries

**Goal:** Ensure the new schema and grouped model are protected by seam-level tests rather than only happy-path coverage.

**Files:**
- `spoon-backend/src/scoop/tests/state.rs`
- `spoon-backend/src/scoop/tests/runtime.rs`
- `spoon-backend/src/tests/control_plane.rs`
- `spoon-backend/tests/scoop_integration.rs`

**Approach:**
- Add tests near the store and runtime seams that assert facet composition, facet-local updates, and schema bootstrap.
- Prefer focused backend assertions over broad integration flows.
- Add regression coverage for the two most likely mistakes: partial facet loss on write, and incorrect integration row replacement.

**Pattern references:**
- `spoon-backend/src/scoop/tests/state.rs`
- `spoon-backend/src/tests/control_plane.rs`
- `spoon-backend/tests/scoop_integration.rs`

**Test scenarios:**
- State roundtrip preserves all facets.
- Updating only integrations does not erase command surface or uninstall facet data.
- Updating only command surface does not erase integration rows.
- Control-plane bootstrap exposes the expected facet tables and identity table shape.
- Package info still reflects canonical grouped state correctly after the schema change.

## System-Wide Impact

- `spoon-backend/src/scoop/mod.rs` will keep exporting `InstalledPackageState`, but the meaning of its fields will become grouped rather than flat.
- `spoon-backend/src/scoop/state/mod.rs` becomes a Scoop-owned shared seam that both persistence code and higher Scoop layers depend on.
- Every Scoop path that reads or writes state is now coupled to the new facet-aware store implementation: installs, updates, uninstall, reapply, package info, runtime status, and package listing.
- The app crate should not need product-level behavior changes, but compile-time updates are likely where it directly consumes backend Scoop state.
- This refactor increases future leverage: integration repair, state doctoring, and facet-specific reads all become easier once the schema reflects the domain.

## Risks & Mitigations

- **Risk:** SQLite migration complexity could create a brittle one-time schema transition.
  **Mitigation:** keep the migration focused on Scoop state only; validate with control-plane bootstrap tests and explicit facet-table assertions.

- **Risk:** Writers may accidentally drop untouched facet data when updating only one slice.
  **Mitigation:** make canonical store helpers own facet replacement logic and add regression tests for partial updates.

- **Risk:** Query/info consumers may quietly reintroduce flat-shape assumptions.
  **Mitigation:** treat `query.rs` and `info/package.rs` as first-class implementation units with explicit verification scenarios.

- **Risk:** The shared model layer could drift into backend-global abstraction pressure.
  **Mitigation:** keep the model layer Scoop-owned and resist factoring it upward unless another domain demonstrates the same semantic needs concretely.

- **Risk:** The grouped model could become more nested but not actually easier to use.
  **Mitigation:** keep the top-level root small and name each facet around a real domain concern; do not split command-surface fields into unnecessary micro-facets yet.

## Execution Notes

- This plan implies a **characterization-first posture** around state persistence: lock down current logical behavior with focused backend tests before and during the refactor, then change the schema/model confidently.

## Next Step

Recommended next step: `/prompts:ce-work`
