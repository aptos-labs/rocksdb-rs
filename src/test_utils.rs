use crate::{ColumnFamily, ColumnFamilyDescriptor, DB, Options};
use std::ops::Deref;
use tempfile::TempDir;

/// A DB in a temporary directory, which is removed after the DB is closed.
pub(crate) struct TestDb {
    db: DB,
    dir: TempDir,
}

impl TestDb {
    /// Opens a new DB with the given column families besides the default one.
    pub(crate) fn new(cfs: &[&str]) -> Self {
        Self::with_cf_options(cfs.iter().map(|name| (*name, Options::default())))
    }

    pub(crate) fn with_cf_options<'a>(cfs: impl IntoIterator<Item = (&'a str, Options)>) -> Self {
        let dir = TempDir::new().unwrap();
        let db = DB::open_cf_descriptors(
            &create_options(),
            dir.path(),
            cfs.into_iter()
                .map(|(name, opts)| ColumnFamilyDescriptor::new(name, opts)),
        )
        .unwrap();
        Self { db, dir }
    }

    pub(crate) fn cf(&self, name: &str) -> &ColumnFamily {
        self.db.cf_handle(name).unwrap()
    }

    /// Closes the DB, keeping its files.
    pub(crate) fn close(self) -> TempDir {
        self.dir
    }
}

impl Deref for TestDb {
    type Target = DB;

    fn deref(&self) -> &DB {
        &self.db
    }
}

pub(crate) fn create_options() -> Options {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts
}

pub(crate) fn cf_descriptors(names: &[&str]) -> Vec<ColumnFamilyDescriptor> {
    names
        .iter()
        .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
        .collect()
}
