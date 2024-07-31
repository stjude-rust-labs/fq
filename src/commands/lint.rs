use std::{
    fmt,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process,
};

use thiserror::Error;
use tracing::{error, info};

use crate::{
    cli::LintArgs,
    fastq::{self, Record},
    validators::{
        self, single::DuplicateNameValidator, LintMode, SingleReadValidatorMut, ValidationLevel,
    },
};

fn exit_with_validation_error<P>(error: validators::Error, pathname: P, record_counter: usize) -> !
where
    P: AsRef<Path>,
{
    let err = ValidationError::new(error, pathname, record_counter);
    eprintln!("{err}");
    process::exit(1);
}

fn log_validation_error<P>(error: validators::Error, pathname: P, record_counter: usize)
where
    P: AsRef<Path>,
{
    let err = ValidationError::new(error, pathname, record_counter);
    error!("{err}");
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
    record_definition_separator: Option<u8>,
    single_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_src: &Path,
) -> Result<bool, LintError> {
    let (single_read_validators, _) =
        validators::filter_validators(single_read_validation_level, None, disabled_validators);

    info!("starting validation");

    let mut record = Record::default();
    let mut record_counter = 0;
    let mut did_fail_validation = false;

    while reader.read_record(&mut record)? != 0 {
        record.reset(record_definition_separator);

        for validator in &single_read_validators {
            validator.validate(&record).unwrap_or_else(|e| {
                did_fail_validation = true;
                handle_validation_error(lint_mode, e, r1_src, record_counter);
            });
        }

        record_counter += 1;
    }

    info!("read {} records", record_counter);

    Ok(did_fail_validation)
}

#[allow(clippy::too_many_arguments)]
fn validate_pair(
    mut reader_1: fastq::Reader<impl BufRead>,
    mut reader_2: fastq::Reader<impl BufRead>,
    record_definition_separator: Option<u8>,
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: ValidationLevel,
    disabled_validators: &[String],
    lint_mode: LintMode,
    r1_src: &Path,
    r2_src: &Path,
) -> Result<bool, LintError> {
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
    let mut did_fail_validation = false;

    loop {
        let r1_len = reader_1.read_record(&mut b)?;
        let r2_len = reader_2.read_record(&mut d)?;

        match (r1_len, r2_len) {
            (0, 0) => break,
            (0, len) if len > 0 => return Err(LintError::UnexpectedEof("r1-src")),
            (len, 0) if len > 0 => return Err(LintError::UnexpectedEof("r2-src")),
            (_, _) => {}
        }

        b.reset(record_definition_separator);
        d.reset(record_definition_separator);

        if use_special_validator {
            duplicate_name_validator.insert(&b);
        }

        for validator in &single_read_validators {
            validator.validate(&b).unwrap_or_else(|e| {
                did_fail_validation = true;
                handle_validation_error(lint_mode, e, r1_src, record_counter);
            });

            validator.validate(&d).unwrap_or_else(|e| {
                did_fail_validation = true;
                handle_validation_error(lint_mode, e, r2_src, record_counter);
            });
        }

        for validator in &paired_read_validators {
            validator.validate(&b, &d).unwrap_or_else(|e| {
                did_fail_validation = true;
                handle_validation_error(lint_mode, e, r1_src, record_counter);
            });
        }

        record_counter += 1;
    }

    info!("read {} * 2 records", record_counter);
    info!("starting validation (pass 2)");

    if !use_special_validator {
        return Ok(did_fail_validation);
    }

    let mut reader =
        crate::fastq::open(r1_src).map_err(|e| LintError::OpenFile(e, r1_src.into()))?;

    let mut record = Record::default();
    let mut record_counter = 0;

    while reader.read_record(&mut record)? != 0 {
        record.reset(record_definition_separator);

        duplicate_name_validator
            .validate(&record)
            .unwrap_or_else(|e| {
                did_fail_validation = true;
                handle_validation_error(lint_mode, e, r1_src, record_counter);
            });

        record_counter += 1;
    }

    info!("read {} records", record_counter);

    Ok(did_fail_validation)
}

pub fn lint(args: LintArgs) -> Result<(), LintError> {
    let lint_mode = args.lint_mode;

    let r1_src = &args.r1_src;
    let r2_src = args.r2_src.as_ref();

    let single_read_validation_level = args.single_read_validation_level;
    let paired_read_validation_level = args.paired_read_validation_level;

    let disabled_validators = &args.disable_validator;

    let record_definition_separator = args.record_definition_separator.map(u8::from);

    info!("fq-lint start");

    let r1 = crate::fastq::open(r1_src).map_err(|e| LintError::OpenFile(e, r1_src.into()))?;

    let did_fail_validation = if let Some(r2_src) = r2_src {
        info!("validating paired end reads");

        let r2 = crate::fastq::open(r2_src).map_err(|e| LintError::OpenFile(e, r2_src.into()))?;

        validate_pair(
            r1,
            r2,
            record_definition_separator,
            single_read_validation_level,
            paired_read_validation_level,
            disabled_validators,
            lint_mode,
            r1_src,
            r2_src,
        )?
    } else {
        info!("validating single end read");

        validate_single(
            r1,
            record_definition_separator,
            single_read_validation_level,
            disabled_validators,
            lint_mode,
            r1_src,
        )?
    };

    info!("fq-lint end");

    if did_fail_validation {
        process::exit(1);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum LintError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("could not open file: {1}")]
    OpenFile(#[source] io::Error, PathBuf),
    #[error("could not create file: {1}")]
    CreateFile(#[source] io::Error, PathBuf),
    #[error("{0} unexpectedly ended")]
    UnexpectedEof(&'static str),
    #[error("validation error")]
    Validation(ValidationError),
}

#[derive(Debug)]
pub struct ValidationError {
    err: validators::Error,
    src: PathBuf,
    record_counter: usize,
}

impl ValidationError {
    fn new<P>(err: validators::Error, src: P, record_counter: usize) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            err,
            src: src.as_ref().into(),
            record_counter,
        }
    }
}

impl std::error::Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line_offset = self.err.line_type as usize;
        let line_no = self.record_counter * 4 + line_offset + 1;
        write!(f, "{}:{}:", self.src.display(), line_no)?;

        if let Some(col_no) = self.err.col_no {
            write!(f, "{col_no}:")?;
        }

        write!(f, " {}", self.err)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::validators::{self, LineType};

    use super::*;

    #[test]
    fn test_validation_error_fmt() {
        let error = ValidationError::new(
            validators::Error::new(
                "S002",
                "AlphabetValidator",
                "Invalid character: m",
                LineType::Sequence,
                Some(76),
            ),
            "in.fastq",
            2,
        );

        assert_eq!(
            error.to_string(),
            "in.fastq:10:76: [S002] AlphabetValidator: Invalid character: m",
        );
    }

    #[test]
    fn test_validation_error_fmt_with_no_col_no() {
        let error = ValidationError::new(
            validators::Error::new(
                "S002",
                "AlphabetValidator",
                "Invalid character: m",
                LineType::Sequence,
                None,
            ),
            "in.fastq",
            2,
        );

        assert_eq!(
            error.to_string(),
            "in.fastq:10: [S002] AlphabetValidator: Invalid character: m",
        );
    }
}
