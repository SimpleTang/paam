## Context

paam-foundation-track 落地后立刻在 dogfooding 中验证 SSH clone 路径，遇到 `Failed getting banner` 错误。诊断结果：

- 用户 `~/.ssh/config` 把 `github.com` 重定向到 `ssh.github.com:443`（绕开 ISP 对 22 端口的封锁）
- libssh2 不读 `~/.ssh/config`，硬连 `github.com:22` → SSH 协议握手层失败
- 同一个仓 `git ls-remote` 能成功，因为系统 git 走 OpenSSH（读 config）

错误分类细化（`SshAgentUnavailable` vs `SshTransport`）+ 文案改进只是把误诊压制住，**根问题没动**：libssh2 与 OpenSSH 的差异是结构性的，未来还会撞到 macOS Keychain、ProxyCommand、known_hosts、ed25519-sk 硬件 key 等长尾。

本 change 接受这个现实，把"远程 git 操作"全权委托给系统 `git` CLI——它已经被全世界各种 git 服务和 SSH 拓扑磨合过 N 年，paam 不需要重写。

## Goals / Non-Goals

**Goals:**
- 让 `paam track` 在用户系统 git 能跑通的所有场景下都能跑通（含 `~/.ssh/config` 重定向、Keychain 加密 key、ProxyCommand 等）
- **彻底单一化 git backend**：删除 git2-rs 依赖，所有 git 操作（远程 + 本地）一律走 system git 子进程
- 简化错误模型：`Error` 中的 git2-specific 变体合并为单个 `GitProcessFailure`，降低后续 change 的认知负担
- 减少编译时间与 binary size（libgit2 + libssh2 静态链接占数 MB）
- 保留 source-management capability 的全部对外行为（URL 解析、alias 推导、回滚语义、CLI 输出格式），仅鉴权机制描述更新

**Non-Goals:**
- 不引入异步 / 多进程并发 / 进度条 / 自定义 GIT_DIR
- 不解析 git stderr 做精细错误分类
- 不在 PATH 之外搜索 git 二进制
- 不改动 source-management 之外的 capability（asset / config / paths 模块均不动）
- 不为系统 git 提供"内嵌备用"（缺 git 直接报错引导安装，不打包二进制）
- 不在 paam-core 中保留任何"原生 git 类型"（`Repository` / `Oid` / `Commit` / `Tree` 等）：调用方一律走子进程

## Decisions

### 1. 切到 system git CLI 子进程，不继续打 libssh2 补丁

**选择：** `git::clone(url, dest)` 内部用 `std::process::Command::new("git").args(["clone", "--quiet", "--no-tags", url, dest]).status()` 实现。

**Why：**
- libssh2 在 dogfooding 中暴露的 4 类不兼容（`.ssh/config` 不读、Keychain passphrase 不可达、ProxyCommand 缺失、known_hosts 与系统解耦）每一项都需要 paam 重新实现 OpenSSH 的对应行为，**累积工程量 ≫ 直接调用 system git**
- 系统 git 已经被各种 SSH 拓扑（公司堡垒机、国内端口屏蔽、硬件 key、企业 PKI）磨合过，零成本继承所有这些行为
- 用户已 `git clone` 成功 = paam 必能 clone 成功，行为完全可预期
- 代码量是减法：`git/auth.rs` 整个删除，`map_git_error` 简化为"非 0 exit 透传 stderr"

**Alternatives considered:**
- 继续 libssh2 + 加 `.ssh/config` parser：不解决 Keychain / ProxyCommand 长尾
- libssh2 + key file fallback：仅覆盖无 passphrase key 用户
- gitoxide（gix）：纯 Rust git 实现，但 SSH 仍需要外部 ssh 客户端（实质等同 fork git）；且 gix M1 暂未稳定
- fork `git2::transport::register` 写自定义 transport：复杂度等同自实现 OpenSSH

**契约固化：** `git::clone` 函数签名 `(url: &str, dest: &Path) -> Result<()>` 保持不变，调用方（`source::track`）零改动。

### 2. 彻底删除 git2-rs 依赖

