# Scoop Command Reference

Complete reference for all scoop subcommands.

## Core Commands

### scoop install
Install one or more apps.
```
scoop install <app> [options]
scoop install extras/<app>          # install from specific bucket
scoop install https://url/app.json  # install from URL manifest
```
Options:
- `-g` / `--global` — install globally (requires admin)
- `-k` / `--no-cache` — don't use download cache
- `-s` / `--skip` — skip hash validation (not recommended)
- `-a <arch>` — specify architecture: `32bit` or `64bit`

### scoop uninstall
Uninstall an app.
```
scoop uninstall <app> [options]
```
Options:
- `-g` / `--global` — uninstall a globally installed app
- `-p` / `--purge` — also remove persistent data

### scoop update
Update scoop and/or apps.
```
scoop update              # update scoop itself + all bucket manifests
scoop update <app>        # update a specific app
scoop update *            # update all installed apps
```
Options:
- `-g` / `--global` — update globally installed apps
- `-f` / `--force` — force update even if up-to-date
- `-k` / `--no-cache` — don't use download cache
- `-q` / `--quiet` — suppress output

### scoop search
Search for available apps.
```
scoop search <query>      # search by name (supports regex)
```

### scoop list
List installed apps.
```
scoop list                # list all installed apps
scoop list <query>        # filter by name
```

### scoop info
Show detailed information about an app.
```
scoop info <app>
```

### scoop status
Show apps that have updates available.
```
scoop status
```

## Bucket Commands

### scoop bucket add
Add a bucket (software repository).
```
scoop bucket add <name>              # add a known bucket
scoop bucket add <name> <git-url>    # add a custom bucket
```

Known official buckets: `main`, `extras`, `versions`, `java`, `nerd-fonts`, `nirsoft`, `sysinternals`, `php`, `nonportable`, `games`.

### scoop bucket rm
Remove a bucket.
```
scoop bucket rm <name>
```

### scoop bucket list
List all added buckets.
```
scoop bucket list
```

### scoop bucket known
List all known official buckets.
```
scoop bucket known
```

## Maintenance Commands

### scoop cleanup
Remove old versions of apps to free disk space.
```
scoop cleanup <app>       # cleanup a specific app
scoop cleanup *           # cleanup all apps
```
Options:
- `-g` / `--global` — cleanup globally installed apps
- `-k` / `--cache` — also remove outdated download cache

### scoop cache
Manage the download cache.
```
scoop cache show          # show cache contents
scoop cache show <app>    # show cache for specific app
scoop cache rm <app>      # remove cache for specific app
scoop cache rm *          # clear entire cache
```

### scoop checkup
Run a health check on scoop. Reports potential issues with the installation.
```
scoop checkup
```

### scoop reset
Reset an app — re-creates shims and shortcuts. Useful for fixing broken app links or switching between versions.
```
scoop reset <app>
scoop reset *             # reset all apps
```

### scoop hold / unhold
Prevent or allow an app from being updated.
```
scoop hold <app>          # prevent updates
scoop unhold <app>        # allow updates again
```

## Utility Commands

### scoop which
Show the path of a command installed by scoop.
```
scoop which <command>
```

### scoop home
Open the homepage of an app in the default browser.
```
scoop home <app>
```

### scoop prefix
Show the install path of an app.
```
scoop prefix <app>
```

### scoop cat
Show the manifest of an app.
```
scoop cat <app>
```

### scoop depends
Show the dependency tree of an app.
```
scoop depends <app>
```

### scoop export / import
Export or import the list of installed apps.
```
scoop export > scoopfile.json    # export installed apps
scoop import scoopfile.json      # import and install from file
```

### scoop config
Manage scoop configuration.
```
scoop config                    # show all config
scoop config <key>              # show a config value
scoop config <key> <value>      # set a config value
scoop config rm <key>           # remove a config value
```

Common config keys:
- `proxy` — HTTP proxy (e.g., `127.0.0.1:7890`)
- `aria2-enabled` — enable aria2 for faster downloads (`true`/`false`)
- `SCOOP_REPO` — custom scoop repository URL
- `SCOOP_BRANCH` — scoop branch to use (`master`/`develop`)

### scoop alias
Manage custom command aliases.
```
scoop alias add <name> <command> <description>
scoop alias rm <name>
scoop alias list
```

## Common Patterns

### Set up a new machine
```bash
# Install scoop, add buckets, import apps
scoop bucket add extras
scoop bucket add versions
scoop import scoopfile.json
```

### Keep everything updated
```bash
scoop update
scoop update *
scoop cleanup *
scoop cache rm *
```

### Switch between app versions
```bash
scoop install versions/python27
scoop reset python27    # switch to python 2.7
scoop reset python      # switch back to latest python
```
