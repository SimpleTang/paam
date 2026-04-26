# paam — Private AI Asset Manager

> **paam** = **P**rivate **A**I **A**sset **M**anager
> Version: 0.1
> Last Updated: 2026-04-25
> Status: Draft

---

## 一、产品概述

### 1.1 产品定位

**paam 是一个面向所有 AI Agent 用户的私有资产管理工具**——以统一、私有、可移植的方式管理用户在 AI 生态中积累的各类资产。

随着 AI Agent 在工作流中越来越深入，每个用户都在不断积累三类资产：

- **Skills**：扩展 Agent 能力的模块化指令包（Anthropic Agent Skills 规范）
- **Prompts**：精心打磨的提示词、Persona、System Message
- **MCP Servers**：Model Context Protocol 服务器及其配置
- 以及未来可能出现的其他 AI 生态资产形态

这些资产目前散落在各处：本地文件夹、GitHub Gist、私有仓库、Notion 笔记、聊天记录里。每次换一个 Agent（Claude Code → Cursor → Codex）就要重新搬一遍；想分享给同事或团队时要手动打包；升级和回滚没有统一机制。

paam 的使命：**让 AI Agent 资产像代码一样被管理——可版本化、可分发、可追溯、跨工具流通，并完全归用户私有**。

### 1.2 目标用户

**所有使用 AI Agent 的用户**，包括但不限于：

- **个人开发者**：在 Claude Code / Cursor / Codex 之间切换，希望自己的 prompt 和 skill 库跟着自己走
- **AI 重度用户**：积累了大量 prompt 和工作流，需要一个家来管理它们
- **团队/小组**：希望把团队最佳实践沉淀为可复用的 Skills，让新人快速对齐
- **创作者**：愿意把自己的 Skills 发布出去给社区或团队使用

不限定技术背景。开发者用 CLI，非开发者用 UI，二者能力完全对等。

### 1.3 核心价值主张

| # | 价值 | 解决什么问题 |
|---|---|---|
| 1 | 一处管理，多端分发 | Agent 切换时不用手动搬运资产 |
| 2 | Git 即 Registry | 复用现有 Git 基础设施，无需额外服务，天然支持版本管理与协作 |
| 3 | UI/CLI 双形态 | 兼顾日常浏览和自动化场景 |
| 4 | 跨资产类型统一抽象 | Skill / Prompt / MCP 用同一套管理范式，降低认知负担 |
| 5 | 私有优先 | 资产存储在本地、由用户掌控、不依赖任何中心化服务 |
| 6 | 开放与可移植 | 资产格式遵循开放规范（agentskills.io 等），不绑定 paam |

### 1.4 产品形态

- **桌面端应用**：macOS（优先）、Windows
- **命令行工具**：macOS / Windows / Linux 全平台
- **共享核心库**：UI 与 CLI 共享同一份核心实现，保证行为一致

CLI 命令：`paam`（4 字母）

---

## 二、长期愿景与版本规划

### 2.1 长期愿景（Roadmap 概览）

