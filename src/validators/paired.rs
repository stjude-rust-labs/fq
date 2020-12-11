//! Validators that use records from paired reads.

mod names;

pub use self::names::NamesValidator;

use crate::{
    fastq::Record,
    validators::{Error, ValidationLevel},
};

pub trait PairedReadValidator {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&self, r: &Record, s: &Record) -> Result<(), Error>;
}
