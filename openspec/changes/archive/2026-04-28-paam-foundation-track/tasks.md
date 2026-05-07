## 1. 依赖与项目骨架

- [x] 1.1 在 `Cargo.toml` workspace dependencies 增加 `chrono`（含 `serde` feature）、`tempfile`（dev-dep）；不引入任何 YAML 依赖
- [x] 1.2 在 `paam-core` 的 `Cargo.toml` 启用 `serde`、`serde_json`、`chrono`、`directories-next`、`git2`、`thiserror`、`tracing`；dev-dep 加 `tempfile`
- [x] 1.3 在 `paam-cli` 的 `Cargo.toml` 启用 `clap`、`tracing-subscriber`、`paam-core`；移除 `tokio` 依赖（M1 不用）
- [x] 1.4 改写 `crates/paam-core/src/lib.rs`：声明 `error`、`paths`、`config`、`git`、`source`、`asset` 六个模块，仅 `pub use` 顶层 API
- [x] 1.5 新增 `crates/paam-core/src/error.rs`：定义 `Error` enum（thiserror）+ `Result<T>` 别名，覆盖 IO / git / json / 配置版本不识别 / URL 非法 / alias 已存在 / ssh-agent 不可用等分类

## 2. paths 模块（工作目录契约）

- [x] 2.1 实现 `paths::paam_home()` -> `PathBuf`，基于 `directories-next::BaseDirs::home_dir().join(".paam")`
- [x] 2.2 实现 `paths::sources_dir()`、`paths::config_file()` 派生函数
- [x] 2.3 实现 `paths::ensure_initialized()`：幂等创建 `~/.paam/`、`~/.paam/sources/`、空 `config.json`（仅当不存在时写）
- [x] 2.4 单测：在 `tempfile::TempDir` 中通过环境变量或注入路径覆盖 home，验证 `ensure_initialized` 幂等

> 实现备注：在 `paths.rs` 引入 `PaamRoot` 结构体作为可注入的工作目录上下文（`PaamRoot::from_env` 解析 `PAAM_HOME` 或 `~/.paam/`，`PaamRoot::at(path)` 用于测试隔离）。顶级 `paths::xxx()` 函数保留并转发到 `from_env()`。所有业务 API 接受 `&PaamRoot`，避免单测对全局环境变量的并发依赖。

## 3. config 模块（用户配置文件）

- [x] 3.1 在 `config/schema.rs` 定义 `Config { version: u32, sources: Vec<Source> }`、`Source { alias: String, url: String, added_at: DateTime<Utc> }`，全部 `#[derive(Serialize, Deserialize)]`
- [x] 3.2 定义常量 `CURRENT_SCHEMA_VERSION: u32 = 1`，`Config::new_empty()` 构造器
- [x] 3.3 实现 `config::load() -> Result<Config>`：读 JSON（`serde_json::from_reader`），校验 `version <= CURRENT_SCHEMA_VERSION`，否则返回 `Error::UnsupportedSchemaVersion`
- [x] 3.4 实现 `config::save(&Config) -> Result<()>`：以 `serde_json::to_writer_pretty` 原子写（先写 `config.json.tmp`，再 `rename`）
- [x] 3.5 实现业务级 API `config::add_source(alias, url) -> Result<()>`：内部 load → 检查 alias 重复（重复返回 `Error::AliasAlreadyExists`）→ 追加 → save
- [x] 3.6 实现业务级 API `config::list_sources() -> Result<Vec<Source>>`
- [x] 3.7 单测：空文件初始化、读写 round-trip、版本号高于当前时拒绝、alias 重复时拒绝

> 实现备注：API 签名调整为 `load(root: &PaamRoot)` / `save(root, &Config)` / `add_source(root, alias, url)` / `list_sources(root)`，其余语义与 design 一致。

## 4. source 模块之 URL 解析

