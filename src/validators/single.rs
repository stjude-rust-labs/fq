//! Validators that use records from a single read.

mod alphabet;
mod complete;
mod consistent_seq_qual;
mod duplicate_name;
mod name;
mod plus_line;
mod quality_string;

pub use self::{
    alphabet::AlphabetValidator, complete::CompleteValidator,
    consistent_seq_qual::ConsistentSeqQualValidator, duplicate_name::DuplicateNameValidator,
    name::NameValidator, plus_line::PlusLineValidator, quality_string::QualityStringValidator,
};

use crate::{
    fastq::Record,
    validators::{Error, ValidationLevel},
};

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
