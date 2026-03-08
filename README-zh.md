# Spoon

Windows 开发环境管理工具集，作为 Claude Code 插件运行。

Spoon 通过一组 skill 自动化软件安装、配置和维护，Claude Code 可在对话中调用这些 skill。

## Skills

### scoop

管理 [Scoop](https://scoop.sh/) 包管理器及通过它安装的所有软件。

- 安装/卸载/更新 scoop 和软件包
- Bucket 管理（添加、移除、列表）
- 健康检查和缓存清理
- 需要额外配置的工具提供安装后 recipe（如 claude-code、codex、pkl-cli）

### proxy

统一管理各开发工具的代理和镜像配置。

- HTTP/SOCKS5 代理：git、scoop、npm、pip、cargo、flutter 等
- 国内镜像源（TUNA、USTC、SJTUG）
- 跨工具统一启用/禁用

## 项目结构

```
spoon/
├── .claude-plugin/
│   ├── plugin.json          # 插件元数据
│   └── marketplace.json     # Marketplace 定义
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md          # Scoop skill 定义
│   │   └── references/
│   │       ├── commands.md       # 命令参考
│   │       ├── commands-zh.md    # 命令参考（中文）
│   │       ├── guide-zh.md       # Skill 指南（中文）
│   │       └── recipes/          # 安装后配置方案
│   │           ├── claude-code.md / claude-code-zh.md
│   │           ├── codex.md / codex-zh.md
│   │           └── pkl-cli.md / pkl-cli-zh.md
│   ├── proxy/
│   │   ├── SKILL.md          # Proxy skill 定义
│   │   └── references/
│   │       └── guide-zh.md       # Skill 指南（中文）
│   └── scripts/
│       ├── run-cmd.ps1       # 从注册表刷新 PATH 后运行命令
│       └── add-path.ps1      # 添加/移除 scoop 应用子目录到 PATH
├── CLAUDE.md                 # Claude Code 项目级指令
└── README.md
```

## 安装

在 Claude Code 中运行：

```
/plugin marketplace add VIDLG/spoon
```

然后从 marketplace 安装 spoon 插件。安装后在所有项目中可用。

### 团队配置

要让团队成员在项目中自动启用 spoon，在项目的 `.claude/settings.json` 中添加：

```json
{
  "extraKnownMarketplaces": {
    "spoon": {
      "source": {
        "source": "github",
        "repo": "VIDLG/spoon"
      }
    }
  },
  "enabledPlugins": {
    "spoon@spoon": true
  }
}
```

团队成员信任项目仓库后会自动提示安装。

## 系统要求

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## 许可证

MIT
