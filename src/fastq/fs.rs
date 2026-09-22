use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
    path::Path,
};

use flate2::{Compression, write::GzEncoder};

use super::io::{Reader, Writer};
use crate::fs::open_raw_or_gz;

const GZ_EXTENSION: &str = "gz";

pub fn create<P>(dst: P) -> io::Result<Writer<Box<dyn Write>>>
where
    P: AsRef<Path>,
{
    let path = dst.as_ref();
    let writer = File::create(path).map(BufWriter::new)?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(GZ_EXTENSION) => {
            let level = Compression::default();
            let encoder = GzEncoder::new(writer, level);
            Ok(Writer::new(Box::new(encoder)))
        }
        _ => Ok(Writer::new(Box::new(writer))),
    }
}

pub fn open<P>(src: P) -> io::Result<Reader<Box<dyn Read>>>
where
    P: AsRef<Path>,
{
    open_raw_or_gz(src).map(Reader::new)
}
