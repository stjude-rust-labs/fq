use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidator, ValidationLevel},
};

/// [S006] (medium) Validator to check if all the characters in the quality line are between "!" and
/// "~" (ordinal values).
pub struct QualityStringValidator;

impl SingleReadValidator for QualityStringValidator {
    fn code(&self) -> &'static str {
        "S006"
    }

    fn name(&self) -> &'static str {
        "QualityStringValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Medium
    }

    fn validate(&self, r: &Record) -> Result<(), validators::Error> {
        for (i, &b) in r.quality_scores().iter().enumerate() {
            if !b.is_ascii_graphic() {
                return Err(validators::Error::new(
                    self.code(),
                    self.name(),
                    ValidationError(char::from(b)),
                    LineType::Quality,
                    Some(i + 1),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("invalid quality score character: '{0}'")]
struct ValidationError(char);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code() {
        let validator = QualityStringValidator;
        assert_eq!(validator.code(), "S006");
    }

    #[test]
    fn test_name() {
        let validator = QualityStringValidator;
        assert_eq!(validator.name(), "QualityStringValidator");
    }

    #[test]
    fn test_level() {
        let validator = QualityStringValidator;
        assert_eq!(validator.level(), ValidationLevel::Medium);
    }

    #[test]
    fn test_validate() {
        let validator = QualityStringValidator;

        let quality = r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##;
        let record = Record::new("", "", "", quality);
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "", "", "ab早いcd");
        assert!(validator.validate(&record).is_err());
    }
}
