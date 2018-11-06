use noodles::formats::fastq::Record;

use validators::{Error, LineType, SingleReadValidator, ValidationLevel};

/// [S006] (medium) Validator to check if all the characters in the quality line are between "!" and
/// "~" (ordinal values).
pub struct QualityStringValidator;

const START_OFFSET: u8 = b'!';
const END_OFFSET: u8 = b'~';

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

    fn validate(&self, r: &Record) -> Result<(), Error> {
        for (i, &b) in r.quality().iter().enumerate() {
            if b < START_OFFSET || b > END_OFFSET {
                return Err(Error::new(
                    self.code(),
                    self.name(),
                    &format!("Invalid character '{}'", b as char),
                    LineType::Quality,
                    Some(i + 1),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use noodles::formats::fastq::Record;

    use validators::{SingleReadValidator, ValidationLevel};
    use super::QualityStringValidator;
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
