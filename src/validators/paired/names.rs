use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, PairedReadValidator, ValidationLevel},
};

/// [P001] (medium) Validator to check if each paired read name is the same, excluding interleave.
pub struct NamesValidator;

impl PairedReadValidator for NamesValidator {
    fn code(&self) -> &'static str {
        "P001"
    }

    fn name(&self) -> &'static str {
        "NamesValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Medium
    }

    fn validate(&self, r: &Record, s: &Record) -> Result<(), validators::Error> {
        if r.name() != s.name() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError {
                    actual: String::from_utf8_lossy(s.name()).into(),
                    expected: String::from_utf8_lossy(r.name()).into(),
                },
                LineType::Name,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
#[error("names mismatch: expected '{actual}', got '{expected}'")]
struct ValidationError {
    actual: String,
    expected: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = NamesValidator;
        assert_eq!(validator.code(), "P001");
    }

    #[test]
    fn test_name() {
        let validator = NamesValidator;
        assert_eq!(validator.name(), "NamesValidator");
    }

    #[test]
    fn test_level() {
        let validator = NamesValidator;
        assert_eq!(validator.level(), ValidationLevel::Medium);
    }

    #[test]
    fn test_validate() {
        let validator = NamesValidator;

        let r = Record::new("@fqlib/1", "", "", "");

        let s = Record::new("@fqlib/1", "", "", "");
        assert!(validator.validate(&r, &s).is_ok());

        let s = Record::new("@/20180523", "", "", "");
        assert!(validator.validate(&r, &s).is_err());
    }
}
