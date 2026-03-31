# External Integrations

**Analysis Date:** 2026-03-28

## APIs & External Services

**Package acquisition and source control:**
- Scoop bucket Git repositories - Spoon clones and updates bucket repositories through `gix` in `spoon-backend/src/gitx.rs` and `spoon-backend/src/scoop/buckets.rs`
  - SDK/Client: `gix` from `spoon-backend/Cargo.toml`
  - Auth: none in repo code; bucket clones rely on public Git URLs or external Git credentials already configured on the machine
- Default Scoop bucket remotes are hardcoded in `spoon-backend/src/scoop/buckets.rs`
  - `main` -> `https://github.com/ScoopInstaller/Main`
  - `extras` -> `https://github.com/ScoopInstaller/Extras`
  - `versions` -> `https://github.com/ScoopInstaller/Versions`
  - `nirsoft` -> `https://github.com/ScoopInstaller/Nirsoft`
  - `sysinternals` -> `https://github.com/niheaven/scoop-sysinternals`
  - `php` -> `https://github.com/ScoopInstaller/PHP`
  - `nerd-fonts` -> `https://github.com/matthewjberger/scoop-nerd-fonts`
  - `nonportable` -> `https://github.com/ScoopInstaller/Nonportable`
  - `java` -> `https://github.com/ScoopInstaller/Java`
  - `games` -> `https://github.com/Calinou/scoop-games`
- Scoop package payload downloads - Runtime downloads arbitrary URLs declared in bucket manifests via `spoon-backend/src/scoop/runtime/download.rs`
  - SDK/Client: `reqwest` from `spoon-backend/Cargo.toml`
  - Auth: none in repo code; proxy support comes from Spoon global config in `spoon/src/config/io.rs`

**AI tool configuration surfaces:**
- Anthropic / Claude Code endpoint configuration - Claude settings are loaded and saved in `spoon/src/config/io.rs`
  - SDK/Client: no Anthropic SDK in the repo; Spoon edits Claude’s native settings file at `~/.claude/settings.json` via `spoon/src/config/paths.rs`
  - Auth: `ANTHROPIC_AUTH_TOKEN` in `~/.claude/settings.json` or process environment; base URL defaults to `https://api.anthropic.com`
- OpenAI / Codex endpoint configuration - Codex settings are loaded and saved in `spoon/src/config/io.rs`
  - SDK/Client: no OpenAI SDK in the repo; Spoon edits Codex native files at `~/.codex/config.toml` and `~/.codex/auth.json` via `spoon/src/config/paths.rs`
  - Auth: `OPENAI_API_KEY` in `~/.codex/auth.json` or process environment; base URL defaults to `https://api.openai.com`

**Microsoft toolchain delivery:**
- Official Visual Studio Build Tools bootstrapper - Downloaded by the official MSVC flow in `spoon-backend/src/msvc/official.rs`
  - SDK/Client: `reqwest` through `spoon-backend/src/msvc/common.rs`
  - Auth: none
  - Endpoint: `https://aka.ms/vs/17/release/vs_BuildTools.exe`
- Managed MSVC manifest cache - The managed MSVC path reads a cached release manifest from `latest.json` under the manifest cache root in `spoon-backend/src/msvc/manifest.rs`
  - SDK/Client: local JSON parsing only
  - Auth: not applicable
  - Current behavior: `sync_release_manifest_cache_async` in `spoon-backend/src/msvc/manifest.rs` does not fetch a remote manifest; it only reports whether the cached manifest exists

**Python ecosystem mirrors:**
- Pip mirror configuration - Spoon maps policy values to well-known mirrors in `spoon/src/packages/python.rs`
  - SDK/Client: native `pip.ini` file mutation via `rust-ini`
  - Auth: none
  - Built-in mirror mappings:
    - `tuna` -> `https://pypi.tuna.tsinghua.edu.cn/simple`
    - `ustc` -> `https://pypi.mirrors.ustc.edu.cn/simple`
    - `sjtug` -> `https://mirror.sjtu.edu.cn/pypi/web/simple`

**Plugin distribution metadata:**
- Claude plugin marketplace source - Metadata in `.claude-plugin/marketplace.json` points the plugin to GitHub repository `VIDLG/spoon`
  - SDK/Client: plugin marketplace metadata only
  - Auth: none in repo code

## Data Storage

**Databases:**
- Not detected

**File Storage:**
- Local filesystem only
- Spoon-owned user config root is `~/.spoon` from `spoon/src/config/paths.rs`
- Spoon-owned tool root is configured in `~/.spoon/config.toml` and expanded into:
  - `<root>\\scoop` via `spoon/src/config/paths.rs` and `spoon-backend/src/scoop/paths.rs`
  - `<root>\\msvc\\managed` via `spoon/src/config/paths.rs` and `spoon-backend/src/msvc/paths.rs`
  - `<root>\\msvc\\official` via `spoon/src/config/paths.rs` and `spoon-backend/src/msvc/paths.rs`
  - `<root>\\shims` via `spoon/src/config/paths.rs` and `spoon-backend/src/msvc/paths.rs`

