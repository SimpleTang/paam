# paam 项目开发流程规范

> 本文档定义 paam 的项目管理与版本迭代规范。
> **本文档仅记录作者个人开发习惯，不对外强制——外部贡献者只需关注 PR 流程。**
> 适用范围：单人开发 + AI 辅助 + 开源（GitHub）模式。
> Last Updated: 2026-04-25

---

## 一、核心理念

paam 是单人独立开发的开源项目，借助 AI 辅助推进。流程设计遵循以下原则：

| 原则 | 含义 |
|---|---|
| **版本驱动，非时间驱动** | 节奏由 Milestone / Version 决定，不设固定 sprint 周期 |
| **轻量但可追溯** | 每个关键决策、每次发布都留痕，半年后回看仍能理解 |
| **事件触发回顾** | 不做周/双周复盘；在完成 milestone、决策点、异常信号时反思 |
| **对外可预期** | 通过 SemVer + CHANGELOG + Release notes 给社区清晰预期 |
| **AI 友好** | 文档结构和决策记录便于 AI 上下文加载，加速后续推进 |

---

## 二、工作流模型

### 2.1 两层职责模型

paam 的开发分为**产品设计层**与**执行层**——这是组织所有工作的核心心智模型：

```
┌────────────────────────────────────────────┐
│         产品设计层（想清楚做什么）          │
│                                            │
│  载体：                                    │
│  · PRODUCT.md         — 长期愿景（宪章）   │
│  · M{N}-plan.md       — 本期 PRD           │
│                                            │
│  关心的问题：                              │
│  · 做什么 / 为谁                           │
│  · 不做什么（边界）                        │
│  · 本期出口标准（DoD）                     │
└────────────────────┬───────────────────────┘
                     │ 设计冻结，进入执行
                     ▼
┌────────────────────────────────────────────┐
│         执行层（逐个 feature 落地）         │
│                                            │
│  载体：                                    │
│  · openspec/changes/<feature>/             │
│    （proposal / specs / design / tasks）   │
│                                            │
│  关心的问题：                              │
│  · 怎么做                                  │
│  · 按什么顺序                              │
│  · 什么时候算完                            │
└────────────────────┬───────────────────────┘
                     │ Milestone 全部 changes 完成
                     ▼
┌────────────────────────────────────────────┐
│         发布层                              │
│  · Milestone retro                          │
│  · Tag → CI 构建发布                        │
└────────────────────────────────────────────┘

  ─── 横向：ADR (.dev/docs/decisions/) ───
  产品决策（云同步要不要做？）── 在产品设计层触发
  技术决策（用 git2-rs 还是 shell？）── 在执行层触发
  两层都可触发，统一收纳到同一个 decisions/ 目录
```

**两条铁律**：

1. **产品设计层不关心怎么实现**：设计文档只写"做什么、为谁、不做什么"
2. **执行层不挑战是否要做**：进入执行后默认上一层已决定。发现产品认知问题 → **显式回到产品设计层**更新，不要在 design.md 里偷偷改 scope

### 2.2 三条工作路径

任何变更都属于这三条路径之一：

```
新需求 / 新想法
   │
   ▼
┌─────────────────────────────────────────────┐
│  问题 1：是产品意图变化吗？                  │
│  · 增删 feature / 改产品定位 / 改边界       │
│  · Yes ─► 进产品设计层                      │
│           （更新 PRODUCT.md 或              │
│            milestone plan，可能触发 ADR）   │
│  · No  ─► 继续                              │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│  问题 2：是已规划 feature 的落地吗？         │
│  · Yes ─► 路径 A：OpenSpec change           │
│  · No  ─► 继续                              │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│  问题 3：是横向技术决策吗？                  │
│  · Yes ─► 路径 B：写 ADR                    │
│  · No  ─► 路径 C：直接 PR                   │
└─────────────────────────────────────────────┘
```

| 路径 | 适用场景 | 产物位置 |
|---|---|---|
| **A · OpenSpec change** | 新 feature / 新 CLI 命令 / 新模块 / 用户可见行为变化 | `openspec/changes/<feature>/` |
| **B · ADR** | 技术栈、协议、跨模块设计、产品级根本决策 | `.dev/docs/decisions/` |
| **C · 直接 PR** | bug 修复、依赖升级、CI 调整、文档微调、不改变行为的小重构 | git PR |

**疑惑时的判断标准**：

