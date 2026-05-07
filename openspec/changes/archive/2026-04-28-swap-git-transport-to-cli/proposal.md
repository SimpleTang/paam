## Why

paam-foundation-track 落地后立刻在 dogfooding 中暴露 libssh2 与系统 OpenSSH 行为差异：

| 问题 | 实测影响 |
|---|---|
| libssh2 不读 `~/.ssh/config` | 用户把 `github.com` 重定向到 `ssh.github.com:443`（绕开 22 端口封锁），paam 硬连 22 直接 banner 失败 |
| libssh2 不会用 macOS Keychain | 即便用户系统 git 能正常跑，paam 仍要求 `ssh-add` |
| libssh2 不支持 ProxyCommand / ProxyJump | 企业堡垒机场景完全不可用 |
| host key 处理与系统 ~/.ssh/known_hosts 解耦 | 后续会出现首次连接需要交互确认却没交互通道的问题 |

每加一项 fallback 都是在重新发明 OpenSSH 的一部分。本 change 选择"减法"：**把 git transport 从 git2-rs/libssh2 切到 fork 系统 `git` CLI 子进程**，让 OpenSSH 配置 / Keychain / known_hosts / ProxyCommand 全部 just work，paam 不再"重新发明 ssh"。

这是架构层修正，不计入 M1 原 4-change 序列（A' 切法之外）。完成后再回到 ② paam-skill-discovery。

## What Changes

- 重写 `paam_core::git::clone(url, dest)`：实现从 `git2::Repository::clone` 改为 `std::process::Command::new("git").args(["clone", "--quiet", "--no-tags", url, dest_str]).status()`；函数签名不变
- 删除 `crates/paam-core/src/git/auth.rs`（ssh-agent 鉴权回调链路彻底移除）
- 简化 `Error` enum：删除四个 git2-specific 变体（`SshAgentUnavailable` / `SshTransport` / `GitNetwork` / `GitGeneric`），统一为 `GitProcessFailure { exit_code: Option<i32>, stderr: String }`
- 新增 `Error::GitNotFound`：当 PATH 上找不到 `git` 时返回，错误消息引导用户安装
- 启动时探测：任何涉及远程 git 操作的子命令（M1 阶段为 `paam track <url>`）开跑前 fork `git --version`，缺失则返回 `GitNotFound`
- 错误展示：捕获 git 子进程 stderr 后透传给用户终端，让 git 自己负责报错文案；exit code 非 0 时构造 `GitProcessFailure`
- 重写 `paam-core::git::tests`：原 file:// bare repo fixture 沿用（system git 同样支持 file://），`clone_local_bare_repo_via_file_protocol` 等测试逻辑保持
- 重写 `paam-core::source::tests`：错误类型断言从 `GitNetwork`/`GitGeneric` 改为 `GitProcessFailure`
- CLI 层 `report_error`：移除 ssh-agent / 传输层错误的特化文案；`GitProcessFailure` 直接打印 stderr

- **彻底删除 `git2-rs` 依赖**（workspace + paam-core 的 Cargo.toml）：所有 git 操作（含远程 transport 与未来本地仓 commit / status）一律走 system git 子进程
- 测试 fixture 改写：原本用 `git2::Repository::init_bare` + `treebuilder` + `commit` 构造 bare repo 的代码，改为 fork system git 子进程（`git init --bare`、`git commit --allow-empty`、`git push`）

**明确不做：**
- 不引入异步运行时（仍同步 `spawn().wait()`）
- 不做进度条 / 速率限制 / 自定义 GIT_DIR
- 不解析 git stderr 做精细错误分类（exit code + stderr 透传足够）
- 不在 PATH 之外搜索 git 二进制（如 `/opt/homebrew/bin`）；依赖系统 PATH 配置
- 不改动 source-management 之外的 capability

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `source-management`: 修改"SSH 鉴权策略" requirement 与"添加订阅源"的 clone 失败 scenario 错误类别；新增"依赖系统 git 命令" requirement

## Impact

**代码：**
- `crates/paam-core/src/git/mod.rs`: 重写 `clone` 与错误映射；新增 `ensure_git_available()` 探测函数
- `crates/paam-core/src/git/auth.rs`: 删除整个文件
- `crates/paam-core/src/error.rs`: 删 4 个变体，加 2 个变体（`GitProcessFailure` / `GitNotFound`）
- `crates/paam-core/src/source/mod.rs`: 调用 `git::ensure_git_available()`；错误类型断言更新
- `crates/paam-cli/src/main.rs`: `report_error` 简化（移除 ssh-agent 特化文案）

**依赖：**
- 不新增依赖
- **删除 `git2-rs`**（workspace + paam-core 的 Cargo.toml）；后续所有 git 操作（含本地仓 commit / status / read）都将走 system git 子进程
- `chrono` / `serde` / `serde_json` / `directories-next` / `thiserror` / `tracing` / `clap` / `tempfile` 依旧

**外部系统：**
- 新增运行期依赖：用户系统 PATH 上必须有 `git`（M1 阶段 paam-foundation-track dogfood 已确认前提满足，因 `git ls-remote` 能跑通）
- 不再依赖 ssh-agent；改为依赖系统 git 已配置好的 OpenSSH 链路（含 `~/.ssh/config`、Keychain、`known_hosts`、ProxyCommand 等全部 OpenSSH 行为）

**用户可观察行为变化：**
- `paam track` 在 macOS Keychain + 加密 key 场景下不再需要 `ssh-add`
- 错误消息从 paam 自定义中文文案变成 git 原生 stderr（更准确，但失去中文化和"修复建议"的便利性——M2 可考虑回归特化文案）
- 新增"未安装 git"错误场景，给出引导

**对后续 change 的契约：**
- **后续所有 git 操作（远程 fetch / pull / push 与本地 init / add / commit / status / read）一律走 system git 子进程**，paam-core 内不再有 git 类型化绑定
- `Error::GitProcessFailure` 是后续所有 git 操作的统一错误形态
