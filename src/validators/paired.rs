//! Validators that use records from paired reads.

use noodles_fastq::Record;

use crate::validators::{Error, ValidationLevel};

pub use self::names::NamesValidator;

mod names;

pub trait PairedReadValidator {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&self, r: &Record, s: &Record) -> Result<(), Error>;
}
