## 1. 文档修订（ADR + M1-plan）

- [x] 1.1 在 `.dev/docs/decisions/0007-phase-extension-design.md` §6 标题下加 "⚠ Superseded by 「修订（2026-04-29）」段" 标注
- [x] 1.2 在该文件末尾追加新「修订（2026-04-29，由 paam-skill-install-and-list 落地）」段：描述混合策略（资产 CRUD type-prefix；track / sync / 全量 list type-agnostic）；列举 M2/M3 平行扩展形态；阐述修订理由（M2 跨类型同名让 type-agnostic 反而繁琐）
- [x] 1.3 更新 ADR-0007 顶部 metadata `Last-Reviewed` 为 2026-04-29，并在 Status 行追加 §6 修订标注
- [x] 1.4 在 ADR-0007 末尾的修订表格中追加一行（Date: 2026-04-29 / Status: Revised §6 / Note: ...）
- [x] 1.5 修订 `.dev/docs/milestones/M1-plan.md` §三 3.2 表格：`paam install <skill>` → `paam skill install <name>`、`paam uninstall <skill>` → `paam skill uninstall <name>`、`paam enable/disable <skill>` → `paam skill enable/disable <name>`、`paam pin <skill> <ref>` → `paam skill pin <name> <ref>`
- [x] 1.6 修订 `.dev/docs/milestones/M1-plan.md` §三 3.4 表格：保留 `paam list` → P0 全量；新增一行"仅列已装 Skills" → `paam skill list` P0；将 `paam info <skill>` 改为 `paam skill info <name>`（仍 P1）；将 `paam search <kw>` 命令名保持（type-agnostic 跨类型搜索）

## 2. 错误模型扩展

- [x] 2.1 在 `crates/paam-core/src/error.rs` 新增 `Error::SkillNotFound { name: String }`，文案"未找到名为 `{name}` 的 skill。提示：使用 `paam track skills <alias>` 查看仓内可用 skill"
- [x] 2.2 新增 `Error::AmbiguousSkill { name: String, candidates: Vec<String> }`，文案"skill `{name}` 在多个 source 中存在：{候选列表}\n  请用 `paam skill install {name} --from <alias>` 显式指定"
- [x] 2.3 新增 `Error::AlreadyInstalled { name: String }`，文案"skill `{name}` 已安装。\n  使用 `--force` 重装，或先 `paam skill uninstall {name}`"
- [x] 2.4 新增 `Error::NotInstalled { name: String }`，文案"skill `{name}` 未安装"

## 3. git 模块加 helper

- [x] 3.1 在 `crates/paam-core/src/git/mod.rs` 提取私有 helper `pub(crate) fn run_capture(args: &[&str], cwd: Option<&Path>) -> Result<String>`：fork git，捕获 stdout（trim 末尾换行）；非 0 → `GitProcessFailure`
- [x] 3.2 重构 `run` 让其与 `run_capture` 共享公共部分（如 NotFound 映射）
- [x] 3.3 实现 `pub fn head_commit(repo: &Path) -> Result<String>`：`run_capture(&["rev-parse", "HEAD"], Some(repo))`
- [x] 3.4 实现 `pub fn subtree_hash(repo: &Path, subpath: &str) -> Result<String>`：`run_capture(&["rev-parse", &format!("HEAD:{}", subpath)], Some(repo))`；若 subpath 含前导 `./` 或 `/` 应在调用前 normalize（trim）
- [x] 3.5 单测：用 `test_support::make_fixture_repo` 构造一个有 commit 的 bare repo + 普通 worktree，验证 `head_commit` 返回 40 位 hex、`subtree_hash` 在子目录上返回不同的 hash

## 4. local_repo 模块

