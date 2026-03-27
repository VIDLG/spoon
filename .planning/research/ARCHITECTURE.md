# Architecture Patterns

**Domain:** Windows developer-tool bootstrapper with Rust CLI/TUI frontend and backend runtime crates
**Project:** spoon
**Researched:** 2026-03-28

## Recommended Architecture

`spoon` should become a thin application shell. `spoon-backend` should become the only place that knows how Spoon-managed runtimes are laid out on disk, how Scoop packages are resolved and installed, how bucket repositories are synced, and how lifecycle state is persisted. The frontend crate should own config loading, command routing, TUI/CLI presentation, and app-owned policy/config side effects only.

The clean split is:

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `spoon/src/cli/*`, `spoon/src/tui/*` | Parse commands, drive views, render progress/output, collect user intent | `spoon/src/app/*` |
| `spoon/src/app/*` | Application orchestration layer only; translate config into backend requests; adapt backend events to UI stream | `spoon-backend` public API, host adapters |
| `spoon/src/config/*` | App-owned config read/write for Spoon itself, Git identity, Claude/Codex settings, skill install paths | `spoon/src/app/*`, host adapters |
| `spoon/src/packages/*` | App-owned policy integration glue only; no Scoop lifecycle orchestration | `spoon/src/app/host.rs` |
| `spoon-backend/src/layout.rs` | Canonical tool-root layout for all domains | `spoon-backend/src/scoop/*`, `spoon-backend/src/msvc/*` |
| `spoon-backend/src/git/*` | Repository sync operations backed by `gix` | `spoon-backend/src/scoop/catalog/*` |
| `spoon-backend/src/scoop/*` | Scoop catalog, lifecycle, persisted state, projections, host integration ports | `spoon`, `git`, shared layout |
| `spoon-backend/src/msvc/*` | MSVC managed/official backend logic | shared layout, optional shared host/context utilities |

The main design rule is simple: if code needs to know the Scoop layout under the configured root, parse a Scoop manifest, mutate a Scoop install, sync a bucket git repo, or persist Scoop state, it belongs in `spoon-backend`. If code only decides when to invoke that behavior, how to display it, or how to edit app-owned config files, it belongs in `spoon`.

### Target Repo Boundary

The current split is still backwards in a few places:

- `spoon/src/service/scoop/actions.rs`
- `spoon/src/service/scoop/bucket.rs`
- `spoon/src/service/scoop/report.rs`
- `spoon/src/service/scoop/runtime.rs`
- `spoon/src/config/paths.rs`

Those files still know too much about backend execution and backend layout. After the refactor, `spoon` should not compute Scoop paths, should not construct Scoop package action plans, should not re-read Scoop state files, and should not host the Scoop runtime trait implementation inside a Scoop-specific service module.

The target call path should be:

```text
CLI/TUI -> spoon app command -> backend request/context -> spoon-backend use case
       -> backend emits typed progress/events -> spoon renders
       -> backend returns typed outcome/read model -> spoon renders
```

### Target `spoon-backend/src/scoop` Structure

Do not keep the current `runtime/` directory as the catch-all center of gravity. It mixes lifecycle orchestration, command surface generation, persisted state, policy integration, and host/platform behavior. Split it by responsibility instead.

Recommended structure:

```text
spoon-backend/src/scoop/
  mod.rs
  api.rs
  context.rs
  host.rs
  catalog/
    mod.rs
    buckets.rs
    manifest_doc.rs
    manifest_store.rs
    resolver.rs
    search.rs
  lifecycle/
    mod.rs
    planner.rs
    install.rs
    uninstall.rs
    reapply.rs
    acquire.rs
    hooks.rs
    persist.rs
    surface.rs
  state/
    mod.rs
    model.rs
    store.rs
    projections.rs
  doctor.rs
```

Mapping from current files:

