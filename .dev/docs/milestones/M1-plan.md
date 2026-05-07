# Milestone M1 — Plan & PRD

> **Version**: `v0.1.0`
> **Theme**: 技术原型（CLI 核心）
> **Status**: ✅ Done（2026-05-07）
> **Created**: 2026-04-25
> **Started**: 2026-04-28
> **Completed**: 2026-05-07
>
> 本文档同时承担 M1 的**项目计划**与**产品需求文档（PRD）**两种角色。
> M1 已完成；复盘见 [`M1-retro.md`](./M1-retro.md)。
> 长期产品定位与永久边界请参考根目录 [`PRODUCT.md`](../../../PRODUCT.md)。

---

## 一、本期目标（What & Why）

完成 paam 的 **CLI 核心最小可用版本**：能够订阅一个 git 仓库、安装其中的 Skill、同步到 Claude Code 目录、列出已安装 Skill。

**核心目的是验证整体架构可行性**，而不是覆盖完整功能。

## 二、出口标准（Definition of Done）

M1 完成必须满足以下全部条件：

- [x] 下文 §三 列出的所有 P0 功能全部实现
- [x] CI 在 macOS 上跑通 lint / build（`.github/workflows/ci.yml`）
- [x] 端到端验收剧本通过（沙盒版；命令名按修订后混合命名空间）：
  ```bash
  paam track <some-public-skills-repo>
  paam track skills <alias>          # 浏览仓内可装的 skill
  paam skill install <skill-name>
  paam sync
  ls ~/.claude/skills/                # 看到 symlink
  paam list                           # 看到已安装条目（type-agnostic 全量）
  paam skill list                     # 仅看 skill
  ```
- [x] 关键路径有单元测试（76 个单测，含 URL 解析 / 配置读写 / discover / git helper / install / sync / metadata round-trip 等）；覆盖率不强制
- [x] CHANGELOG.md 记录 v0.1.0 全部条目
- [ ] tag `v0.1.0` 已推送（待用户在本仓 `git tag v0.1.0 && git push origin v0.1.0`）
- [x] M1-retro.md 已写（[`M1-retro.md`](./M1-retro.md)）

> **原型阶段验收标准刻意降低**：M1 是技术原型，验证架构可行性优先于代码质量。
> 严格的覆盖率 / 测试矩阵推迟到 M2-M5。

---

## 三、本期功能需求（PRD）

### 3.1 模块 M1-A：订阅源管理

订阅一个或多个 Git 仓库作为 Skills 来源。

| 功能 | CLI 命令 | 优先级 |
|---|---|---|
| 添加订阅仓（SSH） | `paam track <git-url>` | P0 |
| 列出已订阅仓 | `paam track list` | P0 |
| 移除订阅仓（含本地缓存清理） | `paam untrack <alias>` | **P1（可推迟到 M2）** |
| 浏览仓内 Skills（不安装） | `paam track skills <alias>` | **P1（可推迟到 M2）** |

**M1 限定的支持范围**：
- ✅ 任意自托管 Git（GitLab / Gitea / 自建）via SSH
- ✅ GitHub 公共仓 via SSH
- ❌ HTTPS / PAT 鉴权（M2）
- ❌ owner/repo 简写（M2）
- ❌ 自签 SSL 证书配置（M2）

### 3.2 模块 M1-B：本地 Skills 管理

> 命令名按 ADR-0007 §6（2026-04-29 修订）的混合命名空间策略：资产 CRUD 用 `paam <type> <verb>`。

| 功能 | CLI 命令 | 优先级 |
|---|---|---|
| 安装单个 Skill | `paam skill install <name>` | P0 |
| 卸载 Skill | `paam skill uninstall <name>` | **P1（在 paam-skill-install-and-list 顺手做）** |
| 启用 / 禁用 | `paam skill enable/disable <name>` | **P2（推迟到 M2）** |
| Pin 到特定版本 | `paam skill pin <name> <ref>` | **P2（推迟到 M2）** |
| 跨 source 同名消歧 | `--from <alias>` | P0（在 paam-skill-install-and-list 落地） |

### 3.3 模块 M1-C：Agent 分发（仅 Claude Code）

