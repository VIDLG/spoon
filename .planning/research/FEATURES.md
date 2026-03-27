# Feature Landscape

**Domain:** Spoon backend refactor, centered on Scoop runtime ownership and app/backend boundary cleanup
**Researched:** 2026-03-28

## Table Stakes

Features the refactor must deliver. Missing any of these means the refactor did not actually fix the architecture.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Single Scoop state model owned by `spoon-backend` | The current backend already has overlapping persisted models in `spoon-backend/src/scoop/package_state.rs` and `spoon-backend/src/scoop/runtime/installed_state.rs`. The top refactor outcome must be one canonical model for installed package state, runtime metadata, and reapply inputs. | High | Keep one persisted state contract for Scoop packages and delete the loser rather than layering adapters over both. |
| Backend-only ownership of Scoop lifecycle execution | `install`, `update`, `uninstall`, bucket sync, manifest resolution, command-surface reapply, and integration reapply already cluster in `spoon-backend/src/scoop/`. The refactor must finish that move and stop leaving behavior split with `spoon/src/service/scoop/`. | High | `spoon/` should call backend operations and render results. It should not reconstruct Scoop behavior. |
| Backend-only Git/Gix responsibility | `gix` usage is already concentrated in `spoon-backend/src/gitx.rs`, while `spoon/Cargo.toml` still carries a separate `gix` dependency. For this refactor, Git access is table stakes backend territory. | Medium | The app should depend on backend Git outcomes, not on `gix` or Git workflow details. |
| Thin, stable app/backend API surface | The app currently reshapes backend results through `spoon/src/service/mod.rs`, `spoon/src/service/scoop/mod.rs`, and `spoon/src/service/scoop/runtime.rs`. The refactor must reduce this to thin orchestration and presentation mapping. | High | Prefer a small set of backend request/response types per operation over many helper calls and app-owned reconstruction. |
| Root-scoped, deterministic backend operations | The repo direction requires root-derived paths, and Scoop path helpers already exist in `spoon-backend/src/scoop/paths.rs`. Every Scoop operation should run only from explicit root context, never from ambient env assumptions. | Medium | No hardcoded install roots, no hidden global runtime mutation, no duplicate path derivation in `spoon/`. |
| Scoop runtime cleanup around typed phases | `spoon-backend/src/scoop/runtime/actions.rs` currently mixes dependency install, extraction, persist sync, hook execution, shim writes, shortcut writes, and state persistence in one large flow. A cleaned runtime pipeline is table stakes for maintainability. | High | Split into explicit phases with narrow helpers and one authoritative transition order. |
| App-facing outcomes that are already presentation-ready | CLI and TUI both need the same backend facts: status, progress, action outcome, package details, and repair/reapply results. Backend results should be complete enough that `spoon/src/cli/` and `spoon/src/tui/` only format them. | Medium | Avoid forcing the app to re-read state files or infer install status after backend actions. |
| Regression-focused test split by ownership | The repo already prefers backend tests in `spoon-backend/tests/` and TUI harness tests in `spoon/tests/tui/`. The refactor must increase backend unit and integration coverage for Scoop flows while shrinking app-side behavioral duplication. | High | Add backend tests for failure paths in `spoon-backend/src/scoop/runtime/actions.rs`; keep `spoon/` tests focused on CLI/TUI flow correctness. |
| Explicit replacement of bad abstractions, not compatibility wrappers | Project direction explicitly says to prefer forward design and not preserve bad abstractions for compatibility. | Medium | Delete stale helper layers when a better backend contract exists. Do not keep both APIs alive "for now." |

## Differentiators

Features that would make this refactor materially stronger than a normal cleanup. These are not the first acceptance bar, but they are worth pursuing once the baseline is stable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Transactional Scoop action pipeline with rollback points | The biggest current runtime risk is partial install/update state in `spoon-backend/src/scoop/runtime/actions.rs`. A rollback-capable pipeline turns the refactor from structural cleanup into reliability improvement users will feel. | High | At minimum, preserve prior install metadata and current-link state until the new version is validated. |
| Canonical backend domain contract for package state, details, and action reports | Right now state, details, and outcomes are spread across `spoon-backend/src/scoop/info.rs`, `spoon-backend/src/scoop/query.rs`, and runtime state files. A coherent backend contract makes future CLI/TUI work cheaper. | Medium | Design around backend domain types, not around existing CLI output shapes. |
| Capability-driven reapply model | `reapply_package_command_surface_streaming*` and `reapply_package_integrations_streaming*` already exist. Formalizing these as capabilities of installed packages makes config changes safer and clearer. | Medium | Persist only the minimal facts needed to replay command surface and integrations deterministically. |
| Backend diagnostics that explain boundary violations | A doctor/report surface that can say "state file missing", "manifest resolved but current root invalid", or "integration metadata stale" would make the new backend self-auditing. | Medium | This fits naturally next to `spoon-backend/src/scoop/doctor.rs` and is valuable during the refactor itself. |
| Operation-oriented backend API instead of helper soup | A few explicit backend entry points such as `plan`, `execute`, `query`, `reapply`, and `doctor` scale better than the current broad export surface from `spoon-backend/src/scoop/mod.rs`. | Medium | This is a differentiator because it improves future change velocity, not just present cleanliness. |
| Failure-path test fixtures for Scoop runtime phases | Most existing Scoop tests validate happy-path parsing or top-level flows. Purpose-built fixtures for hook failure, missing bins, persist sync errors, and interrupted updates would make the refactor unusually safe. | Medium | These tests belong in `spoon-backend/src/scoop/tests/` and `spoon-backend/tests/scoop_integration.rs`. |

