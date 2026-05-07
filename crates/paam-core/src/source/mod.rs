pub mod url;

use std::path::PathBuf;

pub use url::{parse_ssh_url, SourceLocator};

use crate::config::{self, Source};
use crate::error::{Error, Result};
use crate::git;
use crate::paths::PaamRoot;

/// `track` 成功后返回的结果，CLI 层据此格式化输出。
#[derive(Debug, Clone)]
pub struct TrackOutcome {
    pub alias: String,
    pub local_path: PathBuf,
    pub url: String,
}

/// 添加一个订阅源：解析 URL → clone → 注册到配置文件。
/// clone 失败时清理半成品；alias 已存在时直接拒绝。
pub fn track(root: &PaamRoot, input_url: &str) -> Result<TrackOutcome> {
    root.ensure_initialized()?;
    git::ensure_git_available()?;

    let locator = parse_ssh_url(input_url)?;
    let alias = locator.alias();
    let sources_dir = root.sources_dir();
    let local_path = locator.cache_dir(&sources_dir);

    if local_path.exists() {
        return Err(Error::AliasAlreadyExists { alias });
    }
    if config::list_sources(root)?.iter().any(|s| s.alias == alias) {
        return Err(Error::AliasAlreadyExists { alias });
    }

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if let Err(e) = git::clone(input_url, &local_path) {
        let _ = std::fs::remove_dir_all(&local_path);
        return Err(e);
    }

    config::add_source(root, &alias, input_url)?;

    Ok(TrackOutcome {
        alias,
        local_path,
        url: input_url.to_string(),
    })
}

pub fn list_sources(root: &PaamRoot) -> Result<Vec<Source>> {
    config::list_sources(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::make_fixture_repo;
    use tempfile::TempDir;

    fn fresh_root() -> (TempDir, PaamRoot) {
        let dir = TempDir::new().unwrap();
        let root = PaamRoot::at(dir.path());
        root.ensure_initialized().unwrap();
        (dir, root)
    }

    /// track 走 SSH URL 路径（业务编排），但 clone 实际拿 file:// URL —— 因此
    /// 这里直接绕开 parse_ssh_url，用一个手工构造的 alias 走流程。完整流程的
    /// SSH 路径由 url.rs 单测 + 手动验收剧本覆盖。
    fn track_with_file_url(root: &PaamRoot, alias: &str, url: &str) -> Result<TrackOutcome> {
        let local_path = root.sources_dir().join(alias);
        if local_path.exists() {
            return Err(Error::AliasAlreadyExists {
                alias: alias.to_string(),
            });
        }
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Err(e) = git::clone(url, &local_path) {
            let _ = std::fs::remove_dir_all(&local_path);
            return Err(e);
        }
        config::add_source(root, alias, url)?;
        Ok(TrackOutcome {
            alias: alias.to_string(),
            local_path,
            url: url.to_string(),
        })
    }

    #[test]
    fn track_clone_register_round_trip() {
        let (_fixture_dir, fixture_url) = make_fixture_repo();
        let (_root_dir, root) = fresh_root();

        let outcome = track_with_file_url(&root, "fixture/origin", &fixture_url).unwrap();
        assert!(outcome.local_path.join(".git").is_dir());

        let listed = list_sources(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].alias, "fixture/origin");
    }

    #[test]
    fn duplicate_track_is_rejected() {
        let (_fixture_dir, fixture_url) = make_fixture_repo();
        let (_root_dir, root) = fresh_root();

        track_with_file_url(&root, "fixture/origin", &fixture_url).unwrap();
        let err = track_with_file_url(&root, "fixture/origin", &fixture_url)
            .expect_err("重复 alias 必须被拒绝");
        assert!(matches!(err, Error::AliasAlreadyExists { .. }));
    }

    #[test]
    fn clone_failure_does_not_leave_partial_dir_or_config_entry() {
        let (_root_dir, root) = fresh_root();
        let bogus_url = "file:///definitely/does/not/exist/origin.git";

        let err = track_with_file_url(&root, "bogus/origin", bogus_url)
            .expect_err("不存在的远程必须失败");
        assert!(
            matches!(err, Error::GitProcessFailure { .. }),
            "应映射为 GitProcessFailure，实际：{:?}",
            err
        );
        let local_path = root.sources_dir().join("bogus/origin");
        assert!(!local_path.exists(), "clone 失败后不应留下半成品目录");
        assert!(
            list_sources(&root).unwrap().is_empty(),
            "clone 失败时不应向 config 写入记录"
        );
    }

    #[test]
    fn track_rejects_non_ssh_url_input() {
        let (_root_dir, root) = fresh_root();
        let err = track(&root, "https://github.com/foo/bar.git").expect_err("HTTPS 应被拒绝");
        assert!(matches!(err, Error::InvalidGitUrl { .. }));
    }
}
