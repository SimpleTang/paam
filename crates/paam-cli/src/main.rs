//! paam — Private AI Asset Manager (CLI entry).

use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use paam_core::asset::{Asset, Skill};
use paam_core::metadata::InstalledAsset;
use paam_core::sync::SyncReport;
use paam_core::{
    config, config::Source, discover, install, metadata, source, sync, Error, PaamRoot,
};

#[derive(Parser, Debug)]
#[command(
    name = "paam",
    version,
    about = "Private AI Asset Manager — manage your AI Agent assets"
)]
struct Cli {
    /// 提高日志详细程度（DEBUG）
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// 管理订阅源。用法：
    ///   `paam track <ssh-url>`         添加新订阅源
    ///   `paam track list`              列出已订阅源
    ///   `paam track skills <alias>`    列出 alias 仓内的可用 Skill
    Track(TrackArgs),

    /// 管理 Skill 资产（install / list / uninstall）
    Skill(SkillArgs),

    /// 列出所有已安装资产（type-agnostic 全量概览）
    List,

    /// 同步已安装资产到目标 Agent 工作目录（M1 仅 Claude Code）
    Sync(SyncArgs),

    /// 解除同步：仅删 target symlink + 清 metadata.targets[]，不动 local-repo
    Unsync(UnsyncArgs),
}

#[derive(Args, Debug)]
struct SyncArgs {
    /// 强制覆盖非 paam 管理的 target 内容
    #[arg(long)]
    force: bool,
}

#[derive(Args, Debug)]
struct UnsyncArgs {
    /// 要解除同步的 skill 名（与 --all 互斥）
    name: Option<String>,
    /// 解除所有已同步资产（与 name 互斥）
    #[arg(long, conflicts_with = "name")]
    all: bool,
}

#[derive(Args, Debug)]
struct TrackArgs {
    /// 见 `paam track --help`
    #[arg(value_name = "TARGET", num_args = 1..=2)]
    target: Vec<String>,
}

#[derive(Args, Debug)]
struct SkillArgs {
    #[command(subcommand)]
    cmd: SkillCmd,
}