> "如果半年后我忘了为什么这么实现，光看 commit 和代码能否还原意图？"
>
> · 能 → 路径 C
> · 不能 → 路径 A（值得写 OpenSpec change，把意图固化）

### 2.3 路径 A：完整 Feature 工作流（OpenSpec）

paam 最常走的路径——任何新功能都通过 OpenSpec change 落地：

```
┌─────────────────────────────────────────────────┐
│  前置：milestone plan 已列出本期 features        │
└─────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ① PROPOSE — 启动 change                         │
│  /opsx:propose <feature-name>                   │
│  生成 openspec/changes/<feature-name>/          │
│   ├─ proposal.md   ← 动机、范围                 │
│   ├─ specs/        ← 需求 + 接受标准            │
│   ├─ design.md     ← 技术方案                   │
│   └─ tasks.md      ← 实施清单                   │
└─────────────────────┬───────────────────────────┘
                     │
            ┌────────┴────────┐
            │ 设计中遇到根本决策？│
            └────────┬────────┘
                     │ Yes
                     ▼
┌─────────────────────────────────────────────────┐
│  路径 B 插入 — 写 ADR                           │
│  在 .dev/docs/decisions/ 新建 ADR-NNNN          │
│  design.md 引用这个 ADR                         │
└─────────────────────┬───────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ② 冷读 — 自检 propose 三件套                    │
│  · 间隔 ≥ 30 分钟回看 proposal/specs/design     │
│  · OK 才进入 apply；不 OK 改了再读              │
└─────────────────────┬───────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ③ Issue + 分支                                 │
│  · 开 GitHub Issue（关联 milestone）            │
│  · git checkout -b feature/<feature-name>       │
└─────────────────────┬───────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ④ APPLY — 按 tasks.md 实施                     │
│  /opsx:apply 或手动按 tasks 顺序写              │
│                                                 │
│  ⚠️ 实施中发现 design 不对？                     │
│      回到 propose 阶段更新 design / specs，     │
│      不要硬干（spec 与 code 同步演进）          │
└─────────────────────┬───────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ⑤ PR                                           │
│  · push → 开 PR（描述链到 change 文件夹）       │
│  · 冷读 self-review                             │
│  · CI 通过 → merge                              │
└─────────────────────┬───────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│  ⑥ ARCHIVE — 归档                               │
│  /opsx:archive                                  │
│  → openspec/changes/archive/YYYY-MM-DD-<name>/  │
│  · 更新 CHANGELOG.md [Unreleased] 段            │
│  · 关闭 GitHub Issue                            │
└─────────────────────────────────────────────────┘
```

milestone 全部 changes 归档后 → 写 retro → 打 tag → 发布。

### 2.4 路径 B：ADR 流程

```
触发：在产品设计层或执行层发现"这个决策影响多个 feature 或多个 milestone"
   │
   ▼
1. 在 .dev/docs/decisions/ 新建 ADR-NNNN-<kebab-title>.md
2. 写 Context / Decision / Alternatives / Consequences
3. Status: Proposed → 评估 → Accepted（或 Rejected）
4. 在引用方（milestone plan / change 的 design.md）链接这个 ADR
5. 后续相关工作引用同一个 ADR，避免重复决策
```

ADR 的详细规则见后文 §七 决策记录。

### 2.5 路径 C：直接 PR

适用于**无需"对齐意图"**的轻量变更：

```
1. （可选）开 GitHub Issue 描述问题
2. git checkout -b fix|chore|docs|refactor/<short-name>
3. 改 → PR → CI → merge
4. 用户可见的变更：更新 CHANGELOG [Unreleased]
```

**何时用 C 而非 A**：

| 场景 | 路径 |
|---|---|
| 修复 track 命令在某种 URL 下崩溃 | C（既有 feature 的 bug 修复） |
| 升级 tokio 到新版本 | C |
| 改 README、修 typo | C |
| 重构文件结构、不改任何外部行为 | C |
| 添加新 CLI 命令 | **A**（用户可见、需要 spec） |
| 添加新的 target（如 Cursor） | **A** |
| 改变 paam.yaml 字段语义 | **A**（影响用户） |

---

## 三、版本与里程碑

### 3.1 版本号规范

遵循 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)：`MAJOR.MINOR.PATCH`

