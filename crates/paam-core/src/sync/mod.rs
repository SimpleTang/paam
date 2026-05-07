//! local-repo → target 的分发：把已装 skill 通过 symlink 暴露到 Claude Code 能读的目录。
//!
//! - `sync_all`：扫已装 skill，按需创建 / 重建 / 修复 / 跳过 symlink；
//!   写 metadata.targets[]；产生单 commit 含所有变更
//! - `unsync_one` / `unsync_all`：与 sync 对偶，仅删 symlink + 清 targets[]
//! - 与 install::uninstall_skill 智能耦合：uninstall 调 `unsync_one_no_commit` 先清
//!   target，再让自己的 commit 一并 capture（避免双 commit）

use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::error::{Error, Result};
use crate::local_repo;
use crate::metadata::{self, InstalledAsset, Target};
use crate::paths;
use crate::paths::PaamRoot;

/// `paam sync` 一次执行的状态汇总。
#[derive(Debug, Default)]
pub struct SyncReport {
    /// 创建或重建（symlink 之前不存在 / 不正确 / 被强制覆盖前是 paam 旧链路）
    pub synced: Vec<String>,
    /// 已正确指向，跳过；不写 metadata、不 commit
    pub already_ok: Vec<String>,
    /// force=false 时遇到的非 paam 占用，跳过
    pub conflicts: Vec<Conflict>,
    /// force=true 时被覆盖的非 paam 占用
    pub forced: Vec<String>,
}

#[derive(Debug)]
pub struct Conflict {
    pub skill_name: String,
    pub target_path: PathBuf,
    pub reason: String,
}

/// 把所有已装 skill 同步到 target。
///
/// 单 commit 合并多 skill 变更；幂等（全 already_ok 时不写不 commit）。
pub fn sync_all(root: &PaamRoot, force: bool) -> Result<SyncReport> {
    local_repo::ensure_initialized(root)?;
    let target_root = paths::claude_skills_target_dir()?;
    std::fs::create_dir_all(&target_root).map_err(|e| Error::SyncIo {
        skill: "(target_root)".into(),
        target: target_root.clone(),
        message: e.to_string(),
    })?;

    let mut report = SyncReport::default();
    let installed = metadata::list_skills(root)?;

    for asset in installed {
        let name = asset.name.clone();
        let expected = metadata::skill_dir(root, &name);
        let target_path = target_root.join(&name);
        let existing_targets = asset.targets.clone();

        let state = match classify(&target_path, &expected, &existing_targets, root) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: {}", e);
                continue;
            }
        };

        match state {
            TargetState::AlreadyOk => {
                report.already_ok.push(name);
            }
            TargetState::Absent | TargetState::PaamLinkBroken => {
                if let Err(e) = ensure_symlink(&expected, &target_path, &name) {
                    eprintln!("warning: {}", e);
                    continue;
                }
                if let Err(e) = update_targets_for(root, &name, &target_path) {
                    eprintln!("warning: {}", e);
                    continue;
                }
                report.synced.push(name);
            }
            TargetState::ForeignContent { is_symlink } => {
                if !force {
                    report.conflicts.push(Conflict {
                        skill_name: name.clone(),
                        target_path: target_path.clone(),
                        reason: if is_symlink {
                            "已存在非 paam 管理的 symlink".into()
                        } else {
                            "已存在非 paam 管理的文件 / 目录".into()
                        },
                    });
                    eprintln!(
                        "warning: 跳过 {} —— {} 已存在非 paam 内容（用 --force 覆盖）",
                        name,
                        target_path.display()
                    );
                } else {
                    if let Err(e) = remove_target(&target_path, &name) {
                        eprintln!("warning: {}", e);
                        continue;
                    }
                    if let Err(e) = ensure_symlink(&expected, &target_path, &name) {
                        eprintln!("warning: {}", e);
                        continue;
                    }
                    if let Err(e) = update_targets_for(root, &name, &target_path) {
                        eprintln!("warning: {}", e);
                        continue;
                    }
                    report.forced.push(name);
                }
            }
        }
    }

    let total_changes = report.synced.len() + report.forced.len();
    if total_changes > 0 {
        let msg = build_sync_commit_message(&report, &target_root);
        local_repo::commit(root, &msg)?;
    }

    Ok(report)
}

/// 解除单个 skill 的 sync：删 target symlink + 清 metadata.targets[]。
///
/// 自动 commit；若无变更则静默跳过 commit。target 不存在时静默成功。
pub fn unsync_one(root: &PaamRoot, name: &str) -> Result<()> {
    local_repo::ensure_initialized(root)?;
    let changed = unsync_one_no_commit(root, name)?;
    if changed {
        local_repo::commit(root, &format!("解除同步 {}", name))?;
    }
    Ok(())
}

