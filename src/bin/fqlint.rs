#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate clap;
extern crate fqlib;

use std::io;
use std::process;

use clap::{App, Arg};
use fqlib::{FastQReader, PairedReader, readers};
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
) -> ! {
    let message = build_error_message(error, pathname, block_no);
    eprintln!("{}", message);
    process::exit(1);
}

fn log_error(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) {
    let message = build_error_message(error, pathname, block_no);
    error!("{}", message);
}

fn exit_with_io_error(error: io::Error, pathname: Option<&str>) -> ! {
    match pathname {
        Some(p) => eprintln!("{}: {}", error, p),
        None => eprintln!("{}", error),
    }

    process::exit(1);
}

fn validate<R: FastQReader, S: FastQReader>(
    mut reader: PairedReader<R, S>,
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

    let code = duplicate_name_validator.code();
    let name = duplicate_name_validator.name();
    let use_special_validator = !disabled_validators.contains(&code.to_string());

    let validators = if use_special_validator {
        format!(r#""[{}] {}""#, code, name)
    } else {
        String::new()
    };

    info!("enabled special validators: [{}]", validators);

    info!("starting validation (pass 1)");

    let mut block_no = 0;

    while let Some((block_1, block_2)) = reader.next_pair() {
        let b = match block_1 {
            Ok(b) => b,
            Err(e) => exit_with_io_error(e, None),
        };

        let d = match block_2 {
            Ok(b) => b,
            Err(e) => exit_with_io_error(e, None),
        };

        if use_special_validator {
            duplicate_name_validator.insert(b);
        }

        if let Err(e) = validator.validate_pair(b, d) {
            match lint_mode {
                LintMode::Panic => panic_error(e, "<filename>", block_no + 1),
                LintMode::Log => log_error(e, "<filename>", block_no + 1),
            }
        }

        block_no += 1;
    }

    info!("starting validation (pass 2)");

    if !use_special_validator {
        return;
    }

    let mut reader = match readers::factory(r1_input_pathname) {
        Ok(r) => r,
        Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
    };

    let mut block_no = 0;

    while let Some(block) = reader.next_block() {
        let b = match block {
            Ok(b) => b,
            Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
        };

        if let Err(e) = duplicate_name_validator.validate(&b) {
            match lint_mode {
                LintMode::Panic => panic_error(e, "<filename>", block_no + 1),
                LintMode::Log => log_error(e, "<filename>", block_no + 1),
            }
        }

        block_no += 1;
    }
}

fn main() {
    let matches = App::new("fqlint")
        .version(crate_version!())
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
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Use verbose logging"))
        .arg(Arg::with_name("r1-input-pathname")
             .index(1)
             .required(true))
        .arg(Arg::with_name("r2-input-pathname")
             .index(2)
             .required(true))
        .get_matches();

    let lint_mode = value_t!(matches, "lint-mode", LintMode).unwrap_or_else(|e| e.exit());

    if matches.is_present("verbose") || lint_mode == LintMode::Log {
        env_logger::Builder::from_default_env()
            .filter(Some("fqlint"), LevelFilter::Info)
            .filter(Some(crate_name!()), LevelFilter::Info)
            .init();
    }

    let r1_input_pathname = matches.value_of("r1-input-pathname").unwrap();
    let r2_input_pathname = matches.value_of("r2-input-pathname").unwrap();

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

    let r1 = match readers::factory(r1_input_pathname) {
        Ok(r) => r,
        Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
    };

    let r2 = match readers::factory(r2_input_pathname) {
        Ok(r) => r,
        Err(e) => exit_with_io_error(e, Some(r2_input_pathname)),
    };

    let reader = PairedReader::new(r1, r2);

    validate(
        reader,
        single_read_validation_level,
        paired_read_validation_level,
        &disabled_validators,
        lint_mode,
        r1_input_pathname,
    );

    info!("fqlint end");
}

#[cfg(test)]
mod tests {
    use fqlib::validators::{self, LineType};

    use super::build_error_message;

    #[test]
    fn test_build_error_message() {
        let error = validators::Error::new(
            "S002",
            "AlphabetValidator",
            "Invalid character: m",
            LineType::Sequence,
            None,
        );

        let actual = build_error_message(error, "in.fastq", 2);
        let expected = "in.fastq:6: [S002] AlphabetValidator: Invalid character: m";

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_build_error_message_with_col_no() {
        let error = validators::Error::new(
            "S002",
            "AlphabetValidator",
            "Invalid character: m",
            LineType::Sequence,
            Some(44),
        );

        let actual = build_error_message(error, "in.fastq", 2);
        let expected = "in.fastq:6:44: [S002] AlphabetValidator: Invalid character: m";

        assert_eq!(actual, expected);
    }
}
