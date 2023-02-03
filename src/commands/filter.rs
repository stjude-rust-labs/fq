use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use anyhow::Context;
use tracing::info;

use crate::{cli::FilterArgs, fastq};

fn copy_filtered<R, W>(
    mut reader: fastq::Reader<R>,
    names: &HashSet<Vec<u8>>,
    mut writer: fastq::Writer<W>,
) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    let mut record = fastq::Record::default();

    loop {
        let bytes_read = reader.read_record(&mut record)?;

        if bytes_read == 0 {
            break;
        }

        let id = name_id(record.name());

        if names.contains(id) {
            writer.write_record(&record)?;
        }
    }

    Ok(())
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

pub fn filter(args: FilterArgs) -> anyhow::Result<()> {
    let src = &args.src;

    info!("fq-filter start");

    if let Some(names_src) = args.names {
        filter_by_names(src, names_src)?;
    } else {
        cat(src)?;
    }

    info!("fq-filter end");

    Ok(())
}

fn cat<P>(src: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut reader = File::open(src)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    io::copy(&mut reader, &mut handle)?;

    Ok(())
}

fn filter_by_names<P, Q>(src: P, names_src: Q) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    info!("reading names");

    let src = src.as_ref();
    let names_src = names_src.as_ref();

    let file = File::open(names_src)
        .with_context(|| format!("Could not open file: {}", names_src.display()))?;

    let reader = BufReader::new(file);

    let names = read_names(reader)
        .with_context(|| format!("Could not read file: {}", names_src.display()))?;

    info!("read {} names", names.len());

    let stdout = io::stdout();
    let handle = stdout.lock();
    let buf = BufWriter::new(handle);
    let writer = fastq::Writer::new(buf);

    info!("filtering fastq");

    let reader = crate::fastq::open(src)
        .with_context(|| format!("Could not open file: {}", src.display()))?;

    copy_filtered(reader, &names, writer)
        .with_context(|| format!("Could not copy record from {} to stdout", src.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_filtered() {
        let names = [b"fqlib:2".to_vec()].iter().cloned().collect();

        let data = "\
@fqlib:1/1\nAGCT\n+\nabcd
@fqlib:2/1\nTCGA\n+\ndcba
@fqlib:3/1\nGCCA\n+\ngcca
";

        let reader = fastq::Reader::new(data.as_bytes());

        let mut buf = Vec::new();
        let writer = fastq::Writer::new(&mut buf);

        copy_filtered(reader, &names, writer).unwrap();

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
}
