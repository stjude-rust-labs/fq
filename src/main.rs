extern crate log;
extern crate env_logger;
#[macro_use] extern crate clap;
extern crate fqlib;

use clap::{App, AppSettings, Arg, SubCommand};
use fqlib::commands::{generate, lint};
use log::LevelFilter;

fn main() {
    let generate_cmd = SubCommand::with_name("generate")
        .about("Generates a random FASTQ file pair")
        .arg(Arg::with_name("num-blocks")
             .short("n")
             .long("num-blocks")
             .help("Number of blocks to generate")
             .value_name("N")
             .default_value("10000"))
        .arg(Arg::with_name("out1")
             .help("Read 1 output pathname. Output will be gzipped if ends in `.gz`.")
             .index(1)
             .required(true))
        .arg(Arg::with_name("out2")
             .help("Read 2 output pathname. Output will be gzipped if ends in `.gz`.")
             .index(2)
             .required(true));

    let lint_cmd = SubCommand::with_name("lint")
        .about("Validates a FASTQ file pair")
        .arg(Arg::with_name("lint-mode")
             .long("lint-mode")
             .help("Panic on first error or log all errors. Logging forces verbose mode.")
             .value_name("MODE")
             .possible_values(&["panic", "log"])
             .default_value("panic"))
        .arg(Arg::with_name("single-read-validation-level")
             .long("single-read-validation-level")
             .help("Only use single read validators up to a given level")
             .value_name("LEVEL")
             .possible_values(&["low", "medium", "high"])
             .default_value("high"))
        .arg(Arg::with_name("paired-read-validation-level")
             .long("paired-read-validation-level")
             .help("Only use paired read validators up to a given level")
             .value_name("LEVEL")
             .possible_values(&["low", "medium", "high"])
             .default_value("high"))
        .arg(Arg::with_name("disable-validator")
             .long("disable-validator")
             .help("Disable validators by code. Use multiple times to disable more than one.")
             .value_name("CODE")
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("in1")
             .help("Read 1 input pathname. Accepts both raw and gzipped FASTQ inputs.")
             .index(1)
             .required(true))
        .arg(Arg::with_name("in2")
             .help("Read 2 input pathname. Accepts both raw and gzipped FASTQ inputs.")
             .index(2));

    let matches = App::new("fq")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .subcommand(generate_cmd)
        .subcommand(lint_cmd)
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    }

    if let Some(m) = matches.subcommand_matches("generate") {
        generate(m);
    } else if let Some(m) = matches.subcommand_matches("lint") {
        lint(m);
    }
}
