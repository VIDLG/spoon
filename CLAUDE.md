# Spoon

Windows development environment management toolkit, running as a Claude Code plugin.

## Crate Structure

- `spoon/` — CLI/TUI binary crate
- `spoon-core/` — Shared infrastructure (layout, download, gitx, archive, events)
- `spoon-scoop/` — Scoop domain (manifest, bucket, cache, package workflow)
- `spoon-msvc/` — MSVC domain (toolchain install/update/validate, MSI/CAB)

## Skills

- `scoop` — Manage Scoop package manager and scoop-installed software
- `proxy` — Manage proxy settings and mirror sources for development tools
- `ai-toolchain` — Usage guide for workstation tools provisioned by `spoon.exe`
- `python-via-uv` — Python setup via uv

## Conventions

- Skill docs: English primary, `-zh.md` Chinese counterpart with same command blocks
- Recipes go under `skills/<skill>/references/recipes/`
- AI bootstrap flows belong in `spoon/` Rust project, not in skills or recipes
- Config check priority: config files first, environment variables only as last resort

## Git & Release

- Never force-push, reset --hard, or discard uncommitted changes unless explicitly asked
- Do not commit, push, or amend unless the user explicitly asks
- `.claude/settings.json` — Hooks only, committed. Clean auto-inserted permissions before release
- `.claude/settings.local.json` — Permissions, gitignored

### Release Steps

1. Clean auto-inserted permissions from `.claude/settings.json`
2. Update `version` in `.claude-plugin/plugin.json`
3. Update README files if behavior changed
4. Commit and push
5. Create release tag and publish

### Versioning

- `patch`: bug fixes, recipe improvements, docs
- `minor`: new skill or workflow
- `major`: breaking changes
