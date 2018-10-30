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

pub fn factory<P>(pathname: P) -> io::Result<Box<dyn FastQReader>> where P: AsRef<Path> {
    let path = pathname.as_ref();

    match path.extension().and_then(OsStr::to_str) {
        Some("gz") => GzReader::open(path).map(|w| Box::new(w) as Box<dyn FastQReader>),
        _ => FileReader::open(path).map(|w| Box::new(w) as Box<dyn FastQReader>),
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

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::read_line;

    #[test]
    fn test_read_line() {
        let data = "@fqlib\nAGCT\n";
        let mut reader = BufReader::new(data.as_bytes());

        let mut buf = String::new();
        let len = read_line(&mut reader, &mut buf).unwrap();
        assert_eq!(len, 7);
        assert_eq!(buf, "@fqlib");

        buf.clear();
        let len = read_line(&mut reader, &mut buf).unwrap();
        assert_eq!(len, 5);
        assert_eq!(buf, "AGCT");

        buf.clear();
        let len = read_line(&mut reader, &mut buf).unwrap();
        assert_eq!(len, 0);
    }
}
