use rand::{
    Rng,
    distr::{Distribution, slice::Choose},
};

/// Sample a `char`, uniformly distributed over a given character set.
pub struct Character(Choose<'static, u8>);

impl Character {
    pub fn new(alphabet: &'static [u8]) -> Self {
        assert!(!alphabet.is_empty());
        // SAFETY: `alphabet` is non-empty.
        Choose::new(alphabet).map(Self).unwrap()
    }
}

impl Distribution<u8> for Character {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        *self.0.sample(rng)
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::SmallRng};

    use super::*;

    #[test]
    fn test_sample() {
        let mut rng = SmallRng::seed_from_u64(0);
        let distribution = Character::new(b"ACGT");

        let c = rng.sample(&distribution);
        assert_eq!(c, b'C');

        let buf: Vec<u8> = rng.sample_iter(&distribution).take(3).collect();
        assert_eq!(buf, b"CCA");
    }
}
