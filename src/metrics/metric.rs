use std::io;

use crate::fastq::Record;

pub trait Metric {
    fn visit(&mut self, record: &Record) -> io::Result<()>;
    fn println(&self);
}
