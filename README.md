# Spoon

A Claude Code plugin and repository for managing Windows development environments.

`Spoon` is the plugin/repository umbrella. `spoon.exe` is the workstation bootstrap and management executable shipped from this repo.

Spoon focuses on two areas:

- Scoop package manager operations for general Windows development tools
- Proxy and mirror management across common development tooling

Spoon also owns a package integration layer for selected managed tools. Installation is not only
about laying files down under `<root>`: Spoon may additionally materialize stable command shims,
package-local runtime environment, and tool-specific post-install integrations when that produces
a cleaner owned developer experience.

AI workstation bootstrap and ongoing workstation tool management are handled by the Rust project in `spoon/`. The resulting `spoon.exe` executable owns Git, Claude Code, Codex, proxy bootstrap, toolchain management, capability management, and AI helper CLI installation.

## Skills

### scoop

Manages the [Scoop](https://scoop.sh/) package manager and software installed through it.

- Install/uninstall/update scoop and scoop packages
- Bucket management (add, remove, list)
- Health checks and cache cleanup
- Post-install recipes for scoop-managed tools that need extra setup (android-clt, flutter, nodejs, pixi, pkl-cli, rustup)

### ai-toolchain

Usage guide for the workstation tools provisioned by `spoon.exe` (git, claude, codex, gh, rg, fd, jq, yq, bat, delta, sg, uv, zed, scoop toolchain, and MSVC capability).

### proxy

Manages proxy and mirror configuration across development tools.

- HTTP/SOCKS5 proxy for git, scoop, npm, pip, cargo, flutter, etc.
- China mirror sources (TUNA, USTC, SJTUG) for package registries
- Unified enable/disable across all tools

## Project Structure

```text
spoon/
├── spoon/                     # Rust CLI/TUI binary crate
├── spoon-core/                # Shared infrastructure (layout, download, gitx, archive, events)
├── spoon-scoop/               # Scoop domain logic (manifest, bucket, cache, package workflow)
├── spoon-msvc/                # MSVC domain logic (toolchain install/update/validate, MSI/CAB)
├── xtask/                     # Build/deploy automation
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── commands.md / commands-zh.md
│   │       ├── guide-zh.md
│   │       └── recipes/            # Post-install recipes (en + zh pairs)
│   │           └── android-clt, flutter, nodejs, pixi, pkl-cli, rustup
│   ├── proxy/
│   │   ├── SKILL.md
│   │   └── references/
│   │       └── guide-zh.md
│   └── ai-toolchain/
│       ├── SKILL.md
│       └── SKILL-zh.md
├── scripts/
│   ├── run-cmd.ps1
│   └── add-path.ps1
├── CLAUDE.md
├── README.md
└── README-zh.md
```

## Spoon

The `spoon/` directory contains the Rust CLI/TUI for AI workstation bootstrap and management. Build the binary with:

```text
cd spoon && cargo xtask deploy
```

This compiles a release build and copies `spoon.exe` to the repository root (gitignored). Examples:

```text
.\spoon.exe
.\spoon.exe status
.\spoon.exe tools install --tools git,claude,codex,rg
```

## Package Integration Policy

Spoon treats package installation and package integration as related but distinct layers.

- The Scoop runtime is responsible for manifest resolution, download, extraction, install/update/uninstall,
  state, shims, persist, and shortcuts.
- The package integration layer is responsible for Spoon-owned post-install behavior for selected packages,
  such as supplemental managed commands, package-local runtime environment, and tool-specific config application.

Current direction:

- `bin` defines the primary command surface.
- Supplemental commands should be explicit Spoon policy, not a blind expansion of every executable found
  under `env_add_path`.
- `env_add_path` and `env_set` are treated primarily as package-local runtime environment and are applied
  inside Spoon-managed shims/wrappers instead of being broadly persisted into the user's global PATH or
  user-wide environment variables.
- Stable derived values such as install-root locators belong in shim-local environment.
- User-intent settings such as mirrors, proxies, API endpoints, and credentials belong in Spoon-owned config
  or tool-native config, not in generated shims.
- `spoon config` is the user-facing configuration surface:
  - `spoon config root ...` manages Spoon's own runtime configuration such as root and other core executable behavior.
  - `spoon config python ...` / `spoon config git ...` manage user intent for how Spoon-managed packages and tools should be integrated after install.
- Internally, package policy remains a separate typed desired-state layer from applied integration state.
- The normal information flow is `spoon config -> native config`; if a native tool config already exists and Spoon can represent it, use an explicit one-time import such as `spoon config python import` or `spoon config git import` rather than implicit two-way sync.
- `spoon info <pkg>` should make both sides visible:
  - desired package policy from `spoon config <pkg>`
  - applied integration artifacts such as resolved values and policy-managed config files/directories
- High-confidence supplemental commands should live in Spoon's built-in package integration defaults, not in broad runtime heuristics.
  For example, canonical Python commands such as `pip` can be exposed by default without also expanding every
  version-suffixed helper in `Scripts/`.

This keeps Spoon-managed packages explicit, reversible, and deterministic while avoiding broad global
environment pollution.
## Installation

In Claude Code, run:

```text
/plugin marketplace add VIDLG/spoon
```

Then install the spoon plugin from the marketplace. The plugin will be available across all your projects.

## Requirements

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## License

MIT
