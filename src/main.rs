use std::io;

use clap::Parser;
use fq::{
    Cli,
    cli::Command,
    commands::{describe, filter, generate, lint, subsample},
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let cli = Cli::parse();

    match cli.command {
        Command::Describe(args) => describe(args)?,
        Command::Filter(args) => filter(args)?,
        Command::Generate(args) => generate(args)?,
        Command::Lint(args) => lint(args)?,
        Command::Subsample(args) => subsample(args)?,
    }

    Ok(())
}
