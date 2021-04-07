use std::io::{self, BufRead};

use super::Record;

const LINE_FEED: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';

pub struct Reader<R>
where
    R: BufRead,
{
    inner: R,
}

impl<R> Reader<R>
where
    R: BufRead,
{
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    pub fn read_record(&mut self, record: &mut Record) -> io::Result<usize> {
        record.clear();

        let mut len = match read_line(&mut self.inner, record.name_mut()) {
            Ok(0) => return Ok(0),
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        len += read_line(&mut self.inner, record.sequence_mut())?;
        len += read_line(&mut self.inner, record.plus_line_mut())?;
        len += read_line(&mut self.inner, record.quality_scores_mut())?;

        Ok(len)
    }
}

fn read_line<R: BufRead>(reader: &mut R, buf: &mut Vec<u8>) -> io::Result<usize> {
    match reader.read_until(LINE_FEED, buf) {
        Ok(0) => Ok(0),
        Ok(n) => {
            if buf.ends_with(&[LINE_FEED]) {
                buf.pop();

                if buf.ends_with(&[CARRIAGE_RETURN]) {
                    buf.pop();
                }
            }

            Ok(n)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_record() -> io::Result<()> {
        let data = b"\
@fqlib:1/1
ACGT
+
FQLB
";

        let mut reader = Reader::new(&data[..]);
        let mut record = Record::default();

        reader.read_record(&mut record)?;
        assert_eq!(record.name(), b"@fqlib:1/1");
        assert_eq!(record.sequence(), b"ACGT");
        assert_eq!(record.plus_line(), b"+");
        assert_eq!(record.quality_scores(), b"FQLB");

        assert_eq!(reader.read_record(&mut record)?, 0);

        Ok(())
    }

    #[test]
    fn test_read_line() -> io::Result<()> {
        let mut buf = Vec::new();

        let data = b"@fqlib\n";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, b"@fqlib");

        let data = b"@fqlib\r\n";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, b"@fqlib");

        let data = b"@fqlib";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, b"@fqlib");

        Ok(())
    }
}
