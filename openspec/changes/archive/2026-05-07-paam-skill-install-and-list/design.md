## Context

paam 现状：
- 配置层（`~/.paam/config.json`）、订阅层（`paam track`）、扫描层（`paam track skills`）已稳定
- `Asset` trait + `Skill` 类型已定型；`discover::skills_in` 给出可装的 skill 列表
- 远程 git 操作走系统子进程；paam-core 不依赖 git2-rs
- ADR-0007 §6 草案钦定 type-agnostic CLI（`paam install <name>`），但实施 ② 后重新评估，发现 M2 跨类型同名时 type-agnostic 反而繁琐

本 change 要做的事：
1. 落地 install/list 业务流，把 skill 从 source 仓 copy 到 local-repo 工作集，写 metadata，auto-commit
2. **修订 ADR-0007 §6**：CLI 命名空间从 type-agnostic 改为混合策略
3. 顺手做 uninstall（M1-plan §3.2 标 P1）—— install 反向操作零成本
4. 提前落地 `head_commit` / `subtree_hash` git helper，为 M2 update 铺路

约束：
- ADR-0007 §3 钦定 local-repo 物理布局：`~/.paam/local-repo/<type>/<asset>/`，每个资产带 `.metadata.json`
- ADR-0007 §5 钦定 metadata schema 字段：`name / type / origin{kind,repo,subpath,commit,tree_hash} / installed_at / targets[] / version`
- ADR-0007 §7（修订版）钦定动作下放到模块函数；本 change 沿用此模式

## Goals / Non-Goals

**Goals:**
- 让用户能 `paam skill install <name>`，在 `~/.paam/local-repo/skills/<name>/` 落地资产
- 让用户能 `paam list` 看全部已装、`paam skill list` 看仅 skill
- 修订 ADR-0007 §6 与 M1-plan §三，让纸面与代码命令名一致
- 顺手做 uninstall（与 ②/④ 风格一致）
- 给 M2 `paam update` 铺好 `head_commit` / `subtree_hash` helper

**Non-Goals:**
- 不实现 enable / disable / pin / info（M2）
- 不实现 sync 到 target（④）
- 不实现 update / publish / search（M2）
- 不实现 prompt / mcp 类型（M2 / M3）
- 不结构化解析 metadata 的 `targets[]` / `version`（前者 ④ 写入，后者 M2 引入）
- 不实现跨类型 / 跨 source 的复杂查询语法（M1 简洁优先）

## Decisions

### 1. CLI 命名空间混合策略（修订 ADR-0007 §6）

**选择：**

| 操作语义 | 命名风格 | 命令例 |
|---|---|---|
| 仓库级 / 跨类型 / 全量概览 | **type-agnostic** | `paam track <url>` / `paam track list` / `paam track skills <alias>` / `paam list` / `paam sync` |
| 资产级 CRUD | **type-prefix** | `paam skill install <name>` / `paam skill list` / `paam skill uninstall <name>` |

未来 M2/M3 加新资产类型遵循同 pattern：
- `paam prompt install <name>` / `paam prompt list` / `paam prompt uninstall`
- `paam mcp install <name>` / `paam mcp list` / `paam mcp uninstall`
- `paam track prompts <alias>` / `paam track mcps <alias>` 与 `paam track skills <alias>` 平行

**Why：**
- ADR §6 原理由 ① 「资产类型由文件 marker 识别」对 paam 内部成立，但**对用户输入 name 时不成立**：M2 起跨类型同名（一个 `name: foo` 的 skill + 一个 `name: foo` 的 prompt 在不同 source 里）会让 `paam install foo` 强制 `--type=` 消歧
- type-prefix 让范围在命令位置就明确，跨类型同名不再是歧义
- 平行扩展自然：M2 加 `paam prompt <verb>` 与 `paam skill <verb>` 平行，零修改既有命令
- 跨类型 / 仓库级操作（track / sync / 全量 list）保持 type-agnostic：保留 ADR §6 理由 ② 「一仓多类型」的优雅
- CLI Tab 补全友好：`paam <Tab>` 看到 track / skill / prompt / mcp / sync / list / 等顶层入口；`paam skill <Tab>` 看到 install / list / uninstall