| 阶段 | 版本范围 | 含义 |
|---|---|---|
| Phase 1 开发期 | `0.1.0` ~ `0.x.x` | API 不稳定，可破坏性变更 |
| Phase 1 完成 | `1.0.0` | 第一个稳定版本（M5 完成） |
| Phase 2/3 | `1.x.x` / `2.0.0` | 按 SemVer 规则推进 |
| Pre-release | `0.x.0-alpha.N` / `-beta.N` / `-rc.N` | 预发布版本 |

### 3.2 Milestone 与 Version 一一对应

| Milestone | Version | 范围（详见 `.dev/docs/milestones/M{N}-plan.md`） |
|---|---|---|
| M1 | `v0.1.0` | 技术原型：CLI 核心（track / install / sync / list） |
| M2 | `v0.2.0` | CLI 完整版（publish / update / uninstall / multi-target） |
| M3 | `v0.3.0` | 桌面 UI MVP（macOS） |
| M4 | `v0.4.0` | Windows 适配（UI + CLI） |
| M5 | `v1.0.0` | 完善与正式发布（doctor / diff / 安全审计 / 安装包签名） |
| Phase 2 起 | `v1.x.x` 起 | Prompts、MCP 等后续模块 |

完成一个 milestone → 打 tag → CI 自动构建产物并发布 → 启动下一个。**不强制时间间隔。**

### 3.3 Milestone 三段式

每个 milestone 走三个阶段：

#### Plan（启动时）

新建 `.dev/docs/milestones/M{N}-plan.md`，承担**本期 PRD + 项目计划**两种角色，记录：

- **本期功能需求**：P0 / P1 / P2 feature 清单（用户视角 + 接受标准）
- **出口标准（DoD）**：什么条件下算 milestone 完成？
- **依赖与风险**：依赖哪些前置 ADR / 外部决策？已知风险点？
- **不做的事**：明确划出本 milestone 范围外的内容

Plan 一旦写好，作为本 milestone 的"宪法"。中途要扩缩范围必须更新 plan 并写明原因。

#### Build（开发中）

- 每个 feature 走路径 A（OpenSpec change），详见 §2.3
- 决策点出现时走路径 B（写 ADR），详见 §2.4
- bug 修复 / 依赖升级等走路径 C（直接 PR），详见 §2.5
- 每次 PR merge 后更新 `CHANGELOG.md` 的 `[Unreleased]` 段
- 进度通过 GitHub Milestone 进度条 + `openspec/changes/` 目录可视化

#### Retro（收尾时）

新建 `.dev/docs/milestones/M{N}-retro.md`，记录：

- **实际范围 vs 计划**：哪些做了、哪些砍了、哪些超预期加进来
- **跑偏的地方与原因**：估时偏差最大的任务、踩到的坑
- **给下个 milestone 的建议**：流程改进、技术选型反思
- **打 tag**：`git tag -a v0.X.0 -m "..."` → push → 触发 CI 发布

Retro 完成 → 启动下一个 milestone 的 Plan。

---

## 四、分支策略

采用 **GitHub Flow（简化版）**：

```
main                          ← 始终可发布、永远绿
 ├─ feature/<short-name>      ← 新功能
 ├─ fix/<issue-id-or-name>    ← bug 修复
 ├─ docs/<topic>              ← 文档变更
 ├─ chore/<short-name>        ← 杂项（依赖升级、配置等）
 └─ refactor/<short-name>     ← 重构
```

**规则**：

- 所有分支从 `main` 出，PR 回到 `main`
- 分支名小写、用连字符分隔（kebab-case）
- 单个 PR 聚焦单个目标，避免"顺手改一堆"
- main 受保护：不允许直接 push，必须通过 PR
- main 不强制 squash，但鼓励 squash 复杂分支以保持 log 清晰
- 不使用 `develop` / `release` / `hotfix` 长生命周期分支

---

## 五、任务管理

### 5.1 工具

- **GitHub Issues**：所有任务、bug、决策的入口
- **GitHub Milestones**：对应 paam 的 M1-M5
- **GitHub Projects (v2)**：唯一看板视图

### 5.2 看板四列

```
Backlog  →  Ready  →  In Progress  →  Done
```

| 列 | 含义 |
|---|---|
| `Backlog` | 已识别但未规划的任务（PRODUCT / milestone plan 拆出来的、想法、bug 报告） |
| `Ready` | 已纳入当前 milestone、可随时动手的任务 |
| `In Progress` | 正在做的（建议同时 ≤ 2 个） |
| `Done` | 当前 milestone 内已完成（milestone 收尾后归档） |

