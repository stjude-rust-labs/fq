use std::io;

use Block;
use readers::FastQReader;

pub struct PairedReader<R: FastQReader> {
    reader_1: R,
    reader_2: R,
}

impl<R: FastQReader> PairedReader<R> {
    pub fn new(reader_1: R, reader_2: R) -> PairedReader<R> {
        PairedReader { reader_1, reader_2 }
    }

    pub fn next_pair(&mut self) -> Option<(io::Result<&Block>, io::Result<&Block>)> {
        if let Some(b) = self.reader_1.next_block() {
            if let Some(d) = self.reader_2.next_block() {
                return Some((b, d));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use readers::FileReader;
    use super::PairedReader;

    #[test]
    fn test_next_pair() {
        let r1 = FileReader::open("test/fixtures/r1.fastq").unwrap();
        let r2 = FileReader::open("test/fixtures/r2.fastq").unwrap();
        let mut reader = PairedReader::new(r1, r2);

        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_none());
    }
}
