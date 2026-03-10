# Flutter — Post-Install Recipe

## When to install

Flutter is a cross-platform UI toolkit. Install it when you need to:

- Develop Flutter mobile applications
- Develop Flutter web applications
- Develop Flutter Windows desktop applications
- Use the Dart / Flutter toolchain for projects

Installing `flutter` provides the Flutter SDK and related CLI tools, but it does not automatically install every platform-specific dependency.

## Install

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install flutter
```

## Post-Install Configuration

### First verify the Flutter SDK itself

After installation, first confirm that the Flutter SDK binary is available:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version
```

### Use `flutter doctor` to detect missing dependencies

Do not guess the rest of the setup. Use `flutter doctor` as the source of truth:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor
```

`flutter doctor` will report which platform toolchains or components are still missing.

### Handle platform dependencies only as needed

Do not treat every platform dependency as mandatory for every Flutter installation. Follow the user's target platform:

- **Android** — usually requires Android Studio / Android SDK
- **Web** — usually requires Chrome
- **Windows desktop** — usually requires the Visual Studio C++ desktop workload

If the user only targets one platform, only install what that platform needs.

### After Flutter verification, ask whether to install `android-clt`

Once these checks have passed:

- `flutter --version`
- `flutter doctor` confirms that the Flutter SDK itself is working

Use AskUserQuestion to ask whether the user also wants the Android command-line tools (`android-clt`).

If the user confirms:

- install `android-clt`
- then continue with `references/recipes/android-clt.md` for the post-install flow

If the user only targets web or Windows desktop, this step can be skipped.

### Accept Android licenses if required

If `flutter doctor` reports unaccepted Android licenses, then run:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor --android-licenses
```

### Proxy and mirrors

If Flutter or Dart package downloads are slow, or dependency fetching fails, keep proxy and mirror handling centralized in the `proxy` skill rather than duplicating those settings here.

That usually involves:

- `PUB_HOSTED_URL`
- `FLUTTER_STORAGE_BASE_URL`

## Verify

Basic verification:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 dart --version
```

Environment verification:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor
```

## Uninstall

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall flutter
```

Uninstalling `flutter` itself does not automatically remove external platform dependencies such as Android Studio, Android SDK, Chrome, or Visual Studio.

If the user wants to clean Flutter / Dart caches or mirror-related environment variables as well, confirm that separately before removing them.
