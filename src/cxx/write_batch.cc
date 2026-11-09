#include "rocksdb-rs/src/cxx/write_batch.h"

#include "rocksdb-rs/src/ffi.rs.h"

namespace rocksdb_rs {

// Handles are only read during these calls, so they may come from any open
// DB. The DB validates column family IDs when the batch is written.

void WriteBatch::put(Bytes key, Bytes value, Status& status) {
  set_status(status, inner.Put(to_slice(key), to_slice(value)));
}

void WriteBatch::put_cf(const ColumnFamily& cf, Bytes key, Bytes value,
                        Status& status) {
  set_status(status, inner.Put(cf.handle(), to_slice(key), to_slice(value)));
}

void WriteBatch::del(Bytes key, Status& status) {
  set_status(status, inner.Delete(to_slice(key)));
}

void WriteBatch::del_cf(const ColumnFamily& cf, Bytes key, Status& status) {
  set_status(status, inner.Delete(cf.handle(), to_slice(key)));
}

void WriteBatch::clear() { inner.Clear(); }

uint32_t WriteBatch::count() const { return inner.Count(); }

size_t WriteBatch::data_size() const { return inner.GetDataSize(); }

std::unique_ptr<WriteBatch> new_write_batch() {
  return std::make_unique<WriteBatch>();
}

}  // namespace rocksdb_rs
