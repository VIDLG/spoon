# pkl-cli — Post-Install Recipe

## When to install

Install pkl-cli when the user:

- Works with `.pkl` configuration files
- Needs to generate JSON, YAML, or Property Lists from structured configuration
- Mentions Pkl, Apple's configuration language, or "configuration as code"
- Wants to validate or evaluate Pkl modules from the command line

Pkl is a configuration-as-code language by Apple that combines the simplicity of declarative formats (JSON/YAML) with programming language features (classes, functions, conditionals, type validation). The CLI evaluates `.pkl` files and outputs to JSON, YAML, plist, etc.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install pkl-cli
```

## Post-Install Configuration

pkl-cli works out of the box with no mandatory configuration. Optional setup:

### Editor support

If the user works in VS Code, suggest installing the Pkl extension for syntax highlighting, code completion, and validation:

```
ext install apple.pkl-vscode
```

### Environment variable (optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `PKL_HOME` | Custom directory for Pkl caches and packages | `~/.pkl` |

`PKL_HOME` is rarely needed. Only set it if the user wants to customize the cache location.

### Verify

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pkl-cli --version
```

Quick test — create and evaluate a minimal Pkl file:

```bash
echo 'name = "test"; version = 42' > /tmp/test.pkl
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pkl-cli eval /tmp/test.pkl
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall pkl-cli
```

After uninstalling, use AskUserQuestion to ask about leftover data:

- **Keep** — preserve `~/.pkl` cache directory for future reinstall
- **Remove** — delete `~/.pkl` directory
- **Show first** — display directory size before deciding

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.pkl") { Remove-Item -Path "$env:USERPROFILE\.pkl" -Recurse -Force }'
```
