#![deny(bare_trait_objects)]

extern crate bloom;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate noodles;
extern crate rand;

pub use crate::generator::Generator;
pub use crate::pair_writer::PairWriter;
pub use crate::validators::ValidationLevel;

pub mod commands;
pub mod distributions;
pub mod generator;
pub mod pair_writer;
pub mod validators;
pub mod record;
