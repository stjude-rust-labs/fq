use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidator, ValidationLevel},
};

/// [S001] (low) Validator to check if the plus line starts with a "+".
pub struct PlusLineValidator;

impl SingleReadValidator for PlusLineValidator {
    fn code(&self) -> &'static str {
        "S001"
    }

    fn name(&self) -> &'static str {
        "PlusLineValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Low
    }

    fn validate(&self, r: &Record) -> Result<(), validators::Error> {
        match r.plus_line().first() {
            Some(b'+') => Ok(()),
            _ => Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError,
                LineType::PlusLine,
                Some(1),
            )),
        }
    }
}

#[derive(Debug, Error)]
#[error("missing + prefix")]
struct ValidationError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = PlusLineValidator;
        assert_eq!(validator.code(), "S001");
    }

    #[test]
    fn test_name() {
        let validator = PlusLineValidator;
        assert_eq!(validator.name(), "PlusLineValidator");
    }

    #[test]
    fn test_level() {
        let validator = PlusLineValidator;
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = PlusLineValidator;

        let record = Record::new("", "", "+", "");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "", "", "");
        assert!(validator.validate(&record).is_err());
    }
}
