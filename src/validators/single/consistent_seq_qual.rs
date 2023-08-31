use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidator, ValidationLevel},
};

/// [S005] (high) Validator to check if the sequence and quality lengths are the same.
pub struct ConsistentSeqQualValidator;

impl SingleReadValidator for ConsistentSeqQualValidator {
    fn code(&self) -> &'static str {
        "S005"
    }

    fn name(&self) -> &'static str {
        "ConsistentSeqQualValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::High
    }

    fn validate(&self, r: &Record) -> Result<(), validators::Error> {
        if r.sequence().len() != r.quality_scores().len() {
            Err(validators::Error::new(
                self.code(),
                self.name(),
                ValidationError {
                    actual: r.sequence().len(),
                    expected: r.quality_scores().len(),
                },
                LineType::Sequence,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
#[error("sequence-quality scores lengths mismatch: expected {actual}, got {expected}")]
struct ValidationError {
    actual: usize,
    expected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = ConsistentSeqQualValidator;
        assert_eq!(validator.code(), "S005");
    }

    #[test]
    fn test_name() {
        let validator = ConsistentSeqQualValidator;
        assert_eq!(validator.name(), "ConsistentSeqQualValidator");
    }

    #[test]
    fn test_level() {
        let validator = ConsistentSeqQualValidator;
        assert_eq!(validator.level(), ValidationLevel::High);
    }

    #[test]
    fn test_validate() {
        let validator = ConsistentSeqQualValidator;

        let record = Record::new("", "AGTC", "", "ABCD");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "AGTC", "", "ABC");
        assert!(validator.validate(&record).is_err());
    }
}
