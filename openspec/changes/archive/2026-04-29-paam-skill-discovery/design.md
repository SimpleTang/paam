## Context

paam 现状：
- paam-foundation-track 落地了 `Asset` trait 骨架（4 个 getter，无实现者；object-safe；编译期断言）
- swap-git-transport-to-cli 切走了 git 后端，所有 git 操作走系统子进程，paam-core 不再依赖 git2
- `~/.paam/config.json` 已稳定（v1 schema：`version` + `sources`）

ADR-0007 §7 在 2026-04 时给出过 Asset trait 草案，但其中：
- `marker_file(&self) -> &'static str` 实例方法没意义（每个 Skill 实例返回同一字符串）
- `discover_in(repo) -> Vec<Self> where Self: Sized` 静态方法**让 trait 不再 object-safe**
- `install_to(&self, dest)` 在 ② 阶段没有任何调用者

paam-foundation-track 在落地骨架时刻意没采用 ADR 草案的形态——选择"4 个 getter（object-safe）+ 动作下放到模块函数"，并保留这个决定为本 change 的待办（修订 ADR）。

本 change 必须做的事：
1. 写 ADR-0007 修订段（或单独 ADR-0009）—— 让纸面与代码对齐
2. 第一次 impl Asset for Skill —— 验证 trait 形状真的够用
3. 引入 SKILL.md 解析与 discover 扫描 —— ③ install 的输入源
4. 让用户能 `paam track skills <alias>` 看见仓里有什么 —— dogfood / 信任建立

## Goals / Non-Goals

**Goals:**

- 把 `Asset` trait 形状定型，对齐 ADR-0007 与代码实现
- 落地 `Skill` 第一个 `Asset` 实现者，验证 trait 极简形态够用
- 落地 `discover::skills_in(repo, ignore)`：递归扫描 SKILL.md，宽松解析（按需保留未知字段，缺必填则 skip）
- 落地 `paam track skills <alias>` CLI，给用户看见 source 仓内容
- 让 ② 的 discover 与 git transport 完全解耦——测试用 tempdir 即可，不依赖 system git

**Non-Goals:**

- 不实现 install / list / sync（推到 ③ / ④）
- 不持久化 discovered skills（每次扫描）
- 不结构化解析 Anthropic Skills 规范的其它字段（透明保留即可）
- 不引入 `.paam-ignore` 文件 / glob（M2）
- 不解决跨 source 同名冲突（③ install 时再决定）
- 不破坏现有 `Asset` trait 形状或 object-safety

## Decisions

### 1. Asset trait 形状定型（修订 ADR-0007 §7）

**选择：** 保持 paam-foundation-track 落地的 4 个 getter 形态，**不加** `marker_file()` / `discover_in()` / `install_to()` 实例方法。

```rust
pub trait Asset {
    fn id(&self) -> &str;
    fn kind(&self) -> AssetKind;
    fn source_alias(&self) -> &str;
    fn relative_path(&self) -> &Path;
}
```

每个具体类型用**关联常量**声明 marker：

```rust
impl Skill {
    pub const MARKER: &'static str = "SKILL.md";
}
```

discover / install 等"动作"放模块级函数：

```rust
// 本 change 落地
pub fn discover::skills_in(repo: &Path, ignore: &[String]) -> Vec<Skill>;

// ③ change 落地（占位，本 change 不实现）
// pub fn install::skill_to(skill: &Skill, dest: &Path, root: &PaamRoot) -> Result<()>;
```

**Why：**

- **trait 必须 object-safe**：M2 引入 Prompt / Mcp 后，调用方常需要 `Vec<Box<dyn Asset>>` 持有混合类型；ADR 草案的 `discover_in -> Vec<Self> where Self: Sized` 直接破坏 object-safety
- **关联常量比实例方法更准确**：`Skill::MARKER` 表达"所有 Skill 实例的 marker 一致"，比 `skill.marker_file()` 在每个实例上重复 hard-code 同一字符串更干净
- **动作不挂 trait** = trait 是纯数据描述，行为按类型在模块里实现；M2 加 Prompt 时新建 `discover::prompts_in()` 与 `install::prompt_to()`，与 Skill 完全平行，不需要修改 trait
- **install 推到 ③**：本 change 没有 install 调用者，trait 上加 `install_to` 是死代码，反而要在 ③ 时重构；模块函数形态可以等 ③ 真正需要时再加

