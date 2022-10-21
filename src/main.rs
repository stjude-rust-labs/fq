use clap::Parser;
use fq::{
    cli::Command,
    commands::{filter, generate, lint, subsample},
    Cli,
};

use tracing::warn;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if cli.verbose {
        warn!("`--verbose` is deprecated and will be removed in a future version. Logging is now always enabled.");
    }

    match cli.command {
        Command::Filter(args) => filter(args),
        Command::Generate(args) => generate(args),
        Command::Lint(args) => lint(args),
        Command::Subsample(args) => subsample(args),
    }
}
