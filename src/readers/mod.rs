pub use self::gz_reader::GzReader;
pub use self::file_reader::FileReader;
pub use self::paired_reader::PairedReader;

pub mod file_reader;
pub mod gz_reader;
pub mod paired_reader;

use std::ffi::OsStr;
use std::io::{self, BufRead};
use std::path::Path;

use Block;

pub trait FastQReader {
    fn next_block(&mut self) -> Option<io::Result<&Block>>;
}

impl<R: FastQReader + ?Sized> FastQReader for Box<R> {
    fn next_block(&mut self) -> Option<io::Result<&Block>> {
        (**self).next_block()
    }
}

pub fn factory<P>(pathname: P) -> io::Result<Box<FastQReader>> where P: AsRef<Path> {
    let path = pathname.as_ref();

    match path.extension().and_then(OsStr::to_str) {
        Some("gz") => GzReader::open(path).map(|w| Box::new(w) as Box<FastQReader>),
        _ => FileReader::open(path).map(|w| Box::new(w) as Box<FastQReader>),
    }
}

pub fn read_line<R: BufRead>(reader: &mut R, buf: &mut String) -> io::Result<usize> {
    let result = reader.read_line(buf);

    // Chomp newline.
    if result.is_ok() {
        let len = buf.len();

        if len > 0 {
            buf.truncate(len - 1);
        }
    }

    result
}
