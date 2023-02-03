use std::{
    io::{self, BufRead},
    path::Path,
    process,
};

use anyhow::Context;
use tracing::{error, info};

use crate::{
    cli::LintArgs,
    fastq::{self, Record},
    validators::{
        self, single::DuplicateNameValidator, LintMode, SingleReadValidatorMut, ValidationLevel,
    },
};

fn build_error_message<P>(error: validators::Error, pathname: P, record_counter: usize) -> String
where
    P: AsRef<Path>,
{
    use std::fmt::Write;

    let path = pathname.as_ref();

    let mut message = String::new();

    let line_offset = error.line_type as usize;
    let line_no = record_counter * 4 + line_offset + 1;
    let _ = write!(message, "{}:{}:", path.display(), line_no);

    if let Some(col_no) = error.col_no {
        let _ = write!(message, "{col_no}:");
    }

    let _ = write!(
        message,
        " [{}] {}: {}",
        error.code, error.name, error.message
    );

    message
}

fn exit_with_validation_error<P>(error: validators::Error, pathname: P, record_counter: usize) -> !
where
    P: AsRef<Path>,
{
    let message = build_error_message(error, pathname, record_counter);
    eprintln!("{message}");
    process::exit(1);
}

fn log_validation_error<P>(error: validators::Error, pathname: P, record_counter: usize)
where
    P: AsRef<Path>,
{
    let message = build_error_message(error, pathname, record_counter);
    error!("{}", message);
}

fn handle_validation_error<P>(
    lint_mode: LintMode,
    error: validators::Error,
    pathname: P,
    record_counter: usize,
) where
    P: AsRef<Path>,
{
    match lint_mode {
        LintMode::Panic => exit_with_validation_error(error, pathname, record_counter),
        LintMode::Log => log_validation_error(error, pathname, record_counter),
    }
}

fn validate_single(
    mut reader: fastq::Reader<impl BufRead>,
    single_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_src: &Path,
) -> anyhow::Result<()> {
    let (single_read_validators, _) =
        validators::filter_validators(single_read_validation_level, None, disabled_validators);

    info!("starting validation");

    let mut record = Record::default();
    let mut record_counter = 0;

    loop {
        let bytes_read = reader
            .read_record(&mut record)
            .with_context(|| format!("Could not read record from file: {}", r1_src.display()))?;

        if bytes_read == 0 {
            break;
        }

        record.reset();

        for validator in &single_read_validators {
            validator
                .validate(&record)
                .unwrap_or_else(|e| handle_validation_error(lint_mode, e, r1_src, record_counter));
        }

        record_counter += 1;
    }

    info!("read {} records", record_counter);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_pair(
    mut reader_1: fastq::Reader<impl BufRead>,
    mut reader_2: fastq::Reader<impl BufRead>,
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_src: &Path,
    r2_src: &Path,
) -> anyhow::Result<()> {
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
        format!(r#""[{code}] {name}""#)
    } else {
        String::new()
    };

    info!("enabled special validators: [{}]", validators);

    info!("starting validation (pass 1)");

    let mut b = Record::default();
    let mut d = Record::default();
    let mut record_counter = 0;

    loop {
        let r1_len = reader_1
            .read_record(&mut b)
            .with_context(|| format!("Could not read record from file: {}", r1_src.display()))?;

        let r2_len = reader_2
            .read_record(&mut d)
            .with_context(|| format!("Could not read record from file: {}", r2_src.display()))?;

        if r1_len == 0 && r2_len > 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof)).with_context(|| {
                format!(
                    "{} unexpectedly ended before {}",
                    r1_src.display(),
                    r2_src.display()
                )
            });
        } else if r2_len == 0 && r1_len > 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof)).with_context(|| {
                format!(
                    "{} unexpectedly ended before {}",
                    r2_src.display(),
                    r1_src.display()
                )
            });
        } else if r1_len == 0 && r2_len == 0 {
            break;
        }

        b.reset();
        d.reset();

        if use_special_validator {
            duplicate_name_validator.insert(&b);
        }

        for validator in &single_read_validators {
            validator
                .validate(&b)
                .unwrap_or_else(|e| handle_validation_error(lint_mode, e, r1_src, record_counter));

            validator
                .validate(&d)
                .unwrap_or_else(|e| handle_validation_error(lint_mode, e, r2_src, record_counter));
        }

        for validator in &paired_read_validators {
            validator
                .validate(&b, &d)
                .unwrap_or_else(|e| handle_validation_error(lint_mode, e, r1_src, record_counter));
        }

        record_counter += 1;
    }

    info!("read {} * 2 records", record_counter);
    info!("starting validation (pass 2)");

    if !use_special_validator {
        return Ok(());
    }

    let mut reader = crate::fastq::open(r1_src)
        .with_context(|| format!("Could not open file: {}", r1_src.display()))?;

    let mut record = Record::default();
    let mut record_counter = 0;

    loop {
        let bytes_read = reader
            .read_record(&mut record)
            .with_context(|| format!("Could not read record from file: {}", r1_src.display()))?;

        if bytes_read == 0 {
            break;
        }

        record.reset();

        duplicate_name_validator
            .validate(&record)
            .unwrap_or_else(|e| handle_validation_error(lint_mode, e, r1_src, record_counter));

        record_counter += 1;
    }

    info!("read {} records", record_counter);

    Ok(())
}

pub fn lint(args: LintArgs) -> anyhow::Result<()> {
    let lint_mode = args.lint_mode;

    let r1_src = &args.r1_src;
    let r2_src = args.r2_src.as_ref();

    let single_read_validation_level = args.single_read_validation_level;
    let paired_read_validation_level = args.paired_read_validation_level;

    let disabled_validators = &args.disable_validator;

    info!("fq-lint start");

    let r1 = crate::fastq::open(r1_src)
        .with_context(|| format!("Could not open file: {}", r1_src.display()))?;

    if let Some(r2_src) = r2_src {
        info!("validating paired end reads");

        let r2 = crate::fastq::open(r2_src)
            .with_context(|| format!("Could not open file: {}", r2_src.display()))?;

        validate_pair(
            r1,
            r2,
            single_read_validation_level,
            paired_read_validation_level,
            disabled_validators,
            lint_mode,
            r1_src,
            r2_src,
        )?;
    } else {
        info!("validating single end read");

        validate_single(
            r1,
            single_read_validation_level,
            disabled_validators,
            lint_mode,
            r1_src,
        )?;
    }

    info!("fq-lint end");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::validators::{self, LineType};

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
            "in.fastq:10:76: [S002] AlphabetValidator: Invalid character: m",
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
            "in.fastq:10: [S002] AlphabetValidator: Invalid character: m",
        );
    }
}
