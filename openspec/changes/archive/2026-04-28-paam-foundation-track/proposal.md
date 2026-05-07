## Why

paam 当前只有一个空的 Cargo workspace 与 hello-world 级 CLI 骨架，没有任何用户可感知的能力。M1 的 DoD 要求端到端跑通 `track → install → sync → list` 四步剧本，第一步必须先有"能加一个订阅源、能列出来"的最小闭环——它同时也是本仓 OpenSpec 工作流的首跑试运行（M1-plan §六风险表已点名 `paam-track` 充当试金石）。

完成本 change 后，用户能在干净环境上完成 "`paam track <ssh-url>` → `paam track list` 看到一行" 的端到端流程，后续 install / sync / list 三个 change 都可以建立在这一层之上。

## What Changes

- 引入 paam-cli 的 clap derive 子命令分发框架，预留（但不实现）`install` / `sync` / `list` / `uninstall` 等位置
- 引入 `~/.paam/` 工作目录约定：首次运行时自动创建，路径解析交由 `directories-next`
- 引入 paam 的用户级配置文件 `~/.paam/config.json`（JSON 格式，subscriptions 列表）：定义 schema、序列化格式、读写 API
- 实现 `paam track <git-url>` 子命令：解析 SSH URL → 推导 alias → `git clone` 到本地缓存目录 → 写回配置文件
- 实现 `paam track list` 子命令：从配置文件读取并打印订阅源清单
- 在 paam-core 中落地 `config` 模块（用户配置读写）与 `git` 模块（git2-rs 的最小封装：仅 clone）
- 引入 `Asset` trait + `AssetKind` enum 的 type-agnostic 抽象骨架（ADR-0007 落地）：本 change 仅定义 trait 与枚举，不写实现者；后续 ② paam-skill-discovery 提供 `Skill` 实现
- 关键路径单测：git URL → alias 解析、配置文件读写

**明确不做（推迟到后续 change / milestone）：**
- `paam untrack`、`paam track skills <alias>` —— 推到 ② paam-skill-discovery
- HTTPS / PAT 鉴权、owner/repo 简写、自签 SSL —— M2
- `Skill` 具体实现 / SKILL.md 扫描 —— 推到 ② paam-skill-discovery
- 实际的 install / sync / list 行为 —— 推到 ③④

## Capabilities

### New Capabilities

- `source-management`: 订阅源（远程 git 仓库）的添加、列表、本地缓存约定，以及与之配套的用户配置文件契约

### Modified Capabilities

（无，本 change 为本仓首个 OpenSpec change，`openspec/specs/` 当前为空）

## Impact

**代码：**
- `crates/paam-core/src/lib.rs`: 注册 `error`、`paths`、`config`、`git`、`source`、`asset` 六个模块
- `crates/paam-core/src/config/`: 新增（用户配置文件 schema + 读写 + 默认值，JSON 格式）
- `crates/paam-core/src/git/`: 新增（git2 clone 封装、错误映射、ssh-agent 鉴权回调）
- `crates/paam-core/src/source/`: 新增（SSH URL 解析、alias / 本地缓存目录布局、track 业务编排）
- `crates/paam-core/src/asset/`: 新增（`Asset` trait + `AssetKind` enum；本 change 不含实现者）
- `crates/paam-cli/src/main.rs`: 改造为子命令分发；新增 `track` / `track list` 子命令处理

**依赖：**
- 启用 workspace 中已声明的 `git2`、`serde`、`serde_json`、`directories-next`、`thiserror`、`tracing`
- 新增 `chrono`（带 `serde` feature，用于 `added_at` 时间戳序列化）
- 新增 dev-dep `tempfile`（单测使用）
- 不引入 YAML 依赖（配置格式选 JSON，见 design.md 决策 1）
- `paam-cli` 移除 `tokio`（M1 全同步，决策 6）

**文件系统副作用：**
- 首次运行 paam 时在用户主目录下创建 `~/.paam/`（含 `sources/` 子目录与配置文件）
- `paam track` 会向 `~/.paam/sources/<alias>/` 写入完整 git 仓克隆

**外部系统：**
- 通过用户的 SSH agent 与 SSH key 访问远程 git 仓（用户自托管 GitLab / Gitea / GitHub 公共仓）；不引入新的鉴权链路

**对后续 change 的契约：**
- `~/.paam/` 路径常量、配置文件 schema、本地缓存仓目录布局，将被后续 ② / ③ / ④ change 直接复用，需在 design.md 中作为契约层显式定义
