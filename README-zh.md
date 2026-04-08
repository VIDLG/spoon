# Spoon

用于管理 Windows 开发环境的 Claude Code 插件与仓库。

`Spoon` 是插件 / 仓库总名，`spoon.exe` 是从这个仓库发布的工作站初始化与管理可执行程序。

Spoon 现在只聚焦两类能力：

- Scoop 包管理器及其软件生态
- 常见开发工具的代理与镜像配置

AI 工作站初始化与后续工作站工具管理已经独立到仓库中的 Rust 项目 `spoon/`。生成的 `spoon.exe` 负责 Git、Claude Code、Codex、代理引导、工具链管理、capability 管理以及 AI 辅助 CLI 安装。

## Skills

### scoop

负责 [Scoop](https://scoop.sh/) 包管理器及其安装的软件。

- 安装、卸载、更新 scoop 和 scoop 软件包
- Bucket 管理（添加、移除、查看）
- 健康检查与缓存清理
- 需要额外配置的 scoop 软件 recipe（android-clt、flutter、nodejs、pixi、pkl-cli、rustup）

### ai-toolchain

`spoon.exe` 提供的工作站工具使用指南（git、claude、codex、gh、rg、fd、jq、yq、bat、delta、sg、uv、zed、scoop 工具链、MSVC capability）。

### proxy

负责统一管理代理和镜像配置。

- 为 git、scoop、npm、pip、cargo、flutter 等工具设置 HTTP/SOCKS5 代理
- 管理国内镜像源（TUNA、USTC、SJTUG）
- 统一启用 / 禁用代理配置

## 项目结构

```text
spoon/
├── spoon/                     # Rust CLI/TUI 二进制 crate
├── spoon-core/                # 共享基础设施（layout、download、gitx、archive、事件系统）
├── spoon-scoop/               # Scoop 领域逻辑（manifest、bucket、cache、包 workflow）
├── spoon-msvc/                # MSVC 领域逻辑（toolchain install/update/validate、MSI/CAB）
├── xtask/                     # 构建/部署自动化
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── commands.md / commands-zh.md
│   │       ├── guide-zh.md
│   │       └── recipes/            # 安装后配置 recipe（英文 + 中文）
│   │           └── android-clt, flutter, nodejs, pixi, pkl-cli, rustup
│   ├── proxy/
│   │   ├── SKILL.md
│   │   └── references/
│   │       └── guide-zh.md
│   └── ai-toolchain/
│       ├── SKILL.md
│       └── SKILL-zh.md
├── scripts/
│   ├── run-cmd.ps1
│   └── add-path.ps1
├── CLAUDE.md
├── README.md
└── README-zh.md
```

## Spoon

`spoon/` 目录包含 AI 工作站初始化与管理的 Rust CLI/TUI 项目。构建方式：

```text
cd spoon && cargo xtask deploy
```

这会编译 release 版本并将 `spoon.exe` 拷贝到仓库根目录（已 gitignore）。示例：

```text
.\spoon.exe
.\spoon.exe status
.\spoon.exe tools install --tools git,claude,codex,rg
```
## 安装

在 Claude Code 中运行：

```text
/plugin marketplace add VIDLG/spoon
```

然后从 marketplace 安装 spoon 插件。

## 系统要求

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## 许可证

MIT
