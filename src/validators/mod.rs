use std::str::FromStr;

use Block;

pub use self::single::SingleReadValidator;
pub use self::paired::PairedReadValidator;
use self::single::{
    AlphabetValidator,
    CompleteValidator,
    ConsistentSeqQualValidator,
    NameValidator,
    PlusLineValidator,
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

pub struct BlockValidator {
    single_read_validators: Vec<Box<SingleReadValidator>>,
    paired_read_validators: Vec<Box<PairedReadValidator>>,
}

impl BlockValidator {
    pub fn new(
        single_read_validation_level: ValidationLevel,
        paired_read_validation_level: ValidationLevel,
    ) -> BlockValidator {
        let single_read_validators = filter_single_read_validators(single_read_validation_level);
        let paired_read_validators = filter_paired_read_validators(paired_read_validation_level);
        BlockValidator { single_read_validators, paired_read_validators }
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
) -> Vec<Box<SingleReadValidator>> {
    let single_read_validators: Vec<Box<SingleReadValidator>> = vec![
        Box::new(NameValidator),
        Box::new(CompleteValidator),
        Box::new(AlphabetValidator::default()),
        Box::new(PlusLineValidator),
        Box::new(ConsistentSeqQualValidator),
    ];

    single_read_validators
        .into_iter()
        .filter(|v| v.level() <= validation_level)
        .collect()
}

fn filter_paired_read_validators(
    validation_level: ValidationLevel,
) -> Vec<Box<PairedReadValidator>> {
    let paired_read_validators: Vec<Box<PairedReadValidator>> = vec![
        Box::new(NamesValidator),
    ];

    paired_read_validators
        .into_iter()
        .filter(|v| v.level() <= validation_level)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_single_read_validators() {
        let validators = filter_single_read_validators(ValidationLevel::Minimum);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].name(), "CompleteValidator");
        assert_eq!(validators[1].name(), "PlusLineValidator");

        let validators = filter_single_read_validators(ValidationLevel::High);
        assert_eq!(validators.len(), 5);
    }

    #[test]
    fn test_filter_paired_read_validators() {
        let validators = filter_paired_read_validators(ValidationLevel::Minimum);
        assert_eq!(validators.len(), 0);

        let validators = filter_paired_read_validators(ValidationLevel::High);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].name(), "NamesValidator");
    }
}
