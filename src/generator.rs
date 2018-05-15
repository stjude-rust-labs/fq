use rand::{FromEntropy, Rng, SeedableRng, SmallRng};

use Block;

static DEFAULT_SEQUENCE: &'static str = "AAAAAAAAAACCCCCCCCCGGGGGGGGGGTTTTTTTTTT";
static DEFAULT_PLUS_LINE: &'static str = "+";
static DEFAULT_QUALITY: &'static str = "JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ";

pub struct Generator {
    instrument: String,
    run_number: i32,
    flow_cell: String,
    rng: SmallRng,
}

impl Generator {
    pub fn from_rng(mut rng: SmallRng) -> Generator {
        let instrument = format!("fqlib{}", rng.gen_range(1, 10 + 1));
        let run_number = rng.gen_range(1, 1000 + 1);
        let flow_cell = String::from("AABBCC");

        Generator { instrument, flow_cell, run_number, rng }
    }

    pub fn from_seed(seed: [u8; 16]) -> Generator {
        let rng = SmallRng::from_seed(seed);
        Generator::from_rng(rng)
    }

    pub fn new() -> Generator {
        let rng = SmallRng::from_entropy();
        Generator::from_rng(rng)
    }

    fn name(&mut self) -> String {
        let lane = self.rng.gen_range(1, 8 + 1);
        let tile = self.rng.gen_range(1, 60 + 1);
        let x_pos = self.rng.gen_range(1, 10000 + 1);
        let y_pos = self.rng.gen_range(1, 10000 + 1);

        format!(
            "@{}:{}:{}:{}:{}:{}:{}",
            self.instrument, self.run_number, self.flow_cell,
            lane, tile, x_pos, y_pos,
        )
    }

    fn sequence(&self) -> &'static str {
        DEFAULT_SEQUENCE
    }

    fn plus_line(&self) -> &'static str {
        DEFAULT_PLUS_LINE
    }

    fn quality(&self) -> &'static str {
        DEFAULT_QUALITY
    }

    fn next_block(&mut self) -> Block {
        Block::new(
            self.name(),
            self.sequence().to_string(),
            self.plus_line().to_string(),
            self.quality().to_string(),
        )
    }
}

impl Iterator for Generator {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        Some(self.next_block())
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
    fn test_name() {
        let mut generator = Generator::from_seed(SEED);
        assert_eq!(generator.name(), "@fqlib2:898:AABBCC:3:22:4528:5118");
    }

    #[test]
    fn test_sequence() {
        let generator = Generator::new();
        assert_eq!(generator.sequence(), DEFAULT_SEQUENCE);
    }

    #[test]
    fn test_plus_line() {
        let generator = Generator::new();
        assert_eq!(generator.plus_line(), DEFAULT_PLUS_LINE);
    }

    #[test]
    fn test_quality() {
        let generator = Generator::new();
        assert_eq!(generator.quality(), DEFAULT_QUALITY);
    }
}
