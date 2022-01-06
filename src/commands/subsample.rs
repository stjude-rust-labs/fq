use std::io::{self, BufRead, Write};

use anyhow::Context;
use clap::ArgMatches;
use rand::{rngs::SmallRng, SeedableRng};
use tracing::info;

use crate::fastq::{self, Record};

pub fn subsample(matches: &ArgMatches) -> anyhow::Result<()> {
    let r1_src = matches.value_of("r1-src").unwrap();
    let r1_dst = matches.value_of("r1-dst").unwrap();

    let r2_src = matches.value_of("r2-src");
    let r2_dst = matches.value_of("r2-dst");

    info!("fq-subsample start");

    let mut rng = if matches.is_present("seed") {
        let seed = matches.value_of_t("seed").unwrap_or_else(|e| e.exit());
        info!("initializing rng from seed = {}", seed);
        SmallRng::seed_from_u64(seed)
    } else {
        info!("initializing rng from entropy");
        SmallRng::from_entropy()
    };

    let probability = matches
        .value_of_t("probability")
        .unwrap_or_else(|e| e.exit());

    if !(0.0..=1.0).contains(&probability) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput))
            .with_context(|| format!("invalid probability = {}", probability));
    }

    let mut r1 = fastq::open(r1_src).with_context(|| format!("Could not open file: {}", r1_src))?;
    let mut w1 =
        fastq::create(r1_dst).with_context(|| format!("Could not create file: {}", r1_dst))?;

    info!("probability (p) = {}", probability);

    let (n, total) = match (r2_src, r2_dst) {
        (Some(r2_src), Some(r2_dst)) => {
            info!("sampling paired end reads");

            let mut r2 =
                fastq::open(r2_src).with_context(|| format!("Could not open file: {}", r2_src))?;
            let mut w2 = fastq::create(r2_dst)
                .with_context(|| format!("Could not create file: {}", r2_dst))?;

            subsample_paired(
                (&mut r1, &mut w1),
                (&mut r2, &mut w2),
                &mut rng,
                probability,
            )?
        }
        (Some(r2_src), None) => {
            return Err(io::Error::from(io::ErrorKind::InvalidInput))
                .with_context(|| format!("Missing r2-dst for {}", r2_src));
        }
        (None, Some(r2_dst)) => {
            return Err(io::Error::from(io::ErrorKind::InvalidInput))
                .with_context(|| format!("Missing r2-src for {}", r2_dst));
        }
        _ => {
            info!("sampling single end reads");
            subsample_single(&mut r1, &mut w1, &mut rng, probability)?
        }
    };

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

fn subsample_paired<R, S, W, X, Rng>(
    (r1, w1): (&mut fastq::Reader<R>, &mut fastq::Writer<W>),
    (r2, w2): (&mut fastq::Reader<S>, &mut fastq::Writer<X>),
    rng: &mut Rng,
    p: f64,
) -> anyhow::Result<(u64, u64)>
where
    R: BufRead,
    S: BufRead,
    W: Write,
    X: Write,
    Rng: rand::Rng,
{
    let mut s1 = Record::default();
    let mut s2 = Record::default();

    let mut n = 0;
    let mut total = 0;

    loop {
        match (r1.read_record(&mut s1)?, r2.read_record(&mut s2)?) {
            (0, 0) => break,
            (0, len) if len > 0 => {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof))
                    .with_context(|| "r1-src unexpectedly ended before r2-src");
            }
            (len, 0) if len > 0 => {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof))
                    .with_context(|| "r2-src unexpectedly ended before r1-src");
            }
            (_, _) => {
                let q: f64 = rng.gen();

                if q <= p {
                    w1.write_record(&s1)?;
                    w2.write_record(&s2)?;
                    n += 1;
                }

                total += 1;
            }
        }
    }

    Ok((n, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsample_single() -> anyhow::Result<()> {
        let data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let mut reader = fastq::Reader::new(&data[..]);
        let mut writer = fastq::Writer::new(Vec::new());

        let mut rng = SmallRng::seed_from_u64(0);

        subsample_single(&mut reader, &mut writer, &mut rng, 0.45)?;

        let expected = b"@r1\nACGT\n+\nFQLB\n@r2\nACGT\n+\nFQLB\n";
        assert_eq!(writer.get_ref(), expected);

        Ok(())
    }

    #[test]
    fn test_subsample_paired() -> anyhow::Result<()> {
        let r1_data = b"@r1\nACGT\n+\nFQLB
@r2\nACGT\n+\nFQLB
@r3\nACGT\n+\nFQLB
@r4\nACGT\n+\nFQLB
";

        let r2_data = b"@r1\nTGCA\n+\nBLQF
@r2\nTGCA\n+\nBLQF
@r3\nTGCA\n+\nBLQF
@r4\nTGCA\n+\nBLQF
";

        let mut r1 = fastq::Reader::new(&r1_data[..]);
        let mut w1 = fastq::Writer::new(Vec::new());
        let mut r2 = fastq::Reader::new(&r2_data[..]);
        let mut w2 = fastq::Writer::new(Vec::new());

        let mut rng = SmallRng::seed_from_u64(0);

        subsample_paired((&mut r1, &mut w1), (&mut r2, &mut w2), &mut rng, 0.45)?;

        let w1_expected = b"@r1\nACGT\n+\nFQLB\n@r2\nACGT\n+\nFQLB\n";
        assert_eq!(w1.get_ref(), w1_expected);

        let w2_expected = b"@r1\nTGCA\n+\nBLQF\n@r2\nTGCA\n+\nBLQF\n";
        assert_eq!(w2.get_ref(), w2_expected);

        Ok(())
    }
}
