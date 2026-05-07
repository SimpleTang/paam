//! 资产发现：递归扫描已 clone 的 source 仓，按 marker 文件识别 skill / prompt / mcp。
//!
//! M1 阶段仅实现 `skills_in`（按 `SKILL.md` 识别）。本模块**不依赖** git / config /
//! source 等业务上下文：input 是路径，output 是资产列表（除文件系统读取与日志外
//! 无副作用），便于解耦与单测。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::asset::frontmatter::{self, FrontmatterError};
use crate::asset::Skill;

/// 内置默认的扫描忽略目录列表（按 file_name 完全匹配）。
///
/// 用户可在 `~/.paam/config.json` 通过 `scan_ignore` 字段**完全替换**此列表
/// （不合并）。
pub const DEFAULT_IGNORE: &[&str] = &[
    ".git",
    ".github",
    ".gitlab",
    "node_modules",
    "target",
    ".idea",
    ".vscode",
    "dist",
    "build",
    "__pycache__",
    ".next",
    ".cache",
];

/// 在已 clone 的 source 仓中递归扫描所有 SKILL.md，解析为 `Skill` 实例。
///
/// - `repo`: 源仓本地根目录（`~/.paam/sources/<alias>/`）
/// - `source_alias`: 该源在 paam.yaml 中的 alias，写入每个 Skill 的 `source_alias`
/// - `ignore`: 跳过的目录名列表（按 file_name 完全匹配；通常由
///   `config::effective_scan_ignore` 提供）
///
/// 单个 SKILL.md 解析失败（缺必填 / YAML 错误等）时跳过该目录并通过 stderr 输出
/// warning，整体扫描继续。
pub fn skills_in(repo: &Path, source_alias: &str, ignore: &[String]) -> Vec<Skill> {
    let mut out = Vec::new();
    walk(repo, repo, source_alias, ignore, &mut out);
    out
}

fn walk(
    repo_root: &Path,
    current: &Path,
    source_alias: &str,
    ignore: &[String],
    out: &mut Vec<Skill>,
) {
    // 优先在当前目录探测 SKILL.md
    let skill_md = current.join(Skill::MARKER);
    if skill_md.is_file() {
        match try_parse_skill(repo_root, current, source_alias, &skill_md) {
            Ok(skill) => out.push(skill),
            Err(msg) => {
                eprintln!("warning: {}", msg);
            }
        }
    }

    // 继续递归子目录
    let entries = match std::fs::read_dir(current) {
        Ok(it) => it,
        Err(e) => {
            tracing::debug!(path = %current.display(), error = %e, "read_dir 失败，跳过");
            return;
        }
    };
    for entry in entries.flatten() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        // 不跟随 symlink，避免循环
        if meta.file_type().is_symlink() {
            continue;
        }
        if !meta.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if ignore.iter().any(|skip| skip == name_str.as_ref()) {
            tracing::debug!(skipped = %name_str, "ignore 匹配");
            continue;
        }
        walk(repo_root, &entry.path(), source_alias, ignore, out);
    }
}

