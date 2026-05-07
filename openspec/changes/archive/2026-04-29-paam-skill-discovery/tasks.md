## 1. 依赖与 ADR

- [x] 1.1 在 workspace `Cargo.toml` 的 `[workspace.dependencies]` 增加 `serde_yaml_ng = "0.10"`
- [x] 1.2 在 `crates/paam-core/Cargo.toml` 启用 `serde_yaml_ng = { workspace = true }`
- [x] 1.3 在 `.dev/docs/decisions/0007-phase-extension-design.md` §7 末尾追加修订段：标题 `### 修订（2026-04-29，由 paam-skill-discovery 落地）`，列出最终 trait 形状、关联常量、模块函数策略、修订理由，并在原 trait 草案处加 "Superseded by below" 标注
- [x] 1.4 在 ADR-0007 顶部 metadata 加 `Last-Reviewed: 2026-04-29`

## 2. 配置 schema 扩展（scan_ignore）

- [x] 2.1 在 `crates/paam-core/src/config/schema.rs` 给 `Config` struct 加字段 `#[serde(default, skip_serializing_if = "Option::is_none")] pub scan_ignore: Option<Vec<String>>`
- [x] 2.2 验证现有 config.json（仅含 `version` + `sources`）能正常 deserialize（`scan_ignore` → None）
- [x] 2.3 在 `crates/paam-core/src/config/mod.rs` 新增 `pub fn effective_scan_ignore(root: &PaamRoot) -> Result<Vec<String>>`：load → 取 `scan_ignore` → 无值或 None → 返回 `discover::DEFAULT_IGNORE.to_vec()`；有值（含空数组）→ 返回 cloned 用户值
- [x] 2.4 单测：缺省返回默认；用户列表替换默认；空数组返回空 Vec

## 3. SKILL.md frontmatter 解析

- [x] 3.1 新增 `crates/paam-core/src/asset/frontmatter.rs`
- [x] 3.2 定义 `Frontmatter { name: String, description: String, #[serde(flatten)] extra: HashMap<String, serde_yaml_ng::Value> }`
- [x] 3.3 实现 `pub fn parse(skill_md_content: &str) -> Result<Frontmatter, FrontmatterError>`：识别 `---\n...\n---\n` 包围的 YAML 段；调 `serde_yaml_ng::from_str` 解析；返回 `Frontmatter`
- [x] 3.4 定义 `pub enum FrontmatterError { Missing /* 没有 frontmatter 段 */, Yaml(serde_yaml_ng::Error), MissingRequired(&'static str) /* 必填缺失，含字段名 */ }` —— 用 thiserror
- [x] 3.5 解析后检查 `name` 与 `description` 非空；缺一个返回 `MissingRequired("name")` / `MissingRequired("description")`
- [x] 3.6 解析成功后，若 `extra` 非空，`tracing::warn!` 一条日志（path 由调用方注入，所以 warn 在 discover 层做更合适——3.3 仅返回 Frontmatter，extra 由 discover 检查）
- [x] 3.7 单测：合法解析、缺 name、缺 description、缺整个 frontmatter 段、YAML 语法错误、含未知字段

## 4. asset 模块扩展（Skill 类型）

- [x] 4.1 新增 `crates/paam-core/src/asset/skill.rs`
- [x] 4.2 定义 `pub struct Skill { source_alias: String, relative_path: PathBuf, name: String, description: String, extra: HashMap<String, serde_yaml_ng::Value> }`
- [x] 4.3 加关联常量 `impl Skill { pub const MARKER: &'static str = "SKILL.md"; }`
- [x] 4.4 实现 `impl Asset for Skill`：`id()` 返回 `&self.name`、`kind()` 返回 `AssetKind::Skill`、`source_alias()` 返回 `&self.source_alias`、`relative_path()` 返回 `&self.relative_path`
- [x] 4.5 加访问器：`pub fn description(&self) -> &str`、`pub fn extra(&self) -> &HashMap<...>`
- [x] 4.6 实现 `pub fn read_body(&self, repo_root: &Path) -> Result<String>`：打开 `repo_root.join(&self.relative_path).join(Skill::MARKER)`，返回第二个 `---` 行之后的内容
- [x] 4.7 在 `asset/mod.rs` 加 `pub mod skill;` 与 `pub use skill::Skill;`
- [x] 4.8 在 `asset/mod.rs` 加 `pub mod frontmatter;`
- [x] 4.9 在 `asset/mod.rs` 顶层文档注释中加 ADR-0007 §7 修订说明（与 ADR 文件互相引用）
- [x] 4.10 单测：构造 Skill 实例验证 trait 方法返回；read_body 在 tempdir 中读取真实文件

## 5. discover 模块

