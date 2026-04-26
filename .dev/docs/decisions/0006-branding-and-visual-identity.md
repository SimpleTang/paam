# ADR-0006: 品牌与视觉系统

- **Status**: Proposed
- **Date**: 2026-04-25
- **Deciders**: @simpletang1994
- **Tags**: ux | release

## Context

设计阶段将"Logo、图标、配色——是否需要在 M3 之前敲定"列为待决策项（见 `.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #6）。

视觉系统涉及：

- **Logo**：仓库 README、网站、应用图标
- **应用图标**：macOS / Windows 桌面应用图标（多分辨率）
- **配色**：UI 主题色、深浅色模式
- **字体**：UI 字体、文档字体
- **品牌语调**：README、文档、错误信息、CLI 输出风格

paam 的产品调性（参考 [`PRODUCT.md`](../../../PRODUCT.md) §1）：

- 私有、可掌控（vs 中心化云服务）
- 工具属性强（参照 git / npm / cargo）
- 跨平台、跨 Agent

## Decision

待决策。

## Alternatives Considered

### 时机

#### Option A: M1-M2 期间确定（早）

- **Pros**: UI MVP 时即可使用最终视觉
- **Cons**: 早期形态未定，可能浪费

#### Option B: M3 启动前确定（中）

- **Pros**: 已有 CLI 完整版反馈，对产品理解更清晰
- **Cons**: M3 工期更紧

#### Option C: M5 之前确定（晚）

- **Pros**: 完整产品形态明确后再投入
- **Cons**: M3-M4 期间使用占位视觉，发布前需要批量替换

### 投入方式

- 自己设计（成本低、可能不专业）
- 委托设计师（成本中、质量保障）
- 使用开源图标 + 简单调整（极低成本，原创性弱）
- AI 辅助生成 + 微调（中间路线）

## Consequences

待决策后补充。

## Implementation Notes

建议先确定的最小集（即使不做完整视觉系统）：

- 项目名称大小写约定（已定：小写 `paam`）
- 主色（用于 README badges、文档高亮）
- 应用图标的"占位版"（M3 启动前需要）

完整视觉系统可推迟到 M5 之前。

## References

- PRODUCT.md §1 产品定位、§6.2 命名说明
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #6

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
