use std::path::PathBuf;

use clap::{value_parser, Arg, ArgAction, Command};
use fq::{
    commands::{filter, generate, lint, subsample},
    validators::LintMode,
    ValidationLevel,
};

use git_testament::{git_testament, render_testament};
use tracing::warn;

git_testament!(TESTAMENT);

fn main() -> anyhow::Result<()> {
    let version = render_testament!(TESTAMENT);

    let filter_cmd = Command::new("filter")
        .about("Filters a FASTQ from an allowlist of names")
        .arg(
            Arg::new("names")
                .long("names")
                .value_name("path")
                .help("Allowlist of record names")
                .required(true),
        )
        .arg(
            Arg::new("src")
                .help("Source FASTQ")
                .index(1)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        );

    let generate_cmd = Command::new("generate")
        .about("Generates a random FASTQ file pair")
        .arg(
            Arg::new("seed")
                .short('s')
                .long("seed")
                .value_name("u64")
                .value_parser(value_parser!(u64))
                .help("Seed to use for the random number generator"),
        )
        .arg(
            Arg::new("record-count")
                .short('n')
                .long("record-count")
                .help("Number of records to generate")
                .value_name("u64")
                .value_parser(value_parser!(u64))
                .default_value("10000"),
        )
        .arg(
            Arg::new("read-length")
                .long("read-length")
                .help("Number of bases in the sequence")
                .value_name("usize")
                .value_parser(value_parser!(usize))
                .default_value("101"),
        )
        .arg(
            Arg::new("r1-dst")
                .help("Read 1 destination. Output will be gzipped if ends in `.gz`.")
                .index(1)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            Arg::new("r2-dst")
                .help("Read 2 destination. Output will be gzipped if ends in `.gz`.")
                .index(2)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        );

    let lint_cmd = Command::new("lint")
        .about("Validates a FASTQ file pair")
        .arg(
            Arg::new("lint-mode")
                .long("lint-mode")
                .help("Panic on first error or log all errors")
                .value_name("str")
                .value_parser(value_parser!(LintMode))
                .default_value("panic"),
        )
        .arg(
            Arg::new("single-read-validation-level")
                .long("single-read-validation-level")
                .help("Only use single read validators up to a given level")
                .value_name("str")
                .value_parser(value_parser!(ValidationLevel))
                .default_value("high"),
        )
        .arg(
            Arg::new("paired-read-validation-level")
                .long("paired-read-validation-level")
                .help("Only use paired read validators up to a given level")
                .value_name("str")
                .value_parser(value_parser!(ValidationLevel))
                .default_value("high"),
        )
        .arg(
            Arg::new("disable-validator")
                .long("disable-validator")
                .help("Disable validators by code. Use multiple times to disable more than one.")
                .value_name("str")
                .action(ArgAction::Append)
                .num_args(1),
        )
        .arg(
            Arg::new("r1-src")
                .help("Read 1 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(1)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            Arg::new("r2-src")
                .help("Read 2 source. Accepts both raw and gzipped FASTQ inputs.")
                .value_parser(value_parser!(PathBuf))
                .index(2),
        );

    let subsample_cmd = Command::new("subsample")
        .about("Outputs a subset of records")
        .arg(
            Arg::new("probability")
                .short('p')
                .long("probability")
                .value_name("f64")
                .value_parser(value_parser!(f64))
                .help("The probability a record is kept, as a percentage (0.0, 1.0). Cannot be used with `record-count`.")
                .required(true)
                .conflicts_with("record-count"),
        )
        .arg(
            Arg::new("record-count")
                .short('n')
                .long("record-count")
                .value_name("u64")
                .value_parser(value_parser!(u64))
                .help("The exact number of records to keep. Cannot be used with `probability`.")
                .required(true)
                .conflicts_with("probability"),
        )
        .arg(
            Arg::new("seed")
                .short('s')
                .long("seed")
                .value_name("u64")
                .value_parser(value_parser!(u64))
                .help("Seed to use for the random number generator"),
        )
        .arg(
            Arg::new("r1-dst")
                .help("Read 1 destination. Output will be gzipped if ends in `.gz`.")
                .long("r1-dst")
                .value_name("path")
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            Arg::new("r2-dst")
                .help("Read 2 destination. Output will be gzipped if ends in `.gz`.")
                .long("r2-dst")
                .value_name("path")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("r1-src")
                .help("Read 1 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(1)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            Arg::new("r2-src")
                .help("Read 2 source. Accepts both raw and gzipped FASTQ inputs.")
                .value_parser(value_parser!(PathBuf))
                .index(2),
        );

    let matches = Command::new("fq")
        .version(version)
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .hide(true),
        )
        .subcommand(filter_cmd)
        .subcommand(generate_cmd)
        .subcommand(lint_cmd)
        .subcommand(subsample_cmd)
        .get_matches();

    tracing_subscriber::fmt::init();

    if let Some(true) = matches.get_one("verbose").copied() {
        warn!("`--verbose` is deprecated and will be removed in a future version. Logging is now always enabled.");
    }

    if let Some(m) = matches.subcommand_matches("filter") {
        filter(m)
    } else if let Some(m) = matches.subcommand_matches("generate") {
        generate(m)
    } else if let Some(m) = matches.subcommand_matches("lint") {
        lint(m)
    } else if let Some(m) = matches.subcommand_matches("subsample") {
        subsample(m)
    } else {
        unreachable!();
    }
}
