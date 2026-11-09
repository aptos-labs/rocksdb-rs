use crate::ffi;
use cxx::UniquePtr;
use std::pin::Pin;

/// Options for reads and iterators.
pub struct ReadOptions {
    pub(crate) inner: UniquePtr<ffi::ReadOptions>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            inner: ffi::new_read_options(),
        }
    }
}

impl ReadOptions {
    fn pin_mut(&mut self) -> Pin<&mut ffi::ReadOptions> {
        self.inner.pin_mut()
    }

    /// Whether blocks read for this operation are added to the block cache.
    /// Enabled by default.
    pub fn fill_cache(&mut self, value: bool) {
        self.pin_mut().fill_cache(value);
    }

    /// Makes iterators stop at keys whose prefix differs from the seek key's.
    /// Requires a prefix extractor.
    pub fn set_prefix_same_as_start(&mut self, value: bool) {
        self.pin_mut().set_prefix_same_as_start(value);
    }

    /// Ignores the prefix extractor, so that iterators see all keys in order.
    pub fn set_total_order_seek(&mut self, value: bool) {
        self.pin_mut().set_total_order_seek(value);
    }

    /// Makes iterators fail with [`ErrorKind::Incomplete`](crate::ErrorKind::Incomplete)
    /// after skipping more than `num` internal keys (e.g. deletions). 0 means
    /// unlimited.
    pub fn set_max_skippable_internal_keys(&mut self, num: u64) {
        self.pin_mut().set_max_skippable_internal_keys(num);
    }

    /// Sets the exclusive upper bound of iterators.
    pub fn set_iterate_upper_bound(&mut self, key: impl AsRef<[u8]>) {
        self.pin_mut().set_iterate_upper_bound(key.as_ref());
    }
}
