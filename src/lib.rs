//! Rust bindings for [RocksDB](https://rocksdb.org), built on
//! [cxx](https://cxx.rs).
//!
//! RocksDB is built from source and linked statically by the
//! `rocksdb-rs-sys` crate. This crate binds the C++ API directly through a
//! thin shim, instead of going through RocksDB's C API.
//!
//! ```
//! use rocksdb_rs::DB;
//!
//! let dir = tempfile::tempdir().unwrap();
//! let db = DB::open_default(dir.path()).unwrap();
//! db.put(b"key", b"value").unwrap();
//! assert_eq!(db.get(b"key").unwrap().as_deref(), Some(&b"value"[..]));
//! ```

#![deny(unsafe_code)]

// Unused crates are not linked, and the bridge needs RocksDB's symbols.
extern crate rocksdb_rs_sys;

mod cache;
pub mod checkpoint;
mod db;
mod env;
mod error;
#[allow(unsafe_code)]
mod ffi;
mod iterator;
mod options;
mod slice_transform;
pub mod statistics;
mod write_batch;

#[cfg(test)]
mod test_utils;

pub use crate::{
    cache::Cache,
    db::{ColumnFamily, ColumnFamilyDescriptor, DB, DEFAULT_COLUMN_FAMILY_NAME},
    env::Env,
    error::{Error, ErrorKind},
    iterator::{DBIterator, DBRawIterator, Direction, IteratorMode},
    options::{
        BlockBasedIndexType, BlockBasedOptions, DBCompressionType, Options, ReadOptions,
        WriteOptions,
    },
    slice_transform::SliceTransform,
    write_batch::WriteBatch,
};
