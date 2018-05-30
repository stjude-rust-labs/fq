//! Validators that use blocks from two reads.

use Block;
use validators::{Error, ValidationLevel};

pub use self::names::NamesValidator;

mod names;

pub trait PairedReadValidator {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&self, b: &Block, d: &Block) -> Result<(), Error>;
}