### 5.3 优先级标签

沿用 milestone plan 中已建立的体系：

| 标签 | 含义 |
|---|---|
| `P0` | Must — 当前 milestone 必须完成 |
| `P1` | Should — 当前 milestone 计划完成，但可降级到下个 milestone |
| `P2` | Nice-to-have — 时间充裕时做 |

### 5.4 类型标签

| 标签 | 含义 |
|---|---|
| `type:feat` | 新功能 |
| `type:fix` | bug 修复 |
| `type:docs` | 文档 |
| `type:refactor` | 重构（不改变行为） |
| `type:test` | 测试相关 |
| `type:chore` | 杂项 |
| `type:decision` | 需要写 ADR 的决策点 |

### 5.5 范围标签（按 milestone plan 模块）

`scope:cli` / `scope:ui` / `scope:core` / `scope:track` / `scope:sync` / `scope:publish` / `scope:update` / `scope:auth` / `scope:ci` ...

---

## 六、Commit 与 PR 规范

### 6.1 Conventional Commits

格式：

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

**type**（与 issue 标签一致）：`feat` / `fix` / `docs` / `refactor` / `test` / `chore` / `build` / `ci` / `perf` / `style`

**scope**：模块名，与 issue 的 `scope:*` 标签对应

**示例**：

```
feat(track): support GitHub owner/repo shorthand
fix(sync): rollback symlinks when target write fails
docs(adr): accept ADR-0001 selecting Tauri as desktop stack
refactor(core): extract git operations into dedicated module
chore(deps): bump tokio to 1.40
```

**Breaking Change** 标注：

```
feat(config)!: rename `paam.yaml` to `paam.toml`

BREAKING CHANGE: configuration file format changed from YAML to TOML.
Migration: ...
```

### 6.2 PR 流程

即使是单人开发也走 PR 流程：

1. 从 `main` 切出特性分支
2. 开发 → push → 在 GitHub 上开 PR
3. **冷读 Self-Review**：PR 开出后**至少间隔 30 分钟**再回来 review 自己的 diff（热写时漏掉的问题，冷读能发现）
4. CI 通过 + self-review 通过 → merge

### 6.3 PR 描述模板

仓库的 `.github/PULL_REQUEST_TEMPLATE.md` 是面向所有贡献者的简化版（动机 / 改动 / 测试 / 关联 / 自检）。

作者本人在自己的 PR 中**额外**遵循以下扩展自检（不写进对外模板，避免给贡献者造成压力）：

- [ ] PR 已开 ≥ 30 分钟后**冷读**过自己的 diff
- [ ] 更新 `CHANGELOG.md` 的 `[Unreleased]` 段
- [ ] 涉及决策点的已写或更新对应 ADR
- [ ] 没有引入未追踪的 `TODO` / `FIXME`

### 6.4 Merge 策略

- **简单 PR**：直接 merge（保留完整 commit 历史）
- **复杂 PR**（commit 杂乱）：squash merge
- 不使用 rebase merge（保留分支拓扑信息）

---

## 七、决策记录（ADR）

### 7.1 何时必须写 ADR

下列场景**强制**写 ADR：

- 技术栈选型（语言、框架、库）
- 数据格式定义（配置文件 schema、lock 文件结构、API 协议）
- 安全相关决策（凭据存储、签名机制、权限模型）
- 跨模块的接口契约
- 任何"未来想反悔会很贵"的选择
- 拒绝某个明显的方案（记录"为什么不"）

**判断标准**：如果 6 个月后的我看到代码，会问"为什么这样做"——就该写 ADR。

### 7.2 ADR 格式

模板见 `./decisions/template.md`，包含：

- 编号 + 标题
- Status（Proposed / Accepted / Rejected / Superseded by ADR-XXXX）
- Date
- Context（为什么需要做这个决策）
- Decision（决定了什么）
- Alternatives Considered（考虑过的其他方案）
- Consequences（正面 / 负面 / 中性后果）
- References（相关链接）

### 7.3 ADR 编号

- 4 位数字递增：`0001`、`0002`、...、`9999`
- 文件名：`.dev/docs/decisions/{NNNN}-{kebab-case-title}.md`
- 已 reject 的 ADR 也保留编号（不重用）
- 被新 ADR 覆盖时，旧 ADR 标记 `Superseded by ADR-XXXX`，不删除

### 7.4 ADR 索引

`./decisions/README.md` 维护所有 ADR 的列表与状态。

