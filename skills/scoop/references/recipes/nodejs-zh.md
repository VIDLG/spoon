# Node.js / npm — 安装后配置

## 何时安装

Node.js 是 JavaScript 运行时，安装后会同时提供 `node`、`npm` 和 `npx`。需要以下场景时安装：

- 运行 JavaScript / TypeScript 项目
- 安装和管理前端或 Node.js 项目依赖
- 运行 `package.json` 中的 scripts
- 使用 `npx` 执行 CLI 工具
- 发布或维护 npm 包

npm 不作为独立软件单独安装。默认通过安装 Node.js 一起获得。

## 安装

通过 AskUserQuestion 让用户选择安装版本线：

- **`nodejs`（默认）** — 当前稳定版，适合需要较新特性或最新生态兼容性的场景
- **`nodejs-lts`** — 长期支持版，适合偏稳定、版本要求保守的项目

如果用户没有明确偏好，默认安装 `nodejs`：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install <chosen_package>
```

## 安装后配置

### 默认无需额外配置

安装完成后，`node`、`npm` 和 `npx` 应已可用，不需要单独安装 npm。

### 代理与镜像

如果用户在国内，或 npm registry 访问慢、下载失败，不要在本 recipe 中直接改 `npm config`。

代理和镜像统一交给 `proxy` skill 处理，包括：

- npm 代理（`proxy`、`https-proxy`）
- npm registry 镜像
- 恢复官方 registry

## 验证

先确认 Node.js 和 npm 都已可用：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 node --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm --version
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npx --version
```

如果需要验证 npm 网络访问：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm ping
```

## 卸载

卸载安装时选择的包（`nodejs` 或 `nodejs-lts`）：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall <chosen_package>
```

卸载后询问用户是否清理残留数据：

- **保留** — 保留 `~/.npm` 缓存目录和 `~/.npmrc` 配置，便于将来继续使用
- **删除** — 删除这些 Node.js / npm 用户级数据
- **先看看** — 展示配置文件内容或缓存目录大小后再决定

如果用户选择删除：

```bash
powershell -Command 'if (Test-Path "$env:USERPROFILE\.npm") { Remove-Item -Path "$env:USERPROFILE\.npm" -Recurse -Force }'
powershell -Command 'if (Test-Path "$env:USERPROFILE\.npmrc") { Remove-Item -Path "$env:USERPROFILE\.npmrc" -Force }'
```
