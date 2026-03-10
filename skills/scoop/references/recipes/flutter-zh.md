# Flutter — 安装后配置

## 何时安装

Flutter 是跨平台 UI 开发工具包。需要以下场景时安装：

- 开发 Flutter 移动应用
- 开发 Flutter Web 应用
- 开发 Flutter Windows 桌面应用
- 使用 Dart / Flutter 工具链管理项目

安装 `flutter` 只会提供 Flutter SDK 和相关命令行工具，不会自动补齐所有平台开发依赖。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install flutter
```

## 安装后配置

### 先验证 Flutter SDK 已可用

安装完成后，先确认 Flutter SDK 本体已经可执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version
```

### 使用 `flutter doctor` 检查缺失依赖

Flutter 的后续配置不应靠猜，而应以 `flutter doctor` 的输出为准：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor
```

`flutter doctor` 会告诉你当前缺哪些平台工具链或组件。

### 平台依赖按需处理

不要把所有平台依赖都当成 Flutter 的默认必装项。按用户目标平台处理：

- **Android** — 通常需要 Android Studio / Android SDK
- **Web** — 通常需要 Chrome
- **Windows 桌面** — 通常需要 Visual Studio 的 C++ 桌面开发工作负载

如果用户只做其中一个平台，就只补该平台缺失的依赖。

### Flutter 验证成功后，询问是否安装 `android-clt`

当以下验证已经通过时：

- `flutter --version`
- `flutter doctor` 至少确认 Flutter SDK 本体正常可用

通过 AskUserQuestion 询问用户是否还需要安装 Android command-line tools（`android-clt`）。

如果用户确认需要：

- 安装 `android-clt`
- 然后转到 `references/recipes/android-clt-zh.md`，继续执行其安装后配置流程

如果用户当前只做 Web 或 Windows 桌面开发，可以跳过这一步。

### 接受 Android 许可证（如需）

如果 `flutter doctor` 提示 Android licenses 未接受，再执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor --android-licenses
```

### 代理与镜像

如果 Flutter / Dart 包下载慢、`flutter doctor` 拉取依赖失败，代理和镜像统一交给 `proxy` skill 处理，不在本 recipe 中重复配置。

这部分通常涉及：

- `PUB_HOSTED_URL`
- `FLUTTER_STORAGE_BASE_URL`

## 验证

基础验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 dart --version
```

环境验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter doctor
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall flutter
```

卸载 `flutter` 本身不会自动移除外部平台依赖，例如 Android Studio、Android SDK、Chrome 或 Visual Studio。

如果用户要求进一步清理 Flutter / Dart 缓存或镜像环境变量，再单独确认后处理。