**选择：** 从 workspace `Cargo.toml` 与 `paam-core` `Cargo.toml` 删除 `git2`；所有 git 操作（含未来 ② / ③ 的本地仓 init / add / commit / status / read）一律走 system git 子进程。paam-core 内不再有任何 `git2::*` 类型导出。

**Why：**
- **单一 backend，认知负担最低**：调用方不需要分辨"这个操作是远程还是本地"，统一通过 `git::run(args, cwd)` 包装；后续 change 的 design 不必反复论证"为什么这块用 git2 那块用子进程"
- **编译时间 + binary size 收益显著**：`git2-rs` 静态链接 libgit2 + libssh2，是当前依赖图中最重的一块；删除后 `cargo build` 时间下降几个数量级、release binary 减小几 MB
- **不会再被 git2 SSH 行为坑**：libssh2 的所有问题（`.ssh/config` 不读、Keychain 不可达、ProxyCommand 缺失）一次性根除，未来不会有"在测试 fixture 中用 git2 又踩到 libssh2 坑"的二次摩擦
- **性能损失可接受**：paam 是 CLI 工具，操作频率低（用户级而非批量），即便 ③ 的 auto-commit 每次 fork 3-4 个子进程（add / commit），单次延迟仍在毫秒级，远低于网络 IO
- **测试不再依赖 git2**：fixture 构造改用 system git（详见决策 8），CI 上同样能跑

**Alternatives considered:**
- 保留 git2-rs 给本地仓操作：拒绝（双 backend 认知负担、编译时间不减、未来 fetch / pull 走子进程时仍需在 design 里反复论证边界）
- 改用 gitoxide (`gix`)：拒绝（M1 阶段尚未稳定，且 SSH 仍需要外部 ssh 客户端，对核心痛点没帮助）

**契约固化：** "**所有 git 操作走 system git 子进程**" 是后续所有 change 的硬性约束。任何 change 在 paam-core 中重新引入 git 库依赖（git2 / gix / 自实现）都需要先开新 change 推翻本决策。

### 2-bis. 包装函数 `git::run` 作为后续 change 的复用层

**选择：** 在 `paam-core::git` 模块导出一个低层 helper：

```rust
pub fn run(args: &[&str], cwd: Option<&Path>) -> Result<()>;
pub fn run_capture(args: &[&str], cwd: Option<&Path>) -> Result<String>;  // 取 stdout
```

`clone(url, dest)` 内部调 `run`；后续 ③ 的 `init` / `add` / `commit` 也调 `run`；后续 ② 的 `git show HEAD:path` 用 `run_capture` 取 SKILL.md 内容。所有调用都走统一的：
- `Stdio::inherit()` 或 `Stdio::piped()` 选择
- 错误映射（exit 0 → Ok / 非 0 → `GitProcessFailure`）
- `tracing::debug!` 日志

**Why：** 避免后续 change 各自重复 `Command::new("git").args(...).status().map_err(...)` 的样板；本 change 只暴露 `clone` + `ensure_git_available`，但内部为后续复用打好基础。

### 3. 错误模型简化：4 个 git2 变体 → 1 个 GitProcessFailure

**选择：**

```rust
pub enum Error {
    // 删除：
    // SshAgentUnavailable
    // SshTransport(String)
    // GitNetwork(String)
    // GitGeneric(String)

    // 新增：
    #[error("系统未安装 git，或 git 不在 PATH 中。请先安装：brew install git（macOS）/ apt install git（Linux）")]
    GitNotFound,

    #[error("git 子进程失败（exit code: {exit_code:?}）：\n{stderr}")]
    GitProcessFailure {
        exit_code: Option<i32>,
        stderr: String,
    },

    // 其它变体（HomeNotFound / Io / Json / UnsupportedSchemaVersion /
    // InvalidGitUrl / AliasAlreadyExists）保持不变
}
```

**Why：**
- 在 system git 后端下，paam 拿到的是 exit code + stderr，**没有信息能让 paam 比 git 自己分类得更准**
- git 已经写了高质量的中英文（取决于 LANG 环境变量）错误消息——透传比重写好
- 删除 4 个变体后，CLI 层 `report_error` 简化为：`stderr` 直接 print；`GitNotFound` 给安装提示；其它非 git 变体保持现状
- 后续 change（fetch / pull / push 走 system git）可直接复用 `GitProcessFailure`，错误形状统一

