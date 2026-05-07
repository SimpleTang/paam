## Why

前三个 change 让 paam 能"看到"仓里有什么 skill，但还不能"装"。本 change 落地 install / list 流程：把已发现的 Skill 从 source 仓 copy 到 paam 的本地工作集（`~/.paam/local-repo/skills/<name>/`，一个 paam 自管的 git 仓）、写入 `.metadata.json` 记录 provenance、auto-commit 留下变更轨迹；并提供 `paam list` 与 `paam skill list` 让用户看见"我装了哪些"。

完成本 change 后 ④ paam-claude-sync 才能从 local-repo 同步到 `~/.claude/skills/`。

同时本 change **修订 ADR-0007 §6 的 CLI 命名空间策略**：从原 type-agnostic（`paam install <name>`）改为**混合策略**——资产级 CRUD 用 `paam <type> <verb>`（type-prefix），仓库 / 同步 / 全量概览保持 type-agnostic。修订理由：M2 引入 prompts 后跨类型同名（不同类型同名）会让 type-agnostic 反而繁琐，type-prefix 让"我要装哪类资产"在命令位置就明确，平行扩展自然。

> 本 change 取代了 M1-plan 原来的 ③（"paam-install-and-list"），命名调整为 `paam-skill-install-and-list` 以与新 CLI 命名空间一致。

## What Changes

**CLI 命令树（混合命名空间策略）：**
- 新增 `paam skill install <name> [--from <alias>] [--force]`
- 新增 `paam skill list`（仅列已装 skill）
- 新增 `paam skill uninstall <name>`（M1-plan §3.2 标 P1，顺手做）
- 新增 `paam list`（type-agnostic 全量已装资产，含 TYPE 列）
- 保留：`paam track <ssh-url>` / `paam track list` / `paam track skills <alias>` 不动
- 不引入 `paam skill info`（M1-plan §3.4 P1 推迟到 M2）
- 不引入 `--type=skill` flag（混合策略下用 `paam skill list` 即可）

**paam-core 模块新增：**
- `paam-core::local_repo`：管 `~/.paam/local-repo/` 的物理结构与 git 状态（init / 身份 / auto-commit）
- `paam-core::metadata`：管 `.metadata.json` 读写、聚合查询
- `paam-core::install`：业务编排（resolve / install / uninstall），按 ADR-0007 §7 修订版"动作下放到模块函数"
- `paam-core::git` 加 helper：`head_commit(repo)` 与 `subtree_hash(repo, subpath)`（M2 update 必备）

**错误模型：**
- 新增 `Error::SkillNotFound { name }` / `Error::AmbiguousSkill { name, candidates }` / `Error::AlreadyInstalled { name }` / `Error::NotInstalled { name }`

**修订文档：**
- `.dev/docs/decisions/0007-phase-extension-design.md` §6 加 "Superseded" 标注 + 末尾追加修订段
- `.dev/docs/milestones/M1-plan.md` §三 3.2 / 3.4 命令名按新空间更新

**明确不做：**
- 不实现 enable / disable / pin / info（M2）
- 不实现 update / publish / search（M2）
- 不实现 prompt / mcp 类型（M2 / M3）
- 不实现跨 type-prefix 的统一查询（`paam list` 已经是全量）
- 不实现 sync 到 target（推到 ④）
- 不实现 metadata `origin.kind ∈ { authored, adopted }`（M2+）
- 不实现 metadata `targets[]` 字段的写入（④ sync 时填）—— schema 中预留为空数组
- 不实现 metadata `version` 字段读取/校验（schema 中预留固定占位）

## Capabilities

### New Capabilities

- `installed-assets`：定义 local-repo 物理与 git 状态契约、`.metadata.json` schema、install / uninstall 行为、消歧策略、`paam skill install/list/uninstall` 与 `paam list` 命令

### Modified Capabilities

（无。`source-management` / `asset-management` 不动。）

## Impact

**代码：**
- `crates/paam-core/src/local_repo/mod.rs`：新增（`ensure_initialized` / `commit`）
- `crates/paam-core/src/metadata/mod.rs`：新增（`InstalledAsset` / `Origin` 结构 + 读写 + 聚合）
- `crates/paam-core/src/install/mod.rs`：新增（`resolve_skill` / `install_skill` / `uninstall_skill` 业务编排）
- `crates/paam-core/src/git/mod.rs`：加 `head_commit` / `subtree_hash` helper
- `crates/paam-core/src/error.rs`：+4 个变体
- `crates/paam-core/src/lib.rs`：注册 3 个新顶级模块
- `crates/paam-cli/src/main.rs`：加 `Cmd::Skill(SkillArgs)` 与 `Cmd::List`，对应 handler

**依赖：**
- 不新增依赖（cp 用 std::fs；git 操作用已有 `git::run` helper；JSON 用 serde_json）

**文件系统副作用：**
- 首次 `paam skill install ...` 会在 `~/.paam/local-repo/` 自动 `git init` 并设置 paam 身份的 `git config user.email / user.name`
- 每次 install / uninstall 会在 local-repo 产生一个 commit
- `.metadata.json` 写入每个已装资产目录

**对后续 change 的契约：**
- ④ paam-claude-sync：从 `~/.paam/local-repo/skills/*/` 读 metadata，按 metadata 中 `targets[]` 字段管理 symlink；本 change 已为 targets 预留字段空间
- M2 paam update：通过 `git::head_commit` + `git::subtree_hash` 对比新旧值决定是否重装（本 change 已落地这两个 helper）
- M2 paam publish：从 local-repo 推到远程 source；本 change 的 `.metadata.json origin` 字段提供回写依据
