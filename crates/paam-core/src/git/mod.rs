//! 与 git 子进程交互的薄层封装。
//!
//! 全部远程 / 本地 git 操作都通过 `Command::new("git")` 执行。paam-core 内不再
//! 引用任何 in-process git 库（详见 `swap-git-transport-to-cli` change 决策 2）。

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

/// 在执行任何远程 git 操作前调用，确保 PATH 上存在可执行的 `git`。
///
/// `paam track <url>` 等命令应在入口处调用本函数；纯本地命令（如
/// `paam track list`）无需调用。
pub fn ensure_git_available() -> Result<()> {
    let output = Command::new("git").arg("--version").output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(Error::GitProcessFailure {
            exit_code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(Error::GitNotFound),
        Err(e) => Err(Error::Io(e)),
    }
}

/// 内部 helper：以同步方式 fork 一个 git 子进程，stderr 透传到当前终端。
///
/// 后续 change（fetch / commit / status 等）也将通过此函数复用相同的错误映射。
pub(crate) fn run(args: &[&str], cwd: Option<&Path>) -> Result<()> {
    tracing::debug!(?args, ?cwd, "git run");
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd.stderr(Stdio::inherit());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::GitNotFound),
        Err(e) => return Err(Error::Io(e)),
    };
    if status.success() {
        Ok(())
    } else {
        Err(Error::GitProcessFailure {
            exit_code: status.code(),
            stderr: "<see terminal>".into(),
        })
    }
}

/// 将远程 git 仓 clone 到 `dest`。鉴权与配置完全交由系统 git / OpenSSH 处理。
pub fn clone(url: &str, dest: &Path) -> Result<()> {
    let dest_str = dest.to_str().ok_or_else(|| {
        Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "目标路径含非 UTF-8 字节，paam M1 仅支持 UTF-8 路径",
        ))
    })?;
    run(&["clone", "--quiet", "--no-tags", url, dest_str], None)
}

/// 内部 helper：fork git，捕获 stdout 返回（trim 末尾换行）；stderr 透传。
///
/// 错误映射与 `run` 一致；用于 `rev-parse` / `config get` 等需要取 stdout 的命令。
pub(crate) fn run_capture(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    tracing::debug!(?args, ?cwd, "git run_capture");
    let mut cmd = Command::new("git");
    cmd.args(args);
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::piped());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Error::GitNotFound),
        Err(e) => return Err(Error::Io(e)),
    };
    if !output.status.success() {
        return Err(Error::GitProcessFailure {
            exit_code: output.status.code(),
            stderr: "<see terminal>".into(),
        });
    }
    let s = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    Ok(s)
}

/// 取 `repo` 仓 HEAD 当前指向的 commit 全长 hash（40 位 hex）。
pub fn head_commit(repo: &Path) -> Result<String> {
    run_capture(&["rev-parse", "HEAD"], Some(repo))
}

/// 取 `repo` 仓 HEAD 中 `subpath` 对应 subtree 的 git tree hash（40 位 hex）。
///
/// 调用者保证 `subpath` 是相对路径（不含前导 `/` 或 `./`）；本函数会去除前导 `./`。
pub fn subtree_hash(repo: &Path, subpath: &str) -> Result<String> {
    let cleaned = subpath.trim_start_matches("./").trim_start_matches('/');
    let spec = format!("HEAD:{}", cleaned);
    run_capture(&["rev-parse", &spec], Some(repo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::make_fixture_repo;
    use tempfile::TempDir;

    #[test]
    fn ensure_git_available_when_git_in_path() {
        ensure_git_available().expect("CI / 开发机必有 git");
    }

    #[test]
    fn clone_local_bare_repo_via_file_protocol() {
        let (_fixture, url) = make_fixture_repo();
        let dest_dir = TempDir::new().unwrap();
        let dest = dest_dir.path().join("clone");
        clone(&url, &dest).unwrap();
        assert!(dest.join(".git").is_dir(), "clone 目标应包含 .git/");
    }

    #[test]
    fn clone_failure_returns_git_process_failure() {
        let dest_dir = TempDir::new().unwrap();
        let dest = dest_dir.path().join("clone");
        let bogus = "file:///definitely/does/not/exist/origin.git";
        let err = clone(bogus, &dest).expect_err("不存在的远程必须失败");
        assert!(
            matches!(err, Error::GitProcessFailure { .. }),
            "应映射为 GitProcessFailure，实际：{:?}",
            err
        );
    }

    #[test]
    fn head_commit_returns_full_hash() {
        use crate::test_support::make_repo_with_files;
        let dir = make_repo_with_files(&[("README.md", "hello\n")]);
        let h = head_commit(dir.path()).unwrap();
        assert_eq!(h.len(), 40, "实际：{:?}", h);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn subtree_hash_differs_for_different_subpaths() {
        use crate::test_support::make_repo_with_files;
        let dir = make_repo_with_files(&[
            ("a/SKILL.md", "---\nname: a\ndescription: A\n---\n"),
            ("b/SKILL.md", "---\nname: b\ndescription: B\n---\n"),
        ]);
        let ha = subtree_hash(dir.path(), "a").unwrap();
        let hb = subtree_hash(dir.path(), "b").unwrap();
        assert_eq!(ha.len(), 40);
        assert_eq!(hb.len(), 40);
        assert_ne!(ha, hb, "不同 subpath 的 tree hash 必须不同");
    }

    #[test]
    fn subtree_hash_strips_leading_dot_slash() {
        use crate::test_support::make_repo_with_files;
        let dir = make_repo_with_files(&[("a/x.md", "x")]);
        let h1 = subtree_hash(dir.path(), "a").unwrap();
        let h2 = subtree_hash(dir.path(), "./a").unwrap();
        assert_eq!(h1, h2);
    }
}