- [x] 4.1 新增 `crates/paam-core/src/local_repo/mod.rs`，在 `lib.rs` 加 `pub mod local_repo;`
- [x] 4.2 加常量 `pub const LOCAL_REPO_DIRNAME: &str = "local-repo";` 与 `pub fn local_repo_dir(root: &PaamRoot) -> PathBuf { root.home().join(LOCAL_REPO_DIRNAME) }`
- [x] 4.3 实现 `pub fn ensure_initialized(root: &PaamRoot) -> Result<()>`：
  - 若 `<root>/.paam/local-repo/.git/` 已存在 → 直接返回
  - 否则 `fs::create_dir_all` + `git init <local_repo_dir>` + `git config user.email "paam@local"` + `git config user.name "paam"`
  - 创建 `.gitignore` 占位（M1 内容为空，仅一行注释 `# paam local-repo .gitignore`）
- [x] 4.4 实现 `pub fn commit(root: &PaamRoot, message: &str) -> Result<()>`：
  - `git -C <local_repo> add -A`
  - 探测是否有 staged 变更：`git -C <local_repo> diff --staged --quiet`（exit 0 = 无变更，exit 1 = 有变更）
  - 无变更 → 静默返回 Ok
  - 有变更 → `git -C <local_repo> commit -m <message>`
- [x] 4.5 单测：用 tempdir + `PaamRoot::at` 注入；验证首次 init 创建 .git/、设置 config 正确；验证幂等不重置；验证 commit 流（先无变更 silently skip，再写文件后 commit 成功）

## 5. metadata 模块

- [x] 5.1 新增 `crates/paam-core/src/metadata/mod.rs`，在 `lib.rs` 加 `pub mod metadata;`
- [x] 5.2 定义 `pub struct InstalledAsset { name, asset_type: AssetType, origin: Origin, installed_at: DateTime<Utc>, targets: Vec<Target>, version: String }`，全部 derive Serialize/Deserialize
- [x] 5.3 定义 `pub struct Origin { kind: OriginKind, repo: String, subpath: PathBuf, commit: String, tree_hash: String }`
- [x] 5.4 定义 `#[non_exhaustive] pub enum OriginKind { Tracked }`（M2+ 加 Authored / Adopted）；serde 用小写形式（`#[serde(rename_all = "lowercase")]`）
- [x] 5.5 定义 `#[non_exhaustive] pub enum AssetType { Skill }`（与 asset::AssetKind 平行；serde lowercase）—— 命名注意与 `crate::asset::AssetKind` 区分（前者是 metadata 序列化形态，后者是 trait 枚举），可考虑用 `serde(into / from)` 互换或直接复用 AssetKind；本 change 选择**复用 AssetKind**，metadata schema 直接用 `AssetKind` 序列化为小写字符串
- [x] 5.6 定义 `pub struct Target { agent: String, path: PathBuf, mode: String, synced_at: DateTime<Utc> }`，M1 始终序列化为 `[]`，无构造逻辑
- [x] 5.7 实现 `pub const METADATA_FILENAME: &str = ".metadata.json";`
- [x] 5.8 实现 `pub fn write_for(asset_dir: &Path, meta: &InstalledAsset) -> Result<()>`：原子写（先 .tmp 再 rename）pretty JSON
- [x] 5.9 实现 `pub fn read_for(asset_dir: &Path) -> Result<Option<InstalledAsset>>`：文件不存在返回 Ok(None)；存在则 read + parse；解析失败返回 Err
- [x] 5.10 实现 `pub fn list_installed(root: &PaamRoot) -> Result<Vec<InstalledAsset>>`：扫 `local-repo/skills/*/`，对每个调 `read_for`；解析失败的资产 stderr `eprintln!("warning: ...")` + 跳过
- [x] 5.11 实现 `pub fn list_skills(root: &PaamRoot) -> Result<Vec<InstalledAsset>>`：list_installed + filter type=Skill
- [x] 5.12 实现 `pub fn find_skill(root: &PaamRoot, name: &str) -> Result<Option<InstalledAsset>>`：list_skills + find
- [x] 5.13 实现 `pub fn skill_dir(root: &PaamRoot, name: &str) -> PathBuf`：`local_repo_dir/skills/<name>`
- [x] 5.14 单测：write/read round-trip、list_installed 聚合多资产、解析失败的资产被跳过 + warning

