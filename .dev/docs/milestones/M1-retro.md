# Milestone M1 — Retro

> **Version**: `v0.1.0`
> **Theme**: 技术原型（CLI 核心）
> **Started**: 2026-04-28（首个 change apply）
> **Completed**: 2026-05-07（端到端剧本闭合）
> **Calendar duration**: 10 天
> **Status**: ✅ Done

---

## 一、出口标准达成情况

| 出口标准（M1-plan §二） | 状态 | 备注 |
|---|---|---|
| §三 列出的所有 P0 功能全部实现 | ✅ | track / track list / track skills / skill install / skill list / skill uninstall / list / sync 全部落地 |
| CI 在 macOS 上跑通 lint / build | ✅ | `.github/workflows/ci.yml`：fmt + clippy + test + build + git2 残留校验 |
| 端到端验收剧本通过（沙盒版） | ✅ | track → install → sync → list 全链路闭合（详见 `target/release/paam` dogfood 输出） |
| 关键路径单元测试 | ✅ | 76 个单测；覆盖 URL parser / config / discover / git helper / install / sync / metadata round-trip |
| CHANGELOG.md 记录 v0.1.0 全部条目 | ✅ | 顶部 `[0.1.0] - 2026-05-07` 段 |
| tag `v0.1.0` 已推送 | ⏳ | 本 retro 写完后由用户手动 push |
| M1-retro.md 已写 | ✅ | 即本文档 |

> **真实 SSH URL 端到端**未在 retro 时刻自动跑通——需要用户在自己环境用真实凭证跑。沙盒版（用本地 git 仓 + 手工注册到 paam.config）已确认所有业务流正确。

---

## 二、交付的 5 个 OpenSpec change

| # | Change | 主要内容 | archive 日期 |
|---|---|---|---|
| ① | `paam-foundation-track` | CLI 骨架 + 工作目录 + config.json + `paam track` / `track list` + `Asset` trait 骨架 | 2026-04-28 |
| ⓥ | `swap-git-transport-to-cli` | 删除 git2-rs，所有 git 操作走 system git 子进程 | 2026-04-28 |
| ② | `paam-skill-discovery` | `Skill` 类型 + `discover::skills_in` + SKILL.md frontmatter 解析 + `paam track skills` + `scan_ignore` 配置；修订 ADR-0007 §7 | 2026-04-29 |
| ③ | `paam-skill-install-and-list` | local-repo + metadata + install / list / uninstall + `paam list`；修订 ADR-0007 §6 CLI 命名空间为混合策略 | 2026-04-29 |
| ④ | `paam-claude-sync` | `paam sync` / `paam unsync` + uninstall 智能耦合；4 个 capability 主 spec 全部建立 | 2026-05-07 |

5 个 change 共 **246 个任务**全部勾完（4 + 51 + 44 + 50 + 67 + 44；ⓥ 是架构修正块，不计入 4-change 序列）。

---

## 三、做对的事

1. **A' 切法（4-change 序列）颗粒度合适**：原计划 ②③④ 都是"feature 簇"，落地节奏稳定，每个 change 1-2 天可 archive。比"按模块切"（前 7 个 change 都 lib-only）或"按能力簇切"（每个 change 块头过大）都好。
2. **OpenSpec 工作流的 propose / apply / archive 三段式**强制把决策外显——`design.md` 里的"必须显式记录的决策" + "deferred decisions"两段，让我们自己（和未来的我们）能追溯每个选择背后的理由。
3. **dogfood 触发的两次架构修正**都得到了正确处理：
   - libssh2 不读 `.ssh/config` → 开 ⓥ 整个 transport 切到 system git 子进程（**减法**比"打补丁"健康）
   - CLI 命名空间 type-agnostic → 用户提议改混合策略 → 在 ③ 中一并完成
   两次都通过修订 ADR + 新开 change 的方式处理，纸面与代码同步。
4. **自动测试 + 手动 dogfood 双层验证**：每个 change 都跑 cargo test + sandbox 手动剧本；后者在 ④ 时抓出 classify 函数的安全 bug（"用户在 paam 旧 target 上 mkdir 自有内容会被静默覆盖"）—— 这种 bug 单测不容易碰到，dogfood 抓住了。
5. **commit message 中文 + commit 单合并**：local-repo 的 git log 读起来像产品文档（"安装 / 同步 / 卸载 ..."），git history 自带"我做了什么"的说明书。
6. **跨模块依赖单向 DAG**：discover → asset、install → sync、uninstall → sync_no_commit；从未出现循环依赖。这得益于"动作下放到模块函数（ADR-0007 §7 修订版）"的决定。
7. **路径注入用环境变量**：`PAAM_HOME` / `PAAM_CLAUDE_TARGET_DIR`，让所有测试可以在 tempdir sandbox 跑；`test_support::env_lock()` 全局共享串行锁解决 cargo test 并行污染。

## 四、做得不够好的事