```
┌─────────────────────────────────────────────────────────────┐
│                      paam 演进路线                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 1 (当前)         Phase 2              Phase 3        │
│  ┌──────────┐          ┌──────────┐         ┌──────────┐    │
│  │  Skills  │   ───►   │ Prompts  │  ───►   │   MCP    │    │
│  │  管理    │          │  管理    │         │  Servers │    │
│  └──────────┘          └──────────┘         └──────────┘    │
│                                                             │
│              共享基础设施(Git / 凭据 / 同步)                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 各阶段范围

**Phase 1：Skills 管理（本期目标）**

第一期完整覆盖 Skills 全生命周期：订阅、安装、本地开发、发布、升级、卸载。Skills 是当前 AI Agent 生态中规范最成熟、用户痛点最明确的资产类型，先把它做透，验证产品形态。

**Phase 2：Prompts 管理（规划中）**

把 Prompts 当作"轻量化 Skill"纳入同一套管理体系：

- 类似 Skill 的目录结构，但更简单（只需一个 markdown 文件 + frontmatter）
- 支持变量模板、版本化、Tag 分类
- 一键复制到剪贴板 / 注入到 Agent

**Phase 3：MCP Servers 管理（规划中）**

管理 MCP Server 的配置、密钥、运行状态：

- MCP Server 列表与一键启停
- 配置分发到各 Agent 客户端的 mcp config 文件
- 密钥安全存储

**未来可能扩展**：Agent 配置文件统一管理、自定义 Tool 管理、社区市场等。

### 2.3 第一期发布范围（边界）

**做**：
- Skills 全生命周期（track / install / sync / publish / update / uninstall）
- 多 Agent 目标支持（Claude Code、Cursor、Codex，可扩展）
- macOS / Windows 桌面 UI
- 跨平台 CLI
- 个人 + 团队两种使用场景

**不做**：
- 中心化 Registry 服务端（依赖 Git 即可）
- 云端同步 / 账号体系
- Web 端
- 内置编辑器（用户用自己习惯的 IDE）
- Skill 推荐算法 / 评分系统
- 社区市场（Phase 2+ 再考虑）

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

### 3.3 关键非用户场景（明确边界）

- ❌ paam 不是 Skill 编辑器：不内置 markdown / yaml 编辑能力，引导用户用 IDE
- ❌ paam 不是 Skill 运行时：不执行 Skill 里的脚本，只负责管理与分发
- ❌ paam 不替代 Git：所有持久化的远程操作都是对 Git 仓的薄封装
- ❌ paam 不是云服务：所有数据本地存储，用户完全掌控

---

## 四、第一期功能模块

### 4.1 模块概览

```
┌────────────────────────────────────────────────────────────┐
│                       paam 第一期                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐      │
│   │ M1. 订阅源  │   │ M2. 本地    │   │ M3. Agent   │      │
│   │   管理      │   │ Skills 管理 │   │   分发      │      │
│   └─────────────┘   └─────────────┘   └─────────────┘      │
│                                                            │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐      │
│   │ M4. 本地    │   │ M5. 发布    │   │ M6. 升级    │      │
│   │   开发      │   │             │   │             │      │
│   └─────────────┘   └─────────────┘   └─────────────┘      │
│                                                            │
│   ┌─────────────┐   ┌─────────────┐                        │
│   │ M7. 配置与  │   │ M8. 状态    │                        │
│   │   凭据      │   │   查询      │                        │
│   └─────────────┘   └─────────────┘                        │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 4.2 M1：订阅源管理

订阅一个或多个 Git 仓库作为 Skills 来源。

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 添加订阅仓（SSH / HTTPS / GitHub owner/repo 简写） | ✓ | `paam track <url>` | P0 |
| 列出已订阅仓 | ✓ | `paam track list` | P0 |
| 移除订阅仓（含本地缓存清理） | ✓ | `paam untrack <alias>` | P0 |
| 浏览仓内 Skills（不安装） | ✓ | `paam track skills <alias>` | P0 |
| 切换分支 / 重新指定 ref | ✓ | `paam track switch <alias>` | P1 |
| 仓级 include / exclude 规则 | ✓ | 配置文件 | P1 |
| 后台自动检测更新 | ✓ | 守护选项 | P1 |
| 仓级凭据管理 | ✓ | `paam auth set <alias>` | P0 |

**关键支持**：
- 任意自托管 Git（GitLab / Gitea / Gerrit / 自建）
- GitHub / GitLab.com 公共仓
- 个人私有仓（PAT 鉴权）
- 团队私有仓（SSH key 走系统 ssh-agent）
- 自签 SSL 证书（可配置 CA bundle）

