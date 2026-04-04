---
date: 2026-04-04
topic: scoop-installed-state-facets
---

# Scoop Installed State Facets

## Problem Frame

Before this refactor, `spoon-backend/src/scoop/state.rs` stored the Scoop installed-package contract as one flat `InstalledPackageState` model backed by one wide `installed_packages` row. That shape was still correct in the sense that it stored canonical non-derivable facts, but it mixed several different concerns:

- package identity
- command surface and package-local runtime environment
- applied integrations
- uninstall-time lifecycle data

This makes the state harder to reason about, pushes unrelated changes through one model and one storage contract, and leaves the codebase with weaker semantic boundaries than the rest of the recently decomposed Scoop domain.

The goal is to redesign the Scoop installed-state contract in a forward-only way:

- no backward-compatibility layer for the current flat shape
- Rust model, read/write API, and SQLite schema can all change together
- the result should be clearer to query, easier to evolve, and easier to test

## Requirements

**State Model**
- R1. The Scoop installed-state domain must be represented as semantic facets rather than one flat `InstalledPackageState` shape.
- R2. The top-level Scoop installed-state model must group state into at least:
  identity, command surface, integrations, and uninstall state.
- R3. The top-level state model must continue to represent only canonical non-derivable facts; layout-derived values such as absolute current paths must remain reconstructed from runtime layout rather than persisted.

**SQLite Shape**
- R4. SQLite storage must move from one wide `installed_packages` record to a primary package table plus facet-oriented storage.
- R5. Package identity must remain in a primary package row that owns the package-level key and high-level package facts.
- R6. Command-surface state must be stored as a dedicated command-surface facet rather than mixed into the primary identity row.
- R7. Uninstall lifecycle state must be stored as a dedicated uninstall facet rather than mixed into the primary identity row.
- R8. Applied integrations must be stored as structured records with one integration per row, not as a serialized map blob.

**Read/Write Contract**
- R9. Scoop read/write APIs must expose the facet-based model directly rather than reconstructing the previous flat shape as the canonical internal contract.
- R10. The backend must still support reading, writing, listing, and removing installed Scoop package state through stable helpers, but those helpers must operate on the new facet-based state model.
- R11. Queries that need only one facet must be able to evolve toward facet-specific reads later; the redesign must not force every future reader through a single all-fields blob path.

**Behavior Preservation**
- R12. Existing Scoop runtime behavior must remain logically equivalent after the redesign: installs, updates, reapply flows, package info, and uninstall flows must continue to persist and read the same underlying package facts, only through the new facet structure.
- R13. The redesign must keep `integrations` precise enough for future selective repair, doctor, diff, or targeted reapply workflows.

## Success Criteria

- The Scoop installed-state model is easier to understand because identity, command surface, integrations, and uninstall concerns are visibly separated.
- The SQLite shape reflects those same semantic boundaries instead of hiding them in one wide row and several serialized blobs.
- Future work on package info, reapply, doctor, or state repair can target the relevant facet without mentally unpacking one monolithic state object.
- Planning can reason about the new state contract without inventing missing boundaries.

## Scope Boundaries

- This change is Scoop-specific and does not require redesigning MSVC state in the same phase.
- This brainstorm does not require preserving backward compatibility with the current flat Scoop state schema or API shape.
- This brainstorm does not require defining exact table names, SQL migrations, or Rust file layout; those belong in planning.
- This brainstorm does not require splitting every command-surface field into its own table. Command-surface data may stay grouped as one facet table.

## Key Decisions

- **Forward-only redesign:** The new Scoop state contract may replace the old flat model directly without a compatibility bridge.
- **Facet-based model:** `InstalledPackageState` should become a grouped model with identity, command surface, integrations, and uninstall facets.
- **Facet-oriented SQLite shape:** Storage should move to a primary package table plus facet-oriented persistence instead of one wide row.
- **Structured integrations:** `integrations` should be stored as one row per integration because it is the most naturally queryable and selectively repairable facet.
- **Grouped command surface:** `bins`, `shortcuts`, `env_add_path`, `env_set`, and `persist` should stay grouped in one command-surface facet table rather than exploding into many micro-tables immediately.

## Dependencies / Assumptions

- Scoop action, query, info, and reapply code paths require coordinated updates wherever they still assume the former flat installed-state shape.
- The current control-plane/SQLite foundation is stable enough after Phase 12.1 and 12.2 to support another Scoop-specific schema reshape.
- The shared state model should be Scoop-specific rather than a new backend-global generic model package. The goal is to separate Scoop state semantics from Scoop persistence details, not to introduce a cross-domain abstraction prematurely.

## Outstanding Questions

### Deferred to Planning
- [Affects R4][Technical] What exact table layout and foreign-key strategy should the facet tables use?
- [Affects R9][Technical] Should the top-level Rust type remain named `InstalledPackageState`, or should the grouped shape use a more explicit name such as `InstalledPackageRecord` plus facet structs?
- [Affects R10][Needs research] Which existing readers can move to facet-specific reads immediately, and which should continue using a composed full-state read first?
- [Affects R12][Technical] What migration sequence is the safest if the old schema is being replaced rather than bridged?

## Next Steps

-> `/prompts:ce-plan` for structured implementation planning
