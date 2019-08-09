use std::io::Write;

use noodles::formats::fastq::Record;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::distributions::Character;

static UPPER_ALPHA_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
static QUALITY_CHARSET: &[u8] = b"@ABCDEFGHIJ";
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
    quality_distribution: Character,
}

impl Generator<SmallRng> {
    /// Creates a `Generator<SmallRng>` seeded by the system.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Generator;
    /// let _ = Generator::new();
    /// ```
    pub fn new() -> Generator<SmallRng> {
        Generator::default()
    }

    /// Creates a `Generator<SmallRng>` from a given seed.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Generator;
    ///
    /// let seed = [
    ///     0x28, 0x8f, 0x28, 0x22, 0x5e, 0x8b, 0x18, 0x03,
    ///     0x8a, 0x08, 0x9a, 0x77, 0x1d, 0x8f, 0x0b, 0x44,
    /// ];
    ///
    /// let _ = Generator::from_seed(seed);
    /// ```
    pub fn from_seed(seed: [u8; 16]) -> Generator<SmallRng> {
        let rng = SmallRng::from_seed(seed);
        Generator::from_rng(rng)
    }
}

impl Default for Generator<SmallRng> {
    fn default() -> Generator<SmallRng> {
        let rng = SmallRng::from_entropy();
        Generator::from_rng(rng)
    }
}

impl<R> Generator<R>
where
    R: Rng,
{
    /// Creates a new `Generator` with a given `Rng`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rand::{SeedableRng, rngs::SmallRng};
    /// #
    /// # fn main() {
    /// use fqlib::Generator;
    /// let rng = SmallRng::from_entropy();
    /// let _ = Generator::from_rng(rng);
    /// # }
    /// ```
    pub fn from_rng(mut rng: R) -> Generator<R> {
        let instrument = format!("fqlib{}", rng.gen_range(1, 10 + 1));
        let run_number = rng.gen_range(1, 1000 + 1);
        let flow_cell_id = gen_flow_cell_id(&mut rng, FLOW_CELL_ID_LEN);

        let lane_range = Uniform::new(1, LANES + 1);
        let tile_range = Uniform::new(1, TILES + 1);
        let x_pos_range = Uniform::new(1, MAX_X + 1);
        let y_pos_range = Uniform::new(1, MAX_Y + 1);

        let sequence_distribution = Character::new(NUCLEOBASE_CHARSET);
        let quality_distribution = Character::new(QUALITY_CHARSET);

        Generator {
            instrument,
            flow_cell_id,
            run_number,

            rng,
            lane_range,
            tile_range,
            x_pos_range,
            y_pos_range,
            sequence_distribution,
            quality_distribution,
        }
    }

    /// Returns a freshly generated record.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Generator;
    /// use noodles::formats::fastq::Record;
    ///
    /// let mut generator = Generator::new();
    /// let mut record = Record::default();
    /// generator.next_record(&mut record);
    /// ```
    pub fn next_record(&mut self, record: &mut Record) {
        clear_record(record);

        self.next_name(record);
        self.next_sequence(record);
        self.next_quality(record);
    }

    /// Returns a freshly generated record, setting the name to the given input.
    ///
    /// # Examples
    ///
    /// ```
    /// use fqlib::Generator;
    /// use noodles::formats::fastq::Record;
    ///
    /// let mut generator = Generator::new();
    /// let mut record = Record::default();
    /// generator.next_record_with_name(b"@fqlib", &mut record);
    /// assert_eq!(record.name(), b"@fqlib");
    /// ```
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
            .take(READ_LEN);

        let sequence = record.sequence_mut();

        for c in iter {
            sequence.push(c);
        }
    }

    fn next_quality(&mut self, record: &mut Record) {
        let iter = (&mut self.rng)
            .sample_iter(&self.quality_distribution)
            .take(READ_LEN);

        let quality = record.quality_mut();

        for c in iter {
            quality.push(c);
        }
    }
}

fn clear_record(record: &mut Record) {
    record.name_mut().clear();
    record.sequence_mut().clear();
    record.quality_mut().clear();
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
    use noodles::formats::fastq::Record;

    use super::*;

    static SEED: [u8; 16] = [
        0x28, 0x8f, 0x28, 0x22, 0x5e, 0x8b, 0x18, 0x03, 0x8a, 0x08, 0x9a, 0x77, 0x1d, 0x8f, 0x0b,
        0x44,
    ];

    #[test]
    fn test_next_record() {
        let mut generator = Generator::from_seed(SEED);

        let mut record = Record::default();
        generator.next_record(&mut record);

        assert_eq!(
            record.name(),
            "@fqlib1:950:DFZYAUO:3:33:7515:3404".as_bytes()
        );
        assert_eq!(record.sequence(), "TTGATTGAAAATTAGATAATACATCAATTCGGGGCCTAATAGTTGGGGTAAGCAAAGGCAGTCATTGACATGGTATCGTTTGCCCTTCACAGCTTACAACG".as_bytes());
        assert_eq!(record.quality(), "FFI@@AEFEGJG@DDDBBCCIJE@DDCACIDFFJE@GIB@@J@AFEDBCGBB@BAAGDFBJHGA@CEBBGGBJHFGG@C@A@HCAFGGGCFIHIFFAEHDC".as_bytes());
    }
}
