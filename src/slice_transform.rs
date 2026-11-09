use crate::ffi::KeyRange;

/// A prefix extractor, which enables prefix bloom filters and prefix seeks.
/// See [`Options::set_prefix_extractor`](crate::Options::set_prefix_extractor).
pub struct SliceTransform {
    pub(crate) inner: Box<RustSliceTransform>,
}

impl SliceTransform {
    /// Creates a prefix extractor from a function that returns the prefix of
    /// a key. The result must be a subslice of the key, which is usually the
    /// key's actual prefix.
    ///
    /// `in_domain` tells which keys have a prefix. If it is `None`, `transform`
    /// must accept all keys. Panics abort the process, since RocksDB calls
    /// these from C++.
    pub fn create<F>(
        name: impl Into<String>,
        transform: F,
        in_domain: Option<fn(&[u8]) -> bool>,
    ) -> Self
    where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static,
    {
        Self {
            inner: Box::new(RustSliceTransform {
                name: name.into(),
                transform: Box::new(transform),
                in_domain,
            }),
        }
    }
}

type TransformFn = dyn Fn(&[u8]) -> &[u8] + Send + Sync;

/// The Rust side of a prefix extractor, called from C++.
pub(crate) struct RustSliceTransform {
    name: String,
    transform: Box<TransformFn>,
    in_domain: Option<fn(&[u8]) -> bool>,
}

impl RustSliceTransform {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Returns where the transformed key is within `key`, since C++ can't
    /// tell that the result borrows from `key`.
    pub(crate) fn transform(&self, key: &[u8]) -> KeyRange {
        let prefix = (self.transform)(key);
        if prefix.is_empty() {
            return KeyRange { start: 0, len: 0 };
        }
        let start = (prefix.as_ptr() as usize).wrapping_sub(key.as_ptr() as usize);
        assert!(
            start <= key.len() && prefix.len() <= key.len() - start,
            "slice transform `{}` returned a slice outside of its input",
            self.name,
        );
        KeyRange {
            start,
            len: prefix.len(),
        }
    }

    pub(crate) fn in_domain(&self, key: &[u8]) -> bool {
        self.in_domain.is_none_or(|in_domain| in_domain(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: usize, len: usize) -> KeyRange {
        KeyRange { start, len }
    }

    #[test]
    fn callbacks() {
        let t = SliceTransform::create("first_two", |key| &key[..2], Some(|key| key.len() >= 2));
        assert_eq!(t.inner.name(), "first_two");
        assert_eq!(t.inner.transform(b"abc"), range(0, 2));
        assert!(t.inner.in_domain(b"ab"));
        assert!(!t.inner.in_domain(b"a"));
    }

    #[test]
    fn in_domain_defaults_to_all_keys() {
        let prefix_len = 3;
        let t =
            SliceTransform::create("capped", move |key| &key[..key.len().min(prefix_len)], None);
        assert!(t.inner.in_domain(b""));
        assert_eq!(t.inner.transform(b""), range(0, 0));
        assert_eq!(t.inner.transform(b"ab"), range(0, 2));
        assert_eq!(t.inner.transform(b"abcd"), range(0, 3));
    }

    #[test]
    fn subslices() {
        let t = SliceTransform::create("middle", |key| &key[1..3], None);
        assert_eq!(t.inner.transform(b"abcd"), range(1, 2));
        let t = SliceTransform::create("empty", |_| &[], None);
        assert_eq!(t.inner.transform(b"abcd"), range(0, 0));
    }

    #[test]
    #[should_panic(expected = "outside of its input")]
    fn outside_of_input() {
        let t = SliceTransform::create("static", |_| b"static", None);
        t.inner.transform(b"abcd");
    }
}

#[cfg(test)]
mod db_tests {
    use super::*;
    use crate::{Options, ReadOptions, test_utils::TestDb};
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    #[test]
    fn prefix_extractor() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create(
            "rocksdb_rs.test",
            move |key| {
                counter.fetch_add(1, Ordering::Relaxed);
                &key[..key.len().min(2)]
            },
            None,
        ));
        let db = TestDb::with_cf_options([("a", opts)]);
        let cf = db.cf("a");
        for key in ["aa1", "aa2", "ab1", "b"] {
            db.put_cf(cf, key, b"v").unwrap();
        }
        db.flush_cf(cf).unwrap();

        let keys_from_aa = |opts| {
            let mut iter = db.raw_iterator_cf_opt(cf, opts);
            iter.seek(b"aa");
            let mut keys = Vec::new();
            while let Some(key) = iter.key() {
                keys.push(String::from_utf8(key.to_vec()).unwrap());
                iter.next();
            }
            keys
        };
        let mut opts = ReadOptions::default();
        opts.set_prefix_same_as_start(true);
        assert_eq!(keys_from_aa(opts), ["aa1", "aa2"]);
        let mut opts = ReadOptions::default();
        opts.set_total_order_seek(true);
        assert_eq!(keys_from_aa(opts), ["aa1", "aa2", "ab1", "b"]);
        assert!(calls.load(Ordering::Relaxed) > 0);

        // RocksDB records the name in its options file.
        let options_file = fs::read_dir(db.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("OPTIONS-")
            })
            .unwrap();
        assert!(
            fs::read_to_string(options_file)
                .unwrap()
                .contains("prefix_extractor=rocksdb_rs.test")
        );
    }

    #[test]
    fn in_domain() {
        let mut opts = Options::default();
        opts.set_prefix_extractor(SliceTransform::create(
            "first_two",
            // Out of domain keys are never queried against the filter.
            |key| &key[..2],
            Some(|key| key.len() >= 2),
        ));
        let db = TestDb::with_cf_options([("a", opts)]);
        let cf = db.cf("a");
        db.put_cf(cf, b"a", b"short").unwrap();
        db.put_cf(cf, b"abc", b"long").unwrap();
        db.flush_cf(cf).unwrap();
        assert_eq!(db.get_cf(cf, b"a").unwrap(), Some(b"short".to_vec()));
        assert_eq!(db.get_cf(cf, b"abc").unwrap(), Some(b"long".to_vec()));
    }
}
