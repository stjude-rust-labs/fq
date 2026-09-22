use std::{
    io::{self, Read, Write},
    path::Path,
};

use super::io::{Reader, Writer};
use crate::fs::{create_raw_or_gz, open_raw_or_gz};

pub fn create<P>(dst: P) -> io::Result<Writer<Box<dyn Write>>>
where
    P: AsRef<Path>,
{
    create_raw_or_gz(dst).map(Writer::new)
}

pub fn open<P>(src: P) -> io::Result<Reader<Box<dyn Read>>>
where
    P: AsRef<Path>,
{
    open_raw_or_gz(src).map(Reader::new)
}
