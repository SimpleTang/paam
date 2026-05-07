//! 管理 paam 自管的本地工作集 git 仓 `~/.paam/local-repo/`。
//!
//! - 首次执行任何安装类命令时自动 `git init`，并设置独立的 paam 身份
//!   （`paam@local`），不读用户 `~/.gitconfig`。
//! - 安装 / 卸载等业务操作通过 `commit(message)` 触发自动 commit。
//! - 物理布局（按类型分组）见 ADR-0007 §3；`<type>/<name>/` 子目录由 install 模块写入。

use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::git;
use crate::paths::PaamRoot;

pub const LOCAL_REPO_DIRNAME: &str = "local-repo";

/// 返回 `<root>/.paam/local-repo/` 的绝对路径（不保证存在）。
pub fn local_repo_dir(root: &PaamRoot) -> PathBuf {
    root.home().join(LOCAL_REPO_DIRNAME)
}

/// 幂等地确保 local-repo 已 `git init` 并设置 paam 身份。
pub fn ensure_initialized(root: &PaamRoot) -> Result<()> {
    let dir = local_repo_dir(root);
    let dot_git = dir.join(".git");
    if dot_git.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(&dir)?;

    let dir_str = dir.to_str().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "local-repo 路径含非 UTF-8 字节",
        ))
    })?;
    git::run(&["init", "--quiet", dir_str], None)?;
    git::run(&["config", "user.email", "paam@local"], Some(dir.as_path()))?;
    git::run(&["config", "user.name", "paam"], Some(dir.as_path()))?;

    let gitignore = dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, "# paam local-repo .gitignore\n")?;
    }
    Ok(())
}

/// 在 local-repo 中执行 `git add -A` + 条件 commit。
///
/// 如无 staged 变更则静默跳过 commit（避免空 commit）。
pub fn commit(root: &PaamRoot, message: &str) -> Result<()> {
    let dir = local_repo_dir(root);
    git::run(&["add", "-A"], Some(dir.as_path()))?;

    // diff --staged --quiet：exit 0 = 无变更；exit 1 = 有变更
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--staged", "--quiet"]);
    cmd.current_dir(&dir);
    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::GitNotFound
        } else {
            Error::Io(e)
        }
    })?;
    if status.success() {
        // 无变更
        return Ok(());
    }

    git::run(&["commit", "-m", message, "--quiet"], Some(dir.as_path()))?;
    Ok(())
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

    #[test]
    fn ensure_initialized_creates_git_repo_with_paam_identity() {
        let (_dir, root) = fresh_root();
        ensure_initialized(&root).unwrap();
        let lr = local_repo_dir(&root);
        assert!(lr.join(".git").is_dir());

        // 验证 paam 身份
        let email = git::run_capture(&["config", "user.email"], Some(lr.as_path())).unwrap();
        assert_eq!(email, "paam@local");
        let name = git::run_capture(&["config", "user.name"], Some(lr.as_path())).unwrap();
        assert_eq!(name, "paam");
    }

    #[test]
    fn ensure_initialized_is_idempotent() {
        let (_dir, root) = fresh_root();
        ensure_initialized(&root).unwrap();
        // 故意改一个 config 看是否被覆盖
        let lr = local_repo_dir(&root);
        git::run(
            &["config", "user.email", "user-modified@x"],
            Some(lr.as_path()),
        )
        .unwrap();
        ensure_initialized(&root).unwrap();
        let email = git::run_capture(&["config", "user.email"], Some(lr.as_path())).unwrap();
        assert_eq!(email, "user-modified@x", "已存在时不应覆盖既有 config");
    }

    #[test]
    fn commit_silently_skips_when_nothing_staged() {
        let (_dir, root) = fresh_root();
        ensure_initialized(&root).unwrap();
        // 第一次 commit：local-repo 只有 .gitignore，应该 commit 一次
        commit(&root, "首次提交").unwrap();
        let lr = local_repo_dir(&root);
        let log1 = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log1.contains("首次提交"));

        // 第二次：无变更
        commit(&root, "应被跳过").unwrap();
        let log2 = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert_eq!(log1, log2, "无变更时不应创建新 commit");
    }

    #[test]
    fn commit_creates_commit_when_files_change() {
        let (_dir, root) = fresh_root();
        ensure_initialized(&root).unwrap();
        commit(&root, "初始化").unwrap();
        let lr = local_repo_dir(&root);

        // 写一个新文件
        std::fs::write(lr.join("new-file.txt"), "hi").unwrap();
        commit(&root, "添加 new-file").unwrap();
        let log = git::run_capture(&["log", "--oneline"], Some(lr.as_path())).unwrap();
        assert!(log.contains("添加 new-file"));
        assert!(log.contains("初始化"));
    }
}
