use crate::ffi;
use cxx::UniquePtr;

/// Options for writes.
pub struct WriteOptions {
    pub(crate) inner: UniquePtr<ffi::WriteOptions>,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            inner: ffi::new_write_options(),
        }
    }
}

impl WriteOptions {
    /// Syncs the WAL before acknowledging the write. Without it, a machine
    /// crash may lose recent writes, but a process crash does not.
    pub fn set_sync(&mut self, value: bool) {
        self.inner.pin_mut().set_sync(value);
    }
}
