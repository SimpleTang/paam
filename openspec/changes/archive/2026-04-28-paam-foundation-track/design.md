## Context

paam 当前是一个空的 Cargo workspace（`paam-core` + `paam-cli`），所有 workspace 依赖已声明但未启用，crate 内仅有占位代码。本 change 是 M1 milestone 的第①块（A' 切法），也是 OpenSpec 在本仓的工作流首跑。

约束：
- ADR-0001：Tauri v2 + Rust 技术栈（M1 只用 Rust CLI 部分）
- ADR-0002：Cargo workspace；业务逻辑必须沉淀到 `paam-core`，CLI 层只做参数解析与展示
- ADR-0007：`~/.paam/` 是产品钦定的工作目录根，三层模型为 source / local-repo / target。本 change 只触及 source 层
- M1-plan §三 3.1：M1 限定 SSH only、无 HTTPS、无 owner/repo 简写、不处理同名冲突（要求源唯一）

后续 ② / ③ / ④ change 都会复用本 change 落地的工作目录布局、配置文件 schema、git 封装。这意味着这三件事是**面向后续 change 的契约层**，本文档要为它们做出明确决策。

## Goals / Non-Goals

**Goals:**
- 落地 `~/.paam/` 工作目录的初始化与路径常量
- 落地用户配置文件的 schema、格式、读写 API
- 落地 SSH git URL → alias 的解析规则（决定本地缓存目录布局）
- 落地 git2-rs 的最小封装（clone）与 ssh-agent 鉴权链路
- 落地 paam-cli 的子命令分发框架，并实现 `track` / `track list` 两条 P0 命令
- 引入 `Asset` trait + `AssetKind` enum 的 type-agnostic 抽象骨架（ADR-0007 落地），为 ② / ③ / M2 的 prompts / mcp 扩展占位
- 为后续 change（local-repo、metadata、sync）留出清晰的契约边界

**Non-Goals:**
- 不写 `Asset` 的具体实现者（`Skill` 实现推到 ② paam-skill-discovery）
- 不实现 `untrack`、`track skills <alias>`、HTTPS / PAT、owner/repo 简写
- 不引入 metadata 文件（`.metadata.json` 在 ③ change 落地）
- 不引入异步运行时；CLI 主体保持同步
- 不做端到端 CLI 集成测试（M1 DoD 允许靠手动验收剧本）

## Decisions

### 1. 配置文件格式：JSON

**选择：** 用户级配置文件 `~/.paam/config.json`，JSON 格式，由 `serde_json` 序列化（保持有序字段、双空格缩进 pretty-print）。

**Why：**
- 项目钦定 JSON（与 M1-plan §五·B 实现指南一致；§三 3.5 字面写的 "paam.yaml" 与 §五·B "config.json" 之间的不一致，以 JSON 为准）
- 后续 ③ change 引入的 `.metadata.json` 同样是 JSON，整套数据文件保持单一格式，降低工具链与心智负担
- `serde_json` 已在 workspace 依赖中，零额外依赖；不引入 YAML / TOML 链路
- 即便用户偶尔手编，JSON 的 schema 严格性反而是优点（错就报错，不会被注释/缩进迷惑）

**Alternatives considered:**
- YAML：拒绝。引入额外依赖（`serde_yaml` 已停维护、`serde_yaml_ng` 是社区 fork），且与 metadata.json 格式分裂
- TOML：拒绝。M1-plan 全文未约定 TOML，且对嵌套结构表达能力弱

**契约固化：** 配置文件路径硬编码为 `~/.paam/config.json`（M2 再考虑 XDG），后续 change 必须通过 `paam-core::config` 模块访问，不得直接 `fs::read`。

### 2. 配置文件 schema（v0）

```json
{
  "version": 1,
  "sources": [
    {
      "alias": "github.com/simpletang/paam-skills",
      "url": "git@github.com:SimpleTang/paam-skills.git",
      "added_at": "2026-04-28T10:00:00Z"
    }
  ]
}
```

