pub mod schema;

use std::fs;
use std::io::Write;

pub use schema::{Config, Source, CURRENT_SCHEMA_VERSION};

use crate::error::{Error, Result};
use crate::paths::PaamRoot;

/// 读取 `<root>/config.json`。文件不存在时返回空配置。
pub fn load(root: &PaamRoot) -> Result<Config> {
    let path = root.config_file();
    if !path.exists() {
        return Ok(Config::new_empty());
    }
    let bytes = fs::read(&path)?;
    let config: Config = serde_json::from_slice(&bytes)?;
    if config.version > CURRENT_SCHEMA_VERSION {
        return Err(Error::UnsupportedSchemaVersion {
            found: config.version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }
    Ok(config)
}

/// 原子写：先写到 `config.json.tmp`，再 rename 覆盖原文件。
pub fn save(root: &PaamRoot, config: &Config) -> Result<()> {
    let path = root.config_file();
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        let bytes = serde_json::to_vec_pretty(config)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// 业务级 API：追加一个订阅源。alias 重复时返回 `AliasAlreadyExists`。
pub fn add_source(root: &PaamRoot, alias: &str, url: &str) -> Result<()> {
    let mut config = load(root)?;
    if config.sources.iter().any(|s| s.alias == alias) {
        return Err(Error::AliasAlreadyExists {
            alias: alias.to_string(),
        });
    }
    config.sources.push(Source {
        alias: alias.to_string(),
        url: url.to_string(),
        added_at: chrono::Utc::now(),
    });
    save(root, &config)
}

/// 业务级 API：列出已订阅的源。
pub fn list_sources(root: &PaamRoot) -> Result<Vec<Source>> {
    Ok(load(root)?.sources)
}

/// 解析有效的 discover 扫描忽略目录列表。
///
/// 用户在 `config.json` 中提供 `scan_ignore` 时**完全替换**内置默认；
/// 缺省 / null → 用 `discover::DEFAULT_IGNORE`；空数组 → 不忽略任何目录。
pub fn effective_scan_ignore(root: &PaamRoot) -> Result<Vec<String>> {
    let cfg = load(root)?;
    Ok(match cfg.scan_ignore {
        Some(list) => list,
        None => crate::discover::DEFAULT_IGNORE
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
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
    fn load_empty_after_initialization() {
        let (_dir, root) = fresh_root();
        let cfg = load(&root).unwrap();
        assert_eq!(cfg.version, CURRENT_SCHEMA_VERSION);
        assert!(cfg.sources.is_empty());
    }

    #[test]
    fn round_trip_add_then_list() {
        let (_dir, root) = fresh_root();
        add_source(&root, "github.com/foo/bar", "git@github.com:foo/bar.git").unwrap();
        let sources = list_sources(&root).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].alias, "github.com/foo/bar");
        assert_eq!(sources[0].url, "git@github.com:foo/bar.git");
    }

    #[test]
    fn duplicate_alias_is_rejected() {
        let (_dir, root) = fresh_root();
        add_source(&root, "github.com/foo/bar", "git@github.com:foo/bar.git").unwrap();
        let err = add_source(&root, "github.com/foo/bar", "git@github.com:foo/bar.git")
            .expect_err("重复 alias 必须被拒绝");
        assert!(matches!(err, Error::AliasAlreadyExists { .. }));
    }

    #[test]
    fn effective_scan_ignore_uses_defaults_when_field_absent() {
        let (_dir, root) = fresh_root();
        let ignore = effective_scan_ignore(&root).unwrap();
        assert!(ignore.contains(&".git".to_string()), "默认应含 .git");
        assert!(
            ignore.contains(&"node_modules".to_string()),
            "默认应含 node_modules"
        );
    }

    #[test]
    fn effective_scan_ignore_user_list_replaces_default() {
        let (dir, root) = fresh_root();
        let cfg_path = dir.path().join("config.json");
        let payload = serde_json::json!({
            "version": 1,
            "sources": [],
            "scan_ignore": ["target", "my-private"]
        });
        std::fs::write(&cfg_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        let ignore = effective_scan_ignore(&root).unwrap();
        assert_eq!(ignore, vec!["target".to_string(), "my-private".to_string()]);
        assert!(
            !ignore.contains(&".git".to_string()),
            "完全替换，无内置默认"
        );
    }

    #[test]
    fn effective_scan_ignore_empty_array_means_nothing_skipped() {
        let (dir, root) = fresh_root();
        let cfg_path = dir.path().join("config.json");
        let payload = serde_json::json!({
            "version": 1,
            "sources": [],
            "scan_ignore": []
        });
        std::fs::write(&cfg_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        let ignore = effective_scan_ignore(&root).unwrap();
        assert!(ignore.is_empty());
    }

    #[test]
    fn future_schema_version_rejected() {
        let (dir, root) = fresh_root();
        let cfg_path = dir.path().join("config.json");
        let payload = serde_json::json!({ "version": 99, "sources": [] });
        std::fs::write(&cfg_path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();
        let err = load(&root).expect_err("更高版本应被拒绝");
        assert!(matches!(
            err,
            Error::UnsupportedSchemaVersion { found: 99, .. }
        ));
    }
}
