## MODIFIED Requirements

### Requirement: paam skill uninstall 命令

`paam skill uninstall <name>` SHALL 删除 `~/.paam/local-repo/skills/<name>/` 整个目录，并在 local-repo 中自动 commit。skill 未安装时 SHALL 以非零 exit code 失败并给出明确提示。**在删除 local-repo 目录之前，SHALL 自动调用 `sync::unsync_one(name)` 清理该 skill 在所有 target（M1 阶段为 Claude Code）已建立的 symlink，避免 dangling**。target 清理与 local-repo 删除 SHALL 在同一 commit 内完成（commit message 仍为 `卸载 <name>`）。

#### Scenario: 卸载已安装的 skill

- **WHEN** local-repo 中存在 `skills/pdf-review/`
- **AND** 用户执行 `paam skill uninstall pdf-review`
- **THEN** 系统删除 `skills/pdf-review/` 整个目录
- **AND** 在 local-repo 创建一个 commit，message 为 `卸载 pdf-review`
- **AND** 命令以 exit code 0 结束

#### Scenario: 卸载未安装的 skill 时报错

- **WHEN** local-repo 中不存在 `skills/<name>/`
- **AND** 用户执行 `paam skill uninstall <name>`
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息提示该 skill 未安装

#### Scenario: 卸载已同步的 skill 自动清理 target symlink

- **WHEN** local-repo 中存在 `skills/pdf-review/`，且其 `.metadata.json.targets[]` 非空（已 `paam sync` 建立 target symlink）
- **AND** 用户执行 `paam skill uninstall pdf-review`
- **THEN** 系统先删除 target 上的 symlink（沿用 `sync::unsync_one` 行为）
- **AND** 再删除 local-repo 中的 `skills/pdf-review/` 目录
- **AND** 在 local-repo 仅创建一个 commit（含 target 清理 + local-repo 删除两组变更），message 为 `卸载 pdf-review`
- **AND** 命令以 exit code 0 结束

#### Scenario: 卸载未 sync 的 skill 不触发 sync 流程

- **WHEN** local-repo 中存在 `skills/<name>/`，但 `.metadata.json.targets[]` 为空
- **AND** 用户执行 `paam skill uninstall <name>`
- **THEN** 系统跳过 target 清理（无内容可清）
- **AND** 删除 local-repo 中的 `skills/<name>/` 目录
- **AND** 在 local-repo 创建一个 commit，message 为 `卸载 <name>`
