use crate::{Error, ffi};
use cxx::SharedPtr;

/// The environment through which RocksDB accesses the file system and runs
/// background jobs.
///
/// Cloning is cheap and refers to the same environment.
#[derive(Clone)]
pub struct Env {
    pub(crate) inner: SharedPtr<ffi::Env>,
}

impl Env {
    /// Returns the process-wide default environment, which is shared by all
    /// DBs that don't set one explicitly. Currently this never fails.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: ffi::default_env(),
        })
    }

    /// Sets the number of threads for flushes.
    pub fn set_high_priority_background_threads(&mut self, num: i32) {
        ffi::set_high_priority_background_threads(&self.inner, num);
    }

    /// Sets the number of threads for compactions.
    pub fn set_low_priority_background_threads(&mut self, num: i32) {
        ffi::set_low_priority_background_threads(&self.inner, num);
    }
}
