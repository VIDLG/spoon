# Scoop 包管理器 — 中文说明

本文件是 SKILL.md 的中文对照版，方便理解 skill 的工作流程。实际执行以 SKILL.md 为准。

---

## 副作用报告

**关键规则**：任何改变系统状态的操作（安装、卸载、更新、bucket 变更等）完成后，必须向用户汇总副作用清单：

- 新增、修改或删除的环境变量
- PATH 中新增或删除的条目
- 创建或删除的目录
- 创建或移除的 shim（命令行快捷方式）
- 新增或移除的桌面/开始菜单快捷方式
- 其他系统级变更

## 安装 scoop

### 第一步：检测是否已安装

运行 `scoop --version`。

- 已安装 → 报告版本和路径，询问用户要**更新**、**重新配置**还是**跳过**
- 未安装 → 继续安装

### 第二步：确认安装路径

让用户选择：

- `D:\Scoop`（推荐）— 独立目录，不占 C 盘
- `~/scoop` — scoop 默认位置
- 自定义路径

### 第三步：检查目标目录

安装前检查用户选择的目录是否已存在：

- **目录存在且是 scoop 安装**（含 `apps/`、`shims/` 等）→ 询问：**复用**（跳过安装，直接验证）、**清除重装**、还是**换路径**
- **目录存在但不是 scoop** → 警告目录非空，询问：**继续使用**、**清除后安装**、还是**换路径**
- **目录不存在** → 正常继续

### 第四步：执行安装

```powershell
powershell -Command "irm get.scoop.sh -outfile 'install.ps1'; .\install.ps1 -ScoopDir '<路径>'"
```

安装脚本**不会**设置 `SCOOP` 环境变量，需要在安装后手动设置：

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("SCOOP", "<安装路径>", "User")'
```

常用参数：
- `-ScoopDir` — 安装目录
- `-NoProxy` — 跳过代理

需要管理员权限的参数（默认不用）：
- `-ScoopGlobalDir` — 全局安装目录
- `-RunAsAdmin` — 管理员模式

**Shell 环境说明**：安装程序通过 `[Environment]::SetEnvironmentVariable` 将环境变量写入 Windows 注册表。但 Claude Code 的 Bash 工具继承的是**父进程 VSCode** 的环境，而不是注册表。这意味着：

- 同一个 VSCode 窗口内（即使开新对话）：bash 看不到新的 PATH/SCOOP
- 重启 VSCode 后：bash 能获取到新环境
- 即使 `powershell -Command "scoop ..."` 也可能不行，因为新 PowerShell 进程从 bash（父进程）继承 PATH，而不是从注册表

**推荐方案**：使用 `skills/scripts/run-cmd.ps1`（相对于 plugin 根目录）辅助脚本，在运行命令前从注册表刷新 PATH。这样可以避免 bash/PowerShell 之间 `$env`、`$null` 等引号冲突。

根据 plugin 根目录解析 `skills/scripts/run-cmd.ps1` 的绝对路径，然后通过 `powershell -File` 调用：

```bash
# 示例（将 <plugin_root> 替换为 plugin 的绝对路径）：
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install git
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop bucket add extras
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global init.defaultBranch main
```

对于非 scoop 的 PowerShell 命令（如设置环境变量），用 bash 单引号包裹以防止 `$` 被 bash 解释：

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("SCOOP", "D:\Scoop", "User")'
```

**bash 与 PowerShell 混用注意事项**：
- `$env:Path`、`$null` 等会被 bash 吞掉 — 用单引号或 `.ps1` 文件
- 清除环境变量时用 `[NullString]::Value` 代替 `$null`
- 复杂命令优先用 `-File` 而非 `-Command`

或者直接刷新 bash 环境：

```bash
export SCOOP=$(powershell -Command "[Environment]::GetEnvironmentVariable('SCOOP', 'User')")
export PATH="$SCOOP/shims:$PATH"
```

建议安装完成后**重启 VSCode**，这样后续所有会话都能直接使用。

### 第五步：安装 git 和 gh

scoop 的 bucket 由 git 管理，必须先安装 git 才能进行 bucket 操作。gh 强烈推荐，用于 GitHub 集成。

→ 参见 `recipes/git-zh.md`（必须）
→ 参见 `recipes/gh-zh.md`（强烈推荐）

### 第六步：确认并添加 bucket

让用户多选：

- `extras`（推荐）— 常用 GUI 软件和开发工具
- `versions` — 软件历史版本
- `java` — JDK 发行版
- `nerd-fonts` — 开发者字体

### 第七步：运行 scoop update

添加 bucket 后执行 `scoop update`，通过 git 拉取最新的 bucket 清单：

```bash
powershell -Command "scoop update"
```

这一步验证 git 与 scoop 配合正常，并确保所有 bucket 数据是最新的。

### 安装后副作用汇总

