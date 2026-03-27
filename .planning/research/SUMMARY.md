# Project Research Summary

**Project:** spoon
**Domain:** Windows developer-tool bootstrapper refactor with Rust CLI/TUI frontend and shared backend runtime crate
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

`spoon` is not being redesigned as a new product. It is a brownfield architecture correction for a Windows tool bootstrapper that already has the right crate split on paper but not yet in practice. The consistent recommendation across the research is to make `spoon` a thin app shell and make `spoon-backend` the only place that understands Scoop layout, manifest resolution, bucket sync, install/update/uninstall execution, runtime state, and Git-backed bucket operations. Experts build this kind of system around one canonical backend state model, one backend-owned layout abstraction, and a narrow app-to-backend contract that returns presentation-ready outcomes.

The practical direction is clear: keep the current three-crate workspace, consolidate shared dependencies at the workspace level, remove backend implementation crates from `spoon`, and finish the ownership move that has already started in `spoon-backend`. The first real milestone is not UI work. It is establishing backend seams, then replacing duplicated Scoop state with one canonical installed-package record, then splitting the current Scoop lifecycle monolith into explicit phases that the app only invokes and renders.

The main risk is a superficial refactor that moves files without moving responsibility. If `spoon` still computes Scoop paths, re-reads state files, or re-orchestrates lifecycle steps, the project will keep paying the same coordination cost across two crates. The second major risk is breaking install/update/uninstall ordering while cleaning up `runtime/actions.rs`, because the current flow has real side effects and no rollback journal. Mitigation is to freeze the lifecycle order in tests, introduce explicit backend context and phase boundaries first, and only then move code and delete compatibility layers.

## Key Findings

### Recommended Stack

The stack direction is conservative and corrective, not expansive. Keep the existing Cargo workspace and crate split, but use workspace inheritance to remove version drift and move backend implementation dependencies out of the app crate. `spoon-backend` should be the only crate that owns `gix`, `reqwest`, `msi`, archive/checksum tooling, and persisted runtime state; `spoon` should keep only CLI/TUI, config, and app-specific integration glue.

The highest-value dependency changes are the ones that collapse duplicate graphs and reinforce ownership boundaries: unify `gix` on `0.80.x` in `spoon-backend`, unify backend HTTP on `reqwest 0.13.x` with Rustls only, unify `msi` on `0.10.x`, and move shared foundations such as `tokio`, `serde`, `serde_json`, and `tracing` into `[workspace.dependencies]`. Do not add a new shared-types crate and do not switch Git implementations.

**Core technologies:**
- `Cargo workspace` with `resolver = "3"`: single lockfile and shared dependency policy to remove drift across `spoon`, `spoon-backend`, and `xtask`.
- `spoon-backend`: canonical backend crate for Scoop, Git, MSVC, runtime state, and filesystem mutations.
- `gix 0.80.x`: sole Git implementation for bucket sync and repo operations, owned only by `spoon-backend`.
- `reqwest 0.13.x` with Rustls: backend HTTP client aligned with the current Git stack and simpler Windows behavior.
- `tokio 1.x`: shared async runtime for backend execution and app orchestration.
- `serde` and `serde_json`: shared foundation for canonical backend models, persisted state, and app-consumable results.

### Expected Features

This refactor has a hard line between table stakes and differentiators. Table stakes are the architectural corrections required to make the split real: one Scoop state model, backend-only lifecycle ownership, backend-only Git ownership, root-scoped deterministic operations, typed runtime phases, presentation-ready backend outcomes, and a test strategy that shifts risk coverage into `spoon-backend`. Differentiators are reliability and maintainability upgrades that become realistic after the baseline is clean, especially rollback-aware lifecycle execution, better diagnostics, and a smaller operation-oriented backend API.

**Must have (table stakes):**
- One canonical Scoop installed-state model in `spoon-backend`; remove the duplicate persisted model path.
- Backend-only ownership of Scoop install, update, uninstall, bucket sync, query, and reapply behavior.
- Backend-only ownership of Git and `gix`; remove direct app dependency and assumptions.
- Thin app/backend API where `spoon` routes commands and renders outcomes instead of reconstructing backend behavior.
- Backend-owned root/layout derivation so operations run from explicit root context, not duplicate path math.
- Typed Scoop lifecycle phases with focused backend tests around failure paths and partial-state risks.

