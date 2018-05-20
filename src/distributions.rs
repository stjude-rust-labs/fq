use rand::Rng;
use rand::distributions::Distribution;

pub struct Character {
    alphabet: &'static [u8],
}

impl Character {
    pub fn new(alphabet: &'static [u8]) -> Character {
        Character { alphabet }
    }
}

impl Distribution<char> for Character {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        let i = rng.gen_range(0, self.alphabet.len());
        self.alphabet[i] as char
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
        assert_eq!(x, 'a');
    }
}
