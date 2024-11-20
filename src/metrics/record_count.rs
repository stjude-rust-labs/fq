use std::io;

use super::Metric;
use crate::fastq::Record;

const NAME: &str = "record_count";

#[derive(Default)]
pub struct RecordCount(u64);

impl Metric for RecordCount {
    fn visit(&mut self, _: &Record) -> io::Result<()> {
        self.0 += 1;
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
        let mut metric = RecordCount::default();
        assert_eq!(metric.0, 0);

        let record = Record::default();
        metric.visit(&record)?;
        assert_eq!(metric.0, 1);

        Ok(())
    }
}