## 6. install 模块

- [x] 6.1 新增 `crates/paam-core/src/install/mod.rs`，在 `lib.rs` 加 `pub mod install;`
- [x] 6.2 定义 `pub struct ResolvedSkill { pub skill: Skill, pub source_alias: String, pub source_local_path: PathBuf }`
- [x] 6.3 实现 `pub fn resolve_skill(root: &PaamRoot, name: &str, from: Option<&str>) -> Result<ResolvedSkill>`：
  - 调 `config::list_sources(&root)` 拿所有 alias
  - 调 `config::effective_scan_ignore(&root)` 拿 ignore 列表
  - 对每个 source 调 `discover::skills_in(&local_path, &alias, &ignore)`，filter `name` 匹配
  - `from` 给定时：filter 到该 alias 内的匹配；候选 0 → SkillNotFound；1 → Ok；N → AmbiguousSkill
  - `from` 未给定：跨所有 source 的全部匹配；0 → SkillNotFound；1 → Ok；N → AmbiguousSkill（candidates 为各 source alias 列表，去重）
- [x] 6.4 实现 `pub fn install_skill(root: &PaamRoot, resolved: &ResolvedSkill, force: bool) -> Result<InstalledAsset>`：
  - `local_repo::ensure_initialized(root)?`
  - target_dir = `metadata::skill_dir(root, &resolved.skill.id())`
  - target_dir 已存在：force=true → `fs::remove_dir_all(&target_dir)`；force=false → Err AlreadyInstalled
  - source_dir = `resolved.source_local_path.join(resolved.skill.relative_path())`
  - `copy_dir_excluding_git(&source_dir, &target_dir)?`
  - 取 `git::head_commit(&resolved.source_local_path)?` + `git::subtree_hash(&resolved.source_local_path, &subpath_str)?`
  - 构造 InstalledAsset，写 `.metadata.json` 到 target_dir
  - commit message 选 `安装 {name}，来自 {alias}@{commit前7}` 或 `重新安装 ...`
  - `local_repo::commit(root, &msg)?`
  - 返回 InstalledAsset
  - 任意失败时清理 target_dir（已部分写入），不写 metadata
- [x] 6.5 实现私有 helper `fn copy_dir_excluding_git(src: &Path, dst: &Path) -> Result<()>`：递归 std::fs；跳过 file_name == ".git"；不跟随 symlink
- [x] 6.6 实现 `pub fn uninstall_skill(root: &PaamRoot, name: &str) -> Result<()>`：
  - `local_repo::ensure_initialized(root)?`
  - target_dir = `metadata::skill_dir(root, name)`
  - 不存在 → Err NotInstalled
  - `fs::remove_dir_all(&target_dir)`
  - `local_repo::commit(root, &format!("卸载 {}", name))?`
- [x] 6.7 单测（用 tempdir + 手写 SKILL.md fixture，外加本地 `git init` + commit 让 source 有合法 HEAD）：
  - resolve_skill 0/1/N 三种情况（含 `--from` 路径）
  - install_skill：成功路径（验证 target_dir 存在、metadata 内容、commit 已创建）
  - install_skill：AlreadyInstalled、--force 重装路径
  - install_skill：cp 中途失败时清理（mock 一个失败点）
  - uninstall_skill：成功 / 不存在
- [x] 6.8 单测验证 metadata 内容正确：name / type=Skill / origin.repo=alias / origin.subpath / origin.commit（40 位 hex） / origin.tree_hash（40 位 hex） / installed_at（合理时间） / targets=[] / version="1.0"

## 7. CLI 命令树扩展

