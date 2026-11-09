use crate::{ColumnFamily, Error, error::ffi_try, ffi};
use cxx::UniquePtr;
use std::pin::Pin;

/// A batch of updates applied atomically by [`DB::write`](crate::DB::write).
///
/// Updates are applied in the order they are added.
pub struct WriteBatch {
    pub(crate) inner: UniquePtr<ffi::WriteBatch>,
}

impl Default for WriteBatch {
    fn default() -> Self {
        Self {
            inner: ffi::new_write_batch(),
        }
    }
}

impl WriteBatch {
    fn pin_mut(&mut self) -> Pin<&mut ffi::WriteBatch> {
        self.inner.pin_mut()
    }

    /// Adds a put to the default column family.
    pub fn put(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<(), Error> {
        ffi_try(|s| self.pin_mut().put(key.as_ref(), value.as_ref(), s))
    }

    pub fn put_cf(
        &mut self,
        cf: &ColumnFamily,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        ffi_try(|s| self.pin_mut().put_cf(cf, key.as_ref(), value.as_ref(), s))
    }

    /// Adds a deletion from the default column family.
    pub fn delete(&mut self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        ffi_try(|s| self.pin_mut().del(key.as_ref(), s))
    }

    pub fn delete_cf(&mut self, cf: &ColumnFamily, key: impl AsRef<[u8]>) -> Result<(), Error> {
        ffi_try(|s| self.pin_mut().del_cf(cf, key.as_ref(), s))
    }

    /// Returns the number of updates.
    pub fn len(&self) -> usize {
        self.inner.count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the size of the serialized batch.
    pub fn size_in_bytes(&self) -> usize {
        self.inner.data_size()
    }

    pub fn clear(&mut self) {
        self.pin_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_and_size() {
        let mut batch = WriteBatch::default();
        assert!(batch.is_empty());
        let empty_size = batch.size_in_bytes();

        batch.put(b"k1", b"v1").unwrap();
        batch.delete(b"k2").unwrap();
        assert_eq!(batch.len(), 2);
        assert!(batch.size_in_bytes() > empty_size);

        batch.clear();
        assert!(batch.is_empty());
        assert_eq!(batch.size_in_bytes(), empty_size);
    }
}
