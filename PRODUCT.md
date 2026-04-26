# paam — Product Charter

> **paam** = **P**rivate **A**I **A**sset **M**anager
> 本文档承载产品长期定位、愿景与永久边界，不随单期开发频繁变动。
> 本期具体功能需求请见 `.dev/docs/milestones/M{N}-plan.md`。
>
> Last Updated: 2026-04-25

---

## 一、产品定位

### 1.1 一句话定义

**paam 是一个面向所有 AI Agent 用户的私有资产管理工具**——以统一、私有、可移植的方式管理用户在 AI 生态中积累的各类资产。

### 1.2 解决的问题

随着 AI Agent 在工作流中越来越深入，每个用户都在不断积累三类资产：

- **Skills**：扩展 Agent 能力的模块化指令包（Anthropic Agent Skills 规范）
- **Prompts**：精心打磨的提示词、Persona、System Message
- **MCP Servers**：Model Context Protocol 服务器及其配置
- 以及未来可能出现的其他 AI 生态资产形态

这些资产目前散落在各处：本地文件夹、GitHub Gist、私有仓库、Notion 笔记、聊天记录里。每次换一个 Agent（Claude Code → Cursor → Codex）就要重新搬一遍；想分享给同事或团队时要手动打包；升级和回滚没有统一机制。

### 1.3 使命

**让 AI Agent 资产像代码一样被管理——可版本化、可分发、可追溯、跨工具流通，并完全归用户私有。**

### 1.4 目标用户

**所有使用 AI Agent 的用户**，不限定技术背景：

- **个人开发者**：在 Claude Code / Cursor / Codex 之间切换，希望自己的 prompt 和 skill 库跟着自己走
- **AI 重度用户**：积累了大量 prompt 和工作流，需要一个家来管理它们
- **团队 / 小组**：希望把团队最佳实践沉淀为可复用的 Skills，让新人快速对齐
- **创作者**：愿意把自己的 Skills 发布出去给社区或团队使用

开发者用 CLI，非开发者用 UI，二者能力完全对等。

### 1.5 核心价值主张

| # | 价值 | 解决什么问题 |
|---|---|---|
| 1 | 一处管理，多端分发 | Agent 切换时不用手动搬运资产 |
| 2 | Git 即 Registry | 复用现有 Git 基础设施，无需额外服务，天然支持版本管理与协作 |
| 3 | UI / CLI 双形态 | 兼顾日常浏览和自动化场景 |
| 4 | 跨资产类型统一抽象 | Skill / Prompt / MCP 用同一套管理范式，降低认知负担 |
| 5 | 私有优先 | 资产存储在本地、由用户掌控、不依赖任何中心化服务 |
| 6 | 开放与可移植 | 资产格式遵循开放规范（agentskills.io 等），不绑定 paam |

### 1.6 产品形态

- **桌面端应用**：macOS（优先）、Windows
- **命令行工具**：macOS / Windows / Linux 全平台
- **共享核心库**：UI 与 CLI 共享同一份核心实现，保证行为一致

CLI 命令：`paam`（4 字母）

---

## 二、长期愿景

### 2.1 Roadmap 概览

