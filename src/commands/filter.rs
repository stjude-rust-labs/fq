use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use regex::bytes::Regex;
use thiserror::Error;
use tracing::info;

use crate::{cli::FilterArgs, fastq};

fn _filter<R, W, F>(
    readers: &mut [fastq::Reader<R>],
    writers: &mut [fastq::Writer<W>],
    filter: F,
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
    F: Fn(&fastq::Record) -> bool,
{
    let mut record = fastq::Record::default();
    let mut is_match = false;
    let mut is_eof = false;

    while !is_eof {
        for (i, (reader, writer)) in readers.iter_mut().zip(writers.iter_mut()).enumerate() {
            if i == 0 {
                if reader.read_record(&mut record)? == 0 {
                    is_eof = true;
                    break;
                }

                is_match = filter(&record);
            } else if reader.read_record(&mut record)? == 0 {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            if is_match {
                writer.write_record(&record)?;
            }
        }
    }

    Ok(())
}

fn copy_filtered<R, W>(
    readers: &mut [fastq::Reader<R>],
    names: &HashSet<Vec<u8>>,
    writers: &mut [fastq::Writer<W>],
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    _filter(readers, writers, |record| {
        let id = name_id(record.name());
        names.contains(id)
    })
}

fn read_names<R>(reader: R) -> io::Result<HashSet<Vec<u8>>>
where
    R: BufRead,
{
    reader
        .lines()
        .map(|res| res.map(|line| line.into_bytes()))
        .collect()
}

// Names always begin with an `@` character.
const ID_START_OFFSET: usize = 1;

fn name_id(name: &[u8]) -> &[u8] {
    let pos = name.iter().rev().position(|&b| b == b'/' || b == b' ');

    if let Some(i) = pos {
        let len = name.len();
        let end = len - i - 1;
        &name[ID_START_OFFSET..end]
    } else {
        &name[ID_START_OFFSET..]
    }
}

pub fn filter(args: FilterArgs) -> Result<(), FilterError> {
    let srcs = &args.srcs;
    let dsts = &args.dsts;

    info!(command = "filter", "fq");

    if let Some(names_src) = args.names.as_ref() {
        filter_by_names(srcs, dsts, names_src)?;
    } else if let Some(sequence_pattern) = args.sequence_pattern.as_ref() {
        filter_by_sequence_pattern(srcs, dsts, sequence_pattern)?;
    } else {
        cat(srcs, dsts)?;
    }

    info!("done");

    Ok(())
}

fn cat<P, Q>(srcs: &[P], dsts: &[Q]) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    for (src, dst) in srcs.iter().zip(dsts) {
        let mut reader = File::open(src)?;
        let mut writer = File::create(dst)?;
        io::copy(&mut reader, &mut writer)?;
    }

    Ok(())
}

fn filter_by_names<P, Q, R>(srcs: &[P], dsts: &[Q], names_src: R) -> Result<(), FilterError>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    info!("reading names");

    let names_src = names_src.as_ref();

    let reader = File::open(names_src)
        .map(BufReader::new)
        .map_err(|e| FilterError::OpenFile(e, names_src.into()))?;
    let names = read_names(reader).map_err(FilterError::ReadNames)?;

    info!("read {} names", names.len());
    info!("filtering fastq");

    let mut readers = build_readers(srcs)?;
    let mut writers = build_writers(dsts)?;

    copy_filtered(&mut readers, &names, &mut writers)?;

    Ok(())
}

fn copy_filtered_by_sequence_pattern<R, W>(
    readers: &mut [fastq::Reader<R>],
    sequence_pattern: &Regex,
    writers: &mut [fastq::Writer<W>],
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    _filter(readers, writers, |record| {
        sequence_pattern.is_match(record.sequence())
    })
}

fn filter_by_sequence_pattern<P, Q>(
    srcs: &[P],
    dsts: &[Q],
    sequence_pattern: &Regex,
) -> Result<(), FilterError>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut readers = build_readers(srcs)?;
    let mut writers = build_writers(dsts)?;

    info!("filtering fastq where sequence matches `{sequence_pattern}`");

    copy_filtered_by_sequence_pattern(&mut readers, sequence_pattern, &mut writers)?;

    Ok(())
}

fn build_readers<P>(srcs: &[P]) -> Result<Vec<fastq::Reader<Box<dyn BufRead>>>, FilterError>
where
    P: AsRef<Path>,
{
    srcs.iter()
        .map(|src| {
            let src = src.as_ref();
            fastq::open(src).map_err(|e| FilterError::OpenFile(e, src.into()))
        })
        .collect()
}

fn build_writers<P>(dsts: &[P]) -> Result<Vec<fastq::Writer<Box<dyn Write>>>, FilterError>
where
    P: AsRef<Path>,
{
    dsts.iter()
        .map(|dst| {
            let dst = dst.as_ref();
            fastq::create(dst).map_err(|e| FilterError::CreateFile(e, dst.into()))
        })
        .collect()
}

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("could not open file: {1}")]
    OpenFile(#[source] io::Error, PathBuf),
    #[error("could not create file: {1}")]
    CreateFile(#[source] io::Error, PathBuf),
    #[error("could not read read names")]
    ReadNames(#[source] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    static DATA: &[u8] = b"\
@fqlib:1/1\nAGCT\n+\nabcd
@fqlib:2/1\nTCGA\n+\ndcba
@fqlib:3/1\nGCCA\n+\ngcca
";

    #[test]
    fn test_copy_filtered() {
        let names = [b"fqlib:2".to_vec()].iter().cloned().collect();

        let reader = fastq::Reader::new(DATA);
        let mut readers = [reader];

        let mut buf = Vec::new();
        let writer = fastq::Writer::new(&mut buf);
        let mut writers = [writer];

        copy_filtered(&mut readers, &names, &mut writers).unwrap();

        let expected = b"@fqlib:2/1\nTCGA\n+\ndcba\n";
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_read_names() {
        let data = "@fqlib:1/1\n@fqlib:2/1\n@fqlib:3/1\n";

        let names = read_names(data.as_bytes()).unwrap();

        assert_eq!(names.len(), 3);
        assert!(names.contains("@fqlib:1/1".as_bytes()));
        assert!(names.contains("@fqlib:2/1".as_bytes()));
        assert!(names.contains("@fqlib:3/1".as_bytes()));
    }

    #[test]
    fn test_name_id() {
        assert_eq!(name_id("@fqlib:1/1".as_bytes()), b"fqlib:1");
        assert_eq!(name_id("@fqlib:1 1".as_bytes()), b"fqlib:1");
        assert_eq!(name_id("@fqlib:1".as_bytes()), b"fqlib:1");
    }

    #[test]
    fn test_copy_filtered_by_sequence_pattern() -> io::Result<()> {
        let reader = fastq::Reader::new(DATA);
        let mut readers = [reader];

        let pattern = Regex::new("^TC").unwrap();

        let writer = fastq::Writer::new(Vec::new());
        let mut writers = [writer];

        copy_filtered_by_sequence_pattern(&mut readers, &pattern, &mut writers)?;

        let expected = b"@fqlib:2/1\nTCGA\n+\ndcba\n";
        assert_eq!(writers[0].get_ref(), expected);

        Ok(())
    }
}
