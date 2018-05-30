#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate clap;
extern crate fqlib;

use std::process;

use clap::{App, Arg};
use fqlib::{FastQReader, FileReader, GzReader, PairedReader};
use fqlib::validators::single::DuplicateNameValidator;
use fqlib::validators::{
    self,
    BlockValidator,
    LintMode,
    SingleReadValidatorMut,
    ValidationLevel,
};
use log::LevelFilter;

fn build_error_message(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) -> String {
    let mut message = String::new();

    let line_offset = error.line_type as usize;
    let line_no = (block_no - 1) * 4 + line_offset + 1;
    message.push_str(&format!("{}:{}:", pathname, line_no));

    if let Some(col_no) = error.col_no {
        message.push_str(&format!("{}:", col_no));
    }

    message.push_str(&format!(" [{}] {}: {}", error.code, error.name, error.message));

    return message;
}

fn panic_error(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) {
    let message = build_error_message(error, pathname, block_no);
    eprintln!("{}", message);
    process::exit(1);
}

fn report_error(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) {
    let message = build_error_message(error, pathname, block_no);
    error!("{}", message);
}

fn validate<R: FastQReader>(
    mut reader: PairedReader<R>,
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_input_pathname: &str,
) {
    let validator = BlockValidator::new(
        single_read_validation_level,
        paired_read_validation_level,
        disabled_validators,
    );

    let mut duplicate_name_validator = DuplicateNameValidator::new();

    info!("starting validation (pass 1)");

    let mut block_no = 0;

    while let Some((r1_block, r2_block)) = reader.next_pair() {
        let b = r1_block.unwrap();
        let d = r2_block.unwrap();

        duplicate_name_validator.validate(b).unwrap();

        if let Err(e) = validator.validate_pair(b, d) {
            match lint_mode {
                LintMode::Error => panic_error(e, "<filename>", block_no + 1),
                LintMode::Report => report_error(e, "<filename>", block_no + 1),
            }
        }

        block_no += 1;
    }

    info!("starting validation (pass 2)");

    // @TODO check if input is compressed
    let mut reader = FileReader::open(r1_input_pathname).unwrap();

    while let Some(block) = reader.next_block() {
        let b = block.unwrap();

        if !duplicate_name_validator.contains_once(&b.name) {
            panic!("Duplicae name found: {}", b.name);
        }
    }
}

fn main() {
    let matches = App::new("fqlint")
        .version(crate_version!())
        .arg(Arg::with_name("lint-mode")
             .long("lint-mode")
             .value_name("MODE")
             .possible_values(&["error", "report"])
             .default_value("error"))
        .arg(Arg::with_name("single-read-validation-level")
             .long("single-read-validation-level")
             .value_name("LEVEL")
             .possible_values(&["minimum", "low", "high"])
             .default_value("high"))
        .arg(Arg::with_name("paired-read-validation-level")
             .long("paired-read-validation-level")
             .value_name("LEVEL")
             .possible_values(&["minimum", "low", "high"])
             .default_value("high"))
        .arg(Arg::with_name("disable-validator")
             .long("disable-validator")
             .value_name("CODE")
             .multiple(true)
             .number_of_values(1))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .arg(Arg::with_name("r1-input-pathname")
             .help("")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-input-pathname")
             .help("")
             .index(2)
             .required(true))
        .get_matches();

    if matches.is_present("verbose") {
        env_logger::Builder::from_default_env()
            .filter(Some("fqlint"), LevelFilter::Info)
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    }

    let r1_input_pathname = matches.value_of("r1-input-pathname").unwrap();
    let r2_input_pathname = matches.value_of("r2-input-pathname").unwrap();

    let lint_mode = value_t!(matches, "lint-mode", LintMode).unwrap_or_else(|e| e.exit());

    let single_read_validation_level = value_t!(
        matches,
        "single-read-validation-level",
        ValidationLevel
    ).unwrap_or_else(|e| e.exit());

    let paired_read_validation_level = value_t!(
        matches,
        "paired-read-validation-level",
        ValidationLevel
    ).unwrap_or_else(|e| e.exit());

    let disabled_validators: Vec<String> = matches
        .values_of("disable-validator")
        .unwrap_or_default()
        .map(String::from)
        .collect();

    info!("fqlint start");

    let is_compressed = r1_input_pathname.ends_with(".gz");

    if is_compressed {
        info!("inputs are compressed");

        let r1 = GzReader::open(r1_input_pathname).unwrap();
        let r2 = GzReader::open(r2_input_pathname).unwrap();
        let reader = PairedReader::new(r1, r2);

        validate(
            reader,
            single_read_validation_level,
            paired_read_validation_level,
            &disabled_validators,
            lint_mode,
            r1_input_pathname,
        );
    } else {
        let r1 = FileReader::open(r1_input_pathname).unwrap();
        let r2 = FileReader::open(r2_input_pathname).unwrap();
        let reader = PairedReader::new(r1, r2);

        validate(
            reader,
            single_read_validation_level,
            paired_read_validation_level,
            &disabled_validators,
            lint_mode,
            r1_input_pathname,
        );
    }

    info!("fqlint end");
}
