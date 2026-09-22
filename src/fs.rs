use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
    path::Path,
};

use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};

const GZ_EXTENSION: &str = "gz";

pub fn create_raw_or_gz<P>(dst: P) -> io::Result<Box<dyn Write>>
where
    P: AsRef<Path>,
{
    let path = dst.as_ref();
    let writer = File::create(path).map(BufWriter::new)?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(GZ_EXTENSION) => {
            let level = Compression::default();
            let encoder = GzEncoder::new(writer, level);
            Ok(Box::new(encoder))
        }
        _ => Ok(Box::new(writer)),
    }
}

pub fn open_raw_or_gz<P>(src: P) -> io::Result<Box<dyn Read>>
where
    P: AsRef<Path>,
{
    let src = src.as_ref();
    let reader = File::open(src)?;

    match src.extension().and_then(|ext| ext.to_str()) {
        Some(GZ_EXTENSION) => {
            let decoder = MultiGzDecoder::new(reader);
            Ok(Box::new(decoder))
        }
        _ => Ok(Box::new(reader)),
    }
}