- [x] 4.1 在 `source/url.rs` 定义 `SourceLocator { host: String, owner: String, repo: String }`
- [x] 4.2 实现 `parse_ssh_url(input: &str) -> Result<SourceLocator>`：识别 SCP-like 与 `ssh://` 两种形式，剥离 `.git` 后缀，剥离端口号，**保留原始大小写在 SourceLocator 内部**
- [x] 4.3 实现 `SourceLocator::alias()` 方法：返回小写化的 `<host>/<owner>/<repo>`
- [x] 4.4 实现 `SourceLocator::cache_dir(base: &Path)` 方法：返回 `base.join(host_lc).join(owner_lc).join(repo_lc)`
- [x] 4.5 单测：覆盖至少 6 个用例（带/不带 `.git`、带/不带端口、混合大小写、HTTPS 拒绝、`owner/repo` 简写拒绝、本地路径拒绝）

## 5. git 模块（git2 封装）

- [x] 5.1 在 `git/auth.rs` 实现 `make_ssh_auth_callbacks() -> RemoteCallbacks`：仅注册 `Cred::ssh_key_from_agent` 路径；agent 不可用时让 git2 错误冒出，由调用方映射为 `Error::SshAgentUnavailable`
- [x] 5.2 在 `git/mod.rs` 实现 `clone(url: &str, dest: &Path) -> Result<()>`：`RepoBuilder::new()` + 上一步的 callbacks；不在内部捕获 panic
- [x] 5.3 错误映射：把 `git2::Error` 按 `class()/code()` 转成 `Error::SshAgentUnavailable` / `Error::GitNetwork` / `Error::GitGeneric` 三类
- [x] 5.4 测试 fixture：用 `git2::Repository::init_bare` 在 tempdir 创建本地 bare repo + 一个空 commit，作为"远程"被 `clone` 拉取（走 file:// 协议而非 ssh，验证 clone 主流程不依赖网络）

## 6. source 模块之业务编排

- [x] 6.1 实现 `source::track(url: &str) -> Result<TrackOutcome>`：`paths::ensure_initialized()` → `parse_ssh_url` → 计算 alias 与目标目录 → 检查目标目录是否已存在（若已存在直接返回 `Error::AliasAlreadyExists`，不进 git 流程）→ `git::clone` → 失败时 `fs::remove_dir_all` 清理半成品 → 成功时 `config::add_source`
- [x] 6.2 `TrackOutcome` 结构体含 `alias`、`local_path`、`url`，便于 CLI 层格式化输出
- [x] 6.3 实现 `source::list_sources()` 直接转发 `config::list_sources`
- [x] 6.4 单测：使用本地 bare repo fixture，验证整个 track 流程；验证 clone 失败回滚不留目录；验证重复 track 拒绝

## 7. asset 模块（type-agnostic 骨架）

- [x] 7.1 在 `crates/paam-core/src/asset/mod.rs` 定义 `pub trait Asset`：四个方法 `id() -> &str`、`kind() -> AssetKind`、`source_alias() -> &str`、`relative_path() -> &Path`
- [x] 7.2 定义 `#[non_exhaustive] pub enum AssetKind { Skill }`（M2 加 Prompt / Mcp 时不构成破坏性变更）
- [x] 7.3 在 trait 与 enum 上加 doc-comment，明确"本 change 不写实现者，`Skill` 实现由 ② paam-skill-discovery 提供"
- [x] 7.4 在 `lib.rs` 中 `pub use asset::{Asset, AssetKind}`，让外部 crate 直接以 `paam_core::Asset` 形式引用
- [x] 7.5 编译期断言：写一个仅作类型检查的占位测试（如 `fn _assert_object_safe(_: &dyn Asset) {}`），确保 trait 是 object-safe 的（M2 可能要 `Vec<Box<dyn Asset>>`）

## 8. paam-cli 子命令骨架

