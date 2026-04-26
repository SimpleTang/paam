# paam — Private AI Asset Manager

> Manage your AI Agent assets — Skills, Prompts, MCP Servers — in a unified, private, portable way.

**Status**: 🟡 Pre-alpha · 设计与原型阶段

---

## 是什么

paam 是面向所有 AI Agent 用户的**私有资产管理工具**。

随着 AI Agent 在工作流中越来越深入，每个用户都在不断积累 Skills、Prompts、MCP Servers 等资产。它们目前散落在本地文件夹、GitHub Gist、私有仓库、Notion 笔记里。每次换 Agent（Claude Code → Cursor → Codex）就要重新搬运一遍。

paam 的使命：**让 AI Agent 资产像代码一样被管理 —— 可版本化、可分发、可追溯、跨工具流通，并完全归用户私有。**

详见 [PRODUCT.md](./PRODUCT.md)。

## 核心特性（Phase 1）

- 📦 **Skills 全生命周期管理**：track / install / sync / publish / update / uninstall
- 🌐 **Git 即 Registry**：复用现有 Git 基础设施，无需中心化服务
- 🖥️ **UI / CLI 双形态**：开发者用 CLI，非开发者用 UI，能力对等
- 🔄 **多 Agent 分发**：Claude Code / Cursor / Codex（可扩展）
- 🔐 **私有优先**：本地存储，凭据走 OS keychain，不依赖云端
- 🚀 **跨平台**：macOS / Windows（UI + CLI），Linux（CLI）

## 演进路径

```
Phase 1 (当前)        Phase 2             Phase 3
   Skills      ───►   Prompts      ───►   MCP Servers
```

## 项目状态

paam 处于早期设计阶段。当前里程碑：

| Milestone | Version | 状态 |
|---|---|---|
| M1 — 技术原型 | `v0.1.0` | 🟡 规划中 |
| M2 — CLI 完整版 | `v0.2.0` | ⏳ 待启动 |
| M3 — 桌面 UI MVP | `v0.3.0` | ⏳ 待启动 |
| M4 — Windows 适配 | `v0.4.0` | ⏳ 待启动 |
| M5 — 正式发布 | `v1.0.0` | ⏳ 待启动 |

## 文档

- [PRODUCT.md](./PRODUCT.md) — 产品宪章（定位、愿景、永久边界）
- [CHANGELOG.md](./CHANGELOG.md) — 更新日志

## 协议

待定。

## 贡献

paam 目前由 [@simpletang1994](https://github.com/simpletang1994) 独立开发，欢迎通过 Issues 提出建议与反馈。
正式的 `CONTRIBUTING.md` 将在 M2 之后补充。