## Anti-Features

Features to explicitly NOT build during this refactor.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Preserve both `ScoopPackageState` and `InstalledPackageState` behind adapters | This locks in the exact duplication the refactor was started to remove. | Pick one canonical persisted model in `spoon-backend/src/scoop/` and migrate all reads/writes to it. |
| Keep app-owned Scoop behavior in `spoon/src/service/scoop/` because the UI already depends on it | That keeps `spoon/` as a hidden second backend. | Move behavior down, then let `spoon/` adapt only for CLI/TUI rendering and cancellation wiring. |
| Add a new generic abstraction over Scoop and MSVC before Scoop is clean | The project explicitly says Scoop is the main target and `msvc` is only touched when necessary. Premature generalization will preserve bad seams. | Finish a clean Scoop model first; only extract shared patterns after the Scoop boundary is proven. |
| Recreate backend state in UI/status code for convenience | `spoon/src/status/discovery/probe.rs` already reads package state files directly. More app-side state inference would deepen duplication. | Expose query/status data from `spoon-backend` and let the app consume it. |
| Keep direct `gix` usage or dependency drift in `spoon/` | That weakens the "backend owns Git" rule and increases duplicated dependency surface. | Route all Git work through `spoon-backend` and remove app-side dependency where possible. |
| Expand shell-string orchestration as the main cleanup strategy | Inline shell orchestration is already a concern in the current Scoop runtime. More shell glue would make the refactor shallower, not cleaner. | Use typed Rust helpers for state transitions and keep shell execution at a narrow compatibility edge only. |
| Compatibility aliases for every old helper/export in `spoon-backend/src/scoop/mod.rs` | A giant re-export surface makes it hard to know what the supported backend contract actually is. | Break callers onto a smaller intentional API and delete dead exports. |
| PTY-heavy or end-to-end test replacement for backend unit coverage | The repo direction explicitly prefers harness and backend tests over PTY coverage. | Put state and runtime correctness tests in `spoon-backend`; keep CLI/TUI tests focused on user flows. |

## Feature Dependencies

```text
Canonical Scoop state model
  -> Stable backend query/details contract
  -> Reliable reapply operations
  -> App stops reading Scoop state files directly

Backend-only Scoop lifecycle ownership
  -> Thin app/backend API surface
  -> Backend-only Git/Gix responsibility
  -> Backend-focused failure-path tests

Typed runtime phases
  -> Transactional/rollback improvements
  -> Better doctor/diagnostic output
  -> Lower-risk future MSVC cleanup
```

## MVP Recommendation

Prioritize:
1. Canonicalize Scoop installed state in `spoon-backend/src/scoop/` and remove the duplicate persisted model path.
2. Collapse Scoop action/query/reapply ownership into backend entry points so `spoon/src/service/scoop/` becomes thin orchestration only.
3. Add backend tests for the risky runtime boundaries in `spoon-backend/src/scoop/runtime/actions.rs`, especially failure after extraction, failure after `refresh_current_entry(...)`, and reapply/state rewrite behavior.

Defer:
- Rollback-capable transactions: high value, but easier once one state model and one runtime pipeline exist.
- Broad `msvc` cleanup: only touch `spoon-backend/src/msvc/` where Scoop boundary work forces it.
- New UI affordances: the first win is cleaner backend capability, not richer TUI decoration.

## Sources

- Local architecture and ownership context: `.planning/PROJECT.md`
- Current crate boundary analysis: `.planning/codebase/ARCHITECTURE.md`
- Current testing strategy: `.planning/codebase/TESTING.md`
- Current risk inventory: `.planning/codebase/CONCERNS.md`
- Canonical backend exports and current surface sprawl: `spoon-backend/src/scoop/mod.rs`
- Duplicate Scoop state models: `spoon-backend/src/scoop/package_state.rs`, `spoon-backend/src/scoop/runtime/installed_state.rs`
- Current Scoop runtime execution seam: `spoon-backend/src/scoop/runtime/actions.rs`
- Current Scoop runtime host boundary: `spoon-backend/src/scoop/runtime/execution.rs`, `spoon/src/service/scoop/runtime.rs`
- Current app-side Scoop orchestration: `spoon/src/service/scoop/mod.rs`, `spoon/src/service/scoop/actions.rs`
- Current app-side package-owned integrations and shims: `spoon/src/packages/mod.rs`, `spoon/src/packages/git.rs`, `spoon/src/packages/python.rs`
- Current app-side direct Scoop status inference: `spoon/src/status/discovery/probe.rs`
- Current backend and app dependency split for Git: `spoon-backend/src/gitx.rs`, `spoon-backend/Cargo.toml`, `spoon/Cargo.toml`
