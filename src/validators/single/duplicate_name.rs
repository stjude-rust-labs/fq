use std::collections::HashMap;

use bloom::ScalableBloomFilter;

use Block;
use validators::{Error, LineType, SingleReadValidatorMut, ValidationLevel};

const FALSE_POSITIVE_PROBABILITY: f64 = 0.0001;
const INITIAL_CAPACITY: usize = 10000;

pub struct DuplicateNameValidator {
    filter: ScalableBloomFilter,
    possible_duplicates: HashMap<String, u8>,
}

impl DuplicateNameValidator {
    pub fn new() -> DuplicateNameValidator {
        DuplicateNameValidator {
            filter: ScalableBloomFilter::new(
                FALSE_POSITIVE_PROBABILITY,
                INITIAL_CAPACITY,
            ),
            possible_duplicates: HashMap::new(),
        }
    }
}

impl DuplicateNameValidator {
    pub fn insert(&mut self, b: &Block) {
        let name = &b.name;

        if self.filter.contains_or_insert(name) {
            self.possible_duplicates.insert(name.clone(), 0);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.possible_duplicates.is_empty()
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
        let code = self.code();
        let name = self.name();

        if let Some(count) = self.possible_duplicates.get_mut(&b.name) {
            if *count >= 1 {
                return Err(Error::new(
                    code,
                    name,
                    &format!("Duplicate found: '{}'", b.name),
                    LineType::Name,
                    Some(1),
                ));
            }

            *count += 1;
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
    fn test_is_empty() {
        let validator = DuplicateNameValidator::new();
        assert!(validator.is_empty());
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

        let b = Block::new("@fqlib:1", "", "", "");
        let d = Block::new("@fqlib:2", "", "", "");

        validator.insert(&b);
        validator.insert(&d);
        validator.insert(&d);

        assert!(validator.validate(&b).is_ok());
        assert!(validator.validate(&d).is_ok());
        assert!(validator.validate(&d).is_err());
    }
}
