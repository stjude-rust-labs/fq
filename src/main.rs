use clap::{crate_name, App, AppSettings, Arg, SubCommand};
use fqlib::commands::{filter, generate, lint};
use log::LevelFilter;

use git_testament::{git_testament, render_testament};

git_testament!(TESTAMENT);

fn main() {
    let filter_cmd = SubCommand::with_name("filter")
        .about("Filters a FASTQ from a whitelist of names")
        .arg(
            Arg::with_name("names")
                .long("names")
                .value_name("PATH")
                .help("Whitelist of record names")
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
                .long("seed")
                .value_name("N")
                .help("Seed to use for the random number generator"),
        )
        .arg(
            Arg::with_name("n-records")
                .short("n")
                .long("n-records")
                .help("Number of records to generate")
                .value_name("N")
                .default_value("10000"),
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
                .value_name("MODE")
                .possible_values(&["panic", "log"])
                .default_value("panic"),
        )
        .arg(
            Arg::with_name("single-read-validation-level")
                .long("single-read-validation-level")
                .help("Only use single read validators up to a given level")
                .value_name("LEVEL")
                .possible_values(&["low", "medium", "high"])
                .default_value("high"),
        )
        .arg(
            Arg::with_name("paired-read-validation-level")
                .long("paired-read-validation-level")
                .help("Only use paired read validators up to a given level")
                .value_name("LEVEL")
                .possible_values(&["low", "medium", "high"])
                .default_value("high"),
        )
        .arg(
            Arg::with_name("disable-validator")
                .long("disable-validator")
                .help("Disable validators by code. Use multiple times to disable more than one.")
                .value_name("CODE")
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

    let matches = App::new("fq")
        .version(render_testament!(TESTAMENT).as_str())
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
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }

    if let Some(m) = matches.subcommand_matches("filter") {
        filter(m);
    } else if let Some(m) = matches.subcommand_matches("generate") {
        generate(m);
    } else if let Some(m) = matches.subcommand_matches("lint") {
        lint(m);
    }
}
