use std::collections::HashSet;

use Block;
use validators::{Error, SingleReadValidator, ValidationLevel};

pub struct AlphabetValidator {
    alphabet: HashSet<char>,
}

impl AlphabetValidator {
    pub fn new(characters: &str) -> AlphabetValidator {
        AlphabetValidator {
            alphabet: characters.chars().collect(),
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
        ValidationLevel::Low
    }

    fn validate(&self, b: &Block) -> Result<(), Error> {
        if !b.sequence.chars().all(|c| self.alphabet.contains(&c)) {
            let message = format!("Invalid character in sequence");
            Err(Error::Invalid(String::from(message)))
        } else {
            Ok(())
        }
    }
}

impl Default for AlphabetValidator {
    fn default() -> AlphabetValidator {
        AlphabetValidator::new("ACGTNacgtn")
    }
}

#[cfg(test)]
mod tests {
    use super::AlphabetValidator;

    use Block;
    use validators::{SingleReadValidator, ValidationLevel};

    #[test]
    fn test_new() {
        let validator = AlphabetValidator::new("abc");
        assert_eq!(validator.alphabet.len(), 3);
        assert!(validator.alphabet.contains(&'a'));
        assert!(validator.alphabet.contains(&'b'));
        assert!(validator.alphabet.contains(&'c'));
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
        assert_eq!(validator.level(), ValidationLevel::Low);
    }

    #[test]
    fn test_validate() {
        let validator = AlphabetValidator::default();

        let block = Block::new("", "AACCGGTTNNaaccggttnn", "", "");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("", "fqlib", "", "");
        assert!(validator.validate(&block).is_err());
    }
}
