pub use crate::ffi::ColumnFamily;
use crate::{
    DBIterator, DBRawIterator, Error, ErrorKind, IteratorMode, Options, ReadOptions, WriteBatch,
    WriteOptions, error::ffi_try, ffi,
};
use cxx::UniquePtr;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub const DEFAULT_COLUMN_FAMILY_NAME: &str = "default";

static DEFAULT_READ_OPTIONS: LazyLock<ReadOptions> = LazyLock::new(ReadOptions::default);
static DEFAULT_WRITE_OPTIONS: LazyLock<WriteOptions> = LazyLock::new(WriteOptions::default);

/// The name and options of a column family to open.
pub struct ColumnFamilyDescriptor {
    name: String,
    options: Options,
}

impl ColumnFamilyDescriptor {
    pub fn new(name: impl Into<String>, options: Options) -> Self {
        Self {
            name: name.into(),
            options,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// An open RocksDB database. It is safe to use from multiple threads.
///
/// The default column family is always opened. Unless it is passed to one of
/// the `open_cf_*` functions, it uses the column family options in `opts`.
pub struct DB {
    inner: UniquePtr<ffi::Db>,
    // Indexes into the column families held by `inner`.
    column_families: HashMap<String, usize>,
    path: PathBuf,
}

impl DB {
    /// Opens a DB with the default column family, creating it if missing.
    pub fn open_default(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        Self::open(&opts, path)
    }

    /// Opens a DB with the default column family.
    pub fn open(opts: &Options, path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::open_cf_descriptors(opts, path, [])
    }

    pub fn open_cf_descriptors(
        opts: &Options,
        path: impl AsRef<Path>,
        cfs: impl IntoIterator<Item = ColumnFamilyDescriptor>,
    ) -> Result<Self, Error> {
        Self::open_impl(opts, path.as_ref(), cfs, OpenMode::ReadWrite)
    }

    /// Opens a DB that can only be read. It does not see writes made by other
    /// instances after it is opened.
    pub fn open_cf_descriptors_read_only(
        opts: &Options,
        path: impl AsRef<Path>,
        cfs: impl IntoIterator<Item = ColumnFamilyDescriptor>,
        error_if_wal_file_exists: bool,
    ) -> Result<Self, Error> {
        Self::open_impl(
            opts,
            path.as_ref(),
            cfs,
            OpenMode::ReadOnly {
                error_if_wal_file_exists,
            },
        )
    }

    /// Opens a secondary instance of the DB at `primary_path`, which can only
    /// be read. `secondary_path` stores the secondary instance's info log.
    pub fn open_cf_descriptors_as_secondary(
        opts: &Options,
        primary_path: impl AsRef<Path>,
        secondary_path: impl AsRef<Path>,
        cfs: impl IntoIterator<Item = ColumnFamilyDescriptor>,
    ) -> Result<Self, Error> {
        Self::open_impl(
            opts,
            primary_path.as_ref(),
            cfs,
            OpenMode::Secondary(secondary_path.as_ref()),
        )
    }

    fn open_impl(
        opts: &Options,
        path: &Path,
        cfs: impl IntoIterator<Item = ColumnFamilyDescriptor>,
        mode: OpenMode,
    ) -> Result<Self, Error> {
        let mut cfs: Vec<_> = cfs.into_iter().collect();
        if !cfs.iter().any(|cf| cf.name == DEFAULT_COLUMN_FAMILY_NAME) {
            cfs.push(ColumnFamilyDescriptor::new(
                DEFAULT_COLUMN_FAMILY_NAME,
                opts.clone(),
            ));
        }
        let mut descriptors = ffi::new_column_family_descriptors();
        for cf in &cfs {
            descriptors.pin_mut().add(&cf.name, &cf.options.inner);
        }

        let (mode, secondary_path, error_if_wal_file_exists) = match mode {
            OpenMode::ReadWrite => (ffi::OpenMode::ReadWrite, None, false),
            OpenMode::ReadOnly {
                error_if_wal_file_exists,
            } => (ffi::OpenMode::ReadOnly, None, error_if_wal_file_exists),
            OpenMode::Secondary(path) => (ffi::OpenMode::Secondary, Some(path), false),
        };
        let primary_path = path_bytes(path)?;
        let secondary_path = secondary_path.map(path_bytes).transpose()?;
        let inner = ffi_try(|s| {
            ffi::open_db(
                &opts.inner,
                primary_path,
                &descriptors,
                mode,
                secondary_path.unwrap_or_default(),
                error_if_wal_file_exists,
                s,
            )
        })?;
        assert!(!inner.is_null());

        let column_families = cfs
            .into_iter()
            .enumerate()
            .map(|(index, cf)| (cf.name, index))
            .collect();
        Ok(Self {
            inner,
            column_families,
            path: path.to_owned(),
        })
    }

    /// Lists the column families of the DB at `path`.
    pub fn list_cf(opts: &Options, path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
        let path = path_bytes(path.as_ref())?;
        ffi_try(|s| ffi::list_column_families(&opts.inner, path, s))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the handle to an open column family.
    pub fn cf_handle(&self, name: &str) -> Option<&ColumnFamily> {
        let index = *self.column_families.get(name)?;
        Some(self.inner.column_family(index))
    }

    fn default_cf(&self) -> &ColumnFamily {
        self.cf_handle(DEFAULT_COLUMN_FAMILY_NAME)
            .expect("the default column family is always open")
    }

    /// Reads a key from the default column family.
    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, Error> {
        self.get_cf(self.default_cf(), key)
    }

    pub fn get_cf(
        &self,
        cf: &ColumnFamily,
        key: impl AsRef<[u8]>,
    ) -> Result<Option<Vec<u8>>, Error> {
        self.get_cf_opt(cf, key, &DEFAULT_READ_OPTIONS)
    }

    pub fn get_cf_opt(
        &self,
        cf: &ColumnFamily,
        key: impl AsRef<[u8]>,
        opts: &ReadOptions,
    ) -> Result<Option<Vec<u8>>, Error> {
        let mut value = Vec::new();
        let found = ffi_try(|s| self.inner.get(&opts.inner, cf, key.as_ref(), &mut value, s))?;
        Ok(found.then_some(value))
    }

    /// Writes a key to the default column family.
    pub fn put(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<(), Error> {
        self.put_cf(self.default_cf(), key, value)
    }

    pub fn put_cf(
        &self,
        cf: &ColumnFamily,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        ffi_try(|s| {
            self.inner.put(
                &DEFAULT_WRITE_OPTIONS.inner,
                cf,
                key.as_ref(),
                value.as_ref(),
                s,
            )
        })
    }

    /// Deletes a key from the default column family.
    pub fn delete(&self, key: impl AsRef<[u8]>) -> Result<(), Error> {
        self.delete_cf(self.default_cf(), key)
    }

    pub fn delete_cf(&self, cf: &ColumnFamily, key: impl AsRef<[u8]>) -> Result<(), Error> {
        ffi_try(|s| {
            self.inner
                .del(&DEFAULT_WRITE_OPTIONS.inner, cf, key.as_ref(), s)
        })
    }

    pub fn write(&self, batch: WriteBatch) -> Result<(), Error> {
        self.write_opt(batch, &DEFAULT_WRITE_OPTIONS)
    }

    pub fn write_opt(&self, mut batch: WriteBatch, opts: &WriteOptions) -> Result<(), Error> {
        ffi_try(|s| self.inner.write(&opts.inner, batch.inner.pin_mut(), s))
    }

    /// Flushes the memtable of the default column family.
    pub fn flush(&self) -> Result<(), Error> {
        self.flush_cf(self.default_cf())
    }

    pub fn flush_cf(&self, cf: &ColumnFamily) -> Result<(), Error> {
        ffi_try(|s| self.inner.flush(cf, s))
    }

    /// Returns the value of an integer property like
    /// `rocksdb.estimate-num-keys`, or `None` if the property is unknown or
    /// not an integer.
    pub fn property_int_value_cf(
        &self,
        cf: &ColumnFamily,
        name: &str,
    ) -> Result<Option<u64>, Error> {
        let mut value = 0;
        let found = ffi_try(|s| self.inner.get_int_property(cf, name, &mut value, s))?;
        Ok(found.then_some(value))
    }

    /// Iterates over the default column family.
    pub fn iterator(&self, mode: IteratorMode) -> DBIterator<'_> {
        let raw = self.raw_iterator_cf_opt(self.default_cf(), ReadOptions::default());
        DBIterator::new(raw, mode)
    }

    pub fn raw_iterator_cf_opt(&self, cf: &ColumnFamily, opts: ReadOptions) -> DBRawIterator<'_> {
        DBRawIterator::new(self.inner.new_iterator(opts.inner, cf))
    }

    pub(crate) fn create_checkpoint(&self, path: &Path) -> Result<(), Error> {
        let path = path_bytes(path)?;
        ffi_try(|s| self.inner.create_checkpoint(path, s))
    }
}

enum OpenMode<'a> {
    ReadWrite,
    ReadOnly { error_if_wal_file_exists: bool },
    Secondary(&'a Path),
}

/// Converts a path to what RocksDB expects: raw bytes on Unix, and UTF-8
/// elsewhere.
fn path_bytes(path: &Path) -> Result<&[u8], Error> {
    #[cfg(unix)]
    let bytes = std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str());
    #[cfg(not(unix))]
    let bytes = path
        .to_str()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidArgument,
                format!("path is not valid UTF-8: {}", path.display()),
            )
        })?
        .as_bytes();

    // RocksDB would silently truncate the path.
    if bytes.contains(&0) {
        return Err(Error::new(
            ErrorKind::InvalidArgument,
            format!("path contains a NUL byte: {}", path.display()),
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BlockBasedIndexType, BlockBasedOptions, Cache, DBCompressionType, Env, SliceTransform,
        test_utils::{TestDb, cf_descriptors, create_options},
    };
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn put_get_delete() {
        let db = TestDb::new(&[]);
        assert_eq!(db.get(b"k").unwrap(), None);
        db.put(b"k", b"v").unwrap();
        assert_eq!(db.get(b"k").unwrap().as_deref(), Some(&b"v"[..]));
        db.delete(b"k").unwrap();
        assert_eq!(db.get(b"k").unwrap(), None);
    }

    #[test]
    fn empty_and_large_values() {
        let db = TestDb::new(&[]);
        db.put(b"empty", b"").unwrap();
        assert_eq!(db.get(b"empty").unwrap(), Some(vec![]));

        let large = vec![7; 4 << 20];
        db.put(b"large", &large).unwrap();
        db.flush().unwrap();
        assert_eq!(db.get(b"large").unwrap(), Some(large));
    }

    #[test]
    fn open_default() {
        let dir = TempDir::new().unwrap();
        let db = DB::open_default(dir.path()).unwrap();
        db.put(b"k", b"v").unwrap();
        assert_eq!(db.path(), dir.path());
    }

    #[test]
    fn column_families() {
        let db = TestDb::new(&["a", "b"]);
        assert!(db.cf_handle("missing").is_none());

        db.put_cf(db.cf("a"), b"k", b"a").unwrap();
        db.put_cf(db.cf("b"), b"k", b"b").unwrap();
        assert_eq!(db.get(b"k").unwrap(), None);
        assert_eq!(db.get_cf(db.cf("a"), b"k").unwrap(), Some(b"a".to_vec()));
        assert_eq!(db.get_cf(db.cf("b"), b"k").unwrap(), Some(b"b".to_vec()));

        db.delete_cf(db.cf("a"), b"k").unwrap();
        assert_eq!(db.get_cf(db.cf("a"), b"k").unwrap(), None);
        assert_eq!(db.get_cf(db.cf("b"), b"k").unwrap(), Some(b"b".to_vec()));
    }

    #[test]
    fn default_column_family_is_always_open() {
        let db = TestDb::new(&["a"]);
        assert!(db.cf_handle(DEFAULT_COLUMN_FAMILY_NAME).is_some());

        let db = TestDb::new(&[DEFAULT_COLUMN_FAMILY_NAME, "a"]);
        db.put(b"k", b"v").unwrap();
        assert_eq!(
            db.get_cf(db.cf(DEFAULT_COLUMN_FAMILY_NAME), b"k").unwrap(),
            Some(b"v".to_vec())
        );
    }

    #[test]
    fn default_column_family_uses_db_options() {
        let mut opts = create_options();
        opts.set_prefix_extractor(SliceTransform::create(
            "first_byte",
            |key| &key[..key.len().min(1)],
            None,
        ));
        for cfs in [vec![], cf_descriptors(&["a"])] {
            let dir = TempDir::new().unwrap();
            let db = DB::open_cf_descriptors(&opts, dir.path(), cfs).unwrap();
            for key in ["a1", "a2", "b1"] {
                db.put(key, b"v").unwrap();
            }

            let mut read_opts = ReadOptions::default();
            read_opts.set_prefix_same_as_start(true);
            let mut iter = db.raw_iterator_cf_opt(db.default_cf(), read_opts);
            iter.seek(b"a");
            let mut keys = Vec::new();
            while let Some(key) = iter.key() {
                keys.push(key.to_vec());
                iter.next();
            }
            assert_eq!(keys, [b"a1", b"a2"]);
        }
    }

    #[test]
    fn list_cf() {
        let dir = TestDb::new(&["a", "b"]).close();
        let mut cfs = DB::list_cf(&Options::default(), dir.path()).unwrap();
        cfs.sort();
        assert_eq!(cfs, ["a", "b", DEFAULT_COLUMN_FAMILY_NAME]);

        let empty = TempDir::new().unwrap();
        assert!(DB::list_cf(&Options::default(), empty.path().join("missing")).is_err());
    }

    #[test]
    fn reopen() {
        let db = TestDb::new(&["a"]);
        db.put_cf(db.cf("a"), b"k", b"v").unwrap();
        let dir = db.close();

        let db = DB::open_cf_descriptors(&Options::default(), dir.path(), cf_descriptors(&["a"]))
            .unwrap();
        assert_eq!(
            db.get_cf(db.cf_handle("a").unwrap(), b"k").unwrap(),
            Some(b"v".to_vec())
        );
    }

    #[test]
    fn open_errors() {
        let dir = TempDir::new().unwrap();
        let err = DB::open(&Options::default(), dir.path()).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidArgument);

        let dir = TestDb::new(&[]).close();
        let err = DB::open_cf_descriptors(&Options::default(), dir.path(), cf_descriptors(&["a"]))
            .err()
            .unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidArgument);

        // Locked by the first instance.
        let db = TestDb::new(&[]);
        let err = DB::open(&Options::default(), db.path()).err().unwrap();
        assert_eq!(err.kind(), ErrorKind::IOError);
    }

    #[test]
    fn invalid_path() {
        let err = DB::open_default("invalid\0path").err().unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    }

    #[test]
    fn read_only() {
        let db = TestDb::new(&["a"]);
        db.put_cf(db.cf("a"), b"k", b"v").unwrap();
        let dir = db.close();

        let db = DB::open_cf_descriptors_read_only(
            &Options::default(),
            dir.path(),
            cf_descriptors(&["a"]),
            false,
        )
        .unwrap();
        let cf = db.cf_handle("a").unwrap();
        assert_eq!(db.get_cf(cf, b"k").unwrap(), Some(b"v".to_vec()));
        assert_eq!(
            db.put_cf(cf, b"k", b"v2").unwrap_err().kind(),
            ErrorKind::NotSupported
        );
        assert_eq!(db.flush_cf(cf).unwrap_err().kind(), ErrorKind::NotSupported);
        drop(db);

        // The write above is only in the WAL.
        assert!(
            DB::open_cf_descriptors_read_only(
                &Options::default(),
                dir.path(),
                cf_descriptors(&["a"]),
                true
            )
            .is_err()
        );
    }

    #[test]
    fn secondary() {
        let primary = TestDb::new(&["a"]);
        primary.put_cf(primary.cf("a"), b"k", b"v").unwrap();

        let secondary_dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.set_max_open_files(-1);
        let secondary = DB::open_cf_descriptors_as_secondary(
            &opts,
            primary.path(),
            secondary_dir.path(),
            cf_descriptors(&["a"]),
        )
        .unwrap();
        let cf = secondary.cf_handle("a").unwrap();
        assert_eq!(secondary.get_cf(cf, b"k").unwrap(), Some(b"v".to_vec()));
        assert_eq!(
            secondary.put_cf(cf, b"k", b"v2").unwrap_err().kind(),
            ErrorKind::NotSupported
        );
    }

    #[test]
    fn column_family_from_another_db() {
        let db = TestDb::new(&["a"]);
        let other = TestDb::new(&["a"]);
        let cf = other.cf("a");

        fn kind<T>(result: Result<T, Error>) -> ErrorKind {
            result.err().unwrap().kind()
        }
        assert_eq!(kind(db.get_cf(cf, b"k")), ErrorKind::InvalidArgument);
        assert_eq!(kind(db.put_cf(cf, b"k", b"v")), ErrorKind::InvalidArgument);
        assert_eq!(kind(db.delete_cf(cf, b"k")), ErrorKind::InvalidArgument);
        assert_eq!(kind(db.flush_cf(cf)), ErrorKind::InvalidArgument);
        assert_eq!(
            kind(db.property_int_value_cf(cf, "rocksdb.estimate-num-keys")),
            ErrorKind::InvalidArgument
        );

        let mut iter = db.raw_iterator_cf_opt(cf, ReadOptions::default());
        iter.seek_to_first();
        assert!(!iter.valid());
        assert_eq!(
            iter.status().unwrap_err().kind(),
            ErrorKind::InvalidArgument
        );
    }

    #[test]
    fn write_batch() {
        let db = TestDb::new(&["a"]);
        db.put(b"stale", b"v").unwrap();

        let mut batch = WriteBatch::default();
        batch.put(b"k1", b"v1").unwrap();
        batch.put_cf(db.cf("a"), b"k2", b"v2").unwrap();
        batch.delete(b"stale").unwrap();
        batch.put_cf(db.cf("a"), b"k3", b"v3").unwrap();
        batch.delete_cf(db.cf("a"), b"k3").unwrap();
        let mut opts = WriteOptions::default();
        opts.set_sync(true);
        db.write_opt(batch, &opts).unwrap();

        assert_eq!(db.get(b"k1").unwrap(), Some(b"v1".to_vec()));
        assert_eq!(db.get(b"stale").unwrap(), None);
        assert_eq!(db.get_cf(db.cf("a"), b"k2").unwrap(), Some(b"v2".to_vec()));
        assert_eq!(db.get_cf(db.cf("a"), b"k3").unwrap(), None);

        db.write(WriteBatch::default()).unwrap();
    }

    #[test]
    fn write_batch_with_unknown_column_family() {
        let db = TestDb::new(&[]);
        let other = TestDb::new(&["a"]);
        let mut batch = WriteBatch::default();
        batch.put(b"k", b"v").unwrap();
        batch.put_cf(other.cf("a"), b"k", b"v").unwrap();

        assert_eq!(
            db.write(batch).unwrap_err().kind(),
            ErrorKind::InvalidArgument
        );
        // Batches are atomic.
        assert_eq!(db.get(b"k").unwrap(), None);
    }

    #[test]
    fn properties() {
        let db = TestDb::new(&["a"]);
        let cf = db.cf("a");
        let property = |name| db.property_int_value_cf(cf, name).unwrap();
        assert_eq!(property("rocksdb.total-sst-files-size"), Some(0));
        assert_eq!(property("rocksdb.unknown"), None);
        // Not integer properties.
        assert_eq!(property("rocksdb.stats"), None);
        assert_eq!(property("rocksdb.num-files-at-level0"), None);

        db.put_cf(cf, b"k", b"v").unwrap();
        assert_eq!(property("rocksdb.estimate-num-keys"), Some(1));
        db.flush_cf(cf).unwrap();
        assert!(property("rocksdb.total-sst-files-size").unwrap() > 0);
    }

    /// Opens a DB with options similar to a production setup, to make sure
    /// RocksDB accepts them.
    #[test]
    #[cfg_attr(not(feature = "lz4"), ignore = "requires the lz4 feature")]
    fn tuned_options() {
        let mut env = Env::new().unwrap();
        env.set_high_priority_background_threads(2);
        env.set_low_priority_background_threads(4);
        let capacity = 8 << 20;
        let cache = Cache::new_hyper_clock_cache(capacity, 0);

        let mut db_opts = create_options();
        db_opts.set_env(&env);
        db_opts.set_max_open_files(1000);
        db_opts.set_max_total_wal_size(1 << 30);
        db_opts.set_max_background_jobs(4);
        db_opts.set_stats_dump_period_sec(600);

        let cf_opts = |compression| {
            let mut table_opts = BlockBasedOptions::default();
            table_opts.set_block_size(4096);
            table_opts.set_index_type(BlockBasedIndexType::TwoLevelIndexSearch);
            table_opts.set_partition_filters(true);
            table_opts.set_cache_index_and_filter_blocks(true);
            table_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
            table_opts.set_block_cache(&cache);
            table_opts.set_hybrid_ribbon_filter(10.0, 2);
            table_opts.set_whole_key_filtering(false);

            let mut opts = Options::default();
            opts.set_compression_type(compression);
            opts.set_block_based_table_factory(&table_opts);
            opts.add_compact_on_deletion_collector_factory(0, 0, 0.4);
            opts.set_soft_pending_compaction_bytes_limit(0);
            opts.set_hard_pending_compaction_bytes_limit(0);
            opts.set_level_zero_slowdown_writes_trigger(1000);
            opts.set_level_zero_stop_writes_trigger(2000);
            opts
        };

        let dir = TempDir::new().unwrap();
        let db = DB::open_cf_descriptors(
            &db_opts,
            dir.path(),
            [
                ColumnFamilyDescriptor::new("lz4", cf_opts(DBCompressionType::Lz4)),
                ColumnFamilyDescriptor::new("lz4hc", cf_opts(DBCompressionType::Lz4hc)),
                ColumnFamilyDescriptor::new("none", cf_opts(DBCompressionType::None)),
            ],
        )
        .unwrap();

        for name in ["lz4", "lz4hc", "none"] {
            let cf = db.cf_handle(name).unwrap();
            for i in 0..1000u32 {
                db.put_cf(cf, i.to_be_bytes(), [b'x'; 100]).unwrap();
            }
            db.flush_cf(cf).unwrap();
            assert_eq!(
                db.get_cf(cf, 42u32.to_be_bytes()).unwrap(),
                Some(vec![b'x'; 100])
            );
            assert_eq!(
                db.property_int_value_cf(cf, "rocksdb.block-cache-capacity")
                    .unwrap(),
                Some(capacity as u64),
            );
        }
    }

    #[test]
    fn bloom_filter() {
        let mut table_opts = BlockBasedOptions::default();
        table_opts.set_bloom_filter(10.0);
        let mut opts = Options::default();
        opts.set_block_based_table_factory(&table_opts);
        let db = TestDb::with_cf_options([("a", opts)]);

        db.put_cf(db.cf("a"), b"k", b"v").unwrap();
        db.flush_cf(db.cf("a")).unwrap();
        assert_eq!(db.get_cf(db.cf("a"), b"k").unwrap(), Some(b"v".to_vec()));
        assert_eq!(db.get_cf(db.cf("a"), b"missing").unwrap(), None);
    }

    #[test]
    fn get_with_read_options() {
        let db = TestDb::new(&[]);
        db.put(b"k", b"v").unwrap();
        db.flush().unwrap();
        let mut opts = ReadOptions::default();
        opts.fill_cache(false);
        let cf = db.cf(DEFAULT_COLUMN_FAMILY_NAME);
        assert_eq!(db.get_cf_opt(cf, b"k", &opts).unwrap(), Some(b"v".to_vec()));
    }

    #[test]
    fn concurrent_access() {
        let db = TestDb::new(&["a"]);
        thread::scope(|s| {
            for t in 0..8u32 {
                let db = &db;
                s.spawn(move || {
                    let cf = db.cf_handle("a").unwrap();
                    for i in 0..100u32 {
                        let key = [t.to_be_bytes(), i.to_be_bytes()].concat();
                        db.put_cf(cf, &key, i.to_be_bytes()).unwrap();
                        assert_eq!(db.get_cf(cf, &key).unwrap(), Some(i.to_be_bytes().to_vec()));
                    }
                });
            }
        });
        assert_eq!(db.iterator(IteratorMode::Start).count(), 0);
        let mut iter = db.raw_iterator_cf_opt(db.cf("a"), ReadOptions::default());
        iter.seek_to_first();
        let mut count = 0;
        while iter.valid() {
            count += 1;
            iter.next();
        }
        assert_eq!(count, 800);
    }

    #[test]
    fn send_and_sync() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<DB>();
        send_sync::<&ColumnFamily>();
        send_sync::<Options>();
        send_sync::<ReadOptions>();
        send_sync::<WriteOptions>();
        send_sync::<WriteBatch>();
        send_sync::<crate::BlockBasedOptions>();
        send_sync::<crate::Cache>();
        send_sync::<crate::Env>();
        send_sync::<crate::SliceTransform>();
        fn send<T: Send>() {}
        send::<DBRawIterator<'_>>();
        send::<DBIterator<'_>>();
    }
}
