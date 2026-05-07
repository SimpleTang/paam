//! `Skill` 类型：第一个 `Asset` trait 实现者。
//!
//! 由 `paam-core::discover::skills_in` 在扫描时构造；`read_body` 按需读取
//! markdown 正文。SKILL.md 的 frontmatter 字段约定见 ADR-0007 §7（修订版）
//! 与 paam-skill-discovery change 的 design.md。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::asset::{Asset, AssetKind};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Skill {
    source_alias: String,
    relative_path: PathBuf,
    name: String,
    description: String,
    extra: HashMap<String, serde_yaml_ng::Value>,
}

impl Skill {
    /// SKILL.md 是 Skill 的 marker 文件名（约定见 ADR-0007 §1）。
    pub const MARKER: &'static str = "SKILL.md";

    pub(crate) fn new(
        source_alias: String,
        relative_path: PathBuf,
        name: String,
        description: String,
        extra: HashMap<String, serde_yaml_ng::Value>,
    ) -> Self {
        Self {
            source_alias,
            relative_path,
            name,
            description,
            extra,
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn extra(&self) -> &HashMap<String, serde_yaml_ng::Value> {
        &self.extra
    }

    /// 按需读取 SKILL.md 的 markdown 正文（去除 frontmatter 段）。
    ///
    /// `repo_root` 是订阅源在本地的根目录（如 `~/.paam/sources/<alias>/`）。
    pub fn read_body(&self, repo_root: &Path) -> Result<String> {
        let skill_md = repo_root.join(&self.relative_path).join(Self::MARKER);
        let content = std::fs::read_to_string(&skill_md)?;
        Ok(strip_frontmatter(&content).to_string())
    }
}

impl Asset for Skill {
    fn id(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> AssetKind {
        AssetKind::Skill
    }

    fn source_alias(&self) -> &str {
        &self.source_alias
    }

    fn relative_path(&self) -> &Path {
        &self.relative_path
    }
}

/// 切掉文件首部的 `---\n...\n---\n` frontmatter 段，返回正文。
/// 若文件首行不是 `---`，则视为没有 frontmatter，整个文件即正文。
fn strip_frontmatter(content: &str) -> &str {
    let after_first = match content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    {
        Some(rest) => rest,
        None => return content,
    };
    let mut offset = 0usize;
    for line in after_first.split_inclusive('\n') {
        let line_no_eol = line.trim_end_matches(['\n', '\r']);
        if line_no_eol == "---" {
            // body starts after this closing line
            return &after_first[offset + line.len()..];
        }
        offset += line.len();
    }
    // 没找到结尾标记：保守处理，返回原文
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_skill(rel: &str, name: &str, desc: &str) -> Skill {
        Skill::new(
            "github.com/foo/bar".into(),
            PathBuf::from(rel),
            name.into(),
            desc.into(),
            HashMap::new(),
        )
    }

    #[test]
    fn asset_trait_methods() {
        let skill = make_skill("tools/pdf-review", "pdf-review", "Review PDFs");
        assert_eq!(skill.id(), "pdf-review");
        assert_eq!(skill.kind(), AssetKind::Skill);
        assert_eq!(skill.source_alias(), "github.com/foo/bar");
        assert_eq!(skill.relative_path(), Path::new("tools/pdf-review"));
        assert_eq!(skill.description(), "Review PDFs");
        assert_eq!(Skill::MARKER, "SKILL.md");
    }

    #[test]
    fn read_body_returns_markdown_after_frontmatter() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("tools/pdf-review");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: pdf-review\ndescription: x\n---\n\n# Body\nHello\n",
        )
        .unwrap();

        let skill = make_skill("tools/pdf-review", "pdf-review", "x");
        let body = skill.read_body(dir.path()).unwrap();
        assert!(body.starts_with("\n# Body"), "实际 body: {:?}", body);
        assert!(body.contains("Hello"));
    }

    #[test]
    fn read_body_when_no_frontmatter_returns_whole_file() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("plain");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "no frontmatter here\n").unwrap();

        let skill = make_skill("plain", "x", "x");
        let body = skill.read_body(dir.path()).unwrap();
        assert_eq!(body, "no frontmatter here\n");
    }
}
