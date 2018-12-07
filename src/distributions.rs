use rand::Rng;
use rand::distributions::{Distribution, Uniform};

/// Sample a `char`, uniformly distributed over a given character set.
///
/// # Examples
///
/// ```
/// use rand::{Rng, thread_rng};
/// use fqlib::distributions::Character;
///
/// let mut rng = thread_rng();
/// let distribution = Character::new(b"AGTC");
/// let bytes: Vec<u8> = rng.sample_iter(&distribution).take(8).collect();
/// let s = String::from_utf8(bytes).unwrap();
/// println!("{}", s); // e.g., "TCCTCGAG"
/// ```
pub struct Character {
    alphabet: &'static [u8],
    range: Uniform<usize>,
}

impl Character {
    pub fn new(alphabet: &'static [u8]) -> Character {
        let range = Uniform::new(0, alphabet.len());
        Character { alphabet, range }
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
    use rand::Rng;
    use rand::rngs::mock::StepRng;

    use super::Character;

    #[test]
    fn test_sample() {
        let distribution = Character::new(b"abcd");
        let mut rng = StepRng::new(0, 1);
        let x = rng.sample(distribution);
        assert_eq!(x, b'a');
    }
}