---

## 八、CI/CD 与质量保障

### 8.1 CI 工作流

GitHub Actions 配置：

| 触发 | 工作流 |
|---|---|
| PR open / push | `lint` → `test` → `build` (matrix: macOS / Windows / Linux) |
| push to main | 同上 + 更新 main 构建产物 |
| tag `v*` | `release`：构建发布产物 + 创建 GitHub Release + 上传 artifacts + 更新 CHANGELOG |

**Required checks**（保护 main）：lint、test、build（至少 macOS）

### 8.2 测试策略

| 层级 | 覆盖范围 | 第一期目标 |
|---|---|---|
| 单元测试 | 核心库（git 操作、配置解析、版本对比、冲突解决） | ≥ 70% |
| 集成测试 | CLI 命令端到端（用 fixture git 仓库做测试夹具） | 覆盖所有 P0 命令 |
| E2E | UI 关键路径 | M3 之后引入 |

### 8.3 代码风格

具体工具链取决于技术栈选型（见 ADR-0001），但通用原则：

- format / lint 工具配置 commit 到仓库
- pre-commit hook：format + lint + 单元测试，失败不许 commit
- CI 重复跑一遍这些检查（防止本地 hook 被绕过）

### 8.4 安全检查

- **secrets 扫描**：CI 跑 `gitleaks` 或同类工具
- **依赖审计**：CI 定期跑（`cargo audit`）
- **commit / tag 签名（M5 前非强制）**：v1.0.0 发布前用 GPG 或 SSH key 签名（`git tag -s` 或 `git config gpg.format ssh` + `git tag -s`）。M1-M4 阶段不强制
- **发布产物签名**：macOS 公证 + Windows 代码签名（M5 之前完成）

---

## 九、文档体系

### 9.1 文档分层与职责

paam 的文档分为**三层**，每层职责清晰、内容不重复：

| 层级 | 载体 | 关注内容 | 变化频率 |
|---|---|---|---|
| **产品宪章** | `PRODUCT.md` | 长期定位、愿景、Phase 路线、永久边界、非功能性基线 | 几乎不变 |
| **本期 PRD** | `.dev/docs/milestones/M{N}-plan.md` | 本期具体功能需求 + 出口标准 + 风险 + 边界 | 每个 milestone 一份 |
| **决策档案** | `.dev/docs/decisions/ADR-NNNN.md` | 跨期共享的技术 / 产品决策 | 决策时增加一篇 |

**关键约定**：

- **`PRODUCT.md`** 是产品宪章，不是传统意义的 PRD；不要在这里塞具体功能表
- **`M{N}-plan.md`** 兼任"本期 PRD"与"项目计划"——单人开发场景下两者合并最自然
- **OpenSpec change** (`openspec/changes/<feature>/`) 是执行层，把 milestone PRD 中的单个 feature 落地成代码

### 9.2 仓库结构

仓库分为**对外**与**对内**两层，加上**工具固定路径**：

```
# 根目录 — 对外（用户、贡献者可见）
README.md                    # 项目入口（5 分钟读完）
PRODUCT.md                   # 产品宪章（定位、愿景、永久边界）
CHANGELOG.md                 # 版本变更（基于 Keep a Changelog）
LICENSE                      # 开源协议
CONTRIBUTING.md              # 贡献指南（M2 之后补）
SECURITY.md                  # 安全策略（M2 之后补）
docs/                        # 用户文档（教程、CLI 参考、API 文档等，M2 之后逐步补充）
.github/                     # PR / Issue 模板、CI 配置

# 工具固定路径（OpenSpec / Claude Code 强制要求顶层）
openspec/                    # OpenSpec 执行层 changes
├── changes/                 # 进行中的 changes
│   ├── <feature>/           # 单个 feature
│   │   ├── proposal.md
│   │   ├── design.md
│   │   ├── specs/
│   │   └── tasks.md
│   └── archive/             # 归档的 changes
│       └── YYYY-MM-DD-<feature>/
└── specs/                   # 累积的 capability 规范
.claude/                     # Claude Code 项目配置（OpenSpec 自动生成）
├── commands/opsx/           # /opsx:propose / apply / archive / explore
└── skills/openspec-*/       # OpenSpec 工作流 SKILLs

# .dev/ — 对内（作者个人工作空间）
.dev/docs/
├── PROCESS.md               # 本文档（个人开发流程）
├── architecture.md          # 架构总览（M1 之后写）
├── decisions/               # ADR 目录
│   ├── README.md            # ADR 索引
│   ├── template.md          # ADR 模板
│   └── NNNN-*.md
├── milestones/              # 里程碑 plan / retro（本期 PRD）
│   ├── M1-plan.md
│   ├── M1-retro.md
│   └── ...
└── archived/                # 设计期快照、废弃文档
    └── PRD-v0.1-design-snapshot.md
```

