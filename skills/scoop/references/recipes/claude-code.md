# claude-code — Post-Install Recipe

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install claude-code
```

## Post-Install Configuration

claude-code requires API connection settings. Before asking the user, check all possible sources for existing values:

1. **Windows user environment variables**: `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`
2. **User-level settings**: `~/.claude/settings.json` → `env` section
3. **Project-level settings**: `.claude/settings.json` or `.claude/settings.local.json` → `env` section

If values are found in any source, show them to the user (mask tokens, e.g., `sk-...b75e6`) with where they came from, and ask via AskUserQuestion:
- **Keep current** — use existing values, skip configuration
- **Update** — provide new values (written to `~/.claude/settings.json`)
- **Move to settings** — if values are in env vars, offer to move them into `~/.claude/settings.json` and remove the env vars (cleaner approach)

If no values are found, ask the user to provide them.

Values to collect:

1. **API Base URL** — API endpoint (e.g., `https://api.anthropic.com` or a custom proxy)
2. **API Auth Token** — API authentication key

Write the configuration to the user-level Claude Code settings file `~/.claude/settings.json` under the `env` section:

```bash
# Read existing settings (if any), merge env entries, and write back
powershell -Command '$settingsPath = "$env:USERPROFILE\.claude\settings.json"; if (Test-Path $settingsPath) { $settings = Get-Content $settingsPath | ConvertFrom-Json } else { New-Item -ItemType Directory -Path "$env:USERPROFILE\.claude" -Force | Out-Null; $settings = [PSCustomObject]@{} }; if (-not $settings.PSObject.Properties["env"]) { $settings | Add-Member -NotePropertyName "env" -NotePropertyValue ([PSCustomObject]@{}) }; $settings.env | Add-Member -NotePropertyName "ANTHROPIC_BASE_URL" -NotePropertyValue "<base_url>" -Force; $settings.env | Add-Member -NotePropertyName "ANTHROPIC_AUTH_TOKEN" -NotePropertyValue "<auth_token>" -Force; $settings | ConvertTo-Json -Depth 10 | Set-Content $settingsPath -Encoding UTF8'
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_BASE_URL` | API endpoint URL | `https://api.anthropic.com` |
| `ANTHROPIC_AUTH_TOKEN` | API authentication key | None (required) |

### Verify

After configuration, run `claude-code --version` to confirm the installation.

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall claude-code
```

After uninstalling, use AskUserQuestion to ask the user about leftover configuration in `~/.claude/settings.json` (`env.ANTHROPIC_BASE_URL`, `env.ANTHROPIC_AUTH_TOKEN`):

- **Keep** — preserve for future reinstall
- **Remove** — delete the env entries from `~/.claude/settings.json`
- **Show first** — display the current values (masked) so the user can decide

If the user had values set as Windows environment variables, also ask whether to remove those:

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("ANTHROPIC_BASE_URL", [NullString]::Value, "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("ANTHROPIC_AUTH_TOKEN", [NullString]::Value, "User")'
```
