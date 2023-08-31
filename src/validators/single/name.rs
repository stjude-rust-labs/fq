use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidator, ValidationLevel},
};

/// [S003] (high) Validator to check if the name line starts with an "@".
pub struct NameValidator;

impl SingleReadValidator for NameValidator {
    fn code(&self) -> &'static str {
        "S003"
    }

    fn name(&self) -> &'static str {
        "NameValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::High
    }

    fn validate(&self, r: &Record) -> Result<(), validators::Error> {
        match r.name().first() {
            Some(b'@') => Ok(()),
            _ => Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError,
                LineType::Name,
                Some(1),
            )),
        }
    }
}

#[derive(Debug, Error)]
#[error("missing @ prefix")]
struct ValidationError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = NameValidator;
        assert_eq!(validator.code(), "S003");
    }

    #[test]
    fn test_name() {
        let validator = NameValidator;
        assert_eq!(validator.name(), "NameValidator");
    }

    #[test]
    fn test_level() {
        let validator = NameValidator;
        assert_eq!(validator.level(), ValidationLevel::High);
    }

    #[test]
    fn test_validate() {
        let validator = NameValidator;

        let record = Record::new("@fqlib", "", "", "");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("+fqlib", "", "", "");
        assert!(validator.validate(&record).is_err());
    }
}
