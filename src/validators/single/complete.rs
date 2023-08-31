use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidator, ValidationLevel},
};

/// [S004] (low) Validator to check if all four record lines (name, sequence, plus line, and
/// quality) are present.
pub struct CompleteValidator;

impl CompleteValidator {
    fn validate_name(&self, r: &Record) -> Result<(), validators::Error> {
        if r.name().is_empty() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError::EmptyName,
                LineType::Name,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_sequence(&self, r: &Record) -> Result<(), validators::Error> {
        if r.sequence().is_empty() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError::EmptySequence,
                LineType::Sequence,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_plus_line(&self, r: &Record) -> Result<(), validators::Error> {
        if r.plus_line().is_empty() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError::EmptyPlusLine,
                LineType::PlusLine,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_quality(&self, r: &Record) -> Result<(), validators::Error> {
        if r.quality_scores().is_empty() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError::EmptyQuality,
                LineType::Quality,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

impl SingleReadValidator for CompleteValidator {
    fn code(&self) -> &'static str {
        "S004"
    }

    fn name(&self) -> &'static str {
        "CompleteValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Low
    }

    fn validate(&self, r: &Record) -> Result<(), validators::Error> {
        self.validate_name(r)?;
        self.validate_sequence(r)?;
        self.validate_plus_line(r)?;
        self.validate_quality(r)?;
        Ok(())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
enum ValidationError {
    #[error("empty name")]
    EmptyName,
    #[error("empty sequence")]
    EmptySequence,
    #[error("empty plus line")]
    EmptyPlusLine,
    #[error("empty quality")]
    EmptyQuality,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = CompleteValidator;
        assert_eq!(validator.code(), "S004");
    }

    #[test]
    fn test_name() {
        let validator = CompleteValidator;
        assert_eq!(validator.name(), "CompleteValidator");
    }

    #[test]
    fn test_level() {
        let validator = CompleteValidator;
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = CompleteValidator;

        let record = Record::new("@fqlib", "AGCT", "+", "abcd");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "AGCT", "+", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "", "+", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "AGCT", "", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "AGCT", "+", "");
        assert!(validator.validate(&record).is_err());
    }
}
