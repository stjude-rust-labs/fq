use Block;
use validators::{Error, ValidationLevel};

pub use self::alphabet::AlphabetValidator;
pub use self::complete::CompleteValidator;
pub use self::name::NameValidator;
pub use self::plus_line::PlusLineValidator;

mod alphabet;
mod complete;
mod name;
mod plus_line;

pub trait SingleReadValidator {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn level(&self) -> ValidationLevel;
    fn validate(&self, b: &Block) -> Result<(), Error>;
}