**Alternatives considered:**
- 纯 type-prefix（`paam skill install` + `paam skill list-all`）：拒绝。`paam list` 全量概览的语义是用户高频需求；强行 type-prefix 会让 "我装了什么"这种问题需要扫所有 type
- 保持 ADR §6 type-agnostic：拒绝。M2 跨类型同名会触发 `--type=` 必填，反而繁琐
- `kubectl <noun> <verb>` 全 type-prefix：拒绝。paam 不像 k8s 资源类型有几十种，type 数量小（M3 仅 3 种），混合策略足够

**契约固化：** ADR-0007 §6 加 "Superseded" 标注 + 末尾追加「修订（2026-04-29）」段；M1-plan §三 3.2 / 3.4 命令名按新空间更新。

### 2. local-repo 物理布局（沿用 ADR-0007 §3）

```
~/.paam/local-repo/
├── .git/                    ← paam 自管的 git 仓
├── .gitignore               ← 占位（M1 暂空）
└── skills/                  ← M1 唯一类型；M2 加 prompts/，M3 加 mcp/
    └── <skill-name>/
        ├── .metadata.json   ← provenance（每资产一份）
        ├── SKILL.md         ← 来自 source 仓的拷贝
        └── 其他 skill 文件...
```

**Why（沿用 ADR §3）：**
- skills/ / prompts/ / mcp/ 分组让"列所有 skill"等操作只需扫一个目录
- 每资产一个 `.metadata.json`：git diff 在 install 时只新增一个文件，diff 易读；M2 paam update 修改 metadata 时也只动一个文件
- 不引入全局 metadata 索引：避免"加锁 / 一致性"问题

**契约固化：** local-repo 路径硬编码为 `~/.paam/local-repo/`（同 ADR §1）；后续 change 必须通过 `paam-core::local_repo` 模块访问。

### 3. local-repo 的 git 身份独立

**选择：** 首次 `local_repo::ensure_initialized` 时，硬编码：

```bash
git -C <local-repo> init
git -C <local-repo> config user.email "paam@local"
git -C <local-repo> config user.name "paam"
```

**Why：**
- 与用户 `~/.gitconfig` 解耦：避免 paam 自动 commit 把用户工作时间的真实身份留在 paam 工作集里（隐私 / 噪音）
- 用户在 `git log` 里看到 `paam <paam@local>` 一眼就知道是 paam 的自动操作
- 两端分离：用户对 source 仓的 commit 用真实身份；paam 对 local-repo 的 commit 用 paam 身份

**Risk：** 用户日后想"接管"local-repo 用真实身份 commit → 简单 `git -C ... commit --author=...` 即可，本契约不阻拦。

### 4. commit message 中文模板

```
安装 <name>，来自 <alias>@<commit[:7]>
重新安装 <name>，来自 <alias>@<commit[:7]>
卸载 <name>
```

未来 ④ sync 时（不在本 change 范围）建议：
```
同步 <name> -> <agent>:<path>
```

**Why：**
- 与本仓内部文档语言一致；用户 dogfood 时 `git log` 看着舒服
- `<commit[:7]>` 取 source 仓 HEAD commit 前 7 位，与 git 习惯对齐
- "重新安装"与"安装"区分，让 `git log --oneline` 一眼看出 history 中哪些是 force 重装

**Why 不用英文：** ADR 与 M1-plan 都是中文，dogfood 一致性优先。M2 若引入团队协作 commit 规范再评估。

### 5. install 流程与错误恢复

**正常流程：**

```
1. local_repo::ensure_initialized()      ← 首次 init local-repo + git config
2. resolve_skill(name, from)              ← 从 sources discover 找
3. 检查 ~/.paam/local-repo/skills/<name>/ 是否已存在
   - 存在 + !force → Error::AlreadyInstalled
   - 存在 + force  → rm -rf 既有目录（重装路径）
4. cp -r <source>/<rel_path>/ → ~/.paam/local-repo/skills/<name>/
5. 取 source 仓的 head_commit 与 subtree_hash
6. 写 .metadata.json（含 origin.commit / origin.tree_hash / installed_at / targets:[]）
7. local_repo::commit("安装 <name>，来自 <alias>@<sha>") 或 "重新安装 ..."
```

