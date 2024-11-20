use std::io;

use tracing::info;

use crate::{
    cli::DescribeArgs,
    fastq::{self, Record},
    metrics,
};

pub fn describe(args: DescribeArgs) -> io::Result<()> {
    info!(command = "describe", "fq");

    let mut reader = fastq::open(args.src)?;
    let mut record = Record::default();

    let mut metrics = metrics::default();

    while reader.read_record(&mut record)? != 0 {
        for metric in &mut metrics {
            metric.visit(&record)?;
        }
    }

    for metric in &metrics {
        metric.println();
    }

    info!("done");

    Ok(())
}