/// 解除单个 skill 的 sync 但不 commit；返回是否真的有变更。
///
/// 供 `install::uninstall_skill` 调用，让 uninstall 的"卸载 X" commit 一并 capture
/// target 清理变更，避免双 commit。
pub(crate) fn unsync_one_no_commit(root: &PaamRoot, name: &str) -> Result<bool> {
    let target_root = paths::claude_skills_target_dir()?;
    let target_path = target_root.join(name);

    let asset_dir = metadata::skill_dir(root, name);
    let mut asset = match metadata::read_for(&asset_dir)? {
        Some(a) => a,
        None => {
            // skill 未装或 metadata 缺失：仅尝试删 target 上的 symlink（如果是）
            return remove_target_if_symlink(&target_path, name);
        }
    };

    let mut changed = false;

    // 删 target symlink（仅当 target 是 symlink 时）
    if try_remove_symlink(&target_path, name)? {
        changed = true;
    }

    // 清空 metadata.targets[]
    if !asset.targets.is_empty() {
        asset.targets.clear();
        metadata::write_for(&asset_dir, &asset)?;
        changed = true;
    }

    Ok(changed)
}

/// 解除所有已装 skill 的 sync。单 commit。
pub fn unsync_all(root: &PaamRoot) -> Result<()> {
    local_repo::ensure_initialized(root)?;
    let installed = metadata::list_skills(root)?;
    let mut any_changed = false;
    for asset in installed {
        if unsync_one_no_commit(root, &asset.name)? {
            any_changed = true;
        }
    }
    if any_changed {
        local_repo::commit(root, "解除所有同步")?;
    }
    Ok(())
}

// ============================================================================
// 内部 helpers
// ============================================================================

#[derive(Debug)]
enum TargetState {
    Absent,
    AlreadyOk,
    PaamLinkBroken,
    ForeignContent { is_symlink: bool },
}

fn classify(
    target_path: &Path,
    expected: &Path,
    meta_targets: &[Target],
    root: &PaamRoot,
) -> std::result::Result<TargetState, String> {
    // 不存在
    let symlink_meta = match std::fs::symlink_metadata(target_path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(TargetState::Absent),
        Err(e) => return Err(format!("symlink_metadata({:?}) 失败：{}", target_path, e)),
    };

    let is_symlink = symlink_meta.file_type().is_symlink();

    if is_symlink {
        // 是 symlink，看它指向哪
        match std::fs::canonicalize(target_path) {
            Ok(canon) => {
                let expected_canon =
                    std::fs::canonicalize(expected).unwrap_or_else(|_| expected.to_path_buf());
                if canon == expected_canon {
                    return Ok(TargetState::AlreadyOk);
                }
                // 不指向 expected，但可能仍是 paam 旧链路
                if is_paam_managed(target_path, &canon, meta_targets, root) {
                    return Ok(TargetState::PaamLinkBroken);
                }
                Ok(TargetState::ForeignContent { is_symlink: true })
            }
            Err(_) => {
                // canonicalize 失败 → 断链
                if is_paam_managed_by_meta(target_path, meta_targets) {
                    Ok(TargetState::PaamLinkBroken)
                } else {
                    Ok(TargetState::ForeignContent { is_symlink: true })
                }
            }
        }
    } else {
        // 真实文件 / 目录：即便 metadata 中有该路径记录（"paam 之前 sync 过这条
        // path"），fs 上变成真实内容也意味着用户已显式破坏了 symlink。安全起
        // 见仍视为 ForeignContent —— 默认跳过 + warning，要求 `--force` 才覆盖
        // （避免静默删除用户后续放进去的重要文件）。
        Ok(TargetState::ForeignContent { is_symlink: false })
    }
}

fn is_paam_managed(
    target_path: &Path,
    canon: &Path,
    meta_targets: &[Target],
    root: &PaamRoot,
) -> bool {
    if is_paam_managed_by_meta(target_path, meta_targets) {
        return true;
    }
    // 备路径：canonicalize 落在 local-repo 内
    let lr = local_repo::local_repo_dir(root);
    if let Ok(lr_canon) = std::fs::canonicalize(&lr) {
        canon.starts_with(&lr_canon)
    } else {
        canon.starts_with(&lr)
    }
}

fn is_paam_managed_by_meta(target_path: &Path, meta_targets: &[Target]) -> bool {
    meta_targets.iter().any(|t| t.path == target_path)
}

fn ensure_symlink(src: &Path, target: &Path, skill: &str) -> Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::SyncIo {
            skill: skill.to_string(),
            target: target.to_path_buf(),
            message: format!("mkdir 父目录失败：{}", e),
        })?;
    }
    // 如果 target 已存在（可能是 symlink），先删（PaamLinkBroken 路径）
    if std::fs::symlink_metadata(target).is_ok() {
        remove_target(target, skill)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, target).map_err(|e| Error::SyncIo {
            skill: skill.to_string(),
            target: target.to_path_buf(),
            message: format!("symlink({:?} -> {:?}) 失败：{}", src, target, e),
        })?;
    }
    #[cfg(not(unix))]
    {
        return Err(Error::SyncIo {
            skill: skill.to_string(),
            target: target.to_path_buf(),
            message: "M1 仅支持 unix 平台 symlink".into(),
        });
    }
    Ok(())
}

