extern crate bloom;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate rand;

pub use block::Block;
pub use generator::BlockPairGenerator;
pub use readers::{FastQReader, FileReader, GzReader, PairedReader};
pub use writer::Writer;
pub use validators::ValidationLevel;

pub mod block;
pub mod distributions;
pub mod generator;
pub mod readers;
pub mod validators;
pub mod writer;
