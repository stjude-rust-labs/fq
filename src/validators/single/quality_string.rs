use Block;
use validators::{Error, SingleReadValidator, ValidationLevel};

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
        ValidationLevel::Low
    }

    fn validate(&self, b: &Block) -> Result<(), Error> {
        for c in b.quality.chars() {
            let o = c as u32;

            if o < START_OFFSET || o > END_OFFSET {
                let message = format!(r#"Quality string contains an invalid character: "{}""#, c);
                return Err(Error::Invalid(String::from(message)));
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
        assert_eq!(validator.level(), ValidationLevel::Low);
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
