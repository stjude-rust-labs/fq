pub use self::gz_reader::GzReader;
pub use self::file_reader::FileReader;
pub use self::paired_reader::PairedReader;

pub mod file_reader;
pub mod gz_reader;
pub mod paired_reader;

use std::io::{self, BufRead};

use Block;

pub trait FastQReader {
    fn next_block(&mut self) -> Option<io::Result<&Block>>;
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
