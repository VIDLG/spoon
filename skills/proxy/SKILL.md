---
name: proxy
description: >
  This skill should be used when the user asks to "set proxy", "remove proxy", "configure proxy",
  "switch to mirror", "use gitee mirror", "use taobao mirror", "can't access github",
  "can't download packages", "npm too slow", "pip timeout", "github too slow",
  "download failed", or mentions proxy, mirror, network acceleration, or source switching
  for any development tool. Also triggered when any tool operation fails with network/SSL errors.
---

# Proxy & Mirror Configuration

Manage proxy settings and mirror sources for all development tools on Windows. When changing proxy or mirrors, always keep all tools in sync.

## Key Principles

1. **All tools in sync**: when enabling or disabling proxy, configure ALL installed tools together, not just one.
2. **Ask before changing**: always use AskUserQuestion to show current status and confirm the proposed change before executing.
3. **Test before suggesting mirrors**: after disabling proxy, test connectivity first. Only suggest mirrors if the test fails.
4. **Detect installed tools**: before configuring, check which tools are actually installed. Only configure tools that exist on the system.

## When to Trigger

- **User explicitly asks**: "set proxy", "remove proxy", "switch mirror", "use domestic source", etc.
- **Network failure in any skill**: when a tool operation fails with network/SSL/timeout errors, the calling skill should delegate to this skill.
- **User mentions access issues**: "github too slow", "npm can't download", "pip timeout", etc.
- **During tool installation**: after installing a tool that needs network access, check if proxy/mirrors should be configured.

## Detect Installed Tools

Before configuring proxy or mirrors, detect which tools are installed:

```bash
# Check each tool (use run-cmd.ps1 to refresh PATH)
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 cargo --version 2>&1
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 flutter --version 2>&1
```

Only configure tools that are found. Skip the rest silently.

## Proxy Configuration by Tool

### Git

```bash
# Enable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global http.proxy <proxy_url>
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global https.proxy <proxy_url>

# Disable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global --unset http.proxy
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global --unset https.proxy

# Check
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 git config --global --get http.proxy
```

### Scoop

```bash
# Enable (host:port format, no http:// prefix)
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop config proxy <host>:<port>

# Disable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop config rm proxy

# Check
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop config proxy
```

### npm

```bash
# Enable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config set proxy <proxy_url>
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config set https-proxy <proxy_url>

# Disable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config delete proxy
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config delete https-proxy

# Check
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config get proxy
```

### pip

```bash
# Proxy is set via environment variable or pip config
# Enable (writes to pip config)
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config set global.proxy <proxy_url>

# Disable
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config unset global.proxy

# Check
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config get global.proxy
```

### Cargo (Rust)

Cargo uses environment variables for proxy. Set them in the Windows registry:

```bash
powershell -Command '[Environment]::SetEnvironmentVariable("HTTP_PROXY", "<proxy_url>", "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("HTTPS_PROXY", "<proxy_url>", "User")'

# Disable
powershell -Command '[Environment]::SetEnvironmentVariable("HTTP_PROXY", [NullString]::Value, "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("HTTPS_PROXY", [NullString]::Value, "User")'
```

### Flutter / Dart

Flutter uses environment variables for both proxy and mirror:

```bash
# Proxy
powershell -Command '[Environment]::SetEnvironmentVariable("HTTP_PROXY", "<proxy_url>", "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("HTTPS_PROXY", "<proxy_url>", "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("NO_PROXY", "localhost,127.0.0.1", "User")'
```

## Mirror Configuration by Tool (China Mainland)

When proxy is unavailable and connectivity test fails, offer mirror alternatives. **Always use AskUserQuestion to let the user choose which mirror provider to use.** Each tool lists multiple mirror options below.

### Scoop — GitHub mirrors

Use AskUserQuestion with these options:

| Option | Provider | URL |
|--------|----------|-----|
| 1 (Recommended) | Gitee 镜像 | `SCOOP_REPO`: `https://gitee.com/scoop-installer/scoop` |
| 2 | scoop-proxy-cn | `SCOOP_REPO`: `https://gh-proxy.org/github.com/ScoopInstaller/Scoop` |

**Gitee mirror** — replace each bucket with its Gitee counterpart:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop config SCOOP_REPO https://gitee.com/scoop-installer/scoop
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop bucket rm main
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop bucket add main https://gitee.com/scoop-installer/Main
# Repeat for extras, java, etc. using https://gitee.com/scoop-installer/<BucketName>
```

Note: Gitee mirrors sync every ~10 days, packages may lag behind.

**scoop-proxy-cn**:

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop config SCOOP_REPO https://gh-proxy.org/github.com/ScoopInstaller/Scoop
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 scoop bucket add spc https://gitee.com/wlzwme/scoop-proxy-cn.git
```

