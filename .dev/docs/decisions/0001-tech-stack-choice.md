# ADR-0001: 桌面端与核心库技术栈选型

- **Status**: Accepted
- **Date**: 2026-04-26
- **Deciders**: @simpletang1994
- **Tags**: tech-stack

## Context

paam 需要同时提供：

- CLI（macOS / Windows / Linux）
- 桌面 UI（macOS 优先 + Windows）
- 共享核心库（PRODUCT.md §1.6 强约束）

候选方案：Tauri (Rust) / Electron (Node.js) / Wails (Go) / KMP (Compose Multiplatform) / Flutter / 纯原生（Swift + C#）。

设计阶段确认的关键约束：

- **完全 AI 编码**：用户不写代码，仅 review。学习曲线对用户不再是约束，但 AI 编码质量与生态成熟度变成强约束
- **性能基线非硬约束**：PRODUCT.md §5.1 的数字仅作参考目标
- **共享核心库**：CLI 与 UI 必须行为一致（PRODUCT.md §1.6）
- **单人维护**：工具链必须长期稳定，不能选脆弱的小众生态

## Decision

**采用 Tauri v2 + Rust** 作为 paam 的技术栈。

具体配置：
- 业务逻辑层：Rust
- CLI 入口：独立 Rust 二进制
- 桌面 UI：Tauri v2（M3 启动时引入；Web 前端框架届时再选）
- CLI 与 UI 共享：通过 Cargo workspace（详见 ADR-0002）

## Alternatives Considered

### Tauri (Rust) ✅

- **Pros**:
  - **AI 编码 + Rust = 最佳搭配**：编译器是"硬性 reviewer"，AI 写错（借用、类型、内存）当场被拦截，bug 率显著低于动态类型语言
  - 生态对 paam"现成可拼"：`git2-rs` / `clap` / `serde` / `tokio` / `dirs` 都是顶级
  - 包体积 5-10 MB（业界最小桌面框架之一）
  - Tauri v2 系统集成完整：macOS 菜单栏 / 状态栏 / 通知中心；Windows 系统托盘
  - 单 crate workspace 天然实现 CLI/UI 共享，无桥接成本
  - Rust 长期稳定（不会 Node.js / Electron 那样频繁 breaking change）
- **Cons**:
  - 用户 review 时 lifetime / borrow 等概念有理解门槛（但 AI 可解释）
  - 全量编译慢（5-10 min 首次）；增量编译 OK
- **Verdict**: ✅ 接受

### Wails (Go)

- **Pros**: 简洁可读、Go 编译快、`go-git` 完整、AI 写 Go 极佳
- **Cons**: Wails Desktop 生态较 Tauri 新；Go 偶有 nil panic 类 bug，AI 不一定都能避开
- **Verdict**: 备选——若要"用户审阅时更轻松"则切此方案

### Electron (TypeScript)

- **Pros**: AI 训练数据最丰富，用户阅读最自然，Web 生态完整
- **Cons**: 包 100MB+；Node.js 启动慢；内存占用高；与 Rust/Go 静态二进制方案不在同一档
- **Verdict**: ❌ 仅在用户强偏好 TS 时考虑；paam 是工具非应用，包大无价值

### KMP / Compose Multiplatform (Kotlin)

- **Pros**: 单语言跨 CLI/UI 优雅
- **Cons**: Kotlin Native 工具链未成熟；调 libgit2 需手写 cinterop；AI 训练数据稀疏
- **Verdict**: ❌ 拒绝

### Flutter / Dart

- **Pros**: 跨平台单语言
- **Cons**: Flutter Desktop 不如 Mobile 成熟；工具类应用 overkill
- **Verdict**: ❌ 拒绝

### 纯原生（Swift + C#）

- **Verdict**: ❌ 直接违反"共享核心库"硬约束

## Consequences

### Positive

- AI 编码质量高（编译器双保险）
- 包小、启动快、跨平台部署稳
- CLI 与 UI 100% 共享业务逻辑（同一份 Rust crate）
- 工具链长期稳定，单人维护负担可控

### Negative

- 用户 review 时偶尔需要 AI 解释 Rust 特有语法
- 全量编译慢（增量 OK）
- macOS 公证、Windows 代码签名需配置（M5 解决）

### Neutral / Trade-offs

- Web 前端框架（Vue / React / Svelte）M3 启动前再决策
- 异步运行时统一用 `tokio`

## Implementation Notes

**工具链**：

- `rustup` stable（最新稳定版）
- VS Code + `rust-analyzer`（推荐）
- `cargo` 内置即可

**M1 关键依赖**：

| 用途 | crate |
|---|---|
| CLI 框架 | `clap` (derive 宏风格) |
| Git 操作 | `git2`（libgit2 绑定）+ 必要时 fallback 到系统 `git` 子进程 |
| 序列化 | `serde` + `serde_json` |
| 异步运行时 | `tokio` |
| 跨平台路径 | `directories-next` 或 `dirs` |
| 错误类型 | `thiserror` |
| 日志 | `tracing` + `tracing-subscriber` |

**M3 引入**：

| 用途 | crate |
|---|---|
| 桌面 UI | `tauri = "2"` |
| Web 前端 | TBD（M3 启动前决策） |

**质量门槛**：

- 每次主要变更 AI 必跑 `cargo check` + `cargo clippy`
- pre-commit hook：`cargo fmt --check` + `cargo clippy -- -D warnings`

## References

- PRODUCT.md §1.6 产品形态
- PRODUCT.md §5.1 非功能性需求（参考目标）
- 关联 ADR：[ADR-0002](./0002-shared-core-strategy.md) 核心库共享策略
- [Tauri](https://tauri.app/)
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §5.2、§9 待决策 #1

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
| 2026-04-26 | Accepted | 经"完全 AI 编码 + 性能非硬约束"新约束讨论后接受 Tauri (Rust) |
