# rustup — Post-Install Recipe

## When to install

rustup is the recommended way to install Rust on Windows. Install it when you need:

- Rust programming language toolchain (cargo, rustc, rustfmt)
- Building Rust projects
- Cross-compilation support
- Multiple Rust toolchain versions management

## Pre-Install Configuration (Recommended for China)

If the user is in China or Rust downloads are slow, delegate mirror/proxy setup to the `proxy` skill before installation.

That skill owns both of the Rust network settings:

- `rustup` distribution mirrors (`RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`)
- Cargo crate mirrors (`~/.cargo/config.toml`)

Keep those settings centralized there so Rust mirrors stay consistent with the rest of the system.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install rustup
```

### Verify rustup is installed

First confirm that the `rustup` binary itself is available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustup --version
```

## Post-Install Configuration

### Initialize the default toolchain

On first run, rustup will prompt you to choose:

1. **Default toolchain** — typically `stable` (recommended)
2. **Other options** — keep the defaults unless you know you need something different

This step installs the default Rust toolchain, which already includes `rustc` and `cargo`. Do not treat Cargo as a separate installation step.

Run the default invocation to complete setup:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustup default stable
```

### Configure PATH

Scoop automatically shims rustup binaries. If you need full toolchain access (like `rustup.exe` in `~/.cargo/bin`), it's already configured by scoop.

After initialization, verify that `rustc` and `cargo` are available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustc --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo --version
```

## Verify Rust network access

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo search clap
```

If mirrors or proxy were configured earlier, change or restore them through the `proxy` skill rather than editing Rust mirror files in this recipe.

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall rustup
```

After uninstalling, use AskUserQuestion to ask about leftover data:

- **Keep** — preserve `~/.rustup/` and `~/.cargo/` for future reinstallation
- **Remove** — delete these directories completely

If the user chooses to remove:

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.rustup") { Remove-Item -Path "$env:USERPROFILE\.rustup" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:USERPROFILE\.cargo") { Remove-Item -Path "$env:USERPROFILE\.cargo" -Recurse -Force }'
```
