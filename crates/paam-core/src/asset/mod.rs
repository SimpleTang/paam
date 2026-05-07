//! type-agnostic asset 抽象（ADR-0007 §7，paam-skill-discovery 修订）。
//!
//! - `Asset` trait 仅含纯 getter（id / kind / source_alias / relative_path），保持 object-safe
//! - `AssetKind` 枚举类型（`#[non_exhaustive]`，M2/M3 加 variant 不破坏调用方）
//! - 每个具体类型用关联常量声明 marker（如 `Skill::MARKER`）
//! - 动作（discover / install / ...）下放到模块级函数，不挂在 trait 上
//!
//! 当前已有的实现者：
//! - `Skill`（paam-skill-discovery 落地）
//!
//! 后续：`Prompt`（M2）、`Mcp`（M3），按平行模式增加。

pub mod frontmatter;
pub mod skill;

pub use skill::Skill;

use std::path::Path;

/// 一个可被订阅、安装并分发到目标 Agent 工作目录的资产。
///
/// 当前 paam-core 中没有该 trait 的实现者；此处仅提供形状。
pub trait Asset {
    /// 在所属 source 内唯一的标识符（如 SKILL.md 中声明的 name 字段）。
    fn id(&self) -> &str;

    /// 资产类型枚举。
    fn kind(&self) -> AssetKind;

    /// 来自哪个订阅源 alias。
    fn source_alias(&self) -> &str;

    /// 在源仓本地缓存目录内的相对路径。
    fn relative_path(&self) -> &Path;
}

/// 资产类型。
///
/// 标记为 `#[non_exhaustive]`，允许后续在不破坏调用方的前提下加入
/// `Prompt` / `Mcp` 等新 variant（M2 milestone 计划事项）。
///
/// 序列化为小写形式（`"skill"`），与 ADR-0007 §5 metadata schema 一致。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Skill,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 编译期断言：`Asset` 必须是 object-safe，
    /// 后续 change 可能用 `Vec<Box<dyn Asset>>` / `&dyn Asset` 持有混合资产。
    #[allow(dead_code)]
    fn _assert_object_safe(_: &dyn Asset) {}
}