### 4.3 M2：本地 Skills 管理

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 安装单个 Skill | ✓ | `paam install <skill>` | P0 |
| 卸载 Skill | ✓ | `paam uninstall <skill>` | P0 |
| 启用 / 禁用（不删除文件） | ✓ | `paam enable/disable <skill>` | P0 |
| Pin 到特定版本 / commit | ✓ | `paam pin <skill> <ref>` | P1 |
| 同名冲突解决（多源同名） | ✓ 弹窗 | `--prefer <alias>` | P0 |
| Dry-run 预览变更 | ✓ | `--dry-run` | P1 |

### 4.4 M3：Agent 分发（多目标）

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 自动检测已安装的 Agent | ✓ | `paam target detect` | P0 |
| 列出所有 target 状态 | ✓ | `paam target list` | P0 |
| 启用 / 禁用 target | ✓ 开关 | `paam target enable/disable <n>` | P0 |
| 自定义 target 路径 | ✓ | `paam target add <n> <path>` | P1 |
| target 级 include / exclude | ✓ | 配置文件 | P1 |
| symlink / copy 模式切换 | ✓ | `--mode symlink|copy` | P0 |
| 一键全量同步 | ✓ | `paam sync` | P0 |

**第一期支持的 Agent**：
- Claude Code (`~/.claude/skills/`)
- Cursor (`~/.cursor/skills/`)
- Codex (`~/.codex/skills/`)
- 自定义路径

**模式默认值**：
- macOS：默认 symlink
- Windows：默认 copy（symlink 需管理员权限或开发者模式）

### 4.5 M4：本地 Skill 开发

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 新建 Skill（脚手架，含 SKILL.md 模板） | ✓ 引导式 | `paam new <n>` | P0 |
| 浏览本地 Skills（含未发布） | ✓ | `paam list --local` | P0 |
| 校验 SKILL.md 格式 | ✓ 实时 | `paam lint <skill>` | P0 |
| 在外部编辑器打开 | ✓ | `paam open <skill>` | P1 |
| 复制已有 Skill 作为模板 | ✓ | `paam clone <skill> <new-name>` | P1 |
| 预览 SKILL.md 渲染效果 | ✓ | - | P2 |

### 4.6 M5：发布

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 发布到指定仓 | ✓ 引导式 | `paam publish <skill> --repo <url>` | P0 |
| 发布前校验 | ✓ | 自动 | P0 |
| 发布前安全扫描（secrets / 危险命令） | ✓ 报告 | 自动 | P0 |
| 自动版本号 bump | ✓ 选择 | `--bump patch|minor|major` | P0 |
| 自定义版本号 | ✓ | `--version <ver>` | P0 |
| 发布到指定分支 / 打 tag | ✓ | `--branch / --tag` | P1 |
| Dry-run 预览要推什么 | ✓ | `--dry-run` | P1 |
| 标记废弃 | ✓ | `paam deprecate <skill> <version>` | P2 |

**发布机制**：
```
本地 skill → 临时 clone 目标仓 → 拷贝文件 → commit → tag → push
```

### 4.7 M6：升级

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 后台检测更新 | ✓ 红点提示 | 守护选项 | P0 |
| 列出 outdated Skills | ✓ | `paam outdated` | P0 |
| 一键升级全部 / 单个 | ✓ | `paam update [skill] / --all` | P0 |
| 升级前查看 diff | ✓ 预览 | `paam diff <skill>` | P1 |
| 增量检测（commit + tree hash 双层） | 自动 | 自动 | P0 |
| 升级失败回滚 | 自动 | 自动 | P0 |

### 4.8 M7：配置与凭据

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 全局设置 | ✓ | `paam config get/set` | P0 |
| 凭据管理 | ✓ | `paam auth list/set/remove` | P0 |
| 代理配置 | ✓ | 配置项 | P0 |
| 自签 CA bundle 配置 | ✓ | 环境变量 | P1 |
| 导入 / 导出配置 | ✓ | `paam config export/import` | P1 |

