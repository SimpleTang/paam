# ADR-0008: 采用 OpenSpec 作为执行层 spec 工具

- **Status**: Accepted
- **Date**: 2026-04-26
- **Deciders**: @simpletang1994
- **Tags**: tooling | workflow | ai

## Context

paam 是单人 + AI 辅助开发的项目。AI 编码助手（Claude Code）的核心痛点：**功能需求"只活在聊天历史里"**，AI 实现得偏离预期就要返工。

需要一个机制：在写代码前先把"做什么"明确写下来，让人和 AI 在 spec 上达成共识。

候选方案：
- 自己手写 propose 模板（不用工具）
- 使用 OpenSpec（[Fission-AI/OpenSpec](https://github.com/Fission-AI/OpenSpec)，42k+ stars，MIT，v1.3.1）
- 使用 GitHub Issues + 模板代替

## Decision

**采用 OpenSpec 作为执行层的 spec 管理工具**，与 Milestone plan（产品设计层）和 ADR（决策档案）形成三层职责分工：

```
产品设计层 → Milestone plan (.dev/docs/milestones/)
执行层    → OpenSpec change (openspec/changes/)
横向     → ADR (.dev/docs/decisions/)
```

具体配置：
- 全局安装：`npm install -g @fission-ai/openspec@latest`（要求 Node 20.19.0+）
- AI 工具集成：仅 Claude Code（`openspec init --tools claude`）
- Profile：`core`（默认，提供 propose / apply / archive / explore 四个工作流）
- 路径：`openspec/` 与 `.claude/` 在仓库顶层（OpenSpec 不支持自定义路径）

## Alternatives Considered

### Option A: 自己手写 propose 模板

- **Pros**:
  - 零工具依赖，永远不会"工具废弃"
  - 完全可定制
- **Cons**:
  - 需要自己设计 spec → design → tasks 的流转逻辑
  - 没有 CLI 校验、没有 status 跟踪
  - AI 集成需要自己写 SKILL/commands
- **Verdict**: ❌ 自造轮子成本远高于工具收益

### Option B: 采用 OpenSpec ✅

- **Pros**:
  - 现成的 propose / apply / archive 闭环
  - 自动生成 Claude Code slash commands + SKILLs（开箱即用）
  - 工具提供 status / instructions JSON API，AI 友好
  - 模板与 schema 经过社区打磨（spec-driven schema）
  - 与 Milestone plan / ADR 职责无重叠（粒度互补）
- **Cons**:
  - 工具依赖（npm 全局包，Node 运行时）
  - `openspec/` 与 `.claude/` 路径硬编码在顶层（无法移到 `.dev/`）
  - 配置是全局的（`~/.config/openspec/config.json`），跨机器需要重新配置
  - 如果项目废弃，artifacts 仍可读但 workflow 失效
- **Verdict**: ✅ 收益显著大于成本

### Option C: GitHub Issues + 模板

- **Pros**:
  - 与现有 GitHub 流程自然结合
  - 无额外工具依赖
- **Cons**:
  - Issue 文本不适合长篇 spec / design / tasks 文档
  - 无法版本化 spec 演进
  - AI 与 Issues 集成不如本地文件
- **Verdict**: ❌ 不适合 spec-driven 工作流

## Consequences

### Positive

- ✅ 每个 feature 在写代码前都有完整 spec / design / tasks，AI 不再"凭印象"实现
- ✅ Slash commands `/opsx:propose | apply | archive` 让工作流可重复
- ✅ Spec 与 code 一起 commit，半年后回看有完整意图记录
- ✅ 与 OpenSpec 社区演进（v1.3.1，活跃维护）

### Negative

- ❌ 增加一个 npm 全局依赖；新设备开发需重新装
- ❌ `.claude/` 与 `openspec/` 占据仓库顶层（与"对外/对内"分层原则不完全吻合）
- ❌ 学习曲线：需要熟悉 OpenSpec 的 schema、artifact 概念、slash commands

### Neutral / Trade-offs

- 工具的"自动化"程度高，但反过来也意味着"被工具约束"——例如 spec / design / tasks 四件套的结构是固定的
- 如果将来想换工具，artifacts（markdown 文件）能保留，但工作流脚本需重做

## Implementation Notes

**已完成**：

- [x] 全局安装 OpenSpec v1.3.1
- [x] 在 paam 仓库执行 `openspec init --tools claude`
- [x] 生成 `openspec/` 目录结构与 `.claude/` 配置
- [x] 在 `PROCESS.md` 写入 OpenSpec 工作流（§2.3 路径 A）
- [x] 在 `PROCESS.md §9.2` 更新仓库结构图

**待做**：

- [ ] M1 启动后，用第一个 feature（建议 `paam-track`）走完整 OpenSpec 流程，作为工作流试运行
- [ ] 视试运行结果，决定是否需要在 `PROCESS.md` 增补具体使用约定（如：何时触发 `/opsx:explore`）
- [ ] 第一次 commit 时把 `openspec/` 和 `.claude/` 一起 commit

**重启 IDE 触发 slash commands**：`openspec init` 提示需要重启 Claude Code（或新开会话）才能加载 `/opsx:*` slash commands。

## References

- 工具：[Fission-AI/OpenSpec](https://github.com/Fission-AI/OpenSpec) (MIT, v1.3.1)
- 流程定义：[`PROCESS.md §2.3 路径 A`](../PROCESS.md)
- 相关 ADR：暂无（独立的工具决策）
- 相关 Issue / 讨论：本会话讨论（2026-04-25 ~ 2026-04-26）

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-26 | Accepted | 完成评估，安装并初始化；待 M1 启动后实战验证 |
