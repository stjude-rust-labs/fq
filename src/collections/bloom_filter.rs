//! Bloom filter.

#[allow(clippy::module_inception)]
mod bloom_filter;
mod double_hasher;
mod scalable_bloom_filter;

pub use self::scalable_bloom_filter::ScalableBloomFilter;
use self::{bloom_filter::BloomFilter, double_hasher::DoubleHasher};

type DefaultHashBuilder = rapidhash::fast::RandomState;
