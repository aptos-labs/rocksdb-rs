#pragma once

#include <memory>
#include <vector>

#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb-rs/src/cxx/options.h"
#include "rocksdb/db.h"

namespace rocksdb_rs {

// A column family handle owned by a `Db`. Rust only sees it by reference,
// with a lifetime bounded by the `Db`.
class ColumnFamily {
 public:
  ColumnFamily(const rocksdb::DB* db, rocksdb::ColumnFamilyHandle* handle)
      : db_(db), handle_(handle) {}
  ColumnFamily(const ColumnFamily&) = delete;
  ColumnFamily& operator=(const ColumnFamily&) = delete;

  // The DB this column family belongs to. Handles must not be mixed across
  // DBs, so the `Db` methods check this.
  const rocksdb::DB* db() const { return db_; }
  rocksdb::ColumnFamilyHandle* handle() const { return handle_.get(); }

 private:
  const rocksdb::DB* db_;
  std::unique_ptr<rocksdb::ColumnFamilyHandle> handle_;
};

class ColumnFamilyDescriptors {
 public:
  void add(rust::Str name, const Options& options);

  std::vector<rocksdb::ColumnFamilyDescriptor> inner;
};

std::unique_ptr<ColumnFamilyDescriptors> new_column_family_descriptors();

}  // namespace rocksdb_rs
