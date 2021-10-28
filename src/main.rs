use clap::{App, AppSettings, Arg, SubCommand};
use fqlib::commands::{filter, generate, lint, subsample};

use git_testament::{git_testament, render_testament};
use tracing::Level;

git_testament!(TESTAMENT);

fn main() -> anyhow::Result<()> {
    let filter_cmd = SubCommand::with_name("filter")
        .about("Filters a FASTQ from an allowlist of names")
        .arg(
            Arg::with_name("names")
                .long("names")
                .value_name("path")
                .help("Allowlist of record names")
                .required(true),
        )
        .arg(
            Arg::with_name("src")
                .help("Source FASTQ")
                .index(1)
                .required(true),
        );

    let generate_cmd = SubCommand::with_name("generate")
        .about("Generates a random FASTQ file pair")
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("u64")
                .help("Seed to use for the random number generator"),
        )
        .arg(
            Arg::with_name("record-count")
                .short("n")
                .long("record-count")
                .help("Number of records to generate")
                .value_name("u64")
                .default_value("10000"),
        )
        .arg(
            Arg::with_name("read-length")
                .long("read-length")
                .help("Number of bases in the sequence")
                .value_name("usize")
                .default_value("101"),
        )
        .arg(
            Arg::with_name("r1-dst")
                .help("Read 1 destination. Output will be gzipped if ends in `.gz`.")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("r2-dst")
                .help("Read 2 destination. Output will be gzipped if ends in `.gz`.")
                .index(2)
                .required(true),
        );

    let lint_cmd = SubCommand::with_name("lint")
        .about("Validates a FASTQ file pair")
        .arg(
            Arg::with_name("lint-mode")
                .long("lint-mode")
                .help("Panic on first error or log all errors. Logging forces verbose mode.")
                .value_name("str")
                .possible_values(&["panic", "log"])
                .default_value("panic"),
        )
        .arg(
            Arg::with_name("single-read-validation-level")
                .long("single-read-validation-level")
                .help("Only use single read validators up to a given level")
                .value_name("str")
                .possible_values(&["low", "medium", "high"])
                .default_value("high"),
        )
        .arg(
            Arg::with_name("paired-read-validation-level")
                .long("paired-read-validation-level")
                .help("Only use paired read validators up to a given level")
                .value_name("str")
                .possible_values(&["low", "medium", "high"])
                .default_value("high"),
        )
        .arg(
            Arg::with_name("disable-validator")
                .long("disable-validator")
                .help("Disable validators by code. Use multiple times to disable more than one.")
                .value_name("str")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("r1-src")
                .help("Read 1 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("r2-src")
                .help("Read 2 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(2),
        );

    let subsample_cmd = SubCommand::with_name("subsample")
        .about("Outputs a proportional subset of reads")
        .arg(
            Arg::with_name("probability")
                .short("p")
                .long("probability")
                .value_name("f64")
                .required(true),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("u64")
                .help("Seed to use for the random number generator"),
        )
        .arg(
            Arg::with_name("r1-dst")
                .help("Read 1 destination. Output will be gzipped if ends in `.gz`.")
                .long("r1-dst")
                .value_name("path")
                .required(true),
        )
        .arg(
            Arg::with_name("r2-dst")
                .help("Read 2 destination. Output will be gzipped if ends in `.gz`.")
                .long("r2-dst")
                .value_name("path"),
        )
        .arg(
            Arg::with_name("r1-src")
                .help("Read 1 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("r2-src")
                .help("Read 2 source. Accepts both raw and gzipped FASTQ inputs.")
                .index(2),
        );

    let matches = App::new("fq")
        .version(render_testament!(TESTAMENT).as_str())
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Use verbose logging"),
        )
        .subcommand(filter_cmd)
        .subcommand(generate_cmd)
        .subcommand(lint_cmd)
        .subcommand(subsample_cmd)
        .get_matches();

    if matches.is_present("verbose") {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    } else {
        tracing_subscriber::fmt::init();
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
