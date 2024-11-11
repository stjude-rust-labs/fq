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
    println!("min_sequence_length\t{}", metrics.min_sequence_length);
    println!("max_sequence_length\t{}", metrics.max_sequence_length);

    Ok(())
}

struct Metrics {
    record_count: u64,
    min_sequence_length: usize,
    max_sequence_length: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            record_count: 0,
            min_sequence_length: usize::MAX,
            max_sequence_length: usize::MIN,
        }
    }
}

fn visit(metrics: &mut Metrics, _: &Record) {
    metrics.record_count += 1;
}
