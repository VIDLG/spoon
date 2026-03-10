---
name: scoop
description: >
  This skill should be used when the user asks to "install scoop", "install software with scoop",
  "uninstall scoop", "update scoop", "add a scoop bucket", "remove a bucket",
  "clean scoop cache", "check scoop health", "list installed packages", "search for a package",
  "manage dev tools", or mentions scoop, package management, or development tool
  installation/configuration on Windows. Use this skill for any scoop-related operation
  including installing specific tools like git, nodejs, python, pkl, etc.
  On Windows, this skill is the preferred way to install any software — when the user says
  "install X", "I need X", "set up X", or "how to get X" for any development tool or CLI utility,
  use this skill to install it via scoop first. Only fall back to other methods if scoop
  does not have the package.
  For proxy/mirror configuration (set proxy, switch to gitee mirror, etc.), defer to the `proxy` skill.
---

# Scoop Package Manager

Manage the scoop package manager and all development software installed through it on Windows.

## Side Effects Reporting

**CRITICAL**: After any operation that changes system-level state (install, uninstall, update, bucket changes, etc.), always summarize the side effects to the user. This includes:

- Environment variables added, modified, or removed
- PATH entries added or removed
- Directories created or deleted
- Shims created or removed
- Shortcuts added or removed
- Any other system-level changes

Present the summary as a clear list so the user knows exactly what changed on their system.

## Install Scoop

### Step 1: Detect existing installation

Run `scoop --version` to check if scoop is already installed.

- If installed: report the current version and install path (`$env:SCOOP` or default `~/scoop`). Ask the user whether to **update**, **reconfigure**, or **skip**.
- If not installed: proceed with installation.

### Step 2: Confirm install path

Use AskUserQuestion to let the user choose an install directory:

- Option 1: `D:\Scoop` (Recommended) — separate directory, keeps C: drive clean
- Option 2: `~/scoop` — scoop default location
- Option 3: Custom path — user provides their own

### Step 3: Check if target directory exists

Before installing, check if the chosen directory already exists:

- **Directory exists and contains a scoop installation** (has `apps/`, `shims/`, etc.): this is likely a previous scoop install. Ask the user whether to **reuse it** (skip install, just verify), **wipe and reinstall**, or **choose a different path**.
- **Directory exists but is not a scoop installation**: warn the user that the directory is not empty. Ask whether to **use it anyway** (scoop will install into it), **wipe it first**, or **choose a different path**.
- **Directory does not exist**: proceed normally.

### Step 4: Run the installer

Download and execute the official installer with the chosen path:

```powershell
powershell -Command "irm get.scoop.sh -outfile 'install.ps1'; .\install.ps1 -ScoopDir '<chosen_path>'"
```

The installer does NOT set the `SCOOP` environment variable. After installation, explicitly set it:

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("SCOOP", "<chosen_path>", "User")'
```

Available installer parameters (use when relevant):
- `-ScoopDir` — scoop install directory
- `-NoProxy` — bypass system proxy during installation

The following parameters require administrator privileges. Do NOT use unless the user explicitly requests:
- `-ScoopGlobalDir` — global apps install directory (requires admin)
- `-RunAsAdmin` — admin mode installation (requires elevated console)

**Shell environment note**: The installer writes environment variables to the Windows registry via `[Environment]::SetEnvironmentVariable`. However, the Bash tool in Claude Code inherits its environment from the **parent VSCode process**, not from the registry. This means:

- Within the same VSCode window (even new conversations): bash will NOT see the new PATH/SCOOP
- After restarting VSCode: bash will pick up the new environment
- Even `powershell -Command "scoop ..."` may NOT work immediately, because the new PowerShell process inherits PATH from bash (its parent), not from the registry

**Recommended approach**: Use `skills/scripts/run-cmd.ps1` (relative to the plugin root) to refresh PATH from the registry before running any command. This avoids bash/PowerShell quoting conflicts with `$env`, `$null`, etc.

Resolve the absolute path of `skills/scripts/run-cmd.ps1` based on the plugin root directory, then use it with `powershell -File`:

```bash
# Example (replace <plugin_root> with the plugin's absolute path):
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install git
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop bucket add extras
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global init.defaultBranch main
```

For non-scoop PowerShell commands (e.g., setting environment variables), use `powershell -Command` with single quotes in bash to protect `$` from bash interpolation:

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("SCOOP", "D:\Scoop", "User")'
```

**Important caveats when mixing bash and PowerShell**:
- `$env:Path`, `$null`, etc. get swallowed by bash — use single quotes or a `.ps1` file
- Use `[NullString]::Value` instead of `$null` when clearing env vars via `-Command`
- Prefer `-File` over `-Command` for anything beyond simple one-liners

