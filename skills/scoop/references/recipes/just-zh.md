# just — 安装后配置

## 何时安装

`just` 是一个命令运行器，通常用于在项目中定义和执行开发任务。需要以下场景时安装：

- 项目中已经有 `justfile`
- 希望用统一命令封装构建、测试、格式化、发布等任务
- 需要替代一组零散的 shell 脚本或批处理命令

安装 `just` 只会提供命令本身，不会自动创建 `justfile`。

## 安装

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop install just
```

## 安装后配置

### 默认无需额外配置

Scoop 安装完成后，`just` 应已可直接使用。默认不需要额外环境变量，也不需要写全局配置文件。

### `justfile` 是项目文件，不是工具配置

`just` 通常读取当前目录中的 `justfile`。如果当前项目没有 `justfile`，`just` 安装成功后也可能没有任务可执行，这不代表安装失败。

如果当前目录已有 `justfile`，可以先列出可用任务：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --list
```

### 代理与镜像

如果 Scoop 下载 `just` 失败，代理和镜像仍然统一交给 `proxy` skill 处理，不在本 recipe 中重复配置。

## 验证

先确认 `just` 本体已可用：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --version
```

如果当前项目已经有 `justfile`，再进一步验证任务发现是否正常：

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 just --list
```

## 卸载

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop uninstall just
```

卸载 `just` 本身通常不需要清理额外的全局残留数据。

不要自动删除项目中的 `justfile`，因为它属于用户项目文件，而不是 `just` 的安装产物。
