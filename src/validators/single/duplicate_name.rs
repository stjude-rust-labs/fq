use std::collections::HashSet;

use Block;
use validators::{Error, SingleReadValidatorMut, ValidationLevel};



pub struct DuplicateNameValidator {
    set: HashSet<String>,
}

impl DuplicateNameValidator {
    pub fn new() -> DuplicateNameValidator {
        DuplicateNameValidator { set: HashSet::new() }
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
        let name = b.name.clone();

        if self.set.contains(&name) {
            let message = format!(r#"Duplicate name exists: "{}""#, name);
            Err(Error::Invalid(String::from(message)))
        } else {
            self.set.insert(name);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DuplicateNameValidator;

    use Block;
    use validators::{SingleReadValidatorMut, ValidationLevel};

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

        let block = Block::new("@fqlib:1/1", "", "", "");
        assert!(validator.validate(&block).is_ok());

        let block = Block::new("@fqlib:2/1", "", "", "");
        assert!(validator.validate(&block).is_ok());

        assert!(validator.validate(&block).is_err());
    }
}
