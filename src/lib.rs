#![deny(bare_trait_objects)]

extern crate bloom;
#[macro_use] extern crate clap;
extern crate flate2;
#[macro_use] extern crate log;
extern crate noodles;
extern crate rand;

pub use block::Block;
pub use generator::BlockPairGenerator;
pub use pair_reader::PairReader;
pub use validators::ValidationLevel;
pub use writers::PairedWriter;

pub mod block;
pub mod commands;
pub mod distributions;
pub mod generator;
pub mod pair_reader;
pub mod validators;
pub mod writers;