| Current file | Target destination | Why |
|--------------|--------------------|-----|
| `spoon-backend/src/scoop/buckets.rs` | `catalog/buckets.rs` plus `catalog/resolver.rs` | Bucket registry and manifest lookup are catalog concerns |
| `spoon-backend/src/scoop/manifest.rs` | `catalog/manifest_doc.rs`, `catalog/search.rs`, `catalog/manifest_store.rs` | Typed document parsing should not also pretend to be search |
| `spoon-backend/src/scoop/query.rs` | `state/projections.rs` and `catalog/search.rs` | Query/read models should be derived from state plus catalog |
| `spoon-backend/src/scoop/info.rs` | `state/projections.rs` | Package info is a read model, not a state source |
| `spoon-backend/src/scoop/package_state.rs` | delete after consolidation into `state/model.rs` and `state/store.rs` | It is a second persisted state model with no meaningful live integration |
| `spoon-backend/src/scoop/runtime/actions.rs` | `lifecycle/install.rs`, `lifecycle/uninstall.rs`, `lifecycle/reapply.rs` | Current file is the monolith to split first |
| `spoon-backend/src/scoop/runtime/installed_state.rs` | `state/store.rs` | State persistence should not live inside runtime |
| `spoon-backend/src/scoop/runtime/source.rs` | `lifecycle/planner.rs` or `catalog/resolver.rs` | Manifest-to-install selection is planning logic |
| `spoon-backend/src/scoop/runtime/integration.rs` | `host.rs` plus `lifecycle/reapply.rs` | Host integration is a port, not runtime state |
| `spoon-backend/src/scoop/runtime/surface.rs` | `lifecycle/surface.rs` | Shims and shortcuts are lifecycle outputs |
| `spoon-backend/src/scoop/runtime/persist.rs` | `lifecycle/persist.rs` | Persist restore/sync is lifecycle behavior |

`spoon-backend/src/scoop/mod.rs` should shrink to a public facade that re-exports a stable backend API, not a dumping ground for internal types.

### One Canonical Scoop State Model

The highest-priority duplication to remove is persisted package state. Today there are three overlapping concepts:

- `spoon-backend/src/scoop/package_state.rs` defines `ScoopPackageState`
- `spoon-backend/src/scoop/runtime/installed_state.rs` defines `InstalledPackageState`
- `spoon-backend/src/scoop/info.rs` defines `ScoopPackageInstallState`

Only one of these should be canonical and persisted: an installed package record under `spoon-backend/src/scoop/state/model.rs`.

Recommended persisted record:

```rust
pub struct InstalledPackageRecord {
    pub package: String,
    pub bucket: String,
    pub version: String,
    pub architecture: Option<String>,
    pub manifest_relative_path: String,
    pub cache_size_bytes: Option<u64>,
    pub command_surface: CommandSurfaceRecord,
    pub environment: EnvironmentRecord,
    pub persist_entries: Vec<PersistEntry>,
    pub policy_integrations: BTreeMap<String, String>,
    pub lifecycle: LifecycleScriptsRecord,
}
```

Important rules:

- Persist only facts required for uninstall, reapply, and reporting.
- Do not persist derivable absolute paths like current install root or shims root; derive those from backend layout.
- Do not persist UI-specific outcome DTOs.
- Do not keep a second mini-state file at `scoop/state/<pkg>.json`; remove `spoon-backend/src/scoop/package_state.rs`.
- Keep read models separate. `PackageInfoView`, `InstalledPackageSummary`, and operation outcomes should be projections derived from `InstalledPackageRecord` plus current manifest/layout state.

This lets `info`, `list`, `prefix`, uninstall, and reapply all depend on the same truth source.

### Paths and Layout

`spoon/src/config/paths.rs` and `spoon-backend/src/scoop/paths.rs` currently duplicate root/layout knowledge. That should stop. Root selection remains app config, but all derived paths should move behind a backend-owned layout type.

Recommended shared backend layout:

```rust
pub struct BackendLayout {
    pub root: PathBuf,
}

impl BackendLayout {
    pub fn scoop(&self) -> ScoopLayout { ... }
    pub fn msvc(&self) -> MsvcLayout { ... }
    pub fn shims_root(&self) -> PathBuf { ... }
}
```

