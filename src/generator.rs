use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::{FromEntropy, Rng, SeedableRng};

use distributions::Character;

use {Block, BlockBuf};

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

    block_buf_1: BlockBuf,
    block_buf_2: BlockBuf,
}

impl Generator {
    pub fn from_rng(mut rng: SmallRng) -> Generator {
        let instrument = format!("fqlib{}", rng.gen_range(1, 10 + 1));
        let run_number = rng.gen_range(1, 1000 + 1);
        let flow_cell = gen_flow_cell(&mut rng, FLOW_CELL_LEN);

        let mut block_buf_1 = BlockBuf::new();
        block_buf_1.plus_line.push_str(PLUS_LINE);

        let mut block_buf_2 = BlockBuf::new();
        block_buf_2.plus_line.push_str(PLUS_LINE);

        Generator { instrument, flow_cell, run_number, rng, block_buf_1, block_buf_2 }
    }

    pub fn from_seed(seed: [u8; 16]) -> Generator {
        let rng = SmallRng::from_seed(seed);
        Generator::from_rng(rng)
    }

    pub fn new() -> Generator {
        let rng = SmallRng::from_entropy();
        Generator::from_rng(rng)
    }

    pub fn pairs(self) -> Pairs {
        Pairs::new(self)
    }

    fn name(&mut self) -> String {
        let lane = self.rng.gen_range(1, LANES + 1);
        let tile = self.rng.gen_range(1, TILES + 1);
        let x_pos = self.rng.gen_range(1, MAX_X + 1);
        let y_pos = self.rng.gen_range(1, MAX_Y + 1);

        format!(
            "@{}:{}:{}:{}:{}:{}:{}",
            self.instrument, self.run_number, self.flow_cell,
            lane, tile, x_pos, y_pos,
        )
    }

    fn sequence(&mut self) -> String {
        let distribution = Character::new(NUCLEOBASE_CHARSET);
        self.rng.sample_iter(&distribution).take(READ_LEN).collect()
    }

    fn plus_line(&self) -> &'static str {
        PLUS_LINE
    }

    fn quality(&mut self) -> String {
        let distribution = Character::new(QUALITY_CHARSET);
        self.rng.sample_iter(&distribution).take(READ_LEN).collect()
    }

    fn next_block(&mut self) -> Block {
        Block::new(self.name(), self.sequence(), self.plus_line(), self.quality())
    }

    pub fn next_block_buf_pair(&mut self) -> (&BlockBuf, &BlockBuf) {
        self.block_buf_1.name.clear();
        self.block_buf_1.sequence.clear();
        self.block_buf_1.quality.clear();

        self.block_buf_2.name.clear();
        self.block_buf_2.sequence.clear();
        self.block_buf_2.quality.clear();

        let name = self.name();

        self.block_buf_1.name.push_str(&name);
        self.block_buf_2.name.push_str(&name);

        let n_range = Uniform::new(0, NUCLEOBASE_CHARSET.len());
        let q_range = Uniform::new(0, QUALITY_CHARSET.len());

        for _ in 0..READ_LEN {
            let i = n_range.sample(&mut self.rng);
            let c = NUCLEOBASE_CHARSET[i] as char;
            self.block_buf_1.sequence.push(c);

            let i = q_range.sample(&mut self.rng);
            let c = QUALITY_CHARSET[i] as char;
            self.block_buf_1.quality.push(c);

            let i = n_range.sample(&mut self.rng);
            let c = NUCLEOBASE_CHARSET[i] as char;
            self.block_buf_2.sequence.push(c);

            let i = q_range.sample(&mut self.rng);
            let c = QUALITY_CHARSET[i] as char;
            self.block_buf_2.quality.push(c);
        }

        (&self.block_buf_1, &self.block_buf_2)
    }
}

impl Iterator for Generator {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        Some(self.next_block())
    }
}

pub struct Pairs(Generator);

impl Pairs {
    pub fn new(generator: Generator) -> Pairs {
        Pairs(generator)
    }
}

impl Iterator for Pairs {
    type Item = (Block, Block);

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.0.next_block();

        let d = Block::new(
            b.name.clone(),
            self.0.sequence(),
            self.0.plus_line(),
            self.0.quality()
        );

        Some((b, d))
    }
}

fn gen_flow_cell(rng: &mut SmallRng, len: usize) -> String {
    let distribution = Character::new(UPPER_ALPHA_CHARSET);
    rng.sample_iter(&distribution).take(len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    static SEED: [u8; 16] = [
        0x28, 0x8f, 0x28, 0x22, 0x5e, 0x8b, 0x18, 0x03,
        0x8a, 0x08, 0x9a, 0x77, 0x1d, 0x8f, 0x0b, 0x44,
    ];

    #[test]
    fn test_name() {
        let mut generator = Generator::from_seed(SEED);
        assert_eq!(generator.name(), "@fqlib2:898:JSLNGVS:1:32:8896:8166");
    }

    #[test]
    fn test_sequence() {
        let mut generator = Generator::from_seed(SEED);
        assert_eq!(
            generator.sequence(),
            "TCTAGTGCTGGGACATTTGGAGCAGCAGCTAAGAAAGGGGAGAGTGACACTCTTAGGGAATTACAGTTGTCACAGTCGGCCAATAGCCGTGTGGGATCCTG",
        );
    }

    #[test]
    fn test_plus_line() {
        let generator = Generator::new();
        assert_eq!(generator.plus_line(), PLUS_LINE);
    }

    #[test]
    fn test_quality() {
        let mut generator = Generator::from_seed(SEED);
        assert_eq!(
            generator.quality(),
            "FB@GDDIAJFJHJHCCEBCADHGBFFECJG@ECIB@HHJDH@FJBJABAACGC@DAFGJDAE@BHEHGF@BHC@DDJAGF@I@CFFEIE@HJIDDH@FACB",
        );
    }
}