#[derive(Subcommand, Debug)]
enum SkillCmd {
    /// 安装 Skill 到本地工作集
    Install {
        /// Skill 名（来自 SKILL.md frontmatter 的 name 字段）
        name: String,
        /// 跨 source 同名时显式指定来源 alias
        #[arg(long)]
        from: Option<String>,
        /// 已安装时强制重装（先删除既有再装）
        #[arg(long)]
        force: bool,
    },
    /// 列出已安装的 Skill
    List,
    /// 卸载已安装的 Skill
    Uninstall {
        /// Skill 名
        name: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let root = match PaamRoot::from_env() {
        Ok(r) => r,
        Err(e) => {
            report_error(&e);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = root.ensure_initialized() {
        report_error(&e);
        return ExitCode::FAILURE;
    }

    let result = match cli.command {
        Cmd::Track(args) => dispatch_track(&root, &args.target),
        Cmd::Skill(args) => dispatch_skill(&root, args.cmd),
        Cmd::List => handle_list(&root),
        Cmd::Sync(args) => handle_sync(&root, args.force),
        Cmd::Unsync(args) => handle_unsync(&root, args.name.as_deref(), args.all),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            report_error(&e);
            ExitCode::FAILURE
        }
    }
}

fn init_tracing(verbose: bool) {
    let level = if verbose { Level::DEBUG } else { Level::INFO };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .try_init();
}

fn dispatch_track(root: &PaamRoot, target: &[String]) -> Result<(), Error> {
    match target {
        [single] if single == "list" => handle_track_list(root),
        [single] if single == "skills" => Err(Error::InvalidUsage(
            "缺少 alias 参数：paam track skills <alias>".to_string(),
        )),
        [single] => handle_track_add(root, single),
        [first, second] if first == "skills" => handle_track_skills(root, second),
        _ => Err(Error::InvalidUsage(
            "用法：\n  paam track <ssh-url>\n  paam track list\n  paam track skills <alias>"
                .to_string(),
        )),
    }
}

fn handle_track_add(root: &PaamRoot, url: &str) -> Result<(), Error> {
    let outcome = source::track(root, url)?;
    println!("已订阅 alias={}", outcome.alias);
    println!("本地路径={}", outcome.local_path.display());
    Ok(())
}

fn handle_track_list(root: &PaamRoot) -> Result<(), Error> {
    let sources = source::list_sources(root)?;
    if sources.is_empty() {
        println!("暂无已订阅的源，使用 `paam track <git-url>` 添加");
        return Ok(());
    }
    print_sources(&sources);
    Ok(())
}

fn print_sources(sources: &[Source]) {
    let alias_w = sources
        .iter()
        .map(|s| s.alias.len())
        .max()
        .unwrap_or(0)
        .max(5);
    let url_w = sources
        .iter()
        .map(|s| s.url.len())
        .max()
        .unwrap_or(0)
        .max(3);
    println!(
        "{:<alias_w$}  {:<url_w$}  ADDED_AT",
        "ALIAS",
        "URL",
        alias_w = alias_w,
        url_w = url_w
    );
    for s in sources {
        println!(
            "{:<alias_w$}  {:<url_w$}  {}",
            s.alias,
            s.url,
            s.added_at.to_rfc3339(),
            alias_w = alias_w,
            url_w = url_w
        );
    }
}

fn handle_track_skills(root: &PaamRoot, alias: &str) -> Result<(), Error> {
    let sources = source::list_sources(root)?;
    if !sources.iter().any(|s| s.alias == alias) {
        return Err(Error::AliasNotFound {
            alias: alias.to_string(),
        });
    }
    let local_path = root.sources_dir().join(alias);
    let ignore = config::effective_scan_ignore(root)?;
    let skills = discover::skills_in(&local_path, alias, &ignore);
    if skills.is_empty() {
        println!("该订阅源中暂无可用的 Skill（未发现任何 SKILL.md 文件）");
        println!(
            "提示：如有 SKILL.md 但被忽略，可检查 `~/.paam/config.json` 中的 `scan_ignore` 字段"
        );
        return Ok(());
    }
    print_skills(&skills);
    Ok(())
}

fn print_skills(skills: &[Skill]) {
    const DESC_MAX_CHARS: usize = 60;
    let name_w = skills
        .iter()
        .map(|s| s.id().chars().count())
        .max()
        .unwrap_or(0)
        .max(4);

    println!(
        "{:<name_w$}  {:<DESC_MAX_CHARS$}  PATH",
        "NAME",
        "DESCRIPTION",
        name_w = name_w,
        DESC_MAX_CHARS = DESC_MAX_CHARS
    );
    for s in skills {
        let desc = truncate_description(s.description(), DESC_MAX_CHARS);
        println!(
            "{:<name_w$}  {:<DESC_MAX_CHARS$}  {}/",
            s.id(),
            desc,
            s.relative_path().display(),
            name_w = name_w,
            DESC_MAX_CHARS = DESC_MAX_CHARS
        );
    }
}

fn truncate_description(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        // 截断到 max_chars - 1，末尾加 …，总长度仍 ≤ max_chars
        let cutoff = max_chars.saturating_sub(1);
        let mut out: String = chars.iter().take(cutoff).collect();
        out.push('…');
        out
    }
}

fn dispatch_skill(root: &PaamRoot, cmd: SkillCmd) -> Result<(), Error> {
    match cmd {
        SkillCmd::Install { name, from, force } => {
            handle_skill_install(root, &name, from.as_deref(), force)
        }
        SkillCmd::List => handle_skill_list(root),
        SkillCmd::Uninstall { name } => handle_skill_uninstall(root, &name),
    }
}

fn handle_skill_install(
    root: &PaamRoot,
    name: &str,
    from: Option<&str>,
    force: bool,
) -> Result<(), Error> {
    let resolved = install::resolve_skill(root, name, from)?;
    let meta = install::install_skill(root, &resolved, force)?;
    let action = if force {
        "已重新安装"
    } else {
        "已安装"
    };
    println!("{} {}", action, meta.name);
    println!("  来源={}", meta.origin.repo);
    println!(
        "  本地路径={}",
        metadata::skill_dir(root, &meta.name).display()
    );
    Ok(())
}

fn handle_skill_uninstall(root: &PaamRoot, name: &str) -> Result<(), Error> {
    install::uninstall_skill(root, name)?;
    println!("已卸载 {}", name);
    Ok(())
}

fn handle_skill_list(root: &PaamRoot) -> Result<(), Error> {
    let skills = metadata::list_skills(root)?;
    if skills.is_empty() {
        println!("暂无已安装的 Skill，使用 `paam skill install <name>` 安装");
        return Ok(());
    }
    print_installed_table(&skills, false);
    Ok(())
}

fn handle_list(root: &PaamRoot) -> Result<(), Error> {
    let assets = metadata::list_installed(root)?;
    if assets.is_empty() {
        println!("暂无已安装的资产，使用 `paam skill install <name>` 安装");
        return Ok(());
    }
    print_installed_table(&assets, true);
    Ok(())
}

fn print_installed_table(rows: &[InstalledAsset], with_type: bool) {
    let name_w = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(4);
    let source_w = rows
        .iter()
        .map(|r| r.origin.repo.len())
        .max()
        .unwrap_or(0)
        .max(6);
    if with_type {
        let type_w = rows
            .iter()
            .map(|r| format!("{:?}", r.asset_type).to_lowercase().len())
            .max()
            .unwrap_or(0)
            .max(4);
        println!(
            "{:<name_w$}  {:<type_w$}  {:<source_w$}  INSTALLED_AT",
            "NAME",
            "TYPE",
            "SOURCE",
            name_w = name_w,
            type_w = type_w,
            source_w = source_w
        );
        for r in rows {
            let kind = format!("{:?}", r.asset_type).to_lowercase();
            println!(
                "{:<name_w$}  {:<type_w$}  {:<source_w$}  {}",
                r.name,
                kind,
                r.origin.repo,
                r.installed_at.to_rfc3339(),
                name_w = name_w,
                type_w = type_w,
                source_w = source_w
            );
        }
    } else {
        println!(
            "{:<name_w$}  {:<source_w$}  INSTALLED_AT",
            "NAME",
            "SOURCE",
            name_w = name_w,
            source_w = source_w
        );
        for r in rows {
            println!(
                "{:<name_w$}  {:<source_w$}  {}",
                r.name,
                r.origin.repo,
                r.installed_at.to_rfc3339(),
                name_w = name_w,
                source_w = source_w
            );
        }
    }
}

fn handle_sync(root: &PaamRoot, force: bool) -> Result<(), Error> {
    let report = sync::sync_all(root, force)?;
    print_sync_report(&report);
    Ok(())
}

fn handle_unsync(root: &PaamRoot, name: Option<&str>, all: bool) -> Result<(), Error> {
    match (name, all) {
        (Some(_), true) => Err(Error::InvalidUsage("--all 与 <name> 互斥".to_string())),
        (None, false) => Err(Error::InvalidUsage(
            "用法：\n  paam unsync <name>\n  paam unsync --all".to_string(),
        )),
        (Some(n), false) => {
            sync::unsync_one(root, n)?;
            println!("已解除同步 {}", n);
            Ok(())
        }
        (None, true) => {
            sync::unsync_all(root)?;
            println!("已解除所有同步");
            Ok(())
        }
    }
}

fn print_sync_report(report: &SyncReport) {
    let total = report.synced.len()
        + report.already_ok.len()
        + report.conflicts.len()
        + report.forced.len();
    if total == 0 {
        println!("暂无已安装的资产可同步，使用 `paam skill install <name>` 安装");
        return;
    }
    if !report.synced.is_empty() {
        println!("已同步 ({}):", report.synced.len());
        for n in &report.synced {
            println!("  + {}", n);
        }
    }
    if !report.forced.is_empty() {
        println!("已强制覆盖 ({}):", report.forced.len());
        for n in &report.forced {
            println!("  ! {}", n);
        }
    }
    if !report.already_ok.is_empty() {
        println!("已正确指向 ({}):", report.already_ok.len());
        for n in &report.already_ok {
            println!("  = {}", n);
        }
    }
    if !report.conflicts.is_empty() {
        println!("冲突 ({}):", report.conflicts.len());
        for c in &report.conflicts {
            println!(
                "  ? {}  目标={}  原因={}",
                c.skill_name,
                c.target_path.display(),
                c.reason
            );
        }
        println!();
        println!("提示：使用 `paam sync --force` 覆盖；或手动整理对应目录后重试");
    }
}

fn report_error(err: &Error) {
    // GitProcessFailure 时，git 自身的 stderr 已通过 Stdio::inherit 透传到终端；
    // 这里仅打印 paam 层的错误描述（含 exit code）。
    eprintln!("错误：{}", err);
}
