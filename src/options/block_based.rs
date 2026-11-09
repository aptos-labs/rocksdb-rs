use crate::{BlockBasedIndexType, Cache, ffi};
use cxx::UniquePtr;
use std::pin::Pin;

/// Options for the block-based table format, the default SST file format.
/// See [`Options::set_block_based_table_factory`](crate::Options::set_block_based_table_factory).
pub struct BlockBasedOptions {
    pub(crate) inner: UniquePtr<ffi::BlockBasedOptions>,
}

impl Default for BlockBasedOptions {
    fn default() -> Self {
        Self {
            inner: ffi::new_block_based_options(),
        }
    }
}

impl BlockBasedOptions {
    fn pin_mut(&mut self) -> Pin<&mut ffi::BlockBasedOptions> {
        self.inner.pin_mut()
    }

    /// Sets the approximate size of user data packed per block.
    pub fn set_block_size(&mut self, size: usize) {
        self.pin_mut().set_block_size(size);
    }

    pub fn set_index_type(&mut self, index_type: BlockBasedIndexType) {
        self.pin_mut().set_index_type(index_type);
    }

    /// Partitions filters like indexes. Requires
    /// [`BlockBasedIndexType::TwoLevelIndexSearch`].
    pub fn set_partition_filters(&mut self, value: bool) {
        self.pin_mut().set_partition_filters(value);
    }

    /// Puts index and filter blocks in the block cache instead of keeping them
    /// in memory with the table readers.
    pub fn set_cache_index_and_filter_blocks(&mut self, value: bool) {
        self.pin_mut().set_cache_index_and_filter_blocks(value);
    }

    /// Pins index and filter blocks of L0 files in the block cache.
    pub fn set_pin_l0_filter_and_index_blocks_in_cache(&mut self, value: bool) {
        self.pin_mut()
            .set_pin_l0_filter_and_index_blocks_in_cache(value);
    }

    pub fn set_block_cache(&mut self, cache: &Cache) {
        self.pin_mut().set_block_cache(&cache.inner);
    }

    /// Uses a full bloom filter with the given number of bits per key.
    pub fn set_bloom_filter(&mut self, bits_per_key: f64) {
        self.pin_mut().set_bloom_filter(bits_per_key);
    }

    /// Uses bloom filters for levels below `bloom_before_level` and ribbon
    /// filters, which save memory at some CPU cost, for the rest.
    pub fn set_hybrid_ribbon_filter(
        &mut self,
        bloom_equivalent_bits_per_key: f64,
        bloom_before_level: i32,
    ) {
        self.pin_mut()
            .set_hybrid_ribbon_filter(bloom_equivalent_bits_per_key, bloom_before_level);
    }

    /// Adds whole keys to the filter, in addition to prefixes if a prefix
    /// extractor is set. Enabled by default.
    pub fn set_whole_key_filtering(&mut self, value: bool) {
        self.pin_mut().set_whole_key_filtering(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setters() {
        let mut opts = BlockBasedOptions::default();
        opts.set_block_size(4096);
        opts.set_index_type(BlockBasedIndexType::TwoLevelIndexSearch);
        opts.set_partition_filters(true);
        opts.set_cache_index_and_filter_blocks(true);
        opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        opts.set_block_cache(&Cache::new_hyper_clock_cache(1 << 20, 0));
        opts.set_bloom_filter(10.0);
        opts.set_hybrid_ribbon_filter(10.0, 1);
        opts.set_whole_key_filtering(false);
    }
}
