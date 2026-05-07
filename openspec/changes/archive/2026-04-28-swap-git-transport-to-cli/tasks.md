## 1. 删除 git2-rs 依赖

- [x] 1.1 在 workspace `Cargo.toml` 的 `[workspace.dependencies]` 中删除 `git2 = "0.19"`
- [x] 1.2 在 `crates/paam-core/Cargo.toml` 的 `[dependencies]` 中删除 `git2 = { workspace = true }`
- [x] 1.3 全仓搜索确保不再有 `use git2::*` / `git2::Repository` 等引用
- [x] 1.4 跑一次 `cargo build` 期望编译失败（因为后续步骤还没改），失败信息将定位到所有需要重写的位置

## 2. 错误模型简化

- [x] 2.1 在 `crates/paam-core/src/error.rs` 删除 `SshAgentUnavailable` / `SshTransport(String)` / `GitNetwork(String)` / `GitGeneric(String)` 四个变体
- [x] 2.2 新增 `Error::GitNotFound`：文案"系统未安装 git，或 git 不在 PATH 中。请先安装：brew install git（macOS）/ apt install git（Linux）"
- [x] 2.3 新增 `Error::GitProcessFailure { exit_code: Option<i32>, stderr: String }`：文案"git 子进程失败（exit code: {exit_code:?}）：\n{stderr}"
- [x] 2.4 验证 `Error` 仍然实现 `Debug` / `std::error::Error`（thiserror 自动推导）

## 3. git 模块重写

- [x] 3.1 删除 `crates/paam-core/src/git/auth.rs` 整个文件
- [x] 3.2 改写 `crates/paam-core/src/git/mod.rs`：移除所有 `git2::*` 引用与 `pub mod auth;`
- [x] 3.3 实现 `pub fn ensure_git_available() -> Result<()>`：调 `Command::new("git").arg("--version").output()`；`io::ErrorKind::NotFound` 或 exit 非 0 都返回 `Error::GitNotFound`
- [x] 3.4 实现内部 helper `pub(crate) fn run(args: &[&str], cwd: Option<&Path>) -> Result<()>`：用 `Command::new("git")` + `args` + 可选 `current_dir` + `stderr(Stdio::inherit())` 跑 git；exit 0 → Ok；非 0 → `Error::GitProcessFailure { exit_code: status.code(), stderr: "<see terminal>".into() }`；进程启动失败（NotFound）→ `Error::GitNotFound`
- [x] 3.5 重写 `pub fn clone(url: &str, dest: &Path) -> Result<()>`：内部调 `run(&["clone", "--quiet", "--no-tags", url, dest_str], None)`
- [x] 3.6 路径转字符串：`dest.to_str().ok_or_else(|| Error::Io(io::Error::new(InvalidInput, "non-utf8 path")))?`（M1 仅 macOS，遇到非 UTF-8 路径直接报错）
- [x] 3.7 删除函数 `map_git_error` 与对 `git2::ErrorClass` / `ErrorCode` 的所有引用

## 4. 测试 fixture 改写：用 system git 构造 bare repo

- [x] 4.1 新增 `crates/paam-core/src/test_support.rs`（`#[cfg(test)] pub mod test_support;`），抽出 fixture helpers
- [x] 4.2 在 `test_support` 实现 `pub fn make_fixture_repo() -> (TempDir, String)`：fork system git 完成 `init --bare`、`init` 普通 repo、`config user.email`、`config user.name`、`commit --allow-empty`、`push file://<bare>:HEAD:master`，最终返回 `(TempDir, "file://<bare_path>")`
- [x] 4.3 实现 helper `fn run_git(args: &[&str])` 与 `fn run_git_in(cwd: &Path, args: &[&str])`：内部 `Command::new("git").args(...)` + `assert!(status.success())`
- [x] 4.4 删除 `git/mod.rs::tests::make_fixture_repo` 与 `source/mod.rs::tests::make_fixture_repo` 中基于 git2 的旧实现，改 `use crate::test_support::make_fixture_repo;`
- [x] 4.5 在 `lib.rs` 中加 `#[cfg(test)] pub mod test_support;`

