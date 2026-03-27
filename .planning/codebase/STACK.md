# Technology Stack

**Analysis Date:** 2026-03-28

## Languages

**Primary:**
- Rust 2024 Edition - Core application implementation in `spoon/` and `spoon-backend/`
- TOML - Configuration files (`Cargo.toml`, config storage)

**Secondary:**
- JSON - Claude and OpenAI API configuration

## Runtime

**Environment:**
- Windows (11 Home China) - Target platform
- Native compilation via Cargo

**Package Manager:**
- Cargo 1.75+ - Rust package manager
- Lockfile: `Cargo.lock` present

## Frameworks

**Core:**
- TUI Framework - Ratatui 0.30 with Crossterm for terminal UI
- CLI Framework - Clap 4 with derive features for command parsing

**Testing:**
- Built-in test runner - Rust's built-in testing framework
- Test fixtures in `tests/common/fixtures.rs`

**Build/Dev:**
- XTask - Build automation via `xtask/` workspace member
- MSVC Toolchain support for Windows development

## Key Dependencies

**Critical:**
- `spoon-backend` 0.1.0 - Core backend services (workspace dependency)
- `reqwest` 0.12 - HTTP client for API calls, with rustls-tls
- `gix` 0.80 - Git operations and version control
- `ratatui` 0.30 - Terminal UI framework
- `clap` 4 - Command line interface
- `tokio` 1 - Async runtime

**Infrastructure:**
- `serde` 1 - Serialization/deserialization with derive features
- `toml` 0.9 - TOML configuration parsing
- `tracing` 0.1 - Application logging
- `anyhow` 1 - Error handling

## Configuration

**Environment:**
- Home directory: `~/.spoon/`
- Config files: `config.toml` for global settings
- API tokens via environment variables or config files

**Build:**
- Cargo workspace with three members: `spoon`, `spoon-backend`, `xtask`
- Windows-specific dependencies in `target.'cfg(windows)'`
- Feature flags for optional dependencies

## Platform Requirements

**Development:**
- Rust 2024 Edition
- Cargo build system
- Windows for native compilation

**Production:**
- Windows (tested on Windows 11)
- Native binary: `spoon.exe`
- No external runtime required (standalone executable)

---

*Stack analysis: 2026-03-28*
```