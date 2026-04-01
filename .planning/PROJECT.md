## Current State

- Shipped milestone: `v0.6.0`
- Archived roadmap: [`v0.6.0-ROADMAP.md`](/d:/projects/spoon/.planning/milestones/v0.6.0-ROADMAP.md)
- Archived requirements: [`v0.6.0-REQUIREMENTS.md`](/d:/projects/spoon/.planning/milestones/v0.6.0-REQUIREMENTS.md)
- Final audit: [`v0.6.0-MILESTONE-AUDIT.md`](/d:/projects/spoon/.planning/v0.6.0-MILESTONE-AUDIT.md)

- Previous shipped milestone: `v0.5.0`
- Archived roadmap: [`v0.5.0-ROADMAP.md`](/d:/projects/spoon/.planning/milestones/v0.5.0-ROADMAP.md)
- Archived requirements: [`v0.5.0-REQUIREMENTS.md`](/d:/projects/spoon/.planning/milestones/v0.5.0-REQUIREMENTS.md)
- Final audit: [`v0.5.0-MILESTONE-AUDIT.md`](/d:/projects/spoon/.planning/v0.5.0-MILESTONE-AUDIT.md)

## Current Milestone: v0.7.0 Scoop Legacy Cleanup and Domain Refinement

**Goal:** systematically clean the remaining outdated or poorly shaped code in `spoon-backend/src/scoop/` without reopening a giant cross-domain refactor.

**Target features:**
- Remove or downgrade JSON-era and deprecated Scoop path/state assumptions that still survive in active code.
- Consolidate stale helper layers, host seams, and duplicated runtime utilities inside the Scoop backend domain.
- Tighten Scoop read models and runtime contracts where low-value redundancy or legacy abstractions still remain.
- Refresh the Scoop-focused safety net after the cleanup so the reduced legacy surface stays stable.

# Spoon Backend Refactoring

## What This Is / 项目是什么

Spoon 是一个面向 Windows 开发环境管理的工具，`spoon/` 负责 CLI/TUI 前端与应用层编排，`spoon-backend/` 负责可复用的后台核心能力。当前这项工作聚焦于把 Spoon 明确收敛成“前端壳 + 后端核心”的结构，并优先深度清理 `spoon-backend`，尤其是 `spoon-backend/src/scoop/` 中由大语言模型堆出来的重复、混乱和失控实现。

## Core Value / 核心价值

让 `spoon-backend` 成为唯一可信的后台核心层，重要动作都在后端完成，`spoon` 只负责前端编排与呈现。

## Requirements / 需求

### Validated / 已验证

- 已有基于 Rust 的 `spoon` CLI/TUI 应用壳，可作为统一前端入口
- 已有 `spoon-backend`，承载 Scoop、Git、MSVC、代理/环境、缓存/状态等后台能力
- 已有 Spoon 对 Scoop 安装、更新、卸载与 bucket 管理的运行能力
- 已有 Git 相关后台能力与进度事件桥接
- 已有 MSVC 检测、安装、状态与验证能力
- 已有代理、环境变量、PATH、缓存和状态管理能力

### Active / 当前进行中

- [ ] 深度重构 `spoon-backend/src/scoop/`，清理屎山式实现与跨层混杂
- [ ] 将 Scoop 生命周期行为拆分为明确的后端阶段，并进一步收敛 `spoon` 前端层

### Completed in Phase 02: Canonical Scoop State

- [x] 引入 `scoop/state/` 模块，建立 `InstalledPackageState` 为唯一规范持久化记录
- [x] 运行时写操作（安装/更新/卸载）产出规范状态而非旧版平面状态
- [x] 查询、状态与详情视图全部从规范状态的类型化投影派生
- [x] 移除 `ScoopPackageState` 旧版公共 API，doctor 检测并报告遗留平面状态

### Completed in Phase 01: Backend Seams and Ownership

