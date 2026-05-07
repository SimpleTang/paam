# installed-assets Specification

## Purpose
TBD - created by archiving change paam-skill-install-and-list. Update Purpose after archive.
## Requirements
### Requirement: 本地工作集（local-repo）物理与 git 状态

paam SHALL 在 `~/.paam/local-repo/` 维护一个由 paam 自管的 git 仓，作为已选资产的工作集。该仓在首次执行任何安装类命令前 SHALL 被自动初始化（`git init`），并 SHALL 设置独立的 git 身份（`user.email = paam@local`，`user.name = paam`），不读取用户的 `~/.gitconfig`。资产按 `<type>/<name>/` 子目录分组（M1 仅 `skills/`），每次安装 / 卸载 SHALL 触发自动 commit；如无文件变更则 SHALL 静默跳过 commit。

#### Scenario: 首次安装时自动初始化 local-repo

- **WHEN** `~/.paam/local-repo/` 不存在
- **AND** 用户执行 `paam skill install <name>`（其它前置条件满足）
- **THEN** 系统自动创建 `~/.paam/local-repo/`，执行 `git init`
- **AND** 设置 `user.email = paam@local` 与 `user.name = paam`
- **AND** 在该仓中完成本次安装并 commit

#### Scenario: 已初始化的 local-repo 不被重置

- **WHEN** `~/.paam/local-repo/.git/` 已存在且含历史 commit
- **AND** 用户执行任意安装类命令
- **THEN** 系统不重新执行 `git init`
- **AND** 不修改既有 `git config`

#### Scenario: 无变更时 commit 静默跳过

- **WHEN** 安装过程中文件系统未发生变更（如 force 重装但目标内容一致）
- **THEN** 系统不创建空 commit
- **AND** 命令仍以 exit code 0 结束

---

### Requirement: 资产元数据（.metadata.json）schema

每个已安装资产 SHALL 在其目录下持有一个 `.metadata.json` 文件，记录该资产的来源、安装时间、目标分发记录与 schema version。M1 阶段必填字段：`name`（字符串）、`type`（枚举，M1 仅 `skill`）、`origin`（对象，含 `kind` / `repo` / `subpath` / `commit` / `tree_hash`）、`installed_at`（ISO-8601 UTC 字符串）、`targets`（对象数组，M1 始终为 `[]`，由 ④ sync 写入）、`version`（字符串，M1 占位 `"1.0"`）。`origin.kind` M1 仅取值 `tracked`。

#### Scenario: 安装后 metadata.json 包含完整 provenance

- **WHEN** 用户执行 `paam skill install pdf-review --from github.com/foo/bar` 且成功
- **THEN** `~/.paam/local-repo/skills/pdf-review/.metadata.json` 存在
- **AND** 文件内容含 `name = "pdf-review"`、`type = "skill"`、`origin.kind = "tracked"`、`origin.repo = "github.com/foo/bar"`、`origin.subpath` 等于 source 仓内的相对路径
- **AND** `origin.commit` 等于 source 仓 HEAD 的全长 commit hash
- **AND** `origin.tree_hash` 等于该 subpath 在 source 仓 HEAD 中的 tree hash
- **AND** `installed_at` 为本次安装的 ISO-8601 UTC 时间字符串
- **AND** `targets` 为 `[]`
- **AND** `version` 为 `"1.0"`

#### Scenario: list_installed 聚合所有资产的 metadata

- **WHEN** local-repo 中存在多个已装资产（每个含 `.metadata.json`）
- **AND** 调用 `metadata::list_installed(&root)`
- **THEN** 返回的 Vec 包含所有资产的 metadata
- **AND** 缺失或 JSON 解析失败的资产被跳过并通过 stderr 输出 warning，不影响整体返回

---

### Requirement: paam skill install 命令

`paam skill install <name>` SHALL 在已订阅的 source 仓中查找名为 `name` 的 skill，将其完整目录复制到 `~/.paam/local-repo/skills/<name>/`，写入 `.metadata.json`，并在 local-repo 中自动 commit。命令 SHALL 支持 `--from <alias>` 参数限定 source 仓，与 `--force` 参数允许重装已存在的同名 skill。

#### Scenario: 唯一来源时直接安装

- **WHEN** 仅 1 个已订阅 source 含名为 `pdf-review` 的 skill
- **AND** local-repo 中尚无 `skills/pdf-review/`
- **AND** 用户执行 `paam skill install pdf-review`
- **THEN** 系统将该 skill 完整目录复制到 `~/.paam/local-repo/skills/pdf-review/`
- **AND** 写入 `.metadata.json` 含完整 provenance
- **AND** 在 local-repo 创建一个 commit，message 格式为 `安装 pdf-review，来自 <alias>@<commit前7位>`
- **AND** 命令以 exit code 0 结束，stdout 至少打印安装的 name、source alias、本地路径

#### Scenario: 跨 source 同名时拒绝并列候选

- **WHEN** 多个已订阅 source 都含名为 `code-review` 的 skill
- **AND** 用户执行 `paam skill install code-review`（无 `--from`）
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息列出所有候选 alias
- **AND** 错误信息提示用 `paam skill install code-review --from <alias>` 显式指定
- **AND** 不修改 local-repo

#### Scenario: 通过 --from 消除跨 source 歧义

