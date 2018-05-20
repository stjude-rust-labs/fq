use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use Generator;

pub struct Writer<W: Write> {
    r1_writer: W,
    r2_writer: W,
}

impl<W: Write> Writer<W> {
    pub fn create<P, Q>(
        r1_pathname: P,
        r2_pathname: Q,
    ) -> io::Result<Writer<BufWriter<File>>>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let f1 = File::create(r1_pathname)?;
        let f2 = File::create(r2_pathname)?;
        Ok(Writer::new(BufWriter::new(f1), BufWriter::new(f2)))
    }

    pub fn new(r1_writer: W, r2_writer: W) -> Writer<W> {
        Writer { r1_writer, r2_writer }
    }

    pub fn write(&mut self, generator: Generator, iterations: i32) -> io::Result<()> {
        let mut writers = [&mut self.r1_writer, &mut self.r2_writer];

        for block in generator.take(iterations as usize) {
            for (i, writer) in writers.iter_mut().enumerate() {
                let interleave = format!("/{}", i + 1);

                writer.write_all(block.name.as_bytes())?;
                writer.write_all(interleave.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.write_all(block.sequence.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.write_all(block.plus_line.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.write_all(block.quality.as_bytes())?;
                writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }

    pub fn into_inner(self) -> (W, W) {
        (self.r1_writer, self.r2_writer)
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;

    use Generator;

    static SEED: [u8; 16] = [
        0x28, 0x8f, 0x28, 0x22, 0x5e, 0x8b, 0x18, 0x03,
        0x8a, 0x08, 0x9a, 0x77, 0x1d, 0x8f, 0x0b, 0x44,
    ];

    #[test]
    fn test_write() {
        let generator = Generator::from_seed(SEED);
        let mut writer = Writer::new(Vec::new(), Vec::new());
        writer.write(generator, 2).unwrap();

        let (w1, w2) = writer.into_inner();
        let r1 = String::from_utf8(w1).unwrap();
        let r2 = String::from_utf8(w2).unwrap();

        assert_eq!(r1, "@fqlib2:898:JSLNGVS:1:32:8896:8166/1
TAGTGCTGGGACATTTGGAGCAGCAGCTAAGAAAGGGGAGAGTGACACTCTTAGGGAATTACAGTTGTCACAGTCGGCCAATAGCCGTGTGGGATCCTGTG
+
FACCAHDCJJGFJ@GEF@GDC@DC@F@AJGH@JCDEAIGAB@CB@DD@ECCJJFHDBAEGJGHHGJHIGFFIBCEGFFA@IBBADIJDEFD@H@E@HAJDC
@fqlib2:898:JSLNGVS:3:39:4030:8886/1
TTTGCGGTCACGGAGGCCGTTCAACGTGCACGCATCTATTCATGAATGCGCCGGGGGGAGCCATCGCCTGACAGCTTTGAGTGCGTATCAGAAAGTTAGTA
+
DEJHABIHG@DAEGAFFJEG@AGBHHDHHCFBEJBH@GAEBJGDADE@JDDECABFHFADECFGFGA@IBCFADH@EBCFEGFAFJDHEECE@DJGDGIGF
");

        assert_eq!(r2, "@fqlib2:898:JSLNGVS:1:32:8896:8166/2
TAGTGCTGGGACATTTGGAGCAGCAGCTAAGAAAGGGGAGAGTGACACTCTTAGGGAATTACAGTTGTCACAGTCGGCCAATAGCCGTGTGGGATCCTGTG
+
FACCAHDCJJGFJ@GEF@GDC@DC@F@AJGH@JCDEAIGAB@CB@DD@ECCJJFHDBAEGJGHHGJHIGFFIBCEGFFA@IBBADIJDEFD@H@E@HAJDC
@fqlib2:898:JSLNGVS:3:39:4030:8886/2
TTTGCGGTCACGGAGGCCGTTCAACGTGCACGCATCTATTCATGAATGCGCCGGGGGGAGCCATCGCCTGACAGCTTTGAGTGCGTATCAGAAAGTTAGTA
+
DEJHABIHG@DAEGAFFJEG@AGBHHDHHCFBEJBH@GAEBJGDADE@JDDECABFHFADECFGFGA@IBCFADH@EBCFEGFAFJDHEECE@DJGDGIGF
");
    }
}
