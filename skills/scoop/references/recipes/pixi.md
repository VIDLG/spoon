# pixi — Post-Install Recipe

## When to install

`pixi` is a project and environment manager. Install it when you need to:

- manage data science, Python, Rust, or mixed-language project environments
- use `pixi.toml` and `pixi.lock` to describe project dependencies
- create reproducible development environments
- use Pixi to install and manage global CLI tools

Installing `pixi` only provides the command itself. It does not automatically create a workspace or environment.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install main/pixi
```

## Post-Install Configuration

### First verify the pixi binary itself

After installation, first confirm that `pixi` is available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi --version
```

### Project files are not created automatically by installation

Workspace files such as `pixi.toml` and `pixi.lock` belong to a Pixi project. They are not global config files created automatically when `pixi` is installed.

If the user wants to start a new workspace, they can optionally run:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi init my_workspace
```

After that, dependencies can be added as needed, for example:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi add python
```

### Do not treat `pixi install` as a mandatory default step

Commands such as `pixi run` and `pixi shell` will install the required environment automatically when needed, so `pixi install` should not be treated as the default next step after installation.

### Global tools (optional)

If the user wants to manage global CLI tools with Pixi, they can optionally run:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi global install ruff
```

### Optional environment variable

`PIXI_HOME` can be used to customize Pixi's global data directory. It is not required by default.

Only configure it if the user explicitly wants to relocate Pixi's global environments and cache.

### Update strategy

If `pixi` was installed through Scoop, prefer updating it through Scoop rather than treating `pixi self-update` as the default update path:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop update pixi
```

### Proxy and mirrors

If Scoop fails to download `pixi`, first handle the Scoop-side network issue through the `proxy` skill.

Pixi project-level channel, mirror, and proxy configuration is not centralized in this recipe.

## Verify

Basic verification:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi --version
```

If the current directory is already a Pixi workspace, you can further verify it with:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi info
```

If the user has already added dependencies, you can also verify environment resolution with:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi install
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall pixi
```

Uninstalling `pixi` itself should not automatically remove:

- `~/.pixi`
- project `pixi.toml`
- project `pixi.lock`
- project environments or global tool environments

Those are user data or project content. If the user wants them cleaned up as well, confirm that separately before removing anything.
