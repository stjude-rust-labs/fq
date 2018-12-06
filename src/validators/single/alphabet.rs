use std::collections::HashSet;

use noodles::formats::fastq::Record;

use crate::validators::{Error, LineType, SingleReadValidator, ValidationLevel};

/// [S002] (medium) Validator to check if all the characters in the sequence line are included in a
/// given character set.
pub struct AlphabetValidator {
    alphabet: HashSet<u8>,
}

impl AlphabetValidator {
    pub fn new(characters: &[u8]) -> AlphabetValidator {
        AlphabetValidator {
            alphabet: characters.iter().cloned().collect(),
        }
    }
}

impl SingleReadValidator for AlphabetValidator {
    fn code(&self) -> &'static str {
        "S002"
    }

    fn name(&self) -> &'static str {
        "AlphabetValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::Medium
    }

    fn validate(&self, r: &Record) -> Result<(), Error> {
        for (i, &b) in r.sequence().iter().enumerate() {
            if !self.alphabet.contains(&b) {
                return Err(Error::new(
                    self.code(),
                    self.name(),
                    &format!("Invalid character: {}", b as char),
                    LineType::Sequence,
                    Some(i + 1),
                ));
            }
        }

        Ok(())
    }
}

impl Default for AlphabetValidator {
    /// Creates a validator with the alphabet "ACGTN", case-insensitive.
    fn default() -> AlphabetValidator {
        AlphabetValidator::new(b"ACGTNacgtn")
    }
}

#[cfg(test)]
mod tests {
    use noodles::formats::fastq::Record;

    use super::AlphabetValidator;
    use crate::validators::{SingleReadValidator, ValidationLevel};

    #[test]
    fn test_new() {
        let validator = AlphabetValidator::new(b"abc");
        assert_eq!(validator.alphabet.len(), 3);
        assert!(validator.alphabet.contains(&b'a'));
        assert!(validator.alphabet.contains(&b'b'));
        assert!(validator.alphabet.contains(&b'c'));
    }

    #[test]
    fn test_code() {
        let validator = AlphabetValidator::default();
        assert_eq!(validator.code(), "S002");
    }

    #[test]
    fn test_name() {
        let validator = AlphabetValidator::default();
        assert_eq!(validator.name(), "AlphabetValidator");
    }

    #[test]
    fn test_level() {
        let validator = AlphabetValidator::default();
        assert_eq!(validator.level(), ValidationLevel::Medium);
    }

    #[test]
    fn test_validate() {
        let validator = AlphabetValidator::default();

        let record = Record::new("", "AACCGGTTNNaaccggttnn", "", "");
        assert!(validator.validate(&record).is_ok());

        let record = Record::new("", "fqlib", "", "");
        assert!(validator.validate(&record).is_err());
    }
}
