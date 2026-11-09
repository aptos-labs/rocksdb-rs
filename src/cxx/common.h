// Helpers shared by the C++ side of the bridge.
//
// Every function exposed to Rust must be safe to call with any arguments that
// Rust can construct, so the shim validates inputs (e.g. shared enum values,
// iterator validity) instead of relying on RocksDB's debug assertions.

#pragma once

#include <cstdint>
#include <string>

#include "rocksdb/slice.h"
#include "rocksdb/status.h"
#include "rust/cxx.h"

namespace rocksdb_rs {

// Shared types defined by the bridge in `src/ffi.rs`.
struct Status;
enum class StatusCode : uint8_t;
enum class OpenMode : uint8_t;
enum class DBCompressionType : uint8_t;
enum class BlockBasedIndexType : uint8_t;
enum class StatsLevel : uint8_t;
enum class Ticker : uint32_t;
struct RustSliceTransform;

using Bytes = rust::Slice<const uint8_t>;

// Records `s` in `out` unless it is OK. `out` starts as OK on the Rust side.
void set_status(Status& out, const rocksdb::Status& s);

inline rocksdb::Slice to_slice(Bytes bytes) {
  return rocksdb::Slice(reinterpret_cast<const char*>(bytes.data()),
                        bytes.size());
}

inline std::string to_string(Bytes bytes) {
  return std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

inline Bytes to_bytes(const rocksdb::Slice& slice) {
  return Bytes(reinterpret_cast<const uint8_t*>(slice.data()), slice.size());
}

}  // namespace rocksdb_rs
