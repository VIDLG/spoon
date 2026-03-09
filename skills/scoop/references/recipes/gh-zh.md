# gh (GitHub CLI) — 安装后配置

## 何时安装

安装 git 后**强烈推荐**安装 gh。它提供：

- GitHub release 下载（部分 recipe 如 pkl-cli 会用到）
- 仓库管理（clone、fork、PR、issue）
- 命令行访问 GitHub API
- 私有仓库认证

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install gh
```

## 安装后配置

### 登录 GitHub

gh 需要认证才能访问 GitHub API。提示用户登录：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth login
```

这会启动交互式登录流程，用户可选择：
- **GitHub.com** 或 GitHub Enterprise
- **HTTPS** 或 SSH 协议——推荐 **SSH**，实现免密 git 操作
- **浏览器**或 token 认证

选择 SSH 时，gh 会自动：
1. 生成 SSH 密钥（如果不存在，`~/.ssh/id_ed25519`）
2. 上传公钥到用户的 GitHub 账户
3. 配置 git 使用 SSH 访问 GitHub 仓库

登录后验证认证状态：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth status
```

如果选了 SSH，还需验证 SSH 连通性：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 ssh -T git@github.com
```

预期输出：`Hi <用户名>! You've successfully authenticated, but GitHub does not provide shell access.`

如果用户跳过登录，警告访问私有仓库或 GitHub API 的命令会失败。

## 验证

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 gh auth status
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall gh
```

卸载后询问用户是否清理残留配置：

- **保留** — 保留 `~/.config/gh/` 以备将来使用（含认证 token）
- **删除** — 删除 `~/.config/gh/` 目录
- **先看看** — 显示目录内容后再决定

如果用户选择删除：

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.config\gh") { Remove-Item -Path "$env:USERPROFILE\.config\gh" -Recurse -Force }'
```