**关键约束**：
- Token 必须存 OS keychain（macOS Keychain、Windows Credential Manager），配置文件只存 keychain 引用
- 必须支持 HTTP / SOCKS 代理

### 4.9 M8：状态查询

| 功能 | UI | CLI | 优先级 |
|---|---|---|---|
| 列出所有已安装 Skill | ✓ | `paam list` | P0 |
| 查看单个 Skill 详情 | ✓ | `paam info <skill>` | P0 |
| 搜索 | ✓ | `paam search <kw>` | P0 |
| 按来源仓筛选 | ✓ | `paam list --from <alias>` | P1 |
| 健康检查 | ✓ | `paam doctor` | P1 |
| 操作历史 | ✓ Activity 页 | `paam log` | P1 |

---

## 五、产品形态详细设计

### 5.1 命令行（CLI）

**设计原则**：
- 单个二进制文件，跨平台
- 命令风格参考 `npm` / `cargo` / `gh`
- 支持 `--json` 输出便于脚本集成
- 支持 shell 补全（bash / zsh / fish / powershell）
- 错误信息要给出修复建议（参考 git 凭据失败时的可执行提示）

**典型命令**：
```bash
paam track git@gitlab.com:my-team/skills.git
paam sync
paam list
paam install pdf-review
paam new my-new-skill
paam publish my-new-skill --repo git@github.com:me/my-skills.git --bump minor
paam update --all
```

可选简写别名 `pa`（如果 `pa` 名空间冲突可放弃）。

### 5.2 桌面 UI

**技术选型建议**（待决策）：
- 选项 A：Tauri（Rust + Web 前端）—— 包体积小、内存低、与 CLI 天然共享 Rust 核心
- 选项 B：Electron（Node + Web 前端）—— 生态成熟、上手快
- 选项 C：Wails（Go + Web 前端）—— 介于两者之间

**主要视图**：

| 视图 | 内容 |
|---|---|
| Dashboard | 已安装 Skills 总览、待升级提示、最近活动、快捷操作入口 |
| Sources | 订阅仓列表 + 详情 |
| Skills | 全部 Skills 表格(含本地未发布)，筛选 / 搜索 |
| Skill Detail | 单 Skill 详情(frontmatter / 文件树 / provenance / 历史版本) |
| Targets | Agent 目标管理 |
| Activity | 操作历史 |
| Settings | 配置 / 凭据 / 代理 |

**系统集成**：
- macOS：菜单栏 + 状态栏图标
- Windows：系统托盘图标 + 通知中心

---

## 六、数据与配置

### 6.1 配置文件位置

