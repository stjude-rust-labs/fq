use std::{
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use tracing::info;

/// Byte offsets for every record in a FASTQ file.
///
/// Built by a single scan that counts newlines and records the stream
/// position at every 4th newline (the start of each record). Once built,
/// it provides O(1) record count, average record size, and offset lookup.
pub struct RecordIndex {
    offsets: Vec<u64>,
    total_bytes: u64,
}

impl RecordIndex {
    /// Build an index by scanning a seekable reader.
    pub fn build<R: BufRead + Seek>(reader: &mut R) -> io::Result<Self> {
        let total_bytes = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let mut offsets = Vec::new();
        let mut line_in_record: u8 = 0;
        let mut pos: u64 = 0;
        let mut first_byte = true;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }

            for &byte in buf {
                if first_byte {
                    offsets.push(0);
                    first_byte = false;
                }
                if byte == b'\n' {
                    line_in_record += 1;
                    if line_in_record == 4 {
                        line_in_record = 0;
                        let next_pos = pos + 1;
                        if next_pos < total_bytes {
                            offsets.push(next_pos);
                        }
                    }
                }
                pos += 1;
            }

            let len = buf.len();
            reader.consume(len);
        }

        // If the last "record" we pushed was just the offset past the end
        // of the last record's quality line (and there's no data there),
        // remove it. Also handle the case where the file doesn't end with
        // a newline and we have an incomplete record count.
        if line_in_record != 0 {
            // Incomplete final record — remove its start offset
            offsets.pop();
        }

        Ok(Self {
            offsets,
            total_bytes,
        })
    }

    /// Build from an uncompressed FASTQ file on disk.
    pub fn build_from_path(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        Self::build(&mut reader)
    }

    /// Load record count from a `.fai` sidecar file.
    ///
    /// Returns `Ok(Some(index))` if the `.fai` exists and can be read,
    /// `Ok(None)` if it doesn't exist. The index will only contain the
    /// record count (offsets are not populated from `.fai`).
    pub fn from_fai(src: &Path) -> io::Result<Option<usize>> {
        let mut index_path = src.as_os_str().to_owned();
        index_path.push(".fai");
        let index_path = PathBuf::from(index_path);

        if !index_path.exists() {
            return Ok(None);
        }

        info!(index = %index_path.display(), "found FASTQ index");

        let file = File::open(&index_path)?;
        let mut reader = BufReader::new(file);
        let mut n = 0usize;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            n += bytecount::count(buf, b'\n');
            let len = buf.len();
            reader.consume(len);
        }

        Ok(Some(n))
    }

    /// Number of complete records.
    pub fn record_count(&self) -> usize {
        self.offsets.len()
    }

    /// Average bytes per record (total file size / record count).
    pub fn avg_record_size(&self) -> f64 {
        if self.offsets.is_empty() {
            0.0
        } else {
            self.total_bytes as f64 / self.offsets.len() as f64
        }
    }

    /// Byte offset of the i-th record.
    pub fn offset(&self, i: usize) -> u64 {
        self.offsets[i]
    }

    /// Total file size in bytes.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_build_basic() -> io::Result<()> {
        let data = b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nGGGG\n@r3\nCCCC\n+\nHHHH\n";
        let mut cursor = Cursor::new(&data[..]);
        let index = RecordIndex::build(&mut cursor)?;

        assert_eq!(index.record_count(), 3);
        assert_eq!(index.offset(0), 0);
        assert_eq!(index.offset(1), 16);
        assert_eq!(index.offset(2), 32);
        assert!((index.avg_record_size() - 16.0).abs() < 0.01);

        Ok(())
    }

    #[test]
    fn test_build_empty() -> io::Result<()> {
        let data = b"";
        let mut cursor = Cursor::new(&data[..]);
        let index = RecordIndex::build(&mut cursor)?;

        assert_eq!(index.record_count(), 0);
        assert_eq!(index.avg_record_size(), 0.0);

        Ok(())
    }

    #[test]
    fn test_build_single_record() -> io::Result<()> {
        let data = b"@r1\nACGT\n+\nFFFF\n";
        let mut cursor = Cursor::new(&data[..]);
        let index = RecordIndex::build(&mut cursor)?;

        assert_eq!(index.record_count(), 1);
        assert_eq!(index.offset(0), 0);

        Ok(())
    }

    #[test]
    fn test_build_variable_length() -> io::Result<()> {
        // Records with different sequence lengths
        let data = b"@r1\nAC\n+\nFF\n@r2\nACGTACGT\n+\nFFFFFFFF\n";
        let mut cursor = Cursor::new(&data[..]);
        let index = RecordIndex::build(&mut cursor)?;

        assert_eq!(index.record_count(), 2);
        assert_eq!(index.offset(0), 0);
        // @r1\nAC\n+\nFF\n = 4+3+2+3 = 12 bytes
        assert_eq!(index.offset(1), 12);

        Ok(())
    }

    #[test]
    fn test_build_from_path() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_record_index");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("test.fq");
        std::fs::write(&path, b"@r1\nACGT\n+\nFFFF\n@r2\nTGCA\n+\nGGGG\n")?;

        let index = RecordIndex::build_from_path(&path)?;
        assert_eq!(index.record_count(), 2);
        assert_eq!(index.offset(0), 0);
        assert_eq!(index.offset(1), 16);

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_from_fai_found() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_fai_found");
        std::fs::create_dir_all(&dir)?;
        let fq = dir.join("test.fq");
        let fai = dir.join("test.fq.fai");
        std::fs::write(&fq, b"@r1\nACGT\n+\nFFFF\n")?;
        std::fs::write(&fai, b"r1\t4\t4\t4\t5\t14\nr2\t4\t24\t4\t5\t34\n")?;

        let count = RecordIndex::from_fai(&fq)?;
        assert_eq!(count, Some(2));

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_from_fai_missing() -> io::Result<()> {
        let dir = std::env::temp_dir().join("fq_test_fai_missing");
        std::fs::create_dir_all(&dir)?;
        let fq = dir.join("test.fq");
        std::fs::write(&fq, b"@r1\nACGT\n+\nFFFF\n")?;

        let count = RecordIndex::from_fai(&fq)?;
        assert_eq!(count, None);

        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
