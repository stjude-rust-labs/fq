#[macro_use] extern crate clap;
extern crate fqlib;

use clap::{App, Arg};
use fqlib::PairedFastQReader;
use fqlib::validators::{self, BlockValidator, LintMode, ValidationLevel};

fn report_error(error: validators::Error) {
    println!("{:?}", error);
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
        .arg(Arg::with_name("r1-input-pathname")
             .help("")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-input-pathname")
             .help("")
             .index(2)
             .required(true))
        .get_matches();

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

    let reader = PairedFastQReader::open(
        r1_input_pathname,
        r2_input_pathname,
    ).unwrap();

    let validator = BlockValidator::new(
        single_read_validation_level,
        paired_read_validation_level,
        &disabled_validators,
    );

    for (r1_block, r2_block) in reader {
        let b = r1_block.unwrap();
        let d = r2_block.unwrap();

        if let Err(e) = validator.validate_pair(&b, &d) {
            match lint_mode {
                LintMode::Error => panic!("{:?}", e),
                LintMode::Report => report_error(e),
            }
        }
    }
}
