use anyhow::Context;
use clap::{value_t, ArgMatches};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use tracing::info;

use crate::fastq::{self, Record};

pub fn subsample(matches: &ArgMatches<'_>) -> anyhow::Result<()> {
    let probability = value_t!(matches, "probability", f64).unwrap_or_else(|e| e.exit());
    let src = matches.value_of("src").unwrap();
    let dst = matches.value_of("dst").unwrap();

    let mut reader = fastq::open(src).with_context(|| format!("Could not open file: {}", src))?;
    let mut writer =
        fastq::create(dst).with_context(|| format!("Could not create file: {}", dst))?;
    let mut rng = SmallRng::from_entropy();

    let mut record = Record::default();
    let mut n: u64 = 0;
    let mut total: u64 = 0;

    info!("fq-subsample start");
    info!("probability (p) = {}", probability);

    loop {
        match reader.read_record(&mut record) {
            Ok(0) => break,
            Ok(_) => {
                let q: f64 = rng.gen();

                if q <= probability {
                    writer.write_record(&record)?;
                    n += 1;
                }

                total += 1;
            }
            Err(e) => {
                return Err(e).with_context(|| format!("Could not read record from file: {}", src))
            }
        }
    }

    let percentage = (n as f64) / (total as f64) * 100.0;
    info!("sampled {}/{} ({:.4}%) records", n, total, percentage);

    info!("fq-subsample end");

    Ok(())
}