- [x] 将 `spoon` 明确收敛为 CLI/TUI 前端与应用层编排，不再直接承载重要后台实现细节
- [x] 将所有重要后台核心动作明确收敛到 `spoon-backend`
- [x] 将 Git / `gix` 责任完全收口到 `spoon-backend`，`spoon` 不再直接依赖底层 Git 实现
- [x] 在必要时调整 `spoon-backend/src/msvc/` 以配合边界收敛，但不把它作为第一阶段主战场
- [x] 用前向设计替换明显失败的旧抽象，不为了兼容性保留低质量壳层

### Out of Scope / 暂不纳入范围

- 为了兼容旧抽象而保留设计明显不好的接口 - 当前目标是前向设计
- 在第一阶段主动做完整的 `msvc` 深度重构 - 现阶段主攻 `scoop`
- 先做新的终端功能或 UI 花样 - 在 backend 边界收敛前优先级更低
- 把后台核心逻辑重新塞回 `spoon` - 这会再次破坏前后端职责边界

## Context / 背景

- 这是一个已有代码的 brownfield Rust workspace，包含 `spoon`、`spoon-backend` 和 `xtask`
- 当前仓库已经完成 codebase map，可以据此确认现状、边界和问题分布
- `spoon-backend/src` 根下部分文件已被清理，但 `spoon-backend/src/scoop/` 和 `spoon-backend/src/msvc/` 仍有大量历史垃圾
- 这些历史垃圾并不局限于单一问题类型，可能同时表现为重复模型、重复流程、模块边界混乱、命名差、测试与实现混杂、状态落点不清、前后端职责串层等
- 当前最明确的重构优先级是 `scoop`，尤其是重复状态模型
- 用户接受在必要时调整现有接口与行为，不要求为旧设计强行保兼容

## Constraints / 约束

- **Tech stack**: Rust workspace，`spoon` 与 `spoon-backend` 分工必须更清晰 - 这是当前重构主线
- **Architecture**: `spoon` 不应直接碰 `gix`、Scoop 运行细节、MSVC 安装细节、环境落盘细节 - 这些应留在 backend
- **Platform**: 项目明确是 Windows-first - 不为跨平台抽象增加额外复杂度
- **Compatibility**: 默认采用前向设计 - 仅在确有必要时才保留兼容层
- **Refactor strategy**: 第一阶段要求深度清理，不是表面整理 - 边界和重复必须真正收敛

## Key Decisions / 关键决策

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `spoon` 只做前端壳 | 让 CLI/TUI、应用编排与后台实现职责清晰分离 | — Pending |
| 重要后台动作全部进入 `spoon-backend` | 保证后台行为有单一可信实现位置 | — Pending |
| 第一阶段主攻 `spoon-backend/src/scoop/` | 这是当前最脏、最影响边界质量的区域 | — Pending |
| 优先消除重复状态模型 | 这是已明确指出的最高优先级重复问题 | — Pending |
| Git / `gix` 只留在 backend | `spoon` 不应直接依赖底层 Git crate | — Pending |
| 默认采用前向设计 | 不为低质量旧抽象背兼容包袱 | — Pending |
| `msvc` 只在必要时配合调整 | 先集中火力在 `scoop` 主战场 | — Pending |

## Evolution / 文档如何演进

这个文档会在阶段切换与里程碑边界持续演进。

**在每次阶段切换后**:
1. 如果某些需求已不再成立，将其移到 Out of Scope 并写明原因
2. 如果某些需求已经验证完成，将其移到 Validated 并标注对应阶段
3. 如果出现新的需求，将其加入 Active
4. 如果有新的关键决策，补充进 Key Decisions
5. 如果 “What This Is” 已经与项目实际漂移，及时更新

**在每个里程碑结束后**:
1. 对所有章节做一次完整复查
2. 检查 Core Value 是否仍然是当前最重要的事情
3. 审查 Out of Scope 中的理由是否依然成立
4. 按当前状态更新 Context

---
*Last updated: 2026-03-29 after Phase 02 completion*
