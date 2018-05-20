use std::str::FromStr;

use Block;

pub use self::single::SingleReadValidator;
pub use self::paired::PairedReadValidator;
use self::single::{
    AlphabetValidator,
    CompleteValidator,
    NameValidator,
    PlusLineValidator,
};
use self::paired::NamesValidator;

pub mod single;
pub mod paired;

#[derive(Debug)]
pub enum Error {
    Invalid(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum LintMode {
    Report,
    Error,
}

impl FromStr for LintMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "report" => Ok(LintMode::Report),
            "error" => Ok(LintMode::Error),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationLevel {
    Minimum,
    Low,
    High,
}

impl FromStr for ValidationLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minimum" => Ok(ValidationLevel::Minimum),
            "low" => Ok(ValidationLevel::Low),
            "high" => Ok(ValidationLevel::High),
            _ => Err(()),
        }
    }
}

pub struct BlockValidator {
    single_read_validation_level: ValidationLevel,
    paired_read_validation_level: ValidationLevel,
}

impl BlockValidator {
    pub fn new(
        single_read_validation_level: ValidationLevel,
        paired_read_validation_level: ValidationLevel,
    ) -> BlockValidator {
        BlockValidator {
            single_read_validation_level,
            paired_read_validation_level,
        }
    }

    pub fn validate(&self, b: &Block) -> Result<(), Error> {
        lazy_static! {
            static ref NAME_VALIDATOR: NameValidator = NameValidator;
            static ref COMPLETE_VALIDATOR: CompleteValidator = CompleteValidator;
            static ref ALPHABET_VALIDATOR: AlphabetValidator = Default::default();
            static ref PLUS_LINE_VALIDATOR: PlusLineValidator = PlusLineValidator;
        }

        if self.single_read_validation_level >= ValidationLevel::Minimum {
            PLUS_LINE_VALIDATOR.validate(b)?;
            COMPLETE_VALIDATOR.validate(b)?;

            if self.single_read_validation_level >= ValidationLevel::Low {
                ALPHABET_VALIDATOR.validate(b)?;

                if self.single_read_validation_level >= ValidationLevel::High {
                    NAME_VALIDATOR.validate(b)?;
                }
            }
        }

        Ok(())
    }

    pub fn validate_pair(&self, b: &Block, d: &Block) -> Result<(), Error> {
        lazy_static! {
            static ref NAMES_VALIDATOR: NamesValidator = NamesValidator;
        }

        self.validate(b)?;
        self.validate(d)?;

        if self.paired_read_validation_level >= ValidationLevel::Low {
            NAMES_VALIDATOR.validate(b, d)?;
        }

        Ok(())
    }
}
