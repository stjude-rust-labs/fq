use Block;
use validators::{Error, LineType, SingleReadValidator, ValidationLevel};

/// [S006] (medium) Validator to check if all the characters in the quality line are between "!" and
/// "~" (ordinal values).
pub struct QualityStringValidator;

const START_OFFSET: u32 = b'!' as u32;
const END_OFFSET: u32 = b'~' as u32;

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

    fn validate(&self, b: &Block) -> Result<(), Error> {
        for (i, c) in b.quality.chars().enumerate() {
            let o = c as u32;

            if o < START_OFFSET || o > END_OFFSET {
                return Err(Error::new(
                    self.code(),
                    self.name(),
                    &format!("Invalid character '{}'", c),
                    LineType::Name,
                    Some(i + 1),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::QualityStringValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

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
        let block = Block::new("", "", "", quality);
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("", "", "", "ab早いcd");
        assert!(validator.validate(&block).is_err());
    }
}
