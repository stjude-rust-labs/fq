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

        for _ in 0..record_count {
            generator.next_record(&mut r);
            generator.next_record_with_name(r.name(), &mut s);

            write_record(self.writer_1.get_mut(), &r, b"/1")?;
            write_record(self.writer_2.get_mut(), &s, b"/2")?;
        }

        Ok(())
    }
}

fn write_record<W>(writer: &mut W, record: &Record, name_ext: &[u8]) -> io::Result<()>
where
    W: Write,
{
    writer.write_all(record.name())?;
    writer.write_all(name_ext)?;
    writer.write_all(b"\n")?;
    writer.write_all(record.sequence())?;
    writer.write_all(b"\n")?;
    writer.write_all(PLUS_LINE)?;
    writer.write_all(b"\n")?;
    writer.write_all(record.quality_scores())?;
    writer.write_all(b"\n")?;

    Ok(())
}
