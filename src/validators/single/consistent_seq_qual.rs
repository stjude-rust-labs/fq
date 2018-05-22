use Block;
use validators::{Error, SingleReadValidator, ValidationLevel};

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

    fn validate(&self, b: &Block) -> Result<(), Error> {
        if b.sequence.len() != b.quality.len() {
            let message = "Name and quality lengths do not match";
            Err(Error::Invalid(String::from(message)))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConsistentSeqQualValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

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

        let block = Block::new("", "AGTC", "", "ABCD");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("", "AGTC", "", "ABC");
        assert!(validator.validate(&block).is_err());
    }
}