**契约固化：** 修订后的 trait 是**最终形态**——`Asset` trait 在 M1 余下 change 与 M2 / M3 中**只允许新增 getter**（且新 getter 必须是 object-safe 的；不允许 `where Self: Sized`、不允许 generic method）。任何"动作"都通过模块级函数表达。

### 2. ADR-0007 修订方式

**选择：** 在 `.dev/docs/decisions/0007-phase-extension-design.md` §7 末尾追加 `### 修订（2026-04-29，由 paam-skill-discovery 落地）` 段，原文不删除（保留历史可追溯），但加显式 "Superseded by below" 标注。同时更新 ADR 顶部 metadata 的 `Last-Reviewed` 日期。

**Why：**

- ADR 是"决策记录"，不是规范文档——历史草案是真实发生过的思考过程，删掉等于伪造路径
- 加修订段比开新 ADR-0009 更合适：决策主题没变（仍是"核心库资产抽象"），只是细化；开新 ADR 会让"类型扩展"主题分散在两个 ADR 里
- M2 / M3 加新 asset 类型时，开发者读 ADR-0007 能直接看到"trait 形状是这样定的，原因是这些"

### 3. YAML parser 依赖：`serde_yaml_ng`

**选择：** 在 workspace 与 paam-core 加 `serde_yaml_ng = "0.10"`（最新版），仅用于 SKILL.md frontmatter 解析。`~/.paam/config.json` 与 `~/.paam/sources/<alias>/config.json`（如果未来有）仍用 `serde_json`。

**Why：**

- SKILL.md frontmatter 是 **Anthropic Agent Skills 规范** 的外部数据格式，paam 没有选择权
- 官方 `serde_yaml` 已停维护；`serde_yaml_ng` 是社区活跃 fork，API 完全兼容，无迁移成本
- 与之前"paam 配置文件用 JSON"的决定不冲突——这是输入数据格式，不是 paam 自己的存储格式

**Risk：** `serde_yaml_ng` 维护断层 → 缓解：本 change 只用最基础 derive 序列化，必要时切回 `serde_yaml` 或 `yaml-rust2` 几乎零成本。

### 4. SKILL.md frontmatter schema 与未知字段策略

**最小 schema：**

```rust
#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml_ng::Value>,
}
```

**Why：**

- M1 仅识别 `name` + `description`，因为这是用户填写率 100% 的字段，且足以驱动 `paam track skills` 列表与 ③ install 时的标识符选择
- `extra` 字段用 `#[serde(flatten)]` 接收所有未知字段，三个用途：
  1. ③ install 时若需保留 SKILL.md 原文（按 ADR-0007），不依赖 `extra`，但有它在便于程序逻辑层查询
  2. M2 评估"哪些字段值得结构化"时，可以从 `extra` 实测使用频率
  3. 调试时输出 `extra.keys()` 让用户确认 paam 看到了什么
- **未知字段同时 `tracing::warn!`**：单条 warning 含 `path = "<rel>/SKILL.md"` + `unknown_keys = ["foo", "bar"]`；不重复打印每个 key

### 5. 必填字段缺失策略：宽松跳过 + warning

**选择：** 解析失败（必填缺失 / YAML 语法错误 / IO 错误）时**跳过该目录的 SKILL.md，不影响整体扫描**；通过 stderr 输出 `warning: <path>/SKILL.md ...` 一行。

**Why：**

- discover 是"信息呈现"操作，不是"事务性更新"——一个 skill 写错不应该让用户看不到其它 skill
- 友好 warning 让用户知道发生了什么，能自己去修
- 行为参考：linter 默认就是"continue on error + collect warnings"

**消息样例：**

```
warning: tools/broken-skill/SKILL.md 缺少 name 字段，已跳过
warning: tools/yaml-error/SKILL.md frontmatter 解析失败：mapping values are not allowed here at line 3, column 9
```

### 6. scan_ignore 配置语义：完全替换（不合并）