**错误恢复：**
- 1-2 失败：未触碰文件系统；直接返回错误
- 3-4 失败：如果创建了部分目录 → `rm -rf` 清理；config / metadata 未写入
- 5 失败（git rev-parse 异常）：清理已 cp 出的目录；不写 metadata
- 6 失败：清理已 cp 出的目录
- 7 失败：local-repo 进入 staged-but-uncommitted 状态；下次 install 时 `add -A` + commit 会一起带上（local-repo 不直接对外，可接受这种暂态）

**Alternative considered:** 整个 install 走"先 cp 到临时目录再 mv"原子化 → 拒绝。复杂度增加；M1 dogfood 阶段 git 状态重要性 ≫ 文件系统中间状态可见性。

### 6. cp 实现：std::fs 递归

**选择：** 用 `std::fs::create_dir_all` + 递归 `std::fs::copy` 自实现，不引入 `walkdir` / `fs_extra`。跳过 `file_name == ".git"` 的目录（保险）；不跟随 symlink（与 discover 一致）。

**Why：**
- skill 目录通常文件少（<10），自写 30 行可读
- 减依赖（与之前 swap-git-transport-to-cli / discover 自写递归思路一致）
- 显式跳过 `.git` 防御性：source 仓的根 .git 不应该出现在 skill 子目录里，但 SKILL.md 可能在嵌套结构里，万一用户把 skill 放在仓根或子模块边缘，跳过 .git 让 install 不会污染 local-repo

### 7. resolve_skill 消歧策略

**选择：** 扫所有 sources（按 `config::list_sources`）+ 在每个 source 内 `discover::skills_in`，按 frontmatter `name` 匹配。

| 匹配数量 | 是否指定 `--from` | 行为 |
|---|---|---|
| 0 | 任意 | `Error::SkillNotFound` |
| 1 | 任意 | 直接返回那一个 |
| N（N≥2，跨 source） | 否 | `Error::AmbiguousSkill { candidates: [alias_a, alias_b, ...] }` |
| N（N≥2，跨 source） | `--from <alias>` | filter 到 alias，若仍 0 → SkillNotFound；1 → 返回；N → 同 source 内 N>1，AmbiguousSkill（M1 限制） |

**Why：**
- 跨 source 同名是常见场景（公司仓 + 个人仓都有 `code-review`）；reject + 候选列表 + `--from` 是符合直觉的最简交互
- 同 source 内同名（discover 出 N≥2 个相同 `name`）是 source 维护者的问题，paam 不强制让用户先 untrack；M1 显示 warning 让用户去 source 修
- `--path <subpath>` 二级消歧推到 M2（design.md Open Questions）

### 8. metadata.json schema（M1 落地版）

```json
{
  "name": "pdf-review",
  "type": "skill",
  "origin": {
    "kind": "tracked",
    "repo": "github.com/foo/bar",
    "subpath": "tools/pdf-review",
    "commit": "abc123def456...",
    "tree_hash": "789xyz..."
  },
  "installed_at": "2026-04-29T10:30:00Z",
  "targets": [],
  "version": "1.0"
}
```

**字段约定：**
- `name` / `type`：与 Skill 一致
- `origin.kind`：M1 仅 `tracked`；`authored` / `adopted` 留 M2+
- `origin.repo`：source 在 paam 中的 alias（不是 git URL）；`paam list` 时显示这个
- `origin.subpath`：在 source 仓里的相对路径
- `origin.commit`：source HEAD commit 全长
- `origin.tree_hash`：subpath 对应 subtree 的 git tree hash
- `installed_at`：ISO-8601 UTC
- `targets`：M1 始终 `[]`；④ sync 时填
- `version`：M1 占位 `"1.0"`；M2 引入 SKILL.md frontmatter 的 version 字段时再决策映射

**Why：** 完全沿用 ADR-0007 §5 草案；本 change 把 schema 落地为 Rust 类型并实现读写。

### 9. `paam list` vs `paam skill list` 关系