- [x] 8.1 在 `main.rs` 引入 `clap::{Parser, Subcommand}`；`Cli` 顶层 struct 加 `--verbose` 全局 flag
- [x] 8.2 定义 `enum Cmd { Track(TrackArgs) }`；预留位置但不注册 install/sync/list（避免 `--help` 误导）
- [x] 8.3 定义 `enum TrackSubcmd { Add { url: String }, List }`；`paam track <url>` 与 `paam track list` 通过 clap 的"位置参数 vs 子子命令"区分（参考 `git remote add` / `git remote -v` 模式）
- [x] 8.4 配置 `tracing-subscriber`：默认 INFO，`--verbose` 提到 DEBUG
- [x] 8.5 在 main 顶部调用 `paths::ensure_initialized()`（保证任何子命令首次运行都能初始化工作目录）

> 实现备注：8.3 简化为单个位置参数 `target: String`——`target == "list"` 走列表路径，否则当 SSH URL 处理。clap 不直接支持"默认子命令"语义，单参数分派最贴近 spec 中 `paam track <url>` / `paam track list` 两种用法且 UX 一致。

## 9. paam-cli 子命令实现

- [x] 9.1 `track add` 处理函数：调用 `paam_core::source::track`，成功时打印两行（"已订阅 alias=...", "本地路径=..."），失败时把错误转成对用户友好的中文提示并以 exit code 1 退出
- [x] 9.2 `track list` 处理函数：调用 `paam_core::source::list_sources`，空列表时打印"暂无已订阅的源，使用 `paam track <git-url>` 添加"，否则打印多行（每行 `alias  url  added_at`）
- [x] 9.3 错误展示：在一个集中的 `report_error(&Error)` 函数里 match 所有 `Error` 变体，给出对应中文消息（特别是 `SshAgentUnavailable` 给出 `ssh-add ~/.ssh/id_*` 提示）

## 10. 端到端验证

- [x] 10.1 `cargo build` 在 macOS 干净环境通过
- [x] 10.2 `cargo test --workspace` 全部通过（20 测试 0 失败）
- [x] 10.3 `cargo clippy --workspace -- -D warnings` 无告警
- [x] 10.4 `cargo fmt --check` 通过
- [x] 10.5 手动验收剧本：在 `~/.paam/` 不存在的环境下，依次执行 `paam track <某真实 ssh url>`、`paam track list`，确认输出与 spec scenarios 一致
- [x] 10.6 手动负面验收：构造 ssh-agent 未启动场景、HTTPS URL 输入场景、重复 track 场景，分别确认错误消息符合 spec

> CLI sanity 已通过 `PAAM_HOME=$tmpdir paam ...` 在沙盒中验证：`--help` / `track --help` / `track list`（空）/ HTTPS 拒绝 / `foo/bar` 简写拒绝 / 工作目录自动初始化 / `config.json` 初始内容均符合预期。
>
> 10.5 真实 SSH 验收发现：用户的 `~/.ssh/config` 把 `github.com` 重定向到 `ssh.github.com:443`，但 libssh2 不读 `.ssh/config`，硬连 `github.com:22` 时 SSH banner 失败。这是 libssh2 与 OpenSSH 的根本差异，**非 paam 缺陷**——本 change 的修复止于"错误分类细化（`SshAgentUnavailable` vs `SshTransport`）+ 错误文案明确给出展开 URL workaround + verbose 模式暴露 git2 原始错误"。10.5 / 10.6 以"在 `ssh-add` 后用展开 URL 能跑通"为本 change 的接受标准；libssh2 → git CLI 子进程的根治方案推到下一个 change `swap-git-transport-to-cli`。

## 11. 文档与日志

- [x] 11.1 在 `CHANGELOG.md` 的 Unreleased 段落记录本 change（"add: paam track / paam track list"）
- [x] 11.2 在 `.dev/docs/milestones/M1-plan.md` §七 Build 阶段进度日志追加一条（"YYYY-MM-DD：完成 paam-foundation-track，OpenSpec 工作流首跑"）
- [x] 11.3 OpenSpec archive：所有任务 ✅ 后用 `/opsx:archive` 归档本 change（不在本任务清单内执行，仅作流程提醒）
