use noodles::formats::fastq::Record;

use crate::validators::{Error, LineType, SingleReadValidator, ValidationLevel};

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

    fn validate(&self, r: &Record) -> Result<(), Error> {
        if r.sequence().len() != r.quality().len() {
            let message = format!(
                "Name and quality lengths do not match (expected {}, got {})",
                r.sequence().len(),
                r.quality().len(),
            );

            Err(Error::new(
                self.code(),
                self.name(),
                &message,
                LineType::Sequence,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use noodles::formats::fastq::Record;

    use super::ConsistentSeqQualValidator;
    use crate::validators::{SingleReadValidator, ValidationLevel};

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
