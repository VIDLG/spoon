# just — Post-Install Recipe

## When to install

`just` is a command runner typically used to define and execute development tasks inside a project. Install it when you need to:

- Work with a project that already has a `justfile`
- Standardize build, test, format, or release commands
- Replace a collection of ad-hoc shell scripts or batch files with a clearer task interface

Installing `just` only provides the command itself. It does not create a `justfile` automatically.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install just
```

## Post-Install Configuration

### No required extra setup

After Scoop installs it, `just` should be ready to use immediately. No environment variables or global config files are required by default.

### A `justfile` is project content, not tool config

`just` usually reads a `justfile` from the current directory. If the current project does not have one, `just` may install successfully but still have no tasks to run. That is not an installation failure.

If the current directory already contains a `justfile`, you can list the available recipes first:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --list
```

### Proxy and mirrors

If Scoop fails to download `just`, keep proxy and mirror handling centralized in the `proxy` skill rather than duplicating network configuration here.

## Verify

First confirm that the `just` binary is available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --version
```

If the current project already has a `justfile`, verify recipe discovery as well:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --list
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall just
```

Uninstalling `just` usually does not require cleaning up extra global state.

Do not automatically delete a project's `justfile`, because it belongs to the user's project rather than the tool installation.
