# Changelog

本项目所有重要变更记录于此。

格式遵循 [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

### Added

- 集成 [OpenSpec](https://github.com/Fission-AI/OpenSpec) v1.3.1 作为执行层 spec 工具
  - 支持 `/opsx:propose | apply | archive | explore` slash commands（Claude Code）
- 确定技术栈：**Tauri v2 + Rust**（详见 ADR-0001）
- 确定代码组织：**Cargo workspace**（`paam-core` + `paam-cli`，M3 加 `paam-app`；详见 ADR-0002）
- 确定开源协议：**MIT License**（详见 ADR-0003，LICENSE 文件已创建）
- 确定数据架构（详见 ADR-0007）：
  - 工作目录 `~/.paam/`（跨平台一致）
  - 三层资产流转：source → local-repo → target
  - local-repo 按类型分组（mcp / prompts / skills），source 不强加结构
  - type-agnostic CLI 命令（资产类型由文件 marker 识别）
- 创建 Cargo workspace 骨架：`paam-core` (lib) + `paam-cli` (bin `paam`)

项目处于设计阶段，尚未发布版本。M1 已 Ready to Start。
