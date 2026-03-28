# Phase 1: Backend Seams and Ownership - Research

**Researched:** 2026-03-28
**Domain:** Rust workspace seam refactor between `spoon` and `spoon-backend`
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Backend Context and Runtime Ownership
- **D-01:** Phase 1 uses forward design, not a compatibility-heavy transition layer.
- **D-02:** `spoon-backend` will introduce an explicit backend-owned runtime context contract, centered on a `BackendContext` plus backend-owned `RuntimeLayout`.
- **D-03:** Backend operations should consume explicit context instead of scattered `tool_root`, `proxy`, module-local config helpers, or implicit global runtime configuration.

#### OS Integration Boundary
- **D-04:** The OS/runtime boundary is intentionally mixed.
- **D-05:** Generic runtime side effects move into backend ownership: PATH handling, runtime home/layout semantics, shim or command-surface behavior, and backend lifecycle orchestration.
- **D-06:** Spoon-specific configuration writes remain app-owned behind narrow ports only when they belong to Spoon's config domain, such as app-owned package integrations and other product configuration surfaces.

#### Query and State Consumption
- **D-07:** `spoon` should stop directly reading backend state files or reconstructing backend status/detail semantics locally.
- **D-08:** Backend read/query models become the single source the app consumes for runtime status, package detail, and related backend-facing display data.

#### Git and Bucket Interfaces
- **D-09:** The app should consume bucket and backend domain interfaces only. `gitx` remains an internal backend implementation detail and should not shape app contracts.

#### Layout Ownership
- **D-10:** Backend layout derivation is single-owned by `spoon-backend`. The app knows the configured `root`, but backend derives `scoop`, `msvc`, `shims`, `state`, `cache`, and related runtime paths.
- **D-11:** App-side backend path helpers are legacy seams to remove, not a pattern to preserve.

### Claude's Discretion
No additional discretion was requested during discussion. Planning should treat the decisions above as locked.

### Deferred Ideas (OUT OF SCOPE)
None - discussion stayed within Phase 1 ownership and seam design.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BNDR-01 | Route Spoon Scoop install, update, uninstall, and bucket work through backend contracts only. | Backend context, system port split, backend-owned Scoop action and bucket outcomes. |
| BNDR-02 | Route Spoon Git and bucket repository work through backend contracts only. | Keep `gitx` internal, expose only backend repo and bucket outcomes and events. |
| BNDR-03 | Route Spoon MSVC detect and install flows through backend contracts only. | Replace global MSVC runtime config with explicit backend context or request-scoped config. |
| BNDR-04 | Spoon passes configured `root` into backend and stops deriving Scoop/MSVC layout itself. | Introduce `RuntimeLayout` in `spoon-backend` and remove app runtime path helpers from active use. |
| BNDR-05 | Spoon consumes backend result and query models instead of rereading backend state files or rebuilding semantics locally. | Switch status, detail, and package action surfaces to backend read models and outcomes. |
| GIT-01 | Spoon no longer directly depends on `gix`. | Remove `gix` from `spoon/Cargo.toml`; keep Git implementation in backend only. |
| GIT-02 | Backend owns Git and bucket clone, sync, and progress bridging. | Use backend bucket APIs and `BackendEvent`/`RepoSyncOutcome`; no app Git plumbing. |
| GIT-03 | Backend Git interfaces must not leak `gix` types. | Keep `RepoSyncOutcome` and backend progress events as the only app-facing Git surface. |
| LAY-01 | `spoon-backend` owns one root-derived layout implementation for Scoop, MSVC, and shared shims/state. | Consolidate layout in backend path modules under one `RuntimeLayout`. |
| LAY-02 | Spoon owns app config paths only, not backend runtime layout semantics. | Shrink `spoon/src/config/paths.rs`, `view/config.rs`, and package detail helpers to app-only config concerns. |
| LAY-03 | Backend operations run with explicit context, not implicit globals or scattered path inference. | Use `BackendContext` for Scoop, bucket, and MSVC operations; remove `load_backend_config()` and `set_runtime_config()` patterns from runtime paths. |
</phase_requirements>

## Summary

