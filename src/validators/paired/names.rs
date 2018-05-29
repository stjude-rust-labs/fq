use Block;
use validators::{Error, LineType, PairedReadValidator, ValidationLevel};

pub struct NamesValidator;

impl PairedReadValidator for NamesValidator {
    fn code(&self) -> &'static str {
        "P001"
    }

    fn name(&self) -> &'static str {
        "NamesValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Low
    }

    fn validate(&self, b: &Block, d: &Block) -> Result<(), Error> {
        if b.name != d.name {
            Err(Error::new(
                self.code(),
                self.name(),
                &format!("Names do not match (expected '{}', got '{}')", b.name, d.name),
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
    use super::NamesValidator;

    use Block;
    use validators::{PairedReadValidator, ValidationLevel};

    #[test]
    fn test_code() {
        let validator = NamesValidator;
        assert_eq!(validator.code(), "P001");
    }

    #[test]
    fn test_name() {
        let validator = NamesValidator;
        assert_eq!(validator.name(), "NamesValidator");
    }

    #[test]
    fn test_level() {
        let validator = NamesValidator;
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = NamesValidator;

        let b = Block::new("@fqlib/1", "", "", "");

        let d = Block::new("@fqlib/1", "", "", "");
        assert!(validator.validate(&b, &d).is_ok());

        let d = Block::new("@fqlib/2", "", "", "");
        assert!(validator.validate(&b, &d).is_ok());

        let d = Block::new("@/20180523", "", "", "");
        assert!(validator.validate(&b, &d).is_err());
    }
}
