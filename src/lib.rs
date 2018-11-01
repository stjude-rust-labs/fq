#![deny(bare_trait_objects)]

extern crate bloom;
#[macro_use] extern crate clap;
extern crate flate2;
#[macro_use] extern crate log;
extern crate noodles;
extern crate rand;

pub use block::Block;
pub use generator::BlockPairGenerator;
pub use readers::{FastQReader, FileReader, GzReader, PairedReader};
pub use validators::ValidationLevel;
pub use writers::PairedWriter;

pub mod block;
pub mod commands;
pub mod distributions;
pub mod generator;
pub mod readers;
pub mod validators;
pub mod writers;
