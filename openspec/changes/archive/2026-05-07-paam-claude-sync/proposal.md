## Why

M1 至此差最后一公里：用户能 `paam track` / `paam track skills` / `paam skill install/list/uninstall`，但 Claude Code **看不见**——已装的 skill 还在 `~/.paam/local-repo/skills/`，并未暴露到 `~/.claude/skills/`。本 change 完成 local-repo → target 的分发：`paam sync` 把每个已装 skill 用 symlink 暴露到 `~/.claude/skills/`，闭合 M1 端到端剧本（track → install → sync → list）。

同步要解决三件事：
1. **暴露**：symlink 一条对一条
2. **冲突识别**：~/.claude/skills/ 下可能已有用户自己的目录或别的工具留下的 symlink
3. **生命周期对齐**：`paam skill uninstall` 时清理 target 上残留的 symlink，避免 dangling

## What Changes

**CLI 命令树新增：**
- `paam sync [--force]`：把 local-repo/skills/* 全部 symlink 到 `~/.claude/skills/*`
  - `--force`：强制覆盖非-paam 管理的 target（默认跳过 + warning）
- `paam unsync <name>` / `paam unsync --all`：仅删 target symlink + 清 metadata.targets[]，不动 local-repo（与 sync 对偶）

**CLI 命令树修改：**
- `paam skill uninstall <name>`：删 local-repo 目录前自动调用 `sync::unsync_one(name)` 清理 target symlink（智能耦合，避免 dangling）

**paam-core 新模块：**
- `paam-core::sync`（顶级）：`sync_all` / `unsync_one` / `unsync_all`；`SyncReport` / `Conflict` 数据结构
- `paam-core::paths` 加 `claude_skills_target_dir()` + `PAAM_CLAUDE_TARGET_DIR` 环境变量覆盖

**冲突策略：**
- 默认跳过非-paam 管理的 target + warning，加入 `SyncReport.conflicts`
- `--force` 时覆盖（删除原内容 + 创建 symlink），加入 `SyncReport.forced`
- "是否 paam 管理"用双重证据：metadata.targets[] 命中 主路径 + canonicalize 落在 local-repo 内 备路径

**Error 模型：**
- 新增 `Error::SyncIo { skill, target, message }`：symlink IO 错误时携带上下文
- 冲突**不**视为错误（由 SyncReport 表达）

**明确不做：**
- `--mode copy`（M2，M1-plan §3.3 P1）
- `paam target detect`（M2 P2）
- 多 target（Cursor / Codex，M2）
- target 路径写入 config.json（M1 仅 hardcode + 环境变量）
- dry-run（M1-plan §四明确"M1 不做"）

## Capabilities

### New Capabilities

- `target-sync`：sync_all / unsync 行为、target 路径契约、冲突探测与处理策略、metadata.targets[] 写入语义、`paam sync` / `paam unsync` 命令

### Modified Capabilities

- `installed-assets`：在"paam skill uninstall 命令" requirement 中加新 scenario——uninstall 时自动清理已 sync 的 target symlink

## Impact

**代码：**
- `crates/paam-core/src/sync/mod.rs`：新增（`sync_all` / `unsync_one` / `unsync_all` + 内部 helpers）
- `crates/paam-core/src/paths.rs`：加 `claude_skills_target_dir()` + 常量
- `crates/paam-core/src/install/mod.rs`：`uninstall_skill` 调用 `sync::unsync_one`（跨模块依赖）
- `crates/paam-core/src/error.rs`：+`Error::SyncIo`
- `crates/paam-core/src/lib.rs`：注册 `pub mod sync;`
- `crates/paam-cli/src/main.rs`：+`Cmd::Sync(SyncArgs)` +`Cmd::Unsync(UnsyncArgs)` 与对应 handler
- `crates/paam-core/src/test_support.rs`：可能需要新 helper 注入 target dir（也可能直接 env::set_var 处理）

**依赖：**
- 不新增依赖

**文件系统副作用：**
- `paam sync` 在 `~/.claude/skills/` 下创建 / 重建 / 删除 symlink
- 默认 force=false 时**不**触碰非-paam 内容（用户安全网）
- `--force` 时会删除冲突的非-paam 文件 / 目录（用户明示同意）

**外部系统：**
- 新增运行期约束：默认 target 是 `~/.claude/skills/`；用户必须确保该目录或其父目录可写（首次 sync 时自动 `mkdir -p`）

**对后续 change 的契约：**
- M2 多 target：`metadata.targets[]` schema 可平行扩展（每个元素一个 agent）；本 change 的 `target-sync` capability 设计已为多 target 留口（每次 sync 整体重写该 skill 的 targets[]）
- M2 `--mode copy`：sync 模块的 `ensure_symlink` helper 旁可加 `ensure_copied`，业务编排层选择 mode
- M2 paam update：sync 不感知 update；update 走 install → 触发 sync 在下次 `paam sync` 时把更新的 local-repo 内容暴露（symlink 自动跟随）
