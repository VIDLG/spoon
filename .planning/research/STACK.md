# Technology Stack

**Project:** spoon
**Researched:** 2026-03-28
**Focus:** brownfield Rust backend refactor, with `spoon-backend/src/scoop` as the first major target
**Overall confidence:** HIGH for workspace/dependency strategy, MEDIUM for exact upgrade sequencing

## Recommended Stack

### Core Workspace

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Cargo workspace in `Cargo.toml` | `resolver = "3"` | Single lockfile, shared dependency policy, coordinated refactor | Cargo workspaces share one `Cargo.lock` at the root, and workspace inheritance is the right tool for eliminating version skew during a brownfield split. Keep the current workspace shape and expand it instead of adding more crates. |
| `[workspace.package]` in `Cargo.toml` | current repo standard | Centralize `edition`, `version`, and later `rust-version` | The root workspace already owns structure but not package metadata. Centralizing metadata removes drift between `spoon/Cargo.toml`, `spoon-backend/Cargo.toml`, and `xtask/Cargo.toml`. |
| `[workspace.dependencies]` in `Cargo.toml` | current Cargo feature | Shared versions for duplicated crates | This repo currently duplicates core versions across `spoon/Cargo.toml` and `spoon-backend/Cargo.toml`, including `gix` and `msi`. Workspace dependency inheritance is the cleanest fix. |
| Existing crate split: `spoon`, `spoon-backend`, `xtask` | keep | Frontend app, backend library, repo automation | The current three-crate layout is enough. Do not add a fourth crate for "shared types" during this refactor. Put canonical backend models in `spoon-backend` and let `spoon` consume them directly. |

### Backend Crate

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `spoon-backend` | keep as sole backend crate | Git, Scoop, MSVC, runtime actions, canonical state models | This matches the desired architecture and is already partially true in `spoon-backend/src/gitx.rs` and most of `spoon-backend/src/scoop`. The refactor should finish the move instead of wrapping backend logic from `spoon`. |
| `gix` | upgrade and unify on `0.80.x` | All Git and bucket sync work | `spoon-backend/Cargo.toml` is still on `0.70`, while `spoon/Cargo.toml` pulls `0.80`. That split explodes the graph in `cargo tree -d`. Move all Git responsibility into `spoon-backend`, pin one `gix` line there, and remove direct `gix` from `spoon`. |
| `reqwest` | upgrade and unify on `0.13.x` | Backend HTTP client for Git transport and runtime downloads | Current `gix 0.80` already pulls `reqwest 0.13.2`, while both crates still declare `reqwest 0.12`. Unifying backend HTTP on `0.13.x` removes a major duplicate branch and aligns with the current upstream line. Keep `default-features = false` and `rustls` only. |
| `tokio` | keep `1.x` | Async runtime, fs, process, task orchestration | Already used pervasively in both crates. Keep one workspace version and narrow feature sets per crate. |
| `serde` + `serde_json` | keep `1.x` | Canonical backend models and persisted state | The highest-value refactor target is repeated state models. Backend-owned DTOs should be serializable once and consumed directly by the app. |
| `thiserror` | keep, but only where used | Backend error types | `spoon-backend` uses it; `spoon` appears not to. Remove it from `spoon` unless a compile pass proves otherwise. |
| `msi` | upgrade and unify on `0.10.x` | MSI parsing in backend MSVC flows | `spoon-backend/Cargo.toml` is on `0.8`, while `spoon/Cargo.toml` already pins `0.10`. Move MSI ownership fully into `spoon-backend` and upgrade there to collapse the duplicate `cfb` branch. |
| `cab`, `zip`, `sha1`, `sha2`, `mslnk` | keep backend-owned | Archive extraction, checksum verification, shortcut generation | These are implementation dependencies of runtime actions. They belong in `spoon-backend`, not in the app crate. |