向用户报告：
- `SCOOP` 环境变量 → `<安装路径>`
- `<安装路径>\shims` 已加入 PATH
- git 安装在 `<安装路径>\apps\git\current`
- Bucket 克隆到 `<安装路径>\buckets\`
- 创建的目录：`apps`、`buckets`、`cache`、`persist`、`shims`

## 卸载 scoop

不可逆操作，必须二次确认。

1. 确认用户意图
2. `powershell -Command "scoop uninstall scoop"`
3. 清除 `SCOOP` 环境变量（bash 中用 `[NullString]::Value` 代替 `$null`，因为 `$null` 会被 bash 吞掉）：
   ```bash
   powershell -Command "[Environment]::SetEnvironmentVariable('SCOOP', [NullString]::Value, 'User')"
   ```
4. 如有全局安装，清除 `SCOOP_GLOBAL`（需管理员权限）：
   ```bash
   powershell -Command "[Environment]::SetEnvironmentVariable('SCOOP_GLOBAL', [NullString]::Value, 'Machine')"
   ```
5. 清理 PATH 中 scoop 相关条目：
   ```bash
   powershell -Command '$path = [Environment]::GetEnvironmentVariable("PATH", "User"); $cleaned = ($path -split ";" | Where-Object { $_ -notmatch "Scoop" }) -join ";"; [Environment]::SetEnvironmentVariable("PATH", $cleaned, "User")'
   ```
6. 删除安装目录。**注意**：scoop 使用 NTFS junction（如 `current` → 版本目录），PowerShell 的 `Remove-Item -Recurse -Force` 无法删除 junction，必须用 `cmd /c rmdir /s /q`：
   ```bash
   powershell -Command "& cmd /c 'rmdir /s /q <安装路径>'"
   ```
7. 报告所有副作用

## 代理与镜像配置

代理和镜像管理由 **`proxy` skill** 统一处理，覆盖所有工具（git、scoop、npm、pip、cargo、flutter 等）。

当 scoop 操作因网络/SSL 错误失败，或用户询问代理/镜像设置时，交给 `proxy` skill 处理。

安装 scoop 时（第五步，git 装好后），应调用 `proxy` skill 检测已有代理并同步到 scoop。

## 日常操作

| 操作 | 命令 |
|------|------|
| 搜索软件 | `powershell -Command "scoop search <关键词>"` |
| 安装软件 | `powershell -Command "scoop install <应用名>"` |
| 卸载软件 | `powershell -Command "scoop uninstall <应用名>"` |
| 更新 scoop | `powershell -Command "scoop update"` |
| 更新所有软件 | `powershell -Command "scoop update *"` |
| 列出已装软件 | `powershell -Command "scoop list"` |
| 查看可更新 | `powershell -Command "scoop status"` |
| 查看软件信息 | `powershell -Command "scoop info <应用名>"` |
| 添加 bucket | `powershell -Command "scoop bucket add <名称>"` |
| 移除 bucket | `powershell -Command "scoop bucket rm <名称>"` |
| 清理旧版本 | `powershell -Command "scoop cleanup *"` |
| 清空缓存 | `powershell -Command "scoop cache rm *"` |
| 健康检查 | `powershell -Command "scoop checkup"` |
| 重置应用 | `powershell -Command "scoop reset <应用名>"` |

### 备份与恢复

#### 导出（备份）

将已安装的应用、bucket 和版本导出为 JSON 文件：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop export > <备份路径>/scoopfile.json
```

只保存**清单**（应用名、版本、bucket 来源），不含安装文件和缓存。

通过 AskUserQuestion 询问用户：
1. **备份位置** — `scoopfile.json` 保存到哪里（如 `D:\Backup`、云同步目录等）
2. **是否备份下载缓存？** — 复制 `<scoop>/cache` 到备份目录，避免恢复时重新下载大包（如 Flutter ~1GB）。如需备份：
   ```bash
   powershell -Command 'Copy-Item -Path "<scoop>\cache" -Destination "<备份路径>\scoop-cache" -Recurse -Force'
   ```
3. **是否备份持久化数据？** — 备份 `<scoop>/persist`（scoop persist 管理的配置和数据）。如需备份：
   ```bash
   powershell -Command 'Copy-Item -Path "<scoop>\persist" -Destination "<备份路径>\scoop-persist" -Recurse -Force'
   ```

#### 导入（恢复）

在新机器上恢复（需先安装 scoop）。**顺序很重要**：

1. **先恢复缓存**（如有备份）——这样 `scoop import` 会跳过已缓存包的下载：
   ```bash
   powershell -Command 'Copy-Item -Path "<备份路径>\scoop-cache\*" -Destination "<scoop>\cache" -Recurse -Force'
   ```

2. **导入清单** — 安装所有应用并创建 NTFS junction：
   ```bash
   powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop import <备份路径>/scoopfile.json
   ```
   备份中引用的 bucket 会自动添加。

3. **恢复持久化数据**（如有备份）——在导入之后，此时 junction 已指向新路径：
   ```bash
   powershell -Command 'Copy-Item -Path "<备份路径>\scoop-persist\*" -Destination "<scoop>\persist" -Recurse -Force'
   ```

4. **执行 recipe** — 检查并运行安装后配置（如 git 用户信息、gh 登录）。

**路径安全性**：缓存文件与路径无关（按文件名/hash 匹配）。persist 数据也是安全的——`scoop import` 会创建新的 NTFS junction 指向新的 `<scoop>/persist` 位置，即使安装路径变了 junction 也是正确的。极少数情况下，应用自身的配置文件可能包含硬编码的绝对路径，这类需要手动修复。

## Recipes（配置方案）

对于安装后需要额外配置的工具（如设环境变量、写配置文件），在 `references/recipes/` 下维护独立的配置文件。安装工具时自动检查是否有对应 recipe。

## 参考文件

- **`commands.md`** — scoop 全部命令详解
- **`recipes/`** — 工具安装后配置方案
- **本文件 (`guide-zh.md`)** — 中文说明
