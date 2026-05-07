//! 安装 / 卸载业务编排（按 ADR-0007 §7 修订版"动作下放到模块函数"）。
//!
//! - `resolve_skill`：从已订阅 sources 中按 name 找到唯一匹配；处理 0/1/N 三态
//! - `install_skill`：cp -r + 写 metadata + auto commit；force 时重装
//! - `uninstall_skill`：rm -rf + auto commit

use std::path::PathBuf;

use crate::asset::{Asset, AssetKind, Skill};
use crate::config;
use crate::discover;
use crate::error::{Error, Result};
use crate::git;
use crate::local_repo;
use crate::metadata::{self, InstalledAsset, Origin, OriginKind};
use crate::paths::PaamRoot;

/// 解析后的待安装 skill：业务上下文（来自哪个 source、本地仓在哪）。
#[derive(Debug, Clone)]
pub struct ResolvedSkill {
    pub skill: Skill,
    pub source_alias: String,
    pub source_local_path: PathBuf,
}

/// 在已订阅 sources 中按 name 查找 skill。
///
/// `from = Some(alias)` 时限定到该 alias 内（仍可能 0/1/N 个匹配）；
/// `from = None` 时跨所有 source。N≥2 时返回 `Error::AmbiguousSkill { candidates }`。
pub fn resolve_skill(root: &PaamRoot, name: &str, from: Option<&str>) -> Result<ResolvedSkill> {
    let sources = config::list_sources(root)?;
    let ignore = config::effective_scan_ignore(root)?;

    let mut hits: Vec<ResolvedSkill> = Vec::new();
    for src in sources {
        if let Some(filter) = from {
            if src.alias != filter {
                continue;
            }
        }
        let local_path = root.sources_dir().join(&src.alias);
        let skills = discover::skills_in(&local_path, &src.alias, &ignore);
        for s in skills {
            if s.id() == name {
                hits.push(ResolvedSkill {
                    skill: s,
                    source_alias: src.alias.clone(),
                    source_local_path: local_path.clone(),
                });
            }
        }
    }

    match hits.len() {
        0 => Err(Error::SkillNotFound {
            name: name.to_string(),
        }),
        1 => Ok(hits.into_iter().next().unwrap()),
        _ => {
            // 候选 alias 列表（去重保序）
            let mut candidates: Vec<String> = Vec::new();
            for h in &hits {
                if !candidates.contains(&h.source_alias) {
                    candidates.push(h.source_alias.clone());
                }
            }
            Err(Error::AmbiguousSkill {
                name: name.to_string(),
                candidates,
            })
        }
    }
}

/// 把 resolved skill 安装到 local-repo。
///
/// 失败时清理已写入的目标目录 / metadata；不向 local-repo 写入残留。
pub fn install_skill(
    root: &PaamRoot,
    resolved: &ResolvedSkill,
    force: bool,
) -> Result<InstalledAsset> {
    local_repo::ensure_initialized(root)?;

    let name = resolved.skill.id().to_string();
    let target_dir = metadata::skill_dir(root, &name);
    let already_exists = target_dir.exists();

    if already_exists && !force {
        return Err(Error::AlreadyInstalled { name });
    }
    if already_exists {
        // force 重装：先清理既有目录
        std::fs::remove_dir_all(&target_dir)?;
    }

    let source_dir = resolved
        .source_local_path
        .join(resolved.skill.relative_path());

    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if let Err(e) = copy_dir_excluding_git(&source_dir, &target_dir) {
        // 清理半成品
        let _ = std::fs::remove_dir_all(&target_dir);
        return Err(e);
    }

    // 取 source 仓的 commit / tree_hash —— 失败时回滚
    let subpath_str = resolved
        .skill
        .relative_path()
        .to_str()
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "skill 路径含非 UTF-8 字节",
            ))
        })?
        .to_string();
    let commit = match git::head_commit(&resolved.source_local_path) {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&target_dir);
            return Err(e);
        }
    };
    let tree_hash = match git::subtree_hash(&resolved.source_local_path, &subpath_str) {
        Ok(t) => t,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&target_dir);
            return Err(e);
        }
    };

    let meta = InstalledAsset {
        name: name.clone(),
        asset_type: AssetKind::Skill,
        origin: Origin {
            kind: OriginKind::Tracked,
            repo: resolved.source_alias.clone(),
            subpath: resolved.skill.relative_path().to_path_buf(),
            commit: commit.clone(),
            tree_hash,
        },
        installed_at: chrono::Utc::now(),
        targets: vec![],
        version: "1.0".to_string(),
    };

    if let Err(e) = metadata::write_for(&target_dir, &meta) {
        let _ = std::fs::remove_dir_all(&target_dir);
        return Err(e);
    }

    let short_commit = commit.chars().take(7).collect::<String>();
    let action = if already_exists {
        "重新安装"
    } else {
        "安装"
    };
    let msg = format!(
        "{} {}，来自 {}@{}",
        action, name, resolved.source_alias, short_commit
    );
    if let Err(e) = local_repo::commit(root, &msg) {
        // 不清理：cp 与 metadata 已写入；commit 失败让 local-repo 处于 staged 状态
        // 下次 install 的 add -A 会带上。warning 给用户提示但不当致命错误。
        eprintln!(
            "warning: local-repo auto-commit 失败（变更已 stage 但未 commit）：{}",
            e
        );
    }

    Ok(meta)
}

