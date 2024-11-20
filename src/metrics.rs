mod avg_quality_score_per_position;
mod max_sequence_length;
mod metric;
mod min_sequence_length;
mod record_count;

pub use self::metric::Metric;
use self::{
    avg_quality_score_per_position::AvgQualityScorePerPosition,
    max_sequence_length::MaxSequenceLength, min_sequence_length::MinSequenceLength,
    record_count::RecordCount,
};

pub fn default() -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(RecordCount::default()),
        Box::new(MinSequenceLength::default()),
        Box::new(MaxSequenceLength::default()),
        Box::new(AvgQualityScorePerPosition::default()),
    ]
}
