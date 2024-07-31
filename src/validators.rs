pub mod paired;
pub mod single;
pub mod validation_level;

pub use self::{
    paired::{NamesValidator, PairedReadValidator},
    single::{
        AlphabetValidator, CompleteValidator, ConsistentSeqQualValidator, NameValidator,
        PlusLineValidator, QualityStringValidator, SingleReadValidator, SingleReadValidatorMut,
    },
    validation_level::ValidationLevel,
};

use std::{error, fmt};

use tracing::info;

pub type SingleAndPairedValidators = (
    Vec<Box<dyn SingleReadValidator>>,
    Vec<Box<dyn PairedReadValidator>>,
);

#[derive(Clone, Copy, Debug)]
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
    pub error: Box<dyn std::error::Error + Send + Sync>,
    pub line_type: LineType,
    pub col_no: Option<usize>,
}

impl Error {
    pub fn new<E>(
        code: &str,
        name: &str,
        error: E,
        line_type: LineType,
        col_no: Option<usize>,
    ) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            code: code.into(),
            name: name.into(),
            error: error.into(),
            line_type,
            col_no,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.name, self.error)
    }
}

impl error::Error for Error {}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintMode {
    Panic,
    Log,
}

pub fn filter_validators(
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: Option<ValidationLevel>,
    disabled_validators: &[String],
) -> SingleAndPairedValidators {
    info!("disabled validators: {:?}", disabled_validators);

    let single_read_validators =
        filter_single_read_validators(single_read_validation_level, disabled_validators);

    let validators: Vec<String> = single_read_validators
        .iter()
        .map(|v| format!("[{}] {}", v.code(), v.name()))
        .collect();

    info!("enabled single read validators: {:?}", validators);

    let paired_read_validators = paired_read_validation_level
        .map(|level| filter_paired_read_validators(level, disabled_validators))
        .unwrap_or_default();

    let validators: Vec<String> = paired_read_validators
        .iter()
        .map(|v| format!("[{}] {}", v.code(), v.name()))
        .collect();

    info!("enabled paired read validators: {:?}", validators);

    (single_read_validators, paired_read_validators)
}

fn filter_single_read_validators(
    validation_level: ValidationLevel,
    disabled_validators: &[String],
) -> Vec<Box<dyn SingleReadValidator>> {
    let single_read_validators: Vec<Box<dyn SingleReadValidator>> = vec![
        Box::new(NameValidator),
        Box::new(CompleteValidator),
        Box::<AlphabetValidator>::default(),
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
) -> Vec<Box<dyn PairedReadValidator>> {
    let paired_read_validators: Vec<Box<dyn PairedReadValidator>> = vec![Box::new(NamesValidator)];

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
    fn test_filter_validators() {
        let (single_read_validators, paired_read_validators) =
            filter_validators(ValidationLevel::High, None, &[]);

        assert_eq!(single_read_validators.len(), 6);
        assert_eq!(paired_read_validators.len(), 0);

        let (single_read_validators, paired_read_validators) =
            filter_validators(ValidationLevel::High, Some(ValidationLevel::High), &[]);

        assert_eq!(single_read_validators.len(), 6);
        assert_eq!(paired_read_validators.len(), 1);
    }

    #[test]
    fn test_filter_single_read_validators() {
        let disabled_validators = Vec::new();

        let validators = filter_single_read_validators(ValidationLevel::Low, &disabled_validators);

        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].name(), "CompleteValidator");
        assert_eq!(validators[1].name(), "PlusLineValidator");

        let validators = filter_single_read_validators(ValidationLevel::High, &disabled_validators);

        assert_eq!(validators.len(), 6);
    }

    #[test]
    fn test_filter_single_read_validators_with_disabled_validators() {
        let disabled_validators = vec![String::from("S001")];

        let validators = filter_single_read_validators(ValidationLevel::High, &disabled_validators);

        assert_eq!(validators.len(), 5);
        assert!(!validators.iter().any(|v| v.code() == "S001"));
    }

    #[test]
    fn test_filter_paired_read_validators() {
        let disabled_validators = Vec::new();

        let validators = filter_paired_read_validators(ValidationLevel::Low, &disabled_validators);

        assert_eq!(validators.len(), 0);

        let validators = filter_paired_read_validators(ValidationLevel::High, &disabled_validators);

        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].name(), "NamesValidator");
    }

    #[test]
    fn test_filter_paired_read_validators_with_disabled_validators() {
        let disabled_validators = vec![String::from("P001")];

        let validators = filter_paired_read_validators(ValidationLevel::High, &disabled_validators);

        assert_eq!(validators.len(), 0);
        assert!(!validators.iter().any(|v| v.code() == "P001"));
    }
}