/// 卸载已装的 skill：先清 target symlink + targets[]，再 rm -rf 整个目录 + 单 auto commit。
pub fn uninstall_skill(root: &PaamRoot, name: &str) -> Result<()> {
    local_repo::ensure_initialized(root)?;
    let target_dir = metadata::skill_dir(root, name);
    if !target_dir.exists() {
        return Err(Error::NotInstalled {
            name: name.to_string(),
        });
    }
    // 先清 target symlink（若有）；不在此 commit，让最终的"卸载 X" commit 一并 capture
    crate::sync::unsync_one_no_commit(root, name)?;
    std::fs::remove_dir_all(&target_dir)?;
    if let Err(e) = local_repo::commit(root, &format!("卸载 {}", name)) {
        eprintln!(
            "warning: local-repo auto-commit 失败（变更已 stage 但未 commit）：{}",
            e
        );
    }
    Ok(())
}

/// 递归 copy 目录（不跟随 symlink、跳过 file_name == ".git" 的子目录）。
fn copy_dir_excluding_git(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
        let meta = entry.metadata()?;
        if meta.file_type().is_symlink() {
            // 不跟随 symlink
            continue;
        }
        let dst_path = dst.join(&file_name);
        if meta.is_dir() {
            copy_dir_excluding_git(&entry.path(), &dst_path)?;
        } else if meta.is_file() {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::make_repo_with_files;
    use tempfile::TempDir;

    /// 构造一个 paam sandbox：返回 root 的 TempDir + 已注册 source 的 alias 列表。
    /// 每个 source 是一个本地 git 仓（用 make_repo_with_files），注册到 paam 的
    /// `<root>/sources/<alias>/`（通过简单的目录 rename 实现"模拟 clone"）。
    fn make_sandbox_with_sources(
        sources: &[(&str, &[(&str, &str)])],
    ) -> (TempDir, PaamRoot, Vec<TempDir>) {
        let root_tmp = TempDir::new().unwrap();
        let root = PaamRoot::at(root_tmp.path());
        root.ensure_initialized().unwrap();

        // 写 paam config.json 注册 source（模拟已 track）
        let mut cfg = serde_json::json!({
            "version": 1,
            "sources": [],
        });

        let mut keep_alive: Vec<TempDir> = Vec::new();
        for (alias, files) in sources {
            // 在 sandbox 外构造一个本地 git 仓
            let src_repo = make_repo_with_files(files);
            // 把仓 copy 到 paam sources/<alias>/（模拟 clone 后的工作树）
            let dst = root.sources_dir().join(alias);
            std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
            // 简单递归 copy（含 .git）
            copy_recursive_keep_git(src_repo.path(), &dst).unwrap();

            cfg["sources"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "alias": alias,
                    "url": format!("git@example.com:{}.git", alias),
                    "added_at": "2026-04-29T00:00:00Z",
                }));
            keep_alive.push(src_repo);
        }
        std::fs::write(root.config_file(), serde_json::to_vec_pretty(&cfg).unwrap()).unwrap();

        (root_tmp, root, keep_alive)
    }

    fn copy_recursive_keep_git(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            let dst_path = dst.join(entry.file_name());
            if meta.is_dir() {
                copy_recursive_keep_git(&entry.path(), &dst_path)?;
            } else if meta.is_file() {
                std::fs::copy(entry.path(), &dst_path)?;
            }
        }
        Ok(())
    }

    #[test]
    fn resolve_returns_skill_not_found_when_no_match() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[(
                "tools/pdf-review/SKILL.md",
                "---\nname: pdf-review\ndescription: x\n---\n",
            )],
        )]);
        let err = resolve_skill(&root, "missing", None).unwrap_err();
        assert!(matches!(err, Error::SkillNotFound { .. }));
    }

    #[test]
    fn resolve_returns_unique_match() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[(
                "tools/pdf-review/SKILL.md",
                "---\nname: pdf-review\ndescription: x\n---\n",
            )],
        )]);
        let r = resolve_skill(&root, "pdf-review", None).unwrap();
        assert_eq!(r.skill.id(), "pdf-review");
        assert_eq!(r.source_alias, "fixture/a");
    }

    #[test]
    fn resolve_ambiguous_when_two_sources_have_same_name() {
        let (_d, root, _k) = make_sandbox_with_sources(&[
            (
                "fixture/a",
                &[(
                    "code-review/SKILL.md",
                    "---\nname: code-review\ndescription: x\n---\n",
                )],
            ),
            (
                "fixture/b",
                &[(
                    "code-review/SKILL.md",
                    "---\nname: code-review\ndescription: y\n---\n",
                )],
            ),
        ]);
        let err = resolve_skill(&root, "code-review", None).unwrap_err();
        match err {
            Error::AmbiguousSkill { candidates, .. } => {
                assert!(candidates.contains(&"fixture/a".to_string()));
                assert!(candidates.contains(&"fixture/b".to_string()));
            }
            other => panic!("expected AmbiguousSkill, got {:?}", other),
        }
    }

    #[test]
    fn resolve_with_from_disambiguates() {
        let (_d, root, _k) = make_sandbox_with_sources(&[
            (
                "fixture/a",
                &[(
                    "code-review/SKILL.md",
                    "---\nname: code-review\ndescription: x\n---\n",
                )],
            ),
            (
                "fixture/b",
                &[(
                    "code-review/SKILL.md",
                    "---\nname: code-review\ndescription: y\n---\n",
                )],
            ),
        ]);
        let r = resolve_skill(&root, "code-review", Some("fixture/b")).unwrap();
        assert_eq!(r.source_alias, "fixture/b");
    }

    #[test]
    fn install_skill_writes_files_and_metadata_and_commits() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[(
                "tools/pdf-review/SKILL.md",
                "---\nname: pdf-review\ndescription: Review PDFs\n---\n\n# body\n",
            )],
        )]);
        let resolved = resolve_skill(&root, "pdf-review", None).unwrap();
        let meta = install_skill(&root, &resolved, false).unwrap();

        // 验证 metadata 字段
        assert_eq!(meta.name, "pdf-review");
        assert_eq!(meta.asset_type, AssetKind::Skill);
        assert_eq!(meta.origin.kind, OriginKind::Tracked);
        assert_eq!(meta.origin.repo, "fixture/a");
        assert_eq!(
            meta.origin.subpath,
            std::path::Path::new("tools/pdf-review")
        );
        assert_eq!(meta.origin.commit.len(), 40);
        assert_eq!(meta.origin.tree_hash.len(), 40);
        assert_eq!(meta.targets.len(), 0);
        assert_eq!(meta.version, "1.0");

        // 验证文件系统
        let target = metadata::skill_dir(&root, "pdf-review");
        assert!(target.is_dir());
        assert!(target.join("SKILL.md").is_file());
        assert!(target.join(metadata::METADATA_FILENAME).is_file());
        // 不应保留 .git
        assert!(!target.join(".git").exists());

        // 验证 local-repo 有一个 install commit
        let lr = local_repo::local_repo_dir(&root);
        let log = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log.contains("安装 pdf-review"), "实际日志：{}", log);
    }

    #[test]
    fn install_already_installed_rejects_without_force() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[("p/SKILL.md", "---\nname: p\ndescription: x\n---\n")],
        )]);
        let resolved = resolve_skill(&root, "p", None).unwrap();
        install_skill(&root, &resolved, false).unwrap();

        let err = install_skill(&root, &resolved, false).unwrap_err();
        assert!(matches!(err, Error::AlreadyInstalled { .. }));
    }

    #[test]
    fn install_force_reinstalls_with_distinct_commit_message() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[("p/SKILL.md", "---\nname: p\ndescription: x\n---\n")],
        )]);
        let resolved = resolve_skill(&root, "p", None).unwrap();
        install_skill(&root, &resolved, false).unwrap();
        install_skill(&root, &resolved, true).unwrap();

        let lr = local_repo::local_repo_dir(&root);
        let log = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log.contains("安装 p"));
        assert!(log.contains("重新安装 p"));
    }

    #[test]
    fn uninstall_removes_dir_and_commits() {
        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[("p/SKILL.md", "---\nname: p\ndescription: x\n---\n")],
        )]);
        let resolved = resolve_skill(&root, "p", None).unwrap();
        install_skill(&root, &resolved, false).unwrap();

        uninstall_skill(&root, "p").unwrap();
        let target = metadata::skill_dir(&root, "p");
        assert!(!target.exists());

        let lr = local_repo::local_repo_dir(&root);
        let log = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log.contains("卸载 p"));
    }

    #[test]
    fn uninstall_when_not_installed_returns_error() {
        let (_d, root, _k) = make_sandbox_with_sources(&[]);
        let err = uninstall_skill(&root, "ghost").unwrap_err();
        assert!(matches!(err, Error::NotInstalled { .. }));
    }

    /// install + sync + uninstall：target symlink 应被自动清理；commit log 仅
    /// "安装 p" + "同步 ..." + "卸载 p"（uninstall 单 commit 含 target 清理）。
    #[test]
    fn uninstall_after_sync_clears_target_symlink_in_single_commit() {
        // 共享 env 锁串行；与 sync / paths 模块的 env_var 测试相互排斥
        let _g = crate::test_support::acquire_env_lock();

        let (_d, root, _k) = make_sandbox_with_sources(&[(
            "fixture/a",
            &[("p/SKILL.md", "---\nname: p\ndescription: x\n---\n")],
        )]);

        let target_tmp = TempDir::new().unwrap();
        std::env::set_var(crate::paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());

        let resolved = resolve_skill(&root, "p", None).unwrap();
        install_skill(&root, &resolved, false).unwrap();
        let report = crate::sync::sync_all(&root, false).unwrap();

        let target_path = target_tmp.path().join("p");
        let resolved_target_root = crate::paths::claude_skills_target_dir();
        let installed_skills = crate::metadata::list_skills(&root);
        assert!(
            std::fs::symlink_metadata(&target_path).is_ok(),
            "sync 后应存在；report={:?} target_root={:?} target_path={:?} installed={:?}",
            report,
            resolved_target_root,
            target_path,
            installed_skills
                .as_ref()
                .map(|v| v.iter().map(|m| &m.name).collect::<Vec<_>>()),
        );

        uninstall_skill(&root, "p").unwrap();
        std::env::remove_var(crate::paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        // target symlink 应被清理
        assert!(
            std::fs::symlink_metadata(&target_path).is_err(),
            "uninstall 后 target symlink 应消失"
        );

        // commit log 不应有"解除同步 p"（应被合并到"卸载 p"中）
        let lr = local_repo::local_repo_dir(&root);
        let log = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log.contains("卸载 p"));
        assert!(
            !log.contains("解除同步 p"),
            "uninstall 应单 commit；不应出现独立的 解除同步 commit"
        );
    }
}
