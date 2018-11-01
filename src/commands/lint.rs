use std::io;
use std::process;

use clap::ArgMatches;

use ::{FastQReader, PairedReader, readers};
use validators::single::DuplicateNameValidator;
use validators::{self, LintMode, SingleReadValidatorMut, ValidationLevel};

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

    message
}

fn exit_with_validation_error(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) -> ! {
    let message = build_error_message(error, pathname, block_no);
    eprintln!("{}", message);
    process::exit(1);
}

fn log_validation_error(
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) {
    let message = build_error_message(error, pathname, block_no);
    error!("{}", message);
}

fn handle_validation_error(
    lint_mode: LintMode,
    error: validators::Error,
    pathname: &str,
    block_no: usize,
) {
    match lint_mode {
        LintMode::Panic => exit_with_validation_error(error, pathname, block_no + 1),
        LintMode::Log => log_validation_error(error, pathname, block_no + 1),
    }
}

fn exit_with_io_error(error: io::Error, pathname: Option<&str>) -> ! {
    match pathname {
        Some(p) => eprintln!("{}: {}", error, p),
        None => eprintln!("{}", error),
    }

    process::exit(1);
}

fn validate_single(
    mut reader: impl FastQReader,
    single_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_input_pathname: &str,
) {
    let (single_read_validators, _) = validators::filter_validators(
        single_read_validation_level,
        None,
        disabled_validators,
    );

    info!("starting validation");

    let mut block_no = 0;

    while let Some(block) = reader.next_block() {
        let b = match block {
            Ok(b) => b,
            Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
        };

        for validator in &single_read_validators {
            if let Err(e) = validator.validate(b) {
                handle_validation_error(lint_mode, e, r1_input_pathname, block_no + 1);
            }
        }

        block_no += 1;
    }

    info!("read {} blocks", block_no + 1);
}

fn validate_pair<R: FastQReader, S: FastQReader>(
    mut reader: PairedReader<R, S>,
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_input_pathname: &str,
    r2_input_pathname: &str,
) {
    let (single_read_validators, paired_read_validators) = validators::filter_validators(
        single_read_validation_level,
        Some(paired_read_validation_level),
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
            Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
        };

        let d = match block_2 {
            Ok(b) => b,
            Err(e) => exit_with_io_error(e, Some(r2_input_pathname)),
        };

        if use_special_validator {
            duplicate_name_validator.insert(b);
        }

        for validator in &single_read_validators {
            if let Err(e) = validator.validate(b) {
                handle_validation_error(lint_mode, e, r1_input_pathname, block_no + 1);
            }

            if let Err(e) = validator.validate(d) {
                handle_validation_error(lint_mode, e, r2_input_pathname, block_no + 1);
            }
        }

        for validator in &paired_read_validators {
            if let Err(e) = validator.validate(b, d) {
                handle_validation_error(lint_mode, e, r1_input_pathname, block_no + 1);
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
            handle_validation_error(lint_mode, e, r1_input_pathname, block_no + 1);
        }

        block_no += 1;
    }

    info!("read {} * 2 blocks", block_no + 1);
}

pub fn lint(matches: &ArgMatches) {
    let lint_mode = value_t!(matches, "lint-mode", LintMode).unwrap_or_else(|e| e.exit());

    let r1_input_pathname = matches.value_of("in1").unwrap();
    let r2_input_pathname = matches.value_of("in2");

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

    info!("fq-lint start");

    let r1 = match readers::factory(r1_input_pathname) {
        Ok(r) => r,
        Err(e) => exit_with_io_error(e, Some(r1_input_pathname)),
    };

    if let Some(r2_input_pathname) = r2_input_pathname {
        info!("validating paired end reads");

        let r2 = match readers::factory(r2_input_pathname) {
            Ok(r) => r,
            Err(e) => exit_with_io_error(e, Some(r2_input_pathname)),
        };

        let reader = PairedReader::new(r1, r2);

        validate_pair(
            reader,
            single_read_validation_level,
            paired_read_validation_level,
            &disabled_validators,
            lint_mode,
            r1_input_pathname,
            r2_input_pathname,
        );
    } else {
        info!("validating single end read");

        validate_single(
            r1,
            single_read_validation_level,
            &disabled_validators,
            lint_mode,
            r1_input_pathname,
        );
    }

    info!("fq-lint end");
}

#[cfg(test)]
mod tests {
    use validators::{self, LineType};

    use super::*;

    #[test]
    fn test_build_error_message() {
        let error = validators::Error::new(
            "S002",
            "AlphabetValidator",
            "Invalid character: m",
            LineType::Sequence,
            Some(76),
        );

        assert_eq!(
            build_error_message(error, "in.fastq", 2),
            "in.fastq:6:76: [S002] AlphabetValidator: Invalid character: m",
        );
    }

    #[test]
    fn test_build_error_message_with_no_col_no() {
        let error = validators::Error::new(
            "S002",
            "AlphabetValidator",
            "Invalid character: m",
            LineType::Sequence,
            None,
        );

        assert_eq!(
            build_error_message(error, "in.fastq", 2),
            "in.fastq:6: [S002] AlphabetValidator: Invalid character: m",
        );
    }
}
