use std::{io, path::PathBuf};

use rand::{rngs::SmallRng, SeedableRng};
use thiserror::Error;
use tracing::info;

use crate::{cli::GenerateArgs, generator::Builder, Generator, PairWriter};

pub fn generate(args: GenerateArgs) -> Result<(), GenerateError> {
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

    let w1 =
        crate::fastq::create(r1_dst).map_err(|e| GenerateError::CreateFile(e, r1_dst.clone()))?;
    let w2 =
        crate::fastq::create(r2_dst).map_err(|e| GenerateError::CreateFile(e, r2_dst.clone()))?;

    let mut writer = PairWriter::new(w1, w2);

    let record_count = args.record_count;

    writer.write(generator, record_count)?;

    info!("generated {} records", record_count);
    info!("fq-generate end");

    Ok(())
}

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("could not create file: {1}")]
    CreateFile(#[source] io::Error, PathBuf),
}
