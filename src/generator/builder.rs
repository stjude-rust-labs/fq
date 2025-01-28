use rand::{rngs::SmallRng, Rng, SeedableRng};

use super::{Generator, DEFAULT_READ_LENGTH};

pub struct Builder<R> {
    rng: R,
    read_length: usize,
}

impl<R> Builder<R>
where
    R: Rng,
{
    pub fn from_rng(rng: R) -> Self {
        Self {
            rng,
            read_length: DEFAULT_READ_LENGTH,
        }
    }

    pub fn set_read_length(mut self, read_length: usize) -> Self {
        self.read_length = read_length;
        self
    }

    pub fn build(self) -> Generator<R> {
        Generator::from_rng(self.rng, self.read_length)
    }
}

impl Default for Builder<SmallRng> {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_os_rng(),
            read_length: DEFAULT_READ_LENGTH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let generator = Builder::default().set_read_length(4).build();
        assert_eq!(generator.read_length, 4);
    }
}
