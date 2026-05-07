//! 资产元数据 `.metadata.json` 的 schema 与读写。
//!
//! 物理布局（沿用 ADR-0007 §3 / §5）：每个已装资产目录下持有一个 `.metadata.json`，
//! 而非全局索引。优点是 git diff 在 install 时只新增一个文件，diff 易读。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::asset::AssetKind;
use crate::error::Result;
use crate::local_repo::local_repo_dir;
use crate::paths::PaamRoot;

const SKILLS_DIRNAME: &str = "skills";

pub const METADATA_FILENAME: &str = ".metadata.json";

/// 已安装资产的完整 metadata（沿用 ADR-0007 §5 schema 草案）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledAsset {
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: AssetKind,
    pub origin: Origin,
    pub installed_at: DateTime<Utc>,
    #[serde(default)]
    pub targets: Vec<Target>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub kind: OriginKind,
    /// source 在 paam 中的 alias（不是 git URL）；同 `origin.repo`。
    pub repo: String,
    /// 在 source 仓里的相对路径（含 SKILL.md 的目录）。
    pub subpath: PathBuf,
    /// source 仓 HEAD 的全长 commit hash（40 位 hex）。
    pub commit: String,
    /// 该 subpath 在 source 仓 HEAD 中的 git tree hash（40 位 hex）。
    pub tree_hash: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginKind {
    /// 从远程仓 install（M1 仅此种）
    Tracked,
    // M2+: Authored, Adopted
}

/// sync 到目标 Agent 时记录（M1 由 ④ paam-claude-sync 写入；本 change 始终为空数组）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub agent: String,
    pub path: PathBuf,
    pub mode: String,
    pub synced_at: DateTime<Utc>,
}

/// 计算某 skill 的本地工作目录路径：`<local-repo>/skills/<name>/`
pub fn skill_dir(root: &PaamRoot, name: &str) -> PathBuf {
    local_repo_dir(root).join(SKILLS_DIRNAME).join(name)
}

/// 原子写入 metadata.json 到 `<asset_dir>/.metadata.json`。
pub fn write_for(asset_dir: &Path, meta: &InstalledAsset) -> Result<()> {
    let path = asset_dir.join(METADATA_FILENAME);
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        let bytes = serde_json::to_vec_pretty(meta)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// 读取 `<asset_dir>/.metadata.json`；不存在时返回 Ok(None)；解析失败时返回 Err。
pub fn read_for(asset_dir: &Path) -> Result<Option<InstalledAsset>> {
    let path = asset_dir.join(METADATA_FILENAME);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    let meta: InstalledAsset = serde_json::from_slice(&bytes)?;
    Ok(Some(meta))
}

/// 扫描 local-repo 中所有已装资产的 metadata。
///
/// 解析失败 / 缺失的资产通过 stderr 输出 warning 并跳过，不影响整体返回。
pub fn list_installed(root: &PaamRoot) -> Result<Vec<InstalledAsset>> {
    let mut out = Vec::new();
    let lr = local_repo_dir(root);
    if !lr.is_dir() {
        return Ok(out);
    }
    for type_subdir in &[SKILLS_DIRNAME] {
        let type_dir = lr.join(type_subdir);
        if !type_dir.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(&type_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            match read_for(&p) {
                Ok(Some(m)) => out.push(m),
                Ok(None) => {
                    eprintln!(
                        "warning: {} 缺少 {}，已跳过",
                        p.display(),
                        METADATA_FILENAME
                    );
                }
                Err(e) => {
                    eprintln!(
                        "warning: 解析 {}/{} 失败：{}，已跳过",
                        p.display(),
                        METADATA_FILENAME,
                        e
                    );
                }
            }
        }
    }
    Ok(out)
}

/// 仅列出 type=Skill 的已装资产（M1 阶段实质等同 list_installed）。
pub fn list_skills(root: &PaamRoot) -> Result<Vec<InstalledAsset>> {
    Ok(list_installed(root)?
        .into_iter()
        .filter(|m| m.asset_type == AssetKind::Skill)
        .collect())
}

/// 在 local-repo 中按 name 查找已装的 skill；找不到返回 Ok(None)。
pub fn find_skill(root: &PaamRoot, name: &str) -> Result<Option<InstalledAsset>> {
    Ok(list_skills(root)?.into_iter().find(|m| m.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_root() -> (TempDir, PaamRoot) {
        let dir = TempDir::new().unwrap();
        let root = PaamRoot::at(dir.path());
        root.ensure_initialized().unwrap();
        (dir, root)
    }

    fn sample_meta(name: &str) -> InstalledAsset {
        InstalledAsset {
            name: name.into(),
            asset_type: AssetKind::Skill,
            origin: Origin {
                kind: OriginKind::Tracked,
                repo: "github.com/foo/bar".into(),
                subpath: PathBuf::from("tools/pdf-review"),
                commit: "0".repeat(40),
                tree_hash: "1".repeat(40),
            },
            installed_at: chrono::Utc::now(),
            targets: vec![],
            version: "1.0".into(),
        }
    }

    #[test]
    fn write_then_read_round_trip() {
        let dir = TempDir::new().unwrap();
        let asset_dir = dir.path().join("skills/pdf-review");
        std::fs::create_dir_all(&asset_dir).unwrap();
        let meta = sample_meta("pdf-review");
        write_for(&asset_dir, &meta).unwrap();
        let got = read_for(&asset_dir).unwrap().unwrap();
        assert_eq!(got.name, "pdf-review");
        assert_eq!(got.asset_type, AssetKind::Skill);
        assert_eq!(got.origin.repo, "github.com/foo/bar");
        assert_eq!(got.targets.len(), 0);
        assert_eq!(got.version, "1.0");
    }

    #[test]
    fn read_for_missing_file_returns_none() {
        let dir = TempDir::new().unwrap();
        let asset_dir = dir.path().join("empty");
        std::fs::create_dir_all(&asset_dir).unwrap();
        assert!(read_for(&asset_dir).unwrap().is_none());
    }

    #[test]
    fn list_installed_aggregates_multiple_assets() {
        let (root_dir, root) = fresh_root();
        let lr = local_repo_dir(&root);
        for name in &["a", "b", "c"] {
            let asset_dir = lr.join("skills").join(name);
            std::fs::create_dir_all(&asset_dir).unwrap();
            write_for(&asset_dir, &sample_meta(name)).unwrap();
        }
        let mut got = list_installed(&root).unwrap();
        got.sort_by(|a, b| a.name.cmp(&b.name));
        let names: Vec<&str> = got.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c"]);
        drop(root_dir);
    }

    #[test]
    fn list_installed_skips_unparseable_metadata() {
        let (_dir, root) = fresh_root();
        let lr = local_repo_dir(&root);
        let bad_dir = lr.join("skills/bad");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(bad_dir.join(METADATA_FILENAME), "{not json}").unwrap();
        let good_dir = lr.join("skills/good");
        std::fs::create_dir_all(&good_dir).unwrap();
        write_for(&good_dir, &sample_meta("good")).unwrap();

        let got = list_installed(&root).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "good");
    }

    #[test]
    fn find_skill_returns_match_or_none() {
        let (_dir, root) = fresh_root();
        let lr = local_repo_dir(&root);
        let asset_dir = lr.join("skills/pdf-review");
        std::fs::create_dir_all(&asset_dir).unwrap();
        write_for(&asset_dir, &sample_meta("pdf-review")).unwrap();

        assert!(find_skill(&root, "pdf-review").unwrap().is_some());
        assert!(find_skill(&root, "nonexistent").unwrap().is_none());
    }
}
