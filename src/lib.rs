#[macro_use]
extern crate lazy_static;
extern crate rand;

pub use block::Block;
pub use generator::Generator;
pub use reader::PairedFastQReader;
pub use writer::Writer;
pub use validators::ValidationLevel;

pub mod block;
pub mod generator;
pub mod reader;
pub mod validators;
pub mod writer;
