## ADDED Requirements

### Requirement: Asset 抽象与 Skill 类型

paam-core SHALL 提供一个名为 `Asset` 的 trait 作为所有 AI Agent 资产（skill / prompt / mcp 等）的统一抽象，并提供 `AssetKind` 枚举枚举所有支持的资产类型。Trait SHALL 保持 object-safe（即支持 `&dyn Asset` 与 `Box<dyn Asset>`），仅暴露纯 getter 方法（`id` / `kind` / `source_alias` / `relative_path`）；任何"动作"（discover / install / read 等）SHALL 通过模块级函数表达，不挂在 trait 上。M1 阶段 `AssetKind` 仅含 `Skill` 一个 variant，并通过 `#[non_exhaustive]` 允许后续无破坏性扩展。

#### Scenario: Asset trait 是 object-safe 的

- **WHEN** 编译期检查 `&dyn Asset` 或 `Box<dyn Asset>` 类型
- **THEN** 编译通过
- **AND** 任意未来新增的资产类型（Prompt / Mcp 等）只要实现 `Asset` 即可放入 `Vec<Box<dyn Asset>>`

#### Scenario: Skill 类型实现 Asset

- **WHEN** 调用 `skill.kind()`
- **THEN** 返回 `AssetKind::Skill`
- **AND** `skill.id()` 返回 SKILL.md frontmatter 中的 `name` 字段
- **AND** `skill.source_alias()` 返回该 skill 所属订阅源的 alias
- **AND** `skill.relative_path()` 返回从源仓根到含 SKILL.md 的目录的相对路径

---

### Requirement: SKILL.md frontmatter 字段约定

Skill 的元数据 SHALL 来自其所在目录的 `SKILL.md` 文件顶部的 YAML frontmatter（被 `---` 行包围）。M1 必填字段：`name`（字符串，作为 skill 标识符）与 `description`（字符串，用于列表展示）。其它字段（如 `when_to_use` / `required_permissions` / `model` 等 Anthropic Agent Skills 规范定义的可选字段）SHALL 被透明保留到 `Skill::extra` 字段（`HashMap<String, serde_yaml_ng::Value>`），不结构化解析；解析时 SHALL 通过 `tracing::warn!` 一行输出未识别字段名列表，便于开发者审计。

#### Scenario: 解析合法 frontmatter

- **WHEN** SKILL.md 顶部有合法 YAML frontmatter，含 `name: pdf-review` 与 `description: ...`
- **THEN** 返回的 `Skill` 实例 `id() == "pdf-review"`，`description() == "..."`（或对外接口提供的等价访问）
- **AND** `extra` 字段为空 HashMap

#### Scenario: 未识别字段透明保留

- **WHEN** SKILL.md frontmatter 含 `name`、`description` 与未识别的 `when_to_use: "..."`、`required_permissions: [...]`
- **THEN** 返回的 `Skill` 必填字段正常填充
- **AND** `extra` 字段含 `when_to_use` 与 `required_permissions` 两个键
- **AND** 通过 tracing 输出一条 warn 级别日志，标注路径与 `["when_to_use", "required_permissions"]`

---

### Requirement: discover 扫描语义

paam-core SHALL 提供 `discover::skills_in(repo: &Path, ignore: &[String]) -> Vec<Skill>` 函数，递归扫描 `repo` 目录下含 `SKILL.md` 的子目录并解析为 `Skill` 实例。扫描 SHALL 跳过任何 `file_name()` 完全匹配 `ignore` 列表中字符串的目录及其子树。该函数 SHALL 接受任意路径（不要求是 git 仓），与 git transport 完全解耦。

#### Scenario: 单个 SKILL.md 被发现

- **WHEN** 临时目录下存在 `tools/pdf-review/SKILL.md`（合法 frontmatter）
- **AND** 调用 `discover::skills_in(<tmp>, &default_ignore)`
- **THEN** 返回 `Vec<Skill>` 长度为 1
- **AND** 该 skill 的 `relative_path()` 等于 `tools/pdf-review`

#### Scenario: 多层嵌套独立发现

- **WHEN** 临时目录下同时存在 `code-review/SKILL.md`、`tools/pdf-review/SKILL.md`、`docs/how-to/SKILL.md`
- **AND** 调用 `discover::skills_in(<tmp>, &default_ignore)`
- **THEN** 返回 3 个 Skill，路径互不相同

#### Scenario: ignore 目录被跳过

- **WHEN** 临时目录下存在 `.git/HEAD`、`node_modules/some-pkg/SKILL.md`、`tools/real-skill/SKILL.md`
- **AND** ignore 列表为 `[".git", "node_modules"]`
- **AND** 调用 `discover::skills_in(<tmp>, &ignore)`
- **THEN** 仅返回 `tools/real-skill` 一个 Skill
- **AND** 不进入 `.git` 或 `node_modules` 子树（即使其中含 SKILL.md）

