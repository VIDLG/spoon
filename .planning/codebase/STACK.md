# Technology Stack

**Analysis Date:** 2026-03-28

## Languages

**Primary:**
- Rust (edition 2024) - Main application crate in `spoon/Cargo.toml`, shared backend crate in `spoon-backend/Cargo.toml`, and build helper crate in `xtask/Cargo.toml`

**Secondary:**
- TOML - Workspace and crate manifests in `Cargo.toml`, `spoon/Cargo.toml`, `spoon-backend/Cargo.toml`, `xtask/Cargo.toml`, user config in `spoon/src/config/io.rs`, and Codex config output at `spoon/src/config/paths.rs`
- JSON - Claude/Codex config persistence in `spoon/src/config/io.rs`, Scoop state/manifests in `spoon-backend/src/scoop/*.rs`, plugin metadata in `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`
- INI - Native Git and pip integration in `spoon/src/config/io.rs` and `spoon/src/packages/python.rs`
- PowerShell script text - Scoop lifecycle hook execution engine in `spoon-backend/src/scoop/runtime/hooks.rs`
- C/C++ and Rust sample sources - MSVC validation templates in `spoon-backend/src/msvc/validate_templates/cpp/hello.cpp`, `spoon-backend/src/msvc/validate_templates/rust/Cargo.toml`, and `spoon-backend/src/msvc/validate_templates/rust/src/main.rs`

## Runtime

**Environment:**
- Windows-focused native executable runtime; the main binary is `spoon.exe` built from `spoon/src/main.rs`
- Async runtime: `tokio` `1` with multi-thread runtime in `spoon/src/main.rs` and backend async work in `spoon-backend/src/lib.rs`
- Default Cargo target is `x86_64-pc-windows-gnu` from `.cargo/config.toml`
- The embedded official MSVC validation sample also defines `x86_64-pc-windows-msvc` linker config in `spoon-backend/src/msvc/validate_templates/rust/.cargo/config.toml`

**Package Manager:**
- Cargo - workspace root manifest in `Cargo.toml`
- Lockfile: present in `Cargo.lock`

## Frameworks

**Core:**
- `clap` `4` - CLI parsing and subcommands in `spoon/src/main.rs` and `spoon/src/cli/args.rs`
- `tokio` `1` - async orchestration, filesystem, process, and task runtime across `spoon/src/main.rs`, `spoon/src/runtime.rs`, and `spoon-backend/src/*`
- `serde` `1` and `serde_json` `1` - config/state serialization in `spoon/src/config/*.rs` and `spoon-backend/src/scoop/*.rs`
- `anyhow` `1` plus `thiserror` `1` - app-level and backend error handling in `spoon/Cargo.toml` and `spoon-backend/src/error.rs`

**UI / CLI:**
- `ratatui` `0.30` - default interactive TUI in `spoon/src/tui/*`
- `crossterm` `0.29` - terminal control for the TUI in `spoon/Cargo.toml`
- `tui-textarea-2` `0.10.1` and `tui-logger` `0.18.1` - form input and log rendering in `spoon/src/tui/*` and `spoon/src/logger/mod.rs`
- `colored_json` `5` and `syntect` `5` - CLI/TUI rendering helpers in `spoon/src/cli/output.rs`

**Testing:**
- Built-in Rust test harness - crate tests and integration tests declared in `spoon/Cargo.toml`, `spoon-backend/Cargo.toml`, and `xtask/src/main.rs`
- Backend integration tests live in `spoon-backend/tests/scoop_integration.rs` and `spoon-backend/tests/msvc_integration.rs`
- App flow tests live in `spoon/tests/cli/*.rs` and `spoon/tests/tui/*.rs`

**Build/Dev:**
- Cargo workspace alias `cargo xtask` from `.cargo/config.toml`
- `xtask` helper crate in `xtask/src/main.rs` builds and deploys `spoon.exe` to the repo root and `~/.local/bin/spoon.exe`
- `.editorconfig` enforces UTF-8, LF, and trimmed whitespace in `.editorconfig`

## Key Dependencies