| 平台 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/paam/` |
| Windows | `%APPDATA%\paam\` |
| Linux（仅 CLI） | `~/.config/paam/` |

### 6.2 关键文件

| 文件 | 内容 | 是否随项目走 |
|---|---|---|
| `paam.yaml` | 用户级订阅与配置 | 否（用户级） |
| `.paam/paam.yaml` | 项目级配置 | 是（建议 commit） |
| `.paam/paam.lock` | 锁定的 commit / tree hash | 是 |
| `.metadata.json` | 已安装 Skill 状态 | 否 |
| OS Keychain | 所有凭据 | 否 |

### 6.3 source 目录结构

```
~/Library/Application Support/paam/skills/
├── _<repo-alias-1>/              # tracked repo (git clone)
│   ├── .git/
│   ├── <skill-a>/SKILL.md
│   └── <skill-b>/SKILL.md
├── _<repo-alias-2>/
│   └── ...
├── <local-skill>/                # 本地未发布 skill
│   └── SKILL.md
├── .metadata.json
└── .skillignore
```

订阅仓以 `_` 前缀子目录形式并列存在，本地 skill 直接放根目录下。

---

## 七、非功能性需求

### 7.1 性能

| 指标 | 目标 |
|---|---|
| CLI 启动时间(无网络命令) | < 200ms |
| UI 冷启动 | < 2s |
| 1000 个 Skill 的 list 操作 | < 500ms |
| 50+ Skills 批量更新 | 并发，UI 不阻塞 |

### 7.2 安全

- 凭据零落盘（除 OS keychain）
- 发布前必跑 secrets 扫描
- 安装前可选 audit（默认开启 critical 级别拦截）
- 错误日志中自动 sanitize token
- 不上传用户 Skill 内容到任何第三方服务

### 7.3 兼容性

- macOS 12+
- Windows 10 22H2+
- 依赖系统 git 2.31+（凭据注入需要）
- Linux（CLI only）：主流发行版

### 7.4 可观测性

- 本地操作日志（默认 7 天滚动）
- 错误上报：可选、用户可关闭
- 不收集 Skill 内容、不收集仓库地址
- 使用统计：可选、匿名

---

## 八、里程碑规划

| 阶段 | 目标 | 工作量估计 |
|---|---|---|
| M1：技术原型 | CLI 核心：track / install / sync / list，单一 target（Claude Code），SSH 鉴权 | 2~3 周 |
| M2：CLI 完整版 | 加上 publish / update / uninstall / disable，多 target，HTTPS 凭据 | 2~3 周 |
| M3：桌面 UI MVP（macOS） | UI 覆盖 P0 功能，与 CLI 共享核心库 | 3~4 周 |
| M4：Windows 适配 | Windows UI + CLI，copy 模式默认，credential manager 集成 | 2 周 |
| M5：完善与发布 | doctor、diff 预览、安全审计、自动更新检测、安装包签名 | 2 周 |

合计约 11~15 周。

---

## 九、待决策的问题

进入设计阶段前需要明确：

1. **技术栈**：Tauri / Electron / Wails 三选一
2. **CLI 与 UI 共享核心**：必须共享还是允许独立实现？
3. **开源 / 闭源**：影响代码风格、文档投入、依赖选择
4. **是否兼容现有约定**：要不要兼容 skillshare / skilluse 的配置格式（方便用户迁移）
5. **公开发布渠道**：自主分发 / Homebrew / Microsoft Store？签名证书如何处理？
6. **品牌与视觉**：Logo、图标、配色——是否需要在 M3 之前敲定？
7. **第一期是否预留 Phase 2/3 扩展点**：例如配置文件结构是否预留 `prompts:` `mcp:` 字段？

---

## 十、附录

### 10.1 术语表

| 术语 | 定义 |
|---|---|
| Skill | Anthropic Agent Skills 规范定义的能力包，包含 SKILL.md 及附属文件 |
| Source | paam 的本地资产存储目录，所有 Skill 的 single source of truth |
| Target | Agent 的工作目录（例如 `~/.claude/skills/`），Source 通过 sync 分发到这里 |
| Tracked Repo | 通过 `track` 订阅的 Git 仓库，作为 Skills 的来源 |
| Provenance | Skill 的来源元数据：仓库 URL、commit、tree hash、安装时间等 |
| Tree Hash | Git 子目录的 SHA，用于精准检测 Skill 内容变更 |

### 10.2 命名说明

**paam** 是 **P**rivate **A**I **A**sset **M**anager 的缩写。

- 命令名小写：`paam`（参照 git / npm / brew 的惯例）
- 文档中介绍产品时可使用全称："paam (Private AI Asset Manager)"
- 4 字母 CLI 命令兼顾简短性和独占性

### 10.3 参考资料

- Agent Skills 开放规范：https://agentskills.io
- Anthropic Agent Skills 文档：https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview
- 参考实现：
  - skillshare（Go，多 Agent 同步）
  - skilluse（TypeScript，GitHub-based）
  - skills-cli（Python，跨平台 CLI）
  - SkillHub（Java，企业级 registry）

---

**文档维护者**：[待定]
**变更记录**：

| 版本 | 日期 | 变更 |
|---|---|---|
| 0.1 | 2026-04-25 | 初始版本，覆盖 Phase 1 范围。产品名定为 paam |
