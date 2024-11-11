use std::io;

use crate::{
    cli::DescribeArgs,
    fastq::{self, Record},
};

pub fn describe(args: DescribeArgs) -> io::Result<()> {
    let mut reader = fastq::open(args.src)?;
    let mut record = Record::default();

    let mut metrics = Metrics::default();

    while reader.read_record(&mut record)? != 0 {
        visit(&mut metrics, &record);
    }

    println!("record_count\t{}", metrics.record_count);

    Ok(())
}

#[derive(Default)]
struct Metrics {
    record_count: u64,
}

fn visit(metrics: &mut Metrics, _: &Record) {
    metrics.record_count += 1;
}
