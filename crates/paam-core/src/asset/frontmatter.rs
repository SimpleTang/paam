//! 解析 SKILL.md（以及未来 PROMPT.md 等）顶部的 YAML frontmatter。
//!
//! frontmatter 是被 `---` 单行包围的 YAML 块，必须出现在文件第一行。
//! M1 仅识别 `name` / `description` 必填字段；其余字段透明保留到 `extra`。

use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

/// 解析后的 frontmatter 数据。
#[derive(Debug, Clone, Deserialize)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,

    /// 未识别的字段透明保留，便于 ③ install 时复用 / 调试。
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml_ng::Value>,
}

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("文件不含 frontmatter 段（首行不是 `---` 或缺少结尾 `---` 行）")]
    Missing,

    #[error("YAML 解析失败：{0}")]
    Yaml(#[from] serde_yaml_ng::Error),

    #[error("frontmatter 缺少必填字段：{0}")]
    MissingRequired(&'static str),
}

/// 从 SKILL.md 全文中切出 frontmatter 段并解析。
///
/// 期望文件首行为 `---`，紧随其后的若干行 YAML，再以单独 `---` 行结束。
pub fn parse(content: &str) -> Result<Frontmatter, FrontmatterError> {
    let yaml_block = extract_yaml_block(content).ok_or(FrontmatterError::Missing)?;
    let fm: Frontmatter = serde_yaml_ng::from_str(yaml_block)?;
    if fm.name.trim().is_empty() {
        return Err(FrontmatterError::MissingRequired("name"));
    }
    if fm.description.trim().is_empty() {
        return Err(FrontmatterError::MissingRequired("description"));
    }
    Ok(fm)
}

/// 切出 frontmatter 内的 YAML 文本（不含两侧 `---` 标记行）。
fn extract_yaml_block(content: &str) -> Option<&str> {
    let trimmed_start = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    // 找到结尾的 "---" 行（必须独占一行）
    let mut offset = 0usize;
    for line in trimmed_start.split_inclusive('\n') {
        let line_no_eol = line.trim_end_matches(['\n', '\r']);
        if line_no_eol == "---" {
            return Some(&trimmed_start[..offset]);
        }
        offset += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_frontmatter() {
        let content = "---\nname: pdf-review\ndescription: Review PDF documents\n---\n\n# body\n";
        let fm = parse(content).unwrap();
        assert_eq!(fm.name, "pdf-review");
        assert_eq!(fm.description, "Review PDF documents");
        assert!(fm.extra.is_empty());
    }

    #[test]
    fn parse_preserves_unknown_fields() {
        let content = r#"---
name: pdf-review
description: Review PDF documents
when_to_use: 当需要审查 PDF 时
required_permissions:
  - read_files
---
body
"#;
        let fm = parse(content).unwrap();
        assert_eq!(fm.name, "pdf-review");
        assert!(fm.extra.contains_key("when_to_use"));
        assert!(fm.extra.contains_key("required_permissions"));
    }

    #[test]
    fn parse_rejects_missing_name() {
        let content = "---\ndescription: only desc\n---\nbody\n";
        let err = parse(content).unwrap_err();
        // 实际上 serde 会先捕获缺少 name 字段（必填），返回 Yaml；这里两类都接受
        assert!(matches!(
            err,
            FrontmatterError::MissingRequired("name") | FrontmatterError::Yaml(_)
        ));
    }

    #[test]
    fn parse_rejects_missing_description() {
        let content = "---\nname: foo\n---\nbody\n";
        let err = parse(content).unwrap_err();
        assert!(matches!(
            err,
            FrontmatterError::MissingRequired("description") | FrontmatterError::Yaml(_)
        ));
    }

    #[test]
    fn parse_rejects_empty_name() {
        let content = "---\nname: \"\"\ndescription: x\n---\n";
        assert!(matches!(
            parse(content).unwrap_err(),
            FrontmatterError::MissingRequired("name")
        ));
    }

    #[test]
    fn parse_rejects_missing_frontmatter_block() {
        let content = "no frontmatter here\n";
        assert!(matches!(
            parse(content).unwrap_err(),
            FrontmatterError::Missing
        ));
    }

    #[test]
    fn parse_rejects_unterminated_block() {
        let content = "---\nname: foo\ndescription: bar\nno closing marker\n";
        assert!(matches!(
            parse(content).unwrap_err(),
            FrontmatterError::Missing
        ));
    }

    #[test]
    fn parse_rejects_yaml_syntax_error() {
        let content = "---\nname: foo\n  description: invalid indent\n---\n";
        assert!(matches!(
            parse(content).unwrap_err(),
            FrontmatterError::Yaml(_)
        ));
    }
}
