use std::collections::HashMap;

use bbloom::ScalableBloomFilter;
use thiserror::Error;

use crate::{
    fastq::Record,
    validators::{self, LineType, SingleReadValidatorMut, ValidationLevel},
};

const FALSE_POSITIVE_PROBABILITY: f64 = 0.0001;
const INITIAL_CAPACITY: usize = 10_000_000;

/// [S007] (high) Validator to check if all record names are unique.
///
/// The implementation of this validator uses a Bloom filter, a probabilistic data structure.
/// Because of this, it must be used in two passes: the first to add all names to the set
/// ([`insert`]), which may or may not hit duplicates; and the second, checking that list of
/// possible duplicates ([`validate`]).
///
/// [`insert`]: #method.insert
/// [`validate`]: #method.validate
pub struct DuplicateNameValidator {
    filter: ScalableBloomFilter,
    possible_duplicates: HashMap<Vec<u8>, u8>,
}

impl DuplicateNameValidator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DuplicateNameValidator {
    /// Adds a record name to the set.
    ///
    /// This also records possible duplicates to be used in the validation pass.
    pub fn insert(&mut self, r: &Record) {
        let name = r.name();

        if self.filter.contains_or_insert(name) {
            self.possible_duplicates.insert(name.to_vec(), 0);
        }
    }

    /// Returns whether there are possible duplicates.
    ///
    /// This is only useful if [`insert`] was previously called for all names.
    ///
    /// [`insert`]: #method.insert
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

    fn validate(&mut self, r: &Record) -> Result<(), validators::Error> {
        if let Some(count) = self.possible_duplicates.get_mut(r.name()) {
            if *count >= 1 {
                return Err(validators::Error::new(
                    self.code(),
                    self.name(),
                    ValidationError(String::from_utf8_lossy(r.name()).into()),
                    LineType::Name,
                    Some(1),
                ));
            }

            *count += 1;
        }

        Ok(())
    }
}

impl Default for DuplicateNameValidator {
    fn default() -> Self {
        Self {
            filter: ScalableBloomFilter::new(FALSE_POSITIVE_PROBABILITY, INITIAL_CAPACITY),
            possible_duplicates: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
#[error("duplicate name: '{0}'")]
struct ValidationError(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut validator = DuplicateNameValidator::new();
        let record = Record::new("@fqlib:1", "", "", "");
        validator.insert(&record);
    }

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

        let r = Record::new("@fqlib:1", "", "", "");
        let s = Record::new("@fqlib:2", "", "", "");

        // pass 1
        validator.insert(&r);
        validator.insert(&s);
        validator.insert(&s);

        // pass 2
        assert!(validator.validate(&r).is_ok());
        assert!(validator.validate(&s).is_ok());
        assert!(validator.validate(&s).is_err());
    }
}
