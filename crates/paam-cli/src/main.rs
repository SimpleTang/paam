//! paam — Private AI Asset Manager (CLI entry).

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "paam",
    version,
    about = "Private AI Asset Manager — manage your AI Agent assets"
)]
struct Cli {
    // M1 subcommands will be added: track, install, sync, list, etc.
}

fn main() {
    let _cli = Cli::parse();
    println!(
        "paam {} — see https://github.com/SimpleTang/paam",
        env!("CARGO_PKG_VERSION")
    );
}