**Why：**
- `version` 字段必备：迁移空间（ADR-0004 配置兼容性虽未 Accept，但保留前向兼容字段成本极低）
- `sources` 顶层 list；后续 ② 章节再加 `installed_skills` 字段时不冲突
- `alias` 不是用户取的友好名，而是从 URL 推导（见决策 3），保证全局唯一
- `added_at` ISO-8601 字符串，便于 `track list` 排序展示
- M1 限定不预留 `prompts:` / `mcp:` 字段（M1-plan §三 3.5 与 ADR-0007 同），M2 再扩字段

**契约固化：** 任何后续 change 新增字段，必须在 design.md 显式声明并保持 v1 schema 向后兼容。schema 破坏性变更需要 bump `version` 并写迁移逻辑。

### 3. alias 推导规则：`<host>/<owner>/<repo>`

**选择：** 从 SSH URL 解析出 `host`、`owner`、`repo`，小写化后拼成 `<host>/<owner>/<repo>` 作为 alias。

```
git@github.com:SimpleTang/paam-skills.git
  → host=github.com, owner=SimpleTang, repo=paam-skills
  → alias = "github.com/simpletang/paam-skills"
```

本地缓存目录与 alias 一一对应：
```
~/.paam/sources/github.com/simpletang/paam-skills/   ← clone 落地于此
```

**Why：**
- 三段命名能跨主机唯一区分（GitHub / 自托管 GitLab 同名仓不冲突）
- 直接复用为目录路径（含 `/`），路径即标识，不需要额外索引
- 小写化避免 macOS 默认 case-insensitive 文件系统的二义性
- M1 限定一个仓库只允许 track 一次（M1-plan §三 3.2 "M1 不处理同名冲突，要求源唯一"），同 alias 重复 track 直接报错

**Alternatives considered:**
- `<owner>/<repo>` 二段：拒绝。跨主机会冲突
- `<repo>` 单段：拒绝。冲突风险太大
- 用户 `--as <alias>` 显式指定：拒绝。增加认知负担，M1 不需要
- 内容哈希前缀：拒绝，过度设计

### 4. SSH URL 解析：自实现的最小 parser

**选择：** 在 `paam-core::source` 内实现一个仅识别两种 SSH URL 形式的解析器：
- SCP-like：`git@host:owner/repo.git` 或 `git@host:owner/repo`
- ssh://：`ssh://[user@]host[:port]/owner/repo[.git]`

任何其他形式（HTTPS、`owner/repo` 简写、本地路径）一律拒绝，错误信息明确指向 M2。

**Why：**
- 不引入 `git-url-parse` 依赖：它支持太多形式，反而需要写白名单过滤；自写 50 行更可控
- 错误优先：M1 限定明确，越早 reject 非法 URL 越好

### 5. SSH 鉴权：仅 ssh-agent

**选择：** git2 的 `RemoteCallbacks::credentials` 回调中，仅使用 `Cred::ssh_key_from_agent(username)`。未启动 ssh-agent / agent 中无可用 key 时，错误信息明确提示 `ssh-add ~/.ssh/id_*`。

**Why：**
- 直读私钥文件需要处理密码短语 → 需要交互输入，CLI 流程复杂化
- ssh-agent 是开发者机器的事实标准
- M2 再考虑直读 key 文件 / 配置自定义 key path

**Risk：** 用户机器可能未启动 ssh-agent → 缓解：错误消息要给出准确的修复指令；在 README / `paam --help` 中点明前置条件。

### 6. 同步 vs 异步：M1 全同步

**选择：** `paam-cli` 的 `main()` 不加 `#[tokio::main]`；git 操作在主线程同步执行。`tokio` 依赖暂保留在 workspace 但 paam-cli 不实际启用。

**Why：**
- M1 每条命令只做一件事，无并发收益
- git2 是阻塞 FFI；包装到 spawn_blocking 只是徒增样板
- M2 引入 `paam sync` 多源并发时再加 tokio runtime；此时增量改动小

