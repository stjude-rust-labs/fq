use std::str::FromStr;

use Block;

pub use self::single::{SingleReadValidator, SingleReadValidatorMut};
pub use self::paired::PairedReadValidator;
use self::single::{
    AlphabetValidator,
    CompleteValidator,
    ConsistentSeqQualValidator,
    DuplicateNameValidator,
    NameValidator,
    PlusLineValidator,
    QualityStringValidator,
};
use self::paired::NamesValidator;

pub mod single;
pub mod paired;

#[derive(Debug)]
pub enum Error {
    Invalid(String),
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

type SingleReadValidatorMutPair = (Box<SingleReadValidatorMut>, Box<SingleReadValidatorMut>);

pub struct BlockValidator {
    single_read_validators: Vec<Box<SingleReadValidator>>,
    single_read_validators_mut: Vec<SingleReadValidatorMutPair>,
    paired_read_validators: Vec<Box<PairedReadValidator>>,
}

impl BlockValidator {
    pub fn new(
        single_read_validation_level: ValidationLevel,
        paired_read_validation_level: ValidationLevel,
        disabled_validators: &[String],
    ) -> BlockValidator {
        let single_read_validators = filter_single_read_validators(
            single_read_validation_level,
            disabled_validators,
        );

        let single_read_validators_mut = filter_single_read_validators_mut(
            single_read_validation_level,
            disabled_validators,
        );

        let paired_read_validators = filter_paired_read_validators(
            paired_read_validation_level,
            disabled_validators,
        );

        BlockValidator {
            single_read_validators,
            single_read_validators_mut,
            paired_read_validators,
        }
    }

    pub fn validate(&self, b: &Block) -> Result<(), Error> {
        for validator in &self.single_read_validators {
            validator.validate(&b)?;
        }

        Ok(())
    }

    pub fn validate_mut_pair(&mut self, b: &Block, d: &Block) -> Result<(), Error> {
        for (b_validator, d_validator) in &mut self.single_read_validators_mut {
            b_validator.validate(&b)?;
            d_validator.validate(&d)?;
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

    pub fn validate_pair_mut(&mut self, b: &Block, d: &Block) -> Result<(), Error> {
        self.validate_pair(b, d)?;
        self.validate_mut_pair(b, d)?;
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

fn filter_single_read_validators_mut(
    validation_level: ValidationLevel,
    disabled_validators: &[String],
) -> Vec<SingleReadValidatorMutPair> {
    let pairs: Vec<SingleReadValidatorMutPair> = vec![
        (Box::new(DuplicateNameValidator::new()), Box::new(DuplicateNameValidator::new())),
    ];

    pairs
        .into_iter()
        .filter(|p| p.0.level() <= validation_level)
        .filter(|p| {
            let code = p.0.code().to_string();
            !disabled_validators.contains(&code)
        })
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

    #[test]
    fn test_filter_single_read_validators_mut() {
        let disabled_validators = Vec::new();

        let validators = filter_single_read_validators_mut(
            ValidationLevel::Minimum,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 0);

        let validators = filter_single_read_validators_mut(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 1);
    }

    #[test]
    fn test_filter_single_read_validators_mut_with_disabled_validators() {
        let disabled_validators = vec![String::from("S007")];

        let validators = filter_single_read_validators_mut(
            ValidationLevel::High,
            &disabled_validators,
        );

        assert_eq!(validators.len(), 0);
        assert!(validators.iter().find(|p| p.0.code() == "S007").is_none());
    }
}
