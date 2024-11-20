use std::io;

use super::Metric;
use crate::fastq::Record;

const NAME: &str = "min_sequence_length";

pub struct MinSequenceLength {
    len: usize,
    initialized: bool,
}

impl Default for MinSequenceLength {
    fn default() -> Self {
        Self {
            len: usize::MAX,
            initialized: false,
        }
    }
}

impl Metric for MinSequenceLength {
    fn visit(&mut self, record: &Record) -> io::Result<()> {
        self.initialized = true;

        let read_length = record.sequence().len();
        self.len = self.len.min(read_length);

        Ok(())
    }

    fn println(&self) {
        println!("{NAME}\t{}", self.len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visit() -> io::Result<()> {
        let mut metric = MinSequenceLength::default();
        assert_eq!(metric.len, usize::MAX);
        assert!(!metric.initialized);

        let record = Record::new("", "ACGT", "", "");
        metric.visit(&record)?;
        assert_eq!(metric.len, 4);
        assert!(metric.initialized);

        Ok(())
    }
}
