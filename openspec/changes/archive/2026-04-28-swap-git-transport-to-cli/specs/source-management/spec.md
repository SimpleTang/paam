## MODIFIED Requirements

### Requirement: SSH 鉴权策略

paam 在执行 git clone 时 SHALL 通过 fork 系统 `git` CLI 子进程完成网络层操作，从而**完整复用**用户系统 OpenSSH 的全部凭证与连接行为，包括 `~/.ssh/config`（HostName / Port / User / IdentityFile / ProxyCommand / ProxyJump 等指令）、ssh-agent、`~/.ssh/known_hosts`、macOS Keychain 中缓存的 passphrase，以及任何由系统 git 配置文件（`~/.gitconfig` / `/etc/gitconfig`）控制的行为。paam 自身不实现 SSH 客户端、不直接读取私钥文件、不解析 SSH 配置。

#### Scenario: 用户的 ~/.ssh/config 中已有 Host 重定向

- **WHEN** 用户 `~/.ssh/config` 含 `Host github.com` 块（重定向 HostName 到 `ssh.github.com` / Port 到 443 等）
- **AND** 用户执行 `paam track git@github.com:foo/bar.git`
- **THEN** paam fork 的 `git clone` 子进程会按 `~/.ssh/config` 完成实际连接
- **AND** clone 成功，`git@github.com:foo/bar.git` 与系统 `git clone` 行为完全一致

#### Scenario: 加密 key + macOS Keychain 缓存 passphrase

- **WHEN** 用户的 SSH 私钥有 passphrase 且已用 `ssh-add --apple-use-keychain` 把 passphrase 存入 Keychain
- **AND** 用户执行 `paam track <ssh-url>`
- **THEN** paam 不主动询问 passphrase
- **AND** 系统 OpenSSH 自动从 Keychain 读取 passphrase 解锁 key 完成鉴权
- **AND** clone 成功

#### Scenario: 鉴权失败时 stderr 透传

- **WHEN** 用户执行 `paam track <ssh-url>` 但远端拒绝凭证（如 key 不在被授权列表中）
- **THEN** git 子进程的 stderr 直接打印到用户终端（保留原始格式与可能的中英文）
- **AND** paam 返回 `Error::GitProcessFailure { exit_code, stderr: <占位> }`
- **AND** paam 不再添加自定义鉴权失败文案

---

### Requirement: 添加订阅源

`paam track <git-url>` SHALL 完成"解析 URL → 推导 alias → clone 到本地缓存目录 → 注册到配置文件"四步操作，任何一步失败时整体回滚（不留半成品）。clone 实现 SHALL 通过 fork 系统 `git` CLI 子进程完成。

#### Scenario: 成功添加一个新订阅源

- **WHEN** 用户执行 `paam track git@github.com:SimpleTang/paam-skills.git`
- **AND** 该 alias 此前未被订阅
- **AND** 用户系统 git 已配置好 SSH 凭证（能用 `git ls-remote` 访问该仓）
- **THEN** 系统将完整 git 仓克隆到 `~/.paam/sources/github.com/simpletang/paam-skills/`
- **AND** `config.json` 中追加一条 source 记录（含 alias、原始 url、added_at）
- **AND** 命令以 exit code 0 结束，输出至少包含 alias 与本地缓存路径

#### Scenario: alias 已存在时拒绝重复 track

- **WHEN** 用户对已订阅过的同一 git URL 再次执行 `paam track`
- **THEN** 系统拒绝并返回非零 exit code
- **AND** 错误信息提示该 alias 已存在
- **AND** 不修改 `config.json`、不动既有缓存目录

#### Scenario: clone 失败时回滚

- **WHEN** 用户执行 `paam track`，git 子进程因任意原因（鉴权失败 / 网络错误 / 仓不存在 / 用户中断 等）非零退出
- **THEN** 系统不向 `config.json` 写入任何记录
- **AND** 不在 `~/.paam/sources/` 留下不完整的克隆目录（已部分写入的目录被清理）
- **AND** 返回 `Error::GitProcessFailure { exit_code, stderr }`，git 原始 stderr 已透传至用户终端

## ADDED Requirements

### Requirement: 依赖系统 git 命令

paam 在执行涉及远程 git 操作的子命令（M1 阶段为 `paam track <url>`）前 SHALL 先探测 PATH 上是否存在可执行的 `git` 命令；若探测失败，SHALL 以非零 exit code 终止，并在错误信息中提供针对常见操作系统的安装建议。仅本地操作（如 `paam track list`、未来的 `paam list` / `paam info` 等）SHALL NOT 强制要求 git 存在。

#### Scenario: 系统已安装 git

- **WHEN** 用户系统 PATH 上存在 `git` 命令
- **AND** 用户执行 `paam track <ssh-url>`
- **THEN** 探测通过（`git --version` 成功）
- **AND** track 流程继续执行

#### Scenario: 系统未安装 git

- **WHEN** 用户系统 PATH 上不存在 `git` 命令
- **AND** 用户执行 `paam track <ssh-url>`
- **THEN** paam 以非零 exit code 失败，返回 `Error::GitNotFound`
- **AND** 错误信息明确指出系统未安装 git，并给出安装建议（如 `brew install git` / `apt install git`）
- **AND** 不创建本地缓存目录、不修改配置文件

#### Scenario: 仅本地操作不强制要求 git

- **WHEN** 用户系统 PATH 上不存在 `git` 命令
- **AND** 用户执行 `paam track list`
- **THEN** 命令正常执行（不触发 git 探测）
- **AND** 输出当前 `config.json` 中的订阅源列表