**几个关键说明**：

- **`.dev/`**：作者个人开发工作空间，记录流程、决策、里程碑。仍 commit 到仓库（保留版本历史 + AI 协作上下文），但**不对外强制**，外部贡献者无需阅读或遵循
- **`openspec/`**：OpenSpec 工具固定使用顶层路径（不可配置）。语义上属"执行层工作产物"，对外可见但实际服务于 AI + 作者的协作
- **`.claude/`**：Claude Code 项目配置，由 `openspec init` 自动生成。包含 slash commands 和 SKILLs。提交到仓库以便不同设备/会话保持一致工作流

### 9.3 CHANGELOG 维护

遵循 [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)：

- 顶部维护 `[Unreleased]` 段，每个 PR merge 时追加条目
- 发布时把 `[Unreleased]` 重命名为 `[X.Y.Z] - YYYY-MM-DD`，新建空的 `[Unreleased]`
- 分类：`Added` / `Changed` / `Deprecated` / `Removed` / `Fixed` / `Security`

### 9.4 代码文档

- 公开 API：必须有 doc comment（rustdoc / TSDoc 等）
- 私有逻辑：复杂时才注释，注释解释 *why*，不解释 *what*
- 默认不写多段注释，能用清晰命名表达就别加注释

---

## 十、风险信号与异常应对

不做定期 check-in，但出现以下信号时**立即停下来反思**：

| 信号 | 应对 |
|---|---|
| Milestone 进度条 5+ 天没动 | 评估是卡技术问题还是失去焦点；前者寻求帮助/换方案，后者调整范围 |
| 同一类问题反复出现 | 抽象出公共问题，可能是设计缺陷；考虑写 ADR 调整设计 |
| 某任务实际耗时是估时的 3 倍以上 | 评估 milestone 是否需要砍范围、推迟到下个版本 |
| 出现重大技术发现（要不要换方案） | 立即写 ADR，决策后再继续 |
| CI 持续红 | main 必须永远绿，立即修，不许累积 |
| 测试覆盖率连续下降 | 补测试欠债，否则技术债务会爆雷 |

---

## 十一、补充约定

### 11.1 不做的事

- ❌ 不设固定 sprint / 周迭代
- ❌ 不做周回顾、双周回顾（只做 milestone retro）
- ❌ 不直接 push 到 main
- ❌ 不在没有 issue 的情况下开 PR（小修小补除外，单独标 `chore`）
- ❌ 不在没写 ADR 的情况下做技术栈级别的决策
- ❌ 不发布带未通过 CI 的版本

### 11.2 鼓励的事

- ✅ 鼓励小 PR、勤 merge
- ✅ 鼓励先开 draft PR 异步思考
- ✅ 鼓励用 issue 记录想法（哪怕暂时不做）
- ✅ 鼓励在 PR 描述里坦诚写"已知问题 / TODO"
- ✅ 鼓励 retro 写得"诚实而非美化"

---

## 附录 A：常用命令速查

```bash
# 开始一个新 feature
git checkout main && git pull
git checkout -b feature/my-feature

# 提交
git commit -m "feat(scope): description"

# 推送 + 开 PR
git push -u origin feature/my-feature
gh pr create

# 完成 milestone 打 tag
git tag -s v0.1.0 -m "Release v0.1.0: M1 prototype"
git push origin v0.1.0
```

## 附录 B：相关文档链接

- [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
- [GitHub Flow](https://docs.github.com/en/get-started/using-github/github-flow)
- [Architecture Decision Records (ADR)](https://adr.github.io/)

---

**变更记录**：

| 版本 | 日期 | 变更 |
|---|---|---|
| 0.1 | 2026-04-25 | 初始版本，确立版本驱动 + AI 辅助 + 单人开源的流程 |
| 0.2 | 2026-04-26 | 新增 §二 工作流模型（两层职责 + 三条路径 + OpenSpec 整合）；调整 §三 Milestone 三段式以纳入 OpenSpec；后续章节顺次重新编号 |
