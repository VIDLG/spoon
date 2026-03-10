# android-clt — Post-Install Recipe

## When to install

`android-clt` provides the Android SDK Command-Line Tools. Install it when you need to:

- Use `sdkmanager` to install or manage Android SDK components
- Use `avdmanager` to manage Android virtual device definitions
- Provide the low-level Android CLI tooling required by Flutter, Gradle, or other Android workflows
- Get Android command-line capability without installing the full Android Studio IDE first

Installing `android-clt` only provides the command-line tools themselves. It does not automatically install the full set of Android SDK components.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install android-clt
```

## Post-Install Configuration

### First verify the command-line tools themselves

After installation, first confirm that `sdkmanager` is available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --version
```

If you also want to confirm AVD tooling availability:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 avdmanager --help
```

### Install Android SDK components only as needed

`android-clt` is not the same as a complete Android SDK setup. After installation, install only the components the user actually needs rather than treating everything as mandatory.

List available components first:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --list
```

Common optional components include:

- `platform-tools` — provides tools like `adb`
- `platforms;android-<api-level>` — a specific Android platform SDK
- `build-tools;<version>` — Android build toolchain packages
- `emulator` and a system image — only if the user needs an emulator

When installing components, use `sdkmanager` with explicit package names.

### Accept Android licenses if required

If the build tooling or downstream tools report unaccepted licenses, then run:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --licenses
```

### Do not conflate this with the full platform stack

`android-clt` is only the low-level command-line tool layer. Do not treat it as a default instruction to also install or configure:

- Android Studio
- the entire Android SDK component set
- the Flutter SDK
- Gradle or project-specific dependencies

Handle those separately based on the user's actual target workflow.

### Proxy and mirrors

If Scoop fails to download `android-clt`, or `sdkmanager` is slow when fetching packages, keep proxy and mirror handling centralized in the `proxy` skill rather than duplicating network configuration here.

## Verify

Basic verification:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --version
```

On-demand verification:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --list
```

If the user has installed `platform-tools`, you can also verify:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 adb version
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall android-clt
```

Uninstalling `android-clt` itself should not automatically remove installed Android SDK components, AVDs, `~/.android`, or other Android development data.

If the user wants to clean that data as well, confirm it separately before removing anything.