### npm — Registry mirrors

Use AskUserQuestion with these options:

| Option | Provider | Registry URL |
|--------|----------|-------------|
| 1 (Recommended) | 淘宝 (npmmirror) | `https://registry.npmmirror.com` |
| 2 | 腾讯云 | `https://mirrors.cloud.tencent.com/npm/` |
| 3 | 华为云 | `https://repo.huaweicloud.com/repository/npm/` |

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config set registry <chosen_url>

# Restore official
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 npm config set registry https://registry.npmjs.org
```

### pip — Index mirrors

Use AskUserQuestion with these options:

| Option | Provider | Index URL |
|--------|----------|----------|
| 1 (Recommended) | 清华 (TUNA) | `https://pypi.tuna.tsinghua.edu.cn/simple` |
| 2 | 阿里云 | `https://mirrors.aliyun.com/pypi/simple` |
| 3 | 中科大 (USTC) | `https://pypi.mirrors.ustc.edu.cn/simple` |
| 4 | 腾讯云 | `https://mirrors.cloud.tencent.com/pypi/simple` |

```bash
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config set global.index-url <chosen_url>
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config set global.trusted-host <chosen_host>

# Restore official
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config set global.index-url https://pypi.org/simple
powershell -File <plugin_root>/skills/scripts/run-cmd.ps1 pip config unset global.trusted-host
```

### Cargo (Rust) — Crate mirrors

Use AskUserQuestion with these options:

| Option | Provider | Registry |
|--------|----------|----------|
| 1 (Recommended) | 中科大 (USTC) | `sparse+https://mirrors.ustc.edu.cn/crates.io-index/` |
| 2 | 清华 (TUNA) | `sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/` |
| 3 | 字节 (RsProxy) | `sparse+https://rsproxy.cn/index/` |

Create or update `~/.cargo/config.toml`:

```toml
[source.crates-io]
replace-with = "<mirror_name>"

[source.<mirror_name>]
registry = "<chosen_registry_url>"
```

To restore official, remove the `[source.*]` sections from the config file.

### Flutter / Dart — Pub mirrors

Use AskUserQuestion with these options:

| Option | Provider | PUB_HOSTED_URL | FLUTTER_STORAGE_BASE_URL |
|--------|----------|----------------|--------------------------|
| 1 (Recommended) | 清华 (TUNA) | `https://mirrors.tuna.tsinghua.edu.cn/dart-pub` | `https://mirrors.tuna.tsinghua.edu.cn/flutter` |
| 2 | 中科大 (USTC) | `https://mirrors.ustc.edu.cn/dart-pub` | `https://mirrors.ustc.edu.cn/flutter` |
| 3 | 上海交大 (SJTUG) | `https://dart-pub.mirrors.sjtug.sjtu.edu.cn` | `https://mirrors.sjtug.sjtu.edu.cn/flutter_infra` |

```bash
# Set chosen mirror
powershell -Command '[Environment]::SetEnvironmentVariable("PUB_HOSTED_URL", "<chosen_pub_url>", "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("FLUTTER_STORAGE_BASE_URL", "<chosen_storage_url>", "User")'

# Restore official (remove env vars)
powershell -Command '[Environment]::SetEnvironmentVariable("PUB_HOSTED_URL", [NullString]::Value, "User")'
powershell -Command '[Environment]::SetEnvironmentVariable("FLUTTER_STORAGE_BASE_URL", [NullString]::Value, "User")'
```

## Restore All to Official Sources

When the user re-enables proxy or wants to switch back to official sources:

1. Remove `SCOOP_REPO` config, re-add official buckets
2. Reset npm registry to `https://registry.npmjs.org`
3. Reset pip index to `https://pypi.org/simple`
4. Remove Cargo mirror config from `~/.cargo/config.toml`
5. Remove Flutter/Dart env vars (`PUB_HOSTED_URL`, `FLUTTER_STORAGE_BASE_URL`)

Always confirm with the user before restoring, and report what was changed.

## Workflow

### Enable proxy
1. Ask user for proxy address via AskUserQuestion
2. Detect installed tools
3. Configure proxy for all installed tools
4. If any tools currently use mirrors, ask whether to restore official sources
5. Report changes

### Disable proxy
1. Confirm with user via AskUserQuestion
2. Remove proxy from all installed tools
3. Test connectivity (e.g., `scoop update` or `npm ping`)
4. If test fails, offer mirror options via AskUserQuestion
5. Report changes

### Switch mirrors
1. Detect installed tools and current mirror/source status
2. Present options via AskUserQuestion (with recommendations)
3. Apply selected mirrors
4. Verify by running a test operation
5. Report changes
