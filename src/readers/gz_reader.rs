use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use noodles::formats::gz::MultiGzDecoder;

use Block;
use readers::{FastQReader, read_block};

pub struct GzReader {
    reader: BufReader<MultiGzDecoder<BufReader<File>>>,
    block: Block,
}

impl GzReader {
    pub fn open<P>(
        pathname: P,
    ) -> io::Result<GzReader>
    where
        P: AsRef<Path>,
    {
        let file = File::open(pathname)?;
        let reader = BufReader::new(file);
        let decoder = MultiGzDecoder::new(reader);
        Ok(GzReader::new(BufReader::new(decoder)))
    }

    pub fn new(reader: BufReader<MultiGzDecoder<BufReader<File>>>) -> GzReader {
        GzReader {
            reader,
            block: Block::default(),
        }
    }
}

impl FastQReader for GzReader {
    fn next_block(&mut self) -> Option<io::Result<&Block>> {
        self.block.clear();

        if let Ok(bytes_read) = read_block(&mut self.reader, &mut self.block) {
            if bytes_read > 0 {
                self.block.reset();
                return Some(Ok(&self.block));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use Block;
    use readers::FastQReader;
    use super::GzReader;

    #[test]
    fn test_next_block() {
        fn t(reader: &mut GzReader, d: &Block) {
            let b = reader.next_block().unwrap().unwrap();
            assert_eq!(b, d);
        }

        let mut reader = GzReader::open("test/fixtures/r1.fastq.gz").unwrap();

        t(&mut reader, &Block::new("@fqlib:1/1", "AGCT", "+", "abcd"));
        t(&mut reader, &Block::new("@fqlib:2/1", "TCGA", "+", "dcba"));
    }
}
