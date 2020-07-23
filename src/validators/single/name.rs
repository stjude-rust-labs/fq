use crate::{
    fastq::Record,
    validators::{Error, LineType, SingleReadValidator, ValidationLevel},
};

/// [S003] (high) Validator to check if the name line starts with an "@".
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

    fn validate(&self, r: &Record) -> Result<(), Error> {
        match r.name().first() {
            Some(b'@') => Ok(()),
            _ => Err(Error::new(
                self.code(),
                self.name(),
                String::from("Does not start with an '@'"),
                LineType::Name,
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

        let record = Record::new("@fqlib", "", "", "");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("+fqlib", "", "", "");
        assert!(validator.validate(&record).is_err());
    }
}
