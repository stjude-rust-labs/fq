use std::io;

use clap::Parser;
use fq::{
    cli::Command,
    commands::{filter, generate, lint, subsample},
    Cli,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let cli = Cli::parse();

    match cli.command {
        Command::Filter(args) => filter(args)?,
        Command::Generate(args) => generate(args)?,
        Command::Lint(args) => lint(args)?,
        Command::Subsample(args) => subsample(args)?,
    }

    Ok(())
}
