use std::path::{Path, PathBuf};

use directories_next::BaseDirs;

use crate::error::{Error, Result};

const PAAM_DIR_NAME: &str = ".paam";
const ENV_PAAM_HOME: &str = "PAAM_HOME";
const SOURCES_SUBDIR: &str = "sources";
const CONFIG_FILENAME: &str = "config.json";

/// 测试与自定义部署用：覆盖默认 `~/.claude/skills/` 的 target 路径。
pub const ENV_PAAM_CLAUDE_TARGET_DIR: &str = "PAAM_CLAUDE_TARGET_DIR";

/// 业务上下文：paam 的工作目录根。
///
/// 默认从环境变量 `PAAM_HOME` 解析；未设置时回退到 `~/.paam/`。
/// 测试场景下可通过 `PaamRoot::at(tempdir)` 注入隔离目录，避免触碰真实 home。
#[derive(Debug, Clone)]
pub struct PaamRoot {
    home: PathBuf,
}

impl PaamRoot {
    /// 解析当前进程的 paam home 目录（优先 `PAAM_HOME`，否则 `~/.paam/`）。
    pub fn from_env() -> Result<Self> {
        if let Ok(custom) = std::env::var(ENV_PAAM_HOME) {
            if !custom.is_empty() {
                return Ok(Self {
                    home: PathBuf::from(custom),
                });
            }
        }
        let dirs = BaseDirs::new().ok_or(Error::HomeNotFound)?;
        Ok(Self {
            home: dirs.home_dir().join(PAAM_DIR_NAME),
        })
    }

    /// 用任意路径构造 root（测试或自定义部署场景使用）。
    pub fn at(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn sources_dir(&self) -> PathBuf {
        self.home.join(SOURCES_SUBDIR)
    }

    pub fn config_file(&self) -> PathBuf {
        self.home.join(CONFIG_FILENAME)
    }

    /// 幂等地创建 `~/.paam/`、`~/.paam/sources/` 与初始 `config.json`。
    pub fn ensure_initialized(&self) -> Result<()> {
        std::fs::create_dir_all(&self.home)?;
        std::fs::create_dir_all(self.sources_dir())?;
        let cfg = self.config_file();
        if !cfg.exists() {
            let empty = crate::config::schema::Config::new_empty();
            let bytes = serde_json::to_vec_pretty(&empty)?;
            std::fs::write(&cfg, bytes)?;
        }
        Ok(())
    }
}

/// 当前进程的 paam home 目录路径（基于 `PaamRoot::from_env`）。
pub fn paam_home() -> Result<PathBuf> {
    Ok(PaamRoot::from_env()?.home)
}

pub fn sources_dir() -> Result<PathBuf> {
    Ok(PaamRoot::from_env()?.sources_dir())
}

pub fn config_file() -> Result<PathBuf> {
    Ok(PaamRoot::from_env()?.config_file())
}

pub fn ensure_initialized() -> Result<()> {
    PaamRoot::from_env()?.ensure_initialized()
}

/// 解析 Claude Code 的 skill 目录路径（sync 的 target）。
///
/// 优先读环境变量 `PAAM_CLAUDE_TARGET_DIR`（用于测试与自定义部署）；
/// 未设置时回退到 `~/.claude/skills/`。该路径不被 `PaamRoot` 持有
/// （target 是 paam 输出方向，与工作目录语义不同）。
pub fn claude_skills_target_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var(ENV_PAAM_CLAUDE_TARGET_DIR) {
        if !custom.is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    let dirs = BaseDirs::new().ok_or(Error::HomeNotFound)?;
    Ok(dirs.home_dir().join(".claude").join("skills"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ensure_initialized_creates_layout_and_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let root = PaamRoot::at(dir.path());

        root.ensure_initialized().unwrap();
        assert!(dir.path().is_dir());
        assert!(dir.path().join("sources").is_dir());
        let cfg = dir.path().join("config.json");
        assert!(cfg.is_file());
        let first_contents = std::fs::read_to_string(&cfg).unwrap();

        root.ensure_initialized().unwrap();
        let second_contents = std::fs::read_to_string(&cfg).unwrap();
        assert_eq!(
            first_contents, second_contents,
            "已有 config.json 不应被覆盖"
        );
    }

    #[test]
    fn paths_derive_from_root() {
        let root = PaamRoot::at("/tmp/paam-test-xyz");
        assert_eq!(
            root.sources_dir(),
            PathBuf::from("/tmp/paam-test-xyz/sources")
        );
        assert_eq!(
            root.config_file(),
            PathBuf::from("/tmp/paam-test-xyz/config.json")
        );
    }

    #[test]
    fn claude_target_dir_uses_env_when_set() {
        let _g = crate::test_support::env_lock().lock().unwrap();
        std::env::set_var(ENV_PAAM_CLAUDE_TARGET_DIR, "/tmp/paam-target-xyz");
        let p = claude_skills_target_dir().unwrap();
        std::env::remove_var(ENV_PAAM_CLAUDE_TARGET_DIR);
        assert_eq!(p, PathBuf::from("/tmp/paam-target-xyz"));
    }

    #[test]
    fn claude_target_dir_empty_env_is_treated_as_unset() {
        let _g = crate::test_support::env_lock().lock().unwrap();
        std::env::set_var(ENV_PAAM_CLAUDE_TARGET_DIR, "");
        let p = claude_skills_target_dir().unwrap();
        std::env::remove_var(ENV_PAAM_CLAUDE_TARGET_DIR);
        // 空字符串 → 走默认路径；至少应以 ".claude/skills" 结尾
        assert!(p.ends_with(".claude/skills"), "实际：{}", p.display());
    }

    #[test]
    fn claude_target_dir_default_is_claude_skills_under_home() {
        let _g = crate::test_support::env_lock().lock().unwrap();
        std::env::remove_var(ENV_PAAM_CLAUDE_TARGET_DIR);
        let p = claude_skills_target_dir().unwrap();
        assert!(p.ends_with(".claude/skills"));
    }
}