**选择：** 当 `~/.paam/config.json` 中存在 `scan_ignore` 字段时，其值**完全替换** paam-core 内置默认列表；不存在 / null → 用内置默认；空数组 `[]` → 不忽略任何目录。

```jsonc
{
  "version": 1,
  "sources": [...],
  "scan_ignore": [".git", "node_modules", "my_custom_dir"]  // 完全替换默认
}
```

**Why：**

- 合并语义会让用户困惑："我没在配置里写 .git，为啥还是被忽略了？"
- 替换语义可预测：config.json 里看到什么就是什么
- 用户若想"在默认基础上加一个"，只能完整 copy 默认列表 + 加自己的——这是显式选择，可接受
- 空数组 `[]` 是合法语义（开发者可能想 audit 整个仓含 SKILL.md 的目录），与"省略字段"明确区分

**契约固化：** 内置默认列表暴露为 `paam-core::discover::DEFAULT_IGNORE` 常量，方便用户从代码 / 文档中 copy。

### 7. discover 与 git transport 完全解耦

**选择：** `discover::skills_in(repo: &Path, ignore: &[String]) -> Vec<Skill>` 接受任意 `&Path`——只要它指向一个含 `SKILL.md` 的目录树即可，不假设是 git 仓。CLI 层负责把 alias 解析为本地路径。

**Why：**

- discover 是纯文件系统操作，与 git 无关；强行耦合会让单测被迫造 git fixture
- 测试只用 `tempfile::TempDir` + `std::fs::write` 写 SKILL.md，毫秒级、无外部进程
- 未来若 paam 接入"非 git 来源"（本地目录、HTTP zip 等），discover 不需要改
- `paam track skills <alias>` 在 CLI 层做：`alias → ~/.paam/sources/<alias>/ → discover::skills_in(...)`

**契约固化：** `paam-core::discover` 模块**不允许**依赖 git / source / config 中的任何业务上下文；它的 input 是 path + ignore，output 是 `Vec<Skill>`，纯函数（除文件系统读取与日志外无副作用）。

### 8. 目录扫描实现：自写递归 vs 依赖 walkdir

**选择：** 自写 `std::fs::read_dir` 递归（不引入 `walkdir`）。

**Why：**

- 扫描逻辑很简单：BFS 遍历目录，跳过 `entry.file_name() in ignore`，发现 `SKILL.md` 即解析
- `walkdir` 是优秀 crate，但本场景只用到它最基础的"递归 + 跳过目录"功能，30 行 Rust 自写完全够
- 减少依赖 = 减少编译时间 + 安全审查面（与 swap-git-transport-to-cli 减依赖思路一致）
- 自写版本可以写 trace 日志，调试更方便

**Risk：** 处理 symlink / 循环引用？M1 不处理（不跟随 symlink，直接跳过）；M2 若 dogfood 暴露问题再加。

### 9. `paam track skills` CLI 输出格式

**选择：** 三列固定表格 `NAME / DESCRIPTION / PATH`：

```
NAME             DESCRIPTION                                                    PATH
code-review      Review code for common issues and suggest improvements          code-review/
pdf-review       Review PDF documents and extract key information                tools/pdf-review/
how-to-write     编写自定义 paam Skill 的教程，包含 frontmatter 字段说明…             docs/how-to-write/
```

- DESCRIPTION 单行截断到 60 字符末尾加 `…`（中文按字符计算，不按字节）
- PATH 是相对源仓根的目录（含 `SKILL.md` 的目录），结尾 `/` 表强调"目录"
- 列宽自适应（按当前 terminal 宽度），DESCRIPTION 列最后被截断

**空列表友好提示：**

```
该订阅源中暂无可用的 Skill（未发现任何 SKILL.md 文件）
提示：可用 `paam track skills <alias> --verbose` 查看扫描细节（含跳过的目录）
```

**Why：**

- DESCRIPTION 截断保证一行内显示完整记录，不破坏表格视觉
- `--verbose` 提示先放在友好消息里——M1 阶段先不实现 verbose，M2 再加（这条提示是给 M2 留信号的预期管理）
- 不显示 commit hash（用户决策 f）