Alternatively, refresh the bash environment directly:

```bash
export SCOOP=$(powershell -Command "[Environment]::GetEnvironmentVariable('SCOOP', 'User')")
export PATH="$SCOOP/shims:$PATH"
```

Advise the user to **restart VSCode** after installation is complete so that all future sessions have the correct environment natively.

### Step 5: Install git and gh

Scoop buckets are managed by git — it must be installed before proceeding to bucket operations. gh is strongly recommended for GitHub integration.

→ See `references/recipes/git.md` (mandatory)
→ See `references/recipes/gh.md` (strongly recommended)

### Step 6: Confirm and add buckets

Use AskUserQuestion with multiSelect to let the user pick buckets to add:

- `extras` (Recommended — common GUI apps and dev tools)
- `versions` (older/alternative versions of software)
- `java` (JDK distributions: adoptium, zulu, etc.)
- `nerd-fonts` (patched developer fonts)

The user can also specify additional buckets via the "Other" option.

Add each selected bucket: `powershell -Command "scoop bucket add <name>"`

### Step 7: Run scoop update

After adding buckets, run `scoop update` to pull the latest bucket manifests via git:

```bash
powershell -Command "scoop update"
```

This verifies that git is working correctly with scoop and ensures all bucket data is up to date.

### Side effects summary after install

Report to the user:
- `SCOOP` environment variable set to `<install_path>`
- `<install_path>\shims` added to user PATH
- git installed at `<install_path>\apps\git\current`
- gh installed at `<install_path>\apps\gh\current`
- Buckets cloned to `<install_path>\buckets\`
- Directories created: `<install_path>\{apps,buckets,cache,persist,shims}`

## Uninstall Scoop

This is a destructive, irreversible operation. Always confirm with the user before proceeding.

1. Ask the user to confirm they want to uninstall scoop and all installed packages.
2. Run `powershell -Command "scoop uninstall scoop"` (removes scoop and all scoop-installed apps).
3. Clean up environment variables (use `[NullString]::Value` instead of `$null` because `$null` gets swallowed by bash):
   ```bash
   powershell -Command "[Environment]::SetEnvironmentVariable('SCOOP', [NullString]::Value, 'User')"
   ```
   If the user had global installs enabled (requires admin):
   ```bash
   powershell -Command "[Environment]::SetEnvironmentVariable('SCOOP_GLOBAL', [NullString]::Value, 'Machine')"
   ```
4. Remove scoop-related entries from PATH:
   ```bash
   powershell -Command '$path = [Environment]::GetEnvironmentVariable("PATH", "User"); $cleaned = ($path -split ";" | Where-Object { $_ -notmatch "Scoop" }) -join ";"; [Environment]::SetEnvironmentVariable("PATH", $cleaned, "User")'
   ```
5. Delete the install directory. **Important**: scoop uses NTFS junctions (e.g., `current` → version dir). PowerShell's `Remove-Item -Recurse -Force` cannot delete junctions. Use `cmd /c rmdir /s /q` instead:
   ```bash
   powershell -Command "& cmd /c 'rmdir /s /q <install_path>'"
   ```
6. **Report side effects**: list all environment variables removed, PATH entries removed, and directories deleted.

## Proxy and Mirror Configuration

Proxy and mirror management is handled by the **`proxy` skill**. This skill covers all tools (git, scoop, npm, pip, cargo, flutter, etc.) in a unified way.

When a scoop operation fails with a network/SSL error, or the user asks about proxy/mirror settings, delegate to the `proxy` skill.

During scoop installation (Step 5, after git is installed), the `proxy` skill should be consulted to detect existing proxy and sync to scoop if needed.

## Daily Operations

### Search for packages
```bash
powershell -Command "scoop search <query>"
```

### Install a package
```bash
powershell -Command "scoop install <app>"
```
After installing, check if a recipe exists in `references/recipes/` for the app. If found (e.g., `references/recipes/claude-code.md`), read it and follow the post-install configuration steps.

Some user-facing tool names map to a different scoop package and recipe name. Handle these aliases before installation. Example:

- If the user asks to install **`npm`**, interpret that as installing **Node.js**.
- Use AskUserQuestion to let the user choose **`nodejs`** or **`nodejs-lts`**. If the user has no preference, default to **`nodejs`**.
- After installation, apply `references/recipes/nodejs.md` rather than looking for an `npm` recipe.
- If the user asks to install **`pip`**, interpret that as installing **Python**.
- Install **`python`** unless the user explicitly asks for a different version line.
- After installation, apply `references/recipes/python.md` rather than looking for a `pip` recipe.

Report side effects: shims created, PATH changes, environment variables set by the app.

### Uninstall a package
```bash
powershell -Command "scoop uninstall <app>"
```
Report side effects: shims removed, any environment variables or PATH entries cleaned up.

### Update
```bash
powershell -Command "scoop update"            # update scoop itself and bucket manifests
powershell -Command "scoop update *"          # update all installed packages
powershell -Command "scoop update <app>"      # update a specific package
```

### View status
```bash
powershell -Command "scoop list"             # list installed packages
powershell -Command "scoop status"           # show packages that can be updated
powershell -Command "scoop info <app>"       # show detailed info about a package
```

### Bucket management
```bash
powershell -Command "scoop bucket list"          # list added buckets
powershell -Command "scoop bucket add <name>"    # add a bucket
powershell -Command "scoop bucket rm <name>"     # remove a bucket
powershell -Command "scoop bucket known"         # list all known official buckets
```

### Maintenance
```bash
powershell -Command "scoop cleanup *"        # remove old versions of all apps
powershell -Command "scoop cleanup <app>"    # remove old versions of a specific app
powershell -Command "scoop cache rm *"       # clear the download cache
powershell -Command "scoop checkup"          # run a health check on scoop
powershell -Command "scoop reset <app>"      # reset an app (re-link shims and shortcuts)
```

### Backup & Restore

#### Export (backup)

Export the list of installed apps, buckets, and their versions to a JSON file:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop export > <backup_path>/scoopfile.json
```

