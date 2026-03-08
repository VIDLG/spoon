# claude-code — 安装配置方案（中文说明）

本文件是 `claude-code.md` 的中文对照版。实际执行以英文版为准。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install claude-code
```

## 安装后配置

claude-code 需要配置 API 连接信息。配置前按以下来源检查是否已有值：

1. **Windows 用户环境变量**：`ANTHROPIC_BASE_URL`、`ANTHROPIC_AUTH_TOKEN`
2. **用户级设置**：`~/.claude/settings.json` → `env` 部分
3. **项目级设置**：`.claude/settings.json` 或 `.claude/settings.local.json` → `env` 部分

如果任何来源有值，展示给用户（token 脱敏，如 `sk-...b75e6`），并标注来源，通过 AskUserQuestion 询问：
- **保持现有** — 不改动
- **更新** — 提供新值（写入 `~/.claude/settings.json`）
- **迁移到 settings** — 如果值在环境变量中，提议迁移到 `~/.claude/settings.json` 并清除环境变量（更干净）

如果所有来源都没有值，询问用户提供。

需要收集的值：

1. **API Base URL** — API 服务端地址（如 `https://api.anthropic.com` 或自建代理地址）
2. **API Auth Token** — API 认证密钥

将配置写入用户级 Claude Code 设置文件 `~/.claude/settings.json` 的 `env` 部分。脚本会读取现有设置、合并新条目、写回文件。

### 环境变量说明

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `ANTHROPIC_BASE_URL` | API 服务端地址 | `https://api.anthropic.com` |
| `ANTHROPIC_AUTH_TOKEN` | API 认证密钥 | 无（必填） |

### 验证

安装配置完成后，运行 `claude-code --version` 确认安装成功。

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall claude-code
```

卸载后，通过 AskUserQuestion 询问用户是否清理 `~/.claude/settings.json` 中的残留配置（`env.ANTHROPIC_BASE_URL`、`env.ANTHROPIC_AUTH_TOKEN`）：

- **保留** — 留着以便将来重装时复用
- **清除** — 从 `~/.claude/settings.json` 中删除相关条目
- **先查看** — 展示当前值（脱敏），然后再决定

如果用户的值存储在 Windows 环境变量中，也询问是否一并清除。
