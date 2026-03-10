# Python / pip — 安装后配置

## 何时安装

Python 是通用编程语言运行时，安装后会同时提供 `python` 和 `pip`。需要以下场景时安装：

- 运行 Python 项目或脚本
- 安装和管理 Python 包
- 使用 `pip` 安装命令行工具或库
- 为项目创建虚拟环境

`pip` 不作为独立软件单独安装。默认通过安装 Python 一起获得。

## 安装

默认安装主线 Python：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install python
```

如果用户明确需要特定 Python 大版本或旧版本，再搜索并安装对应的版本化包，而不是把旧版本作为默认选择。

## 安装后配置

### 默认无需额外配置

安装完成后，`python` 和 `pip` 应已可用，不需要单独安装 pip。

### 推荐使用 `python -m pip`

执行 pip 操作时，优先使用 `python -m pip`，这样可以确保 pip 作用于当前这个 Python 解释器：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m pip --version
```

### 虚拟环境（推荐用于项目）

对于项目依赖，优先使用虚拟环境而不是把包直接装到全局环境：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m venv .venv
```

### 代理与镜像

如果用户在国内，或 PyPI 访问慢、下载失败，不要在本 recipe 中直接改 `pip config`。

代理和镜像统一交给 `proxy` skill 处理，包括：

- pip 代理
- PyPI 镜像
- 恢复官方 PyPI 源

## 验证

先确认 Python 和 pip 都已可用：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 python -m pip --version
```

如果用户坚持使用 `pip` 命令，也可以补充验证：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip --version
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall python
```

卸载后询问用户是否清理残留数据：

- **保留** — 保留 pip 缓存和用户配置，便于将来继续使用
- **删除** — 删除 pip 用户级缓存和配置
- **先看看** — 先展示相关目录后再决定

如果用户选择删除：

```bash
powershell -Command 'if (Test-Path "$env:LOCALAPPDATA\pip\Cache") { Remove-Item -Path "$env:LOCALAPPDATA\pip\Cache" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:APPDATA\pip") { Remove-Item -Path "$env:APPDATA\pip" -Recurse -Force }'
```