### 10. CLI 命令树扩展：`paam track skills <alias>`

**选择：** 在 `Cmd::Track(TrackArgs)` 内部，把单参数 `target: String` 改造为支持三种用法：

| 输入 | 解释 |
|---|---|
| `paam track <ssh-url>` | 添加新订阅源（保持原行为） |
| `paam track list` | 列出已订阅源（保持原行为） |
| `paam track skills <alias>` | 列出 alias 对应仓内的 skills（**本 change 新增**） |

实现：把 `TrackArgs::target` 改为 `Vec<String>`，根据元素数与首个 token 区分：

- 1 个 token == "list" → list 子命令
- 1 个 token != "list" → 视为 ssh-url，走 add 路径
- 2 个 token，首个 == "skills" → 视为 `skills <alias>`
- 其它 → InvalidUsage 错误，打印用法

**Why：**

- 与现有"位置参数 vs `list` 字面量"模式一致；不破坏 spec 中已有 scenario
- clap 不支持"位置参数 + 可选子子命令"混合，自手分派比硬塞 clap 子命令更直观
- 三种用法的 help 文本在 `paam track --help` 中明确

**Alternative considered:** 用 clap 真正的子子命令 `paam track add <url>` / `paam track list` / `paam track skills <alias>`——拒绝。会破坏现有 spec 中 `paam track <ssh-url>` 的简化用法（用户被迫每次写 add）。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| `serde_yaml_ng` 维护断层 | 仅用基础 derive；切回 serde_yaml 或换 yaml-rust2 零成本 |
| 自写递归在大仓上慢于 walkdir | M1 dogfood 仓 < 100 个文件，无感；M2 若 1000+ skills 性能成瓶颈再换 |
| 不跟随 symlink 可能漏掉用户故意 link 进来的 skill 目录 | M1 接受；ADR-0007 也未要求跟随 symlink；M2 评估 |
| 用户写错 SKILL.md 频繁 → 大量 stderr warning 噪音 | 行数 ≤ skill 总数；M2 可加 `--quiet` 抑制 |
| `extra` 字段 `serde_yaml_ng::Value` 类型暴露在公共 API（`Skill::extra`），未来 yaml lib 切换会破坏 API | M1 接受；可考虑用 newtype 包一下，但 ② 阶段过度抽象 |
| 修订 ADR 不彻底，未来读者仍按草案设计 | 修订段加显式 "Superseded by below" 标注；M1-retro 复盘 ADR 流程是否需要更严格 |
| `paam track skills` 输出列宽自适应在窄终端会破版 | M1 接受；用 `terminal_size` 不引入新 crate 即可（M2 看是否要） |

## Migration Plan

**配置文件 schema 变更：**

`Config` 加 `scan_ignore: Option<Vec<String>>` 字段（serde 默认 None）。这是**向后兼容**变更：

- 现有 `config.json`（仅 `version` + `sources`）能正常 deserialize（缺字段视为 None → 用默认）
- `serde` 默认会跳过 None 字段输出（用 `#[serde(skip_serializing_if = "Option::is_none")]`），保持 config.json 简洁
- schema `version` 不 bump（仍 v1）

**ADR-0007 修订：**

- 不删除原文，加修订段
- ADR 顶部 metadata 加 `Last-Reviewed: 2026-04-29`

**回滚：**

`git revert` 整个 commit 即可；不影响已有 sources / config（`scan_ignore` 字段被 deserialize 后 silently dropped）。

## Open Questions

1. **是否在 stderr warning 前加 `[paam]` 前缀？**——参考 cargo / rustc 风格"warning: ..." 已足够清晰；M1 不加前缀，M2 若与子进程 stderr 混杂时再考虑。
2. **`Skill::extra` 是否值得用 newtype 包装？**——M1 直接用 `HashMap<String, serde_yaml_ng::Value>`，让公共 API 暴露 `serde_yaml_ng` 依赖类型；M2 评估 yaml lib 是否要切换时再做。
3. **DESCRIPTION 中文截断按"字符"还是"字形簇 / grapheme"**？——M1 按 `chars().count()`；emoji ZWJ 序列等罕见 case 接受；M2 评估是否要 `unicode-segmentation`。
