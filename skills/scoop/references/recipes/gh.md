# gh (GitHub CLI) — Post-Install Recipe

## When to install

gh is **strongly recommended** after installing git. It enables:

- GitHub release downloads (used by some post-install recipes like pkl-cli)
- Repository management (clone, fork, PR, issue)
- GitHub API access from the command line
- Authentication for private repos

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install gh
```

## Post-Install Configuration

### Authenticate with GitHub

gh requires authentication to access GitHub APIs. Use AskUserQuestion to ask the user to log in:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth login
```

This launches an interactive flow. The user can choose:
- **GitHub.com** or GitHub Enterprise
- **HTTPS** or SSH protocol — recommend **SSH** for passwordless git operations
- **Browser** or token-based authentication

When the user chooses SSH, gh will:
1. Generate an SSH key if none exists (`~/.ssh/id_ed25519`)
2. Upload the public key to the user's GitHub account
3. Configure git to use SSH for GitHub repos

After login, verify the authentication status:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth status
```

If SSH was selected, also verify SSH connectivity:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 ssh -T git@github.com
```

Expected output: `Hi <username>! You've successfully authenticated, but GitHub does not provide shell access.`

If the user skips login, warn that commands accessing private repos or GitHub APIs will fail.

## Verify

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth status
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall gh
```

After uninstalling, use AskUserQuestion to ask about leftover config:

- **Keep** — preserve `~/.config/gh/` for future use (contains auth tokens)
- **Remove** — delete `~/.config/gh/` directory
- **Show first** — display directory contents before deciding

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.config\gh") { Remove-Item -Path "$env:USERPROFILE\.config\gh" -Recurse -Force }'
```
