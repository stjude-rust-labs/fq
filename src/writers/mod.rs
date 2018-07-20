pub use self::paired_writer::PairedWriter;

pub mod file_writer;
pub mod gz_writer;
pub mod paired_writer;

use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;

pub fn factory<P>(pathname: P) -> io::Result<Box<dyn Write>> where P: AsRef<Path> {
    let path = pathname.as_ref();

    match path.extension().and_then(OsStr::to_str) {
        Some("gz") => gz_writer::create(path).map(|w| Box::new(w) as Box<dyn Write>),
        _ => file_writer::create(path).map(|w| Box::new(w) as Box<dyn Write>),
    }
}
