use std::fmt::Write;

use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::{FromEntropy, Rng, SeedableRng};

use distributions::Character;

use Block;

static UPPER_ALPHA_CHARSET: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
static QUALITY_CHARSET: &'static [u8] = b"@ABCDEFGHIJ";
static NUCLEOBASE_CHARSET: &'static [u8] = b"AGTC";

static PLUS_LINE: &'static str = "+";

const READ_LEN: usize = 101;
const FLOW_CELL_LEN: usize = 7;

const LANES: u32 = 8;
const TILES: u32 = 60;
const MAX_X: u32 = 10000;
const MAX_Y: u32 = 10000;

pub struct Generator {
    instrument: String,
    run_number: i32,
    flow_cell: String,

    rng: SmallRng,
    lane_range: Uniform<u32>,
    tile_range: Uniform<u32>,
    x_pos_range: Uniform<u32>,
    y_pos_range: Uniform<u32>,
    sequence_distribution: Character,
    quality_distribution: Character,

    block: Block,
}

impl Generator {
    pub fn from_rng(mut rng: SmallRng) -> Generator {
        let instrument = format!("fqlib{}", rng.gen_range(1, 10 + 1));
        let run_number = rng.gen_range(1, 1000 + 1);
        let flow_cell = gen_flow_cell(&mut rng, FLOW_CELL_LEN);

        let lane_range = Uniform::new(1, LANES + 1);
        let tile_range = Uniform::new(1, TILES + 1);
        let x_pos_range = Uniform::new(1, MAX_X + 1);
        let y_pos_range = Uniform::new(1, MAX_Y + 1);

        let sequence_distribution = Character::new(NUCLEOBASE_CHARSET);
        let quality_distribution = Character::new(QUALITY_CHARSET);

        let mut block = Block::default();
        block.plus_line.push_str(PLUS_LINE);

        Generator {
            instrument,
            flow_cell,
            run_number,

            rng,
            lane_range,
            tile_range,
            x_pos_range,
            y_pos_range,
            sequence_distribution,
            quality_distribution,

            block,
        }
    }

    pub fn from_seed(seed: [u8; 16]) -> Generator {
        let rng = SmallRng::from_seed(seed);
        Generator::from_rng(rng)
    }

    pub fn new() -> Generator {
        let rng = SmallRng::from_entropy();
        Generator::from_rng(rng)
    }

    pub fn next_block(&mut self) -> &Block {
        self.clear_block();

        self.next_name();
        self.next_sequence();
        self.next_quality();

        &self.block
    }

    pub fn next_block_with_name(&mut self, name: &str) -> &Block {
        self.clear_block();

        self.block.name.push_str(name);
        self.next_sequence();
        self.next_quality();

        &self.block
    }

    fn next_name(&mut self) {
        let lane = self.lane_range.sample(&mut self.rng);
        let tile = self.tile_range.sample(&mut self.rng);
        let x_pos = self.x_pos_range.sample(&mut self.rng);
        let y_pos = self.y_pos_range.sample(&mut self.rng);

        write!(
            &mut self.block.name,
            "@{}:{}:{}:{}:{}:{}:{}",
            self.instrument, self.run_number, self.flow_cell,
            lane, tile, x_pos, y_pos,
        ).unwrap();
    }

    fn next_sequence(&mut self) {
        let iter = self.rng
            .sample_iter(&self.sequence_distribution)
            .take(READ_LEN);

        for c in iter {
            self.block.sequence.push(c);
        }
    }

    fn next_quality(&mut self) {
        let iter = self.rng
            .sample_iter(&self.quality_distribution)
            .take(READ_LEN);

        for c in iter {
            self.block.quality.push(c);
        }
    }

    // Clears all buffers but the plus line since that never changes.
    fn clear_block(&mut self) {
        self.block.name.clear();
        self.block.sequence.clear();
        self.block.quality.clear();
    }
}

fn gen_flow_cell(rng: &mut SmallRng, len: usize) -> String {
    let distribution = Character::new(UPPER_ALPHA_CHARSET);
    rng.sample_iter(&distribution).take(len).collect()
}

pub struct BlockPairGenerator {
    generator_1: Generator,
    generator_2: Generator,
}

impl BlockPairGenerator {
    pub fn from_seed(seed: [u8; 16]) -> BlockPairGenerator {
        let rng_1 = SmallRng::from_seed(seed);
        let rng_2 = SmallRng::from_seed(seed);

        BlockPairGenerator {
            generator_1: Generator::from_rng(rng_1),
            generator_2: Generator::from_rng(rng_2),
        }
    }

    pub fn new() -> BlockPairGenerator {
        BlockPairGenerator {
            generator_1: Generator::new(),
            generator_2: Generator::new(),
        }
    }

    pub fn next_block_pair(&mut self) -> (&Block, &Block) {
        let b = self.generator_1.next_block();
        let d = self.generator_2.next_block_with_name(&b.name);
        (b, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SEED: [u8; 16] = [
        0x28, 0x8f, 0x28, 0x22, 0x5e, 0x8b, 0x18, 0x03,
        0x8a, 0x08, 0x9a, 0x77, 0x1d, 0x8f, 0x0b, 0x44,
    ];

    #[test]
    fn test_next_block() {
        let mut generator = Generator::from_seed(SEED);

        let block = generator.next_block();

        assert_eq!(block.name, "@fqlib2:898:JSYLNGV:8:44:169:5281");
        assert_eq!(block.sequence, "CTACTATCGGCCCACGACTCTCGCTGGGAGAGCTCACATTCTTGGCGTAGGCAATTCGCAGCTCAAGACAAAAGAGTGGAAGGCAGTTCGACGCGAACTCT");
        assert_eq!(block.plus_line, "+");
        assert_eq!(block.quality, "GGIFD@BCBHC@DDJAAIGFF@I@CFFCEIE@DH@CFAJJIDDHJH@@FACBAHJHIHJCDFDHEHBBCCBABFIJHFCFCB@FAFCCAHFDBCJJGFJI@");
    }

    #[test]
    fn test_next_block_with_name() {
        let mut generator = Generator::from_seed(SEED);
        let block = generator.next_block_with_name("@fqlib");
        assert_eq!(block.name, "@fqlib");
    }
}
