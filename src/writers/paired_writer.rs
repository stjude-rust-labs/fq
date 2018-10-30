use std::io::{self, Write};

use {Block, BlockPairGenerator};

pub struct PairedWriter<W: Write> {
    writer_1: W,
    writer_2: W,
}

impl<W: Write> PairedWriter<W> {
    pub fn new(writer_1: W, writer_2: W) -> PairedWriter<W> {
        PairedWriter { writer_1, writer_2 }
    }

    pub fn write(
        &mut self,
        mut generator: BlockPairGenerator,
        iterations: i32,
    ) -> io::Result<()> {
        for _ in 0..iterations {
            let (b, d) = generator.next_block_pair();
            write_block(&mut self.writer_1, b, "1")?;
            write_block(&mut self.writer_2, d, "2")?;
        }

        Ok(())
    }
}

fn write_block<W>(
    writer: &mut W,
    block: &Block,
    interleave: &str,
) -> io::Result<()>
where
    W: Write
{
    writer.write_all(block.name())?;
    writer.write_all(b"/")?;
    writer.write_all(interleave.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.write_all(block.sequence())?;
    writer.write_all(b"\n")?;
    writer.write_all(block.plus_line())?;
    writer.write_all(b"\n")?;
    writer.write_all(block.quality())?;
    writer.write_all(b"\n")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use Block;
    use super::write_block;

    #[test]
    fn test_write_block() {
        let block = Block::new("@fqlib", "AGCT", "+", "abcd");
        let mut buf = Vec::new();
        write_block(&mut buf, &block, "2").unwrap();
        let data = String::from_utf8(buf).unwrap();
        assert_eq!(data, "@fqlib/2\nAGCT\n+\nabcd\n");
    }
}
