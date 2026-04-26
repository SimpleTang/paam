# ADR-0007: 数据架构与 Phase 2/3 扩展策略

- **Status**: Accepted
- **Date**: 2026-04-26
- **Deciders**: @simpletang1994
- **Tags**: data-format | architecture

## Context

paam 演进路径：Phase 1 (Skills) → Phase 2 (Prompts) → Phase 3 (MCP)。

如果 Phase 1 数据架构不预留扩展点，Phase 2/3 引入新资产类型时会带破坏性变更（用户配置文件、目录结构、命令体系都会破坏）。

本 ADR 综合决策以下几个紧密相关的设计层面：

1. 工作目录与物理布局
2. 配置文件 schema
3. 资产元数据 schema
4. CLI 命令命名空间
5. 核心库内部资产抽象

## Decision

### 1. 工作目录

**`~/.paam/`** —— 跨平台一致的 dotfile 风格。

| 平台 | 路径 |
|---|---|
| macOS | `~/.paam/` |
| Linux | `~/.paam/` |
| Windows | `%USERPROFILE%\.paam\` |

放弃 macOS 标准的 `~/Library/Application Support/paam/`，换取跨平台路径一致性（用户在 README / 文档中可以一句话描述）。

### 2. 物理布局：三层资产流转模型

```
远程 git 仓
   │
   │ paam track <url>
   ▼
~/.paam/source/<repo>/                 ← 远程镜像（只读视角）
   │   · paam clone 进来，保留 .git
   │   · 仓内任意结构（不强加类型）
   │   · paam 通过 marker 文件扫描发现资产
   │
   │ paam install <asset>
   ▼
~/.paam/local-repo/<type>/<asset>/     ← 已选资产工作集
   │   · git 仓（auto-init，无 remote）
   │   · paam 主动按类型分组
   │   · 每个资产带 .metadata.json
   │   · paam 自动 commit（每次操作）
   │
   │ paam sync
   ▼
~/.claude/skills/<asset>  (target)     ← agent 真实读
   ↑ symlink ↑
```

**操作映射**：

| 操作 | 流转方向 | 实现 |
|---|---|---|
| `paam track` | 远程 → source | `git clone` |
| `paam untrack` | 删除 source 条目 | `rm -rf` source/<repo> |
| `paam install` | source → local-repo | `cp -r` + auto git commit in local-repo |
| `paam uninstall` | 删除 local-repo 条目 | `rm -rf` + auto git commit |
| `paam sync` | local-repo → target | 创建 symlink |
| `paam update` (M2) | 远程 → source → local-repo | git fetch + 重新 install 受影响资产 |
| `paam publish` (M2) | local-repo → 远程 | 临时 clone 远程 + 拷贝 + git push |

### 3. `~/.paam/` 顶层结构

```
~/.paam/
├── config.json                ← 全局配置（policy 层）
│
├── local-repo/                ← paam 主动组织（按类型分组）
│   ├── .git/
│   ├── mcp/                   (Phase 3 预留)
│   ├── prompts/               (Phase 2 预留)
│   └── skills/
│       └── <name>/
│           ├── SKILL.md
│           ├── (附属文件...)
│           └── .metadata.json  ← 每个资产带自己的元数据
│
└── source/                    ← 镜像远程（不强加类型）
    └── <repo>/                ← 一个 source 仓可能同时含多种资产
        ├── .git/
        ├── skills/            ← 仓内任意结构
        ├── prompts/
        └── (其他文件)
```

**对称性原则**：

- `source/` = 远程视角（不强加类型，因为远程仓的作者决定结构）
- `local-repo/` = paam 视角（按类型分组，paam 主动组织）
- 这种"非对称布局"是有意识的设计，反映两层语义本质不同

**为什么 source 不分类型**：一个 source 仓可能同时含 skills + prompts + mcp，paam 不应在 source 层强加结构。

**为什么 local-repo 分类型**：paam 主动决定布局；按类型分组让"列出所有 skill"等操作只需扫一个目录。

### 4. config.json schema 草案（M1 版本）

```json
{
  "version": 1,
  "sources": [
    {
      "alias": "team-skills",
      "url": "git@gitlab.com:my-team/skills.git",
      "ref": "main",
      "auth": "ssh"
    }
  ],
  "targets": [
    {
      "agent": "claude-code",
      "enabled": true,
      "path": "~/.claude/skills"
    }
  ],
  "sync": {
    "mode": "symlink"
  }
}
```

**Phase 2/3 演进路径**：

- Phase 2：在 `targets` 中加入 `prompts` agent（具体 path 由 prompt 工具生态决定）
- Phase 3：加入 `mcp` agent，path 指向 mcp config 文件
- `sync.mode` 可能扩展为 per-type 配置（M3 视情况）

### 5. 资产 .metadata.json schema 草案

每个 local-repo 中的资产带一个 `.metadata.json`。

**tracked 来源**：

```json
{
  "name": "pdf-review",
  "type": "skill",
  "origin": {
    "kind": "tracked",
    "repo": "team-skills",
    "subpath": "tools/skills/pdf-review",
    "asset_type": "skill",
    "commit": "abc123def456...",
    "tree_hash": "789xyz..."
  },
  "installed_at": "2026-04-26T10:30:00Z",
  "targets": [
    {
      "agent": "claude-code",
      "path": "/Users/me/.claude/skills/pdf-review",
      "mode": "symlink",
      "synced_at": "2026-04-26T10:30:05Z"
    }
  ],
  "version": "1.2.0"
}
```

**authored 来源**（M2+ paam new 创建的本地原创）：

```json
{
  "name": "my-new-skill",
  "type": "skill",
  "origin": { "kind": "authored" },
  "created_at": "2026-05-10T08:00:00Z",
  "targets": [...]
}
```

**`origin.kind` 三种**：

- `tracked` — 从远程仓 install
- `authored` — paam new 创建（M2+ 实现）
- `adopted` — 从外部目录 import 进 local-repo（M2+ 实现）

### 6. CLI 命令命名空间

**采用 type-agnostic 命令**：

```bash
paam track <url>            # 不带类型前缀
paam install <name>         # 不带类型前缀，类型由文件 marker 识别
paam sync                   # 全资产类型同步
paam list                   # 全资产类型列表
```

**不预留** `paam <type> <verb>` 形式（如 `paam skills track`）。理由：

- 资产类型由文件 marker 识别（`SKILL.md` / `PROMPT.md` / `mcp.json`），命令层无需指定
- 一个仓可能同时含多类型，type-agnostic 命令更优雅
- Phase 2/3 加新资产类型时无需新增命令命名空间

**例外**：明确 type-scoped 操作通过 flag 实现：

```bash
paam list --type=skill           # 仅列 skill
paam install <name> --type=skill # 强制指定类型（解决歧义）
```

### 7. 核心库资产抽象

`paam-core` 中的资产抽象（M1 仅实现 Skill）：

```rust
pub enum AssetType {
    Skill,
    Prompt,    // M2 实现
    Mcp,       // M3 实现
}

