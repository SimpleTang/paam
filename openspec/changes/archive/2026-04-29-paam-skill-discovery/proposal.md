## Why

前两个 change 让 paam 能"订阅一个 git 仓"，但仓里有什么 paam 还看不见。本 change 让 paam **看见**——递归扫描已 track 的 source 仓，按 `SKILL.md` marker 识别出可装的 skill 列表，解析其 YAML frontmatter，对外暴露 `paam track skills <alias>` 命令。完成本步是 ③ paam-install 能选目标的前提。

同时，本 change 把 `Asset` trait 的形状定型——它在 paam-foundation-track 阶段是空骨架（4 个 getter，无实现者），现在要写第一个具体实现 `Skill` 时必须明确 trait 的最终边界，否则 ③/④ 会反复重构。借此机会**修订 ADR-0007 §7**，让纸面上的 trait 草案与 paam-core 实际实现对齐。

## What Changes

- **修订 ADR-0007 §7「核心库资产抽象」**：把 trait 草案从"`name() / asset_type() / marker_file() / discover_in() / install_to()`"改为"`id() / kind() / source_alias() / relative_path()`（4 个 getter），动作下放到模块级函数"。理由：保持 trait object-safe，让 `Vec<Box<dyn Asset>>` 永远可用；marker file 改用关联常量更准确
- 新增 `paam-core::asset::skill::Skill` 类型，第一个 `Asset` trait 实现者
- 新增 `paam-core::asset::frontmatter` 模块：YAML frontmatter 解析；必填 `name` / `description`；未知字段透明保留到 `extra: HashMap<String, serde_yaml_ng::Value>`
- 新增 `paam-core::discover` 顶级模块：`pub fn skills_in(repo: &Path, ignore: &[String]) -> Vec<Skill>`
- 递归扫描含 `SKILL.md` 的目录；目录名匹配 ignore 列表则跳过整棵子树
- 内置默认 ignore 列表：`.git` / `.github` / `.gitlab` / `node_modules` / `target` / `.idea` / `.vscode` / `dist` / `build` / `__pycache__` / `.next` / `.cache`
- 配置文件 schema 新增可选字段 `scan_ignore: Option<Vec<String>>`：用户级覆盖**完全替换**内置默认（不合并）；缺省 / null / 空 → 用默认；空数组 → 不忽略任何目录
- 异常处理（按"宽松扫描 + 友好提示"原则）：
  - 未知 frontmatter 字段 → 保留到 `extra` + `tracing::warn!` 提示
  - 必填字段缺失 → 跳过该目录的 SKILL.md + stderr 打印 warning
  - YAML 解析失败 → 同上 跳过 + warning
- `Skill::read_body() -> Result<String>`：按需加载 markdown 正文（discover 阶段不读，避免大量 IO）
- 新增 CLI 命令 `paam track skills <alias>`：从 alias 找到本地 clone 路径 → 调 `discover::skills_in` → 表格输出 NAME / DESCRIPTION / PATH 三列；DESCRIPTION 截断 60 字符；空列表友好提示
- 引入新依赖 `serde_yaml_ng`（活跃 fork，仅用于解析 SKILL.md frontmatter——这是 Anthropic Skills 外部数据格式，paam 无选择权）

**明确不做：**

- 不实现 install / list / sync（推到 ③ / ④）
- 不持久化 discovered skills（每次按需扫描）
- 不引入 `.paam-ignore` 文件 / gitignore 风格 glob（M2）
- 不解决跨 source 同名 skill 冲突（M1 discover 不处理；③ install 时再决定）
- 不结构化解析 Anthropic Skills 规范的其它字段（when_to_use / required_permissions / model 等），全部透明保留到 `extra`

## Capabilities

### New Capabilities

- `asset-management`：定义 `Asset` trait 行为、`AssetKind` 枚举、`Skill` 类型语义、SKILL.md frontmatter 字段约定、discover 扫描行为、`scan_ignore` 配置、`paam track skills <alias>` 命令、错误处理策略

### Modified Capabilities

（无。`source-management` 不动。）

## Impact

**代码：**
- `crates/paam-core/src/asset/mod.rs`：在已有 `Asset` trait 文档中加修订说明（trait 形状不变）；导出 `Skill`
- `crates/paam-core/src/asset/skill.rs`：新增（`Skill` struct + `impl Asset` + `read_body()`）
- `crates/paam-core/src/asset/frontmatter.rs`：新增（YAML frontmatter 解析）
- `crates/paam-core/src/discover/mod.rs`：新增（`skills_in` 主入口 + 内置默认 ignore + 递归 walker）
- `crates/paam-core/src/config/schema.rs`：`Config` 加 `scan_ignore: Option<Vec<String>>` 字段
- `crates/paam-core/src/config/mod.rs`：新增 `effective_scan_ignore(root: &PaamRoot) -> Result<Vec<String>>`（合并默认或用户覆盖）
- `crates/paam-cli/src/main.rs`：`Cmd::Track` 增加 `skills <alias>` 子分支处理 + 输出格式化

**依赖：**
- 新增 `serde_yaml_ng = "0.10"`（workspace + paam-core）—— 仅用于 SKILL.md frontmatter；不影响 `~/.paam/config.json`（仍 JSON）
- `walkdir`（M1 阶段为简化扫描实现，可考虑引入；或自写 std::fs 递归）—— 在 design 决策中确定

**ADR：**
- `.dev/docs/decisions/0007-phase-extension-design.md` §7 加修订段：原 trait 草案被本 change 修订；记录新 trait 形状与理由

**对后续 change 的契约：**
- ③ paam-install：拿到 `Vec<Skill>` 后用模块级函数 `install::skill_to(&skill, dest, root)` 落地，不动 trait
- ④ paam-claude-sync：sync 不依赖 Asset trait（按 metadata 中的安装记录直接 symlink）
- M2 引入 Prompt：新增 `asset::prompt::Prompt` 类型 + `discover::prompts_in()`，平行于 Skill；`AssetKind` 加 variant 不破坏调用方
