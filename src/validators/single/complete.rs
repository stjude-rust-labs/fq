use Block;
use validators::{Error, SingleReadValidator, ValidationLevel};

pub struct CompleteValidator;

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
        validate_name(b)?;
        validate_sequence(b)?;
        validate_plus_line(b)?;
        validate_quality(b)?;
        Ok(())
    }
}

fn validate_name(b: &Block) -> Result<(), Error> {
    if b.name.is_empty() {
        Err(Error::Invalid(String::from("Block incomplete (name is empty)")))
    } else {
        Ok(())
    }
}

fn validate_sequence(b: &Block) -> Result<(), Error> {
    if b.sequence.is_empty() {
        Err(Error::Invalid(String::from("Block incomplete (sequence is empty)")))
    } else {
        Ok(())
    }
}

fn validate_plus_line(b: &Block) -> Result<(), Error> {
    if b.plus_line.is_empty() {
        Err(Error::Invalid(String::from("Block incomplete (plus line is empty)")))
    } else {
        Ok(())
    }
}

fn validate_quality(b: &Block) -> Result<(), Error> {
    if b.quality.is_empty() {
        Err(Error::Invalid(String::from("Block incomplete (quality is empty)")))
    } else {
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
