use std::io::{BufRead, Write};

use anyhow::Context;
use clap::{value_t, ArgMatches};
use rand::{rngs::SmallRng, SeedableRng};
use tracing::info;

use crate::fastq::{self, Record};

pub fn subsample(matches: &ArgMatches<'_>) -> anyhow::Result<()> {
    let src = matches.value_of("src").unwrap();
    let dst = matches.value_of("dst").unwrap();

    let mut rng = if matches.is_present("seed") {
        let seed = value_t!(matches, "seed", u64).unwrap_or_else(|e| e.exit());
        info!("initializing rng from seed = {}", seed);
        SmallRng::seed_from_u64(seed)
    } else {
        info!("initializing rng from entropy");
        SmallRng::from_entropy()
    };

    let probability = value_t!(matches, "probability", f64).unwrap_or_else(|e| e.exit());

    let mut reader = fastq::open(src).with_context(|| format!("Could not open file: {}", src))?;
    let mut writer =
        fastq::create(dst).with_context(|| format!("Could not create file: {}", dst))?;

    info!("fq-subsample start");
    info!("probability (p) = {}", probability);

    let (n, total) = subsample_single(&mut reader, &mut writer, &mut rng, probability)?;

    let percentage = (n as f64) / (total as f64) * 100.0;
    info!("sampled {}/{} ({:.4}%) records", n, total, percentage);

    info!("fq-subsample end");

    Ok(())
}

fn subsample_single<R, W, Rng>(
    reader: &mut fastq::Reader<R>,
    writer: &mut fastq::Writer<W>,
    rng: &mut Rng,
    p: f64,
) -> anyhow::Result<(u64, u64)>
where
    R: BufRead,
    W: Write,
    Rng: rand::Rng,
{
    let mut record = Record::default();

    let mut n = 0;
    let mut total = 0;

    loop {
        match reader.read_record(&mut record)? {
            0 => break,
            _ => {
                let q: f64 = rng.gen();

                if q <= p {
                    writer.write_record(&record)?;
                    n += 1;
                }

                total += 1;
            }
        }
    }

    Ok((n, total))
}