**Caching:**
- Scoop download cache under `<root>\\scoop\\cache` from `spoon-backend/src/scoop/paths.rs`
- Scoop bucket registry under `<root>\\scoop\\state\\buckets.json` from `spoon-backend/src/scoop/paths.rs`
- Scoop installed package state under `<root>\\scoop\\state\\packages\\*.json` from `spoon-backend/src/scoop/paths.rs`
- Managed MSVC archives, expanded payloads, and manifest cache under `<root>\\msvc\\managed\\cache` from `spoon-backend/src/msvc/paths.rs`
- Official MSVC bootstrapper and logs under `<root>\\msvc\\official\\cache` from `spoon-backend/src/msvc/official.rs`
- Local application log file under `~/.spoon/logs/spoon.log` from `spoon/src/logger/mod.rs`

## Authentication & Identity

**Auth Provider:**
- No first-party auth service is implemented in this repository
  - Implementation: Spoon persists third-party CLI credentials in the native files those CLIs expect, rather than running its own login flow

**Credential-bearing integrations:**
- Claude Code
  - Config paths: `~/.claude/settings.json` from `spoon/src/config/paths.rs`
  - Credential field: `ANTHROPIC_AUTH_TOKEN` handled by `spoon/src/config/io.rs`
- Codex
  - Config paths: `~/.codex/config.toml` and `~/.codex/auth.json` from `spoon/src/config/paths.rs`
  - Credential field: `OPENAI_API_KEY` handled by `spoon/src/config/io.rs`
- Git
  - Native config path: `~/.gitconfig` from `spoon/src/config/paths.rs`
  - Identity fields: `user.name`, `user.email`, and proxy settings handled by `spoon/src/config/io.rs`

## Monitoring & Observability

**Error Tracking:**
- None detected

**Logs:**
- Local tracing logs written to `~/.spoon/logs/spoon.log` via `spoon/src/logger/mod.rs`
- TUI log streaming is enabled with `tui-logger` in `spoon/src/logger/mod.rs`
- Verbose CLI logging is buffered to stdout when requested through `LoggerSettings::standard` in `spoon/src/logger/settings.rs`

## CI/CD & Deployment

**Hosting:**
- Not applicable; the primary artifact is a local Windows executable, `spoon.exe`, at the repository root

**CI Pipeline:**
- No top-level repository CI workflow was detected under `.github/`
- Contributor snapshots exist under `contrib/` and `tmp/`, but they are not the primary repo CI surface for the workspace analyzed here

**Deployment path:**
- Local deployment is handled by `xtask/src/main.rs`
- `cargo xtask deploy` builds `spoon` and replaces:
  - `spoon.exe` at the repository root
  - `~/.local/bin/spoon.exe`

## Environment Configuration

**Required env vars:**
- None are strictly required for the binary to start; the core runtime can run from local config alone
- Critical optional vars for external integrations:
  - `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL` in `spoon/src/config/io.rs`
  - `OPENAI_API_KEY` and `OPENAI_BASE_URL` in `spoon/src/config/io.rs`
  - `EDITOR` fallback in `spoon/src/editor/discovery.rs`
- Test-only env vars used by repo code:
  - `SPOON_TEST_MODE` and `SPOON_TEST_HOME` in `spoon/src/main.rs`
  - `SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE` in `spoon-backend/src/msvc/official.rs`
  - `SPOON_TEST_SCOOP_BUCKET_*_SOURCE` in `spoon-backend/src/scoop/buckets.rs`

**Secrets location:**
- Not stored in repository files
- Anthropic token is stored in `~/.claude/settings.json` or process environment via `spoon/src/config/io.rs`
- OpenAI API key is stored in `~/.codex/auth.json` or process environment via `spoon/src/config/io.rs`
- Spoon global config at `~/.spoon/config.toml` stores root/proxy/editor/MSVC selection state, not API tokens, via `spoon/src/config/io.rs`

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- HTTP downloads for Scoop payload URLs declared in bucket manifests through `spoon-backend/src/scoop/runtime/download.rs`
- HTTP download of the official Microsoft bootstrapper in `spoon-backend/src/msvc/official.rs`
- Git repository sync against Scoop bucket remotes through `spoon-backend/src/gitx.rs`
- No webhook callback handlers or HTTP server endpoints were detected

## Platform / Native Integrations

**Windows registry and environment:**
- Spoon mutates `HKCU\\Environment` and user `Path` via `winreg` in `spoon/src/config/env.rs`
- Spoon also updates current-process environment variables separately in `spoon/src/config/env.rs`

**Native tool config files:**
- Git proxy and identity integration in `~/.gitconfig` via `spoon/src/config/io.rs`
- pip mirror integration in `pip.ini` via `spoon/src/packages/python.rs`
- Claude and Codex native config file mutation in `spoon/src/config/io.rs`

**Shell/runtime execution:**
- Scoop lifecycle scripts are executed through PowerShell in `spoon-backend/src/scoop/runtime/hooks.rs`
- Official MSVC install/update/uninstall launches the Microsoft bootstrapper process in `spoon-backend/src/msvc/official.rs`
- `xtask/src/main.rs` uses `taskkill` to replace a locked `spoon.exe` during deploy

---

*Integration audit: 2026-03-28*
