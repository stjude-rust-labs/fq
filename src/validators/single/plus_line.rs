use crate::{
    fastq::Record,
    validators::{Error, LineType, SingleReadValidator, ValidationLevel},
};

/// [S001] (low) Validator to check if the plus line starts with a "+".
pub struct PlusLineValidator;

impl SingleReadValidator for PlusLineValidator {
    fn code(&self) -> &'static str {
        "S001"
    }

    fn name(&self) -> &'static str {
        "PlusLineValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Low
    }

    fn validate(&self, r: &Record) -> Result<(), Error> {
        match r.plus_line().first() {
            Some(b'+') => Ok(()),
            _ => Err(Error::new(
                self.code(),
                self.name(),
                String::from("Does not start with a '+'"),
                LineType::PlusLine,
                Some(1),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = PlusLineValidator;

        let record = Record::new("", "", "+", "");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "", "", "");
        assert!(validator.validate(&record).is_err());
    }
}
