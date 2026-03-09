# git — 安装后配置

## 何时安装

git 是 scoop 的**必要依赖**——bucket 操作（添加、更新）都依赖 git。安装 scoop 后必须立即安装 git。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install git
```

## 安装后配置

### 把 git 工具加到 PATH

scoop 只 shim 了 `git`、`sh`、`git-bash` 等少数几个，`bash.exe` 和 Unix 工具（`less`、`awk` 等）在 git 自己的目录下，需要手动加 PATH：

```bash
powershell -File <plugin_root>/skills/scripts/add-path.ps1 git bin usr/bin
```

### 配置 git

1. **默认分支设为 main**（直接设置）：
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global init.defaultBranch main
   ```

2. **询问用户姓名和邮箱**（git 提交必需），跳过则警告提交会失败：
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global user.name '<姓名>'
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global user.email '<邮箱>'
   ```

3. 如果已有 `~/.gitconfig`，先展示内容，避免覆盖。

## 验证

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git --version
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall git
```

移除安装时添加的 PATH 条目：

```bash
powershell -File <plugin_root>/skills/scripts/add-path.ps1 git bin usr/bin -Remove
```

注意：卸载 scoop 时，PATH 清理中的 `-notmatch "Scoop"` 会自动移除这些条目（因为路径都包含 "Scoop"）。`-Remove` 仅用于单独卸载 git 而保留 scoop 的情况。

卸载后询问用户是否清理残留配置：

- **保留** — 保留 `~/.gitconfig` 以备将来使用
- **删除** — 删除 `~/.gitconfig`
- **先看看** — 显示内容后再决定

如果用户选择删除：

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.gitconfig") { Remove-Item -Path "$env:USERPROFILE\.gitconfig" -Force }'
```
