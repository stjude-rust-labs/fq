//! Validators that use records from a single read.

use crate::{
    fastq::Record,
    validators::{Error, ValidationLevel},
};

pub use self::alphabet::AlphabetValidator;
pub use self::complete::CompleteValidator;
pub use self::consistent_seq_qual::ConsistentSeqQualValidator;
pub use self::duplicate_name::DuplicateNameValidator;
pub use self::name::NameValidator;
pub use self::plus_line::PlusLineValidator;
pub use self::quality_string::QualityStringValidator;

mod alphabet;
mod complete;
mod consistent_seq_qual;
mod duplicate_name;
mod name;
mod plus_line;
mod quality_string;

pub trait SingleReadValidator {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&self, r: &Record) -> Result<(), Error>;
}

pub trait SingleReadValidatorMut {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&mut self, r: &Record) -> Result<(), Error>;
}
