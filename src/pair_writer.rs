use std::io::{self, Write};

use noodles::formats::fastq::{self, Record};

use Generator;

static PLUS_LINE: &[u8] = b"+";

pub struct PairWriter<W: Write, X: Write> {
    writer_1: fastq::Writer<W>,
    writer_2: fastq::Writer<X>,
}

impl<W, X> PairWriter<W, X> where W: Write, X: Write {
    pub fn new(
        writer_1: fastq::Writer<W>,
        writer_2: fastq::Writer<X>,
    ) -> PairWriter<W, X> {
        PairWriter { writer_1, writer_2 }
    }

    pub fn write(
        &mut self,
        mut generator: Generator,
        iterations: i32,
    ) -> io::Result<()> {
        let mut r = Record::default();
        let mut s = Record::default();

        r.plus_line_mut().extend_from_slice(PLUS_LINE);
        s.plus_line_mut().extend_from_slice(PLUS_LINE);

        for _ in 0..iterations {
            generator.next_block(&mut r);
            generator.next_block_with_name(r.name(), &mut s);

            r.name_mut().extend_from_slice(b"/1");
            s.name_mut().extend_from_slice(b"/2");

            self.writer_1.write_record(&r)?;
            self.writer_2.write_record(&s)?;
        }

        Ok(())
    }
}
