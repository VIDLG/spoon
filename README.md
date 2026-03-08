# Spoon

A Claude Code plugin for managing Windows development environments.

Spoon automates software installation, configuration, and maintenance through a set of skills that Claude Code can invoke during conversations.

## Skills

### scoop

Manages the [Scoop](https://scoop.sh/) package manager and all software installed through it.

- Install/uninstall/update scoop and packages
- Bucket management (add, remove, list)
- Health checks and cache cleanup
- Post-install recipes for tools that need extra configuration (e.g., claude-code, codex)

### proxy

Manages proxy and mirror configuration across development tools.

- HTTP/SOCKS5 proxy for git, scoop, npm, pip, cargo, flutter, etc.
- China mirror sources (TUNA, USTC, SJTUG) for package registries
- Unified enable/disable across all tools

## Project Structure

```
spoon/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md          # Scoop skill definition
│   │   └── references/
│   │       ├── commands.md       # Command reference
│   │       ├── commands-zh.md    # Command reference (Chinese)
│   │       ├── guide-zh.md       # Skill guide (Chinese)
│   │       └── recipes/          # Post-install configuration recipes
│   │           ├── claude-code.md / claude-code-zh.md
│   │           └── codex.md / codex-zh.md
│   ├── proxy/
│   │   ├── SKILL.md          # Proxy skill definition
│   │   └── references/
│   │       └── guide-zh.md       # Skill guide (Chinese)
│   └── scripts/
│       ├── run-cmd.ps1       # Run commands with fresh PATH from registry
│       └── add-path.ps1      # Add/remove scoop app subdirectories to/from PATH
├── CLAUDE.md                 # Project-level instructions for Claude Code
└── README.md
```

## Installation

Add spoon as a local plugin in your Claude Code user settings (`~/.claude/settings.json`):

```json
{
  "enabledPlugins": {
    "spoon@local": true
  }
}
```

Then register the plugin path. Claude Code will discover the skills automatically from the `.claude-plugin/plugin.json` file.

## Requirements

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## License

MIT