Then:

- `spoon` reads `root` from `spoon/src/config/*`
- `spoon` passes `root` into backend context
- `spoon-backend/src/layout.rs` owns all derived layout rules
- `spoon/src/config/paths.rs` stops exposing Scoop and MSVC layout helpers except app-owned config file locations under the user home directory

That change matters beyond Scoop. It also removes the current leak where `spoon` still knows things like `scoop_git_usr_bin_from()` and `shims_root_from()`.

### Runtime vs Host Integration

Separate these two concerns cleanly:

1. Runtime lifecycle inside `spoon-backend`
2. App-owned host integration hooks supplied by `spoon`

Target host boundary:

```rust
pub trait ScoopHost {
    fn test_mode_enabled(&self) -> bool;
    fn home_dir(&self) -> PathBuf;
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()>;
    fn remove_user_path_entry(&self, path: &Path) -> Result<()>;
    fn ensure_process_path_entry(&self, path: &Path);
    fn remove_process_path_entry(&self, path: &Path);

    async fn apply_package_policy(
        &self,
        package_name: &str,
        current_root: &Path,
        persist_root: &Path,
        emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<BTreeMap<String, String>>;
}
```

Rules for this boundary:

- `spoon-backend` owns lifecycle sequencing.
- `spoon-backend` decides when to call host policy hooks.
- `spoon` only implements the host trait with app-owned config logic from `spoon/src/packages/*`.
- `spoon` must not wrap backend lifecycle entry points in its own Scoop-specific service orchestration anymore.
- Helper display functions like the current pip-mirror rendering should move out of backend traits. The backend should return raw applied values; frontend can pretty-print them if needed.

Also split system integration from package-policy integration where practical. PATH and home-directory behavior are platform concerns. Package-specific config mutation is Spoon policy glue. They should not be mixed in the same module like `spoon/src/service/scoop/runtime.rs`.

### Package Lifecycle Separation

The lifecycle stack inside `spoon-backend/src/scoop/lifecycle/*` should be explicit:

| Layer | Responsibility |
|-------|----------------|
| `planner.rs` | Resolve manifest, selected source, dependency list, and action kind into an executable plan |
| `acquire.rs` | Download/copy payloads, cache reuse, hash verification |
| `install.rs` | Extract/materialize payloads, run install hooks, write command surface, write state |
| `uninstall.rs` | Read installed record, run uninstall hooks, sync persist, remove command surface, remove state |
| `reapply.rs` | Rebuild command surface or policy integrations from existing installed record |
| `surface.rs` | Create/remove shims and shortcuts only |
| `persist.rs` | Sync/restore persisted files only |
| `hooks.rs` | Hook execution only |

Current `spoon-backend/src/scoop/runtime/actions.rs` should become the orchestration entry point that calls these smaller units, then disappear once the split is complete.

## Data Flow

Recommended install flow:

```text
spoon app command
  -> build ScoopContext { layout, proxy, action }
  -> spoon-backend::scoop::api::execute_package_action(ctx, host, request)
  -> catalog resolves bucket + manifest
  -> lifecycle planner builds InstallPlan
  -> lifecycle executor installs dependencies, downloads payloads, extracts, restores persist
  -> surface writes shims/shortcuts
  -> host applies Spoon-owned policy integrations
  -> state store writes InstalledPackageRecord
  -> projections build typed outcome
  -> spoon renders outcome
```

Recommended read flow:

```text
spoon command
  -> backend projection query
  -> state store loads InstalledPackageRecord
  -> catalog loads manifest if needed
  -> projections assemble info/list/status DTO
  -> spoon renders
```

The key point is that reads should not deserialize arbitrary raw JSON values ad hoc. `spoon-backend/src/scoop/info.rs` currently re-reads installed state as `serde_json::Value`. That should be replaced with typed projections from the canonical record.

## Patterns to Follow

### Pattern 1: Backend Context, Not Global Runtime State

**What:** Pass explicit context into backend calls instead of storing mutable global backend config.

