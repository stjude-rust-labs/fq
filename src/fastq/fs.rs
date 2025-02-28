use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use flate2::{Compression, bufread::MultiGzDecoder, write::GzEncoder};

use super::io::{Reader, Writer};

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

pub fn open<P>(src: P) -> io::Result<Reader<Box<dyn BufRead>>>
where
    P: AsRef<Path>,
{
    let path = src.as_ref();
    let reader = File::open(path).map(BufReader::new)?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(GZ_EXTENSION) => {
            let decoder = MultiGzDecoder::new(reader);
            Ok(Reader::new(Box::new(BufReader::new(decoder))))
        }
        _ => Ok(Reader::new(Box::new(reader))),
    }
}
