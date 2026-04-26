# ADR-0004: 是否兼容 skillshare / skilluse 的配置格式

- **Status**: Proposed
- **Date**: 2026-04-25
- **Deciders**: @simpletang1994
- **Tags**: data-format | ux

## Context

设计阶段将"是否兼容现有约定"列为待决策问题（见 `.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #4）。

参考项目：

- **skillshare**（Go，多 Agent 同步）
- **skilluse**（TypeScript，GitHub-based）
- **skills-cli**（Python，跨平台 CLI）
- **SkillHub**（Java，企业级 registry）

如果兼容这些工具的配置格式，新用户可以平滑迁移到 paam；但兼容会带来格式约束，可能影响 paam 自有特性（例如多源订阅、provenance 元数据等）的设计。

## Decision

待决策。

## Alternatives Considered

### Option A: 完全兼容现有约定

- **Pros**:
  - 用户迁移成本低
  - 可直接读取已有配置
- **Cons**:
  - 受限于已有格式的局限
  - 现有工具之间格式也不统一，"兼容哪个"本身是问题

### Option B: 自有格式 + 提供导入工具

- **Pros**:
  - 设计自由，可以为 paam 的多源 / 多 target / provenance 等特性优化
  - 一次性导入工具的成本可控
- **Cons**:
  - 用户首次使用有迁移成本
  - 需要为每个参考项目单独写导入逻辑

### Option C: 自有格式 + 不做兼容

- **Pros**:
  - 实现最简单
- **Cons**:
  - 已有用户迁移困难
  - 可能错过早期种子用户

## Consequences

待决策后补充。

## Implementation Notes

需先调研：

- skillshare / skilluse / skills-cli / SkillHub 的实际配置文件结构
- 它们与 Anthropic Agent Skills 开放规范的关系
- 用户基数（哪个工具用户多）

## References

- PRODUCT.md §6.3 参考资料（含参考实现列表）
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #4、§10.3
- [agentskills.io](https://agentskills.io/)

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