Phase 1 should be planned as a contract refactor, not as a broad cleanup pass. The repo already has usable backend primitives for Scoop runtime work, bucket inventory, package detail, search, runtime status, and Git progress bridging in `spoon-backend`, but `spoon` still owns too much of the runtime contract: root and proxy resolution, layout derivation, some PATH and integration host behavior, status/detail file reads, and duplicated package action result reconstruction. The planner should treat those as the concrete leak sites to eliminate, not as incidental cleanup.

The most important design decision is to make `BackendContext` the only way backend operations get runtime inputs, and to make `RuntimeLayout` the only place that derives runtime paths from `root`. That should be paired with a split port design: backend-owned orchestration decides when PATH, shim, and command-surface side effects happen, while app-owned package integrations remain behind a narrow package port. The biggest sequencing risk is trying to solve Phase 2 state cleanup inside Phase 1. Phase 1 should not change persisted Scoop state shape; it should only stop the app from reading and rebuilding that state directly.

The repo is already close enough that this can be landed in slices. Scoop and bucket contracts are the easiest entry because backend entry points and result types already exist. MSVC is the highest-risk slice because it still uses a hidden mutable global runtime config. Status and detail surfaces are the highest fan-out slice because they touch CLI, TUI, JSON, and view helpers.

**Primary recommendation:** Land Phase 1 in this order: add backend context and layout, split host ports, move Scoop and bucket adapters to explicit backend requests, replace MSVC global runtime config with explicit context, then switch status and detail rendering to backend read models before deleting app path helpers and the app-side `gix` dependency.

## Project Constraints (from CLAUDE.md)

