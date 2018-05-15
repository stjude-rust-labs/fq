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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

fn validate(b: &Block) -> Result<(), Error> {
    lazy_static! {
        static ref NAME_VALIDATOR: NameValidator = NameValidator;
        static ref COMPLETE_VALIDATOR: CompleteValidator = CompleteValidator;
        static ref ALPHABET_VALIDATOR: AlphabetValidator = Default::default();
        static ref PLUS_LINE_VALIDATOR: PlusLineValidator = PlusLineValidator;
    }

    PLUS_LINE_VALIDATOR.validate(b)?;
    ALPHABET_VALIDATOR.validate(b)?;
    NAME_VALIDATOR.validate(b)?;
    COMPLETE_VALIDATOR.validate(b)
}

pub fn validate_pair(b: &Block, d: &Block) -> Result<(), Error> {
    lazy_static! {
        static ref NAMES_VALIDATOR: NamesValidator = NamesValidator;
    }

    validate(b)?;
    validate(d)?;

    NAMES_VALIDATOR.validate(b, d)
}
