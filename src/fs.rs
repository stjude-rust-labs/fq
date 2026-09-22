use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use flate2::read::MultiGzDecoder;

const GZ_EXTENSION: &str = "gz";

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