This only saves the **manifest** (app names, versions, bucket sources) — not the actual binaries or cached downloads.

Use AskUserQuestion to let the user choose:
1. **Backup location** — where to save `scoopfile.json` (e.g., `D:\Backup`, a cloud-synced folder, etc.)
2. **Include download cache?** — whether to also copy `<scoop>/cache` to the backup location. This avoids re-downloading large packages (e.g., Flutter ~1GB) on restore. If yes:
   ```bash
   powershell -Command 'Copy-Item -Path "<scoop>\cache" -Destination "<backup_path>\scoop-cache" -Recurse -Force'
   ```
3. **Include persist data?** — whether to back up `<scoop>/persist` (app configs and data managed by scoop's persist feature). If yes:
   ```bash
   powershell -Command 'Copy-Item -Path "<scoop>\persist" -Destination "<backup_path>\scoop-persist" -Recurse -Force'
   ```

#### Import (restore)

Restore on a new machine (scoop must already be installed). **Order matters**:

1. **Restore cache first** (if backed up) — so `scoop import` skips downloading:
   ```bash
   powershell -Command 'Copy-Item -Path "<backup_path>\scoop-cache\*" -Destination "<scoop>\cache" -Recurse -Force'
   ```

2. **Import the manifest** — installs all apps and creates NTFS junctions:
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop import <backup_path>/scoopfile.json
   ```
   Buckets referenced in the file are added automatically.

3. **Restore persist data** (if backed up) — after import, so junctions already point to the correct new paths:
   ```bash
   powershell -Command 'Copy-Item -Path "<backup_path>\scoop-persist\*" -Destination "<scoop>\persist" -Recurse -Force'
   ```

4. **Apply recipes** — check for recipes and run post-install configuration (e.g., git user config, gh auth login).

**Path safety**: Cache files are path-independent (matched by filename/hash). Persist data is also safe — `scoop import` creates new NTFS junctions pointing to the new `<scoop>/persist` location, so even if the scoop install path changed, junctions are correct. In rare cases, an app's own config files inside persist may contain hardcoded absolute paths — these would need manual fixing.

## Recipes

For tools that need post-install configuration beyond just `scoop install`, recipe files are stored in `references/recipes/`. Each recipe describes environment variables to set, config files to create, or verification steps to run after installation.

When installing a tool, check for a matching recipe and apply it automatically.

Recipe matching is based on the resolved install target, not only the exact words the user typed. If the requested tool name is an alias, first map it to the actual scoop package and recipe. Example: a request to install `npm` maps to `nodejs` or `nodejs-lts`, then uses `references/recipes/nodejs.md`.

Likewise, a request to install `pip` maps to `python`, then uses `references/recipes/python.md`.

## Additional Resources

### Reference Files
- **`references/commands.md`** — Complete scoop command reference with detailed options and examples
- **`references/commands-zh.md`** — Chinese version of the command reference
- **`references/guide-zh.md`** — Chinese translation of this skill for easier understanding
- **`references/recipes/`** — Post-install configuration recipes for specific tools (added as needed)
