use std::path::PathBuf;

use anyhow::Context;
use clap::ArgMatches;
use rand::{rngs::SmallRng, SeedableRng};
use tracing::info;

use crate::{generator::Builder, Generator, PairWriter};

pub fn generate(matches: &ArgMatches) -> anyhow::Result<()> {
    let r1_dst: &PathBuf = matches.get_one("r1-dst").unwrap();
    let r2_dst: &PathBuf = matches.get_one("r2-dst").unwrap();

    let record_count: u64 = *matches.get_one("record-count").unwrap();
    let read_length: usize = *matches.get_one("read-length").unwrap();

    info!("fq-generate start");

    let builder = if let Some(seed) = matches.get_one("seed") {
        let rng = SmallRng::seed_from_u64(*seed);
        Builder::from_rng(rng)
    } else {
        Generator::builder()
    };

    let generator = builder.set_read_length(read_length).build();

    let w1 = crate::fastq::create(r1_dst)
        .with_context(|| format!("Could not create file: {}", r1_dst.display()))?;

    let w2 = crate::fastq::create(r2_dst)
        .with_context(|| format!("Could not create file: {}", r2_dst.display()))?;

    let mut writer = PairWriter::new(w1, w2);

    writer
        .write(generator, record_count)
        .context("Could not write generated records")?;

    info!("generated {} records", record_count);
    info!("fq-generate end");

    Ok(())
}