/// 删除 target（无论是 symlink、文件还是目录）。
fn remove_target(target: &Path, skill: &str) -> Result<()> {
    let meta = match std::fs::symlink_metadata(target) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(Error::SyncIo {
                skill: skill.to_string(),
                target: target.to_path_buf(),
                message: format!("symlink_metadata 失败：{}", e),
            });
        }
    };
    let r = if meta.file_type().is_symlink() || meta.is_file() {
        std::fs::remove_file(target)
    } else {
        std::fs::remove_dir_all(target)
    };
    r.map_err(|e| Error::SyncIo {
        skill: skill.to_string(),
        target: target.to_path_buf(),
        message: format!("删除失败：{}", e),
    })
}

/// 仅当 target 是 symlink 时删除；返回是否删除了。
fn try_remove_symlink(target: &Path, skill: &str) -> Result<bool> {
    match std::fs::symlink_metadata(target) {
        Ok(m) if m.file_type().is_symlink() => {
            std::fs::remove_file(target).map_err(|e| Error::SyncIo {
                skill: skill.to_string(),
                target: target.to_path_buf(),
                message: format!("删除 symlink 失败：{}", e),
            })?;
            Ok(true)
        }
        Ok(_) => Ok(false), // 真实文件 / 目录：不动（保护用户内容）
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(Error::SyncIo {
            skill: skill.to_string(),
            target: target.to_path_buf(),
            message: format!("symlink_metadata 失败：{}", e),
        }),
    }
}

/// `unsync_one_no_commit` 的简化分支：metadata 不存在但仍尝试清 target 上的 symlink。
fn remove_target_if_symlink(target: &Path, skill: &str) -> Result<bool> {
    try_remove_symlink(target, skill)
}

fn update_targets_for(root: &PaamRoot, name: &str, target_path: &Path) -> Result<()> {
    let asset_dir = metadata::skill_dir(root, name);
    let mut asset: InstalledAsset = match metadata::read_for(&asset_dir)? {
        Some(a) => a,
        None => return Ok(()), // 极端 case：刚才 list_skills 还有它，这里没了；保守跳过
    };
    asset.targets = vec![Target {
        agent: "claude-code".to_string(),
        path: target_path.to_path_buf(),
        mode: "symlink".to_string(),
        synced_at: Utc::now(),
    }];
    metadata::write_for(&asset_dir, &asset)
}

fn build_sync_commit_message(report: &SyncReport, target_root: &Path) -> String {
    let total = report.synced.len() + report.forced.len();
    if total == 1 {
        let name = report.synced.first().or(report.forced.first()).unwrap();
        let short = short_target_path(target_root, name);
        format!("同步 {} -> claude-code:{}", name, short)
    } else {
        format!("同步 {} 个 skill 到 claude-code", total)
    }
}

