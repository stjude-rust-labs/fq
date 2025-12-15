use std::io::{self, Read};

use bytes::BytesMut;
use memchr::memchr_iter;

use crate::fastq::Record;

const DEFAULT_BUF_SIZE: usize = 1024 * 128;

pub struct Reader<R>
where
    R: Read,
{
    inner: R,
    buf: BytesMut,
}

impl<R> Reader<R>
where
    R: Read,
{
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: BytesMut::with_capacity(DEFAULT_BUF_SIZE),
        }
    }

    pub fn read_record(&mut self, record: &mut Record) -> io::Result<usize> {
        match read_frame(&mut self.inner, &mut self.buf)? {
            Some(frame) => {
                *record = frame;
                Ok(record.as_ref().len())
            }
            None => Ok(0),
        }
    }
}

fn read_frame<R>(reader: &mut R, buf: &mut BytesMut) -> io::Result<Option<Record>>
where
    R: Read,
{
    loop {
        if let Some(frame) = decode(buf)? {
            return Ok(Some(frame));
        } else if read_buf(reader, buf)? == 0 {
            if let Some(frame) = decode_eof(buf)? {
                return Ok(Some(frame));
            } else {
                return Ok(None);
            }
        }
    }
}

fn decode(buf: &mut BytesMut) -> io::Result<Option<Record>> {
    const LINES_PER_RECORD: usize = 4;
    const LINE_FEED: u8 = b'\n';

    let iter = memchr_iter(LINE_FEED, &buf[..]);
    let mut ends = [0; 4];
    let mut len = 0;

    for (end, i) in ends.iter_mut().zip(iter) {
        *end = i + 1;
        len += 1;
    }

    if len == LINES_PER_RECORD {
        Ok(Some(Record {
            buf: buf.split_to(ends[3]).freeze(),
            definition_end: ends[0],
            name_end: ends[0],
            sequence_end: ends[1],
            plus_line_end: ends[2],
        }))
    } else {
        Ok(None)
    }
}

fn decode_eof(buf: &mut BytesMut) -> io::Result<Option<Record>> {
    match decode(buf)? {
        Some(frame) => Ok(Some(frame)),
        None if buf.is_empty() => Ok(None),
        None => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
    }
}

fn read_buf<R>(reader: &mut R, buf: &mut BytesMut) -> io::Result<usize>
where
    R: Read,
{
    let len = buf.len();
    buf.resize(len + DEFAULT_BUF_SIZE, 0);
    let n = reader.read(&mut buf[len..])?;
    buf.truncate(len + n);
    Ok(n)
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
}