### Frontend Crate

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `spoon` | keep | CLI/TUI orchestration, config forms, output formatting, app glue | `spoon` should remain the executable-facing crate only. It should load config, call backend APIs, translate backend events to CLI/TUI output, and own app-specific package/config presentation. |
| `clap`, `ratatui`, `crossterm`, `tui-logger` | keep in `spoon` only | CLI/TUI UX | These are clearly frontend-only and should not leak into `spoon-backend`. |
| `anyhow` | keep in `spoon` only | UX-facing error composition | Useful at the app boundary. Do not use it for backend core domain errors. |
| `toml`, `toml_edit`, `winreg`, `arboard`, `syntect`, `indicatif`, `regex` | keep in `spoon` only | Config IO, Windows UX, terminal presentation | These are app concerns, not backend concerns. |

## Boundary Decisions

### Backend/App Boundary For Git And Runtime Responsibilities

**Put these in `spoon-backend`:**

| Responsibility | Where it should live | Evidence in repo | Recommendation |
|----------------|----------------------|------------------|----------------|
| Git clone/fetch/bucket sync | `spoon-backend` only | `spoon-backend/src/gitx.rs`, `spoon-backend/src/scoop/buckets.rs` | Keep `gix` and all Git-facing logic here. `spoon` should not depend on `gix` directly at all. |
| Scoop manifests, planners, install/uninstall execution, persisted package state | `spoon-backend` only | `spoon-backend/src/scoop/*` | Make backend models authoritative. Remove mirrored app-side wrappers where they only rename or repackage backend data. |
| Runtime file mutations under tool root, shims, persisted runtime state, archive extraction, checksum verification | `spoon-backend` only | `spoon-backend/src/scoop/runtime/*`, `spoon-backend/src/msvc/*` | These are backend actions, not UI behavior. |
| Public backend result/event types | `spoon-backend` only | `spoon-backend/src/lib.rs`, `spoon-backend/src/event.rs` | Expand these instead of introducing new app-side copies. |

**Keep these in `spoon`:**

| Responsibility | Where it should live | Evidence in repo | Recommendation |
|----------------|----------------------|------------------|----------------|
| CLI argument parsing, TUI state, streaming output, help text | `spoon` only | `spoon/src/cli/*`, `spoon/src/tui/*` | Pure frontend. |
| App-owned config loading and persistence | `spoon` only | `spoon/src/config/*`, `spoon/src/packages/*` | `spoon` owns user-facing config for Git identity/default branch, Claude, Codex, and editor setup. |
| App-specific integration glue | `spoon` only, but as a thin adapter | `spoon/src/service/scoop/runtime.rs` | Keep only the glue that interprets app policy and config. The backend should accept a narrow host/context interface. |

**Prescriptive boundary call:** Git ownership is already pointing the right direction in `spoon-backend/src/gitx.rs`. Finish that move. Runtime ownership should follow the same rule: `spoon` may provide config and host callbacks, but the action graph, filesystem mutations, and state persistence must live in `spoon-backend`.

### Runtime Injection Strategy

Use one explicit backend context or host interface per domain, not global mutable config.

- `spoon/src/service/msvc/mod.rs` currently pushes app config into `spoon-backend` through `set_runtime_config(...)`.
- `spoon/src/service/scoop/runtime.rs` currently implements a `ScoopRuntimeHost` that still carries app-owned policy and PATH mutation hooks.

**Recommendation:** keep the Scoop host pattern, but narrow it. For both Scoop and MSVC, pass an explicit config snapshot plus optional host callbacks from `spoon` into backend entrypoints. Do not expand the `OnceLock<RwLock<...>>` pattern from `spoon-backend/src/msvc/mod.rs` into the rest of the backend.

## Dependency Consolidation Strategy

### Immediate Consolidation

1. Move duplicated foundational versions into `[workspace.dependencies]` in `Cargo.toml`.
2. Convert `spoon/Cargo.toml` and `spoon-backend/Cargo.toml` to `workspace = true` for shared crates.
3. Remove backend-implementation crates from `spoon/Cargo.toml`.
4. Keep app-only dependencies local to `spoon/Cargo.toml`.
5. Keep backend-only dependencies local to `spoon-backend/Cargo.toml`.

### Shared Dependencies That Should Be Workspace-Managed

