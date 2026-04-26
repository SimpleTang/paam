# ADR-0002: CLI 与 UI 核心库共享策略

- **Status**: Accepted
- **Date**: 2026-04-26
- **Deciders**: @simpletang1994
- **Tags**: tech-stack | architecture

## Context

依赖 [ADR-0001](./0001-tech-stack-choice.md) 选定 Tauri (Rust)。

paam 演进路径：
- M1 / M2：纯 CLI
- M3：引入桌面 UI（macOS）
- M4：Windows 适配

两端必须**行为一致**（PRODUCT.md §1.6 强约束）。需要确定 CLI 与 UI 怎么共享业务逻辑。

可能的策略：

- A. 单 Rust crate 双产物（CLI bin + 内嵌核心 lib，UI 通过 lib 调用）
- B. Cargo workspace：core lib + cli bin + app bin 三个独立 crate
- C. UI 通过子进程调用 CLI 二进制

## Decision

**采用 Cargo workspace（选项 B）**。仓库结构：

```
paam/
├── Cargo.toml                         ← workspace 配置
├── Cargo.lock
├── crates/
│   ├── paam-core/                     ← 业务逻辑库（核心）
│   │   ├── Cargo.toml                 (lib crate)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config/                ← config.json 读写
│   │       ├── git/                   ← git2-rs 封装
│   │       ├── source/                ← source 管理（track / 扫描）
│   │       ├── local_repo/            ← local-repo 管理（install / 自动 commit）
│   │       ├── sync/                  ← target 分发（symlink / 冲突）
│   │       ├── metadata/              ← .metadata.json 读写
│   │       ├── asset/                 ← 资产抽象 trait
│   │       └── error.rs               ← 统一错误类型
│   │
│   ├── paam-cli/                      ← CLI 入口
│   │   ├── Cargo.toml                 (bin crate, deps: paam-core, clap)
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/              ← 子命令实现
│   │
│   └── paam-app/                      ← Tauri 桌面应用（M3 引入，先不创建）
│       ├── Cargo.toml                 (bin crate, deps: paam-core, tauri)
│       ├── src-tauri/
│       └── ui/                        ← Web 前端
│
└── (其他: .github/, .dev/, openspec/, ...)
```

## Alternatives Considered

### A. 单 crate 双产物

- **Pros**: 最简单
- **Cons**:
  - CLI 编译时也得编 Tauri 依赖（即使 CLI 不用），CI 慢
  - 不利于以后拆出 paam-server / paam-daemon 等可能的产物
- **Verdict**: ❌ 拒绝

### B. Cargo workspace ✅

- **Pros**:
  - 关注点分离：core 不依赖任何 UI/CLI 框架
  - CLI 独立编译（M1/M2 阶段不引入 Tauri 依赖，编译快）
  - 易扩展：未来加 paam-server / paam-daemon 仅是新 crate
  - 符合 Rust 生态约定（`cargo new --lib` 工作区是标准模式）
- **Cons**:
  - 项目结构稍复杂（但模板成熟，AI 熟练）
- **Verdict**: ✅ 接受

### C. UI 通过子进程调用 CLI

- **Pros**: 行为天然一致（同一二进制）
- **Cons**:
  - IPC 复杂：UI 需序列化命令、解析 stdout、处理 exit code
  - 错误处理痛苦：CLI 错误信息以人读字符串呈现，UI 难以结构化展示
  - 状态同步：UI 需要轮询或自定义 IPC，与 CLI 操作 race condition
- **Verdict**: ❌ 拒绝

## Consequences

### Positive

- CLI 和 UI 行为完全一致（同一份 Rust 代码）
- 修复一处惠及两端
- 测试集中在 `paam-core`（不需要分别测 CLI 和 UI 各自的业务逻辑）
- M3 引入 UI 不需要重构核心
- M1 / M2 阶段编译速度更快（不引入 Tauri 依赖）

### Negative

- 三个 crate 的版本号需要统一（用 `[workspace.package] version = ...` 解决）
- AI 修改业务逻辑时需要意识到"这是 paam-core 的事，不要在 paam-cli 里塞业务"

### Neutral / Trade-offs

- `paam-core` 的公开 API 即"业务接口"——破坏性变更影响两端，需要谨慎设计
- 如果未来要给社区开放 `paam-core` 作为库使用，需要稳定的 SemVer 政策

## Implementation Notes

**M1 启动时**：

- 仅创建 `paam-core` + `paam-cli` 两个 crate
- `paam-app` 等 M3 启动前再添加（避免提前引入 Tauri 依赖）

**`paam-core` 公开 API 风格**：

- 所有公开函数返回 `Result<T, paam_core::Error>`
- 错误类型用 `thiserror` 派生
- 无 panic（除非真正不可恢复）
- 不依赖任何 UI/CLI 框架（独立可测）

**workspace Cargo.toml 模板**：

```toml
[workspace]
members = ["crates/paam-core", "crates/paam-cli"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "TBD"        # 见 ADR-0003
repository = "https://github.com/simpletang1994/paam"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "fs"] }
git2 = "0"
clap = { version = "4", features = ["derive"] }
thiserror = "1"
tracing = "0"
```

## References

- ADR-0001 技术栈选型
- PRODUCT.md §1.6 产品形态（行为一致约束）
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §5.2、§9 待决策 #2
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
| 2026-04-26 | Accepted | 经讨论后接受 workspace 三 crate 方案 |
