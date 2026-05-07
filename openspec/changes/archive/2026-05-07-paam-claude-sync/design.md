## Context

paam 在 ③ paam-skill-install-and-list 之后已经能：
- `paam track` 订阅源、`paam track skills` 看仓内 skill
- `paam skill install` 把 skill 装到 `~/.paam/local-repo/skills/<name>/`
- `paam skill list` / `paam list` 看已装

但 Claude Code 实际读的是 `~/.claude/skills/<name>/`——paam 的 local-repo 还**对外不可见**。本 change 完成最后一公里：sync 把每个已装 skill symlink 到 target，让 Claude Code 立刻能用。

约束：
- ADR-0007 §2 钦定 sync = "local-repo → target 创建 symlink"
- ADR-0007 §5 metadata 中已有 `targets[]` 字段（③ 时预留为空数组）；本 change 正式写入
- M1-plan §3.3 钦定：M1 仅 Claude Code，target 路径硬编码 `~/.claude/skills/`，仅 symlink 模式
- M1-plan §四 明确：M1 不做 dry-run

## Goals / Non-Goals

**Goals:**
- 闭合 M1 端到端剧本：track → install → sync → 在 Claude Code 中可用
- 落地 sync / unsync 业务编排，写入 metadata.targets[]
- 让 sync 幂等且安全（不破坏非-paam 内容）
- 与 ③ install/uninstall 智能耦合：uninstall 自动清理 target symlink

**Non-Goals:**
- 不实现 `--mode copy` / `paam target detect` / 多 target / dry-run（均 M2）
- 不引入新依赖
- 不改 Claude Code 行为；paam 只负责把 symlink 摆好
- 不读 / 写 config.json 中的 target 字段（M1 不引入）

## Decisions

### 1. 默认 symlink 模式 + 不实现 --mode copy

**选择：** `paam sync` M1 仅做 symlink；`--mode copy` 推到 M2。

**Why：**
- ADR-0007 §2 钦定，M1-plan §3.3 钦定
- 单 target、单平台（macOS）下 symlink 是最佳：local-repo 改了 target 立刻生效，磁盘占用 0
- copy 的真实需求（某些 Agent 不识别 symlink、用户想在 target 改东西）M1 dogfood 暂未触发；M2 引入多 target 时再加

**契约固化：** `sync::sync_all` 函数签名 `(root: &PaamRoot, force: bool) -> Result<SyncReport>` 不带 mode 参数；M2 加 mode 时再扩签名。

### 2. target 路径硬编码 + 环境变量覆盖

**选择：**

```rust
pub const ENV_PAAM_CLAUDE_TARGET_DIR: &str = "PAAM_CLAUDE_TARGET_DIR";

pub fn claude_skills_target_dir() -> Result<PathBuf> {
    // 优先环境变量
    if let Ok(custom) = std::env::var(ENV_PAAM_CLAUDE_TARGET_DIR) {
        if !custom.is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    // 默认 ~/.claude/skills/
    let dirs = BaseDirs::new().ok_or(Error::HomeNotFound)?;
    Ok(dirs.home_dir().join(".claude").join("skills"))
}
```

**Why：**
- 与 `PAAM_HOME` 同套思路（① paam-foundation-track 落地的模式）
- M1 仅 Claude Code，硬编码合理；M2 多 target 时改为 config.json `targets:` 字段
- 测试用环境变量覆盖（避免污染真实 `~/.claude/`）

**契约固化：** target 路径**不**进 PaamRoot——它是 paam 输出方向，与工作目录语义不同。`claude_skills_target_dir()` 是顶层独立函数。

### 3. 冲突策略：默认跳过 + warning，--force 覆盖

target 路径 `~/.claude/skills/<name>` 已存在时四态分类与处理：

| 状态 | force=false | force=true |
|---|---|---|
| 是 symlink，canonicalize 后等于 local-repo/skills/\<name\> | 跳过 → already_ok | 跳过 → already_ok |
| 是 symlink，指向其他（含断链） | 重建 → synced（视为 paam 旧链路修复） | 重建 → synced |
| 是真实文件 / 目录（非 paam） | **跳过 → conflicts + warning** | **删除原内容 + symlink → forced** |
| 不存在 | 创建 → synced | 创建 → synced |

**Why：**
- 与 `paam skill install --force` 语义对齐：默认安全（不破坏用户内容），--force 表达"我知道在做什么"
- "一个冲突阻塞全局"是糟糕的 UX；其它 skill 应该继续 sync，冲突项独立报告
- conflicts 由 `SyncReport` 表达，不抛 Error（见决策 6）

**冲突探测的双重证据：**

1. **主路径（metadata.targets[]）**：read 该 skill 的 .metadata.json，若 `targets[]` 中存在某项的 `path` 等于 target_path → paam 管理
2. **备路径（fs canonicalize）**：`std::fs::canonicalize(target_path)` 后路径前缀是 `local-repo/skills/`（且对应 paam 自管的 local-repo 路径）→ paam 管理