| 功能 | CLI 命令 | 优先级 |
|---|---|---|
| 一键全量同步到 Claude Code | `paam sync` | P0 |
| symlink 模式（macOS 默认） | 自动 | P0 |
| copy 模式（备用） | `--mode copy` | **P1** |
| 自动检测已安装的 Agent | `paam target detect` | **P2（M2）** |
| 多 target 支持（Cursor / Codex） | — | **M1 不做（M2）** |

**M1 target 路径**：`~/.claude/skills/`（硬编码，M2 改为可配置）

### 3.4 模块 M1-D：状态查询

> 全量概览保持 type-agnostic；单类型聚焦走 `paam <type> list`。

| 功能 | CLI 命令 | 优先级 |
|---|---|---|
| 列出所有已安装资产（type-agnostic 全量） | `paam list` | P0 |
| 仅列已装 Skills | `paam skill list` | P0 |
| 查看单个 Skill 详情 | `paam skill info <name>` | **P1（推迟到 M2）** |
| 跨类型搜索 | `paam search <kw>` | **P2（M2）** |

### 3.5 配置文件

- 必须有：用户级 `paam.yaml`（订阅仓列表 + 同步配置）
- 必须有：本地 source 元数据 `.metadata.json`（已安装 Skill 列表 + provenance）
- M1 限定：不预留 `prompts:` / `mcp:` 字段（简化原型实现，由 ADR-0007 决策）

---

## 四、明确不做的事（M1 边界）

> 永久边界请见 [`PRODUCT.md`](../../../PRODUCT.md) §四，本节仅为 M1 内推迟到后续 milestone 的能力。

- ❌ HTTPS / PAT 鉴权（M2）
- ❌ Cursor / Codex target（M2）
- ❌ Windows 适配（M4）
- ❌ `paam publish`（M2）
- ❌ `paam update`（M2）
- ❌ 桌面 UI（M3）
- ❌ 同名 Skill 冲突解决（M2）
- ❌ Pin 到特定版本（M2）
- ❌ disable / enable（M2）
- ❌ Dry-run（M2）

---

## 五、依赖与前置决策

**已 Accept 的阻塞 ADR**（M1 可启动）：

- [x] **ADR-0001** 技术栈：**Tauri v2 + Rust**（M3 引入 Tauri；M1 仅 Rust CLI）
- [x] **ADR-0002** 核心库共享：**Cargo workspace**（`paam-core` + `paam-cli`，M3 加 `paam-app`）
- [x] **ADR-0007** 数据架构：`~/.paam/` 工作目录、source/local-repo/target 三层模型、type-agnostic 命令、`Asset` trait 抽象
- [x] **ADR-0008** 执行层工具：OpenSpec v1.3.1（slash commands `/opsx:propose | apply | archive | explore`）

**可推迟的 ADR**：

- ADR-0003 开源协议 — v0.1.0 发布前必须有
- ADR-0004 配置兼容性（与 skillshare 等的迁移）— M2 之前决
- ADR-0005 发布渠道与签名 — M5 之前决
- ADR-0006 品牌视觉 — M3 启动前决

## 五·B、技术栈速查（M1 实现指南）

**仓库结构**（按 ADR-0002）：

```
paam/
├── Cargo.toml                     ← workspace
├── crates/
│   ├── paam-core/                 ← 业务逻辑 lib
│   │   └── src/
│   │       ├── config/
│   │       ├── git/
│   │       ├── source/
│   │       ├── local_repo/
│   │       ├── sync/
│   │       ├── metadata/
│   │       └── asset/
│   └── paam-cli/                  ← CLI 入口 bin
│       └── src/main.rs
└── (paam-app 在 M3 引入)
```

**M1 关键 crate 依赖**：

| 用途 | crate |
|---|---|
| CLI | `clap` (derive) |
| Git | `git2` |
| 序列化 | `serde` + `serde_json` |
| 异步 | `tokio` |
| 路径 | `directories-next` |
| 错误 | `thiserror` |
| 日志 | `tracing` |

**实现顺序建议**（按 paam-core 模块）：

1. `config` 模块（config.json 读写）
2. `git` 模块（git2-rs 封装：clone / fetch / commit / tree-hash）
3. `source` 模块（track / 扫描 SKILL.md）
4. `local_repo` 模块（install copy + auto git commit）
5. `sync` 模块（创建 symlink + 同名冲突检测）
6. `metadata` 模块（.metadata.json 读写）
7. `asset` 模块（trait + Skill 实现）
8. `paam-cli` 各子命令

