mod builder;

pub use self::builder::Builder;

use std::io::Write;

use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    Rng, SeedableRng,
};

use super::{
    distributions::{Character, QualityScores},
    fastq::Record,
};

static UPPER_ALPHA_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
static NUCLEOBASE_CHARSET: &[u8] = b"AGTC";

const READ_LEN: usize = 101;
const FLOW_CELL_ID_LEN: usize = 7;

const LANES: u32 = 8;
const TILES: u32 = 60;
const MAX_X: u32 = 10000;
const MAX_Y: u32 = 10000;

/// A FASTQ record generator.
pub struct Generator<R> {
    instrument: String,
    run_number: i32,
    flow_cell_id: String,

    rng: R,
    lane_range: Uniform<u32>,
    tile_range: Uniform<u32>,
    x_pos_range: Uniform<u32>,
    y_pos_range: Uniform<u32>,
    sequence_distribution: Character,
    quality_distribution: QualityScores,

    read_length: usize,
}

impl Generator<SmallRng> {
    pub fn builder() -> Builder<SmallRng> {
        Builder::default()
    }

    /// Creates a `Generator<SmallRng>` seeded by the system.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_from_u64(seed: u64) -> Self {
        let rng = SmallRng::seed_from_u64(seed);
        Self::from_rng(rng, READ_LEN)
    }
}

impl Default for Generator<SmallRng> {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl<R> Generator<R>
where
    R: Rng,
{
    /// Creates a new `Generator` with a given `Rng` and read length.
    pub fn from_rng(mut rng: R, read_length: usize) -> Self {
        let instrument = format!("fqlib{}", rng.gen_range(1..=10));
        let run_number = rng.gen_range(1..=1000);
        let flow_cell_id = gen_flow_cell_id(&mut rng, FLOW_CELL_ID_LEN);

        let lane_range = Uniform::new(1, LANES + 1);
        let tile_range = Uniform::new(1, TILES + 1);
        let x_pos_range = Uniform::new(1, MAX_X + 1);
        let y_pos_range = Uniform::new(1, MAX_Y + 1);

        let sequence_distribution = Character::new(NUCLEOBASE_CHARSET);
        let quality_distribution = QualityScores::default();

        Self {
            instrument,
            run_number,
            flow_cell_id,

            rng,
            lane_range,
            tile_range,
            x_pos_range,
            y_pos_range,
            sequence_distribution,
            quality_distribution,

            read_length,
        }
    }

    /// Returns a freshly generated record.
    pub fn next_record(&mut self, record: &mut Record) {
        clear_record(record);

        self.next_name(record);
        self.next_sequence(record);
        self.next_quality(record);
    }

    /// Returns a freshly generated record, setting the name to the given input.
    pub fn next_record_with_name(&mut self, name: &[u8], record: &mut Record) {
        clear_record(record);

        record.name_mut().extend_from_slice(name);
        self.next_sequence(record);
        self.next_quality(record);
    }

    // Generates a name following Illumina's naming format, sans interleave.
    //
    // @see <https://help.basespace.illumina.com/articles/descriptive/fastq-files/>
    fn next_name(&mut self, record: &mut Record) {
        let lane = self.lane_range.sample(&mut self.rng);
        let tile = self.tile_range.sample(&mut self.rng);
        let x_pos = self.x_pos_range.sample(&mut self.rng);
        let y_pos = self.y_pos_range.sample(&mut self.rng);

        write!(
            record.name_mut(),
            "@{}:{}:{}:{}:{}:{}:{}",
            self.instrument,
            self.run_number,
            self.flow_cell_id,
            lane,
            tile,
            x_pos,
            y_pos,
        )
        .unwrap();
    }

    fn next_sequence(&mut self, record: &mut Record) {
        let iter = (&mut self.rng)
            .sample_iter(&self.sequence_distribution)
            .take(self.read_length);

        record.sequence_mut().extend(iter);
    }

    fn next_quality(&mut self, record: &mut Record) {
        let iter = (&mut self.rng)
            .sample_iter(&self.quality_distribution)
            .take(self.read_length)
            .map(|phred| phred + 33);

        record.quality_scores_mut().extend(iter);
    }
}

fn clear_record(record: &mut Record) {
    record.name_mut().clear();
    record.sequence_mut().clear();
    record.quality_scores_mut().clear();
}

fn gen_flow_cell_id<R>(rng: &mut R, len: usize) -> String
where
    R: Rng,
{
    let distribution = Character::new(UPPER_ALPHA_CHARSET);
    let bytes = rng.sample_iter(&distribution).take(len).collect();
    String::from_utf8(bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_record() {
        let mut generator = Generator::seed_from_u64(0);

        let mut record = Record::default();
        generator.next_record(&mut record);

        assert_eq!(
            record.name(),
            "@fqlib5:440:ZMXYPLK:7:15:9764:6446".as_bytes()
        );
        assert_eq!(record.sequence(), "ACAAGCTTAGCGCCACGCAGCGGGTGATCGAGTGGGCTAACAATTAAACTTTGAAGTACCGGCCCCTCCTGATGCATCCGGCGGTCCTTGTAGAATGACCC".as_bytes());
        assert_eq!(record.quality_scores(), "6547759627579>3111:817:585;87246;6;425;773656:857836434354769:6574745887;74348774:7358566335664964387".as_bytes());
    }

    #[test]
    fn test_next_record_with_read_length() {
        const READ_LENGTH: usize = 4;

        let rng = SmallRng::seed_from_u64(0);
        let mut generator = Generator::from_rng(rng, READ_LENGTH);

        let mut record = Record::default();
        generator.next_record(&mut record);

        assert_eq!(record.sequence().len(), READ_LENGTH);
        assert_eq!(record.quality_scores().len(), READ_LENGTH);
    }

    #[test]
    fn test_next_record_with_name() {
        let mut generator = Generator::seed_from_u64(0);

        let mut record = Record::default();

        let name = b"@fqlib";
        generator.next_record_with_name(name, &mut record);

        assert_eq!(record.name(), name);
    }
}
