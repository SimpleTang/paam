use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub sources: Vec<Source>,

    /// 自定义 discover 扫描时要跳过的目录名列表。
    /// 缺省 / null → 用 `discover::DEFAULT_IGNORE`；非空数组 → 完全替换默认；空数组 → 不忽略任何目录。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_ignore: Option<Vec<String>>,
}

impl Config {
    pub fn new_empty() -> Self {
        Self {
            version: CURRENT_SCHEMA_VERSION,
            sources: Vec::new(),
            scan_ignore: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub alias: String,
    pub url: String,
    pub added_at: DateTime<Utc>,
}
