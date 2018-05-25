use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

use Block;

pub struct FastQReader<R: BufRead> {
    reader: R,
    block: Block,
}

impl<R: BufRead> FastQReader<R> {
    pub fn open<P>(
        pathname: P,
    ) -> io::Result<FastQReader<BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(pathname)?;
        let reader = BufReader::new(file);
        Ok(FastQReader::new(reader))
    }

    pub fn new(reader: R) -> FastQReader<R> {
        FastQReader {
            reader,
            block: Block::new("", "", "", ""),
        }
    }

    pub fn next_block(&mut self) -> Option<io::Result<&Block>> {
        if let Ok(bytes_read) = read_line(&mut self.reader, &mut self.block.name) {
            if bytes_read > 0 {
                // FIXME
                read_line(&mut self.reader, &mut self.block.sequence).unwrap();
                read_line(&mut self.reader, &mut self.block.plus_line).unwrap();
                read_line(&mut self.reader, &mut self.block.quality).unwrap();

                self.block.reset();

                return Some(Ok(&self.block));
            }
        }

        None
    }
}

fn read_line<R: BufRead>(reader: &mut R, buf: &mut String) -> io::Result<usize> {
    buf.clear();

    let result = reader.read_line(buf);

    if result.is_ok() {
        let len = buf.len();

        if len > 0 {
            buf.truncate(len - 1);
        }
    }

    result
}

#[cfg(test)]
mod fastq_reader_tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::FastQReader;

    use Block;

    #[test]
    fn test_next() {
        let mut reader = FastQReader::<BufReader<File>>::open(
            "test/fixtures/r1.fastq",
        ).unwrap();

        {
            let actual = reader.next_block().unwrap().unwrap();
            let exepcted = Block::new("@fqlib:1/1", "AGCT", "+", "abcd");
            assert_eq!(actual, &exepcted);
        }

        {
            let actual = reader.next_block().unwrap().unwrap();
            let exepcted = Block::new("@fqlib:2/1", "TCGA", "+", "dcba");
            assert_eq!(actual, &exepcted);
        }
    }
}

pub struct PairedFastQReader {
    r1_reader: FastQReader<BufReader<File>>,
    r2_reader: FastQReader<BufReader<File>>,
}

impl PairedFastQReader {
    pub fn open<P, Q>(
        r1_pathname: P,
        r2_pathname: Q,
    ) -> io::Result<PairedFastQReader>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        Ok(PairedFastQReader {
            r1_reader: FastQReader::<BufReader<File>>::open(r1_pathname)?,
            r2_reader: FastQReader::<BufReader<File>>::open(r2_pathname)?,
        })
    }

    pub fn next_pair(&mut self) -> Option<(io::Result<&Block>, io::Result<&Block>)> {
        if let Some(r1_block) = self.r1_reader.next_block() {
            if let Some(r2_block) = self.r2_reader.next_block() {
                return Some((r1_block, r2_block));
            }
        }

        None
    }
}

#[cfg(test)]
mod paired_fastq_reader_tests {
    use super::PairedFastQReader;

    #[test]
    fn test_next() {
        let mut reader = PairedFastQReader::open(
            "test/fixtures/r1.fastq",
            "test/fixtures/r2.fastq",
        ).unwrap();;

        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_none());
    }
}