| Dependency | Recommendation | Why |
|------------|----------------|-----|
| `tokio` | workspace-managed | Shared runtime, already used in both crates |
| `tokio-util` | workspace-managed | Shared cancellation/event plumbing |
| `serde` | workspace-managed | Shared backend models |
| `serde_json` | workspace-managed | Shared persistence and CLI JSON |
| `tracing` | workspace-managed | Shared instrumentation |
| `thiserror` | workspace-managed only if still used in both after cleanup | Otherwise keep only in backend |
| `dirs`, `walkdir`, `which` | workspace-managed if they remain in both crates after cleanup | They are currently used in both crates |

### Dependencies To Remove From `spoon/Cargo.toml`

These either belong in the backend or appear unused in app production code after local inspection:

- `gix`
- `reqwest`
- `cab`
- `msi`
- `mslnk`
- `sha1`
- `async-recursion`
- `fs-err`
- `thiserror`
- `base64`
- `bytesize`
- `colored_json`
- `fs_extra`
- `unicode-width`

### Dependencies To Move To `spoon` Dev-Dependencies

- `zip`
- `sha2`

Reason: local usage shows them in `spoon/tests/*`, not in `spoon/src/*`.

### Dependencies To Keep In `spoon-backend/Cargo.toml`

- `gix`
- `reqwest`
- `msi`
- `cab`
- `zip`
- `mslnk`
- `sha1`
- `sha2`
- `async-recursion`
- `fs-err`

Reason: these are backend implementation details visible in files like `spoon-backend/src/gitx.rs`, `spoon-backend/src/msvc/msi_extract.rs`, `spoon-backend/src/scoop/runtime/download.rs`, and `spoon-backend/src/scoop/runtime/surface.rs`.

## Cargo Workspace Strategy

### Recommended Shape

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Workspace topology | Keep `spoon`, `spoon-backend`, `xtask` | Add `spoon-types` or `spoon-core` now | Another crate would spread the duplicate state problem across more boundaries instead of solving it. |
| Lockfile | One root `Cargo.lock` in `Cargo.toml` workspace root | Per-member lockfiles such as `spoon/Cargo.lock` | Cargo workspaces already share one root lockfile. Keep one source of truth. |
| Shared deps | `[workspace.dependencies]` | Hand-managed duplicate versions in each crate | The repo already shows why this fails: `gix 0.70` vs `0.80`, `msi 0.8` vs `0.10`. |
| Git stack | One `gix` dependency in `spoon-backend` | `gix` in both crates, or switching to `git2` | Two Git stacks are worse than one. `git2` would add native dependency complexity for no refactor benefit. |
| HTTP stack | One backend-owned `reqwest` line | Mixed explicit `reqwest 0.12` plus transitive `0.13` | This keeps duplicate TLS/client trees alive unnecessarily. |

### Suggested Manifest Shape

```toml
[workspace]
members = ["spoon", "spoon-backend", "xtask"]
default-members = ["spoon"]
resolver = "3"

[workspace.package]
edition = "2024"
version = "0.1.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = "1"
tokio-util = "0.7"
tracing = "0.1"
dirs = "6"
walkdir = "2"
which = "8"
```

Then keep backend-specific lines in `spoon-backend/Cargo.toml` and app-specific lines in `spoon/Cargo.toml`.

## What To Keep, Upgrade, Remove

### Keep

| Library / Tool | Action | Why |
|----------------|--------|-----|
| `gix` | keep as the only Git implementation | Already integrated in `spoon-backend/src/gitx.rs`; pure Rust; no reason to add `git2`. |
| `tokio` | keep | Already the runtime backbone in both crates. |
| `serde` / `serde_json` | keep | Essential for canonical state models. |
| `xtask` | keep | Small and appropriate for workspace automation. |
| `ratatui` test harness strategy | keep in `spoon` | Matches repo testing direction and AGENTS guidance. |

### Upgrade During This Refactor

| Library | From | To | Why |
|---------|------|----|-----|
| `gix` in `spoon-backend/Cargo.toml` | `0.70` | `0.80.x` | Collapse the split with `spoon/Cargo.toml`, then remove `gix` from `spoon`. |
| `msi` in `spoon-backend/Cargo.toml` | `0.8` | `0.10.x` | Collapse the split with `spoon/Cargo.toml`. |
| `reqwest` in backend | `0.12` | `0.13.x` | Align explicit backend HTTP with the current upstream line already pulled transitively by `gix 0.80`. |