- Keep `spoon` as the CLI/TUI app shell and keep generic backend logic in `spoon-backend`.
- Do not reintroduce PowerShell or shell scripts as the primary Spoon entrypoint.
- Keep CLI mode for automation and TUI as the default interactive experience.
- Use a configurable total `root`; do not hardcode install paths.
- `root` comes from Spoon config or explicit CLI override, not persistent environment variables.
- Avoid writing product configuration like `root` or proxy into user environment variables unless there is no cleaner integration.
- Keep current-process environment mutation distinct from persisted user or machine mutation.
- Replace the repository-root `spoon.exe` in place whenever possible.
- Prefer backend behavior tests in `spoon-backend` and keep `spoon` tests focused on app flows and integration glue.
- Prefer `ratatui` state-machine tests through the TUI `Harness`; keep PTY and real terminal tests minimal.
- Keep real backend flows isolated, temporary, and opt-in.
- Do not move generic backend logic back into `spoon/` unless it is truly app-specific glue.
- Keep docs aligned with the Rust executable workflow.
- No project-local skills were found under `.claude/skills/` or `.agents/skills/`.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `spoon-backend` | `0.1.0` | Backend contract owner for Scoop, Git, MSVC, and runtime layout/query behavior. | Already contains the reusable runtime, query, and bucket primitives this phase should consolidate around. |
| `spoon` | `0.1.0` | Thin CLI/TUI shell and Spoon-owned integration/config adapter. | Matches the locked phase goal and current crate split. |
| `tokio` | `1` | Async runtime for backend operations, event streaming, file IO, and task orchestration. | Already used by both crates; no reason to introduce a second async pattern. |
| `serde` / `serde_json` | `1` | Serializable backend outcomes and read models. | Current backend result and query models already use it; keep using typed serializable contracts. |
| `gix` | `0.70` in backend, `0.80` legacy in app | Git transport and progress internals. | Must remain backend-only; the app dependency is a leak to remove, not a standard to preserve. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ratatui` | `0.30` | TUI rendering and harness-driven flow tests. | App-shell regression tests for CLI/TUI orchestration changes. |
| `clap` | `4` | CLI surface. | Keep CLI routing thin while backend contracts change underneath. |
| `anyhow` | `1` | App-edge error translation. | Boundary-layer conversion only; do not use it to model backend contract errors. |
| `thiserror` | `1` | Typed backend errors. | Backend contract and runtime failure modeling. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Explicit `BackendContext` | Keep `load_backend_config()` and `tool_root` helpers | Lower short-term churn, but it preserves hidden dependencies and fails LAY-03. |
| Backend-owned `RuntimeLayout` | Keep app-side `config::paths` helpers as shared runtime API | Easier migration, but it locks in duplicated layout ownership and fails D-10/D-11. |
| Forward design | Compatibility-heavy bridge layer | Fewer immediate call-site edits, but it preserves the same seam leaks under new names. |
| Narrow ports for system and package integration behavior | One mixed host trait | Smaller API surface, but it keeps generic OS effects and Spoon-owned package writes entangled. |

**Installation:**
```bash
# No new crates are recommended for Phase 1.
# Use the workspace-pinned versions already in the repo.
```

**Version verification:** Phase 1 should stay on the versions pinned in [`spoon/Cargo.toml`](/d:/projects/spoon/spoon/Cargo.toml) and [`spoon-backend/Cargo.toml`](/d:/projects/spoon/spoon-backend/Cargo.toml). The only dependency action recommended here is removing the legacy `gix` dependency from [`spoon/Cargo.toml`](/d:/projects/spoon/spoon/Cargo.toml) once app call sites no longer require it.

## Current Seam Leaks

| Leak | Evidence | Planning Implication |
|------|----------|----------------------|
| App owns backend config inputs | [`spoon/src/service/mod.rs`](/d:/projects/spoon/spoon/src/service/mod.rs#L56), [`spoon/src/service/scoop/actions.rs`](/d:/projects/spoon/spoon/src/service/scoop/actions.rs#L52), [`spoon/src/service/scoop/bucket.rs`](/d:/projects/spoon/spoon/src/service/scoop/bucket.rs#L75) | Replace `BackendConfig` loading inside service code with explicit `BackendContext` construction at the app boundary. |
| App runtime host mixes generic OS effects with Spoon package integrations | [`spoon/src/service/scoop/runtime.rs`](/d:/projects/spoon/spoon/src/service/scoop/runtime.rs#L18), [`spoon-backend/src/scoop/runtime/execution.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/execution.rs#L18) | Split host responsibilities so backend owns orchestration and app provides only narrow ports. |
| App reconstructs Scoop action state that backend already knows how to build | [`spoon/src/service/scoop/actions.rs`](/d:/projects/spoon/spoon/src/service/scoop/actions.rs#L237), [`spoon-backend/src/scoop/info.rs`](/d:/projects/spoon/spoon-backend/src/scoop/info.rs#L63) | Delete app-local action outcome rebuilding and return backend outcomes directly. |
| App rereads backend state files for status | [`spoon/src/status/discovery/probe.rs`](/d:/projects/spoon/spoon/src/status/discovery/probe.rs#L111), [`spoon/src/status/mod.rs`](/d:/projects/spoon/spoon/src/status/mod.rs#L204) | Add backend read models for status and package runtime summaries before changing UI surfaces. |
| App derives backend runtime paths locally in multiple places | [`spoon/src/config/paths.rs`](/d:/projects/spoon/spoon/src/config/paths.rs#L103), [`spoon/src/view/config.rs`](/d:/projects/spoon/spoon/src/view/config.rs#L23), [`spoon/src/packages/tool.rs`](/d:/projects/spoon/spoon/src/packages/tool.rs#L171), [`spoon/src/view/tools/detail.rs`](/d:/projects/spoon/spoon/src/view/tools/detail.rs#L342) | Centralize layout derivation in backend and convert these call sites to read-model consumers. |
| MSVC backend still depends on mutable global runtime config | [`spoon/src/service/msvc/mod.rs`](/d:/projects/spoon/spoon/src/service/msvc/mod.rs#L19), [`spoon-backend/src/msvc/mod.rs`](/d:/projects/spoon/spoon-backend/src/msvc/mod.rs#L83) | This must be removed or wrapped by explicit request-scoped context in Phase 1, otherwise LAY-03 is not met. |
| App manifest still depends on `gix` directly | [`spoon/Cargo.toml`](/d:/projects/spoon/spoon/Cargo.toml#L20), [`spoon-backend/src/gitx.rs`](/d:/projects/spoon/spoon-backend/src/gitx.rs#L1) | Plan an explicit dependency cleanup step after contract migration. |

## Architecture Patterns

### Recommended Project Structure

```text
spoon-backend/src/
├── context.rs        # BackendContext and request-scoped runtime inputs
├── layout.rs         # RuntimeLayout, ScoopLayout, MsvcLayout
├── ports.rs          # Narrow system/package ports implemented by spoon
├── gitx.rs           # internal Git implementation only
├── scoop/
│   ├── query.rs      # backend read models for runtime/package surfaces
│   ├── buckets.rs    # bucket ops using internal gitx
│   └── runtime/      # action orchestration using BackendContext + ports
└── msvc/
    ├── status.rs     # backend-owned MSVC read models
    └── ...           # explicit-context operations

spoon/src/service/
├── mod.rs            # error and event mapping only
├── scoop/            # thin request adapters
└── msvc/             # thin request adapters

spoon/src/status/     # presentation-only transformation over backend read models
spoon/src/view/       # presentation-only rendering helpers
```

### Pattern 1: `BackendContext` + `RuntimeLayout`
**What:** Introduce one backend-owned context type that carries runtime inputs and one backend-owned layout tree that derives all runtime paths from `root`.

**When to use:** Every backend operation that currently takes `tool_root`, `proxy`, test-mode flags, MSVC arch/profile, or touches runtime layout.

**Example:** Inference from [`spoon-backend/src/scoop/runtime/execution.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/execution.rs#L18), [`spoon-backend/src/scoop/paths.rs`](/d:/projects/spoon/spoon-backend/src/scoop/paths.rs#L1), and [`spoon-backend/src/msvc/paths.rs`](/d:/projects/spoon/spoon-backend/src/msvc/paths.rs#L1).
```rust
pub struct BackendContext<P> {
    pub root: PathBuf,
    pub layout: RuntimeLayout,
    pub proxy: Option<String>,
    pub test_mode: bool,
    pub msvc_target_arch: String,
    pub msvc_command_profile: String,
    pub ports: P,
}

pub struct RuntimeLayout {
    pub root: PathBuf,
    pub shims: PathBuf,
    pub scoop: ScoopLayout,
    pub msvc: MsvcLayout,
}
```

**Planning note:** `RuntimeLayout::from_root(root)` should become the only place that knows `scoop/state`, `shims`, `msvc/managed`, `msvc/official`, cache, and manifest path rules.

### Pattern 2: Split system ports from Spoon-owned package ports
**What:** Keep backend orchestration in `spoon-backend`, but pass narrow ports for app-owned behaviors instead of a mixed kitchen-sink host.

**When to use:** Scoop runtime operations that need OS mutation or Spoon-owned package integrations.

**Example:** Inference from the current mixed host in [`spoon/src/service/scoop/runtime.rs`](/d:/projects/spoon/spoon/src/service/scoop/runtime.rs#L18).
```rust
pub trait SystemPort {
    fn home_dir(&self) -> PathBuf;
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()>;
    fn ensure_process_path_entry(&self, path: &Path);
    fn remove_user_path_entry(&self, path: &Path) -> Result<()>;
    fn remove_process_path_entry(&self, path: &Path);
}

pub trait PackageIntegrationPort {
    fn supplemental_shims(&self, package: &str, current_root: &Path) -> Vec<SupplementalShimSpec>;
    async fn apply_integrations(
        &self,
        package: &str,
        current_root: &Path,
        persist_root: &Path,
        emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<BTreeMap<String, String>>;
}
```

**Planning note:** PATH and shim activation are generic runtime effects, so backend decides when they happen. Package-specific config writes stay app-owned behind `PackageIntegrationPort`.

### Pattern 3: Backend read models are the single source for status and detail
**What:** Replace app file reads and path calculations with backend query models.

**When to use:** Status screen rows, JSON status output, package detail views, runtime status summaries, bucket inventory, and package action results.

**Example:** Verified current backend read-model surface in [`spoon-backend/src/scoop/query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs#L143) and [`spoon-backend/src/scoop/info.rs`](/d:/projects/spoon/spoon-backend/src/scoop/info.rs#L176).
```rust
let runtime = spoon_backend::scoop::runtime_status(tool_root).await;
let details = spoon_backend::scoop::package_info(tool_root, package, desired_policy, desired_key).await;
```

**Planning note:** It is acceptable for `spoon` to format backend models into CLI/TUI rows. It is not acceptable for `spoon` to reread `packages/*.json`, walk cache directories for backend semantics, or reconstruct install state itself.

### Pattern 4: Keep `gitx` internal and expose backend-level repo/bucket outcomes
**What:** Preserve `gix` as a backend implementation detail and keep app-facing Git contracts on backend events and result structs only.

**When to use:** Bucket add/update flows and any future repo sync surface.

**Example:** Verified current shape in [`spoon-backend/src/gitx.rs`](/d:/projects/spoon/spoon-backend/src/gitx.rs#L16) and [`spoon-backend/src/scoop/buckets.rs`](/d:/projects/spoon/spoon-backend/src/scoop/buckets.rs#L282).
```rust
pub struct RepoSyncOutcome {
    pub head_commit: Option<String>,
    pub head_branch: Option<String>,
}

pub async fn clone_repo(
    source: &str,
    target: &Path,
    branch: Option<&str>,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<RepoSyncOutcome>;
```

**Planning note:** The app should depend on bucket outcomes and generic backend progress events, not on a generic Git API unless a real frontend use case exists.

### Pattern 5: Sequence by seam, not by domain cleanup
**What:** Plan the work in small seam slices instead of trying to clean all backend internals in one phase.

**When to use:** Phase planning and plan decomposition.

**Recommended slice order:**
1. Add `BackendContext`, `RuntimeLayout`, and narrow ports in `spoon-backend`.
2. Convert backend Scoop path helpers and runtime entry points to use context/layout.
3. Convert backend bucket APIs to consume context and keep Git internal.
4. Replace MSVC global runtime config with explicit context or request-scoped config.
5. Switch `spoon/src/service/` to thin adapters that only build requests and map events.
6. Switch `spoon/src/status/`, `spoon/src/view/`, and CLI/TUI JSON status surfaces to backend read models.
7. Remove dead app path helpers and the direct `gix` dependency from `spoon`.

### Anti-Patterns to Avoid
- **Context in name only:** adding `BackendContext` while backend code still calls `configured_tool_root()` or `load_backend_config()` internally.
- **One giant host trait:** preserving the current mixed `ScoopRuntimeHost` shape and just renaming it.
- **Split-brain status:** switching actions to backend contracts while status and detail views still reread state files in the app.
- **Phase 2 creep:** changing Scoop persisted state schema during Phase 1.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Runtime layout ownership | More app-side path helper functions | Backend-owned `RuntimeLayout` over existing backend path modules | Duplicated path rules already exist in both crates and are the main source of ownership drift. |
| Scoop action result state | App-local reconstruction from `package_current_root()` and `installed_package_states_filtered()` | Backend `ScoopPackageOperationOutcome` and `package_operation_outcome()` | The backend already owns the data and app duplication is already visible in the repo. |
| Bucket sync progress plumbing | App-side Git progress mapping | Backend `BackendEvent` and `RepoSyncOutcome` | `gitx` already converts `gix` progress into backend events; duplicating this in app code would directly violate GIT-02/GIT-03. |
| MSVC runtime configuration | Hidden mutable global plus repeated `apply_runtime_config()` calls | Explicit context or request-scoped config passed into backend operations | Global state makes tests and backend ownership ambiguous and fails LAY-03. |
| Status/detail semantics | App-side state-file parsing and path guessing | Backend query models for runtime, package, and toolchain surfaces | The app should render semantics, not rediscover them. |

**Key insight:** Phase 1 is not blocked by missing backend capability. It is blocked by duplicated contract ownership. The safest plan is to reuse and widen backend result/query primitives rather than inventing new app adapters.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Existing Scoop package state under `<root>\\scoop\\state\\packages`, bucket registry under `<root>\\scoop\\state\\buckets.json`, and MSVC state under `<root>\\msvc\\...\\state`. No rename-specific records were identified for this phase. | Code edit only. Keep current persisted shapes stable in Phase 1; do not schedule a data migration here. |
| Live service config | None identified outside repo-managed files. Bucket registration is filesystem-backed, not an external UI-only service config. | None. |
| OS-registered state | User PATH entries and shared shim availability are live runtime state, but no rename or re-registration is required for this seam refactor. | Code edit only. Validation should ensure PATH writes remain correct and non-duplicating after the contract move. |
| Secrets/env vars | Existing proxy and AI auth env vars are app configuration concerns, not a rename or migration concern for this phase. | None for migration. Preserve current names and behavior. |
| Build artifacts | `spoon.exe` at repo root and Cargo `target/` outputs will become stale after contract changes. | Rebuild artifact after implementation. No data migration required. |

## Common Pitfalls

### Pitfall 1: Context Wrapper Without Ownership Change
**What goes wrong:** A new `BackendContext` type is added, but backend code still reads config or infers layout on its own.
**Why it happens:** It is tempting to reduce signature churn by keeping helper calls in place.
**How to avoid:** Make context construction the only place where `root`, proxy, MSVC arch/profile, and test mode are read.
**Warning signs:** New functions accept `&BackendContext` and still call `configured_tool_root()`, `load_backend_config()`, or `runtime_config()`.

### Pitfall 2: Port Boundary That Still Mixes Domains
**What goes wrong:** Generic PATH and home-directory behavior stays coupled to Spoon-owned package integrations.
**Why it happens:** The current `ScoopRuntimeHost` already mixes both concerns.
**How to avoid:** Separate system effects from package integration ports, or nest them clearly under `BackendContext`.
**Warning signs:** One trait still exposes both user PATH mutation and package-specific config writes.

### Pitfall 3: Read-Model Split Brain
**What goes wrong:** Actions use backend contracts, but status and detail screens still parse state files and derive paths locally.
**Why it happens:** UI surfaces are spread across `status`, `view`, `cli/json`, and TUI background refresh code.
**How to avoid:** Plan a dedicated read-model slice and treat file IO under `spoon/src/status` and `spoon/src/view` as a blocking leak.
**Warning signs:** `read_to_string`, `scoop_state_root_from`, `msvc_root_from`, or `package_current_root` remain in app status/detail code after the refactor.

### Pitfall 4: MSVC Global State Survives the Refactor
**What goes wrong:** Scoop is converted to explicit context, but MSVC still relies on `set_runtime_config`.
**Why it happens:** MSVC already has a lot of code and the global setter feels like an easy bridge.
**How to avoid:** Plan a dedicated MSVC context slice before closing the phase.
**Warning signs:** `apply_runtime_config()` remains in `spoon/src/service/msvc/mod.rs` after Phase 1 work is otherwise "done".

### Pitfall 5: Dependency Cleanup Lands Before Contract Cleanup
**What goes wrong:** `gix` is removed from the app manifest early, but hidden app call paths still rely on service exports or tests that assume old wiring.
**Why it happens:** `spoon/src` does not currently import `gix`, so the dependency looks trivially removable.
**How to avoid:** Remove the app dependency only after contract and compile-surface cleanup is complete.
**Warning signs:** Build failures in CLI/TUI or service modules after manifest cleanup reveal undeclared seam assumptions.

## Code Examples

Verified patterns from repo sources:

### Backend-Owned Runtime Query
```rust
let status = spoon_backend::scoop::runtime_status(tool_root).await;
let bucket_count = status.runtime.bucket_count;
```
Source: [`spoon-backend/src/scoop/query.rs`](/d:/projects/spoon/spoon-backend/src/scoop/query.rs#L143)

### Backend-Owned Package Detail
```rust
let details = spoon_backend::scoop::package_info(
    tool_root,
    package_name,
    desired_policy,
    |entry| entry.key.as_str(),
).await;
```
Source: [`spoon/src/service/scoop/mod.rs`](/d:/projects/spoon/spoon/src/service/scoop/mod.rs#L87)

### Internal Git Contract Hidden Behind Backend Types
```rust
let outcome = clone_repo(source, target, Some(branch), proxy, cancel, emit).await?;
let head = outcome.head_commit;
```
Source: [`spoon-backend/src/gitx.rs`](/d:/projects/spoon/spoon-backend/src/gitx.rs#L351)

### Recommended Explicit Context Shape
Inference from backend path modules and host traits:
```rust
let ctx = BackendContext::new(root, app_inputs, ports)?;
let outcome = scoop::execute_package_action(&ctx, &request, emit).await?;
```
Source basis: [`spoon-backend/src/scoop/paths.rs`](/d:/projects/spoon/spoon-backend/src/scoop/paths.rs#L1), [`spoon-backend/src/msvc/paths.rs`](/d:/projects/spoon/spoon-backend/src/msvc/paths.rs#L1), [`spoon-backend/src/scoop/runtime/execution.rs`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/execution.rs#L18)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| App-side runtime path derivation in multiple modules | Backend-owned `RuntimeLayout` derived once from `root` | Planned for Phase 1 | Removes layout drift and satisfies LAY-01/LAY-02. |
| Hidden mutable `MsvcRuntimeConfig` global | Explicit backend context or request-scoped config | Planned for Phase 1 | Removes implicit state and makes MSVC match Scoop ownership rules. |
| App reconstructs Scoop install/action status | Backend returns action outcomes and read models | Partially available now; complete in Phase 1 | Eliminates split-brain status/detail logic. |
| App manifest carries `gix` | Backend-only `gitx` implementation | Planned for Phase 1 | Makes GIT-01 real and avoids backend implementation bleed. |

**Deprecated/outdated:**
- `spoon/src/config/paths.rs` as an app-facing backend runtime API.
- `spoon/src/service::BackendConfig` as the long-term backend contract surface.
- `spoon-backend::msvc::set_runtime_config()` as a runtime dependency mechanism.

## Open Questions

1. **Should Phase 1 add one aggregated backend status snapshot, or only domain-level read models?**
   - What we know: `spoon/src/status/`, `spoon/src/view/`, `spoon/src/cli/json.rs`, and `spoon/src/tui/background.rs` all currently depend on app-owned status collection.
   - What's unclear: whether replacing those call sites is simpler with one backend snapshot model or a small set of domain queries.
   - Recommendation: start with domain-level read models, but allow one thin backend aggregate if the app-side fan-out stays high.

2. **Should the current `ScoopRuntimeHost` evolve or be replaced?**
   - What we know: it already proves backend-owned orchestration with app-supplied hooks works.
   - What's unclear: whether backward-compatible trait surgery is worth the complexity.
   - Recommendation: replace it with clearer ports rather than preserving the mixed shape.

3. **How aggressively should Phase 1 delete app path helpers?**
   - What we know: they are widely re-exported and used in status, view, settings, and package helper code.
   - What's unclear: whether immediate deletion creates unnecessary churn.
   - Recommendation: stop using them for backend semantics in Phase 1, then delete dead exports once read-model conversion is complete.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Cargo | Build and test execution | Yes | `1.94.0` | None |
| Rust compiler | Build and test execution | Yes | `1.94.0` | None |
| Git | Repo operations and some workflow tooling | Yes | `2.53.0.windows.2` | None |

**Missing dependencies with no fallback:**
- None detected.

**Missing dependencies with fallback:**
- None detected.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` |
| Config file | none - standard Cargo test discovery |
| Quick run command | `cargo test -p spoon-backend --lib` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BNDR-01 | Scoop actions route through backend contracts, not app reconstruction. | backend integration | `cargo test -p spoon-backend scoop_action_contract_uses_context -- --nocapture` | No - Wave 0 |
| BNDR-02 | Bucket and repo work stay behind backend interfaces. | backend integration | `cargo test -p spoon-backend bucket_sync_uses_backend_git_contract -- --nocapture` | No - Wave 0 |
| BNDR-03 | MSVC detect/install uses explicit backend context. | backend unit/integration | `cargo test -p spoon-backend msvc_context_drives_status_and_install -- --nocapture` | No - Wave 0 |
| BNDR-04 | Backend derives layout from `root`; app does not. | backend unit | `cargo test -p spoon-backend runtime_layout_derives_from_root -- --nocapture` | No - Wave 0 |
| BNDR-05 | App status/detail surfaces consume backend read models only. | app CLI/TUI integration | `cargo test -p spoon json_status_uses_backend_read_models -- --nocapture` | No - Wave 0 |
| GIT-01 | `spoon` no longer depends on `gix`. | build/audit | `cargo metadata --no-deps --format-version 1 | rg '\"name\":\"gix\"'` | Yes - existing command check |
| GIT-02 | Backend owns clone/sync progress bridging. | backend integration | `cargo test -p spoon-backend clone_repo_emits_progress -- --nocapture` | Yes - existing gitx test area |
| GIT-03 | App-facing Git results contain backend types only. | backend unit | `cargo test -p spoon-backend test_repo_sync_outcome_with_values -- --nocapture` | Yes - [`spoon-backend/src/tests/gitx.rs`](/d:/projects/spoon/spoon-backend/src/tests/gitx.rs) |
| LAY-01 | One backend layout implementation covers Scoop, MSVC, and shared shims/state. | backend unit | `cargo test -p spoon-backend runtime_layout_derives_from_root -- --nocapture` | No - Wave 0 |
| LAY-02 | App owns config semantics only, not backend layout semantics. | app integration | `cargo test -p spoon tui_tool_detail_uses_backend_layout_model -- --nocapture` | No - Wave 0 |
| LAY-03 | Backend operations run with explicit context, not globals. | backend unit/integration | `cargo test -p spoon-backend explicit_context_required_for_runtime_ops -- --nocapture` | No - Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p spoon-backend --lib`
- **Per wave merge:** `cargo test -p spoon-backend && cargo test -p spoon --test json_flow && cargo test -p spoon --test tui_scoop_action_flow`
- **Phase gate:** `cargo test --workspace`

### Wave 0 Gaps

- [ ] `spoon-backend/src/tests/context.rs` - covers BNDR-04, GIT-01, LAY-01, LAY-03.
- [ ] `spoon-backend/src/scoop/tests/contracts.rs` - covers BNDR-01, BNDR-02, GIT-02, GIT-03.
- [ ] `spoon-backend/src/msvc/tests/context.rs` - covers BNDR-03 and the removal of `set_runtime_config`.
- [ ] `spoon/tests/cli/status_backend_flow.rs` - covers BNDR-05 and LAY-02 at the app shell boundary.

## Sources

### Primary (HIGH confidence)
- `.planning/phases/01-backend-seams-and-ownership/01-CONTEXT.md` - locked decisions and phase boundary.
- `.planning/ROADMAP.md` - phase goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - requirement IDs and phase mapping.
- `AGENTS.md` - repo-specific ownership, path, and testing rules.
- `CLAUDE.md` - project-level constraints and refactor safety instructions.
- `spoon/src/service/mod.rs` - current app-owned backend config and host ports.
- `spoon/src/service/scoop/actions.rs` - root/proxy inference and duplicated Scoop action state reconstruction.
- `spoon/src/service/scoop/runtime.rs` - current mixed host implementation.
- `spoon/src/service/msvc/mod.rs` - current MSVC global-runtime bridge.
- `spoon/src/status/mod.rs` and `spoon/src/status/discovery/probe.rs` - current app-side status and state-file reads.
- `spoon/src/packages/tool.rs`, `spoon/src/view/config.rs`, `spoon/src/view/tools/detail.rs` - app-side layout and detail path derivation.
- `spoon-backend/src/scoop/paths.rs`, `spoon-backend/src/msvc/paths.rs` - existing backend layout rules.
- `spoon-backend/src/scoop/query.rs` and `spoon-backend/src/scoop/info.rs` - backend read models and package outcome semantics.
- `spoon-backend/src/scoop/runtime/execution.rs` and `spoon-backend/src/scoop/runtime/actions.rs` - current backend runtime host orchestration shape.
- `spoon-backend/src/scoop/buckets.rs` and `spoon-backend/src/gitx.rs` - backend-owned bucket and Git contract shape.
- `spoon-backend/src/msvc/mod.rs` - current MSVC runtime config and backend contract surface.
- `spoon/src/tui/test_support.rs`, `spoon/tests/tui/tui_scoop_action_flow.rs`, `spoon-backend/tests/scoop_integration.rs`, `spoon-backend/src/tests/gitx.rs` - existing validation patterns.

### Secondary (MEDIUM confidence)
- None. This research did not need external sources because the phase is defined by current repo contracts and code ownership.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all recommendations stay on workspace-pinned crates and existing internal modules.
- Architecture: HIGH - based on direct code evidence from both crates and the locked phase decisions.
- Pitfalls: MEDIUM - the major leaks are clear, but some secondary app call sites may still emerge during implementation.

**Research date:** 2026-03-28
**Valid until:** 2026-04-27