## 5. git 模块测试重写

- [x] 5.1 重写 `clone_local_bare_repo_via_file_protocol`：用 `test_support::make_fixture_repo` 拿 fixture URL，调 `git::clone`，断言 `dest.join(".git").is_dir()`
- [x] 5.2 新增测试 `ensure_git_available_when_git_in_path`：直接调 `ensure_git_available()` 应 Ok（CI 上 git 可用）
- [x] 5.3 新增测试 `clone_failure_returns_git_process_failure`：用不存在的 file:// 路径调 `clone`，断言 `matches!(err, Error::GitProcessFailure { .. })`

## 6. source 模块更新

- [x] 6.1 在 `source::track` 入口处（`paths::ensure_initialized()` 之后、`parse_ssh_url` 之前）调用 `git::ensure_git_available()?`
- [x] 6.2 `source::list_sources` 不调用 `ensure_git_available`（本地操作）
- [x] 6.3 重写 `source::tests::clone_failure_does_not_leave_partial_dir_or_config_entry` 中的错误断言：从 "不强限定" 改为 `assert!(matches!(err, Error::GitProcessFailure { .. }))`
- [x] 6.4 `source::tests::make_fixture_repo` 改为 `use crate::test_support::make_fixture_repo;`，删除内联实现
- [x] 6.5 其他 source 测试保持（track_clone_register_round_trip / duplicate_track_is_rejected / track_rejects_non_ssh_url_input）

## 7. paam-cli 简化

- [x] 7.1 移除 `report_error` 中针对 `SshAgentUnavailable` 的特化分支（变体已不存在，编译器会提醒）
- [x] 7.2 `GitProcessFailure` 在 `report_error` 中只打印 `eprintln!("错误：{}", err)` 即可（stderr 在 git 子进程时已透传，paam 不重复打印）
- [x] 7.3 `GitNotFound` 同样让 `Display` 文案输出（已含安装建议）

## 8. 端到端验证（自动）

- [x] 8.1 `cargo build --workspace` 通过（确认无 git2 残留引用）
- [x] 8.2 `cargo test --workspace` 全部通过（旧测试断言更新后；新增测试通过）
- [x] 8.3 `cargo clippy --workspace --all-targets -- -D warnings` 无告警
- [x] 8.4 `cargo fmt --all -- --check` 通过
- [x] 8.5 `cargo tree -p paam-core | grep -i git2` 期望无输出（确认 git2 已彻底移除，含传递依赖）
- [x] 8.6 对比 release binary 大小：记录 `cargo build --release` 后 `target/release/paam` 体积，与 paam-foundation-track 时对比（用于验证 binary size 收益）

## 9. 端到端验证（手动 dogfood）

- [x] 9.1 用真实 GitHub SSH URL 跑 `paam track`，**不预先 ssh-add**，验证用户系统 git 能跑通的场景 paam 也能跑通
- [x] 9.2 重复 paam-foundation-track 的负面剧本：HTTPS URL / `foo/bar` 简写 / 重复 track 仍按 spec 报错
- [x] 9.3 `PATH=/tmp paam track <ssh-url>` 验证 `GitNotFound` 错误与安装提示
- [x] 9.4 验证 `paam track list` 在 PATH=/tmp 仍可正常工作（不强制 git）

## 10. 文档与日志

- [x] 10.1 在 `CHANGELOG.md` Unreleased 追加：「change: 删除 git2-rs 依赖，所有 git 操作走系统 git CLI 子进程；移除 ssh-agent 限制」
- [x] 10.2 在 `.dev/docs/milestones/M1-plan.md` §七 Build 阶段进度日志追加一条（"YYYY-MM-DD：完成 swap-git-transport-to-cli。彻底删除 git2-rs 依赖；所有 git 操作走 system git 子进程；fixture 改用 system git 构造；记录 binary size 减小 X MB"）
- [x] 10.3 OpenSpec archive：所有任务 ✅ 后用 `/opsx:archive` 归档（流程提醒，不在本任务清单内执行）
