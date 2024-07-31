use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

/// Sample a `char`, uniformly distributed over a given character set.
pub struct Character {
    alphabet: &'static [u8],
    range: Uniform<usize>,
}

impl Character {
    pub fn new(alphabet: &'static [u8]) -> Self {
        let range = Uniform::new(0, alphabet.len());
        Self { alphabet, range }
    }
}

impl Distribution<u8> for Character {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        let i = self.range.sample(rng);
        self.alphabet[i]
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::mock::StepRng;

    use super::*;

    #[test]
    fn test_sample() {
        let mut rng = StepRng::new(0, 1);
        let distribution = Character::new(b"ACGT");

        let c = rng.sample(&distribution);
        assert_eq!(c, b'A');

        let buf: Vec<u8> = rng.sample_iter(&distribution).take(3).collect();
        assert_eq!(buf, b"AAA");
    }
}
