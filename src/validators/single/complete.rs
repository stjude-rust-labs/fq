use Block;
use validators::{Error, LineType, SingleReadValidator, ValidationLevel};

pub struct CompleteValidator;

impl CompleteValidator {
    fn validate_name(&self, b: &Block) -> Result<(), Error> {
        if b.name.is_empty() {
            Err(Error::new(
                self.code(),
                self.name(),
                &String::from("Incomplete block: name is empty"),
                LineType::Name,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_sequence(&self, b: &Block) -> Result<(), Error> {
        if b.sequence.is_empty() {
            Err(Error::new(
                self.code(),
                self.name(),
                &String::from("Incomplete block: sequence is empty"),
                LineType::Sequence,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_plus_line(&self, b: &Block) -> Result<(), Error> {
        if b.plus_line.is_empty() {
            Err(Error::new(
                self.code(),
                self.name(),
                &String::from("Incomplete block: plus line is empty"),
                LineType::PlusLine,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_quality(&self, b: &Block) -> Result<(), Error> {
        if b.quality.is_empty() {
            Err(Error::new(
                self.code(),
                self.name(),
                &String::from("Incomplete block: quality is empty"),
                LineType::Quality,
                Some(1),
            ))
        } else {
            Ok(())
        }
    }
}

impl SingleReadValidator for CompleteValidator {
    fn code(&self) -> &'static str {
        "S004"
    }

    fn name(&self) -> &'static str {
        "CompleteValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Minimum
    }

    fn validate(&self, b: &Block) -> Result<(), Error> {
        self.validate_name(b)?;
        self.validate_sequence(b)?;
        self.validate_plus_line(b)?;
        self.validate_quality(b)?;
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::CompleteValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

    #[test]
    fn test_code() {
        let validator = CompleteValidator;
        assert_eq!(validator.code(), "S004");
    }

    #[test]
    fn test_name() {
        let validator = CompleteValidator;
        assert_eq!(validator.name(), "CompleteValidator");
    }

    #[test]
    fn test_level() {
        let validator = CompleteValidator;
        assert_eq!(validator.level(), ValidationLevel::Minimum);
    }

    #[test]
    fn test_validate() {
        let validator = CompleteValidator;

        let block = Block::new("@fqlib", "AGCT", "+", "abcd");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("", "AGCT", "+", "abcd");
        assert!(validator.validate(&block).is_err());

        let block = Block::new("@fqlib", "", "+", "abcd");
        assert!(validator.validate(&block).is_err());

        let block = Block::new("@fqlib", "AGCT", "", "abcd");
        assert!(validator.validate(&block).is_err());

        let block = Block::new("@fqlib", "AGCT", "+", "");
        assert!(validator.validate(&block).is_err());
    }
}
