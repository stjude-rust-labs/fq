use std::io;

use super::Metric;
use crate::fastq::Record;

const NAME: &str = "max_sequence_length";

#[derive(Default)]
pub struct MaxSequenceLength(usize);

impl Metric for MaxSequenceLength {
    fn visit(&mut self, record: &Record) -> io::Result<()> {
        let read_length = record.sequence().len();
        self.0 = self.0.max(read_length);
        Ok(())
    }

    fn println(&self) {
        println!("{NAME}\t{}", self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visit() -> io::Result<()> {
        let mut metric = MaxSequenceLength::default();
        assert_eq!(metric.0, 0);

        let record = Record::new("", "ACGT", "", "");
        metric.visit(&record)?;
        assert_eq!(metric.0, 4);

        Ok(())
    }
}
