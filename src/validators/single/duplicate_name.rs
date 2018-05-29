use std::collections::HashMap;

use bloom::ScalableBloomFilter;

use Block;
use validators::{Error, SingleReadValidatorMut, ValidationLevel};

const FALSE_POSITIVE_PROBABILITY: f64 = 0.0001;
const INITIAL_CAPACITY: usize = 10000;

pub struct DuplicateNameValidator {
    filter: ScalableBloomFilter,
    false_positives: HashMap<String, u8>,
}

impl DuplicateNameValidator {
    pub fn new() -> DuplicateNameValidator {
        DuplicateNameValidator {
            filter: ScalableBloomFilter::new(
                FALSE_POSITIVE_PROBABILITY,
                INITIAL_CAPACITY,
            ),
            false_positives: HashMap::new(),
        }
    }
}

impl DuplicateNameValidator {
    pub fn contains_once(&mut self, name: &str) -> bool {
        if let Some(count) = self.false_positives.get_mut(name) {
            if *count >= 1 {
                return false;
            }

            *count += 1;
        }

        true
    }
}

impl SingleReadValidatorMut for DuplicateNameValidator {
    fn code(&self) -> &'static str {
        "S007"
    }

    fn name(&self) -> &'static str {
        "DuplicateNameValidator"
    }

    fn level(&self) -> ValidationLevel {
        ValidationLevel::High
    }

    fn validate(&mut self, b: &Block) -> Result<(), Error> {
        let name = &b.name;

        if self.filter.contains_or_insert(name) {
            self.false_positives.insert(name.clone(), 0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DuplicateNameValidator;

    use Block;
    use validators::{SingleReadValidatorMut, ValidationLevel};

    #[test]
    fn test_contains_once() {
        let mut validator = DuplicateNameValidator::new();

        let block = Block::new("@fqlib:1", "", "", "");
        validator.validate(&block).unwrap();

        let block = Block::new("@fqlib:1", "", "", "");
        validator.validate(&block).unwrap();

        let block = Block::new("@fqlib:2", "", "", "");
        validator.validate(&block).unwrap();

        assert!(validator.contains_once("@fqlib:1"));
        assert!(!validator.contains_once("@fqlib:1"));
        assert!(validator.contains_once("@fqlib:2"));
    }

    #[test]
    fn test_code() {
        let validator = DuplicateNameValidator::new();
        assert_eq!(validator.code(), "S007");
    }

    #[test]
    fn test_name() {
        let validator = DuplicateNameValidator::new();
        assert_eq!(validator.name(), "DuplicateNameValidator");
    }

    #[test]
    fn test_level() {
        let validator = DuplicateNameValidator::new();
        assert_eq!(validator.level(), ValidationLevel::High);
    }

    #[test]
    fn test_validate() {
        let mut validator = DuplicateNameValidator::new();

        let block = Block::new("@fqlib:1", "", "", "");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("@fqlib:2", "", "", "");
        assert!(validator.validate(&block).is_ok());
    }
}