```
┌─────────────────────────────────────────────────────────────┐
│                      paam 演进路线                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 1                Phase 2              Phase 3        │
│  ┌──────────┐          ┌──────────┐         ┌──────────┐    │
│  │  Skills  │   ───►   │ Prompts  │  ───►   │   MCP    │    │
│  │  管理    │          │  管理    │         │  Servers │    │
│  └──────────┘          └──────────┘         └──────────┘    │
│                                                             │
│              共享基础设施(Git / 凭据 / 同步)                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 各阶段定位

**Phase 1：Skills 管理**

完整覆盖 Skills 全生命周期：订阅、安装、本地开发、发布、升级、卸载。Skills 是当前 AI Agent 生态中规范最成熟、用户痛点最明确的资产类型，先把它做透，验证产品形态。

**Phase 2：Prompts 管理**

把 Prompts 当作"轻量化 Skill"纳入同一套管理体系：

- 类似 Skill 的目录结构，但更简单（只需一个 markdown 文件 + frontmatter）
- 支持变量模板、版本化、Tag 分类
- 一键复制到剪贴板 / 注入到 Agent

**Phase 3：MCP Servers 管理**

管理 MCP Server 的配置、密钥、运行状态：

- MCP Server 列表与一键启停
- 配置分发到各 Agent 客户端的 mcp config 文件
- 密钥安全存储

**未来可能扩展**：Agent 配置文件统一管理、自定义 Tool 管理、社区市场等。

### 2.3 里程碑总览（Phase 1）

| Milestone | Version | 主题 | 详细 PRD |
|---|---|---|---|
| M1 | `v0.1.0` | 技术原型（CLI 核心） | [`.dev/docs/milestones/M1-plan.md`](.dev/docs/milestones/M1-plan.md) |
| M2 | `v0.2.0` | CLI 完整版 | （未启动） |
| M3 | `v0.3.0` | 桌面 UI MVP（macOS） | （未启动） |
| M4 | `v0.4.0` | Windows 适配 | （未启动） |
| M5 | `v1.0.0` | 完善与正式发布 | （未启动） |

每个 milestone 在启动时创建对应的 plan 文件，承担"本期 PRD"角色。

---

## 三、用户场景

### 3.1 个人用户场景

**S1：跨 Agent 切换无感**
张同学是个独立开发者，主力用 Claude Code，偶尔会切到 Cursor 试新功能。他在 paam 里维护自己的 Skill 库，无论用哪个 Agent 都能立刻看到所有自己写的 Skill。

**S2：自我资产积累**
李同学是 AI 重度用户，写了不少好用的 Skill。他把自己的 Skills 推到一个个人 GitHub 私有仓，换电脑后只需登录 paam、添加这个仓，就能一键拉齐所有 Skill。

**S3：尝试社区资产**
王同学发现 GitHub 上有人开源了一组优秀的 Skills。他在 paam 里添加这个仓地址，浏览预览后挑选感兴趣的安装，想升级时一键完成。

### 3.2 团队场景

**S4：团队最佳实践沉淀**
某团队把内部的代码审查规范、部署流程、合规检查等沉淀为 Skills，存在团队 GitLab 仓库。新人入职配置好 paam 并 restore 一次，所有团队 Skill 立刻可用。

**S5：跨团队协作**
前端组、后端组各维护自己的 Skill 仓。开发者按需订阅自己关心的几个团队仓，paam 自动合并并分发。

**S6：贡献回团队**
开发者在本地开发了一个新 Skill，验证好用之后通过 paam 一键发布到团队仓（背后是 git push + tag），团队其他成员的 paam 检测到更新并提示升级。

---

## 四、永久边界（paam 不做什么）

以下边界跨越所有 Phase，是 paam 的产品定位红线：

- ❌ **paam 不是 Skill 编辑器**：不内置 markdown / yaml 编辑能力，引导用户用 IDE
- ❌ **paam 不是 Skill 运行时**：不执行 Skill 里的脚本，只负责管理与分发
- ❌ **paam 不替代 Git**：所有持久化的远程操作都是对 Git 仓的薄封装
- ❌ **paam 不是云服务**：所有数据本地存储，用户完全掌控
- ❌ **paam 不做中心化 Registry 服务端**：依赖 Git 即可
- ❌ **paam 不做账号体系 / 云端同步**
- ❌ **paam 不做 Skill 推荐算法 / 评分系统**
- ❌ **paam 不主动提供 Web 端**

---

## 五、非功能性需求

### 5.1 性能基线

| 指标 | 目标 |
|---|---|
| CLI 启动时间（无网络命令） | < 200ms |
| UI 冷启动 | < 2s |
| 1000 个 Skill 的 list 操作 | < 500ms |
| 50+ Skills 批量更新 | 并发，UI 不阻塞 |

### 5.2 安全要求

- 凭据零落盘（除 OS keychain）
- 发布前必跑 secrets 扫描
- 安装前可选 audit（默认开启 critical 级别拦截）
- 错误日志中自动 sanitize token
- 不上传用户 Skill 内容到任何第三方服务

### 5.3 兼容性

- macOS 12+
- Windows 10 22H2+
- 依赖系统 git 2.31+（凭据注入需要）
- Linux（CLI only）：主流发行版

### 5.4 可观测性

- 本地操作日志（默认 7 天滚动）
- 错误上报：可选、用户可关闭
- 不收集 Skill 内容、不收集仓库地址
- 使用统计：可选、匿名

---

## 六、附录

### 6.1 术语表

| 术语 | 定义 |
|---|---|
| Skill | Anthropic Agent Skills 规范定义的能力包，包含 SKILL.md 及附属文件 |
| Source | paam 的本地资产存储目录，所有 Skill 的 single source of truth |
| Target | Agent 的工作目录（例如 `~/.claude/skills/`），Source 通过 sync 分发到这里 |
| Tracked Repo | 通过 `track` 订阅的 Git 仓库，作为 Skills 的来源 |
| Provenance | Skill 的来源元数据：仓库 URL、commit、tree hash、安装时间等 |
| Tree Hash | Git 子目录的 SHA，用于精准检测 Skill 内容变更 |

### 6.2 命名说明

**paam** 是 **P**rivate **A**I **A**sset **M**anager 的缩写。

- 命令名小写：`paam`（参照 git / npm / brew 的惯例）
- 文档中介绍产品时可使用全称："paam (Private AI Asset Manager)"
- 4 字母 CLI 命令兼顾简短性和独占性

### 6.3 参考资料

- Agent Skills 开放规范：https://agentskills.io
- Anthropic Agent Skills 文档：https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview
- 参考实现：
  - skillshare（Go，多 Agent 同步）
  - skilluse（TypeScript，GitHub-based）
  - skills-cli（Python，跨平台 CLI）
  - SkillHub（Java，企业级 registry）

---

**变更记录**：

| 版本 | 日期 | 变更 |
|---|---|---|
| v0.1 (PRD) | 2026-04-25 | 初始版本，作为综合 PRD 文档（详见 `.dev/docs/archived/PRD-v0.1-design-snapshot.md`） |
| Charter | 2026-04-25 | 重构为产品宪章：移除本期功能详情（迁入 milestone plans）、技术设计（迁入 ADR）、待决策项（已在 ADR） |