**Risk：** 失去"鉴权失败"vs"网络错误"的程序级区分能力 → 缓解：M1 阶段 paam 没有 retry / fallback 等差错路径业务逻辑，本来就不需要这个区分；M2 引入 `paam update` 等批量操作时再评估是否需要解析 stderr。

### 4. system git 缺失探测：首次远程操作前一次性探测

**选择：**

- 探测函数 `git::ensure_git_available() -> Result<()>` 内部调 `Command::new("git").arg("--version").output()`
  - 成功（exit 0）→ Ok(())
  - 失败 / 进程启动错误（io::ErrorKind::NotFound）→ `Error::GitNotFound`
- 调用时机：`source::track()` 入口处（任何远程 git 操作之前），不放在 `paths::ensure_initialized()`（`paam track list` 不需要 git）
- 不缓存探测结果（CLI 一次进程一次调用，缓存无收益）

**Why：**
- 在 `track list` 等不需要 git 的命令上不强制要求 git 存在（用户可能只想 list 已订阅的源）
- 探测命令 `git --version` 极轻（毫秒级），不引入感知性能成本
- 错误消息提供具体安装建议（`brew install git` / `apt install git`），降低用户摩擦

**Alternative considered:** 在 main 启动时探测——拒绝。会让 `paam track list` 也强制要求 git，违反最小依赖原则。

### 5. stderr 透传策略

**选择：**
- 用 `Command::stderr(Stdio::inherit())` 让 git 子进程直接写到 paam 的 stderr（保留 ANSI 颜色 / progress bar）
- exit code 非 0 时返回 `GitProcessFailure { exit_code, stderr: "<已透传至终端>".into() }`，stderr 字段仅用于程序级判断（如单测断言），不再保存原文

**Why：**
- 最简：`Stdio::inherit()` 一行解决，git 自动处理 TTY 检测、颜色、缓冲
- 用户已经在 paam 命令行下，git 的输出风格用户熟悉（dogfood 友好）
- 不会出现 paam 把 stderr 缓存后再打印的双重显示问题

**Trade-off：** 单测时 stderr 直接打到 cargo test 的输出里（视觉噪音），但测试断言只看 exit_code 与 `Error` 枚举类型，不影响断言。

**Alternative considered:** 用 `Stdio::piped()` 完全捕获后由 paam 决定是否打印——拒绝。需要自己处理 TTY 检测、颜色、缓冲，复杂度激增。

### 6. clone 命令参数选择

**选择：** `git clone --quiet --no-tags <url> <dest>`

**Why：**
- `--quiet`：抑制 "Cloning into..." 等非错误信息，减少噪音；错误仍会打印
- `--no-tags`：M1 paam 不消费 tags，避免无谓网络流量与本地存储
- 不加 `--depth=1`：保留完整历史以便后续 ③ change 读取 SKILL.md 的提交时间 / 作者
- 不加 `--branch=<x>`：让 git 用远程默认分支（HEAD 指向）

**未来扩展（不在本 change 范围）：**
- `paam update` 时考虑 `--prune` 让删除的远程分支同步消失
- 若 source 仓特别大，未来加用户配置允许 `--depth=N`

### 7. 测试 fixture 改用 system git 子进程构造

**选择：** 原本用 `git2::Repository::init_bare` + `treebuilder` + `commit` 构造 bare repo 的代码（在 `git/mod.rs::tests` 与 `source/mod.rs::tests` 中重复）抽出到 `paam-core` 顶层 `#[cfg(test)] mod test_support`，改为通过 fork system git 子进程实现：

```rust
// 伪代码
fn make_fixture_repo() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let bare_path = dir.path().join("origin.git");
    let work_path = dir.path().join("work");

    run_git(&["init", "--bare", bare_path.to_str().unwrap()]);
    run_git(&["init", work_path.to_str().unwrap()]);
    run_git_in(&work_path, &["config", "user.email", "test@example.com"]);
    run_git_in(&work_path, &["config", "user.name", "test"]);
    run_git_in(&work_path, &["commit", "--allow-empty", "-m", "init"]);
    run_git_in(&work_path, &["push", bare_path.to_str().unwrap(), "HEAD:master"]);

    (dir, format!("file://{}", bare_path.display()))
}
```

