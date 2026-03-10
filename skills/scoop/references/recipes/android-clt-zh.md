# android-clt — 安装后配置

## 何时安装

`android-clt` 提供 Android SDK Command-Line Tools。需要以下场景时安装：

- 需要用 `sdkmanager` 安装或管理 Android SDK 组件
- 需要用 `avdmanager` 管理 Android 虚拟设备定义
- 需要为 Flutter、Gradle 或其他 Android 工具链补齐底层命令行 SDK 工具
- 不想安装完整 Android Studio，只想先具备基础命令行能力

安装 `android-clt` 只会提供 command-line tools，本身不会自动安装所有 Android SDK 组件。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install android-clt
```

## 安装后配置

### 先验证 command-line tools 已可用

安装完成后，先确认 `sdkmanager` 已可执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --version
```

如需确认 AVD 相关命令也可用：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 avdmanager --help
```

### 按需安装 Android SDK 组件

`android-clt` 不等于完整 Android SDK。安装完成后，应按用户目标安装所需组件，而不是默认全部安装。

先列出可用组件：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --list
```

常见按需组件包括：

- `platform-tools` — 提供 `adb` 等工具
- `platforms;android-<api-level>` — 指定 Android 平台 SDK
- `build-tools;<version>` — 构建工具链
- `emulator` 和 system image — 只有需要模拟器时才安装

安装组件时，使用 `sdkmanager` 指定明确包名。

### 接受 Android 许可证（如需）

如果构建工具或下游工具链提示 licenses 未接受，再执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --licenses
```

### 平台依赖不要混为一谈

`android-clt` 只是底层命令行工具，不应默认替用户安装或配置以下内容：

- Android Studio
- 完整 Android SDK 组件集合
- Flutter SDK
- Gradle 或项目级依赖

这些都应根据用户实际目标再分别处理。

### 代理与镜像

如果 Scoop 下载 `android-clt` 失败，或 `sdkmanager` 拉取组件很慢，代理和镜像仍然统一交给 `proxy` skill 处理，不在本 recipe 中重复配置。

## 验证

基础验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --version
```

按需验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 sdkmanager --list
```

如果用户已安装 `platform-tools`，还可以进一步验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 adb version
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall android-clt
```

卸载 `android-clt` 本身不应默认自动删除已安装的 Android SDK 组件、AVD、`~/.android` 或其他 Android 开发数据。

如果用户要求进一步清理这些数据，再单独确认后处理。