pub trait Asset {
    fn name(&self) -> &str;
    fn asset_type(&self) -> AssetType;
    fn marker_file(&self) -> &'static str;  // SKILL.md | PROMPT.md | mcp.json
    fn discover_in(repo_root: &Path) -> Vec<Self> where Self: Sized;
    fn install_to(&self, dest: &Path) -> Result<()>;
    // ...
}
```

Phase 2/3 加新类型 = 在 `AssetType` 枚举里新增 + 实现 `Asset` trait，**不动其他代码**。

## Alternatives Considered

### Option A: 不预留任何扩展点（M1 完全聚焦 Skills）

- **Pros**: M1 实现最快
- **Cons**: Phase 2/3 必然破坏配置文件 / 目录结构，需要写迁移工具
- **Verdict**: ❌ 拒绝

### Option B: 完全预留（Phase 1 就实现 Prompt/MCP 占位接口）

- **Pros**: 演进零成本
- **Cons**: 基于尚未实现的功能预设接口，YAGNI 风险
- **Verdict**: ❌ 拒绝

### Option C: 折中预留 ✅

预留低成本的扩展点：

| 预留点 | 实现 |
|---|---|
| 物理布局 | local-repo 立刻三类型分组，source 不分类型 |
| 配置 schema | 数组形式 `sources` / `targets`，加新条目即可 |
| CLI 命令 | type-agnostic 设计，不需要扩展命名空间 |
| 核心库抽象 | `AssetType` enum + `Asset` trait，加类型只需扩展枚举 |

- **Verdict**: ✅ 接受

## Consequences

### Positive

- Phase 2/3 引入新资产类型**不需要破坏配置文件**
- 用户不需要做 schema migration
- 核心库扩展边界清晰
- M2 引入 publish / update 时只需扩展现有结构

### Negative

- M1 实现要写一些"为未来准备"的代码（trait 抽象、enum）
- 物理布局非对称（source 不分类型 vs local-repo 分类型），新协作者需要文档解释
- 放弃 macOS 标准目录约定（部分高级用户可能有意见）

### Neutral / Trade-offs

- 资产 marker 文件（SKILL.md 等）作为类型识别 anchor 是关键约定，需明确文档化
- `.metadata.json` 字段 schema 在 v1.0 之前可能微调；用 `version` 字段标识

## Implementation Notes

**M1 实现顺序建议**（在 paam-core 中）：

1. `config` 模块（config.json 读写）
2. `git` 模块（git2-rs 封装）
3. `source` 模块（track / 扫描发现）
4. `local_repo` 模块（install / 自动 commit）
5. `sync` 模块（symlink + 冲突检测）
6. `metadata` 模块（.metadata.json 读写）
7. `asset` 模块（trait + Skill 实现）

**M1 不实现**：

- `Prompt` / `Mcp` 的具体实现（仅 enum 占位）
- `authored` / `adopted` 来源类型（仅 schema 预留）
- `update` / `publish` 流程（M2）

**marker 文件约定**（M1 仅 skill）：

| 资产类型 | marker | 识别方式 |
|---|---|---|
| Skill | `SKILL.md` | 任何含此文件的目录视为一个 skill |
| Prompt (M2) | TBD | 待 ADR-0009 决定 |
| MCP (M3) | TBD | 待 ADR-00xx 决定 |

## References

- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §6（数据与配置）、§9 待决策 #7
- PRODUCT.md §2 长期愿景与 Phase 规划
- 关联 ADR：[ADR-0001](./0001-tech-stack-choice.md)、[ADR-0002](./0002-shared-core-strategy.md)
- [Anthropic Agent Skills Spec](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview)
- [agentskills.io](https://agentskills.io)

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
| 2026-04-26 | Accepted | 经详细物理布局、schema、命令空间讨论后接受；扩展为综合数据架构 ADR |
