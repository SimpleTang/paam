use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// SSH git URL 解析后的三段标识。`host` / `owner` / `repo` 保留原始大小写；
/// `alias()` 与 `cache_dir()` 在派生输出时统一小写化。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocator {
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl SourceLocator {
    /// `<host>/<owner>/<repo>`（小写化）—— 全局唯一别名，也是本地缓存目录的相对路径。
    pub fn alias(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host.to_ascii_lowercase(),
            self.owner.to_ascii_lowercase(),
            self.repo.to_ascii_lowercase()
        )
    }

    /// `<base>/<host>/<owner>/<repo>`（小写化）—— clone 的目标目录。
    pub fn cache_dir(&self, base: &Path) -> PathBuf {
        base.join(self.host.to_ascii_lowercase())
            .join(self.owner.to_ascii_lowercase())
            .join(self.repo.to_ascii_lowercase())
    }
}

/// 仅识别两种 SSH URL 形式（M1 范围）：
/// - SCP-like：`[user@]host:owner/repo[.git]`
/// - `ssh://`：`ssh://[user@]host[:port]/owner/repo[.git]`
///
/// 其他形式（HTTPS、`owner/repo` 简写、本地路径等）一律返回 `Error::InvalidGitUrl`。
pub fn parse_ssh_url(input: &str) -> Result<SourceLocator> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return invalid(input);
    }

    if let Some(rest) = trimmed.strip_prefix("ssh://") {
        return parse_ssh_proto(input, rest);
    }

    parse_scp_like(input, trimmed)
}

fn invalid(url: &str) -> Result<SourceLocator> {
    Err(Error::InvalidGitUrl {
        url: url.to_string(),
    })
}

fn parse_ssh_proto(orig: &str, rest: &str) -> Result<SourceLocator> {
    let slash = match rest.find('/') {
        Some(idx) => idx,
        None => return invalid(orig),
    };
    let (authority, path_part) = rest.split_at(slash);
    let host_part = match authority.rfind('@') {
        Some(idx) => &authority[idx + 1..],
        None => authority,
    };
    let host = match host_part.rfind(':') {
        Some(idx) => &host_part[..idx],
        None => host_part,
    };
    if host.is_empty() || host.contains('/') {
        return invalid(orig);
    }
    parse_owner_repo(orig, host, path_part)
}

fn parse_scp_like(orig: &str, input: &str) -> Result<SourceLocator> {
    let colon = match input.find(':') {
        Some(idx) => idx,
        None => return invalid(orig),
    };
    if input[colon..].starts_with("://") {
        return invalid(orig);
    }
    let authority = &input[..colon];
    let path_part = &input[colon + 1..];

    if authority.is_empty() || authority.starts_with('/') || authority.starts_with('.') {
        return invalid(orig);
    }
    let host = match authority.rfind('@') {
        Some(idx) => &authority[idx + 1..],
        None => authority,
    };
    if host.is_empty() || host.contains('/') {
        return invalid(orig);
    }
    parse_owner_repo(orig, host, path_part)
}

fn parse_owner_repo(orig: &str, host: &str, path: &str) -> Result<SourceLocator> {
    let path = path.trim_start_matches('/').trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() != 2 {
        return invalid(orig);
    }
    let owner = parts[0];
    let repo = parts[1];
    if owner.is_empty() || repo.is_empty() {
        return invalid(orig);
    }
    Ok(SourceLocator {
        host: host.to_string(),
        owner: owner.to_string(),
        repo: repo.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scp_like_basic() {
        let loc = parse_ssh_url("git@github.com:SimpleTang/paam-skills.git").unwrap();
        assert_eq!(loc.host, "github.com");
        assert_eq!(loc.owner, "SimpleTang");
        assert_eq!(loc.repo, "paam-skills");
        assert_eq!(loc.alias(), "github.com/simpletang/paam-skills");
    }

    #[test]
    fn scp_like_without_dot_git_suffix() {
        let loc = parse_ssh_url("git@github.com:foo/bar").unwrap();
        assert_eq!(loc.repo, "bar");
    }

    #[test]
    fn ssh_proto_with_port() {
        let loc = parse_ssh_url("ssh://git@gitlab.example.com:2222/team/repo.git").unwrap();
        assert_eq!(loc.host, "gitlab.example.com");
        assert_eq!(loc.owner, "team");
        assert_eq!(loc.repo, "repo");
        assert_eq!(loc.alias(), "gitlab.example.com/team/repo");
    }

    #[test]
    fn ssh_proto_without_user_or_port() {
        let loc = parse_ssh_url("ssh://gitlab.example.com/team/repo.git").unwrap();
        assert_eq!(loc.host, "gitlab.example.com");
        assert_eq!(loc.owner, "team");
        assert_eq!(loc.repo, "repo");
    }

    #[test]
    fn case_normalized_in_alias() {
        let loc = parse_ssh_url("git@GitHub.com:Foo/Bar.git").unwrap();
        assert_eq!(loc.alias(), "github.com/foo/bar");
    }

    #[test]
    fn https_url_is_rejected() {
        let err = parse_ssh_url("https://github.com/foo/bar.git").unwrap_err();
        assert!(matches!(err, Error::InvalidGitUrl { .. }));
    }

    #[test]
    fn owner_repo_shorthand_is_rejected() {
        let err = parse_ssh_url("foo/bar").unwrap_err();
        assert!(matches!(err, Error::InvalidGitUrl { .. }));
    }

    #[test]
    fn local_path_is_rejected() {
        let err = parse_ssh_url("/local/path/repo").unwrap_err();
        assert!(matches!(err, Error::InvalidGitUrl { .. }));
        let err = parse_ssh_url("./relative:path").unwrap_err();
        assert!(matches!(err, Error::InvalidGitUrl { .. }));
    }

    #[test]
    fn cache_dir_layout() {
        let loc = parse_ssh_url("git@github.com:Foo/Bar.git").unwrap();
        let dir = loc.cache_dir(Path::new("/base"));
        assert_eq!(dir, PathBuf::from("/base/github.com/foo/bar"));
    }
}
