## ADDED Requirements

### Requirement: target 路径契约

paam SHALL 把已安装 skill 同步到 Claude Code 的 skill 目录，默认路径为 `~/.claude/skills/`。该路径 SHALL 可通过环境变量 `PAAM_CLAUDE_TARGET_DIR` 覆盖（用于测试与自定义部署）。target 父目录不存在时 SHALL 在首次 sync 时自动创建（`mkdir -p`）。M1 阶段不支持多 target；不读 / 写 paam config.json 中的 target 配置。

#### Scenario: 默认 target 路径

- **WHEN** 环境变量 `PAAM_CLAUDE_TARGET_DIR` 未设置
- **AND** 调用 `paths::claude_skills_target_dir()`
- **THEN** 返回 `<home>/.claude/skills`（home 由 `directories-next::BaseDirs` 解析）

#### Scenario: 环境变量覆盖 target 路径

- **WHEN** 环境变量 `PAAM_CLAUDE_TARGET_DIR` 设为 `/tmp/test-target`
- **AND** 调用 `paths::claude_skills_target_dir()`
- **THEN** 返回 `/tmp/test-target`

#### Scenario: target 父目录不存在时自动创建

- **WHEN** target 路径父目录不存在
- **AND** 用户执行 `paam sync`
- **THEN** 系统自动 `mkdir -p` 创建父目录
- **AND** 创建对应的 symlink

---

### Requirement: paam sync 命令

`paam sync [--force]` SHALL 把 `~/.paam/local-repo/skills/*` 中的每个已装 skill 通过 symlink 暴露到 target 路径下的同名目录。命令 SHALL 是幂等的：已正确指向的 symlink 不重建、不更新 metadata、不产生 commit。命令 SHALL 返回 `SyncReport`，包含 `synced` / `already_ok` / `conflicts` / `forced` 四类状态分组；冲突 SHALL NOT 触发整体失败。

#### Scenario: 干净 target 上的首次同步

- **WHEN** target 目录不含任何 skill 同名条目
- **AND** local-repo 中存在 N 个已装 skill
- **AND** 用户执行 `paam sync`
- **THEN** 系统在 target 下创建 N 条 symlink，每条指向对应的 `local-repo/skills/<name>/`
- **AND** 每个 skill 的 `.metadata.json.targets[]` 写入一条 `{ agent: "claude-code", path: <target_path>, mode: "symlink", synced_at: <UTC ISO-8601> }`
- **AND** local-repo 创建一个 commit；多 skill 时 message 为 `同步 N 个 skill 到 claude-code`，单 skill 时为 `同步 X -> claude-code:<short_path>`
- **AND** SyncReport.synced 含所有 N 个 skill 名

#### Scenario: 已正确指向时幂等跳过

- **WHEN** target 中已存在 symlink 指向正确的 local-repo 目录
- **AND** 用户再次执行 `paam sync`
- **THEN** 该 skill 加入 SyncReport.already_ok
- **AND** 不修改 metadata.targets[]
- **AND** 全部 already_ok 时 `paam sync` 不在 local-repo 创建任何 commit

#### Scenario: 旧 paam symlink 失效时重建

- **WHEN** target 中存在 symlink，但其指向的 local-repo 路径已不存在或已变化
- **AND** metadata.targets[] 中含该 target 的记录（即 paam 旧链路）
- **AND** 用户执行 `paam sync`
- **THEN** 系统删除旧 symlink 并创建正确指向的新 symlink
- **AND** 该 skill 加入 SyncReport.synced
- **AND** 写入新的 metadata.targets[]

#### Scenario: 非 paam 内容的冲突默认跳过

- **WHEN** target 中存在与 skill 同名的真实文件 / 目录（非 paam 创建的 symlink）
- **AND** 用户执行 `paam sync`（无 `--force`）
- **THEN** 该 skill 加入 SyncReport.conflicts，含 skill_name / target_path / 简短 reason
- **AND** stderr 输出 warning 行
- **AND** 不修改该 target 内容
- **AND** 其它 skill 仍正常同步（冲突不阻塞全局）

#### Scenario: --force 覆盖非 paam 内容

- **WHEN** target 中存在非 paam 管理的文件 / 目录与 skill 同名
- **AND** 用户执行 `paam sync --force`
- **THEN** 系统删除原内容并创建正确指向的 symlink
- **AND** 该 skill 加入 SyncReport.forced
- **AND** 写入 metadata.targets[]