- [x] 7.1 在 `crates/paam-cli/src/main.rs` 加 `enum Cmd` 新分支：`Skill(SkillArgs)` 与 `List`
- [x] 7.2 定义 `struct SkillArgs { #[command(subcommand)] cmd: SkillCmd }`
- [x] 7.3 定义 `enum SkillCmd { Install { name: String, #[arg(long)] from: Option<String>, #[arg(long)] force: bool }, List, Uninstall { name: String } }`
- [x] 7.4 在 main 的 dispatcher 中加 `Cmd::Skill(args)` 分派到子 handler；`Cmd::List` 直接到全量列表 handler
- [x] 7.5 实现 `fn handle_skill_install(root, name, from, force) -> Result<(), Error>`：调 `install::resolve_skill` → `install::install_skill`；成功后打印 `已安装 {name}\n  来源={alias}\n  本地路径={path}`
- [x] 7.6 实现 `fn handle_skill_uninstall(root, name) -> Result<(), Error>`：调 `install::uninstall_skill`；成功后打印 `已卸载 {name}`
- [x] 7.7 实现 `fn handle_skill_list(root) -> Result<(), Error>`：调 `metadata::list_skills`；空列表友好提示；非空打印表格 NAME / SOURCE / INSTALLED_AT
- [x] 7.8 实现 `fn handle_list(root) -> Result<(), Error>`：调 `metadata::list_installed`；空列表友好提示；非空打印表格 NAME / TYPE / SOURCE / INSTALLED_AT
- [x] 7.9 实现 helper `fn print_installed_table(rows: &[InstalledAsset], with_type: bool)` —— 列宽自适应、`installed_at` 用 RFC3339

## 8. 端到端验证（自动）

- [x] 8.1 `cargo build --workspace` 通过
- [x] 8.2 `cargo test --workspace` 全部通过（含新增 local_repo / metadata / install / git helper 单测）
- [x] 8.3 `cargo clippy --workspace --all-targets -- -D warnings` 无告警
- [x] 8.4 `cargo fmt --all -- --check` 通过
- [x] 8.5 `cargo tree -p paam-core | grep -E 'git2|libgit2|libssh2'` 仍无输出（确认上一 change 的契约未破坏）

## 9. 端到端验证（手动 dogfood）

- [x] 9.1 沙盒中构造一个本地 git 仓含 1 个合法 SKILL.md，模拟 source；手动构造 config.json 注册该 source；执行 `paam skill install <name>`，验证：
  - target_dir 创建在 `~/.paam/local-repo/skills/<name>/`
  - `.metadata.json` 字段正确（commit / tree_hash 等）
  - local-repo 有一个新 commit，message 为"安装 ..."
- [x] 9.2 重复 9.1 流程后执行 `paam skill list` 与 `paam list`，验证表格输出与 spec 一致
- [x] 9.3 已装时再次 `paam skill install <name>` 验证 `Error::AlreadyInstalled`
- [x] 9.4 `paam skill install <name> --force` 验证重装成功 + commit message 为"重新安装 ..."
- [x] 9.5 `paam skill install <未发现>` 验证 `Error::SkillNotFound`
- [x] 9.6 构造两个 source 同名 skill，`paam skill install <name>` 验证 `Error::AmbiguousSkill`；用 `--from <alias>` 验证消歧
- [x] 9.7 `paam skill uninstall <name>` 验证目录删除 + commit message 为"卸载 ..."
- [x] 9.8 `paam skill uninstall <未装>` 验证 `Error::NotInstalled`

## 10. 文档与日志

- [x] 10.1 在 `CHANGELOG.md` Unreleased 追加：「add: paam skill install/list/uninstall + paam list；引入 local-repo 工作集（auto-init git，paam 身份）；metadata.json schema 落地（每资产一份）。修订 ADR-0007 §6 CLI 命名空间为混合策略（资产 CRUD type-prefix；仓库/同步/全量 type-agnostic）」
- [x] 10.2 在 `.dev/docs/milestones/M1-plan.md` §七 Build 阶段进度日志追加一条（"YYYY-MM-DD：完成 paam-skill-install-and-list..."），简述本 change 的核心交付与 ADR §6 修订
- [x] 10.3 OpenSpec archive：所有任务 ✅ 后用 `/opsx:archive` 归档（流程提醒）
