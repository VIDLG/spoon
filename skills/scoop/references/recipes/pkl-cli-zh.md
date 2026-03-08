# pkl-cli — 安装配置方案（中文说明）

本文件是 `pkl-cli.md` 的中文对照版。实际执行以英文版为准。

## 何时安装

以下场景建议安装 pkl-cli：

- 用户使用 `.pkl` 配置文件
- 需要从结构化配置生成 JSON、YAML 或 Property Lists
- 提及 Pkl、Apple 配置语言、"配置即代码"
- 需要从命令行验证或评估 Pkl 模块

Pkl 是 Apple 推出的配置即代码语言，结合了声明式格式（JSON/YAML）的简洁性和编程语言特性（类、函数、条件、类型验证）。CLI 工具评估 `.pkl` 文件并输出为 JSON、YAML、plist 等格式。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install pkl-cli
```

## 安装后配置

pkl-cli 开箱即用，无需强制配置。可选设置：

### 编辑器支持

如果用户使用 VS Code，建议安装 Pkl 扩展以获得语法高亮、代码补全和验证。该扩展**未发布到 VS Code Marketplace**，需从 GitHub 手动安装：

1. 从 https://github.com/apple/pkl-vscode/releases/latest/ 下载最新 `.vsix` 文件
2. 在 VS Code 中：`Ctrl+Shift+P` → "Extensions: Install from VSIX..." → 选择下载的文件

注意：该扩展需要 **Java 22+** 来运行 Pkl Language Server。默认从 `PATH` 或 `JAVA_HOME` 查找 Java，也可通过 `pkl.lsp.java.path` 设置配置。

### 环境变量（可选）

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PKL_HOME` | Pkl 缓存和包的自定义目录 | `~/.pkl` |

`PKL_HOME` 很少需要设置，仅在用户想自定义缓存位置时使用。

### 验证

安装后运行 `pkl-cli --version` 确认安装成功。

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall pkl-cli
```

卸载后，通过 AskUserQuestion 询问用户是否清理残留数据：

- **保留** — 保留 `~/.pkl` 缓存目录，以便将来重装时复用
- **清除** — 删除 `~/.pkl` 目录
- **先查看** — 展示目录大小后再决定
