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
        visit(&mut metrics, &record)?;
    }

    print_metrics(&metrics);

    Ok(())
}

#[derive(Clone, Copy, Default)]
struct ErrorProbability {
    sum: f64,
    count: u64,
}

struct Metrics {
    record_count: u64,
    min_sequence_length: usize,
    max_sequence_length: usize,
    error_probability_sums_per_position: Vec<ErrorProbability>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            record_count: 0,
            min_sequence_length: usize::MAX,
            max_sequence_length: usize::MIN,
            error_probability_sums_per_position: Vec::new(),
        }
    }
}

fn visit(metrics: &mut Metrics, record: &Record) -> io::Result<()> {
    metrics.record_count += 1;

    let read_length = record.sequence().len();

    metrics.min_sequence_length = metrics.min_sequence_length.min(read_length);
    metrics.max_sequence_length = metrics.max_sequence_length.max(read_length);

    if read_length > metrics.error_probability_sums_per_position.len() {
        metrics
            .error_probability_sums_per_position
            .resize(read_length, ErrorProbability::default());
    }

    for (quality_score, error_probability) in record
        .quality_scores()
        .iter()
        .zip(&mut metrics.error_probability_sums_per_position)
    {
        let q = decode_score(*quality_score)?;
        let p = phred_score_to_error_probability(q);
        error_probability.sum += p;
        error_probability.count += 1;
    }

    Ok(())
}

fn decode_score(c: u8) -> io::Result<u8> {
    const OFFSET: u8 = b'!';

    c.checked_sub(OFFSET)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid quality score"))
}

// https://en.wikipedia.org/wiki/Phred_quality_score#Definition
const BASE: f64 = 10.0;
const FACTOR: f64 = 10.0;

fn phred_score_to_error_probability(n: u8) -> f64 {
    BASE.powf(-f64::from(n) / FACTOR)
}

fn error_probability_to_phred_score(p: f64) -> f64 {
    -FACTOR * p.log10()
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

    let avg_quality_score_per_position: Vec<_> = metrics
        .error_probability_sums_per_position
        .iter()
        .map(|error_probability| {
            let n = error_probability.count as f64;
            let avg_error_probability = error_probability.sum / n;
            error_probability_to_phred_score(avg_error_probability)
        })
        .collect();

    println!("avg_quality_score_per_position\t{avg_quality_score_per_position:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_score() -> io::Result<()> {
        assert_eq!(decode_score(b'!')?, 0);
        assert_eq!(decode_score(b'~')?, 93);
        assert!(matches!(
            decode_score(0x00),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));
        Ok(())
    }

    #[test]
    fn test_phred_score_to_error_probability() {
        assert_eq!(phred_score_to_error_probability(0), 1.0);
        assert_eq!(phred_score_to_error_probability(10), 0.1);
        assert_eq!(phred_score_to_error_probability(20), 0.01);
        assert_eq!(phred_score_to_error_probability(30), 0.001);
        assert_eq!(phred_score_to_error_probability(40), 0.0001);
    }

    #[test]
    fn test_error_probability_to_phred_score() {
        assert_eq!(error_probability_to_phred_score(1.0), 0.0);
        assert_eq!(error_probability_to_phred_score(0.1), 10.0);
        assert_eq!(error_probability_to_phred_score(0.01), 20.0);
        assert_eq!(error_probability_to_phred_score(0.001), 30.0);
        assert_eq!(error_probability_to_phred_score(0.0001), 40.0);
    }
}
