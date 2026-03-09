# git — Post-Install Recipe

## When to install

git is **mandatory** for scoop — bucket operations (add, update) require it. Always install git immediately after scoop itself.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install git
```

## Post-Install Configuration

### Add git tools to PATH

Scoop only shims a few binaries (`git`, `sh`, `git-bash`), but `bash.exe` and Unix utilities (`less`, `awk`, etc.) live in git's own directories and need explicit PATH entries:

```bash
powershell -File <plugin_root>/skills/scripts/add-path.ps1 git bin usr/bin
```

### Configure git

1. **Set default branch to main** (always, no need to ask):
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global init.defaultBranch main
   ```

2. **Ask user for name and email** via AskUserQuestion. These are required for git commits. If the user skips, warn that commits will fail without them.
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global user.name '<name>'
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global user.email '<email>'
   ```

3. **Check existing git config** — if `~/.gitconfig` already exists, report its contents and ask before overwriting any values.

## Verify

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git --version
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall git
```

Remove PATH entries added during installation:

```bash
powershell -File <plugin_root>/skills/scripts/add-path.ps1 git bin usr/bin -Remove
```

Note: When uninstalling scoop entirely, the PATH cleanup (`-notmatch "Scoop"`) already covers these entries since the paths contain "Scoop". The `-Remove` flag is for cases where git is uninstalled individually while scoop remains.

After uninstalling, use AskUserQuestion to ask about leftover config:

- **Keep** — preserve `~/.gitconfig` for future use
- **Remove** — delete `~/.gitconfig`
- **Show first** — display contents before deciding

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.gitconfig") { Remove-Item -Path "$env:USERPROFILE\.gitconfig" -Force }'
```