**When:** Always, for Scoop immediately and for MSVC as the first shared cleanup when touched.

**Example:**

```rust
pub struct ScoopContext {
    pub layout: BackendLayout,
    pub proxy: String,
}

pub async fn execute_package_action(
    ctx: &ScoopContext,
    host: &dyn ScoopHost,
    request: PackageActionRequest,
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<PackageActionOutcome> { ... }
```

**Why:** This removes the pattern currently visible in `spoon/src/service/msvc/mod.rs` and `spoon-backend/src/msvc/mod.rs`, where runtime behavior is driven by global mutable config. Scoop should not adopt that pattern, and MSVC should eventually leave it.

### Pattern 2: Stable Backend Facade, Internal Module Freedom

**What:** Keep `spoon-backend/src/scoop/mod.rs` as the compatibility facade while internals are moved.

**When:** During the brownfield migration.

**Why:** It allows the first refactor phase to change structure aggressively without breaking every frontend call site at once.

### Pattern 3: State Store Plus Projections

**What:** One state store writes canonical records; separate projection functions build info/list/status/action DTOs.

**When:** For all Scoop read operations.

**Why:** This removes the current spread of duplicated install-state structs and raw JSON probing.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Frontend Re-Orchestrating Backend Work

**What:** `spoon/src/service/scoop/*` calling multiple backend helpers and reconstructing action outcomes.

**Why bad:** It leaves lifecycle knowledge in the wrong crate and makes the frontend a second backend.

**Instead:** `spoon` should call one backend use case per command and render its result.

### Anti-Pattern 2: Sync Tokio Runtime Construction Inside Business Logic

**What:** `spoon-backend/src/scoop/planner.rs` currently creates a current-thread Tokio runtime to resolve manifests synchronously.

**Why bad:** It hides async boundaries, makes testing harder, and spreads runtime assumptions across layers.

**Instead:** Make planning async inside backend. Keep sync wrappers, if any, only at the outermost app adapter boundary.

### Anti-Pattern 3: Mixed Domain and Transport Types

**What:** Persisted state, read DTOs, and UI outcome types are mixed together in the current Scoop module.

**Why bad:** Refactors become state-schema changes even when only UI output changes.

**Instead:** Separate:

- domain records
- use-case requests/responses
- UI formatting

### Anti-Pattern 4: `gix` in More Than One Crate

**What:** `spoon-backend/Cargo.toml` and `spoon/Cargo.toml` both currently depend on `gix`, and on different versions.

**Why bad:** It duplicates a backend-only concern and creates needless version skew.

**Instead:** Keep all git repository sync behind `spoon-backend/src/git/*`. `spoon` should remove its direct `gix` dependency.

## Git / `gix` Placement

`gix` belongs only in `spoon-backend`.

Concrete recommendation:

- Rename `spoon-backend/src/gitx.rs` to `spoon-backend/src/git/mod.rs`
- Keep a tiny backend-local API such as `sync_repo()` or `clone_or_replace_repo()`
- Let `spoon-backend/src/scoop/catalog/buckets.rs` depend on that API, not on `gix` directly
- Remove direct `gix` from `spoon/Cargo.toml`

Reasoning:

- Bucket sync is backend infrastructure, not UI behavior.
- `gix` progress already maps cleanly into backend events in `spoon-backend/src/gitx.rs`.
- The frontend should only see backend progress events and sync outcomes, never VCS implementation details.

## What MSVC Should Keep vs Defer

Keep in scope only the minimum MSVC alignment needed to support the Scoop refactor well.

Keep now:

- `spoon-backend/src/msvc/*` remains backend-owned
- shared root/layout abstraction should be introduced in backend and consumed by MSVC when touched
- explicit backend context should replace global mutable runtime config when a touched path requires it

Defer until after Scoop phase 1:

- large internal breakup of `spoon-backend/src/msvc/mod.rs`
- unifying Scoop and MSVC under one fake common package-lifecycle abstraction
- deep changes in `spoon-backend/src/msvc/official.rs` unless a shared context/layout extraction forces a small edit

