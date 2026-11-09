use crate::{Error, error::ffi_try, ffi};
use cxx::UniquePtr;
use std::pin::Pin;

/// A low-level iterator over a column family, which can be repositioned.
///
/// The iterator starts out invalid. Position it with one of the seek methods
/// first.
///
/// It borrows the DB, which stays open as long as the iterator is alive:
///
/// ```compile_fail,E0505
/// # use rocksdb_rs::{DB, DEFAULT_COLUMN_FAMILY_NAME, ReadOptions};
/// # let dir = tempfile::tempdir().unwrap();
/// let db = DB::open_default(dir.path()).unwrap();
/// let cf = db.cf_handle(DEFAULT_COLUMN_FAMILY_NAME).unwrap();
/// let mut iter = db.raw_iterator_cf_opt(cf, ReadOptions::default());
/// drop(db);
/// iter.seek_to_first();
/// ```
pub struct DBRawIterator<'a> {
    inner: UniquePtr<ffi::DbIterator<'a>>,
}

impl<'a> DBRawIterator<'a> {
    pub(crate) fn new(inner: UniquePtr<ffi::DbIterator<'a>>) -> Self {
        assert!(!inner.is_null());
        Self { inner }
    }

    fn pin_mut(&mut self) -> Pin<&mut ffi::DbIterator<'a>> {
        self.inner.pin_mut()
    }

    /// Returns true if the iterator is positioned at an entry. Otherwise,
    /// either the end was reached or there was an error, see [`Self::status`].
    pub fn valid(&self) -> bool {
        self.inner.valid()
    }

    /// Returns the error that invalidated the iterator, if any.
    pub fn status(&self) -> Result<(), Error> {
        ffi_try(|s| self.inner.status(s))
    }

    pub fn seek_to_first(&mut self) {
        self.pin_mut().seek_to_first();
    }

    pub fn seek_to_last(&mut self) {
        self.pin_mut().seek_to_last();
    }

    /// Seeks to the first key at or after `key`.
    pub fn seek(&mut self, key: impl AsRef<[u8]>) {
        self.pin_mut().seek(key.as_ref());
    }

    /// Seeks to the last key at or before `key`.
    pub fn seek_for_prev(&mut self, key: impl AsRef<[u8]>) {
        self.pin_mut().seek_for_prev(key.as_ref());
    }

    /// Moves to the next entry. No-op if the iterator is not valid.
    pub fn next(&mut self) {
        self.pin_mut().next();
    }

    /// Moves to the previous entry. No-op if the iterator is not valid.
    pub fn prev(&mut self) {
        self.pin_mut().prev();
    }

    pub fn key(&self) -> Option<&[u8]> {
        self.valid().then(|| self.inner.key())
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.valid().then(|| self.inner.value())
    }

    pub fn item(&self) -> Option<(&[u8], &[u8])> {
        self.valid().then(|| (self.inner.key(), self.inner.value()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
}

/// Where a [`DBIterator`] starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorMode<'a> {
    /// Forward from the first key.
    Start,
    /// Backward from the last key.
    End,
    /// From the given key, in the given direction. Starts at the first key at
    /// or after it when going forward, and at or before it otherwise.
    From(&'a [u8], Direction),
}

/// An iterator over the key-value pairs of a column family.
pub struct DBIterator<'a> {
    raw: DBRawIterator<'a>,
    direction: Direction,
    done: bool,
}

impl<'a> DBIterator<'a> {
    pub(crate) fn new(mut raw: DBRawIterator<'a>, mode: IteratorMode) -> Self {
        let direction = match mode {
            IteratorMode::Start => {
                raw.seek_to_first();
                Direction::Forward
            }
            IteratorMode::End => {
                raw.seek_to_last();
                Direction::Reverse
            }
            IteratorMode::From(key, Direction::Forward) => {
                raw.seek(key);
                Direction::Forward
            }
            IteratorMode::From(key, Direction::Reverse) => {
                raw.seek_for_prev(key);
                Direction::Reverse
            }
        };
        Self {
            raw,
            direction,
            done: false,
        }
    }
}

impl Iterator for DBIterator<'_> {
    type Item = Result<(Box<[u8]>, Box<[u8]>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let Some((key, value)) = self.raw.item() else {
            self.done = true;
            return self.raw.status().err().map(Err);
        };
        let item = (Box::from(key), Box::from(value));
        match self.direction {
            Direction::Forward => self.raw.next(),
            Direction::Reverse => self.raw.prev(),
        }
        Some(Ok(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_COLUMN_FAMILY_NAME, ErrorKind, ReadOptions, test_utils::TestDb};
    use std::thread;

    fn test_db() -> TestDb {
        let db = TestDb::new(&[]);
        for (key, value) in [("k1", "v1"), ("k2", "v2"), ("k3", "v3")] {
            db.put(key, value).unwrap();
        }
        db
    }

    fn raw_iter(db: &TestDb, opts: ReadOptions) -> DBRawIterator<'_> {
        db.raw_iterator_cf_opt(db.cf(DEFAULT_COLUMN_FAMILY_NAME), opts)
    }

    fn remaining_keys(iter: &mut DBRawIterator) -> Vec<Vec<u8>> {
        let mut keys = Vec::new();
        while let Some(key) = iter.key() {
            keys.push(key.to_vec());
            iter.next();
        }
        iter.status().unwrap();
        keys
    }

    #[test]
    fn raw_iterator() {
        let db = test_db();
        let mut iter = raw_iter(&db, ReadOptions::default());
        assert!(!iter.valid());
        assert_eq!(iter.item(), None);
        // Moving an invalid iterator is a no-op.
        iter.next();
        iter.prev();
        assert!(!iter.valid());

        iter.seek_to_first();
        assert_eq!(iter.item(), Some((&b"k1"[..], &b"v1"[..])));
        iter.next();
        assert_eq!(iter.key(), Some(&b"k2"[..]));
        assert_eq!(iter.value(), Some(&b"v2"[..]));
        iter.next();
        iter.next();
        assert!(!iter.valid());
        assert_eq!(iter.value(), None);
        iter.status().unwrap();

        iter.seek_to_last();
        assert_eq!(iter.key(), Some(&b"k3"[..]));
        iter.prev();
        assert_eq!(iter.key(), Some(&b"k2"[..]));

        iter.seek(b"k15");
        assert_eq!(iter.key(), Some(&b"k2"[..]));
        iter.seek(b"k4");
        assert!(!iter.valid());
        iter.seek_for_prev(b"k25");
        assert_eq!(iter.key(), Some(&b"k2"[..]));
        iter.seek_for_prev(b"k0");
        assert!(!iter.valid());
        iter.status().unwrap();
    }

    #[test]
    fn upper_bound() {
        let db = test_db();
        let mut opts = ReadOptions::default();
        opts.set_iterate_upper_bound(b"k3");
        let mut iter = raw_iter(&db, opts);
        iter.seek_to_first();
        assert_eq!(remaining_keys(&mut iter), [b"k1", b"k2"]);
    }

    #[test]
    fn max_skippable_internal_keys() {
        let db = TestDb::new(&[]);
        for i in 0..10u8 {
            db.put([i], [i]).unwrap();
        }
        for i in 1..9u8 {
            db.delete([i]).unwrap();
        }
        let mut opts = ReadOptions::default();
        opts.set_max_skippable_internal_keys(2);
        let mut iter = raw_iter(&db, opts);
        iter.seek_to_first();
        assert_eq!(iter.key(), Some(&[0][..]));
        iter.next();
        assert!(!iter.valid());
        assert_eq!(iter.status().unwrap_err().kind(), ErrorKind::Incomplete);
    }

    #[test]
    fn iterator_modes() {
        let db = test_db();
        let keys = |mode| -> Vec<_> {
            db.iterator(mode)
                .map(|item| item.unwrap().0.into_vec())
                .collect()
        };
        assert_eq!(keys(IteratorMode::Start), [b"k1", b"k2", b"k3"]);
        assert_eq!(keys(IteratorMode::End), [b"k3", b"k2", b"k1"]);
        assert_eq!(
            keys(IteratorMode::From(b"k15", Direction::Forward)),
            [b"k2", b"k3"]
        );
        assert_eq!(
            keys(IteratorMode::From(b"k25", Direction::Reverse)),
            [b"k2", b"k1"]
        );

        let (key, value) = db.iterator(IteratorMode::Start).next().unwrap().unwrap();
        assert_eq!((&*key, &*value), (&b"k1"[..], &b"v1"[..]));
    }

    #[test]
    fn iterator_error() {
        let db = TestDb::new(&[]);
        let other = TestDb::new(&["a"]);
        let raw = db.raw_iterator_cf_opt(other.cf("a"), ReadOptions::default());
        let mut iter = DBIterator::new(raw, IteratorMode::Start);
        assert_eq!(
            iter.next().unwrap().unwrap_err().kind(),
            ErrorKind::InvalidArgument
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn send_to_another_thread() {
        let db = test_db();
        let mut iter = raw_iter(&db, ReadOptions::default());
        iter.seek_to_first();
        let keys = thread::scope(|s| s.spawn(move || remaining_keys(&mut iter)).join().unwrap());
        assert_eq!(keys.len(), 3);
    }
}
