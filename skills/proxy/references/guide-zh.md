# 代理与镜像源配置 — 中文说明

本文件是 SKILL.md 的中文对照版，方便理解 skill 的工作流程。实际执行以 SKILL.md 为准。

---

## 核心原则

1. **所有工具同步**：开启或关闭代理时，同时配置所有已安装的工具，不能只改一个。
2. **操作前询问**：通过 AskUserQuestion 展示当前状态，确认后再执行。
3. **先测再切镜像**：关闭代理后先测试连通性，测试失败才建议切镜像。
4. **检测已装工具**：只配置系统中实际存在的工具，跳过未安装的。

## 触发条件

- 用户说"设置代理"、"取消代理"、"切换镜像"、"用国内源"等。
- 任何工具操作因网络/SSL/超时错误失败时，由对应 skill 转交本 skill 处理。
- 用户提到"github 太慢"、"npm 下载不了"、"pip 超时"等。
- 安装新工具后，检查是否需要配置代理/镜像。

## 检测已装工具

配置前先检测哪些工具已安装，使用 `run-cmd.ps1` 辅助脚本：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version 2>&1
```

只配置已安装的工具，跳过未安装的。

## 各工具代理配置

### Git

| 操作 | 命令 |
|------|------|
| 开启 | `git config --global http.proxy <proxy_url>` + `https.proxy` |
| 关闭 | `git config --global --unset http.proxy` + `--unset https.proxy` |
| 查看 | `git config --global --get http.proxy` |

### Scoop

| 操作 | 命令 |
|------|------|
| 开启 | `scoop config proxy <host>:<port>`（不带 `http://`） |
| 关闭 | `scoop config rm proxy` |
| 查看 | `scoop config proxy` |

### npm

| 操作 | 命令 |
|------|------|
| 开启 | `npm config set proxy <proxy_url>` + `https-proxy` |
| 关闭 | `npm config delete proxy` + `https-proxy` |
| 查看 | `npm config get proxy` |

### pip

| 操作 | 命令 |
|------|------|
| 开启 | `pip config set global.proxy <proxy_url>` |
| 关闭 | `pip config unset global.proxy` |
| 查看 | `pip config get global.proxy` |

### Cargo (Rust)

通过 Windows 用户环境变量设置：`HTTP_PROXY` 和 `HTTPS_PROXY`。

### Flutter / Dart

通过 Windows 用户环境变量设置：`HTTP_PROXY`、`HTTPS_PROXY`、`NO_PROXY`。

## 各工具国内镜像源

当代理不可用且连通性测试失败时，提供镜像替代方案。通过 AskUserQuestion 让用户选择镜像提供商。

### Scoop — GitHub 镜像

| 方案 | 提供商 | 说明 |
|------|--------|------|
| 1（推荐）| Gitee 镜像 | `SCOOP_REPO`: `https://gitee.com/scoop-installer/scoop`，bucket 换 Gitee 版本。约 10 天同步一次。|
| 2 | scoop-proxy-cn | `SCOOP_REPO`: `https://gh-proxy.org/github.com/ScoopInstaller/Scoop`，含 1.4w+ 应用。|

### npm — 注册表镜像

| 方案 | 提供商 | 注册表 URL |
|------|--------|-----------|
| 1（推荐）| 淘宝 (npmmirror) | `https://registry.npmmirror.com` |
| 2 | 腾讯云 | `https://mirrors.cloud.tencent.com/npm/` |
| 3 | 华为云 | `https://repo.huaweicloud.com/repository/npm/` |

恢复官方：`npm config set registry https://registry.npmjs.org`

### pip — 索引镜像

| 方案 | 提供商 | 索引 URL |
|------|--------|---------|
| 1（推荐）| 清华 (TUNA) | `https://pypi.tuna.tsinghua.edu.cn/simple` |
| 2 | 阿里云 | `https://mirrors.aliyun.com/pypi/simple` |
| 3 | 中科大 (USTC) | `https://pypi.mirrors.ustc.edu.cn/simple` |
| 4 | 腾讯云 | `https://mirrors.cloud.tencent.com/pypi/simple` |

恢复官方：`pip config set global.index-url https://pypi.org/simple`

### Cargo — Crate 镜像

| 方案 | 提供商 | Registry |
|------|--------|----------|
| 1（推荐）| 中科大 (USTC) | `sparse+https://mirrors.ustc.edu.cn/crates.io-index/` |
| 2 | 清华 (TUNA) | `sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/` |
| 3 | 字节 (RsProxy) | `sparse+https://rsproxy.cn/index/` |

修改 `~/.cargo/config.toml`，恢复官方时删除 `[source.*]` 部分。

### Flutter / Dart — Pub 镜像

| 方案 | 提供商 | 环境变量 |
|------|--------|----------|
| 1（推荐）| 清华 (TUNA) | `PUB_HOSTED_URL=https://mirrors.tuna.tsinghua.edu.cn/dart-pub`，`FLUTTER_STORAGE_BASE_URL=https://mirrors.tuna.tsinghua.edu.cn/flutter` |
| 2 | 中科大 (USTC) | `PUB_HOSTED_URL=https://mirrors.ustc.edu.cn/dart-pub`，`FLUTTER_STORAGE_BASE_URL=https://mirrors.ustc.edu.cn/flutter` |
| 3 | 上海交大 (SJTUG) | `PUB_HOSTED_URL=https://dart-pub.mirrors.sjtug.sjtu.edu.cn`，`FLUTTER_STORAGE_BASE_URL=https://mirrors.sjtug.sjtu.edu.cn/flutter_infra` |

恢复官方：删除上述环境变量。

## 恢复所有官方源

重新启用代理或用户要求恢复官方源时：

1. 移除 `SCOOP_REPO` 配置，重新添加官方 bucket
2. 重置 npm registry 为 `https://registry.npmjs.org`
3. 重置 pip index 为 `https://pypi.org/simple`
4. 删除 Cargo 镜像配置（`~/.cargo/config.toml` 中的 `[source.*]`）
5. 删除 Flutter/Dart 环境变量（`PUB_HOSTED_URL`、`FLUTTER_STORAGE_BASE_URL`）

操作前确认，操作后报告。

## 工作流程

### 开启代理
1. 询问代理地址 → 2. 检测已装工具 → 3. 全部配置 → 4. 有镜像则询问是否恢复官方源 → 5. 报告

### 关闭代理
1. 确认 → 2. 全部移除 → 3. 测试连通性 → 4. 失败则提供镜像选择 → 5. 报告

### 切换镜像
1. 检测已装工具和当前源状态 → 2. 展示选项（带推荐）→ 3. 应用 → 4. 验证 → 5. 报告

## 参考文件

- **本文件 (`guide-zh.md`)** — 中文说明
