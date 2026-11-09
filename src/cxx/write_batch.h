#pragma once

#include <cstdint>
#include <memory>

#include "rocksdb-rs/src/cxx/column_family.h"
#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb/write_batch.h"

namespace rocksdb_rs {

class WriteBatch {
 public:
  void put(Bytes key, Bytes value, Status& status);
  void put_cf(const ColumnFamily& cf, Bytes key, Bytes value, Status& status);
  void del(Bytes key, Status& status);
  void del_cf(const ColumnFamily& cf, Bytes key, Status& status);
  void clear();
  uint32_t count() const;
  size_t data_size() const;

  rocksdb::WriteBatch inner;
};

std::unique_ptr<WriteBatch> new_write_batch();

}  // namespace rocksdb_rs