---

## 六、已知风险

| 风险 | 影响 | 缓解 |
|---|---|---|
| 选 Rust（Tauri）但学习曲线超出预期 | 整体进度延后 | AI 辅助 + 必要时降级到熟悉语言 |
| git 操作的边界情况复杂（auth、网络、子模块） | 范围蔓延 | 严格限定 M1 仅支持 SSH + 公共仓 + 简单结构 |
| 配置文件 schema 设计不当 | 后续 milestone 需要破坏性变更 | 先完成 ADR-0007，再开工 |
| symlink 在某些 macOS 配置下行为异常 | 同步失败 | 测试覆盖 APFS / iCloud 同步目录等场景 |
| 第一次跑 OpenSpec 工作流不熟练 | 早期 feature 周期慢 | 第一个 feature（建议是 paam-track）当作工作流试运行 |

---

## 七、Build 阶段进度日志

> 在开发期间按需追加条目，记录关键进展、决策、坑。

- **2026-04-28**：完成 paam-foundation-track（OpenSpec 工作流首跑）。落地 `paam track` / `paam track list`、工作目录契约（`~/.paam/`，`PAAM_HOME` 可覆盖）、JSON 配置文件、SSH URL 解析、git2 clone（仅 ssh-agent 鉴权）、`Asset` trait 骨架（无实现者）。20 个单测通过，clippy / fmt 干净。两条 deferred decision 在 design.md 记录：alias 大小写小写化、`paam track` 成功输出格式（两行）。剩余的 SSH 真实端到端验收（task 10.5 / 10.6）需用户用自己的 ssh-agent 与真实仓手动跑。
- **2026-04-28（坑记录）**：手动验收时遇到 `Failed getting banner` —— 用户 `~/.ssh/config` 把 `github.com` 重定向到 `ssh.github.com:443`（绕开国内 ISP 对 22 端口的拦截），但 **libssh2 不读 `~/.ssh/config`**，硬连 `github.com:22` → SSH 握手失败。因此重新拆分错误分类：仅当 `code=Auth` 时才映射 `SshAgentUnavailable`，其余 `class=Ssh` 错误归为新变体 `SshTransport`，文案点明 libssh2 不读 `.ssh/config` 并给出展开 URL workaround（如 `ssh://git@ssh.github.com:443/owner/repo.git`）。同时给 `git::map_git_error` 加 `tracing::debug!`，`--verbose` 模式可看到 git2 原始 class/code/message。这是 M1 已知限制，design.md Risks 表已新增条目；正式支持 `.ssh/config` 推迟到 M2/M3 评估。
- **2026-05-07**：完成 `paam-claude-sync`（A' 切法 ④，**M1 收官**）。新增 `paam-core::sync` 模块（顶级，与 discover / install / metadata / local_repo 平行）：`sync_all` / `unsync_one` / `unsync_one_no_commit` / `unsync_all` + `SyncReport` / `Conflict` 数据结构。CLI 新增 `paam sync [--force]` / `paam unsync [<name>] [--all]`。冲突策略：默认跳过非 paam 管理的 target 内容 + warning + 加入 conflicts；`--force` 才覆盖。`paam skill uninstall` 自动清理 target symlink（智能耦合：install → sync 单向依赖，无循环），通过 `unsync_one_no_commit` helper 让"卸载 X" 单 commit capture 所有变更。target 路径硬编码 `~/.claude/skills/`，环境变量 `PAAM_CLAUDE_TARGET_DIR` 可覆盖（用于测试）。**dogfood 修了一个安全 bug**：原 classify 把"metadata 命中 + fs 是真实目录"视为 PaamLinkBroken（删除重建），实测会**静默覆盖用户后续放在该路径的内容**；改为视为 ForeignContent，要求 `--force` 才覆盖。76 单测通过、clippy / fmt 干净。**M1 端到端剧本闭合**：track → install → sync → list 全链路工作；用户在 Claude Code 中能真实用上 paam 装的 skill。
- **2026-04-29**：完成 `paam-skill-install-and-list`（A' 切法 ③，本 milestone 最大块）。落地三类新模块：`local_repo`（paam 自管 git 仓，独立身份 `paam@local`）、`metadata`（每资产一份 `.metadata.json` 的 schema 与读写）、`install`（resolve + install + uninstall 业务编排）。CLI 大改：新增 `paam skill install/list/uninstall` 与 `paam list`（type-agnostic 全量）；保留 `paam track ...` 不变。**修订 ADR-0007 §6** 把 CLI 命名空间从 type-agnostic 改为混合策略——资产 CRUD 用 `paam <type> <verb>` type-prefix（M1 仅 skill），跨类型操作（track / sync / 全量 list）保持 type-agnostic；同步修订 M1-plan §三 3.2 / 3.4 命令名。`git` 模块新加 `head_commit` / `subtree_hash` helper（M2 paam update 必备）。dogfood 全过：14 个手动剧本（含 AmbiguousSkill 候选列表、--from 消歧、--force 重装、SkillNotFound、NotInstalled、local-repo 中文 commit history 等）；64 个单测通过。本 change 实施过程中决定将"用户提议的命名空间调整"作为整改的一部分一并完成（避免 ② / ③ archive 完毕后再开 change 折腾）；ADR §6 修订段与 §7 修订段并列，标志 ADR-0007 在 dogfood 阶段的两次重要调整。
- **2026-04-29**：完成 `paam-skill-discovery`（A' 切法 ②）。引入 `Skill` 类型（首个 `Asset` trait 实现者）、`discover::skills_in` 递归扫描、SKILL.md frontmatter（YAML）解析，CLI 新增 `paam track skills <alias>`。**修订 ADR-0007 §7**：trait 形态定型为"4 getter（object-safe）+ 模块级动作函数"，原 trait 草案标 "Superseded by below"。`config.json` 新增可选字段 `scan_ignore`（完全替换语义，不合并）；内置默认含 12 个常见噪音目录。frontmatter 解析采取宽松策略：未知字段透明保留到 `extra` + tracing::warn!；缺必填 / YAML 错误时跳过该目录的 SKILL.md（不影响其它）+ stderr warning。新增依赖 `serde_yaml_ng`（仅用于 SKILL.md，paam 自己的 config 仍 JSON）。43 个单测通过，clippy / fmt 干净；dogfood 覆盖了所有 spec scenarios（合法 / 缺必填跳过 / YAML 错误跳过 / 未知字段保留 + warn / 默认 ignore 生效 / scan_ignore 替换 / 空数组不忽略 / alias 不存在）。
- **2026-04-28（架构修正）**：完成 `swap-git-transport-to-cli`。原因是上面那条坑：libssh2 与 OpenSSH 的差异是结构性的，每加一项 fallback 都是在重新发明 OpenSSH 的一部分（接下来还会撞到 Keychain、ProxyCommand、known_hosts 等长尾）。因此选择**减法**：彻底删除 `git2-rs` 依赖，`git::clone` 改为 fork 系统 `git` CLI 子进程；后续所有 git 操作（含 ② / ③ 的本地仓 commit）也走子进程；测试 fixture 改用 system git 构造（`init --bare` + `commit --allow-empty` + `push file://`）；新增 `Error::GitNotFound` + `Error::GitProcessFailure`；删除 `git/auth.rs` 与 4 个 SSH 错误变体；新增 `git::ensure_git_available()` 在远程操作入口探测。**收益**：release binary `target/release/paam` 从约 8 MB（含 libgit2/libssh2/openssl 静态链接）缩小到 **2.0 MB**；编译时间显著下降（不再编译 libgit2-sys / libssh2-sys / openssl-sys / libz-sys）。**dogfood 验证**：之前 banner 失败的 `paam track git@github.com:SimpleTang/AndroidBaseArchitecture.git` 现在直接成功，**且无需 `ssh-add`**——系统 git 通过 `~/.ssh/config` 重定向到 `ssh.github.com:443` + Keychain 缓存 passphrase 完成鉴权。22 个单测通过、clippy / fmt 干净、`cargo tree | grep git2` 无输出。

---

## 八、引用

- 长期产品宪章：[`PRODUCT.md`](../../../PRODUCT.md)
- ADR 索引：[`../decisions/README.md`](../decisions/README.md)
- 流程规范：[`../PROCESS.md`](../PROCESS.md)
- M1 复盘：[`M1-retro.md`](./M1-retro.md)
- v0.1 设计快照（含原 PRD 全部细节）：[`../archived/PRD-v0.1-design-snapshot.md`](../archived/PRD-v0.1-design-snapshot.md)
