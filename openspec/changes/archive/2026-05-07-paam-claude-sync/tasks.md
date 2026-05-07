## 1. paths 模块加 target dir helper

- [x] 1.1 在 `crates/paam-core/src/paths.rs` 加常量 `pub const ENV_PAAM_CLAUDE_TARGET_DIR: &str = "PAAM_CLAUDE_TARGET_DIR";`
- [x] 1.2 实现 `pub fn claude_skills_target_dir() -> Result<PathBuf>`：优先读环境变量；否则 `BaseDirs::new().home_dir().join(".claude").join("skills")`；BaseDirs 失败 → `Error::HomeNotFound`
- [x] 1.3 单测：环境变量未设时返回默认；设置后返回环境变量值；空字符串视为未设置（与 `paam_home` 同语义）

## 2. 错误模型扩展

- [x] 2.1 在 `crates/paam-core/src/error.rs` 新增 `Error::SyncIo { skill: String, target: PathBuf, message: String }`，文案：`"同步 \`{skill}\` 到 {target} 时 IO 错误：{message}"`

## 3. sync 模块（核心）

- [x] 3.1 新增 `crates/paam-core/src/sync/mod.rs`，在 `lib.rs` 加 `pub mod sync;`
- [x] 3.2 定义 `pub struct SyncReport { pub synced: Vec<String>, pub already_ok: Vec<String>, pub conflicts: Vec<Conflict>, pub forced: Vec<String> }` + `impl Default`
- [x] 3.3 定义 `pub struct Conflict { pub skill_name: String, pub target_path: PathBuf, pub reason: String }`
- [x] 3.4 内部枚举 `enum TargetState { Absent, AlreadyOk, PaamLinkBroken, ForeignContent { is_symlink: bool } }`
- [x] 3.5 实现私有 `fn classify(target_path: &Path, expected: &Path, meta_targets: &[Target]) -> Result<TargetState>`：
  - target 不存在 → Absent
  - target 是 symlink：`canonicalize` 与 expected 比较；相等 → AlreadyOk；不相等但 metadata.targets[] 命中此 path 或 canonicalize 落在 local-repo 内 → PaamLinkBroken；否则 → ForeignContent { is_symlink: true }
  - target 是真实文件 / 目录 → ForeignContent { is_symlink: false }（先看 metadata.targets[] 是否命中：命中则视为 PaamLinkBroken 的怪异变体——M1 处理为 PaamLinkBroken，删除 + 重建 symlink）
- [x] 3.6 实现私有 `fn ensure_symlink(src: &Path, target: &Path) -> Result<()>`：用 `std::os::unix::fs::symlink`（`#[cfg(unix)]`）；非 unix → `Error::SyncIo` 拒绝；含 mkdir -p 父目录
- [x] 3.7 实现私有 `fn remove_symlink_if_paam(target: &Path) -> Result<()>`：仅当为 symlink 时 `fs::remove_file`；其他情况由调用方判断
- [x] 3.8 实现 `pub fn sync_all(root: &PaamRoot, force: bool) -> Result<SyncReport>`：
  - `local_repo::ensure_initialized(root)?`
  - target_root = `paths::claude_skills_target_dir()?`
  - 调 `metadata::list_skills(root)?` 取所有已装 skill
  - 对每个 skill：
    1. expected = `metadata::skill_dir(root, name)`（local-repo 中的目录路径，需为绝对路径）
    2. target_path = target_root.join(name)
    3. 读 skill 的 metadata.targets[]
    4. classify(target_path, expected, &targets)
    5. 按 design.md 决策 3 表格分派；行动后更新 SyncReport 与 metadata.targets[]
  - 真有变更（synced.len + forced.len > 0）→ 调 `local_repo::commit` 一次：
    - 若总变更 1 → message `同步 X -> claude-code:<short_path>`
    - 若总变更 ≥2 → message `同步 N 个 skill 到 claude-code`
  - 全部 already_ok / conflicts → 不 commit
  - 返回 SyncReport