**Should have (differentiators):**
- Rollback-aware Scoop action pipeline for package activation and bucket replacement.
- Canonical backend contract for package state, details, and action reports.
- Capability-driven reapply model for command-surface and integration replay.
- Backend diagnostics and doctor output that explain broken state and boundary violations.
- Operation-oriented backend facade such as `plan`, `execute`, `query`, `reapply`, and `doctor`.

**Defer (v2+):**
- Broad MSVC internal cleanup beyond the minimum alignment needed for shared layout/context.
- New generic abstraction spanning Scoop and MSVC before Scoop is clean.
- New UI affordances or decorative presentation work.

### Architecture Approach

The target architecture is a strict shell/core split. `spoon` should own CLI/TUI routing, config loading, app-owned policy/config integration, and output formatting. `spoon-backend` should own the backend layout, Git repo sync, Scoop catalog and lifecycle logic, canonical installed state, projections for info/status/list views, and backend events/outcomes. The current `spoon/src/service/scoop/*` layer should collapse into thin app adapters, while `spoon-backend/src/scoop/runtime/*` should be split into intentional catalog, lifecycle, state, and host/context modules behind a stable facade.

**Major components:**
1. `spoon/src/cli/*`, `spoon/src/tui/*`, `spoon/src/app/*`: command routing, UI state, config-to-request translation, and rendering.
2. `spoon-backend/src/layout.rs`, `spoon-backend/src/git/*`, `spoon-backend/src/scoop/catalog/*`: root/layout derivation, bucket sync, manifest resolution, and search.
3. `spoon-backend/src/scoop/lifecycle/*`, `spoon-backend/src/scoop/state/*`, `spoon-backend/src/scoop/host.rs`: lifecycle execution, canonical persisted state, projections, and narrow app-owned side-effect ports.

### Critical Pitfalls

1. **Moving files without moving ownership**: if `spoon` still computes Scoop paths or orchestrates backend work, the architecture is still split-brain. Avoid this by forcing backend-owned layout, lifecycle, and state reads before pruning wrappers.
2. **Reordering runtime side effects without an explicit state machine**: cleanup inside `runtime/actions.rs` can leave half-applied installs. Avoid this by freezing current phase order in tests and introducing explicit states such as resolved, staged, activated, integrated, and committed.
3. **Deleting duplicate state structs without mapping behavior**: the duplicate models encode different semantics today. Avoid this by building a producer/consumer matrix first, then migrating every read/write path onto one canonical installed-package record.
4. **Centralizing Git nominally but keeping dependency/API leakage**: if `spoon` still depends on `gix` or backend APIs leak gitoxide details, version skew and coupling remain. Avoid this by keeping `gix` only in `spoon-backend` and exposing only backend-level repo operations and events.
5. **Breaking MSVC while cleaning up Scoop shims and paths**: Scoop and MSVC share root topology. Avoid this by scoping cleanup to per-package ownership, never wiping shared shim roots, and adding parity checks where Scoop and MSVC path derivation intersect.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Backend Seams and Dependency Consolidation
**Rationale:** This must come first because every later change depends on a clean crate boundary, unified dependency graph, and explicit backend context/layout types.
**Delivers:** Workspace dependency inheritance, backend-owned layout/context/host seams, backend-local Git API, and removal of direct backend implementation dependencies from `spoon`.
**Addresses:** Thin app/backend API, backend-only Git ownership, root-scoped deterministic operations.
**Avoids:** Ownership-leak pitfall, path-duplication pitfall, Git version-skew pitfall.

### Phase 2: Canonical Scoop State and Read Models
**Rationale:** State duplication is the largest source of behavioral ambiguity and should be removed before lifecycle code is rearranged.
**Delivers:** One canonical installed-package record, one backend state store, typed projections for info/list/status/prefix, and removal of app-side direct state probing.
**Addresses:** Single Scoop state model, presentation-ready backend outcomes, app no longer reconstructing install state.
**Avoids:** Delete-by-name state-model pitfall, frontend status drift, path duplication across crates.

