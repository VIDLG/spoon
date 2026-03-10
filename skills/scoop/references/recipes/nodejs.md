# Node.js / npm — Post-Install Recipe

## When to install

Node.js is the JavaScript runtime. Installing it also provides `node`, `npm`, and `npx`. Install it when you need to:

- Run JavaScript / TypeScript projects
- Install and manage frontend or Node.js project dependencies
- Run scripts from `package.json`
- Use `npx` to execute CLI tools
- Publish or maintain npm packages

npm should not be treated as a separate standalone install. The normal path is to install Node.js and get it together with `node` and `npx`.

## Install

Use AskUserQuestion to let the user choose which release line to install:

- **`nodejs` (default)** — current stable release, suitable when the user wants newer platform features or the latest ecosystem compatibility
- **`nodejs-lts`** — long-term support release, suitable for more conservative or stability-focused projects

If the user has no clear preference, default to `nodejs`:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install <chosen_package>
```

## Post-Install Configuration

### No required extra setup

After installation, `node`, `npm`, and `npx` should already be available. Do not add a separate npm installation step.

### Proxy and registry mirrors

If the user is in China, or npm downloads are slow or failing, do not edit `npm config` directly in this recipe.

Delegate all npm network configuration to the `proxy` skill, including:

- npm proxy settings (`proxy`, `https-proxy`)
- npm registry mirrors
- restoring the official registry

## Verify

First confirm that Node.js and npm are available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 node --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npx --version
```

If you need to verify npm network access:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm ping
```

## Uninstall

Uninstall whichever package was installed (`nodejs` or `nodejs-lts`):

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall <chosen_package>
```

After uninstalling, use AskUserQuestion to ask about leftover data:

- **Keep** — preserve the `~/.npm` cache and `~/.npmrc` config for future use
- **Remove** — delete these Node.js / npm user-level files
- **Preview** — show the config contents or cache size before deciding

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.npm") { Remove-Item -Path "$env:USERPROFILE\.npm" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:USERPROFILE\.npmrc") { Remove-Item -Path "$env:USERPROFILE\.npmrc" -Force }'
```