- **WHEN** 多个 source 含同名 skill
- **AND** 用户执行 `paam skill install code-review --from github.com/foo/bar`
- **AND** `github.com/foo/bar` 中存在该 skill
- **THEN** 安装来自 `github.com/foo/bar` 的版本
- **AND** metadata 的 `origin.repo` 字段为 `github.com/foo/bar`

#### Scenario: 已安装时拒绝（无 --force）

- **WHEN** local-repo 中已存在 `skills/pdf-review/`
- **AND** 用户执行 `paam skill install pdf-review`（无 `--force`）
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息提示 skill 已安装，建议用 `--force` 重装或先 `paam skill uninstall pdf-review`
- **AND** 不修改 local-repo

#### Scenario: --force 重装

- **WHEN** local-repo 中已存在 `skills/pdf-review/`
- **AND** 用户执行 `paam skill install pdf-review --force`
- **THEN** 系统先删除既有 `skills/pdf-review/` 目录
- **AND** 重新复制并写入 `.metadata.json`
- **AND** 在 local-repo 创建一个 commit，message 格式为 `重新安装 pdf-review，来自 <alias>@<commit前7位>`

#### Scenario: skill 不存在时报错

- **WHEN** 用户执行 `paam skill install <未发现的 name>`
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息提示该 skill 在所有已订阅 source 中均未找到
- **AND** 错误信息建议用 `paam track skills <alias>` 查看仓内可用 skill

#### Scenario: 安装中途失败时回滚

- **WHEN** 用户执行 `paam skill install pdf-review`
- **AND** 在 cp -r 或 commit 阶段失败
- **THEN** 系统不在 `~/.paam/local-repo/skills/` 留下不完整的目录
- **AND** 不写入 `.metadata.json`
- **AND** 错误信息包含失败的具体原因

---

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

### Requirement: paam skill list 命令

`paam skill list` SHALL 列出 local-repo 中所有已安装的 skill，以表格形式输出 NAME / SOURCE / INSTALLED_AT 三列；空列表时 SHALL 给出友好提示。

#### Scenario: 列出已装 skill

- **WHEN** local-repo 中存在多个已装 skill
- **AND** 用户执行 `paam skill list`
- **THEN** 输出表头 `NAME  SOURCE  INSTALLED_AT` 与若干行记录
- **AND** SOURCE 列显示每个 skill 的 `origin.repo`（即 source alias）
- **AND** INSTALLED_AT 列显示 ISO-8601 字符串
- **AND** 命令以 exit code 0 结束

#### Scenario: 无已装 skill 时友好提示

- **WHEN** local-repo 中 `skills/` 目录不存在或为空
- **AND** 用户执行 `paam skill list`
- **THEN** 输出明确提示（如"暂无已安装的 Skill，使用 `paam skill install <name>` 安装"）
- **AND** 命令以 exit code 0 结束

---

### Requirement: paam list 命令（type-agnostic 全量）

`paam list` SHALL 列出所有已安装资产（M1 仅 skill；M2 起含 prompt / mcp），以表格形式输出 NAME / TYPE / SOURCE / INSTALLED_AT 四列。该命令保持 type-agnostic 形式，与 type-prefix 命令（`paam skill list`）并存——前者面向"我装了什么"的全量概览，后者面向"我有哪些 skill"的类型聚焦视图。

#### Scenario: 列出所有类型的已装资产

- **WHEN** local-repo 中存在若干已装 skill（M1 仅此类型）
- **AND** 用户执行 `paam list`
- **THEN** 输出表头 `NAME  TYPE  SOURCE  INSTALLED_AT` 与每个资产一行
- **AND** TYPE 列在 M1 阶段始终为 `skill`
- **AND** 命令以 exit code 0 结束

#### Scenario: 无已装资产时友好提示

- **WHEN** local-repo 中无任何已装资产
- **AND** 用户执行 `paam list`
- **THEN** 输出明确提示（如"暂无已安装的资产，使用 `paam skill install <name>` 安装"）
- **AND** 命令以 exit code 0 结束

---

### Requirement: 安装时记录 source 的 commit 与 tree_hash

paam 在执行 install 时 SHALL 调用 `paam-core::git::head_commit(source_local_path)` 与 `paam-core::git::subtree_hash(source_local_path, subpath)` 获取 source 仓 HEAD 的 commit hash 与该 subpath 的 tree hash，并写入 metadata 的 `origin.commit` / `origin.tree_hash` 字段。这两项是 M2 `paam update` 命令判断"该资产内容是否变更"的基础数据。

#### Scenario: 写入精确的 commit 与 tree_hash

- **WHEN** 用户执行安装命令
- **AND** source 仓 HEAD commit 为某具体值 X，subpath 对应 subtree hash 为某具体值 Y
- **THEN** 写入 metadata 的 `origin.commit` 等于 X（全长 40 位）
- **AND** `origin.tree_hash` 等于 Y（全长 40 位）

#### Scenario: source 仓 HEAD 异常时报错并回滚

- **WHEN** source 仓不是合法 git 仓（如 `.git` 不存在）或 HEAD 指向无效
- **AND** 用户执行安装命令
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息说明 git 调用失败原因
- **AND** 不在 local-repo 留下任何变更（cp / metadata / commit 全部回滚）