### Phase 3: Scoop Lifecycle Split and App Thinning
**Rationale:** Once state and seams are stable, the monolithic runtime flow can be split safely and the frontend service layer can collapse into thin adapters.
**Delivers:** Explicit lifecycle modules for planner/acquire/install/uninstall/reapply/surface/persist/hooks, backend-owned operation entry points, and reduced `spoon/src/service/scoop/*` responsibility.
**Addresses:** Backend-only lifecycle ownership, typed runtime phases, focused backend tests for risky flows.
**Avoids:** Runtime reordering pitfall, frontend re-orchestration pitfall, uninstall-hook behavior drift.

### Phase 4: Reliability Hardening and Diagnostic Cleanup
**Rationale:** Rollback and doctor improvements have high value, but they should be layered onto a stable state model and stable lifecycle boundaries.
**Delivers:** Backup-and-swap or journaled replacement for buckets/current/state writes, stronger doctor/report surfaces, and pruning of deprecated compatibility exports.
**Addresses:** Transactional lifecycle differentiator, backend diagnostics differentiator, cleaner long-term backend API.
**Avoids:** Bucket replacement/activation rollback pitfall, silent partial-state corruption, dead-export cleanup errors.

### Phase Ordering Rationale

- Phase 1 creates the compile-time and ownership seams the rest of the work needs.
- Phase 2 removes the biggest semantic ambiguity before touching side-effect ordering.
- Phase 3 reorganizes runtime behavior only after state and catalog boundaries are stable.
- Phase 4 hardens correctness and deletes compatibility layers once behavior is already verified.
- MSVC stays mostly out of band; only shared layout/shim safety work should move in these phases.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** legacy state migration and compatibility handling need explicit planning if existing installs must survive schema changes in place.
- **Phase 3:** lifecycle split needs careful behavior capture around hooks, persist restore, and current-link refresh before code motion.
- **Phase 4:** rollback design on Windows needs concrete validation for locked files, rename semantics, and recovery behavior.

Phases with standard patterns (skip research-phase):
- **Phase 1:** workspace dependency consolidation, backend context injection, and facade-first boundary cleanup are well-understood patterns.
- **Parts of Phase 2:** projection/read-model extraction from one canonical store is straightforward once the schema is chosen.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Strong repo-local evidence plus stable Cargo and crate-doc guidance support the dependency and ownership recommendations. |
| Features | HIGH | The table-stakes set is derived directly from current duplication and product-direction constraints, not speculation. |
| Architecture | HIGH | The target split follows the repo's existing crate intent and maps cleanly onto current hot spots in `spoon` and `spoon-backend`. |
| Pitfalls | HIGH | The highest-risk failures are grounded in concrete files and current side-effect ordering, with only rollback mechanics needing more validation. |

**Overall confidence:** HIGH

### Gaps to Address

- Legacy installed-state migration: decide whether old persisted Scoop state must be migrated in place or can be invalidated with a repair path during rollout.
- Windows replacement semantics: validate bucket swap, `current` activation, and state-file writes under locked-file and interrupted-process conditions.
- Hook contract preservation: explicitly decide whether uninstall hook failures continue to be tolerated while install hook failures remain fatal.
- Async boundary cleanup: confirm whether planning/query APIs go fully async in backend now or retain limited sync wrappers at the app edge.

## Sources

### Primary (HIGH confidence)
- `.planning/research/STACK.md` - dependency consolidation, crate-boundary, and backend ownership direction
- `.planning/research/FEATURES.md` - table-stakes, differentiators, anti-features, and MVP priority
- `.planning/research/ARCHITECTURE.md` - target crate split, module structure, lifecycle split, and build order
- `.planning/research/PITFALLS.md` - refactor failure modes, phase warnings, and prevention strategies
- Local code referenced through the research set, especially `spoon-backend/src/scoop/*`, `spoon/src/service/scoop/*`, `spoon/src/config/paths.rs`, `spoon/Cargo.toml`, and `spoon-backend/Cargo.toml`

### Secondary (MEDIUM confidence)
- Cargo workspace reference - workspace dependency inheritance and resolver guidance
- Rust 2024 resolver documentation - resolver `3` behavior
- `gix` crate documentation and feature listings - Git stack placement and feature guidance
- `reqwest` crate documentation - backend HTTP alignment
- `msi` crate documentation - version alignment guidance
- Scoop folder layout and manifest documentation - backend layout and manifest ownership expectations

### Tertiary (LOW confidence)
- None beyond the inferred roadmap sequencing and rollback-hardening recommendations derived from repo-local findings

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
