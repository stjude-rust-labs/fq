use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use bytes::BytesMut;
use memchr::memchr_iter;

use crate::fastq::Record;

use super::record_index::RecordIndex;

/// A FASTQ reader backed by a [`RecordIndex`] for O(1) random access.
///
/// Wraps a seekable reader and an index of record byte offsets. Supports
/// sequential reading, random access by record index, and skip-ahead
/// iteration with geometric jumps.
pub struct IndexedReader<R> {
    inner: BufReader<R>,
    index: RecordIndex,
    buf: BytesMut,
}

const DEFAULT_BUF_SIZE: usize = 1024 * 128;

impl IndexedReader<File> {
    /// Open an uncompressed FASTQ file and build its index.
    pub fn open(path: &Path) -> io::Result<Self> {
        let index = RecordIndex::build_from_path(path)?;
        let file = File::open(path)?;
        Ok(Self {
            inner: BufReader::new(file),
            index,
            buf: BytesMut::with_capacity(DEFAULT_BUF_SIZE),
        })
    }
}

impl<R: Read + Seek> IndexedReader<R> {
    /// Create from an existing reader and pre-built index.
    pub fn new(inner: R, index: RecordIndex) -> Self {
        Self {
            inner: BufReader::new(inner),
            index,
            buf: BytesMut::with_capacity(DEFAULT_BUF_SIZE),
        }
    }

    pub fn index(&self) -> &RecordIndex {
        &self.index
    }

    /// Read the record at position `i` in the file.
    pub fn read_record_at(&mut self, i: usize, record: &mut Record) -> io::Result<usize> {
        let offset = self.index.offset(i);
        self.inner.seek(SeekFrom::Start(offset))?;
        self.buf.clear();

        // Determine how many bytes to read for this record
        let end = if i + 1 < self.index.record_count() {
            self.index.offset(i + 1)
        } else {
            self.index.total_bytes()
        };
        let record_len = (end - offset) as usize;

        // Read exactly this record's bytes
        self.buf.resize(record_len, 0);
        self.inner.read_exact(&mut self.buf)?;

        // Decode the record from the buffer
        match decode_record(&self.buf) {
            Some(r) => {
                let len = r.as_ref().len();
                *record = r;
                Ok(len)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to decode record at index {i} (offset {offset})"),
            )),
        }
    }

    /// Read selected records by index. Indices must be sorted ascending.
    pub fn read_records_at(
        &mut self,
        indices: &[usize],
        mut callback: impl FnMut(&Record) -> io::Result<()>,
    ) -> io::Result<usize> {
        let mut record = Record::default();
        let mut count = 0;

        for &i in indices {
            self.read_record_at(i, &mut record)?;
            callback(&record)?;
            count += 1;
        }

        Ok(count)
    }

    /// Skip-ahead sampling: yield approximately `target` records using
    /// exponential jumps through the index. Since jumps are over record
    /// indices (not byte positions), no boundary detection is needed.
    pub fn skip_ahead_sample<Rng: rand::Rng>(
        &mut self,
        target: usize,
        rng: &mut Rng,
        mut callback: impl FnMut(&Record) -> io::Result<()>,
    ) -> io::Result<usize> {
        let total = self.index.record_count();
        if total == 0 || target == 0 {
            return Ok(0);
        }

        let mean_jump = total as f64 / target as f64;
        let mut record = Record::default();
        let mut count = 0;

        // Draw the first jump to avoid always starting at record 0
        let first = (-mean_jump * rng.random::<f64>().ln()).ceil() as usize;
        let mut current: usize = first.min(total.saturating_sub(1));

        while count < target && current < total {
            self.read_record_at(current, &mut record)?;
            callback(&record)?;
            count += 1;

            // Geometric jump in record space (at least 1)
            let jump = (-mean_jump * rng.random::<f64>().ln()).ceil() as usize;
            current += jump.max(1);
        }

        Ok(count)
    }
}

/// Decode a single FASTQ record from a byte buffer.
fn decode_record(buf: &[u8]) -> Option<Record> {
    let mut ends = [0usize; 4];
    let mut len = 0;

    for (end, i) in ends.iter_mut().zip(memchr_iter(b'\n', buf)) {
        *end = i + 1;
        len += 1;
    }

    if len == 4 {
        Some(Record {
            buf: bytes::Bytes::copy_from_slice(&buf[..ends[3]]),
            definition_end: ends[0],
            name_end: ends[0],
            sequence_end: ends[1],
            plus_line_end: ends[2],
        })
    } else if len == 3 && !buf.is_empty() {
        // Last record without trailing newline
        Some(Record {
            buf: bytes::Bytes::copy_from_slice(buf),
            definition_end: ends[0],
            name_end: ends[0],
            sequence_end: ends[1],
            plus_line_end: ends[2],
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::SmallRng};
    use std::io::Cursor;

    fn test_data() -> &'static [u8] {
        b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nGGGG\n@r3\nCCCC\n+\nHHHH\n@r4\nAAAA\n+\nIIII\n@r5\nTTTT\n+\nJJJJ\n"
    }

    fn make_reader(data: &[u8]) -> io::Result<IndexedReader<Cursor<Vec<u8>>>> {
        let mut cursor = Cursor::new(data.to_vec());
        let index = RecordIndex::build(&mut cursor)?;
        cursor.seek(SeekFrom::Start(0))?;
        Ok(IndexedReader::new(cursor, index))
    }

    #[test]
    fn test_read_record_at() -> io::Result<()> {
        let mut reader = make_reader(test_data())?;
        let mut record = Record::default();

        reader.read_record_at(0, &mut record)?;
        assert_eq!(record.name(), b"@r1");
        assert_eq!(record.sequence(), b"ACGT");

        reader.read_record_at(2, &mut record)?;
        assert_eq!(record.name(), b"@r3");
        assert_eq!(record.sequence(), b"CCCC");

        reader.read_record_at(4, &mut record)?;
        assert_eq!(record.name(), b"@r5");
        assert_eq!(record.sequence(), b"TTTT");

        Ok(())
    }

    #[test]
    fn test_read_records_at() -> io::Result<()> {
        let mut reader = make_reader(test_data())?;
        let mut names = Vec::new();

        reader.read_records_at(&[1, 3], |record| {
            names.push(record.name().to_vec());
            Ok(())
        })?;

        assert_eq!(names.len(), 2);
        assert_eq!(names[0], b"@r2");
        assert_eq!(names[1], b"@r4");

        Ok(())
    }

    #[test]
    fn test_skip_ahead_sample() -> io::Result<()> {
        let mut reader = make_reader(test_data())?;
        let mut rng = SmallRng::seed_from_u64(42);
        let mut names = Vec::new();

        let count = reader.skip_ahead_sample(3, &mut rng, |record| {
            names.push(record.name().to_vec());
            Ok(())
        })?;

        assert_eq!(count, names.len());
        // Should get approximately 3 records from 5
        assert!(count >= 2 && count <= 4, "expected ~3, got {count}");

        Ok(())
    }

    #[test]
    fn test_open_file() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_indexed_reader");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("test.fq");
        std::fs::write(&path, test_data())?;

        let mut reader = IndexedReader::open(&path)?;
        assert_eq!(reader.index().record_count(), 5);

        let mut record = Record::default();
        reader.read_record_at(3, &mut record)?;
        assert_eq!(record.name(), b"@r4");

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
