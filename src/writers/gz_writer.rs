use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;

pub fn create<P>(
    pathname: P,
) -> io::Result<GzEncoder<BufWriter<File>>>
where
    P: AsRef<Path>,
{
    let file = File::create(pathname)?;
    let writer = BufWriter::new(file);
    let level = Compression::default();
    Ok(GzEncoder::new(writer, level))
}