The right sequencing is Scoop-first. MSVC is parallel domain logic, not the driver for this architecture.

## Build-Order Implications

The brownfield refactor should be staged in this order:

1. **Create stable backend seams first**
   - Add `spoon-backend/src/layout.rs`
   - Add `spoon-backend/src/scoop/context.rs`
   - Add `spoon-backend/src/scoop/host.rs`
   - Keep existing `spoon-backend/src/scoop/mod.rs` exports alive

2. **Consolidate Scoop state before moving lifecycle logic**
   - Introduce `InstalledPackageRecord` and `state/store.rs`
   - Migrate `info`, `query`, uninstall, and reapply to read only the new record
   - Delete `spoon-backend/src/scoop/package_state.rs`

3. **Extract catalog from lifecycle**
   - Split bucket registry, manifest resolution, and search out of current mixed files
   - Move git sync behind `spoon-backend/src/git/*`
   - Fix `search_manifests_async()` so search has one real implementation

4. **Split lifecycle monolith**
   - Carve `runtime/actions.rs` into install/uninstall/reapply/acquire/surface/persist
   - Keep behavior stable while moving files

5. **Collapse frontend Scoop services into app adapters**
   - Replace `spoon/src/service/scoop/*` logic with thin adapter calls
   - Move any remaining host implementation to a generic app host module, not a Scoop runtime submodule

6. **Only then prune compatibility exports**
   - Once `spoon` is thin and tests are green, reduce `spoon-backend/src/scoop/mod.rs` surface

Why this order:

- state consolidation removes the largest rewrite risk first
- catalog extraction isolates `gix` and manifest lookup early
- lifecycle split is safer once state and catalog are stable
- frontend thinning last avoids a temporary explosion of moving parts on both crates at once

## Scalability Considerations

| Concern | Current scale | After Scoop refactor | Later growth |
|---------|---------------|----------------------|--------------|
| Backend ownership clarity | blurred between `spoon` and `spoon-backend` | backend owns all Scoop lifecycle and git sync | same rule extends cleanly to new runtime domains |
| State schema evolution | risky because models are duplicated | one canonical installed record | add fields once, project many read models |
| Package query behavior | mixed state and manifest reads | projection layer composes state plus catalog | search/status/info can evolve independently |
| Testability | monolithic action flow files | install/uninstall/reapply units are directly testable | domain-specific regression tests stay focused |

## Sources

Local codebase:

- `spoon-backend/src/scoop/mod.rs`
- `spoon-backend/src/scoop/buckets.rs`
- `spoon-backend/src/scoop/info.rs`
- `spoon-backend/src/scoop/package_state.rs`
- `spoon-backend/src/scoop/query.rs`
- `spoon-backend/src/scoop/planner.rs`
- `spoon-backend/src/scoop/paths.rs`
- `spoon-backend/src/scoop/runtime/actions.rs`
- `spoon-backend/src/scoop/runtime/installed_state.rs`
- `spoon-backend/src/scoop/runtime/integration.rs`
- `spoon-backend/src/scoop/runtime/source.rs`
- `spoon-backend/src/scoop/runtime/surface.rs`
- `spoon-backend/src/gitx.rs`
- `spoon-backend/src/msvc/mod.rs`
- `spoon-backend/src/msvc/official.rs`
- `spoon/src/service/mod.rs`
- `spoon/src/service/scoop/mod.rs`
- `spoon/src/service/scoop/actions.rs`
- `spoon/src/service/scoop/bucket.rs`
- `spoon/src/service/scoop/report.rs`
- `spoon/src/service/scoop/runtime.rs`
- `spoon/src/config/paths.rs`
- `spoon/src/packages/mod.rs`
- `spoon/Cargo.toml`
- `spoon-backend/Cargo.toml`

Primary sources:

- Scoop folder layout wiki: https://github.com/ScoopInstaller/Scoop/wiki/Scoop-Folder-Layout
- Scoop app manifest wiki: https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests
- `gix` crate documentation: https://docs.rs/gix/latest/gix/
