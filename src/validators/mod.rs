use std::str::FromStr;

use Block;

pub use self::single::{SingleReadValidator, SingleReadValidatorMut};
pub use self::paired::PairedReadValidator;
use self::single::{
    AlphabetValidator,
    CompleteValidator,
    ConsistentSeqQualValidator,
    NameValidator,
    PlusLineValidator,
    QualityStringValidator,
};
use self::paired::NamesValidator;

pub mod single;
pub mod paired;

#[derive(Debug)]
pub enum LineType {
    Name,
    Sequence,
    PlusLine,
    Quality,
}

/// The error type for validation failures.
#[derive(Debug)]
pub struct Error {
    pub code: String,
    pub name: String,
    pub message: String,
    pub line_type: LineType,
    pub col_no: Option<usize>,
}

impl Error {
    pub fn new(
        code: &str,
        name: &str,
        message: &str,
        line_type: LineType,
        col_no: Option<usize>,
    ) -> Error {
        Error {
            code: code.into(),
            name: name.into(),
            message: message.into(),
            line_type,
            col_no,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum LintMode {
    Report,
    Error,
}

impl FromStr for LintMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "report" => Ok(LintMode::Report),
            "error" => Ok(LintMode::Error),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationLevel {
    Minimum,
    Low,
    High,
}

impl FromStr for ValidationLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minimum" => Ok(ValidationLevel::Minimum),
            "low" => Ok(ValidationLevel::Low),
            "high" => Ok(ValidationLevel::High),
            _ => Err(()),
        }
    }
}

/// Validator that runs immutable validators over blocks.
pub struct BlockValidator {
    single_read_validators: Vec<Box<SingleReadValidator>>,
    paired_read_validators: Vec<Box<PairedReadValidator>>,
}

impl BlockValidator {
    pub fn new(
        single_read_validation_level: ValidationLevel,
        paired_read_validation_level: ValidationLevel,
        disabled_validators: &[String],
    ) -> BlockValidator {
        info!("disabled validators: {:?}", disabled_validators);

        let single_read_validators = filter_single_read_validators(
            single_read_validation_level,
            disabled_validators,
        );

        let validators: Vec<String> = single_read_validators
            .iter()
            .map(|v| format!("[{}] {}", v.code(), v.name()))
            .collect();
        info!("enabled single read validators: {:?}", validators);

        let paired_read_validators = filter_paired_read_validators(
            paired_read_validation_level,
            disabled_validators,
        );

        let validators: Vec<String> = paired_read_validators
            .iter()
            .map(|v| format!("[{}] {}", v.code(), v.name()))
            .collect();
        info!("enabled paired read validators: {:?}", validators);

        BlockValidator {
            single_read_validators,
            paired_read_validators,
        }
    }

    pub fn validate(&self, b: &Block) -> Result<(), Error> {
        for validator in &self.single_read_validators {
            validator.validate(&b)?;
        }

        Ok(())
    }

    pub fn validate_pair(&self, b: &Block, d: &Block) -> Result<(), Error> {
        self.validate(b)?;
        self.validate(d)?;

        for validator in &self.paired_read_validators {
            validator.validate(&b, &d)?;
        }

        Ok(())
    }
}

fn filter_single_read_validators(
    validation_level: ValidationLevel,
    disabled_validators: &[String],
) -> Vec<Box<SingleReadValidator>> {
    let single_read_validators: Vec<Box<SingleReadValidator>> = vec![
        Box::new(NameValidator),
        Box::new(CompleteValidator),
        Box::new(AlphabetValidator::default()),
        Box::new(PlusLineValidator),
        Box::new(ConsistentSeqQualValidator),
        Box::new(QualityStringValidator),
    ];

    single_read_validators
        .into_iter()
        .filter(|v| v.level() <= validation_level)
        .filter(|v| !disabled_validators.contains(&v.code().to_string()))
        .collect()
}

fn filter_paired_read_validators(
    validation_level: ValidationLevel,
    disabled_validators: &[String],
) -> Vec<Box<PairedReadValidator>> {
    let paired_read_validators: Vec<Box<PairedReadValidator>> = vec![
        Box::new(NamesValidator),
    ];

    paired_read_validators
        .into_iter()
        .filter(|v| v.level() <= validation_level)
        .filter(|v| !disabled_validators.contains(&v.code().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_single_read_validators() {
        let disabled_validators = Vec::new();

        let validators = filter_single_read_validators(
            ValidationLevel::Minimum,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].name(), "CompleteValidator");
        assert_eq!(validators[1].name(), "PlusLineValidator");

        let validators = filter_single_read_validators(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 6);
    }

    #[test]
    fn test_filter_single_read_validators_with_disabled_validators() {
        let disabled_validators = vec![String::from("S001")];

        let validators = filter_single_read_validators(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 5);
        assert!(validators.iter().find(|v| v.code() == "S001").is_none());
    }

    #[test]
    fn test_filter_paired_read_validators() {
        let disabled_validators = Vec::new();

        let validators = filter_paired_read_validators(
            ValidationLevel::Minimum,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 0);

        let validators = filter_paired_read_validators(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].name(), "NamesValidator");
    }

    #[test]
    fn test_filter_paired_read_validators_with_disabled_validators() {
        let disabled_validators = vec![String::from("P001")];

        let validators = filter_paired_read_validators(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 0);
        assert!(validators.iter().find(|v| v.code() == "P001").is_none());
    }
}
