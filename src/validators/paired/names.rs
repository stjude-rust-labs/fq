use noodles::formats::fastq::Record;

use validators::{Error, LineType, PairedReadValidator, ValidationLevel};

/// [P001] (medium) Validator to check if each paired read name is the same, excluding interleave.
pub struct NamesValidator;

impl PairedReadValidator for NamesValidator {
    fn code(&self) -> &'static str {
        "P001"
    }

    fn name(&self) -> &'static str {
        "NamesValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Medium
    }

    fn validate(&self, r: &Record, s: &Record) -> Result<(), Error> {
        if r.name() != s.name() {
            Err(Error::new(
                self.code(),
                self.name(),
                &format!(
                    "Names do not match (expected '{}', got '{}')",
                    String::from_utf8_lossy(r.name()),
                    String::from_utf8_lossy(s.name()),
                ),
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
    use noodles::formats::fastq::Record;

    use super::NamesValidator;
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
        assert_eq!(validator.level(), ValidationLevel::Medium);
    }

    #[test]
    fn test_validate() {
        let validator = NamesValidator;

        let r = Record::new("@fqlib/1", "", "", "");

        let s = Record::new("@fqlib/1", "", "", "");
        assert!(validator.validate(&r, &s).is_ok());

        let s = Record::new("@/20180523", "", "", "");
        assert!(validator.validate(&r, &s).is_err());
    }
}