任一命中视作 paam 管理；都不命中且为非-symlink → 真实冲突。

**Why 双重证据：**
- 主路径：精确，与 metadata 对齐
- 备路径：处理"metadata 缺 targets[] 但 fs 上确实是 paam symlink"的边角（如用户手动改过 metadata，或 sync 失败后状态不同步）

### 4. uninstall 智能清理 targets（跨模块依赖）

**选择：** `install::uninstall_skill` 在 `fs::remove_dir_all(local-repo skill 目录)` **之前**调用 `sync::unsync_one(root, name)`，再删目录，再 commit。

```
install::uninstall_skill(root, name):
    sync::unsync_one(root, name)?            ← 先清 target，避免 dangling
    local_repo::ensure_initialized(root)?
    let target_dir = metadata::skill_dir(...)
    if !target_dir.exists() → NotInstalled
    fs::remove_dir_all(&target_dir)
    local_repo::commit(root, "卸载 X")?       ← 单 commit 含 target 清理 + local-repo 删除
```

**Why：**
- 避免 dangling symlink：用户视角 uninstall 应该把所有痕迹清掉
- 单 commit 让 git log 简洁（"卸载 X"涵盖 target 清理 + local-repo 删除两组变更）
- 跨模块依赖方向：install → sync（**不是反向**）；sync 不感知 uninstall

**Risk：** install 模块测试需要更新（uninstall scenario 含 target 清理）；module 间依赖增加。但方向单向，无循环；可接受。

### 5. SyncReport 而非 Error 表达冲突

**选择：**

```rust
#[derive(Debug, Default)]
pub struct SyncReport {
    pub synced: Vec<String>,       // 创建或重建（symlink 之前不正确）
    pub already_ok: Vec<String>,   // 已正确指向，跳过
    pub conflicts: Vec<Conflict>,  // force=false 时遇到的非-paam 占用
    pub forced: Vec<String>,       // force=true 时被覆盖的非-paam 占用
}

#[derive(Debug)]
pub struct Conflict {
    pub skill_name: String,
    pub target_path: PathBuf,
    pub reason: String,        // "已存在非 paam 管理的目录" 等
}
```

**Why：**
- 冲突是预期内的状态分类（用户机器上有别的工具留下的目录），**不是失败**
- 用 Error 表达会让 `paam sync` 在第一个冲突时整体失败 —— 不可接受
- IO 错误（symlink 创建/删除失败）才用 `Error::SyncIo` 抛出
- CLI 层把 SyncReport 渲染成可读输出（含 conflict 列表 + 引导用户用 --force 或手动处理）

### 6. metadata.targets[] 写入语义：整体重写

**选择：** 每次 sync 该 skill 时 `set_targets(asset_dir, &[Target { agent: "claude-code", ... }])`（M1 始终 0 或 1 个元素）。

```rust
// sync 成功后
let targets = vec![Target {
    agent: "claude-code".to_string(),
    path: target_path.clone(),
    mode: "symlink".to_string(),
    synced_at: chrono::Utc::now(),
}];
// read_for + 修改 + write_for 整体覆盖
```

unsync 该 skill：`targets = vec![]` 整体重写（清空）。

**Why：**
- M1 单 target 下增量与重写无差别
- 实现最简：read + 改 targets 字段 + write
- M2 多 target 时再决策（按 agent 增量更新还是仍整体重写）

**契约固化：** sync 模块**不**为 metadata 加新公开 API；直接调 `metadata::read_for` + `write_for` 实现。

### 7. 同步幂等：无变更不写不 commit

**选择：**
- 全部 already_ok → 不动 metadata、不调 `local_repo::commit`（命令仅打印 SyncReport）
- 任一 synced / forced → 写对应 skill 的 metadata.targets[] + 调 `local_repo::commit`
- conflicts 不参与 commit（没有变更）

**Why：**
- 与 `local_repo::commit` 的"无 staged 变更不 commit"语义协同
- 用户跑两次 `paam sync` 不会留下两个一样的 commit
- `git log` 出现新 commit = 真发生了变更

### 8. 单 commit 合并多 skill 变更

**选择：** `paam sync` 一次最多产生一个 commit，message 模板：

| 场景 | message |
|---|---|
| 单 skill 同步 | `同步 X -> claude-code:<short_path>` |
| 多 skill 同步 | `同步 N 个 skill 到 claude-code` |
| `paam unsync <name>` | `解除同步 X` |
| `paam unsync --all` | `解除所有同步` |
| `paam skill uninstall X`（含 unsync） | `卸载 X`（沿用 ③ 模板，target 清理与 rm 在同一 commit） |

