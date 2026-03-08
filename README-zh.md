# Spoon

Windows 开发环境管理工具集，作为 Claude Code 插件运行。

Spoon 通过一组 skill 自动化软件安装、配置和维护，Claude Code 可在对话中调用这些 skill。

## Skills

### scoop

管理 [Scoop](https://scoop.sh/) 包管理器及通过它安装的所有软件。

- 安装/卸载/更新 scoop 和软件包
- Bucket 管理（添加、移除、列表）
- 健康检查和缓存清理
- 需要额外配置的工具提供安装后 recipe（如 claude-code、codex）

### proxy

统一管理各开发工具的代理和镜像配置。

- HTTP/SOCKS5 代理：git、scoop、npm、pip、cargo、flutter 等
- 国内镜像源（TUNA、USTC、SJTUG）
- 跨工具统一启用/禁用

## 项目结构

```
spoon/
├── .claude-plugin/
│   └── plugin.json          # 插件元数据
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md          # Scoop skill 定义
│   │   └── references/
│   │       ├── commands.md       # 命令参考
│   │       ├── commands-zh.md    # 命令参考（中文）
│   │       ├── guide-zh.md       # Skill 指南（中文）
│   │       └── recipes/          # 安装后配置方案
│   │           ├── claude-code.md / claude-code-zh.md
│   │           └── codex.md / codex-zh.md
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

在 Claude Code 用户设置（`~/.claude/settings.json`）中添加 spoon 为本地插件：

```json
{
  "enabledPlugins": {
    "spoon@local": true
  }
}
```

然后注册插件路径。Claude Code 会从 `.claude-plugin/plugin.json` 自动发现 skill。

## 系统要求

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## 许可证

MIT
