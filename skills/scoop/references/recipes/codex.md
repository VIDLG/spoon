# codex — Post-Install Recipe

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install codex
```

## Post-Install Configuration

Codex uses a TOML config file at `~/.codex/config.toml`. It requires API connection settings and model configuration.

Before asking the user, check all possible sources for existing values:

1. **Auth file**: `~/.codex/auth.json` → `OPENAI_API_KEY`
2. **Codex config file**: `~/.codex/config.toml` → `[model_providers.OpenAI]` section and top-level settings
3. **Project-level config**: `.codex/config.toml` in the current project
4. **Windows user environment variables**: `OPENAI_API_KEY`, `OPENAI_BASE_URL` (legacy, not recommended)

If values are found in any source, show them to the user (mask tokens, e.g., `sk-...16f8`) with where they came from, and ask via AskUserQuestion:
- **Keep current** — use existing values, skip configuration
- **Update** — provide new values (written to `~/.codex/auth.json` and `~/.codex/config.toml`)
- **Move to auth.json** — if values are in env vars, offer to move them into `~/.codex/auth.json` and remove the env vars (cleaner, avoids polluting system environment)

If no values are found, ask the user to provide them.

### Values to collect

1. **API Base URL** — OpenAI-compatible API endpoint (e.g., `https://api.openai.com` or a custom proxy)
2. **API Key** — OpenAI API authentication key (`OPENAI_API_KEY`)
3. **Model** — model name to use. Before asking the user, use WebSearch to find the latest OpenAI model names (search: `OpenAI latest models site:platform.openai.com`), then present the top results as AskUserQuestion options. The user can also enter a custom model name via the "Other" option

### Write configuration

Create or update `~/.codex/config.toml` with the collected values:

```bash
powershell -Command '$configDir = "$env:USERPROFILE\.codex"; $configPath = "$configDir\config.toml"; if (-not (Test-Path $configDir)) { New-Item -ItemType Directory -Path $configDir -Force | Out-Null }; $content = @"
model_provider = "OpenAI"
model = "<model>"
review_model = "<model>"
model_reasoning_effort = "medium"
disable_response_storage = true
network_access = "enabled"
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "<base_url>"
wire_api = "responses"
"@; Set-Content -Path $configPath -Value $content -Encoding UTF8'
```

Store the API key in `~/.codex/auth.json` (keeps credentials in a file, avoids polluting system environment variables):

```bash
powershell -Command '$authPath = "$env:USERPROFILE\.codex\auth.json"; @{ OPENAI_API_KEY = "<api_key>" } | ConvertTo-Json | Set-Content $authPath -Encoding UTF8'
```

Codex reads `OPENAI_API_KEY` from environment variables at runtime. To load it from `auth.json`, the key must be set in the process environment before launching codex. The `run-cmd.ps1` helper can be extended to source `auth.json`, or users can configure their shell profile to load it.

### Configuration Reference

#### Top-level settings

| Setting | Description | Default |
|---------|-------------|---------|
| `model_provider` | Provider name (must match a `[model_providers.*]` section) | `"OpenAI"` |
| `model` | Model to use for code generation | User-selected (fetched via WebSearch) |
| `review_model` | Model to use for code review | Same as `model` |
| `model_reasoning_effort` | Reasoning effort level (`low`, `medium`, `high`, `xhigh`) | `"medium"` |
| `disable_response_storage` | Disable server-side response storage | `true` |
| `network_access` | Allow network access (`"enabled"` / `"disabled"`) | `"enabled"` |
| `model_context_window` | Maximum context window size (tokens) | `1000000` |
| `model_auto_compact_token_limit` | Token threshold for auto-compaction | `900000` |

#### Provider settings (`[model_providers.OpenAI]`)

| Setting | Description | Default |
|---------|-------------|---------|
| `name` | Display name for the provider | `"OpenAI"` |
| `base_url` | API endpoint URL | `https://api.openai.com` |
| `wire_api` | API wire format (`"responses"` or `"chat"`) | `"responses"` |
| `requires_openai_auth` | Whether `OPENAI_API_KEY` is required | `true` |

#### Auth file (`~/.codex/auth.json`)

Stores API credentials separately from configuration. Format:

```json
{
  "OPENAI_API_KEY": "sk-..."
}
```

This file is read at launch to set the process environment. Keeping credentials in a file (rather than system environment variables) avoids leaking keys to all processes.

### Verify

After configuration, run `codex --version` to confirm the installation, then run `codex` to verify it connects to the API successfully.

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall codex
```

After uninstalling, use AskUserQuestion to ask the user about leftover configuration:

- **Keep** — preserve `~/.codex/` directory (config.toml + auth.json) for future reinstall
- **Remove** — delete `~/.codex/` directory
- **Show first** — display the current config and key (masked) so the user can decide

If the user chooses to remove:

```bash
powershell -Command 'Remove-Item -Path "$env:USERPROFILE\.codex" -Recurse -Force'
```

If the user also has legacy `OPENAI_API_KEY` or `OPENAI_BASE_URL` in Windows environment variables, offer to clean those up too:

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("OPENAI_API_KEY", [NullString]::Value, "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("OPENAI_BASE_URL", [NullString]::Value, "User")'
```
