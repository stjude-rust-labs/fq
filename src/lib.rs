#![deny(bare_trait_objects)]

pub mod cli;
pub mod commands;
pub mod distributions;
pub mod fastq;
pub mod generator;
pub mod pair_writer;
pub mod validators;

pub use crate::{
    cli::Cli, generator::Generator, pair_writer::PairWriter, validators::ValidationLevel,
};
