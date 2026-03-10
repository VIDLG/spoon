# Python / pip — Post-Install Recipe

## When to install

Python is a general-purpose programming language runtime. Installing it also provides `python` and `pip`. Install it when you need to:

- Run Python projects or scripts
- Install and manage Python packages
- Use `pip` to install CLI tools or libraries
- Create virtual environments for projects

`pip` should not be treated as a separate standalone install. The normal path is to install Python and get `pip` with it.

## Install

By default, install the main Python package:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install python
```

If the user explicitly needs a specific major or older Python version, search for and install the matching versioned package instead of making old versions the default path.

## Post-Install Configuration

### No required extra setup

After installation, `python` and `pip` should already be available. Do not add a separate pip installation step.

### Prefer `python -m pip`

When running pip operations, prefer `python -m pip` so the command is guaranteed to target the current Python interpreter:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m pip --version
```

### Virtual environments (recommended for projects)

For project dependencies, prefer a virtual environment instead of installing packages into the global interpreter:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m venv .venv
```

### Proxy and mirrors

If the user is in China, or PyPI access is slow or failing, do not edit `pip config` directly in this recipe.

Delegate all pip network configuration to the `proxy` skill, including:

- pip proxy settings
- PyPI mirrors
- restoring the official PyPI index

## Verify

First confirm that Python and pip are available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m pip --version
```

If the user insists on using the plain `pip` command, you can also verify it explicitly:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip --version
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall python
```

After uninstalling, use AskUserQuestion to ask about leftover data:

- **Keep** — preserve pip cache and user config for future use
- **Remove** — delete pip user-level cache and config
- **Preview** — inspect the relevant directories before deciding

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:LOCALAPPDATA\pip\Cache") { Remove-Item -Path "$env:LOCALAPPDATA\pip\Cache" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:APPDATA\pip") { Remove-Item -Path "$env:APPDATA\pip" -Recurse -Force }'
```
