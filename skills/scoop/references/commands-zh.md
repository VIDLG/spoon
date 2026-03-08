# Scoop 命令参考

所有 scoop 子命令的完整参考。

## 核心命令

### scoop install — 安装应用
```
scoop install <应用名> [选项]
scoop install extras/<应用名>          # 从指定 bucket 安装
scoop install https://url/app.json    # 从 URL 清单安装
```
选项：
- `-g` / `--global` — 全局安装（需管理员权限）
- `-k` / `--no-cache` — 不使用下载缓存
- `-s` / `--skip` — 跳过哈希校验（不推荐）
- `-a <arch>` — 指定架构：`32bit` 或 `64bit`

### scoop uninstall — 卸载应用
```
scoop uninstall <应用名> [选项]
```
选项：
- `-g` / `--global` — 卸载全局安装的应用
- `-p` / `--purge` — 同时删除持久化数据

### scoop update — 更新
```
scoop update              # 更新 scoop 自身 + 所有 bucket 清单
scoop update <应用名>      # 更新指定应用
scoop update *            # 更新所有已安装的应用
```
选项：
- `-g` / `--global` — 更新全局安装的应用
- `-f` / `--force` — 强制更新（即使已是最新）
- `-k` / `--no-cache` — 不使用下载缓存
- `-q` / `--quiet` — 静默输出

### scoop search — 搜索应用
```
scoop search <关键词>      # 按名称搜索（支持正则）
```

### scoop list — 列出已安装的应用
```
scoop list                # 列出全部
scoop list <关键词>        # 按名称过滤
```

### scoop info — 查看应用详情
```
scoop info <应用名>
```

### scoop status — 查看可更新的应用
```
scoop status
```

## Bucket 命令

### scoop bucket add — 添加 bucket
```
scoop bucket add <名称>              # 添加已知 bucket
scoop bucket add <名称> <git-url>    # 添加自定义 bucket
```

官方已知 bucket：`main`、`extras`、`versions`、`java`、`nerd-fonts`、`nirsoft`、`sysinternals`、`php`、`nonportable`、`games`。

### scoop bucket rm — 移除 bucket
```
scoop bucket rm <名称>
```

### scoop bucket list — 列出已添加的 bucket
```
scoop bucket list
```

### scoop bucket known — 列出所有已知官方 bucket
```
scoop bucket known
```

## 维护命令

### scoop cleanup — 清理旧版本
释放磁盘空间。
```
scoop cleanup <应用名>       # 清理指定应用
scoop cleanup *             # 清理所有应用
```
选项：
- `-g` / `--global` — 清理全局安装的应用
- `-k` / `--cache` — 同时清理过期的下载缓存

### scoop cache — 管理下载缓存
```
scoop cache show            # 查看缓存内容
scoop cache show <应用名>   # 查看指定应用的缓存
scoop cache rm <应用名>     # 删除指定应用的缓存
scoop cache rm *            # 清空全部缓存
```

### scoop checkup — 健康检查
检查 scoop 安装的潜在问题。
```
scoop checkup
```

### scoop reset — 重置应用
重新创建 shim 和快捷方式。适用于修复损坏的应用链接或切换版本。
```
scoop reset <应用名>
scoop reset *               # 重置所有应用
```

### scoop hold / unhold — 锁定/解锁更新
```
scoop hold <应用名>          # 阻止该应用被更新
scoop unhold <应用名>        # 允许更新
```

## 实用命令

### scoop which — 查看命令路径
```
scoop which <命令名>
```

### scoop home — 打开应用主页
```
scoop home <应用名>
```

### scoop prefix — 查看应用安装路径
```
scoop prefix <应用名>
```

### scoop cat — 查看应用清单
```
scoop cat <应用名>
```

### scoop depends — 查看依赖树
```
scoop depends <应用名>
```

### scoop export / import — 导出/导入应用列表
```
scoop export > scoopfile.json    # 导出已安装的应用
scoop import scoopfile.json      # 从文件导入并安装
```

### scoop config — 管理配置
```
scoop config                    # 查看所有配置
scoop config <键>               # 查看某个配置
scoop config <键> <值>           # 设置配置
scoop config rm <键>            # 删除配置
```

常用配置项：
- `proxy` — HTTP 代理（如 `127.0.0.1:7890`）
- `aria2-enabled` — 启用 aria2 加速下载（`true`/`false`）
- `SCOOP_REPO` — 自定义 scoop 仓库 URL
- `SCOOP_BRANCH` — 使用的 scoop 分支（`master`/`develop`）

### scoop alias — 管理命令别名
```
scoop alias add <名称> <命令> <描述>
scoop alias rm <名称>
scoop alias list
```

## 常用操作模式

### 新机器初始化
```bash
# 安装 scoop 后添加 bucket 并导入应用列表
scoop bucket add extras
scoop bucket add versions
scoop import scoopfile.json
```

### 保持全部更新
```bash
scoop update
scoop update *
scoop cleanup *
scoop cache rm *
```

### 切换应用版本
```bash
scoop install versions/python27
scoop reset python27    # 切换到 Python 2.7
scoop reset python      # 切回最新版 Python
```
