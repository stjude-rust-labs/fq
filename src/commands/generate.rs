use anyhow::Context;
use clap::{value_t, ArgMatches};
use log::info;

use crate::{Generator, PairWriter};

pub fn generate(matches: &ArgMatches<'_>) -> anyhow::Result<()> {
    let r1_dst = matches.value_of("r1-dst").unwrap();
    let r2_dst = matches.value_of("r2-dst").unwrap();

    let n_records = value_t!(matches, "n-records", i32).unwrap_or_else(|e| e.exit());

    info!("fq-generate start");

    let generator = if matches.is_present("seed") {
        let seed = value_t!(matches, "seed", u64).unwrap_or_else(|e| e.exit());
        Generator::seed_from_u64(seed)
    } else {
        Generator::new()
    };

    let w1 = crate::fastq::create(r1_dst)
        .with_context(|| format!("Could not create file: {}", r1_dst))?;

    let w2 = crate::fastq::create(r2_dst)
        .with_context(|| format!("Could not create file: {}", r2_dst))?;

    let mut writer = PairWriter::new(w1, w2);

    writer
        .write(generator, n_records)
        .context("Could not write generated records")?;

    info!("fq-generate end");

    Ok(())
}
