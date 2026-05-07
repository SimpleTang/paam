# Changelog

本项目所有重要变更记录于此。

格式遵循 [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

（暂无）

## [0.1.0] - 2026-05-07

M1 技术原型 —— CLI 核心最小可用版本。端到端剧本闭合：用户能订阅一个 SSH git 仓，浏览仓内 skill，安装到本地工作集，同步到 Claude Code 的 `~/.claude/skills/` 让 Agent 立刻能用。

### Added

**项目基础**

- 集成 [OpenSpec](https://github.com/Fission-AI/OpenSpec) v1.3.1 作为执行层 spec 工具（slash commands `/opsx:propose | apply | archive | explore`）
- 确定技术栈：**Tauri v2 + Rust**（详见 ADR-0001）；M1 仅 Rust CLI 部分落地
- 确定代码组织：**Cargo workspace**（`paam-core` + `paam-cli`，M3 加 `paam-app`；详见 ADR-0002）
- 确定开源协议：**MIT License**（详见 ADR-0003，LICENSE 文件已创建）
- 确定数据架构（详见 ADR-0007）：工作目录 `~/.paam/`、三层资产流转 source → local-repo → target、local-repo 按类型分组、type-agnostic CLI 命令
- 创建 Cargo workspace 骨架：`paam-core` (lib) + `paam-cli` (bin `paam`)
- 新增 GitHub Actions CI（macOS：`cargo fmt --check` / `clippy -D warnings` / `test` / `build --release` + git2 残留校验）

**CLI 命令（按 ADR-0007 §6 修订版混合命名空间策略）**

- `paam track <ssh-url>`：订阅一个 SSH git 仓
- `paam track list`：列出已订阅源
- `paam track skills <alias>`：浏览仓内可用 skill（不安装）
- `paam skill install <name> [--from <alias>] [--force]`：安装 skill 到本地工作集；跨 source 同名用 `--from` 消歧；已装时 `--force` 重装
- `paam skill list`：列出已装 skill
- `paam skill uninstall <name>`：卸载已装 skill；自动清理已 sync 的 target symlink
- `paam list`：列出所有已装资产（type-agnostic 全量，含 TYPE 列）
- `paam sync [--force]`：把已装 skill 通过 symlink 暴露到 `~/.claude/skills/`；冲突默认跳过 + warning，`--force` 覆盖
- `paam unsync <name>` / `paam unsync --all`：仅删 target symlink + 清 metadata.targets[]，不动 local-repo

**架构与运行时**

- 工作目录 `~/.paam/`（含 `sources/` / `local-repo/` / `config.json`），首次运行时自动初始化；`PAAM_HOME` 环境变量可覆盖（测试与自定义部署）
- target 路径默认 `~/.claude/skills/`；`PAAM_CLAUDE_TARGET_DIR` 环境变量可覆盖
- 用户配置 `~/.paam/config.json`（JSON 格式，schema v1）：含 `version` / `sources` / 可选 `scan_ignore`（覆盖默认扫描忽略目录列表）
- local-repo 是 paam 自管 git 仓，独立身份 `paam@local`（不读 `~/.gitconfig`）；每次 install / sync / uninstall 自动 commit，message 中文（如 `安装 X，来自 Y@abc1234` / `同步 N 个 skill 到 claude-code` / `卸载 X`）
- 每个已装资产带独立 `.metadata.json`（沿用 ADR-0007 §5 schema：`name` / `type` / `origin{kind,repo,subpath,commit,tree_hash}` / `installed_at` / `targets[]` / `version`）
- `Asset` trait + `AssetKind` 枚举（object-safe，4 getter；动作下放到模块函数；ADR-0007 §7 修订版）
- 所有 git 操作走系统 `git` CLI 子进程（不依赖 git2-rs / libgit2 / libssh2）；release binary **2.4 MB**

**对运行环境的依赖**

- 系统已安装 `git`（PATH 上可访问）
- macOS 12+（M1 范围）；其它 Unix 平台未测试

### Changed during M1（reflected here for transparency）

- 修订 **ADR-0007 §6**：CLI 命名空间从 type-agnostic 改为混合策略（资产 CRUD 用 `paam <type> <verb>`；track / sync / 全量 list 仍 type-agnostic）；详见 ADR 修订段
- 修订 **ADR-0007 §7**：`Asset` trait 形态从草案的 `name() / asset_type() / marker_file() / discover_in() / install_to()` 改为 4 getter（object-safe）+ 模块级动作函数

### Known limitations / explicitly out of M1 scope

- **仅 macOS**（Linux / Windows 未测试，详见 PRODUCT.md）
- **仅 SSH git URL**（HTTPS / PAT / owner/repo 简写推到 M2）
- **仅 Claude Code 单 target**（Cursor / Codex 多 target 推到 M2）
- **仅 Skill 单类型**（Prompt / MCP 推到 M2 / M3）
- **仅 symlink 同步模式**（`--mode copy` 推到 M2）
- 不支持 `paam skill info` / `paam skill enable/disable/pin` / `paam skill update` / `paam search` / `paam publish` / `paam target detect` / dry-run（M2+）
- 桌面 UI 推到 M3（Tauri）

### Internal notes

- M1 期间共完成 5 个 OpenSpec change：① paam-foundation-track / ⓥ swap-git-transport-to-cli / ② paam-skill-discovery / ③ paam-skill-install-and-list / ④ paam-claude-sync
- 76 个单测，覆盖关键路径（URL 解析 / 配置读写 / discover / install / sync / metadata round-trip 等）
- 完整复盘见 `.dev/docs/milestones/M1-retro.md`
