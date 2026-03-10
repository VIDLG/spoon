# pixi — 安装后配置

## 何时安装

`pixi` 是一个面向项目和环境的包管理与工作区工具。需要以下场景时安装：

- 管理数据科学、Python、Rust 或多语言项目环境
- 在项目中使用 `pixi.toml` 和 `pixi.lock` 管理依赖
- 为项目提供可复现的开发环境
- 使用 Pixi 安装和管理全局命令行工具

安装 `pixi` 只会提供命令本体，不会自动创建项目工作区或环境。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install main/pixi
```

## 安装后配置

### 先验证 pixi 本体已可用

安装完成后，先确认 `pixi` 已可执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi --version
```

### 项目文件不是安装时自动生成的

`pixi` 的项目文件（如 `pixi.toml`、`pixi.lock`）属于工作区内容，不是安装 `pixi` 时自动创建的全局配置。

如果用户需要开始一个新工作区，可选地执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi init my_workspace
```

初始化工作区后，再按需添加依赖，例如：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi add python
```

### 不要把 `pixi install` 当成默认必做步骤

`pixi run`、`pixi shell` 等命令在需要时会自动安装环境，因此不需要把 `pixi install` 写成安装完成后的默认下一步。

### 全局工具（可选）

如果用户想用 Pixi 管理全局 CLI 工具，可选地执行：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi global install ruff
```

### 可选环境变量

`PIXI_HOME` 可用于自定义 Pixi 的全局数据目录。默认情况下无需设置。

只有在用户明确想把 Pixi 的全局环境和缓存移到自定义位置时，才考虑配置它。

### 更新方式

如果 `pixi` 是通过 Scoop 安装的，优先通过 Scoop 更新，而不是把 `pixi self-update` 作为默认更新路径：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop update pixi
```

### 代理与镜像

如果 Scoop 下载 `pixi` 失败，先按 `proxy` skill 处理 Scoop 网络问题。

Pixi 项目级的 channel / mirror / proxy 配置不在本 recipe 中统一展开。

## 验证

基础验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi --version
```

如果当前目录已经是 Pixi 工作区，可进一步验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi info
```

如果用户已经添加依赖，也可以验证环境求解是否正常：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pixi install
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall pixi
```

卸载 `pixi` 本身不应默认自动删除以下内容：

- `~/.pixi`
- 项目中的 `pixi.toml`
- 项目中的 `pixi.lock`
- 项目环境目录和全局工具环境

这些都属于用户数据或项目内容。如需进一步清理，应单独确认后再处理。
