# ADR-0003: 开源协议选择

- **Status**: Accepted
- **Date**: 2026-04-27
- **Deciders**: @SimpleTang
- **Tags**: legal | release

## Context

paam 已确定为开源项目，托管在 GitHub。需要选择具体的开源协议。

paam 的特点：

- 工具类软件（CLI + 桌面 UI），非 SaaS / 平台
- 个人开发，目标社区使用 + 团队使用
- 用户的核心资产（Skills）由用户自有，不归 paam
- 以"私有优先"为定位的工具

## Decision

**采用 MIT License**。

具体动作：

- 仓库根目录创建 `LICENSE` 文件（MIT 标准文本，Copyright © 2026 SimpleTang）
- 在 `Cargo.toml` `workspace.package` 设置 `license = "MIT"`
- README 顶部声明协议

## Alternatives Considered

### MIT ✅

- **Pros**:
  - 工具类软件最广泛接受（git / npm / cargo / brew / homebrew 都是 MIT）
  - 协议短（一页），用户和贡献者都易理解
  - 商业友好（无任何使用限制）
  - 与 paam 的"私有优先"定位一致——给用户最大自由
- **Cons**:
  - 无显式专利授权（但工具类软件通常无专利风险）
- **Verdict**: ✅ 接受

### Apache-2.0

- **Pros**: 包含显式专利授权，对企业贡献者更友好
- **Cons**: 协议文件较长，对小项目偏重；普及度略不及 MIT
- **Verdict**: 候补；未来若涉及专利风险可考虑切换

### GPL-3.0

- **Pros**: 强 copyleft，防止商业方"白嫖"
- **Cons**: 阻止商业使用与集成，与 paam"广泛被使用"的目标冲突
- **Verdict**: ❌ 拒绝

### MPL-2.0

- **Pros**: 文件级 copyleft，平衡商业友好与开源保护
- **Cons**: 用户和贡献者较少熟悉
- **Verdict**: ❌ 拒绝

### BSL (Business Source License)

- **Verdict**: ❌ 不符合"开源"目标

## Consequences

### Positive

- 任何人可以自由使用、修改、分发
- 商业用户也能放心采用
- 协议短、好理解，降低贡献门槛

### Negative

- 任何人可 fork 并闭源商业化（接受这个权衡——"工具被广泛使用"优先于"防止商业 fork"）

### Neutral / Trade-offs

- 不在每个源文件头部强制添加版权声明（MIT 不要求；保持代码简洁）

## Implementation Notes

- `LICENSE` 文件已创建（首次 commit 一同推送）
- `Cargo.toml [workspace.package]` 已设置 `license = "MIT"`

## References

- [Choose a License — MIT](https://choosealicense.com/licenses/mit/)
- [SPDX MIT](https://spdx.org/licenses/MIT.html)
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #3

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
| 2026-04-27 | Accepted | 选定 MIT，与工具类开源生态一致 |
