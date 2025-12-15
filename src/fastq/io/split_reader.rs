use std::io::{self, Read};

use super::Reader;
use crate::fastq::Record;

pub struct SplitReader<R> {
    inners: [Reader<R>; 2],
}

impl<R> SplitReader<R>
where
    R: Read,
{
    pub fn new(inners: [Reader<R>; 2]) -> Self {
        Self { inners }
    }

    pub fn read_records(&mut self, records: &mut [Record; 2]) -> io::Result<[usize; 2]> {
        let mut lens = [0; 2];

        for ((inner, record), len) in self.inners.iter_mut().zip(records).zip(&mut lens) {
            *len = inner.read_record(record)?;
        }

        Ok(lens)
    }
}
