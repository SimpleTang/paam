# Architecture Decision Records (ADR)

本目录记录 paam 项目的所有架构与关键决策。

## 什么是 ADR？

ADR (Architecture Decision Record) 是一种轻量级文档形式，用于记录"我们做了什么决策、为什么这么决策、考虑过哪些替代方案"。

详见：[adr.github.io](https://adr.github.io/)

## 何时写 ADR

详见 [`PROCESS.md §六`](../PROCESS.md)。简言之——**6 个月后的我会问"为什么这样做"** 的事，就该写。

## 索引

| # | 标题 | 状态 | 日期 |
|---|---|---|---|
| [0001](./0001-tech-stack-choice.md) | 桌面端与核心库技术栈选型 → **Tauri (Rust)** | 🟢 Accepted | 2026-04-26 |
| [0002](./0002-shared-core-strategy.md) | CLI 与 UI 核心库共享策略 → **Cargo workspace** | 🟢 Accepted | 2026-04-26 |
| [0003](./0003-open-source-license.md) | 开源协议选择 → **MIT** | 🟢 Accepted | 2026-04-27 |
| [0004](./0004-config-format-compatibility.md) | 是否兼容 skillshare/skilluse 配置格式 | 🟡 Proposed | 2026-04-25 |
| [0005](./0005-distribution-channels.md) | 公开发布渠道与代码签名 | 🟡 Proposed | 2026-04-25 |
| [0006](./0006-branding-and-visual-identity.md) | 品牌与视觉系统 | 🟡 Proposed | 2026-04-25 |
| [0007](./0007-phase-extension-design.md) | 数据架构与 Phase 2/3 扩展策略 | 🟢 Accepted | 2026-04-26 |
| [0008](./0008-adopt-openspec.md) | 采用 OpenSpec 作为执行层 spec 工具 | 🟢 Accepted | 2026-04-26 |

## 状态图标

- 🟡 **Proposed**：已识别决策点，尚未决定
- 🟢 **Accepted**：已决策并执行
- 🔴 **Rejected**：评估后拒绝（保留记录避免重新讨论）
- ⚫ **Deprecated**：曾经接受但不再使用
- 🟠 **Superseded**：被新 ADR 取代

## 模板

新建 ADR 时复制 [template.md](./template.md)，编号取当前最大值 +1。