fn try_parse_skill(
    repo_root: &Path,
    skill_dir: &Path,
    source_alias: &str,
    skill_md: &Path,
) -> Result<Skill, String> {
    let rel = skill_dir
        .strip_prefix(repo_root)
        .map(PathBuf::from)
        .unwrap_or_else(|_| skill_dir.to_path_buf());
    let display_path = format!("{}/{}", rel.display(), Skill::MARKER);

    let content = std::fs::read_to_string(skill_md)
        .map_err(|e| format!("{} 读取失败：{}", display_path, e))?;

    let fm = match frontmatter::parse(&content) {
        Ok(fm) => fm,
        Err(FrontmatterError::Missing) => {
            return Err(format!(
                "{} 缺少 frontmatter 段（首行需为 `---`，并以 `---` 行结束），已跳过",
                display_path
            ));
        }
        Err(FrontmatterError::MissingRequired(field)) => {
            return Err(format!("{} 缺少必填字段 `{}`，已跳过", display_path, field));
        }
        Err(FrontmatterError::Yaml(e)) => {
            return Err(format!(
                "{} frontmatter 解析失败：{}，已跳过",
                display_path, e
            ));
        }
    };

    if !fm.extra.is_empty() {
        let unknown_keys: Vec<&String> = fm.extra.keys().collect();
        tracing::warn!(
            path = %display_path,
            unknown_keys = ?unknown_keys,
            "SKILL.md frontmatter 含未识别字段，已透明保留到 extra"
        );
    }

    let extra: HashMap<String, serde_yaml_ng::Value> = fm.extra;
    Ok(Skill::new(
        source_alias.to_string(),
        rel,
        fm.name,
        fm.description,
        extra,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetKind};
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn default_ignore_owned() -> Vec<String> {
        DEFAULT_IGNORE.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn discovers_single_skill() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("tools/pdf-review/SKILL.md"),
            "---\nname: pdf-review\ndescription: Review PDFs\n---\nbody\n",
        );

        let skills = skills_in(dir.path(), "github.com/foo/bar", &default_ignore_owned());
        assert_eq!(skills.len(), 1);
        let s = &skills[0];
        assert_eq!(s.id(), "pdf-review");
        assert_eq!(s.relative_path(), Path::new("tools/pdf-review"));
        assert_eq!(s.source_alias(), "github.com/foo/bar");
        assert_eq!(s.kind(), AssetKind::Skill);
    }

    #[test]
    fn discovers_multiple_nested() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("code-review/SKILL.md"),
            "---\nname: code-review\ndescription: a\n---\n",
        );
        write(
            &dir.path().join("tools/pdf-review/SKILL.md"),
            "---\nname: pdf-review\ndescription: b\n---\n",
        );
        write(
            &dir.path().join("docs/how-to/SKILL.md"),
            "---\nname: how-to\ndescription: c\n---\n",
        );

        let mut skills = skills_in(dir.path(), "alias", &default_ignore_owned());
        skills.sort_by(|a, b| a.id().cmp(b.id()));
        assert_eq!(skills.len(), 3);
        let ids: Vec<&str> = skills.iter().map(|s| s.id()).collect();
        assert_eq!(ids, vec!["code-review", "how-to", "pdf-review"]);
    }

    #[test]
    fn ignored_directories_are_skipped() {
        let dir = TempDir::new().unwrap();
        write(&dir.path().join(".git/HEAD"), "ref: refs/heads/main\n");
        write(
            &dir.path().join("node_modules/some-pkg/SKILL.md"),
            "---\nname: should-not-see\ndescription: x\n---\n",
        );
        write(
            &dir.path().join("tools/real/SKILL.md"),
            "---\nname: real-skill\ndescription: x\n---\n",
        );

        let skills = skills_in(dir.path(), "alias", &default_ignore_owned());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id(), "real-skill");
    }

    #[test]
    fn empty_ignore_does_not_skip_any_dir() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("node_modules/p/SKILL.md"),
            "---\nname: in-nm\ndescription: x\n---\n",
        );
        write(
            &dir.path().join("real/SKILL.md"),
            "---\nname: real\ndescription: x\n---\n",
        );

        let skills = skills_in(dir.path(), "alias", &[]);
        let mut ids: Vec<&str> = skills.iter().map(|s| s.id()).collect();
        ids.sort();
        assert_eq!(ids, vec!["in-nm", "real"]);
    }

    #[test]
    fn invalid_skills_are_skipped_with_warning() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("valid/SKILL.md"),
            "---\nname: valid\ndescription: x\n---\n",
        );
        write(
            &dir.path().join("broken/SKILL.md"),
            "---\ndescription: only desc\n---\n",
        );
        write(
            &dir.path().join("yaml-error/SKILL.md"),
            "---\nname: foo\n  description: bad indent\n---\n",
        );

        let skills = skills_in(dir.path(), "alias", &default_ignore_owned());
        assert_eq!(skills.len(), 1, "仅 valid/ 通过");
        assert_eq!(skills[0].id(), "valid");
    }

    #[test]
    fn unknown_fields_preserved_in_extra() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("s/SKILL.md"),
            "---\nname: s\ndescription: d\nwhen_to_use: 在 X 时\nrequired_permissions: [a, b]\n---\n",
        );
        let skills = skills_in(dir.path(), "alias", &default_ignore_owned());
        assert_eq!(skills.len(), 1);
        let extra = skills[0].extra();
        assert!(extra.contains_key("when_to_use"));
        assert!(extra.contains_key("required_permissions"));
    }

    #[test]
    fn does_not_follow_symlinks() {
        let dir = TempDir::new().unwrap();
        write(
            &dir.path().join("real/SKILL.md"),
            "---\nname: real\ndescription: x\n---\n",
        );
        // 创建指向自己的 symlink，若跟随会无限循环
        let link_path = dir.path().join("loop");
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path(), &link_path).unwrap();

        let skills = skills_in(dir.path(), "alias", &default_ignore_owned());
        assert_eq!(skills.len(), 1);
    }
}