**Risk：** 后续 ④ change 加 sync 时需要在 paam-cli 引入 runtime → 缓解：届时只是改 `main` 上的属性宏，不影响 paam-core 的 API（core 始终保持 sync API）。

### 7. paam-core 模块边界

```
paam-core/src/
├── lib.rs              ← 仅 pub use 顶层 API + 错误类型
├── error.rs            ← Error enum (thiserror)
├── paths.rs            ← ~/.paam/ 路径常量与初始化
├── config/
│   ├── mod.rs          ← pub fn load() / save() / add_source() / list_sources()
│   ├── schema.rs       ← Config struct + Source struct (serde)
│   └── tests.rs
├── git/
│   ├── mod.rs          ← pub fn clone(url, dest) -> Result<()>
│   └── auth.rs         ← ssh-agent 回调
├── source/
│   ├── mod.rs          ← pub fn track(url) / list_sources()
│   ├── url.rs          ← SSH URL parser → SourceLocator { host, owner, repo }
│   └── tests.rs
└── asset/
    ├── mod.rs          ← Asset trait + AssetKind enum（无实现者）
    └── tests.rs        ← 仅类型层 doctest / 编译期断言
```

**Why：**
- `paths` 单独抽出：所有 change 都要用，是契约层
- `config` 不暴露内部 struct，对外只给业务级 API（`add_source`、`list_sources`），将来加 `installed_assets` 字段时调用方零改动
- `source::track` 是业务编排：URL parse → clone → 调用 config::add_source；CLI 直接调它
- `error.rs` 顶层统一错误类型，CLI 层只 match 一次
- `asset` 与 `source` 平级而非嵌套：`source` 关心"订阅源"层（git 仓），`asset` 关心"被分发的资产"层（Skill / Prompt / Mcp）—— 两者语义不同；详见决策 10

### 8. paam-cli 子命令骨架

```rust
#[derive(Subcommand)]
enum Cmd {
    Track(TrackArgs),     // 含子子命令: <url> | list
    // 占位（不实现，clap subcommand 暂不注册）：
    // Install, Sync, List, Uninstall, Info
}
```

**Why：** 不实现的子命令暂不注册，避免误导用户 `--help` 时看到一堆 unimplemented。后续 change 各自加自己的。

### 9. 测试策略

- **单测**（在 `paam-core` 各模块内 `#[cfg(test)]`）：
  - URL parser：列举 SSH 合法 / 非法形式
  - alias 推导：大小写、`.git` 后缀、端口号
  - config 读写：用 `tempfile` 在临时目录读写 JSON
- **git 模块测试**：用 `git2` 在 tempdir 创建 bare repo + 1 个 commit 作为 fixture，测试 `clone` 能拉到内容；不走网络
- **CLI 测试**：M1 DoD 允许手动验收剧本，本 change 不写 `assert_cmd` 集成测试
- **覆盖率**：M1 不强制（M1-plan §二）

### 10. 提前引入 `Asset` trait 骨架（无实现者）

**选择：** 本 change 在 `paam-core::asset` 引入下面这套 type-agnostic 骨架，但**不写任何实现者**：

```rust
pub trait Asset {
    fn id(&self) -> &str;            // 在所属 source 内唯一
    fn kind(&self) -> AssetKind;
    fn source_alias(&self) -> &str;  // 来自哪个订阅源
    fn relative_path(&self) -> &Path; // 在源仓中的相对路径
}

#[non_exhaustive]
pub enum AssetKind {
    Skill,
    // Prompt / Mcp 在 M2 加入；#[non_exhaustive] 允许新增 variant 不破坏调用方
}
```

**Why（为什么提前引入而不是推到 ②）：**
- ADR-0007 钦定 type-agnostic，骨架越早立越好；推到 ② 时 paam-core 内部数据流（`source::track` → ② 的 discovery）的 API 形状会受 trait 反向影响——届时改本 change 的代码成本不为零
- trait 本身只有四个方法签名 + 一个枚举，**代码量 < 30 行**，提前引入的成本极低
- `#[non_exhaustive]` 给 `AssetKind` 留 M2 扩展空间（加 `Prompt` / `Mcp` 不构成破坏性变更）
- 提前定下 trait 也能在 ② design 阶段消除"trait 形状"这一不确定性

