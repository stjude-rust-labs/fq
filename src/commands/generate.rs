use anyhow::Context;
use rand::{rngs::SmallRng, SeedableRng};
use tracing::info;

use crate::{cli::GenerateArgs, generator::Builder, Generator, PairWriter};

pub fn generate(args: GenerateArgs) -> anyhow::Result<()> {
    info!("fq-generate start");

    let builder = if let Some(seed) = args.seed {
        let rng = SmallRng::seed_from_u64(seed);
        Builder::from_rng(rng)
    } else {
        Generator::builder()
    };

    let r1_dst = &args.r1_dst;
    let r2_dst = &args.r2_dst;

    let generator = builder.set_read_length(args.read_length).build();

    let w1 = crate::fastq::create(r1_dst)
        .with_context(|| format!("Could not create file: {}", r1_dst.display()))?;

    let w2 = crate::fastq::create(r2_dst)
        .with_context(|| format!("Could not create file: {}", r2_dst.display()))?;

    let mut writer = PairWriter::new(w1, w2);

    let record_count = args.record_count;

    writer
        .write(generator, record_count)
        .context("Could not write generated records")?;

    info!("generated {} records", record_count);
    info!("fq-generate end");

    Ok(())
}
