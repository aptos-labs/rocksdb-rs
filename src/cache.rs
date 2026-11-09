use crate::ffi;
use cxx::SharedPtr;

/// A block cache, which can be shared by multiple column families and DBs.
///
/// Cloning is cheap and refers to the same cache.
#[derive(Clone)]
pub struct Cache {
    pub(crate) inner: SharedPtr<ffi::Cache>,
}

impl Cache {
    /// Creates a HyperClockCache, which scales better than an LRU cache under
    /// concurrent access. With `estimated_entry_charge` set to 0, the cache
    /// sizes its table automatically.
    pub fn new_hyper_clock_cache(capacity: usize, estimated_entry_charge: usize) -> Self {
        Self {
            inner: ffi::new_hyper_clock_cache(capacity, estimated_entry_charge),
        }
    }
}
