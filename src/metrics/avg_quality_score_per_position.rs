use std::io;

use super::Metric;
use crate::fastq::Record;

const NAME: &str = "avg_quality_score_per_position";

#[derive(Clone, Copy, Default)]
struct ErrorProbability {
    sum: f64,
    count: u64,
}

#[derive(Default)]
pub struct AvgQualityScorePerPosition {
    error_probability_sums_per_position: Vec<ErrorProbability>,
}

impl Metric for AvgQualityScorePerPosition {
    fn visit(&mut self, record: &Record) -> io::Result<()> {
        let read_length = record.sequence().len();

        if read_length > self.error_probability_sums_per_position.len() {
            self.error_probability_sums_per_position
                .resize(read_length, ErrorProbability::default());
        }

        for (quality_score, error_probability) in record
            .quality_scores()
            .iter()
            .zip(&mut self.error_probability_sums_per_position)
        {
            let q = decode_score(*quality_score)?;
            let p = phred_score_to_error_probability(q);
            error_probability.sum += p;
            error_probability.count += 1;
        }

        Ok(())
    }

    fn println(&self) {
        let avg_quality_score_per_position: Vec<_> = self
            .error_probability_sums_per_position
            .iter()
            .map(|error_probability| {
                let n = error_probability.count as f64;
                let avg_error_probability = error_probability.sum / n;
                error_probability_to_phred_score(avg_error_probability)
            })
            .collect();

        println!("{NAME}\t{avg_quality_score_per_position:?}");
    }
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