---

### Requirement: paam unsync 命令

`paam unsync <name>` 与 `paam unsync --all` SHALL 仅删除 target 上的 symlink 并清空对应 skill 的 `metadata.targets[]`，不修改 local-repo 中的 skill 内容。`name` 与 `--all` 互斥。target 不存在时 SHALL 静默成功（用户意图是"清理"，目标不存在视为已达成）。变更后 SHALL 在 local-repo 创建 commit；单 skill message `解除同步 X`、--all message `解除所有同步`。

#### Scenario: 解除单个已同步 skill

- **WHEN** local-repo 中存在 `skills/pdf-review/`，target 中有对应 symlink，metadata.targets[] 非空
- **AND** 用户执行 `paam unsync pdf-review`
- **THEN** 系统删除 target symlink
- **AND** 该 skill 的 metadata.targets[] 清空为 `[]`
- **AND** local-repo 创建一个 commit，message 为 `解除同步 pdf-review`

#### Scenario: --all 解除所有同步

- **WHEN** 多个已装 skill 各有 target symlink
- **AND** 用户执行 `paam unsync --all`
- **THEN** 系统删除所有 target symlink
- **AND** 所有 skill 的 metadata.targets[] 清空
- **AND** local-repo 创建一个 commit，message 为 `解除所有同步`

#### Scenario: target 不存在时静默成功

- **WHEN** local-repo 中存在 `skills/<name>/` 但 target 上无对应 symlink，metadata.targets[] 已为空
- **AND** 用户执行 `paam unsync <name>`
- **THEN** 命令以 exit code 0 结束（不抛错）
- **AND** 不创建新 commit（无变更）

#### Scenario: name 与 --all 互斥

- **WHEN** 用户执行 `paam unsync foo --all`
- **THEN** 系统以非零 exit code 失败
- **AND** 错误信息提示参数互斥

---

### Requirement: paam 管理的 target 探测使用双重证据

判断某 target 路径是否由 paam 管理 SHALL 同时考虑两条证据，任一命中即视作 paam 管理：
1. **主路径**：从 `metadata.targets[]` 中查询是否有项的 `path` 字段等于该 target 路径
2. **备路径**：fs `canonicalize` 后是否落在 `~/.paam/local-repo/skills/` 下

两条证据均不命中且 target 是真实文件 / 目录（非 symlink）→ 视为非 paam 管理（触发冲突或 --force 覆盖）。

#### Scenario: metadata 命中即视作 paam 管理

- **WHEN** metadata.targets[] 中含 `{ path: "/test/target/foo", ... }`
- **AND** 该 target 实际是一条指向其他位置的 symlink（如 paam 旧链路漂移）
- **THEN** sync 视其为 paam 管理，按"旧 paam symlink 失效"路径重建

#### Scenario: canonicalize 命中即视作 paam 管理

- **WHEN** metadata.targets[] 为空
- **AND** target 是 symlink，canonicalize 后落在 `<paam_home>/local-repo/skills/<name>/`
- **THEN** sync 视其为 paam 管理（视为 metadata 与 fs 不同步的边角，按 sync 流程修复 metadata.targets[]）

#### Scenario: 双双不命中视为冲突

- **WHEN** metadata.targets[] 不含该 target
- **AND** target 是真实文件 / 目录，不是指向 local-repo 的 symlink
- **THEN** sync 视为非 paam 管理；force=false → conflicts；force=true → forced

---

### Requirement: SyncIo 错误处理

paam SHALL 在 symlink 创建 / 删除 / readlink / mkdir 失败时返回 `Error::SyncIo { skill, target, message }` 携带上下文（哪个 skill、哪个 target 路径、底层 IO 错误描述）。Sync 业务流程**不**应在某个 skill 的 IO 错误下整体失败——已成功的 skill SHALL 已写入 metadata 并参与 commit；失败的 skill 通过 stderr 输出 warning 跳过。

#### Scenario: 部分 skill 失败时其它 skill 仍正常完成

- **WHEN** 多个 skill 同步过程中，某一个的 symlink 创建失败（如权限不足）
- **AND** 用户执行 `paam sync`
- **THEN** 失败的 skill 通过 stderr 输出 warning（含 skill 名 + target 路径 + 错误描述）
- **AND** 其它 skill 仍正常 sync 并写入 metadata + 参与 commit
- **AND** 命令仍以 exit code 0 结束（部分成功不当作整体失败）