**Why：**
- 删 git2 依赖后 fixture 不能再用 git2 构造，必须找替代
- system git 同样支持 `init --bare` / `commit --allow-empty` / `push file://` —— 100% 等价能力
- 5-6 次 fork 在测试中是毫秒级开销，CI 上无感
- fixture helper 抽到 `test_support` 共享，避免 git/tests 与 source/tests 重复样板
- `--allow-empty` 让 root commit 用空 tree，无需写入文件

**Risk：** CI 环境 / 本地开发机若没装 git，所有测试集体 fail → 缓解：`Cargo.toml` 文档 / 项目 README 标注"开发与测试需要 system git"；这与运行期约束一致，不引入新前提。

**Alternative considered:** 用纯 Rust 构造空 git pack 文件（自己写 .git/refs/heads/master + .git/HEAD + 一个 root commit）—— 拒绝，重写 git 内部格式得不偿失。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| 用户系统未安装 git | 启动时探测，错误消息引导安装；M1 dogfood 已确认前提满足 |
| git 子进程 stderr 不带中文化 / paam 风格化 | M1 接受：git 自己的错误消息更准确，且支持 LANG=zh_CN 时已有中文；M2 可在 `report_error` 内对常见 git 错误做后处理特化 |
| Windows 上 `git` 二进制可能在 `Program Files\Git\bin\git.exe` 而非 PATH | M1 不支持 Windows（M4 适配），届时再处理；macOS / Linux 上 PATH 配置稳定 |
| fork 子进程比 in-process 慢（~ms 级） | 远程 IO 本身百毫秒级以上，子进程开销可忽略；本地仓 commit 单次 fork 几个进程仍毫秒级，CLI 用户级操作可接受 |
| `Stdio::inherit()` 让单测输出有视觉噪音 | 接受；测试断言看 exit code + `Error` 枚举，噪音不影响正确性 |
| 删除 git2 后某天发现仍需要 in-process git（如 GUI 进度条 / 高频本地仓查询） | M2 paam-app GUI 阶段重新评估；届时若必要，可作为新 change 重新引入 git 库（gix / git2）作为补充 backend，但需先推翻本 change 的契约固化 |
| 测试与开发依赖 system git | 与运行期依赖一致，不引入新前提；README 标注 |

## Migration Plan

不涉及用户数据迁移（仅代码层 transport 替换；`~/.paam/sources/` 已有 clone 目录的格式没变）。

代码层：
1. 删除 `git/auth.rs`
2. 重写 `git/mod.rs` 中 `clone` 与测试
3. 简化 `error.rs`
4. 更新 `source/mod.rs` 中 `track` 入口的 `ensure_git_available()` 调用与测试断言
5. 简化 `paam-cli/src/main.rs` 的 `report_error`
6. 更新 spec（MODIFIED + ADDED）

回滚：本 change 是单 commit 修改，git revert 即可回到 paam-foundation-track 的 libssh2 实现。

## Open Questions

1. **system git 探测时是否检查最低版本？** —— M1 决定不检查（任意 modern git 都支持 `clone --quiet --no-tags`，2.0+ 即可）。M2 若用上 `git switch`、稀疏 checkout 等较新特性时再加版本最低门限。
2. **`GitProcessFailure` 在 stderr 已透传的情况下是否还需要存储 stderr 字符串？** —— 当前决定存储但置为占位 `"<see terminal>"` 表示已透传；这样单测断言能正常匹配 `matches!(err, GitProcessFailure { .. })`；M2 若加 GUI 模式可改为捕获完整 stderr 用于弹窗显示。
3. **是否同时把 file:// 协议视为合法的 SSH URL 输入**（便于内部测试）？—— 不动。`source::url` 仍只接受两种 SSH 形式；测试 fixture 直接调用 `git::clone(file_url, dest)` 绕开 URL parser，与现有做法一致。