| 命令 | 输出 | 列 |
|---|---|---|
| `paam list` | 所有已装资产（M1 仅 skill） | NAME / TYPE / SOURCE / INSTALLED_AT |
| `paam skill list` | 仅已装 skill | NAME / SOURCE / INSTALLED_AT（无 TYPE 列） |

**Why：**
- M1 阶段两者输出几乎相同；TYPE 列预留兼容 M2/M3，避免破坏性变更
- `paam skill list` 不重复显示 TYPE（既然命令已在子树指定）
- SOURCE 显示 `origin.repo`（即 source alias）

### 10. head_commit / subtree_hash 提前落地

**选择：** 在 `paam-core::git` 加：

```rust
pub fn head_commit(repo: &Path) -> Result<String>;
pub fn subtree_hash(repo: &Path, subpath: &str) -> Result<String>;
```

实现：复用 `git::run_capture(args, cwd)` —— 不存在则现加（之前 design 中提及但未实现）。

**Why：**
- M2 `paam update` 必备：对比 source 仓新旧 `tree_hash` 决定是否需要重装
- 本 change 写 metadata 已经需要这两个值；与 M2 同款 helper
- 成本低（每个 ~5 行）；先落地避免 M2 重新设计签名

**契约固化：** `git::run` / `run_capture` 是后续所有 change 的统一 git 调用入口；任何业务模块禁止直接 `Command::new("git")`，必须走这两个 helper。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| install 流程多步，中间失败时清理逻辑分散在多个分支 | 测试覆盖每个失败点；引入 `try_install_inner` + 错误时统一清理；commit 失败容忍（local-repo 不对外） |
| 跨 source 同名 skill 用户找不到 `--from` 怎么办 | 错误消息明确列候选 alias，并打印示例命令 `paam skill install foo --from <alias>` |
| 同 source 内同名 skill discover 不报错（按设计），但 install 时报 AmbiguousSkill 让用户困惑 | 错误消息提示"同 source 内同名是 source 维护者问题，请联系 source 作者" |
| 用户在 local-repo 手工 commit（"接管" git 仓） | M1 不阻拦也不文档化；M2 评估 |
| `paam skill list` 与 `paam list` 输出 90% 重复，让 M1 用户疑惑两者区别 | help 文本明确说明：`list` = 全量，`skill list` = 仅 skill；M2 起自然分化 |
| `cp -r` 自实现错误处理（如 file mode 在 macOS 与 Linux 差异） | M1 仅 macOS；保守复制内容 + 默认 mode 即可，不显式保留 mode bits |
| ADR §6 修订让旧 commit history 中的 PRD 引用失效 | 修订段保留原文；用户读到旧文档时能从修订段定位到当前形态 |

## Migration Plan

**用户数据：** 不涉及（首次跑 `paam skill install` 时初始化 local-repo；无既有数据迁移）。

**ADR 修订：**
- §6 加 "⚠ Superseded by below" 标注，原文保留
- 末尾追加「修订（2026-04-29，由 paam-skill-install-and-list 落地）」段
- 顶部 metadata 加 `Last-Reviewed: 2026-04-29`

**M1-plan 修订：**
- §三 3.2 / 3.4 命令名按新空间更新
- §七 build log 追加本 change 完成条目（apply 阶段）

**回滚：** 单 commit revert 即可；不影响 sources / config（local-repo 是新增目录，删除不破坏其他状态）。

## Open Questions

1. **同 source 内同名 skill 的 `--path <subpath>` 二级消歧**：M1 不做（reject + warning），M2 评估；如果 dogfood 阶段频繁遇到再加。
2. **`paam list` 是否支持 `--type=skill` flag**：M1 不加；M2 评估若用户呼声高再加（虽然有 `paam skill list`，但 flag 形式更适合 scripting）。
3. **metadata schema `version` 字段的语义**：M1 固定 `"1.0"`；M2 引入 SKILL.md frontmatter 的 `version` 字段时再决定（是 paam metadata schema version？还是 skill 自身 version？两者语义不同，需要 split 字段）。
4. **local-repo 是否暴露给用户作为 git 仓直接操作**：M1 文档不鼓励；M2 评估"用 git log 看 paam install 历史"是否要文档化为正式用户路径。
