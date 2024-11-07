use std::io;

use crate::{
    cli::DescribeArgs,
    fastq::{self, Record},
};

pub fn describe(args: DescribeArgs) -> io::Result<()> {
    let mut record_count = 0;

    let mut reader = fastq::open(args.src)?;
    let mut record = Record::default();

    while reader.read_record(&mut record)? != 0 {
        record_count += 1;
    }

    println!("record_count\t{record_count}");

    Ok(())
}