fn short_target_path(target_root: &Path, name: &str) -> String {
    // 取 target_root 末段 + name，例如 ".claude/skills/pdf-review"
    let last = target_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("skills");
    let parent_last = target_root
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if parent_last.is_empty() {
        format!("{}/{}", last, name)
    } else {
        format!("{}/{}/{}", parent_last, last, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetKind;
    use crate::metadata::{InstalledAsset, Origin, OriginKind};
    use tempfile::TempDir;

    use crate::test_support::acquire_env_lock;

    fn fresh_sandbox() -> (TempDir, TempDir, PaamRoot) {
        let root_tmp = TempDir::new().unwrap();
        let target_tmp = TempDir::new().unwrap();
        let root = PaamRoot::at(root_tmp.path());
        root.ensure_initialized().unwrap();
        local_repo::ensure_initialized(&root).unwrap();
        (root_tmp, target_tmp, root)
    }

    /// 直接构造一个已装 skill：在 local-repo/skills/<name>/ 写文件 + 写 metadata。
    fn install_fake_skill(root: &PaamRoot, name: &str) {
        let asset_dir = metadata::skill_dir(root, name);
        std::fs::create_dir_all(&asset_dir).unwrap();
        std::fs::write(
            asset_dir.join("SKILL.md"),
            format!("---\nname: {}\ndescription: x\n---\n", name),
        )
        .unwrap();
        let meta = InstalledAsset {
            name: name.to_string(),
            asset_type: AssetKind::Skill,
            origin: Origin {
                kind: OriginKind::Tracked,
                repo: "fixture/x".into(),
                subpath: PathBuf::from(name),
                commit: "0".repeat(40),
                tree_hash: "0".repeat(40),
            },
            installed_at: Utc::now(),
            targets: vec![],
            version: "1.0".into(),
        };
        metadata::write_for(&asset_dir, &meta).unwrap();
    }

    #[test]
    fn sync_all_creates_symlinks_on_clean_target() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "pdf-review");
        install_fake_skill(&root, "code-review");

        let report = sync_all(&root, false).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        assert_eq!(report.synced.len(), 2);
        assert!(report.already_ok.is_empty());
        assert!(report.conflicts.is_empty());
        // 验证 symlink
        for name in &["pdf-review", "code-review"] {
            let target = target_tmp.path().join(name);
            assert!(
                std::fs::symlink_metadata(&target)
                    .unwrap()
                    .file_type()
                    .is_symlink(),
                "应该是 symlink: {}",
                target.display()
            );
            // 验证 metadata.targets[] 写入
            let meta = metadata::find_skill(&root, name).unwrap().unwrap();
            assert_eq!(meta.targets.len(), 1);
            assert_eq!(meta.targets[0].agent, "claude-code");
            assert_eq!(meta.targets[0].mode, "symlink");
            assert_eq!(meta.targets[0].path, target);
        }
    }

    #[test]
    fn sync_all_is_idempotent() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");

        sync_all(&root, false).unwrap();
        let log_before = crate::git::run_capture(
            &["log", "--oneline"],
            Some(local_repo::local_repo_dir(&root).as_path()),
        )
        .unwrap();

        let report = sync_all(&root, false).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        assert!(report.synced.is_empty());
        assert_eq!(report.already_ok.len(), 1);

        let log_after = crate::git::run_capture(
            &["log", "--oneline"],
            Some(local_repo::local_repo_dir(&root).as_path()),
        )
        .unwrap();
        assert_eq!(log_before, log_after, "全 already_ok 不应产生新 commit");
    }

    #[test]
    fn sync_all_skips_foreign_content_without_force() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");
        // 在 target 上手动写一个真实目录占位
        let conflict_path = target_tmp.path().join("p");
        std::fs::create_dir_all(&conflict_path).unwrap();
        std::fs::write(conflict_path.join("user-file.txt"), "user data").unwrap();

        let report = sync_all(&root, false).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].skill_name, "p");
        // 用户内容仍存在
        assert!(conflict_path.join("user-file.txt").is_file());
    }

    #[test]
    fn sync_all_force_overwrites_foreign_content() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");
        let conflict_path = target_tmp.path().join("p");
        std::fs::create_dir_all(&conflict_path).unwrap();
        std::fs::write(conflict_path.join("user-file.txt"), "user data").unwrap();

        let report = sync_all(&root, true).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        assert_eq!(report.forced.len(), 1);
        // 用户内容已被覆盖：现在 target 是 symlink
        assert!(std::fs::symlink_metadata(&conflict_path)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn sync_all_repairs_paam_broken_link() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");
        sync_all(&root, false).unwrap(); // 先建好

        // 在 target 上把 symlink 指向一个不存在的路径（模拟漂移）
        let target_path = target_tmp.path().join("p");
        std::fs::remove_file(&target_path).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/tmp/nonexistent-paam-fake", &target_path).unwrap();

        // 此时 metadata.targets[] 仍记录旧 path，符合 paam 管理 主路径条件
        let report = sync_all(&root, false).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        assert_eq!(report.synced.len(), 1);
    }

    #[test]
    fn unsync_one_removes_symlink_and_clears_targets() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");
        sync_all(&root, false).unwrap();

        unsync_one(&root, "p").unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        let target_path = target_tmp.path().join("p");
        assert!(std::fs::symlink_metadata(&target_path).is_err());

        let meta = metadata::find_skill(&root, "p").unwrap().unwrap();
        assert!(meta.targets.is_empty());
    }

    #[test]
    fn unsync_when_target_absent_is_silent_success() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "p");
        // 不 sync；直接 unsync

        unsync_one(&root, "p").unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);
    }

    #[test]
    fn unsync_all_clears_multiple_skills() {
        let _g = acquire_env_lock();
        let (_r_tmp, target_tmp, root) = fresh_sandbox();
        std::env::set_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR, target_tmp.path());
        install_fake_skill(&root, "a");
        install_fake_skill(&root, "b");
        sync_all(&root, false).unwrap();

        unsync_all(&root).unwrap();
        std::env::remove_var(paths::ENV_PAAM_CLAUDE_TARGET_DIR);

        for n in &["a", "b"] {
            let target_path = target_tmp.path().join(n);
            assert!(std::fs::symlink_metadata(&target_path).is_err());
            let meta = metadata::find_skill(&root, n).unwrap().unwrap();
            assert!(meta.targets.is_empty());
        }
    }
}
