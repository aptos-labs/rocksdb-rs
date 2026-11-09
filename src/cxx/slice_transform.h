#pragma once

#include <memory>

#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb/slice_transform.h"

namespace rocksdb_rs {

// Wraps a Rust prefix extractor. Rust guarantees it is `Send + Sync`, which
// RocksDB requires since it calls the transform from many threads.
std::shared_ptr<const rocksdb::SliceTransform> new_slice_transform(
    rust::Box<RustSliceTransform> transform);

}  // namespace rocksdb_rs