- [x] 3.9 实现 `pub fn unsync_one(root: &PaamRoot, name: &str) -> Result<()>`：
  - `local_repo::ensure_initialized(root)?`
  - 计算 target_path = `claude_skills_target_dir()?.join(name)`
  - 读该 skill 的 metadata（用 `metadata::find_skill`）—— 不存在 → 静默返回 Ok
  - 若 target_path 存在且是 symlink → 删除（`fs::remove_file`）
  - metadata.targets[] 清空 → write_for
  - `local_repo::commit(root, &format!("解除同步 {}", name))?`（无变更时静默 skip）
- [x] 3.10 实现 `pub fn unsync_all(root: &PaamRoot) -> Result<()>`：遍历 `metadata::list_skills`，对每个调 unsync_one 内部逻辑（不重复 commit），最后一次 commit `解除所有同步`
- [x] 3.11 单测（用 tempdir + PAAM_HOME + PAAM_CLAUDE_TARGET_DIR 环境变量沙盒）：
  - 干净状态首次 sync_all：N 个 skill → N 条 symlink；metadata.targets[] 写入；commit 含 message
  - 第二次 sync_all 全 already_ok：无 commit
  - 冲突 force=false：跳过加 conflicts；其它 skill 仍 sync
  - 冲突 force=true：覆盖加 forced
  - 旧 paam symlink 漂移：识别为 PaamLinkBroken 重建
  - target 父目录不存在：mkdir -p 后建 symlink 成功
  - unsync_one：删 symlink 清 targets[] commit
  - unsync_all：删多个 symlink 一个 commit
  - target 不存在的 unsync 静默成功
- [x] 3.12 测试 helper：在 `test_support` 加 `pub fn with_env_target_dir(path: &Path, f: impl FnOnce())`：set_var → 执行闭包 → remove_var；用于隔离测试

## 4. metadata 模块小改

- [x] 4.1 暴露 `pub use Target;`（已 derive Serialize/Deserialize）—— 实际 ③ 时已 pub，确认仍可外部构造
- [x] 4.2 不加新公开函数；sync 模块直接 read_for + 改 targets + write_for

## 5. install 模块更新（智能耦合）

- [x] 5.1 在 `install::uninstall_skill` 顶部、`local_repo::ensure_initialized` 之前加 `crate::sync::unsync_one(root, name)?`
  - 注意：unsync_one 内部已 ensure_initialized，所以 install 不需要重复调（但留着 ensure_initialized 也无害）
  - 顺序：unsync_one（清 target + 写 targets[] + 自己的 commit "解除同步"）→ rm -rf target_dir → commit "卸载"
  - **问题**：会产生两个 commit（"解除同步 X" + "卸载 X"），与 design.md 决策 4 "单 commit" 不符。修复方案：
    - 让 unsync_one 接受 `commit: bool` 参数（内部 helper 模式：核心 fn 不 commit；公共 API `unsync_one` 带 commit；install 调内部 fn 不 commit）
    - 重构：拆 `unsync_one_no_commit(root, name) -> Result<bool>`（返回是否真有变更）+ `unsync_one(root, name)` = 调内部 + 自己 commit
    - install::uninstall_skill 调 `unsync_one_no_commit` → rm -rf → 单 commit "卸载 X"
- [x] 5.2 落实上述 sync 模块拆分：
  - sync::unsync_one_no_commit(root, name) -> Result<bool>（是否有变更）
  - sync::unsync_one(root, name) = unsync_one_no_commit + 条件 commit
- [x] 5.3 install::uninstall_skill 修改后单测：验证 target symlink 在 uninstall 后被清理；commit 仍是单条 "卸载 X"
- [x] 5.4 ③ 已有的 uninstall 测试不变（不 sync 的场景）；新加 scenario：先 install + sync + uninstall，验证 target 消失

## 6. CLI 命令树扩展

