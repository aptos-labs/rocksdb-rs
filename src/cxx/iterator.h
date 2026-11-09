#pragma once

#include <memory>

#include "rocksdb-rs/src/cxx/common.h"
#include "rocksdb-rs/src/cxx/options.h"
#include "rocksdb/iterator.h"

namespace rocksdb_rs {

// Unlike `rocksdb::Iterator`, all methods are safe to call on an invalid
// iterator: moving is a no-op and the key and value are empty.
class DbIterator {
 public:
  DbIterator(std::unique_ptr<ReadOptions> options,
             std::unique_ptr<rocksdb::Iterator> iter)
      : options_(std::move(options)), iter_(std::move(iter)) {}

  bool valid() const;
  void seek_to_first();
  void seek_to_last();
  void seek(Bytes key);
  void seek_for_prev(Bytes key);
  void next();
  void prev();
  Bytes key() const;
  Bytes value() const;
  void status(Status& status) const;

 private:
  // Declared first so that it outlives `iter_`, which may point to the
  // iterate bounds it owns.
  std::unique_ptr<ReadOptions> options_;
  std::unique_ptr<rocksdb::Iterator> iter_;
};

}  // namespace rocksdb_rs
