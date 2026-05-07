use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("无法解析用户主目录")]
    HomeNotFound,

    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("配置文件 schema 版本 {found} 高于当前支持的最高版本 {supported}，请升级 paam")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },

    #[error(
        "不支持的 git URL 形式：{url}\n  M1 仅支持 SSH URL，请改用 `git@host:owner/repo.git` 形式"
    )]
    InvalidGitUrl { url: String },

    #[error("订阅源已存在: alias={alias}")]
    AliasAlreadyExists { alias: String },

    #[error("订阅源不存在: alias={alias}\n  使用 `paam track list` 查看已订阅源")]
    AliasNotFound { alias: String },

    #[error("命令用法错误：{0}")]
    InvalidUsage(String),

    #[error("未找到名为 `{name}` 的 skill。\n  提示：使用 `paam track skills <alias>` 查看仓内可用 skill")]
    SkillNotFound { name: String },

    #[error(
        "skill `{name}` 在多个 source 中存在：{candidates:?}\n  请用 `paam skill install {name} --from <alias>` 显式指定"
    )]
    AmbiguousSkill {
        name: String,
        candidates: Vec<String>,
    },

    #[error("skill `{name}` 已安装。\n  使用 `--force` 重装，或先 `paam skill uninstall {name}`")]
    AlreadyInstalled { name: String },

    #[error("skill `{name}` 未安装")]
    NotInstalled { name: String },

    #[error("同步 `{skill}` 到 {target} 时 IO 错误：{message}")]
    SyncIo {
        skill: String,
        target: std::path::PathBuf,
        message: String,
    },

    #[error(
        "系统未安装 git，或 git 不在 PATH 中。请先安装：\n  \
         brew install git    （macOS）\n  \
         apt install git     （Debian / Ubuntu）\n  \
         dnf install git     （Fedora）"
    )]
    GitNotFound,

    #[error("git 子进程失败（exit code: {exit_code:?}）：\n{stderr}")]
    GitProcessFailure {
        exit_code: Option<i32>,
        stderr: String,
    },
}