### Remove

| Library / Tool | Remove From | Why |
|----------------|-------------|-----|
| direct `gix` dependency | `spoon/Cargo.toml` | `spoon` does not directly import `gix`; Git should live only in backend. |
| direct `reqwest` dependency | `spoon/Cargo.toml` | No direct app usage found. |
| direct `msi`, `cab`, `mslnk`, `sha1` dependencies | `spoon/Cargo.toml` | These are backend implementation crates. |
| duplicate command/result wrappers where not app-specific | `spoon/src/service/*` | Prefer backend-owned models unless the app is adding presentation-only data. |
| second persisted Scoop state model | `spoon-backend/src/scoop/package_state.rs` or `spoon-backend/src/scoop/runtime/installed_state.rs` after redesign | There should be one canonical persisted package/runtime state model in backend, not two overlapping ones. |

## What Not To Use During This Refactor

| Anti-Pattern / Tool | Why Avoid |
|---------------------|-----------|
| `git2` / `libgit2` | Adds a second Git stack and native dependency burden while `gix` is already working in `spoon-backend/src/gitx.rs`. |
| New "shared types" crate | The current problem is model duplication, not crate scarcity. Another crate will harden bad boundaries. |
| `cargo-hakari` / workspace-hack crates | This workspace is small. Extra workspace-hack machinery is not justified for `spoon`, `spoon-backend`, and `xtask`. |
| More global runtime state like `set_runtime_config(...)` | Harder to test and reason about than explicit config snapshots passed into backend entrypoints. |
| Compatibility shims that preserve bad abstractions | The stated goal is forward design. Delete wrappers that only relay backend calls without adding app-specific behavior. |
| Backend logic in `spoon/src/service/*` | This recreates the split-brain architecture the refactor is trying to remove. |
| Feature-bloating `gix` with `max-performance-safe` | Current upstream docs mark `max-performance-safe` as deprecated. Do not carry it forward when consolidating `gix`. |
| `native-tls` for `reqwest` | Keep Rustls-only networking for deterministic backend behavior and to avoid extra Windows TLS/platform complexity during the refactor. |

## Installation

```bash
# After moving shared versions into [workspace.dependencies]
cargo add -p spoon-backend gix@0.80 --no-default-features --features blocking-network-client,blocking-http-transport-reqwest-rust-tls,worktree-mutation
cargo add -p spoon-backend reqwest@0.13 --no-default-features --features rustls-tls,json
cargo add -p spoon-backend msi@0.10

cargo rm -p spoon gix reqwest cab msi mslnk sha1 async-recursion fs-err thiserror base64 bytesize colored_json fs_extra unicode-width
cargo add -p spoon --dev zip@2 sha2@0.10
```

## Sources

- Local repo inspection: `Cargo.toml`
- Local repo inspection: `spoon/Cargo.toml`
- Local repo inspection: `spoon-backend/Cargo.toml`
- Local repo inspection: `spoon-backend/src/gitx.rs`
- Local repo inspection: `spoon-backend/src/scoop/package_state.rs`
- Local repo inspection: `spoon-backend/src/scoop/runtime/installed_state.rs`
- Local repo inspection: `spoon/src/service/mod.rs`
- Local repo inspection: `spoon/src/service/scoop/mod.rs`
- Local repo inspection: `spoon/src/service/scoop/runtime.rs`
- Local repo inspection: `spoon/src/service/msvc/mod.rs`
- Cargo workspace reference: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Rust 2024 resolver 3 reference: https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html
- `gix` crate docs and feature notes: https://docs.rs/gix/latest/gix/
- `gix` feature flags listing: https://docs.rs/crate/gix/latest/features
- `reqwest` crate docs: https://docs.rs/reqwest/latest/reqwest/
- `reqwest` crate release page: https://docs.rs/crate/reqwest/latest
- `msi` crate release page: https://docs.rs/crate/msi/0.10.0
- `cargo-hakari` docs: https://docs.rs/cargo-hakari/latest/cargo_hakari/
