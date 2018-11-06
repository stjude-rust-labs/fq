use std::io::BufRead;

use noodles::formats::fastq::{self, Record};

use block;

pub struct PairReader<R: BufRead, S: BufRead> {
    reader_1: fastq::Reader<R>,
    reader_2: fastq::Reader<S>,
    record_1: Record,
    record_2: Record,
}

impl<R, S> PairReader<R, S> where R: BufRead, S: BufRead {
    pub fn new(reader_1: fastq::Reader<R>, reader_2: fastq::Reader<S>) -> PairReader<R, S> {
        PairReader {
            reader_1,
            reader_2,
            record_1: Record::default(),
            record_2: Record::default(),
        }
    }

    pub fn next_pair(&mut self) -> Option<(&Record, &Record)> {
        let bytes_read = self.reader_1.read_record(&mut self.record_1).ok()?;

        if bytes_read == 0 {
            return None;
        }

        let bytes_read = self.reader_2.read_record(&mut self.record_2).ok()?;

        if bytes_read == 0 {
            return None;
        }

        block::reset(&mut self.record_1);
        block::reset(&mut self.record_2);

        Some((&self.record_1, &self.record_2))
    }
}

#[cfg(test)]
mod tests {
    use noodles::formats::fastq;

    use super::PairReader;

    #[test]
    fn test_next_pair() {
        let data = "\
@fqlib:1/1
AGCT
+
abcd
@fqlib:2/1
TCGA
+
dcba
";

        let reader_1 = fastq::Reader::new(data.as_bytes());
        let reader_2 = fastq::Reader::new(data.as_bytes());

        let mut reader = PairReader::new(reader_1, reader_2);

        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_some());
        assert!(reader.next_pair().is_none());
    }
}
