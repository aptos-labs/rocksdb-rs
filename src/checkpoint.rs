//! Checkpoints, i.e. openable snapshots of a DB.

use crate::{DB, Error};
use std::path::Path;

pub struct Checkpoint<'db> {
    db: &'db DB,
}

impl<'db> Checkpoint<'db> {
    pub fn new(db: &'db DB) -> Result<Self, Error> {
        Ok(Self { db })
    }

    /// Creates a checkpoint in `path`, which must not exist. SST files are
    /// hard-linked when `path` is on the same file system as the DB.
    pub fn create_checkpoint(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.db.create_checkpoint(path.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorKind, Options,
        test_utils::{TestDb, cf_descriptors},
    };
    use tempfile::TempDir;

    #[test]
    fn create_and_open() {
        let db = TestDb::new(&["a"]);
        db.put_cf(db.cf("a"), b"k", b"v1").unwrap();
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("checkpoint");
        let checkpoint = Checkpoint::new(&db).unwrap();
        checkpoint.create_checkpoint(&path).unwrap();
        db.put_cf(db.cf("a"), b"k", b"v2").unwrap();

        let err = checkpoint.create_checkpoint(&path).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidArgument);

        let copy =
            DB::open_cf_descriptors(&Options::default(), &path, cf_descriptors(&["a"])).unwrap();
        assert_eq!(
            copy.get_cf(copy.cf_handle("a").unwrap(), b"k").unwrap(),
            Some(b"v1".to_vec())
        );
    }
}