- [x] 5.1 新增 `crates/paam-core/src/discover/mod.rs`，在 `lib.rs` 加 `pub mod discover;`
- [x] 5.2 定义 `pub const DEFAULT_IGNORE: &[&str]` 含 12 个目录名（详见 design.md 决策 6）
- [x] 5.3 实现 `pub fn skills_in(repo: &Path, ignore: &[String]) -> Vec<Skill>`：自写 BFS 递归扫描；调用 `frontmatter::parse` 解析每个 SKILL.md；解析失败时 stderr `eprintln!("warning: ...")` + 跳过；不抛 panic / 不返回 Result
- [x] 5.4 处理 source_alias：`skills_in` 不知道 alias，需要新增第二个公开函数 `pub fn skills_in_source(repo: &Path, source_alias: &str, ignore: &[String]) -> Vec<Skill>`，CLI 层调用此函数；或者让 `skills_in` 接受 `source_alias`（更直接）—— 决策时实现 `skills_in(repo: &Path, source_alias: &str, ignore: &[String]) -> Vec<Skill>`，CLI 层把 alias 传进来
- [x] 5.5 处理 frontmatter `extra`：解析成功后若 `extra` 非空，`tracing::warn!(path = %display, unknown_keys = ?extra.keys().collect::<Vec<_>>())` 一行
- [x] 5.6 不跟随 symlink（用 `metadata().is_dir()` + 不调用 `read_link`）
- [x] 5.7 单测：使用 tempdir + std::fs::write 手写 SKILL.md，覆盖 spec scenarios（单 skill / 多嵌套 / ignore 生效 / 失败跳过 / 自定义 ignore 替换默认 / 空数组不忽略）

## 6. CLI 命令树扩展

- [x] 6.1 把 `TrackArgs::target: String` 改为 `target: Vec<String>`（`#[arg(value_name = "TARGET", num_args = 1..=2)]`）
- [x] 6.2 在 `main.rs` 中根据 `target` 元素数与首个 token 分派：
  - 1 元素 == "list" → `handle_track_list`
  - 1 元素 != "list" → `handle_track_add`（视为 ssh-url）
  - 2 元素，第 1 个 == "skills" → `handle_track_skills(&root, &target[1])`
  - 其它 → 报错 + 用法提示
- [x] 6.3 实现 `fn handle_track_skills(root: &PaamRoot, alias: &str) -> Result<(), Error>`：
  - 通过 `source::list_sources(&root)` 获取已订阅源列表
  - 验证 alias 存在；不存在 → 返回新错误 `Error::AliasNotFound { alias }`，文案"订阅源 {alias} 不存在；可用 `paam track list` 查看已订阅源"
  - 计算本地路径 `root.sources_dir().join(<alias 对应的 host/owner/repo 嵌套目录>)`—— 实际实现：alias 本身就是嵌套目录形式，直接 `root.sources_dir().join(alias)`
  - 调 `config::effective_scan_ignore(&root)` 取 ignore 列表
  - 调 `discover::skills_in(&local_path, alias, &ignore)`
  - 调 `print_skills(&skills)` 表格输出；空列表时打印友好提示（design 决策 9）
- [x] 6.4 实现 `fn print_skills(skills: &[Skill])`：列宽自适应（NAME / DESCRIPTION / PATH），DESCRIPTION 截断 60 字符按 `chars().count()` + 末尾 `…`
- [x] 6.5 在 `error.rs` 新增 `Error::AliasNotFound { alias: String }`，文案见 6.3
- [x] 6.6 更新 `Cmd::Track` 的 doc comment，列出三种用法

## 7. 端到端验证（自动）

- [x] 7.1 `cargo build --workspace` 通过
- [x] 7.2 `cargo test --workspace` 全部通过（含新增 frontmatter / discover / config 单测）
- [x] 7.3 `cargo clippy --workspace --all-targets -- -D warnings` 无告警
- [x] 7.4 `cargo fmt --all -- --check` 通过
- [x] 7.5 `cargo tree -p paam-core | grep -E 'git2|libgit2|libssh2'` 仍无输出（确认上一 change 的契约未破坏）

## 8. 端到端验证（手动 dogfood）

- [x] 8.1 在沙盒下 `paam track <真实仓>` → `paam track skills <alias>`，验证表格输出与 spec 一致
- [x] 8.2 在沙盒中手动构造一个含若干 SKILL.md 的本地 git 仓，覆盖：含未知字段（验证 warn 日志）/ 缺 name（验证跳过 + warning）/ YAML 语法错误（验证跳过 + warning）/ 嵌套 .git 与 node_modules（验证 ignore 生效）
- [x] 8.3 在 `~/.paam/config.json` 中加 `"scan_ignore": [".git", "my-custom"]`，验证用户列表替换默认（如 `node_modules` 不再被忽略）
- [x] 8.4 验证 alias 不存在时 `paam track skills bogus/alias` 给出友好错误

## 9. 文档与日志

- [x] 9.1 在 `CHANGELOG.md` Unreleased 追加：「add: paam track skills <alias>，扫描已订阅源中的 SKILL.md 并展示；引入 Skill 类型与 discover 模块；config.json 新增可选 scan_ignore 字段」
- [x] 9.2 在 `.dev/docs/milestones/M1-plan.md` §七 Build 阶段进度日志追加一条（"YYYY-MM-DD：完成 paam-skill-discovery"），简述本 change 修订 ADR-0007、引入 Skill 类型与 discover、scan_ignore 配置等
- [x] 9.3 OpenSpec archive：所有任务 ✅ 后用 `/opsx:archive` 归档（流程提醒）
