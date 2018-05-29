use Block;
use validators::{Error, LineType, SingleReadValidator, ValidationLevel};

pub struct NameValidator;

impl SingleReadValidator for NameValidator {
    fn code(&self) -> &'static str {
        "S003"
    }

    fn name(&self) -> &'static str {
        "NameValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::High
    }

    fn validate(&self, b: &Block) -> Result<(), Error> {
        if !b.name.starts_with("@") {
            Err(Error::new(
                self.code(),
                self.name(),
                &String::from("Does not start with an '@'"),
                LineType::Name,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NameValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

    #[test]
    fn test_code() {
        let validator = NameValidator;
        assert_eq!(validator.code(), "S003");
    }

    #[test]
    fn test_name() {
        let validator = NameValidator;
        assert_eq!(validator.name(), "NameValidator");
    }

    #[test]
    fn test_level() {
        let validator = NameValidator;
        assert_eq!(validator.level(), ValidationLevel::High);
    }

    #[test]
    fn test_validate() {
        let validator = NameValidator;

        let block = Block::new("@fqlib", "", "", "");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("+fqlib", "", "", "");
        assert!(validator.validate(&block).is_err());
    }
}
