#pragma once

#include <cstdint>
#include <memory>
#include <vector>

#include "rocksdb-rs/src/cxx/column_family.h"
#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb-rs/src/cxx/iterator.h"
#include "rocksdb-rs/src/cxx/options.h"
#include "rocksdb-rs/src/cxx/write_batch.h"
#include "rocksdb/db.h"

namespace rocksdb_rs {

// `rocksdb::DB` is thread-safe, so all methods are const.
class Db {
 public:
  Db(std::unique_ptr<rocksdb::DB> db,
     std::vector<rocksdb::ColumnFamilyHandle*> handles,
     std::shared_ptr<rocksdb::Env> env);
  ~Db();
  Db(const Db&) = delete;
  Db& operator=(const Db&) = delete;

  size_t num_column_families() const;
  // Column families are in the order they were passed to `open_db`.
  const ColumnFamily& column_family(size_t index) const;

  // Returns false if the key is not found.
  bool get(const ReadOptions& options, const ColumnFamily& cf, Bytes key,
           rust::Vec<uint8_t>& value, Status& status) const;
  void put(const WriteOptions& options, const ColumnFamily& cf, Bytes key,
           Bytes value, Status& status) const;
  void del(const WriteOptions& options, const ColumnFamily& cf, Bytes key,
           Status& status) const;
  void write(const WriteOptions& options, WriteBatch& batch,
             Status& status) const;
  void flush(const ColumnFamily& cf, Status& status) const;
  // Returns false if the property is unknown or not an integer.
  bool get_int_property(const ColumnFamily& cf, rust::Str name, uint64_t& value,
                        Status& status) const;
  std::unique_ptr<DbIterator> new_iterator(std::unique_ptr<ReadOptions> options,
                                           const ColumnFamily& cf) const;
  void create_checkpoint(Bytes path, Status& status) const;

 private:
  bool check_owner(const ColumnFamily& cf, Status& status) const;

  std::unique_ptr<rocksdb::DB> db_;
  std::vector<std::unique_ptr<ColumnFamily>> column_families_;
  // Owns the DB's `Env`, which RocksDB holds as a raw pointer.
  std::shared_ptr<rocksdb::Env> env_;
};

std::unique_ptr<Db> open_db(const Options& options, Bytes path,
                            const ColumnFamilyDescriptors& column_families,
                            OpenMode mode, Bytes secondary_path,
                            bool error_if_wal_file_exists, Status& status);

rust::Vec<rust::String> list_column_families(const Options& options, Bytes path,
                                             Status& status);

}  // namespace rocksdb_rs
