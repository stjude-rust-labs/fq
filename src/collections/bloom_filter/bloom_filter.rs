use std::f64;
use std::hash::{BuildHasher, Hash};

use bit_vec::BitVec;

use super::{DefaultHashBuilder, DoubleHasher};

/// A probabilistic data structure to test whether an element may be in a set or definitely not in
/// a set.
pub struct BloomFilter<S = DefaultHashBuilder> {
    bits: BitVec,

    // bit array length
    m: usize,
    // number of inserted elements
    n: usize,
    // number of hash functions
    k: usize,

    builder_1: S,
    builder_2: S,
}

impl<S> BloomFilter<S>
where
    S: BuildHasher,
{
    /// Creates a new bloom filter that targets a false positive probability `p` ([0.0, 1.0]) with
    /// an expected number of inserted elements `n`, using `builder_1` and `builder_2` to hash the
    /// data.
    ///
    /// The optimal size of the bit array `m` and number of hash functions `k` are automatically
    /// calculated. See "[Optimal number of hash functions][1]".
    ///
    /// [1]: https://en.wikipedia.org/wiki/Bloom_filter#Optimal_number_of_hash_functions
    pub fn from_fpp_with_hashers(p: f64, n: usize, builder_1: S, builder_2: S) -> Self {
        let m = optimal_required_bits(p, n);
        let k = optimal_number_of_hash_functions(m, n);
        Self::with_hashers(m, k, builder_1, builder_2)
    }

    /// Creates a new bloom filter with a predetermined bit array size `m` and number of hash
    /// functions `k`, using `builder_1` and `builder_2` to hash the data.
    pub fn with_hashers(m: usize, k: usize, builder_1: S, builder_2: S) -> Self {
        Self {
            bits: BitVec::from_elem(m, false),
            m,
            n: 0,
            k,
            builder_1,
            builder_2,
        }
    }

    /// Tests whether an element may be in the filter or definitely not in the filter.
    ///
    /// Remember that false positives can occur, meaning that if this returns `true`, there is only
    /// a possibility that the element is in the filter. If this returns `false`, the element is
    /// definitely not in the filter.
    pub fn contains<H: Hash + ?Sized>(&self, key: &H) -> bool {
        let hasher = self.build_hasher(key);

        for hash in hasher.take(self.k) {
            let i = (hash as usize) % self.m;

            if !self.bits[i] {
                return false;
            }
        }

        true
    }

    /// Adds a value to the bloom filter.
    ///
    /// Returns whether the value is already (maybe) in the filter or not. Duplicate values do not
    /// affect the load factor.
    pub fn insert<H: Hash + ?Sized>(&mut self, key: &H) -> bool {
        let mut present = true;

        let hasher = self.build_hasher(key);

        for hash in hasher.take(self.k) {
            let i = (hash as usize) % self.m;

            if !self.bits[i] {
                present = false;
                self.bits.set(i, true);
            }
        }

        if !present {
            self.n += 1;
        }

        !present
    }

    fn build_hasher<H>(&self, key: &H) -> DoubleHasher
    where
        H: Hash + ?Sized,
    {
        DoubleHasher::new(key, &self.builder_1, &self.builder_2)
    }
}

// Calculates the optimal size of the bit array given a target false positive probability `p`
// ([0.0, 1.0]) and the expected number of inserted elements `n`.
fn optimal_required_bits(p: f64, n: usize) -> usize {
    let ln_2 = f64::consts::LN_2;
    let n = n as f64;
    let m = -(n * p.ln()) / (ln_2 * ln_2);
    m.ceil() as usize
}

// Calculates the optimal number of hash functions given the size of the bit array `m` and the
// expected number of inserted elements `n`.
fn optimal_number_of_hash_functions(m: usize, n: usize) -> usize {
    let m = m as f64;
    let n = n as f64;
    let k = m / n * f64::consts::LN_2;
    k.ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let mut filter = BloomFilter::from_fpp_with_hashers(
            0.0001,
            64,
            DefaultHashBuilder::new(),
            DefaultHashBuilder::new(),
        );

        filter.insert("a");
        filter.insert("b");

        assert!(filter.contains("a"));
        assert!(filter.contains("b"));
        assert!(!filter.contains("c"));
    }

    #[test]
    fn test_optimal_required_bits() {
        let p = 0.01;
        let n = 128;
        let m = optimal_required_bits(p, n);
        assert_eq!(m, 1227);
    }

    #[test]
    fn test_optimal_number_of_hash_functions() {
        let m = 1227;
        let n = 128;
        let k = optimal_number_of_hash_functions(m, n);
        assert_eq!(k, 7);
    }
}
