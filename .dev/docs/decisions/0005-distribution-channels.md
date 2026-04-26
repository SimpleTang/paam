# ADR-0005: 公开发布渠道与代码签名

- **Status**: Proposed
- **Date**: 2026-04-25
- **Deciders**: @simpletang1994
- **Tags**: release | security

## Context

paam 需要分发：

- **CLI 二进制**：macOS / Windows / Linux
- **桌面 UI 安装包**：macOS（.dmg / .pkg）、Windows（.msi / .exe）

设计阶段将"公开发布渠道 / 签名证书"列为待决策项（见 `.dev/docs/archived/PRD-v0.1-design-snapshot.md` §9 待决策 #5）。

需要决策的子问题：

1. **分发渠道**：自主分发（GitHub Releases）/ Homebrew / Microsoft Store / Mac App Store / winget / 包管理器 (apt / yum / cargo)
2. **代码签名**：macOS 公证 + Windows 代码签名证书的获取与配置
3. **自动更新机制**：内置 updater / 完全依赖外部包管理器

## Decision

待决策。

## Alternatives Considered

### 分发渠道

#### Option A: GitHub Releases（基础）

- **Pros**: 零成本、CI 集成简单、对开源项目最自然
- **Cons**: 用户需要手动下载，发现性弱

#### Option B: Homebrew (macOS / Linux)

- **Pros**: macOS 开发者最常用、`brew install paam` 体验好
- **Cons**: tap 维护、CLI tap 与 cask（GUI）分别管理

#### Option C: winget / Chocolatey (Windows)

- **Pros**: Windows 开发者主流分发
- **Cons**: 提交清单与维护成本

#### Option D: 应用商店（Mac App Store / Microsoft Store）

- **Pros**: 用户信任度高
- **Cons**: 沙盒限制（paam 需要访问任意路径、git 操作，沙盒下复杂）；审核周期长

#### Option E: Cargo / npm / pip 等语言包管理器

- **Pros**: 适合 CLI 用户群
- **Cons**: 仅 CLI、不适合 GUI

### 代码签名

#### macOS

- 需要 Apple Developer 账号（$99/年）
- 公证（Notarization）+ Gatekeeper 友好
- 不签名会被系统警告

#### Windows

- 需要 OV / EV 代码签名证书（约 $200-400/年起）
- 不签名会触发 SmartScreen 警告
- 替代：先不签名 + 文档说明绕过方法（早期可行，长期需要签名）

## Consequences

待决策后补充。

## Implementation Notes

建议路线（待评估）：

- **早期（M1-M3）**：仅 GitHub Releases，CLI 同时通过 cargo / npm 等发布
- **M4-M5**：补 Homebrew tap（macOS）+ winget 清单（Windows）
- **M5 之前**：拿到 Apple Developer 账号 + Windows 代码签名证书
- **暂不进入**：Mac App Store / Microsoft Store（沙盒不友好）

## References

- M5 milestone 计划包含"安装包签名"（详见对应 milestone plan）
- 历史设计快照：`.dev/docs/archived/PRD-v0.1-design-snapshot.md` §8、§9 待决策 #5
- [Apple Notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
- [Windows Code Signing](https://learn.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)

---

**Changelog**:

| Date | Status | Note |
|---|---|---|
| 2026-04-25 | Proposed | 初始提案 |