**Why：**
- 每 skill 一个 commit 在用户大量同步时会让 git log 爆炸
- 多 skill 的详细列表先不进 commit body（M2 视需要加）
- `<short_path>` 取 target_path 的最后两段（`.claude/skills/X`），避免 message 太长

### 9. 新 capability `target-sync`

**选择：** sync / unsync / target 路径契约 / 冲突探测 都在新 capability `target-sync` 中；`installed-assets` 仅在 uninstall scenario 加一条新行为。

**Why：**
- installed-assets 关心"我装了什么"（local-repo 视角）
- target-sync 关心"我装的有没有暴露到 target"（target 视角）
- 两个视角不同，未来 M2 多 target 时 target-sync 自然成长

### 10. 不引入 dry-run

**选择：** M1 不实现 `paam sync --dry-run`。

**Why：**
- M1-plan §四明确"M1 不做"
- 默认 force=false 已经是"安全模式"——不会破坏用户内容；用户可先 `paam sync` 看 SyncReport（含 conflicts 列表），再决定是否 `--force`
- 等于天然的"两阶段确认"流程，无需额外 flag

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| 用户在 paam 之外手动 rm `~/.claude/skills/<name>` 后 `mkdir <name>` 自有内容 → paam 下次 sync 视为冲突跳过 | M1 接受（user 显式毁约）；M2 评估 stale 检测启发式 |
| `--force` 误删用户重要内容 | 文档强调 `--force` 含义；CLI 层在 forced 列表时打印 "已覆盖：[...]" 让用户立刻看到 |
| target 父目录 `~/.claude/` 不存在或不可写 | sync 首次跑时 `mkdir -p` 创建；不可写 → `Error::SyncIo` 含路径与 io::Error 文本 |
| install → sync 跨模块依赖 | 单向（不循环）；sync 不调 install / metadata 写；模块图保持 DAG |
| 测试要双注入 PAAM_HOME + PAAM_CLAUDE_TARGET_DIR，env 全局会被并发测试相互污染 | 与 PAAM_HOME 同样：业务 API 不依赖全局 env，测试用 `PaamRoot::at(...)` 注入 home，但 target 仍走 env_var；新增测试 helper `with_target_dir(dir, |&path| { ... })` 在测试内 set_var + 跑 + remove_var；并发问题接受（cargo test 默认并行，但通过精心设计单测内不同 tempdir 避免冲突） |
| sync 中途 IO 错误（如 symlink 创建失败）让部分 skill synced 部分未处理 | 接受：sync 是"尽力而为"，部分成功也写部分 targets[] 与 partial commit；错误 skill 在错误信息中明确，用户重跑 sync 即可 |
| symlink 在跨文件系统时可能行为异常（如 macOS APFS / iCloud Drive 同步目录） | M1 ~/.claude/ 通常在用户家目录 APFS 卷；APFS 支持 symlink 无问题；iCloud 同步目录边角不在 M1 范围（用户责任） |
| Windows 上 symlink 需要 admin 权限或开发者模式 | M1 仅 macOS（M4 适配） |

## Migration Plan

不涉及用户数据迁移。

代码层：
1. `paths.rs` 加 `claude_skills_target_dir()`
2. `sync/mod.rs` 新增（`sync_all` / `unsync_one` / `unsync_all`）
3. `error.rs` 加 `SyncIo`
4. `install/mod.rs` 中 `uninstall_skill` 加一行 `sync::unsync_one(root, name)?`
5. `lib.rs` 注册 `pub mod sync;`
6. `paam-cli/main.rs` 加 `Cmd::Sync(SyncArgs)` / `Cmd::Unsync(UnsyncArgs)` 与 handler

回滚：单 commit revert；之前 sync 创建的 symlink 与 metadata.targets[] 不会自动清理，但仍可通过手动 `rm -rf ~/.claude/skills/` 或回到 ③ 版本（targets[] 字段在 ③ 已存在但始终为空，不破坏 schema）。

## Open Questions

1. **`paam sync` 输出格式**：SyncReport 渲染为表格还是分组列表？M1 倾向分组列表（"Synced:" / "Already OK:" / "Conflicts:" / "Forced:" 四段），简洁；M2 评估表格。
2. **多 skill commit message 是否含 skill 名列表**：当前 `同步 N 个 skill 到 claude-code`；可改为 `同步 N 个 skill 到 claude-code: a, b, c`（短）/ 写到 commit body（更结构化）。M1 选最简版，M2 评估。
3. **unsync 是否要 --force**：默认行为就是删 paam 自管的 symlink；不存在的 target 就 silent skip。无需 --force。
4. **target 不存在时的 `paam unsync`** 行为：当前设计 silently skip（不抛错）；与 `paam skill uninstall <未装>` 的语义不一致（后者会抛 NotInstalled）。M1 决策：unsync 不抛错（用户意图是"清理"，目标不存在视为已达成）。