**Why 不写实现者：**
- 本 change 不扫描 SKILL.md、不安装、不查询 asset，**没有调用者**
- 写一个无调用者的 `Skill struct` 是死代码，反而要在 ② 重构掉
- 单测层面也无法覆盖（没有业务流程驱动它）

**Risk：** trait 签名在 ② 写实现者时被发现需要调整 → 缓解：四个方法都是最小、明显必要的（id / kind / 来源 / 路径），ADR-0007 已经讨论过 type-agnostic 的语义；如真的要改，本 change 的代码量小，重构成本可控。

**契约固化：** trait 与枚举一旦合并，在 ② 之前的任何修改都需要新开 change（不允许在 apply 阶段顺手改）。

**Spec 边界：** 本 change 不为 `asset-management` 写 capability spec——trait 是类型层定义，没有可观察的运行时行为可以用 WHEN/THEN scenario 描述。② paam-skill-discovery 引入第一个调用者（SKILL.md 扫描）时，再正式新建 `asset-management` capability spec。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| Asset trait 签名在 ② 写实现者时被发现需要调整 | trait 仅四个最小方法；如需调整再开 change，本 change 代码量可控（< 30 行） |
| 自写 SSH URL parser 漏掉合法形式 | 单测覆盖 §四 决策 4 列出的两种形式；非法形式宁可误拒，错误信息要求"换成 git@host:owner/repo.git 形式" |
| **libssh2 不读 `~/.ssh/config`，与系统 `git` 行为不一致**（首跑实测发现：用户在 config 里把 github.com 重定向到 ssh.github.com:443 时 paam 直接 banner 失败） | 错误分类细化：`code=Auth` → `SshAgentUnavailable`、其余 `class=Ssh` → `SshTransport`，后者文案明确指出 libssh2 不读 .ssh/config 并给出展开 URL workaround；`--verbose` 暴露 git2 原始错误。正式支持 `.ssh/config`（或 fork `git` CLI 子进程）推迟到 M2/M3 评估 |
| ssh-agent 未启动导致 clone 失败 | 错误消息明确给出 `ssh-add` 指令；不试图回退到读 key 文件 |
| alias 包含 `/`，跨平台路径处理可能踩坑 | M1 仅 macOS（PRODUCT.md），`/` 在 POSIX 是合法分隔符；Windows 适配在 M4 处理 |
| 重复 track 同一个仓如何报错 | 解决策略：检测到 alias 已存在直接 reject，错误消息提示用 `paam untrack`（M2 实现，M1 提示用户手动 `rm -rf`） |
| OpenSpec 工作流首跑，spec / design 边界拿捏不准 | 本 change 文档刻意写得偏详细，作为后续 change 的样板；M1-retro 复盘时再精简 |

## Migration Plan

不涉及（首个 change，无既有数据 / API）。

后续 change 对配置文件 schema 的扩展必须保持 v1 向后兼容。如需破坏性变更，需要：
1. bump `version` 字段
2. 在 `config::load()` 内增加 `match version` 的迁移分支
3. 在 design.md 显式记录迁移逻辑

## Open Questions

1. **alias 的小写化是否会让用户困惑？** —— 用户输入 `git@GitHub.com:Foo/Bar.git`，alias 显示为 `github.com/foo/bar`。一致性强，但与原仓库展示形式不符。当前决定接受这一权衡，M1-retro 时复盘是否回滚。
2. **`paam track <url>` 成功时的输出格式** —— 是否打印 alias、本地路径、远程默认分支？建议至少打印 alias 与路径，便于用户后续 `cd` 进去看。具体措辞在 tasks 阶段决定。
3. **是否在 `track list` 中显示最后一次 fetch 的时间？** —— M1 不实现 `paam update`，"最后 fetch" 等于"track 时刻"。当前决定 M1 只显示 `added_at`，不显示 fetch 时间，避免误导。
