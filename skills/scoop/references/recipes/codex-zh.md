# codex — 安装配置方案（中文说明）

本文件是 `codex.md` 的中文对照版。实际执行以英文版为准。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install codex
```

## 安装后配置

Codex 使用 TOML 配置文件 `~/.codex/config.toml`，需要配置 API 连接信息和模型参数。

配置前按以下来源检查是否已有值：

1. **认证文件**：`~/.codex/auth.json` → `OPENAI_API_KEY`
2. **Codex 配置文件**：`~/.codex/config.toml` → `[model_providers.OpenAI]` 部分及顶层设置
3. **项目级配置**：当前项目中的 `.codex/config.toml`
4. **Windows 用户环境变量**：`OPENAI_API_KEY`、`OPENAI_BASE_URL`（遗留方式，不推荐）

如果任何来源有值，展示给用户（token 脱敏，如 `sk-...16f8`），并标注来源，通过 AskUserQuestion 询问：
- **保持现有** — 不改动
- **更新** — 提供新值（写入 `~/.codex/auth.json` 和 `~/.codex/config.toml`）
- **迁移到 auth.json** — 如果值在环境变量中，提议迁移到 `~/.codex/auth.json` 并清除环境变量（更干净，避免污染系统环境）

如果所有来源都没有值，询问用户提供。

### 需要收集的值

1. **API Base URL** — OpenAI 兼容的 API 服务端地址（如 `https://api.openai.com` 或自建代理地址）
2. **API Key** — OpenAI API 认证密钥（`OPENAI_API_KEY`）
3. **Model** — 使用的模型名称。询问用户前，先通过 WebSearch 搜索最新的 OpenAI 模型列表（搜索：`OpenAI latest models site:platform.openai.com`），将最新结果作为 AskUserQuestion 的选项供用户选择。用户也可以通过"Other"选项输入自定义模型名称

### 写入配置

创建或更新 `~/.codex/config.toml`：

```bash
powershell -Command '$configDir = "$env:USERPROFILE\.codex"; $configPath = "$configDir\config.toml"; if (-not (Test-Path $configDir)) { New-Item -ItemType Directory -Path $configDir -Force | Out-Null }; $content = @"
model_provider = "OpenAI"
model = "<model>"
review_model = "<model>"
model_reasoning_effort = "medium"
disable_response_storage = true
network_access = "enabled"
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "<base_url>"
wire_api = "responses"
"@; Set-Content -Path $configPath -Value $content -Encoding UTF8'
```

将 API Key 存入 `~/.codex/auth.json`（避免污染系统环境变量）：

```bash
powershell -Command '$authPath = "$env:USERPROFILE\.codex\auth.json"; @{ OPENAI_API_KEY = "<api_key>" } | ConvertTo-Json | Set-Content $authPath -Encoding UTF8'
```

Codex 运行时从环境变量读取 `OPENAI_API_KEY`。要从 `auth.json` 加载，需在启动 codex 前将密钥设置到进程环境中。可扩展 `run-cmd.ps1` 来加载 `auth.json`，或在 shell 配置文件中加载。

### 配置参数说明

#### 顶层设置

| 设置 | 说明 | 默认值 |
|------|------|--------|
| `model_provider` | 提供商名称（需匹配 `[model_providers.*]` 段） | `"OpenAI"` |
| `model` | 代码生成模型 | 用户选择（通过 WebSearch 获取最新列表） |
| `review_model` | 代码审查模型 | 与 `model` 相同 |
| `model_reasoning_effort` | 推理努力级别（`low`/`medium`/`high`/`xhigh`） | `"medium"` |
| `disable_response_storage` | 禁用服务端响应存储 | `true` |
| `network_access` | 允许网络访问（`"enabled"` / `"disabled"`） | `"enabled"` |
| `model_context_window` | 最大上下文窗口大小（token 数） | `1000000` |
| `model_auto_compact_token_limit` | 自动压缩触发阈值（token 数） | `900000` |

#### 提供商设置（`[model_providers.OpenAI]`）

| 设置 | 说明 | 默认值 |
|------|------|--------|
| `name` | 提供商显示名称 | `"OpenAI"` |
| `base_url` | API 端点地址 | `https://api.openai.com` |
| `wire_api` | API 协议格式（`"responses"` 或 `"chat"`） | `"responses"` |
| `requires_openai_auth` | 是否需要 `OPENAI_API_KEY` | `true` |

#### 认证文件（`~/.codex/auth.json`）

将 API 密钥与配置分开存储，格式：

```json
{
  "OPENAI_API_KEY": "sk-..."
}
```

密钥保存在文件中而非系统环境变量，避免泄露到所有进程。

### 验证

安装配置完成后，运行 `codex --version` 确认安装成功，然后运行 `codex` 验证能否成功连接 API。

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall codex
```

卸载后，通过 AskUserQuestion 询问用户是否清理残留配置：

- **保留** — 保留 `~/.codex/` 目录（config.toml + auth.json），以便将来重装时复用
- **清除** — 删除 `~/.codex/` 目录
- **先查看** — 展示当前配置和密钥（脱敏），然后再决定

如果用户环境变量中还有遗留的 `OPENAI_API_KEY` 或 `OPENAI_BASE_URL`，也提议一并清除。
