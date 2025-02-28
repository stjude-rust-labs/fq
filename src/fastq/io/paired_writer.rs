use std::io::{self, Write};

use rand::Rng;

use crate::{
    Generator,
    fastq::{self, Record},
};

static PLUS_LINE: &[u8] = b"+";

pub struct PairedWriter<W: Write, X: Write> {
    writer_1: fastq::io::Writer<W>,
    writer_2: fastq::io::Writer<X>,
}

impl<W, X> PairedWriter<W, X>
where
    W: Write,
    X: Write,
{
    pub fn new(writer_1: fastq::io::Writer<W>, writer_2: fastq::io::Writer<X>) -> Self {
        Self { writer_1, writer_2 }
    }

    pub fn write<R>(&mut self, mut generator: Generator<R>, record_count: u64) -> io::Result<()>
    where
        R: Rng,
    {
        let mut r = Record::default();
        let mut s = Record::default();

        r.plus_line_mut().extend_from_slice(PLUS_LINE);
        s.plus_line_mut().extend_from_slice(PLUS_LINE);

        for _ in 0..record_count {
            generator.next_record(&mut r);
            generator.next_record_with_name(r.name(), &mut s);

            r.name_mut().extend_from_slice(b"/1");
            s.name_mut().extend_from_slice(b"/2");

            self.writer_1.write_record(&r)?;
            self.writer_2.write_record(&s)?;
        }

        Ok(())
    }
}
