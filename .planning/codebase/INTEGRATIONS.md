# External Integrations

**Analysis Date:** 2026-03-28

## APIs & External Services

**AI Services:**
- Anthropic Claude API
  - SDK: Direct HTTP via `reqwest`
  - Auth: `ANTHROPIC_AUTH_TOKEN` env var or `.claude/settings.json`
  - Base URL: `https://api.anthropic.com` (configurable)

- OpenAI Codex API
  - SDK: Direct HTTP via `reqwest`
  - Auth: `OPENAI_API_KEY` env var or codex auth file
  - Base URL: `https://api.openai.com` (configurable)

**Development Tools:**
- Git
  - Integration: Via `gix` crate for Git operations
  - Configuration: Managed through `~/.gitconfig`
  - Proxy support: HTTP/HTTPS proxy configuration

## Data Storage

**Databases:**
- None detected - Uses file-based configuration
- Config files stored in `~/.spoon/` directory

**File Storage:**
- Local filesystem only
- Scoop manifests and packages via Windows filesystem

**Caching:**
- No explicit caching system detected
- File system caching for downloaded packages

## Authentication & Identity

**Auth Providers:**
- Custom auth implementation
  - API tokens stored in config files or environment variables
  - No external OAuth providers detected

**Token Management:**
- Claude API tokens: `ANTHROPIC_AUTH_TOKEN`
- OpenAI API tokens: `OPENAI_API_KEY`
- Git credentials: Standard Git credential helpers

## Monitoring & Observability

**Error Tracking:**
- Built-in tracing via `tracing` crate
- Custom error handling with `anyhow`
- No external error tracking service detected

**Logs:**
- Terminal output via `tracing-appender`
- TUI logging integration via `tui-logger`
- No centralized logging system

## CI/CD & Deployment

**Hosting:**
- Claude Code plugin marketplace
- Local binary distribution via `spoon.exe`

**CI Pipeline:**
- Manual build process via Cargo
- No automated CI detected

## Environment Configuration

**Required env vars:**
- `ANTHROPIC_AUTH_TOKEN` - Claude API authentication
- `ANTHROPIC_BASE_URL` - Optional Claude API endpoint
- `OPENAI_API_KEY` - OpenAI/Codex API authentication
- `OPENAI_BASE_URL` - Optional OpenAI API endpoint
- `PATH` - System PATH for executable discovery

**Secrets location:**
- Config files: `~/.spoon/config.toml`, `~/.claude/settings.json`
- Environment variables preferred over file storage

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- HTTP requests to:
  - Anthropic API (`/messages` endpoint)
  - OpenAI API (`/chat/completions` endpoint)
  - Git repositories via `gix` crate

---

*Integration audit: 2026-03-28*
```