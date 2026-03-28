# Requirements: Spoon Backend Refactoring

**Defined:** 2026-03-28
**Core Value:** 让 `spoon-backend` 成为唯一可信的后台核心层，重要动作都在后端完成，`spoon` 只负责前端编排与呈现。

## v1 Requirements

### Backend Boundary

- [x] **BNDR-01**: `spoon` 中的 Scoop 安装、更新、卸载与 bucket 操作只能通过 `spoon-backend` 暴露的后端接口触发
- [x] **BNDR-02**: `spoon` 中的 Git / bucket 仓库相关操作只能通过 `spoon-backend` 暴露的后端接口触发
- [x] **BNDR-03**: `spoon` 中的 MSVC 检测与安装动作只能通过 `spoon-backend` 暴露的后端接口触发
- [ ] **BNDR-04**: `spoon` 不再直接推导 Scoop/MSVC 后台运行布局路径，只负责把配置好的 `root` 传给 backend
- [x] **BNDR-05**: `spoon` 消费 backend 返回的结果模型与查询模型，而不是重新读取 backend 状态文件或重建后台行为

### Scoop State

- [ ] **SCST-01**: `spoon-backend` 为 Scoop 安装状态保留一套唯一、规范、可持久化的状态模型
- [ ] **SCST-02**: 包信息、已安装状态、卸载输入与 reapply 输入都可以从这套规范状态模型导出
- [ ] **SCST-03**: `spoon-backend/src/scoop/` 中重复的 Scoop 状态模型被删除，而不是继续通过适配层并存
- [ ] **SCST-04**: Scoop 状态持久化只保存真正必要且不可推导的事实，不把可由布局推导出的绝对路径硬写进状态

### Scoop Lifecycle

- [ ] **SCLF-01**: `spoon-backend` 将 Scoop install 流程拆成清晰的生命周期阶段，而不是维持单个巨型流程文件
- [ ] **SCLF-02**: `spoon-backend` 将 Scoop update 流程纳入同一套后端生命周期模型，而不是让 app 侧补逻辑
- [ ] **SCLF-03**: `spoon-backend` 将 Scoop uninstall 流程纳入同一套后端生命周期模型，而不是让 app 侧补逻辑
- [ ] **SCLF-04**: command-surface reapply、integration reapply、persist restore/sync、hook 执行都由 backend 生命周期入口统一协调
- [ ] **SCLF-05**: hook、persist、surface、planner、acquire 等阶段拆成聚焦模块，减少 `runtime/actions.rs` 式巨型控制流

### Git Ownership

- [x] **GIT-01**: `spoon` 不再直接依赖 `gix`
- [x] **GIT-02**: Git / bucket repo 的 clone、sync、progress 事件桥接由 `spoon-backend` 独占
- [x] **GIT-03**: backend 暴露给 app 的 Git 相关接口不泄漏 `gix` 细节，而是返回 backend 级别的结果与事件

### Layout and Context

- [ ] **LAY-01**: `spoon-backend` 拥有根路径派生布局的单一实现，覆盖 Scoop、MSVC 与共享 shim/state 布局
- [x] **LAY-02**: `spoon` 只拥有应用配置文件路径与应用层配置语义，不再拥有后台运行布局语义
- [x] **LAY-03**: backend 操作在显式上下文中运行，不依赖隐式全局环境或分散路径推导

### Testing and Safety

- [ ] **TEST-01**: `spoon-backend` 为 Scoop 生命周期高风险路径补充后端测试，至少覆盖安装、更新、卸载中的关键失败路径
- [ ] **TEST-02**: `spoon` 测试保持聚焦 CLI/TUI 与应用编排，不继续承担 backend 细节正确性的回归覆盖
- [ ] **TEST-03**: 重构过程中新增或更新的 backend 接口，都有与其职责相邻的聚焦测试，而不是只靠端到端流程兜底

## v2 Requirements

### Reliability

- **RELY-01**: Scoop install/update 流程支持更明确的回滚或 journal 语义，避免半切换状态
- **RELY-02**: backend doctor / diagnostics 能解释状态损坏、边界违规或重放失败原因

### MSVC

- **MSVC-01**: 在 Scoop 主战场稳定后，对 `spoon-backend/src/msvc/` 做更系统的内部清理
- **MSVC-02**: 抽取 Scoop 与 MSVC 真正共享的后端模式，但只在 Scoop 边界已稳定后进行

## Out of Scope

| Feature | Reason |
|---------|--------|
| 为旧的低质量抽象保留兼容层 | 当前明确采用前向设计，优先删除坏抽象 |
| 第一阶段主动完成完整的 MSVC 深度重构 | 当前主战场是 `spoon-backend/src/scoop/` |
| 在 backend 清理前先做新的 UI/交互扩展 | 现阶段价值不如边界与重复收敛 |
| 让 `spoon` 继续直接依赖 `gix` 或重建 Git 行为 | 与目标边界相冲突 |
| 同时保留多套 Scoop persisted state 模型 | 这正是本轮要优先消灭的重复 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BNDR-01 | Phase 1 | Complete |
| BNDR-02 | Phase 1 | Complete |
| BNDR-03 | Phase 1 | Complete |
| BNDR-04 | Phase 1 | Pending |
| BNDR-05 | Phase 1 | Complete |
| SCST-01 | Phase 2 | Pending |
| SCST-02 | Phase 2 | Pending |
| SCST-03 | Phase 2 | Pending |
| SCST-04 | Phase 2 | Pending |
| SCLF-01 | Phase 3 | Pending |
| SCLF-02 | Phase 3 | Pending |
| SCLF-03 | Phase 3 | Pending |
| SCLF-04 | Phase 3 | Pending |
| SCLF-05 | Phase 3 | Pending |
| GIT-01 | Phase 1 | Complete |
| GIT-02 | Phase 1 | Complete |
| GIT-03 | Phase 1 | Complete |
| LAY-01 | Phase 1 | Pending |
| LAY-02 | Phase 1 | Complete |
| LAY-03 | Phase 1 | Complete |
| TEST-01 | Phase 4 | Pending |
| TEST-02 | Phase 4 | Pending |
| TEST-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 23 total
- Mapped to phases: 23
- Unmapped: 0

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after roadmap creation*
