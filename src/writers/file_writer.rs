use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

pub fn create<P>(pathname: P) -> io::Result<BufWriter<File>> where P: AsRef<Path> {
    let file = File::create(pathname)?;
    Ok(BufWriter::new(file))
}