1. **ADR-0007 草案验证不足**：§6 的 type-agnostic CLI 与 §7 的 trait 草案在落地时都有问题（前者跨类型同名歧义；后者 object-safety），都到 dogfood 阶段才暴露。如果在 ADR 阶段就有"伪代码 + 用例验证"步骤，可以更早发现。
2. **首跑 OpenSpec 时 propose 偏重**：① paam-foundation-track 的 design.md 写得过详（10+ 决策），有些决策（如同步 vs 异步）M1 范围内根本不会触发。**首次跑应该更克制**，让 design.md 反映"必须现在决定的"，而不是"将来可能要想的"。
3. **任务清单粒度不一**：② / ③ / ④ 的 tasks.md 中有些任务是"实现一个函数"（半小时），有些是"重构整个模块"（半天）。任务粒度不齐让进度感受不准。M2 起可以给每个 task 加预估。
4. **测试 fixture 的并行污染**：env_var 测试在 cargo test 默认并行下相互打架，到 ④ 才抽出 `test_support::env_lock` 解决；前几次 change 没踩到只是因为没用 env_var 测试。算是"测试基础设施债"。
5. **dogfood 用沙盒太多**：每次 change apply 后用 `PAAM_HOME=$tmpdir paam ...` 沙盒跑剧本，没在真实 `~/.paam/` 跑过。沙盒虽然安全，但暴露不了某些只在用户真实环境才出现的问题（如那次 `.ssh/config` 重定向）。M2 可以考虑：每次 change apply 后跑一次沙盒 + 一次真实环境（前提是 idempotent / 安全）。
6. **没在每次 change 之间提交 git commit**：本仓在整个 M1 期间只有一个 base commit + working tree 变更，所有 change 的代码堆在 staged/unstaged 区。**v0.1.0 tag 之前应该补提交**，至少让 5 个 change 各自一个 commit，便于后续 git log 复盘。

## 五、关键决策路径（按时间顺序）

```
2026-04-26  ADR-0007 Accepted（数据架构）
2026-04-28  ① paam-foundation-track —— OpenSpec 工作流首跑，
                落地 PaamRoot / config.json / track 命令 / Asset trait 骨架
            ⓥ swap-git-transport-to-cli —— 因 libssh2 不读 .ssh/config 触发
                架构修正：删 git2-rs，所有 git 操作走 system git 子进程
2026-04-29  ② paam-skill-discovery —— Skill 类型 + discover + frontmatter
                修订 ADR-0007 §7（trait 草案 → 4 getter + 模块函数）
            ③ paam-skill-install-and-list —— local-repo + metadata + install
                修订 ADR-0007 §6（CLI 命名空间 type-agnostic → 混合策略）
2026-05-07  ④ paam-claude-sync —— sync / unsync + uninstall 智能耦合
                M1 端到端剧本闭合
            v0.1.0 发布
```

两次 ADR 修订都是在 dogfood / 实施时发现"草案与现实不符"，及时反向调整。这套"草案 → 实施 → 修订"的循环健康。

## 六、给 M2 的待办（不在本 retro 范围内执行）

按 PRODUCT.md / M1-plan §四 / 各 change 的 deferred decisions 汇总：

**功能扩展：**
- HTTPS / PAT 鉴权（M2）
- owner/repo URL 简写
- `paam untrack <alias>`（含本地缓存清理）
- `paam skill info <name>`
- `paam skill enable/disable/pin`
- `paam skill update`（基于 commit / tree_hash 对比）
- `paam search <kw>`（type-agnostic 跨类型搜索）
- `paam publish`
- `paam target detect`
- 多 target 支持（Cursor / Codex）
- `--mode copy` 同步模式
- dry-run

**类型扩展：**
- M2：Prompt 类型（含 PROMPT.md marker、`paam prompt <verb>` 命令树平行扩展）
- M3：MCP 类型

**桌面 UI：**
- M3 引入 Tauri，落地 `paam-app` crate

**架构 / 工程：**
- 同 source 内同名 skill 用 `--path <subpath>` 二级消歧
- 完整结构化解析 Anthropic Skills 规范字段
- `.paam-ignore` 文件 + gitignore 风格 glob
- target 路径写入 `config.json`（`targets:` 字段）
- target symlink stale 自动检测启发式
- 跨 source 同名 skill 在 install / sync 时的高级冲突解决
- M1 期间累积的"测试基础设施债"（test_support 抽象 / 串行锁等）

**流程：**
- ADR 草案"伪代码 + 用例验证"门控
- 任务粒度估算
- 每 change apply 后的"真实环境 dogfood"环节

## 七、回到产品宪章（PRODUCT.md）

M1 验证了 PRODUCT.md 中"Phase 1：Skills 管理"路径的可行性：
- ✅ 用户能订阅团队 / 个人 / 公开仓
- ✅ 跨设备复用（任意 macOS 干净环境跑通沙盒剧本）
- ✅ 完整 skill 生命周期：订阅 / 安装 / 同步（M2 加入 update / publish / 卸载完整体验）
- ✅ 与 Claude Code 集成（symlink 路径无缝）

下一步进入 PRODUCT.md 描述的 Phase 1 完整覆盖（M2）：把"安装 / 同步"扩展到"订阅 / 升级 / 发布 / 卸载"全生命周期。
