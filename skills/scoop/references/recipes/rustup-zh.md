# rustup — 安装后配置

## 何时安装

rustup 是 Windows 上安装 Rust 的推荐方式。需要以下场景时安装：

- Rust 编程语言工具链（cargo、rustc、rustfmt）
- 构建 Rust 项目
- 跨平台编译支持
- 多版本 Rust 工具链管理

## 安装前配置（推荐在国内使用）

在国内使用或 Rust 下载较慢时，应在安装前把镜像/代理配置交给 `proxy` skill 处理。

Rust 相关的网络设置统一由它管理，包括：

- `rustup` 工具链分发镜像（`RUSTUP_DIST_SERVER`、`RUSTUP_UPDATE_ROOT`）
- Cargo crate 镜像（`~/.cargo/config.toml`）

这样可以避免 Rust 镜像配置和系统其他工具的代理/镜像策略分散、冲突。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install rustup
```

### 验证 rustup 已安装

先确认 `rustup` 本体已经安装成功：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustup --version
```

## 安装后配置

### 初始化默认工具链

首次运行时，rustup 会提示你选择：

1. **默认工具链** — 推荐选择 `stable`
2. **其他选项** — 不确定时保持默认即可

这一步会安装默认 Rust 工具链，其中已经包含 `rustc` 和 `cargo`，不需要单独安装 cargo。

运行以下命令完成设置：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustup default stable
```

### 配置 PATH

Scoop 会自动 shim rustup 的二进制文件。如果需要完整工具链访问（如 `~/.cargo/bin` 中的 `rustup.exe`），scoop 已自动配置。

初始化完成后，再验证 `rustc` 和 `cargo` 是否可用：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 rustc --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo --version
```

## 验证 Rust 网络访问

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo search clap
```

如果此前配置过镜像或代理，应回到 `proxy` skill 中调整或恢复官方源，而不是在本 recipe 里直接改 Rust 镜像文件。

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall rustup
```

卸载后询问用户是否清理残留数据：

- **保留** — 保留 `~/.rustup/` 和 `~/.cargo/` 以便将来重新安装
- **删除** — 完全删除这些目录

如果用户选择删除：

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.rustup") { Remove-Item -Path "$env:USERPROFILE\.rustup" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:USERPROFILE\.cargo") { Remove-Item -Path "$env:USERPROFILE\.cargo" -Recurse -Force }'
```
