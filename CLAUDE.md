# Spoon

Windows development environment management toolkit, running as a Claude Code plugin.

## Skills

- **scoop** — Manage scoop package manager and software installed through it (install/uninstall/update/maintain/bucket management)
- **proxy** — Manage proxy settings and mirror sources for all development tools (git/scoop/npm/pip/cargo/flutter)

## Project Conventions

- All skill documents use English as the primary version, with a `-zh.md` Chinese counterpart
- Chinese versions must include the same executable command blocks as the English version — never omit them
- Recipes go under the corresponding skill's `references/recipes/` directory
- Shared scripts go in `skills/scripts/`, referenced via `<plugin_root>` placeholder

## Development Guidelines

### Adding a Recipe

For tools installed via scoop that require additional post-install configuration. Template structure:

1. **When to install** — Describe scenarios where this tool should be installed
2. **Install** — `scoop install` command
3. **Post-Install Configuration** — Check existing config → ask user → write config
4. **Verify** — Confirm successful installation
5. **Uninstall** — Uninstall command + ask whether to clean up leftover config

Config check priority: config files → environment variables (not recommended)
Sensitive values must be masked when displayed (e.g., `sk-...16f8`)

### Adding a Skill

For tool categories requiring independent, ongoing management (e.g., proxy, containers, databases). Difference from recipes: skills involve continuous operations and complex management; recipes are one-time post-install configuration.

### Script Invocation

- Run commands: `powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 <cmd> <args>`
- Manage PATH: `powershell -File <plugin_root>/skills/scripts/add-path.ps1 <app> <subdirs>`
- Non-scoop PowerShell commands: `powershell -Command '...'` (use single quotes in bash to prevent `$` interpretation)

## Git & Release

### settings.json Strategy

- `.claude/settings.json` — Hooks only, committed to git
- `.claude/settings.local.json` — Permissions, gitignored
- Claude Code auto-inserts permission entries into settings.json — must clean before release

### Release Steps

1. Clean auto-inserted permissions from `.claude/settings.json` — verify it only contains `hooks`, no `permissions` key
2. Update `version` in `.claude-plugin/plugin.json`
3. Check whether `README.md` and `README-zh.md` need updates to reflect changes
4. Commit all changes and push to GitHub
5. Create release with `gh release create`, tag matches version (e.g., `v0.2.1`)

### Versioning

- **patch** (x.x.1): bug fixes, recipe improvements, documentation updates
- **minor** (x.1.0): new skill or recipe
- **major** (1.0.0): breaking changes
