//! 测试辅助：用 system git 子进程构造 bare repo fixture。
//!
//! 仅在 `#[cfg(test)]` 编译期可见。被 `git/tests` 与 `source/tests` 共享，
//! 替代原先依赖 git2-rs 的内存内构造方式。

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn run_git_in(cwd: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("system git 必须存在以构造测试 fixture");
    assert!(status.success(), "git {:?} 在 {} 失败", args, cwd.display());
}

fn run_git(args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .status()
        .expect("system git 必须存在以构造测试 fixture");
    assert!(status.success(), "git {:?} 失败", args);
}

/// 构造一个含一个 root commit 的 bare repo，返回它的 file:// URL。
///
/// 实现：先 `git init --bare <dir>/origin.git`，再在 `<dir>/work/` 创建一个临时
/// 普通 repo 并 `commit --allow-empty`，最后 `git push file://<bare> HEAD:master`。
pub fn make_fixture_repo() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    let bare_path = dir.path().join("origin.git");
    let work_path = dir.path().join("work");
    std::fs::create_dir_all(&work_path).unwrap();

    let bare_str = bare_path.to_str().unwrap();
    let work_str = work_path.to_str().unwrap();

    run_git(&["init", "--bare", bare_str]);
    run_git(&["init", work_str]);
    run_git_in(&work_path, &["config", "user.email", "test@example.com"]);
    run_git_in(&work_path, &["config", "user.name", "paam-test"]);
    run_git_in(&work_path, &["commit", "--allow-empty", "-m", "init"]);
    run_git_in(&work_path, &["push", bare_str, "HEAD:master"]);

    let url = format!("file://{}", bare_path.display());
    (dir, url)
}

/// 全局环境变量串行锁。
///
/// 进程级 `std::env::set_var` 与 `remove_var` 在 cargo test 默认并行下会互相
/// 污染（如 sync 模块测试与 install 模块测试都用 `PAAM_CLAUDE_TARGET_DIR`）。
/// 任何用 env_var 的测试 SHALL 在测试入口 `let _g = acquire_env_lock();`
/// 一份。所有这类测试共享同一把锁 → 串行执行。
///
/// **注意**：返回的 `MutexGuard` 已对 `PoisonError` 容错——某个测试持锁时
/// panic 会让锁 poison，若用 `lock().unwrap()` 后续测试会全军覆没（CI 上常
/// 见的 PoisonError 雪崩）；此 helper 用 `unwrap_or_else(into_inner)` 让真正
/// 的根因测试单独失败，不连累其它。
pub fn acquire_env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

/// 构造一个含若干文件的本地 git 仓（worktree 形式，非 bare），返回 TempDir。
///
/// `files` 形如 `&[("path/in/repo.txt", "content"), ...]`。所有文件被 add + commit。
/// 用于：测试 git helper（head_commit / subtree_hash）、install 模块构造 source 仓。
pub fn make_repo_with_files(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    run_git_in(dir.path(), &["init", "--quiet"]);
    run_git_in(dir.path(), &["config", "user.email", "test@example.com"]);
    run_git_in(dir.path(), &["config", "user.name", "paam-test"]);
    for (rel, content) in files {
        let p = dir.path().join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, content).unwrap();
    }
    run_git_in(dir.path(), &["add", "-A"]);
    run_git_in(dir.path(), &["commit", "-m", "init", "--quiet"]);
    dir
}