#### Scenario: 发现失败的 SKILL.md 被宽松跳过

- **WHEN** 临时目录下存在 `valid/SKILL.md`（合法）、`broken/SKILL.md`（缺 `name` 字段）、`yaml-error/SKILL.md`（YAML 语法错误）
- **AND** 调用 `discover::skills_in`
- **THEN** 返回的 Vec 长度为 1（仅 `valid/SKILL.md` 对应的 Skill）
- **AND** 通过 stderr 输出至少 2 行 warning，分别提示 `broken/SKILL.md` 与 `yaml-error/SKILL.md` 跳过原因
- **AND** 函数本身不返回错误（discover 失败的个体不影响整体）

---

### Requirement: scan_ignore 配置

paam SHALL 在用户配置文件 `~/.paam/config.json` 支持可选字段 `scan_ignore: Vec<String>`，用于自定义 discover 扫描时跳过的目录名列表。该字段语义为**完全替换**内置默认列表：字段缺失或为 null 时使用内置默认；为非空数组时使用用户列表（不与默认合并）；为空数组时不忽略任何目录。配置文件 schema 版本号 SHALL 保持 v1（向后兼容变更）。

#### Scenario: 缺省时使用内置默认

- **WHEN** `config.json` 不含 `scan_ignore` 字段
- **AND** 调用 `config::effective_scan_ignore(&root)`
- **THEN** 返回 `discover::DEFAULT_IGNORE` 内置列表（含 `.git` / `node_modules` / `target` 等）

#### Scenario: 用户提供的列表完全替换默认

- **WHEN** `config.json` 中 `scan_ignore: ["target", "my-private"]`
- **AND** 调用 `config::effective_scan_ignore(&root)`
- **THEN** 返回的列表恰为 `["target", "my-private"]`
- **AND** 不包含 `.git` / `node_modules` 等内置默认值

#### Scenario: 空数组明确表示不忽略任何目录

- **WHEN** `config.json` 中 `scan_ignore: []`
- **AND** 调用 `config::effective_scan_ignore(&root)`
- **THEN** 返回空 `Vec<String>`
- **AND** discover 时不会跳过任何子目录（即使是 `.git`）

---

### Requirement: Skill 内容按需加载

`Skill` 结构在 discover 阶段 SHALL NOT 加载 SKILL.md 的 markdown 正文，仅解析 frontmatter；正文通过 `Skill::read_body() -> Result<String>` 在调用方明确需要时按需读取。

#### Scenario: discover 不读 body

- **WHEN** SKILL.md 文件大小 1 MB（含大量正文）
- **AND** 调用 `discover::skills_in`
- **THEN** 返回的 Skill 实例不持有 body 字符串
- **AND** 进程内存增长仅与 frontmatter + 路径相关，与正文长度无关

#### Scenario: read_body 按需读取

- **WHEN** 在 discover 之后调用 `skill.read_body()`
- **THEN** 系统打开对应 SKILL.md 文件
- **AND** 返回 frontmatter 之后（第二个 `---` 行之后）的全部内容
- **AND** 文件不存在或读取失败时返回 `Err(Error::Io(_))`

---

### Requirement: paam track skills 命令

`paam track skills <alias>` SHALL 根据 alias 解析到本地缓存目录（`~/.paam/sources/<alias>/`），调用 `discover::skills_in` 扫描，并以表格形式输出三列 NAME / DESCRIPTION / PATH。DESCRIPTION 单行截断到 60 字符（按 `chars().count()`），末尾加 `…`。alias 不存在时 SHALL 返回非零 exit code 并给出明确错误。

#### Scenario: 列出仓内 skill

- **WHEN** alias `github.com/foo/bar` 已订阅，本地仓含 3 个合法 SKILL.md
- **AND** 用户执行 `paam track skills github.com/foo/bar`
- **THEN** 输出包含表头 `NAME  DESCRIPTION  PATH` 与 3 行记录
- **AND** 命令以 exit code 0 结束

#### Scenario: 仓内无 skill 时友好提示

- **WHEN** alias `github.com/foo/empty` 已订阅，本地仓不含任何 SKILL.md
- **AND** 用户执行 `paam track skills github.com/foo/empty`
- **THEN** 输出明确提示"该订阅源中暂无可用的 Skill"
- **AND** 命令以 exit code 0 结束（不视为错误）

#### Scenario: alias 不存在时报错

- **WHEN** 用户执行 `paam track skills <未订阅的 alias>`
- **THEN** 系统返回非零 exit code
- **AND** 错误信息提示该 alias 未订阅，建议用 `paam track list` 查看
