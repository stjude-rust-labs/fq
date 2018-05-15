use std::io::{self, BufRead, BufReader, Lines};
use std::fs::File;
use std::path::Path;

use Block;

pub struct FastQReader<R: BufRead> {
    lines: Lines<R>,
    line_no: usize,
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
            lines: reader.lines(),
            line_no: 0,
        }
    }

    pub fn next_block(&mut self) -> Option<io::Result<Block>> {
        if let Some(name) = self.next_line() {
            if let Some(sequence) = self.next_line() {
                if let Some(plus_line) = self.next_line() {
                    if let Some(quality) = self.next_line() {
                        return Some(
                            Ok(Block::new(
                                name.unwrap(),
                                sequence.unwrap(),
                                plus_line.unwrap(),
                                quality.unwrap(),
                            ))
                        )
                    }
                }
            }

            let message = format!("unexpected EOF (line {})", self.line_no);
            return Some(Err(io::Error::new(io::ErrorKind::UnexpectedEof, message)));
        }

        None
    }

    fn next_line(&mut self) -> Option<io::Result<String>> {
        self.line_no += 1;
        self.lines.next()
    }
}

impl<R: BufRead> Iterator for FastQReader<R> {
    type Item = io::Result<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_block()
    }
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

        let actual = reader.next().unwrap().unwrap();
        let exepcted = Block::new("@fqlib:1/1", "AGCT", "+", "abcd");
        assert_eq!(actual, exepcted);

        let actual = reader.next().unwrap().unwrap();
        let exepcted = Block::new("@fqlib:2/1", "TCGA", "+", "dcba");
        assert_eq!(actual, exepcted);
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

    pub fn next_pair(&mut self) -> Option<(io::Result<Block>, io::Result<Block>)> {
        if let Some(r1_block) = self.r1_reader.next_block() {
            if let Some(r2_block) = self.r2_reader.next_block() {
                return Some((r1_block, r2_block));
            }
        }

        None
    }
}

impl Iterator for PairedFastQReader {
    type Item = (io::Result<Block>, io::Result<Block>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_pair()
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

        assert!(reader.next().is_some());
        assert!(reader.next().is_some());
        assert!(reader.next().is_none());
    }
}
