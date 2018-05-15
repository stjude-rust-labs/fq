use Block;
use validators::{Error, SingleReadValidator, ValidationLevel};

pub struct PlusLineValidator;

impl SingleReadValidator for PlusLineValidator {
    fn code(&self) -> &'static str {
        "S001"
    }

    fn name(&self) -> &'static str {
        "PlusLineValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Minimum
    }

    fn validate(&self, b: &Block) -> Result<(), Error> {
        if !b.plus_line.starts_with("+") {
            let message = "The plus line does not start with a plus.";
            Err(Error::Invalid(String::from(message)))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PlusLineValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

    #[test]
    fn test_code() {
        let validator = PlusLineValidator;
        assert_eq!(validator.code(), "S001");
    }

    #[test]
    fn test_name() {
        let validator = PlusLineValidator;
        assert_eq!(validator.name(), "PlusLineValidator");
    }

    #[test]
    fn test_level() {
        let validator = PlusLineValidator;
        assert_eq!(validator.level(), ValidationLevel::Minimum);
    }

    #[test]
    fn test_validate() {
        let validator = PlusLineValidator;

        let block = Block::new("", "", "+", "");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("", "", "", "");
        assert!(validator.validate(&block).is_err());
    }
}
