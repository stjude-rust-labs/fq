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

    print_metrics(&metrics);

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

fn visit(metrics: &mut Metrics, record: &Record) {
    metrics.record_count += 1;

    let read_length = record.sequence().len();

    metrics.min_sequence_length = metrics.min_sequence_length.min(read_length);
    metrics.max_sequence_length = metrics.max_sequence_length.max(read_length);
}

fn print_metrics(metrics: &Metrics) {
    let record_count = metrics.record_count;

    println!("record_count\t{record_count}");

    let min_sequence_length = if record_count == 0 {
        0
    } else {
        metrics.min_sequence_length
    };

    println!("min_sequence_length\t{min_sequence_length}");

    let max_sequence_length = if record_count == 0 {
        0
    } else {
        metrics.max_sequence_length
    };

    println!("max_sequence_length\t{max_sequence_length}");
}
