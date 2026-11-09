//! Builds RocksDB from the bundled sources and links it statically.
//!
//! This crate has no Rust API. Dependents get the RocksDB include directory
//! through the `DEP_ROCKSDB_INCLUDE` environment variable in their build
//! scripts.

#![no_std]

// Unused crates are not linked, so reference the compression libraries
// explicitly to keep their symbols available to RocksDB.
#[cfg(feature = "lz4")]
extern crate lz4_sys;
