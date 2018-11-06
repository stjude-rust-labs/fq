use noodles::formats::fastq::Record;

use validators::{Error, LineType, SingleReadValidator, ValidationLevel};

/// [S004] (low) Validator to check if all four block lines (name, sequence, plus line, and
/// quality) are present.
pub struct CompleteValidator;

impl CompleteValidator {
    fn validate_name(&self, r: &Record) -> Result<(), Error> {
        if r.name().is_empty() {
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

    fn validate_sequence(&self, r: &Record) -> Result<(), Error> {
        if r.sequence().is_empty() {
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

    fn validate_plus_line(&self, r: &Record) -> Result<(), Error> {
        if r.plus_line().is_empty() {
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

    fn validate_quality(&self, r: &Record) -> Result<(), Error> {
        if r.quality().is_empty() {
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
        ValidationLevel::Low
    }

    fn validate(&self, r: &Record) -> Result<(), Error> {
        self.validate_name(r)?;
        self.validate_sequence(r)?;
        self.validate_plus_line(r)?;
        self.validate_quality(r)?;
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use noodles::formats::fastq::Record;

    use super::CompleteValidator;
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
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = CompleteValidator;

        let record = Record::new("@fqlib", "AGCT", "+", "abcd");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "AGCT", "+", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "", "+", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "AGCT", "", "abcd");
        assert!(validator.validate(&record).is_err());

        let record = Record::new("@fqlib", "AGCT", "+", "");
        assert!(validator.validate(&record).is_err());
    }
}