- [x] 6.1 在 `crates/paam-cli/src/main.rs` 加 `Cmd::Sync(SyncArgs)` 与 `Cmd::Unsync(UnsyncArgs)`
- [x] 6.2 `struct SyncArgs { #[arg(long)] force: bool }`
- [x] 6.3 `struct UnsyncArgs { name: Option<String>, #[arg(long)] all: bool }`：业务层校验 name 与 all 互斥（也可用 clap `conflicts_with`）
- [x] 6.4 在 dispatcher 加分派：
  - `Cmd::Sync(args)` → `handle_sync(&root, args.force)`
  - `Cmd::Unsync(args)` → `handle_unsync(&root, args.name.as_deref(), args.all)`
- [x] 6.5 实现 `fn handle_sync(root, force) -> Result<(), Error>`：
  - 调 `sync::sync_all(root, force)`
  - 渲染 SyncReport：分组列表（"已同步:" / "已正确:" / "冲突:" / "已强制覆盖:"）；每组按 skill 名输出
  - 至少有一项 conflict 时给出引导："使用 `paam sync --force` 覆盖；或手动整理 ~/.claude/skills/"
- [x] 6.6 实现 `fn handle_unsync(root, name, all) -> Result<(), Error>`：
  - 互斥校验：name + all 同时给 → `Error::InvalidUsage`
  - 都没给 → `Error::InvalidUsage`（提示 `paam unsync <name>` 或 `paam unsync --all`）
  - 仅 name → `sync::unsync_one(root, name)`；打印 `已解除同步 {name}`
  - 仅 all → `sync::unsync_all(root)`；打印 `已解除所有同步`

## 7. 端到端验证（自动）

- [x] 7.1 `cargo build --workspace` 通过
- [x] 7.2 `cargo test --workspace` 全部通过（含新增 sync / paths / install 修改的单测）
- [x] 7.3 `cargo clippy --workspace --all-targets -- -D warnings` 无告警
- [x] 7.4 `cargo fmt --all -- --check` 通过
- [x] 7.5 `cargo tree -p paam-core | grep -E 'git2|libgit2|libssh2'` 仍无输出（确认 transport 契约未破坏）

## 8. 端到端验证（手动 dogfood）

- [x] 8.1 沙盒（PAAM_HOME + PAAM_CLAUDE_TARGET_DIR 双注入）：
  - 构造 1 个 source（含 1 个 SKILL.md 的本地 git 仓 + 注册到 paam.config）
  - `paam skill install <name>` → `paam sync` → 验证 target 下出现 symlink + metadata.targets[] 正确
- [x] 8.2 `paam sync` 第二次：验证 already_ok 输出；local-repo 无新 commit
- [x] 8.3 在 target 下手动 `mkdir foo` 占位（同 skill 名）；`paam sync` 验证 conflicts 跳过 + warning；`paam sync --force` 验证 forced 覆盖
- [x] 8.4 `paam unsync <name>` 验证 target symlink 删除 + targets[] 清空 + commit "解除同步 X"
- [x] 8.5 `paam unsync --all` 验证多个 skill 一次性清理
- [x] 8.6 `paam skill install <name>` → `paam sync` → `paam skill uninstall <name>` 验证 target symlink 自动消失，且仅一个 commit "卸载 X"
- [x] 8.7 验证 target 不存在时 `paam unsync <name>` 静默成功
- [x] 8.8 验证 `paam unsync foo --all` 与 `paam unsync`（无参）触发 InvalidUsage

## 9. 文档与日志

- [x] 9.1 在 `CHANGELOG.md` Unreleased 追加：「add: paam sync / paam unsync；`paam skill uninstall` 自动清理 target symlink；新增 sync 模块、target 路径解析（PAAM_CLAUDE_TARGET_DIR 可覆盖）。M1 端到端剧本（track → install → sync → list）闭合。」
- [x] 9.2 在 `.dev/docs/milestones/M1-plan.md` §七 Build 阶段进度日志追加一条（"YYYY-MM-DD：完成 paam-claude-sync"），简述本 change 的核心交付与 M1 闭合
- [x] 9.3 OpenSpec archive：所有任务 ✅ 后用 `/opsx:archive` 归档（流程提醒）