**Critical:**
- `spoon-backend` path dependency - Shared Scoop, MSVC, proxy, and Git backend for the app crate; wired in `spoon/Cargo.toml` and re-exported from `spoon/src/lib.rs`
- `reqwest` `0.12` - HTTP downloads for Scoop payloads and official MSVC bootstrapper in `spoon-backend/src/scoop/runtime/download.rs`, `spoon-backend/src/msvc/common.rs`, and `spoon-backend/src/msvc/official.rs`
- `gix` `0.80` in `spoon` and `0.70` in `spoon-backend` - Git transport for bucket cloning and repository sync in `spoon-backend/src/gitx.rs`
- `ratatui` `0.30` and `crossterm` `0.29` - primary interactive UX in `spoon/src/tui/mod.rs` and related TUI modules
- `clap` `4` - CLI entrypoint and automation surface in `spoon/src/main.rs` and `spoon/src/cli/*`

**Infrastructure:**
- `tracing` `0.1`, `tracing-subscriber` `0.3`, and `tracing-appender` `0.2` - local structured logging in `spoon/src/logger/mod.rs`
- `toml` `0.9`, `toml_edit` `0.25`, and `rust-ini` `0.21` - edits user config files without owning whole-file formatting in `spoon/src/config/io.rs`
- `winreg` `0.55` - Windows registry environment mutation in `spoon/src/config/env.rs`
- `cab` `0.6`, `msi` `0.8`/`0.10`, `zip` `2`, and `mslnk` `0.1.8` - package extraction, MSI handling, and shortcut generation in `spoon-backend/src/msvc/*` and `spoon-backend/src/scoop/*`
- `sha1` `0.10` and `sha2` `0.10` - payload verification in `spoon-backend/src/scoop/runtime/download.rs` and `spoon-backend/src/msvc/mod.rs`
- `which` `8` and `walkdir` `2` - binary discovery and filesystem traversal in `spoon/src/editor/discovery.rs`, `spoon-backend/src/msvc/common.rs`, and `spoon-backend/src/msvc/mod.rs`

## Configuration

**Environment:**
- Spoon-owned global config is stored in `~/.spoon/config.toml` via `spoon/src/config/paths.rs` and `spoon/src/config/io.rs`
- Native Git integration writes `~/.gitconfig` through `spoon/src/config/io.rs`
- Claude config is read from and written to `~/.claude/settings.json` by `spoon/src/config/io.rs`
- Codex config is read from and written to `~/.codex/config.toml` and `~/.codex/auth.json` by `spoon/src/config/io.rs`
- Python mirror integration writes `pip.ini` under the platform config directory resolved in `spoon/src/config/paths.rs`
- Optional environment overrides detected in code:
  - `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN` in `spoon/src/config/io.rs`
  - `OPENAI_BASE_URL`, `OPENAI_API_KEY` in `spoon/src/config/io.rs`
  - `EDITOR` in `spoon/src/editor/discovery.rs`
  - Test-only overrides such as `SPOON_TEST_HOME`, `SPOON_TEST_MODE`, `SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE`, and `SPOON_TEST_SCOOP_BUCKET_*_SOURCE` in `spoon/src/main.rs`, `spoon-backend/src/msvc/official.rs`, and `spoon-backend/src/scoop/buckets.rs`
- `.env` files were not detected at the repository root during this analysis

**Build:**
- Workspace membership is defined in `Cargo.toml`
- Default target and `cargo xtask` alias are defined in `.cargo/config.toml`
- Release deployment logic is implemented in `xtask/src/main.rs`
- Plugin packaging metadata for Claude is stored in `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`

## Platform Requirements

**Development:**
- Windows 10/11 is the intended host platform according to `README.md`
- A Rust toolchain new enough for edition 2024 is required; no workspace-level `rust-toolchain.toml` is present at the repo root
- Cargo is required for `cargo build`, `cargo test`, and `cargo xtask deploy`
- Native Windows utilities such as `cmd.exe`, `taskkill`, `robocopy.exe`, and `msiexec.exe` are invoked from `xtask/src/main.rs`, `spoon-backend/src/msvc/official.rs`, and `spoon-backend/src/scoop/runtime/hooks.rs`

**Production:**
- Deployment target is a local Windows workstation managed through the repo-root `spoon.exe`
- `xtask/src/main.rs` also deploys a copy to `~/.local/bin/spoon.exe`
- Runtime state is filesystem-backed under the configured tool root, especially `<root>\\scoop`, `<root>\\msvc`, and `<root>\\shims` from `spoon/src/config/paths.rs`

---

*Stack analysis: 2026-03-28*
